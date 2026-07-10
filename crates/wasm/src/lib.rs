use std::rc::Rc;

use wasm_bindgen::prelude::*;

pub mod load_envelope;

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
use pr4xis_domains::formal::meta::grounding::ground_loaded_set;
use pr4xis_domains::social::software::markup::xml::lmf::prx::load_english_store_bundle_gz_gated;
use pr4xis_domains::social::software::markup::xml::owl::bridge::owl_runtime_ontology;
use pr4xis_domains::social::software::markup::xml::owl::prx::load_prx_gz;
use pr4xis_domains::social::software::markup::xml::owl::reader::read_owl;
use pr4xis_domains::social::software::markup::xml::owl::vocabulary::LoadedOwlVocabulary;
use pr4xis_domains::social::software::markup::xml::uslm::corpus::UsCode;
use pr4xis_domains::social::software::markup::xml::uslm::corpus::bridge::usc_runtime_ontology;
use pr4xis_domains::social::software::markup::xml::uslm::lens::read_uslm_title;
use pr4xis_runtime::address::ContentAddress;
use pr4xis_runtime::ontology::{RuntimeOntology, materialize};

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
/// ontologies `Pr4xis::new` installs at construction (currently the single
/// LegalSources base). The one place the `default_loaded` residency partition
/// is read on the install side, so `new()` carries no per-name special case.
fn embedded_base() -> impl Iterator<Item = &'static embedded_prx::EmbeddedOntology> {
    embedded_prx::EMBEDDED_PRX
        .iter()
        .filter(|e| e.default_loaded)
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
}

/// WHAT is being loaded — the ONE typed selector that resolves the (decoder,
/// projection functor) pair by TYPED dispatch (doc §3), never a byte-sniff or a
/// string match on format. Grounds on the cited `ContentType` /
/// `SourceTaxonomyConcept` provisioning ontology
/// (`applied::data_provisioning::ontology`): [`Encoding::UslmTitle`] is that
/// ontology's `UslmXml` (1 U.S.C. §204), [`Encoding::OwlSource`] its `Owl`
/// (W3C OWL 2 RDF/XML). The two praxis-native envelope forms —
/// [`Encoding::OwlPrxGz`] (the OWL `.prx.gz` distribution envelope) and
/// [`Encoding::ContentAddressedArchive`] (the content-addressed `.prx` Archive) —
/// belong to praxis's own serialization ontology. The `(decoder, functor)` per
/// variant is [`decode_and_project`]'s single typed match; the JS↔wasm boundary
/// carries only the wire tag, decoded ONCE by [`Encoding::from_wire`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Encoding {
    /// USLM XML title (1 U.S.C. §204). Decoder `read_uslm_title`; functor
    /// `usc_runtime_ontology`. Cited `ContentType::UslmXml`.
    UslmTitle,
    /// OWL 2 RDF/XML source (W3C OWL 2). Decoder `read_owl`; functor
    /// `owl_runtime_ontology`. Cited `ContentType::Owl`.
    OwlSource,
    /// The OWL `.prx.gz` distribution envelope. Decoder `owl::prx::load_prx_gz`
    /// (gunzip + bytecheck + three-pin verify); functor `owl_runtime_ontology`.
    OwlPrxGz,
    /// The content-addressed `.prx` Archive. Decoder `load::load` (re-derive +
    /// refuse on Merkle-root mismatch); projection is identity (`materialize`).
    ContentAddressedArchive,
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
            "content-addressed-archive" => Ok(Encoding::ContentAddressedArchive),
            other => Err(LoadError::UnknownEncoding(other.to_string())),
        }
    }

    /// The wire tag for this encoding — the inverse of [`Encoding::from_wire`],
    /// used in the [`Loaded`] receipt so the UI can name what it loaded.
    fn wire_tag(self) -> &'static str {
        match self {
            Encoding::UslmTitle => "uslm-title",
            Encoding::OwlSource => "owl-source",
            Encoding::OwlPrxGz => "owl-prx-gz",
            Encoding::ContentAddressedArchive => "content-addressed-archive",
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
    /// Content-addressed `.prx`: the trusted Merkle root from OUTSIDE the bytes;
    /// [`load::load`](pr4xis_runtime::load::load) re-derives it and refuses on
    /// mismatch. ([`Encoding::ContentAddressedArchive`].)
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
/// typed [`Encoding`], never a byte-sniff), carries the resolved PAYLOAD bytes,
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
    /// [`LoadRequest::from_wire`].
    payload: Vec<u8>,
    /// HOW the load is made fail-closed — one typed anchor, verified in one place.
    trust: TrustAnchor,
}

