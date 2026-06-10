//! Registry-driven loaded OWL vocabularies + the corpus-wide audit.
//!
//! Every `[sources.X]` of kind
//! [`SourceTaxonomyConcept::OntologyVocabulary`] in `praxis.toml` (the
//! SPAR family — CiTO, DoCO, C4O, BiRO — plus W3C PROV-O) is a *loaded*
//! runtime corpus here, materialised from its bundled RDF/XML the same way
//! [`crate::social::software::markup::xml::uslm::corpus::loaded`] loads the
//! U.S. Code titles and [`crate::cognitive::linguistics::english::English`]
//! loads WordNet.
//!
//! ## The hydration chain
//!
//! For one registered source, the runtime materialisation is:
//!
//! ```text
//! entry.local_path()  ─►  std::fs::read   (the bundled .owl bytes)
//!   └► build_envelope(bytes, name, version, url)        (read_owl → owl_to_builder → owned)
//!        └► env.data.to_codegen_data_leaked()           (CodegenData<LoadedOwlVocabulary>)
//!             └► LoadedOwlVocabulary::from_codegen       (the runtime corpus)
//! ```
//!
//! [`loaded_vocabularies`] walks every `OntologyVocabulary` entry the
//! registry declares, hydrates each through that chain, and caches the map
//! for the process lifetime behind a `OnceLock`. A registered source whose
//! bytes are on disk but fail to read or parse is a **defect**, so the
//! loader `panic!`s with the vocabulary name and the underlying error,
//! exactly as [`loaded`][crate::social::software::markup::xml::uslm::corpus::loaded]
//! panics on a registered title that fails to parse. A source registered
//! but *not* on disk is skipped (the same graceful skip the USC loader
//! makes for a title XML that isn't bundled).
//!
//! ## The corpus-wide audit
//!
//! [`audit_loaded_vocabularies`] is the heart of this module. Per
//! `feedback_corpus_wide_audit_on_load`, it walks **every** record of
//! **every** loaded vocabulary — not a spot-check — and fails loudly on the
//! first unresolved item:
//!
//! - every [`OwlEntityRecord`][rec] resolves: `find(iri)` is `Some`, the
//!   IRI is non-empty, and the round-trip index matches its position;
//! - every subsumption edge has both endpoints in `0..entity_count()`;
//! - the loaded class / property counts equal what [`read_owl`] saw in the
//!   same bytes, and equal the `omv:numberOfClasses` /
//!   `omv:numberOfProperties` [`build_envelope`] recorded — nothing was
//!   silently dropped.
//!
//! Counts are derived from the data and cross-checked for internal
//! consistency; no count is asserted against a hardcoded magic number, and
//! the set of vocabularies is discovered by walking the registry, never
//! enumerated.
//!
//! Alongside those asserted invariants the audit *reports* per-source
//! annotation coverage — how many entities the source actually annotates
//! with `rdfs:label` / `rdfs:comment`, counted from the source's real
//! `Option`s. Coverage is never asserted: a source may label every entity
//! or only a fraction (OLiA is legitimately sparse), and either is valid.
//!
//! ## Citations
//!
//! - **W3C OWL 2 Web Ontology Language: Structural Specification and
//!   Functional-Style Syntax (2nd ed.)**, Motik, Patel-Schneider & Parsia
//!   (eds.), W3C Recommendation 2012-12-11, §5 (Entities).
//!   <https://www.w3.org/TR/owl2-syntax/>.
//! - **RDF Schema 1.1**, Brickley & Guha (eds.), W3C Recommendation
//!   2014-02-25, §2.1 (`rdfs:subClassOf`), §5.1.7 (`rdfs:subPropertyOf`).
//!   <https://www.w3.org/TR/rdf-schema/>.
//! - **Hartmann, Palma & Sure (2005)** "OMV — Ontology Metadata
//!   Vocabulary", *ISWC 2005* — `omv:numberOfClasses` /
//!   `omv:numberOfProperties`, the structural metrics cross-checked here.
//!
//! [rec]: super::vocabulary::OwlEntityRecord
//! [`read_owl`]: super::reader::read_owl
//! [`build_envelope`]: super::prx::build_envelope

