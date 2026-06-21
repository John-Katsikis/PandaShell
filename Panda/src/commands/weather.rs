use crate::parser;
use reqwest::blocking;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WeatherMode {
    Current,
    Hourly,
    Tomorrow,
    Days,
    Alerts,
}

#[derive(Debug, PartialEq, Eq)]
struct WeatherOptions {
    city: String,
    mode: WeatherMode,
    days: usize,
}

#[derive(Deserialize)]
struct GeocodeResult {
    results: Option<Vec<GeoEntry>>,
}

#[derive(Deserialize)]
struct GeoEntry {
    latitude: f64,
    longitude: f64,
    name: String,
    country: String,
}

#[derive(Deserialize)]
struct ApiError {
    reason: Option<String>,
    error: Option<bool>,
}

#[derive(Deserialize)]
struct WeatherResponseCurrent {
    current: CurrentWeather,
}

#[derive(Deserialize)]
struct CurrentWeather {
    temperature_2m: f64,
    wind_speed_10m: f64,
    relative_humidity_2m: f64,
    apparent_temperature: f64,
    precipitation: f64,
    weather_code: i32,
}

#[derive(Deserialize)]
struct WeatherResponseHourly {
    hourly: HourlyData,
    hourly_units: HourlyUnits,
}

#[derive(Deserialize)]
struct HourlyData {
    time: Vec<String>,
    temperature_2m: Vec<f64>,
    precipitation: Vec<f64>,
    weather_code: Vec<i32>,
}

#[derive(Deserialize)]
struct HourlyUnits {
    temperature_2m: String,
    precipitation: String,
}

#[derive(Deserialize)]
struct WeatherResponseDaily {
    daily: DailyData,
    daily_units: DailyUnits,
}

#[derive(Deserialize)]
struct DailyData {
    time: Vec<String>,
    temperature_2m_max: Vec<f64>,
    temperature_2m_min: Vec<f64>,
    precipitation_sum: Vec<f64>,
    weather_code: Vec<i32>,
}

#[derive(Deserialize)]
struct DailyUnits {
    temperature_2m_max: String,
    temperature_2m_min: String,
    precipitation_sum: String,
}

fn describe_weather(code: i32) -> &'static str {
    match code {
        0 => "Clear sky",
        1 | 2 | 3 => "Partly cloudy",
        45 | 48 => "Fog",
        51 | 53 | 55 => "Drizzle",
        61 | 63 | 65 => "Rain",
        71 | 73 | 75 => "Snow",
        95 => "Thunderstorm",
        _ => "Unknown",
    }
}

fn emoji_for_code(code: i32) -> &'static str {
    match code {
        0 => "☀️",
        1 | 2 | 3 => "⛅",
        45 | 48 => "🌫️",
        51 | 53 | 55 => "🌦️",
        61 | 63 | 65 => "🌧️",
        71 | 73 | 75 => "❄️",
        95 => "⛈️",
        _ => "🌍",
    }
}

fn panda_msg_for_temp(t: f64) -> &'static str {
    if t < 0.0 {
        "Panda says: It's freezing, bring a coat."
    } else if t < 10.0 {
        "Panda says: Chilly weather, stay warm."
    } else if t < 20.0 {
        "Panda says: Mild weather, comfy."
    } else if t < 30.0 {
        "Panda says: Warm and pleasant."
    } else {
        "Panda says: It's hot, stay hydrated."
    }
}

fn geocode_city(city: &str) -> Option<GeoEntry> {
    let city = encode_query_component(city);
    let geo_url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1",
        city
    );

    let geo_resp = blocking::get(&geo_url).ok()?;
    let geo_data: GeocodeResult = geo_resp.json().ok()?;
    geo_data.results.and_then(|mut r| r.pop())
}

fn fetch_current(entry: &GeoEntry) -> Result<WeatherResponseCurrent, String> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,wind_speed_10m,relative_humidity_2m,apparent_temperature,precipitation,weather_code",
        entry.latitude, entry.longitude
    );
    fetch_json(&url)
}

