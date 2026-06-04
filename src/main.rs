use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

fn main() -> Result<(), std::io::Error> {
    let file = File::open("nginx.log")?;
    let reader = BufReader::new(file);

    let mut methods_map: HashMap<String, Vec<String>> = HashMap::new();

    let target_methods = ["GET", "POST", "PUT", "DELETE", "PATCH"];

    for line in reader.lines() {
        let line = line?;

        let parse: Vec<_> = line.split_whitespace().collect();

        if parse.len() < 7 {
            continue;
        }

        let ip = parse[0].to_string();

        let method = parse[5].trim_matches('\"');

        if target_methods.contains(&method) {
            methods_map
                .entry(method.to_string())
                .or_insert_with(Vec::new)
                .push(ip);
        }
    }
    for (method, ips) in &methods_map {
        println!("Метод: {}, Количество запросов: {}", method, ips.len());
        println!("IP-адреса: {:?}", ips);
        println!("-----------------------------------");
    }

    Ok(())
}
