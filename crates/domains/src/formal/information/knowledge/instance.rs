#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::cognitive::cognition::self_model::AwarenessLevel;
use crate::formal::information::knowledge::catalog::{SourceStatus, staging_label};
use crate::formal::information::schema::transport::{Present, Presentation, SchemaValue};
use pr4xis::ontology::{Axiom, Vocabulary, describe_knowledge_base};

// SelfModelInstance — runtime eigenform of the SelfModel ontology.
//
// This is the bridge between the pure ontology (self_model.rs)
// and the runtime system. Constructing this IS the self-observation
// operator F from von Foerster. The result IS X = F(X).

/// Runtime instance of the self-model — the eigenform.
///
/// `components` is the object-level the meta-level *monitors* (the loaded
/// ontologies). `catalog` is the meta-level's model of the **knowledge
/// boundary**: every registered source tagged Loaded / Available
/// (Nelson & Narens 1990; see
/// [`crate::formal::information::knowledge::catalog`]). An empty `catalog`
/// means the boundary was not supplied (e.g. a caller that only reports
/// the loaded components).
#[derive(Debug, Clone)]
pub struct SelfModelInstance {
    pub name: &'static str,
    pub version: &'static str,
    pub awareness: AwarenessLevel,
    pub components: Vec<Vocabulary>,
    pub catalog: Vec<SourceStatus>,
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
            catalog: Vec::new(),
            total_concepts,
            total_morphisms,
        }
    }

    /// Attach the source catalog — the meta-level's model of the
    /// knowledge boundary (loaded vs available registered sources).
    pub fn with_catalog(mut self, catalog: Vec<SourceStatus>) -> Self {
        self.catalog = catalog;
        self
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
                // DOLCE Being was removed from Vocabulary per #165.
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

        // The knowledge boundary — every registered source tagged
        // Loaded / Available (Nelson & Narens 1990 meta-level model).
        // Source-agnostic: each entry is rendered from registry data, so
        // a UI can list the full catalog and offer to load what isn't yet
        // materialized without knowing what any source *is*.
        let sources: Vec<SchemaValue> = self
            .catalog
            .iter()
            .map(|s| {
                let mut src = Presentation::new();
                src.set("name", SchemaValue::Text(s.name.clone()));
                src.set("version", SchemaValue::Text(s.version.clone()));
                src.set("kind", SchemaValue::Text(s.kind.clone()));
                src.set("source", SchemaValue::Text(s.citation.clone()));
                // Wire field name mirrors the Rust ontology
                // (`SourceAvailability` per
                // `crates/domains/src/formal/information/knowledge/catalog.rs`).
                // Keeping the on-the-wire identifier identical to the ontology
                // concept prevents a UI ↔ wire drift like the chat ontologies
                // renderer reading `s.availability` while the wire was emitting
                // `s.status` (the bug that silently made `.prx.gz` loads not
                // reach `.source-card.loaded` in the catalog).
                src.set(
                    "availability",
                    SchemaValue::Text(s.availability.label().into()),
                );
                src.set(
                    "staging",
                    match s.staging {
                        Some(st) => SchemaValue::Text(staging_label(st).into()),
                        None => SchemaValue::Absent,
                    },
                );
                src.set("concepts", SchemaValue::Unsigned(s.concepts as u64));
                src.set("morphisms", SchemaValue::Unsigned(s.morphisms as u64));
                SchemaValue::Record(src)
            })
            .collect();
        let loaded_sources = self
            .catalog
            .iter()
            .filter(|s| s.availability.is_loaded())
            .count();
        p.set(
            "source_count",
            SchemaValue::Unsigned(self.catalog.len() as u64),
        );
        p.set(
            "loaded_source_count",
            SchemaValue::Unsigned(loaded_sources as u64),
        );
        p.set("sources", SchemaValue::List(sources));
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
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if !describe_knowledge_base().is_empty() {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "KnowledgeBaseIsNonEmpty",
        "describe_knowledge_base() returns at least one registered Vocabulary",
        "Smith (1984) Reflection and Semantics in Lisp, POPL 1984"
    );
}
pr4xis::register_axiom!(
    KnowledgeBaseIsNonEmpty,
    "Smith (1984) Reflection and Semantics in Lisp, POPL 1984"
);

/// Axiom: SelfModelOntology is registered in the knowledge base.
/// The system can describe itself iff its own SelfModel ontology is
/// reachable through the auto-registration mechanism (linkme).
pub struct SelfModelIsRegistered;

impl Axiom for SelfModelIsRegistered {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if describe_knowledge_base()
            .iter()
            .any(|v| v.name() == "SelfModelOntology")
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "SelfModelIsRegistered",
        "SelfModelOntology is registered in the knowledge base",
        "Smith (1984) Reflection and Semantics in Lisp, POPL 1984"
    );
}
pr4xis::register_axiom!(
    SelfModelIsRegistered,
    "Smith (1984) Reflection and Semantics in Lisp, POPL 1984"
);

