use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;

// Constants
const USER_AGENT: &str = "waybar-weather-cli/2.0 (github.com/user/weather-cli)";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_MS: u64 = 1000;

// Temperature conversion
const CELSIUS_TO_FAHRENHEIT_MULTIPLIER: f64 = 9.0 / 5.0;
const CELSIUS_TO_FAHRENHEIT_OFFSET: f64 = 32.0;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Number parsing error: {0}")]
    Parse(#[from] std::num::ParseFloatError),
    #[error("Invalid ZIP code: {0}")]
    InvalidZip(String),
    #[error("Location not found")]
    LocationNotFound,
    #[error("No weather data available")]
    NoWeatherData,
    #[error("API error: {0}")]
    Api(String),
    #[error("Invalid coordinates: lat must be between -90 and 90, lon between -180 and 180")]
    InvalidCoordinates,
}

#[derive(Parser)]
#[command(author, version, about = "Get weather information for Waybar")]
struct Args {
    /// ZIP code, e.g. 90210
    #[arg(long, conflicts_with_all=&["lat", "lon"])]
    zip: Option<String>,

    /// Latitude, e.g. 37.9
    #[arg(long, requires = "lon")]
    lat: Option<f64>,

    /// Longitude, e.g. -122.3
    #[arg(long, requires = "lat")]
    lon: Option<f64>,

    /// Temperature unit (F or C)
    #[arg(long, default_value = "F", value_parser = parse_unit)]
    unit: TemperatureUnit,

    /// Icon set to use
    #[arg(long, default_value = "nerdfont", value_parser = parse_icon_set)]
    icons: IconSet,

    /// Include additional weather details in tooltip
    #[arg(long)]
    detailed: bool,

    /// Output format
    #[arg(long, default_value = "waybar", value_parser = parse_output_format)]
    format: OutputFormat,
}

#[derive(Clone, Debug)]
enum TemperatureUnit {
    Fahrenheit,
    Celsius,
}

#[derive(Clone, Debug)]
enum IconSet {
    Unicode,
    Emoji,
    Text,
    NerdFont,
}

#[derive(Clone, Debug)]
enum OutputFormat {
    Waybar,
    Plain,
    Json,
}

fn parse_unit(s: &str) -> Result<TemperatureUnit, String> {
    match s.to_uppercase().as_str() {
        "F" | "FAHRENHEIT" => Ok(TemperatureUnit::Fahrenheit),
        "C" | "CELSIUS" => Ok(TemperatureUnit::Celsius),
        _ => Err(format!("Invalid unit: {}. Use F or C", s)),
    }
}

fn parse_icon_set(s: &str) -> Result<IconSet, String> {
    match s.to_lowercase().as_str() {
        "unicode" => Ok(IconSet::Unicode),
        "emoji" => Ok(IconSet::Emoji),
        "text" => Ok(IconSet::Text),
        "nerdfont" | "nerd" => Ok(IconSet::NerdFont),
        _ => Err(format!("Invalid icon set: {}. Use unicode, emoji, text, or nerdfont", s)),
    }
}

fn parse_output_format(s: &str) -> Result<OutputFormat, String> {
    match s.to_lowercase().as_str() {
        "waybar" => Ok(OutputFormat::Waybar),
        "plain" => Ok(OutputFormat::Plain),
        "json" => Ok(OutputFormat::Json),
        _ => Err(format!("Invalid format: {}. Use waybar, plain, or json", s)),
    }
}

#[derive(Deserialize)]
struct ZippopotamPlace {
    #[serde(rename = "place name")]
    place_name: String,
    longitude: String,
    latitude: String,
}

#[derive(Deserialize)]
struct ZippopotamResponse {
    places: Vec<ZippopotamPlace>,
}

#[derive(Deserialize)]
struct NWSPointProperties {
    #[serde(rename = "gridId")]
    grid_id: String,
    #[serde(rename = "gridX")]
    grid_x: i64,
    #[serde(rename = "gridY")]
    grid_y: i64,
    #[serde(rename = "observationStations")]
    observation_stations: String,
}

#[derive(Deserialize)]
struct NWSPointResponse {
    properties: NWSPointProperties,
}

#[derive(Deserialize)]
struct ForecastPeriod {
    temperature: i64,
    #[serde(rename = "temperatureUnit")]
    temperature_unit: String,
    #[serde(rename = "shortForecast")]
    short_forecast: String,
}

#[derive(Deserialize)]
struct ForecastProperties {
    periods: Vec<ForecastPeriod>,
}

#[derive(Deserialize)]
struct ForecastResponse {
    properties: ForecastProperties,
}

