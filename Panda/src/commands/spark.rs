use sha2::{Digest, Sha256};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[38;5;82m";
const CYAN: &str = "\x1b[38;5;51m";
const YELLOW: &str = "\x1b[38;5;220m";
const RED: &str = "\x1b[91m";
const COLORS: [&str; 8] = [
    "\x1b[38;5;82m",
    "\x1b[38;5;51m",
    "\x1b[38;5;75m",
    "\x1b[38;5;141m",
    "\x1b[38;5;213m",
    "\x1b[38;5;220m",
    "\x1b[38;5;203m",
    "\x1b[38;5;118m",
];
const GLYPHS: [char; 10] = [' ', '·', '∙', '◆', '◇', '✦', '✧', '█', '▓', '▒'];

#[derive(Debug, Clone, PartialEq, Eq)]
enum Mode {
    Seed(String),
    File(String),
    Compare(String, String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Options {
    mode: Mode,
    compact: bool,
    signature_only: bool,
}

pub fn run_seed(rest: &str) {
    match parse_args(rest) {
        Ok(options) => run_options(&options),
        Err(e) if e == "help" => print_usage(),
        Err(e) => {
            eprintln!("{RED}{e}{RESET}");
            print_usage();
        }
    }
}

fn run_options(options: &Options) {
    match &options.mode {
        Mode::Seed(seed_text) => render_seed(seed_text, "seed", options),
        Mode::File(path) => match fs::read(path) {
            Ok(bytes) => {
                let digest = sha256_bytes(&bytes);
                render_seed(&digest, &format!("file {path}"), options);
            }
            Err(e) => eprintln!("{RED}Failed to read {path}: {e}{RESET}"),
        },
        Mode::Compare(left, right) => compare_inputs(left, right, options),
    }
}

fn render_seed(seed_text: &str, label: &str, options: &Options) {
    for line in render_seed_lines(seed_text, label, options) {
        println!("{line}");
    }
}

fn render_seed_lines(seed_text: &str, label: &str, options: &Options) -> Vec<String> {
    let seed = hash_seed(&seed_text);

    if options.signature_only {
        return vec![format!("{:016x}", seed)];
    }

    let title_color = COLORS[(seed as usize) % COLORS.len()];
    let accent_color = COLORS[((seed >> 8) as usize) % COLORS.len()];
    let rows = if options.compact { 7 } else { 15 };
    let cols = if options.compact { 17 } else { 29 };
    let center = cols / 2;
    let mut lines = Vec::new();

    lines.push(String::new());
    lines.push(format!(
        "{title_color}╭───────────────────── PANDA SPARK ─────────────────────╮{RESET}"
    ));
    lines.push(format!(
        "{title_color}│{RESET} {DIM}{:<9}{RESET} {accent_color}{:<42}{RESET}{title_color}│{RESET}",
        truncate(label, 9),
        truncate(seed_text, 42)
    ));
    lines.push(format!(
        "{title_color}├────────────────────────────────────────────────────────╯{RESET}"
    ));

    for row in 0..rows {
        let mut line = String::from("   ");
        for col in 0..cols {
            let mirrored_col = if col <= center { col } else { cols - 1 - col };
            let value = cell_value(seed, row, mirrored_col);
            let color = COLORS[(value as usize + row + col) % COLORS.len()];
            let glyph = GLYPHS[(value as usize) % GLYPHS.len()];
            line.push_str(&format!("{color}{glyph}{RESET}"));
        }
        lines.push(line);
    }

    lines.push(format!(
        "{accent_color}   ╰─ signature {RESET}{DIM}{:016x}{RESET}{accent_color} ─╯{RESET}",
        seed
    ));
    lines.push(String::new());
    lines
}

fn compare_inputs(left: &str, right: &str, options: &Options) {
    let left_seed = input_seed(left);
    let right_seed = input_seed(right);
    let left_sig = hash_seed(&left_seed);
    let right_sig = hash_seed(&right_seed);

    println!();
    println!("{GREEN}PANDA SPARK COMPARE{RESET}");
    println!("{DIM}left {RESET}{CYAN}{left}{RESET}");
    println!("{DIM}right{RESET} {CYAN}{right}{RESET}");
    println!();

    if left_sig == right_sig {
        println!("{GREEN}[MATCH]{RESET} signatures are identical: {left_sig:016x}");
    } else {
        println!("{YELLOW}[DIFF]{RESET} left  signature {left_sig:016x}");
        println!("{YELLOW}[DIFF]{RESET} right signature {right_sig:016x}");
    }

    if !options.signature_only {
        print_side_by_side(
            &render_seed_lines(&left_seed, "left", options),
            &render_seed_lines(&right_seed, "right", options),
        );
    }
}

fn print_side_by_side(left: &[String], right: &[String]) {
    let left_width = left.iter().map(|line| visible_len(line)).max().unwrap_or(0);
    let rows = left.len().max(right.len());

    for row in 0..rows {
        let left_line = left.get(row).map_or("", String::as_str);
        let right_line = right.get(row).map_or("", String::as_str);
        println!("{}    {}", pad_visible(left_line, left_width), right_line);
    }
}

fn pad_visible(line: &str, width: usize) -> String {
    let mut output = line.to_string();
    let visible = visible_len(line);

    if visible < width {
        output.push_str(&" ".repeat(width - visible));
    }

    output
}

fn visible_len(line: &str) -> usize {
    let mut chars = line.chars().peekable();
    let mut len = 0;

    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            let _ = chars.next();
            for code in chars.by_ref() {
                if code.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            len += 1;
        }
    }

    len
}

fn input_seed(input: &str) -> String {
    if let Some(path) = input.strip_prefix('@') {
        return fs::read(path)
            .map(|bytes| sha256_bytes(&bytes))
            .unwrap_or_else(|_| format!("missing-file:{path}"));
    }

    input.to_string()
}

fn parse_args(rest: &str) -> Result<Options, String> {
    let args = tokenize(rest)?;
    let mut compact = false;
    let mut signature_only = false;
    let mut mode: Option<Mode> = None;
    let mut seed_parts = Vec::new();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => return Err("help".into()),
            "--compact" | "-c" => compact = true,
            "--signature" | "--sig" => signature_only = true,
            "--file" | "-f" => {
                index += 1;
                let Some(path) = args.get(index) else {
                    return Err("Missing path after --file".into());
                };
                set_mode(&mut mode, Mode::File(path.clone()))?;
            }
            "--compare" => {
                let Some(left) = args.get(index + 1) else {
                    return Err("Missing left value after --compare".into());
                };
                let Some(right) = args.get(index + 2) else {
                    return Err("Missing right value after --compare".into());
                };
                set_mode(&mut mode, Mode::Compare(left.clone(), right.clone()))?;
                index += 2;
            }
            flag if flag.starts_with('-') => return Err(format!("Unknown spark flag '{flag}'")),
            value => seed_parts.push(value.to_string()),
        }

        index += 1;
    }

    if mode.is_none() {
        let seed = if seed_parts.is_empty() {
            default_seed()
        } else {
            seed_parts.join(" ")
        };
        mode = Some(Mode::Seed(seed));
    } else if !seed_parts.is_empty() {
        return Err("Spark mode flags cannot be mixed with extra seed text".into());
    }

    Ok(Options {
        mode: mode.unwrap(),
        compact,
        signature_only,
    })
}

