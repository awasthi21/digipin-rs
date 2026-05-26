//! Rust implementation of India Post DIGIPIN encode/decode logic.
//!
//! DIGIPIN is a 10-character geocode derived from latitude/longitude within
//! India's official bounding box. This crate is a small, dependency-light port
//! of the Department of Posts reference implementation.

use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

const GRID: [[char; 4]; 4] = [
    ['F', 'C', '9', '8'],
    ['J', '3', '2', '7'],
    ['K', '4', '5', '6'],
    ['L', 'M', 'P', 'T'],
];

const MIN_LAT: f64 = 2.5;
const MAX_LAT: f64 = 38.5;
const MIN_LON: f64 = 63.5;
const MAX_LON: f64 = 99.5;
const DIGIPIN_LEN: usize = 10;

/// Approximate DIGIPIN grid cell size at the equator after 10 levels.
pub const APPROX_CELL_SIZE_METERS: f64 = 3.8;

/// DIGIPIN-supported geographic bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min_latitude: f64,
    pub max_latitude: f64,
    pub min_longitude: f64,
    pub max_longitude: f64,
}

/// A latitude/longitude pair.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

/// Bounds and center for one DIGIPIN grid cell.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Cell {
    pub center: Coordinates,
    pub bounds: BoundingBox,
}

/// Errors returned by DIGIPIN operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DigiPinError {
    LatitudeOutOfRange,
    LongitudeOutOfRange,
    InvalidLength,
    InvalidCharacter { character: char },
}

impl fmt::Display for DigiPinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LatitudeOutOfRange => write!(f, "latitude out of range"),
            Self::LongitudeOutOfRange => write!(f, "longitude out of range"),
            Self::InvalidLength => write!(f, "invalid DIGIPIN length"),
            Self::InvalidCharacter { character } => {
                write!(f, "invalid DIGIPIN character '{}'", character)
            }
        }
    }
}

impl Error for DigiPinError {}

/// Returns the supported bounding box for DIGIPIN coordinates.
pub fn supported_bounds() -> BoundingBox {
    BoundingBox {
        min_latitude: MIN_LAT,
        max_latitude: MAX_LAT,
        min_longitude: MIN_LON,
        max_longitude: MAX_LON,
    }
}

/// Returns true if the latitude/longitude is inside the supported DIGIPIN box.
pub fn is_supported_coordinate(latitude: f64, longitude: f64) -> bool {
    (MIN_LAT..=MAX_LAT).contains(&latitude) && (MIN_LON..=MAX_LON).contains(&longitude)
}

/// Encodes latitude/longitude into canonical `XXX-XXX-XXXX` DIGIPIN format.
pub fn encode(latitude: f64, longitude: f64) -> Result<String, DigiPinError> {
    if !(MIN_LAT..=MAX_LAT).contains(&latitude) {
        return Err(DigiPinError::LatitudeOutOfRange);
    }
    if !(MIN_LON..=MAX_LON).contains(&longitude) {
        return Err(DigiPinError::LongitudeOutOfRange);
    }

    let mut min_lat = MIN_LAT;
    let mut max_lat = MAX_LAT;
    let mut min_lon = MIN_LON;
    let mut max_lon = MAX_LON;
    let mut digipin = String::with_capacity(12);

    for level in 1..=DIGIPIN_LEN {
        let lat_div = (max_lat - min_lat) / 4.0;
        let lon_div = (max_lon - min_lon) / 4.0;

        let mut row = 3_i32 - ((latitude - min_lat) / lat_div).floor() as i32;
        let mut col = ((longitude - min_lon) / lon_div).floor() as i32;
        row = row.clamp(0, 3);
        col = col.clamp(0, 3);

        digipin.push(GRID[row as usize][col as usize]);
        if level == 3 || level == 6 {
            digipin.push('-');
        }

        max_lat = min_lat + lat_div * f64::from(4 - row);
        min_lat += lat_div * f64::from(3 - row);
        min_lon += lon_div * f64::from(col);
        max_lon = min_lon + lon_div;
    }

    Ok(digipin)
}

/// Decodes a DIGIPIN into the center-point of its grid cell.
pub fn decode(digipin: &str) -> Result<Coordinates, DigiPinError> {
    Ok(cell(digipin)?.center)
}