#[allow(unused_imports)]
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec::Vec,
};

use std::collections::HashMap;
use std::sync::OnceLock;

use pr4xis::codegen_data::CodegenData;

use super::prx::build_envelope;
use super::reader::read_owl;
use super::vocabulary::LoadedOwlVocabulary;
use crate::applied::data_provisioning::registry::data_sources;
use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;

/// The workspace root — the parent of this crate's parent (i.e.
/// `crates/domains/`'s grandparent). `RegistryEntry::local_path()` returns
/// a workspace-relative path (`crates/domains/data/...`), so it must be
/// resolved against this root, mirroring
/// [`crate::social::software::markup::xml::uslm::corpus::loaded`].
fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

/// Hydrate every registered [`OntologyVocabulary`][ov] source into a
/// `name → LoadedOwlVocabulary` map, materialised once per process behind a
/// `OnceLock`.
///
/// Walks every entry in [`data_sources`] whose `kind` is
/// [`SourceTaxonomyConcept::OntologyVocabulary`], reads its bundled RDF/XML
/// from `workspace_root.join(entry.local_path())`, and runs the
/// [module-level hydration chain][self] (`build_envelope` →
/// `to_codegen_data_leaked` → `from_codegen`). A registered source whose
/// bytes are on disk but fail to parse is a defect: the loader panics with
/// the vocabulary name and error. A source not on disk is skipped, matching
/// the USC corpus loader's graceful skip.
///
/// [ov]: crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept::OntologyVocabulary
pub fn loaded_vocabularies() -> &'static HashMap<String, LoadedOwlVocabulary> {
    static INSTANCE: OnceLock<HashMap<String, LoadedOwlVocabulary>> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        let root = workspace_root();
        let mut map: HashMap<String, LoadedOwlVocabulary> = HashMap::new();
        for entry in data_sources() {
            if entry.kind != SourceTaxonomyConcept::OntologyVocabulary {
                continue;
            }
            let path = root.join(entry.local_path());
            let Ok(bytes) = std::fs::read(&path) else {
                // Registered but not on disk — skip gracefully, exactly as
                // the USC corpus loader skips a title XML that isn't bundled.
                continue;
            };
            // On-disk but unparseable is a defect, not a skip.
            let env = build_envelope(&bytes, &entry.name, &entry.version, &entry.url)
                .unwrap_or_else(|e| {
                    panic!(
                        "loaded_vocabularies() failed building envelope for registered \
                         OntologyVocabulary `{}@{}` from {}: {e}",
                        entry.name,
                        entry.version,
                        path.display()
                    )
                });
            let data: CodegenData<LoadedOwlVocabulary> = env.data.to_codegen_data_leaked();
            let vocab = LoadedOwlVocabulary::from_codegen(&data);
            map.insert(entry.name.clone(), vocab);
        }
        map
    })
}

/// The loaded vocabulary registered under `registry_name` (the
/// `[sources.<name>]` key in `praxis.toml`), or `None` when no such
/// `OntologyVocabulary` source is registered or its bytes are not on disk.
///
/// The primary, registry-driven accessor — `loaded_vocabulary("cito")`
/// reads the same way `loaded()` reads the USC corpus.
pub fn loaded_vocabulary(registry_name: &str) -> Option<&'static LoadedOwlVocabulary> {
    loaded_vocabularies().get(registry_name)
}

