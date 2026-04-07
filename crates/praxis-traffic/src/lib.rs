mod intersection;
mod signal;

pub use intersection::{Intersection, IntersectionResult};
pub use signal::{Signal, SignalAction, SignalState};

pub mod engine;
pub mod ontology;

#[cfg(test)]
mod tests;
