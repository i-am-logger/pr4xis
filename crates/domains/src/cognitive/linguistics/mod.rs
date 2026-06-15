// `composed` grounds loaded `.prx` ontologies into the English lexicon and
// presents the union as one `LexicalReasoner`. It consumes `pr4xis-runtime`'s
// `RuntimeOntology` (a std crate), so it is `std`-gated — consistent with the
// crate's other std-only surfaces. The WASM build enables `std` via `prx`, so
// the browser chat still gets it.
#[cfg(feature = "std")]
pub mod composed;
pub mod english;
pub mod grammar;
pub mod lambek;
pub mod language;
pub mod lemon;
pub mod lexicon;

pub mod morphology;
pub mod orthography;
pub mod pipeline;
pub mod pragmatics;
pub mod relation_lexicon;
pub mod semantics;
pub mod symbols;
pub mod text;
pub mod wordnet;
