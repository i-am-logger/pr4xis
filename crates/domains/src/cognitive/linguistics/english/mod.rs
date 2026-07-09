pub mod concept_store;
pub mod function_word_store;
pub mod morphology_store;
pub mod ontology;
pub mod relation_store;
pub mod synset_index;
pub mod taxonomy_store;
pub mod verb_transitivity_index;
pub mod word_index;
pub mod writing_system_store;

/// The WordNet → `.prx` runtime bridge — projects the loaded [`English`] struct
/// into a content-addressed [`Archive`](pr4xis_runtime::archive::Archive) and
/// relabels it into the praxis schema with a projection carried as data. Needs
/// `std` (it drives the `pr4xis-runtime` `apply` / `materialize` engine, as
/// [`composed`](super::composed) does).
#[cfg(feature = "std")]
pub mod bridge;

pub use concept_store::{ConceptStore, ConceptStrs, ConceptView};
pub use ontology::{
    Concept, ConceptId, English, LexicalReasoner, SenseId, WordnetRelations, english_load_owned,
    english_loaded,
};
pub use relation_store::{RelationKind, RelationStore};
pub use synset_index::SynsetIndex;
pub use taxonomy_store::TaxonomyStore;

#[cfg(test)]
mod tests;
