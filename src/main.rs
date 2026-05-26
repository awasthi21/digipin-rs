use clap::{Parser, Subcommand};
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(name = "digipin")]
#[command(about = "Encode/decode India Post DIGIPIN values")]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Emit JSON instead of plain text.
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Encode latitude/longitude to DIGIPIN.
    Encode { latitude: f64, longitude: f64 },
    /// Decode DIGIPIN to center-point latitude/longitude.
    Decode { digipin: String },
    /// Validate and normalize a DIGIPIN.
    Validate { digipin: String },
    /// Print the center and bounding box for a DIGIPIN cell.
    Cell { digipin: String },
}

#[derive(Debug, Serialize)]
struct EncodeOutput {
    latitude: f64,
    longitude: f64,
    digipin: String,
}

#[derive(Debug, Serialize)]
struct DecodeOutput {
    digipin: String,
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, Serialize)]
struct ValidateOutput {
    input: String,
    valid: bool,
    normalized: Option<String>,
    error: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    if let Err(error) = run(cli) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Command::Encode {
            latitude,
            longitude,
        } => {
            let digipin = digipin::encode(latitude, longitude)?;
            if cli.json {
                print_json(&EncodeOutput {
                    latitude,
                    longitude,
                    digipin,
                })?;
            } else {
                println!("{digipin}");
            }
        }
        Command::Decode { digipin: input } => {
            let normalized = digipin::normalize(&input)?;
            let coords = digipin::decode(&normalized)?;
            if cli.json {
                print_json(&DecodeOutput {
                    digipin: normalized,
                    latitude: coords.latitude,
                    longitude: coords.longitude,
                })?;
            } else {
                println!("{},{}", coords.latitude, coords.longitude);
            }
        }
        Command::Validate { digipin: input } => match digipin::normalize(&input) {
            Ok(normalized) => {
                if cli.json {
                    print_json(&ValidateOutput {
                        input,
                        valid: true,
                        normalized: Some(normalized),
                        error: None,
                    })?;
                } else {
                    println!("{normalized}");
                }
            }
            Err(error) => {
                if cli.json {
                    print_json(&ValidateOutput {
                        input,
                        valid: false,
                        normalized: None,
                        error: Some(error.to_string()),
                    })?;
                } else {
                    return Err(Box::new(error));
                }
            }
        },
        Command::Cell { digipin: input } => {
            let normalized = digipin::normalize(&input)?;
            let cell = digipin::cell(&normalized)?;
            if cli.json {
                print_json(&cell)?;
            } else {
                println!(
                    "{normalized} center={},{} bounds={},{},{},{}",
                    cell.center.latitude,
                    cell.center.longitude,
                    cell.bounds.min_latitude,
                    cell.bounds.min_longitude,
                    cell.bounds.max_latitude,
                    cell.bounds.max_longitude
                );
            }
        }
    }
    Ok(())
}

fn print_json<T: Serialize>(value: &T) -> Result<(), serde_json::Error> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
