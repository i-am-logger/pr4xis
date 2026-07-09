//! Ontology-agnostic data produced by codegen.
//!
//! [`CodegenData`] is the build-time→runtime interchange shape: codegen
//! emits a `static CODEGEN_DATA: CodegenData<P>` populated with frozen
//! `&'static` slices; the runtime `from_codegen` functor walks those
//! slices to materialize a typed corpus.
//!
//! The phantom marker `P` ties every [`EntityRef`] in the data to a
//! specific ontology — `CodegenData<English>` and `CodegenData<UsCode>`
//! are distinct types at compile time. The codegen-side emitter is
//! configured with the marker's fully-qualified path (see
//! `pr4xis::codegen::GenerateConfig::entity_marker_path`).
//!
//! Always available (no feature flag). The `codegen` module that
//! generates this data requires the `codegen` feature, but consuming
//! the output does not.

use crate::EntityRef;

/// Ontology-agnostic codegen output, generic over a phantom marker `P`.
///
/// The codegen emitter validates this data at build time; the runtime
/// `from_codegen` functor materializes a typed corpus from the static.
pub struct CodegenData<P: 'static> {
    pub entity_count: usize,
    pub entity_ids: &'static [&'static str],
    /// What kind each entity is — for `pr4xis_domains::cognitive::linguistics::english::English`
    /// this is the WordNet POS tag (`"n"`, `"v"`, `"a"`, `"r"`); for
    /// statute ontologies it is `"statute_term"`; for USC titles
    /// (forthcoming `UsCode`) it is the USLM element name.
    pub entity_kind: &'static [&'static str],
    pub entity_labels: &'static [&'static str],
    pub entity_defs: &'static [&'static str],
    pub word_index: &'static [(&'static str, &'static [EntityRef<P>])],
    pub taxonomy: &'static [(EntityRef<P>, EntityRef<P>)],
    pub mereology: &'static [(EntityRef<P>, EntityRef<P>)],
    pub opposition: &'static [(EntityRef<P>, EntityRef<P>)],
    pub equivalence: &'static [(EntityRef<P>, EntityRef<P>)],
    pub causation: &'static [(EntityRef<P>, EntityRef<P>)],
    /// Cross-references (`rdfs:seeAlso` / `skos:related`). Populated from
    /// WordNet's `also_synset` / `also_sense` relations on the English
    /// side; statute and USC paths populate it from explicit
    /// cross-reference edges where present.
    ///
    /// Literature: Miles & Bechhofer (2009) "SKOS Simple Knowledge
    /// Organization System Reference", W3C Recommendation §8 (mapping
    /// properties).
    pub references: &'static [(EntityRef<P>, EntityRef<P>)],
}

impl<P: 'static> CodegenData<P> {
    /// Look up concept handles by word text (binary search on sorted
    /// `word_index`).
    pub fn lookup(&self, word: &str) -> &'static [EntityRef<P>] {
        match self.word_index.binary_search_by_key(&word, |(w, _)| w) {
            Ok(idx) => self.word_index[idx].1,
            Err(_) => &[],
        }
    }
}
