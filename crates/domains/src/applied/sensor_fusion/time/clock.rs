#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;
use crate::formal::math::temporal::clock::ClockModel;
use crate::formal::math::temporal::duration::Duration;

use crate::applied::sensor_fusion::sensor::modality::SensorType;

/// Sensor clock model: wraps the time ontology's ClockModel with sensor identity.
///
/// Each sensor has its own clock with its own error characteristics. The
/// SensorClock tracks:
/// - Which sensor owns this clock
/// - The clock error model (bias, drift, noise)
/// - The estimated offset from system time
///
/// This is essential for multi-sensor fusion: measurements from different
/// sensors arrive with timestamps from different clocks that must be
/// reconciled to a common time base.
///
/// Source: Groves (2013), Section 9.2 — "Receiver clock modelling."
///         Riley (2008), NIST SP 1065 — "Handbook of Frequency Stability Analysis."
#[derive(Debug, Clone, PartialEq)]
pub struct SensorClock {
    /// Which sensor this clock belongs to.
    pub sensor: SensorType,
    /// The clock error model (bias, drift, noise characteristics).
    pub model: ClockModel,
    /// Estimated offset from system time.
    /// Positive means sensor clock is ahead of system time.
    pub offset_from_system: Duration,
}

impl SensorClock {
    /// Create a new sensor clock.
    pub fn new(sensor: SensorType, model: ClockModel, offset_from_system: Duration) -> Self {
        Self {
            sensor,
            model,
            offset_from_system,
        }
    }

    /// Create an ideal sensor clock (no errors, zero offset).
    pub fn ideal(sensor: SensorType) -> Self {
        Self {
            sensor,
            model: ClockModel::ideal(),
            offset_from_system: Duration::zero(),
        }
    }

    /// Convert a sensor timestamp to system time.
    ///
    /// system_time = sensor_time - offset
    pub fn to_system_time(&self, sensor_time: f64) -> Quantity {
        Quantity::from_unit(
            sensor_time - self.offset_from_system.seconds(),
            &unit::SECOND,
        )
    }

    /// Convert a system timestamp to sensor time.
    ///
    /// sensor_time = system_time + offset
    pub fn from_system_time(&self, system_time: f64) -> Quantity {
        Quantity::from_unit(
            system_time + self.offset_from_system.seconds(),
            &unit::SECOND,
        )
    }

    /// Predicted clock error at elapsed time since last calibration.
    pub fn predicted_error(&self, elapsed: &Duration) -> Quantity {
        Quantity::from_unit(self.model.error_at(elapsed), &unit::SECOND)
    }

    /// Allan deviation at averaging time tau.
    ///
    /// Dimensionless by convention (same family as
    /// `formal::math::temporal::clock::ClockModel::allan_deviation`).
    pub fn allan_deviation(&self, tau: f64) -> Quantity {
        self.model.allan_deviation(tau)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ideal_clock_zero_offset() {
        let clock = SensorClock::ideal(SensorType::GnssReceiver);
        assert!((clock.offset_from_system.seconds()).abs() < 1e-15);
        assert!((clock.to_system_time(100.0).value - 100.0).abs() < 1e-15);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn round_trip_time_conversion() {
        let clock = SensorClock::new(
            SensorType::IMU,
            ClockModel::ideal(),
            Duration::from_seconds(0.005), // 5ms ahead
        );
        let system_time = 1000.0;
        let sensor_time = clock.from_system_time(system_time);
        let recovered = clock.to_system_time(sensor_time.value);
        assert!((recovered.value - system_time).abs() < 1e-12);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn offset_correction() {
        let clock = SensorClock::new(
            SensorType::GnssReceiver,
            ClockModel::ideal(),
            Duration::from_seconds(0.1),
        );
        // Sensor says 100.1, but system time is 100.0
        assert!((clock.to_system_time(100.1).value - 100.0).abs() < 1e-12);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn predicted_error_ideal_is_zero() {
        let clock = SensorClock::ideal(SensorType::IMU);
        let error = clock.predicted_error(&Duration::from_seconds(100.0));
        assert!(error.value.abs() < 1e-15);
    }
}
