//! Knowledge-base ontology — a system's self-description of what it
//! knows. Grounded in the W3C metadata-vocabulary stack (VoID, DCAT,
//! SKOS) plus the meta-ontological architecture of Herre & Loebe (2005).
//!
//! Per Smith (1984), the description is *causally connected*: it is
//! computed from the actual loaded state, not from static metadata.
//!
//! # Literature
//!
//! - **W3C VoID (2011)** *Vocabulary of Interlinked Datasets*, W3C
//!   Interest Group Note 03 March 2011 — `void:Dataset`,
//!   `void:entity`, dataset-level statistics.
//! - **W3C DCAT (2024)** *Data Catalog Vocabulary v3*, W3C
//!   Recommendation — `dcat:Catalog`, `dcat:dataset`.
//! - **W3C SKOS (2009)** *Simple Knowledge Organization System*, W3C
//!   Recommendation 18 August 2009 — `skos:Concept`,
//!   `skos:ConceptScheme`, `skos:inScheme`.
//! - **Herre & Loebe (2005)** "A Meta-ontological Architecture for
//!   Foundational Ontologies", *FOIS 2005* — vocabulary-vs-instance
//!   separation; the structural-instance distinction this ontology
//!   formalises as a quality.
//! - **Smith (1984)** "Reflection and Semantics in Lisp", *POPL 1984*
//!   — causal connection between description and described.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Knowledge",
    source: "W3C VoID (2011) Vocabulary of Interlinked Datasets; W3C DCAT (2024) Data Catalog Vocabulary v3; W3C SKOS (2009) Simple Knowledge Organization System; Herre & Loebe (2005) A Meta-ontological Architecture for Foundational Ontologies, FOIS 2005; Smith (1984) Reflection and Semantics in Lisp, POPL 1984",

    concepts: [
        KnowledgeBase,
        Vocabulary,
        Schema,
        Entry,
        Descriptor,
        DataSource,
    ],

    labels: {
        KnowledgeBase: ("en", "Knowledge base",
            "W3C DCAT (2024) dcat:Catalog: the system as a whole - the top-level container of vocabularies."),
        Vocabulary: ("en", "Vocabulary",
            "W3C VoID (2011) void:Dataset: a loaded ontology - a coherent set of terms with shared provenance."),
        Schema: ("en", "Schema",
            "W3C SKOS (2009) skos:ConceptScheme: the formal structure of a vocabulary - taxonomies, relations, and rules."),
        Entry: ("en", "Entry",
            "W3C SKOS (2009) skos:Concept: a single term within a vocabulary - a named concept the knowledge base can reason about."),
        Descriptor: ("en", "Descriptor",
            "W3C VoID (2011) statistics: counts, sizes, distributions - structural metadata about a vocabulary."),
        DataSource: ("en", "Data source",
            "W3C PROV-O prov:Entity (PROV-DM): the origin of the data - a paper, spec, file, or feed - bridging knowledge to provenance."),
    },

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
}

/// Quality: structural (schema-level) vs instance-level. Per Herre &
/// Loebe (2005), the structural / instance separation is the core
/// meta-ontological distinction. KnowledgeBase, Vocabulary, Schema
/// are structural; Entry, Descriptor, DataSource are instances.
#[derive(Debug, Clone)]
pub struct IsStructural;

impl Quality for IsStructural {
    type Individual = KnowledgeConcept;
    type Value = bool;

    fn get(&self, individual: &KnowledgeConcept) -> Option<bool> {
        Some(matches!(
            individual,
            KnowledgeConcept::KnowledgeBase
                | KnowledgeConcept::Vocabulary
                | KnowledgeConcept::Schema
        ))
    }
}

impl Ontology for KnowledgeOntology {
    type Cat = KnowledgeCategory;
    type Qual = IsStructural;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<KnowledgeCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        KnowledgeOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_identity_law() {
        for obj in KnowledgeConcept::variants() {
            let id = KnowledgeCategory::identity(&obj);
            assert_eq!(id.from, obj);
            assert_eq!(id.to, obj);
        }
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_composition_with_identity() {
        for m in &KnowledgeCategory::morphisms() {
            let left =
                KnowledgeCategory::compose(&KnowledgeCategory::identity(&m.from), m).unwrap();
            assert_eq!(left.from, m.from);
            assert_eq!(left.to, m.to);
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn has_six_concepts() {
        assert_eq!(KnowledgeConcept::variants().len(), 6);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn knowledge_base_catalogs_vocabulary() {
        assert!(
            KnowledgeCategory::morphisms()
                .iter()
                .any(|m| m.from == KnowledgeConcept::KnowledgeBase
                    && m.to == KnowledgeConcept::Vocabulary
                    && m.kind == KnowledgeRelationKind::Catalogs)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn vocabulary_derived_from_datasource() {
        assert!(
            KnowledgeCategory::morphisms()
                .iter()
                .any(|m| m.from == KnowledgeConcept::Vocabulary
                    && m.to == KnowledgeConcept::DataSource
                    && m.kind == KnowledgeRelationKind::DerivedFrom)
        );
    }

    fn arb_concept() -> impl Strategy<Value = KnowledgeConcept> {
        proptest::sample::select(KnowledgeConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_is_structural_total(c in arb_concept()) {
            prop_assert!(IsStructural.get(&c).is_some());
        }

        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            use pr4xis::category::Arrow;
            for m in KnowledgeCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in KnowledgeOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(
                        false,
                        "axiom failed: {}",
                        c.meta().name.as_str()
                    );
                }
            }
        }
    }

    pr4xis::register_praxis_value!(prop_is_structural_total, Verifiable);
    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
}
