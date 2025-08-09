use clap::Parser;
use weather_cli::{run_weather_app, Args, WeatherClient, WeatherError};
use weather_cli::models::WaybarOutput;

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
            if matches!(args.format, weather_cli::config::OutputFormat::Waybar) {
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
