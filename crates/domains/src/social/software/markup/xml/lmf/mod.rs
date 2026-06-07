pub mod dtd;
// The compact interned encoding of a WN-LMF `WordNet` — the size-reduced `.prx`
// ontology core (string pool + u32 handles). `prx`-only: it carries rkyv derives
// and is the on-the-wire/embedded compact form of the ontology.
#[cfg(feature = "prx")]
pub mod compact;
// The succinct `.prx` codec (bit-packed CSR structure) + `.prx.gz` emit/load.
#[cfg(feature = "prx")]
pub mod compact_succinct;
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
