use crate::parser;

pub fn run(input: &str) {
    let args = parser::parse_line(input)
        .ok()
        .and_then(|ast| ast.commands.first().map(|cmd| cmd.args.clone()))
        .unwrap_or_default();

    match args.first().map(String::as_str) {
        Some("search") => {
            let Some(term) = args.get(1) else {
                eprintln!("Usage: envx search <term>");
                return;
            };
            for (key, value) in std::env::vars().filter(|(k, _)| k.contains(term)) {
                println!("{}={}", key, value);
            }
        }
        Some("list") | None => {
            for (key, value) in std::env::vars() {
                println!("{}={}", key, value);
            }
        }
        Some(key) => match std::env::var(key) {
            Ok(value) => println!("{}", value),
            Err(_) => eprintln!("Environment variable not found: {}", key),
        },
    }
}
