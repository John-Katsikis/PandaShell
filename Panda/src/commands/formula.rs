use crate::commands::calc::{eval, eval_with_x, parse_expression};

const GRAPH_WIDTH: usize = 72;
const GRAPH_HEIGHT: usize = 20;

pub fn run(input: &str) {
    let rest = input.trim().strip_prefix("formula").unwrap_or("").trim();

    if rest.is_empty() {
        print_usage();
        return;
    }

    let parts: Vec<&str> = rest.split_whitespace().collect();

    if parts.len() < 3 {
        print_usage();
        return;
    }

    let min = match eval_range_bound(parts[parts.len() - 2]) {
        Ok(v) => v,
        Err(e) => {
            println!("\x1b[91mInvalid minimum value: {}\x1b[0m", e);
            return;
        }
    };

    let max = match eval_range_bound(parts[parts.len() - 1]) {
        Ok(v) => v,
        Err(e) => {
            println!("\x1b[91mInvalid maximum value: {}\x1b[0m", e);
            return;
        }
    };

    if min >= max {
        println!("\x1b[91mMinimum must be less than maximum\x1b[0m");
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

    let samples = match sample_expression(&expr, min, max, GRAPH_WIDTH) {
        Ok(samples) => samples,
        Err(e) => {
            println!("\x1b[91mError evaluating formula: {}\x1b[0m", e);
            return;
        }
    };
    let finite_values: Vec<f64> = samples
        .iter()
        .filter_map(|sample| sample.y.filter(|y| y.is_finite()))
        .collect();

    if finite_values.is_empty() {
        println!("\x1b[91mNo finite values to plot in this range\x1b[0m");
        return;
    }

    let y_min = finite_values.iter().copied().fold(f64::INFINITY, f64::min);
    let y_max = finite_values
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let y_avg = finite_values.iter().sum::<f64>() / finite_values.len() as f64;

    println!("\x1b[93mformula: y = {}\x1b[0m", expr_str);
    println!(
        "x: {:.3}..{:.3} | y: {:.3}..{:.3} | avg: {:.3}",
        min, max, y_min, y_max, y_avg
    );
    println!();

    for line in render_graph(&samples, y_min, y_max) {
        println!("{}", line);
    }
}

#[derive(Debug)]
struct Sample {
    x: f64,
    y: Option<f64>,
}

fn print_usage() {
    println!("\x1b[91mUsage: formula <expression> <min> <max>\x1b[0m");
    println!("Example: formula sin(x) 0 tau");
}

fn eval_range_bound(input: &str) -> Result<f64, String> {
    parse_expression(input).and_then(|expr| eval(&expr))
}

fn sample_expression(
    expr: &crate::commands::calc::Expr,
    min: f64,
    max: f64,
    width: usize,
) -> Result<Vec<Sample>, String> {
    let last = width.saturating_sub(1);

    (0..width)
        .map(|i| {
            let t = if last == 0 {
                0.0
            } else {
                i as f64 / last as f64
            };
            let x = min + (max - min) * t;
            let y = eval_with_x(expr, x)?;

            Ok(Sample {
                x,
                y: y.is_finite().then_some(y),
            })
        })
        .collect()
}

fn render_graph(samples: &[Sample], y_min: f64, y_max: f64) -> Vec<String> {
    let mut grid = vec![vec![' '; samples.len()]; GRAPH_HEIGHT];
    let y_range = y_max - y_min;
    let zero_row = (y_min <= 0.0 && y_max >= 0.0)
        .then(|| row_for_y(0.0, y_min, y_range))
        .flatten();

    if let Some(row) = zero_row {
        for col in 0..samples.len() {
            grid[row][col] = '-';
        }
    }

    let zero_col = samples
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            a.x.abs()
                .partial_cmp(&b.x.abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .and_then(|(col, _)| {
            let first = samples.first()?.x;
            let last = samples.last()?.x;
            (first <= 0.0 && last >= 0.0).then_some(col)
        });

    if let Some(col) = zero_col {
        for row in &mut grid {
            row[col] = if row[col] == '-' { '+' } else { '|' };
        }
    }

    for (col, sample) in samples.iter().enumerate() {
        if let Some(y) = sample.y {
            if let Some(row) = row_for_y(y, y_min, y_range) {
                grid[row][col] = '*';
            }
        }
    }

    grid.into_iter()
        .enumerate()
        .map(|(row, cells)| {
            let label = y_label_for_row(row, y_min, y_max);
            format!(
                "{:>10.3} | {}",
                label,
                cells.into_iter().collect::<String>()
            )
        })
        .collect()
}

fn row_for_y(y: f64, y_min: f64, y_range: f64) -> Option<usize> {
    if y_range == 0.0 {
        return Some(GRAPH_HEIGHT / 2);
    }

    if y < y_min || y > y_min + y_range {
        return None;
    }

    let normalized = (y - y_min) / y_range;
    let row = (GRAPH_HEIGHT - 1) as f64 - normalized * (GRAPH_HEIGHT - 1) as f64;
    Some(row.round() as usize)
}

fn y_label_for_row(row: usize, y_min: f64, y_max: f64) -> f64 {
    if GRAPH_HEIGHT <= 1 {
        return y_min;
    }

    let t = row as f64 / (GRAPH_HEIGHT - 1) as f64;
    y_max - (y_max - y_min) * t
}
