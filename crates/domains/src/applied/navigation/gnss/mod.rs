//! GNSS navigation ontology.
//!
//! Global Navigation Satellite System (GNSS) positioning provides absolute
//! position fixes by measuring pseudoranges to orbiting satellites.
//!
//! Source: IS-GPS-200 (2022), Groves (2013) Chapter 8,
//!         Misra & Enge (2011), Kaplan & Hegarty (2006).

pub mod engine;
pub mod ontology;

#[cfg(test)]
mod tests;
