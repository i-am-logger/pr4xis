use std::rc::Rc;

use wasm_bindgen::prelude::*;

pub mod load_envelope;

use pr4xis::ontology::Staging;
use pr4xis::ontology::meta::OntologyName;
use pr4xis_chat::ChatOutcome;
use pr4xis_domains::applied::data_provisioning::registry::{
    lock_archive_signature, lock_canonical_signature, lock_hashes,
};
use pr4xis_domains::applied::data_provisioning::usc_title_lexicon::{
    USC_TITLE_LEXICON_NAME, titles_held_in, usc_title_lexicon,
};
use pr4xis_domains::cognitive::linguistics::composed::ComposedReasoner;
use pr4xis_domains::cognitive::linguistics::english::English;
use pr4xis_domains::formal::information::diagnostics::trace_functors::PipelineTrace;
use pr4xis_domains::formal::information::knowledge::{
    LoadEvent, LoadEventKind, LoadedRef, Residency, ontology_capabilities,
    runtime_ontology_vocabulary, source_catalog,
};
use pr4xis_domains::formal::information::schema::transport::{Presentation, SchemaValue};
use pr4xis_domains::formal::math::probability::natural_breaks::two_class_break;
use pr4xis_domains::formal::meta::grounding::ground_loaded_set;
use pr4xis_domains::formal::systems::mape_k::ontology::{MapeKConcept, MapeKOntology};
use pr4xis_domains::social::software::markup::xml::lmf::prx::load_english_store_bundle_gz_gated;
use pr4xis_domains::social::software::markup::xml::owl::bridge::owl_runtime_ontology;
use pr4xis_domains::social::software::markup::xml::owl::prx::load_prx_gz;
use pr4xis_domains::social::software::markup::xml::owl::reader::read_owl;
use pr4xis_domains::social::software::markup::xml::owl::vocabulary::LoadedOwlVocabulary;
use pr4xis_domains::social::software::markup::xml::uslm::corpus::UsCode;
use pr4xis_domains::social::software::markup::xml::uslm::corpus::bridge::usc_runtime_ontology_from_cached_defines;
use pr4xis_domains::social::software::markup::xml::uslm::corpus::identifiers::title_cited_by;
use pr4xis_domains::social::software::markup::xml::uslm::lens::read_uslm_title;
use pr4xis_runtime::address::ContentAddress;
use pr4xis_runtime::ontology::{RuntimeOntology, materialize_bytes};
use rkyv::util::AlignedVec;
use std::collections::BTreeMap;

/// The complete WordNet ontology, baked in as the STORE BUNDLE `.stores.gz`
/// (the nine BUILT English store buffers, framed + gzipped — emitted by
/// build.rs). `load_english` gunzips and ASSEMBLES the full `English` by
/// per-store validation alone — fail-closed against the `praxis.lock`
/// `[store_bundle_signatures]` pin. NO WordNet decode and NO `from_wordnet`
/// run in the browser: the former +348 MiB owned-map load transient — which
/// wasm32's never-shrinking linear memory paid permanently — collapses to
/// ~the resident cost.
const ENGLISH_STORES_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/english.stores.gz"));

/// Materialize the embedded English through the SAME fail-closed store-bundle
/// content gate the native `english_load_owned()` fastest tier takes: look the
/// `[store_bundle_signatures]` pin up in the embedded registry by the ONE
/// registered `Language`-kind source (never a name literal), then
/// gunzip → verify the framed bytes hash to that pin → split frames →
/// per-store validate → assemble. The pin ships inside the wasm (the registry
/// `.prx` is baked in), so the gate needs no filesystem, and the pin's
/// same-toolchain trust class holds BY CONSTRUCTION: build.rs emitted the
/// embedded bundle from this build's own Cargo.lock. Fail-closed and LOUD: an
/// unpinned Language source, or embedded bytes that do not hash to the
/// committed pin (tampered / stale / an empty-corpus build), refuse — a
/// build-invariant violation, never a silent install.
fn load_english() -> English {
    use pr4xis_domains::applied::data_provisioning::registry::{
        data_sources, lock_store_bundle_signature,
    };
    use pr4xis_domains::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;

    let entry = data_sources()
        .iter()
        .find(|e| e.kind == SourceTaxonomyConcept::Language)
        .expect("load_english(): no Language-kind source registered");
    let key = format!("{}@{}", entry.name, entry.version);
    let pin = lock_store_bundle_signature(&entry.name, &entry.version).unwrap_or_else(|| {
        panic!("load_english(): no [store_bundle_signatures] pin for {key}; refusing to load")
    });
    load_english_store_bundle_gz_gated(ENGLISH_STORES_GZ, pin, &key).unwrap_or_else(|e| {
        panic!(
            "load_english(): embedded english.stores.gz failed the store-bundle content gate: {e}"
        )
    })
}

/// The single embedded English instance, materialized ONCE from the baked
/// `.prx.gz` behind a process-wide `OnceLock` and shared as a `&'static English`.
///
/// This is the browser analogue of the native `english_loaded()` — but built from
/// the baked bytes ([`load_english`]) instead of `std::fs`, because the wasm
/// runtime has no filesystem. Both [`Pr4xis`] and the [`ComposedReasoner`] it
/// builds BORROW this one instance, so the ~73 MiB WordNet model is resident
/// exactly once rather than owned twice. Sound because `English` is `Sync` and the
/// `OnceLock` makes the instance genuinely `'static`.
fn english_static() -> &'static English {
    use std::sync::OnceLock;
    static INSTANCE: OnceLock<English> = OnceLock::new();
    INSTANCE.get_or_init(load_english)
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

/// Build-time catalog of USC titles with a pre-projected, zero-copy-loadable
/// compact archive (task #21): `(name, version, url, bytes, root_hex)`. A
/// title absent here still appears in [`sources_manifest`] (the raw-XML
/// fallback) — this is an ADDITIONAL, faster route, not a replacement.
mod usc_archives_manifest {
    include!(concat!(env!("OUT_DIR"), "/usc_archives_manifest.rs"));
}

/// The cached statutory-definition overlay, baked at build time so BOTH USC
/// load routes ground the same `defines` edges.
///
/// Only `defines` edges grounded into English populate the definition index
/// `ComposedReasoner` consults, so a title materialized without this overlay
/// loads successfully and answers nothing about the terms it defines. The
/// pre-projected archives carried it and the raw-USLM path did not, which made
/// "Load raw XML" a route that reported success and delivered no capability.
/// Deriving the pairs in the browser is not an option (~1.5h for Title 42);
/// they are known at build time, so they ship.
mod usc_defines_overlay {
    include!(concat!(env!("OUT_DIR"), "/usc_defines_overlay.rs"));
}

/// The names this deployment fetches at startup — the third residency state,
/// beside `EMBEDDED_PRX`'s resident and one-click entries. See
/// `emit_eager_residency` in `build.rs` for why the policy lives at the
/// deployment layer rather than in the registry.
mod eager_residency {
    include!(concat!(env!("OUT_DIR"), "/eager_residency.rs"));
}

/// The embedded new-format `.prx` manifest — one `EMBEDDED_PRX` table of
/// [`embedded_prx::EmbeddedOntology`] entries, each a compiled domain `Category`
/// projected to a content-addressed Archive by `build.rs` with its bytes,
/// trusted Merkle root, name, and `default_loaded` residency baked in. The
/// LegalSources base (`default_loaded: true`) and the Avizienis et al. (2004)
/// Dependability demo (`default_loaded: false`) are ordinary manifest entries;
/// the browser loads any of them fail-closed against its root without a network,
/// and a fetched/uploaded `.prx` flows through the exact same
/// [`Pr4xis::load`] path.
mod embedded_prx {
    include!(concat!(env!("OUT_DIR"), "/embedded_prx.rs"));
}

/// The embedded default-loaded base entries — the always-present grounding
/// ontologies `Pr4xis::new` installs at construction (the LegalSources base and
/// the two caregiving lexicons). The one place the `default_loaded` residency
/// partition is read on the install side, so `new()` carries no per-name
/// special case, and adding a base changes only `build.rs`.
fn embedded_base() -> impl Iterator<Item = &'static embedded_prx::EmbeddedOntology> {
    embedded_prx::EMBEDDED_PRX
        .iter()
        .filter(|e| e.default_loaded)
}

/// How the ontology named `name` *arrived*: baked into this binary, or fetched
/// over the network at runtime.
///
/// Read from the embedded manifest, never assumed. Every `EMBEDDED_PRX` entry —
/// the resident bases AND the one-click demo alike — ships as `include_bytes!`
/// and is materialized without a network, which is exactly [`Staging::Embedded`]
/// (`StaticInput`, frozen at build time), the same class as the English base.
/// Everything else was streamed and parsed at runtime: [`Staging::Async`].
///
/// The one exception is the derived title lexicon, which arrived by neither
/// route: it was never a byte stream and is not in the manifest, it is composed
/// in-process from ontologies already present. That is exactly what
/// [`Staging::Composed`] names, and reporting it as `Async` would claim a
/// download that never happened.
fn staging_of(name: &str) -> Staging {
    if name == USC_TITLE_LEXICON_NAME {
        return Staging::Composed;
    }
    match embedded_entry(name) {
        Some(_) => Staging::Embedded,
        None => Staging::Async,
    }
}

/// How the ontology named `name` came to be loaded — the property that decides
/// whether the host may offer to release it. See [`Residency`].
///
/// Three of the four answers come from declared deployment data rather than
/// from any judgement here: the embedded manifest's `default_loaded` flag names
/// the resident bases, `EAGER_RESIDENT` names what this deployment prefetches,
/// and anything else is loaded because someone asked for it.
///
/// The fourth compares against `USC_TITLE_LEXICON_NAME`. That is identity
/// equality against the canonical name its own producer mints from — not a
/// string test on input — but it IS a name this function knows, and an earlier
/// version of this comment claimed none was. Stamping residency onto the
/// ontology at install time, so `loaded_refs` could read `onto.residency()`
/// instead, would remove even that; recorded rather than done because it
/// touches all four states and none of them is currently wrong.
///
/// The distinction is not cosmetic. A resident base reported as though a reader
/// had fetched it gets an unload control the host cannot honour in reverse —
/// there is no load act to re-run, because the reader never performed one.
fn residency_of(name: &str) -> Residency {
    // Derived first: the title lexicon is a FUNCTION of the loaded set, so it
    // was obtained by no control act and no control act releases it. Offering
    // an Unload here would appear to succeed while every title it names stays
    // loaded — and the next load would silently bring it back.
    if name == USC_TITLE_LEXICON_NAME {
        return Residency::Derived;
    }
    if embedded_entry(name).is_some_and(|e| e.default_loaded) {
        return Residency::Resident;
    }
    if eager_residency::EAGER_RESIDENT.contains(&name) {
        return Residency::Eager;
    }
    Residency::Elective
}

/// The number of on-demand (non-`default_loaded`) entries in the embedded
/// manifest, counted in a `const fn` so the cardinality is known at compile time.
const fn on_demand_demo_count() -> usize {
    let mut count = 0;
    let mut i = 0;
    while i < embedded_prx::EMBEDDED_PRX.len() {
        if !embedded_prx::EMBEDDED_PRX[i].default_loaded {
            count += 1;
        }
        i += 1;
    }
    count
}

/// The manifest must carry EXACTLY ONE on-demand demo — the single entry
/// [`embedded_demo`] returns. Enforced at COMPILE time (not a runtime or
/// `debug_assert` check that could disappear between profiles): a second
/// non-`default_loaded` entry added to `build.rs` fails the build right here, so
/// [`embedded_demo`]'s `.find` can never silently shadow one behind another.
const _: () = assert!(
    on_demand_demo_count() == 1,
    "the embedded manifest must carry exactly one on-demand demo .prx"
);

/// The single on-demand embedded demo `.prx` — the manifest's non-`default_loaded`
/// entry (the Dependability taxonomy). The UI offers it as a one-click load; it
/// flows through the same fail-closed `.prx` core as the base and any fetched
/// `.prx`. `.find` returns THE one entry, not merely the first of several: the
/// `on_demand_demo_count() == 1` compile-time assertion above guarantees exactly
/// one non-`default_loaded` entry exists, so the `.expect` is an unreachable total
/// witness, never a silent tie-break.
pub(crate) fn embedded_demo() -> &'static embedded_prx::EmbeddedOntology {
    embedded_prx::EMBEDDED_PRX
        .iter()
        .find(|e| !e.default_loaded)
        .expect("the embedded manifest carries exactly one on-demand demo .prx")
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
    /// The embedded English language model — BORROWED from the single
    /// [`english_static`] instance, not owned. The `ComposedReasoner` in
    /// `composed` borrows the SAME instance, so English is resident once.
    english: &'static English,
    /// Every runtime-loaded source — USC titles, OWL vocabularies, and
    /// new-format `.prx` ontologies — projected into the generic
    /// [`Archive`](pr4xis_runtime::archive::Archive) by its functor-as-data
    /// bridge and materialized into one queryable [`RuntimeOntology`] set
    /// (content-address identity). THE single loaded-knowledge collection: the
    /// chat reasons over all of it (grounded into English by `composed`) and the
    /// self-model catalog reports all of it. No source is held aside in a second
    /// collection the reasoner never sees.
    ///
    /// Held as shared `Rc` handles: the `ComposedReasoner` in `composed` reasons
    /// over the SAME `RuntimeOntology` instances (cheap `Rc` clones), so each
    /// loaded archive/closure buffer is resident once, never deep-copied into the
    /// reasoner.
    runtime_ontologies: Vec<Rc<RuntimeOntology>>,
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
    /// Multi-turn slot-filling state (task #17) — one session for this
    /// `Pr4xis` instance's whole page lifetime, so a `ChatOutcome::
    /// Conditional` prompt's pending rule survives to the next `chat` call.
    session: pr4xis_chat::ChatSession,
}

/// WHAT is being loaded — the ONE typed selector that resolves the (decoder,
/// projection functor) pair by TYPED dispatch (doc §3), never a byte-sniff or a
/// string match on format. Grounds on the cited `ContentType` /
/// `SourceTaxonomyConcept` provisioning ontology
/// (`applied::data_provisioning::ontology`): [`Encoding::UslmTitle`] is that
/// ontology's `UslmXml` (1 U.S.C. §204), [`Encoding::OwlSource`] its `Owl`
/// (W3C OWL 2 RDF/XML). The two praxis-native envelope forms —
/// [`Encoding::OwlPrxGz`] (the OWL `.prx.gz` distribution envelope) and
/// [`Encoding::RkyvArchive`] (the `rkyv` local-cache `.prx` archive) — belong
/// to praxis's own serialization ontology. The `(decoder, functor)` per
/// variant is [`decode_and_project`]'s single typed match; the JS↔wasm boundary
/// carries only the wire tag, decoded ONCE by [`Encoding::from_wire`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Encoding {
    /// USLM XML title (1 U.S.C. §204). Decoder `read_uslm_title`; functor
    /// `usc_runtime_ontology_from_cached_defines`, which applies the SAME
    /// baked statutory-definition overlay the pre-projected archive carries —
    /// so both routes yield a title whose defined terms chat can actually
    /// answer, not merely one that loaded. Cited `ContentType::UslmXml`. An
    /// honest fallback for a title with no [`Encoding::RkyvArchive`] yet —
    /// never the default path for one that has it (build.rs stages an archive
    /// for every registered USC title it can parse).
    UslmTitle,
    /// OWL 2 RDF/XML source (W3C OWL 2). Decoder `read_owl`; functor
    /// `owl_runtime_ontology`. Cited `ContentType::Owl`.
    OwlSource,
    /// The OWL `.prx.gz` distribution envelope. Decoder `owl::prx::load_prx_gz`
    /// (gunzip + bytecheck + three-pin verify); functor `owl_runtime_ontology`.
    OwlPrxGz,
    /// A pre-projected `rkyv` local-cache archive — the zero-copy path for
    /// ontologies built by the SAME toolchain/`Cargo.lock` that compiled this
    /// wasm binary, whether the bytes are ALWAYS-EMBEDDED (`include_bytes!`,
    /// e.g. LegalSources, the two chat lexicons, the Dependability demo —
    /// task #29) or NETWORK-FETCHED on demand (e.g. a USC title — task #21).
    /// Neither case needs DAG-CBOR's cross-toolchain, long-lived content
    /// addressing (no bytes here are read by anything but this exact build).
    /// Decoder: copy the payload into a 16-byte-aligned buffer, then
    /// `ontology::materialize_bytes` + re-derived-root refusal — no owned
    /// decode/re-encode pass a DAG-CBOR wire form would pay.
    RkyvArchive,
}

