pub fn run(input: &str) {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.is_empty() || parts[0] != "weather" {
        eprintln!("Usage: weather <city> [--hourly|--tomorrow|--days N|--alerts]");
        return;
    }

    if parts.len() < 2 {
        eprintln!("Usage: weather <city> [--hourly|--tomorrow|--days N|--alerts]");
        return;
    }

    // FIX: join all words until a flag appears
    let city = parts[1..]
        .iter()
        .take_while(|&&p| !p.starts_with("--"))
        .cloned()
        .collect::<Vec<&str>>()
        .join(" ");

    let mut flag: Option<&str> = None;
    let mut days: usize = 3;

    let mut i = 2;
    while i < parts.len() {
        match parts[i] {
            "--hourly" | "--tomorrow" | "--alerts" => {
                flag = Some(parts[i]);
            }
            "--days" => {
                if i + 1 < parts.len() {
                    if let Ok(n) = parts[i + 1].parse::<usize>() {
                        days = n;
                        flag = Some("--days");
                        i += 1;
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    let Some(entry) = geocode_city(&city) else {
        eprintln!("City not found");
        return;
    };

    match flag {
        Some("--hourly") => {
            if let Some(hourly) = fetch_hourly(&entry) {
                print_hourly(&entry, &hourly);
            } else {
                eprintln!("Failed to fetch hourly forecast");
            }
        }
        Some("--tomorrow") => {
            if let Some(daily) = fetch_daily(&entry, 2) {
                print_tomorrow(&entry, &daily);
            } else {
                eprintln!("Failed to fetch daily forecast for tomorrow");
            }
        }
        Some("--days") => {
            if let Some(daily) = fetch_daily(&entry, days) {
                print_daily(&entry, &daily);
            } else {
                eprintln!("Failed to fetch multi-day forecast");
            }
        }
        Some("--alerts") => {
            if let Some(daily) = fetch_daily(&entry, 7) {
                print_alerts(&entry, &daily);
            } else {
                eprintln!("Failed to fetch forecast for alerts");
            }
        }
        None => {
            if let Some(current) = fetch_current(&entry) {
                print_current(&entry, &current);
            } else {
                eprintln!("Failed to fetch current weather");
            }
        }
        Some(_) => {
            eprintln!("Unknown flag. Usage: weather <city> [--hourly|--tomorrow|--days N|--alerts]");
        }
    }
}
