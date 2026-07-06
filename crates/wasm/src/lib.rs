use wasm_bindgen::prelude::*;

use pr4xis::ontology::Staging;
use pr4xis::ontology::meta::OntologyName;
use pr4xis_domains::applied::data_provisioning::registry::{
    lock_archive_signature, lock_canonical_signature, lock_hashes,
};
use pr4xis_domains::cognitive::linguistics::composed::ComposedReasoner;
use pr4xis_domains::cognitive::linguistics::english::English;
use pr4xis_domains::formal::information::knowledge::{
    LoadEvent, LoadEventKind, LoadedRef, ontology_capabilities, runtime_ontology_vocabulary,
    source_catalog,
};
use pr4xis_domains::formal::information::schema::transport::{Presentation, SchemaValue};
use pr4xis_domains::social::software::markup::xml::lmf::compact_succinct::load_prx_gz as load_english_prx;
use pr4xis_domains::social::software::markup::xml::owl::bridge::owl_runtime_ontology;
use pr4xis_domains::social::software::markup::xml::owl::prx::load_prx_gz;
use pr4xis_domains::social::software::markup::xml::owl::reader::read_owl;
use pr4xis_domains::social::software::markup::xml::owl::vocabulary::LoadedOwlVocabulary;
use pr4xis_domains::social::software::markup::xml::uslm::corpus::UsCode;
use pr4xis_domains::social::software::markup::xml::uslm::corpus::bridge::usc_runtime_ontology;
use pr4xis_domains::social::software::markup::xml::uslm::lens::read_uslm_title;
use pr4xis_runtime::address::ContentAddress;
use pr4xis_runtime::ontology::{RuntimeOntology, materialize};

/// The complete WordNet ontology, baked in as the compact `.prx.gz` (emitted by
/// build.rs). `load_english` gunzips and materializes the full `English`.
const ENGLISH_PRX_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/english.prx.gz"));

fn load_english() -> English {
    load_english_prx(ENGLISH_PRX_GZ)
}

/// Build-time catalog of the authoritative source documents available to
/// download at runtime — emitted by build.rs from the registered USC
/// title XML on disk. `(registry name, version, served URL, byte size)`.
mod sources_manifest {
    include!(concat!(env!("OUT_DIR"), "/sources_manifest.rs"));
}

/// Build-time catalog of the registered OWL vocabularies and the metadata
/// the dual-load needs: `(name, version, prx_url, source_url, lock_pin)`.
/// The `lock_pin` is the praxis.lock source-hash for `name@version`, baked
/// in because the wasm runtime has no filesystem to read the lock from —
/// it is the value the `.prx.gz` source-hash gate validates against.
mod ontologies_manifest {
    include!(concat!(env!("OUT_DIR"), "/ontologies_manifest.rs"));
}

/// The embedded new-format `.prx` demo ontology — the Avizienis et al. (2004)
/// Dependability taxonomy, projected to a content-addressed Archive by
/// `build.rs` (via `emit::<DependabilityCategory>()`), with its bytes and
/// trusted Merkle root baked in. The browser loads these bytes fail-closed
/// against the root without any network — and a fetched/uploaded `.prx` would
/// flow through the exact same [`Pr4xis::load_ontology_prx`] path.
mod embedded_prx {
    include!(concat!(env!("OUT_DIR"), "/embedded_prx.rs"));
}

/// Registry primary key of the embedded English base
/// (praxis.toml `[sources.english_wordnet]`). English is the one source
/// processed at build time and baked in (`Embedded` staging); every other
/// registered source is downloaded from its authoritative document at
/// runtime and parsed into a live ontology (`Async` staging).
const ENGLISH_SOURCE: &str = "english_wordnet";

/// The runtime. Source-agnostic: it holds the embedded English language
/// model plus a set of on-demand-loaded ontologies. It has no notion of
/// "statute" — a loaded U.S. Code title is just one [`RuntimeOntology`]
/// among whatever the registry offers.
#[wasm_bindgen]
pub struct Pr4xis {
    english: English,
    /// Every runtime-loaded source — USC titles, OWL vocabularies, and
    /// new-format `.prx` ontologies — projected into the generic
    /// [`Archive`](pr4xis_runtime::archive::Archive) by its functor-as-data
    /// bridge and materialized into one queryable [`RuntimeOntology`] set
    /// (content-address identity). THE single loaded-knowledge collection: the
    /// chat reasons over all of it (grounded into English by `composed`) and the
    /// self-model catalog reports all of it. No source is held aside in a second
    /// collection the reasoner never sees.
    runtime_ontologies: Vec<RuntimeOntology>,
    /// The embedded English model COMPOSED with the loaded `.prx` ontologies as
    /// one [`ComposedReasoner`] — `None` until at least one `.prx` is loaded.
    /// Rebuilt whenever `runtime_ontologies` changes (a rare, deliberate load
    /// action), so the per-chat path is a cheap branch, not a re-grounding.
    /// When present, `chat` reasons through it (the loaded gloss answers);
    /// when absent, `chat` reasons through `english` alone (it abstains on an
    /// unloaded concept, exactly as today).
    composed: Option<ComposedReasoner>,
    /// The append-only load history (doc §2.4) — the system's MEMORY of what it
    /// loaded, in order, content-addressed by root. Surfaced in `self_describe`.
    history: Vec<LoadEvent>,
}

/// Why loading a new-format `.prx` failed — the typed core error, rendered to a
/// `JsValue` only at the wasm boundary. Carries the underlying runtime verdict
/// so the failure is precise (a wrong hex pin, a refused root, a dangling edge),
/// never a stringly-typed blob.
#[derive(Debug)]
enum LoadPrxError {
    /// `expected_root_hex` was not a 64-char lowercase-hex digest.
    BadRootHex(String),
    /// The fail-closed gate refused the bytes — decode failure or a re-derived
    /// root that does not match the trusted root (tampered / stale / wrong).
    Refused(pr4xis_runtime::load::LoadError),
    /// The admitted archive could not be materialized (e.g. a dangling edge —
    /// referential closure violated).
    Materialize(pr4xis_runtime::ontology::MaterializeError),
}

impl core::fmt::Display for LoadPrxError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LoadPrxError::BadRootHex(got) => {
                write!(
                    f,
                    "expected_root must be 64-char lowercase hex; got {got:?}"
                )
            }
            LoadPrxError::Refused(e) => write!(f, ".prx load refused: {e}"),
            LoadPrxError::Materialize(e) => write!(f, ".prx materialize failed: {e}"),
        }
    }
}

