use qrcode::{Color, QrCode};

use crate::parser;

const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[38;5;82m";
const CYAN: &str = "\x1b[38;5;51m";
const YELLOW: &str = "\x1b[38;5;220m";
const RED: &str = "\x1b[91m";

#[derive(Debug, Clone, PartialEq, Eq)]
struct Options {
    text: String,
    border: usize,
    compact: bool,
    invert: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            text: String::new(),
            border: 2,
            compact: false,
            invert: false,
        }
    }
}

pub fn run(input: &str) {
    let args = parser::parse_line(input)
        .ok()
        .and_then(|ast| ast.commands.first().map(|cmd| cmd.args.clone()))
        .unwrap_or_default();

    match parse_args(&args) {
        Ok(options) => {
            if options.text.is_empty() {
                usage();
                return;
            }

            match render_qr(&options) {
                Ok(output) => {
                    println!(
                        "{GREEN}QR CODE{RESET} {DIM}{} bytes{RESET}",
                        options.text.len()
                    );
                    println!("{output}");
                    println!(
                        "{DIM}Tip: use `qr --compact \"{}\"` for a shorter terminal version.{RESET}",
                        preview_text(&options.text)
                    );
                }
                Err(e) => eprintln!("{RED}{e}{RESET}"),
            }
        }
        Err(e) if e == "help" => usage(),
        Err(e) => {
            eprintln!("{RED}{e}{RESET}");
            usage();
        }
    }
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut options = Options::default();
    let mut text_parts = Vec::new();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => return Err("help".into()),
            "--compact" | "-c" => options.compact = true,
            "--invert" | "-i" => options.invert = true,
            "--border" | "-b" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("Missing value after --border".into());
                };
                options.border = value
                    .parse::<usize>()
                    .map_err(|_| "Border must be a whole number".to_string())?;
                if options.border > 12 {
                    return Err("Border must be 12 or less".into());
                }
            }
            flag if flag.starts_with('-') => return Err(format!("Unknown qr flag '{flag}'")),
            value => text_parts.push(value.to_string()),
        }

        index += 1;
    }

    options.text = text_parts.join(" ");
    Ok(options)
}

fn render_qr(options: &Options) -> Result<String, String> {
    let code = QrCode::new(options.text.as_bytes()).map_err(|e| e.to_string())?;
    let matrix = matrix_with_border(&code, options.border, options.invert);

    if options.compact {
        Ok(render_compact(&matrix))
    } else {
        Ok(render_full(&matrix))
    }
}

fn matrix_with_border(code: &QrCode, border: usize, invert: bool) -> Vec<Vec<bool>> {
    let width = code.width();
    let size = width + border * 2;
    let mut matrix = vec![vec![false; size]; size];

    for y in 0..width {
        for x in 0..width {
            let filled = code[(x, y)] == Color::Dark;
            matrix[y + border][x + border] = if invert { !filled } else { filled };
        }
    }

    if invert && border > 0 {
        for y in 0..size {
            for x in 0..size {
                if x < border || y < border || x >= width + border || y >= width + border {
                    matrix[y][x] = true;
                }
            }
        }
    }

    matrix
}

fn render_full(matrix: &[Vec<bool>]) -> String {
    let mut output = String::new();

    for row in matrix {
        output.push_str(CYAN);
        for filled in row {
            output.push_str(if *filled { "██" } else { "  " });
        }
        output.push_str(RESET);
        output.push('\n');
    }

    output
}

fn render_compact(matrix: &[Vec<bool>]) -> String {
    let mut output = String::new();
    let mut y = 0;

    while y < matrix.len() {
        output.push_str(YELLOW);
        for x in 0..matrix[y].len() {
            let top = matrix[y][x];
            let bottom = matrix.get(y + 1).map(|row| row[x]).unwrap_or(false);

            let ch = match (top, bottom) {
                (true, true) => '█',
                (true, false) => '▀',
                (false, true) => '▄',
                (false, false) => ' ',
            };

            output.push(ch);
        }
        output.push_str(RESET);
        output.push('\n');
        y += 2;
    }

    output
}

fn preview_text(text: &str) -> String {
    const MAX: usize = 24;
    let mut chars = text.chars();
    let preview: String = chars.by_ref().take(MAX).collect();

    if chars.next().is_some() {
        format!("{preview}...")
    } else {
        preview
    }
}

fn usage() {
    eprintln!("{GREEN}Usage:{RESET} qr [options] <text>");
    eprintln!();
    eprintln!("{CYAN}Options{RESET}");
    eprintln!("  --compact, -c       Render half-height QR output");
    eprintln!("  --invert, -i        Invert dark and light modules");
    eprintln!("  --border N, -b N    Set quiet-zone border, default 2");
    eprintln!("  --help, -h          Show this help");
    eprintln!();
    eprintln!("{YELLOW}Examples{RESET}");
    eprintln!("{DIM}  qr \"hello panda\"{RESET}");
    eprintln!("{DIM}  qr --compact https://example.com{RESET}");
    eprintln!("{DIM}  qr --border 4 --invert \"secret message\"{RESET}");
}

#[cfg(test)]
mod tests {
    use super::{parse_args, render_qr};

    #[test]
    fn parses_text_and_flags() {
        let args = vec![
            "--compact".to_string(),
            "--border".to_string(),
            "4".to_string(),
            "hello".to_string(),
            "panda".to_string(),
        ];

        let options = parse_args(&args).unwrap();

        assert_eq!(options.text, "hello panda");
        assert_eq!(options.border, 4);
        assert!(options.compact);
    }

    #[test]
    fn renders_full_qr_output() {
        let options = parse_args(&["hello".to_string()]).unwrap();
        let output = render_qr(&options).unwrap();

        assert!(output.contains("██"));
        assert!(output.lines().count() > 10);
    }

    #[test]
    fn renders_compact_qr_output() {
        let options = parse_args(&["--compact".to_string(), "hello".to_string()]).unwrap();
        let output = render_qr(&options).unwrap();

        assert!(output.contains('█') || output.contains('▀') || output.contains('▄'));
        assert!(output.lines().count() > 5);
    }
}
