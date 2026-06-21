use std::env;
use std::path::Path;
use std::process::Command;

use crate::parser;

const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[38;5;82m";
const CYAN: &str = "\x1b[38;5;51m";
const YELLOW: &str = "\x1b[38;5;220m";
const RED: &str = "\x1b[91m";
const MAGENTA: &str = "\x1b[38;5;213m";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    All,
    Disk,
    Project,
    Tools,
}

pub fn run(input: &str) {
    let args = parser::parse_line(input)
        .ok()
        .and_then(|ast| ast.commands.first().map(|cmd| cmd.args.clone()))
        .unwrap_or_default();

    match parse_scope(&args) {
        Ok(scope) => run_scope(scope),
        Err(e) if e == "help" => usage(),
        Err(e) => {
            eprintln!("{RED}{e}{RESET}");
            usage();
        }
    }
}

fn parse_scope(args: &[String]) -> Result<Scope, String> {
    if args.is_empty() {
        return Ok(Scope::All);
    }

    if args.len() > 1 {
        return Err("doctor accepts at most one mode flag".into());
    }

    match args[0].as_str() {
        "--help" | "-h" => Err("help".into()),
        "--disk" => Ok(Scope::Disk),
        "--project" => Ok(Scope::Project),
        "--tools" => Ok(Scope::Tools),
        other => Err(format!("Unknown doctor mode '{other}'")),
    }
}

fn run_scope(scope: Scope) {
    header();

    match scope {
        Scope::All => {
            disk_report();
            project_report();
            tools_report();
            cleanup_report();
        }
        Scope::Disk => disk_report(),
        Scope::Project => project_report(),
        Scope::Tools => tools_report(),
    }

    println!();
}

fn header() {
    println!();
    println!("{GREEN}PANDA DOCTOR{RESET} {DIM}machine and project triage{RESET}");
    println!("{DIM}Current dir: {}{RESET}", cwd());
}

fn disk_report() {
    section("Disk");
    match command_output("df", &["-h", "/System/Volumes/Data"]) {
        Some(output) => {
            let mut lines = output.lines();
            let _ = lines.next();
            if let Some(line) = lines.next() {
                println!("{} {}", status_for_capacity(line), line.trim());
            } else {
                println!("{YELLOW}[WARN]{RESET} Could not parse disk usage.");
            }
        }
        None => println!("{YELLOW}[WARN]{RESET} Could not run df."),
    }

    for path in [
        "target",
        "node_modules",
        ".venv",
        "dist",
        "build",
        ".next",
        ".git",
    ] {
        if Path::new(path).exists() {
            print_size(path);
        }
    }
}

fn project_report() {
    section("Project");

    if Path::new("Cargo.toml").exists() {
        println!("{GREEN}[OK]{RESET} Rust project detected.");
    }
    if Path::new("package.json").exists() {
        println!("{GREEN}[OK]{RESET} Node project detected.");
    }
    if Path::new("pyproject.toml").exists() || Path::new("requirements.txt").exists() {
        println!("{GREEN}[OK]{RESET} Python project detected.");
    }

    match command_output("git", &["rev-parse", "--show-toplevel"]) {
        Some(root) => {
            println!("{GREEN}[OK]{RESET} Git repo: {}", root.trim());
            if let Some(status) = command_output("git", &["status", "--short"]) {
                let changed = status.lines().count();
                if changed == 0 {
                    println!("{GREEN}[OK]{RESET} Working tree clean.");
                } else {
                    println!("{YELLOW}[WARN]{RESET} {changed} changed/untracked git paths.");
                    for line in status.lines().take(8) {
                        println!("{DIM}  {line}{RESET}");
                    }
                }
            }
        }
        None => println!("{DIM}[INFO] Not inside a Git repo.{RESET}"),
    }
}

fn tools_report() {
    section("Tools");
    for tool in ["git", "cargo", "rustc", "python3", "node", "npm", "brew"] {
        match command_output("which", &[tool]) {
            Some(path) => println!("{GREEN}[OK]{RESET} {:<8} {}", tool, path.trim()),
            None => println!("{YELLOW}[MISS]{RESET} {tool}"),
        }
    }
}

fn cleanup_report() {
    section("Cleanup Hints");

    let mut found = false;
    for path in ["target", "node_modules", ".venv", "dist", "build", ".next"] {
        if Path::new(path).exists() {
            found = true;
            println!("{CYAN}{:<14}{RESET} {}", path, cleanup_hint(path));
        }
    }

    if !found {
        println!("{GREEN}[OK]{RESET} No common local build/cache folders in this directory.");
    }

    println!(
        "{DIM}Doctor only reports. It will not delete files or run cleanup commands for you.{RESET}"
    );
}

fn section(title: &str) {
    println!();
    println!("{MAGENTA}-- {title} --{RESET}");
}

fn status_for_capacity(df_line: &str) -> &'static str {
    let percent = df_line
        .split_whitespace()
        .find(|part| part.ends_with('%'))
        .and_then(|part| part.trim_end_matches('%').parse::<u8>().ok())
        .unwrap_or(0);

    if percent >= 95 {
        "\x1b[91m[CRITICAL]\x1b[0m"
    } else if percent >= 85 {
        "\x1b[38;5;220m[WARN]\x1b[0m"
    } else {
        "\x1b[38;5;82m[OK]\x1b[0m"
    }
}

fn print_size(path: &str) {
    match command_output("du", &["-sh", path]) {
        Some(size) => println!("{CYAN}{:<14}{RESET} {}", path, size.trim()),
        None => println!("{YELLOW}[WARN]{RESET} Could not size {path}."),
    }
}

fn cleanup_hint(path: &str) -> &'static str {
    match path {
        "target" => "Rust build output; `cargo clean` removes it.",
        "node_modules" => "Node dependencies; reinstall with npm/pnpm/yarn.",
        ".venv" => "Python virtualenv; recreate when needed.",
        "dist" | "build" | ".next" => "Generated app output; usually safe to rebuild.",
        _ => "Inspect before deleting.",
    }
}

fn command_output(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn cwd() -> String {
    env::current_dir()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "?".into())
}

fn usage() {
    eprintln!("{GREEN}Usage:{RESET} doctor [--disk|--project|--tools]");
    eprintln!();
    eprintln!("{CYAN}Examples{RESET}");
    eprintln!("{DIM}  doctor{RESET}");
    eprintln!("{DIM}  doctor --disk{RESET}");
    eprintln!("{DIM}  doctor --project{RESET}");
}

#[cfg(test)]
mod tests {
    use super::{cleanup_hint, parse_scope, status_for_capacity, Scope};

    #[test]
    fn parses_scope_flags() {
        assert_eq!(parse_scope(&[]).unwrap(), Scope::All);
        assert_eq!(parse_scope(&["--disk".into()]).unwrap(), Scope::Disk);
        assert_eq!(parse_scope(&["--project".into()]).unwrap(), Scope::Project);
    }

    #[test]
    fn classifies_disk_capacity() {
        assert!(status_for_capacity("disk 100G 99G 1G 99% /").contains("CRITICAL"));
        assert!(status_for_capacity("disk 100G 86G 14G 86% /").contains("WARN"));
        assert!(status_for_capacity("disk 100G 40G 60G 40% /").contains("OK"));
    }

    #[test]
    fn gives_cleanup_hints() {
        assert!(cleanup_hint("target").contains("cargo clean"));
        assert!(cleanup_hint("node_modules").contains("Node"));
    }
}