impl Default for Pr4xis {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl Pr4xis {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        let mut this = Self {
            english: load_english(),
            runtime_ontologies: Vec::new(),
            composed: None,
            history: Vec::new(),
        };
        // Install the always-loaded LegalSources BASE — the LKIF-Core formal
        // sources-of-law taxonomy, baked in by build.rs (`emit_with_forms`, so its
        // labels "law"/"case law" ride as `ontolex:Form` surfaces). It goes in
        // through the EXACT fail-closed core a fetched/uploaded `.prx` takes, so
        // from construction `composed` is `Some(...)` and EVERY chat reasons over
        // the formal sources of law: "is a statute a law" answers Yes out of the
        // box, with no explicit load. A failure here is a build-time invariant
        // violation (the bytes + pin ship embedded in the wasm).
        this.load_ontology_prx_core(
            embedded_prx::EMBEDDED_LEGAL_SOURCES_PRX,
            embedded_prx::EMBEDDED_LEGAL_SOURCES_ONTOLOGY_NAME.to_string(),
            embedded_prx::EMBEDDED_LEGAL_SOURCES_PRX_ROOT_HEX,
        )
        .expect("the embedded LegalSources base .prx loads fail-closed against its baked root");
        this
    }

    pub fn chat(&self, input: &str) -> String {
        // Once a `.prx` is loaded, reason through the ComposedReasoner (the
        // loaded ontology grounded into English) so "what is X" answers from
        // the loaded gloss; otherwise reason through English alone (abstains on
        // an unloaded concept). The linguistic substrate (`lang`) is always the
        // composed reasoner's own English when present, so tokenize/parse and
        // the lexical surface agree.
        let result = match &self.composed {
            Some(composed) => {
                pr4xis_chat::process_with_reasoner(composed.english(), composed, input)
            }
            None => pr4xis_chat::process_with_metadata(&self.english, input),
        };
        let reasoned = result.trace.reasoned_over();
        let trace = result.trace.serialize_with_functors();

        let mut p = Presentation::new();
        p.set("response", result.response.into());
        p.set("duration_us", result.duration_us.into());
        p.set("parsed", result.parsed.into());
        p.set("from_ontology", result.from_ontology.into());
        // The TYPED outcome (doc §4.1): answered, or abstained naming the surfaces
        // to load — so the UI can model what the system cannot answer, not sniff it.
        match &result.outcome {
            pr4xis_chat::ChatOutcome::Answered => {
                p.set("outcome", "answered".into());
            }
            pr4xis_chat::ChatOutcome::Abstained { unresolved } => {
                p.set("outcome", "abstained".into());
                p.set(
                    "unresolved",
                    SchemaValue::List(unresolved.iter().map(|s| s.clone().into()).collect()),
                );
            }
        }
        // U6/U7: the ontologies this answer REASONED OVER — the compiled pipeline
        // PLUS every loaded `.prx` it drew on (`reasoned_over`, not the compiled-only
        // `all_participating_ontologies`), each a structured record carrying its
        // provenance + success bit. The page projects this, so the list GENERALISES
        // as ontologies load — never a hardcoded pipeline.
        p.set(
            "ontologies",
            SchemaValue::List(
                reasoned
                    .into_iter()
                    .map(|(ont, success)| {
                        let mut r = Presentation::new();
                        r.set("ontology", SchemaValue::Text(ont.name().to_string()));
                        r.set("kind", SchemaValue::Text(ont.provenance().to_string()));
                        r.set("success", SchemaValue::Boolean(success));
                        SchemaValue::Record(r)
                    })
                    .collect(),
            ),
        );
        p.set("trace", trace.into());
        p.to_json()
    }

    pub fn concept_count(&self) -> usize {
        self.english.concept_count()
    }

    pub fn word_count(&self) -> usize {
        self.english.word_count()
    }

    /// Total queryable units across every runtime-loaded ontology (0 until one
    /// is loaded): the sum of node counts over `runtime_ontologies` — USC
    /// provisions, OWL entities, and loaded `.prx` concepts alike. Source-agnostic
    /// (the unification: every loaded source now contributes through the one set).
    pub fn loaded_section_count(&self) -> usize {
        // CONCEPTS only — the §9 `ontolex:Form` surface atoms are not concepts.
        // Counted through the ONE blessed Form-aware lowering
        // (`runtime_ontology_vocabulary`), the same counter `self_describe`'s
        // eigenform and `loaded_refs` use, so the three can never drift apart.
        self.runtime_ontologies
            .iter()
            .map(|o| runtime_ontology_vocabulary(o).concept_count())
            .sum()
    }

