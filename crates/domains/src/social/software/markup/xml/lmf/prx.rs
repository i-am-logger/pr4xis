//! `.prx.gz` — the self-describing, load-validated distribution envelope
//! for the loaded English (Open English WordNet) lexicon.
//!
//! This is the **WordNet / English consumer** of the
//! [`OntologyArchiveStorage`](crate::formal::meta::ontology_archive)
//! ontology — the lexical sibling of the OWL leaf
//! ([`crate::social::software::markup::xml::owl::prx`]) and the USC corpus
//! leaf ([`crate::social::software::markup::xml::uslm::corpus::prx`]). It
//! realises the SAME archive ontology (same runnable axioms, same
//! content-addressed load gate, same OMV/PROV-O-grounded metadata schema)
//! over a WN-LMF lexicon rather than over an OWL vocabulary or a USLM
//! statute corpus — a parallel *realisation*, never a parallel envelope
//! module. Every shared primitive ([`OwnedCodegenData`], [`RawSource`],
//! [`PrxError`], [`gzip`] / [`gunzip`], [`ContentAddress`],
//! [`prx_archive_address`], [`EmittedArtifact`], `raw_hash::verify`,
//! [`RoundTripFidelity`]) is reused VERBATIM from the OWL leaf; only the two
//! private monomorphic gate *legs* and a WordNet-specific metadata block are
//! mirrored, because OWL's legs are `&PrxEnvelope`-typed and return
//! [`LoadedOwlVocabulary`](crate::social::software::markup::xml::owl::vocabulary::LoadedOwlVocabulary).
//!
//! ## The flat shape — no aux side-channel
//!
//! WordNet at the first fidelity tier is FLAT, like OWL: synsets are
//! entities and every WordNet relation (`hypernym` / `holo_*` / `mero_*` /
//! `causes` / `also`) projects onto one of the `(u64, u64)` edge tables
//! [`OwnedCodegenData`] already carries. There is **no subdivision-depth
//! `aux` field** (the USLM analogue) — a WN-LMF lexicon has no nested
//! document tree. The one delta from the OWL leaf is the OPPOSITE of USC's:
//! WordNet POPULATES [`OwnedCodegenData::word_index`] (lemma → synset
//! handles) where the OWL emitter leaves it empty, because the English
//! lexicon's whole purpose is `word → concept` lookup. The
//! parse-faithful `aux` (sense-level relations, ILI, examples, subcat
//! frames) is a deliberately-deferred later fidelity tier, NOT built here.
//!
//! ## The lexical interchange, bottom-up
//!
//! ```text
//! read_wordnet(source)                  authoritative WN-LMF XML  (one reader)
//!   └► wn_builder_to_owned ─► OwnedCodegenData   the M4.ε interchange shape,
//!                                                 word_index POPULATED + SORTED
//!        └► WordNetPrxEnvelope             + OMV/PROV-O metadata block
//!             └► rkyv bytes                deterministic, bytecheck-validated
//!                  └► gzip                 RFC 1952 wrapper ─► `.prx.gz`
//! ```
//!
//! Unlike `owl::prx`'s emit (gated on `codegen` because `owl_to_builder`
//! needs `pr4xis::codegen`), WordNet emit is reachable under `prx` ALONE:
//! it parses via the ungated [`read_wordnet`] reader (the pure-Rust
//! `xml::reader`, NOT `xsd-parser`), the exact path `English::from_wordnet`
//! uses, so no `xsd-parser` substrate leaks into the WASM-facing build.
//! This is the same legitimate divergence the USC leaf documents.
//!
//! ## The word_index sort is load-bearing
//!
//! [`OwnedCodegenData::word_index`] MUST be SORTED by word: the runtime
//! [`CodegenData::lookup`](pr4xis::codegen_data::CodegenData::lookup) does a
//! `binary_search_by_key` on it, and an unsorted or process-nondeterministic
//! order would (a) break lookups and (b) make the rkyv `MerkleRoot`
//! non-reproducible, breaking the archive pin. The codegen-side emitter
//! (`pr4xis::codegen::generate::write_word_index`) sorts for exactly the
//! same reasons; `wn_builder_to_owned` mirrors that sort so an
//! archive-materialized [`English`] is identity-identical to the
//! codegen-materialized one.
//!
//! ## Corpus-faithful round-trip target
//!
//! The round-trip equivalence target is the materialized [`English`] value
//! produced by [`language::from_codegen`] — the SAME functor the
//! WASM/codegen consumer runs. `emit → load` must reproduce, on
//! `concept_count` / `word_count` / `lookup`, the corpus
//! [`English::from_wordnet`] yields from the same source.
//!
//! Byte-exact source fidelity (`hash(out) == hash(in)`, the #186 invariant) is
//! GRAPH-FAITHFUL: WN-LMF's `.prx`
//! regenerates the exact source bytes from the typed [`WordNet`] ontology plus a
//! content-addressed concrete-syntax complement ([`WnGraphFaithful`] — the
//! §2.8 `<!DOCTYPE>`, the root namespaces, the §2.4 inter-element white-space,
//! the §3.1 intra-tag layout, the §4.6 entity-reference form, the source
//! attribute sequences), with NO stored raw blob. The capture/reconstruct pair
//! ([`capture_wn_complement`] / [`reconstruct_wn_lmf_source`], parser
//! `source_syntax` residue + the WN-LMF structural writer) is proven a byte-exact
//! inverse over the real 89 MB corpus. English therefore emits
//! [`RoundTripFidelity::ByteExactGraphFaithful`], as do OWL (via
//! `write_owl_exact`) and USC (via `write_uslm`) through their own writers; the
//! [`RoundTripFidelity::RawBytesComplementFloor`] raw-blob leaf
//! [`RawSource::blob`] is the generic fallback for a source with no registered
//! byte-exact lens, kept here for the floor reconstruction path.
//!
//! ## Citations
//!
//! - **McCrae, Rademaker, Bond, Rudnicka & Fellbaum (2019)** "English
//!   WordNet 2019 — An Open-Source WordNet for English", *Proc. GWC 2019*.
//!   The Open English WordNet this leaf archives.
//! - **Fellbaum (1998)** *WordNet: An Electronic Lexical Database*, MIT
//!   Press — synsets, hypernymy, meronymy, `also`.
//! - **ISO 24613:2008** *Language resource management — Lexical markup
//!   framework (LMF)*; **Global WordNet Association WN-LMF 1.3 schema**
//!   (<https://globalwordnet.github.io/schemas/>) — the lexicon serialization
//!   (`Lexicon` / `Synset` / `LexicalEntry` / `Sense`) the metric counts.
//! - **McCrae, Bosque-Gil, Gracia, Buitelaar & Cimiano (2017)**
//!   "The OntoLex-Lemon Model", *LDK 2017* — the lexicon↔ontology bridge
//!   grounding `omv:URI` for a lexical resource.
//! - **Hartmann, Palma & Sure (2005)** OMV; **Lebo, Sahoo & McGuinness
//!   (2013)** PROV-O — the metadata grounding reused from the OWL leaf.
//! - **Foster, Greenwald, Moore, Pierce & Schmitt (2007)** ACM TOPLAS 29(3)
//!   §2.2; **Aumasson, O'Connor, Neves & Wilcox-O'Hearn (2020)** BLAKE3; **Dolstra (2006)**
//!   content-addressing — the shared lens / hash grounding.
//!
//! [`read_wordnet`]: super::reader::read_wordnet
//! [`English`]: crate::cognitive::linguistics::english::English
//! [`English::from_wordnet`]: crate::cognitive::linguistics::english::English::from_wordnet
//! [`language::from_codegen`]: crate::cognitive::linguistics::language::from_codegen
//! [`CodegenData`]: pr4xis::codegen_data::CodegenData

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use pr4xis::codegen_data::CodegenData;
use pr4xis_runtime::address::ContentAddress;

use super::ontology::WordNet;
use super::reader::read_wordnet;
use super::writer::{WnSyntaxComplement, capture_wn_complement, reconstruct_wn_lmf_source};
use crate::applied::data_provisioning::registry::LockDigest;
use crate::cognitive::linguistics::english::English;
use crate::cognitive::linguistics::language;
use crate::formal::meta::artifact_identity::ontology::{
    IdentityClaim, IdentityConcept, VerificationResult,
};
use crate::formal::meta::artifact_identity::schemes::raw_hash;
use crate::formal::meta::well_behaved_lens::RoundTripFidelity;
// Shared archive primitives — reused VERBATIM from the OWL leaf (the genuine
// second/third-consumer boundary): the content-hash, the gzip/rkyv codecs,
// the owned codegen mirror, the raw-source complement, and the error type.
// Only the private monomorphic gate legs below (`wn_verify_content_address`
// / `wn_admit_validated`) are mirrored, because OWL's are `&PrxEnvelope`-
// typed and return `LoadedOwlVocabulary`.
use crate::social::software::markup::xml::owl::prx::{
    EmittedArtifact, OwnedCodegenData, PrxError, RawSource, gunzip, gzip, prx_archive_address,
};

// =============================================================================
// WnPrxMetadata — the OMV/PROV-O-grounded metadata block, WordNet metrics.
// =============================================================================

