use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::formal::math::statistics::confidence;

/// A state estimate: the central object of sensor fusion.
///
/// This is what sensor fusion PRODUCES: a best estimate of reality
/// with quantified uncertainty.
///
/// x̂ = state vector (position, velocity, attitude, biases, ...)
/// P = error covariance matrix (uncertainty)
///
/// Source: Kalman (1960), Maybeck (1979).
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct StateEstimate {
    /// State vector x̂.
    pub state: Vector,
    /// Error covariance matrix P (must be symmetric PSD).
    pub covariance: Matrix,
    /// Current timestamp (seconds since epoch).
    pub epoch: f64,
    /// Step counter.
    pub step: usize,
}

impl StateEstimate {
    pub fn new(state: Vector, covariance: Matrix, epoch: f64) -> Self {
        debug_assert_eq!(state.dim(), covariance.rows);
        debug_assert_eq!(covariance.rows, covariance.cols);
        Self {
            state,
            covariance,
            epoch,
            step: 0,
        }
    }

    /// Dimension of the state vector.
    pub fn dim(&self) -> usize {
        self.state.dim()
    }

    /// Trace of covariance (total uncertainty).
    pub fn uncertainty(&self) -> f64 {
        self.covariance.trace()
    }

    /// Confidence interval for state component at the given index.
    ///
    /// Uses the statistics ontology to compute the interval from the
    /// estimator's distribution: the state component has mean x̂[i] and
    /// standard deviation sqrt(P[i,i]).
    ///
    /// The confidence level (e.g. 0.95 for 95%) determines the z-score
    /// used to scale the standard deviation.
    ///
    /// Source: Maybeck (1979), Chapter 1 (state estimation uncertainty).
    ///         Fisher (1925) / Neyman (1937) (confidence intervals).
    pub fn confidence_interval(
        &self,
        index: usize,
        level: f64,
    ) -> Option<confidence::ConfidenceInterval> {
        if index >= self.dim() {
            return None;
        }
        let z = confidence::z_score_for_level(level)?;
        let mean = self.state.get(index);
        let std_dev = self.covariance.get(index, index).sqrt();
        Some(confidence::confidence_interval_for_mean(
            mean, std_dev, z, level,
        ))
    }
}

impl PartialEq for StateEstimate {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state && self.covariance == other.covariance
    }
}