#[derive(Deserialize)]
struct StationFeature {
    properties: StationProperties,
}

#[derive(Deserialize)]
struct StationProperties {
    #[serde(rename = "stationIdentifier")]
    station_identifier: String,
}

#[derive(Deserialize)]
struct StationsResponse {
    features: Vec<StationFeature>,
}

#[derive(Deserialize)]
struct ObservationProperties {
    temperature: ObservationValue<f64>,
    #[serde(rename = "relativeHumidity")]
    relative_humidity: Option<ObservationValue<f64>>,
    #[serde(rename = "windSpeed")]
    wind_speed: Option<ObservationValue<f64>>,
    #[serde(rename = "windDirection")]
    wind_direction: Option<ObservationValue<f64>>,
}

#[derive(Deserialize)]
struct ObservationValue<T> {
    value: Option<T>,
}

#[derive(Deserialize)]
struct ObservationResponse {
    properties: ObservationProperties,
}

#[derive(Debug)]
struct Location {
    lat: f64,
    lon: f64,
    name: String,
}

#[derive(Debug)]
struct WeatherData {
    temperature: i64,
    condition: String,
    humidity: Option<f64>,
    wind_speed: Option<f64>,
    wind_direction: Option<f64>,
}

#[derive(Serialize)]
struct WaybarOutput {
    text: String,
    tooltip: String,
    class: String,
}

struct WeatherClient {
    client: Client,
}

impl WeatherClient {
    fn new() -> Self {
        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client }
    }

    async fn get_with_retry<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T, WeatherError> {
        let mut last_error = None;
        
        for attempt in 1..=MAX_RETRIES {
            match self.client.get(url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<T>().await {
                            Ok(data) => return Ok(data),
                            Err(e) => last_error = Some(WeatherError::Network(e)),
                        }
                    } else {
                        last_error = Some(WeatherError::Api(format!(
                            "HTTP {}: {}", 
                            response.status(), 
                            response.text().await.unwrap_or_default()
                        )));
                    }
                }
                Err(e) => last_error = Some(WeatherError::Network(e)),
            }
            
            if attempt < MAX_RETRIES {
                sleep(Duration::from_millis(RETRY_DELAY_MS * attempt as u64)).await;
            }
        }
        
        Err(last_error.unwrap_or(WeatherError::Api("Unknown error".to_string())))
    }

    async fn resolve_location(&self, zip: Option<String>, lat: Option<f64>, lon: Option<f64>) -> Result<Location, WeatherError> {
        if let Some(zip) = zip {
            self.resolve_zip_location(&zip).await
        } else if let (Some(lat), Some(lon)) = (lat, lon) {
            self.validate_coordinates(lat, lon)?;
            Ok(Location {
                lat,
                lon,
                name: format!("Coordinates ({:.2}, {:.2})", lat, lon),
            })
        } else {
            Err(WeatherError::LocationNotFound)
        }
    }

    async fn resolve_zip_location(&self, zip: &str) -> Result<Location, WeatherError> {
        if !zip.chars().all(|c| c.is_ascii_digit()) || zip.len() != 5 {
            return Err(WeatherError::InvalidZip(zip.to_string()));
        }

        let url = format!("https://api.zippopotam.us/us/{}", zip);
        let response: ZippopotamResponse = self.get_with_retry(&url).await?;
        
        let place = response.places.first()
            .ok_or(WeatherError::LocationNotFound)?;
        
        let lat = place.latitude.parse()?;
        let lon = place.longitude.parse()?;
        
        Ok(Location {
            lat,
            lon,
            name: place.place_name.clone(),
        })
    }

    fn validate_coordinates(&self, lat: f64, lon: f64) -> Result<(), WeatherError> {
        if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
            return Err(WeatherError::InvalidCoordinates);
        }
        Ok(())
    }

    async fn get_weather_data(&self, location: &Location) -> Result<WeatherData, WeatherError> {
        // Get NWS grid info and forecast concurrently
        let point_url = format!("https://api.weather.gov/points/{},{}", location.lat, location.lon);
        let nws_point: NWSPointResponse = self.get_with_retry(&point_url).await?;
        
        let forecast_url = format!(
            "https://api.weather.gov/gridpoints/{}/{},{}/forecast",
            nws_point.properties.grid_id,
            nws_point.properties.grid_x,
            nws_point.properties.grid_y
        );

        // Get forecast and stations info concurrently
        let (forecast_result, stations_result) = tokio::join!(
            self.get_with_retry::<ForecastResponse>(&forecast_url),
            self.get_with_retry::<StationsResponse>(&nws_point.properties.observation_stations)
        );

        let forecast = forecast_result?;
        let stations = stations_result?;

        let first_period = forecast.properties.periods.first()
            .ok_or(WeatherError::NoWeatherData)?;

        let mut weather_data = WeatherData {
            temperature: first_period.temperature,
            condition: first_period.short_forecast.clone(),
            humidity: None,
            wind_speed: None,
            wind_direction: None,
        };

        // Convert temperature to Celsius if forecast is in Fahrenheit
        if first_period.temperature_unit == "F" {
            weather_data.temperature = ((first_period.temperature as f64 - CELSIUS_TO_FAHRENHEIT_OFFSET) / CELSIUS_TO_FAHRENHEIT_MULTIPLIER).round() as i64;
        }

        // Try to get current observations for more accurate data
        if let Some(station) = stations.features.first() {
            if let Ok(observation) = self.get_current_observation(&station.properties.station_identifier).await {
                if let Some(temp_c) = observation.properties.temperature.value {
                    weather_data.temperature = temp_c.round() as i64;
                }
                weather_data.humidity = observation.properties.relative_humidity
                    .and_then(|h| h.value);
                weather_data.wind_speed = observation.properties.wind_speed
                    .and_then(|w| w.value);
                weather_data.wind_direction = observation.properties.wind_direction
                    .and_then(|w| w.value);
            }
        }

        Ok(weather_data)
    }

    async fn get_current_observation(&self, station_id: &str) -> Result<ObservationResponse, WeatherError> {
        let url = format!("https://api.weather.gov/stations/{}/observations/latest", station_id);
        self.get_with_retry(&url).await
    }
}

