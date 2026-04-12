/// Structural health monitoring data processing.
///
/// Source: Farrar & Worden (2007), "An Introduction to Structural Health Monitoring"
/// A strain measurement reading.
#[derive(Debug, Clone)]
pub struct StrainReading {
    /// Strain in microstrain (1e-6).
    pub microstrain: f64,
    /// Sensor location identifier.
    pub sensor_id: usize,
    /// Timestamp in seconds.
    pub timestamp: f64,
}

/// Damage index computed from sensor data.
#[derive(Debug, Clone)]
pub struct DamageIndex {
    pub value: f64,
    pub location: usize,
    pub severity: DamageSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DamageSeverity {
    None,
    Minor,
    Moderate,
    Severe,
    Critical,
}

/// Check if strain is within elastic limits.
///
/// yield_microstrain: typical yield strain for the material (e.g., 2000 for steel).
pub fn is_elastic(microstrain: f64, yield_microstrain: f64) -> bool {
    microstrain.abs() < yield_microstrain
}

/// Compute damage index from a set of strain readings.
///
/// Uses the ratio of maximum strain to yield strain as a simple damage indicator.
pub fn compute_damage_index(
    readings: &[StrainReading],
    yield_microstrain: f64,
) -> Option<DamageIndex> {
    if readings.is_empty() {
        return None;
    }
    let max_reading = readings
        .iter()
        .max_by(|a, b| {
            a.microstrain
                .abs()
                .partial_cmp(&b.microstrain.abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap();

    let ratio = max_reading.microstrain.abs() / yield_microstrain;
    let severity = classify_severity(ratio);

    Some(DamageIndex {
        value: ratio,
        location: max_reading.sensor_id,
        severity,
    })
}

/// Classify damage severity from strain ratio.
pub fn classify_severity(strain_ratio: f64) -> DamageSeverity {
    if strain_ratio < 0.3 {
        DamageSeverity::None
    } else if strain_ratio < 0.6 {
        DamageSeverity::Minor
    } else if strain_ratio < 0.8 {
        DamageSeverity::Moderate
    } else if strain_ratio < 1.0 {
        DamageSeverity::Severe
    } else {
        DamageSeverity::Critical
    }
}

/// Compute RMS strain from a time series.
pub fn rms_strain(readings: &[StrainReading]) -> f64 {
    if readings.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = readings.iter().map(|r| r.microstrain * r.microstrain).sum();
    (sum_sq / readings.len() as f64).sqrt()
}
