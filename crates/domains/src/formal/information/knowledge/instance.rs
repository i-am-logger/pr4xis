use crate::cognitive::cognition::self_model::AwarenessLevel;
use crate::formal::information::schema::transport::{Present, Presentation, SchemaValue};
use pr4xis::ontology::{Axiom, Vocabulary, describe_knowledge_base};

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
    pub components: Vec<Vocabulary>,
    pub total_concepts: usize,
    pub total_morphisms: usize,
}

impl SelfModelInstance {
    /// The self-observation operator F. X = F(X).
    pub fn observe(components: Vec<Vocabulary>) -> Self {
        let total_concepts = components.iter().map(|v| v.concept_count()).sum();
        let total_morphisms = components.iter().map(|v| v.morphism_count()).sum();
        Self {
            name: "pr4xis",
            version: env!("CARGO_PKG_VERSION"),
            awareness: AwarenessLevel::MetaSelf,
            components,
            total_concepts,
            total_morphisms,
        }
    }

    /// Transport via Schema Presentation → JSON surface.
    pub fn to_json(&self) -> String {
        self.present().to_json()
    }
}

/// Presents morphism: Algebra → Presentation (Spivak).
/// The SelfModelInstance IS the Algebra (live runtime form).
/// present() produces the Presentation (transport form).
impl Present for SelfModelInstance {
    fn present(&self) -> Presentation {
        let mut p = Presentation::new();
        p.set("name", SchemaValue::Text(self.name.into()));
        p.set("version", SchemaValue::Text(self.version.into()));
        p.set(
            "awareness",
            SchemaValue::Text(self.awareness.label().into()),
        );
        p.set(
            "ontology_count",
            SchemaValue::Unsigned(self.components.len() as u64),
        );
        p.set(
            "total_concepts",
            SchemaValue::Unsigned(self.total_concepts as u64),
        );
        p.set(
            "total_morphisms",
            SchemaValue::Unsigned(self.total_morphisms as u64),
        );

        let ontologies: Vec<SchemaValue> = self
            .components
            .iter()
            .map(|v| {
                let mut ont = Presentation::new();
                ont.set("name", SchemaValue::Text(v.name().into()));
                ont.set("domain", SchemaValue::Text(v.domain()));
                ont.set(
                    "being",
                    SchemaValue::Text(v.being.map_or("Unknown", |b| b.label()).into()),
                );
                ont.set("source", SchemaValue::Text(v.source.as_str().to_string()));
                ont.set("concepts", SchemaValue::Unsigned(v.concept_count() as u64));
                ont.set(
                    "morphisms",
                    SchemaValue::Unsigned(v.morphism_count() as u64),
                );
                SchemaValue::Record(ont)
            })
            .collect();

        p.set("ontologies", SchemaValue::List(ontologies));
        p
    }
}

// ---------------------------------------------------------------------------
// Axioms about the registered knowledge base.
//
// These are first-class claims about what `describe_knowledge_base()` must
// return — discoverable, citable, and reusable across tests / runtime
// health checks. Per memory `feedback_ontological_assertions.md`.
// ---------------------------------------------------------------------------

/// Axiom: at least one ontology is registered in the knowledge base.
/// Catches misconfiguration where linkme registration is missing.
pub struct KnowledgeBaseIsNonEmpty;

impl Axiom for KnowledgeBaseIsNonEmpty {
    fn description(&self) -> &str {
        "describe_knowledge_base() returns at least one registered Vocabulary"
    }
    fn holds(&self) -> bool {
        !describe_knowledge_base().is_empty()
    }
}
pr4xis::register_axiom!(KnowledgeBaseIsNonEmpty);

/// Axiom: SelfModelOntology is registered in the knowledge base.
/// The system can describe itself iff its own SelfModel ontology is
/// reachable through the auto-registration mechanism (linkme).
pub struct SelfModelIsRegistered;

impl Axiom for SelfModelIsRegistered {
    fn description(&self) -> &str {
        "SelfModelOntology is registered in the knowledge base"
    }
    fn holds(&self) -> bool {
        describe_knowledge_base()
            .iter()
            .any(|v| v.name() == "SelfModelOntology")
    }
}
pr4xis::register_axiom!(SelfModelIsRegistered);

/// Axiom: KnowledgeOntology is registered in the knowledge base.
/// The Knowledge ontology — root of the registry — must register itself.
pub struct KnowledgeIsRegistered;

impl Axiom for KnowledgeIsRegistered {
    fn description(&self) -> &str {
        "KnowledgeOntology is registered in the knowledge base"
    }
    fn holds(&self) -> bool {
        describe_knowledge_base()
            .iter()
            .any(|v| v.name() == "KnowledgeOntology")
    }
}
pr4xis::register_axiom!(KnowledgeIsRegistered);
