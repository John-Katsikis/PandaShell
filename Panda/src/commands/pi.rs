use rug::{float::Constant, Float};

const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[38;5;82m";
const CYAN: &str = "\x1b[38;5;51m";
const MAGENTA: &str = "\x1b[38;5;213m";
const YELLOW: &str = "\x1b[38;5;220m";
const RED: &str = "\x1b[38;5;203m";
const DIGIT_COLORS: [&str; 6] = [
    "\x1b[38;5;82m",
    "\x1b[38;5;51m",
    "\x1b[38;5;75m",
    "\x1b[38;5;141m",
    "\x1b[38;5;213m",
    "\x1b[38;5;220m",
];

const MAX_PRETTY_DIGITS: u32 = 5000;

#[derive(Debug)]
struct PiOptions {
    digits: u32,
    plain: bool,
    group: usize,
}

pub fn run(input: &str) {
    let options = match parse_options(input) {
        Ok(options) => options,
        Err(e) => {
            println!("\x1b[91m{}\x1b[0m", e);
            print_usage();
            return;
        }
    };

    let pi_string = calculate_pi(options.digits);

    if options.plain {
        println!("{}", pi_string);
        return;
    }

    print_pretty_pi(&pi_string, &options);
}

fn parse_options(input: &str) -> Result<PiOptions, String> {
    let parts: Vec<&str> = input.split_whitespace().collect();

    if parts.len() < 2 || parts.iter().any(|part| matches!(*part, "--help" | "-h")) {
        return Err("Usage: pi <digits> [--plain] [--group N]".into());
    }

    let digits = parts[1]
        .parse::<u32>()
        .map_err(|_| "Please enter a valid number of digits.".to_string())?;

    if digits == 0 {
        return Err("Digit count must be greater than zero.".into());
    }

    if digits > MAX_PRETTY_DIGITS && !parts.contains(&"--plain") {
        return Err(format!(
            "Pretty pi is capped at {} digits. Use --plain for huge output.",
            MAX_PRETTY_DIGITS
        ));
    }

    let mut options = PiOptions {
        digits,
        plain: false,
        group: 5,
    };

    let mut i = 2;
    while i < parts.len() {
        match parts[i] {
            "--plain" => options.plain = true,
            "--group" => {
                i += 1;
                let Some(value) = parts.get(i) else {
                    return Err("Missing number after --group".into());
                };
                options.group = value
                    .parse::<usize>()
                    .map_err(|_| format!("Invalid group size '{}'", value))?;
                if options.group == 0 {
                    return Err("Group size must be greater than zero.".into());
                }
            }
            other => return Err(format!("Unknown pi option '{}'", other)),
        }

        i += 1;
    }

    Ok(options)
}

fn calculate_pi(digits: u32) -> String {
    let precision_bits = ((digits as f64) * 3.321_928_094_887_362 + 16.0) as u32;
    let pi = Float::with_val(precision_bits, Constant::Pi);
    pi.to_string_radix(10, Some(digits as usize + 2))
}

fn print_pretty_pi(pi_string: &str, options: &PiOptions) {
    let fractional_digits = pi_string
        .split_once('.')
        .map(|(_, digits)| digits)
        .unwrap_or("");

    println!();
    println!("{GREEN}╭──────────────────── PI ORBIT ────────────────────╮{RESET}");
    println!(
        "{GREEN}│{RESET} {MAGENTA}π{RESET} requested digits {YELLOW}{:<8}{RESET} precision bits {CYAN}{:<8}{RESET}{GREEN}│{RESET}",
        options.digits,
        ((options.digits as f64) * 3.321_928_094_887_362 + 16.0) as u32
    );
    println!("{GREEN}╰──────────────────────────────────────────────────╯{RESET}");
    println!("{DIM}        ◌      ◍        ◌     ◍        ◌{RESET}");
    println!();

    print_digit_stream(pi_string, options.group);
    print_digit_histogram(fractional_digits);
    print_pi_footer(fractional_digits);
    println!();
}

fn print_digit_stream(pi_string: &str, group: usize) {
    let mut chars = pi_string.chars();
    let Some(first) = chars.next() else {
        return;
    };

    print!("{MAGENTA}{first}{RESET}");

    if chars.next() == Some('.') {
        print!("{DIM}.{RESET}");
    }

    let mut count = 0usize;
    for (index, digit) in chars.enumerate() {
        if count > 0 && count % group == 0 {
            print!("{DIM} {RESET}");
        }
        if count > 0 && count % (group * 8) == 0 {
            println!();
            print!("  ");
        }

        let color = DIGIT_COLORS[index % DIGIT_COLORS.len()];
        print!("{color}{digit}{RESET}");
        count += 1;
    }

    println!();
}

fn print_digit_histogram(digits: &str) {
    let mut counts = [0usize; 10];
    for digit in digits.chars().filter_map(|c| c.to_digit(10)) {
        counts[digit as usize] += 1;
    }

    let max = counts.iter().copied().max().unwrap_or(0).max(1);
    println!();
    println!("{CYAN}digit constellation{RESET}");

    for (digit, count) in counts.iter().enumerate() {
        let bar_len = (*count * 24 / max).max(usize::from(*count > 0));
        let bar = "█".repeat(bar_len);
        let color = DIGIT_COLORS[digit % DIGIT_COLORS.len()];
        println!("{DIM}{digit}{RESET} {color}{bar:<24}{RESET} {count}");
    }
}

fn print_pi_footer(digits: &str) {
    let checksum: u32 = digits.chars().filter_map(|c| c.to_digit(10)).sum();
    let first_twenty = digits.chars().take(20).collect::<String>();

    println!();
    println!(
        "{RED}trail{RESET} {DIM}first 20:{RESET} {}  {DIM}digit-sum:{RESET} {}",
        first_twenty, checksum
    );
}

fn print_usage() {
    println!("Usage: pi <digits> [--plain] [--group N]");
    println!("Examples:");
    println!("  pi 100");
    println!("  pi 250 --group 10");
    println!("  pi 1000 --plain");
}

#[cfg(test)]
mod tests {
    use super::{calculate_pi, parse_options};

    #[test]
    fn calculates_requested_digits() {
        let pi = calculate_pi(10);
        assert!(pi.starts_with("3.14159265"));
    }

    #[test]
    fn parses_plain_and_group() {
        let options = parse_options("pi 100 --plain --group 10").unwrap();

        assert_eq!(options.digits, 100);
        assert!(options.plain);
        assert_eq!(options.group, 10);
    }

    #[test]
    fn rejects_zero_digits() {
        assert!(parse_options("pi 0").is_err());
    }
}