fn fetch_hourly(entry: &GeoEntry) -> Result<WeatherResponseHourly, String> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m,precipitation,weather_code&forecast_days=1",
        entry.latitude, entry.longitude
    );
    fetch_json(&url)
}

fn fetch_daily(entry: &GeoEntry, days: usize) -> Result<WeatherResponseDaily, String> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&daily=temperature_2m_max,temperature_2m_min,precipitation_sum,weather_code&forecast_days={}",
        entry.latitude, entry.longitude, days
    );
    fetch_json(&url)
}

fn fetch_json<T: for<'de> Deserialize<'de>>(url: &str) -> Result<T, String> {
    let resp = blocking::get(url).map_err(|e| format!("Network error: {e}"))?;
    let status = resp.status();
    let body = resp
        .text()
        .map_err(|e| format!("Failed to read response: {e}"))?;

    if !status.is_success() {
        return Err(format!("HTTP {status}: {body}"));
    }

    if let Ok(api_error) = serde_json::from_str::<ApiError>(&body) {
        if api_error.error.unwrap_or(false) {
            return Err(api_error
                .reason
                .unwrap_or_else(|| "Weather API returned an error".into()));
        }
    }

    serde_json::from_str(&body).map_err(|e| format!("Could not parse weather response: {e}"))
}

fn print_current(entry: &GeoEntry, weather: &WeatherResponseCurrent) {
    let c = &weather.current;
    let emoji = emoji_for_code(c.weather_code);
    let panda_msg = panda_msg_for_temp(c.temperature_2m);
    let conditions = describe_weather(c.weather_code);

    println!(
        "\x1b[93m{} ({})\x1b[0m {}\n\
Temperature: {:.1}°C (Feels like {:.1}°C)\n\
Humidity: {:.0}%\n\
Wind: {:.1} km/h\n\
Precipitation: {:.1} mm\n\
Conditions: {}\n\
{}\n",
        entry.name,
        entry.country,
        emoji,
        c.temperature_2m,
        c.apparent_temperature,
        c.relative_humidity_2m,
        c.wind_speed_10m,
        c.precipitation,
        conditions,
        panda_msg
    );
}

fn print_hourly(entry: &GeoEntry, hourly: &WeatherResponseHourly) {
    println!(
        "\x1b[93mHourly forecast for {} ({})\x1b[0m",
        entry.name, entry.country
    );
    for i in 0..hourly.hourly.time.len().min(12) {
        let time = &hourly.hourly.time[i];
        let temp = hourly.hourly.temperature_2m[i];
        let precip = hourly.hourly.precipitation[i];
        let code = hourly.hourly.weather_code[i];
        let emoji = emoji_for_code(code);
        let cond = describe_weather(code);
        let panda = panda_msg_for_temp(temp);

        println!(
            "{} {}: {:.1}{} | Precip: {:.1}{} | {} | {}",
            emoji,
            time,
            temp,
            hourly.hourly_units.temperature_2m,
            precip,
            hourly.hourly_units.precipitation,
            cond,
            panda
        );
    }
    println!();
}

fn print_daily(entry: &GeoEntry, daily: &WeatherResponseDaily) {
    println!(
        "\x1b[93mMulti-day forecast for {} ({})\x1b[0m",
        entry.name, entry.country
    );
    for i in 0..daily.daily.time.len() {
        let date = &daily.daily.time[i];
        let tmax = daily.daily.temperature_2m_max[i];
        let tmin = daily.daily.temperature_2m_min[i];
        let precip = daily.daily.precipitation_sum[i];
        let code = daily.daily.weather_code[i];
        let emoji = emoji_for_code(code);
        let cond = describe_weather(code);
        let panda = panda_msg_for_temp(tmax);

        println!(
            "{} {}:\n  Max: {:.1}{}\n  Min: {:.1}{}\n  Precipitation: {:.1}{}\n  Conditions: {}\n  {}\n",
            emoji,
            date,
            tmax,
            daily.daily_units.temperature_2m_max,
            tmin,
            daily.daily_units.temperature_2m_min,
            precip,
            daily.daily_units.precipitation_sum,
            cond,
            panda
        );
    }
}