fn get_weather_icon(condition: &str, icon_set: &IconSet) -> &'static str {
    let condition_lower = condition.to_lowercase();
    
    match icon_set {
        IconSet::NerdFont => {
            if condition_lower.contains("sunny") || condition_lower.contains("clear") {
                "󰖙" // nf-weather-day_sunny
            } else if condition_lower.contains("partly") {
                "󰖕" // nf-weather-day_cloudy
            } else if condition_lower.contains("mostly sunny") {
                "󰖐" // nf-weather-day_sunny_overcast
            } else if condition_lower.contains("cloud") || condition_lower.contains("overcast") {
                "󰖐" // nf-weather-cloudy
            } else if condition_lower.contains("thunder") || condition_lower.contains("storm") {
                "󰖓" // nf-weather-thunderstorm
            } else if condition_lower.contains("rain") || condition_lower.contains("showers") {
                if condition_lower.contains("light") {
                    "󰖗" // nf-weather-sprinkle
                } else {
                    "󰖖" // nf-weather-rain
                }
            } else if condition_lower.contains("snow") {
                if condition_lower.contains("light") {
                    "󰖘" // nf-weather-snow
                } else {
                    "󰼶" // nf-weather-snow_heavy
                }
            } else if condition_lower.contains("fog") || condition_lower.contains("mist") {
                "󰖑" // nf-weather-fog
            } else if condition_lower.contains("wind") {
                "󰖝" // nf-weather-strong_wind
            } else if condition_lower.contains("hot") {
                "󰔐" // nf-weather-hot
            } else if condition_lower.contains("cold") {
                "󰔒" // nf-weather-snowflake_cold
            } else {
                "󰖚" // nf-weather-na (not available)
            }
        }
        IconSet::Unicode => {
            if condition_lower.contains("sunny") || condition_lower.contains("clear") {
                "☀"
            } else if condition_lower.contains("partly") {
                "⛅"
            } else if condition_lower.contains("mostly sunny") {
                "🌤"
            } else if condition_lower.contains("cloud") {
                "☁"
            } else if condition_lower.contains("thunder") {
                "⛈"
            } else if condition_lower.contains("rain") || condition_lower.contains("showers") {
                "🌧"
            } else if condition_lower.contains("snow") {
                "❄"
            } else if condition_lower.contains("fog") || condition_lower.contains("mist") {
                "🌫"
            } else if condition_lower.contains("wind") {
                "💨"
            } else {
                "🌡"
            }
        }
        IconSet::Emoji => {
            if condition_lower.contains("sunny") || condition_lower.contains("clear") {
                "☀️"
            } else if condition_lower.contains("partly") {
                "⛅"
            } else if condition_lower.contains("mostly sunny") {
                "🌤️"
            } else if condition_lower.contains("cloud") {
                "☁️"
            } else if condition_lower.contains("thunder") {
                "⛈️"
            } else if condition_lower.contains("rain") || condition_lower.contains("showers") {
                "🌧️"
            } else if condition_lower.contains("snow") {
                "❄️"
            } else if condition_lower.contains("fog") || condition_lower.contains("mist") {
                "🌫️"
            } else if condition_lower.contains("wind") {
                "💨"
            } else {
                "🌡️"
            }
        }
        IconSet::Text => {
            if condition_lower.contains("sunny") || condition_lower.contains("clear") {
                "SUN"
            } else if condition_lower.contains("partly") {
                "P.CLY"
            } else if condition_lower.contains("mostly sunny") {
                "M.SUN"
            } else if condition_lower.contains("cloud") {
                "CLDY"
            } else if condition_lower.contains("thunder") {
                "THRM"
            } else if condition_lower.contains("rain") || condition_lower.contains("showers") {
                "RAIN"
            } else if condition_lower.contains("snow") {
                "SNOW"
            } else if condition_lower.contains("fog") || condition_lower.contains("mist") {
                "FOG"
            } else if condition_lower.contains("wind") {
                "WIND"
            } else {
                "WX"
            }
        }
    }
}

