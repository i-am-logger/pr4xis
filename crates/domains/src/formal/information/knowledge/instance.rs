#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::cognitive::cognition::self_model::AwarenessLevel;
use crate::formal::information::knowledge::catalog::{SourceStatus, staging_label};
use crate::formal::information::knowledge::vocabulary::OntologyCapability;
use crate::formal::information::schema::transport::{Present, Presentation, SchemaValue};
use pr4xis::ontology::{Axiom, Vocabulary, describe_knowledge_base};

// SelfModelInstance — runtime eigenform of the SelfModel ontology.
//
// This is the bridge between the pure ontology (self_model.rs)
// and the runtime system. Constructing this IS the self-observation
// operator F from von Foerster. The result IS X = F(X).

/// The system's canonical self-identity name — the single typed source for
/// "what the system calls itself" (MAPE-K `Ksys` identity; von Foerster
/// eigenform). Every surface that names the system reads this rather than
/// re-spelling the literal.
pub const SYSTEM_NAME: &str = "pr4xis";

/// The surface forms that *denote the system itself* — its self-referents.
///
/// A reference is self-referential iff its surface form is in this set. The
/// set is owned by the self-model layer (not enumerated at a call site): it is
/// the system's identity name [`SYSTEM_NAME`] together with its conventional
/// spelling variant, plus the second-person indexicals English resolves to the
/// addressee. In a single-agent self-model the addressee of a question *is* the
/// system, so "you"/"yourself" denote it (Kaplan 1989, *Demonstratives* — the
/// indexical's referent is fixed by the utterance context, here the system as
/// the sole conversational agent).
///
/// This is the smallest typed step toward a full self-model lexical bridge:
/// the routing decision consults this self-model-owned set instead of bare
/// word literals. The remaining work — resolving each token to a SelfModel
/// `ConceptId`/`SenseId` and testing membership in the SelfModel reflexive
/// closure — is tracked as a follow-up (no indexical→SelfModel sense
/// resolution exists in the pipeline yet).
pub fn self_referents() -> [&'static str; 4] {
    // [identity name, spelling variant, 2nd-person indexical, reflexive form]
    [SYSTEM_NAME, "praxis", "you", "yourself"]
}

/// Whether `surface` denotes the system itself — membership in
/// [`self_referents`]. The typed self-reference predicate the conversational
/// layer asks the self-model, rather than comparing word literals inline.
pub fn is_self_referent(surface: &str) -> bool {
    self_referents().contains(&surface)
}

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
    /// What each loaded ontology can ANSWER (doc §4.7) — gloss / which relation
    /// kinds its closure populates. So the self-model is honest about capability,
    /// not just size: a Parthood-only USC card no longer shows green while its
    /// taxonomy queries are dark. Empty until [`with_capabilities`] is attached.
    ///
    /// [`with_capabilities`]: Self::with_capabilities
    pub capabilities: Vec<OntologyCapability>,
    /// The append-only LOAD HISTORY (doc §2.4) — the temporal dimension the
    /// system entirely lacked. Each event records a `.prx` becoming part of the
    /// reasoned-over set, content-addressed by its Merkle root, so the system
    /// REMEMBERS what it loaded and in what order. Empty until [`with_history`]
    /// is attached.
    ///
    /// [`with_history`]: Self::with_history
    pub history: Vec<LoadEvent>,
    /// The content-addressed fingerprint of the CURRENT loaded state — a Merkle
    /// fold over the sorted loaded roots (doc §2.4). Two systems with the same
    /// `state_cid` have loaded exactly the same knowledge; it changes the moment a
    /// load does. `None` until [`with_history`](Self::with_history) supplies it.
    pub state_cid: Option<String>,
    /// The wasm linear-memory footprint in bytes at observation time (U2) — the
    /// self-model reporting its OWN live size in the host, the byte dimension of
    /// the eigenform alongside its concept/morphism counts. `None` off-wasm (a
    /// native build has no single linear-memory measure) and until
    /// [`with_footprint`](Self::with_footprint) supplies it; the presentation then
    /// omits `linear_memory_bytes` entirely.
    pub footprint_bytes: Option<u64>,
}

/// What kind of load an event records (doc §2.4 / §4.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadEventKind {
    /// A `.prx` not previously loaded under its name entered the set.
    Load,
    /// A `.prx` DISPLACED an earlier one of the same `OntologyName` (a new
    /// version replacing the old — the displaced root is carried).
    Replace,
}

/// One entry in the append-only load history (doc §2.4) — a content-addressed
/// record of a `.prx` joining the reasoned-over set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadEvent {
    pub kind: LoadEventKind,
    /// The ontology's name.
    pub ontology: String,
    /// The Merkle root (hex) of the loaded archive — its content identity.
    pub root: String,
    /// The root (hex) of the displaced archive, for a [`Replace`](LoadEventKind::Replace).
    pub displaced: Option<String>,
}

