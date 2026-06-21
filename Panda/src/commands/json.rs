use crate::parser;
use serde_json::Value;
use std::fs;

pub fn run(input: &str) {
    let args = match args(input) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{}", e);
            usage();
            return;
        }
    };

    if args.is_empty() || matches!(args[0].as_str(), "--help" | "-h") {
        usage();
        return;
    }

    let result = match args[0].as_str() {
        "pretty" => parse_text(&args[1..])
            .and_then(|v| serde_json::to_string_pretty(&v).map_err(|e| e.to_string())),
        "min" => parse_text(&args[1..])
            .and_then(|v| serde_json::to_string(&v).map_err(|e| e.to_string())),
        "file" => {
            let Some(path) = args.get(1) else {
                eprintln!("Usage: json file <path>");
                return;
            };
            fs::read_to_string(path)
                .map_err(|e| e.to_string())
                .and_then(|text| serde_json::from_str::<Value>(&text).map_err(|e| e.to_string()))
                .and_then(|v| serde_json::to_string_pretty(&v).map_err(|e| e.to_string()))
        }
        "valid" | "validate" => parse_text(&args[1..]).map(|_| "valid JSON".to_string()),
        other => Err(format!("Unknown json mode '{}'", other)),
    };

    match result {
        Ok(output) => println!("{}", output),
        Err(e) => eprintln!("\x1b[91mJSON error: {}\x1b[0m", e),
    }
}

fn args(input: &str) -> Result<Vec<String>, String> {
    parser::parse_line(input)?
        .commands
        .first()
        .map(|cmd| cmd.args.clone())
        .ok_or_else(|| "Missing json command".into())
}

fn parse_text(args: &[String]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("Missing JSON text".into());
    }
    serde_json::from_str(&args.join(" ")).map_err(|e| e.to_string())
}

fn usage() {
    eprintln!("Usage: json pretty|min|validate <json> | json file <path>");
}
