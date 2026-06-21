use std::process::Command;
use std::thread;
use std::time::Duration;

use crate::parser;

const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[38;5;82m";
const CYAN: &str = "\x1b[38;5;51m";
const YELLOW: &str = "\x1b[38;5;220m";
const RED: &str = "\x1b[91m";

#[derive(Debug, Clone, PartialEq, Eq)]
struct Options {
    app: String,
    force: bool,
    wait_ms: u64,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            app: String::new(),
            force: false,
            wait_ms: 900,
        }
    }
}

pub fn run(input: &str) {
    let args = parser::parse_line(input)
        .ok()
        .and_then(|ast| ast.commands.first().map(|cmd| cmd.args.clone()))
        .unwrap_or_default();

    match parse_args(&args) {
        Ok(options) => restart_app(&options),
        Err(e) if e == "help" => usage(),
        Err(e) => {
            eprintln!("{RED}{e}{RESET}");
            usage();
        }
    }
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut options = Options::default();
    let mut app_parts = Vec::new();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => return Err("help".into()),
            "--force" | "-f" => options.force = true,
            "--wait" | "-w" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("Missing value after --wait".into());
                };
                options.wait_ms = parse_wait_ms(value)?;
            }
            flag if flag.starts_with('-') => return Err(format!("Unknown restart flag '{flag}'")),
            value => app_parts.push(value.to_string()),
        }

        index += 1;
    }

    options.app = app_parts.join(" ");
    if options.app.trim().is_empty() {
        return Err("help".into());
    }

    Ok(options)
}

fn parse_wait_ms(value: &str) -> Result<u64, String> {
    let trimmed = value.trim();
    let seconds = trimmed
        .strip_suffix('s')
        .unwrap_or(trimmed)
        .parse::<f64>()
        .map_err(|_| "Wait must be a number of seconds, like 1.5 or 2s".to_string())?;

    if !(0.0..=30.0).contains(&seconds) {
        return Err("Wait must be between 0 and 30 seconds".into());
    }

    Ok((seconds * 1000.0).round() as u64)
}

fn restart_app(options: &Options) {
    if !cfg!(target_os = "macos") {
        eprintln!("{RED}restart is currently macOS-only.{RESET}");
        return;
    }

    let app = options.app.trim();
    println!("{YELLOW}Restarting{RESET} {CYAN}{app}{RESET}");

    if options.force {
        force_quit(app);
    } else {
        graceful_quit(app);
    }

    if options.wait_ms > 0 {
        thread::sleep(Duration::from_millis(options.wait_ms));
    }

    match Command::new("open").args(["-a", app]).status() {
        Ok(status) if status.success() => {
            println!("{GREEN}Relaunched{RESET} {CYAN}{app}{RESET}");
        }
        _ => {
            eprintln!("{RED}Could not relaunch '{app}'.{RESET}");
            eprintln!("{DIM}Try `app --list {app}` to check the exact app name.{RESET}");
        }
    }
}

fn graceful_quit(app: &str) {
    let script = format!("tell application \"{}\" to quit", escape_applescript(app));
    let status = Command::new("osascript").args(["-e", &script]).status();

    match status {
        Ok(status) if status.success() => {
            println!("{DIM}Asked {app} to quit gracefully.{RESET}");
        }
        _ => {
            eprintln!("{YELLOW}Graceful quit failed; trying force quit fallback.{RESET}");
            force_quit(app);
        }
    }
}

fn force_quit(app: &str) {
    match Command::new("killall").args(["-9", app]).status() {
        Ok(status) if status.success() => println!("{DIM}Force quit {app}.{RESET}"),
        _ => println!("{DIM}{app} was not running, or macOS uses a different process name.{RESET}"),
    }
}

fn escape_applescript(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}

fn usage() {
    eprintln!("{GREEN}Usage:{RESET} restart [options] <app>");
    eprintln!();
    eprintln!("{CYAN}Options{RESET}");
    eprintln!("  --force, -f       Force quit before reopening");
    eprintln!("  --wait N, -w N    Seconds to wait before reopening, default 0.9");
    eprintln!("  --help, -h        Show this help");
    eprintln!();
    eprintln!("{YELLOW}Examples{RESET}");
    eprintln!("{DIM}  restart Safari{RESET}");
    eprintln!("{DIM}  restart \"Visual Studio Code\"{RESET}");
    eprintln!("{DIM}  restart --force --wait 1.5 Slack{RESET}");
}

#[cfg(test)]
mod tests {
    use super::{escape_applescript, parse_args};

    #[test]
    fn parses_restart_app_name() {
        let args = vec!["Visual".into(), "Studio".into(), "Code".into()];
        let options = parse_args(&args).unwrap();

        assert_eq!(options.app, "Visual Studio Code");
        assert!(!options.force);
        assert_eq!(options.wait_ms, 900);
    }

    #[test]
    fn parses_force_and_wait() {
        let args = vec![
            "--force".into(),
            "--wait".into(),
            "1.5s".into(),
            "Safari".into(),
        ];
        let options = parse_args(&args).unwrap();

        assert_eq!(options.app, "Safari");
        assert!(options.force);
        assert_eq!(options.wait_ms, 1500);
    }

    #[test]
    fn escapes_applescript_name() {
        assert_eq!(
            escape_applescript("A \"Quoted\" App"),
            "A \\\"Quoted\\\" App"
        );
        assert_eq!(escape_applescript("Back\\Slash"), "Back\\\\Slash");
    }
}