/// Self-describing metadata carried by a [`WordNetPrxEnvelope`].
///
/// Reuses the OWL leaf's OMV (Hartmann, Palma & Sure 2005) / PROV-O (Lebo,
/// Sahoo & McGuinness 2013) grounding for identity — `name` (`omv:name`),
/// `version` (`omv:version`), `lexicon_uri` (`omv:URI`), `source_url`
/// (`prov:atLocation`), `source_address` (`prov:wasDerivedFrom` content
/// address) — but swaps OWL's `omv:numberOfClasses` / `omv:numberOfProperties`
/// structural metrics (meaningless for a lexicon) for the
/// lexically-appropriate [`Self::number_of_synsets`] /
/// [`Self::number_of_senses`], cited to ISO 24613 LMF + the Global WordNet
/// Association WN-LMF schema (a `Lexicon` is a set of `Synset`s and
/// `Sense`s — Fellbaum 1998). The OWL
/// [`PrxMetadata`](crate::social::software::markup::xml::owl::prx::PrxMetadata)
/// and the USC `UscPrxMetadata` are left untouched (their rkyv layouts —
/// hence the existing `[archive_signatures]` pins — must not change for a
/// WordNet feature).
///
/// The two structural metrics are **self-description**, exactly like OWL's
/// `omv:numberOf*` fields: the load gate does NOT check them (it binds the
/// whole envelope through the `MerkleRoot` content address, which already
/// covers `data`). They self-describe the archive without materializing it.
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct WnPrxMetadata {
    /// `omv:name` (Hartmann 2005) — the registry source name, e.g.
    /// `"english_wordnet"`. With [`Self::version`] this is the
    /// `"{name}@{version}"` key the load gate looks pins up under.
    pub name: String,
    /// `omv:version` (Hartmann 2005) — the source version, e.g. `"2025"`
    /// (the Open English WordNet edition year).
    pub version: String,
    /// `omv:URI` (Hartmann 2005) — the canonical IRI of the WN-LMF lexicon
    /// namespace the archived corpus realises. The lexical analogue of OWL's
    /// ontology IRI; grounded by OntoLex-Lemon (McCrae et al. 2017) as the
    /// lexicon↔ontology bridge identifier.
    pub lexicon_uri: String,
    /// `prov:atLocation` (Lebo 2013 §3.2) — the URL the source WN-LMF XML is
    /// published at (the registry `url`).
    pub source_url: String,
    /// `prov:wasDerivedFrom` / `prov:Entity` (Lebo 2013) content address —
    /// the content address (BLAKE3 — Aumasson, O'Connor, Neves & Wilcox-O'Hearn
    /// 2020; Dolstra 2006) of the exact source
    /// WN-LMF bytes [`read_wordnet`] consumed.
    /// The load gate validates this against the `praxis.lock` `[hashes]`
    /// pin; a mismatch fails closed.
    pub source_address: String,
    /// Count of `<Synset>` elements in the archived lexicon — a structural
    /// metric per ISO 24613 LMF / Global WordNet WN-LMF schema, the lexical
    /// analogue of `omv:numberOfClasses`. A synset IS a concept (Fellbaum
    /// 1998), so this equals the archived [`OwnedCodegenData::entity_count`].
    pub number_of_synsets: u64,
    /// Count of `<Sense>` elements across the archived lexicon's
    /// `<LexicalEntry>`s — a structural metric per ISO 24613 LMF, the lexical
    /// analogue of `omv:numberOfProperties`. A sense is one (word, synset)
    /// pairing; the total is the number of lemma↦synset edges WordNet
    /// records (Fellbaum 1998), i.e. the flat `(lemma, synset)` row count
    /// the word index is grouped from.
    pub number_of_senses: u64,
}

/// The WN-LMF 1.3 lexicon namespace (Global WordNet Association schema) —
/// the `omv:URI` a WordNet archive's [`WnPrxMetadata::lexicon_uri`] carries.
/// The published schema location for the serialization the corpus realises.
pub const WN_LMF_NAMESPACE_URI: &str = "https://globalwordnet.github.io/schemas/";

// =============================================================================
// WnGraphFaithful — the typed ontology + concrete-syntax complement, the
// graph-faithful reconstruction payload (no stored raw blob).
// =============================================================================

/// The graph-faithful reconstruction payload: the typed WN-LMF
/// [`WordNet`] ontology PLUS the concrete-syntax [`WnSyntaxComplement`] the
/// byte-exact `put` ([`reconstruct_wn_lmf_source`]) re-applies. Present in a
/// [`WordNetPrxEnvelope`] iff `mode == ByteExactGraphFaithful`.
///
/// This is the WordNet realisation of #186's graph-faithful tier: the source
/// bytes are regenerated from the ONTOLOGY GRAPH (`wn`) plus a
/// content-addressed SYNTAX residue (`complement`) — the §2.8 `<!DOCTYPE>`, the
/// root namespaces, the §2.4 inter-element white-space, the §3.1 intra-tag
/// layout, the §4.6 entity-reference form, the source attribute sequences — and
/// NO stored raw blob (the `RawBytesComplementFloor` constant-complement). The
/// complement is concrete-syntax, NOT ontology: the same `wn` serialized two
/// ways keeps one content address; only the per-source `complement` differs. The
/// capture/reconstruct pair ([`capture_wn_complement`] /
/// [`reconstruct_wn_lmf_source`]) is proven a byte-exact inverse over the real
/// 89 MB Open English WordNet 2025 corpus.
///
/// rkyv-serializable through the `prx`-gated derives on [`WordNet`] and
/// [`WnSyntaxComplement`] (and the XML/residue types they reference).
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct WnGraphFaithful {
    /// The typed WN-LMF lexicon ontology — the GRAPH the source is regenerated
    /// from. Captured by [`capture_wn_complement`] (the same `read_wordnet`
    /// model `data` projects from), serialized here directly.
    pub wn: WordNet,
    /// The concrete-syntax COMPLEMENT — the byte-affecting residue the typed
    /// ontology does not carry (DOCTYPE, namespaces, white-space layout,
    /// entity-reference form, source attribute sequences). Re-applied by
    /// [`reconstruct_wn_lmf_source`] to reproduce the source bytes exactly.
    pub complement: WnSyntaxComplement,
}

// =============================================================================
// WordNetPrxEnvelope — owned data + metadata, the thing that gets rkyv'd + gzip'd.
// =============================================================================

/// The rkyv-serializable `.prx` envelope for the English lexicon: the
/// archived synset corpus (with a POPULATED word index) plus the
/// OMV/PROV-O-grounded metadata. Serialized to rkyv bytes and gzip-wrapped
/// to form the `.prx.gz` artifact.
///
/// Structurally the OWL
/// [`PrxEnvelope`](crate::social::software::markup::xml::owl::prx::PrxEnvelope)
/// — the FLAT shape, with NO `aux` field (a WN-LMF lexicon has no
/// subdivision tree, unlike the USC `UsCodePrxEnvelope`). The one
/// substantive difference from the OWL emitter is that [`Self::data`]'s
/// `word_index` is populated and sorted (see `wn_builder_to_owned`).
///
/// # The two reconstruction tiers — one envelope, exactly one payload
///
/// `mode` selects which source-reconstruction payload the envelope carries, and
/// the two are mutually exclusive:
///
/// - [`RoundTripFidelity::ByteExactGraphFaithful`] — `graph` is `Some`, `raw`
///   is `None`: the source regenerates from the typed [`WordNet`] ontology plus
///   the concrete-syntax [`WnSyntaxComplement`] ([`WnGraphFaithful`]), NO stored
///   raw blob. This is English's tier.
/// - [`RoundTripFidelity::RawBytesComplementFloor`] — `raw` is `Some`, `graph`
///   is `None`: the source bytes are stored as a content-addressed constant
///   complement (the generic fallback for a source with no registered byte-exact
///   lens).
///
/// In both tiers [`Self::data`] (the reasoning view) is carried unchanged — the
/// runtime materializes [`English`] from it identically regardless of the
/// reconstruction tier.
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct WordNetPrxEnvelope {
    /// OMV/PROV-O-grounded self-description, incl. the source content hash
    /// the load gate validates.
    pub metadata: WnPrxMetadata,
    /// The archived synset corpus — the owned mirror of the
    /// `CodegenData<English>` interchange, with `word_index` populated. The
    /// runtime reasoning view, carried unchanged in both reconstruction tiers.
    pub data: OwnedCodegenData,
    /// The source lens's [`RoundTripFidelity`] — `ByteExactGraphFaithful` for
    /// English (the typed ontology + concrete-syntax complement
    /// regenerate the source from the graph alone).
    pub mode: RoundTripFidelity,
    /// The graph-faithful reconstruction payload (typed ontology + concrete-
    /// syntax complement) — `Some` iff `mode == ByteExactGraphFaithful`, `None`
    /// otherwise (the floor stores `raw` instead). No raw blob is kept in this
    /// tier; the source is regenerated from the graph.
    pub graph: Option<WnGraphFaithful>,
    /// The content-addressed source bytes (the constant-complement) — `Some`
    /// iff `mode == RawBytesComplementFloor`. `None` in the graph-faithful tier.
    pub raw: Option<RawSource>,
}

// =============================================================================
// rkyv layer — WordNet envelope ⇄ bytes (bytecheck-validated).
// =============================================================================

/// Serialize a WordNet envelope to rkyv bytes (the lens *put*).
/// Deterministic — equal envelopes yield equal bytes (the sorted word index
/// pins the layout), so the blob's digest is a stable `MerkleRoot` content
/// address.
pub fn wordnet_envelope_to_bytes(envelope: &WordNetPrxEnvelope) -> Result<Vec<u8>, PrxError> {
    rkyv::to_bytes::<rkyv::rancor::Error>(envelope)
        .map(|v| v.to_vec())
        .map_err(|e| PrxError::Rkyv(e.to_string()))
}

