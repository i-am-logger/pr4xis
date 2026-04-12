mod action;
#[allow(clippy::module_inception)]
mod engine;
mod precondition;
mod situation;
mod trace;

pub use action::Action;
pub use engine::{Engine, EngineError};
pub use precondition::{Precondition, PreconditionResult};
pub use situation::Situation;
pub use trace::{Trace, TraceEntry};

#[cfg(test)]
mod tests;
