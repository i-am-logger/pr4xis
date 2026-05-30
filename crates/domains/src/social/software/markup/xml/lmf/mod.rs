pub mod dtd;
pub mod ontology;
pub mod reader;

pub use dtd::{WN_LMF_1_3_DTD, is_wn_lmf_element, loaded_wn_lmf_dtd};
pub use ontology::*;

#[cfg(test)]
mod tests;