/// Materialize a WordNet envelope from rkyv bytes (the lens *get*). Copies
/// into an aligned buffer, then `bytecheck`-validates before materializing,
/// so a corrupted blob fails closed.
pub fn wordnet_envelope_from_bytes(bytes: &[u8]) -> Result<WordNetPrxEnvelope, PrxError> {
    let mut aligned = rkyv::util::AlignedVec::<16>::new();
    aligned.extend_from_slice(bytes);
    rkyv::from_bytes::<WordNetPrxEnvelope, rkyv::rancor::Error>(&aligned)
        .map_err(|e| PrxError::Rkyv(e.to_string()))
}

// =============================================================================
// The load gate — mirrors the OWL legs over the WordNet envelope, SHARED primitive.
// =============================================================================

/// Discharge a content-hash `IntegrityClaim` over `bytes` against a trusted
/// pin — byte-for-byte the OWL `verify_content_address` leg, re-implemented
/// here only because the OWL one is private and `&PrxEnvelope`-typed. The
/// integrity primitive [`raw_hash::verify`] is the SAME one the fetch path,
/// the OWL gate, and the USC gate use (Dolstra 2006 content-addressing; W3C
/// SRI 2016): `raw_hash::verify` re-hashes `bytes` under the pin's named algorithm
/// (the one verify leg, `hash_hex`), so the pin is checked against bytes
/// actually present, never a self-asserted label.
fn wn_verify_content_address(
    bytes: &[u8],
    trusted_pin: &LockDigest,
    key: &str,
) -> Result<(), PrxError> {
    let claim = IdentityClaim {
        concept: IdentityConcept::RawHash,
        data: trusted_pin.claim_data(),
    };
    match raw_hash::verify(&claim, bytes) {
        VerificationResult::Verified(_) => Ok(()),
        VerificationResult::Mismatch { expected, actual } => Err(PrxError::HashMismatch {
            key: key.to_string(),
            expected,
            found: actual,
        }),
        VerificationResult::Unverifiable { reason } => Err(PrxError::IntegrityUnverifiable {
            key: key.to_string(),
            reason,
        }),
    }
}

/// Reconstruct the exact source WN-LMF bytes from a WordNet envelope — the
/// `.prx → xml` leg of the #186 byte-hash invariant. Mirrors the OWL
/// `reconstruct_source` over the WordNet envelope.
///
/// For [`RoundTripFidelity::RawBytesComplementFloor`]: return the stored
/// `raw.blob` after enforcing the in-envelope honesty doctrine
/// (`address(blob) == raw.content_address == metadata.source_address`). A
/// tampered blob is rejected.
///
/// For [`RoundTripFidelity::ByteExactGraphFaithful`] (English):
/// regenerate the source from the typed [`WordNet`] ontology PLUS the
/// concrete-syntax [`WnSyntaxComplement`] carried in `graph` via
/// [`reconstruct_wn_lmf_source`] (the graph-faithful `put`, NO stored raw blob),
/// then enforce the SAME content-address honesty gate the floor arm uses — the
/// regenerated bytes MUST hash to `metadata.source_address` (the
/// `praxis.lock` `[byte_exact_signatures]` / `[hashes]` pin). A regeneration
/// that does not reproduce the pinned source is rejected ([`PrxError::HashMismatch`]),
/// fail-closed, never fabricating or returning unverified bytes.
pub fn wn_reconstruct_source(envelope: &WordNetPrxEnvelope) -> Result<Vec<u8>, PrxError> {
    let key = format!("{}@{}", envelope.metadata.name, envelope.metadata.version);
    match envelope.mode {
        RoundTripFidelity::RawBytesComplementFloor => {
            let raw = envelope
                .raw
                .as_ref()
                .ok_or_else(|| PrxError::SourceNotReconstructible {
                    reason: "RawBytesComplementFloor envelope is missing its raw source leaf"
                        .to_string(),
                })?;
            let computed = ContentAddress::of(&raw.blob).to_hex();
            if computed != raw.content_address {
                return Err(PrxError::HashMismatch {
                    key: format!("{key} (raw content address)"),
                    expected: raw.content_address.clone(),
                    found: computed,
                });
            }
            if computed != envelope.metadata.source_address {
                return Err(PrxError::HashMismatch {
                    key: format!("{key} (raw vs metadata)"),
                    expected: envelope.metadata.source_address.clone(),
                    found: computed,
                });
            }
            Ok(raw.blob.clone())
        }
        RoundTripFidelity::ByteExactGraphFaithful => {
            // The graph-faithful payload (typed ontology + concrete-syntax
            // complement) must be present — its absence is a malformed envelope,
            // not a fabrication opportunity.
            let graph =
                envelope
                    .graph
                    .as_ref()
                    .ok_or_else(|| PrxError::SourceNotReconstructible {
                        reason: "ByteExactGraphFaithful envelope is missing its graph payload \
                                 (typed ontology + concrete-syntax complement)"
                            .to_string(),
                    })?;
            // Regenerate from the GRAPH alone (no stored raw blob): the typed
            // WordNet ontology + the captured concrete-syntax complement. This
            // is the byte-exact `put` proven inverse over the real corpus.
            let bytes = reconstruct_wn_lmf_source(&graph.wn, &graph.complement).map_err(|e| {
                PrxError::SourceNotReconstructible {
                    reason: format!("graph-faithful WN-LMF reconstruction failed: {e}"),
                }
            })?;
            // The SAME honesty gate the floor arm enforces: the regenerated
            // bytes must hash to the pinned source content address. A
            // regeneration that drifts from the pinned source fails closed.
            let computed = ContentAddress::of(&bytes).to_hex();
            if computed != envelope.metadata.source_address {
                return Err(PrxError::HashMismatch {
                    key: format!("{key} (graph-faithful reconstruction vs metadata)"),
                    expected: envelope.metadata.source_address.clone(),
                    found: computed,
                });
            }
            Ok(bytes)
        }
    }
}

/// Verify the source-identity leg: reconstruct the source and bind it to the
/// trusted `SourcePin` (`praxis.lock` `[hashes]`). Mirrors OWL `verify_source_leg`.
fn wn_verify_source_leg(
    envelope: &WordNetPrxEnvelope,
    source_pin: &LockDigest,
    key: &str,
) -> Result<(), PrxError> {
    // Both tiers reconstruct the source and bind it to the trusted source pin —
    // the floor from its stored raw complement, the graph-faithful tier from the
    // ontology + concrete-syntax complement. `wn_reconstruct_source` already
    // enforces the in-envelope honesty gate (regenerated == metadata hash);
    // binding to `source_pin` additionally anchors it to the EXTERNAL
    // `praxis.lock` pin (`[hashes]` == `[byte_exact_signatures]` for English,
    // since `put(get(b)) == b` makes the round-trip hash the raw-source hash).
    let source_bytes = wn_reconstruct_source(envelope)?;
    wn_verify_content_address(&source_bytes, source_pin, key)
}

/// Admit a decoded WordNet envelope only after BOTH gate legs verify, then
/// materialize the lexicon. The fail-closed realisation of the `LoadGate`
/// concept, identical in structure to OWL `admit_validated`:
///
/// 1. **Installed-node integrity** — the `BinaryEnvelope`'s `MerkleRoot`,
///    re-derived from `rkyv_bytes`, must equal `archive_pin`. This binds the
///    whole envelope INCLUDING the word index, so a poisoned `word_index`
///    entry under a genuine source label is rejected (Merkle 1987; Benet
///    2014; W3C SRI 2016).
/// 2. **Source identity** — the carried source re-hashes to `source_pin`.
/// 3. Only on both `Verified` rebuild the `CodegenData<English>` view and
///    materialize via [`language::from_codegen`] — the SAME functor the
///    WASM/codegen consumer runs.
fn wn_admit_validated(
    rkyv_bytes: &[u8],
    envelope: WordNetPrxEnvelope,
    archive_pin: &LockDigest,
    source_pin: &LockDigest,
) -> Result<English, PrxError> {
    let key = format!("{}@{}", envelope.metadata.name, envelope.metadata.version);
    wn_verify_content_address(rkyv_bytes, archive_pin, &key)?;
    wn_verify_source_leg(&envelope, source_pin, &key)?;
    let data: CodegenData<English> = envelope.data.to_codegen_data_leaked();
    Ok(language::from_codegen(&data))
}

/// Load a WordNet `.prx.gz` blob into a materialized [`English`], gated on
/// two externally trusted pins (`praxis.lock` `[archive_signatures]` +
/// `[hashes]`). Mirrors OWL `load_prx_gz`.
pub fn load_wordnet_prx_gz(
    prx_gz: &[u8],
    archive_pin: &LockDigest,
    source_pin: &LockDigest,
) -> Result<English, PrxError> {
    let rkyv_bytes = gunzip(prx_gz)?;
    let envelope = wordnet_envelope_from_bytes(&rkyv_bytes)?;
    wn_admit_validated(&rkyv_bytes, envelope, archive_pin, source_pin)
}

