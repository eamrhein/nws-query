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

fn forecast_icon(condition: &str) -> &'static str {
    let c = condition.to_lowercase();
    let mappings = [
        ("sunny", ""),
        ("clear", ""),
        ("partly", ""),
        ("mostly sunny", ""),
        ("cloud", ""),
        ("thunder", ""),
        ("rain", ""),
        ("showers", ""),
        ("snow", ""),
        ("fog", ""),
        ("mist", ""),
        ("wind", ""),
    ];

    for (key, icon) in &mappings {
        if c.contains(key) {
            return icon;
        }
    }
    "" // Default icon
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let client = Client::new();

    // 1. Get lat/lon & location name
    let (lat, lon, location_name) = if let Some(zip) = args.zip {
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

    // 2. Get NWS grid info
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

    // 3. Get forecast (for icon & fallback temp)
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

    let first_period = &forecast["properties"]["periods"][0];
    let forecast_temp = first_period["temperature"].as_i64().unwrap_or(0);
    let short_forecast = first_period["shortForecast"].as_str().unwrap_or("Unknown");

    // 4. Get observation stations URL
    let stations_url = resp["properties"]["observationStations"]
        .as_str()
        .ok_or("Missing observationStations")?;
    let stations_resp: Value = client
        .get(stations_url)
        .header("User-Agent", "my-waybar-weather/1.0")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    // 5. Get first station id
    let first_station_id = stations_resp["features"][0]["properties"]["stationIdentifier"]
        .as_str()
        .ok_or("No stations found")?;

    // 6. Get latest observation
    let obs_url = format!(
        "https://api.weather.gov/stations/{}/observations/latest",
        first_station_id
    );
    let observation: Value = client
        .get(&obs_url)
        .header("User-Agent", "my-waybar-weather/1.0")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    // 7. Parse temp (C) -> F and handle missing data
    let temp_c = observation["properties"]["temperature"]["value"].as_f64();
    let temp_f = temp_c.map(|c| c * 9.0 / 5.0 + 32.0);

    // 8. Decide which temp to show
    let display_temp = temp_f
        .map(|f| f.round() as i64)
        .unwrap_or(forecast_temp);

    // 9. Get icon
    let icon_char = forecast_icon(short_forecast);

    // 10. Print JSON for Waybar
    let output = serde_json::json!({
        "text": format!("{}  {}°F", icon_char, display_temp),
        "tooltip": format!("{}: {}", location_name, short_forecast),
        "class": "weather"
    });

    println!("{}", output);

    Ok(())
}
