//! The fully-faithful functor `OntologyArchiveStorage → PraxisKnowledgeGraph`.
//!
//! The content-addressed archive store
//! ([`OntologyArchiveStorage`](crate::formal::meta::ontology_archive)) is a
//! *full subcategory* of the whole knowledge graph: its twelve concepts are
//! exactly the storage/wire-envelope nodes of the graph, embedded by name.
//! This functor is the formal statement "the archive IS the graph's storage
//! substratum" — created *alongside*, never a rewrite of the landed
//! ontology (`feedback_evolution_via_functor`).
//!
//! It is declared [`FullyFaithful`](pr4xis::category::kinds::FunctorKind)
//! and *machine-proven* so by [`super::axioms::FunctorLawPreservation`] via
//! [`pr4xis::category::laws::fully_faithful_law_axioms`] — identity +
//! composition + faithful + full-onto-image. Per Mac Lane (1971) CWM Ch. I
//! §4, a full-and-faithful functor is an isomorphism onto its image (a full
//! subcategory inclusion); nothing of the archive is dropped, only the
//! ambient vocabulary grows.

use pr4xis::category::Functor;
use pr4xis::category::kinds::FunctorKind;

use super::ontology::{
    PraxisKnowledgeGraphCategory, PraxisKnowledgeGraphConcept, PraxisKnowledgeGraphRelation,
    PraxisKnowledgeGraphRelationKind,
};
use crate::formal::meta::ontology_archive::ontology::{
    OntologyArchiveStorageCategory, OntologyArchiveStorageConcept, OntologyArchiveStorageRelation,
    OntologyArchiveStorageRelationKind,
};

/// The full-and-faithful embedding of the content-addressed archive store
/// into the whole knowledge graph (Mac Lane 1971 CWM Ch. I §4).
pub struct ArchiveIntoGraph;

impl Functor for ArchiveIntoGraph {
    type Source = OntologyArchiveStorageCategory;
    type Target = PraxisKnowledgeGraphCategory;

    const KIND: FunctorKind = FunctorKind::FullyFaithful;

    fn map_object(obj: &OntologyArchiveStorageConcept) -> PraxisKnowledgeGraphConcept {
        use OntologyArchiveStorageConcept as A;
        use PraxisKnowledgeGraphConcept as G;
        // By-name inclusion — each archive concept is the same-named storage
        // node of the graph (injective on objects: 12 distinct → 12 distinct).
        match obj {
            A::ContentAddressableNode => G::ContentAddressableNode,
            A::MerkleEdge => G::MerkleEdge,
            A::MerkleDag => G::MerkleDag,
            A::MerkleRoot => G::MerkleRoot,
            A::BinaryEnvelope => G::BinaryEnvelope,
            A::CompressedForm => G::CompressedForm,
            A::SourcePin => G::SourcePin,
            A::LoadGate => G::LoadGate,
            A::Attestation => G::Attestation,
            A::IntegrityClaim => G::IntegrityClaim,
            A::AttestationChain => G::AttestationChain,
            A::SupplyChainStep => G::SupplyChainStep,
        }
    }

    fn map_morphism(m: &OntologyArchiveStorageRelation) -> PraxisKnowledgeGraphRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        // Explicit, by-name kind translation — never a defaulting arm (a
        // wrong-kind image would make the graph's identity-aware compose
        // break FunctorCompositionLaw). The archive declares only is_a
        // (Subsumption) and has_a (Parthood); identities map to identities.
        let kind = match m.kind {
            OntologyArchiveStorageRelationKind::Identity => {
                PraxisKnowledgeGraphRelationKind::Identity
            }
            OntologyArchiveStorageRelationKind::Subsumption => {
                PraxisKnowledgeGraphRelationKind::Subsumption
            }
            OntologyArchiveStorageRelationKind::Parthood => {
                PraxisKnowledgeGraphRelationKind::Parthood
            }
            // The macro emits a fixed relation-kind enum; the archive never
            // declares Causation / Opposition edges, but the match stays
            // exhaustive and by-name (each maps to its same-named graph kind,
            // never a defaulting arm).
            OntologyArchiveStorageRelationKind::Causation => {
                PraxisKnowledgeGraphRelationKind::Causation
            }
            OntologyArchiveStorageRelationKind::Opposition => {
                PraxisKnowledgeGraphRelationKind::Opposition
            }
        };
        PraxisKnowledgeGraphRelation { from, to, kind }
    }
}

pr4xis::register_functor!(
    ArchiveIntoGraph,
    "Mac Lane (1971) Categories for the Working Mathematician Ch. I §4"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::{assert_functor_laws, fully_faithful_law_axioms};

    #[test]
    fn archive_into_graph_is_a_functor() {
        // Identity + composition preservation (Mac Lane CWM Ch. II §1).
        assert_functor_laws::<ArchiveIntoGraph>();
    }

    #[test]
    fn archive_into_graph_is_fully_faithful() {
        // Beyond functoriality: faithful (injective on each hom-set) + full
        // onto image — the machine proof that KIND = FullyFaithful is
        // earned, not merely tagged (Mac Lane CWM Ch. I §4).
        for law in fully_faithful_law_axioms::<ArchiveIntoGraph>() {
            law.verify()
                .unwrap_or_else(|c| panic!("{}", c.meta().name.as_str()));
        }
    }
}
