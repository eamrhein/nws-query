use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ZippopotamPlace {
    #[serde(rename = "place name")]
    pub place_name: String,
    pub longitude: String,
    pub latitude: String,
}

#[derive(Deserialize)]
pub struct ZippopotamResponse {
    pub places: Vec<ZippopotamPlace>,
}

#[derive(Deserialize)]
pub struct NWSPointProperties {
    #[serde(rename = "gridId")]
    pub grid_id: String,
    #[serde(rename = "gridX")]
    pub grid_x: i64,
    #[serde(rename = "gridY")]
    pub grid_y: i64,
    #[serde(rename = "observationStations")]
    pub observation_stations: String,
}

#[derive(Deserialize)]
pub struct NWSPointResponse {
    pub properties: NWSPointProperties,
}

#[derive(Deserialize)]
pub struct ForecastPeriod {
    pub temperature: i64,
    #[serde(rename = "temperatureUnit")]
    pub temperature_unit: String,
    #[serde(rename = "shortForecast")]
    pub short_forecast: String,
}

#[derive(Deserialize)]
pub struct ForecastProperties {
    pub periods: Vec<ForecastPeriod>,
}

#[derive(Deserialize)]
pub struct ForecastResponse {
    pub properties: ForecastProperties,
}

#[derive(Deserialize)]
pub struct StationFeature {
    pub properties: StationProperties,
}

#[derive(Deserialize)]
pub struct StationProperties {
    #[serde(rename = "stationIdentifier")]
    pub station_identifier: String,
}

#[derive(Deserialize)]
pub struct StationsResponse {
    pub features: Vec<StationFeature>,
}

#[derive(Deserialize)]
pub struct ObservationProperties {
    pub temperature: ObservationValue<f64>,
    #[serde(rename = "relativeHumidity")]
    pub relative_humidity: Option<ObservationValue<f64>>,
    #[serde(rename = "windSpeed")]
    pub wind_speed: Option<ObservationValue<f64>>,
    #[serde(rename = "windDirection")]
    pub wind_direction: Option<ObservationValue<f64>>,
}

#[derive(Deserialize)]
pub struct ObservationValue<T> {
    pub value: Option<T>,
}

#[derive(Deserialize)]
pub struct ObservationResponse {
    pub properties: ObservationProperties,
}

#[derive(Debug)]
pub struct Location {
    pub lat: f64,
    pub lon: f64,
    pub name: String,
}

#[derive(Debug)]
pub struct WeatherData {
    pub temperature: i64,
    pub condition: String,
    pub humidity: Option<f64>,
    pub wind_speed: Option<f64>,
    pub wind_direction: Option<f64>,
}

#[derive(Serialize)]
pub struct WaybarOutput {
    pub text: String,
    pub tooltip: String,
    pub class: String,
}
