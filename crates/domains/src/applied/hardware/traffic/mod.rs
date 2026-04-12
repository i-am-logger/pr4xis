pub mod engine;
pub mod intersection;
pub mod ontology;
pub mod signal;

pub use intersection::{Intersection, IntersectionResult};
pub use signal::{Signal, SignalAction, SignalState};

#[cfg(test)]
pub(crate) use engine::*;

#[cfg(test)]
mod tests;
