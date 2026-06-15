//! Vocabulary — runtime instance of `KnowledgeConcept::Vocabulary`.
//!
//! The `Vocabulary` struct lives in pr4xis core
//! (`pr4xis::ontology::Vocabulary`). This module provides the
//! `KnowledgeBase` aggregate (the catalog of every loaded `Vocabulary`)
//! and the `present()` transport via Schema Presentation.
//!
//! Each ontology produces a Vocabulary through the proc macro
//! `pr4xis::ontology!`'s `vocabulary()` method. The SelfModel
//! eigenform (von Foerster 1981) is the `KnowledgeBase` that
//! catalogues all `Vocabulary` instances.
//!
//! # Literature
//!
//! - **W3C VoID (2011)** *Vocabulary of Interlinked Datasets*
//! - **Spivak (2012)** *Functorial Data Migration*

use alloc::collections::BTreeSet;
use alloc::string::ToString;
use alloc::vec::Vec;

use pr4xis::ontology::Vocabulary;
use pr4xis::ontology::meta::{ConceptName, Morphism, MorphismKind};
use pr4xis_runtime::ontology::RuntimeOntology;

use crate::cognitive::linguistics::english::bridge::FORM_KIND;
use crate::formal::information::schema::transport::{Presentation, SchemaValue};

/// Present a Vocabulary as a Schema Presentation for transport.
pub fn present_vocabulary(v: &Vocabulary) -> Presentation {
    let mut p = Presentation::new();
    p.set("module_path", v.module_path.as_str().into());
    p.set("domain", SchemaValue::Text(v.domain()));
    p.set("source", v.source.as_str().into());
    p.set("concept_count", (v.concept_count() as u64).into());
    p.set("morphism_count", (v.morphism_count() as u64).into());
    p
}

/// Project a loaded [`RuntimeOntology`] into a [`Vocabulary`] — the eigenform
/// adapter that lets the SelfModel OBSERVE the live loaded set, not just the
/// compiled substrate (doc §2: `F` applied to the live object level, never a
/// constant — so loading Title 15 MOVES `total_concepts`).
///
/// Its concepts are the CONCEPT nodes (the §9 `ontolex:Form` surface atoms are
/// queryable surfaces, not concepts — excluded, exactly as the catalog's
/// `loaded_refs` excludes them); its morphisms are the generating typed edges
/// between concepts (a lexicalization edge pointing AT a `Form` is a surface, not
/// taxonomy — excluded). The counts are read off the loaded archive
/// (`archive().nodes`) as data, never hardcoded.
pub fn runtime_ontology_vocabulary(onto: &RuntimeOntology) -> Vocabulary {
    let archive = onto.archive();
    let form_names: BTreeSet<&str> = archive
        .nodes
        .iter()
        .filter(|n| n.kind == FORM_KIND)
        .map(|n| n.name.as_str())
        .collect();

    let mut concepts: Vec<ConceptName> = Vec::new();
    let mut morphisms: Vec<Morphism> = Vec::new();
    for node in archive.nodes.iter().filter(|n| n.kind != FORM_KIND) {
        concepts.push(ConceptName::new(node.name.to_string()));
        for (kind, target) in &node.edges {
            // A cross-ontology grounded edge (no local target) is a denotation link
            // to a foreign atom, not an intra-ontology morphism between concepts —
            // skip it (else it would mint a morphism with an empty target name and
            // inflate the count).
            let Some(target_name) = target.local_name() else {
                continue;
            };
            // A lexicalization edge into a Form surface is not a taxonomy morphism.
            if form_names.contains(target_name) {
                continue;
            }
            morphisms.push(Morphism::new(
                ConceptName::new(node.name.to_string()),
                ConceptName::new(target_name.to_string()),
                MorphismKind::from_name(kind),
            ));
        }
    }

    Vocabulary::from_captured(
        onto.id().as_str().to_string(),
        "pr4xis_runtime::ontology (loaded .prx)",
        "Loaded at runtime — content-addressed .prx",
        concepts,
        morphisms,
    )
}

