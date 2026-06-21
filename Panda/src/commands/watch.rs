use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::parser;

const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[38;5;82m";
const CYAN: &str = "\x1b[38;5;51m";
const YELLOW: &str = "\x1b[38;5;220m";
const RED: &str = "\x1b[91m";
const MAGENTA: &str = "\x1b[38;5;213m";

#[derive(Debug, Clone, PartialEq)]
struct Options {
    interval_secs: f64,
    count: Option<usize>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            interval_secs: 2.0,
            count: Some(5),
        }
    }
}

pub fn run(input: &str) {
    let args = parser::parse_line(input)
        .ok()
        .and_then(|ast| ast.commands.first().map(|cmd| cmd.args.clone()))
        .unwrap_or_default();

    match parse_args(&args) {
        Ok(options) => run_watch(&options),
        Err(e) if e == "help" => usage(),
        Err(e) => {
            eprintln!("{RED}{e}{RESET}");
            usage();
        }
    }
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut options = Options::default();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => return Err("help".into()),
            "--live" | "-l" => options.count = None,
            "--once" => options.count = Some(1),
            "--count" | "-c" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("Missing value after --count".into());
                };
                let count = value
                    .parse::<usize>()
                    .map_err(|_| "Count must be a whole number".to_string())?;
                if count == 0 {
                    return Err("Count must be at least 1".into());
                }
                options.count = Some(count);
            }
            "--interval" | "-i" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("Missing value after --interval".into());
                };
                options.interval_secs = parse_seconds(value)?;
            }
            flag if flag.starts_with('-') => return Err(format!("Unknown watch flag '{flag}'")),
            value => return Err(format!("Unexpected watch argument '{value}'")),
        }

        index += 1;
    }

    Ok(options)
}

fn parse_seconds(value: &str) -> Result<f64, String> {
    let trimmed = value.trim().strip_suffix('s').unwrap_or(value.trim());
    let seconds = trimmed
        .parse::<f64>()
        .map_err(|_| "Interval must be a number of seconds".to_string())?;

    if !(0.2..=60.0).contains(&seconds) {
        return Err("Interval must be between 0.2 and 60 seconds".into());
    }

    Ok(seconds)
}

fn run_watch(options: &Options) {
    let mut tick = 0usize;

    loop {
        print!("\x1b[2J\x1b[H");
        render_dashboard(tick + 1, options);
        tick += 1;

        if options.count.is_some_and(|count| tick >= count) {
            break;
        }

        thread::sleep(Duration::from_secs_f64(options.interval_secs));
    }
}

fn render_dashboard(tick: usize, options: &Options) {
    println!(
        "{GREEN}PANDA WATCH{RESET} {DIM}snapshot #{tick} - {}{RESET}",
        timestamp()
    );
    println!(
        "{DIM}Interval: {:.1}s | Mode: {}{RESET}",
        options.interval_secs,
        options
            .count
            .map(|count| format!("{count} snapshots"))
            .unwrap_or_else(|| "live until Ctrl+C".into())
    );

    disk_line();
    memory_line();
    load_line();
    git_line();
    process_line();
}

fn disk_line() {
    println!();
    println!("{MAGENTA}-- Disk --{RESET}");
    match command_output("df", &["-h", "."]) {
        Some(output) => {
            let mut lines = output.lines();
            let _ = lines.next();
            if let Some(line) = lines.next() {
                println!("{CYAN}{}{RESET}", line.trim());
            }
        }
        None => println!("{YELLOW}Disk usage unavailable.{RESET}"),
    }
}

fn memory_line() {
    println!();
    println!("{MAGENTA}-- Memory --{RESET}");
    if cfg!(target_os = "macos") {
        let page_size = command_output("pagesize", &[])
            .and_then(|value| value.trim().parse::<u64>().ok())
            .unwrap_or(4096);
        match command_output("vm_stat", &[]) {
            Some(output) => {
                let free_pages = vm_stat_value(&output, "Pages free").unwrap_or(0);
                let inactive_pages = vm_stat_value(&output, "Pages inactive").unwrap_or(0);
                let free = (free_pages + inactive_pages) * page_size;
                println!("{CYAN}Available-ish:{RESET} {}", format_bytes(free));
            }
            None => println!("{YELLOW}Memory unavailable.{RESET}"),
        }
    } else {
        println!("{DIM}Memory summary is macOS-focused for now.{RESET}");
    }
}

fn load_line() {
    println!();
    println!("{MAGENTA}-- Load --{RESET}");
    match command_output("uptime", &[]) {
        Some(output) => println!("{CYAN}{}{RESET}", output.trim()),
        None => println!("{YELLOW}Load unavailable.{RESET}"),
    }
}

fn git_line() {
    println!();
    println!("{MAGENTA}-- Git --{RESET}");
    match command_output("git", &["status", "--short"]) {
        Some(status) if status.is_empty() => println!("{GREEN}Clean working tree.{RESET}"),
        Some(status) => {
            println!(
                "{YELLOW}{} changed/untracked paths.{RESET}",
                status.lines().count()
            );
            for line in status.lines().take(5) {
                println!("{DIM}  {line}{RESET}");
            }
        }
        None => println!("{DIM}Not in a Git repo.{RESET}"),
    }
}

fn process_line() {
    println!();
    println!("{MAGENTA}-- Busy Processes --{RESET}");
    match command_output("ps", &["-arcwwwxo", "pid,%cpu,%mem,comm"]) {
        Some(output) => {
            for line in output.lines().take(6) {
                println!("{CYAN}{line}{RESET}");
            }
        }
        None => println!("{YELLOW}Process list unavailable.{RESET}"),
    }
}

fn vm_stat_value(output: &str, label: &str) -> Option<u64> {
    output.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        if name.trim() != label {
            return None;
        }
        value
            .trim()
            .trim_end_matches('.')
            .replace([',', '.'], "")
            .parse::<u64>()
            .ok()
    })
}

fn command_output(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format!("{seconds}s since epoch")
}

fn format_bytes(bytes: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit + 1 < units.len() {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", units[unit])
}

fn usage() {
    eprintln!("{GREEN}Usage:{RESET} watch [--once|--live] [--count N] [--interval N]");
    eprintln!();
    eprintln!("{CYAN}Examples{RESET}");
    eprintln!("{DIM}  watch{RESET}");
    eprintln!("{DIM}  watch --once{RESET}");
    eprintln!("{DIM}  watch --count 20 --interval 1{RESET}");
    eprintln!("{DIM}  watch --live --interval 2{RESET}");
}

#[cfg(test)]
mod tests {
    use super::{parse_args, parse_seconds, vm_stat_value};

    #[test]
    fn parses_watch_defaults_and_flags() {
        let defaults = parse_args(&[]).unwrap();
        assert_eq!(defaults.count, Some(5));
        assert_eq!(defaults.interval_secs, 2.0);

        let live = parse_args(&["--live".into(), "--interval".into(), "1.5s".into()]).unwrap();
        assert_eq!(live.count, None);
        assert_eq!(live.interval_secs, 1.5);
    }

    #[test]
    fn rejects_bad_intervals() {
        assert!(parse_seconds("0.1").is_err());
        assert!(parse_seconds("61").is_err());
    }

    #[test]
    fn parses_vm_stat_values() {
        let sample = "Pages free:                               12,345.\nPages inactive: 8.";
        assert_eq!(vm_stat_value(sample, "Pages free"), Some(12345));
        assert_eq!(vm_stat_value(sample, "Pages inactive"), Some(8));
    }
}