/// Returns the center and bounding box of a DIGIPIN grid cell.
pub fn cell(digipin: &str) -> Result<Cell, DigiPinError> {
    let normalized = normalize(digipin)?;
    let compact = compact(&normalized);
    let mut min_lat = MIN_LAT;
    let mut max_lat = MAX_LAT;
    let mut min_lon = MIN_LON;
    let mut max_lon = MAX_LON;

    for ch in compact.chars() {
        let (row, col) =
            find_grid_position(ch).ok_or(DigiPinError::InvalidCharacter { character: ch })?;

        let lat_div = (max_lat - min_lat) / 4.0;
        let lon_div = (max_lon - min_lon) / 4.0;

        let lat1 = max_lat - lat_div * (row as f64 + 1.0);
        let lat2 = max_lat - lat_div * row as f64;
        let lon1 = min_lon + lon_div * col as f64;
        let lon2 = min_lon + lon_div * (col as f64 + 1.0);

        min_lat = lat1;
        max_lat = lat2;
        min_lon = lon1;
        max_lon = lon2;
    }

    Ok(Cell {
        center: Coordinates {
            latitude: round_6((min_lat + max_lat) / 2.0),
            longitude: round_6((min_lon + max_lon) / 2.0),
        },
        bounds: BoundingBox {
            min_latitude: min_lat,
            max_latitude: max_lat,
            min_longitude: min_lon,
            max_longitude: max_lon,
        },
    })
}

/// Normalizes a DIGIPIN to canonical `XXX-XXX-XXXX` format.
pub fn normalize(digipin: &str) -> Result<String, DigiPinError> {
    let compact = compact(digipin);

    if compact.len() != DIGIPIN_LEN {
        return Err(DigiPinError::InvalidLength);
    }
    for ch in compact.chars() {
        if find_grid_position(ch).is_none() {
            return Err(DigiPinError::InvalidCharacter { character: ch });
        }
    }

    Ok(format!(
        "{}-{}-{}",
        &compact[0..3],
        &compact[3..6],
        &compact[6..10]
    ))
}

fn compact(digipin: &str) -> String {
    digipin
        .chars()
        .filter(|ch| *ch != '-' && !ch.is_whitespace())
        .map(|ch| ch.to_ascii_uppercase())
        .collect()
}

/// Returns true if a string is a syntactically valid DIGIPIN.
pub fn is_valid(digipin: &str) -> bool {
    normalize(digipin).is_ok()
}

fn find_grid_position(ch: char) -> Option<(usize, usize)> {
    for (row, cols) in GRID.iter().enumerate() {
        for (col, grid_ch) in cols.iter().enumerate() {
            if *grid_ch == ch {
                return Some((row, col));
            }
        }
    }
    None
}

fn round_6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_official_bengaluru_example() {
        assert_eq!(encode(12.9716, 77.5946).unwrap(), "4P3-JK8-52C9");
    }

    #[test]
    fn decodes_official_bengaluru_example() {
        let coords = decode("4P3-JK8-52C9").unwrap();
        assert_eq!(coords.latitude, 12.971601);
        assert_eq!(coords.longitude, 77.594584);
    }

    #[test]
    fn normalizes_flexible_input() {
        assert_eq!(normalize(" 4p3 jk8 52c9 ").unwrap(), "4P3-JK8-52C9");
    }

    #[test]
    fn rejects_invalid_inputs() {
        assert_eq!(encode(39.0, 77.0), Err(DigiPinError::LatitudeOutOfRange));
        assert_eq!(encode(28.0, 100.0), Err(DigiPinError::LongitudeOutOfRange));
        assert_eq!(normalize("ABC"), Err(DigiPinError::InvalidLength));
        assert_eq!(
            normalize("4P3-JK8-52CZ"),
            Err(DigiPinError::InvalidCharacter { character: 'Z' })
        );
    }

    #[test]
    fn reports_cell_bounds_containing_center() {
        let cell = cell("4P3-JK8-52C9").unwrap();
        assert!(cell.bounds.min_latitude <= cell.center.latitude);
        assert!(cell.bounds.max_latitude >= cell.center.latitude);
        assert!(cell.bounds.min_longitude <= cell.center.longitude);
        assert!(cell.bounds.max_longitude >= cell.center.longitude);
    }
}
