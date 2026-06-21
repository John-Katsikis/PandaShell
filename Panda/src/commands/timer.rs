use crate::parser;
use std::thread;
use std::time::{Duration, Instant};

pub fn run(input: &str) {
    let args = parser::parse_line(input)
        .ok()
        .and_then(|ast| ast.commands.first().map(|cmd| cmd.args.clone()))
        .unwrap_or_default();

    if args.first().is_some_and(|arg| arg == "stopwatch") {
        let start = Instant::now();
        println!("Stopwatch started. Press Enter to stop.");
        let mut line = String::new();
        let _ = std::io::stdin().read_line(&mut line);
        println!("Elapsed: {:.2}s", start.elapsed().as_secs_f64());
        return;
    }

    let Some(raw) = args.first() else {
        eprintln!("Usage: timer <10s|5m|1h> | timer stopwatch");
        return;
    };

    match parse_duration(raw) {
        Some(duration) => {
            println!("Timer started for {}s", duration.as_secs());
            thread::sleep(duration);
            println!("\x1b[38;5;82mDone.\x1b[0m");
        }
        None => eprintln!("Invalid duration. Try 10s, 5m, or 1h."),
    }
}

fn parse_duration(input: &str) -> Option<Duration> {
    let (num, unit) = input.split_at(input.len().saturating_sub(1));
    let value = num.parse::<u64>().ok()?;
    match unit {
        "s" => Some(Duration::from_secs(value)),
        "m" => Some(Duration::from_secs(value * 60)),
        "h" => Some(Duration::from_secs(value * 3600)),
        _ => input.parse::<u64>().ok().map(Duration::from_secs),
    }
}