    /// Load a registered USLM source from its authoritative XML (downloaded
    /// by the host from the source's served document). Parses in-browser into a
    /// transient [`UsCode`], then PROJECTS it through the `uslm::corpus::bridge`
    /// functor into one queryable [`RuntimeOntology`] held in the chat-reasoning
    /// set — the SAME `project → materialize → install` path the `.prx` and OWL
    /// loads use (the `UsCode` itself is not retained; there is no `self.loaded`
    /// corpus held aside from the reasoner). This IS the Nelson-Narens *control*
    /// operation: Available → Loaded. A malformed document fails closed via the
    /// USLM reader. Idempotent by name: loading a name already present replaces it.
    pub fn load_source(&mut self, name: String, xml: &str) -> Result<(), JsValue> {
        self.load_source_core(name, xml)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// The plain-Rust core of [`Self::load_source`]: parse USLM → [`UsCode`] →
    /// **project into the generic runtime [`Archive`](pr4xis_runtime::archive::Archive)** (the
    /// `uslm::corpus::bridge` functor) → [`materialize`] into one queryable
    /// [`RuntimeOntology`] → install into the chat-reasoning set via
    /// [`Self::install_runtime_ontology`]. This is the SAME
    /// `project → materialize → install` path the `.prx` load uses, so a loaded
    /// statute title is reasoned over by [`Self::chat`] exactly like any other
    /// loaded ontology — no longer held aside as a `self.loaded` corpus the
    /// reasoner never sees. Native-testable (typed `String` error, no `JsValue`).
    fn load_source_core(&mut self, name: String, xml: &str) -> Result<(), String> {
        let title = read_uslm_title(xml).map_err(|e| format!("USLM parse failed: {e:?}"))?;
        let usc = UsCode::from_uslm_titles_owned(vec![title]);
        let onto = usc_runtime_ontology(&usc, OntologyName::new(name))
            .map_err(|e| format!("USC materialize failed: {e:?}"))?;
        self.install_runtime_ontology(onto);
        Ok(())
    }

    /// Load a registered OWL vocabulary from its `.prx.gz` distribution
    /// envelope (downloaded by the host from the vocabulary's served
    /// `prx_url`). **Content-addressed, fail-closed, identity-bound**: all
    /// three pins are looked up by the caller's `(name, version)` from the
    /// embedded `praxis.lock`, then the gate gunzips, bytecheck-validates the
    /// rkyv envelope, and verifies three content-hash integrity claims — the
    /// archive's `MerkleRoot` (re-derived from the envelope's own bytes), the
    /// source pin, and the RDFC-1.0 graph-identity pin (the canonical N-Quads
    /// of the loaded source graph). Because the pins are the *caller-named*
    /// vocabulary's, a genuine archive for a DIFFERENT vocabulary fails (its
    /// `MerkleRoot` won't match the named pin), so the install key cannot
    /// disagree with the loaded content. On any mismatch nothing is installed.
    /// Idempotent.
    ///
    /// This differs from [`Self::load_owl_source`] in *where* trust is
    /// anchored: `load_prx` re-derives the content address from the bytes it
    /// is about to install and checks it against the lock, so a tampered or
    /// stale `.prx.gz` is rejected even if the transport was honest.
    pub fn load_prx(
        &mut self,
        name: String,
        version: String,
        prx_gz: &[u8],
    ) -> Result<(), JsValue> {
        let key = format!("{name}@{version}");
        let archive_pin = lock_archive_signature(&name, &version).ok_or_else(|| {
            JsValue::from_str(&format!(
                "no embedded praxis.lock [archive_signatures] pin for {key}; cannot validate .prx.gz"
            ))
        })?;
        let source_pin = lock_hashes().get(&key).ok_or_else(|| {
            JsValue::from_str(&format!(
                "no embedded praxis.lock pin for {key}; cannot validate .prx.gz"
            ))
        })?;
        let canonical_pin = lock_canonical_signature(&name, &version).ok_or_else(|| {
            JsValue::from_str(&format!(
                "no embedded praxis.lock [canonical_signatures] pin for {key}; cannot validate .prx.gz"
            ))
        })?;
        let vocab = load_prx_gz(prx_gz, archive_pin, source_pin, canonical_pin).map_err(|e| {
            JsValue::from_str(&format!(".prx.gz load/validate failed for {key}: {e}"))
        })?;
        // Project the OWL vocabulary into the generic runtime ontology (the
        // functor-as-data `owl::bridge`) and install it into the ONE
        // chat-reasoning set — no longer held aside in a corpus the reasoner
        // never sees.
        let onto = owl_runtime_ontology(&vocab, OntologyName::new(name))
            .map_err(|e| JsValue::from_str(&format!("OWL materialize failed for {key}: {e:?}")))?;
        self.install_runtime_ontology(onto);
        Ok(())
    }

    /// Load a registered OWL vocabulary from its authoritative `.owl`
    /// source (downloaded by the host from the vocabulary's served
    /// `source_url`). Parses in-browser via the pure-Rust OWL reader and
    /// materialises into the same [`LoadedOwlVocabulary`] corpus the
    /// `.prx.gz` path produces (`read_owl` →
    /// [`LoadedOwlVocabulary::from_owl_ontology`]). Idempotent.
    ///
    /// **Trust model — note the contrast with [`Self::load_prx`].** This
    /// path trusts the fetched bytes: integrity rests on the host having
    /// fetched from the pinned `source_url`. It does not re-hash here,
    /// because the source bytes carry no embedded hash — the `.prx.gz`
    /// envelope does, which is why `load_prx` re-validates against the lock
    /// pin and this path does not. A malformed document fails closed via
    /// the OWL reader.
    pub fn load_owl_source(&mut self, name: String, owl_xml: &str) -> Result<(), JsValue> {
        let ont =
            read_owl(owl_xml).map_err(|e| JsValue::from_str(&format!("OWL parse failed: {e}")))?;
        let vocab = LoadedOwlVocabulary::from_owl_ontology(&ont);
        let onto = owl_runtime_ontology(&vocab, OntologyName::new(name))
            .map_err(|e| JsValue::from_str(&format!("OWL materialize failed: {e:?}")))?;
        self.install_runtime_ontology(onto);
        Ok(())
    }

    /// Load a NEW-FORMAT `.prx` ontology — the content-addressed
    /// [`Archive`](pr4xis_runtime::archive::Archive)
    /// (not the legacy `.prx.gz` envelope) — fail-closed, and ground it into the
    /// chat so the loaded gloss can answer "what is X".
    ///
    /// **Fail-closed.** `expected_root_hex` is the trusted Merkle root from
    /// OUTSIDE the bytes (here the build-baked root of the embedded demo; for a
    /// fetched/uploaded `.prx` it would be the peer's / lock's pin). The kernel
    /// [`load::load`](pr4xis_runtime::load::load) DECODES the bytes, RE-DERIVES
    /// the archive's root from the content it is about to admit, and admits the
    /// archive only if it equals the trusted root — a tampered, stale, or wrong
    /// `.prx` is REFUSED, never loaded. The admitted archive is then
    /// [`materialize`]d into a live [`RuntimeOntology`] (its Subsumption closure
    /// folded once), and the [`ComposedReasoner`] is rebuilt so `chat` reasons
    /// over it. Idempotent by content address: re-loading the same archive (same
    /// root) replaces the prior copy.
    ///
    /// Browser-only: no server, no filesystem — the trusted root is supplied by
    /// the caller, which is what makes the check meaningful.
    pub fn load_ontology_prx(
        &mut self,
        bytes: &[u8],
        name: String,
        expected_root_hex: &str,
    ) -> Result<(), JsValue> {
        // The wasm boundary is a THIN wrapper over the plain-Rust core (so the
        // load logic — and the demo — is testable natively, with no JsValue):
        // the typed `LoadPrxError` is rendered to a `JsValue` only here.
        self.load_ontology_prx_core(bytes, name, expected_root_hex)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Load the BUILT-IN demo `.prx` — the Avizienis et al. (2004) Dependability
    /// taxonomy embedded at build time — through the exact same fail-closed path
    /// [`Self::load_ontology_prx`] takes. The bytes and the trusted root are the
    /// ones `build.rs` baked in (the root re-derived from the same archive whose
    /// bytes are embedded), so the demo needs no network. Returns the loaded
    /// ontology's name so the UI can name it.
    pub fn load_embedded_demo_prx(&mut self) -> Result<String, JsValue> {
        self.load_embedded_demo_prx_core()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// The embedded demo `.prx` descriptor the UI offers as a one-click load:
    /// `{ name, root, bytes }`. `root` is the trusted Merkle-root pin the
    /// fail-closed load checks against; `bytes` is the size of the embedded
    /// content-addressed archive (no network — it ships in the wasm).
    pub fn embedded_demo_prx(&self) -> String {
        let mut p = Presentation::new();
        p.set(
            "name",
            SchemaValue::Text(embedded_prx::EMBEDDED_DEMO_ONTOLOGY_NAME.into()),
        );
        p.set(
            "root",
            SchemaValue::Text(embedded_prx::EMBEDDED_DEMO_PRX_ROOT_HEX.into()),
        );
        p.set(
            "bytes",
            SchemaValue::Unsigned(embedded_prx::EMBEDDED_DEMO_PRX.len() as u64),
        );
        p.set(
            "loaded",
            SchemaValue::Boolean(
                self.runtime_ontologies
                    .iter()
                    .any(|o| o.id().as_str() == embedded_prx::EMBEDDED_DEMO_ONTOLOGY_NAME),
            ),
        );
        p.to_json()
    }

    /// How many new-format `.prx` ontologies are currently loaded into the chat
    /// (0 until one is loaded). The dashboard reflects this.
    pub fn loaded_ontology_count(&self) -> usize {
        self.runtime_ontologies.len()
    }

    /// The authoritative source documents available to download:
    /// `{ sources: [{ name, version, url, bytes }] }`. The host streams
    /// `url` (showing download progress), then calls [`Self::load_source`]
    /// with the text. The meta page offers a Load action only for catalog
    /// sources that appear here.
    pub fn available_sources(&self) -> String {
        let list: Vec<SchemaValue> = sources_manifest::AVAILABLE_SOURCES
            .iter()
            .map(|(name, version, url, bytes)| {
                let mut r = Presentation::new();
                r.set("name", SchemaValue::Text((*name).into()));
                r.set("version", SchemaValue::Text((*version).into()));
                r.set("url", SchemaValue::Text((*url).into()));
                r.set("bytes", SchemaValue::Unsigned(*bytes));
                SchemaValue::Record(r)
            })
            .collect();
        let mut p = Presentation::new();
        p.set("sources", SchemaValue::List(list));
        p.to_json()
    }

    /// The registered OWL vocabularies available to load, each by either
    /// route: `{ ontologies: [{ name, version, prx_url, source_url }] }`.
    /// The host streams `prx_url` and calls [`Self::load_prx`] (validated
    /// against the embedded lock pin) OR streams `source_url` and calls
    /// [`Self::load_owl_source`]. The embedded lock pin is not exposed here
    /// — it is a build-time validation secret consumed by `load_prx`, not a
    /// URL the host fetches.
    pub fn available_ontologies(&self) -> String {
        let list: Vec<SchemaValue> = ontologies_manifest::AVAILABLE_ONTOLOGIES
            .iter()
            .map(|(name, version, prx_url, source_url, _pin)| {
                let mut r = Presentation::new();
                r.set("name", SchemaValue::Text((*name).into()));
                r.set("version", SchemaValue::Text((*version).into()));
                r.set("prx_url", SchemaValue::Text((*prx_url).into()));
                r.set("source_url", SchemaValue::Text((*source_url).into()));
                SchemaValue::Record(r)
            })
            .collect();
        let mut p = Presentation::new();
        p.set("ontologies", SchemaValue::List(list));
        p.to_json()
    }

    /// The self-model JSON — the eigenform plus the knowledge-boundary
    /// catalog (every registered source tagged Loaded / Available). The
    /// UI renders this directly; it carries no source-specific knowledge.
    pub fn self_describe(&self) -> String {
        let catalog = source_catalog(&self.loaded_refs());
        // The eigenform observes the LIVE loaded set: one Vocabulary per loaded
        // runtime ontology, so `total_concepts`/`total_morphisms` reflect what is
        // actually loaded (the self-model is causally connected, not vacuous).
        let loaded = self
            .runtime_ontologies
            .iter()
            .map(runtime_ontology_vocabulary)
            .collect();
        // Per-ontology capabilities (doc §4.7) — what each loaded ontology can
        // answer (gloss / populated relation kinds), so "loaded" stops lying.
        let capabilities = self
            .runtime_ontologies
            .iter()
            .map(ontology_capabilities)
            .collect();
        pr4xis_chat::self_describe_with_loaded(&self.english, loaded)
            .with_catalog(catalog)
            .with_capabilities(capabilities)
            .with_history(self.history.clone(), self.state_cid())
            .with_footprint(linear_memory_bytes())
            .to_json()
    }

    /// The content-addressed fingerprint of the CURRENT loaded state (doc §2.4) —
    /// a Merkle fold over the SORTED loaded roots, so it is order-independent
    /// (loading A then B identifies the same state as B then A) and changes the
    /// moment a load does. Reuses the kernel [`ContentAddress`] hash — no new
    /// codec. `None` when nothing is loaded.
    fn state_cid(&self) -> Option<String> {
        if self.runtime_ontologies.is_empty() {
            return None;
        }
        let mut roots: Vec<[u8; 32]> = self
            .runtime_ontologies
            .iter()
            .map(|o| *o.root().as_bytes())
            .collect();
        roots.sort_unstable();
        let mut bytes = Vec::with_capacity(roots.len() * 32);
        for r in &roots {
            bytes.extend_from_slice(r);
        }
        Some(ContentAddress::of(&bytes).to_hex())
    }
}

/// The wasm linear-memory footprint in bytes (U2) — `memory_size(0)` pages × 64 KiB.
/// The self-model reports its OWN live size in the host. `None` off-wasm, where
/// there is no single linear-memory measure — the self-model then omits the
/// `linear_memory_bytes` field (so the native test build compiles unchanged).
#[cfg(target_arch = "wasm32")]
fn linear_memory_bytes() -> Option<u64> {
    // `memory_size(0)` is the current page count of linear memory 0 (64 KiB pages).
    Some((core::arch::wasm32::memory_size(0) as u64) << 16)
}

#[cfg(not(target_arch = "wasm32"))]
fn linear_memory_bytes() -> Option<u64> {
    None
}

impl Pr4xis {
    /// The *monitoring* input: which registered sources are live in memory,
    /// with their staging + counts. English is the embedded base; every other
    /// loaded source was downloaded + parsed, then projected (by its
    /// functor-as-data bridge) into one queryable [`RuntimeOntology`] at runtime
    /// (`Async` staging) — a USC title, an OWL vocabulary, or a loaded `.prx`.
    fn loaded_refs(&self) -> Vec<LoadedRef> {
        let mut refs = Vec::with_capacity(self.runtime_ontologies.len() + 1);
        refs.push(LoadedRef::new(
            ENGLISH_SOURCE,
            Staging::Embedded,
            self.english.concept_count(),
            0,
        ));
        // Every runtime-loaded ontology the chat reasons over (USC / OWL / .prx):
        // CONCEPTS and their generating typed morphisms, counted through the ONE
        // blessed Form-aware lowering (`runtime_ontology_vocabulary`) — the same
        // counter the eigenform vocabularies and `loaded_section_count` use, so the
        // §9 `ontolex:Form` surface-exclusion rule lives in exactly one place and
        // the catalog count, the stat, and the eigenform totals cannot drift.
        for onto in &self.runtime_ontologies {
            let vocab = runtime_ontology_vocabulary(onto);
            refs.push(LoadedRef::new(
                onto.id().as_str().to_string(),
                Staging::Async,
                vocab.concept_count(),
                vocab.morphism_count(),
            ));
        }
        refs
    }

    /// Install a materialized [`RuntimeOntology`] into the chat-reasoning set
    /// and rebuild the [`ComposedReasoner`]. Idempotent BY NAME ([`OntologyName`]):
    /// re-loading the same source name displaces the prior version (one current
    /// version per source, doc §4.5), recording a `Replace` event; a DIFFERENT
    /// name with identical content coexists (content equality is not the identity).
    ///
    /// The composed reasoner OWNS its own [`English`] (rebuilt once here from the
    /// baked codegen data — the same constructor `Pr4xis::new` uses), so the
    /// loaded ontologies are grounded into a complete English lexicon via the
    /// Lemon functor. This rebuild happens only on a load (a rare, deliberate
    /// action), keeping the per-chat path a cheap branch.
    fn install_runtime_ontology(&mut self, onto: RuntimeOntology) {
        // Replace BY NAME — a `.prx` is identified by its OntologyName, so a new
        // version displaces the old (one current version per source, doc §4.5),
        // not by content (which would let two versions of Title 15 coexist). The
        // displaced root is captured for the history's Replace event.
        let displaced = self
            .runtime_ontologies
            .iter()
            .find(|o| o.id() == onto.id())
            .map(|o| o.root().to_hex());
        self.runtime_ontologies.retain(|o| o.id() != onto.id());

        // The append-only load history (doc §2.4): content-addressed memory of
        // what was loaded, in order. Re-loading identical content is still
        // recorded (the system observed a load), but the state_cid is unchanged.
        let event = LoadEvent {
            kind: if displaced.is_some() {
                LoadEventKind::Replace
            } else {
                LoadEventKind::Load
            },
            ontology: onto.id().as_str().to_string(),
            root: onto.root().to_hex(),
            displaced,
        };
        self.history.push(event);

        self.runtime_ontologies.push(onto);
        let english = load_english();
        self.composed = Some(ComposedReasoner::new(
            english,
            self.runtime_ontologies.clone(),
        ));
    }

    /// The plain-Rust core of [`Self::load_ontology_prx`] — fail-closed load +
    /// materialize + ground — returning a TYPED [`LoadPrxError`] (no `JsValue`),
    /// so it is exercisable in native tests. The wasm method is a thin wrapper
    /// that renders the error to a `JsValue`. This IS the browser load path; it
    /// just carries the typed verdict instead of a string.
    fn load_ontology_prx_core(
        &mut self,
        bytes: &[u8],
        name: String,
        expected_root_hex: &str,
    ) -> Result<(), LoadPrxError> {
        let trusted_root = ContentAddress::from_hex(expected_root_hex)
            .ok_or_else(|| LoadPrxError::BadRootHex(expected_root_hex.to_string()))?;
        // Fail-closed: decode + re-derive the root + refuse on mismatch.
        let archive =
            pr4xis_runtime::load::load(bytes, trusted_root).map_err(LoadPrxError::Refused)?;
        // Materialize the admitted open form into one live, queryable ontology.
        let onto =
            materialize(archive, OntologyName::new(name)).map_err(LoadPrxError::Materialize)?;
        self.install_runtime_ontology(onto);
        Ok(())
    }

    /// The plain-Rust core of [`Self::load_embedded_demo_prx`] — load the
    /// build-baked Dependability `.prx` through the same fail-closed core and
    /// return its name.
    fn load_embedded_demo_prx_core(&mut self) -> Result<String, LoadPrxError> {
        let name = embedded_prx::EMBEDDED_DEMO_ONTOLOGY_NAME.to_string();
        self.load_ontology_prx_core(
            embedded_prx::EMBEDDED_DEMO_PRX,
            name.clone(),
            embedded_prx::EMBEDDED_DEMO_PRX_ROOT_HEX,
        )?;
        Ok(name)
    }
}

// =========================================================================
// The #186 acceptance, proven at the Pr4xis level
// =========================================================================
//
// The heart of the issue: the browser loads a `.prx` and the chat answers
// about its CONTENT. These tests exercise the real `Pr4xis` struct (the wasm
// entry type) on the host — construct it, load the EMBEDDED new-format `.prx`
// through the SAME fail-closed path the browser uses, ask a question over a
// concept the loaded ontology defines, and assert the answer is the loaded
// gloss. WITHOUT the load, the chat abstains on that concept. The same
// `Pr4xis::chat` code, opposite epistemic outcome — the contrast IS the demo.
//
// Nothing here is hardcoded: the gloss the WITH case asserts is read back from
// the loaded ontology's OWN `lexical`, and the demo concept is discovered as a
// concept the embedded ontology defines but full WordNet does not — so the
// WITHOUT abstention is genuine, not staged.
//
// Gated to NON-wasm32: these are native `#[test]`s that run under `cargo test`
// / `cargo nextest`. The in-browser counterpart (`browser_acceptance`) runs the
// same demo under `wasm-pack test` on wasm32.
#[cfg(all(test, not(target_arch = "wasm32")))]
mod acceptance {
    use super::*;

    /// The loaded-ontology count of a freshly-constructed `Pr4xis`: the always-
    /// loaded LegalSources BASE (installed in `Pr4xis::new`), before any demo /
    /// USC / OWL load. The demo tests assert RELATIVE to this base, so they test
    /// the state CHANGE a load makes — not an absolute empty start (which no longer
    /// holds now that the formal sources of law are an always-present base).
    const BASE_LOADED: usize = 1;
    /// The load-history length of a freshly-constructed `Pr4xis`: one event — the
    /// LegalSources base load recorded at construction.
    const BASE_HISTORY: usize = 1;

    /// Materialize the embedded demo `.prx` straight from its bytes (the same
    /// bytes `Pr4xis` loads) so the test can read its glosses and pick a demo
    /// concept — without reaching into `Pr4xis`'s private state.
    fn embedded_ontology() -> RuntimeOntology {
        let root = ContentAddress::from_hex(embedded_prx::EMBEDDED_DEMO_PRX_ROOT_HEX).unwrap();
        let archive = pr4xis_runtime::load::load(embedded_prx::EMBEDDED_DEMO_PRX, root)
            .expect("embedded demo .prx loads fail-closed against its baked root");
        materialize(
            archive,
            OntologyName::new(embedded_prx::EMBEDDED_DEMO_ONTOLOGY_NAME),
        )
        .expect("embedded demo .prx materializes")
    }

    /// A demo concept the embedded ontology DEFINES (carries a gloss for) whose
    /// lowercased surface FULL WordNet does NOT know — so "what is a <concept>"
    /// genuinely abstains without the corpus, and answers from the loaded gloss
    /// with it. Returns `(surface, gloss)`. Discovered, not hardcoded: we scan
    /// the loaded nodes against a fresh English model.
    fn demo_concept(english: &English) -> (String, String) {
        let onto = embedded_ontology();
        for node in &onto.archive().nodes {
            let surface = node.name.to_lowercase();
            let cref = onto.concept(node.name.clone());
            if let Some(gloss) = onto.lexical(&cref)
                && english.lookup(&surface).is_empty()
            {
                return (surface, gloss.to_string());
            }
        }
        panic!("expected at least one glossed embedded concept unknown to WordNet");
    }

    #[test]
    fn browser_loads_a_prx_and_the_chat_answers_about_its_content_and_abstains_without_it() {
        // A fresh English to choose a discriminating concept + to assert the
        // WITHOUT precondition. `Pr4xis::new()` builds the same model.
        let english = load_english();
        let (surface, gloss) = demo_concept(&english);
        let question = format!("what is a {surface}");

        // Precondition: full WordNet does not know this surface — the abstention
        // below is therefore about the LOADED concept, not a staged unknown.
        assert!(
            english.lookup(&surface).is_empty(),
            "precondition: WordNet must not know {surface:?} (else the contrast is muddied)"
        );

        // --- WITHOUT the corpus: a fresh Pr4xis, nothing loaded. The chat
        //     abstains on the unloaded concept and never surfaces its gloss. ---
        let without = Pr4xis::new();
        assert_eq!(
            without.loaded_ontology_count(),
            BASE_LOADED,
            "a fresh Pr4xis carries only the always-loaded LegalSources base"
        );
        let without_json = without.chat(&question);
        let without_resp = response_of(&without_json);
        assert!(
            !without_resp.contains(gloss.as_str()),
            "english-only must NOT surface the loaded gloss (it isn't loaded); got: {without_resp:?}"
        );
        let lc = without_resp.to_lowercase();
        assert!(
            lc.contains("do not") || lc.contains("don't") || lc.contains("not know"),
            "english-only must abstain on the unloaded concept {surface:?}; got: {without_resp:?}"
        );

        // --- WITH the corpus: load the EMBEDDED `.prx` through the fail-closed
        //     path the browser uses, then ask the SAME question through the SAME
        //     chat. The answer is the loaded gloss. (The native test drives the
        //     typed core; the wasm method `load_embedded_demo_prx` is a thin
        //     wrapper over exactly this call.) ---
        let mut with = Pr4xis::new();
        let loaded_name = with
            .load_embedded_demo_prx_core()
            .expect("the embedded demo .prx loads (fail-closed root matches)");
        assert_eq!(loaded_name, embedded_prx::EMBEDDED_DEMO_ONTOLOGY_NAME);
        assert_eq!(
            with.loaded_ontology_count(),
            BASE_LOADED + 1,
            "the demo load adds one ontology on top of the LegalSources base"
        );

        let with_json = with.chat(&question);
        let with_resp = response_of(&with_json);
        assert!(
            with_resp.contains(gloss.as_str()),
            "with the corpus loaded, the chat must answer from the loaded gloss \
             ({gloss:?}); got: {with_resp:?}"
        );
        assert!(
            with_resp.to_lowercase().contains(&surface),
            "the answer must name the queried concept {surface:?}; got: {with_resp:?}"
        );

        // The contrast is the whole demo: same question, same `Pr4xis::chat`,
        // opposite outcome — grounded entirely through the lexicon.
        assert_ne!(
            without_resp, with_resp,
            "loading the .prx must change the answer"
        );

        // The TYPED outcome (doc §4.1) crosses the wire: abstained without the
        // corpus, answered with it — the UI models what it cannot answer.
        let outcome_of = |json: &str| -> String {
            serde_json::from_str::<serde_json::Value>(json).expect("chat JSON")["outcome"]
                .as_str()
                .unwrap_or_default()
                .to_string()
        };
        assert_eq!(
            outcome_of(&without_json),
            "abstained",
            "english-only must report a typed abstention"
        );
        assert_eq!(
            outcome_of(&with_json),
            "answered",
            "the loaded corpus must report a typed answer"
        );

        // The eigenform OBSERVES the load: `total_concepts` MOVES (doc §2 — the
        // page's "entities" stat is no longer blind to the live loaded set). The
        // self-model is causally connected to what is loaded, not a vacuous fixed
        // point that reports only the compiled substrate.
        let total_concepts = |p: &Pr4xis| -> u64 {
            serde_json::from_str::<serde_json::Value>(&p.self_describe())
                .expect("self_describe is JSON")["total_concepts"]
                .as_u64()
                .expect("total_concepts is a number")
        };
        assert!(
            total_concepts(&with) > total_concepts(&without),
            "loading the .prx must move the self-model's total_concepts (the eigenform sees it): \
             with={} without={}",
            total_concepts(&with),
            total_concepts(&without),
        );

        // The self-model reports the loaded ontology's CAPABILITIES (doc §4.7) —
        // what it can answer, not just that it is loaded. Empty before, present after.
        let capability_count = |p: &Pr4xis| -> usize {
            serde_json::from_str::<serde_json::Value>(&p.self_describe())
                .expect("self_describe is JSON")["capabilities"]
                .as_array()
                .map(|a| a.len())
                .unwrap_or(0)
        };
        assert!(
            capability_count(&with) > capability_count(&without),
            "loading the .prx must add its capabilities to the self-model: with={} without={}",
            capability_count(&with),
            capability_count(&without),
        );

        // The load HISTORY + content-addressed state fingerprint (doc §2.4): the
        // system REMEMBERS the load (an append-only event) and identifies its
        // current knowledge state by a content-address that only a load changes.
        let describe = |p: &Pr4xis| {
            serde_json::from_str::<serde_json::Value>(&p.self_describe())
                .expect("self_describe JSON")
        };
        let with_d = describe(&with);
        assert_eq!(
            with_d["history"].as_array().map(|a| a.len()).unwrap_or(0),
            BASE_HISTORY + 1,
            "the demo load is recorded on top of the base's load event"
        );
        assert_eq!(
            with_d["history"][BASE_HISTORY]["event"].as_str(),
            Some("load"),
            "a first load of the demo name is a `load` event"
        );
        assert!(
            with_d["state_cid"].as_str().is_some(),
            "a loaded state has a content-addressed fingerprint"
        );
        assert_eq!(
            describe(&without)["history"]
                .as_array()
                .map(|a| a.len())
                .unwrap_or(0),
            BASE_HISTORY,
            "only the base load is recorded before the demo load"
        );

        // §3: the embedded demo `.prx` is loaded but NOT a registered source — it
        // must still appear in the catalog's sources panel (by its OntologyName,
        // tagged loaded), not be silently dropped.
        let sources = with_d["sources"].as_array().expect("sources array");
        let demo = sources
            .iter()
            .find(|s| s["name"].as_str() == Some(embedded_prx::EMBEDDED_DEMO_ONTOLOGY_NAME))
            .expect("the unregistered loaded demo .prx appears in the catalog (doc §3)");
        assert_eq!(
            demo["availability"].as_str(),
            Some("loaded"),
            "the loaded demo is tagged loaded"
        );
        assert!(
            describe(&without)["sources"]
                .as_array()
                .map(|ss| !ss
                    .iter()
                    .any(|s| s["name"].as_str() == Some(embedded_prx::EMBEDDED_DEMO_ONTOLOGY_NAME)))
                .unwrap_or(true),
            "without the load, the unregistered demo is absent from the catalog"
        );
    }

    #[test]
    fn re_loading_a_name_replaces_it_and_records_a_replace_event() {
        // Name-based replace (doc §4.5): a `.prx` is its OntologyName, so loading
        // the same name again DISPLACES the prior one (one current version, not two
        // copies) — and the history records the temporal fact (doc §2.4): a Load,
        // then a Replace carrying the displaced root.
        let mut p = Pr4xis::new();
        p.load_embedded_demo_prx_core().expect("first load");
        p.load_embedded_demo_prx_core()
            .expect("second load (same name)");

        assert_eq!(
            p.loaded_ontology_count(),
            BASE_LOADED + 1,
            "re-loading a name replaces it — not two demo copies atop the base"
        );

        let d = serde_json::from_str::<serde_json::Value>(&p.self_describe()).expect("JSON");
        let history = d["history"].as_array().expect("history is an array");
        assert_eq!(
            history.len(),
            BASE_HISTORY + 2,
            "both demo loads are recorded (append-only) after the base load"
        );
        assert_eq!(history[BASE_HISTORY]["event"].as_str(), Some("load"));
        assert_eq!(
            history[BASE_HISTORY + 1]["event"].as_str(),
            Some("replace"),
            "the second load of the same name is a replace"
        );
        assert!(
            history[BASE_HISTORY + 1]["displaced"].as_str().is_some(),
            "a replace event carries the displaced root"
        );
    }

    /// A minimal USLM Title (Title 18 §1, heading "First section") — the fixture
    /// for the load-a-statute-then-query-it acceptance tests.
    const SAMPLE_USLM_TITLE: &str = r##"<title xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18"><num value="18">Title 18—</num><heading>CRIMES AND CRIMINAL PROCEDURE</heading><section identifier="/us/usc/t18/s1"><num value="1">§ 1.</num><heading>First section</heading><content>Body text.</content></section></title>"##;

    #[test]
    fn loading_a_usc_title_routes_it_into_the_reasoner() {
        // Architecture Step 1, the WIRE: a loaded USLM statute must reach the SAME
        // reasoning set `chat()` composes over (`runtime_ontologies`), not a
        // separate `self.loaded` corpus the reasoner never sees. Before the load
        // the reasoner is empty; after, it holds the one projected statute
        // ontology. This is the structural half of "load Title 15 → ask about it".
        let mut p = Pr4xis::new();
        assert_eq!(p.loaded_ontology_count(), BASE_LOADED);
        p.load_source("Title 18 (test)".to_string(), SAMPLE_USLM_TITLE)
            .expect("a well-formed USLM title loads");
        assert_eq!(
            p.loaded_ontology_count(),
            BASE_LOADED + 1,
            "a loaded statute must become a RuntimeOntology the chat reasons over"
        );
    }

    /// THE acceptance test for the maintainer's exact symptom: load a statute,
    /// ask about a section it defines, and get its content.
    ///
    /// NOW RESOLVED (§9 + the multi-token recognizer): a URN does not tokenize as
    /// natural language, but the USC bridge mints the section's `"section <num>"`
    /// CITATION as an `otherForm` surface (and its heading as a `canonicalForm`),
    /// and the chat's phrase-lookup collapses the multi-token citation into one
    /// lookup unit. So "what is section 1" resolves the URN-named section and
    /// answers from its heading — the maintainer's exact symptom, fixed.
    #[test]
    fn loading_a_usc_title_makes_it_queryable() {
        let mut p = Pr4xis::new();
        p.load_source("Title 18 (test)".to_string(), SAMPLE_USLM_TITLE)
            .expect("a well-formed USLM title loads");
        let resp = response_of(&p.chat("what is section 1")).to_lowercase();
        assert!(
            resp.contains("first section"),
            "after loading the statute, the chat must answer about its section by its \
             citation surface; got: {resp:?}"
        );
    }

    #[test]
    fn load_is_fail_closed_a_wrong_root_is_refused() {
        // The gate re-derives the archive's Merkle root and refuses on mismatch.
        // Hand the real embedded bytes but the WRONG trusted root → refused,
        // nothing installed.
        let mut p = Pr4xis::new();
        let wrong_root = ContentAddress::of(b"not the dependability root").to_hex();
        let err = p
            .load_ontology_prx_core(
                embedded_prx::EMBEDDED_DEMO_PRX,
                "Dependability".into(),
                &wrong_root,
            )
            .expect_err("a wrong trusted root must be refused (fail-closed)");
        // The typed verdict IS a root mismatch (not a decode error, not a
        // materialize error) — the gate re-derived the root and rejected it.
        assert!(
            matches!(
                err,
                LoadPrxError::Refused(pr4xis_runtime::load::LoadError::RootMismatch { .. })
            ),
            "the refusal must be a typed root mismatch; got: {err:?}"
        );
        assert_eq!(
            p.loaded_ontology_count(),
            BASE_LOADED,
            "a refused .prx must install nothing beyond the always-loaded base"
        );
        // The reasoner still reflects ONLY the base — the refused demo did not join it.
        assert!(
            p.composed.is_some(),
            "the always-loaded base reasoner is present; the refused demo just isn't in it"
        );
        assert!(
            !p.runtime_ontologies
                .iter()
                .any(|o| o.id().as_str() == "Dependability"),
            "the refused demo must not be among the loaded ontologies"
        );
    }

    #[test]
    fn load_is_fail_closed_tampered_bytes_are_refused() {
        // Flip a byte of the embedded `.prx`: either decode fails or the
        // re-derived root no longer matches the (correct) trusted root. Either
        // way — refused.
        let mut bytes = embedded_prx::EMBEDDED_DEMO_PRX.to_vec();
        *bytes.last_mut().unwrap() ^= 0xff;
        let mut p = Pr4xis::new();
        let err = p
            .load_ontology_prx_core(
                &bytes,
                "Dependability".into(),
                embedded_prx::EMBEDDED_DEMO_PRX_ROOT_HEX,
            )
            .expect_err("tampered .prx bytes must be refused (fail-closed)");
        assert!(
            matches!(err, LoadPrxError::Refused(_)),
            "tampered bytes must be refused by the gate; got: {err:?}"
        );
        assert_eq!(
            p.loaded_ontology_count(),
            BASE_LOADED,
            "a refused .prx installs nothing beyond the always-loaded base"
        );
    }

    #[test]
    fn a_fresh_pr4xis_answers_that_a_statute_is_a_law_from_the_always_loaded_base() {
        // THE headline of the always-loaded base: NO explicit load. `Pr4xis::new`
        // installs the LegalSources base at construction, so the chat routes through
        // the ComposedReasoner with the formal sources of law present. "is a statute
        // a law" resolves both surfaces to loaded concepts (the label "law" grounds
        // because the base was emitted with `emit_with_forms`) and reads the
        // Subsumption closure Statute ⊑ LegalDocument ⊑ LegalSource → Yes.
        let p = Pr4xis::new();
        let json = p.chat("is a statute a law");
        let resp = response_of(&json);
        assert!(
            resp.to_lowercase().contains("yes"),
            "the always-loaded base must answer that a statute is a law with no explicit \
             load; got: {resp:?}"
        );

        // The typed outcome crosses the wire as answered.
        let outcome =
            serde_json::from_str::<serde_json::Value>(&json).expect("chat JSON")["outcome"]
                .as_str()
                .unwrap_or_default()
                .to_string();
        assert_eq!(
            outcome, "answered",
            "the base answer is a typed answer, not an abstention"
        );

        // The answer credits the LegalSources base in its reasoned-over provenance.
        let ontologies =
            serde_json::from_str::<serde_json::Value>(&json).expect("chat JSON")["ontologies"]
                .as_array()
                .cloned()
                .unwrap_or_default();
        assert!(
            ontologies.iter().any(|o| o["ontology"].as_str()
                == Some(embedded_prx::EMBEDDED_LEGAL_SOURCES_ONTOLOGY_NAME)),
            "the Yes must credit the LegalSources base it reasoned over; got: {ontologies:?}"
        );

        // self_describe lists the LegalSources base (tagged loaded) from construction,
        // with a non-zero contribution to the loaded concept set.
        let d = serde_json::from_str::<serde_json::Value>(&p.self_describe())
            .expect("self_describe JSON");
        let sources = d["sources"].as_array().expect("sources array");
        let legal = sources
            .iter()
            .find(|s| {
                s["name"].as_str() == Some(embedded_prx::EMBEDDED_LEGAL_SOURCES_ONTOLOGY_NAME)
            })
            .expect("the always-loaded LegalSources base appears in the self-model catalog");
        assert_eq!(
            legal["availability"].as_str(),
            Some("loaded"),
            "the always-loaded base is tagged loaded"
        );
        assert!(
            p.loaded_section_count() > 0,
            "the LegalSources base contributes concepts to the loaded set from construction"
        );
    }

    /// Pull the `response` field out of the chat JSON envelope.
    fn response_of(json: &str) -> String {
        let v: serde_json::Value = serde_json::from_str(json).expect("chat returns JSON");
        v.get("response")
            .and_then(|r| r.as_str())
            .expect("chat JSON has a string `response`")
            .to_string()
    }
}

// =========================================================================
// The same acceptance, in a REAL browser (wasm32 + headless Firefox)
// =========================================================================
//
// `wasm-pack test --headless --firefox` (the `dev-test-wasm` script) runs this
// against the actual wasm artifact in a browser: construct `Pr4xis`, load the
// EMBEDDED new-format `.prx` through the WASM `load_embedded_demo_prx` method
// (the exact entry the worker calls), and assert the chat answers from the
// loaded gloss — and abstains before the load. This exercises the real
// wasm-bindgen boundary (JsValue marshalling included), not just the native
// core.
#[cfg(all(test, target_arch = "wasm32"))]
mod browser_acceptance {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn response_of(json: &str) -> String {
        let v: serde_json::Value = serde_json::from_str(json).unwrap();
        v["response"].as_str().unwrap().to_string()
    }

    /// Materialize the embedded demo `.prx` straight from its bytes (the same
    /// bytes `Pr4xis` loads) so the test can read its glosses and DISCOVER a demo
    /// concept at test time — the wasm32 mirror of the native acceptance
    /// harness. Nothing about the gloss is hardcoded.
    fn embedded_ontology() -> RuntimeOntology {
        let root = ContentAddress::from_hex(embedded_prx::EMBEDDED_DEMO_PRX_ROOT_HEX).unwrap();
        let archive = pr4xis_runtime::load::load(embedded_prx::EMBEDDED_DEMO_PRX, root)
            .expect("embedded demo .prx loads fail-closed against its baked root");
        materialize(
            archive,
            OntologyName::new(embedded_prx::EMBEDDED_DEMO_ONTOLOGY_NAME),
        )
        .expect("embedded demo .prx materializes")
    }

    /// A demo concept the embedded ontology DEFINES (carries a gloss for) whose
    /// lowercased surface full WordNet does NOT know — so "what is a <concept>"
    /// genuinely abstains without the corpus and answers from the loaded gloss
    /// with it. Returns `(surface, gloss)`, both READ FROM the loaded ontology at
    /// test time via [`RuntimeOntology::lexical`] — never a constant. This is the
    /// same discovery the native `acceptance::demo_concept` does, so a faked
    /// embedded ontology carrying a different gloss cannot pass: the asserted
    /// gloss is whatever the loaded `.prx` actually declares.
    fn demo_concept(english: &English) -> (String, String) {
        let onto = embedded_ontology();
        for node in &onto.archive().nodes {
            let surface = node.name.to_lowercase();
            let cref = onto.concept(node.name.clone());
            if let Some(gloss) = onto.lexical(&cref)
                && english.lookup(&surface).is_empty()
            {
                return (surface, gloss.to_string());
            }
        }
        panic!("expected at least one glossed embedded concept unknown to WordNet");
    }

    #[wasm_bindgen_test]
    fn browser_loads_the_embedded_prx_and_the_chat_answers_from_its_gloss() {
        // A fresh English to choose a discriminating concept + assert the WITHOUT
        // precondition — the same model `Pr4xis::new()` builds in the browser.
        let english = load_english();
        let (surface, gloss) = demo_concept(&english);
        let question = format!("what is a {surface}");

        // Precondition: full WordNet does not know this surface — so the
        // abstention below is about the LOADED concept, not a staged unknown.
        assert!(
            english.lookup(&surface).is_empty(),
            "precondition: WordNet must not know {surface:?} in the browser too"
        );

        // WITHOUT the corpus: a fresh Pr4xis abstains and never surfaces the
        // dynamically-read gloss.
        let without = Pr4xis::new();
        // A fresh Pr4xis carries the always-loaded LegalSources base (1), but NOT
        // the Dependability demo — so the demo concept still abstains here.
        assert_eq!(without.loaded_ontology_count(), 1);
        let without_resp = response_of(&without.chat(&question));
        assert!(
            !without_resp.contains(gloss.as_str()),
            "english-only must not surface the loaded gloss; got: {without_resp:?}"
        );
        let lc = without_resp.to_lowercase();
        assert!(
            lc.contains("do not") || lc.contains("don't") || lc.contains("not know"),
            "english-only must abstain on the unloaded concept {surface:?}; got: {without_resp:?}"
        );

        // WITH the corpus: load the embedded `.prx` through the WASM method (the
        // fail-closed gate runs in the browser), then chat the same question. The
        // asserted gloss is the one DISCOVERED from the loaded ontology, so the
        // evidence chain is real — a hardcoded gloss could not satisfy it.
        let mut with = Pr4xis::new();
        with.load_embedded_demo_prx()
            .expect("the embedded demo .prx loads in the browser (fail-closed)");
        // The LegalSources base (1) plus the just-loaded Dependability demo (1).
        assert_eq!(with.loaded_ontology_count(), 2);
        let with_resp = response_of(&with.chat(&question));
        assert!(
            with_resp.contains(gloss.as_str()),
            "with the .prx loaded, the chat must answer from the loaded gloss \
             ({gloss:?}); got: {with_resp:?}"
        );
        assert!(
            with_resp.to_lowercase().contains(&surface),
            "the answer must name the queried concept {surface:?}; got: {with_resp:?}"
        );
        assert_ne!(without_resp, with_resp);
    }
}
