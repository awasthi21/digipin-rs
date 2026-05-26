# digipin-rs

Rust toolkit for India Post DIGIPIN.

`digipin-rs` gives you a fast, offline, deterministic implementation of the
India Post DIGIPIN algorithm, plus developer utilities for CLI workflows, batch
conversion, GeoJSON export, and a tiny local HTTP API.

The core encoder/decoder stays compatible with the India Post reference logic.
Everything else is built around it so developers can inspect cells, prefixes,
bounds, neighboring cells, and structured JSON output without changing official
DIGIPIN behavior.

[![Rust](https://img.shields.io/badge/rust-2021-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)
[![DIGIPIN](https://img.shields.io/badge/India%20Post-DIGIPIN-green)](https://www.indiapost.gov.in/digipin)

## Why This Exists

DIGIPIN is useful, but the official reference implementation is only the start.
Real developer workflows need more:

- Convert coordinates to DIGIPIN from CLI, code, CSV, JSONL, or HTTP.
- Decode a DIGIPIN into its center point and cell bounds.
- Inspect partial prefixes like `4P3` as larger grid cells.
- Export cells as GeoJSON for maps and GIS tools.
- Run everything offline with no network calls.

## Install

From this repository:

```bash
cargo install --path .
```

Use it as a Rust library:

```toml
[dependencies]
digipin-rs = { path = "." }
```

## Quick Start

Encode latitude/longitude:

```bash
digipin encode 12.9716 77.5946
```

```text
4P3-JK8-52C9
```

Decode a DIGIPIN:

```bash
digipin decode 4P3-JK8-52C9
```

```text
12.971601,77.594584
```

Get JSON output:

```bash
digipin --json locate 12.9716 77.5946
```

Trimmed preview:

```json
{
  "input": {
    "latitude": 12.9716,
    "longitude": 77.5946
  },
  "digipin": "4P3-JK8-52C9",
  "cell": {
    "center": {
      "latitude": 12.971601,
      "longitude": 77.594584
    }
  }
}
```

## What You Can Do

| Need | Command/API |
| --- | --- |
| Encode coordinates | `digipin encode <lat> <lon>` |
| Decode DIGIPIN | `digipin decode <digipin>` |
| Validate and normalize | `digipin validate <digipin>` |
| Inspect cell bounds | `digipin cell <digipin>` |
| Inspect a prefix cell | `digipin partial-cell <prefix>` |
| Export GeoJSON | `digipin geojson <digipin>` |
| List neighboring cells | `digipin neighbors <digipin>` |
| Measure distance | `digipin distance <lat1> <lon1> <lat2> <lon2>` |
| Convert CSV files | `digipin batch-csv input.csv --mode encode` |
| Convert JSONL streams | `digipin batch-jsonl input.jsonl --mode decode` |
| Run local API | `digipin serve --port 8080` |
| Print OpenAPI | `digipin openapi` |
| Generate completions | `digipin completions zsh` |

## CLI Examples

Validate flexible input:

```bash
digipin validate " 4p3 jk8 52c9 "
```

```text
4P3-JK8-52C9
```

Inspect a full cell:

```bash
digipin --json cell 4P3-JK8-52C9
```

Inspect a partial prefix:

```bash
digipin --json partial-cell 4P3
```

Export a cell as GeoJSON:

```bash
digipin geojson 4P3-JK8-52C9
```

Process a CSV file:

```bash
digipin batch-csv input.csv --mode encode --output output.csv
```

Process JSONL from stdin:

```bash
printf '{"latitude":12.9716,"longitude":77.5946}\n' \
  | digipin batch-jsonl - --mode encode
```

Run the local API:

```bash
digipin serve --host 127.0.0.1 --port 8080
curl 'http://127.0.0.1:8080/encode?latitude=12.9716&longitude=77.5946'
```

Generate shell completions:

```bash
digipin completions zsh > _digipin
```

## Rust Library

```rust
let code = digipin::encode(12.9716, 77.5946)?;
let coords = digipin::decode(&code)?;
let cell = digipin::cell(&code)?;
let info = digipin::locate(12.9716, 77.5946)?;
let prefix = digipin::partial_cell("4P3")?;
let geojson = digipin::digipin_geojson_feature(&code)?;
```

Run the included example:

```bash
cargo run --example quickstart
```

## HTTP API

Start the local server:

```bash
digipin serve --port 8080
```

Available endpoints:

| Endpoint | Purpose |
| --- | --- |
| `GET /health` | Health check |
| `GET /encode?latitude=...&longitude=...` | Encode coordinates |
| `GET /decode?digipin=...` | Decode DIGIPIN |
| `GET /locate?latitude=...&longitude=...` | Encode with metadata |
| `GET /cell?digipin=...` | Return cell center and bounds |
| `GET /geojson?digipin=...` | Return cell as GeoJSON Feature |
| `GET /openapi.json` | Return OpenAPI document |

## Current Features

- Official-compatible DIGIPIN encode/decode.
- Canonical `XXX-XXX-XXXX` formatting.
- Flexible normalization for lowercase, spaces, and missing separators.
- Typed errors through `DigiPinError`.
- Full cell center and bounds.
- Partial prefix cells for hierarchy exploration.
- Approximate cell width and height in meters.
- Coordinate containment checks.
- Neighboring DIGIPIN cells.
- Haversine distance utility.
- GeoJSON Polygon and Feature export.
- CSV and JSONL batch workflows.
- Local HTTP API server.
- OpenAPI document generation.
- Shell completion generation.
- Unit and integration tests.

## Roadmap

| Priority | Feature | Why It Matters |
| --- | --- | --- |
| 1 | Prefix Explorer | Inspect each DIGIPIN prefix level with center, bounds, and approximate cell size. |
| 2 | GeoHash Comparison Tool | Compare DIGIPIN with GeoHash and Plus Codes using the same coordinate. Useful for developers evaluating location systems. |
| 3 | Plus Code Converter | Return DIGIPIN and Open Location Code side by side for interoperability. |
| 4 | Web Playground | Browser UI for encode/decode, prefix explorer, GeoJSON preview, and map display. |
| 5 | WASM Build | Run the encoder directly in browser apps without a backend. |
| 6 | Docker Image | One-command API server for local or cloud deployment. |
| 7 | Python Package | Make it useful for data teams and notebooks. |
| 8 | Node Package | Make it easy to use from web apps and backend services. |
| 9 | Postgres Functions | Enable DIGIPIN conversion directly inside databases. |
| 10 | Benchmark Suite | Publish performance numbers for batch conversion. |

## Official References

- India Post DIGIPIN page: <https://www.indiapost.gov.in/digipin>
- India Post GitHub repository: <https://github.com/INDIAPOST-gov/digipin>

## Development

Run tests:

```bash
cargo test
```

Format:

```bash
cargo fmt
```

Check CLI:

```bash
cargo run -- --help
```

## License

Apache-2.0