fn format_temperature(temp_c: i64, unit: &TemperatureUnit) -> (i64, &'static str) {
    match unit {
        TemperatureUnit::Celsius => (temp_c, "°C"),
        TemperatureUnit::Fahrenheit => {
            let temp_f = (temp_c as f64 * CELSIUS_TO_FAHRENHEIT_MULTIPLIER + CELSIUS_TO_FAHRENHEIT_OFFSET).round() as i64;
            (temp_f, "°F")
        }
    }
}

fn create_output(location: &Location, weather: &WeatherData, args: &Args) -> Result<String, WeatherError> {
    let icon = get_weather_icon(&weather.condition, &args.icons);
    let (temp, unit) = format_temperature(weather.temperature, &args.unit);
    
    match args.format {
        OutputFormat::Plain => {
            Ok(format!("{} {}{}  {}", icon, temp, unit, weather.condition))
        }
        OutputFormat::Json => {
            let output = serde_json::json!({
                "location": location.name,
                "temperature": temp,
                "unit": unit,
                "condition": weather.condition,
                "icon": icon,
                "humidity": weather.humidity,
                "wind_speed": weather.wind_speed,
                "wind_direction": weather.wind_direction
            });
            Ok(serde_json::to_string_pretty(&output)?)
        }
        OutputFormat::Waybar => {
            let text = format!("{} {}{}", icon, temp, unit);
            
            let tooltip = if args.detailed {
                let mut tooltip_parts = vec![
                    format!("{}: {}", location.name, weather.condition),
                    format!("Temperature: {}{}", temp, unit),
                ];
                
                if let Some(humidity) = weather.humidity {
                    tooltip_parts.push(format!("Humidity: {:.0}%", humidity));
                }
                
                if let Some(wind_speed) = weather.wind_speed {
                    let wind_text = if let Some(wind_dir) = weather.wind_direction {
                        format!("Wind: {:.0} mph from {}°", wind_speed * 2.237, wind_dir) // Convert m/s to mph
                    } else {
                        format!("Wind: {:.0} mph", wind_speed * 2.237)
                    };
                    tooltip_parts.push(wind_text);
                }
                
                tooltip_parts.join("\n")
            } else {
                format!("{}: {}", location.name, weather.condition)
            };
            
            let output = WaybarOutput {
                text,
                tooltip,
                class: "weather".to_string(),
            };
            
            Ok(serde_json::to_string(&output)?)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let client = WeatherClient::new();

    // Validate input
    if args.zip.is_none() && (args.lat.is_none() || args.lon.is_none()) {
        eprintln!("Error: You must provide either --zip ZIPCODE or --lat LAT --lon LON");
        std::process::exit(1);
    }

    match run_weather_app(&client, &args).await {
        Ok(output) => {
            println!("{}", output);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            
            // Provide fallback output for Waybar to prevent breaking the bar
            if matches!(args.format, OutputFormat::Waybar) {
                let fallback = WaybarOutput {
                    text: "Weather Error".to_string(),
                    tooltip: format!("Failed to get weather data: {}", e),
                    class: "weather-error".to_string(),
                };
                println!("{}", serde_json::to_string(&fallback).unwrap());
            }
            
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn run_weather_app(client: &WeatherClient, args: &Args) -> Result<String, WeatherError> {
    let location = client.resolve_location(args.zip.clone(), args.lat, args.lon).await?;
    let weather = client.get_weather_data(&location).await?;
    create_output(&location, &weather, args)
}