fn print_tomorrow(entry: &GeoEntry, daily: &WeatherResponseDaily) {
    if daily.daily.time.len() < 2 {
        println!("Not enough forecast data for tomorrow.");
        return;
    }
    let i = 1;
    let date = &daily.daily.time[i];
    let tmax = daily.daily.temperature_2m_max[i];
    let tmin = daily.daily.temperature_2m_min[i];
    let precip = daily.daily.precipitation_sum[i];
    let code = daily.daily.weather_code[i];
    let emoji = emoji_for_code(code);
    let cond = describe_weather(code);
    let panda = panda_msg_for_temp(tmax);

    println!(
        "\x1b[93mTomorrow in {} ({})\x1b[0m {}\n\
Date: {}\n\
Max: {:.1}{}\n\
Min: {:.1}{}\n\
Precipitation: {:.1}{}\n\
Conditions: {}\n\
{}\n",
        entry.name,
        entry.country,
        emoji,
        date,
        tmax,
        daily.daily_units.temperature_2m_max,
        tmin,
        daily.daily_units.temperature_2m_min,
        precip,
        daily.daily_units.precipitation_sum,
        cond,
        panda
    );
}

fn print_alerts(entry: &GeoEntry, daily: &WeatherResponseDaily) {
    println!(
        "\x1b[93mAlerts for {} ({})\x1b[0m",
        entry.name, entry.country
    );
    let mut any = false;

    for i in 0..daily.daily.time.len() {
        let date = &daily.daily.time[i];
        let tmax = daily.daily.temperature_2m_max[i];
        let precip = daily.daily.precipitation_sum[i];
        let code = daily.daily.weather_code[i];

        let mut alerts = Vec::new();

        if tmax >= 30.0 {
            alerts.push("Heat alert: very warm day.");
        }
        if tmax <= 0.0 {
            alerts.push("Cold alert: freezing temperatures.");
        }
        if precip >= 5.0 {
            alerts.push("Rain alert: heavy precipitation expected.");
        }
        if code == 95 {
            alerts.push("Storm alert: thunderstorm conditions.");
        }

        if !alerts.is_empty() {
            any = true;
            println!("{} {}:", emoji_for_code(code), date);
            for a in alerts {
                println!("  - {}", a);
            }
        }
    }

    if !any {
        println!("No significant alerts based on forecast data.\n");
    } else {
        println!();
    }
}

pub fn run(input: &str) {
    let options = match parse_weather_options(input) {
        Ok(options) => options,
        Err(e) => {
            eprintln!("\x1b[91m{}\x1b[0m", e);
            print_usage();
            return;
        }
    };

    let Some(entry) = geocode_city(&options.city) else {
        eprintln!("City not found");
        return;
    };

    match options.mode {
        WeatherMode::Hourly => {
            fetch_hourly(&entry)
                .map(|hourly| print_hourly(&entry, &hourly))
                .unwrap_or_else(|e| eprintln!("Failed to fetch hourly forecast: {e}"));
        }
        WeatherMode::Tomorrow => {
            fetch_daily(&entry, 2)
                .map(|daily| print_tomorrow(&entry, &daily))
                .unwrap_or_else(|e| eprintln!("Failed to fetch daily forecast for tomorrow: {e}"));
        }
        WeatherMode::Days => {
            fetch_daily(&entry, options.days)
                .map(|daily| print_daily(&entry, &daily))
                .unwrap_or_else(|e| eprintln!("Failed to fetch multi-day forecast: {e}"));
        }
        WeatherMode::Alerts => {
            fetch_daily(&entry, 7)
                .map(|daily| print_alerts(&entry, &daily))
                .unwrap_or_else(|e| eprintln!("Failed to fetch forecast for alerts: {e}"));
        }
        WeatherMode::Current => {
            fetch_current(&entry)
                .map(|current| print_current(&entry, &current))
                .unwrap_or_else(|e| eprintln!("Failed to fetch current weather: {e}"));
        }
    }
}

