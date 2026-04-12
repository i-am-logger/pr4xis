use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;

use crate::applied::sensor_fusion::observation::observation_model::LinearObservationModel;

/// Innovation (measurement residual): ν = z - h(x̂).
///
/// The difference between what the sensor measured and what the
/// filter predicted the sensor should measure.
///
/// Source: Kalman (1960), Bar-Shalom et al. (2001), Section 2.3.
#[derive(Debug, Clone)]
pub struct Innovation {
    /// Innovation vector ν = z - z_predicted.
    pub residual: Vector,
    /// Innovation covariance S = H P H^T + R.
    pub covariance: Matrix,
}

impl Innovation {
    /// Compute innovation from measurement, state estimate, and observation model.
    pub fn compute(
        measurement: &Vector,
        state: &Vector,
        state_covariance: &Matrix,
        model: &LinearObservationModel,
        noise_covariance: &Matrix,
    ) -> Self {
        let z_predicted = model.predict(state);
        let residual = measurement.sub(&z_predicted);
        let h = &model.matrix;
        let covariance = h
            .multiply(state_covariance)
            .multiply(&h.transpose())
            .add(noise_covariance);
        Self {
            residual,
            covariance,
        }
    }

    /// Dimension of innovation.
    pub fn dim(&self) -> usize {
        self.residual.dim()
    }
}
