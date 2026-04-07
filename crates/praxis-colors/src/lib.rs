mod mixing;
mod rgb;

pub use mixing::{MixMode, blend, complement, mix, mix_many};
pub use rgb::{Channel, Rgb};

pub mod engine;
pub mod ontology;

#[cfg(test)]
mod tests;
