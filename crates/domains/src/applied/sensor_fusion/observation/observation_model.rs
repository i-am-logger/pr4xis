use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;

/// Observation model: maps state to expected measurement.
///
/// z_predicted = h(x)
///
/// For linear models: z = H * x
/// For nonlinear models: z = h(x), with Jacobian H = ∂h/∂x
///
/// Source: Kalman (1960), Bar-Shalom et al. (2001), Chapter 2.
#[derive(Debug, Clone)]
pub struct LinearObservationModel {
    /// Observation matrix H: maps state to measurement space.
    /// Dimensions: m × n where m = measurement dim, n = state dim.
    pub matrix: Matrix,
}

impl LinearObservationModel {
    pub fn new(matrix: Matrix) -> Self {
        Self { matrix }
    }

    /// Direct observation of the full state: H = I.
    pub fn identity(n: usize) -> Self {
        Self {
            matrix: Matrix::identity(n),
        }
    }

    /// Observe only position (first n/2 components of a pos+vel state).
    pub fn position_only(state_dim: usize) -> Self {
        let obs_dim = state_dim / 2;
        let mut data = vec![0.0; obs_dim * state_dim];
        for i in 0..obs_dim {
            data[i * state_dim + i] = 1.0;
        }
        Self {
            matrix: Matrix::new(obs_dim, state_dim, data),
        }
    }

    /// Predicted measurement: z_pred = H * x.
    pub fn predict(&self, state: &Vector) -> Vector {
        self.matrix.multiply_vector(state)
    }

    /// Measurement dimension.
    pub fn measurement_dim(&self) -> usize {
        self.matrix.rows
    }

    /// State dimension.
    pub fn state_dim(&self) -> usize {
        self.matrix.cols
    }
}
