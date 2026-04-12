use crate::formal::math::linear_algebra::decomposition;
use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;

use crate::applied::sensor_fusion::state::estimate::StateEstimate;

/// Information form of the state estimate.
///
/// Y = P^{-1} (information matrix)
/// y = Y * x̂  (information vector)
///
/// The information form is additive: fusing two independent estimates
/// is just Y_fused = Y1 + Y2, y_fused = y1 + y2.
/// This is the mathematical basis of decentralized fusion.
///
/// Source: Maybeck (1979), Vol. 1; Mutambara (1998).
#[derive(Debug, Clone)]
pub struct InformationEstimate {
    /// Information matrix Y = P^{-1}.
    pub information_matrix: Matrix,
    /// Information vector y = P^{-1} * x̂.
    pub information_vector: Vector,
}

impl InformationEstimate {
    /// Convert from standard (mean, covariance) form.
    pub fn from_estimate(est: &StateEstimate) -> Option<Self> {
        let y_mat = decomposition::inverse_spd(&est.covariance)?;
        let y_vec = y_mat.multiply_vector(&est.state);
        Some(Self {
            information_matrix: y_mat,
            information_vector: y_vec,
        })
    }

    /// Convert back to standard form.
    pub fn to_estimate(&self, epoch: f64) -> Option<StateEstimate> {
        let p = decomposition::inverse_spd(&self.information_matrix)?;
        let x = p.multiply_vector(&self.information_vector);
        Some(StateEstimate::new(x, p, epoch))
    }

    /// Fuse two independent estimates (additive in information form).
    pub fn fuse(&self, other: &Self) -> Self {
        Self {
            information_matrix: self.information_matrix.add(&other.information_matrix),
            information_vector: self.information_vector.add(&other.information_vector),
        }
    }

    /// Dimension.
    pub fn dim(&self) -> usize {
        self.information_vector.dim()
    }
}
