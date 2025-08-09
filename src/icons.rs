use crate::config::IconSet;

pub fn get_weather_icon(condition: &str, icon_set: &IconSet) -> &'static str {
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