impl Encoding {
    /// The SINGLE fail-closed wire-tag decode at the JS↔wasm boundary (doc §3
    /// point 3). wasm-bindgen has no rich sum-type marshalling, so the encoding
    /// crosses the FFI as one tag; this is a closed match on it (error on
    /// unknown), NOT a content sniff.
    fn from_wire(tag: &str) -> Result<Self, LoadError> {
        match tag {
            "uslm-title" => Ok(Encoding::UslmTitle),
            "owl-source" => Ok(Encoding::OwlSource),
            "owl-prx-gz" => Ok(Encoding::OwlPrxGz),
            "rkyv-archive" => Ok(Encoding::RkyvArchive),
            other => Err(LoadError::UnknownEncoding(other.to_string())),
        }
    }

    /// The wire tag for this encoding — the inverse of [`Encoding::from_wire`],
    /// used in the `Loaded` receipt so the UI can name what it loaded.
    fn wire_tag(self) -> &'static str {
        match self {
            Encoding::UslmTitle => "uslm-title",
            Encoding::OwlSource => "owl-source",
            Encoding::OwlPrxGz => "owl-prx-gz",
            Encoding::RkyvArchive => "rkyv-archive",
        }
    }
}

/// HOW a load is made fail-closed — the three trust models unify into ONE typed
/// anchor verified at one place (doc §3). This is legitimate variation, not
/// fragmentation: a Merkle-root re-derivation, a triple praxis.lock-pin lookup,
/// and transport-only trust are genuinely different checks, but they dispatch
/// off one enum instead of living in five methods.
#[derive(Debug, Clone)]
pub(crate) enum TrustAnchor {
    /// Source bytes carry no embedded hash; integrity rests on the host having
    /// fetched from the registry-pinned URL. ([`Encoding::UslmTitle`],
    /// [`Encoding::OwlSource`].)
    Transport,
    /// OWL `.prx.gz`: three praxis.lock pins looked up by `(name, version)` —
    /// archive signature, source hash, RDFC-1.0 canonical graph id.
    /// ([`Encoding::OwlPrxGz`].)
    LockPinned { version: String },
    /// `rkyv` archive: the trusted Merkle root from OUTSIDE the bytes;
    /// `decode_and_project`'s [`Encoding::RkyvArchive`] arm re-derives it
    /// (via `ontology::materialize_bytes` + `RuntimeOntology::root()`) and
    /// refuses on mismatch.
    MerkleRoot(ContentAddress),
}

impl TrustAnchor {
    /// The `Transport` anchor (bytes carry no embedded hash), or a typed mismatch.
    fn expect_transport(&self, enc: Encoding) -> Result<(), LoadError> {
        match self {
            TrustAnchor::Transport => Ok(()),
            _ => Err(LoadError::TrustMismatch {
                encoding: enc.wire_tag(),
            }),
        }
    }

    /// The lock-pinned `version` (for the three-pin OWL `.prx.gz` lookup), or a
    /// typed mismatch.
    fn expect_lock_version(&self, enc: Encoding) -> Result<&str, LoadError> {
        match self {
            TrustAnchor::LockPinned { version } => Ok(version),
            _ => Err(LoadError::TrustMismatch {
                encoding: enc.wire_tag(),
            }),
        }
    }

    /// The trusted Merkle root (for the content-addressed `.prx` gate), or a
    /// typed mismatch.
    fn expect_merkle_root(&self, enc: Encoding) -> Result<ContentAddress, LoadError> {
        match self {
            TrustAnchor::MerkleRoot(root) => Ok(*root),
            _ => Err(LoadError::TrustMismatch {
                encoding: enc.wire_tag(),
            }),
        }
    }
}

/// The ONE typed entry for loading knowledge (doc §3). Names WHAT is loaded (a
/// typed `Encoding`, never a byte-sniff), carries the resolved PAYLOAD bytes,
/// and the TRUST anchor that makes the load fail-closed. [`Pr4xis::load_core`]
/// resolves the decoder + projection functor for the encoding, verifies the
/// anchor, decodes → projects → installs — the single path every source now
/// takes into [`Pr4xis::install_runtime_ontology`].
#[derive(Debug)]
struct LoadRequest {
    /// The runtime ontology name the projected [`RuntimeOntology`] installs under.
    name: String,
    /// WHAT this is — selects `(decoder, functor)` by typed dispatch.
    encoding: Encoding,
    /// The raw bytes (text payloads are their UTF-8 bytes, so ONE representation
    /// crosses the boundary regardless of format). An absent boundary payload is
    /// resolved from the build-baked embedded corpus by `name` in
    /// `LoadRequest::from_wire`.
    payload: Vec<u8>,
    /// HOW the load is made fail-closed — one typed anchor, verified in one place.
    trust: TrustAnchor,
}

impl LoadRequest {
    /// Build the typed request from the JS↔wasm wire fields — the SINGLE tagged
    /// decode at the boundary (doc §3 point 3). Resolves the payload (an absent
    /// boundary payload ⇒ the build-baked embedded bytes for `name`) and the
    /// trust anchor per the typed `Encoding` (version pins `OwlPrxGz`; a
    /// Merkle root — supplied or resolved from the embedded manifest — pins
    /// `RkyvArchive`). Fail-closed: an unknown encoding, a missing
    /// version, a missing/ill-formed root, or no embedded bytes is a typed error.
    fn from_wire(
        name: String,
        encoding: &str,
        version: Option<String>,
        root_hex: Option<String>,
        payload: Option<Vec<u8>>,
    ) -> Result<Self, LoadError> {
        let encoding = Encoding::from_wire(encoding)?;
        // Resolve the payload: an absent boundary payload means "load the
        // build-baked bytes for this name" (the demo/base case).
        let embedded = if payload.is_none() {
            embedded_entry(&name)
        } else {
            None
        };
        let payload = match payload {
            Some(bytes) => bytes,
            None => embedded
                .ok_or_else(|| LoadError::NoEmbedded(name.clone()))?
                .bytes
                .to_vec(),
        };
        let trust = match encoding {
            // Source bytes carry no embedded hash — transport trust.
            Encoding::UslmTitle | Encoding::OwlSource => TrustAnchor::Transport,
            // The OWL `.prx.gz` three-pin lock lookup is keyed by version.
            Encoding::OwlPrxGz => TrustAnchor::LockPinned {
                version: version.ok_or(LoadError::MissingVersion)?,
            },
            // The rkyv archive's root is the trusted anchor: supplied by the
            // caller, or (for an embedded load) the manifest's baked root.
            Encoding::RkyvArchive => {
                let hex = match root_hex {
                    Some(hex) => hex,
                    None => embedded.ok_or(LoadError::MissingRoot)?.root_hex.to_string(),
                };
                TrustAnchor::MerkleRoot(
                    ContentAddress::from_hex(&hex).ok_or(LoadError::BadRootHex(hex))?,
                )
            }
        };
        Ok(LoadRequest {
            name,
            encoding,
            payload,
            trust,
        })
    }
}

/// The receipt a successful [`Pr4xis::load`] returns — a small structured record
/// so the UI can name WHAT it loaded (subsumes the old `load_embedded_demo_prx`
/// String return). Projected to JSON at the wasm boundary.
struct Loaded {
    name: String,
    encoding: Encoding,
    bytes: usize,
    root: String,
}

impl Loaded {
    fn to_json(&self) -> String {
        let mut p = Presentation::new();
        p.set("name", SchemaValue::Text(self.name.clone()));
        p.set(
            "encoding",
            SchemaValue::Text(self.encoding.wire_tag().to_string()),
        );
        p.set("bytes", SchemaValue::Unsigned(self.bytes as u64));
        p.set("root", SchemaValue::Text(self.root.clone()));
        p.to_json()
    }
}

/// Why a [`Pr4xis::load`] failed — the ONE typed core error the whole load path
/// shares, rendered to a `JsValue` only at the wasm boundary. Unifies the old
/// `LoadPrxError` (the content-addressed gate's precise verdicts) with the
/// USLM/OWL decode/materialize failures that were previously stringly-typed, so
/// every failure is a precise typed value, never a `format!` blob.
#[derive(Debug)]
pub(crate) enum LoadError {
    /// The wire tag was not a known `Encoding`.
    UnknownEncoding(String),
    /// [`Encoding::OwlPrxGz`] needs a `version` for its three-pin lock lookup.
    MissingVersion,
    /// [`Encoding::RkyvArchive`] needs a trusted Merkle root.
    MissingRoot,
    /// The supplied trust anchor does not match the encoding's required kind.
    TrustMismatch { encoding: &'static str },
    /// An absent boundary payload named no build-baked embedded ontology.
    NoEmbedded(String),
    /// A Merkle root hex was not a 64-char lowercase-hex digest.
    BadRootHex(String),
    /// The three praxis.lock pins for `name@version` were absent — the `.prx.gz`
    /// cannot be validated.
    MissingLockPin(String),
    /// USLM parse / UTF-8 decode failure.
    UslmParse(String),
    /// USC projection into the runtime ontology failed.
    UscMaterialize(String),
    /// OWL parse / UTF-8 decode failure.
    OwlParse(String),
    /// OWL projection into the runtime ontology failed.
    OwlMaterialize(String),
    /// The OWL `.prx.gz` gate rejected the envelope (gunzip / bytecheck / a pin
    /// mismatch).
    PrxGz(String),
    /// The admitted archive could not be materialized (e.g. a dangling edge —
    /// referential closure violated, or the buffer failed `bytecheck`
    /// validation).
    Materialize(pr4xis_runtime::ontology::MaterializeError),
    /// [`Encoding::RkyvArchive`]: the buffer `bytecheck`-validated and
    /// materialized, but its re-derived [`ContentAddress`] does not match the
    /// trusted root — tampered, stale, or wrong bytes. `materialize_bytes`
    /// takes no trusted root of its own to check against, so this composes
    /// the missing "verify" leg at the call site.
    RkyvRootMismatch {
        expected: ContentAddress,
        actual: ContentAddress,
    },
    /// Installing surfaced a LOUD grounding fault from the single
    /// [`ground_loaded_set`] pass — a declared target concept NAME absent from a
    /// present peer, an unsupported multi-level chain, or a skew. Fail-closed:
    /// nothing installs. (A merely not-yet-loaded target ontology DEFERS.)
    Grounding(pr4xis_runtime::grounding::LinkError),
}

impl core::fmt::Display for LoadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LoadError::UnknownEncoding(tag) => write!(f, "unknown load encoding {tag:?}"),
            LoadError::MissingVersion => {
                write!(
                    f,
                    "the owl-prx-gz encoding requires a version for the lock lookup"
                )
            }
            LoadError::MissingRoot => write!(
                f,
                "the rkyv-archive encoding requires a trusted Merkle root"
            ),
            LoadError::TrustMismatch { encoding } => {
                write!(
                    f,
                    "the supplied trust anchor does not match the {encoding} encoding"
                )
            }
            LoadError::NoEmbedded(name) => {
                write!(f, "no build-baked embedded ontology named {name:?}")
            }
            LoadError::BadRootHex(got) => {
                write!(
                    f,
                    "expected_root must be 64-char lowercase hex; got {got:?}"
                )
            }
            LoadError::MissingLockPin(key) => {
                write!(
                    f,
                    "no embedded praxis.lock pin for {key}; cannot validate .prx.gz"
                )
            }
            LoadError::UslmParse(e) => write!(f, "USLM parse failed: {e}"),
            LoadError::UscMaterialize(e) => write!(f, "USC materialize failed: {e}"),
            LoadError::OwlParse(e) => write!(f, "OWL parse failed: {e}"),
            LoadError::OwlMaterialize(e) => write!(f, "OWL materialize failed: {e}"),
            LoadError::PrxGz(e) => write!(f, ".prx.gz load/validate failed: {e}"),
            LoadError::Materialize(e) => write!(f, ".prx materialize failed: {e}"),
            LoadError::RkyvRootMismatch { expected, actual } => write!(
                f,
                "rkyv archive root mismatch: expected {}, got {}",
                expected.to_hex(),
                actual.to_hex()
            ),
            LoadError::Grounding(e) => write!(f, "grounding failed: {e}"),
        }
    }
}

/// The build-baked embedded ontology named `name`, if any — the resolver an
/// absent boundary payload uses to recover the bytes (and, for a content-
/// addressed archive, the trusted root) that ship inside the wasm.
fn embedded_entry(name: &str) -> Option<&'static embedded_prx::EmbeddedOntology> {
    embedded_prx::EMBEDDED_PRX.iter().find(|e| e.name == name)
}

/// The ONE typed decode-and-project step (doc §3). Resolves the `(decoder,
/// projection functor)` for `encoding` by a single typed match — the functor-
/// as-typed-selector — verifying the `TrustAnchor` in the same arm (trust and
/// decode are fused for the two binary envelope formats, which re-derive the
/// content address from the very bytes they decode). Every arm converges on one
/// [`RuntimeOntology`] codomain, which is why [`Pr4xis::load_core`] has a single
/// structural shape.
pub(crate) fn decode_and_project(
    name: &str,
    encoding: Encoding,
    trust: &TrustAnchor,
    bytes: &[u8],
) -> Result<RuntimeOntology, LoadError> {
    match encoding {
        Encoding::UslmTitle => {
            trust.expect_transport(encoding)?;
            let xml = core::str::from_utf8(bytes)
                .map_err(|e| LoadError::UslmParse(format!("not UTF-8: {e}")))?;
            let title = read_uslm_title(xml).map_err(|e| LoadError::UslmParse(format!("{e:?}")))?;
            let usc = UsCode::from_uslm_titles_owned(vec![title]);
            // WITH the defines overlay — the same one the pre-projected
            // archive carries. Without it this route materialized a title
            // whose statutory definitions no `define_word` call could reach,
            // so the load succeeded and the capability did not arrive.
            usc_runtime_ontology_from_cached_defines(
                &usc,
                OntologyName::new(name.to_string()),
                usc_defines_overlay::USC_DEFINES_OVERLAY
                    .iter()
                    .map(|(urn, term)| ((*urn).to_string(), (*term).to_string()))
                    .collect::<Vec<_>>()
                    .as_slice(),
            )
            .map_err(|e| LoadError::UscMaterialize(format!("{e:?}")))
        }
        Encoding::OwlSource => {
            trust.expect_transport(encoding)?;
            let xml = core::str::from_utf8(bytes)
                .map_err(|e| LoadError::OwlParse(format!("not UTF-8: {e}")))?;
            let ont = read_owl(xml).map_err(|e| LoadError::OwlParse(format!("{e}")))?;
            let vocab = LoadedOwlVocabulary::from_owl_ontology(&ont);
            owl_runtime_ontology(&vocab, OntologyName::new(name.to_string()))
                .map_err(|e| LoadError::OwlMaterialize(format!("{e:?}")))
        }
        Encoding::OwlPrxGz => {
            let version = trust.expect_lock_version(encoding)?;
            let key = format!("{name}@{version}");
            let archive_pin = lock_archive_signature(name, version)
                .ok_or_else(|| LoadError::MissingLockPin(format!("[archive_signatures] {key}")))?;
            let source_pin = lock_hashes()
                .get(&key)
                .ok_or_else(|| LoadError::MissingLockPin(key.clone()))?;
            let canonical_pin = lock_canonical_signature(name, version).ok_or_else(|| {
                LoadError::MissingLockPin(format!("[canonical_signatures] {key}"))
            })?;
            let vocab = load_prx_gz(bytes, archive_pin, source_pin, canonical_pin)
                .map_err(|e| LoadError::PrxGz(format!("{key}: {e}")))?;
            owl_runtime_ontology(&vocab, OntologyName::new(name.to_string()))
                .map_err(|e| LoadError::OwlMaterialize(format!("{key}: {e:?}")))
        }
        Encoding::RkyvArchive => {
            let root = trust.expect_merkle_root(encoding)?;
            // Fetched bytes carry no alignment guarantee; rkyv's zero-copy
            // access requires 16-byte alignment. Copy into a fresh aligned
            // buffer — the SAME two-step idiom every other rkyv consumer in
            // this workspace uses (e.g. the English store bundle's per-frame
            // `aligned()` helper), never a "validate unaligned in place" API
            // (none exists here or upstream).
            let mut buf = AlignedVec::<16>::with_capacity(bytes.len());
            buf.extend_from_slice(bytes);
            // `materialize_bytes` bytecheck-validates once, then does ONE
            // owned decode solely to derive the root/closure/node-index
            // bookkeeping (no DAG-CBOR pass, no re-PUT of the buffer itself
            // — the validated bytes are kept verbatim as the retained open
            // form). It takes no trusted root of its own, so the fail-closed
            // guarantee is completed here: refuse unless the re-derived root
            // matches the one supplied from OUTSIDE the bytes.
            let ontology = materialize_bytes(buf, OntologyName::new(name.to_string()))
                .map_err(LoadError::Materialize)?;
            if ontology.root() != root {
                return Err(LoadError::RkyvRootMismatch {
                    expected: root,
                    actual: ontology.root(),
                });
            }
            Ok(ontology)
        }
    }
}

