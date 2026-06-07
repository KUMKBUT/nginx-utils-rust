use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let config_path = match find_nginx_config() {
        Some(path) => path,
        None => {
            eprintln!("❌ Error: Failed to find nginx.conf in any of the standard directories.");
            println!("You can specify the path manually: sudo ./nginx-fix /path/to/nginx.conf");
            return;
        }
    };

    println!("📖 Using configuration file: {:?}", config_path);

    let port = match extract_port_from_nginx(&config_path) {
        Some(p) => p,
        None => {
            eprintln!("❌ Failed to find or parse the 'listen' directive in file {:?}", config_path);
            return;
        }
    };

    println!("🔍 Nginx should be listening on port: [:{}]", port);

    match listeners::get_all() {
        Ok(all_listeners) => {
            let is_nginx_already_running = all_listeners
                .iter()
                .any(|l| l.socket.port() == port && l.process.name == "nginx");

            if is_nginx_already_running {
                println!("✅ Everything is fine! Port {} is already occupied by Nginx itself. The server is operating normally.", port);
                return;
            }

            let conflicts: Vec<_> = all_listeners
                .into_iter()
                .filter(|l| l.socket.port() == port && l.process.name != "nginx")
                .collect();

            if conflicts.is_empty() {
                println!("✅ Port {} is completely free and ready for Nginx to start.", port);
                return;
            }

            // 4. Interactive liquidation
            println!("\n⚠️ CONFLICT! Port {} is occupied by other processes:", port);
            for l in &conflicts {
                println!("  -> PID: {:<6} Process: {}", l.process.pid, l.process.name);
            }

            print!("\nDo you want to terminate (kill -9) these conflicting processes? [y/N]: ");
            io::stdout().flush().unwrap();

            let mut answer = String::new();
            io::stdin().read_line(&mut answer).unwrap();
            let answer = answer.trim().to_lowercase();

            if answer == "y" || answer == "yes" {
                for l in conflicts {
                    println!("💥 Killing {} (PID: {})...", l.process.name, l.process.pid);
                    let status = Command::new("kill")
                        .arg("-9")
                        .arg(l.process.pid.to_string())
                        .status();

                    match status {
                        Ok(s) if s.success() => println!("💀 Successfully terminated."),
                        _ => eprintln!("❌ Failed to kill the process. Sudo privileges might be required!"),
                    }
                }
            } else {
                println!("Operation cancelled.");
            }
        }
        Err(err) => eprintln!("❌ Port scanning error: {}", err),
    }
}

fn find_nginx_config() -> Option<PathBuf> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let custom_path = PathBuf::from(&args[1]);
        if custom_path.exists() && custom_path.is_file() {
            return Some(custom_path);
        } else {
            println!("⚠️ The file specified in arguments {:?} does not exist. Searching automatically...", custom_path);
        }
    }

    let standard_paths = [
        "nginx.conf",                       // Current build/run folder
        "/etc/nginx/nginx.conf",             // Standard on Ubuntu, Debian, Arch, CentOS
        "/usr/local/nginx/conf/nginx.conf",  // Standard when built from source
        "/usr/local/etc/nginx/nginx.conf",  // Standard on FreeBSD and older macOS
        "/opt/homebrew/etc/nginx/nginx.conf", // Standard for macOS Homebrew (M1/M2/M3)
    ];

    for path in &standard_paths {
        let p = PathBuf::from(path);
        if p.exists() && p.is_file() {
            return Some(p);
        }
    }

    None
}

fn extract_port_from_nginx(file_path: &Path) -> Option<u16> {
    let file = File::open(file_path).ok()?;
    let reader = BufReader::new(file);

    for line in reader.lines().flatten() {
        let trimmed = line.trim();
        if trimmed.starts_with("listen") && trimmed.ends_with(';') {
            let clean_line = trimmed.trim_end_matches(';');
            let parts: Vec<&str> = clean_line.split_whitespace().collect();

            if parts.len() >= 2 {
                let port_part = parts[1];
                if let Some(idx) = port_part.rfind(':') {
                    if let Ok(p) = port_part[idx + 1..].parse::<u16>() {
                        return Some(p);
                    }
                } else if let Ok(p) = port_part.parse::<u16>() {
                    return Some(p);
                }
            }
        }
    }
    None
}
