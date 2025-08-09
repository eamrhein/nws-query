pub mod client;
pub mod config;
pub mod error;
pub mod icons;
pub mod models;
pub mod output;

pub use client::WeatherClient;
pub use config::Args;
pub use error::WeatherError;
pub use output::create_output;

use std::time::Duration;
use tokio::time::sleep;

const INITIAL_DELAY_MS: u64 = 3000;

pub async fn run_weather_app(client: &WeatherClient, args: &Args) -> Result<String, WeatherError> {
    // Wait for network if requested or add initial delay for resume scenarios
    if args.wait_for_network {
        client.wait_for_network().await?;
    } else {
        // Small delay to handle resume scenarios
        sleep(Duration::from_millis(INITIAL_DELAY_MS)).await;
    }

    let location = client.resolve_location(args.zip.clone(), args.lat, args.lon).await?;
    let weather = client.get_weather_data(&location).await?;
    create_output(&location, &weather, args)
}
