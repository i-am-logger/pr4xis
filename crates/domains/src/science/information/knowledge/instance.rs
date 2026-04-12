use super::descriptor::VocabularyDescriptor;
use crate::science::cognition::self_model::AwarenessLevel;

// SelfModelInstance — runtime eigenform of the SelfModel ontology.
//
// This is the bridge between the pure ontology (self_model.rs)
// and the runtime system. Constructing this IS the self-observation
// operator F from von Foerster. The result IS X = F(X).

/// Runtime instance of the self-model — the eigenform.
#[derive(Debug, Clone)]
pub struct SelfModelInstance {
    pub name: &'static str,
    pub version: &'static str,
    pub awareness: AwarenessLevel,
    pub components: Vec<VocabularyDescriptor>,
    pub total_concepts: usize,
    pub total_morphisms: usize,
}

impl SelfModelInstance {
    /// The self-observation operator F. X = F(X).
    pub fn observe(components: Vec<VocabularyDescriptor>) -> Self {
        let total_concepts = components.iter().map(|v| v.concepts).sum();
        let total_morphisms = components.iter().map(|v| v.morphisms).sum();
        Self {
            name: "pr4xis",
            version: env!("CARGO_PKG_VERSION"),
            awareness: AwarenessLevel::MetaSelf,
            components,
            total_concepts,
            total_morphisms,
        }
    }

    /// JSON encoding — transport only, not the self-knowledge.
    pub fn to_json(&self) -> String {
        let onto_json: Vec<String> = self.components.iter().map(|v| {
            format!(
                r#"{{"name":"{}","domain":"{}","being":"{}","source":"{}","concepts":{},"morphisms":{}}}"#,
                v.name, v.domain, v.being.label(), v.source, v.concepts, v.morphisms,
            )
        }).collect();

        format!(
            r#"{{"name":"{}","version":"{}","awareness":"{}","ontology_count":{},"total_concepts":{},"total_morphisms":{},"ontologies":[{}]}}"#,
            self.name,
            self.version,
            self.awareness.label(),
            self.components.len(),
            self.total_concepts,
            self.total_morphisms,
            onto_json.join(","),
        )
    }
}
