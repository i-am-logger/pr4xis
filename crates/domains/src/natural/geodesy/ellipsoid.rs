/// Reference ellipsoid parameters.
///
/// An oblate spheroid used as a mathematical model of the Earth's shape.
/// Different ellipsoids serve different purposes and eras.
///
/// Source: NIMA TR8350.2 (2000). "Department of Defense World Geodetic System 1984."
#[derive(Debug, Clone, PartialEq)]
pub struct Ellipsoid {
    /// Semi-major axis (equatorial radius) in meters.
    pub a: f64,
    /// Flattening: f = (a - b) / a.
    pub f: f64,
}

impl Ellipsoid {
    /// Semi-minor axis (polar radius): b = a(1 - f).
    pub fn b(&self) -> f64 {
        self.a * (1.0 - self.f)
    }

    /// First eccentricity squared: e² = 2f - f².
    pub fn e_squared(&self) -> f64 {
        2.0 * self.f - self.f * self.f
    }

    /// Second eccentricity squared: e'² = e² / (1 - e²).
    pub fn ep_squared(&self) -> f64 {
        let e2 = self.e_squared();
        e2 / (1.0 - e2)
    }

    /// Radius of curvature in the prime vertical at geodetic latitude φ.
    /// N(φ) = a / √(1 - e² sin²φ)
    pub fn prime_vertical_radius(&self, lat: f64) -> f64 {
        let sin_lat = lat.sin();
        self.a / (1.0 - self.e_squared() * sin_lat * sin_lat).sqrt()
    }

    /// Radius of curvature in the meridian at geodetic latitude φ.
    /// M(φ) = a(1 - e²) / (1 - e² sin²φ)^{3/2}
    pub fn meridian_radius(&self, lat: f64) -> f64 {
        let sin_lat = lat.sin();
        let denom = (1.0 - self.e_squared() * sin_lat * sin_lat).powf(1.5);
        self.a * (1.0 - self.e_squared()) / denom
    }
}

/// WGS84 ellipsoid (World Geodetic System 1984).
///
/// The standard reference ellipsoid for GPS and most modern systems.
///
/// Source: NIMA TR8350.2 (2000), Table 3.1.
///   a = 6378137.0 m
///   f = 1/298.257223563
pub fn wgs84() -> Ellipsoid {
    Ellipsoid {
        a: 6_378_137.0,
        f: 1.0 / 298.257_223_563,
    }
}

/// GRS80 ellipsoid (Geodetic Reference System 1980).
///
/// Virtually identical to WGS84 (differs in flattening by ~0.1 mm).
///
/// Source: Moritz, H. (2000). "Geodetic Reference System 1980."
pub fn grs80() -> Ellipsoid {
    Ellipsoid {
        a: 6_378_137.0,
        f: 1.0 / 298.257_222_101,
    }
}