/// Per-vocabulary structural metrics derived by the audit. Every field is
/// counted from the loaded corpus and the source `read_owl` returns — no
/// field is a hardcoded constant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VocabularyAuditMetrics {
    /// The registry name (`[sources.<name>]`).
    pub name: String,
    /// Total loaded entity records (classes + object properties).
    pub entity_count: usize,
    /// Loaded `owl:Class` entities.
    pub class_count: usize,
    /// Loaded `owl:ObjectProperty` entities.
    pub property_count: usize,
    /// Loaded subsumption edges (`rdfs:subClassOf` ∪ `rdfs:subPropertyOf`).
    pub subsumption_edge_count: usize,
    /// Entities the source actually annotates with an `rdfs:label` (RDF
    /// Schema §2.4). Counted from the source `Option`s, where an absent
    /// `rdfs:label` is `None` — *not* from the runtime accessor, which
    /// substitutes the IRI local name when a label is absent and so can
    /// never report a missing one. A coverage metric reported per source,
    /// never asserted: real ontologies (e.g. OLiA) are legitimately
    /// sparsely labelled.
    pub labeled_entities: usize,
    /// Entities the source actually annotates with an `rdfs:comment` (RDF
    /// Schema §2.5). Counted from the source `Option`s (absent comment is
    /// `None`), not the runtime `definition_of`, which yields an empty
    /// string for an absent comment. Reported, never asserted.
    pub commented_entities: usize,
}

/// One unresolved-item finding from the corpus-wide audit. Names the
/// vocabulary and the specific item that failed to resolve so a defect is
/// actionable, never silent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VocabularyAuditFinding {
    /// An entity record's IRI is empty (W3C OWL 2 §5.5 requires an IRI).
    EmptyIri { vocabulary: String, index: usize },
    /// An entity record's IRI does not resolve back through `find`.
    UnresolvedIri { vocabulary: String, iri: String },
    /// `find(iri)` resolved to a different index than the record's own
    /// position — the IRI index is inconsistent.
    IndexMismatch {
        vocabulary: String,
        iri: String,
        record_index: usize,
        found_index: usize,
    },
    /// A subsumption edge endpoint is out of range for the corpus.
    EdgeEndpointOutOfRange {
        vocabulary: String,
        edge: (usize, usize),
        entity_count: usize,
    },
    /// The loaded class count disagrees with what `read_owl` saw in the
    /// same bytes (something was silently dropped or duplicated).
    ClassCountDrift {
        vocabulary: String,
        loaded: usize,
        from_source: usize,
    },
    /// The loaded property count disagrees with what `read_owl` saw.
    PropertyCountDrift {
        vocabulary: String,
        loaded: usize,
        from_source: usize,
    },
    /// The loaded class / property counts disagree with the
    /// `omv:numberOfClasses` / `omv:numberOfProperties` the envelope
    /// recorded (Hartmann 2005).
    EnvelopeMetricDrift {
        vocabulary: String,
        loaded_classes: usize,
        loaded_properties: usize,
        envelope_classes: u64,
        envelope_properties: u64,
    },
}

/// Outcome of the corpus-wide audit over every registered loaded OWL
/// vocabulary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VocabularyAuditReport {
    /// Per-vocabulary derived metrics, one entry per loaded vocabulary, in
    /// registry (sorted-name) order.
    pub metrics: Vec<VocabularyAuditMetrics>,
    /// Every unresolved-item finding. Empty iff the whole corpus resolves.
    pub findings: Vec<VocabularyAuditFinding>,
}

impl VocabularyAuditReport {
    /// True iff every record of every vocabulary resolved cleanly (no
    /// findings) and at least one vocabulary was walked.
    #[must_use]
    pub fn is_fully_resolved(&self) -> bool {
        self.findings.is_empty() && !self.metrics.is_empty()
    }
}

