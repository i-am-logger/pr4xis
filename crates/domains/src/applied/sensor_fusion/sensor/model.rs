#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::applied::sensor_fusion::sensor::characteristic::{
    MeasurementDimension, SensorCharacteristics,
};
use crate::applied::sensor_fusion::sensor::modality::SensorType;

// ---------------------------------------------------------------------------
// Representative sensor specifications
//
// The presets below model *representative-grade* sensors, not any specific
// commercial part — no product part numbers are implied. The noise parameters
// (noise/spectral density and bias instability) are the Allan-variance
// noise-identification coefficients defined by IEEE Std 952-1997, "IEEE
// Standard Specification Format Guide and Test Procedure for Single-Axis
// Interferometric Fiber Optic Gyros" — the standard reference for random-walk
// and bias-instability characterisation of inertial sensors. The representative
// grade figures track Groves (2013), Table 4.1 (as cited on `SensorModel`).
// ---------------------------------------------------------------------------

/// Seconds per hour. Unit conversion for spec figures quoted per hour (e.g. a
/// gyro bias instability in deg/hr) into per-second SI units.
const SECONDS_PER_HOUR: f64 = 3600.0;

// -- Tactical-grade MEMS accelerometer --------------------------------------

/// Output-data-rate range (min_hz, max_hz) of a representative tactical-grade
/// MEMS accelerometer. Representative grade figure (Groves 2013, Table 4.1).
const TACTICAL_ACCEL_SAMPLE_RATE_HZ: (f64, f64) = (100.0, 1000.0);

/// Full-scale measurement range in multiples of standard gravity g (±300 g).
/// Representative grade figure (Groves 2013, Table 4.1).
const TACTICAL_ACCEL_FULL_SCALE_G: f64 = 300.0;

/// Noise density (velocity random walk) in multiples of g per √Hz
/// (50 µg/√Hz). Random-walk / spectral-density noise coefficient per
/// IEEE Std 952-1997; representative grade figure (Groves 2013, Table 4.1).
const TACTICAL_ACCEL_NOISE_DENSITY_G: f64 = 50e-6;

/// Bias instability in multiples of g (25 µg). Allan-variance bias-instability
/// (flicker) floor per IEEE Std 952-1997; representative grade figure
/// (Groves 2013, Table 4.1).
const TACTICAL_ACCEL_BIAS_INSTABILITY_G: f64 = 25e-6;

// -- MEMS gyroscope ---------------------------------------------------------

/// Output-data-rate range (min_hz, max_hz) of a representative MEMS gyroscope.
/// Representative grade figure (Groves 2013, Table 4.1).
const MEMS_GYRO_SAMPLE_RATE_HZ: (f64, f64) = (100.0, 8000.0);

/// Full-scale angular-rate measurement range in degrees/second (±2000 °/s),
/// converted to rad/s at use. Representative grade figure (Groves 2013,
/// Table 4.1).
const MEMS_GYRO_FULL_SCALE_DEG_PER_S: f64 = 2000.0;

/// Noise density (angle random walk) in degrees/second per √Hz
/// (0.007 °/s/√Hz), converted to rad/s at use. Angle-random-walk coefficient
/// per IEEE Std 952-1997; representative grade figure (Groves 2013, Table 4.1).
const MEMS_GYRO_NOISE_DENSITY_DEG_PER_S: f64 = 0.007;

/// Bias instability in degrees/hour (0.5 °/hr), converted to rad/s at use.
/// Allan-variance bias-instability floor per IEEE Std 952-1997; representative
/// grade figure (Groves 2013, Table 4.1).
const MEMS_GYRO_BIAS_INSTABILITY_DEG_PER_HR: f64 = 0.5;

// -- GNSS receiver ----------------------------------------------------------

/// Output-data-rate range (min_hz, max_hz) of a representative GNSS receiver.
/// Representative grade figure (Groves 2013, Table 4.1).
const GNSS_SAMPLE_RATE_HZ: (f64, f64) = (1.0, 20.0);

/// Position measurement half-range in metres (1e7 m ≈ Earth-scale ECEF
/// coordinate span; used symmetrically as ±). Representative grade figure.
const GNSS_POSITION_HALF_RANGE_M: f64 = 1e7;

/// Position noise (1-sigma) in metres (1 m). Representative grade figure
/// (Groves 2013, Table 4.1).
const GNSS_NOISE_DENSITY_M: f64 = 1.0;

// -- Automotive radar -------------------------------------------------------

