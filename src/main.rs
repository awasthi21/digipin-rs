use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
};

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
    /// Encode coordinates and print cell precision metadata.
    Locate { latitude: f64, longitude: f64 },
    /// Decode DIGIPIN to center-point latitude/longitude.
    Decode { digipin: String },
    /// Validate and normalize a DIGIPIN.
    Validate { digipin: String },
    /// Print the center and bounding box for a DIGIPIN cell.
    Cell { digipin: String },
    /// Print the center and bounding box for a partial DIGIPIN prefix.
    PartialCell { digipin: String },
    /// Print a DIGIPIN cell as a GeoJSON Feature.
    Geojson { digipin: String },
    /// Print the 8 adjacent DIGIPIN cells.
    Neighbors { digipin: String },
    /// Print distance between two coordinates in meters.
    Distance {
        from_latitude: f64,
        from_longitude: f64,
        to_latitude: f64,
        to_longitude: f64,
    },
    /// Process a CSV file with latitude/longitude or DIGIPIN columns.
    BatchCsv {
        /// Input CSV path. Use '-' for stdin.
        input: PathBuf,
        /// Output CSV path. Defaults to stdout.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Operation to run for each row.
        #[arg(short, long, default_value = "auto")]
        mode: BatchMode,
    },
    /// Process newline-delimited JSON records.
    BatchJsonl {
        /// Input JSONL path. Use '-' for stdin.
        input: PathBuf,
        /// Output JSONL path. Defaults to stdout.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Operation to run for each row.
        #[arg(short, long, default_value = "auto")]
        mode: BatchMode,
    },
    /// Run a small local HTTP API server.
    Serve {
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
    /// Generate shell completion script.
    Completions { shell: Shell },
    /// Print a small OpenAPI document for the HTTP server.
    Openapi,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum BatchMode {
    Auto,
    Encode,
    Decode,
    Locate,
    Validate,
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

#[derive(Debug, Serialize)]
struct DistanceOutput {
    from: digipin::Coordinates,
    to: digipin::Coordinates,
    distance_meters: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonlRecord {
    latitude: Option<f64>,
    longitude: Option<f64>,
    digipin: Option<String>,
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
        Command::Locate {
            latitude,
            longitude,
        } => {
            let info = digipin::locate(latitude, longitude)?;
            if cli.json {
                print_json(&info)?;
            } else {
                println!(
                    "{} center={},{} size={}m x {}m",
                    info.digipin,
                    info.cell.center.latitude,
                    info.cell.center.longitude,
                    info.cell_size.height_meters,
                    info.cell_size.width_meters
                );
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
        Command::PartialCell { digipin: input } => {
            let normalized = digipin::normalize_partial(&input)?;
            let cell = digipin::partial_cell(&normalized)?;
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
        Command::Geojson { digipin: input } => {
            let feature = digipin::digipin_geojson_feature(&input)?;
            if cli.json {
                print_json(&feature)?;
            } else {
                println!("{}", serde_json::to_string_pretty(&feature)?);
            }
        }
        Command::Neighbors { digipin: input } => {
            let normalized = digipin::normalize(&input)?;
            let neighbors = digipin::neighbors(&normalized)?;
            if cli.json {
                print_json(&neighbors)?;
            } else {
                for neighbor in neighbors {
                    println!("{:?} {}", neighbor.direction, neighbor.digipin);
                }
            }
        }
        Command::Distance {
            from_latitude,
            from_longitude,
            to_latitude,
            to_longitude,
        } => {
            let from = digipin::Coordinates {
                latitude: from_latitude,
                longitude: from_longitude,
            };
            let to = digipin::Coordinates {
                latitude: to_latitude,
                longitude: to_longitude,
            };
            let distance_meters = digipin::distance_meters(from, to);
            if cli.json {
                print_json(&DistanceOutput {
                    from,
                    to,
                    distance_meters,
                })?;
            } else {
                println!("{distance_meters:.3}");
            }
        }
        Command::BatchCsv {
            input,
            output,
            mode,
        } => run_batch_csv(input, output, mode)?,
        Command::BatchJsonl {
            input,
            output,
            mode,
        } => run_batch_jsonl(input, output, mode)?,
        Command::Serve { host, port } => serve(&host, port)?,
        Command::Completions { shell } => {
            let mut command = Cli::command();
            generate(shell, &mut command, "digipin", &mut io::stdout());
        }
        Command::Openapi => {
            println!("{}", openapi_document()?);
        }
    }
    Ok(())
}

fn print_json<T: Serialize>(value: &T) -> Result<(), serde_json::Error> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn run_batch_csv(
    input: PathBuf,
    output: Option<PathBuf>,
    mode: BatchMode,
) -> Result<(), Box<dyn std::error::Error>> {
    let reader: Box<dyn Read> = if input.to_string_lossy() == "-" {
        Box::new(io::stdin())
    } else {
        Box::new(File::open(input)?)
    };
    let writer: Box<dyn Write> = match output {
        Some(path) => Box::new(File::create(path)?),
        None => Box::new(io::stdout()),
    };

    let mut csv_reader = csv::Reader::from_reader(reader);
    let headers = csv_reader.headers()?.clone();
    let mut output_headers = headers.clone();
    output_headers.push_field("digipin_result");
    output_headers.push_field("latitude_result");
    output_headers.push_field("longitude_result");
    output_headers.push_field("valid");
    output_headers.push_field("error");

    let mut csv_writer = csv::Writer::from_writer(writer);
    csv_writer.write_record(&output_headers)?;

    for record in csv_reader.records() {
        let record = record?;
        let row = headers
            .iter()
            .zip(record.iter())
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect::<HashMap<_, _>>();
        let processed = process_record(
            mode,
            row.get("latitude").and_then(|value| value.parse().ok()),
            row.get("longitude").and_then(|value| value.parse().ok()),
            row.get("digipin").cloned(),
        );

        let mut output_record = record.clone();
        output_record.push_field(processed.digipin.as_deref().unwrap_or(""));
        output_record.push_field(
            &processed
                .latitude
                .map(|value| value.to_string())
                .unwrap_or_default(),
        );
        output_record.push_field(
            &processed
                .longitude
                .map(|value| value.to_string())
                .unwrap_or_default(),
        );
        output_record.push_field(if processed.valid { "true" } else { "false" });
        output_record.push_field(processed.error.as_deref().unwrap_or(""));
        csv_writer.write_record(&output_record)?;
    }
    csv_writer.flush()?;
    Ok(())
}

fn run_batch_jsonl(
    input: PathBuf,
    output: Option<PathBuf>,
    mode: BatchMode,
) -> Result<(), Box<dyn std::error::Error>> {
    let reader: Box<dyn BufRead> = if input.to_string_lossy() == "-" {
        Box::new(BufReader::new(io::stdin()))
    } else {
        Box::new(BufReader::new(File::open(input)?))
    };
    let mut writer: Box<dyn Write> = match output {
        Some(path) => Box::new(File::create(path)?),
        None => Box::new(io::stdout()),
    };

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let record: JsonlRecord = serde_json::from_str(&line)?;
        let processed = process_record(mode, record.latitude, record.longitude, record.digipin);
        writeln!(writer, "{}", serde_json::to_string(&processed)?)?;
    }
    Ok(())
}

#[derive(Debug, Serialize)]
struct ProcessedRecord {
    digipin: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    valid: bool,
    error: Option<String>,
}

fn process_record(
    mode: BatchMode,
    latitude: Option<f64>,
    longitude: Option<f64>,
    digipin: Option<String>,
) -> ProcessedRecord {
    let resolved_mode = match mode {
        BatchMode::Auto if latitude.is_some() && longitude.is_some() => BatchMode::Encode,
        BatchMode::Auto if digipin.is_some() => BatchMode::Decode,
        BatchMode::Auto => BatchMode::Validate,
        other => other,
    };

    match resolved_mode {
        BatchMode::Encode | BatchMode::Locate => match (latitude, longitude) {
            (Some(lat), Some(lon)) => match digipin::encode(lat, lon) {
                Ok(code) => ProcessedRecord {
                    digipin: Some(code),
                    latitude: Some(lat),
                    longitude: Some(lon),
                    valid: true,
                    error: None,
                },
                Err(error) => ProcessedRecord::error(error.to_string()),
            },
            _ => ProcessedRecord::error("latitude and longitude are required".to_string()),
        },
        BatchMode::Decode => match digipin {
            Some(input) => match digipin::normalize(&input).and_then(|code| {
                let coords = digipin::decode(&code)?;
                Ok((code, coords))
            }) {
                Ok((code, coords)) => ProcessedRecord {
                    digipin: Some(code),
                    latitude: Some(coords.latitude),
                    longitude: Some(coords.longitude),
                    valid: true,
                    error: None,
                },
                Err(error) => ProcessedRecord::error(error.to_string()),
            },
            None => ProcessedRecord::error("digipin is required".to_string()),
        },
        BatchMode::Validate | BatchMode::Auto => match digipin {
            Some(input) => match digipin::normalize(&input) {
                Ok(code) => ProcessedRecord {
                    digipin: Some(code),
                    latitude: None,
                    longitude: None,
                    valid: true,
                    error: None,
                },
                Err(error) => ProcessedRecord::error(error.to_string()),
            },
            None => ProcessedRecord::error("digipin is required".to_string()),
        },
    }
}

impl ProcessedRecord {
    fn error(error: String) -> Self {
        Self {
            digipin: None,
            latitude: None,
            longitude: None,
            valid: false,
            error: Some(error),
        }
    }
}

fn serve(host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind((host, port))?;
    eprintln!("digipin server listening on http://{host}:{port}");
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(error) = handle_http(&mut stream) {
                    let body = serde_json::json!({ "error": error.to_string() });
                    write_response(&mut stream, 500, &body)?;
                }
            }
            Err(error) => eprintln!("connection error: {error}"),
        }
    }
    Ok(())
}

fn handle_http(stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = [0; 4096];
    let read = stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..read]);
    let request_line = request.lines().next().unwrap_or_default();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let target = parts.next().unwrap_or("/");
    if method != "GET" {
        return write_response(
            stream,
            405,
            &serde_json::json!({ "error": "method not allowed" }),
        );
    }

    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let params = parse_query(query);
    match path {
        "/health" => write_response(stream, 200, &serde_json::json!({ "status": "ok" })),
        "/encode" => {
            let (latitude, longitude) = parse_lat_lon(&params)?;
            let digipin = digipin::encode(latitude, longitude)?;
            write_response(
                stream,
                200,
                &EncodeOutput {
                    latitude,
                    longitude,
                    digipin,
                },
            )
        }
        "/decode" => {
            let input = required(&params, "digipin")?;
            let normalized = digipin::normalize(input)?;
            let coords = digipin::decode(&normalized)?;
            write_response(
                stream,
                200,
                &DecodeOutput {
                    digipin: normalized,
                    latitude: coords.latitude,
                    longitude: coords.longitude,
                },
            )
        }
        "/locate" => {
            let (latitude, longitude) = parse_lat_lon(&params)?;
            write_response(stream, 200, &digipin::locate(latitude, longitude)?)
        }
        "/cell" => {
            let input = required(&params, "digipin")?;
            write_response(stream, 200, &digipin::cell(input)?)
        }
        "/geojson" => {
            let input = required(&params, "digipin")?;
            write_response(stream, 200, &digipin::digipin_geojson_feature(input)?)
        }
        "/openapi.json" => {
            let body: serde_json::Value = serde_json::from_str(&openapi_document()?)?;
            write_response(stream, 200, &body)
        }
        _ => write_response(stream, 404, &serde_json::json!({ "error": "not found" })),
    }
}

