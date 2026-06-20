use crate::commands::calc::{parse_expression, eval_with_x};

pub fn run(input: &str) {
    let rest = input.trim().strip_prefix("formula").unwrap_or("").trim();

    if rest.is_empty() {
        println!("\x1b[91mUsage: formula <expression> <min> <max>\x1b[0m");
        return;
    }

    let parts: Vec<&str> = rest.split_whitespace().collect();

    if parts.len() < 3 {
        println!("\x1b[91mUsage: formula <expression> <min> <max>\x1b[0m");
        return;
    }

    let min: f64 = match parts[parts.len() - 2].parse() {
        Ok(v) => v,
        Err(_) => {
            println!("\x1b[91mInvalid minimum value\x1b[0m");
            return;
        }
    };

    let max: f64 = match parts[parts.len() - 1].parse() {
        Ok(v) => v,
        Err(_) => {
            println!("\x1b[91mInvalid maximum value\x1b[0m");
            return;
        }
    };

    if min > max {
        println!("\x1b[91mMinimum cannot be greater than maximum\x1b[0m");
        return;
    }

    let expr_str = parts[..parts.len() - 2].join(" ");

    let expr = match parse_expression(&expr_str) {
        Ok(e) => e,
        Err(e) => {
            println!("\x1b[91mError parsing expression: {}\x1b[0m", e);
            return;
        }
    };

    let mut results = Vec::new();

    let steps = 1000;
    let step_size = (max - min) / steps as f64;

    for i in 0..=steps {
        let x = min + i as f64 * step_size;
        let y = eval_with_x(&expr, x);
        results.push(y);
    }

    println!("{:?}", results);
}
