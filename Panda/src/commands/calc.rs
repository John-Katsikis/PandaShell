use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    Variable,
    Constant(String),
    UnaryOp {
        op: char,
        expr: Box<Expr>,
    },
    BinaryOp {
        left: Box<Expr>,
        op: char,
        right: Box<Expr>,
    },
    Factorial(Box<Expr>),
    Func {
        name: String,
        args: Vec<Expr>,
    },
}

pub fn run(input: &str) {
    let expr_str = input.trim().strip_prefix("calc").unwrap_or("").trim();

    if expr_str.is_empty() {
        print_usage();
        return;
    }

    match parse_expression(expr_str).and_then(|expr| eval(&expr)) {
        Ok(value) => println!("{}", value),
        Err(e) => println!("\x1b[91mError: {}\x1b[0m", e),
    }
}

pub fn parse_expression(input: &str) -> Result<Expr, String> {
    let mut chars = input.chars().peekable();
    let expr = parse_expr(&mut chars)?;
    skip_spaces(&mut chars);

    if chars.peek().is_some() {
        return Err("Unexpected characters at end of expression".into());
    }

    Ok(expr)
}

pub fn eval_with_x(expr: &Expr, x: f64) -> Result<f64, String> {
    eval_inner(expr, Some(x))
}

pub fn eval(expr: &Expr) -> Result<f64, String> {
    eval_inner(expr, None)
}

fn print_usage() {
    println!("\x1b[91mUsage: calc <expression>\x1b[0m");
    println!("Examples: calc 2sin(pi/4) + sqrt(9), calc log(2, 8), calc 5!");
}

fn parse_expr(chars: &mut Peekable<Chars>) -> Result<Expr, String> {
    parse_add_sub(chars)
}

fn parse_add_sub(chars: &mut Peekable<Chars>) -> Result<Expr, String> {
    let mut node = parse_mul_div(chars)?;

    loop {
        skip_spaces(chars);
        match chars.peek().copied() {
            Some('+') | Some('-') => {
                let op = chars.next().unwrap();
                let right = parse_mul_div(chars)?;
                node = Expr::BinaryOp {
                    left: Box::new(node),
                    op,
                    right: Box::new(right),
                };
            }
            _ => break,
        }
    }

    Ok(node)
}

fn parse_mul_div(chars: &mut Peekable<Chars>) -> Result<Expr, String> {
    let mut node = parse_unary(chars)?;

    loop {
        skip_spaces(chars);
        match chars.peek().copied() {
            Some('*') | Some('/') | Some('%') => {
                let op = chars.next().unwrap();
                let right = parse_unary(chars)?;
                node = Expr::BinaryOp {
                    left: Box::new(node),
                    op,
                    right: Box::new(right),
                };
            }
            Some(c) if starts_primary(c) => {
                let right = parse_unary(chars)?;
                node = Expr::BinaryOp {
                    left: Box::new(node),
                    op: '*',
                    right: Box::new(right),
                };
            }
            _ => break,
        }
    }

    Ok(node)
}

fn parse_unary(chars: &mut Peekable<Chars>) -> Result<Expr, String> {
    skip_spaces(chars);

    match chars.peek().copied() {
        Some('+') | Some('-') => {
            let op = chars.next().unwrap();
            let expr = parse_unary(chars)?;
            Ok(Expr::UnaryOp {
                op,
                expr: Box::new(expr),
            })
        }
        _ => parse_pow(chars),
    }
}

fn parse_pow(chars: &mut Peekable<Chars>) -> Result<Expr, String> {
    let node = parse_postfix(chars)?;

    skip_spaces(chars);
    if chars.peek() == Some(&'^') {
        chars.next();
        let right = parse_unary(chars)?;
        return Ok(Expr::BinaryOp {
            left: Box::new(node),
            op: '^',
            right: Box::new(right),
        });
    }

    Ok(node)
}

fn parse_postfix(chars: &mut Peekable<Chars>) -> Result<Expr, String> {
    let mut node = parse_primary(chars)?;

    loop {
        skip_spaces(chars);
        if chars.peek() == Some(&'!') {
            chars.next();
            node = Expr::Factorial(Box::new(node));
        } else {
            break;
        }
    }

    Ok(node)
}

