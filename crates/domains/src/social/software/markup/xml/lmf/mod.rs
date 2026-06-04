pub mod dtd;
// The WN-LMF byte-exact graph-faithful lens + its harness registration (flips
// `english_wordnet` off the universal floor in the completeness meter). Native
// only — `register_lens!`'s `linkme` distributed slice is unsupported on wasm32,
// and the round-trip harness it feeds is a native CI/audit tool.
#[cfg(not(target_arch = "wasm32"))]
pub mod lens;
pub mod ontology;
#[cfg(feature = "prx")]
pub mod prx;
pub mod reader;
pub mod writer;

pub use dtd::{WN_LMF_1_3_DTD, is_wn_lmf_element, loaded_wn_lmf_dtd};
pub use ontology::*;

#[cfg(test)]
mod tests;
