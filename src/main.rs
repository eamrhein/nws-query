use clap::Parser;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

#[derive(Parser)]
#[command(author, version, about)]
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let client = Client::new();

    let (lat, lon, location_name) = if let Some(zip) = args.zip {
        // ZIP -> lat/lon lookup
        let zip_url = format!("https://api.zippopotam.us/us/{}", zip);
        let zip_data: ZippopotamResponse = client
            .get(&zip_url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let lat = zip_data.places[0].latitude.parse::<f64>()?;
        let lon = zip_data.places[0].longitude.parse::<f64>()?;
        let place_name = zip_data.places[0].place_name.clone();
        (lat, lon, place_name)
    } else if let (Some(lat), Some(lon)) = (args.lat, args.lon) {
        (lat, lon, String::from("Your location"))
    } else {
        eprintln!("Error: You must provide either --zip ZIPCODE or --lat LAT --lon LON");
        std::process::exit(1);
    };

    // Get grid info from NWS
    let point_url = format!("https://api.weather.gov/points/{},{}", lat, lon);
    let resp: Value = client
        .get(&point_url)
        .header("User-Agent", "my-waybar-weather/1.0")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let office = resp["properties"]["gridId"]
        .as_str()
        .ok_or("Missing gridId")?;
    let gridx = resp["properties"]["gridX"].as_i64().ok_or("Missing gridX")?;
    let gridy = resp["properties"]["gridY"].as_i64().ok_or("Missing gridY")?;

    // Get forecast
    let forecast_url = format!(
        "https://api.weather.gov/gridpoints/{}/{}, {}/forecast",
        office, gridx, gridy
    );
    let forecast: Value = client
        .get(&forecast_url)
        .header("User-Agent", "my-waybar-weather/1.0")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let first = &forecast["properties"]["periods"][0];
    let temp = first["temperature"].as_i64().unwrap_or(0);
    let icon = first["shortForecast"].as_str().unwrap_or("Unknown");

    // Output for Waybar
    let output = serde_json::json!({
        "text": format!("{}°F {}", temp, icon),
        "tooltip": format!("{}: {}", location_name, icon),
        "class": "weather"
    });

    println!("{}", output);

    Ok(())
}
