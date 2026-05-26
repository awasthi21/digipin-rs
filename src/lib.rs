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
const GEOHASH_ALPHABET: &[u8; 32] = b"0123456789bcdefghjkmnpqrstuvwxyz";
const PLUS_CODE_ALPHABET: &[u8; 20] = b"23456789CFGHJMPQRVWX";
const PLUS_CODE_SEPARATOR_POSITION: usize = 8;
const PLUS_CODE_PAIR_RESOLUTIONS: [f64; 5] = [20.0, 1.0, 0.05, 0.0025, 0.000125];

/// Approximate DIGIPIN grid cell size at the equator after 10 levels.
pub const APPROX_CELL_SIZE_METERS: f64 = 3.8;

const EARTH_RADIUS_METERS: f64 = 6_371_000.0;

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

/// Approximate physical dimensions for one DIGIPIN cell.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CellSize {
    pub latitude_span_degrees: f64,
    pub longitude_span_degrees: f64,
    pub height_meters: f64,
    pub width_meters: f64,
}

/// GeoJSON Polygon geometry for a DIGIPIN cell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoJsonPolygon {
    #[serde(rename = "type")]
    pub geometry_type: String,
    pub coordinates: Vec<Vec<[f64; 2]>>,
}

/// GeoJSON properties attached to a DIGIPIN cell feature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoJsonCellProperties {
    pub digipin: Option<String>,
    pub center: Coordinates,
    pub bounds: BoundingBox,
}

/// GeoJSON Feature for a DIGIPIN cell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoJsonCellFeature {
    #[serde(rename = "type")]
    pub feature_type: String,
    pub geometry: GeoJsonPolygon,
    pub properties: GeoJsonCellProperties,
}

/// A neighboring DIGIPIN cell around a source cell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Neighbor {
    pub direction: Direction,
    pub digipin: String,
    pub cell: Cell,
}

/// Compass direction for a neighboring cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    NorthWest,
    North,
    NorthEast,
    West,
    East,
    SouthWest,
    South,
    SouthEast,
}

/// Rich metadata for one coordinate lookup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocationInfo {
    pub input: Coordinates,
    pub digipin: String,
    pub cell: Cell,
    pub cell_size: CellSize,
    pub distance_to_cell_center_meters: f64,
}

/// One row in a DIGIPIN prefix hierarchy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrefixLevel {
    pub level: usize,
    pub prefix: String,
    pub cell: Cell,
    pub cell_size: CellSize,
}

/// Side-by-side code comparison for one coordinate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeComparison {
    pub input: Coordinates,
    pub digipin: String,
    pub geohash: String,
    pub plus_code: String,
    pub cell: Cell,
    pub cell_size: CellSize,
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

/// Encodes a coordinate and returns precision-aware metadata.
pub fn locate(latitude: f64, longitude: f64) -> Result<LocationInfo, DigiPinError> {
    let digipin = encode(latitude, longitude)?;
    let cell = cell(&digipin)?;
    Ok(LocationInfo {
        input: Coordinates {
            latitude,
            longitude,
        },
        digipin,
        cell,
        cell_size: cell_size(&cell),
        distance_to_cell_center_meters: round_3(distance_meters(
            Coordinates {
                latitude,
                longitude,
            },
            cell.center,
        )),
    })
}

/// Returns every prefix level for a full or partial DIGIPIN.
pub fn explore_prefixes(digipin: &str) -> Result<Vec<PrefixLevel>, DigiPinError> {
    let normalized = normalize_partial(digipin)?;
    let compact = compact(&normalized);
    let mut levels = Vec::with_capacity(compact.len());

    for level in 1..=compact.len() {
        let prefix = normalize_partial(&compact[..level])?;
        let cell = partial_cell(&prefix)?;
        levels.push(PrefixLevel {
            level,
            prefix,
            cell,
            cell_size: cell_size(&cell),
        });
    }

    Ok(levels)
}

/// Encodes latitude/longitude to a standard GeoHash.
pub fn geohash(latitude: f64, longitude: f64, precision: usize) -> Result<String, DigiPinError> {
    if !(MIN_LAT..=MAX_LAT).contains(&latitude) {
        return Err(DigiPinError::LatitudeOutOfRange);
    }
    if !(MIN_LON..=MAX_LON).contains(&longitude) {
        return Err(DigiPinError::LongitudeOutOfRange);
    }

    let mut lat_range = [-90.0, 90.0];
    let mut lon_range = [-180.0, 180.0];
    let mut even_bit = true;
    let mut bit = 0;
    let mut ch = 0_u8;
    let mut output = String::with_capacity(precision);

    while output.len() < precision {
        if even_bit {
            let mid = (lon_range[0] + lon_range[1]) / 2.0;
            if longitude >= mid {
                ch = (ch << 1) | 1;
                lon_range[0] = mid;
            } else {
                ch <<= 1;
                lon_range[1] = mid;
            }
        } else {
            let mid = (lat_range[0] + lat_range[1]) / 2.0;
            if latitude >= mid {
                ch = (ch << 1) | 1;
                lat_range[0] = mid;
            } else {
                ch <<= 1;
                lat_range[1] = mid;
            }
        }

        even_bit = !even_bit;
        bit += 1;

        if bit == 5 {
            output.push(GEOHASH_ALPHABET[ch as usize] as char);
            bit = 0;
            ch = 0;
        }
    }

    Ok(output)
}