fn write_response<T: Serialize>(
    stream: &mut TcpStream,
    status: u16,
    body: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    let reason = match status {
        200 => "OK",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "OK",
    };
    let body = serde_json::to_string_pretty(body)?;
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
        body.len()
    )?;
    Ok(())
}

fn parse_lat_lon(
    params: &HashMap<String, String>,
) -> Result<(f64, f64), Box<dyn std::error::Error>> {
    let latitude = required(params, "latitude")?.parse()?;
    let longitude = required(params, "longitude")?.parse()?;
    Ok((latitude, longitude))
}

fn required<'a>(
    params: &'a HashMap<String, String>,
    key: &str,
) -> Result<&'a String, Box<dyn std::error::Error>> {
    params
        .get(key)
        .ok_or_else(|| format!("missing {key}").into())
}

fn parse_query(query: &str) -> HashMap<String, String> {
    query
        .split('&')
        .filter(|part| !part.is_empty())
        .filter_map(|part| {
            let (key, value) = part.split_once('=').unwrap_or((part, ""));
            Some((percent_decode(key)?, percent_decode(value)?))
        })
        .collect()
}

fn percent_decode(input: &str) -> Option<String> {
    let mut output = String::new();
    let mut chars = input.as_bytes().iter().copied();
    while let Some(byte) = chars.next() {
        match byte {
            b'+' => output.push(' '),
            b'%' => {
                let high = chars.next()?;
                let low = chars.next()?;
                let hex = [high, low];
                let value = u8::from_str_radix(std::str::from_utf8(&hex).ok()?, 16).ok()?;
                output.push(value as char);
            }
            other => output.push(other as char),
        }
    }
    Some(output)
}

fn openapi_document() -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&serde_json::json!({
        "openapi": "3.1.0",
        "info": {
            "title": "digipin-rs API",
            "version": env!("CARGO_PKG_VERSION")
        },
        "paths": {
            "/health": { "get": { "summary": "Health check" } },
            "/encode": { "get": { "summary": "Encode latitude/longitude to DIGIPIN" } },
            "/decode": { "get": { "summary": "Decode DIGIPIN to center coordinates" } },
            "/locate": { "get": { "summary": "Return DIGIPIN and precision metadata" } },
            "/cell": { "get": { "summary": "Return DIGIPIN cell center and bounds" } },
            "/geojson": { "get": { "summary": "Return DIGIPIN cell as GeoJSON Feature" } },
            "/openapi.json": { "get": { "summary": "Return this OpenAPI document" } }
        }
    }))
}