/// Axiom: KnowledgeOntology is registered in the knowledge base.
/// The Knowledge ontology — root of the registry — must register itself.
pub struct KnowledgeIsRegistered;

impl Axiom for KnowledgeIsRegistered {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if describe_knowledge_base()
            .iter()
            .any(|v| v.name() == "KnowledgeOntology")
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "KnowledgeIsRegistered",
        "KnowledgeOntology is registered in the knowledge base",
        "Smith (1984) Reflection and Semantics in Lisp, POPL 1984"
    );
}
pr4xis::register_axiom!(
    KnowledgeIsRegistered,
    "Smith (1984) Reflection and Semantics in Lisp, POPL 1984"
);

// --------------------------------------------------------------------------
// Wire-surface contract tests
//
// The `present() → to_json()` boundary is consumed by the chat UI
// (docs/chat/index.html). UI ↔ wire drift is the bug class that broke
// `.prx.gz` ontology loads in the dual-load meta page — the UI read
// `s.availability` while the wire emitted `s.status`. These tests pin
// the field names so a future rename can't silently regress the UI
// without a failing test alongside it.
// --------------------------------------------------------------------------
#[cfg(test)]
mod wire_surface {
    use super::*;
    use crate::formal::information::knowledge::catalog::{SourceAvailability, SourceStatus};
    use serde_json::Value;

    fn one_loaded_one_available() -> Vec<SourceStatus> {
        vec![
            SourceStatus {
                name: "biro".into(),
                version: "1.1.1".into(),
                kind: "OntologyVocabulary".into(),
                citation: "BiRO Bibliographic Reference Ontology".into(),
                availability: SourceAvailability::Loaded,
                staging: None,
                concepts: 14,
                morphisms: 1,
            },
            SourceStatus {
                name: "doco".into(),
                version: "1.3".into(),
                kind: "OntologyVocabulary".into(),
                citation: "DoCO Document Components Ontology".into(),
                availability: SourceAvailability::Available,
                staging: None,
                concepts: 0,
                morphisms: 0,
            },
        ]
    }

    fn parse(json: &str) -> Value {
        serde_json::from_str(json).expect("self_describe must emit valid JSON")
    }

    #[test]
    fn each_source_record_carries_the_field_names_the_chat_ui_reads() {
        // Field names the chat UI consumes (docs/chat/index.html) for
        // every per-source record in the catalog. Renaming any of these
        // on the wire without updating the UI is the regression this
        // test exists to catch.
        let required = ["name", "version", "kind", "source", "availability"];

        let json = SelfModelInstance::observe(Vec::new())
            .with_catalog(one_loaded_one_available())
            .to_json();
        let v = parse(&json);
        let sources = v
            .get("sources")
            .and_then(Value::as_array)
            .expect("self_describe JSON must include `sources` as an array");

        for (i, src) in sources.iter().enumerate() {
            for field in required {
                assert!(
                    src.get(field).is_some(),
                    "self_describe.sources[{i}] is missing required field `{field}` \
                     consumed by docs/chat/index.html: {src}"
                );
            }
        }
    }

    #[test]
    fn availability_field_takes_only_loaded_or_available() {
        // The UI compares `s.availability === 'loaded'` to decide
        // whether to render the card as `.source-card.loaded`. The
        // value space is the [`SourceAvailability`] label() output.
        let json = SelfModelInstance::observe(Vec::new())
            .with_catalog(one_loaded_one_available())
            .to_json();
        let v = parse(&json);
        let sources = v
            .get("sources")
            .and_then(Value::as_array)
            .expect("sources array");
        for src in sources {
            let availability = src
                .get("availability")
                .and_then(Value::as_str)
                .expect("availability must be a string");
            assert!(
                availability == "loaded" || availability == "available",
                "availability must be one of {{\"loaded\", \"available\"}}; got {availability:?}"
            );
        }
    }

    #[test]
    fn loaded_source_count_equals_count_of_loaded_records() {
        // The header pill (`{loaded}/{total} SOURCES LOADED`) reads
        // `data.loaded_source_count` + `data.source_count`. Catching
        // drift between the per-source `availability` field and the
        // top-level loaded count is the second half of the wire
        // contract.
        let json = SelfModelInstance::observe(Vec::new())
            .with_catalog(one_loaded_one_available())
            .to_json();
        let v = parse(&json);
        let loaded_count = v
            .get("loaded_source_count")
            .and_then(Value::as_u64)
            .expect("loaded_source_count present and unsigned");
        let total = v
            .get("source_count")
            .and_then(Value::as_u64)
            .expect("source_count present and unsigned");
        assert_eq!(loaded_count, 1);
        assert_eq!(total, 2);

        let sources = v
            .get("sources")
            .and_then(Value::as_array)
            .expect("sources array");
        let per_source_loaded = sources
            .iter()
            .filter(|s| s.get("availability").and_then(Value::as_str) == Some("loaded"))
            .count() as u64;
        assert_eq!(
            per_source_loaded, loaded_count,
            "top-level loaded_source_count must equal the number of per-source \
             records with availability == \"loaded\""
        );
    }
}
