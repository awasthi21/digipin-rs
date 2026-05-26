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

## Library

```rust
let code = digipin::encode(12.9716, 77.5946)?;
let coords = digipin::decode(&code)?;
let cell = digipin::cell(&code)?;
```

## Features

- Encode latitude/longitude to canonical `XXX-XXX-XXXX` DIGIPIN.
- Decode DIGIPIN to grid-cell center coordinates.
- Normalize flexible input such as `4p3jk852c9`.
- Validate DIGIPIN strings.
- Return grid-cell bounds for precision-aware applications.
- CLI supports plain text and JSON output.

## License

Apache-2.0
