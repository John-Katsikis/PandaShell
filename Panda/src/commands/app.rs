use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::parser;

const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[38;5;82m";
const CYAN: &str = "\x1b[38;5;51m";
const YELLOW: &str = "\x1b[38;5;220m";
const RED: &str = "\x1b[91m";

#[derive(Debug, Clone, PartialEq, Eq)]
enum Mode {
    Open(String),
    List(Option<String>),
}

pub fn run(input: &str) {
    let args = parser::parse_line(input)
        .ok()
        .and_then(|ast| ast.commands.first().map(|cmd| cmd.args.clone()))
        .unwrap_or_default();

    match parse_args(&args) {
        Ok(Mode::Open(name)) => open_app(&name),
        Ok(Mode::List(filter)) => list_apps(filter.as_deref()),
        Err(e) if e == "help" => usage(),
        Err(e) => {
            eprintln!("{RED}{e}{RESET}");
            usage();
        }
    }
}

fn parse_args(args: &[String]) -> Result<Mode, String> {
    if args.is_empty() {
        return Err("help".into());
    }

    match args[0].as_str() {
        "--help" | "-h" => Err("help".into()),
        "--list" | "-l" => {
            let filter = if args.len() > 1 {
                Some(args[1..].join(" "))
            } else {
                None
            };
            Ok(Mode::List(filter))
        }
        flag if flag.starts_with('-') => Err(format!("Unknown app flag '{flag}'")),
        _ => Ok(Mode::Open(args.join(" "))),
    }
}

fn open_app(name: &str) {
    if !cfg!(target_os = "macos") {
        eprintln!("{RED}The app command is currently macOS-only.{RESET}");
        return;
    }

    let trimmed = name.trim();
    if trimmed.is_empty() {
        usage();
        return;
    }

    match Command::new("open").args(["-a", trimmed]).status() {
        Ok(status) if status.success() => {
            println!("{GREEN}Opened{RESET} {CYAN}{trimmed}{RESET}");
        }
        _ => suggest_app(trimmed),
    }
}

fn suggest_app(name: &str) {
    let matches = find_apps(Some(name));

    match matches.len() {
        0 => {
            eprintln!("{RED}Could not open '{name}'.{RESET}");
            eprintln!("{DIM}Try `app --list {name}` to search installed apps.{RESET}");
        }
        1 => {
            let app = &matches[0].name;
            println!("{YELLOW}Trying closest match:{RESET} {CYAN}{app}{RESET}");
            match Command::new("open").args(["-a", app]).status() {
                Ok(status) if status.success() => {
                    println!("{GREEN}Opened{RESET} {CYAN}{app}{RESET}")
                }
                _ => eprintln!("{RED}Could not open closest match '{app}'.{RESET}"),
            }
        }
        _ => {
            eprintln!("{RED}Could not open '{name}' exactly.{RESET}");
            eprintln!("{YELLOW}Possible matches:{RESET}");
            for app in matches.iter().take(8) {
                eprintln!(
                    "  {CYAN}{}{RESET} {DIM}{}{RESET}",
                    app.name,
                    app.path.display()
                );
            }
        }
    }
}

fn list_apps(filter: Option<&str>) {
    let apps = find_apps(filter);

    if apps.is_empty() {
        match filter {
            Some(term) => println!("{DIM}No apps matched '{term}'.{RESET}"),
            None => println!("{DIM}No apps found in the usual macOS app folders.{RESET}"),
        }
        return;
    }

    println!("{GREEN}Mac Apps{RESET} {DIM}{} found{RESET}", apps.len());
    for app in apps {
        println!(
            "{CYAN}{:<34}{RESET} {DIM}{}{RESET}",
            app.name,
            app.path.display()
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AppEntry {
    name: String,
    path: PathBuf,
}

fn find_apps(filter: Option<&str>) -> Vec<AppEntry> {
    let needle = filter.map(|value| value.to_lowercase());
    let mut apps = Vec::new();

    for dir in app_dirs() {
        collect_apps(&dir, needle.as_deref(), &mut apps);
    }

    apps.sort_by_key(|app| app.name.to_lowercase());
    apps.dedup_by(|a, b| a.name.eq_ignore_ascii_case(&b.name));
    apps
}

fn app_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![PathBuf::from("/Applications")];

    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join("Applications"));
    }

    dirs
}

fn collect_apps(dir: &Path, needle: Option<&str>, apps: &mut Vec<AppEntry>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = app_name_from_path(&path) else {
            continue;
        };

        if needle.is_some_and(|term| !name.to_lowercase().contains(term)) {
            continue;
        }

        apps.push(AppEntry { name, path });
    }
}

fn app_name_from_path(path: &Path) -> Option<String> {
    if path.extension().and_then(|ext| ext.to_str()) != Some("app") {
        return None;
    }

    path.file_stem()
        .and_then(|name| name.to_str())
        .map(|name| name.to_string())
}

fn usage() {
    eprintln!("{GREEN}Usage:{RESET} app <name> | app --list [search]");
    eprintln!();
    eprintln!("{CYAN}Examples{RESET}");
    eprintln!("{DIM}  app Safari{RESET}");
    eprintln!("{DIM}  app \"Visual Studio Code\"{RESET}");
    eprintln!("{DIM}  app --list code{RESET}");
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{app_name_from_path, parse_args, Mode};

    #[test]
    fn parses_open_name() {
        let args = vec![
            "Visual".to_string(),
            "Studio".to_string(),
            "Code".to_string(),
        ];

        assert_eq!(
            parse_args(&args).unwrap(),
            Mode::Open("Visual Studio Code".to_string())
        );
    }

    #[test]
    fn parses_list_filter() {
        let args = vec!["--list".to_string(), "code".to_string()];

        assert_eq!(
            parse_args(&args).unwrap(),
            Mode::List(Some("code".to_string()))
        );
    }

    #[test]
    fn extracts_app_name() {
        assert_eq!(
            app_name_from_path(Path::new("/Applications/Safari.app")),
            Some("Safari".to_string())
        );
        assert_eq!(app_name_from_path(Path::new("/Applications/Notes")), None);
    }
}
