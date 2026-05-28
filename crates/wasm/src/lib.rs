use wasm_bindgen::prelude::*;

use pr4xis::ontology::compose::Staging;
use pr4xis_domains::cognitive::linguistics::english::English;
use pr4xis_domains::cognitive::linguistics::language;
use pr4xis_domains::formal::information::knowledge::{LoadedRef, source_catalog};
use pr4xis_domains::formal::information::schema::transport::{Presentation, SchemaValue};
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
/// runtime download + parse instead of build-time codegen. Not a blob:
/// `usc` is a materialized [`UsCode`] the system can query.
struct LoadedSource {
    name: String,
    usc: UsCode,
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

    /// Total sections across the on-demand-loaded sources (0 until one is
    /// loaded). Source-agnostic.
    pub fn loaded_section_count(&self) -> usize {
        self.loaded.iter().map(|s| s.usc.section_count()).sum()
    }

    /// Load a registered source from its authoritative USLM XML
    /// (downloaded by the host from the source's served document). Parses
    /// in-browser into a LIVE [`UsCode`] — the same materialization path
    /// English takes, only at runtime — and holds it in memory, queryable.
    /// This IS the Nelson-Narens *control* operation: Available → Loaded.
    /// A malformed document fails closed via the USLM reader. Idempotent:
    /// loading a name already present replaces it.
    pub fn load_source(&mut self, name: String, xml: &str) -> Result<(), JsValue> {
        let title = read_uslm_title(xml)
            .map_err(|e| JsValue::from_str(&format!("USLM parse failed: {e:?}")))?;
        let usc = UsCode::from_uslm_titles_owned(vec![title]);
        self.loaded.retain(|s| s.name != name);
        self.loaded.push(LoadedSource { name, usc });
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
    /// loaded statute was downloaded + parsed into a live `UsCode`.
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
                s.usc.section_count(),
                0,
            ));
        }
        refs
    }
}
