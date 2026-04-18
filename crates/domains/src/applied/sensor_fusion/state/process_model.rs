#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::linear_algebra::matrix::Matrix;

/// Process model: how the state evolves over time.
///
/// x_{k+1} = F(dt) * x_k + w_k
///
/// where F is the state transition matrix and w is process noise.
///
/// This module defines the interface. Concrete models are in
/// domain crates (tracking/motion_model, navigation/imu/strapdown).
///
/// Source: Maybeck (1979), Vol. 1, Chapter 5; Bar-Shalom (2001), Chapter 6.
/// A linear process model: state transition matrix F and process noise Q.
#[derive(Debug, Clone)]
pub struct LinearProcessModel {
    /// State transition matrix F(dt).
    pub transition: Matrix,
    /// Process noise covariance Q(dt).
    pub process_noise: Matrix,
    /// Time step used to compute F and Q.
    pub dt: f64,
}

impl LinearProcessModel {
    /// Static model: state doesn't change. F = I, Q = small.
    pub fn static_model(dim: usize, noise: f64) -> Self {
        Self {
            transition: Matrix::identity(dim),
            process_noise: Matrix::identity(dim).scale(noise),
            dt: 0.0,
        }
    }

    /// State dimension.
    pub fn dim(&self) -> usize {
        self.transition.rows
    }
}