/// Load a WordNet `.prx.gz` blob, reaching both pins through the live
/// registry: the `MerkleRoot` from `[archive_signatures]`, the `SourcePin`
/// from `[hashes]`, keyed by `"{name}@{version}"`. Mirrors OWL
/// `load_prx_gz_from_lock`. Fail-closed if either pin is unregistered.
pub fn load_wordnet_prx_gz_from_lock(prx_gz: &[u8]) -> Result<English, PrxError> {
    use crate::applied::data_provisioning::registry::{lock_archive_signature, lock_hashes};
    let rkyv_bytes = gunzip(prx_gz)?;
    let envelope = wordnet_envelope_from_bytes(&rkyv_bytes)?;
    let key = format!("{}@{}", envelope.metadata.name, envelope.metadata.version);
    let archive_pin = lock_archive_signature(&envelope.metadata.name, &envelope.metadata.version)
        .ok_or_else(|| PrxError::NoArchivePin { key: key.clone() })?;
    let source_pin = lock_hashes()
        .get(&key)
        .ok_or_else(|| PrxError::NoLockPin { key: key.clone() })?;
    wn_admit_validated(&rkyv_bytes, envelope, archive_pin, source_pin)
}

/// Load a WordNet `.prx.gz` blob back into the parsed [`WordNet`] ontology
/// (NOT a materialized [`English`]) — the loader a closed-class lexicon needs.
///
/// The function-word lexicon (`ClosedClassLexicon`) is consumed by
/// `language::function_words_from_lmf`, which projects the parsed `WordNet`
/// into a rich `LexicalEntry` map keyed off the human-meaningful `fw-*` synset
/// ids (`fw-definite-det`, `fw-greeting`, …). Those ids MUST survive the
/// round-trip — the compact succinct codec mints synthetic `s{i}` ids and
/// would silently collapse every determiner to `Indefinite` and every
/// interjection to `Expressive`, so this rides the rkyv `WordNetPrxEnvelope`
/// instead: [`wn_reconstruct_source`] returns the exact original source bytes
/// (fail-closed against the embedded content address — and, for a graph-faithful
/// registered lens, against the `praxis.lock` pin), which [`read_wordnet`]
/// re-parses with the `fw-*` ids intact.
///
/// This composes existing, already-axiom-witnessed pieces (envelope decode +
/// source reconstruct, the byte-exact inverse proven by
/// `wordnet_graph_faithful_reconstructs_source_byte_exact`); it adds no new
/// envelope, codec, or archive axiom — function-words is a 2nd consumer of the
/// SAME `WordNetPrxEnvelope` (the USC #271 / us_legal_lexicon pattern).
pub fn function_words_wordnet_from_prx(prx_gz: &[u8]) -> Result<WordNet, PrxError> {
    let rkyv_bytes = gunzip(prx_gz)?;
    let envelope = wordnet_envelope_from_bytes(&rkyv_bytes)?;
    let source = wn_reconstruct_source(&envelope)?;
    let text = core::str::from_utf8(&source).map_err(|e| {
        PrxError::Read(format!(
            "reconstructed function-words source is not UTF-8: {e}"
        ))
    })?;
    read_wordnet(text).map_err(|e| {
        PrxError::Read(format!(
            "reconstructed function-words source failed to parse: {e}"
        ))
    })
}

// =============================================================================
// Emit — read_wordnet → wn_builder_to_owned → envelope → rkyv → gzip.
// =============================================================================

/// Project a parsed [`WordNet`](super::ontology::WordNet) lexicon into the
/// owned archival shape [`OwnedCodegenData`], the WordNet analogue of the
/// OWL `builder_to_owned`.
///
/// The projection mirrors the authoritative build-time path
/// (`pr4xis::codegen::wordnet::parse_wordnet_xml` →
/// `pr4xis::codegen::generate::write_word_index` / `write_raw_relations`)
/// so an archive-materialized [`English`] is identity-identical to the
/// codegen-materialized one:
///
/// - **Entities** are synsets in document order; `entity_ids` = synset id,
///   `entity_kind` = the WN POS tag (`"n"` / `"v"` / `"a"` / `"r"`, the tag
///   [`language::from_codegen`] re-parses), `entity_labels` = synset id,
///   `entity_defs` = the synset's first `<Definition>` (or empty).
/// - **`word_index`** is POPULATED (where OWL leaves it empty): every
///   `(lemma, synset)` sense AND every inflected `<Form>` of an entry's
///   lemma resolves its synset id to the entity index and pushes
///   `(text, index)`, grouped by word, then **SORTED by word** — the
///   `binary_search`-on-`word_index` runtime invariant (and the
///   reproducible-`MerkleRoot` invariant).
/// - **Edges** map each WN synset relation onto the reasoning ontology
///   tables, dropping any edge whose endpoint synset is unknown (the same
///   `id_to_idx` resolution + dangling-edge drop the codegen emitter does):
///   `hypernym` / `instance_hypernym` → `taxonomy(this, target)`;
///   `holo_*` → `mereology(target, this)`; `mero_*` → `mereology(this, target)`;
///   `causes` → `causation(this, target)`; synset-level `also` (+ sense-level
///   `also`, resolved synset↦synset) → `references` (SKOS `seeAlso`, Miles &
///   Bechhofer 2009).
fn wn_builder_to_owned(wn: &super::ontology::WordNet) -> OwnedCodegenData {
    use hashbrown::HashMap;

    use super::ontology::{SenseRelationType, SynsetRelationType};

    // id → entity index, in synset document order (the codegen `id_to_idx`).
    let id_to_idx: HashMap<&str, u64> = wn
        .synsets
        .iter()
        .enumerate()
        .map(|(i, s)| (s.id.as_str(), i as u64))
        .collect();
    let resolve = |id: &str| id_to_idx.get(id).copied();

    let entity_ids: Vec<String> = wn.synsets.iter().map(|s| s.id.clone()).collect();
    let entity_kind: Vec<String> = wn
        .synsets
        .iter()
        .map(|s| s.pos.to_tag().to_string())
        .collect();
    let entity_labels: Vec<String> = entity_ids.clone();
    let entity_defs: Vec<String> = wn
        .synsets
        .iter()
        .map(|s| s.definitions.first().cloned().unwrap_or_default())
        .collect();

    // ── word index: (text, synset-index), grouped by word then SORTED ──
    // Mirror `finalize_entry`: index the lemma AND every inflected Form of
    // each entry against each synset the entry senses into. Resolution
    // through `id_to_idx` drops senses whose synset is absent (a dangling
    // reference), exactly as the codegen path does.
    let mut by_word: HashMap<String, Vec<u64>> = HashMap::new();
    for entry in &wn.entries {
        for sense in &entry.senses {
            let Some(idx) = resolve(&sense.synset) else {
                continue;
            };
            by_word
                .entry(entry.lemma.written_form.clone())
                .or_default()
                .push(idx);
            for form in &entry.forms {
                by_word
                    .entry(form.written_form.clone())
                    .or_default()
                    .push(idx);
            }
        }
    }
    // SORT by word — the binary_search + reproducible-MerkleRoot invariant.
    let mut word_index: Vec<(String, Vec<u64>)> = by_word.into_iter().collect();
    word_index.sort_by(|(a, _), (b, _)| a.cmp(b));

    // ── relation tables: map each WN relType onto the reasoning ontology ──
    let mut taxonomy: Vec<(u64, u64)> = Vec::new();
    let mut mereology: Vec<(u64, u64)> = Vec::new();
    let mut causation: Vec<(u64, u64)> = Vec::new();
    let mut references: Vec<(u64, u64)> = Vec::new();
    for synset in &wn.synsets {
        let Some(this) = resolve(&synset.id) else {
            continue;
        };
        for rel in &synset.relations {
            let Some(target) = resolve(&rel.target) else {
                continue;
            };
            match &rel.rel_type {
                // Taxonomy (is-a): child = this, parent = target.
                SynsetRelationType::Hypernym | SynsetRelationType::InstanceHypernym => {
                    taxonomy.push((this, target));
                }
                // Holonym ("this is part of target"): whole = target, part = this.
                SynsetRelationType::HoloMember
                | SynsetRelationType::HoloPart
                | SynsetRelationType::HoloSubstance => {
                    mereology.push((target, this));
                }
                // Meronym ("target is part of this"): whole = this, part = target.
                SynsetRelationType::MeroMember
                | SynsetRelationType::MeroPart
                | SynsetRelationType::MeroSubstance => {
                    mereology.push((this, target));
                }
                SynsetRelationType::Causes => {
                    causation.push((this, target));
                }
                // SKOS seeAlso (Miles & Bechhofer 2009 §8).
                SynsetRelationType::Also => {
                    references.push((this, target));
                }
                _ => {}
            }
        }
    }

    // Sense-level `also` (SKOS seeAlso) resolved synset↦synset. Mirrors
    // `parse_wordnet_xml`'s deferred sense-`also` pass: a SenseRelation
    // targets a SENSE id, so resolve both endpoints through their entry's
    // sense→synset binding, then add the synset-level reference.
    let mut sense_to_synset: HashMap<&str, &str> = HashMap::new();
    for entry in &wn.entries {
        for sense in &entry.senses {
            if !sense.id.is_empty() {
                sense_to_synset.insert(sense.id.as_str(), sense.synset.as_str());
            }
        }
    }
    for entry in &wn.entries {
        for sense in &entry.senses {
            for rel in &sense.relations {
                if rel.rel_type == SenseRelationType::Also
                    && let (Some(&src_syn), Some(&tgt_syn)) = (
                        sense_to_synset.get(sense.id.as_str()),
                        sense_to_synset.get(rel.target.as_str()),
                    )
                    && let (Some(s), Some(t)) = (resolve(src_syn), resolve(tgt_syn))
                {
                    references.push((s, t));
                }
            }
        }
    }

    OwnedCodegenData {
        entity_count: entity_ids.len() as u64,
        entity_ids,
        entity_kind,
        entity_labels,
        entity_defs,
        word_index,
        taxonomy,
        mereology,
        opposition: Vec::new(),
        equivalence: Vec::new(),
        causation,
        references,
    }
}

