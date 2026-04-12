//! The probability ontology — distributions, Bayesian inference, Mahalanobis distance, estimation theory
pub mod bayesian;
pub mod distribution;
pub mod entropy;
pub mod gaussian;
pub mod mahalanobis;
pub mod ontology;

#[cfg(test)]
mod tests;
