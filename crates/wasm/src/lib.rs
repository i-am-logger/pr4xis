use wasm_bindgen::prelude::*;

use pr4xis::ontology::Staging;
use pr4xis::ontology::meta::OntologyName;
use pr4xis_domains::applied::data_provisioning::registry::{
    lock_archive_signature, lock_canonical_signature, lock_hashes,
};
use pr4xis_domains::cognitive::linguistics::composed::ComposedReasoner;
use pr4xis_domains::cognitive::linguistics::english::English;
use pr4xis_domains::formal::information::knowledge::{LoadedRef, source_catalog};
use pr4xis_domains::formal::information::schema::transport::{Presentation, SchemaValue};
use pr4xis_domains::social::software::markup::xml::lmf::compact_succinct::load_prx_gz as load_english_prx;
use pr4xis_domains::social::software::markup::xml::owl::prx::load_prx_gz;
use pr4xis_domains::social::software::markup::xml::owl::reader::read_owl;
use pr4xis_domains::social::software::markup::xml::owl::vocabulary::LoadedOwlVocabulary;
use pr4xis_domains::social::software::markup::xml::uslm::corpus::UsCode;
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
/// "statute" — a loaded U.S. Code title is just one [`LoadedSource`]
/// among whatever the registry offers.
#[wasm_bindgen]
pub struct Pr4xis {
    english: English,
    loaded: Vec<LoadedSource>,
    /// New-format `.prx` ontologies loaded fail-closed at runtime and
    /// materialized into live, queryable [`RuntimeOntology`]s (content-address
    /// identity). Distinct from [`LoadedSource`]: those are corpus blobs
    /// (USC / OWL) surfaced only through the self-model catalog; these are
    /// ontologies the CHAT reasons over, grounded into English by `composed`.
    runtime_ontologies: Vec<RuntimeOntology>,
    /// The embedded English model COMPOSED with the loaded `.prx` ontologies as
    /// one [`ComposedReasoner`] — `None` until at least one `.prx` is loaded.
    /// Rebuilt whenever `runtime_ontologies` changes (a rare, deliberate load
    /// action), so the per-chat path is a cheap branch, not a re-grounding.
    /// When present, `chat` reasons through it (the loaded gloss answers);
    /// when absent, `chat` reasons through `english` alone (it abstains on an
    /// unloaded concept, exactly as today).
    composed: Option<ComposedReasoner>,
}

/// A registered source downloaded and parsed into a LIVE in-memory
/// ontology at runtime — exactly the end state English reaches, only via
/// runtime download + parse instead of build-time codegen. Not a blob: the
/// payload is a materialized ontology the system can query.
struct LoadedSource {
    name: String,
    payload: LoadedPayload,
}

/// What a runtime-loaded source materialised into. A USLM statute title
/// becomes a [`UsCode`]; an OWL vocabulary (loaded from its `.prx.gz` or
/// its `.owl` source) becomes a [`LoadedOwlVocabulary`]. The runtime stays
/// source-agnostic — it holds whichever live ontology the registry offered.
enum LoadedPayload {
    /// A USLM statute title materialised into the U.S. Code corpus.
    UsCode(UsCode),
    /// An OWL vocabulary materialised into its loaded class/property
    /// subsumption corpus.
    Owl(LoadedOwlVocabulary),
}

impl LoadedPayload {
    /// The number of queryable units — USC sections, or OWL entities
    /// (classes + object properties). Drives the catalog concept count.
    fn concept_count(&self) -> usize {
        match self {
            LoadedPayload::UsCode(usc) => usc.section_count(),
            LoadedPayload::Owl(v) => v.entity_count(),
        }
    }

    /// The morphism count: 0 for USC (its section corpus is not exposed as
    /// a morphism graph here), the subsumption-edge count for OWL.
    fn morphism_count(&self) -> usize {
        match self {
            LoadedPayload::UsCode(_) => 0,
            LoadedPayload::Owl(v) => v.subsumption_edge_count(),
        }
    }
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
        Self {
            english: load_english(),
            loaded: Vec::new(),
            runtime_ontologies: Vec::new(),
            composed: None,
        }
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
        let ontologies = result.trace.all_participating_ontologies();
        let trace = result.trace.serialize_with_functors();

