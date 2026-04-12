use pr4xis::category::Category;
use pr4xis::category::entity::Entity;
use pr4xis::category::relationship::Relationship;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl Entity for KnowledgeConcept {
    fn variants() -> Vec<Self> {
        vec![
            Self::KnowledgeBase,
            Self::Vocabulary,
            Self::Schema,
            Self::Entry,
            Self::Descriptor,
            Self::DataSource,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KnowledgeRelation {
    pub from: KnowledgeConcept,
    pub to: KnowledgeConcept,
    pub kind: KnowledgeRelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KnowledgeRelationKind {
    Identity,
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
    Composed,
}

impl Relationship for KnowledgeRelation {
    type Object = KnowledgeConcept;
    fn source(&self) -> KnowledgeConcept {
        self.from
    }
    fn target(&self) -> KnowledgeConcept {
        self.to
    }
}

pub struct KnowledgeBaseCategory;

impl Category for KnowledgeBaseCategory {
    type Object = KnowledgeConcept;
    type Morphism = KnowledgeRelation;

    fn identity(obj: &KnowledgeConcept) -> KnowledgeRelation {
        KnowledgeRelation {
            from: *obj,
            to: *obj,
            kind: KnowledgeRelationKind::Identity,
        }
    }

    fn compose(f: &KnowledgeRelation, g: &KnowledgeRelation) -> Option<KnowledgeRelation> {
        if f.to != g.from {
            return None;
        }
        if f.kind == KnowledgeRelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == KnowledgeRelationKind::Identity {
            return Some(f.clone());
        }
        Some(KnowledgeRelation {
            from: f.from,
            to: g.to,
            kind: KnowledgeRelationKind::Composed,
        })
    }

    fn morphisms() -> Vec<KnowledgeRelation> {
        use KnowledgeConcept::*;
        use KnowledgeRelationKind::*;
        let mut m = Vec::new();

        for c in KnowledgeConcept::variants() {
            m.push(KnowledgeRelation {
                from: c,
                to: c,
                kind: Identity,
            });
        }

        m.push(KnowledgeRelation {
            from: KnowledgeBase,
            to: Vocabulary,
            kind: Catalogs,
        });
        m.push(KnowledgeRelation {
            from: Vocabulary,
            to: Schema,
            kind: ConformsTo,
        });
        m.push(KnowledgeRelation {
            from: Vocabulary,
            to: Entry,
            kind: Contains,
        });
        m.push(KnowledgeRelation {
            from: Vocabulary,
            to: Descriptor,
            kind: DescribedBy,
        });
        m.push(KnowledgeRelation {
            from: Vocabulary,
            to: DataSource,
            kind: DerivedFrom,
        });
        m.push(KnowledgeRelation {
            from: Schema,
            to: Entry,
            kind: Defines,
        });

        // Transitive
        m.push(KnowledgeRelation {
            from: KnowledgeBase,
            to: Entry,
            kind: Composed,
        });
        m.push(KnowledgeRelation {
            from: KnowledgeBase,
            to: Schema,
            kind: Composed,
        });
        m.push(KnowledgeRelation {
            from: KnowledgeBase,
            to: Descriptor,
            kind: Composed,
        });
        m.push(KnowledgeRelation {
            from: KnowledgeBase,
            to: DataSource,
            kind: Composed,
        });

        for c in KnowledgeConcept::variants() {
            m.push(KnowledgeRelation {
                from: c,
                to: c,
                kind: Composed,
            });
        }

        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::entity::Entity;

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
