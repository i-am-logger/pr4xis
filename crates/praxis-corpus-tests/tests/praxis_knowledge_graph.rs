//! Whole-graph `PraxisKnowledgeGraph` (#272) validation over the real
//! registries — lifted out of the `pr4xis-domains` `#[cfg(test)]` modules.
//!
//! `PraxisKnowledgeGraphOntology::validate()` runs the ontology's full domain
//! axiom set, which includes `LensLawPreservation` — the round-trip lens harness
//! over EVERY registered lens, parsing each on-disk source. Under nextest that
//! parse is paid per process-isolated test; here it runs once in one process.
//! That is why this validation lives in the heavy-corpus lane rather than
//! in-crate under the strict fast-lane per-test cap.
//!
//! The cheap, ontology-only legs stay in the fast lane: the `ArchiveIntoGraph`
//! functor/fully-faithful laws are owned by
//! `praxis_knowledge_graph::functor::tests` (a fast in-crate test), and the
//! per-axiom non-lens legs by
//! `praxis_knowledge_graph::axioms::tests::all_domain_axioms_hold`.

use pr4xis::ontology::Ontology;
use pr4xis_domains::formal::meta::praxis_knowledge_graph::ontology::PraxisKnowledgeGraphOntology;

/// Through the fully-faithful `ArchiveIntoGraph` functor, the OWL/USC/WordNet
/// `.prx` realisation is the storage substratum of the whole-graph
/// `PraxisKnowledgeGraph` (#272). Binding the realisation to the graph spec: the
/// graph ontology validates — its storage-subset axioms run against THIS code,
/// including the heavy `LensLawPreservation` leg that drives the round-trip lens
/// harness over every registered lens.
#[test]
fn realisation_witnesses_the_graph_storage_substratum() {
    PraxisKnowledgeGraphOntology::validate()
        .unwrap_or_else(|c| panic!("graph validation failed: {}", c.meta().name.as_str()));
}
