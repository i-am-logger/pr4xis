#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::applied::sensor_fusion::sensor::characteristic::{
    MeasurementDimension, SensorCharacteristics,
};
use crate::applied::sensor_fusion::sensor::modality::SensorType;

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
                sample_rate_range: (100.0, 1000.0),
                measurement_range: (
                    -300.0 * crate::formal::math::quantity::constants::standard_gravity().value,
                    300.0 * crate::formal::math::quantity::constants::standard_gravity().value,
                ),
                typical_noise_density: 50e-6
                    * crate::formal::math::quantity::constants::standard_gravity().value,
            },
            bias_instability: 25e-6
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
                sample_rate_range: (100.0, 8000.0),
                measurement_range: (-2000.0_f64.to_radians(), 2000.0_f64.to_radians()),
                typical_noise_density: 0.007_f64.to_radians(),
            },
            bias_instability: 0.5_f64.to_radians() / 3600.0,
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
                sample_rate_range: (1.0, 20.0),
                measurement_range: (-1e7, 1e7),
                typical_noise_density: 1.0,
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
                sample_rate_range: (10.0, 30.0),
                measurement_range: (0.2, 250.0),
                typical_noise_density: 0.1,
            },
            bias_instability: 0.0,
        }
    }
}
