mod chord;
mod interval;
mod note;
mod scale;

pub use chord::{Chord, ChordKind, ChordResult};
pub use interval::Interval;
pub use note::Note;
pub use scale::{Scale, ScaleKind};

pub mod engine;
pub mod ontology;

#[cfg(test)]
mod tests;
