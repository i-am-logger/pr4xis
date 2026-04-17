//! Celestial navigation ontology.
//!
//! Celestial navigation determines position by measuring angles to celestial
//! bodies (stars, sun, moon, planets). It is the oldest form of absolute
//! positioning and remains relevant for spacecraft and GNSS-denied environments.
//!
//! Source: Wertz (2001), Bowditch (2002), Groves (2013) Section 6.5.

pub mod body;
pub mod engine;
pub mod observable;
pub mod ontology;
pub mod property_functor;

#[cfg(test)]
mod tests;
