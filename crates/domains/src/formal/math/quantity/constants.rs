use crate::formal::math::quantity::dimension::Dimension;
use crate::formal::math::quantity::value::Quantity;

/// Physical constants as ontological quantities.
///
/// Every constant carries its dimension, ensuring dimensional correctness
/// at every use site. No bare floats for physical values.
///
/// Sources:
///   CODATA 2018, IAU, SI 2019, WGS84.
/// Standard gravity (CODATA 2018): 9.80665 m/s².
pub fn standard_gravity() -> Quantity {
    Quantity::new(9.80665, Dimension::ACCELERATION)
}

/// Earth gravitational parameter (IAU): 3.986004418e14 m³/s².
///
/// Dimension: L³·T⁻².
pub fn mu_earth() -> Quantity {
    Quantity::new(
        3.986004418e14,
        Dimension {
            length: 3,
            time: -2,
            ..Dimension::DIMENSIONLESS
        },
    )
}

/// Speed of light in vacuum (exact, SI 2019): 299_792_458.0 m/s.
pub fn speed_of_light() -> Quantity {
    Quantity::new(299_792_458.0, Dimension::VELOCITY)
}

/// WGS84 semi-major axis: 6_378_137.0 m.
pub fn wgs84_a() -> Quantity {
    Quantity::new(6_378_137.0, Dimension::LENGTH)
}
