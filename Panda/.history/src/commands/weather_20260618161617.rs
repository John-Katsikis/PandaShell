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
}

pub fn run(args: &str) {
    let city = args.trim().replace("weather", "").trim().to_string();

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
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,wind_speed_10m",
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

    // 3. Print summary
    println!(
        "\x1b[93m{} ({})\x1b[0m\nTemperature: {:.1}°C\nWind: {:.1} km/h",
        entry.name,
        entry.country,
        weather.current.temperature_2m,
        weather.current.wind_speed_10m
    );
}