/// Build a [`WordNetPrxEnvelope`] from WN-LMF source bytes plus its registry
/// `(name, version, url)`, preferring the GRAPH-FAITHFUL tier — English's tier —
/// and gracefully degrading to the universal floor only for a
/// source whose concrete syntax the structural writer cannot yet reproduce.
///
/// 1. Parse the typed [`WordNet`] ontology once ([`read_wordnet`]) and project
///    the [`OwnedCodegenData`] reasoning view (`word_index` populated + sorted).
///    This is carried unchanged in BOTH tiers.
/// 2. ATTEMPT the graph-faithful `get`: [`capture_wn_complement`] re-parses the
///    source and captures the concrete-syntax [`WnSyntaxComplement`] (DOCTYPE,
///    namespaces, white-space layout, entity-reference form, source attribute
///    sequences) the byte-exact `put` re-applies.
///    - **Captured** → emit `mode = ByteExactGraphFaithful`, the `graph` payload
///      (ontology + complement), `raw = None` (NO stored raw blob; the source
///      regenerates from the graph). This is English's tier.
///    - **Backbone divergence** ([`WnReconstructError::Complement`](crate::social::software::markup::xml::lmf::writer::WnReconstructError::Complement)) → the
///      structural writer cannot yet regenerate THIS source's element backbone
///      (e.g. a WN-LMF lexicon whose `<LexicalEntry>`/`<Synset>` child order the
///      DTD-ordered writer reorders). Degrade HONESTLY to the universal floor:
///      emit `mode = RawBytesComplementFloor`, `graph = None`, `raw =` the
///      content-addressed source blob — the same constant-complement generic
///      fallback OWL + USC degrade to when their writer cannot reproduce a
///      source. NEVER a silent lie: the floor tier is explicit in `mode`, and the
///      completeness meter only declares a source graph-faithful when a lens is
///      registered for it.
///    - **Malformed source** ([`WnReconstructError::Parse`](crate::social::software::markup::xml::lmf::writer::WnReconstructError::Parse)) → a hard error;
///      a non-well-formed WN-LMF file is a defect, not a floor candidate.
///
/// The OMV/PROV-O metadata's `source_address` is the content address of the exact
/// source bytes (the `[hashes]` / `[byte_exact_signatures]` pin), against which
/// [`wn_reconstruct_source`] gates the regenerated bytes fail-closed in BOTH
/// tiers.
pub fn build_wordnet_envelope(
    source: &[u8],
    name: &str,
    version: &str,
    url: &str,
) -> Result<WordNetPrxEnvelope, PrxError> {
    use super::writer::WnReconstructError;

    let text = core::str::from_utf8(source)
        .map_err(|e| PrxError::Read(format!("source is not UTF-8: {e}")))?;

    // Parse the typed ontology once — the reasoning view is projected from it in
    // both tiers, so the materialized `English` is identical either way.
    let wn = read_wordnet(text).map_err(|e| PrxError::Read(format!("{e}")))?;
    let number_of_synsets = wn.synsets.len() as u64;
    // A sense is one (lemma, synset) pairing recorded under a LexicalEntry
    // (ISO 24613 LMF; Fellbaum 1998).
    let number_of_senses = wn.entries.iter().map(|e| e.senses.len()).sum::<usize>() as u64;
    let data = wn_builder_to_owned(&wn);

    let source_address = ContentAddress::of(source).to_hex();
    let metadata = WnPrxMetadata {
        name: name.to_string(),
        version: version.to_string(),
        lexicon_uri: WN_LMF_NAMESPACE_URI.to_string(),
        source_url: url.to_string(),
        source_address: source_address.clone(),
        number_of_synsets,
        number_of_senses,
    };

    // The emitted tier MUST agree with the completeness meter's DECLARED tier —
    // a successful `capture_wn_complement` is necessary but NOT sufficient to
    // claim the graph-faithful tier. The meter declares a source graph-faithful
    // ONLY when a graph-faithful lens is REGISTERED for `(name, version)` (it
    // reads `lens_registrations().find(name@version).fidelity`). We consult THAT
    // SAME registry here so emit-tier == meter-tier: `english_wordnet`
    // (registered `WordNetLmfLens`) qualifies; `us_legal_lexicon` (no
    // registration) rides the floor even when capture succeeds — matching the
    // meter (and mirroring `build_usc_envelope`'s registry gate). The lens
    // registry is native-only (`linkme`); emit is a native `prx`/`fetch` path,
    // so the lookup is sound here.
    let registered_graph_faithful =
        crate::formal::meta::well_behaved_lens::lens_by_name(&format!("{name}@{version}"))
            .is_some_and(|r| r.fidelity == RoundTripFidelity::ByteExactGraphFaithful);

    // Attempt the graph-faithful capture. Prefer it ONLY for a registered
    // graph-faithful source; otherwise degrade to the floor even on a successful
    // capture. We still also degrade on a structural-writer backbone divergence.
    match capture_wn_complement(text) {
        Ok((wn_captured, complement)) if registered_graph_faithful => Ok(WordNetPrxEnvelope {
            metadata,
            data,
            mode: RoundTripFidelity::ByteExactGraphFaithful,
            graph: Some(WnGraphFaithful {
                wn: wn_captured,
                complement,
            }),
            raw: None,
        }),
        // Capture SUCCEEDED but no graph-faithful lens is registered for this
        // source — the meter declares it FLOOR, so emit the floor (the raw
        // content-addressed blob), keeping emit-tier == meter-tier.
        Ok(_) => Ok(WordNetPrxEnvelope {
            metadata,
            data,
            mode: RoundTripFidelity::RawBytesComplementFloor,
            graph: None,
            raw: Some(RawSource {
                content_address: source_address,
                blob: source.to_vec(),
            }),
        }),
        // The structural writer cannot reproduce this source's backbone — ride
        // the universal floor (the content-addressed raw blob), honestly tiered.
        Err(WnReconstructError::Complement(_)) => Ok(WordNetPrxEnvelope {
            metadata,
            data,
            mode: RoundTripFidelity::RawBytesComplementFloor,
            graph: None,
            raw: Some(RawSource {
                content_address: source_address,
                blob: source.to_vec(),
            }),
        }),
        // A malformed WN-LMF source is a defect, not a floor candidate.
        Err(e @ WnReconstructError::Parse(_)) => {
            Err(PrxError::Read(format!("graph-faithful capture: {e}")))
        }
    }
}

/// Emit a WordNet `.prx.gz` artifact from WN-LMF source bytes:
/// `build_wordnet_envelope → wordnet_envelope_to_bytes (rkyv) → gzip`.
pub fn emit_wordnet_prx_gz(
    source: &[u8],
    name: &str,
    version: &str,
    url: &str,
) -> Result<Vec<u8>, PrxError> {
    let envelope = build_wordnet_envelope(source, name, version, url)?;
    let rkyv_bytes = wordnet_envelope_to_bytes(&envelope)?;
    gzip(&rkyv_bytes)
}

/// Workspace root — the grandparent of `CARGO_MANIFEST_DIR` (`crates/domains/`),
/// against which registry `local_path()`s and `praxis.lock` resolve. Mirrors
/// the OWL + USC emitters.
fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

