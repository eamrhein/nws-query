use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

use crate::error::WeatherError;
use crate::models::*;

// Constants
const USER_AGENT: &str = "waybar-weather-cli/2.0 (github.com/user/weather-cli)";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_RETRIES: u32 = 5;
const RETRY_DELAY_MS: u64 = 2000;

// Temperature conversion
const CELSIUS_TO_FAHRENHEIT_MULTIPLIER: f64 = 9.0 / 5.0;
const CELSIUS_TO_FAHRENHEIT_OFFSET: f64 = 32.0;

pub struct WeatherClient {
    client: Client,
}

impl WeatherClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .user_agent(USER_AGENT)
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client }
    }

    pub async fn wait_for_network(&self) -> Result<(), WeatherError> {
        let test_urls = [
            "https://api.weather.gov",
            "https://8.8.8.8",
            "https://1.1.1.1",
        ];

        for attempt in 1..=10 {
            for url in &test_urls {
                if let Ok(response) = tokio::time::timeout(
                    Duration::from_secs(3),
                    self.client.head(*url).send()
                ).await {
                    if response.is_ok() {
                        return Ok(());
                    }
                }
            }
            
            if attempt < 10 {
                sleep(Duration::from_millis(2000)).await;
            }
        }
        
         Err(WeatherError::Api("Network connectivity check failed".to_string()))
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

    pub async fn resolve_location(&self, zip: Option<String>, lat: Option<f64>, lon: Option<f64>) -> Result<Location, WeatherError> {
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

    pub async fn get_weather_data(&self, location: &Location) -> Result<WeatherData, WeatherError> {
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