/// What a loaded ontology can actually ANSWER — its capabilities (doc §4.7).
/// "Loaded" alone lies: a Parthood-only USC card goes green while its taxonomy
/// queries are dark. This reports, DATA-DRIVEN, which reachability queries the
/// materialized ontology really supports over its CLOSED (transitive) relation
/// kinds — so the self-model is honest about capability, not just size. (Purely
/// non-transitive edge kinds carry no reachability closure, so they are not among
/// the reported `relation_kinds`.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OntologyCapability {
    /// The loaded ontology, by name.
    pub ontology: alloc::string::String,
    /// Can a concept's gloss be read back? (At least one concept carries lexical.)
    pub gloss: bool,
    /// The relation kinds whose closure is POPULATED — `Subsumption`, `Parthood`,
    /// … — read off the materialized closure, never a hardcoded `{subsumption,
    /// parthood}` set. Sorted for a stable surface.
    pub relation_kinds: alloc::vec::Vec<alloc::string::String>,
}

/// Derive a loaded ontology's [`OntologyCapability`] from its MATERIALIZED form —
/// `gloss` from whether any CONCEPT carries lexical (Form surface atoms are not
/// concepts), `relation_kinds` from
/// [`populated_kinds`](pr4xis_runtime::ontology::MaterializedClosure::populated_kinds)
/// (the kinds whose closure actually has reachability). So a USC reports
/// `Parthood`, an OWL `Subsumption` — emergent from the loaded data.
pub fn ontology_capabilities(onto: &RuntimeOntology) -> OntologyCapability {
    let gloss = onto
        .archive()
        .nodes
        .iter()
        .filter(|n| n.kind != FORM_KIND)
        .any(|n| n.lexical.is_some());
    let mut relation_kinds: Vec<alloc::string::String> = onto
        .closure()
        .populated_kinds()
        .into_iter()
        .map(|k| k.name)
        .collect();
    relation_kinds.sort();
    OntologyCapability {
        ontology: onto.id().as_str().to_string(),
        gloss,
        relation_kinds,
    }
}

/// The KnowledgeBase — catalogs all Vocabulary instances.
/// This IS the self-model eigenform: X = F(X).
#[derive(Debug, Clone)]
pub struct KnowledgeBase {
    pub vocabularies: Vec<Vocabulary>,
}

impl KnowledgeBase {
    /// The eigenform operator. Catalogs all vocabularies.
    pub fn catalog(vocabularies: Vec<Vocabulary>) -> Self {
        Self { vocabularies }
    }

    pub fn vocabulary_count(&self) -> usize {
        self.vocabularies.len()
    }

    pub fn total_concepts(&self) -> usize {
        self.vocabularies.iter().map(|v| v.concept_count()).sum()
    }

    pub fn total_morphisms(&self) -> usize {
        self.vocabularies.iter().map(|v| v.morphism_count()).sum()
    }

