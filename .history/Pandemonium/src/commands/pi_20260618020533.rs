pub fn run(input: &str) {
    let parts: Vec<&str> = input.split_whitespace().collect();

    if parts.len() != 2 {
        println!("Usage: pi <digits>");
        return;
    }

    let digits: u32 = match parts[1].parse() {
        Ok(n) => n,
        Err(_) => {
            println!("Please enter a valid number of digits.");
            return;
        }
    };

    let precision_bits = ((digits as f64) * 3.321928094887362 + 16.0) as u32;

    let pi = Float::with_val(precision_bits, Constant::Pi);
    let pi_string = pi.to_string_radix(10, Some(digits as usize + 2));

    println!("{}", pi_string);
}
