/// Convert polar (range, azimuth) to Cartesian (x, y).
///
/// x = r * sin(az)  (east)
/// y = r * cos(az)  (north)
///
/// Convention: azimuth measured clockwise from north.
pub fn polar_to_cartesian_2d(range: f64, azimuth: f64) -> (f64, f64) {
    let x = range * azimuth.sin();
    let y = range * azimuth.cos();
    (x, y)
}

/// Convert Cartesian (x, y) to polar (range, azimuth).
pub fn cartesian_to_polar_2d(x: f64, y: f64) -> (f64, f64) {
    let range = (x * x + y * y).sqrt();
    let azimuth = x.atan2(y); // atan2(east, north) = azimuth from north
    (range, azimuth)
}

/// Convert spherical (range, azimuth, elevation) to Cartesian (x, y, z).
///
/// x = r * cos(el) * sin(az)
/// y = r * cos(el) * cos(az)
/// z = r * sin(el)
pub fn spherical_to_cartesian(range: f64, azimuth: f64, elevation: f64) -> (f64, f64, f64) {
    let cos_el = elevation.cos();
    let x = range * cos_el * azimuth.sin();
    let y = range * cos_el * azimuth.cos();
    let z = range * elevation.sin();
    (x, y, z)
}

/// Convert Cartesian (x, y, z) to spherical (range, azimuth, elevation).
pub fn cartesian_to_spherical(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let range = (x * x + y * y + z * z).sqrt();
    let azimuth = x.atan2(y);
    let elevation = if range > 1e-15 {
        (z / range).asin()
    } else {
        0.0
    };
    (range, azimuth, elevation)
}
