fn main() -> Result<(), digipin::DigiPinError> {
    let latitude = 12.9716;
    let longitude = 77.5946;

    let code = digipin::encode(latitude, longitude)?;
    let normalized = digipin::normalize(&code)?;
    let coords = digipin::decode(&normalized)?;
    let cell = digipin::cell(&normalized)?;
    let size = digipin::cell_size(&cell);
    let neighbors = digipin::neighbors(&normalized)?;

    println!("DIGIPIN: {code}");
    println!("Normalized: {normalized}");
    println!("Center: {},{}", coords.latitude, coords.longitude);
    println!(
        "Cell size: {:.3}m high x {:.3}m wide",
        size.height_meters, size.width_meters
    );
    println!("Neighbor count: {}", neighbors.len());

    Ok(())
}
