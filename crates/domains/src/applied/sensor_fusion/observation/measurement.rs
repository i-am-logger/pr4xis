#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;

use crate::applied::sensor_fusion::time::epoch::FusionEpoch;

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
    /// The sensor and instant that produced this measurement — bundles
    /// sensor identity with the temporal instant per
    /// [`FusionEpoch`]'s
    /// own doc comment, rather than carrying the two separately.
    pub epoch: FusionEpoch,
}

impl Measurement {
    pub fn new(value: Vector, noise_covariance: Matrix, epoch: FusionEpoch) -> Self {
        assert_eq!(value.dim(), noise_covariance.rows);
        Self {
            value,
            noise_covariance,
            epoch,
        }
    }

    /// Dimension of the measurement vector.
    pub fn dim(&self) -> usize {
        self.value.dim()
    }
}
