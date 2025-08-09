use clap::Parser;

#[derive(Parser)]
#[command(author, version, about = "Get weather information for Waybar")]
pub struct Args {
    /// ZIP code, e.g. 90210
    #[arg(long, conflicts_with_all=&["lat", "lon"])]
    pub zip: Option<String>,

    /// Latitude, e.g. 37.9
    #[arg(long, requires = "lon")]
    pub lat: Option<f64>,

    /// Longitude, e.g. -122.3
    #[arg(long, requires = "lat")]
    pub lon: Option<f64>,

    /// Temperature unit (F or C)
    #[arg(long, default_value = "F", value_parser = parse_unit)]
    pub unit: TemperatureUnit,

    /// Icon set to use
    #[arg(long, default_value = "nerdfont", value_parser = parse_icon_set)]
    pub icons: IconSet,

    /// Include additional weather details in tooltip
    #[arg(long)]
    pub detailed: bool,

    /// Wait for network connectivity before starting
    #[arg(long)]
    pub wait_for_network: bool,

    /// Output format
    #[arg(long, default_value = "waybar", value_parser = parse_output_format)]
    pub format: OutputFormat,
}

#[derive(Clone, Debug)]
pub enum TemperatureUnit {
    Fahrenheit,
    Celsius,
}

#[derive(Clone, Debug)]
pub enum IconSet {
    Unicode,
    Emoji,
    Text,
    NerdFont,
}

#[derive(Clone, Debug)]
pub enum OutputFormat {
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