fn parse_primary(chars: &mut Peekable<Chars>) -> Result<Expr, String> {
    skip_spaces(chars);

    match chars.peek().copied() {
        Some(c) if c.is_ascii_digit() || c == '.' => parse_number(chars),
        Some(c) if c.is_ascii_alphabetic() => parse_identifier(chars),
        Some('(') => {
            chars.next();
            let expr = parse_expr(chars)?;
            skip_spaces(chars);

            if chars.next() != Some(')') {
                return Err("Missing ')'".into());
            }

            Ok(expr)
        }
        Some(c) => Err(format!("Unexpected character '{}'", c)),
        None => Err("Unexpected end of expression".into()),
    }
}

fn parse_identifier(chars: &mut Peekable<Chars>) -> Result<Expr, String> {
    let name = parse_identifier_name(chars)?.to_lowercase();
    skip_spaces(chars);

    if name == "x" {
        return Ok(Expr::Variable);
    }

    if is_constant(&name) && chars.peek() != Some(&'(') {
        return Ok(Expr::Constant(name));
    }

    if chars.peek() == Some(&'(') {
        chars.next();
        let args = parse_function_args(chars)?;
        return Ok(Expr::Func { name, args });
    }

    let arg = parse_primary(chars)?;
    Ok(Expr::Func {
        name,
        args: vec![arg],
    })
}

fn parse_function_args(chars: &mut Peekable<Chars>) -> Result<Vec<Expr>, String> {
    let mut args = Vec::new();

    skip_spaces(chars);
    if chars.peek() == Some(&')') {
        chars.next();
        return Ok(args);
    }

    loop {
        args.push(parse_expr(chars)?);
        skip_spaces(chars);

        match chars.next() {
            Some(',') => {
                skip_spaces(chars);
            }
            Some(')') => break,
            _ => return Err("Missing ')' after function argument".into()),
        }
    }

    Ok(args)
}

fn parse_identifier_name(chars: &mut Peekable<Chars>) -> Result<String, String> {
    let mut name = String::new();

    while let Some(&c) = chars.peek() {
        if c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '_' {
            name.push(c);
            chars.next();
        } else {
            break;
        }
    }

    if name.is_empty() {
        Err("Expected identifier".into())
    } else {
        Ok(name)
    }
}

fn parse_number(chars: &mut Peekable<Chars>) -> Result<Expr, String> {
    skip_spaces(chars);

    let mut num = String::new();
    let mut seen_dot = false;

    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            num.push(c);
            chars.next();
        } else if c == '.' && !seen_dot {
            seen_dot = true;
            num.push(c);
            chars.next();
        } else {
            break;
        }
    }

    if num == "." {
        return Err("Invalid number".into());
    }

    num.parse::<f64>()
        .map(Expr::Number)
        .map_err(|_| "Invalid number".into())
}

pub fn skip_spaces(chars: &mut Peekable<Chars>) {
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
}

fn starts_primary(c: char) -> bool {
    c.is_ascii_digit() || c.is_ascii_alphabetic() || c == '.' || c == '('
}

fn is_constant(name: &str) -> bool {
    matches!(name, "pi" | "e" | "tau" | "phi")
}

fn eval_inner(expr: &Expr, x: Option<f64>) -> Result<f64, String> {
    match expr {
        Expr::Number(n) => Ok(*n),
        Expr::Variable => x.ok_or_else(|| "Variable x is only available in formula".into()),
        Expr::Constant(name) => constant_value(name),
        Expr::UnaryOp { op: '+', expr } => eval_inner(expr, x),
        Expr::UnaryOp { op: '-', expr } => Ok(-eval_inner(expr, x)?),
        Expr::UnaryOp { op, .. } => Err(format!("Unsupported unary operator '{}'", op)),
        Expr::BinaryOp { left, op, right } => {
            let l = eval_inner(left, x)?;
            let r = eval_inner(right, x)?;
            eval_binary(*op, l, r)
        }
        Expr::Factorial(expr) => factorial(eval_inner(expr, x)?),
        Expr::Func { name, args } => {
            let values = args
                .iter()
                .map(|arg| eval_inner(arg, x))
                .collect::<Result<Vec<_>, _>>()?;
            eval_function(name, &values)
        }
    }
}

