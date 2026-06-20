use reqwest::blocking;
use serde::Deserialize;

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
    let geo_url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1",
        city
    );

    let geo_resp = blocking::get(&geo_url).ok()?;
    let geo_data: GeocodeResult = geo_resp.json().ok()?;
    geo_data.results.and_then(|mut r| r.pop())
}

fn fetch_current(entry: &GeoEntry) -> Option<WeatherResponseCurrent> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,wind_speed_10m,relative_humidity_2m,apparent_temperature,precipitation,weather_code",
        entry.latitude, entry.longitude
    );
    let resp = blocking::get(&url).ok()?;
    resp.json().ok()
}

fn fetch_hourly(entry: &GeoEntry) -> Option<WeatherResponseHourly> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m,precipitation,weather_code&forecast_days=1&timezone=auto",
        entry.latitude, entry.longitude
    );
    let resp = blocking::get(&url).ok()?;
    resp.json().ok()
}

fn fetch_daily(entry: &GeoEntry, days: usize) -> Option<WeatherResponseDaily> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&daily=temperature_2m_max,temperature_2m_min,precipitation_sum,weather_code&forecast_days={}&timezone=auto",
        entry.latitude, entry.longitude, days
    );

    let resp = blocking::get(&url).ok()?;
    resp.json().ok()
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
    println!("\x1b[93mHourly forecast for {} ({})\x1b[0m", entry.name, entry.country);
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
    println!("\x1b[93mAlerts for {} ({})\x1b[0m", entry.name, entry.country);
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
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.is_empty() || parts[0] != "weather" {
        eprintln!("Usage: weather <city> [--hourly|--tomorrow|--days N|--alerts]");
        return;
    }

    if parts.len() < 2 {
        eprintln!("Usage: weather <city> [--hourly|--tomorrow|--days N|--alerts]");
        return;
    }

    let city = parts[1];
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

    let Some(entry) = geocode_city(city) else {
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
