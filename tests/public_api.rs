use digipin::{
    cell, cell_size, compare_codes, contains, decode, encode, explore_prefixes, geohash,
    is_supported_coordinate, is_valid, locate, neighbors, normalize, plus_code, supported_bounds,
    Coordinates, DigiPinError, Direction,
};

#[test]
fn public_api_round_trip_keeps_coordinate_inside_cell() {
    let code = encode(12.9716, 77.5946).expect("encode Bengaluru coordinate");
    assert_eq!(code, "4P3-JK8-52C9");

    let decoded = decode(&code).expect("decode encoded DIGIPIN");
    let cell = cell(&code).expect("inspect encoded cell");

    assert!(contains(&cell, decoded.latitude, decoded.longitude));
    assert!(contains(&cell, 12.9716, 77.5946));
}

#[test]
fn public_api_normalizes_validates_and_reports_errors() {
    assert_eq!(normalize(" 4p3 jk8 52c9 ").unwrap(), "4P3-JK8-52C9");
    assert!(is_valid("4p3jk852c9"));
    assert!(!is_valid("4P3-JK8-52CZ"));

    assert_eq!(
        normalize("4P3-JK8-52CZ"),
        Err(DigiPinError::InvalidCharacter { character: 'Z' })
    );
    assert_eq!(encode(39.0, 77.0), Err(DigiPinError::LatitudeOutOfRange));
}

#[test]
fn public_api_exposes_bounds_location_and_neighbors() {
    let bounds = supported_bounds();
    assert!(bounds.min_latitude < bounds.max_latitude);
    assert!(bounds.min_longitude < bounds.max_longitude);
    assert!(is_supported_coordinate(28.6139, 77.2090));
    assert!(!is_supported_coordinate(1.0, 77.2090));

    let info = locate(12.9716, 77.5946).expect("locate Bengaluru coordinate");
    assert_eq!(info.digipin, "4P3-JK8-52C9");
    assert!(info.distance_to_cell_center_meters < 5.0);

    let size = cell_size(&info.cell);
    assert!(size.height_meters > 0.0);
    assert!(size.width_meters > 0.0);

    let neighbors = neighbors(&info.digipin).expect("neighbors");
    assert_eq!(neighbors.len(), 8);
    assert!(neighbors
        .iter()
        .any(|neighbor| neighbor.direction == Direction::North));
}

#[test]
fn public_api_distance_is_symmetric() {
    let a = Coordinates {
        latitude: 12.9716,
        longitude: 77.5946,
    };
    let b = Coordinates {
        latitude: 12.9717,
        longitude: 77.5946,
    };

    let ab = digipin::distance_meters(a, b);
    let ba = digipin::distance_meters(b, a);

    assert!(ab > 10.0);
    assert!((ab - ba).abs() < f64::EPSILON);
}

#[test]
fn public_api_explores_and_compares_codes() {
    let levels = explore_prefixes("4P3-JK8").expect("prefix explorer");
    assert_eq!(levels.len(), 6);
    assert_eq!(levels.first().unwrap().prefix, "4");
    assert_eq!(levels.last().unwrap().prefix, "4P3-JK8");

    assert_eq!(geohash(12.9716, 77.5946, 10).unwrap(), "tdr1v9qtj1");
    assert_eq!(plus_code(12.9716, 77.5946).unwrap(), "7J4VXHCV+JR");

    let comparison = compare_codes(12.9716, 77.5946, 10).expect("comparison");
    assert_eq!(comparison.digipin, "4P3-JK8-52C9");
    assert_eq!(comparison.geohash, "tdr1v9qtj1");
    assert_eq!(comparison.plus_code, "7J4VXHCV+JR");
}