fn parse_weather_options(input: &str) -> Result<WeatherOptions, String> {
    let ast = parser::parse_line(input)?;
    let Some(command) = ast.commands.first() else {
        return Err("Missing weather command".into());
    };

    if command.name != "weather" {
        return Err("Expected weather command".into());
    }

    parse_weather_args(&command.args)
}

fn parse_weather_args(args: &[String]) -> Result<WeatherOptions, String> {
    let mut city_parts = Vec::new();
    let mut mode = WeatherMode::Current;
    let mut days = 3usize;
    let mut i = 0usize;

    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => return Err("Usage requested".into()),
            "--hourly" => set_mode(&mut mode, WeatherMode::Hourly)?,
            "--tomorrow" => set_mode(&mut mode, WeatherMode::Tomorrow)?,
            "--alerts" => set_mode(&mut mode, WeatherMode::Alerts)?,
            "--days" | "-d" => {
                set_mode(&mut mode, WeatherMode::Days)?;
                i += 1;
                let Some(value) = args.get(i) else {
                    return Err("Missing number after --days".into());
                };
                days = value
                    .parse::<usize>()
                    .map_err(|_| format!("Invalid day count '{}'", value))?;
                if !(1..=16).contains(&days) {
                    return Err("Day count must be between 1 and 16".into());
                }
            }
            option if option.starts_with('-') => {
                return Err(format!("Unknown weather option '{}'", option));
            }
            city_part => city_parts.push(city_part.to_string()),
        }

        i += 1;
    }

    if city_parts.is_empty() {
        return Err("Missing city. Try: weather Athens --days 3".into());
    }

    Ok(WeatherOptions {
        city: city_parts.join(" "),
        mode,
        days,
    })
}

fn set_mode(current: &mut WeatherMode, next: WeatherMode) -> Result<(), String> {
    if *current != WeatherMode::Current && *current != next {
        return Err("Use only one weather mode: --hourly, --tomorrow, --days, or --alerts".into());
    }

    *current = next;
    Ok(())
}

fn print_usage() {
    eprintln!("Usage: weather <city> [--hourly|--tomorrow|--days N|--alerts]");
    eprintln!("Examples:");
    eprintln!("  weather Athens");
    eprintln!("  weather \"New York\" --days 3");
    eprintln!("  weather --hourly London");
    eprintln!("  weather Tokyo --alerts");
}

fn encode_query_component(input: &str) -> String {
    input
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            b' ' => vec!['+'],
            _ => {
                let hex = format!("%{:02X}", byte);
                hex.chars().collect()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{encode_query_component, parse_weather_args, parse_weather_options, WeatherMode};

    #[test]
    fn parses_quoted_city_and_days() {
        let options = parse_weather_options("weather \"New York\" --days 3").unwrap();

        assert_eq!(options.city, "New York");
        assert_eq!(options.mode, WeatherMode::Days);
        assert_eq!(options.days, 3);
    }

    #[test]
    fn parses_flag_before_city() {
        let args = ["--hourly".to_string(), "Athens".to_string()];
        let options = parse_weather_args(&args).unwrap();

        assert_eq!(options.city, "Athens");
        assert_eq!(options.mode, WeatherMode::Hourly);
    }

    #[test]
    fn rejects_conflicting_modes() {
        let args = [
            "Athens".to_string(),
            "--hourly".to_string(),
            "--alerts".to_string(),
        ];

        assert!(parse_weather_args(&args).is_err());
    }

    #[test]
    fn encodes_city_for_query() {
        assert_eq!(encode_query_component("New York"), "New+York");
    }

    #[test]
    fn parses_tomorrow_mode() {
        let options = parse_weather_options("weather Athens --tomorrow").unwrap();

        assert_eq!(options.city, "Athens");
        assert_eq!(options.mode, WeatherMode::Tomorrow);
    }
}
