use crate::natural::geodesy::coordinate::{Ecef, Geodetic, Ned};
use crate::natural::geodesy::ellipsoid::Ellipsoid;

/// Convert geodetic (lat, lon, alt) to ECEF (x, y, z).
///
/// Source: Torge & Müller, *Geodesy* (2012), Eq. 5.25.
///         Bowring, B.R. (1976). "Transformation from spatial to geographical coordinates."
pub fn geodetic_to_ecef(geo: &Geodetic, ellipsoid: &Ellipsoid) -> Ecef {
    let sin_lat = geo.lat.sin();
    let cos_lat = geo.lat.cos();
    let sin_lon = geo.lon.sin();
    let cos_lon = geo.lon.cos();

    let n = ellipsoid.prime_vertical_radius(geo.lat);

    Ecef {
        x: (n + geo.alt) * cos_lat * cos_lon,
        y: (n + geo.alt) * cos_lat * sin_lon,
        z: (n * (1.0 - ellipsoid.e_squared()) + geo.alt) * sin_lat,
    }
}

/// Convert ECEF (x, y, z) to geodetic (lat, lon, alt).
///
/// Uses Bowring's iterative method (converges in 2-3 iterations for Earth).
///
/// Source: Bowring, B.R. (1976). "Transformation from spatial to geographical coordinates."
///         Survey Review, 23(181):323-327.
pub fn ecef_to_geodetic(ecef: &Ecef, ellipsoid: &Ellipsoid) -> Geodetic {
    let a = ellipsoid.a;
    let b = ellipsoid.b();
    let e2 = ellipsoid.e_squared();

    let p = (ecef.x * ecef.x + ecef.y * ecef.y).sqrt();
    let lon = ecef.y.atan2(ecef.x);

    // Bowring's initial estimate
    let theta = (ecef.z * a).atan2(p * b);
    let sin_theta = theta.sin();
    let cos_theta = theta.cos();

    let ep2 = (a * a - b * b) / (b * b);

    let lat = (ecef.z + ep2 * b * sin_theta * sin_theta * sin_theta)
        .atan2(p - e2 * a * cos_theta * cos_theta * cos_theta);

    let sin_lat = lat.sin();
    let n = ellipsoid.prime_vertical_radius(lat);

    let cos_lat = lat.cos();
    let alt = if cos_lat.abs() > 1e-10 {
        p / lat.cos() - n
    } else {
        ecef.z / sin_lat - n * (1.0 - e2)
    };

    Geodetic { lat, lon, alt }
}

/// Compute the DCM that rotates from ECEF to NED at a given geodetic position.
///
/// R_ned_ecef: transforms a vector from ECEF frame to NED frame.
///
/// Source: Groves (2013), Eq. 2.150.
pub fn ecef_to_ned_rotation(lat: f64, lon: f64) -> [[f64; 3]; 3] {
    let sl = lat.sin();
    let cl = lat.cos();
    let sn = lon.sin();
    let cn = lon.cos();

    [
        [-sl * cn, -sl * sn, cl],
        [-sn, cn, 0.0],
        [-cl * cn, -cl * sn, -sl],
    ]
}

/// Convert ECEF difference to NED relative to a reference point.
pub fn ecef_to_ned(ecef: &Ecef, ref_point: &Geodetic) -> Ned {
    let ref_ecef = geodetic_to_ecef(ref_point, &crate::natural::geodesy::ellipsoid::wgs84());
    let dx = ecef.x - ref_ecef.x;
    let dy = ecef.y - ref_ecef.y;
    let dz = ecef.z - ref_ecef.z;

    let r = ecef_to_ned_rotation(ref_point.lat, ref_point.lon);

    Ned {
        north: r[0][0] * dx + r[0][1] * dy + r[0][2] * dz,
        east: r[1][0] * dx + r[1][1] * dy + r[1][2] * dz,
        down: r[2][0] * dx + r[2][1] * dy + r[2][2] * dz,
    }
}

/// Great circle distance between two geodetic points (Haversine formula).
///
/// Returns distance in meters on WGS84 ellipsoid (spherical approximation).
///
/// Source: Sinnott, R.W. (1984). "Virtues of the Haversine." Sky and Telescope.
pub fn great_circle_distance(a: &Geodetic, b: &Geodetic, ellipsoid: &Ellipsoid) -> f64 {
    let dlat = b.lat - a.lat;
    let dlon = b.lon - a.lon;
    let slat = (dlat / 2.0).sin();
    let slon = (dlon / 2.0).sin();
    let h = slat * slat + a.lat.cos() * b.lat.cos() * slon * slon;
    2.0 * ellipsoid.a * h.sqrt().asin()
}

/// Initial bearing from point a to point b (forward azimuth).
///
/// Returns bearing in radians from north (0 = north, π/2 = east).
pub fn initial_bearing(a: &Geodetic, b: &Geodetic) -> f64 {
    let dlon = b.lon - a.lon;
    let y = dlon.sin() * b.lat.cos();
    let x = a.lat.cos() * b.lat.sin() - a.lat.sin() * b.lat.cos() * dlon.cos();
    y.atan2(x)
}