/// Encodes latitude/longitude to a 10-character Open Location Code / Plus Code.
pub fn plus_code(latitude: f64, longitude: f64) -> Result<String, DigiPinError> {
    if !(MIN_LAT..=MAX_LAT).contains(&latitude) {
        return Err(DigiPinError::LatitudeOutOfRange);
    }
    if !(MIN_LON..=MAX_LON).contains(&longitude) {
        return Err(DigiPinError::LongitudeOutOfRange);
    }

    let mut adjusted_latitude = latitude.min(90.0 - 1e-12) + 90.0;
    let mut adjusted_longitude = normalize_longitude(longitude) + 180.0;
    let mut code = String::with_capacity(11);

    for resolution in PLUS_CODE_PAIR_RESOLUTIONS {
        let lat_digit = (adjusted_latitude / resolution).floor().clamp(0.0, 19.0) as usize;
        let lon_digit = (adjusted_longitude / resolution).floor().clamp(0.0, 19.0) as usize;
        code.push(PLUS_CODE_ALPHABET[lat_digit] as char);
        code.push(PLUS_CODE_ALPHABET[lon_digit] as char);
        adjusted_latitude -= lat_digit as f64 * resolution;
        adjusted_longitude -= lon_digit as f64 * resolution;
    }

    code.insert(PLUS_CODE_SEPARATOR_POSITION, '+');
    Ok(code)
}

/// Returns DIGIPIN, GeoHash, Plus Code, and DIGIPIN cell metadata for one point.
pub fn compare_codes(
    latitude: f64,
    longitude: f64,
    geohash_precision: usize,
) -> Result<CodeComparison, DigiPinError> {
    let digipin = encode(latitude, longitude)?;
    let cell = cell(&digipin)?;
    Ok(CodeComparison {
        input: Coordinates {
            latitude,
            longitude,
        },
        geohash: geohash(latitude, longitude, geohash_precision)?,
        plus_code: plus_code(latitude, longitude)?,
        digipin,
        cell,
        cell_size: cell_size(&cell),
    })
}

/// Decodes a DIGIPIN into the center-point of its grid cell.
pub fn decode(digipin: &str) -> Result<Coordinates, DigiPinError> {
    Ok(cell(digipin)?.center)
}

/// Returns the center and bounding box of a DIGIPIN grid cell.
pub fn cell(digipin: &str) -> Result<Cell, DigiPinError> {
    let normalized = normalize(digipin)?;
    let compact = compact(&normalized);
    cell_from_compact(&compact)
}

/// Returns the center and bounding box of a partial DIGIPIN prefix.
///
/// Accepts 1 to 10 compact DIGIPIN characters. Shorter prefixes return the
/// larger grid cell that contains every full DIGIPIN beginning with the prefix.
pub fn partial_cell(digipin: &str) -> Result<Cell, DigiPinError> {
    let normalized = normalize_partial(digipin)?;
    let compact = compact(&normalized);
    cell_from_compact(&compact)
}

