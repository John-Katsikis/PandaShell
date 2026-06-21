use sha2::{Sha256, Digest};
use std::fs;

pub fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn sha256_file(path: &str) -> Result<String, String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(hex::encode(hasher.finalize()))
}

pub fn run(input: &str) {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();

    if parts.is_empty() || parts[0] != "hash" {
        eprintln!("Usage: hash <text> | hash --sha256 <text> | hash --file <path>");
        return;
    }

    if parts.len() < 2 {
        eprintln!("Usage: hash <text> | hash --sha256 <text> | hash --file <path>");
        return;
    }

    match parts[1] {
        "--sha256" => {
            if parts.len() < 3 {
                eprintln!("Usage: hash --sha256 <text>");
                return;
            }
            let text = parts[2..].join(" ");
            let digest = sha256_hex(&text);
            println!("\x1b[92mSHA-256:\x1b[0m {}", digest);
        }

        "--file" => {
            if parts.len() < 3 {
                eprintln!("Usage: hash --file <path>");
                return;
            }
            let path = parts[2];
            match sha256_file(path) {
                Ok(digest) => println!("\x1b[92mSHA-256 (file):\x1b[0m {}", digest),
                Err(err) => eprintln!("{}", err),
            }
        }

        // Default: hash the text directly
        _ => {
            let text = parts[1..].join(" ");
            let digest = sha256_hex(&text);
            println!("\x1b[92mSHA-256:\x1b[0m {}", digest);
        }
    }
}