impl SelfModelInstance {
    /// The self-observation operator F. X = F(X).
    pub fn observe(components: Vec<Vocabulary>) -> Self {
        let total_concepts = components.iter().map(|v| v.concept_count()).sum();
        let total_morphisms = components.iter().map(|v| v.morphism_count()).sum();
        Self {
            name: SYSTEM_NAME,
            version: env!("CARGO_PKG_VERSION"),
            awareness: AwarenessLevel::MetaSelf,
            components,
            catalog: Vec::new(),
            total_concepts,
            total_morphisms,
            capabilities: Vec::new(),
            history: Vec::new(),
            state_cid: None,
            footprint_bytes: None,
        }
    }

    /// Attach the source catalog — the meta-level's model of the
    /// knowledge boundary (loaded vs available registered sources).
    pub fn with_catalog(mut self, catalog: Vec<SourceStatus>) -> Self {
        self.catalog = catalog;
        self
    }

    /// Attach the per-ontology capabilities (doc §4.7) — what each loaded
    /// ontology can actually answer, so "loaded" stops lying about capability.
    pub fn with_capabilities(mut self, capabilities: Vec<OntologyCapability>) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Attach the load history + the current state fingerprint (doc §2.4) — the
    /// system's MEMORY of what it loaded and a content-addressed identity of its
    /// current knowledge state.
    pub fn with_history(mut self, history: Vec<LoadEvent>, state_cid: Option<String>) -> Self {
        self.history = history;
        self.state_cid = state_cid;
        self
    }

    /// Attach the wasm linear-memory footprint (U2) — the self-model reporting its
    /// own live size in the host. `None` off-wasm (no single linear-memory measure
    /// natively), in which case the presentation omits `linear_memory_bytes`.
    pub fn with_footprint(mut self, bytes: Option<u64>) -> Self {
        self.footprint_bytes = bytes;
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
        // U2: the self-model's own live linear-memory footprint, present only when
        // observed on wasm (omitted off-wasm — no single native measure).
        if let Some(bytes) = self.footprint_bytes {
            p.set("linear_memory_bytes", SchemaValue::Unsigned(bytes));
        }

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

        // Per-ontology capabilities (doc §4.7) — what each loaded ontology can
        // ANSWER (gloss / the relation kinds its closure populates), so a UI can
        // show capability, not just a green "loaded" badge.
        let capabilities: Vec<SchemaValue> = self
            .capabilities
            .iter()
            .map(|c| {
                let mut cap = Presentation::new();
                cap.set("ontology", SchemaValue::Text(c.ontology.clone()));
                cap.set("gloss", SchemaValue::Boolean(c.gloss));
                cap.set(
                    "relation_kinds",
                    SchemaValue::List(
                        c.relation_kinds
                            .iter()
                            .map(|k| SchemaValue::Text(k.clone()))
                            .collect(),
                    ),
                );
                SchemaValue::Record(cap)
            })
            .collect();
        p.set("capabilities", SchemaValue::List(capabilities));

        // The load history + the content-addressed state fingerprint (doc §2.4) —
        // the temporal/memory dimension: what was loaded, in order, and an identity
        // of the current knowledge state that changes the moment a load does.
        let history: Vec<SchemaValue> = self
            .history
            .iter()
            .map(|e| {
                let mut ev = Presentation::new();
                ev.set(
                    "event",
                    SchemaValue::Text(
                        match e.kind {
                            LoadEventKind::Load => "load",
                            LoadEventKind::Replace => "replace",
                        }
                        .into(),
                    ),
                );
                ev.set("ontology", SchemaValue::Text(e.ontology.clone()));
                ev.set("root", SchemaValue::Text(e.root.clone()));
                if let Some(displaced) = &e.displaced {
                    ev.set("displaced", SchemaValue::Text(displaced.clone()));
                }
                SchemaValue::Record(ev)
            })
            .collect();
        p.set("history", SchemaValue::List(history));
        if let Some(cid) = &self.state_cid {
            p.set("state_cid", SchemaValue::Text(cid.clone()));
        }
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

#[cfg(test)]
mod self_reference {
    use super::*;

    #[test]
    fn the_system_identity_is_a_self_referent() {
        // The system's own name denotes itself.
        assert!(is_self_referent(SYSTEM_NAME));
        assert!(is_self_referent("praxis"));
    }

    #[test]
    fn second_person_indexicals_denote_the_addressee_system() {
        // In a single-agent self-model the addressee IS the system, so the
        // 2nd-person indexicals are self-referents (Kaplan 1989).
        assert!(is_self_referent("you"));
        assert!(is_self_referent("yourself"));
    }

    #[test]
    fn an_unrelated_word_is_not_a_self_referent() {
        assert!(!is_self_referent("dog"));
        assert!(!is_self_referent(""));
    }

    #[test]
    fn the_identity_name_is_in_the_self_referent_set() {
        // The set is derived from the self-model identity, not re-spelled.
        assert!(self_referents().contains(&SYSTEM_NAME));
    }
}

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
