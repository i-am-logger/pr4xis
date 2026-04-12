//! Predict step abstraction for the sensor fusion engine.
//!
//! The predict step (time update) propagates the state estimate forward in time
//! using a dynamic model. It is the first half of the Kalman filter cycle:
//!
//!   x_pred = F * x_est
//!   P_pred = F * P_est * F^T + Q
//!
//! where:
//! - F is the state transition matrix (dynamics model)
//! - Q is the process noise covariance (model uncertainty)
//!
//! Key property: prediction ALWAYS increases uncertainty (trace(P_pred) >= trace(P_est))
//! because we are propagating without new information.
//!
//! The actual implementation lives in `engine::FusionAction::Predict` and
//! `engine::apply_fusion`. This module provides the conceptual documentation
//! and convenience re-exports.
//!
//! Source: Kalman (1960), "A New Approach to Linear Filtering and Prediction Problems."
//!         Maybeck (1979), *Stochastic Models, Estimation, and Control*, Vol. 1, Ch. 5.

pub use crate::applied::sensor_fusion::fusion::engine::FusionAction;

use crate::formal::math::linear_algebra::matrix::Matrix;

/// Create a constant-velocity state transition matrix for n position dimensions.
///
/// State vector layout: [pos_1, ..., pos_n, vel_1, ..., vel_n]
/// F = [[I, dt*I], [0, I]] (2n x 2n)
///
/// This is the simplest non-trivial dynamics model for tracking.
///
/// Source: Bar-Shalom et al. (2001), Section 6.3.2.
pub fn constant_velocity_transition(n_pos: usize, dt: f64) -> Matrix {
    let dim = 2 * n_pos;
    let mut data = vec![0.0; dim * dim];
    for i in 0..dim {
        data[i * dim + i] = 1.0; // diagonal (identity blocks)
    }
    for i in 0..n_pos {
        data[i * dim + (n_pos + i)] = dt; // dt * I in upper-right block
    }
    Matrix::new(dim, dim, data)
}

/// Create a constant-velocity process noise matrix.
///
/// Q = q * [[dt^3/3 * I, dt^2/2 * I], [dt^2/2 * I, dt * I]]
///
/// where q is the continuous-time process noise spectral density.
///
/// Source: Bar-Shalom et al. (2001), Eq. 6.3.2-2.
pub fn constant_velocity_process_noise(n_pos: usize, dt: f64, q: f64) -> Matrix {
    let dim = 2 * n_pos;
    let mut data = vec![0.0; dim * dim];
    let dt2 = dt * dt;
    let dt3 = dt2 * dt;
    for i in 0..n_pos {
        // Position-position block: dt^3/3
        data[i * dim + i] = q * dt3 / 3.0;
        // Position-velocity block: dt^2/2
        data[i * dim + (n_pos + i)] = q * dt2 / 2.0;
        // Velocity-position block: dt^2/2
        data[(n_pos + i) * dim + i] = q * dt2 / 2.0;
        // Velocity-velocity block: dt
        data[(n_pos + i) * dim + (n_pos + i)] = q * dt;
    }
    Matrix::new(dim, dim, data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_velocity_transition_1d() {
        let f = constant_velocity_transition(1, 0.1);
        assert_eq!(f.rows, 2);
        assert_eq!(f.cols, 2);
        assert!((f.get(0, 0) - 1.0).abs() < 1e-10);
        assert!((f.get(0, 1) - 0.1).abs() < 1e-10);
        assert!((f.get(1, 0)).abs() < 1e-10);
        assert!((f.get(1, 1) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn constant_velocity_transition_2d() {
        let f = constant_velocity_transition(2, 1.0);
        assert_eq!(f.rows, 4);
        assert_eq!(f.cols, 4);
        // pos_x row: [1, 0, dt, 0]
        assert!((f.get(0, 2) - 1.0).abs() < 1e-10);
        assert!((f.get(0, 3)).abs() < 1e-10);
        // pos_y row: [0, 1, 0, dt]
        assert!((f.get(1, 2)).abs() < 1e-10);
        assert!((f.get(1, 3) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn process_noise_is_symmetric() {
        let q = constant_velocity_process_noise(2, 0.1, 1.0);
        for i in 0..q.rows {
            for j in 0..q.cols {
                assert!(
                    (q.get(i, j) - q.get(j, i)).abs() < 1e-12,
                    "Q must be symmetric: Q[{},{}]={} != Q[{},{}]={}",
                    i,
                    j,
                    q.get(i, j),
                    j,
                    i,
                    q.get(j, i)
                );
            }
        }
    }

    #[test]
    fn process_noise_positive_diagonal() {
        let q = constant_velocity_process_noise(3, 0.5, 2.0);
        for i in 0..q.rows {
            assert!(
                q.get(i, i) > 0.0,
                "Q diagonal must be positive: Q[{},{}]={}",
                i,
                i,
                q.get(i, i)
            );
        }
    }
}