        let mut p = Presentation::new();
        p.set("response", result.response.into());
        p.set("duration_us", result.duration_us.into());
        p.set("parsed", result.parsed.into());
        p.set("from_ontology", result.from_ontology.into());
        p.set(
            "ontologies",
            SchemaValue::List(ontologies.into_iter().map(|o| o.into()).collect()),
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

    /// Total queryable units across the on-demand-loaded sources (0 until
    /// one is loaded): USC sections plus OWL entities. Source-agnostic.
    pub fn loaded_section_count(&self) -> usize {
        self.loaded.iter().map(|s| s.payload.concept_count()).sum()
    }

    /// Load a registered USLM source from its authoritative XML (downloaded
    /// by the host from the source's served document). Parses in-browser
    /// into a LIVE [`UsCode`] — the same materialization path English
    /// takes, only at runtime — and holds it in memory, queryable. This IS
    /// the Nelson-Narens *control* operation: Available → Loaded. A
    /// malformed document fails closed via the USLM reader. Idempotent:
    /// loading a name already present replaces it.
    pub fn load_source(&mut self, name: String, xml: &str) -> Result<(), JsValue> {
        let title = read_uslm_title(xml)
            .map_err(|e| JsValue::from_str(&format!("USLM parse failed: {e:?}")))?;
        let usc = UsCode::from_uslm_titles_owned(vec![title]);
        self.install(name, LoadedPayload::UsCode(usc));
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
        self.install(name, LoadedPayload::Owl(vocab));
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
        self.install(name, LoadedPayload::Owl(vocab));
        Ok(())
    }

    /// Load a NEW-FORMAT `.prx` ontology — the content-addressed [`Archive`]
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
        pr4xis_chat::self_describe(&self.english)
            .with_catalog(catalog)
            .to_json()
    }
}

impl Pr4xis {
    /// The *monitoring* input: which registered sources are live in memory,
    /// with their staging + counts. English is the embedded base; every
    /// loaded source was downloaded + parsed into a live ontology at
    /// runtime (`Async` staging) — a `UsCode` (sections) or a
    /// `LoadedOwlVocabulary` (entities + subsumption edges).
    fn loaded_refs(&self) -> Vec<LoadedRef> {
        let mut refs = Vec::with_capacity(self.loaded.len() + 1);
        refs.push(LoadedRef::new(
            ENGLISH_SOURCE,
            Staging::Embedded,
            self.english.concept_count(),
            0,
        ));
        for s in &self.loaded {
            refs.push(LoadedRef::new(
                s.name.clone(),
                Staging::Async,
                s.payload.concept_count(),
                s.payload.morphism_count(),
            ));
        }
        // The new-format `.prx` runtime ontologies the chat reasons over: one
        // node per concept, and the generating typed edges as morphisms.
        for onto in &self.runtime_ontologies {
            let concepts = onto.archive().nodes.len();
            let morphisms: usize = onto.archive().nodes.iter().map(|n| n.edges.len()).sum();
            refs.push(LoadedRef::new(
                onto.id().as_str().to_string(),
                Staging::Async,
                concepts,
                morphisms,
            ));
        }
        refs
    }

    /// Install a freshly materialised payload under `name`, replacing any
    /// existing load of the same name (idempotent — the latest load wins).
    fn install(&mut self, name: String, payload: LoadedPayload) {
        self.loaded.retain(|s| s.name != name);
        self.loaded.push(LoadedSource { name, payload });
    }

    /// Install a materialized [`RuntimeOntology`] into the chat-reasoning set
    /// and rebuild the [`ComposedReasoner`]. Idempotent by content address:
    /// re-loading the same archive (equal Merkle root, hence equal
    /// `RuntimeOntology`) replaces the prior copy rather than duplicating it.
    ///
    /// The composed reasoner OWNS its own [`English`] (rebuilt once here from the
    /// baked codegen data — the same constructor `Pr4xis::new` uses), so the
    /// loaded ontologies are grounded into a complete English lexicon via the
    /// Lemon functor. This rebuild happens only on a load (a rare, deliberate
    /// action), keeping the per-chat path a cheap branch.
    fn install_runtime_ontology(&mut self, onto: RuntimeOntology) {
        self.runtime_ontologies.retain(|o| o != &onto);
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
            if let Some(gloss) = onto.lexical(&cref) {
                if english.lookup(&surface).is_empty() {
                    return (surface, gloss.to_string());
                }
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
        assert_eq!(without.loaded_ontology_count(), 0);
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
        assert_eq!(with.loaded_ontology_count(), 1);

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
            0,
            "a refused .prx must install nothing"
        );
        // And the chat still abstains (no corpus was admitted).
        assert!(
            p.composed.is_none(),
            "a refused load must not build a reasoner"
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
        assert_eq!(p.loaded_ontology_count(), 0);
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
            if let Some(gloss) = onto.lexical(&cref) {
                if english.lookup(&surface).is_empty() {
                    return (surface, gloss.to_string());
                }
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
        assert_eq!(without.loaded_ontology_count(), 0);
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
        assert_eq!(with.loaded_ontology_count(), 1);
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
