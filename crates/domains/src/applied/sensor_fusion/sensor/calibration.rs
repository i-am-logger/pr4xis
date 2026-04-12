use crate::applied::sensor_fusion::sensor::modality::SensorType;

/// Sensor calibration — intrinsic and extrinsic parameters.
///
/// Intrinsic: factory calibration (bias, scale factor, misalignment).
/// Extrinsic: mounting calibration (lever arm, boresight).
///
/// Rich type: carries the full calibration state, not just an enum.
///
/// Source: Groves (2013), Section 14.2.
#[derive(Debug, Clone, PartialEq)]
pub struct SensorCalibration {
    pub sensor_type: SensorType,
    /// Bias offsets per axis (SI units).
    pub bias: Vec<f64>,
    /// Scale factor errors per axis (dimensionless, e.g., 1.001 = 0.1% error).
    pub scale_factor: Vec<f64>,
    /// Cross-axis misalignment matrix (identity = no misalignment).
    pub misalignment: Vec<Vec<f64>>,
    /// Is this calibration from factory (true) or estimated online (false)?
    pub is_factory: bool,
}

impl SensorCalibration {
    /// Perfect calibration (no errors).
    pub fn ideal(sensor_type: SensorType, axes: usize) -> Self {
        Self {
            sensor_type,
            bias: vec![0.0; axes],
            scale_factor: vec![1.0; axes],
            misalignment: (0..axes)
                .map(|i| {
                    let mut row = vec![0.0; axes];
                    row[i] = 1.0;
                    row
                })
                .collect(),
            is_factory: true,
        }
    }

    /// Apply calibration correction to a raw measurement.
    /// corrected = M * (scale .* raw - bias)
    pub fn correct(&self, raw: &[f64]) -> Vec<f64> {
        let n = raw.len();
        assert_eq!(n, self.bias.len());
        // Scale and remove bias
        let scaled: Vec<f64> = raw
            .iter()
            .zip(&self.scale_factor)
            .zip(&self.bias)
            .map(|((r, s), b)| r * s - b)
            .collect();
        // Apply misalignment correction
        (0..n)
            .map(|i| {
                self.misalignment[i]
                    .iter()
                    .zip(&scaled)
                    .map(|(m, s)| m * s)
                    .sum()
            })
            .collect()
    }
}
