use wasm_bindgen::prelude::*;

use pr4xis::archive::{OwnedCodegenData, from_archive_bytes};
use pr4xis::ontology::compose::Staging;
use pr4xis_domains::cognitive::linguistics::english::English;
use pr4xis_domains::cognitive::linguistics::language;
use pr4xis_domains::formal::information::knowledge::{LoadedRef, source_catalog};
use pr4xis_domains::formal::information::schema::transport::{Presentation, SchemaValue};

#[allow(dead_code)]
mod codegen_output {
    include!(concat!(env!("OUT_DIR"), "/english_codegen.rs"));
}

/// Build-time catalog of on-demand-loadable rkyv archives — emitted by
/// build.rs from the USC titles on disk. `(registry name, version,
/// archive filename served at /archives/<file>)`.
mod archives_manifest {
    include!(concat!(env!("OUT_DIR"), "/archives_manifest.rs"));
}

/// Registry primary key of the embedded English base
/// (praxis.toml `[sources.english_wordnet]`). English is the one source
/// baked into the binary (`Embedded` staging); every other registered
/// source is loaded on demand from its rkyv archive (`Async` staging).
const ENGLISH_SOURCE: &str = "english_wordnet";

/// The runtime. Source-agnostic: it holds the embedded English language
/// model plus a generic set of on-demand-loaded ontologies. It has no
/// notion of "statute" — a loaded U.S. Code title is just one
/// [`LoadedSource`] among whatever the registry offers.
#[wasm_bindgen]
pub struct Pr4xis {
    english: English,
    loaded: Vec<LoadedSource>,
}

/// A registered source materialized at runtime from its rkyv archive —
/// the object-level the self-model's meta-level monitors as `Loaded`.
struct LoadedSource {
    name: String,
    staging: Staging,
    owned: OwnedCodegenData,
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

    /// Total entities across the on-demand-loaded sources (0 until the
    /// host loads an archive). Source-agnostic — it does not single out
    /// any source kind.
    pub fn loaded_entity_count(&self) -> usize {
        self.loaded
            .iter()
            .map(|s| s.owned.entity_count as usize)
            .sum()
    }

    /// Load a registered source from its rkyv archive bytes (fetched by
    /// the host from the source's `.rkyv` blob). This IS the Nelson-Narens
    /// *control* operation — the meta-level moving a source from
    /// `Available` to `Loaded`. Validation is by `bytecheck` inside
    /// [`from_archive_bytes`]; a corrupted blob fails closed. Idempotent:
    /// loading a name already present replaces it.
    pub fn load_source(&mut self, name: String, bytes: &[u8]) -> Result<(), JsValue> {
        let owned = from_archive_bytes(bytes)
            .map_err(|e| JsValue::from_str(&format!("archive decode failed: {e}")))?;
        self.loaded.retain(|s| s.name != name);
        self.loaded.push(LoadedSource {
            name,
            staging: Staging::Async,
            owned,
        });
        Ok(())
    }

    /// The on-demand-loadable archives: `{ archives: [{ name, version,
    /// url }] }`. The host fetches `url`, then calls [`Self::load_source`]
    /// with the bytes. The meta page offers a Load action only for catalog
    /// sources that appear here (others are registered but have no archive
    /// to fetch yet).
    pub fn available_archives(&self) -> String {
        let list: Vec<SchemaValue> = archives_manifest::AVAILABLE_ARCHIVES
            .iter()
            .map(|(name, version, file)| {
                let mut r = Presentation::new();
                r.set("name", SchemaValue::Text((*name).into()));
                r.set("version", SchemaValue::Text((*version).into()));
                r.set("url", SchemaValue::Text(format!("./archives/{file}")));
                SchemaValue::Record(r)
            })
            .collect();
        let mut p = Presentation::new();
        p.set("archives", SchemaValue::List(list));
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
    /// The *monitoring* input: which registered sources are loaded, with
    /// their staging + counts. English is the embedded base; everything
    /// else arrived through an archive.
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
                s.staging,
                s.owned.entity_count as usize,
                edge_count(&s.owned),
            ));
        }
        refs
    }
}

fn edge_count(o: &OwnedCodegenData) -> usize {
    o.taxonomy.len()
        + o.mereology.len()
        + o.opposition.len()
        + o.equivalence.len()
        + o.causation.len()
        + o.references.len()
}
