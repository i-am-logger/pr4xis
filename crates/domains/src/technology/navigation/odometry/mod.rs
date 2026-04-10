//! Odometry ontology.
//!
//! Odometry estimates position change from motion sensor data —
//! wheel encoders, visual odometry, laser odometry, or inertial dead reckoning.
//!
//! Source: Borenstein et al. (1996), Thrun, Burgard & Fox (2005) Chapter 5,
//!         Scaramuzza & Fraundorfer (2011).

pub mod engine;
pub mod ontology;

#[cfg(test)]
mod tests;
