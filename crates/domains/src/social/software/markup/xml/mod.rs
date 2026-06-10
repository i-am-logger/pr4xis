pub mod english_projection_v1;
pub mod lmf;
pub mod loader_v1;
pub mod mods;
pub mod ontology;
pub mod ontology_v1;
pub mod owl;
pub mod parser;
pub mod rdf;
pub mod reader;
pub mod spec_1_0;
// Source-agnostic succinct bit-packing primitives the compact `.prx` codecs
// share (bit-packed columns, gap-coded offsets, front-coded dictionaries). `prx`-
// only: the gzip wrapper needs std, and every codec that uses it is `prx`-gated.
#[cfg(feature = "prx")]
pub mod succinct;
pub mod uslm;

pub use ontology::*;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_v1;
