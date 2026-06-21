pub fn run(input: &str) {
    let parts: Vec<&str> = input.split_whitespace().collect();

    if parts.len() < 3 {
        println!("\x1b[91mUsage: calc <number> <operator> <number>\x1b[0m");
        return;
    }

    let left: i32 = parts[1].parse().unwrap();
    let op = parts[2];

    let right: i32 = if op == "sqrt" || op == "square_root" {
        0
    } else {
        if parts.len() < 4 {
            println!("\x1b[91mUsage: calc <number> <operator> <number>\x1b[0m");
            return;
        }

        parts[3].parse().unwrap()
    };

    match op {
        "+" | "add" => println!("{}", left + right),
        "-" | "sub" => println!("{}", left - right),
        "*" | "mul" => println!("{}", left * right),
        "/" | "div" => println!("{}", left / right),
        "^" | "pow" => println!("{}", left.pow(right as u32)),
        "%" | "mod" => println!("{}", left % right),
        "sqrt" | "square_root" => println!("{}", (left as f64).sqrt()),
        _ => println!("Unknown operator"),
    }
}