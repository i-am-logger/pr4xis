//! Update step abstraction for the sensor fusion engine.
//!
//! The update step (measurement update) incorporates a new observation into
//! the state estimate. It is the second half of the Kalman filter cycle:
//!
//!   Innovation:  nu = z - H * x_pred
//!   Kalman gain: K = P_pred * H^T * (H * P_pred * H^T + R)^{-1}
//!   State:       x_est = x_pred + K * nu
//!   Covariance:  P_est = (I - K*H) * P_pred * (I - K*H)^T + K * R * K^T  (Joseph form)
//!
//! where:
//! - H is the observation matrix (what the sensor measures)
//! - z is the measurement vector
//! - R is the measurement noise covariance
//!
//! Key property: update ALWAYS decreases uncertainty (trace(P_est) <= trace(P_pred))
//! because we gain information from the measurement.
//!
//! The actual implementation lives in `engine::FusionAction::Update` and
//! `engine::apply_fusion`. This module provides the conceptual documentation
//! and convenience re-exports.
//!
//! Source: Kalman (1960), "A New Approach to Linear Filtering and Prediction Problems."
//!         Maybeck (1979), *Stochastic Models, Estimation, and Control*, Vol. 1, Ch. 7.

pub use crate::applied::sensor_fusion::fusion::engine::FusionAction;

use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;

/// Compute the innovation (measurement residual): nu = z - H * x.
///
/// The innovation tells us how much the measurement disagrees with
/// the predicted measurement. A large innovation means either:
/// 1. The state estimate is poor (needs more updates)
/// 2. The measurement is an outlier (should be gated)
///
/// Source: Bar-Shalom et al. (2001), Section 5.2.1.
pub fn innovation(observation_matrix: &Matrix, state: &Vector, measurement: &Vector) -> Vector {
    let predicted = observation_matrix.multiply_vector(state);
    measurement.sub(&predicted)
}

/// Compute the innovation covariance: S = H * P * H^T + R.
///
/// The innovation covariance determines the expected spread of innovations.
/// Used for gating (rejecting outliers) and NIS (Normalized Innovation Squared).
///
/// Source: Bar-Shalom et al. (2001), Section 5.2.2.
pub fn innovation_covariance(
    observation_matrix: &Matrix,
    covariance: &Matrix,
    measurement_noise: &Matrix,
) -> Matrix {
    observation_matrix
        .multiply(covariance)
        .multiply(&observation_matrix.transpose())
        .add(measurement_noise)
}

/// Direct position observation matrix for n-dimensional state with n position states.
///
/// H = [I_n | 0] — observes position directly, not velocity.
/// State: [pos_1, ..., pos_n, vel_1, ..., vel_n]
///
/// Source: Bar-Shalom et al. (2001), Section 6.3.3.
pub fn position_observation_matrix(n_pos: usize, state_dim: usize) -> Matrix {
    let mut data = vec![0.0; n_pos * state_dim];
    for i in 0..n_pos {
        data[i * state_dim + i] = 1.0;
    }
    Matrix::new(n_pos, state_dim, data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn innovation_zero_when_perfect() {
        let h = Matrix::new(1, 1, vec![1.0]);
        let x = Vector::new(vec![5.0]);
        let z = Vector::new(vec![5.0]);
        let nu = innovation(&h, &x, &z);
        assert!((nu.get(0)).abs() < 1e-10);
    }

    #[test]
    fn innovation_nonzero_on_mismatch() {
        let h = Matrix::new(1, 1, vec![1.0]);
        let x = Vector::new(vec![3.0]);
        let z = Vector::new(vec![5.0]);
        let nu = innovation(&h, &x, &z);
        assert!((nu.get(0) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn innovation_covariance_includes_both_sources() {
        let h = Matrix::new(1, 1, vec![1.0]);
        let p = Matrix::new(1, 1, vec![4.0]);
        let r = Matrix::new(1, 1, vec![1.0]);
        let s = innovation_covariance(&h, &p, &r);
        // S = H*P*H^T + R = 4 + 1 = 5
        assert!((s.get(0, 0) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn position_observation_2d_state4() {
        let h = position_observation_matrix(2, 4);
        assert_eq!(h.rows, 2);
        assert_eq!(h.cols, 4);
        // H = [[1,0,0,0],[0,1,0,0]]
        assert!((h.get(0, 0) - 1.0).abs() < 1e-10);
        assert!((h.get(0, 1)).abs() < 1e-10);
        assert!((h.get(0, 2)).abs() < 1e-10);
        assert!((h.get(1, 1) - 1.0).abs() < 1e-10);
        assert!((h.get(1, 2)).abs() < 1e-10);
        assert!((h.get(1, 3)).abs() < 1e-10);
    }
}
