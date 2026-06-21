use crate::parser;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub fn run(input: &str) {
    let args = parser::parse_line(input)
        .ok()
        .and_then(|ast| ast.commands.first().map(|cmd| cmd.args.clone()))
        .unwrap_or_default();

    match args.first().map(String::as_str) {
        Some("add") => add(&args[1..].join(" ")),
        Some("list") | None => list(),
        Some("done") => {
            let Some(index) = args.get(1).and_then(|n| n.parse::<usize>().ok()) else {
                eprintln!("Usage: todo done <number>");
                return;
            };
            done(index);
        }
        Some("clear") => {
            let _ = fs::write(path(), "");
            println!("Todo list cleared.");
        }
        _ => usage(),
    }
}

fn add(text: &str) {
    if text.trim().is_empty() {
        eprintln!("Usage: todo add <task>");
        return;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path())
        .expect("todo file");
    let _ = writeln!(file, "{}", text.trim());
    println!("Added: {}", text.trim());
}

fn list() {
    let text = fs::read_to_string(path()).unwrap_or_default();
    if text.trim().is_empty() {
        println!("No todos.");
        return;
    }
    for (i, line) in text.lines().enumerate() {
        println!("{:>2}. {}", i + 1, line);
    }
}

fn done(index: usize) {
    let text = fs::read_to_string(path()).unwrap_or_default();
    let mut lines: Vec<&str> = text.lines().collect();
    if index == 0 || index > lines.len() {
        eprintln!("No todo at index {}", index);
        return;
    }
    let removed = lines.remove(index - 1);
    let _ = fs::write(path(), lines.join("\n") + "\n");
    println!("Done: {}", removed);
}

fn path() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".panda_todo")
}

fn usage() {
    eprintln!("Usage: todo add <task> | todo list | todo done <n> | todo clear");
}
