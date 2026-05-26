use std::time::Instant;

fn main() -> Result<(), digipin::DigiPinError> {
    let iterations = std::env::args()
        .nth(1)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1_000_000);

    let samples = [
        (12.9716, 77.5946),
        (28.6139, 77.2090),
        (19.0760, 72.8777),
        (22.5726, 88.3639),
        (13.0827, 80.2707),
    ];

    let started = Instant::now();
    let mut checksum = 0_usize;

    for index in 0..iterations {
        let (latitude, longitude) = samples[index % samples.len()];
        let code = digipin::encode(latitude, longitude)?;
        checksum = checksum.wrapping_add(code.as_bytes()[index % code.len()] as usize);
    }

    let elapsed = started.elapsed();
    let per_second = iterations as f64 / elapsed.as_secs_f64();

    println!("iterations={iterations}");
    println!("elapsed_ms={:.3}", elapsed.as_secs_f64() * 1000.0);
    println!("encodes_per_second={per_second:.0}");
    println!("checksum={checksum}");

    Ok(())
}
