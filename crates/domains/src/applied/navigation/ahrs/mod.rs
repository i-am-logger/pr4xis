//! Attitude and Heading Reference System (AHRS) ontology.
//!
//! An AHRS fuses accelerometer, gyroscope, and magnetometer data to
//! estimate orientation (roll, pitch, yaw) without external position fixes.
//!
//! Source: Madgwick (2010), Mahony et al. (2008),
//!         Titterton & Weston (2004) Chapter 10.

pub mod engine;
pub mod ontology;

#[cfg(test)]
mod tests;
