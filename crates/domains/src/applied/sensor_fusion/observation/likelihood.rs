use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::formal::math::probability::gaussian::GaussianND;

use crate::applied::sensor_fusion::observation::innovation::Innovation;

/// Measurement likelihood via the probability ontology.
///
/// Delegates to GaussianND.log_pdf() — the probability ontology owns
/// the Gaussian math. The observation module USES it, not reimplements it.
///
/// Source: Bar-Shalom et al. (2001), Section 2.4.
pub fn log_likelihood(innovation: &Innovation) -> f64 {
    let gaussian = GaussianND::new(
        Vector::zeros(innovation.dim()),
        innovation.covariance.clone(),
    );
    gaussian
        .log_pdf(&innovation.residual)
        .unwrap_or(f64::NEG_INFINITY)
}

/// Likelihood (exp of log-likelihood). Use log form when possible.
pub fn likelihood(innovation: &Innovation) -> f64 {
    log_likelihood(innovation).exp()
}
