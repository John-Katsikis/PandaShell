use crate::parser;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};

pub fn run(input: &str) {
    let args = parser::parse_line(input)
        .ok()
        .and_then(|ast| ast.commands.first().map(|cmd| cmd.args.clone()))
        .unwrap_or_default();
    let port = args
        .first()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8000);
    let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let addr = format!("127.0.0.1:{}", port);

    let listener = match TcpListener::bind(&addr) {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Failed to bind {}: {}", addr, e);
            return;
        }
    };

    println!("Serving {} at http://{}", root.display(), addr);
    println!("Press Ctrl+C to stop.");
    for stream in listener.incoming().flatten() {
        handle_client(stream, &root);
    }
}

fn handle_client(mut stream: TcpStream, root: &Path) {
    let mut buffer = [0u8; 1024];
    let Ok(size) = stream.read(&mut buffer) else {
        return;
    };
    let request = String::from_utf8_lossy(&buffer[..size]);
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");
    let safe = path.trim_start_matches('/').replace("..", "");
    let mut file_path = root.join(if safe.is_empty() { "index.html" } else { &safe });
    if file_path.is_dir() {
        file_path = file_path.join("index.html");
    }

    match fs::read(&file_path) {
        Ok(body) => {
            let header = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", body.len());
            let _ = stream.write_all(header.as_bytes());
            let _ = stream.write_all(&body);
        }
        Err(_) => {
            let body = b"404 not found";
            let header = format!(
                "HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(header.as_bytes());
            let _ = stream.write_all(body);
        }
    }
}