    /// Present the entire knowledge base as a Presentation.
    pub fn present(&self) -> Presentation {
        let mut p = Presentation::new();
        p.set("name", "pr4xis".into());
        p.set("version", env!("CARGO_PKG_VERSION").into());
        p.set("vocabulary_count", (self.vocabularies.len() as u64).into());
        p.set("total_concepts", (self.total_concepts() as u64).into());
        p.set("total_morphisms", (self.total_morphisms() as u64).into());
        p.set(
            "vocabularies",
            SchemaValue::List(
                self.vocabularies
                    .iter()
                    .map(|v| SchemaValue::Record(present_vocabulary(v)))
                    .collect(),
            ),
        );
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vocabulary_from_ontology() {
        let v = Vocabulary::from_ontology::<
            crate::formal::information::knowledge::ontology::KnowledgeCategory,
            crate::formal::information::knowledge::ontology::KnowledgeConcept,
        >(
            "KnowledgeOntology",
            "pr4xis_domains::formal::information::knowledge::ontology",
            "W3C VoID (2011)",
        );
        assert!(v.concept_count() > 0);
        assert!(v.morphism_count() > 0);
        assert!(v.domain().contains("knowledge"));
    }

    #[test]
    fn runtime_ontology_vocabulary_counts_concepts_not_form_surfaces() {
        use pr4xis::ontology::meta::OntologyName;
        use pr4xis_runtime::archive::Archive;
        use pr4xis_runtime::definition::{Definition, EdgeTarget};
        use pr4xis_runtime::ontology::materialize;

        // Two concepts (a Subsumption edge between them) + one Form surface the
        // first concept denotes via a lexicalization edge.
        let archive = Archive {
            nodes: alloc::vec![
                Definition {
                    kind: "Concept".to_string(),
                    name: "dog".to_string(),
                    edges: alloc::vec![
                        (
                            "Subsumption".to_string(),
                            EdgeTarget::Local("animal".to_string())
                        ),
                        (
                            "canonicalForm".to_string(),
                            EdgeTarget::Local("doggo".to_string())
                        ),
                    ],
                    axioms: alloc::vec![],
                    lexical: Some("a domesticated canine".to_string()),
                },
                Definition {
                    kind: "Concept".to_string(),
                    name: "animal".to_string(),
                    edges: alloc::vec![],
                    axioms: alloc::vec![],
                    lexical: None,
                },
                Definition {
                    kind: FORM_KIND.to_string(),
                    name: "doggo".to_string(),
                    edges: alloc::vec![],
                    axioms: alloc::vec![],
                    lexical: Some("doggo".to_string()),
                },
            ],
            connections: alloc::vec![],
        };
        let onto = materialize(archive, OntologyName::new_static("zoo"))
            .expect("the archive materializes");

        let vocab = runtime_ontology_vocabulary(&onto);
        assert_eq!(vocab.name(), "zoo");
        assert_eq!(
            vocab.concept_count(),
            2,
            "two concepts (dog, animal); the Form surface 'doggo' is NOT a concept"
        );
        assert_eq!(
            vocab.morphism_count(),
            1,
            "one taxonomy morphism (dog→animal); the canonicalForm edge into the Form is excluded"
        );
        assert_eq!(
            vocab.morphisms()[0].kind,
            MorphismKind::Subsumption,
            "the edge kind is recovered as the typed Subsumption morphism"
        );
    }

    #[test]
    fn ontology_capabilities_are_the_data_driven_populated_kinds() {
        use pr4xis::ontology::meta::OntologyName;
        use pr4xis_runtime::archive::Archive;
        use pr4xis_runtime::definition::{Definition, EdgeTarget};
        use pr4xis_runtime::ontology::materialize;

        // A Parthood mereology (subsection part-of section) with a gloss — but NO
        // is-a edge. The capability must report Parthood (populated) and NOT
        // Subsumption (the doc §4.7 point: capability ≠ vocabulary, emergent from
        // the materialized data — "loaded" with only half its closure populated).
        let archive = Archive {
            nodes: alloc::vec![
                Definition {
                    kind: "Concept".to_string(),
                    name: "subsection".to_string(),
                    edges: alloc::vec![(
                        "Parthood".to_string(),
                        EdgeTarget::Local("section".to_string())
                    )],
                    axioms: alloc::vec![],
                    lexical: Some("a subdivision of a section".to_string()),
                },
                Definition {
                    kind: "Concept".to_string(),
                    name: "section".to_string(),
                    edges: alloc::vec![],
                    axioms: alloc::vec![],
                    lexical: None,
                },
            ],
            connections: alloc::vec![],
        };
        let onto = materialize(archive, OntologyName::new_static("usc")).expect("materializes");

        let cap = ontology_capabilities(&onto);
        assert_eq!(cap.ontology, "usc");
        assert!(cap.gloss, "a concept carries a gloss");
        assert!(
            cap.relation_kinds.contains(&"Parthood".to_string()),
            "the Parthood closure is populated; got {:?}",
            cap.relation_kinds
        );
        assert!(
            !cap.relation_kinds.contains(&"Subsumption".to_string()),
            "no is-a edge → Subsumption is NOT a reported capability; got {:?}",
            cap.relation_kinds
        );
    }

    #[test]
    fn knowledge_base_presents() {
        let v = Vocabulary::from_ontology::<
            crate::formal::information::knowledge::ontology::KnowledgeCategory,
            crate::formal::information::knowledge::ontology::KnowledgeConcept,
        >(
            "KnowledgeOntology",
            "pr4xis_domains::formal::information::knowledge::ontology",
            "W3C VoID (2011)",
        );
        let kb = KnowledgeBase::catalog(vec![v]);
        let p = kb.present();
        assert_eq!(p.text("name"), Some("pr4xis"));
        assert_eq!(p.unsigned("vocabulary_count"), Some(1));
    }
}
