pub mod engine;
pub mod intersection;
pub mod ontology;
pub mod signal;

pub use intersection::{Intersection, IntersectionResult};
pub use signal::{Signal, SignalAction, SignalState};

#[cfg(test)]
mod tests;
pub(crate) use engine::*;
pub(crate) use ontology::*;
