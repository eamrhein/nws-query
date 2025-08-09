use thiserror::Error;

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
