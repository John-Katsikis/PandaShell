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
struct WeatherResponse {
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

pub fn run(input: &str) {
    let city = input.trim().replace("weather", "").trim().to_string();

    if city.is_empty() {
        eprintln!("Usage: weather <city>");
        return;
    }

    // 1. Geocode city → coordinates
    let geo_url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1",
        city
    );

    let geo_resp = blocking::get(&geo_url);
    let geo_data: GeocodeResult = match geo_resp {
        Ok(resp) => match resp.json() {
            Ok(json) => json,
            Err(_) => {
                eprintln!("Failed to parse geocoding response");
                return;
            }
        },
        Err(_) => {
            eprintln!("Failed to reach geocoding API");
            return;
        }
    };

    let Some(entry) = geo_data.results.and_then(|mut r| r.pop()) else {
        eprintln!("City not found");
        return;
    };

    // 2. Fetch weather
    let weather_url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,wind_speed_10m,relative_humidity_2m,apparent_temperature,precipitation,weather_code",
        entry.latitude, entry.longitude
    );

    let weather_resp = blocking::get(&weather_url);
    let weather: WeatherResponse = match weather_resp {
        Ok(resp) => match resp.json() {
            Ok(json) => json,
            Err(_) => {
                eprintln!("Failed to parse weather data");
                return;
            }
        },
        Err(_) => {
            eprintln!("Failed to reach weather API");
            return;
        }
    };

    // 3. Emoji + Panda personality
    let emoji = match weather.current.weather_code {
        0 => "☀️",
        1 | 2 | 3 => "⛅",
        45 | 48 => "🌫️",
        51 | 53 | 55 => "🌦️",
        61 | 63 | 65 => "🌧️",
        71 | 73 | 75 => "❄️",
        95 => "⛈️",
        _ => "🌍",
    };

    let panda_msg = match weather.current.temperature_2m {
        t if t < 0.0 => "Panda says: It's freezing, bring a coat.",
        t if t < 10.0 => "Panda says: Chilly weather, stay warm.",
        t if t < 20.0 => "Panda says: Mild weather, comfy.",
        t if t < 30.0 => "Panda says: Warm and pleasant.",
        _ => "Panda says: It's hot, stay hydrated.",
    };

    let conditions = describe_weather(weather.current.weather_code);

    // 4. Unified output block
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
        weather.current.temperature_2m,
        weather.current.apparent_temperature,
        weather.current.relative_humidity_2m,
        weather.current.wind_speed_10m,
        weather.current.precipitation,
        conditions,
        panda_msg
    );
}
