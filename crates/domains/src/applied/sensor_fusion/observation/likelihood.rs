use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::formal::math::probability::gaussian::GaussianND;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

use crate::applied::sensor_fusion::observation::innovation::Innovation;

/// Measurement likelihood via the probability ontology.
///
/// Delegates to GaussianND.log_pdf() — the probability ontology owns
/// the Gaussian math. The observation module USES it, not reimplements it.
///
/// A log-likelihood is the log of a (dimensionless) probability density
/// ratio, and is by convention treated as dimensionless — the same
/// treatment `Dimension::INFORMATION` already gives log-based quantities
/// (ISO/IEC 80000-13:2008 item 13-24; Shannon 1948).
///
/// Source: Bar-Shalom et al. (2001), Section 2.4.
pub fn log_likelihood(innovation: &Innovation) -> Quantity {
    let gaussian = GaussianND::new(
        Vector::zeros(innovation.dim()),
        innovation.covariance.clone(),
    );
    Quantity::from_unit(
        gaussian
            .log_pdf(&innovation.residual)
            .unwrap_or(f64::NEG_INFINITY),
        &unit::UNITLESS,
    )
}

/// Likelihood (exp of log-likelihood). Use log form when possible.
pub fn likelihood(innovation: &Innovation) -> Quantity {
    Quantity::from_unit(log_likelihood(innovation).value.exp(), &unit::UNITLESS)
}
