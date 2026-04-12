pub mod chord;
pub mod engine;
pub mod interval;
pub mod note;
pub mod ontology;
pub mod scale;

pub use chord::{Chord, ChordKind, ChordResult};
pub use interval::Interval;
pub use note::Note;
pub use scale::{Scale, ScaleKind};

#[cfg(test)]
pub(crate) use engine::*;

#[cfg(test)]
mod tests;