/// Project a [`ChatOutcome`] onto a [`Presentation`]'s outcome fields — the ONE
/// place the typed outcome (doc §4.1) lowers to the wire, shared by the single
/// [`Pr4xis::chat`] turn and each per-question row of [`Pr4xis::chat_batch`], so a
/// single answer and a batch row carry byte-identical outcome shape. Answered
/// tags `outcome` only; Abstained names its unresolved surfaces (WHAT TO LOAD
/// next); a Conditional/RuleResolved carries the cited rule (`rule_name`,
/// `rule_definition`, `rule_citation`) — the rule identifier the console renders —
/// plus, for a still-open Conditional, the `missing_facts` a human must supply.
fn write_outcome(p: &mut Presentation, outcome: &ChatOutcome) {
    // The rule's citation surface (statutory source text, else its subsection in
    // Bluebook form) — the same projection `chat`'s Conditional/RuleResolved arms
    // used inline before this became the shared lowering.
    fn rule_citation(
        rule: &pr4xis_domains::social::judicial::conditional_rule::ConditionalRule,
    ) -> String {
        rule.term
            .source_text
            .as_ref()
            .map(|s| s.text.clone())
            .or_else(|| rule.term.subsection.as_ref().map(|c| c.to_bluebook()))
            .unwrap_or_default()
    }

    match outcome {
        ChatOutcome::Answered => {
            p.set("outcome", "answered".into());
        }
        ChatOutcome::Abstained { unresolved } => {
            p.set("outcome", "abstained".into());
            // Each unresolved surface travels WITH the source that would
            // resolve it, when the engine can say which.
            //
            // The host used to work that out for itself, by regexing a US Code
            // citation out of the term and rebuilding `usc_title_${n}` — a
            // citation grammar and a registry naming convention, both encoded
            // in the page, both facts this engine already owns
            // (`UsCodeTitleId::source_name` documents itself as the single
            // point where that convention lives). `title_cited_by` recognizes
            // the surface against each REGISTERED title's own published
            // citation forms, so nothing about citation syntax is written down
            // anywhere, and a title this deployment does not carry routes to
            // nothing instead of to a card that does not exist.
            //
            // `source_id` is absent when no registered source is named — the
            // host then filters the catalog by the surface itself, which is
            // what it did before and remains the honest fallback.
            p.set(
                "unresolved",
                SchemaValue::List(
                    unresolved
                        .iter()
                        .map(|surface| {
                            let mut u = Presentation::new();
                            u.set("surface", SchemaValue::Text(surface.clone()));
                            u.set(
                                "source_id",
                                match title_cited_by(surface) {
                                    Some(id) => SchemaValue::Text(id.source_name()),
                                    None => SchemaValue::Absent,
                                },
                            );
                            SchemaValue::Record(u)
                        })
                        .collect(),
                ),
            );
        }
        ChatOutcome::Conditional { rule, missing } => {
            p.set("outcome", "conditional".into());
            p.set("rule_name", rule.term.name.text.clone().into());
            p.set("rule_definition", rule.term.definition.text.clone().into());
            p.set("rule_citation", rule_citation(rule).into());
            p.set(
                "missing_facts",
                SchemaValue::List(
                    missing
                        .iter()
                        .map(|el| el.requirement.field.text.clone().into())
                        .collect(),
                ),
            );
        }
        ChatOutcome::RuleResolved { rule, applies } => {
            p.set(
                "outcome",
                if *applies {
                    "rule_applies"
                } else {
                    "rule_does_not_apply"
                }
                .into(),
            );
            p.set("rule_name", rule.term.name.text.clone().into());
            p.set("rule_definition", rule.term.definition.text.clone().into());
            p.set("rule_citation", rule_citation(rule).into());
        }
    }
}

/// The MAPE-K phase's own word for a control-loop role, read from the MapeK
/// ontology's [`labels()`](MapeKOntology::labels) table — NOT a hand-written
/// enum→string map. The trace's `phase` field is thus the ontology's label
/// (Kephart & Chess 2003), so a renamed phase moves with its ontology.
fn mape_k_phase_label(phase: MapeKConcept) -> &'static str {
    MapeKOntology::labels()
        .iter()
        .find(|(concept, ..)| *concept == phase)
        .map(|(_, _, label, _)| *label)
        .unwrap_or_default()
}

/// Lower a [`PipelineTrace`] to a structured [`SchemaValue::List`] of per-step
/// records — the TYPED trace (doc §5.3) that crosses the wire beside the existing
/// flattened `pipe|colon` string, so the console renders each stage's ontology,
/// operation, MAPE-K phase, status, detail, the loaded ontologies it reasoned
/// over, and its proven functor connections (each carrying the literature
/// `reference` the flatten discarded — Shannon 1948, Kamp 1981, …) without ever
/// parsing the string form. The same Presentation idiom the `ontologies` field
/// already uses, applied per trace entry.
fn trace_to_structured(trace: &PipelineTrace) -> SchemaValue {
    SchemaValue::List(
        trace
            .entries
            .iter()
            .map(|entry| {
                let mut r = Presentation::new();
                r.set("ontology", SchemaValue::Text(entry.ontology().to_string()));
                r.set(
                    "operation",
                    SchemaValue::Text(entry.operation().to_string()),
                );
                r.set(
                    "phase",
                    SchemaValue::Text(mape_k_phase_label(entry.step.phase()).to_string()),
                );
                r.set("detail", SchemaValue::Text(entry.detail.clone()));
                r.set("success", SchemaValue::Boolean(entry.success));
                // The same ok/warn status the flattened `serialize()` derives from
                // `success`, carried explicitly so the renderer needs no re-derivation.
                r.set(
                    "status",
                    SchemaValue::Text(if entry.success { "ok" } else { "warn" }.to_string()),
                );
                r.set(
                    "reasoned_over",
                    SchemaValue::List(
                        entry
                            .reasoned_over
                            .iter()
                            .map(|n| SchemaValue::Text(n.as_str().to_string()))
                            .collect(),
                    ),
                );
                r.set(
                    "functor_connections",
                    SchemaValue::List(
                        entry
                            .functor_connections()
                            .iter()
                            .map(|c| {
                                let mut fc = Presentation::new();
                                fc.set("target", SchemaValue::Text(c.target_ontology.to_string()));
                                fc.set("functor", SchemaValue::Text(c.functor_name.to_string()));
                                fc.set("reference", SchemaValue::Text(c.reference.to_string()));
                                SchemaValue::Record(fc)
                            })
                            .collect(),
                    ),
                );
                SchemaValue::Record(r)
            })
            .collect(),
    )
}

