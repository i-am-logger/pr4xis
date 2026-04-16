// Vocabulary — runtime instance of KnowledgeConcept::Vocabulary.
//
// An ontology's self-description as an instance of the Knowledge
// category. Not a custom struct — an instance of Vocabulary with
// morphisms to DataSource, Entry, Descriptor, Schema.
//
// Each ontology produces a Vocabulary through define_ontology!'s
// descriptor() method. The SelfModel eigenform IS the KnowledgeBase
// that catalogs all Vocabulary instances.
//
// Source: W3C VoID (2011); Spivak (2012)

use crate::formal::information::schema::transport::{Presentation, SchemaValue};

/// A Vocabulary instance — an ontology describing itself.
///
/// Instance of KnowledgeConcept::Vocabulary. The fields correspond
/// to morphisms in the Knowledge category:
///   Vocabulary →(DerivedFrom)→ DataSource = source
///   Vocabulary →(Contains)→ Entry = concepts (counted)
///   Vocabulary →(DescribedBy)→ Descriptor = statistics
///   Vocabulary →(ConformsTo)→ Schema = structure
///
/// Every field is populated from the ontology's own traits:
///   Entity::variants().len() = concept count
///   Category::morphisms().len() = morphism count
///   Classified::being() = DOLCE classification
///   define_ontology! source: = primary citation
#[derive(Debug, Clone)]
pub struct Vocabulary {
    /// The ontology's module path — its identity.
    pub module_path: &'static str,
    /// DerivedFrom → DataSource: primary citation.
    pub source: &'static str,
    /// DOLCE Being classification.
    pub being: Option<pr4xis::ontology::upper::being::Being>,
    /// Contains → Entry count (void:classes).
    pub concept_count: usize,
    /// DescribedBy → Descriptor morphism count (void:properties).
    pub morphism_count: usize,
}

impl Vocabulary {
    /// Domain path derived from module_path.
    pub fn domain(&self) -> String {
        let s = self.module_path;
        let s = s.strip_prefix("pr4xis_domains::").unwrap_or(s);
        let s = s.strip_suffix("::ontology").unwrap_or(s);
        s.replace("::", ".")
    }

    /// The ontology struct name (from module path).
    pub fn name(&self) -> &str {
        self.module_path
            .rsplit("::")
            .nth(1)
            .unwrap_or(self.module_path)
    }

    /// Present as a Schema Presentation for transport.
    pub fn present(&self) -> Presentation {
        let mut p = Presentation::new();
        p.set("module_path", self.module_path.into());
        p.set("domain", SchemaValue::Text(self.domain()));
        p.set("source", self.source.into());
        p.set(
            "being",
            self.being.map_or(SchemaValue::Absent, |b| b.label().into()),
        );
        p.set("concept_count", (self.concept_count as u64).into());
        p.set("morphism_count", (self.morphism_count as u64).into());
        p
    }

    /// Construct from any Category + Entity + Classified ontology.
    pub fn from_ontology<C, E>(
        module_path: &'static str,
        source: &'static str,
        being: Option<pr4xis::ontology::upper::being::Being>,
    ) -> Self
    where
        C: pr4xis::category::Category,
        E: pr4xis::category::entity::Entity,
    {
        Self {
            module_path,
            source,
            being,
            concept_count: E::variants().len(),
            morphism_count: C::morphisms().len(),
        }
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
        self.vocabularies.iter().map(|v| v.concept_count).sum()
    }

    pub fn total_morphisms(&self) -> usize {
        self.vocabularies.iter().map(|v| v.morphism_count).sum()
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
                    .map(|v| SchemaValue::Record(v.present()))
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
            "pr4xis_domains::formal::information::knowledge::ontology",
            "W3C VoID (2011)",
            Some(pr4xis::ontology::upper::being::Being::AbstractObject),
        );
        assert!(v.concept_count > 0);
        assert!(v.morphism_count > 0);
        assert!(v.domain().contains("knowledge"));
    }

    #[test]
    fn knowledge_base_presents() {
        let v = Vocabulary::from_ontology::<
            crate::formal::information::knowledge::ontology::KnowledgeBaseCategory,
            crate::formal::information::knowledge::ontology::KnowledgeConcept,
        >(
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