/// Emit a WordNet `.prx.gz` artifact for EVERY registered
/// [`Language`][lang]-kind source on disk into `out_dir`,
/// round-trip-validating each before returning it — the WordNet analogue of
/// `owl::prx::emit_all_prx_gz` / `usc::prx::emit_all_usc_prx_gz`, the
/// release-distribution entry point. Registry-driven (never a hardcoded
/// lexicon set): it walks every entry [`data_sources`][ds] reports whose
/// `kind` is [`SourceTaxonomyConcept::Language`][lang] (a Lexicon for a
/// natural language — Vossen 1998), reads the bundled WN-LMF XML from
/// `workspace_root.join(entry.local_path())`, and for each:
///
/// 1. `emit_wordnet_prx_gz(source, name, version, url)` → the gzip-wrapped
///    rkyv envelope;
/// 2. writes it to `<out_dir>/<name>-<version>.prx.gz`;
/// 3. **round-trip-validates** by reading the file back through the
///    fail-closed gate against the `MerkleRoot` this emit just produced and
///    the source's `praxis.lock` `[hashes]` pin — proving the published
///    artifact is loadable, content-anchored, and source-faithful (the
///    GetPut leg of the bytes ⇄ lexicon lens) before the operator pins its
///    `archive_address` into `[archive_signatures]`.
///
/// A registered lexicon whose XML is NOT on disk is skipped gracefully — the
/// same graceful skip `loaded_vocabularies` and the USC corpus loader make.
/// A lexicon that emits but fails to round-trip-load is a defect (Err), never
/// a skip.
///
/// [lang]: crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept::Language
/// [ds]: crate::applied::data_provisioning::registry::data_sources
pub fn emit_all_wordnet_prx_gz(
    out_dir: &std::path::Path,
) -> Result<Vec<EmittedArtifact>, PrxError> {
    use crate::applied::data_provisioning::registry::{data_sources, lock_hashes};
    use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;

    std::fs::create_dir_all(out_dir)
        .map_err(|e| PrxError::Gzip(format!("create out_dir {}: {e}", out_dir.display())))?;
    let root = workspace_root();
    let mut emitted = Vec::new();
    for entry in data_sources() {
        if entry.kind != SourceTaxonomyConcept::Language {
            continue;
        }
        let src_path = root.join(entry.local_path());
        let Ok(source) = std::fs::read(&src_path) else {
            // Registered but not on disk — skip gracefully.
            continue;
        };
        let prx_gz = emit_wordnet_prx_gz(&source, &entry.name, &entry.version, &entry.url)?;
        let archive_address = prx_archive_address(&prx_gz)?;
        let path = out_dir.join(format!("{}-{}.prx.gz", entry.name, entry.version));
        std::fs::write(&path, &prx_gz)
            .map_err(|e| PrxError::Gzip(format!("write {}: {e}", path.display())))?;

        let key = format!("{}@{}", entry.name, entry.version);
        let source_pin = lock_hashes()
            .get(&key)
            .ok_or_else(|| PrxError::NoLockPin { key: key.clone() })?;
        let read_back = std::fs::read(&path)
            .map_err(|e| PrxError::Gzip(format!("read-back {}: {e}", path.display())))?;
        load_wordnet_prx_gz(
            &read_back,
            &LockDigest::address(archive_address.clone()),
            source_pin,
        )?;

        emitted.push(EmittedArtifact {
            name: entry.name.clone(),
            version: entry.version.clone(),
            path,
            byte_len: prx_gz.len() as u64,
            archive_address,
        });
    }
    Ok(emitted)
}

// =============================================================================
// Compact English `.prx` — the portable, content-address-gated fast path that
// `english_loaded()` reads. The WordNet analogue of the USC compact-archive
// infra (`uslm::corpus::prx`): the same succinct CODEC the wasm runtime uses,
// wrapped here in a `[compact_archive_signatures]` content gate + an on-disk
// cache. (The wasm build embeds the succinct bytes directly under its own
// Merkle-root gate; it does not yet go through this compact-archive pin.)
// =============================================================================

/// Emit a COMPACT English `.prx.gz` from WN-LMF source bytes: parse → succinct
/// encode → gzip. The portable, dependency-free sibling of
/// [`emit_wordnet_prx_gz`] (no rkyv envelope) — the bytes `english_loaded()`
/// loads through the content gate.
pub fn emit_compact_english_prx_gz(source: &[u8]) -> Result<Vec<u8>, PrxError> {
    let text = core::str::from_utf8(source)
        .map_err(|e| PrxError::Read(format!("WN-LMF source is not UTF-8: {e}")))?;
    let wn = read_wordnet(text).map_err(|e| PrxError::Read(format!("{e}")))?;
    Ok(super::compact_succinct::emit_prx_gz(&wn))
}

/// The content address of a compact English `.prx.gz` — the digest of its
/// uncompressed succinct bytes (gzip-level-independent), as 64-char lowercase
/// hex. The value pinned in `praxis.lock` `[compact_archive_signatures]` and the
/// one [`load_compact_english_prx_gz_gated`] re-derives and verifies. Portable:
/// the succinct codec is dependency-free bit-packing, stable across toolchains
/// and targets (unlike the rkyv [`prx_archive_address`]).
pub fn compact_english_archive_address(cprx_gz: &[u8]) -> Result<String, PrxError> {
    Ok(ContentAddress::of(&gunzip(cprx_gz)?).to_hex())
}

/// Load a compact English `.prx.gz` into a materialized [`English`] through the
/// fail-closed content-address gate: gunzip → verify the succinct bytes hash
/// to `archive_pin` (the `[compact_archive_signatures]` pin) → succinct-decode →
/// `English::from_wordnet`. A compact archive whose bytes do not match the pin
/// is rejected before any data is installed (Dolstra 2006 content-addressing;
/// W3C SRI 2016). The portable, no-source-reconstruction sibling of
/// [`load_wordnet_prx_gz`].
pub fn load_compact_english_prx_gz_gated(
    cprx_gz: &[u8],
    archive_pin: &LockDigest,
    key: &str,
) -> Result<English, PrxError> {
    let raw = gunzip(cprx_gz)?;
    wn_verify_content_address(&raw, archive_pin, key)?;
    Ok(English::from_wordnet(&super::compact::decode(
        &super::compact_succinct::from_succinct(&raw),
    )))
}

/// The compiled COMPACT-English-archive cache directory:
/// `<workspace_root>/.prx-cache/wordnet-compact`. `pr4xis compile --compact`
/// writes one `{name}-{version}.cprx.gz` here per registered `Language` source,
/// and [`english_loaded`](crate::cognitive::linguistics::english::english_loaded)
/// reads it as its content-address-gated fast path (falling back to the WN-LMF
/// XML parse when absent). Gitignored build output — never committed.
pub fn english_compact_prx_cache_dir(workspace_root: &std::path::Path) -> std::path::PathBuf {
    workspace_root.join(".prx-cache").join("wordnet-compact")
}

