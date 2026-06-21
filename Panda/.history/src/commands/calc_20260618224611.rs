use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone)]
pub enum Expr {
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
        Ok(expr) => println!("{}", eval(&expr)),
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
                let op = chars.next().unwrap();
                let right = parse_pow(chars)?;
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

fn parse_pow(chars: &mut Peekable<Chars>) -> Result<Expr, String> {
    let mut node = parse_unary(chars)?;

    loop {
        skip_spaces(chars);
        match chars.peek().copied() {
            Some('^') => {
                chars.next();
                let right = parse_unary(chars)?;
                node = Expr::BinaryOp {
                    left: Box::new(node),
                    op: '^',
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

    if let Some(&c) = chars.peek() {
        if c == '-' {
            chars.next();
            let expr = parse_unary(chars)?;
            return Ok(Expr::UnaryOp {
                op: '-',
                expr: Box::new(expr),
            });
        }
    }

    parse_primary(chars)
}

fn parse_primary(chars: &mut Peekable<Chars>) -> Result<Expr, String> {
    skip_spaces(chars);

    if let Some(&c) = chars.peek() {
        // variable x
        if c == 'x' {
            chars.next();
            return Ok(Expr::Func {
                name: "x".into(),
                arg: Box::new(Expr::Number(0.0)),
            });
        }

        // number
        if c.is_ascii_digit() {
            return parse_number(chars);
        }

        // identifier (function or implicit call)
        if c.is_ascii_alphabetic() {
            return parse_identifier(chars);
        }

        // parentheses
        if c == '(' {
            chars.next();
            skip_spaces(chars);

            let expr = parse_expr(chars)?;
            skip_spaces(chars);

            if chars.next() != Some(')') {
                return Err("Missing ')'".into());
            }

            return Ok(expr);
        }
    }

    Err("Unexpected character".into())
}

fn parse_identifier(chars: &mut Peekable<Chars>) -> Result<Expr, String> {
    skip_spaces(chars);

    // read identifier name
    let mut name = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_alphabetic() {
            name.push(c);
            chars.next();
        } else {
            break;
        }
    }

    skip_spaces(chars);

    // variable x
    if name == "x" {
        return Ok(Expr::Func {
            name: "x".into(),
            arg: Box::new(Expr::Number(0.0)),
        });
    }

    // function with parentheses: sin(x)
    if chars.peek() == Some(&'(') {
        chars.next(); // consume '('
        skip_spaces(chars);

        let arg = parse_expr(chars)?;
        skip_spaces(chars);

        if chars.next() != Some(')') {
            return Err("Missing ')' after function argument".into());
        }

        return Ok(Expr::Func {
            name,
            arg: Box::new(arg),
        });
    }

    // implicit function call: sin x
    let arg = parse_primary(chars)?;

    Ok(Expr::Func {
        name,
        arg: Box::new(arg),
    })
}

fn parse_number(chars: &mut Peekable<Chars>) -> Result<Expr, String> {
    skip_spaces(chars);

    let mut num = String::new();

    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() || c == '.' {
            num.push(c);
            chars.next();
        } else {
            break;
        }
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

pub fn eval_with_x(expr: &Expr, x: f64) -> f64 {
    match expr {
        Expr::Number(n) => *n,

        Expr::UnaryOp { op: '-', expr } => -eval_with_x(expr, x),
        Expr::UnaryOp { op, .. } => panic!("Unsupported unary operator '{}'", op),

        Expr::BinaryOp { left, op, right } => {
            let l = eval_with_x(left, x);
            let r = eval_with_x(right, x);
            match op {
                '+' => l + r,
                '-' => l - r,
                '*' => l * r,
                '/' => l / r,
                '^' => l.powf(r),
                _ => panic!("Unknown operator"),
            }
        }

        Expr::Func { name, arg } => {
            let v = eval_with_x(arg, x);
            match name.as_str() {
                "x" => x,
                "sqrt" => v.sqrt(),
                "sin" => v.sin(),
                "cos" => v.cos(),
                "tan" => v.tan(),
                _ => panic!("Unknown function"),
            }
        }
    }
}

fn eval(expr: &Expr) -> f64 {
    match expr {
        Expr::Number(n) => *n,

        Expr::UnaryOp { op: '-', expr } => -eval(expr),
        Expr::UnaryOp { op, .. } => panic!("Unsupported unary operator '{}'", op),

        Expr::BinaryOp { left, op, right } => {
            let l = eval(left);
            let r = eval(right);
            match op {
                '+' => l + r,
                '-' => l - r,
                '*' => l * r,
                '/' => l / r,
                '^' => l.powf(r),
                _ => panic!("Unknown operator"),
            }
        }

        Expr::Func { name, arg } => {
            let v = eval(arg);
            match name.as_str() {
                "sqrt" => v.sqrt(),
                "sin" => v.sin(),
                "cos" => v.cos(),
                "tan" => v.tan(),
                _ => panic!("Unknown function"),
            }
        }
    }
}
