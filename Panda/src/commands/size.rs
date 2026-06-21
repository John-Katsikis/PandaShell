use crate::parser;
use std::fs;
use std::path::Path;

pub fn run(input: &str) {
    let args = parser::parse_line(input)
        .ok()
        .and_then(|ast| ast.commands.first().map(|cmd| cmd.args.clone()))
        .unwrap_or_default();
    let path = args.first().map_or(".", String::as_str);

    match dir_size(Path::new(path)) {
        Ok((bytes, files, dirs)) => {
            println!("{}  ({} files, {} dirs)", format_bytes(bytes), files, dirs);
        }
        Err(e) => eprintln!("{}", e),
    }
}

fn dir_size(path: &Path) -> Result<(u64, usize, usize), String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if metadata.is_file() {
        return Ok((metadata.len(), 1, 0));
    }
    if !metadata.is_dir() {
        return Ok((0, 0, 0));
    }

    let mut total = 0u64;
    let mut files = 0usize;
    let mut dirs = 1usize;
    for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let (b, f, d) = dir_size(&entry.path()).unwrap_or((0, 0, 0));
        total += b;
        files += f;
        dirs += d;
    }
    Ok((total, files, dirs))
}

fn format_bytes(bytes: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit + 1 < units.len() {
        value /= 1024.0;
        unit += 1;
    }
    format!("{:.1} {}", value, units[unit])
}
