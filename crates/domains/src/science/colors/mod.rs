pub mod engine;
pub mod mixing;
pub mod ontology;
pub mod rgb;

pub use mixing::{MixMode, blend, complement, mix, mix_many};
pub use rgb::{Channel, Rgb};

#[cfg(test)]
pub(crate) use engine::*;

#[cfg(test)]
mod tests;