impl LoadRequest {
    /// Build the typed request from the JS↔wasm wire fields — the SINGLE tagged
    /// decode at the boundary (doc §3 point 3). Resolves the payload (an absent
    /// boundary payload ⇒ the build-baked embedded bytes for `name`) and the
    /// trust anchor per the typed [`Encoding`] (version pins `OwlPrxGz`; a
    /// Merkle root — supplied or resolved from the embedded manifest — pins
    /// `ContentAddressedArchive`). Fail-closed: an unknown encoding, a missing
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
            // The content-addressed `.prx` root is the trusted anchor: supplied
            // by the caller, or (for an embedded load) the manifest's baked root.
            Encoding::ContentAddressedArchive => {
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
    /// The wire tag was not a known [`Encoding`].
    UnknownEncoding(String),
    /// [`Encoding::OwlPrxGz`] needs a `version` for its three-pin lock lookup.
    MissingVersion,
    /// [`Encoding::ContentAddressedArchive`] needs a trusted Merkle root.
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
    /// The content-addressed gate refused the bytes — decode failure or a
    /// re-derived root that does not match the trusted root (tampered / stale /
    /// wrong).
    Refused(pr4xis_runtime::load::LoadError),
    /// The admitted archive could not be materialized (e.g. a dangling edge —
    /// referential closure violated).
    Materialize(pr4xis_runtime::ontology::MaterializeError),
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
                "the content-addressed-archive encoding requires a trusted Merkle root"
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
            LoadError::Refused(e) => write!(f, ".prx load refused: {e}"),
            LoadError::Materialize(e) => write!(f, ".prx materialize failed: {e}"),
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
/// as-typed-selector — verifying the [`TrustAnchor`] in the same arm (trust and
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
            usc_runtime_ontology(&usc, OntologyName::new(name.to_string()))
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
        Encoding::ContentAddressedArchive => {
            let root = trust.expect_merkle_root(encoding)?;
            let archive = pr4xis_runtime::load::load(bytes, root).map_err(LoadError::Refused)?;
            materialize(archive, OntologyName::new(name.to_string()))
                .map_err(LoadError::Materialize)
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
            english: english_static(),
            runtime_ontologies: Vec::new(),
            composed: None,
            history: Vec::new(),
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
            None => pr4xis_chat::process_with_metadata(self.english, input),
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

    /// THE load path — the ONE typed entry every source now takes (doc §3). One
    /// message shape, one dispatch: [`LoadRequest::from_wire`] decodes the
    /// boundary fields into a typed [`LoadRequest`] (the single tagged decode at
    /// the FFI), then [`Self::load_core`] resolves the `(decoder, projection
    /// functor)` for the [`Encoding`] by typed match, verifies the
    /// [`TrustAnchor`] fail-closed, decodes → projects to a [`RuntimeOntology`] →
    /// installs it through the shared grounding + reasoner-rebuild tail
    /// ([`Self::install_runtime_ontology`]). Returns a small [`Loaded`] receipt
    /// (`{ name, encoding, bytes, root }`) so the UI can name what it loaded —
    /// subsuming the old per-format `load_source` / `load_prx` / `load_owl_source`
    /// / `load_ontology_prx` / `load_embedded_demo_prx` methods.
    ///
    /// - `encoding` is the wire tag for the typed [`Encoding`]: `"uslm-title"`,
    ///   `"owl-source"`, `"owl-prx-gz"`, or `"content-addressed-archive"`.
    /// - `version` pins the three-lock lookup for `"owl-prx-gz"` (ignored else).
    /// - `root_hex` supplies the trusted Merkle root for
    ///   `"content-addressed-archive"` (ignored else).
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

    /// THE single load core (doc §3) — the plain-Rust path behind the public
    /// [`Self::load`] and every native-test load helper. One structural arm:
    /// resolve `(decoder, functor)` for the request's [`Encoding`] and verify its
    /// [`TrustAnchor`] ([`decode_and_project`]), then install through the shared
    /// grounding + reasoner-rebuild tail ([`Self::install_runtime_ontology`]).
    /// Per-encoding knowledge is the typed [`Encoding`] variant; per-trust
    /// knowledge is the [`TrustAnchor`] variant — neither is a method. Returns the
    /// [`Loaded`] receipt. Fail-closed and transactional: a decode/verify refusal
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
    /// [`Self::load_core`] path (a [`LoadRequest`] carrying [`Encoding::UslmTitle`]
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

    /// Native-test helper: load a content-addressed `.prx` from its bytes +
    /// trusted root through the ONE typed [`Self::load_core`] path (a
    /// [`LoadRequest`] carrying [`Encoding::ContentAddressedArchive`] +
    /// [`TrustAnchor::MerkleRoot`]). This IS the browser load path; it just
    /// carries the typed verdict instead of a `JsValue`.
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
            encoding: Encoding::ContentAddressedArchive,
            payload: bytes.to_vec(),
            trust: TrustAnchor::MerkleRoot(root),
        })
        .map(|_| ())
    }

    /// Native-test helper: load the build-baked Dependability demo `.prx` through
    /// the same fail-closed [`Self::load_core`] path and return its name — the
    /// native mirror of the public embedded load (`load(name, "content-addressed-
    /// archive", None, None, None)`). Gated to the native test build.
    #[cfg(all(test, not(target_arch = "wasm32")))]
    fn load_embedded_demo_prx_core(&mut self) -> Result<String, LoadError> {
        let demo = embedded_demo();
        let root = ContentAddress::from_hex(demo.root_hex)
            .ok_or_else(|| LoadError::BadRootHex(demo.root_hex.to_string()))?;
        self.load_core(LoadRequest {
            name: demo.name.to_string(),
            encoding: Encoding::ContentAddressedArchive,
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
        let demo = embedded_demo();
        let root = ContentAddress::from_hex(demo.root_hex).unwrap();
        let archive = pr4xis_runtime::load::load(demo.bytes, root)
            .expect("embedded demo .prx loads fail-closed against its baked root");
        materialize(archive, OntologyName::new(demo.name)).expect("embedded demo .prx materializes")
    }

    /// The name of the always-loaded base (the manifest's single `default_loaded`
    /// entry — currently the LegalSources base), read from the manifest so the
    /// test never restates the string.
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
        //     typed core; the wasm `load` method with the
        //     `"content-addressed-archive"` encoding is a thin wrapper over
        //     exactly this call.) ---
        let mut with = Pr4xis::new();
        let loaded_name = with
            .load_embedded_demo_prx_core()
            .expect("the embedded demo .prx loads (fail-closed root matches)");
        assert_eq!(loaded_name, embedded_demo().name);
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
        assert_eq!(p.loaded_ontology_count(), BASE_LOADED);
        p.load_source_core("Title 18 (test)".to_string(), SAMPLE_USLM_TITLE)
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
        p.load_source_core("Title 18 (test)".to_string(), SAMPLE_USLM_TITLE)
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
            .load_ontology_prx_core(embedded_demo().bytes, "Dependability".into(), &wrong_root)
            .expect_err("a wrong trusted root must be refused (fail-closed)");
        // The typed verdict IS a root mismatch (not a decode error, not a
        // materialize error) — the gate re-derived the root and rejected it.
        assert!(
            matches!(
                err,
                LoadError::Refused(pr4xis_runtime::load::LoadError::RootMismatch { .. })
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
        let mut bytes = embedded_demo().bytes.to_vec();
        *bytes.last_mut().unwrap() ^= 0xff;
        let mut p = Pr4xis::new();
        let err = p
            .load_ontology_prx_core(&bytes, "Dependability".into(), embedded_demo().root_hex)
            .expect_err("tampered .prx bytes must be refused (fail-closed)");
        assert!(
            matches!(err, LoadError::Refused(_)),
            "tampered bytes must be refused by the gate; got: {err:?}"
        );
        assert_eq!(
            p.loaded_ontology_count(),
            BASE_LOADED,
            "a refused .prx installs nothing beyond the always-loaded base"
        );
    }

    /// ∀-ARBITRARY-PAYLOAD TOTALITY, per [`Encoding`] arm: `decode_and_project`
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
                Encoding::ContentAddressedArchive,
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
        let bytes = pr4xis_runtime::load::emit(&archive).expect("emit the menagerie .prx");
        let root = archive.root().expect("root").to_hex();
        (bytes, root)
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
            BASE_LOADED + 1,
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
            BASE_LOADED,
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
// `"content-addressed-archive"` encoding (the exact entry the worker calls),
// and assert the chat answers from the loaded gloss — and abstains before the
// load. This exercises the real wasm-bindgen boundary (JsValue marshalling
// included), not just the native core.
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
        let archive = pr4xis_runtime::load::load(demo.bytes, root)
            .expect("embedded demo .prx loads fail-closed against its baked root");
        materialize(archive, OntologyName::new(demo.name)).expect("embedded demo .prx materializes")
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

        // WITH the corpus: load the embedded `.prx` through the ONE public WASM
        // `load` method — the exact typed entry the worker calls — with an absent
        // payload (the bytes ship in the wasm; `None` resolves them by name) and
        // the content-addressed encoding. The fail-closed gate runs in the
        // browser. The asserted gloss is the one DISCOVERED from the loaded
        // ontology, so the evidence chain is real — a hardcoded gloss could not
        // satisfy it. This exercises the real wasm-bindgen boundary end to end.
        let mut with = Pr4xis::new();
        with.load(
            embedded_demo().name.to_string(),
            "content-addressed-archive",
            None,
            None,
            None,
        )
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
        Encoding::ContentAddressedArchive,
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
                | Encoding::ContentAddressedArchive => {}
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
            "content-addressed-archive",
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
            "content-addressed-archive",
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
        let req = LoadRequest::from_wire(
            demo.name.to_string(),
            "content-addressed-archive",
            None,
            None,
            None,
        )
        .expect("the embedded demo must resolve");
        assert_eq!(req.payload, demo.bytes, "payload must be the baked bytes");
        let baked = ContentAddress::from_hex(demo.root_hex).expect("baked root parses");
        assert!(
            matches!(req.trust, TrustAnchor::MerkleRoot(r) if r == baked),
            "trust anchor must be the manifest's baked root"
        );
    }
}