fn set_mode(target: &mut Option<Mode>, value: Mode) -> Result<(), String> {
    if target.is_some() {
        return Err("Only one spark mode can be used at a time".into());
    }
    *target = Some(value);
    Ok(())
}

fn tokenize(input: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut escaped = false;

    for c in input.chars() {
        if escaped {
            current.push(c);
            escaped = false;
            continue;
        }

        if c == '\\' {
            escaped = true;
            continue;
        }

        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => current.push(c),
            None if c == '\'' || c == '"' => quote = Some(c),
            None if c.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            None => current.push(c),
        }
    }

    if let Some(q) = quote {
        return Err(format!("Unclosed quote '{q}'"));
    }
    if escaped {
        current.push('\\');
    }
    if !current.is_empty() {
        tokens.push(current);
    }

    Ok(tokens)
}

fn print_usage() {
    println!("Usage: spark [options] [seed text]");
    println!("       spark --file <path>");
    println!("       spark --compare <left> <right>");
    println!("Examples:");
    println!("  spark");
    println!("  spark panda shell");
    println!("  spark --compact \"release 1\"");
    println!("  spark --file Cargo.lock");
    println!("  spark --compare @Cargo.toml @Cargo.lock");
    println!("  spark --signature \"session id\"");
}

fn default_seed() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format!("panda-{now}")
}

fn hash_seed(input: &str) -> u64 {
    input.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        let mixed = hash ^ u64::from(byte);
        mixed.wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn cell_value(seed: u64, row: usize, col: usize) -> u64 {
    let mut value = seed
        ^ ((row as u64 + 1).wrapping_mul(0x9e37_79b9_7f4a_7c15))
        ^ ((col as u64 + 1).wrapping_mul(0xbf58_476d_1ce4_e5b9));

    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn truncate(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }

    let mut text = input
        .chars()
        .take(max_chars.saturating_sub(1))
        .collect::<String>();
    text.push('…');
    text
}

#[cfg(test)]
mod tests {
    use super::{hash_seed, pad_visible, parse_args, tokenize, visible_len, Mode};

    #[test]
    fn keeps_literal_seed_text() {
        let options = parse_args("hash Yianni").unwrap();

        assert_eq!(options.mode, Mode::Seed("hash Yianni".into()));
    }

    #[test]
    fn parses_file_mode_and_compact() {
        let options = parse_args("--compact --file Cargo.lock").unwrap();

        assert!(options.compact);
        assert_eq!(options.mode, Mode::File("Cargo.lock".into()));
    }

    #[test]
    fn parses_compare_mode() {
        let options = parse_args("--compare @Cargo.toml @Cargo.lock --signature").unwrap();

        assert!(options.signature_only);
        assert_eq!(
            options.mode,
            Mode::Compare("@Cargo.toml".into(), "@Cargo.lock".into())
        );
    }

    #[test]
    fn tokenizes_quoted_seed() {
        assert_eq!(
            tokenize("--compare \"left side\" \"right side\"").unwrap(),
            ["--compare", "left side", "right side"]
        );
    }

    #[test]
    fn stable_signatures_are_deterministic() {
        assert_eq!(hash_seed("panda"), hash_seed("panda"));
        assert_ne!(hash_seed("panda"), hash_seed("Panda"));
    }

    #[test]
    fn pads_ansi_text_by_visible_width() {
        let line = "\x1b[38;5;82mhi\x1b[0m";

        assert_eq!(visible_len(line), 2);
        assert_eq!(visible_len(&pad_visible(line, 5)), 5);
    }
}
