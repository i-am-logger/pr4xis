// Vocabulary — runtime instance of KnowledgeConcept::Vocabulary.
//
// The Vocabulary struct lives in pr4xis core (pr4xis::ontology::Vocabulary).
// This module provides KnowledgeBase (the catalog of all Vocabulary instances)
// and the present() transport via Schema Presentation.
//
// Each ontology produces a Vocabulary through define_ontology!'s
// vocabulary() method. The SelfModel eigenform IS the KnowledgeBase
// that catalogs all Vocabulary instances.
//
// Source: W3C VoID (2011); Spivak (2012)

use crate::formal::information::schema::transport::{Presentation, SchemaValue};
use pr4xis::ontology::Vocabulary;

/// Present a Vocabulary as a Schema Presentation for transport.
pub fn present_vocabulary(v: &Vocabulary) -> Presentation {
    let mut p = Presentation::new();
    p.set("module_path", v.module_path.as_str().into());
    p.set("domain", SchemaValue::Text(v.domain()));
    p.set("source", v.source.as_str().into());
    p.set(
        "being",
        v.being.map_or(SchemaValue::Absent, |b| b.label().into()),
    );
    p.set("concept_count", (v.concept_count() as u64).into());
    p.set("morphism_count", (v.morphism_count() as u64).into());
    p
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
            crate::formal::information::knowledge::ontology::KnowledgeBaseCategory,
            crate::formal::information::knowledge::ontology::KnowledgeConcept,
        >(
            "KnowledgeOntology",
            "pr4xis_domains::formal::information::knowledge::ontology",
            "W3C VoID (2011)",
            Some(pr4xis::ontology::upper::being::Being::AbstractObject),
        );
        assert!(v.concept_count() > 0);
        assert!(v.morphism_count() > 0);
        assert!(v.domain().contains("knowledge"));
    }

    #[test]
    fn knowledge_base_presents() {
        let v = Vocabulary::from_ontology::<
            crate::formal::information::knowledge::ontology::KnowledgeBaseCategory,
            crate::formal::information::knowledge::ontology::KnowledgeConcept,
        >(
            "KnowledgeOntology",
            "pr4xis_domains::formal::information::knowledge::ontology",
            "W3C VoID (2011)",
            Some(pr4xis::ontology::upper::being::Being::AbstractObject),
        );
        let kb = KnowledgeBase::catalog(vec![v]);
        let p = kb.present();
        assert_eq!(p.text("name"), Some("pr4xis"));
        assert_eq!(p.unsigned("vocabulary_count"), Some(1));
    }
}
