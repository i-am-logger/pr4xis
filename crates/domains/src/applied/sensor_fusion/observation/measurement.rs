#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;

use crate::applied::sensor_fusion::sensor::modality::SensorType;

/// A measurement: the raw output of a sensor at a point in time.
///
/// z_k = h(x_k) + v_k
///
/// Source: Kalman (1960), Bar-Shalom et al. (2001), Chapter 2.
#[derive(Debug, Clone)]
pub struct Measurement {
    /// Measurement vector z.
    pub value: Vector,
    /// Measurement noise covariance R.
    pub noise_covariance: Matrix,
    /// Which sensor produced this measurement.
    pub sensor: SensorType,
    /// Timestamp (seconds since epoch).
    pub timestamp: f64,
}

impl Measurement {
    pub fn new(
        value: Vector,
        noise_covariance: Matrix,
        sensor: SensorType,
        timestamp: f64,
    ) -> Self {
        assert_eq!(value.dim(), noise_covariance.rows);
        Self {
            value,
            noise_covariance,
            sensor,
            timestamp,
        }
    }

    /// Dimension of the measurement vector.
    pub fn dim(&self) -> usize {
        self.value.dim()
    }
}