fn constant_value(name: &str) -> Result<f64, String> {
    match name {
        "pi" => Ok(std::f64::consts::PI),
        "e" => Ok(std::f64::consts::E),
        "tau" => Ok(std::f64::consts::TAU),
        "phi" => Ok(1.618_033_988_749_895),
        _ => Err(format!("Unknown constant '{}'", name)),
    }
}

fn eval_binary(op: char, l: f64, r: f64) -> Result<f64, String> {
    match op {
        '+' => Ok(l + r),
        '-' => Ok(l - r),
        '*' => Ok(l * r),
        '/' => Ok(l / r),
        '%' => Ok(l % r),
        '^' => Ok(l.powf(r)),
        _ => Err(format!("Unknown operator '{}'", op)),
    }
}

fn eval_function(name: &str, args: &[f64]) -> Result<f64, String> {
    match (name, args) {
        ("abs", [x]) => Ok(x.abs()),
        ("sqrt", [x]) => Ok(x.sqrt()),
        ("cbrt", [x]) => Ok(x.cbrt()),
        ("exp", [x]) => Ok(x.exp()),
        ("ln", [x]) => Ok(x.ln()),
        ("log", [x]) => Ok(x.log10()),
        ("log", [base, x]) => Ok(x.log(*base)),
        ("log10", [x]) => Ok(x.log10()),
        ("log2", [x]) => Ok(x.log2()),
        ("sin", [x]) => Ok(x.sin()),
        ("cos", [x]) => Ok(x.cos()),
        ("tan", [x]) => Ok(x.tan()),
        ("asin", [x]) => Ok(x.asin()),
        ("acos", [x]) => Ok(x.acos()),
        ("atan", [x]) => Ok(x.atan()),
        ("sinh", [x]) => Ok(x.sinh()),
        ("cosh", [x]) => Ok(x.cosh()),
        ("tanh", [x]) => Ok(x.tanh()),
        ("floor", [x]) => Ok(x.floor()),
        ("ceil", [x]) => Ok(x.ceil()),
        ("round", [x]) => Ok(x.round()),
        ("sign", [x]) => Ok(x.signum()),
        ("deg", [x]) => Ok(x.to_degrees()),
        ("rad", [x]) => Ok(x.to_radians()),
        ("recip", [x]) => Ok(x.recip()),
        ("min", values) if !values.is_empty() => {
            Ok(values.iter().copied().fold(f64::INFINITY, f64::min))
        }
        ("max", values) if !values.is_empty() => {
            Ok(values.iter().copied().fold(f64::NEG_INFINITY, f64::max))
        }
        ("root", [n, x]) => Ok(x.powf(1.0 / n)),
        _ if is_constant(name) && args.is_empty() => constant_value(name),
        _ => Err(format!(
            "Unknown function or wrong number of arguments: {}",
            name
        )),
    }
}

fn factorial(value: f64) -> Result<f64, String> {
    if !value.is_finite() || value < 0.0 || value.fract() != 0.0 {
        return Err("Factorial is only defined for non-negative integers".into());
    }

    let n = value as u64;
    if n > 170 {
        return Err("Factorial is too large for f64".into());
    }

    Ok((1..=n).fold(1.0, |acc, n| acc * n as f64))
}

#[cfg(test)]
mod tests {
    use super::{eval, eval_with_x, parse_expression};

    fn calc(input: &str) -> f64 {
        eval(&parse_expression(input).unwrap()).unwrap()
    }

    fn formula(input: &str, x: f64) -> f64 {
        eval_with_x(&parse_expression(input).unwrap(), x).unwrap()
    }

    #[test]
    fn supports_common_math() {
        assert!((calc("2sin(pi/2) + sqrt(9)") - 5.0).abs() < 1e-10);
        assert!((calc("log(2, 8)") - 3.0).abs() < 1e-10);
        assert!((calc("5!") - 120.0).abs() < 1e-10);
        assert!((calc("-2^2") + 4.0).abs() < 1e-10);
    }

    #[test]
    fn supports_formula_variable() {
        assert!((formula("x^2 + 2x + 1", 3.0) - 16.0).abs() < 1e-10);
    }

    #[test]
    fn unknown_functions_return_errors() {
        let expr = parse_expression("notafunction(2)").unwrap();
        assert!(eval(&expr).is_err());
    }
}
