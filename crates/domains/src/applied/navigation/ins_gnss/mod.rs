//! INS/GNSS integration ontology.
//!
//! Integrating an Inertial Navigation System (INS) with GNSS combines
//! the high-rate, short-term accuracy of inertial sensors with the
//! long-term absolute accuracy of satellite positioning.
//!
//! Source: Groves (2013) Chapters 14-17, Titterton & Weston (2004) Chapter 13.

pub mod coupling;
pub mod engine;
pub mod ontology;

#[cfg(test)]
mod tests;
