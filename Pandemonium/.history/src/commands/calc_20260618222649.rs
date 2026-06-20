use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone)]
enum Expr {
    Number(f64),
    UnaryOp { op: char, expr: Box<Expr> },
    BinaryOp { left: Box<Expr>, op: char, right: Box<Expr> },
    Func { name: String, arg: Box<Expr> },
}

pub fn run(input: &str) {
    let expr_str = input.trim().strip_prefix("calc").unwrap_or("").trim();

    if expr_str.is_empty() {
        println!("\x1b[91mUsage: calc <expression>\x1b[0m");
        return;
    }

    match parse_expression(expr_str) {
        Ok(expr) => {
            let result = eval(&expr);
            println!("{}", result);
        }
        Err(e) => println!("\x1b[91mError: {}\x1b[0m", e),
    }
}

fn parse_expression(input: &str) -> Result<Expr, String> {
    let mut chars = input.chars().peekable();
    let expr = parse_expr(&mut chars)?;
    skip_spaces(&mut chars);

    if chars.peek().is_some() {
        return Err("Unexpected characters at end of expression".into());
    }

    Ok(expr)
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
    let mut node = parse_pow(chars)?;

    loop {
        skip_spaces(chars);
        match chars.peek().copied() {
            Some('*') | Some('/') => {
                let op = chars.next