/// Build the FULL `chat` wire envelope for a completed [`ProcessResult`] — the ONE
/// place a turn's every field is projected, shared by the single [`Pr4xis::chat`]
/// turn and each per-question row of [`Pr4xis::chat_batch`]. A batch row is thus
/// byte-identical to a single `chat` answer (plus its echoed `question`), so the
/// page's ONE renderer handles both surfaces without hand-mapping. Carries:
/// `response`, `duration_us`, `parsed`, `from_ontology`, the typed outcome (via
/// [`write_outcome`]), the `ontologies` this answer reasoned over, the flattened
/// `trace` string, and the typed `trace_structured` list.
fn build_chat_presentation(result: &pr4xis_chat::ProcessResult) -> Presentation {
    let mut p = Presentation::new();
    p.set("response", result.response.clone().into());
    p.set("duration_us", result.duration_us.into());
    p.set("parsed", result.parsed.into());
    p.set("from_ontology", result.from_ontology.into());
    // The TYPED outcome (doc §4.1): answered, or abstained naming the surfaces
    // to load — so the UI can model what the system cannot answer, not sniff it.
    write_outcome(&mut p, &result.outcome);
    // U6/U7: the ontologies this answer REASONED OVER — the compiled pipeline PLUS
    // every loaded `.prx` it drew on (`reasoned_over`, not the compiled-only
    // `all_participating_ontologies`), each a structured record carrying its
    // provenance + success bit. The page projects this, so the list GENERALISES as
    // ontologies load — never a hardcoded pipeline.
    p.set(
        "ontologies",
        SchemaValue::List(
            result
                .trace
                .reasoned_over()
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
    // The flattened `pipe|colon` trace string (kept for the existing renderer) and
    // the TYPED trace (doc §5.3) crossing the wire BESIDE it — additive, never a
    // replacement, so a consumer can render each stage structurally without ever
    // parsing the string.
    p.set("trace", result.trace.serialize_with_functors().into());
    p.set("trace_structured", trace_to_structured(&result.trace));
    // The plain-language "Why?" explanation (doc §5.2), a sibling of
    // `trace_structured` — the frontend renders a Why? panel whenever the wire
    // carries `why`. Additive and non-breaking: set only when the engine produced
    // one (a self-explaining outcome leaves it absent, so the page shows no
    // panel). The sentence is a LOADED frame realized in the engine
    // (`realize_why`), never assembled in JS.
    if let Some(why) = &result.why {
        p.set("why", why.clone().into());
    }
    // The DEFINITION-PROVENANCE channel — its own record, never folded into
    // `why` and never derivable from `ontologies`. `ontologies` says what this
    // turn opened; this says what a lexicographer read when writing a gloss the
    // answer recited. A page that had to tell those apart by inspecting the
    // `why` sentence would be doing engine work in JavaScript, so both the
    // channel's LABEL and its SENTENCE are realized here, in the engine, from
    // the loaded explain-frames table.
    if let Some(provenance) = &result.definition_provenance {
        let mut r = Presentation::new();
        r.set("label", SchemaValue::Text(provenance.label.clone()));
        r.set("detail", SchemaValue::Text(provenance.detail.clone()));
        p.set("definition_provenance", SchemaValue::Record(r));
    }
    p
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
            english: english_static(),
            runtime_ontologies: Vec::new(),
            composed: None,
            history: Vec::new(),
            session: pr4xis_chat::ChatSession::new(),
        };
        // De-privileged base install: iterate the embedded `.prx` manifest and
        // load every `default_loaded` entry through the EXACT fail-closed `.prx`
        // core a fetched/uploaded `.prx` takes — no hand-wired special case for
        // any one ontology. The always-loaded LegalSources BASE (LKIF-Core formal
        // sources-of-law taxonomy, lexicalized so "law"/"case law" ground for
        // chat) is simply the manifest's `default_loaded` entry, so from
        // construction `composed` is `Some(...)` and EVERY chat reasons over the
        // formal sources of law: "is a statute a law" answers Yes out of the box,
        // with no explicit load. A failure here is a build-time invariant
        // violation (the bytes + pins ship embedded in the wasm).
        for entry in embedded_base() {
            this.load_ontology_prx_core(entry.bytes, entry.name.to_string(), entry.root_hex)
                .expect(
                    "an embedded default-loaded base .prx loads fail-closed against its baked root",
                );
        }
        this
    }

    pub fn chat(&mut self, input: &str) -> String {
        // Once a `.prx` is loaded, reason through the ComposedReasoner (the
        // loaded ontology grounded into English) so "what is X" answers from
        // the loaded gloss; otherwise reason through English alone (abstains on
        // an unloaded concept). The linguistic substrate (`lang`) is always the
        // composed reasoner's own English when present, so tokenize/parse and
        // the lexical surface agree. Routed through `self.session` (task #17)
        // rather than `process_with_reasoner`/`process_with_metadata`
        // directly, so a `ChatOutcome::Conditional` prompt's pending rule
        // survives to the next call on this SAME `Pr4xis` instance.
        let result = match &self.composed {
            Some(composed) => self.session.ask(composed.english(), composed, input),
            None => self.session.ask(self.english, self.english, input),
        };
        // The full wire envelope — the SAME projection `chat_batch` writes per
        // question, INCLUDING the echoed `question`, so a single turn and a
        // batch row are byte-identical in shape and one renderer handles both.
        // The echo is what makes a downloaded decision record self-contained:
        // a record that carries the reasoning but not the question it answers
        // is not a record a compliance function can read months later.
        let mut r = build_chat_presentation(&result);
        r.set("question", SchemaValue::Text(input.to_string()));
        self.stamp_provenance(&mut r);
        r.to_json()
    }

    /// Stamp WHICH ENGINE answered and UNDER WHAT KNOWLEDGE onto one turn's
    /// envelope — applied identically by [`Self::chat`] and [`Self::chat_batch`].
    ///
    /// A decision record is meant to be re-derivable months later, and "which
    /// authorities were loaded" is the fact that decides whether it can be. The
    /// host cannot supply it safely: a `self_describe` issued after the turn
    /// can observe a DIFFERENT state, because eager residency loads sources in
    /// the background while the reader is asking questions — so the record
    /// would attribute an answer to a knowledge state that did not produce it.
    /// Stamping here closes that window: `state_cid` is the Merkle fold over
    /// the loaded roots at the moment of the answer.
    ///
    /// It is ONE method rather than two call sites because the two paths must
    /// not drift: a batch row is evidence exactly as a single turn is, and
    /// `chat_batch_runs_each_question_statelessly_and_in_order` holds their
    /// wire keys equal. `state_cid` is absent — not a zero hash — when nothing
    /// is loaded, which is the honest "no knowledge state".
    fn stamp_provenance(&self, r: &mut Presentation) {
        r.set(
            "state_cid",
            match self.state_cid() {
                Some(cid) => SchemaValue::Text(cid),
                None => SchemaValue::Absent,
            },
        );
        r.set(
            "engine_version",
            SchemaValue::Text(env!("CARGO_PKG_VERSION").to_string()),
        );
    }

    /// Run a batch of questions through the STATELESS pipeline
    /// ([`process_with_reasoner`](pr4xis_chat::process_with_reasoner)) — the SAME
    /// entry the native Smart-40 probe drives (`scratch_probe.rs`
    /// `probe_smart40_validation_log`), NOT a loop over the stateful [`Self::chat`].
    /// Statelessness per question is the point: `chat` routes through
    /// `self.session`, where a `Conditional` turn leaves a pending rule that would
    /// consume the NEXT question as a slot-fill — corrupting every subsequent row
    /// of a batch. Here each question is independent, mirroring the native harness,
    /// so a page can reproduce the published protocol exactly.
    ///
    /// Reasons through the loaded [`ComposedReasoner`] when a `.prx` is loaded
    /// (answers from loaded glosses), else through embedded English alone (abstains
    /// on an unloaded concept) — the SAME reasoner selection [`Self::chat`] makes.
    ///
    /// Returns `{ results: [ <record>, ... ] }` — an ENVELOPE around one record per
    /// input question, in input order. Each record is the FULL [`Self::chat`] wire
    /// envelope (`build_chat_presentation`) — `response`, `outcome` (+ its
    /// `unresolved` / `rule_name` / `rule_definition` / `rule_citation` /
    /// `missing_facts`), `ontologies`, `from_ontology`, `parsed`, `duration_us`,
    /// `trace`, `trace_structured` — PLUS the echoed `question`, so a batch row and
    /// a single `chat` answer are byte-identical and the page's one renderer handles
    /// both. The input crosses the FFI as a JS `string[]` (wasm-bindgen's native
    /// array marshalling) — no JSON parse at the boundary. Stateless per question,
    /// so calling it repeatedly over sub-slices (chunked progress) yields the SAME
    /// results as one call over the whole array.
    pub fn chat_batch(&self, questions: Vec<String>) -> String {
        let results: Vec<SchemaValue> = questions
            .iter()
            .map(|question| {
                let result = match &self.composed {
                    Some(composed) => {
                        pr4xis_chat::process_with_reasoner(composed.english(), composed, question)
                    }
                    None => {
                        pr4xis_chat::process_with_reasoner(self.english, self.english, question)
                    }
                };
                let mut r = build_chat_presentation(&result);
                r.set("question", SchemaValue::Text(question.clone()));
                self.stamp_provenance(&mut r);
                SchemaValue::Record(r)
            })
            .collect();
        let mut p = Presentation::new();
        p.set("results", SchemaValue::List(results));
        p.to_json()
    }

    /// Clear any pending multi-turn slot-fill state, returning this instance's
    /// stateful [`Self::chat`] session to a fresh start. A `ChatOutcome::
    /// Conditional` turn leaves the session awaiting a fact
    /// ([`ChatSession::is_awaiting_fact`](pr4xis_chat::ChatSession::is_awaiting_fact)),
    /// so the NEXT `chat` call is interpreted as that fact; a host abandoning a
    /// slot-fill dialogue calls this to drop the open frame before asking an
    /// unrelated question. Replacing the session with a fresh
    /// [`ChatSession::new`](pr4xis_chat::ChatSession::new) is the one authoritative
    /// way to reach the no-pending state (`pending: None`).
    pub fn reset_session(&mut self) {
        self.session = pr4xis_chat::ChatSession::new();
    }

    /// Verify the page's live CSS custom-property palette against the theming
    /// ontology's OWN WCAG contrast axioms — the "engine audits its own page"
    /// exhibit (doc §3.6), which also cures the theming ontology's zero-caller
    /// orphan debt. `slot_keys[i]` is a CSS var name (e.g. `"--base05"`) and
    /// `hexes[i]` its value (`"#c9d1d9"`), read live from `getComputedStyle`; a
    /// key naming no base16 slot (a semantic alias like `"--danger"`) or an
    /// unparseable hex is skipped, so the caller may pass its whole token set.
    /// Every ratio is the ontology's own [`contrast_ratio`](pr4xis_domains::natural::colors::srgb::contrast_ratio) — no contrast
    /// math is reimplemented here.
    ///
    /// Returns `{ polarity?, checks: [ { axiom, citation, pair?, ratio?,
    /// required?, pass } ] }`: one summary record per palette axiom
    /// (`WcagForegroundContrast`, `RenderedPairsMeetAa`) carrying its NAME and
    /// CITATION straight from the axiom's own `axiom_meta!` plus its live
    /// verdict, then one detail record per rendered foreground/background pair
    /// (`"<fg> on <bg>"`, the computed contrast `ratio`, its `required` AA
    /// threshold, and `pass`). Pure and deterministic — identical tokens always
    /// yield identical verdicts.
    pub fn verify_palette(&self, slot_keys: Vec<String>, hexes: Vec<String>) -> String {
        use pr4xis::category::FinitelyGenerated;
        use pr4xis::ontology::Axiom;
        use pr4xis_domains::applied::hmi::theming::base16::ColorSlot;
        use pr4xis_domains::applied::hmi::theming::ontology::{
            Palette, RenderedPairsMeetAa, WcagForegroundContrast, detect_polarity, rendered_pairs,
        };
        use pr4xis_domains::natural::colors::rgb::Rgb;
        use pr4xis_domains::natural::colors::srgb;

        // Build the Palette by KEY-MATCHING each CSS var to a ColorSlot (never a
        // format sniff): strip the `--` custom-property prefix and match against
        // `ColorSlot::key()`. Unknown keys (semantic aliases) and bad hexes skip.
        let mut palette = Palette::default();
        for (key, hex) in slot_keys.iter().zip(hexes.iter()) {
            let stripped = key.strip_prefix("--").unwrap_or(key);
            if let Some(slot) = ColorSlot::variants()
                .into_iter()
                .find(|s| s.key() == stripped)
                && let Some(rgb) = Rgb::from_hex(hex)
            {
                palette.insert(slot, rgb);
            }
        }

        // Each palette axiom's name + citation come straight from its own
        // `axiom_meta!` — never restated here — with its live verdict.
        let summarize = |axiom: &dyn Axiom| -> SchemaValue {
            let mut r = Presentation::new();
            r.set(
                "axiom",
                SchemaValue::Text(axiom.name().as_str().to_string()),
            );
            r.set(
                "citation",
                SchemaValue::Text(axiom.citation().as_str().to_string()),
            );
            r.set("pass", SchemaValue::Boolean(axiom.verify().is_ok()));
            SchemaValue::Record(r)
        };

        let mut checks: Vec<SchemaValue> = vec![
            summarize(&WcagForegroundContrast {
                palette: palette.clone(),
            }),
            summarize(&RenderedPairsMeetAa {
                palette: palette.clone(),
            }),
        ];

        // Per-pair detail: the numeric contrast ratio for each rendered pair
        // both of whose slots are present, tagged with the RenderedPairsMeetAa
        // axiom that governs the full pair set. `pass` is the axiom's OWN typed
        // Quantity comparison, so a row's verdict cannot drift from the axiom.
        let governing = RenderedPairsMeetAa {
            palette: palette.clone(),
        };
        let axiom_name = governing.name().as_str().to_string();
        let axiom_citation = governing.citation().as_str().to_string();
        for pair in rendered_pairs() {
            if let (Some(fg), Some(bg)) =
                (palette.get(&pair.foreground), palette.get(&pair.background))
            {
                let ratio = srgb::contrast_ratio(fg, bg);
                let required = pair.demand.min_ratio(srgb::WcagLevel::AA);
                let mut r = Presentation::new();
                r.set("axiom", SchemaValue::Text(axiom_name.clone()));
                r.set("citation", SchemaValue::Text(axiom_citation.clone()));
                r.set(
                    "pair",
                    SchemaValue::Text(format!(
                        "{} on {}",
                        pair.foreground.key(),
                        pair.background.key()
                    )),
                );
                r.set("ratio", SchemaValue::Float(ratio.value));
                r.set("required", SchemaValue::Float(required.value));
                r.set("pass", SchemaValue::Boolean(ratio >= required));
                checks.push(SchemaValue::Record(r));
            }
        }

        let mut p = Presentation::new();
        if let Some(polarity) = detect_polarity(&palette) {
            p.set("polarity", SchemaValue::Text(polarity.as_str().to_string()));
        }
        p.set("checks", SchemaValue::List(checks));
        p.to_json()
    }

    pub fn concept_count(&self) -> usize {
        self.english.concept_count().value as usize
    }

    pub fn word_count(&self) -> usize {
        self.english.word_count().value as usize
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

    /// THE load path — the ONE typed entry every source now takes (doc §3). One
    /// message shape, one dispatch: `LoadRequest::from_wire` decodes the
    /// boundary fields into a typed `LoadRequest` (the single tagged decode at
    /// the FFI), then `load_core` resolves the `(decoder, projection
    /// functor)` for the `Encoding` by typed match, verifies the
    /// `TrustAnchor` fail-closed, decodes → projects to a [`RuntimeOntology`] →
    /// installs it through the shared grounding + reasoner-rebuild tail
    /// (`install_runtime_ontology`). Returns a small `Loaded` receipt
    /// (`{ name, encoding, bytes, root }`) so the UI can name what it loaded —
    /// subsuming the old per-format `load_source` / `load_prx` / `load_owl_source`
    /// / `load_ontology_prx` / `load_embedded_demo_prx` methods.
    ///
    /// - `encoding` is the wire tag for the typed `Encoding`: `"uslm-title"`,
    ///   `"owl-source"`, `"owl-prx-gz"`, or `"rkyv-archive"`.
    /// - `version` pins the three-lock lookup for `"owl-prx-gz"` (ignored else).
    /// - `root_hex` supplies the trusted Merkle root for `"rkyv-archive"`
    ///   (ignored else).
    /// - `payload` is the fetched bytes (text formats travel as UTF-8 bytes);
    ///   `None` resolves the build-baked embedded bytes for `name` (the demo/base).
    pub fn load(
        &mut self,
        name: String,
        encoding: &str,
        version: Option<String>,
        root_hex: Option<String>,
        payload: Option<Vec<u8>>,
    ) -> Result<JsValue, JsValue> {
        let request = LoadRequest::from_wire(name, encoding, version, root_hex, payload)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let loaded = self
            .load_core(request)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(JsValue::from_str(&loaded.to_json()))
    }

    /// The embedded demo `.prx` descriptor the UI offers as a one-click load:
    /// `{ name, root, bytes }`. `root` is the trusted Merkle-root pin the
    /// fail-closed load checks against; `bytes` is the size of the embedded
    /// content-addressed archive (no network — it ships in the wasm).
    pub fn embedded_demo_prx(&self) -> String {
        let demo = embedded_demo();
        let mut p = Presentation::new();
        p.set("name", SchemaValue::Text(demo.name.into()));
        p.set("root", SchemaValue::Text(demo.root_hex.into()));
        p.set("bytes", SchemaValue::Unsigned(demo.bytes.len() as u64));
        p.set(
            "loaded",
            SchemaValue::Boolean(
                self.runtime_ontologies
                    .iter()
                    .any(|o| o.id().as_str() == demo.name),
            ),
        );
        p.to_json()
    }

    /// How many new-format `.prx` ontologies are currently loaded into the chat
    /// (0 until one is loaded). The dashboard reflects this.
    pub fn loaded_ontology_count(&self) -> usize {
        self.runtime_ontologies.len()
    }

    /// The sources this deployment fetches at startup, as `{names:[…]}`.
    ///
    /// The page loads what this returns, looking each name up in the catalogs
    /// it already holds to learn the route — so the demonstrator's residency
    /// policy is declared once, in typed Rust with its reasoning attached, and
    /// no source name is ever written into the page's JavaScript. Reading a
    /// name here that the catalogs do not carry is a build error waiting to
    /// happen, which is why `eager_residency_names_are_all_loadable` gates it.
    pub fn eager_resident(&self) -> String {
        let names: Vec<SchemaValue> = eager_residency::EAGER_RESIDENT
            .iter()
            .map(|n| SchemaValue::Text((*n).to_string()))
            .collect();
        let mut p = Presentation::new();
        p.set("names", SchemaValue::List(names));
        p.to_json()
    }

    /// The ontologies the reasoning pipeline itself runs through:
    /// `{ steps: [{ ontology, operation, phase }] }`.
    ///
    /// These are a DIFFERENT population from [`Self::self_describe`]'s loaded
    /// set, and the distinction is why they were invisible. A loaded ontology
    /// is a vocabulary fetched at runtime (a USC title, an OWL file, a
    /// `.prx`); these are compile-time ontologies the pipeline is built
    /// against — Lemon for tokenisation, Lambek for the chart parse, Montague
    /// for interpretation, and so on. The panel listed only the former, so a
    /// reader saw no compositional-semantics stack at all and could
    /// reasonably conclude there was none.
    ///
    /// Read from [`PipelineStep::ALL`](pr4xis_domains::formal::information::diagnostics::trace_functors::PipelineStep::ALL), the same constants the trace emits, so
    /// this cannot drift from what a turn reports. A CI guard already holds
    /// every one of those names to a currently-registered ontology.
    /// The largest transfer this deployment will start on the reader's behalf
    /// without being asked, in bytes: `{ bytes, rule, reference }`.
    ///
    /// A host that routes an abstention to "load the source that would answer
    /// this" is about to spend someone's data. Below this figure it may simply
    /// do it; at or above it, it must present the control and let the reader
    /// decide. The demonstrator's own catalog makes the difference stark —
    /// several titles land in a second, and Title 42 is about 35 MB.
    ///
    /// **Computed, not chosen.** The page previously declared 5 MiB with a
    /// comment explaining which gap in the shipped catalog it sat in. That
    /// derivation was correct and completely inert: nothing recomputed it, so
    /// staging one 5.5 MB title would have left the number in place and the
    /// explanation false. Here the figure is the two-class natural break over
    /// the sizes this build actually stages ([`two_class_break`], Jenks 1967) —
    /// it moves with the catalog rather than sitting there as a constant whose
    /// justification has quietly expired.
    ///
    /// The wire carries the FIGURE only. It also shipped a prose `rule` and a
    /// `reference`, on the reasoning that a reader could then check the
    /// argument — but nothing read either field, and a bibliography maintained
    /// in two places drifts: that copy still described the widest-gap rule
    /// after [`two_class_break`] had been corrected to Jenks's actual
    /// objective, so the wire was describing an algorithm the engine no longer
    /// ran. The citation lives with the algorithm, in `natural_breaks`, which
    /// is the one place it can be wrong in.
    ///
    /// `bytes` is absent when the catalog offers no real division (fewer than
    /// two distinct sizes), and the host then asks about every load — the
    /// conservative reading of "the data does not support a boundary".
    pub fn auto_load_budget(&self) -> String {
        // ONE size per SOURCE — the bytes a reader would actually spend.
        //
        // This chained both manifests, so every title with a pre-projected
        // archive contributed TWICE: once at its `.rprx` size and again at its
        // raw XML size. The two representations of Title 42 are 34.86 MiB and
        // 107.79 MiB, and that pair is by far the widest separation in the
        // combined list — so the break landed at 107.79 MiB, every real source
        // fell below it, and the page auto-started every download including the
        // one this budget exists to stop. A population that counts one thing
        // twice is not the population the decision is about.
        //
        // The archive is preferred because that is the route the host takes
        // when it exists (`goLoadSource` looks up `[data-load-fast]` before
        // `[data-load]`), so it is the cost actually incurred.
        let mut by_source: BTreeMap<&str, f64> = BTreeMap::new();
        for (name, _, _, bytes) in sources_manifest::AVAILABLE_SOURCES {
            if *bytes > 0 {
                by_source.insert(name, *bytes as f64);
            }
        }
        for (name, _, _, bytes, _) in usc_archives_manifest::AVAILABLE_USC_ARCHIVES {
            if *bytes > 0 {
                by_source.insert(name, *bytes as f64);
            }
        }
        let sizes: Vec<f64> = by_source.into_values().collect();
        let mut p = Presentation::new();
        p.set(
            "bytes",
            match two_class_break(&sizes) {
                Some(b) => SchemaValue::Unsigned(b as u64),
                None => SchemaValue::Absent,
            },
        );
        p.to_json()
    }

    pub fn pipeline_ontologies(&self) -> String {
        use pr4xis_domains::formal::information::diagnostics::trace_functors::PipelineStep;
        let steps: Vec<SchemaValue> = PipelineStep::ALL
            .iter()
            .map(|s| {
                let mut r = Presentation::new();
                r.set("ontology", SchemaValue::Text(s.ontology_name().into()));
                r.set("operation", SchemaValue::Text(s.operation_name().into()));
                r.set("phase", SchemaValue::Text(format!("{:?}", s.phase())));
                SchemaValue::Record(r)
            })
            .collect();
        let mut p = Presentation::new();
        p.set("steps", SchemaValue::List(steps));
        p.to_json()
    }

    /// Unload one runtime ontology by its id, the inverse of [`Self::load`].
    ///
    /// Loading is a deliberate act the reader takes, so undoing it must be one
    /// too: a reader who loaded a 36 MB title to answer one citation should be
    /// able to put it back down. Returns `true` when an ontology was removed,
    /// `false` when no loaded ontology carries that id.
    ///
    /// The derived state is rebuilt exactly as a load rebuilds it — the
    /// grounding pass re-runs over the REMAINING set, so cross-ontology edges
    /// minted against the departing ontology are re-derived rather than left
    /// dangling, and `composed` returns to `None` once nothing is loaded (the
    /// reasoner falls back to embedded English, abstaining on the concepts that
    /// just left, which is the honest outcome).
    ///
    /// The load history is deliberately NOT truncated: it is the append-only
    /// record of what this session did, and an unload is part of that record.
    pub fn unload(&mut self, id: &str) -> bool {
        // RESIDENCY DECIDES, and the ENGINE enforces it — not just the host.
        //
        // `Residency::is_releasable` is what the page reads to decide whether to
        // draw an Unload control, and until now that was the only thing standing
        // between a caller and a one-way door: this method would happily remove
        // a `Resident` base that nothing can re-acquire (there is no load act to
        // re-run, because the reader never performed one), and would return
        // `true` for the `Derived` title lexicon it re-derives on the very next
        // line. A model that governs a UI affordance but not the operation
        // underneath it is advisory, and a second host — or a direct call to
        // this exported method — would not be bound by it.
        if !residency_of(id).is_releasable() {
            return false;
        }
        let Some(pos) = self
            .runtime_ontologies
            .iter()
            .position(|o| o.id().as_str() == id)
        else {
            return false;
        };
        self.runtime_ontologies.remove(pos);
        // Re-derive BEFORE the emptiness check: unloading the last title must
        // also take its definitional entry with it, and a set holding only a
        // now-basisless derivative is not "non-empty" in any useful sense.
        self.refresh_usc_title_lexicon();
        if self.runtime_ontologies.is_empty() {
            self.composed = None;
            return true;
        }
        // Re-ground the remaining set. A failure here cannot be rolled back into
        // a valid prior state (the ontology is already gone), so fall back to
        // embedded-English-only rather than leave a half-grounded reasoner: the
        // engine then abstains where it would otherwise answer from a stale edge.
        match ground_loaded_set(&mut self.runtime_ontologies, english_static()) {
            Ok(()) => {
                self.composed = Some(ComposedReasoner::new(
                    english_static(),
                    self.runtime_ontologies.clone(),
                ));
            }
            Err(_) => {
                self.composed = None;
            }
        }
        true
    }

    /// The authoritative source documents available to download:
    /// `{ sources: [{ name, version, url, bytes }] }`. The host streams
    /// `url` (showing download progress), then calls [`Self::load`] with the
    /// `"uslm-title"` encoding and the fetched text as its payload. The meta
    /// page offers a Load action only for catalog sources that appear here.
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

    /// USC titles with a pre-projected, zero-copy-loadable `rkyv` archive
    /// (task #21): `{ archives: [{ name, version, url, bytes, root }] }`.
    /// The host streams `url` and calls [`Self::load`] with the
    /// `"rkyv-archive"` encoding and `root` as the trusted Merkle root:
    /// `materialize_bytes` re-derives the root from the fetched bytes and
    /// the load refuses on mismatch, so tampered or stale bytes are never
    /// silently admitted — no client-side USLM XML parse, no owned DAG-CBOR
    /// decode/re-encode pass either. A title absent here has no fast route
    /// yet; the host falls back to [`Self::available_sources`]'s
    /// `"uslm-title"` raw-XML path, unaffected by this catalog.
    pub fn available_usc_archives(&self) -> String {
        let list: Vec<SchemaValue> = usc_archives_manifest::AVAILABLE_USC_ARCHIVES
            .iter()
            .map(|(name, version, url, bytes, root)| {
                let mut r = Presentation::new();
                r.set("name", SchemaValue::Text((*name).into()));
                r.set("version", SchemaValue::Text((*version).into()));
                r.set("url", SchemaValue::Text((*url).into()));
                r.set("bytes", SchemaValue::Unsigned(*bytes));
                r.set("root", SchemaValue::Text((*root).into()));
                SchemaValue::Record(r)
            })
            .collect();
        let mut p = Presentation::new();
        p.set("archives", SchemaValue::List(list));
        p.to_json()
    }

    /// The registered OWL vocabularies available to load, each by either
    /// route: `{ ontologies: [{ name, version, prx_url, source_url }] }`.
    /// The host streams `prx_url` and calls [`Self::load`] with the
    /// `"owl-prx-gz"` encoding (validated against the embedded lock pin) OR
    /// streams `source_url` and calls [`Self::load`] with the `"owl-source"`
    /// encoding. The embedded lock pin is not exposed here — it is a
    /// build-time validation secret consumed by the `"owl-prx-gz"` path of
    /// `load`, not a URL the host fetches.
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
            .map(|o| runtime_ontology_vocabulary(o))
            .collect();
        // Per-ontology capabilities (doc §4.7) — what each loaded ontology can
        // answer (gloss / populated relation kinds), so "loaded" stops lying.
        let capabilities = self
            .runtime_ontologies
            .iter()
            .map(|o| ontology_capabilities(o))
            .collect();
        pr4xis_chat::self_describe_with_loaded(self.english, loaded)
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
            Residency::Resident,
            self.english.concept_count().value as usize,
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
            let name = onto.id().as_str();
            refs.push(LoadedRef::new(
                name.to_string(),
                staging_of(name),
                residency_of(name),
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
    /// The composed reasoner BORROWS the single embedded [`English`]
    /// ([`english_static`], the same instance `Pr4xis` holds), so the loaded
    /// ontologies are grounded into a complete English lexicon via the Lemon
    /// functor without rebuilding or re-owning the ~73 MiB model. This rebuild of
    /// the grounding happens only on a load (a rare, deliberate action), keeping
    /// the per-chat path a cheap branch.
    fn install_runtime_ontology(
        &mut self,
        onto: RuntimeOntology,
    ) -> Result<(), pr4xis_runtime::grounding::LinkError> {
        // Fail-closed rollback snapshot: if grounding the new set surfaces a LOUD
        // fault (a declared target NAME absent from a present peer, a multi-level
        // chain, a skew), NOTHING installs — the prior good set + `composed` stand.
        // Cloning a `Vec<Rc<_>>` bumps refcounts, it does NOT deep-copy archives.
        let prior_ontologies = self.runtime_ontologies.clone();
        let prior_history_len = self.history.len();

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

        self.runtime_ontologies.push(Rc::new(onto));
        // The derived title lexicon is a function of the loaded set, so it is
        // recomputed HERE — inside the same transaction, before the grounding
        // pass that the reasoner is built from, and inside the same rollback
        // (`prior_ontologies` was snapshotted above). Going back through
        // `install_runtime_ontology` would recurse, and would also record a
        // LoadEvent for something no one loaded.
        self.refresh_usc_title_lexicon();
        // GROUNDING PASS: the SINGLE, order-independent grounding authority — mint
        // every loaded ontology's declared cross-ontology type edges against the
        // current loaded set (a USC title into `LegalSources`, an into-English
        // `.prx` onto WordNet synsets — the seeded English target peer), driven by
        // the functor each carries as data. A source whose base has not yet loaded
        // DEFERS (re-grounds on the base's arrival); a declared-but-unrealizable
        // grounding is LOUD and rolls the install back. Idempotent.
        match ground_loaded_set(&mut self.runtime_ontologies, english_static()) {
            Ok(()) => {
                // The reasoner BORROWS the single embedded English and reasons over
                // the SAME loaded ontologies.
                self.composed = Some(ComposedReasoner::new(
                    english_static(),
                    self.runtime_ontologies.clone(),
                ));
                Ok(())
            }
            Err(fault) => {
                // Roll back to the prior good set — install nothing, `composed`
                // untouched.
                self.runtime_ontologies = prior_ontologies;
                self.history.truncate(prior_history_len);
                Err(fault)
            }
        }
    }

    /// Recompute the derived U.S. Code title lexicon against the CURRENT loaded
    /// set, replacing (or removing) whatever the previous derivation left.
    ///
    /// A title arrives as 17,713 sections and no name of its own: nothing in
    /// the archive is the concept "title 42", so "what is title 42" resolves
    /// against no English atom and the reader is told the engine knows only a
    /// number. This contributes one definitional entry per HELD title —
    /// surfaces from the title's own published citation forms, gloss from its
    /// own registered description — through the same WN-LMF bridge the
    /// caregiving lexicon rides.
    ///
    /// Idempotent and total: it is a pure function of which title-named
    /// ontologies are loaded, so calling it twice changes nothing and calling
    /// it with no title loaded removes the previous derivation. It deliberately
    /// does NOT go through `install_runtime_ontology` — that would recurse, and
    /// would file a load event for a source nobody loaded.
    ///
    /// A materialization failure leaves the previous derivation in place rather
    /// than a stale-but-plausible one: it is removed first, so the worst case
    /// is that titles are unnamed again, never that a title is named with the
    /// wrong gloss.
    fn refresh_usc_title_lexicon(&mut self) {
        self.runtime_ontologies
            .retain(|o| o.id().as_str() != USC_TITLE_LEXICON_NAME);
        // Held is read from the CONTENT, not from what a caller named the
        // ontology. Deriving it from source names made the engine's answer
        // about a title a claim about a LABEL: load Title 18's USLM under the
        // name `usc_title_5` and it would recite Title 5's registered gloss —
        // a confident definition of a document it does not hold. Every USC node
        // carries its own USLM URN, so the title actually present is derivable,
        // which is what the CLI already did.
        let held = titles_held_in(self.runtime_ontologies.iter().map(|o| o.as_ref()));
        if let Some(Ok(onto)) = usc_title_lexicon(&held) {
            self.runtime_ontologies.push(Rc::new(onto));
        }
    }

    /// THE single load core (doc §3) — the plain-Rust path behind the public
    /// [`Self::load`] and every native-test load helper. One structural arm:
    /// resolve `(decoder, functor)` for the request's `Encoding` and verify its
    /// `TrustAnchor` ([`decode_and_project`]), then install through the shared
    /// grounding + reasoner-rebuild tail (`install_runtime_ontology`).
    /// Per-encoding knowledge is the typed `Encoding` variant; per-trust
    /// knowledge is the `TrustAnchor` variant — neither is a method. Returns the
    /// `Loaded` receipt. Fail-closed and transactional: a decode/verify refusal
    /// or a LOUD grounding fault installs nothing.
    fn load_core(&mut self, request: LoadRequest) -> Result<Loaded, LoadError> {
        let onto = decode_and_project(
            &request.name,
            request.encoding,
            &request.trust,
            &request.payload,
        )?;
        let root = onto.root().to_hex();
        let bytes = request.payload.len();
        self.install_runtime_ontology(onto)
            .map_err(LoadError::Grounding)?;
        Ok(Loaded {
            name: request.name,
            encoding: request.encoding,
            bytes,
            root,
        })
    }

    /// Native-test helper: load a USLM title from its XML through the ONE typed
    /// `load_core` path (a `LoadRequest` carrying [`Encoding::UslmTitle`]
    /// and transport trust). Same typed verdict as the public [`Self::load`], no
    /// `JsValue` — so the load path is exercisable under `cargo test`. Gated to
    /// the native test build (the browser suite drives the public `load` instead).
    #[cfg(all(test, not(target_arch = "wasm32")))]
    fn load_source_core(&mut self, name: String, xml: &str) -> Result<(), LoadError> {
        self.load_core(LoadRequest {
            name,
            encoding: Encoding::UslmTitle,
            payload: xml.as_bytes().to_vec(),
            trust: TrustAnchor::Transport,
        })
        .map(|_| ())
    }

    /// Load a build-baked or fetched `rkyv` `.prx` from its bytes + trusted
    /// root through the ONE typed `load_core` path (a `LoadRequest`
    /// carrying [`Encoding::RkyvArchive`] + [`TrustAnchor::MerkleRoot`]). This
    /// IS the browser load path (the SAME method `Pr4xis::new` calls to
    /// install every `default_loaded` base); the native tests reuse it for
    /// its typed verdict instead of a `JsValue`.
    fn load_ontology_prx_core(
        &mut self,
        bytes: &[u8],
        name: String,
        expected_root_hex: &str,
    ) -> Result<(), LoadError> {
        let root = ContentAddress::from_hex(expected_root_hex)
            .ok_or_else(|| LoadError::BadRootHex(expected_root_hex.to_string()))?;
        self.load_core(LoadRequest {
            name,
            encoding: Encoding::RkyvArchive,
            payload: bytes.to_vec(),
            trust: TrustAnchor::MerkleRoot(root),
        })
        .map(|_| ())
    }

    /// Native-test helper: load the build-baked Dependability demo `.prx` through
    /// the same fail-closed `load_core` path and return its name — the
    /// native mirror of the public embedded load (`load(name, "rkyv-archive",
    /// None, None, None)`). Gated to the native test build.
    #[cfg(all(test, not(target_arch = "wasm32")))]
    fn load_embedded_demo_prx_core(&mut self) -> Result<String, LoadError> {
        let demo = embedded_demo();
        let root = ContentAddress::from_hex(demo.root_hex)
            .ok_or_else(|| LoadError::BadRootHex(demo.root_hex.to_string()))?;
        self.load_core(LoadRequest {
            name: demo.name.to_string(),
            encoding: Encoding::RkyvArchive,
            payload: demo.bytes.to_vec(),
            trust: TrustAnchor::MerkleRoot(root),
        })
        .map(|loaded| loaded.name)
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

    /// Every eagerly-resident name must actually be loadable by some route.
    ///
    /// The page fetches this list at startup and looks each name up in the
    /// catalogs to learn how to load it. A name the catalogs do not carry is
    /// therefore not an error anyone sees — the loop simply skips it, and the
    /// demonstrator boots without the corpus it was supposed to hold while
    /// reporting nothing wrong. That silence is the whole risk of declaring
    /// residency in one place and resolving it in another, so it is gated
    /// here: rename a source, or drop it from the staged set, and this fails
    /// at build time rather than degrading the page in production.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn eager_residency_names_are_all_loadable() {
        for name in eager_residency::EAGER_RESIDENT {
            let in_archives = usc_archives_manifest::AVAILABLE_USC_ARCHIVES
                .iter()
                .any(|(n, ..)| n == name);
            let in_ontologies = ontologies_manifest::AVAILABLE_ONTOLOGIES
                .iter()
                .any(|(n, ..)| n == name);
            let in_sources = sources_manifest::AVAILABLE_SOURCES
                .iter()
                .any(|(n, ..)| n == name);
            assert!(
                in_archives || in_ontologies || in_sources,
                "eager-resident {name:?} is in no catalog, so the page would \
                 silently skip it and boot without the source it declares"
            );
        }
    }

    /// **Every eagerly-resident source must be releasable, and re-loadable.**
    ///
    /// Declaring a source eager takes its Load button off the page — the page
    /// offers one only on a card that is not loaded. If that source were also
    /// unreleasable, the deployment would have made a decision on the reader's
    /// behalf that the reader cannot reverse, and it would simultaneously
    /// retire whatever load path the source was the last user of. That is not
    /// hypothetical: all six published OWL vocabularies are eager, so the
    /// hash-validated `.prx.gz` envelope leg is reachable ONLY by releasing one
    /// first. The end-to-end suite drives exactly that round-trip, and this is
    /// the invariant that keeps it drivable.
    #[test]
    fn every_eager_source_can_be_put_down_and_picked_back_up() {
        for name in eager_residency::EAGER_RESIDENT {
            assert!(
                residency_of(name).is_releasable(),
                "eager-resident {name:?} is not releasable, so declaring it \
                 eager silently removed the reader's only say over it"
            );
            assert!(
                has_load_route(name),
                "eager-resident {name:?} has no published load route, so once \
                 released it could never be loaded again"
            );
        }
    }

    /// The auto-load budget is the SHIPPED CATALOG's own division, not a number
    /// anyone picked — and it separates the titles that land in a moment from
    /// the ones that cost a reader real data.
    ///
    /// The figure it replaced was a literal in the page whose comment derived it
    /// from exactly this gap. The derivation was sound and inert: nothing
    /// recomputed it, so staging one title inside that gap would have left the
    /// number in place and its justification false. This asserts the property
    /// the literal only claimed — that the boundary actually falls between the
    /// small titles and the large ones for the catalog as built.
    #[test]
    fn the_auto_load_budget_is_the_catalogs_own_natural_break() {
        let p = Pr4xis::new();
        let v: serde_json::Value =
            serde_json::from_str(&p.auto_load_budget()).expect("auto_load_budget returns JSON");
        let Some(budget) = v["bytes"].as_u64() else {
            // Fewer than two distinct sizes staged: no division exists, and the
            // host correctly asks about every load. Nothing further to assert.
            return;
        };

        // ONE size per source, matching what `auto_load_budget` classifies. The
        // earlier version chained both manifests here too, so the gate scored
        // the budget against a population that counted every archived title
        // twice — the same mistake it existed to catch.
        let mut by_source: BTreeMap<&str, u64> = BTreeMap::new();
        for (name, _, _, bytes) in sources_manifest::AVAILABLE_SOURCES {
            if *bytes > 0 {
                by_source.insert(name, *bytes);
            }
        }
        for (name, _, _, bytes, _) in usc_archives_manifest::AVAILABLE_USC_ARCHIVES {
            if *bytes > 0 {
                by_source.insert(name, *bytes);
            }
        }
        let sizes: Vec<u64> = by_source.values().copied().collect();
        let (below, at_or_above): (Vec<u64>, Vec<u64>) = sizes.iter().partition(|b| **b < budget);
        assert!(
            !below.is_empty() && !at_or_above.is_empty(),
            "the budget must actually divide the catalog — {budget} leaves one \
             side empty over {sizes:?}"
        );

        // THE PROPERTY, stated as Jenks states it: the chosen split must have
        // the LOWEST summed within-class squared deviation of any split.
        //
        // The previous form compared the gap across the boundary to the widest
        // gap within each class — and passed vacuously whenever a class held a
        // single element, because a one-element class has no internal gap. That
        // is exactly the shape the broken budget produced (upper class = one
        // outlier), so the gate certified the bug it was written to catch.
        // Scoring the objective over ALL candidate splits cannot go vacuous.
        let sdcm = |split: u64| -> f64 {
            let (lo, hi): (Vec<u64>, Vec<u64>) = sizes.iter().partition(|b| **b < split);
            [lo, hi]
                .iter()
                .filter(|cls| !cls.is_empty())
                .map(|cls| {
                    let m = cls.iter().sum::<u64>() as f64 / cls.len() as f64;
                    cls.iter().map(|v| (*v as f64 - m).powi(2)).sum::<f64>()
                })
                .sum()
        };
        let chosen = sdcm(budget);
        for &candidate in &sizes {
            if candidate == *sizes.iter().min().expect("non-empty") {
                continue; // not a split: it would leave the lower class empty
            }
            assert!(
                chosen <= sdcm(candidate) + 1.0,
                "budget {budget} (within-class deviation {chosen:.0}) is not \
                 optimal — splitting at {candidate} gives {:.0} over {sizes:?}",
                sdcm(candidate)
            );
        }
    }

    /// Is there any published route by which a host could load `name`?
    ///
    /// The three manifests are the only routes that exist — the page builds
    /// every Load button from them — so a name in none of them is unloadable
    /// however it appears in the catalog.
    fn has_load_route(name: &str) -> bool {
        usc_archives_manifest::AVAILABLE_USC_ARCHIVES
            .iter()
            .any(|(n, ..)| *n == name)
            || ontologies_manifest::AVAILABLE_ONTOLOGIES
                .iter()
                .any(|(n, ..)| *n == name)
            || sources_manifest::AVAILABLE_SOURCES
                .iter()
                .any(|(n, ..)| *n == name)
    }

    /// **Nothing may render as "available" that cannot then be loaded.**
    ///
    /// `us_legal_lexicon` shipped exactly this way: a card reading "available",
    /// carrying no Load button, because the catalog admitted it as chat
    /// knowledge while no manifest published a route to it. To a reader that is
    /// worse than an honest absence — the page names a source, says the engine
    /// could hold it, and offers no way to say yes.
    ///
    /// The catalog here is the very one `self_describe` serialises, so this
    /// gates what the page actually renders rather than a restatement of it.
    /// A source with no decoder belongs to a non-`ChatKnowledge` role (which is
    /// what `is_chat_loadable` filters on, and how the original was fixed), not
    /// to the knowledge boundary with a dead card.
    #[test]
    fn nothing_offers_itself_as_available_without_a_way_to_load_it() {
        let p = Pr4xis::new();
        let stranded: Vec<String> = source_catalog(&p.loaded_refs())
            .into_iter()
            .filter(|s| !s.availability.is_loaded())
            .filter(|s| !has_load_route(&s.name))
            .map(|s| s.name)
            .collect();
        assert!(
            stranded.is_empty(),
            "these sources render as `available` with no published load route, \
             so their cards carry no Load button and a reader cannot act on \
             them: {stranded:?} — either publish a route, or give the source \
             the role that reflects it is not chat-loadable"
        );
    }

    /// The loaded-ontology count of a freshly-constructed `Pr4xis`: every
    /// `default_loaded` entry in the embedded manifest (LegalSources, and the
    /// Caregiver AI Challenge chat lexicons), installed in `Pr4xis::new` before
    /// any demo / USC / OWL load. Derived from the manifest, not hardcoded, so
    /// adding a future `default_loaded` base never silently breaks this count.
    /// The demo tests assert RELATIVE to this base, so they test the state
    /// CHANGE a load makes — not an absolute empty start.
    fn base_loaded() -> usize {
        embedded_base().count()
    }
    /// The load-history length of a freshly-constructed `Pr4xis`: one event per
    /// `default_loaded` base recorded at construction — same cardinality as
    /// [`base_loaded`].
    fn base_history() -> usize {
        base_loaded()
    }

    /// Materialize the embedded demo `.prx` straight from its bytes (the same
    /// bytes `Pr4xis` loads) so the test can read its glosses and pick a demo
    /// concept — without reaching into `Pr4xis`'s private state.
    fn embedded_ontology() -> RuntimeOntology {
        let demo = embedded_demo();
        let root = ContentAddress::from_hex(demo.root_hex).unwrap();
        let mut buf = AlignedVec::<16>::with_capacity(demo.bytes.len());
        buf.extend_from_slice(demo.bytes);
        let ontology = materialize_bytes(buf, OntologyName::new(demo.name))
            .expect("embedded demo .prx materializes");
        assert_eq!(
            ontology.root(),
            root,
            "embedded demo .prx must re-derive its baked root"
        );
        ontology
    }

    /// The name of the FIRST always-loaded base, read from the manifest so the
    /// test never restates the string. Several bases are `default_loaded`; the
    /// tests that use this need one real resident name, not all of them.
    fn base_name() -> &'static str {
        embedded_base()
            .next()
            .expect("a default-loaded embedded base")
            .name
    }

    /// A demo concept the embedded ontology DEFINES (carries a gloss for) whose
    /// lowercased surface FULL WordNet does NOT know — so "what is a <concept>"
    /// genuinely abstains without the corpus, and answers from the loaded gloss
    /// with it. Returns `(surface, gloss)`. Discovered, not hardcoded: we scan
    /// the loaded nodes against a fresh English model.
    fn demo_concept(english: &English) -> (String, String) {
        let onto = embedded_ontology();
        for node in onto.archive().nodes.iter() {
            let surface = node.name.to_lowercase();
            let cref = onto.concept(node.name.to_string());
            if let Some(gloss) = onto.lexical(&cref)
                && english.lookup(&surface).is_empty()
            {
                return (surface, gloss.to_string());
            }
        }
        panic!("expected at least one glossed embedded concept unknown to WordNet");
    }

    /// TEETH for the embedded-English content gate: the baked `.stores.gz`
    /// loads ONLY against the committed `[store_bundle_signatures]` pin. Both
    /// legs refuse loudly — the TRUE bytes against a WRONG pin (the pin leg),
    /// and TAMPERED bytes against the true pin (the content leg) — mirroring
    /// the native `english_load_owned()` store-bundle gate's fail-closed
    /// contract.
    #[test]
    fn tampered_embedded_english_refuses_loudly() {
        use pr4xis_domains::applied::data_provisioning::registry::{
            LockDigest, data_sources, lock_store_bundle_signature,
        };
        use pr4xis_domains::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;

        let entry = data_sources()
            .iter()
            .find(|e| e.kind == SourceTaxonomyConcept::Language)
            .expect("the one registered Language-kind source");
        let key = format!("{}@{}", entry.name, entry.version);
        let pin = lock_store_bundle_signature(&entry.name, &entry.version)
            .expect("english carries a [store_bundle_signatures] pin");

        // Positive control: the baked bytes verify against the committed pin.
        assert!(
            load_english_store_bundle_gz_gated(ENGLISH_STORES_GZ, pin, &key).is_ok(),
            "the embedded english.stores.gz must load through the committed pin"
        );

        // Pin leg: the true bytes against a WRONG pin refuse.
        let wrong = LockDigest::address("0".repeat(64));
        assert!(
            load_english_store_bundle_gz_gated(ENGLISH_STORES_GZ, &wrong, &key).is_err(),
            "a wrong pin must refuse the embedded english.stores.gz"
        );

        // Content leg: tampered bytes against the TRUE pin refuse (a corrupted
        // stream fails the gunzip/decode, an intact-but-altered one fails the
        // content-address check — refusal either way, never a silent install).
        let mut tampered = ENGLISH_STORES_GZ.to_vec();
        let mid = tampered.len() / 2;
        tampered[mid] ^= 0xff;
        assert!(
            load_english_store_bundle_gz_gated(&tampered, pin, &key).is_err(),
            "tampered english.stores.gz bytes must refuse loudly"
        );
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
        let mut without = Pr4xis::new();
        assert_eq!(
            without.loaded_ontology_count(),
            base_loaded(),
            "a fresh Pr4xis carries only the always-loaded default_loaded bases (LegalSources + the chat lexicons)"
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
        //     typed core; the wasm `load` method with the `"rkyv-archive"`
        //     encoding is a thin wrapper over exactly this call.) ---
        let mut with = Pr4xis::new();
        let loaded_name = with
            .load_embedded_demo_prx_core()
            .expect("the embedded demo .prx loads (fail-closed root matches)");
        assert_eq!(loaded_name, embedded_demo().name);
        assert_eq!(
            with.loaded_ontology_count(),
            base_loaded() + 1,
            "the demo load adds one ontology on top of the default_loaded bases"
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
            base_history() + 1,
            "the demo load is recorded on top of the base's load event"
        );
        assert_eq!(
            with_d["history"][base_history()]["event"].as_str(),
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
            base_history(),
            "only the base load is recorded before the demo load"
        );

        // §3: the embedded demo `.prx` is loaded but NOT a registered source — it
        // must still appear in the catalog's sources panel (by its OntologyName,
        // tagged loaded), not be silently dropped.
        let sources = with_d["sources"].as_array().expect("sources array");
        let demo = sources
            .iter()
            .find(|s| s["name"].as_str() == Some(embedded_demo().name))
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
                    .any(|s| s["name"].as_str() == Some(embedded_demo().name)))
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
            base_loaded() + 1,
            "re-loading a name replaces it — not two demo copies atop the base"
        );

        let d = serde_json::from_str::<serde_json::Value>(&p.self_describe()).expect("JSON");
        let history = d["history"].as_array().expect("history is an array");
        assert_eq!(
            history.len(),
            base_history() + 2,
            "both demo loads are recorded (append-only) after the base load"
        );
        assert_eq!(history[base_history()]["event"].as_str(), Some("load"));
        assert_eq!(
            history[base_history() + 1]["event"].as_str(),
            Some("replace"),
            "the second load of the same name is a replace"
        );
        assert!(
            history[base_history() + 1]["displaced"].as_str().is_some(),
            "a replace event carries the displaced root"
        );
    }

    /// A minimal USLM Title (Title 18 §1, heading "First section") — the fixture
    /// for the load-a-statute-then-query-it acceptance tests, shared with the
    /// registered `LoadEnvelopeFailClosed` axiom.
    use crate::load_envelope::SAMPLE_USLM_TITLE;

    #[test]
    fn loading_a_usc_title_routes_it_into_the_reasoner() {
        // Architecture Step 1, the WIRE: a loaded USLM statute must reach the SAME
        // reasoning set `chat()` composes over (`runtime_ontologies`), not a
        // separate `self.loaded` corpus the reasoner never sees. Before the load
        // the reasoner is empty; after, it holds the one projected statute
        // ontology. This is the structural half of "load Title 15 → ask about it".
        let mut p = Pr4xis::new();
        assert_eq!(p.loaded_ontology_count(), base_loaded());
        p.load_source_core("Title 18 (test)".to_string(), SAMPLE_USLM_TITLE)
            .expect("a well-formed USLM title loads");
        // +2, and the second one is the point: the statute itself, PLUS the
        // derived title index that now recognises it.
        //
        // The fixture loads under the name "Title 18 (test)", which is not a
        // `usc_title_N` registry key — so while held-ness was read from NAMES,
        // this load contributed no index entry and the count was +1. Held-ness
        // is now read from the loaded nodes' own USLM URNs, so `/us/usc/t18/…`
        // is recognised as Title 18 whatever the caller called it. The number
        // moved because the engine stopped trusting a label, which is the fix,
        // not drift to be papered over.
        assert_eq!(
            p.loaded_ontology_count(),
            base_loaded() + 2,
            "a loaded statute must become a RuntimeOntology the chat reasons \
             over, and its title index must be derived from the content"
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
        p.load_source_core("Title 18 (test)".to_string(), SAMPLE_USLM_TITLE)
            .expect("a well-formed USLM title loads");
        let resp = response_of(&p.chat("what is section 1")).to_lowercase();
        assert!(
            resp.contains("first section"),
            "after loading the statute, the chat must answer about its section by its \
             citation surface; got: {resp:?}"
        );
    }

    /// A held title becomes answerable BY NAME, and stops being so when it is
    /// released — the derived title lexicon's whole contract in one test.
    ///
    /// A title arrives as sections and nothing that IS the title, so "what is
    /// title 18" used to reach only the bare numeral. The lexicon is derived
    /// from the loaded set, so the same act that makes the title held makes it
    /// nameable, and the same act that releases it takes the name away. The
    /// question is asked by the title's OWN published designation, obtained
    /// from the typed id — no citation text is written here.
    #[test]
    fn a_held_title_is_answerable_by_its_own_citation_and_stops_being_so_when_released() {
        use pr4xis_domains::social::software::markup::xml::uslm::corpus::identifiers::{
            UsCodeTitleCitationForm, UsCodeTitleId,
        };
        let id = UsCodeTitleId::try_from_number(18).expect("Title 18 is a valid title");
        // The bare designation — the phrasing that failed, and the one no
        // mechanical alias derives from the other two forms.
        let question = format!(
            "what is {}",
            id.citation(UsCodeTitleCitationForm::Designation)
        );
        let gloss_head =
            pr4xis_domains::applied::data_provisioning::registry::by_name(&id.source_name())
                .and_then(|e| e.description.clone())
                .expect("Title 18 is registered with a description to recite");

        let mut p = Pr4xis::new();
        assert!(
            !response_of(&p.chat(&question)).contains(&gloss_head),
            "precondition: with no title held, the engine must not recite a title gloss"
        );

        // Loaded under its REGISTRY name — that is how the browser installs a
        // title, and it is the only observation of holding the host has.
        p.load_source_core(id.source_name(), SAMPLE_USLM_TITLE)
            .expect("a well-formed USLM title loads");
        assert_eq!(
            p.loaded_ontology_count(),
            base_loaded() + 2,
            "the title AND its derived definitional entry are both in the reasoning set"
        );
        assert!(
            response_of(&p.chat(&question)).contains(&gloss_head),
            "a held title must answer to its own designation, from its own registered gloss"
        );

        // Releasing the basis releases the derivative: no orphan lexicon
        // naming a title the engine no longer holds.
        assert!(
            p.unload(&id.source_name()),
            "the loaded title is releasable"
        );
        assert_eq!(
            p.loaded_ontology_count(),
            base_loaded(),
            "unloading the title removes the entry derived from it too"
        );
        assert!(
            !response_of(&p.chat(&question)).contains(&gloss_head),
            "with the title released, the engine must stop naming it"
        );
    }

    /// The derived lexicon is not itself releasable, and the host is told so.
    /// It was obtained by no control act; an Unload that appeared to succeed
    /// while every title it names stayed loaded would misreport the system's
    /// own state, and the next load would silently bring it back.
    #[test]
    fn the_derived_title_lexicon_is_reported_as_derived_and_unreleasable() {
        use pr4xis_domains::social::software::markup::xml::uslm::corpus::identifiers::UsCodeTitleId;
        let id = UsCodeTitleId::try_from_number(18).expect("Title 18 is a valid title");
        let mut p = Pr4xis::new();
        p.load_source_core(id.source_name(), SAMPLE_USLM_TITLE)
            .expect("a well-formed USLM title loads");

        assert_eq!(residency_of(USC_TITLE_LEXICON_NAME), Residency::Derived);
        assert!(
            !residency_of(USC_TITLE_LEXICON_NAME).is_releasable(),
            "control never acquired the derivation, so no control releases it"
        );
        assert_eq!(
            staging_of(USC_TITLE_LEXICON_NAME),
            Staging::Composed,
            "it was composed in-process, not streamed — reporting `async` would \
             claim a download that never happened"
        );

        // The load history records what the reader DID. A derivation is not an
        // act the reader took, so it files no event.
        let described: serde_json::Value =
            serde_json::from_str(&p.self_describe()).expect("self_describe is JSON");
        let history = described["history"]
            .as_array()
            .expect("history is an array");
        assert!(
            history
                .iter()
                .all(|e| e["ontology"].as_str() != Some(USC_TITLE_LEXICON_NAME)),
            "the derived lexicon must not appear in the append-only record of loads"
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
            .load_ontology_prx_core(embedded_demo().bytes, "Dependability".into(), &wrong_root)
            .expect_err("a wrong trusted root must be refused (fail-closed)");
        // The typed verdict IS a root mismatch (not a decode error, not a
        // materialize error) — the gate re-derived the root and rejected it.
        assert!(
            matches!(err, LoadError::RkyvRootMismatch { .. }),
            "the refusal must be a typed root mismatch; got: {err:?}"
        );
        assert_eq!(
            p.loaded_ontology_count(),
            base_loaded(),
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
        let mut bytes = embedded_demo().bytes.to_vec();
        *bytes.last_mut().unwrap() ^= 0xff;
        let mut p = Pr4xis::new();
        let err = p
            .load_ontology_prx_core(&bytes, "Dependability".into(), embedded_demo().root_hex)
            .expect_err("tampered .prx bytes must be refused (fail-closed)");
        assert!(
            matches!(
                err,
                LoadError::Materialize(_) | LoadError::RkyvRootMismatch { .. }
            ),
            "tampered bytes must be refused by the gate (either bytecheck rejects the \
             corrupted buffer, or a corruption bytecheck admits still re-derives a \
             different root); got: {err:?}"
        );
        assert_eq!(
            p.loaded_ontology_count(),
            base_loaded(),
            "a refused .prx installs nothing beyond the always-loaded base"
        );
    }

    /// ∀-ARBITRARY-PAYLOAD TOTALITY, per `Encoding` arm: `decode_and_project`
    /// is TOTAL over hostile payload bytes on EVERY arm — a deterministic
    /// high-entropy corpus (the fixed-seed xorshift64* stream the raw-source
    /// store-if-smaller test uses), format-shaped prefixes that get past each
    /// arm's cheap front door (a USLM/OWL XML head, the RFC 1952 gzip magic),
    /// truncations and single-byte mutations of the REAL embedded demo `.prx`
    /// — every case is a typed `Err`, never a panic-through and never a silent
    /// `Ok`. The wasm-side port of `prop_mutated_prx_always_rejected`'s
    /// totality half (the native arms' unit tests cover the typed verdicts;
    /// this pins totality over arbitrary bytes).
    #[test]
    fn decode_and_project_is_total_over_arbitrary_payload_bytes_on_every_arm() {
        use pr4xis_domains::applied::data_provisioning::ontology::ContentType;
        use pr4xis_domains::applied::data_provisioning::registry::data_sources;

        // Deterministic pseudo-random bytes (xorshift64*, fixed seed).
        fn noise(mut x: u64, len: usize) -> Vec<u8> {
            core::iter::repeat_with(move || {
                x ^= x >> 12;
                x ^= x << 25;
                x ^= x >> 27;
                (x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 56) as u8
            })
            .take(len)
            .collect()
        }

        let demo = embedded_demo();
        let mut corpus: Vec<Vec<u8>> = vec![
            Vec::new(),
            noise(0x9E37_79B9_7F4A_7C15, 1),
            noise(0xDEAD_BEEF_CAFE_F00D, 64),
            noise(0x0123_4567_89AB_CDEF, 1024),
            // Format-shaped heads with garbage tails — past the front door.
            b"<title xmlns=\"http://xml.house.gov/schemas/uslm/1.0\">".to_vec(),
            b"<?xml version=\"1.0\"?><rdf:RDF xmlns:rdf=\"ns\">".to_vec(),
            alloc_gzip_head(),
            // Truncations of the real demo archive.
            demo.bytes[..demo.bytes.len() / 2].to_vec(),
            demo.bytes[..1].to_vec(),
        ];
        // Sampled single-byte mutations of the real demo archive.
        for i in (0..demo.bytes.len()).step_by(97) {
            let mut m = demo.bytes.to_vec();
            m[i] ^= 0x80;
            corpus.push(m);
        }
        fn alloc_gzip_head() -> Vec<u8> {
            // RFC 1952 magic + deflate method, then garbage.
            let mut v = vec![0x1f, 0x8b, 0x08, 0x00, 0, 0, 0, 0, 0, 0xff];
            v.extend_from_slice(&[0x55; 32]);
            v
        }

        // One (encoding, matching-anchor) pair per arm. The OwlPrxGz arm runs
        // BOTH the unpinned key (refused at the three-pin lookup) and — when
        // the registry carries a pinned OWL source — the pinned key, so the
        // gunzip/bytecheck gate itself sees the hostile bytes.
        let wrong_root = ContentAddress::of(b"totality probe root");
        let mut arms: Vec<(&str, Encoding, TrustAnchor)> = vec![
            ("t", Encoding::UslmTitle, TrustAnchor::Transport),
            ("t", Encoding::OwlSource, TrustAnchor::Transport),
            (
                "no-such-vocabulary",
                Encoding::OwlPrxGz,
                TrustAnchor::LockPinned {
                    version: "0.0.0".to_string(),
                },
            ),
            (
                "t",
                Encoding::RkyvArchive,
                TrustAnchor::MerkleRoot(wrong_root),
            ),
        ];
        if let Some(entry) = data_sources().iter().find(|e| {
            e.content_type() == ContentType::Owl
                && lock_archive_signature(&e.name, &e.version).is_some()
                && lock_canonical_signature(&e.name, &e.version).is_some()
                && lock_hashes().contains_key(&format!("{}@{}", e.name, e.version))
        }) {
            arms.push((
                entry.name.as_str(),
                Encoding::OwlPrxGz,
                TrustAnchor::LockPinned {
                    version: entry.version.clone(),
                },
            ));
        }

        for (ci, bytes) in corpus.iter().enumerate() {
            for (name, encoding, trust) in &arms {
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    decode_and_project(name, *encoding, trust, bytes)
                }));
                match res {
                    Ok(Err(_)) => {} // correct: typed fail-closed refusal
                    Ok(Ok(_)) => panic!(
                        "hostile corpus case {ci} ({} bytes) decoded Ok under {encoding:?}",
                        bytes.len()
                    ),
                    Err(_) => panic!(
                        "decode_and_project PANICKED on corpus case {ci} ({} bytes) under {encoding:?}",
                        bytes.len()
                    ),
                }
            }
        }
    }

    /// Build an into-English menagerie `.prx` in memory: one `Canine` node named
    /// `rex` and one `InstanceFunctor` connection typing `Canine ↦ <synset>` into
    /// `english_wordnet`. Returns `(bytes, root_hex)` for the fail-closed public
    /// load. The `synset` is a REAL WordNet synset id (discovered from
    /// `english_static()`), so the grounding is exercised against the SAME full
    /// WordNet the browser carries, not a sample.
    fn into_english_menagerie_prx(synset: &str) -> (Vec<u8>, String) {
        use pr4xis_domains::cognitive::linguistics::english::bridge::ENGLISH_ONTOLOGY;
        use pr4xis_runtime::archive::Archive;
        use pr4xis_runtime::connection::{Connection, GeneratorAction};
        use pr4xis_runtime::definition::Definition;
        use pr4xis_runtime::lens::archive_lens::ArchiveLens;
        let archive = Archive {
            nodes: vec![Definition {
                kind: "Canine".to_string(),
                name: "rex".to_string(),
                edges: vec![],
                axioms: vec![],
                lexical: Some("a companion dog kept in the menagerie".to_string()),
            }],
            connections: vec![Connection {
                kind: "InstanceFunctor".to_string(),
                source: "menagerie".to_string(),
                target: ENGLISH_ONTOLOGY.to_string(),
                action: GeneratorAction::Functor {
                    map_object: vec![("Canine".to_string(), synset.to_string())],
                    map_morphism: vec![("denotes".to_string(), "Subsumption".to_string())],
                },
                laws: vec!["PreservesTyping".to_string()],
            }],
        };
        let root = archive.root().expect("root").to_hex();
        let buf = ArchiveLens::put_aligned(&archive);
        (buf.as_slice().to_vec(), root)
    }

    /// A real WordNet synset id for a `dog` sense that is-a some `animal` sense —
    /// discovered from the live `english_static()`, never hardcoded. The grounding
    /// target for the into-English load tests.
    fn a_dog_synset_that_is_an_animal() -> String {
        let english = english_static();
        let animals = english.lookup("animal");
        let dog = english
            .lookup("dog")
            .iter()
            .copied()
            .find(|&d| animals.iter().any(|&a| english.is_a(d, a)))
            .expect("WordNet has a 'dog' sense that is-a an 'animal' sense");
        english
            .concept(dog)
            .expect("the discovered dog synset resolves")
            .original_id()
            .to_string()
    }

    /// FIX 1 REGRESSION: a valid into-English `.prx` (declares a grounding functor
    /// whose target is `english_wordnet`, which is NOT among the loaded peer
    /// archives) must load through the PUBLIC `load_ontology_prx_core` path — NOT
    /// be refused before install. Before the fix, the old pre-materialize peer-set
    /// pre-check built its peer set only from the loaded `runtime_ontologies` (never
    /// seeding English), so a `ground_declared` pre-check against it REFUSED an
    /// into-English target with `MissingPeerArchive`. Now the single
    /// `ground_loaded_set` pass (which seeds English) grounds it, and "is rex an
    /// animal" answers through WordNet's own is-a chain.
    #[test]
    fn an_into_english_prx_loads_and_grounds_through_the_public_path() {
        use pr4xis_runtime::definition::EdgeTarget;
        let synset = a_dog_synset_that_is_an_animal();
        let (bytes, root_hex) = into_english_menagerie_prx(&synset);

        let mut p = Pr4xis::new();
        // THE regression: this returns Ok now (before the fix: Err(Grounding(
        // MissingPeerArchive { english_wordnet }))).
        p.load_ontology_prx_core(&bytes, "menagerie".to_string(), &root_hex)
            .expect("a valid into-English .prx must load through the public path, not be refused");
        assert_eq!(
            p.loaded_ontology_count(),
            base_loaded() + 1,
            "the into-English menagerie installs on top of the base"
        );

        // The grounding actually happened: the installed `rex` node carries a
        // Grounded edge into `english_wordnet` (minted by the single grounding pass,
        // not silently dropped, not refused). Read the OWNED archive (plain
        // Definition/EdgeTarget, not the rkyv-archived view).
        let menagerie = p
            .runtime_ontologies
            .iter()
            .find(|o| o.id().as_str() == "menagerie")
            .expect("the menagerie is installed")
            .to_owned_archive()
            .expect("the menagerie archive decodes");
        let rex = menagerie
            .nodes
            .iter()
            .find(|n| n.name == "rex")
            .expect("rex is a node");
        assert!(
            rex.edges.iter().any(|(_, t)| matches!(
                t,
                EdgeTarget::Grounded { ontology, .. } if ontology == "english_wordnet"
            )),
            "rex must carry the minted Grounded edge into english_wordnet; got {:?}",
            rex.edges
        );

        // The NET RESULT: "is rex an animal" is answerable through English's is-a
        // chain — the same public chat the browser drives.
        let json = p.chat("is rex an animal");
        let resp = response_of(&json).to_lowercase();
        let outcome =
            serde_json::from_str::<serde_json::Value>(&json).expect("chat JSON")["outcome"]
                .as_str()
                .unwrap_or_default()
                .to_string();
        assert_eq!(
            outcome, "answered",
            "the grounded menagerie must answer 'is rex an animal', not abstain; got: {resp:?}"
        );
        assert!(
            resp.contains("yes"),
            "rex (Canine ↦ a dog synset) is an animal via WordNet's chain; got: {resp:?}"
        );
    }

    /// FIX 1/2 FAIL-CLOSED at the public path: an into-English `.prx` whose declared
    /// target synset does NOT exist in the present English peer is a
    /// declared-but-unrealizable grounding — the load must FAIL CLOSED (typed
    /// `Grounding` verdict) and install NOTHING, not silently install ungrounded.
    #[test]
    fn an_into_english_prx_with_an_absent_target_fails_closed_at_the_public_path() {
        let (bytes, root_hex) = into_english_menagerie_prx("s-DOESNOTEXIST-in-wordnet");
        let mut p = Pr4xis::new();
        let err = p
            .load_ontology_prx_core(&bytes, "menagerie".to_string(), &root_hex)
            .expect_err("a declared-but-absent target must fail closed at the public path");
        assert!(
            matches!(
                err,
                LoadError::Grounding(
                    pr4xis_runtime::grounding::LinkError::GroundTargetAbsent { .. }
                )
            ),
            "the refusal must be a typed declared-but-absent grounding fault; got: {err:?}"
        );
        assert_eq!(
            p.loaded_ontology_count(),
            base_loaded(),
            "a fail-closed grounding install nothing beyond the base (transactional rollback)"
        );
        assert!(
            !p.runtime_ontologies
                .iter()
                .any(|o| o.id().as_str() == "menagerie"),
            "the mis-grounded menagerie must not be among the loaded ontologies"
        );
    }

    #[test]
    fn a_fresh_pr4xis_answers_that_a_statute_is_a_law_from_the_always_loaded_base() {
        // THE headline of the always-loaded base: NO explicit load. `Pr4xis::new`
        // installs the LegalSources base at construction, so the chat routes through
        // the ComposedReasoner with the formal sources of law present. "is a statute
        // a law" resolves both surfaces to loaded concepts (the label "law" grounds
        // because the base was emitted with the default lexicalizing `emit`) and reads the
        // Subsumption closure Statute ⊑ LegalDocument ⊑ LegalSource → Yes.
        let mut p = Pr4xis::new();
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
            ontologies
                .iter()
                .any(|o| o["ontology"].as_str() == Some(base_name())),
            "the Yes must credit the LegalSources base it reasoned over; got: {ontologies:?}"
        );

        // self_describe lists the LegalSources base (tagged loaded) from construction,
        // with a non-zero contribution to the loaded concept set.
        let d = serde_json::from_str::<serde_json::Value>(&p.self_describe())
            .expect("self_describe JSON");
        let sources = d["sources"].as_array().expect("sources array");
        let legal = sources
            .iter()
            .find(|s| s["name"].as_str() == Some(base_name()))
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

    /// `chat_batch` runs each question through the STATELESS pipeline
    /// (`process_with_reasoner`), one record per input in order — the browser
    /// mirror of the native Smart-40 probe. A known question answers from the
    /// always-loaded base; an unloaded concept abstains, naming the surface it
    /// could not resolve — and a batch row matches a single `chat` turn for a
    /// non-Conditional question (a session with no open frame is a no-op).
    #[test]
    fn chat_batch_runs_each_question_statelessly_and_in_order() {
        let english = load_english();
        let (surface, _gloss) = demo_concept(&english);
        let known = "is a statute a law".to_string();
        let unknown = format!("what is a {surface}");

        let p = Pr4xis::new();
        let json = p.chat_batch(vec![known.clone(), unknown.clone()]);
        let v: serde_json::Value = serde_json::from_str(&json).expect("chat_batch returns JSON");
        let results = v["results"].as_array().expect("results is an array");
        assert_eq!(results.len(), 2, "one record per input question");

        // Order preserved; each record echoes its OWN question.
        assert_eq!(results[0]["question"].as_str(), Some(known.as_str()));
        assert_eq!(results[1]["question"].as_str(), Some(unknown.as_str()));

        // Row 0 answers from the always-loaded LegalSources base.
        assert_eq!(results[0]["outcome"].as_str(), Some("answered"));
        assert!(
            results[0]["response"]
                .as_str()
                .is_some_and(|s| !s.is_empty()),
            "an answered row carries a non-empty response"
        );

        // Row 1 abstains, naming the surface it could not resolve.
        assert_eq!(results[1]["outcome"].as_str(), Some("abstained"));
        let unresolved = results[1]["unresolved"]
            .as_array()
            .expect("an abstained row carries an unresolved[] list");
        assert!(
            !unresolved.is_empty(),
            "the abstention names at least one unresolved surface"
        );
        // Each entry is `{surface, source_id?}` — the term to show the reader,
        // and the source that would resolve it where the engine can say which.
        // The host routes on `source_id` and never inspects the characters of
        // `surface`; a bare string here would send it back to deriving the
        // route itself, which is the defect this shape removed.
        for u in unresolved {
            assert!(
                u["surface"].as_str().is_some_and(|s| !s.is_empty()),
                "every unresolved entry carries a non-empty `surface`: {u}"
            );
        }

        // Each row is the FULL `chat` envelope (not a stripped subset): the
        // reasoned-over `ontologies`, the flattened `trace` string, and the typed
        // `trace_structured` list — so the page's one renderer handles a batch row
        // exactly like a single answer.
        assert!(
            results[0]["ontologies"].as_array().is_some(),
            "a batch row carries the full `ontologies` list"
        );
        assert!(
            results[0]["trace"].as_str().is_some(),
            "a batch row carries the flattened `trace` string"
        );
        assert!(
            results[0]["trace_structured"].as_array().is_some(),
            "a batch row carries the typed `trace_structured` list"
        );

        // Faithful to the single-turn path: for a non-Conditional question the
        // stateless batch agrees with `chat` byte-for-byte on outcome + response.
        let mut single = Pr4xis::new();
        let single_v: serde_json::Value =
            serde_json::from_str(&single.chat(&known)).expect("chat JSON");
        assert_eq!(
            results[0]["response"].as_str(),
            single_v["response"].as_str()
        );
        assert_eq!(results[0]["outcome"].as_str(), single_v["outcome"].as_str());

        // The single turn echoes its OWN question too, so the two envelopes
        // really do have the same key set rather than nearly the same one.
        // The page's downloadable decision record reads this field: a record
        // carrying the reasoning but not the question it answers is not a
        // record a compliance function can read months later.
        assert_eq!(
            single_v["question"].as_str(),
            Some(known.as_str()),
            "a single `chat` turn echoes the question it answered"
        );
        let batch_keys: Vec<&String> = results[0]
            .as_object()
            .expect("a batch row is an object")
            .keys()
            .collect();
        let single_keys: Vec<&String> = single_v
            .as_object()
            .expect("a single turn is an object")
            .keys()
            .collect();
        assert_eq!(
            batch_keys, single_keys,
            "a batch row and a single turn carry the SAME wire keys — one \
             renderer, one record schema, no per-path drift"
        );
    }

    /// `reset_session` returns the stateful chat session to the no-pending state —
    /// the post-condition a host relies on to safely begin a fresh line of
    /// questioning after abandoning a `Conditional` slot-fill dialogue. (The
    /// pending-frame lifecycle itself is proven in `pr4xis-chat`'s session tests;
    /// this pins the wasm reset contract and that stateless `chat_batch` never
    /// opens a frame.)
    #[test]
    fn reset_session_leaves_no_pending_slot_fill_frame() {
        let mut p = Pr4xis::new();
        assert!(
            !p.session.is_awaiting_fact(),
            "a freshly-constructed session holds no open slot-fill frame"
        );
        // Ordinary turns and a stateless batch keep the invariant; reset re-establishes it.
        let _ = p.chat("is a statute a law");
        let _ = p.chat_batch(vec!["what is dementia".to_string()]);
        assert!(
            !p.session.is_awaiting_fact(),
            "a stateless batch never opens a slot-fill frame"
        );
        p.reset_session();
        assert!(
            !p.session.is_awaiting_fact(),
            "reset_session must leave no pending slot-fill frame"
        );
    }

    /// The TYPED trace (doc §5.3) crosses the wire as a structured list BESIDE the
    /// flattened string — each step carrying its ontology, operation, MAPE-K phase
    /// (the ontology's own label), status/success, detail, reasoned-over set, and
    /// the per-step functor connections whose literature `reference` the flatten
    /// used to discard.
    #[test]
    fn chat_emits_a_typed_trace_beside_the_flattened_string() {
        let mut p = Pr4xis::new();
        let v: serde_json::Value =
            serde_json::from_str(&p.chat("is a statute a law")).expect("chat JSON");

        // The flattened string is still present (additive, back-compat).
        assert!(
            v["trace"].as_str().is_some(),
            "the flattened `trace` string must remain for the existing renderer"
        );

        // The typed trace is a non-empty list of structured step records.
        let steps = v["trace_structured"]
            .as_array()
            .expect("trace_structured is a list");
        assert!(!steps.is_empty(), "a real turn traces at least one step");
        let first = &steps[0];
        for field in ["ontology", "operation", "phase", "detail", "status"] {
            assert!(
                first[field].as_str().is_some(),
                "each trace step carries `{field}` text; got {first:?}"
            );
        }
        assert!(
            first["success"].as_bool().is_some(),
            "each step carries a boolean `success`"
        );
        assert!(
            first["reasoned_over"].as_array().is_some(),
            "each step carries a reasoned_over list"
        );
        assert!(
            first["functor_connections"].as_array().is_some(),
            "each step carries a functor_connections list"
        );

        // The phase is the MapeK ontology's OWN label — the Monitor phase
        // (tokenize/parse/interpret) always appears.
        assert!(
            steps.iter().any(|s| s["phase"].as_str() == Some("Monitor")),
            "the Monitor phase must appear, labeled from the MapeK ontology"
        );

        // A functor connection names its literature reference — the citation the
        // flatten discarded (e.g. tokenize → Communication (Shannon), "Shannon 1948").
        let surfaces_a_reference = steps.iter().any(|s| {
            s["functor_connections"].as_array().is_some_and(|fcs| {
                fcs.iter()
                    .any(|fc| fc["reference"].as_str().is_some_and(|r| !r.is_empty()))
            })
        });
        assert!(
            surfaces_a_reference,
            "the typed trace surfaces per-step literature references"
        );
    }

    /// The DEFINITION-PROVENANCE record reaches the BROWSER-FACING wire, in both
    /// directions, over the real embedded caregiving lexicon a visitor meets.
    ///
    /// This is the end of the plumbing the panel reads. A turn that recites a
    /// lexicon gloss carries the document that gloss was WRITTEN FROM, with its
    /// own realized label, in its own field — and the `why` sentence (which names
    /// the ontologies the turn OPENED) does not mention that document at all.
    /// That separation IS the fix: while the citation rode inside the gloss, an
    /// answer read as statute-backed while only a lexicon entry had been
    /// consulted, and the page had nothing structural to say otherwise.
    ///
    /// The negative half matters as much: a turn answered from the embedded
    /// English substrate declares no document, so the field is simply absent and
    /// the page renders no provenance step.
    #[test]
    fn chat_emits_the_definition_provenance_of_a_recited_lexicon_gloss() {
        let mut p = Pr4xis::new();
        let v: serde_json::Value =
            serde_json::from_str(&p.chat("what is respite care")).expect("chat JSON");

        assert!(
            !v["response"]
                .as_str()
                .unwrap_or_default()
                .contains("42 USC 300ii(7)"),
            "the recited answer no longer wears its citation as prose; got {:?}",
            v["response"]
        );
        let prov = &v["definition_provenance"];
        assert!(
            prov["detail"]
                .as_str()
                .is_some_and(|d| d.contains("42 USC 300ii(7)")),
            "the wire carries the document the gloss was authored from; got {prov:?}"
        );
        assert!(
            prov["label"].as_str().is_some_and(|l| !l.is_empty()),
            "…with the channel's own engine-realized label, so the page writes none"
        );
        assert!(
            !v["why"]
                .as_str()
                .unwrap_or_default()
                .contains("42 USC 300ii(7)"),
            "the authority must stay OUT of the reasoned-over sentence — a page \
             that had to split `why` to separate them is one edit from \
             conflating them again; got {:?}",
            v["why"]
        );

        // The other direction: a substrate answer declares no document at all.
        let substrate: serde_json::Value =
            serde_json::from_str(&p.chat("is a statute a law")).expect("chat JSON");
        assert!(
            substrate["definition_provenance"].is_null(),
            "a turn that recites no sourced definition claims no authorship; got {:?}",
            substrate["definition_provenance"]
        );
    }

    /// `verify_palette` audits a live CSS token set against the theming
    /// ontology's OWN WCAG axioms — the "engine audits its own page" exhibit.
    /// An AA-passing dark palette clears every axiom and every rendered pair
    /// (base05/base00 text at 4.5:1, accents at 3:1); dropping the default
    /// foreground to a near-background grey flips the verdicts to fail — the
    /// panel shows a real regression, never fakes a pass. A non-slot alias key is
    /// skipped, not errored, so the caller can pass its whole token set.
    #[test]
    fn verify_palette_audits_the_page_tokens_against_the_theming_axioms() {
        // Keys carry the CSS `--` prefix exactly as `getComputedStyle` reports
        // them; the trailing `--danger` is a semantic alias (not a base slot) and
        // must be skipped rather than error.
        let keys: Vec<String> = [
            "--base00", "--base01", "--base02", "--base03", "--base04", "--base05", "--base06",
            "--base07", "--base08", "--base09", "--base0A", "--base0B", "--base0C", "--base0D",
            "--base0E", "--base0F", "--danger",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let good: Vec<String> = [
            "#0d1117", "#161b22", "#30363d", "#8b949e", "#9aa4b0", "#c9d1d9", "#e6edf3", "#f0f6fc",
            "#f85149", "#db8b3f", "#e3b341", "#3fb950", "#56d4dd", "#58a6ff", "#bc8cff", "#db6d28",
            "#f85149",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        let p = Pr4xis::new();
        let v: serde_json::Value =
            serde_json::from_str(&p.verify_palette(keys.clone(), good.clone()))
                .expect("verify_palette returns JSON");

        // base00 is dark, so the detected polarity is dark.
        assert_eq!(v["polarity"].as_str(), Some("dark"));

        let checks = v["checks"].as_array().expect("checks is an array");
        assert!(!checks.is_empty());
        // Every check carries an axiom name + a WCAG citation from the ontology's
        // own metadata + a boolean verdict.
        for c in checks {
            assert!(c["axiom"].as_str().is_some(), "each check names its axiom");
            assert!(
                c["citation"].as_str().is_some_and(|s| s.contains("WCAG")),
                "each check cites WCAG from the axiom's own axiom_meta!"
            );
            assert!(c["pass"].as_bool().is_some());
        }

        // The palette-axiom SUMMARY rows (no `pair`) both hold on this AA palette.
        let summary_pass = |cs: &[serde_json::Value], name: &str| -> Option<bool> {
            cs.iter()
                .find(|c| c["axiom"].as_str() == Some(name) && c["pair"].is_null())
                .and_then(|c| c["pass"].as_bool())
        };
        assert_eq!(summary_pass(checks, "WcagForegroundContrast"), Some(true));
        assert_eq!(summary_pass(checks, "RenderedPairsMeetAa"), Some(true));

        // A per-pair row: base05-on-base00 is NORMAL text (>= 4.5:1).
        let pair = |cs: &[serde_json::Value], label: &str| -> serde_json::Value {
            cs.iter()
                .find(|c| c["pair"].as_str() == Some(label))
                .cloned()
                .unwrap_or_else(|| panic!("missing pair row {label:?}"))
        };
        let base05 = pair(checks, "base05 on base00");
        assert_eq!(base05["required"].as_f64(), Some(4.5));
        assert!(base05["ratio"].as_f64().unwrap() > 4.5);
        assert_eq!(base05["pass"].as_bool(), Some(true));
        // An accent is a UI/large pair (>= 3:1), the looser but CORRECT demand.
        let accent = pair(checks, "base0D on base00");
        assert_eq!(accent["required"].as_f64(), Some(3.0));
        assert_eq!(accent["pass"].as_bool(), Some(true));

        // HONESTY: drop the default foreground to a near-background grey — the
        // base05 pair AND both foreground axioms must flip to a failing verdict.
        let mut bad = good.clone();
        bad[5] = "#1a1f27".to_string();
        let bv: serde_json::Value =
            serde_json::from_str(&p.verify_palette(keys, bad)).expect("verify_palette JSON");
        let bad_checks = bv["checks"].as_array().unwrap();
        assert_eq!(
            summary_pass(bad_checks, "WcagForegroundContrast"),
            Some(false),
            "a failing base05 must fail WcagForegroundContrast"
        );
        assert_eq!(summary_pass(bad_checks, "RenderedPairsMeetAa"), Some(false));
        assert_eq!(
            pair(bad_checks, "base05 on base00")["pass"].as_bool(),
            Some(false)
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
// EMBEDDED new-format `.prx` through the ONE public WASM `load` method with the
// `"rkyv-archive"` encoding (the exact entry the worker calls), and assert the
// chat answers from the loaded gloss — and abstains before the load. This
// exercises the real wasm-bindgen boundary (JsValue marshalling included),
// not just the native core.
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
        let demo = embedded_demo();
        let root = ContentAddress::from_hex(demo.root_hex).unwrap();
        let mut buf = AlignedVec::<16>::with_capacity(demo.bytes.len());
        buf.extend_from_slice(demo.bytes);
        let ontology = materialize_bytes(buf, OntologyName::new(demo.name))
            .expect("embedded demo .prx materializes");
        assert_eq!(
            ontology.root(),
            root,
            "embedded demo .prx must re-derive its baked root"
        );
        ontology
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
        for node in onto.archive().nodes.iter() {
            let surface = node.name.to_lowercase();
            let cref = onto.concept(node.name.to_string());
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
        let mut without = Pr4xis::new();
        // A fresh Pr4xis carries every always-loaded `default_loaded` base
        // (LegalSources + the chat lexicons — `embedded_base().count()`,
        // never a hardcoded count that goes stale as more bases are
        // embedded), but NOT the Dependability demo — so the demo concept
        // still abstains here.
        assert_eq!(without.loaded_ontology_count(), embedded_base().count());
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

        // WITH the corpus: load the embedded `.prx` through the ONE public WASM
        // `load` method — the exact typed entry the worker calls — with an absent
        // payload (the bytes ship in the wasm; `None` resolves them by name) and
        // the rkyv-archive encoding. The fail-closed gate runs in the
        // browser. The asserted gloss is the one DISCOVERED from the loaded
        // ontology, so the evidence chain is real — a hardcoded gloss could not
        // satisfy it. This exercises the real wasm-bindgen boundary end to end.
        let mut with = Pr4xis::new();
        with.load(
            embedded_demo().name.to_string(),
            "rkyv-archive",
            None,
            None,
            None,
        )
        .expect("the embedded demo .prx loads in the browser (fail-closed)");
        // Every always-loaded `default_loaded` base plus the just-loaded
        // Dependability demo (1) — `embedded_base().count()`, never a
        // hardcoded count.
        assert_eq!(with.loaded_ontology_count(), embedded_base().count() + 1);
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

// ─── The wire boundary, pinned ───────────────────────────────────────────────
//
// `LoadRequest::from_wire` / `Encoding::from_wire` are the SINGLE tagged
// decode at the JS↔wasm boundary — the one blessed string lowering. These
// native tests pin (a) the inverse pair `from_wire(wire_tag(e)) == e` over
// EVERY variant (exhaustive match: adding an `Encoding` variant without
// extending the list is a compile error, so the pin cannot silently go
// partial), and (b) each fail-closed refusal arm per typed `LoadError`
// variant. Pure functions — no browser required; the browser suite exercises
// the downstream decode/gate arms.
#[cfg(all(test, not(target_arch = "wasm32")))]
mod wire_boundary {
    use super::*;

    /// Every `Encoding` variant. The exhaustive match in
    /// [`encoding_wire_tag_round_trips_for_every_variant`] forces this list to
    /// grow with the enum.
    const ALL_ENCODINGS: [Encoding; 4] = [
        Encoding::UslmTitle,
        Encoding::OwlSource,
        Encoding::OwlPrxGz,
        Encoding::RkyvArchive,
    ];

    /// The blessed lowering is an inverse pair: `from_wire(wire_tag(e)) == e`
    /// for every variant — the drift a second hand-assembled lowering (or a
    /// swapped match arm) would introduce fails here.
    #[test]
    fn encoding_wire_tag_round_trips_for_every_variant() {
        for e in ALL_ENCODINGS {
            // Exhaustive: a new Encoding variant fails to compile here until
            // it is added to ALL_ENCODINGS and this match.
            match e {
                Encoding::UslmTitle
                | Encoding::OwlSource
                | Encoding::OwlPrxGz
                | Encoding::RkyvArchive => {}
            }
            let tag = e.wire_tag();
            let back = Encoding::from_wire(tag).expect("known tag must decode");
            assert_eq!(back, e, "from_wire(wire_tag({tag:?})) must be identity");
        }
    }

    /// An unknown wire tag is a typed `UnknownEncoding` refusal carrying the
    /// offending tag — never a fallback to some default encoding.
    #[test]
    fn unknown_wire_tag_is_a_typed_refusal() {
        for bad in ["prx-gz", "", "USLM-TITLE", "uslm_title"] {
            let err = LoadRequest::from_wire("x".into(), bad, None, None, Some(vec![1]))
                .expect_err("unknown wire tag must refuse");
            assert!(
                matches!(err, LoadError::UnknownEncoding(ref t) if t == bad),
                "got {err:?} for tag {bad:?}"
            );
        }
    }

    /// `owl-prx-gz` without a `version` cannot key the three-pin lock lookup —
    /// typed `MissingVersion`, never an unpinned load.
    #[test]
    fn owl_prx_gz_without_version_is_missing_version() {
        let err = LoadRequest::from_wire("x".into(), "owl-prx-gz", None, None, Some(vec![1]))
            .expect_err("owl-prx-gz without version must refuse");
        assert!(matches!(err, LoadError::MissingVersion), "got {err:?}");
    }

    /// The cross-wiring seam: a SUPPLIED payload under an embedded name with
    /// no `root_hex` must be `MissingRoot` — foreign bytes must NEVER inherit
    /// the manifest's baked root (the embedded manifest is consulted only when
    /// the payload is absent).
    #[test]
    fn supplied_payload_under_embedded_name_never_inherits_the_baked_root() {
        let demo = embedded_demo();
        let err = LoadRequest::from_wire(
            demo.name.to_string(),
            "rkyv-archive",
            None,
            None,
            Some(vec![0xAA; 4]),
        )
        .expect_err("a supplied payload with no trusted root must refuse");
        assert!(matches!(err, LoadError::MissingRoot), "got {err:?}");
    }

    /// An absent payload naming no build-baked ontology is a typed
    /// `NoEmbedded` refusal carrying the name — never an empty load.
    #[test]
    fn absent_payload_with_unknown_name_is_no_embedded() {
        let err = LoadRequest::from_wire("no-such-embedded".into(), "uslm-title", None, None, None)
            .expect_err("unknown embedded name must refuse");
        assert!(
            matches!(err, LoadError::NoEmbedded(ref n) if n == "no-such-embedded"),
            "got {err:?}"
        );
    }

    /// A supplied Merkle root that is not 64-char lowercase hex is a typed
    /// `BadRootHex` refusal carrying the offending string.
    #[test]
    fn malformed_root_hex_is_bad_root_hex() {
        let err = LoadRequest::from_wire(
            "x".into(),
            "rkyv-archive",
            None,
            Some("zz".into()),
            Some(vec![1]),
        )
        .expect_err("malformed root hex must refuse");
        assert!(
            matches!(err, LoadError::BadRootHex(ref h) if h == "zz"),
            "got {err:?}"
        );
    }

    /// The embedded happy path stays intact: an absent payload under the
    /// embedded demo's name resolves BOTH the baked bytes and the baked root.
    #[test]
    fn absent_payload_with_embedded_name_resolves_baked_bytes_and_root() {
        let demo = embedded_demo();
        let req = LoadRequest::from_wire(demo.name.to_string(), "rkyv-archive", None, None, None)
            .expect("the embedded demo must resolve");
        assert_eq!(req.payload, demo.bytes, "payload must be the baked bytes");
        let baked = ContentAddress::from_hex(demo.root_hex).expect("baked root parses");
        assert!(
            matches!(req.trust, TrustAnchor::MerkleRoot(r) if r == baked),
            "trust anchor must be the manifest's baked root"
        );
    }
}
