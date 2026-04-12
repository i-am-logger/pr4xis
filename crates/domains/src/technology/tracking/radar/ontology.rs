use pr4xis::category::Entity;
use pr4xis::ontology::{Axiom, Quality};

/// Radar measurement components.
///
/// A radar measures range, bearing, and optionally elevation and Doppler.
///
/// Source: Bar-Shalom et al. (2001), Chapter 10.
///         Skolnik (2008), *Introduction to Radar Systems*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RadarMeasurement {
    /// Slant range (meters).
    Range,
    /// Azimuth bearing (radians from north).
    Azimuth,
    /// Elevation angle (radians from horizontal).
    Elevation,
    /// Doppler radial velocity (m/s).
    Doppler,
}

impl Entity for RadarMeasurement {
    fn variants() -> Vec<Self> {
        vec![Self::Range, Self::Azimuth, Self::Elevation, Self::Doppler]
    }
}

#[derive(Debug, Clone)]
pub struct RadarMeasurementUnit;

impl Quality for RadarMeasurementUnit {
    type Individual = RadarMeasurement;
    type Value = &'static str;

    fn get(&self, m: &RadarMeasurement) -> Option<&'static str> {
        Some(match m {
            RadarMeasurement::Range => "meters",
            RadarMeasurement::Azimuth => "radians",
            RadarMeasurement::Elevation => "radians",
            RadarMeasurement::Doppler => "m/s",
        })
    }
}

/// Radar range is always non-negative.
pub struct RangeNonNegative;

impl Axiom for RangeNonNegative {
    fn description(&self) -> &str {
        "radar range is non-negative"
    }
    fn holds(&self) -> bool {
        true
    } // structural: range = sqrt(x²+y²+z²) ≥ 0
}
