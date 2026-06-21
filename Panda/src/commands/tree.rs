use crate::parser;
use std::fs;
use std::path::{Path, PathBuf};

const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[38;5;82m";
const CYAN: &str = "\x1b[38;5;51m";
const BLUE: &str = "\x1b[38;5;75m";
const MAGENTA: &str = "\x1b[38;5;213m";
const YELLOW: &str = "\x1b[38;5;220m";
const RED: &str = "\x1b[38;5;203m";

const FINDER_DEPTH: usize = 1;
const ALL_DEPTH: usize = 4;
const MAX_ENTRIES: usize = 2000;

#[derive(Debug)]
struct TreeOptions {
    root: PathBuf,
    max_depth: usize,
    show_hidden: bool,
    dirs_only: bool,
    all_mode: bool,
}

#[derive(Debug, Default)]
struct TreeStats {
    dirs: usize,
    files: usize,
    skipped: usize,
}

#[derive(Debug)]
struct TreeEntry {
    path: PathBuf,
    name: String,
    is_dir: bool,
    is_symlink: bool,
}

pub fn run(input: &str) {
    let options = match parse_options(input) {
        Ok(options) => options,
        Err(e) => {
            eprintln!("\x1b[91m{}\x1b[0m", e);
            print_usage();
            return;
        }
    };

    let root = options.root.clone();
    if !root.exists() {
        eprintln!("Path does not exist: {}", root.display());
        return;
    }

    let mut stats = TreeStats::default();
    println!();
    let mode = if options.all_mode { "all" } else { "finder" };
    println!("{GREEN}╭──────────────────── PANDA TREE ────────────────────╮{RESET}");
    println!(
        "{GREEN}│{RESET} {CYAN}{:<51}{RESET}{GREEN}│{RESET}",
        root.display()
    );
    println!(
        "{GREEN}│{RESET} {DIM}mode: {:<45}{RESET}{GREEN}│{RESET}",
        mode
    );
    println!("{GREEN}╰─────────────────────────────────────────────────────╯{RESET}");
    println!("{MAGENTA}{}{RESET}", display_name(&root, true, false));

    walk_tree(&root, "", 0, &options, &mut stats);

    println!(
        "{DIM}{} dirs, {} files, {} skipped{RESET}",
        stats.dirs, stats.files, stats.skipped
    );
    println!();
}

fn parse_options(input: &str) -> Result<TreeOptions, String> {
    let ast = parser::parse_line(input)?;
    let Some(command) = ast.commands.first() else {
        return Err("Missing tree command".into());
    };

    if command.name != "tree" {
        return Err("Expected tree command".into());
    }

    let mut root = PathBuf::from(".");
    let mut max_depth = None;
    let mut show_hidden = false;
    let mut dirs_only = false;
    let mut all_mode = false;
    let mut root_seen = false;
    let mut i = 0usize;

    while i < command.args.len() {
        match command.args[i].as_str() {
            "--help" | "-h" => return Err("Usage requested".into()),
            "--all" | "-a" => {
                all_mode = true;
                show_hidden = true;
            }
            "--dirs-only" | "-d" => dirs_only = true,
            "--depth" | "-L" => {
                i += 1;
                let Some(value) = command.args.get(i) else {
                    return Err("Missing number after --depth".into());
                };
                max_depth = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| format!("Invalid depth '{}'", value))?,
                );
            }
            option if option.starts_with('-') => {
                return Err(format!("Unknown tree option '{}'", option));
            }
            path => {
                if root_seen {
                    return Err("Only one root path can be provided".into());
                }
                root = PathBuf::from(path);
                root_seen = true;
            }
        }

        i += 1;
    }

    Ok(TreeOptions {
        root,
        max_depth: max_depth.unwrap_or(if all_mode { ALL_DEPTH } else { FINDER_DEPTH }),
        show_hidden,
        dirs_only,
        all_mode,
    })
}

fn walk_tree(
    path: &Path,
    prefix: &str,
    depth: usize,
    options: &TreeOptions,
    stats: &mut TreeStats,
) {
    if depth >= options.max_depth {
        return;
    }

    let entries = match read_entries(path, options) {
        Ok(entries) => entries,
        Err(e) => {
            stats.skipped += 1;
            println!("{prefix}{RED}└── <{}>{RESET}", e);
            return;
        }
    };

    for (index, entry) in entries.iter().enumerate() {
        if stats.dirs + stats.files >= MAX_ENTRIES {
            stats.skipped += 1;
            println!("{prefix}{YELLOW}└── ... entry limit reached{RESET}");
            return;
        }

        let is_last = index + 1 == entries.len();
        let branch = if is_last { "└── " } else { "├── " };
        let next_prefix = if is_last { "    " } else { "│   " };

        if entry.is_dir {
            stats.dirs += 1;
        } else {
            stats.files += 1;
        }

        println!(
            "{prefix}{DIM}{branch}{RESET}{}",
            display_name(&entry.path, entry.is_dir, entry.is_symlink)
        );

        if entry.is_dir && !entry.is_symlink {
            walk_tree(
                &entry.path,
                &format!("{prefix}{next_prefix}"),
                depth + 1,
                options,
                stats,
            );
        }
    }
}

fn read_entries(path: &Path, options: &TreeOptions) -> Result<Vec<TreeEntry>, String> {
    let mut entries = Vec::new();

    for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if !options.show_hidden && name.starts_with('.') {
            continue;
        }

        let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        let file_type = metadata.file_type();
        let is_dir = file_type.is_dir();
        let is_symlink = file_type.is_symlink();

        if options.dirs_only && !is_dir {
            continue;
        }

        entries.push(TreeEntry {
            path,
            name,
            is_dir,
            is_symlink,
        });
    }

    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(entries)
}

fn display_name(path: &Path, is_dir: bool, is_symlink: bool) -> String {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string());

    if is_symlink {
        format!("{YELLOW}{name}{RESET} {DIM}->{RESET}")
    } else if is_dir {
        format!("{BLUE}{name}/{RESET}")
    } else {
        format!("{GREEN}{name}{RESET}")
    }
}

fn print_usage() {
    eprintln!("Usage: tree [path] [--depth N] [--all] [--dirs-only]");
    eprintln!("Examples:");
    eprintln!("  tree              # Finder-like visible contents only");
    eprintln!("  tree . --all      # detailed recursive view, including hidden files");
    eprintln!("  tree src --depth 2");
    eprintln!("  tree /tmp --dirs-only");
}

#[cfg(test)]
mod tests {
    use super::parse_options;

    #[test]
    fn parses_tree_defaults() {
        let options = parse_options("tree").unwrap();

        assert_eq!(options.root.to_string_lossy(), ".");
        assert_eq!(options.max_depth, 1);
        assert!(!options.show_hidden);
        assert!(!options.all_mode);
    }

    #[test]
    fn parses_tree_path_depth_and_all() {
        let options = parse_options("tree src --depth 2 --all").unwrap();

        assert_eq!(options.root.to_string_lossy(), "src");
        assert_eq!(options.max_depth, 2);
        assert!(options.show_hidden);
        assert!(options.all_mode);
    }

    #[test]
    fn all_mode_expands_default_depth() {
        let options = parse_options("tree --all").unwrap();

        assert_eq!(options.max_depth, 4);
        assert!(options.show_hidden);
        assert!(options.all_mode);
    }
}
