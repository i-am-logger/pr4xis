use wasm_bindgen::prelude::*;

use pr4xis::ontology::compose::Staging;
use pr4xis_domains::cognitive::linguistics::english::English;
use pr4xis_domains::cognitive::linguistics::language;
use pr4xis_domains::formal::information::knowledge::{LoadedRef, source_catalog};
use pr4xis_domains::formal::information::schema::transport::{Presentation, SchemaValue};
use pr4xis_domains::social::software::markup::xml::owl::prx::load_prx_gz;
use pr4xis_domains::social::software::markup::xml::owl::reader::read_owl;
use pr4xis_domains::social::software::markup::xml::owl::vocabulary::LoadedOwlVocabulary;
use pr4xis_domains::social::software::markup::xml::uslm::corpus::UsCode;
use pr4xis_domains::social::software::markup::xml::uslm::lens::read_uslm_title;

#[allow(dead_code)]
mod codegen_output {
    include!(concat!(env!("OUT_DIR"), "/english_codegen.rs"));
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
            english: language::from_codegen(&codegen_output::CODEGEN_DATA),
            loaded: Vec::new(),
        }
    }

    pub fn chat(&self, input: &str) -> String {
        let result = pr4xis_chat::process_with_metadata(&self.english, input);
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
    /// `prx_url`). **Hash-validated, fail-closed**: the gate gunzips,
    /// bytecheck-validates the rkyv envelope, and asserts the envelope's
    /// embedded `source_sha256` equals the praxis.lock pin baked into the
    /// build-time manifest for `(name, version)`. On any mismatch nothing
    /// is installed and an `Err` is returned. Idempotent: loading a name
    /// already present replaces it.
    ///
    /// This differs from [`Self::load_owl_source`] in *where* trust is
    /// anchored: `load_prx` re-checks the embedded source hash against the
    /// lock pin here in the runtime, so a tampered or stale `.prx.gz` is
    /// rejected even if the transport was honest.
    pub fn load_prx(
        &mut self,
        name: String,
        version: String,
        prx_gz: &[u8],
    ) -> Result<(), JsValue> {
        let pin = self.lock_pin(&name, &version).ok_or_else(|| {
            JsValue::from_str(&format!(
                "no embedded praxis.lock pin for {name}@{version}; cannot validate .prx.gz"
            ))
        })?;
        let vocab = load_prx_gz(prx_gz, pin)
            .map_err(|e| JsValue::from_str(&format!(".prx.gz load/validate failed: {e}")))?;
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
        refs
    }

    /// Install a freshly materialised payload under `name`, replacing any
    /// existing load of the same name (idempotent — the latest load wins).
    fn install(&mut self, name: String, payload: LoadedPayload) {
        self.loaded.retain(|s| s.name != name);
        self.loaded.push(LoadedSource { name, payload });
    }

    /// The praxis.lock source-hash pin for `name@version`, from the
    /// build-time ontology manifest. `None` for an unregistered or unpinned
    /// vocabulary — in which case [`Self::load_prx`] refuses to install
    /// (it has nothing to validate the envelope's embedded hash against).
    fn lock_pin(&self, name: &str, version: &str) -> Option<&'static str> {
        ontologies_manifest::AVAILABLE_ONTOLOGIES
            .iter()
            .find(|(n, v, _, _, _)| *n == name && *v == version)
            .map(|(_, _, _, _, pin)| *pin)
    }
}