fn cell_from_compact(compact: &str) -> Result<Cell, DigiPinError> {
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

/// Returns approximate width/height for a cell in meters.
pub fn cell_size(cell: &Cell) -> CellSize {
    let south_west = Coordinates {
        latitude: cell.bounds.min_latitude,
        longitude: cell.bounds.min_longitude,
    };
    let north_west = Coordinates {
        latitude: cell.bounds.max_latitude,
        longitude: cell.bounds.min_longitude,
    };
    let west_center = Coordinates {
        latitude: cell.center.latitude,
        longitude: cell.bounds.min_longitude,
    };
    let east_center = Coordinates {
        latitude: cell.center.latitude,
        longitude: cell.bounds.max_longitude,
    };

    CellSize {
        latitude_span_degrees: cell.bounds.max_latitude - cell.bounds.min_latitude,
        longitude_span_degrees: cell.bounds.max_longitude - cell.bounds.min_longitude,
        height_meters: round_3(distance_meters(south_west, north_west)),
        width_meters: round_3(distance_meters(west_center, east_center)),
    }
}

/// Returns true if a coordinate lies inside the supplied cell bounds.
pub fn contains(cell: &Cell, latitude: f64, longitude: f64) -> bool {
    (cell.bounds.min_latitude..=cell.bounds.max_latitude).contains(&latitude)
        && (cell.bounds.min_longitude..=cell.bounds.max_longitude).contains(&longitude)
}

/// Returns the great-circle distance between two coordinates in meters.
pub fn distance_meters(from: Coordinates, to: Coordinates) -> f64 {
    let lat1 = from.latitude.to_radians();
    let lat2 = to.latitude.to_radians();
    let delta_lat = (to.latitude - from.latitude).to_radians();
    let delta_lon = (to.longitude - from.longitude).to_radians();

    let a =
        (delta_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    EARTH_RADIUS_METERS * c
}

/// Returns the 8 adjacent DIGIPIN cells, skipping neighbors outside India Post's supported box.
pub fn neighbors(digipin: &str) -> Result<Vec<Neighbor>, DigiPinError> {
    let source = cell(digipin)?;
    let lat_step = source.bounds.max_latitude - source.bounds.min_latitude;
    let lon_step = source.bounds.max_longitude - source.bounds.min_longitude;

    let specs = [
        (Direction::NorthWest, 1.0, -1.0),
        (Direction::North, 1.0, 0.0),
        (Direction::NorthEast, 1.0, 1.0),
        (Direction::West, 0.0, -1.0),
        (Direction::East, 0.0, 1.0),
        (Direction::SouthWest, -1.0, -1.0),
        (Direction::South, -1.0, 0.0),
        (Direction::SouthEast, -1.0, 1.0),
    ];

    let mut output = Vec::with_capacity(8);
    for (direction, lat_mul, lon_mul) in specs {
        let latitude = source.center.latitude + lat_step * lat_mul;
        let longitude = source.center.longitude + lon_step * lon_mul;
        if !is_supported_coordinate(latitude, longitude) {
            continue;
        }
        let neighbor_digipin = encode(latitude, longitude)?;
        output.push(Neighbor {
            direction,
            cell: cell(&neighbor_digipin)?,
            digipin: neighbor_digipin,
        });
    }

    Ok(output)
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

/// Normalizes a partial DIGIPIN prefix with canonical grouping where possible.
///
/// Hyphens and whitespace are ignored, characters are uppercased, and lengths
/// from 1 to 10 compact characters are accepted.
pub fn normalize_partial(digipin: &str) -> Result<String, DigiPinError> {
    let compact = compact(digipin);

    if compact.is_empty() || compact.len() > DIGIPIN_LEN {
        return Err(DigiPinError::InvalidLength);
    }
    for ch in compact.chars() {
        if find_grid_position(ch).is_none() {
            return Err(DigiPinError::InvalidCharacter { character: ch });
        }
    }

    if compact.len() <= 3 {
        Ok(compact)
    } else if compact.len() <= 6 {
        Ok(format!("{}-{}", &compact[0..3], &compact[3..]))
    } else {
        Ok(format!(
            "{}-{}-{}",
            &compact[0..3],
            &compact[3..6],
            &compact[6..]
        ))
    }
}

/// Returns a GeoJSON Polygon geometry for a cell.
pub fn cell_geojson_polygon(cell: &Cell) -> GeoJsonPolygon {
    GeoJsonPolygon {
        geometry_type: "Polygon".to_string(),
        coordinates: vec![vec![
            [cell.bounds.min_longitude, cell.bounds.min_latitude],
            [cell.bounds.max_longitude, cell.bounds.min_latitude],
            [cell.bounds.max_longitude, cell.bounds.max_latitude],
            [cell.bounds.min_longitude, cell.bounds.max_latitude],
            [cell.bounds.min_longitude, cell.bounds.min_latitude],
        ]],
    }
}

/// Returns a GeoJSON Polygon geometry for a full or partial DIGIPIN.
pub fn digipin_geojson_polygon(digipin: &str) -> Result<GeoJsonPolygon, DigiPinError> {
    Ok(cell_geojson_polygon(&partial_cell(digipin)?))
}

/// Returns a GeoJSON Feature for a cell.
pub fn cell_geojson_feature(cell: &Cell, digipin: Option<&str>) -> GeoJsonCellFeature {
    GeoJsonCellFeature {
        feature_type: "Feature".to_string(),
        geometry: cell_geojson_polygon(cell),
        properties: GeoJsonCellProperties {
            digipin: digipin.map(str::to_string),
            center: cell.center,
            bounds: cell.bounds,
        },
    }
}

/// Returns a GeoJSON Feature for a full or partial DIGIPIN cell.
pub fn digipin_geojson_feature(digipin: &str) -> Result<GeoJsonCellFeature, DigiPinError> {
    let normalized = normalize_partial(digipin)?;
    let cell = partial_cell(&normalized)?;
    Ok(cell_geojson_feature(&cell, Some(&normalized)))
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

fn round_3(value: f64) -> f64 {
    (value * 1_000.0).round() / 1_000.0
}

fn normalize_longitude(longitude: f64) -> f64 {
    let mut normalized = (longitude + 180.0) % 360.0;
    if normalized < 0.0 {
        normalized += 360.0;
    }
    normalized - 180.0
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
    fn normalizes_partial_input() {
        assert_eq!(normalize_partial("4").unwrap(), "4");
        assert_eq!(normalize_partial("4p3j").unwrap(), "4P3-J");
        assert_eq!(normalize_partial("4p3 jk8 52").unwrap(), "4P3-JK8-52");
        assert_eq!(normalize_partial(""), Err(DigiPinError::InvalidLength));
        assert_eq!(
            normalize_partial("4P3-JK8-52CZ"),
            Err(DigiPinError::InvalidCharacter { character: 'Z' })
        );
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

    #[test]
    fn reports_partial_cell_bounds() {
        let full = cell("4P3-JK8-52C9").unwrap();
        let partial = partial_cell("4P3").unwrap();
        assert!(contains(
            &partial,
            full.center.latitude,
            full.center.longitude
        ));
        assert!(partial.bounds.max_latitude - partial.bounds.min_latitude > 0.5);
        assert!(partial.bounds.max_longitude - partial.bounds.min_longitude > 0.5);
    }

    #[test]
    fn explores_prefix_levels() {
        let levels = explore_prefixes("4P3-JK8").unwrap();
        assert_eq!(levels.len(), 6);
        assert_eq!(levels[0].prefix, "4");
        assert_eq!(levels[5].prefix, "4P3-JK8");
        assert!(levels[0].cell_size.width_meters > levels[5].cell_size.width_meters);
    }

    #[test]
    fn reports_location_info() {
        let info = locate(12.9716, 77.5946).unwrap();
        assert_eq!(info.digipin, "4P3-JK8-52C9");
        assert!(info.cell_size.height_meters > 0.0);
        assert!(info.cell_size.width_meters > 0.0);
        assert!(contains(
            &info.cell,
            info.input.latitude,
            info.input.longitude
        ));
    }

    #[test]
    fn reports_neighbors() {
        let neighbors = neighbors("4P3-JK8-52C9").unwrap();
        assert_eq!(neighbors.len(), 8);
        assert!(neighbors
            .iter()
            .any(|neighbor| neighbor.direction == Direction::North));
    }

    #[test]
    fn measures_distance() {
        let distance = distance_meters(
            Coordinates {
                latitude: 12.9716,
                longitude: 77.5946,
            },
            Coordinates {
                latitude: 12.9717,
                longitude: 77.5946,
            },
        );
        assert!(distance > 10.0);
    }

    #[test]
    fn encodes_geohash_and_plus_code() {
        assert_eq!(geohash(12.9716, 77.5946, 10).unwrap(), "tdr1v9qtj1");
        assert_eq!(plus_code(12.9716, 77.5946).unwrap(), "7J4VXHCV+JR");
    }

    #[test]
    fn compares_codes() {
        let comparison = compare_codes(12.9716, 77.5946, 10).unwrap();
        assert_eq!(comparison.digipin, "4P3-JK8-52C9");
        assert_eq!(comparison.geohash, "tdr1v9qtj1");
        assert_eq!(comparison.plus_code, "7J4VXHCV+JR");
        assert!(contains(
            &comparison.cell,
            comparison.input.latitude,
            comparison.input.longitude
        ));
    }

    #[test]
    fn exports_cell_geojson() {
        let feature = digipin_geojson_feature("4P3-JK8-52C9").unwrap();
        assert_eq!(feature.feature_type, "Feature");
        assert_eq!(feature.geometry.geometry_type, "Polygon");
        assert_eq!(feature.properties.digipin.unwrap(), "4P3-JK8-52C9");
        assert_eq!(feature.geometry.coordinates[0].len(), 5);
        assert_eq!(
            feature.geometry.coordinates[0][0],
            feature.geometry.coordinates[0][4]
        );
    }

    #[test]
    fn encoded_coordinates_are_contained_by_their_cells_for_sample_grid() {
        let latitudes = [3.0, 8.75, 14.5, 20.25, 26.0, 31.75, 37.5];
        let longitudes = [64.0, 69.75, 75.5, 81.25, 87.0, 92.75, 99.0];

        for latitude in latitudes {
            for longitude in longitudes {
                let digipin = encode(latitude, longitude).unwrap();
                let cell = cell(&digipin).unwrap();
                assert!(
                    contains(&cell, latitude, longitude),
                    "{latitude},{longitude} encoded to {digipin} outside {:?}",
                    cell.bounds
                );
            }
        }
    }
}
