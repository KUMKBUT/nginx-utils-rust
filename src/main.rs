use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let config_path = match find_nginx_config() {
        Some(path) => path,
        None => {
            eprintln!("❌ Ошибка: Не удалось найти nginx.conf ни в одной из стандартных директорий.");
            println!("Вы можете указать путь вручную: sudo ./nginx-fix /путь/к/nginx.conf");
            return;
        }
    };

    println!("📖 Используем конфигурационный файл: {:?}", config_path);

    let port = match extract_port_from_nginx(&config_path) {
        Some(p) => p,
        None => {
            eprintln!("❌ Не удалось найти или распарсить директиву 'listen' в файле {:?}", config_path);
            return;
        }
    };

    println!("🔍 Nginx должен слушать порт: [:{}]", port);

    match listeners::get_all() {
        Ok(all_listeners) => {
            let is_nginx_already_running = all_listeners
                .iter()
                .any(|l| l.socket.port() == port && l.process.name == "nginx");

            if is_nginx_already_running {
                println!("✅ Всё отлично! Порт {} уже занят самим Nginx. Сервер работает штатно.", port);
                return;
            }

            let conflicts: Vec<_> = all_listeners
                .into_iter()
                .filter(|l| l.socket.port() == port && l.process.name != "nginx")
                .collect();

            if conflicts.is_empty() {
                println!("✅ Порт {} абсолютно свободен и готов к запуску Nginx.", port);
                return;
            }

            // 4. Интерактивная ликвидация
            println!("\n⚠️ КОНФЛИКТ! Порт {} занят сторонними процессами:", port);
            for l in &conflicts {
                println!("  -> PID: {:<6} Процесс: {}", l.process.pid, l.process.name);
            }

            print!("\nЖелаете завершить (kill -9) эти конфликтующие процессы? [y/N]: ");
            io::stdout().flush().unwrap();

            let mut answer = String::new();
            io::stdin().read_line(&mut answer).unwrap();
            let answer = answer.trim().to_lowercase();

            if answer == "y" || answer == "yes" {
                for l in conflicts {
                    println!("💥 Уничтожаем {} (PID: {})...", l.process.name, l.process.pid);
                    let status = Command::new("kill")
                        .arg("-9")
                        .arg(l.process.pid.to_string())
                        .status();

                    match status {
                        Ok(s) if s.success() => println!("💀 Успешно ликвидирован."),
                        _ => eprintln!("❌ Не удалось убить процесс. Нужен sudo!"),
                    }
                }
            } else {
                println!("Операция отменена.");
            }
        }
        Err(err) => eprintln!("❌ Ошибка сканирования портов: {}", err),
    }
}

fn find_nginx_config() -> Option<PathBuf> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let custom_path = PathBuf::from(&args[1]);
        if custom_path.exists() && custom_path.is_file() {
            return Some(custom_path);
        } else {
            println!("⚠️ Указанный в аргументах файл {:?} не существует. Ищем автоматически...", custom_path);
        }
    }

    let standard_paths = [
        "nginx.conf",// Текущая папка сборки/запуска
        "/etc/nginx/nginx.conf",// Стандарт в Ubuntu, Debian, Arch, CentOS
        "/usr/local/nginx/conf/nginx.conf",// Стандарт при сборке из исходников
        "/usr/local/etc/nginx/nginx.conf",// Стандарт на FreeBSD и старых macOS
        "/opt/homebrew/etc/nginx/nginx.conf",// Стандарт для macOS Homebrew (M1/M2/M3)
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
