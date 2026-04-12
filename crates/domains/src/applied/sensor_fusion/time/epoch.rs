use crate::formal::math::temporal::instant::Instant;
use crate::formal::math::temporal::time_system::TimeSystem;

use crate::applied::sensor_fusion::sensor::modality::SensorType;

/// A fusion-aware timestamp: an instant qualified by the sensor that produced it.
///
/// In multi-sensor fusion, every measurement arrives with a timestamp from a
/// specific sensor clock. The FusionEpoch pairs the temporal instant with the
/// sensor identity so the fusion engine can:
/// 1. Identify clock offset/drift for each sensor
/// 2. Detect stale measurements (sensor dropout)
/// 3. Order measurements across sensors for sequential processing
///
/// Source: Bar-Shalom et al. (2001), Section 6.2 — "Measurement time alignment."
#[derive(Debug, Clone, PartialEq)]
pub struct FusionEpoch {
    /// The temporal instant (seconds in a time system).
    pub instant: Instant,
    /// Which sensor produced this timestamp.
    pub sensor: SensorType,
}

impl FusionEpoch {
    /// Create a new fusion epoch.
    pub fn new(instant: Instant, sensor: SensorType) -> Self {
        Self { instant, sensor }
    }

    /// Create from raw seconds in GPS time.
    pub fn from_gps_seconds(seconds: f64, sensor: SensorType) -> Self {
        Self {
            instant: Instant::new(seconds, TimeSystem::GPS),
            sensor,
        }
    }

    /// Age of this measurement relative to a reference time (seconds).
    /// Returns positive if the measurement is older than the reference.
    pub fn age(&self, reference: &Instant) -> Option<f64> {
        self.instant.duration_to(reference).map(|d| d.seconds())
    }

    /// Is this measurement stale? (older than `max_age_seconds` relative to reference)
    pub fn is_stale(&self, reference: &Instant, max_age_seconds: f64) -> Option<bool> {
        self.age(reference).map(|age| age > max_age_seconds)
    }

    /// Duration since another fusion epoch (in seconds).
    /// Returns None if time systems differ.
    pub fn duration_since(&self, other: &FusionEpoch) -> Option<f64> {
        other
            .instant
            .duration_to(&self.instant)
            .map(|d| d.seconds())
    }

    /// Is this epoch from the same sensor as another?
    pub fn same_sensor(&self, other: &FusionEpoch) -> bool {
        self.sensor == other.sensor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn age_positive_for_older_measurement() {
        let epoch = FusionEpoch::from_gps_seconds(100.0, SensorType::GnssReceiver);
        let now = Instant::new(105.0, TimeSystem::GPS);
        let age = epoch.age(&now).unwrap();
        assert!((age - 5.0).abs() < 1e-10);
    }

    #[test]
    fn is_stale_detects_old_measurement() {
        let epoch = FusionEpoch::from_gps_seconds(100.0, SensorType::GnssReceiver);
        let now = Instant::new(102.0, TimeSystem::GPS);
        assert_eq!(epoch.is_stale(&now, 1.0), Some(true));
        assert_eq!(epoch.is_stale(&now, 5.0), Some(false));
    }

    #[test]
    fn duration_since_same_system() {
        let e1 = FusionEpoch::from_gps_seconds(100.0, SensorType::IMU);
        let e2 = FusionEpoch::from_gps_seconds(100.5, SensorType::GnssReceiver);
        let dt = e2.duration_since(&e1).unwrap();
        assert!((dt - 0.5).abs() < 1e-10);
    }

    #[test]
    fn duration_since_different_system_returns_none() {
        let e1 = FusionEpoch::new(Instant::new(100.0, TimeSystem::GPS), SensorType::IMU);
        let e2 = FusionEpoch::new(Instant::new(100.0, TimeSystem::TAI), SensorType::IMU);
        assert!(e2.duration_since(&e1).is_none());
    }

    #[test]
    fn same_sensor_check() {
        let e1 = FusionEpoch::from_gps_seconds(100.0, SensorType::IMU);
        let e2 = FusionEpoch::from_gps_seconds(101.0, SensorType::IMU);
        let e3 = FusionEpoch::from_gps_seconds(101.0, SensorType::GnssReceiver);
        assert!(e1.same_sensor(&e2));
        assert!(!e1.same_sensor(&e3));
    }
}
