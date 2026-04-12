//! The time ontology — instants, intervals, durations, time systems (UTC/TAI/GPS), clock models, Allen's interval algebra
pub mod allen;
pub mod clock;
pub mod duration;
pub mod instant;
pub mod interval;
pub mod ontology;
pub mod time_system;

#[cfg(test)]
mod tests;
