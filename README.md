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

JSON output:

```bash
digipin --json encode 12.9716 77.5946
```

Validate/normalize:

```bash
digipin validate 4p3jk852c9
```

Inspect cell center and bounds:

```bash
digipin --json cell 4P3-JK8-52C9
```

Get precision-aware metadata for a coordinate:

```bash
digipin --json locate 12.9716 77.5946
```

List adjacent DIGIPIN cells:

```bash
digipin neighbors 4P3-JK8-52C9
```

List DIGIPIN candidates around a coordinate and radius, useful near grid
boundaries:

```bash
digipin candidates 12.924933 77.599893 5
```

Measure distance between two coordinates:

```bash
digipin distance 12.9716 77.5946 12.9717 77.5946
```

## Library

```rust
let code = digipin::encode(12.9716, 77.5946)?;
let coords = digipin::decode(&code)?;
let cell = digipin::cell(&code)?;
let info = digipin::locate(12.9716, 77.5946)?;
let nearby = digipin::candidates_within_radius(12.9716, 77.5946, 5.0)?;
```

## Features

- Encode latitude/longitude to canonical `XXX-XXX-XXXX` DIGIPIN.
- Decode DIGIPIN to grid-cell center coordinates.
- Normalize flexible input such as `4p3jk852c9`.
- Validate DIGIPIN strings.
- Return grid-cell bounds for precision-aware applications.
- Return approximate cell width/height in meters.
- Check whether a coordinate falls inside a DIGIPIN cell.
- Return all 8 adjacent DIGIPIN cells.
- Return nearby DIGIPIN candidates for radius/boundary checks.
- Measure haversine distance between coordinates.
- Return rich `locate` metadata for one coordinate lookup.
- CLI supports plain text and JSON output.

## Design note

The core encode/decode algorithm is kept compatible with the India Post
reference behavior. Extra APIs are built around that algorithm so applications
can reason about precision, boundaries, neighboring cells, and JSON workflows
without changing official DIGIPIN output.

## License

Apache-2.0