/// Run the corpus-wide audit over every registered loaded OWL vocabulary.
///
/// For each [`OntologyVocabulary`][ov] entry discovered in
/// [`data_sources`] (never a hardcoded set), this re-reads the bundled
/// bytes, hydrates the corpus via [`loaded_vocabulary`], and:
///
/// 1. walks every [`OwlEntityRecord`][rec] and verifies `find(iri)` is
///    `Some`, the IRI is non-empty, and the resolved index equals the
///    record's own position;
/// 2. walks every subsumption edge and verifies both endpoints are valid
///    in-corpus indices (`< entity_count()`);
/// 3. cross-checks the loaded class / property counts against what
///    [`read_owl`] saw on the same bytes, and against the envelope's
///    `omv:numberOf*` metrics — proving nothing was silently dropped.
///
/// It also derives per-source annotation coverage (`labeled_entities` /
/// `commented_entities`) from the source's real `rdfs:label` /
/// `rdfs:comment` `Option`s. Coverage is *reported*, never asserted: a
/// source may annotate every entity (the SPAR + PROV-O vocabularies) or
/// only a fraction (OLiA), and both are valid.
///
/// Counts are derived from the data and checked for internal consistency;
/// nothing is asserted against a magic number.
///
/// [ov]: crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept::OntologyVocabulary
/// [rec]: super::vocabulary::OwlEntityRecord
/// [`read_owl`]: super::reader::read_owl
#[must_use]
pub fn audit_loaded_vocabularies() -> VocabularyAuditReport {
    let root = workspace_root();
    let mut metrics: Vec<VocabularyAuditMetrics> = Vec::new();
    let mut findings: Vec<VocabularyAuditFinding> = Vec::new();

    for entry in data_sources() {
        if entry.kind != SourceTaxonomyConcept::OntologyVocabulary {
            continue;
        }
        // Skip a source whose bytes aren't on disk — `loaded_vocabulary`
        // makes the same graceful skip, so it would return `None` anyway.
        let path = root.join(entry.local_path());
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };

        let vocab = loaded_vocabulary(&entry.name).unwrap_or_else(|| {
            panic!(
                "audit: registered OntologyVocabulary `{}` is on disk but not loaded",
                entry.name
            )
        });

        // (1) Every entity record resolves through the typed accessor.
        for (record_index, record) in vocab.entities().iter().enumerate() {
            if record.iri.is_empty() {
                findings.push(VocabularyAuditFinding::EmptyIri {
                    vocabulary: entry.name.clone(),
                    index: record_index,
                });
                continue;
            }
            match vocab.find(&record.iri) {
                None => findings.push(VocabularyAuditFinding::UnresolvedIri {
                    vocabulary: entry.name.clone(),
                    iri: record.iri.clone(),
                }),
                Some(found_index) if found_index != record_index => {
                    findings.push(VocabularyAuditFinding::IndexMismatch {
                        vocabulary: entry.name.clone(),
                        iri: record.iri.clone(),
                        record_index,
                        found_index,
                    })
                }
                Some(_) => {}
            }
        }

        // (2) Every subsumption edge has both endpoints in range.
        let entity_count = vocab.entity_count();
        for &(child, parent) in vocab.subsumption_edges() {
            if child >= entity_count || parent >= entity_count {
                findings.push(VocabularyAuditFinding::EdgeEndpointOutOfRange {
                    vocabulary: entry.name.clone(),
                    edge: (child, parent),
                    entity_count,
                });
            }
        }

        // (3) Structural cross-check against what read_owl actually saw.
        // read_owl deduplicates by IRI and never emits an empty-IRI entity,
        // and owl_to_builder emits one entity per distinct non-empty class
        // IRI + one per distinct non-empty property IRI — so the loaded
        // class / property counts must equal the source's distinct,
        // non-empty class / property IRI counts.
        let loaded_classes = vocab.classes().len();
        let loaded_properties = vocab.properties().len();

        let source_text = core::str::from_utf8(&bytes)
            .unwrap_or_else(|e| panic!("audit: `{}` source bytes are not UTF-8: {e}", entry.name));
        let ont = read_owl(source_text)
            .unwrap_or_else(|e| panic!("audit: re-read_owl of `{}` failed: {e}", entry.name));
        let source_classes = distinct_nonempty(ont.classes.iter().map(|c| c.iri.as_str()));
        let source_properties = distinct_nonempty(ont.properties.iter().map(|p| p.iri.as_str()));

        if loaded_classes != source_classes {
            findings.push(VocabularyAuditFinding::ClassCountDrift {
                vocabulary: entry.name.clone(),
                loaded: loaded_classes,
                from_source: source_classes,
            });
        }
        if loaded_properties != source_properties {
            findings.push(VocabularyAuditFinding::PropertyCountDrift {
                vocabulary: entry.name.clone(),
                loaded: loaded_properties,
                from_source: source_properties,
            });
        }

        // And the envelope's omv:numberOf* structural metrics must agree
        // with the loaded counts (Hartmann 2005 OMV).
        let env = build_envelope(&bytes, &entry.name, &entry.version, &entry.url)
            .unwrap_or_else(|e| panic!("audit: build_envelope for `{}` failed: {e}", entry.name));
        if env.metadata.number_of_classes != loaded_classes as u64
            || env.metadata.number_of_properties != loaded_properties as u64
        {
            findings.push(VocabularyAuditFinding::EnvelopeMetricDrift {
                vocabulary: entry.name.clone(),
                loaded_classes,
                loaded_properties,
                envelope_classes: env.metadata.number_of_classes,
                envelope_properties: env.metadata.number_of_properties,
            });
        }

        // Annotation coverage, counted from the source's real `Option`s:
        // `read_owl` already deduplicated `ont.classes` / `ont.properties`
        // by IRI, and an absent `rdfs:label` / `rdfs:comment` is `None`
        // (RDF Schema §2.4 / §2.5). This is the honest count the runtime
        // accessors cannot give — `label_of` falls back to the IRI local
        // name and `definition_of` to an empty string, so both always
        // report present. Reported, never asserted: coverage is a per-
        // source property, not a universal invariant.
        let has_text =
            |o: &Option<String>| o.as_deref().map(str::trim).is_some_and(|s| !s.is_empty());
        let labeled_entities = ont.classes.iter().filter(|c| has_text(&c.label)).count()
            + ont.properties.iter().filter(|p| has_text(&p.label)).count();
        let commented_entities = ont.classes.iter().filter(|c| has_text(&c.comment)).count()
            + ont
                .properties
                .iter()
                .filter(|p| has_text(&p.comment))
                .count();

        metrics.push(VocabularyAuditMetrics {
            name: entry.name.clone(),
            entity_count,
            class_count: loaded_classes,
            property_count: loaded_properties,
            subsumption_edge_count: vocab.subsumption_edges().len(),
            labeled_entities,
            commented_entities,
        });
    }

    VocabularyAuditReport { metrics, findings }
}