/// Emit a compact English `.prx.gz` for EVERY registered
/// [`Language`][lang]-kind source on disk into `out_dir`,
/// round-trip-validating each (the emitted bytes load back through the content
/// gate) before returning it. Registry-driven; a source not on disk is skipped
/// gracefully — the same discipline `emit_all_wordnet_prx_gz` follows.
///
/// [lang]: crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept::Language
pub fn emit_all_compact_english_prx_gz(
    out_dir: &std::path::Path,
) -> Result<Vec<EmittedArtifact>, PrxError> {
    use crate::applied::data_provisioning::registry::data_sources;
    use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;

    std::fs::create_dir_all(out_dir)
        .map_err(|e| PrxError::Gzip(format!("create out_dir {}: {e}", out_dir.display())))?;
    let root = workspace_root();
    let mut emitted = Vec::new();
    for entry in data_sources() {
        if entry.kind != SourceTaxonomyConcept::Language {
            continue;
        }
        let src_path = root.join(entry.local_path());
        let Ok(source) = std::fs::read(&src_path) else {
            continue;
        };
        let cprx_gz = emit_compact_english_prx_gz(&source)?;
        let archive_address = compact_english_archive_address(&cprx_gz)?;
        let path = out_dir.join(format!("{}-{}.cprx.gz", entry.name, entry.version));
        std::fs::write(&path, &cprx_gz)
            .map_err(|e| PrxError::Gzip(format!("write {}: {e}", path.display())))?;

        // Round-trip-validate against the address this emit just produced.
        let key = format!("{}@{}", entry.name, entry.version);
        let read_back = std::fs::read(&path)
            .map_err(|e| PrxError::Gzip(format!("read-back {}: {e}", path.display())))?;
        load_compact_english_prx_gz_gated(
            &read_back,
            &LockDigest::address(archive_address.clone()),
            &key,
        )?;

        emitted.push(EmittedArtifact {
            name: entry.name.clone(),
            version: entry.version.clone(),
            path,
            byte_len: cprx_gz.len() as u64,
            archive_address,
        });
    }
    Ok(emitted)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal but full-shape WN-LMF lexicon: `<LexicalResource>` wrapper,
    /// one `<Lexicon>`, a small dog/cat/mammal/animal taxonomy plus a verb
    /// and an adjective, with `<Sense>`s wiring lemmas to synsets and a
    /// `hypernym` chain. `read_wordnet` accepts it; emit projects it through
    /// `wn_builder_to_owned`. Mirrors `English::sample`'s inline fixture so
    /// the corpus-equality test has a known reference.
    ///
    /// Written in REAL Open English WordNet 2025 SHAPE so it is byte-exactly
    /// CAPTURABLE by [`capture_wn_complement`] (the graph-faithful emit):
    /// a `<!DOCTYPE>`, the root `xmlns:dc` declaration, two-space inter-element
    /// indentation, and DTD-ordered children (`Lemma, Form*, Sense*` per
    /// `<LexicalEntry>`; `Definition, …, SynsetRelation*` per `<Synset>`). The
    /// structural writer regenerates this exact backbone, so the complement is a
    /// pure white-space/decl residue and `reconstruct == source` byte-for-byte.
    const SAMPLE_WN_LMF: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE LexicalResource SYSTEM \"http://globalwordnet.github.io/schemas/WN-LMF-1.3.dtd\">\n\
<LexicalResource xmlns:dc=\"https://globalwordnet.github.io/schemas/dc/\">\n\
  <Lexicon id=\"test-en\" label=\"Test English\" language=\"en\" email=\"\" license=\"\" version=\"1.0\" url=\"\">\n\
    <LexicalEntry id=\"e-dog-n\">\n\
      <Lemma writtenForm=\"dog\" partOfSpeech=\"n\"/>\n\
      <Form writtenForm=\"dogs\"/>\n\
      <Sense id=\"dog-n-01\" synset=\"s-dog\"/>\n\
    </LexicalEntry>\n\
    <LexicalEntry id=\"e-cat-n\">\n\
      <Lemma writtenForm=\"cat\" partOfSpeech=\"n\"/>\n\
      <Sense id=\"cat-n-01\" synset=\"s-cat\"/>\n\
    </LexicalEntry>\n\
    <LexicalEntry id=\"e-mammal-n\">\n\
      <Lemma writtenForm=\"mammal\" partOfSpeech=\"n\"/>\n\
      <Sense id=\"mammal-n-01\" synset=\"s-mammal\"/>\n\
    </LexicalEntry>\n\
    <LexicalEntry id=\"e-animal-n\">\n\
      <Lemma writtenForm=\"animal\" partOfSpeech=\"n\"/>\n\
      <Sense id=\"animal-n-01\" synset=\"s-animal\"/>\n\
    </LexicalEntry>\n\
    <LexicalEntry id=\"e-run-v\">\n\
      <Lemma writtenForm=\"run\" partOfSpeech=\"v\"/>\n\
      <Sense id=\"run-v-01\" synset=\"s-run\"/>\n\
    </LexicalEntry>\n\
    <LexicalEntry id=\"e-big-a\">\n\
      <Lemma writtenForm=\"big\" partOfSpeech=\"a\"/>\n\
      <Sense id=\"big-a-01\" synset=\"s-big\"/>\n\
    </LexicalEntry>\n\
    <Synset id=\"s-dog\" ili=\"i1\" partOfSpeech=\"n\">\n\
      <Definition>a domesticated canine</Definition>\n\
      <SynsetRelation relType=\"hypernym\" target=\"s-mammal\"/>\n\
    </Synset>\n\
    <Synset id=\"s-cat\" ili=\"i2\" partOfSpeech=\"n\">\n\
      <Definition>a small feline</Definition>\n\
      <SynsetRelation relType=\"hypernym\" target=\"s-mammal\"/>\n\
    </Synset>\n\
    <Synset id=\"s-mammal\" ili=\"i3\" partOfSpeech=\"n\">\n\
      <Definition>warm-blooded vertebrate</Definition>\n\
      <SynsetRelation relType=\"hypernym\" target=\"s-animal\"/>\n\
    </Synset>\n\
    <Synset id=\"s-animal\" ili=\"i4\" partOfSpeech=\"n\">\n\
      <Definition>a living organism</Definition>\n\
    </Synset>\n\
    <Synset id=\"s-run\" ili=\"i5\" partOfSpeech=\"v\">\n\
      <Definition>move fast on foot</Definition>\n\
    </Synset>\n\
    <Synset id=\"s-big\" ili=\"i7\" partOfSpeech=\"a\">\n\
      <Definition>of considerable size</Definition>\n\
    </Synset>\n\
  </Lexicon>\n\
</LexicalResource>\n";

    const FX_NAME: &str = "english_wordnet";
    const FX_VERSION: &str = "2025";
    const FX_URL: &str = "https://github.com/globalwordnet/english-wordnet/releases/download/2025-edition/english-wordnet-2025.xml.gz";

    // ── interned-complete measurement substrate (words+meanings vs connections) ──

    // ── envelope / gate round-trip + determinism ──────────────────────

    /// to_bytes ∘ from_bytes is identity (the GetPut leg, incl. the word
    /// index), and emitting twice gives byte-identical `.prx.gz` — proving
    /// the `word_index` sort makes the rkyv layout process-deterministic (no
    /// HashMap iteration order leaks into the bytes).
    /// The compact English `.prx.gz` round-trips through the content gate to an
    /// `English` equal to `from_wordnet`, and the gate REJECTS bytes whose hash
    /// does not match the pinned content address (fail-closed). Inline sample —
    /// the giant-corpus compactness gate lives in praxis-corpus-tests.
    #[pr4xis::praxis_value(Deterministic, Honest)]
    #[test]
    fn compact_english_prx_gz_round_trips_and_gate_rejects_tampering() {
        let wn = read_wordnet(SAMPLE_WN_LMF).expect("parse sample");
        let reference = English::from_wordnet(&wn);

        let cprx_gz = emit_compact_english_prx_gz(SAMPLE_WN_LMF.as_bytes()).expect("emit compact");
        let addr = compact_english_archive_address(&cprx_gz).expect("address");
        let key = "english_wordnet@2025";

        let loaded = load_compact_english_prx_gz_gated(&cprx_gz, &LockDigest::address(addr), key)
            .expect("gated load succeeds");
        assert_eq!(
            loaded.concept_count(),
            reference.concept_count(),
            "compact-loaded English concept_count differs from from_wordnet"
        );
        assert_eq!(
            loaded.word_index, reference.word_index,
            "compact-loaded English word→concept index differs from from_wordnet"
        );

        // A wrong pin is rejected before any data is installed.
        let wrong_pin = LockDigest::address("0".repeat(64));
        assert!(
            load_compact_english_prx_gz_gated(&cprx_gz, &wrong_pin, key).is_err(),
            "the content gate must reject a compact archive whose hash != the pin"
        );
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn wordnet_envelope_bytes_round_trip_and_deterministic() {
        let envelope =
            build_wordnet_envelope(SAMPLE_WN_LMF.as_bytes(), FX_NAME, FX_VERSION, FX_URL)
                .expect("build envelope");
        let a = wordnet_envelope_to_bytes(&envelope).expect("serialize a");
        let b = wordnet_envelope_to_bytes(&envelope).expect("serialize b");
        assert_eq!(a, b, "rkyv serialization must be deterministic");
        let back = wordnet_envelope_from_bytes(&a).expect("deserialize");
        assert_eq!(envelope, back, "rkyv round-trip must be lossless");

        // word_index is populated (the WordNet delta from the OWL emitter)
        // and sorted (the binary_search invariant).
        assert!(
            !envelope.data.word_index.is_empty(),
            "WordNet must populate word_index"
        );
        assert!(
            envelope
                .data
                .word_index
                .windows(2)
                .all(|w| w[0].0 <= w[1].0),
            "word_index must be sorted by word"
        );

        // Two independent emits over the same source → byte-identical .prx.gz.
        let first = emit_wordnet_prx_gz(SAMPLE_WN_LMF.as_bytes(), FX_NAME, FX_VERSION, FX_URL)
            .expect("emit first");
        let second = emit_wordnet_prx_gz(SAMPLE_WN_LMF.as_bytes(), FX_NAME, FX_VERSION, FX_URL)
            .expect("emit second");
        assert_eq!(
            first, second,
            "two independent emits must yield byte-identical .prx.gz"
        );
    }

    // ── round-trip fidelity: emit → load == from_wordnet corpus ───────

    /// emit → load (genuine pins computed from the emit) reconstructs the
    /// lexicon value: the materialized [`English`] equals the
    /// `English::from_wordnet` one on `concept_count`, `word_count`, and a
    /// few lookups (dog → its synset, the hypernym chain). Both functors
    /// (`from_wordnet`, `from_codegen`) index lemmas + Forms against synset
    /// concepts, so the metrics agree.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn wordnet_emit_then_load_equals_corpus() {
        let src = SAMPLE_WN_LMF.as_bytes();

        // The reference corpus: read_wordnet → English::from_wordnet.
        let wn = read_wordnet(SAMPLE_WN_LMF).expect("parse sample WN-LMF");
        let reference = English::from_wordnet(&wn);

        // The archived corpus: emit → load → from_codegen.
        let prx_gz = emit_wordnet_prx_gz(src, FX_NAME, FX_VERSION, FX_URL).expect("emit");
        let archive_pin =
            LockDigest::address(prx_archive_address(&prx_gz).expect("archive address"));
        let source_pin = LockDigest::address(ContentAddress::of(src).to_hex());
        let loaded =
            load_wordnet_prx_gz(&prx_gz, &archive_pin, &source_pin).expect("load + validate");

        // Concept / word counts match the from_wordnet corpus.
        assert_eq!(
            loaded.concept_count(),
            reference.concept_count(),
            "concept_count must survive the archive"
        );
        assert_eq!(
            loaded.word_count(),
            reference.word_count(),
            "word_count must survive the archive"
        );
        assert_eq!(loaded.concept_count(), 6, "six synsets in the sample");

        // Lookups resolve to the same concept (by synset id, order-stable).
        for word in ["dog", "cat", "mammal", "animal", "run", "big", "dogs"] {
            let in_archive: Vec<&str> = loaded
                .lookup(word)
                .iter()
                .filter_map(|&c| loaded.concept(c).map(|x| x.original_id.as_str()))
                .collect();
            let in_reference: Vec<&str> = reference
                .lookup(word)
                .iter()
                .filter_map(|&c| reference.concept(c).map(|x| x.original_id.as_str()))
                .collect();
            assert_eq!(
                in_archive, in_reference,
                "lookup({word}) must resolve to the same synsets after the archive"
            );
        }

        // The hypernym chain survives: dog is-a mammal is-a animal.
        let dog = loaded.concept_by_synset("s-dog").expect("dog synset").id;
        let mammal = loaded
            .concept_by_synset("s-mammal")
            .expect("mammal synset")
            .id;
        let animal = loaded
            .concept_by_synset("s-animal")
            .expect("animal synset")
            .id;
        assert!(loaded.is_a(dog, mammal), "dog is_a mammal");
        assert!(loaded.is_a(dog, animal), "dog is_a animal (transitive)");
    }

    // ── graph-faithful leaf: .prx → source byte-exact from the graph (#186) ─

    /// The graph-faithful emit carries the typed ontology + concrete-syntax
    /// complement (NO raw blob), and `wn_reconstruct_source` regenerates the
    /// EXACT source bytes from the GRAPH alone — praxis's first byte-exact
    /// graph-faithful `.prx` source.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn wordnet_graph_faithful_reconstructs_source_byte_exact() {
        let envelope =
            build_wordnet_envelope(SAMPLE_WN_LMF.as_bytes(), FX_NAME, FX_VERSION, FX_URL)
                .expect("build envelope");
        // The tier is graph-faithful: graph payload present, NO raw blob.
        assert_eq!(envelope.mode, RoundTripFidelity::ByteExactGraphFaithful);
        assert!(
            envelope.graph.is_some(),
            "graph-faithful envelope carries the ontology + complement payload"
        );
        assert!(
            envelope.raw.is_none(),
            "graph-faithful envelope stores NO raw blob"
        );
        let src = wn_reconstruct_source(&envelope).expect("reconstruct");
        assert_eq!(
            src,
            SAMPLE_WN_LMF.as_bytes(),
            "wn_reconstruct_source must regenerate the exact source bytes from the graph"
        );
        assert_eq!(
            ContentAddress::of(&src).to_hex(),
            envelope.metadata.source_address,
            "reconstructed bytes must hash to the pinned source content address"
        );
    }

    /// Fail-closed: a graph-faithful envelope with no graph payload cannot
    /// reconstruct its source — `wn_reconstruct_source` refuses `graph = None`
    /// rather than fabricating bytes.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn wordnet_reconstruct_refuses_missing_graph_payload() {
        let mut envelope =
            build_wordnet_envelope(SAMPLE_WN_LMF.as_bytes(), FX_NAME, FX_VERSION, FX_URL)
                .expect("build envelope");
        envelope.graph = None;
        let err = wn_reconstruct_source(&envelope)
            .expect_err("graph-faithful envelope without its payload must be rejected");
        assert!(
            matches!(err, PrxError::SourceNotReconstructible { .. }),
            "got {err:?}"
        );
    }

    /// Fail-closed: a tampered complement (one whose reconstruction no longer
    /// hashes to the pinned source content address) is rejected by the in-
    /// envelope honesty gate rather than returning wrong bytes. Here we corrupt
    /// the metadata pin so the (correct) regenerated bytes fail the gate — the
    /// same fail-closed behaviour the floor arm's tampered-blob test asserts.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn wordnet_graph_faithful_rejects_pin_drift() {
        let mut envelope =
            build_wordnet_envelope(SAMPLE_WN_LMF.as_bytes(), FX_NAME, FX_VERSION, FX_URL)
                .expect("build envelope");
        // Drift the pinned source hash: the graph still reconstructs the true
        // source, but it no longer matches the (now-wrong) metadata pin, so the
        // honesty gate must refuse it rather than return unverified bytes.
        envelope.metadata.source_address = "0".repeat(64);
        let err = wn_reconstruct_source(&envelope)
            .expect_err("pin drift must fail closed (HashMismatch)");
        assert!(matches!(err, PrxError::HashMismatch { .. }), "got {err:?}");
    }

    // ── load gate: teeth on a poisoned word_index ─────────────────────

    /// The MerkleRoot leg's reason to exist: an envelope carrying a genuine
    /// source label + honest graph payload but ONE poisoned `word_index` entry
    /// is rejected. The source leg alone would pass (the graph still
    /// reconstructs the true source); the MerkleRoot leg binds the whole
    /// installed node — including the word index — so poisoning it changes the
    /// content address and the gate refuses it. Mirrors OWL's
    /// `load_rejects_poisoned_data_under_honest_label`.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn wordnet_load_rejects_poisoned_word_index_under_honest_label() {
        let honest = build_wordnet_envelope(SAMPLE_WN_LMF.as_bytes(), FX_NAME, FX_VERSION, FX_URL)
            .expect("build envelope");
        let honest_archive_pin = LockDigest::address(
            ContentAddress::of(&wordnet_envelope_to_bytes(&honest).unwrap()).to_hex(),
        );
        let source_pin = LockDigest::address(honest.metadata.source_address.clone());

        // Same genuine source label + graph payload, only one word_index entry poisoned.
        let mut poisoned = honest;
        assert!(!poisoned.data.word_index.is_empty());
        poisoned.data.word_index[0].0 = "POISON".to_string();
        let gz = gzip(&wordnet_envelope_to_bytes(&poisoned).unwrap()).expect("gzip");

        let err = load_wordnet_prx_gz(&gz, &honest_archive_pin, &source_pin)
            .expect_err("poisoned word_index must be rejected by the MerkleRoot leg");
        assert!(
            matches!(err, PrxError::HashMismatch { .. }),
            "expected HashMismatch from the MerkleRoot leg, got {err:?}"
        );
    }

    // ── load gate: corrupted blob fails closed ────────────────────────

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn wordnet_load_rejects_corrupted_blob() {
        let any = LockDigest::address("0".repeat(64));
        let garbage = gzip(b"not a valid WordNet rkyv envelope at all").expect("gzip");
        let err = load_wordnet_prx_gz(&garbage, &any, &any).expect_err("garbage rkyv must fail");
        assert!(matches!(err, PrxError::Rkyv(_)), "got {err:?}");
    }

    // ── lock gate: unpinned source fails closed ───────────────────────

    /// The lock-driven load path fails closed for a WordNet envelope whose
    /// `"{name}@{version}"` has no `[archive_signatures]` pin — the
    /// MerkleRoot pin is looked up first, so an unregistered lexicon is
    /// refused there (mirrors OWL `load_validation_rejects_unpinned_source`).
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn wordnet_load_from_lock_rejects_unpinned() {
        let prx_gz = emit_wordnet_prx_gz(
            SAMPLE_WN_LMF.as_bytes(),
            "not_a_registered_lexicon",
            "9.9.9",
            FX_URL,
        )
        .expect("emit");
        let err = load_wordnet_prx_gz_from_lock(&prx_gz)
            .expect_err("unpinned WordNet source must be rejected");
        assert!(matches!(err, PrxError::NoArchivePin { .. }), "got {err:?}");
    }

    // ── archive anchor: every emitted WordNet .prx matches its lock pin ─

    // The WordNet `.prx` archive-anchor gate (every emitted archive's MerkleRoot
    // equals its `praxis.lock` `[archive_signatures]` pin, over the `Language`
    // partition) is a heavy producer — it re-emits the 89 MB English lexicon
    // `.prx` — so it lives in the heavy-corpus lane: see
    // `crates/praxis-corpus-tests/tests/wordnet.rs::wordnet_archive_anchors_match_lock`.
    // One process there parses the corpus once; the fast nextest lane no longer
    // pays the re-emit per process-isolated test. The pinning helper below stays.

    /// One-shot helper: print the MerkleRoot of every emitted WordNet archive so
    /// `[archive_signatures]` can be (re-)pinned after a layout change. Run with
    /// `cargo test … dump_wordnet_archive_addresses -- --nocapture --ignored`.
    #[pr4xis::praxis_value(Explainable)]
    #[test]
    #[ignore = "prints archive addresses for pinning; not an assertion"]
    fn dump_wordnet_archive_addresses() {
        let dir = std::env::temp_dir().join("prx_wn_archive_dump");
        for a in emit_all_wordnet_prx_gz(&dir).expect("emit all WordNet") {
            println!(
                "ARCHIVE \"{}@{}\" = \"{}\"",
                a.name, a.version, a.archive_address
            );
        }
    }

    // ── metadata grounding: OMV/PROV-O fields populated correctly ─────

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn wordnet_metadata_is_omv_prov_grounded() {
        let envelope =
            build_wordnet_envelope(SAMPLE_WN_LMF.as_bytes(), FX_NAME, FX_VERSION, FX_URL)
                .expect("build envelope");
        let m = &envelope.metadata;
        assert_eq!(m.name, FX_NAME);
        assert_eq!(m.version, FX_VERSION);
        assert_eq!(m.source_url, FX_URL);
        assert_eq!(m.lexicon_uri, WN_LMF_NAMESPACE_URI);
        // number_of_synsets == archived entity count (a synset IS a concept).
        assert_eq!(m.number_of_synsets, envelope.data.entity_count);
        assert_eq!(m.number_of_synsets, 6, "six synsets in the sample");
        // number_of_senses == the (lemma, synset) sense rows.
        assert_eq!(m.number_of_senses, 6, "six senses in the sample");
    }

    // ── realisation witnesses: this leaf upholds the archive axioms ───

    /// `xml::lmf::prx` realises the
    /// [`OntologyArchiveStorage`](crate::formal::meta::ontology_archive)
    /// ontology — its runnable axioms (now widened over the WordNet
    /// envelope) must hold against the real machinery here.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn realisation_witnesses_the_archive_axioms() {
        use crate::formal::meta::ontology_archive::axioms::{
            CompressionRoundTrip, EmitLoadWellBehaved, LoadGateFailsClosed, MerkleDedupCorrect,
            MerkleHashDeterministic, RkyvDeterminism, SourceHashFaithfulness,
        };
        use pr4xis::ontology::Axiom;
        assert!(MerkleHashDeterministic.verify().is_ok());
        assert!(MerkleDedupCorrect.verify().is_ok());
        assert!(CompressionRoundTrip.verify().is_ok());
        assert!(RkyvDeterminism.verify().is_ok());
        assert!(EmitLoadWellBehaved.verify().is_ok());
        assert!(SourceHashFaithfulness.verify().is_ok());
        assert!(LoadGateFailsClosed.verify().is_ok());
    }

    // ── THE HARD GATE: graph-faithful .prx round-trip over the real corpus ──
}
