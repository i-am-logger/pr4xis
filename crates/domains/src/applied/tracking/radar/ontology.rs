#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Concept, FinitelyGenerated};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Quality};

use crate::applied::tracking::radar::engine::radar_measurement_2d;
use crate::formal::math::coordinate::SphericalCoordinate;
use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::formal::math::quantity::unit::{self, Unit};

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

impl Concept for RadarMeasurement {}
impl FinitelyGenerated for RadarMeasurement {
    fn variants() -> Vec<Self> {
        vec![Self::Range, Self::Azimuth, Self::Elevation, Self::Doppler]
    }
}

/// SI unit of each radar measurement component. The value is the typed
/// `quantity::unit::Unit` from the quantity ontology, not a prose symbol.
#[derive(Debug, Clone)]
pub struct RadarMeasurementUnit;

impl Quality for RadarMeasurementUnit {
    type Individual = RadarMeasurement;
    type Value = Unit;

    fn get(&self, m: &RadarMeasurement) -> Option<Unit> {
        Some(match m {
            RadarMeasurement::Range => unit::METER,
            RadarMeasurement::Azimuth => unit::RADIAN,
            RadarMeasurement::Elevation => unit::RADIAN,
            RadarMeasurement::Doppler => unit::METER_PER_SECOND,
        })
    }
}

/// Radar range is always non-negative.
pub struct RangeNonNegative;

impl Axiom for RangeNonNegative {
    fn verify(&self) -> Verdict {
        // Computed, not asserted: run the module's real radar observation
        // model over canonical Cartesian state fixtures and read back the
        // Range component of each polar measurement, then check the
        // non-negativity property range = sqrt(x²+y²) ≥ 0. The `>= 0.0`
        // test is a genuine threshold: a NaN or (impossibly) negative
        // range from the engine falsifies the claim and yields a
        // counterexample.

        // Cartesian state fixtures [x, vx, y, vy] spanning all four
        // quadrants plus the degenerate origin.
        let states: [Vector; 5] = [
            Vector::new(vec![0.0, 0.0, 0.0, 0.0]),
            Vector::new(vec![1000.0, 5.0, 2000.0, -3.0]),
            Vector::new(vec![-1500.0, -2.0, 800.0, 1.0]),
            Vector::new(vec![-3000.0, 0.0, -4000.0, 4.0]),
            Vector::new(vec![500.0, 7.0, -1200.0, -6.0]),
        ];

        for state in states.iter() {
            let measurement = radar_measurement_2d(state);
            // Index 0 is the Range component (RadarMeasurement::Range);
            // index 1 is Azimuth. See `engine::radar_measurement_2d`.
            let range = measurement.get(0);
            if range < 0.0 || range.is_nan() {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }

        // Cross-check the 3D spherical range: sqrt(x²+y²+z²) ≥ 0, using
        // the module's real Cartesian→spherical conversion.
        let cartesian: [(f64, f64, f64); 3] = [
            (0.0, 0.0, 0.0),
            (100.0, 200.0, -50.0),
            (-300.0, -400.0, 120.0),
        ];
        for &(x, y, z) in cartesian.iter() {
            let spherical = SphericalCoordinate::from_cartesian(&Vector::new(vec![x, y, z]));
            if spherical.range < 0.0 || spherical.range.is_nan() {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }

        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "RangeNonNegative",
        "radar range is non-negative",
        "Skolnik (2008) Introduction to Radar Systems; Bar-Shalom et al. (2001) Chapter 10"
    );
}
pr4xis::register_axiom!(
    RangeNonNegative,
    "Skolnik (2008) Introduction to Radar Systems; Bar-Shalom et al. (2001) Chapter 10"
);
