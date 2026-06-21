use std::process::Command;

const RESET: &str = "\x1b[0m";
const GREEN: &str = "\x1b[38;5;82m";
const CYAN: &str = "\x1b[38;5;51m";
const DIM: &str = "\x1b[2m";

pub fn run() {
    if git(&["rev-parse", "--is-inside-work-tree"]).as_deref() != Some("true") {
        eprintln!("Not inside a git repository");
        return;
    }

    println!();
    println!("{GREEN}╭──────────────────── GIT INFO ─────────────────────╮{RESET}");
    row(
        "Branch",
        git(&["branch", "--show-current"]).unwrap_or_else(|| "detached".into()),
    );
    row(
        "Root",
        git(&["rev-parse", "--show-toplevel"]).unwrap_or_else(|| "?".into()),
    );
    row(
        "HEAD",
        git(&["log", "-1", "--pretty=%h %s"]).unwrap_or_else(|| "?".into()),
    );
    row("Status", status_summary());
    println!("{GREEN}╰────────────────────────────────────────────────────╯{RESET}");
    println!();
}

fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!text.is_empty()).then_some(text)
}

fn status_summary() -> String {
    let status = git(&["status", "--short"]).unwrap_or_default();
    if status.is_empty() {
        return "clean".into();
    }

    let staged = status
        .lines()
        .filter(|line| {
            line.as_bytes()
                .first()
                .is_some_and(|c| *c != b' ' && *c != b'?')
        })
        .count();
    let unstaged = status
        .lines()
        .filter(|line| line.as_bytes().get(1).is_some_and(|c| *c != b' '))
        .count();
    let untracked = status.lines().filter(|line| line.starts_with("??")).count();
    format!(
        "{} staged, {} unstaged, {} untracked",
        staged, unstaged, untracked
    )
}

fn row(label: &str, value: String) {
    println!(
        "{GREEN}│{RESET} {CYAN}{:<10}{RESET} {DIM}->{RESET} {}",
        label, value
    );
}
