use crate::parser;

pub fn run(input: &str) {
    let args = parser::parse_line(input)
        .ok()
        .and_then(|ast| ast.commands.first().map(|cmd| cmd.args.clone()))
        .unwrap_or_default();

    if args.len() < 2 || matches!(args[0].as_str(), "--help" | "-h") {
        usage();
        return;
    }

    let text = args[1..].join(" ");
    match args[0].as_str() {
        "encode" => println!("{}", encode(&text)),
        "decode" => match decode(&text) {
            Ok(value) => println!("{}", value),
            Err(e) => eprintln!("\x1b[91m{}\x1b[0m", e),
        },
        other => {
            eprintln!("Unknown url mode '{}'", other);
            usage();
        }
    }
}

fn encode(input: &str) -> String {
    input
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            b' ' => vec!['+'],
            _ => format!("%{:02X}", byte).chars().collect(),
        })
        .collect()
}

fn decode(input: &str) -> Result<String, String> {
    let mut output = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => output.push(b' '),
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).map_err(|e| e.to_string())?;
                output.push(u8::from_str_radix(hex, 16).map_err(|_| "Invalid percent escape")?);
                i += 2;
            }
            b'%' => return Err("Incomplete percent escape".into()),
            byte => output.push(byte),
        }
        i += 1;
    }
    String::from_utf8(output).map_err(|e| e.to_string())
}

fn usage() {
    eprintln!("Usage: url encode <text> | url decode <text>");
}