/// Output-data-rate range (min_hz, max_hz) of a representative 77 GHz
/// automotive radar. Representative grade figure.
const AUTOMOTIVE_RADAR_SAMPLE_RATE_HZ: (f64, f64) = (10.0, 30.0);

/// Range measurement span (near_m, far_m) of a representative 77 GHz
/// automotive radar: 0.2 m near limit to 250 m far limit. Representative
/// grade figure.
const AUTOMOTIVE_RADAR_RANGE_M: (f64, f64) = (0.2, 250.0);

/// Range noise (1-sigma) in metres (0.1 m). Representative grade figure.
const AUTOMOTIVE_RADAR_NOISE_DENSITY_M: f64 = 0.1;

/// Rich sensor model — carries physical characteristics.
///
/// Praxis principle: "Rich types, not enums with optional fields."
/// SensorType enum is the categorical entity (for taxonomy).
/// SensorModel struct carries the actual characteristics.
///
/// Source: Groves (2013), Table 4.1; Allan (1966).
#[derive(Debug, Clone, PartialEq)]
pub struct SensorModel {
    pub sensor_type: SensorType,
    pub name: &'static str,
    pub characteristics: SensorCharacteristics,
    /// Bias instability (SI units).
    pub bias_instability: f64,
}

impl SensorModel {
    /// Tactical-grade accelerometer.
    pub fn tactical_accelerometer() -> Self {
        Self {
            sensor_type: SensorType::Accelerometer,
            name: "Tactical Accelerometer",
            characteristics: SensorCharacteristics {
                measures: MeasurementDimension::Acceleration,
                axes: 3,
                sample_rate_range: TACTICAL_ACCEL_SAMPLE_RATE_HZ,
                measurement_range: (
                    -TACTICAL_ACCEL_FULL_SCALE_G
                        * crate::formal::math::quantity::constants::standard_gravity().value,
                    TACTICAL_ACCEL_FULL_SCALE_G
                        * crate::formal::math::quantity::constants::standard_gravity().value,
                ),
                typical_noise_density: TACTICAL_ACCEL_NOISE_DENSITY_G
                    * crate::formal::math::quantity::constants::standard_gravity().value,
            },
            bias_instability: TACTICAL_ACCEL_BIAS_INSTABILITY_G
                * crate::formal::math::quantity::constants::standard_gravity().value,
        }
    }

    /// MEMS gyroscope.
    pub fn mems_gyroscope() -> Self {
        Self {
            sensor_type: SensorType::Gyroscope,
            name: "MEMS Gyroscope",
            characteristics: SensorCharacteristics {
                measures: MeasurementDimension::AngularRate,
                axes: 3,
                sample_rate_range: MEMS_GYRO_SAMPLE_RATE_HZ,
                measurement_range: (
                    -MEMS_GYRO_FULL_SCALE_DEG_PER_S.to_radians(),
                    MEMS_GYRO_FULL_SCALE_DEG_PER_S.to_radians(),
                ),
                typical_noise_density: MEMS_GYRO_NOISE_DENSITY_DEG_PER_S.to_radians(),
            },
            bias_instability: MEMS_GYRO_BIAS_INSTABILITY_DEG_PER_HR.to_radians() / SECONDS_PER_HOUR,
        }
    }

    /// GNSS receiver.
    pub fn gnss_receiver() -> Self {
        Self {
            sensor_type: SensorType::GnssReceiver,
            name: "GNSS Receiver",
            characteristics: SensorCharacteristics {
                measures: MeasurementDimension::Position,
                axes: 3,
                sample_rate_range: GNSS_SAMPLE_RATE_HZ,
                measurement_range: (-GNSS_POSITION_HALF_RANGE_M, GNSS_POSITION_HALF_RANGE_M),
                typical_noise_density: GNSS_NOISE_DENSITY_M,
            },
            bias_instability: 0.0,
        }
    }

    /// Automotive radar.
    pub fn automotive_radar() -> Self {
        Self {
            sensor_type: SensorType::Radar,
            name: "77GHz Automotive Radar",
            characteristics: SensorCharacteristics {
                measures: MeasurementDimension::Range,
                axes: 2,
                sample_rate_range: AUTOMOTIVE_RADAR_SAMPLE_RATE_HZ,
                measurement_range: AUTOMOTIVE_RADAR_RANGE_M,
                typical_noise_density: AUTOMOTIVE_RADAR_NOISE_DENSITY_M,
            },
            bias_instability: 0.0,
        }
    }
}