/// Count the distinct, non-empty strings in an iterator — the cardinality
/// `read_owl` + `owl_to_builder` collapse a class / property list to (both
/// dedup by IRI and drop empty IRIs).
fn distinct_nonempty<'a>(iter: impl Iterator<Item = &'a str>) -> usize {
    let mut seen: hashbrown::HashSet<&str> = hashbrown::HashSet::new();
    for s in iter {
        if !s.is_empty() {
            seen.insert(s);
        }
    }
    seen.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social::software::markup::xml::owl::vocabulary::OwlEntityKind;

    /// The loader discovers at least one `OntologyVocabulary` from the
    /// registry and hydrates it. The count is derived (every registered,
    /// on-disk source), never asserted against a magic number — only that
    /// the bundled SPAR + PROV-O sources actually loaded.
    #[test]
    fn loads_registered_ontology_vocabularies() {
        let map = loaded_vocabularies();
        // Every registered, on-disk OntologyVocabulary appears in the map;
        // since the SPAR + PROV-O files are bundled, the map is non-empty.
        let registered_on_disk = data_sources()
            .iter()
            .filter(|e| e.kind == SourceTaxonomyConcept::OntologyVocabulary)
            .filter(|e| {
                workspace_root()
                    .join(e.local_path())
                    .try_exists()
                    .unwrap_or(false)
            })
            .count();
        assert_eq!(
            map.len(),
            registered_on_disk,
            "every registered, on-disk OntologyVocabulary must be loaded"
        );
        assert!(
            registered_on_disk > 0,
            "the bundled SPAR + PROV-O vocabularies must be on disk and loaded"
        );
    }

    /// `loaded_vocabulary` is the registry-driven accessor and returns the
    /// same `&'static` corpus the map holds. CiTO is the canonical SPAR
    /// example; its `cites` / `citesAsEvidence` properties must resolve and
    /// stand in the documented `subPropertyOf` relation.
    #[test]
    fn loaded_vocabulary_resolves_cito_subproperty() {
        let Some(cito) = loaded_vocabulary("cito") else {
            panic!("cito must be a registered, on-disk OntologyVocabulary");
        };
        const CITES: &str = "http://purl.org/spar/cito/cites";
        const CITES_AS_EVIDENCE: &str = "http://purl.org/spar/cito/citesAsEvidence";
        let cae = cito
            .find(CITES_AS_EVIDENCE)
            .expect("citesAsEvidence must be loaded");
        assert_eq!(
            cito.entity(cae).unwrap().kind,
            OwlEntityKind::ObjectProperty
        );
        assert!(
            cito.is_a(CITES_AS_EVIDENCE, CITES),
            "citesAsEvidence subsumes under cites (CiTO rdfs:subPropertyOf)"
        );
    }

    /// The corpus-wide audit walks every record of every vocabulary and
    /// finds zero unresolved items. This is the heart of the milestone:
    /// per-vocabulary `entity / class / property / edge` counts are derived
    /// here and printed for the report, and `is_fully_resolved()` is the
    /// pass condition.
    #[test]
    fn corpus_wide_audit_fully_resolves() {
        let report = audit_loaded_vocabularies();
        assert!(
            report.findings.is_empty(),
            "corpus-wide audit surfaced unresolved items: {:?}",
            report.findings
        );
        assert!(
            report.is_fully_resolved(),
            "audit must walk ≥1 vocabulary and resolve every record"
        );

        // Print the derived per-vocabulary counts so the audit's coverage
        // is visible (proves it actually walked the corpus, not a sample).
        // Annotation coverage (labeled / commented) is printed too — a
        // per-source metric, never asserted against a threshold.
        for m in &report.metrics {
            println!(
                "owl-vocabulary-audit: {} — entities={} classes={} properties={} subsumption_edges={} labeled={}/{} commented={}/{}",
                m.name,
                m.entity_count,
                m.class_count,
                m.property_count,
                m.subsumption_edge_count,
                m.labeled_entities,
                m.entity_count,
                m.commented_entities,
                m.entity_count
            );
            // Internal consistency: classes + properties partition the
            // entity set (every OWL entity is one or the other, W3C OWL 2
            // §5), and every vocabulary carries a non-trivial corpus.
            assert_eq!(
                m.class_count + m.property_count,
                m.entity_count,
                "{}: classes + properties must partition the entity set",
                m.name
            );
            assert!(
                m.entity_count > 0,
                "{}: a loaded vocabulary is non-empty",
                m.name
            );
            // Coverage counts are bounded by the entity count — never more
            // entities are annotated than exist. (A structural sanity check
            // on the derived metric, not a coverage threshold.)
            assert!(
                m.labeled_entities <= m.entity_count && m.commented_entities <= m.entity_count,
                "{}: annotation coverage cannot exceed the entity count",
                m.name
            );
        }
    }

    /// Every entity in every loaded vocabulary round-trips through the
    /// typed accessors: `find(iri)` → `entity(idx)` → same IRI. A separate,
    /// stricter walk than the audit's index-equality check — it exercises
    /// the `entity(usize)` accessor on every record of the whole corpus.
    #[test]
    fn every_entity_round_trips_through_accessors() {
        for (name, vocab) in loaded_vocabularies() {
            for record in vocab.entities() {
                let idx = vocab
                    .find(&record.iri)
                    .unwrap_or_else(|| panic!("{name}: IRI {} did not resolve", record.iri));
                let back = vocab
                    .entity(idx)
                    .unwrap_or_else(|| panic!("{name}: index {idx} out of range"));
                assert_eq!(
                    back.iri, record.iri,
                    "{name}: find→entity round-trip must return the same IRI"
                );
            }
        }
    }

    /// The map accessor and the named accessor agree (same pointer), and a
    /// second call returns the cached instance.
    #[test]
    fn accessor_is_cached_and_consistent() {
        let map1 = loaded_vocabularies();
        let map2 = loaded_vocabularies();
        assert!(core::ptr::eq(map1, map2), "loaded map is cached");
        if let Some(named) = loaded_vocabulary("cito") {
            let via_map = map1.get("cito").expect("cito in map");
            assert!(
                core::ptr::eq(named, via_map),
                "named accessor and map return the same corpus"
            );
        }
    }
}
