# digipin-rs

Rust implementation of India Post DIGIPIN encode/decode logic.

DIGIPIN converts latitude/longitude within India's official bounding box into a
10-character geocode. This crate provides a reusable Rust library and a small
CLI for local/offline conversion.

Official reference:

- India Post DIGIPIN page: <https://www.indiapost.gov.in/digipin>
- India Post GitHub repository: <https://github.com/INDIAPOST-gov/digipin>

## Install

```bash
cargo install --path .
```

Use it as a library from this repository:

```toml
[dependencies]
digipin-rs = { path = "." }
```

## CLI

Encode coordinates:

```bash
digipin encode 12.9716 77.5946
```

Output:

```text
4P3-JK8-52C9
```

Decode a DIGIPIN:

```bash
digipin decode 4P3-JK8-52C9
```

Output:

```text
12.971601,77.594584
```

JSON output:

```bash
digipin --json encode 12.9716 77.5946
```

Example JSON:

```json
{
  "latitude": 12.9716,
  "longitude": 77.5946,
  "digipin": "4P3-JK8-52C9"
}
```

Validate/normalize:

```bash
digipin validate 4p3jk852c9
```

Output:

```text
4P3-JK8-52C9
```

Inspect cell center and bounds:

```bash
digipin --json cell 4P3-JK8-52C9
```

Inspect a partial DIGIPIN prefix as a larger grid cell:

```bash
digipin --json partial-cell 4P3
```

Export a DIGIPIN cell as GeoJSON:

```bash
digipin geojson 4P3-JK8-52C9
```

Get precision-aware metadata for a coordinate:

```bash
digipin --json locate 12.9716 77.5946
```

Plain text output is also available:

```bash
digipin locate 12.9716 77.5946
```

List adjacent DIGIPIN cells:

```bash
digipin neighbors 4P3-JK8-52C9
```

Measure distance between two coordinates:

```bash
digipin distance 12.9716 77.5946 12.9717 77.5946
```

Process a CSV file:

```bash
digipin batch-csv input.csv --mode encode --output output.csv
```

Process JSONL from stdin:

```bash
printf '{"latitude":12.9716,"longitude":77.5946}\n' | digipin batch-jsonl - --mode encode
```

Run the local HTTP API:

```bash
digipin serve --host 127.0.0.1 --port 8080
curl 'http://127.0.0.1:8080/encode?latitude=12.9716&longitude=77.5946'
```

Generate shell completions:

```bash
digipin completions zsh > _digipin
```

### More CLI examples

Install and run from a clean checkout:

```bash
cargo run -- encode 28.6139 77.2090
cargo run -- decode 4P3-JK8-52C9
cargo run -- validate " 4p3 jk8 52c9 "
cargo run -- --json locate 12.9716 77.5946
cargo run -- --json neighbors 4P3-JK8-52C9
```

Handle invalid input with a non-zero exit:

```bash
digipin encode 39.0 77.0
```

```text
error: latitude out of range
```

## Library

```rust
let code = digipin::encode(12.9716, 77.5946)?;
let coords = digipin::decode(&code)?;
let cell = digipin::cell(&code)?;
let info = digipin::locate(12.9716, 77.5946)?;
let partial = digipin::partial_cell("4P3")?;
let geojson = digipin::digipin_geojson_feature(&code)?;
```

Run the example:

```bash
cargo run --example quickstart
```

## Features

- Encode latitude/longitude to canonical `XXX-XXX-XXXX` DIGIPIN.
- Decode DIGIPIN to grid-cell center coordinates.
- Normalize flexible input such as `4p3jk852c9`.
- Validate DIGIPIN strings.
- Report the India Post supported latitude/longitude bounding box.
- Check whether a coordinate is inside the supported bounding box before encode.
- Return grid-cell bounds for precision-aware applications.
- Return approximate cell width/height in meters.
- Check whether a coordinate falls inside a DIGIPIN cell.
- Return all 8 adjacent DIGIPIN cells.
- Measure haversine distance between coordinates.
- Return rich `locate` metadata for one coordinate lookup.
- Decode partial DIGIPIN prefixes into larger grid cells.
- Export DIGIPIN cells as GeoJSON Polygon/Feature data.
- Process CSV files with encode/decode/validate modes.
- Process newline-delimited JSON records.
- Run a small local HTTP API server.
- Generate shell completion scripts.
- Print a small OpenAPI document for the HTTP API.
- CLI supports plain text and JSON output.
- JSON responses use `serde`-serializable public structs.
- Errors use a typed `DigiPinError` enum with display messages.
- Canonical formatting inserts separators after the third and sixth characters.
- Flexible normalization accepts lowercase, whitespace, and missing separators.
- Library functions are deterministic and offline; no network calls are made.
- Includes unit and integration test coverage for public API behavior.

## Roadmap

- Publish the crate to crates.io once the public API settles.
- Add official reference vectors as fixtures when India Post publishes stable examples.
- Add CLI snapshot tests for plain text and JSON output.
- Add examples for web service integration.
- Add no-std feasibility notes for embedded/offline deployments.
- Add benchmark coverage for high-volume batch conversion.
- Add fuzz/property tests for normalize, encode, decode, and bounds invariants.
- Add release automation for tagged GitHub releases and generated changelogs.

## Design note

The core encode/decode algorithm is kept compatible with the India Post
reference behavior. Extra APIs are built around that algorithm so applications
can reason about precision, larger prefixes, neighboring cells, and JSON
workflows without changing official DIGIPIN output.

## License

Apache-2.0
