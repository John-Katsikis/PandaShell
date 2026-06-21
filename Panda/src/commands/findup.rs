use crate::parser;
use std::env;

pub fn run(input: &str) {
    let args = parser::parse_line(input)
        .ok()
        .and_then(|ast| ast.commands.first().map(|cmd| cmd.args.clone()))
        .unwrap_or_default();

    let Some(target) = args.first() else {
        eprintln!("Usage: findup <file-or-dir-name>");
        return;
    };

    let Ok(mut dir) = env::current_dir() else {
        eprintln!("Could not read current directory");
        return;
    };

    loop {
        let candidate = dir.join(target);
        if candidate.exists() {
            println!("{}", candidate.display());
            return;
        }
        if !dir.pop() {
            break;
        }
    }

    eprintln!("Not found above current directory: {}", target);
}
