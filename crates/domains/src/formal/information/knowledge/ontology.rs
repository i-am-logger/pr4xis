#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;
use pr4xis::define_ontology;
use pr4xis::ontology::{Ontology, Quality};

// Knowledge Base ontology — a system's self-description of what it knows.
//
// Grounded in:
// - VoID (Vocabulary of Interlinked Datasets, W3C 2011)
// - DCAT (Data Catalog Vocabulary v3, W3C 2024)
// - SKOS (Simple Knowledge Organization Systems, W3C 2009)
// - Herre & Loebe, "A Meta-ontological Architecture" (FOIS 2005)
//
// Causally connected (Smith 1984): the self-description IS computed
// from the actual loaded state, not from static metadata.

/// Concepts in the KnowledgeBase ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
pub enum KnowledgeConcept {
    /// The system as a whole (dcat:Catalog).
    KnowledgeBase,
    /// A loaded ontology (void:Dataset).
    Vocabulary,
    /// Formal structure of a vocabulary (skos:ConceptScheme).
    Schema,
    /// A single concept within a vocabulary (skos:Concept).
    Entry,
    /// Structural metadata — counts, statistics (void statistics).
    Descriptor,
    /// Origin of the data — paper, spec, file (bridges to Provenance).
    DataSource,
}

define_ontology! {
    pub KnowledgeOntology for KnowledgeBaseCategory {
        concepts: KnowledgeConcept,
        relation: KnowledgeRelation,
        kind: KnowledgeRelationKind,
        kinds: [
            /// KnowledgeBase catalogs Vocabulary (dcat:dataset).
            Catalogs,
            /// Vocabulary conforms to Schema (dct:conformsTo).
            ConformsTo,
            /// Vocabulary contains Entry (void:entity).
            Contains,
            /// Vocabulary described by Descriptor (void statistics).
            DescribedBy,
            /// Vocabulary derived from DataSource (prov:wasDerivedFrom).
            DerivedFrom,
            /// Schema defines Entry (skos:inScheme inverse).
            Defines,
        ],
        edges: [
            (KnowledgeBase, Vocabulary, Catalogs),
            (Vocabulary, Schema, ConformsTo),
            (Vocabulary, Entry, Contains),
            (Vocabulary, Descriptor, DescribedBy),
            (Vocabulary, DataSource, DerivedFrom),
            (Schema, Entry, Defines),
        ],
        composed: [
            (KnowledgeBase, Entry),
            (KnowledgeBase, Schema),
            (KnowledgeBase, Descriptor),
            (KnowledgeBase, DataSource),
        ],
        being: AbstractObject,
        source: "W3C VoID (2011); Herre & Loebe (2005)",
    }
}

/// Whether a knowledge concept is structural (schema-level) or instance-level.
#[derive(Debug, Clone)]
pub struct IsStructural;

impl Quality for IsStructural {
    type Individual = KnowledgeConcept;
    type Value = bool;

    fn get(&self, individual: &KnowledgeConcept) -> Option<bool> {
        match individual {
            KnowledgeConcept::KnowledgeBase => Some(true),
            KnowledgeConcept::Vocabulary => Some(true),
            KnowledgeConcept::Schema => Some(true),
            KnowledgeConcept::Entry => Some(false),
            KnowledgeConcept::Descriptor => Some(false),
            KnowledgeConcept::DataSource => Some(false),
        }
    }
}

impl Ontology for KnowledgeOntology {
    type Cat = KnowledgeBaseCategory;
    type Qual = IsStructural;

    fn structural_axioms() -> Vec<Box<dyn pr4xis::ontology::Axiom>> {
        Self::generated_structural_axioms()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::entity::Concept;

    #[test]
    fn ontology_validates() {
        KnowledgeOntology::validate().unwrap();
    }

    #[test]
    fn category_identity_law() {
        for obj in KnowledgeConcept::variants() {
            let id = KnowledgeBaseCategory::identity(&obj);
            assert_eq!(id.from, obj);
            assert_eq!(id.to, obj);
        }
    }

    #[test]
    fn category_composition_with_identity() {
        for m in &KnowledgeBaseCategory::morphisms() {
            let left = KnowledgeBaseCategory::compose(&KnowledgeBaseCategory::identity(&m.from), m)
                .unwrap();
            assert_eq!(left.from, m.from);
            assert_eq!(left.to, m.to);
        }
    }

    #[test]
    fn has_six_concepts() {
        assert_eq!(KnowledgeConcept::variants().len(), 6);
    }

    #[test]
    fn knowledge_base_catalogs_vocabulary() {
        assert!(
            KnowledgeBaseCategory::morphisms()
                .iter()
                .any(|m| m.from == KnowledgeConcept::KnowledgeBase
                    && m.to == KnowledgeConcept::Vocabulary
                    && m.kind == KnowledgeRelationKind::Catalogs)
        );
    }

    #[test]
    fn vocabulary_derived_from_datasource() {
        assert!(
            KnowledgeBaseCategory::morphisms()
                .iter()
                .any(|m| m.from == KnowledgeConcept::Vocabulary
                    && m.to == KnowledgeConcept::DataSource
                    && m.kind == KnowledgeRelationKind::DerivedFrom)
        );
    }
}
