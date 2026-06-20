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
    parse_expr(&mut chars)
}

fn parse_expr(chars: &mut Peekable<Chars>) -> Result<Expr, String> {
    parse_add_sub(chars)
}

fn parse_add_sub(chars: &mut Peekable<Chars>) -> Result<Expr, String> {
    let mut node = parse_mul_div(chars)?;

    while let Some(&c) = chars.peek() {
        if c == '+' || c == '-' {
            chars.next();
            let right = parse_mul_div(chars)?;
            node = Expr::BinaryOp {
                left: Box::new(node),
                op: c,
                right: Box::new(right),
            };
        } else {
            break;
        }
    }

    Ok(node)
}

fn parse_mul_div(chars: &mut Peekable<Chars>) -> Result<Expr, String> {
    let mut node = parse_pow(chars)?;

    while let Some(&c) = chars.peek() {
        if c == '*' || c == '/' {
            chars.next();
            let right = parse_pow(chars)?;
            node = Expr::BinaryOp {
                left: Box::new(node),
                op: c,
                right: Box::new(right),
            };
        } else {
            break;
        }
    }

    Ok(node)
}

fn parse_pow(chars: &mut Peekable<Chars>) -> Result<Expr, String> {
    let mut node = parse_unary(chars)?;

    while let Some(&c) = chars.peek() {
        if c == '^' {
            chars.next();
            let right = parse_unary(chars)?;
            node = Expr::BinaryOp {
                left: Box::new(node),
                op: '^',
                right: Box::new(right),
            };
        } else {
            break;
        }
    }

    Ok(node)
}

fn parse_unary(chars: &mut Peekable<Chars>) -> Result<Expr, String> {
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
        if c.is_ascii_digit() {
            return parse_number(chars);
        }

        if c.is_ascii_alphabetic() {
            return parse_function(chars);
        }

        if c == '(' {
            chars.next();
            let expr = parse_expr(chars)?;
            if chars.next() != Some(')') {
                return Err("Missing ')'".into());
            }
            return Ok(expr);
        }
    }

    Err("Unexpected character".into())
}

fn parse_number(chars: &mut Peekable<Chars>) -> Result<Expr, String> {
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

fn parse_function(chars: &mut Peekable<Chars>) -> Result<Expr, String> {
    let mut name = String::new();

    while let Some(&c) = chars.peek() {
        if c.is_ascii_alphabetic() {
            name.push(c);
            chars.next();
        } else {
            break;
        }
    }

    if chars.next() != Some('(') {
        return Err("Expected '(' after function name".into());
    }

    let arg = parse_expr(chars)?;

    if chars.next() != Some(')') {
        return Err("Missing ')' after function argument".into());
    }

    Ok(Expr::Func {
        name,
        arg: Box::new(arg),
    })
}

fn skip_spaces(chars: &mut Peekable<Chars>) {
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else {
            break;
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
