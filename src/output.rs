use crate::config::{Args, OutputFormat, TemperatureUnit};
use crate::error::WeatherError;
use crate::icons::get_weather_icon;
use crate::models::{Location, WeatherData, WaybarOutput};

// Temperature conversion
const CELSIUS_TO_FAHRENHEIT_MULTIPLIER: f64 = 9.0 / 5.0;
const CELSIUS_TO_FAHRENHEIT_OFFSET: f64 = 32.0;

pub fn format_temperature(temp_c: i64, unit: &TemperatureUnit) -> (i64, &'static str) {
    match unit {
        TemperatureUnit::Celsius => (temp_c, "°C"),
        TemperatureUnit::Fahrenheit => {
            let temp_f = (temp_c as f64 * CELSIUS_TO_FAHRENHEIT_MULTIPLIER + CELSIUS_TO_FAHRENHEIT_OFFSET).round() as i64;
            (temp_f, "°F")
        }
    }
}

pub fn create_output(location: &Location, weather: &WeatherData, args: &Args) -> Result<String, WeatherError> {
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
