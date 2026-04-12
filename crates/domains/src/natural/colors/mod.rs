pub mod engine;
pub mod mixing;
pub mod ontology;
pub mod rgb;
pub mod srgb;

pub use mixing::{MixMode, blend, complement, mix, mix_many};
pub use rgb::{Channel, Rgb};
pub use srgb::{WcagLevel, contrast_ratio, is_dark, relative_luminance, wcag_compliant};

#[cfg(test)]
pub(crate) use engine::*;

#[cfg(test)]
mod tests;
