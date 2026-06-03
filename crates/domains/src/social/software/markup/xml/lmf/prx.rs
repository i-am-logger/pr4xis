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
//! [`PrxError`], [`gzip`] / [`gunzip`], [`source_content_hash`],
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
//! [`English::from_wordnet`] yields from the same source. Byte-exact source
//! fidelity (`hash(out) == hash(in)`, the #186 invariant) is discharged
//! ENTIRELY by [`RawSource::blob`] — the whole unzipped WN-LMF XML,
//! content-addressed to the `praxis.lock` `[hashes]` pin — exactly as the
//! OWL and USC leaves do. WN-LMF has no byte-exact writer + canonicalization
//! (the analogue of the OWL `write_owl` + RDFC #258 gap), so English is
//! permanently [`RoundTripFidelity::RawBytesComplementFloor`] today;
//! `ByteExactGraphFaithful` stays unemitted, not stubbed.
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
//!   §2.2; **NIST (2015)** FIPS 180-4 §6.2; **Dolstra (2006)**
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

use super::reader::read_wordnet;
use crate::cognitive::linguistics::english::English;
use crate::cognitive::linguistics::language;
use crate::formal::meta::artifact_identity::ontology::{
    ClaimData, IdentityClaim, IdentityConcept, VerificationResult,
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
    source_content_hash,
};

// =============================================================================
// WnPrxMetadata — the OMV/PROV-O-grounded metadata block, WordNet metrics.
// =============================================================================

/// Self-describing metadata carried by a [`WordNetPrxEnvelope`].
///
/// Reuses the OWL leaf's OMV (Hartmann, Palma & Sure 2005) / PROV-O (Lebo,
/// Sahoo & McGuinness 2013) grounding for identity — `name` (`omv:name`),
/// `version` (`omv:version`), `lexicon_uri` (`omv:URI`), `source_url`
/// (`prov:atLocation`), `source_sha256` (`prov:wasDerivedFrom` content
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
    /// the SHA-256 (NIST FIPS 180-4 §6.2; Dolstra 2006) of the exact source
    /// WN-LMF bytes [`read_wordnet`] consumed.
    /// The load gate validates this against the `praxis.lock` `[hashes]`
    /// pin; a mismatch fails closed.
    pub source_sha256: String,
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
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct WordNetPrxEnvelope {
    /// OMV/PROV-O-grounded self-description, incl. the source content hash
    /// the load gate validates.
    pub metadata: WnPrxMetadata,
    /// The archived synset corpus — the owned mirror of the
    /// `CodegenData<English>` interchange, with `word_index` populated.
    pub data: OwnedCodegenData,
    /// The source lens's [`RoundTripFidelity`] — `RawBytesComplementFloor`
    /// for English today (no byte-exact WN-LMF writer + canonicalization yet).
    pub mode: RoundTripFidelity,
    /// The content-addressed source bytes (the constant-complement) — `Some`
    /// iff `mode == RawBytesComplementFloor`.
    pub raw: Option<RawSource>,
}

// =============================================================================
// rkyv layer — WordNet envelope ⇄ bytes (bytecheck-validated).
// =============================================================================

/// Serialize a WordNet envelope to rkyv bytes (the lens *put*).
/// Deterministic — equal envelopes yield equal bytes (the sorted word index
/// pins the layout), so the blob's SHA-256 is a stable `MerkleRoot` content
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
/// SRI 2016): `raw_hash::verify` re-hashes `bytes`, so the pin is checked
/// against bytes actually present, never a self-asserted label.
fn wn_verify_content_address(bytes: &[u8], trusted_pin: &str, key: &str) -> Result<(), PrxError> {
    let claim = IdentityClaim {
        concept: IdentityConcept::RawHash,
        data: ClaimData::Sha256(trusted_pin.to_string()),
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
/// For [`RoundTripFidelity::RawBytesComplementFloor`] (English today):
/// return the stored `raw.blob` after enforcing the in-envelope honesty
/// doctrine (`sha256(blob) == raw.content_address == metadata.source_sha256`).
/// A tampered blob is rejected. [`RoundTripFidelity::ByteExactGraphFaithful`]
/// would regenerate from `data` via a byte-exact `write_wordnet` + WN-LMF
/// canonicalization — the WordNet analogue of the OWL `write_owl` + RDFC gap
/// (#258), unimplemented, so no envelope is emitted in that mode today.
pub fn wn_reconstruct_source(envelope: &WordNetPrxEnvelope) -> Result<Vec<u8>, PrxError> {
    match envelope.mode {
        RoundTripFidelity::RawBytesComplementFloor => {
            let raw = envelope
                .raw
                .as_ref()
                .ok_or_else(|| PrxError::SourceNotReconstructible {
                    reason: "RawBytesComplementFloor envelope is missing its raw source leaf"
                        .to_string(),
                })?;
            let computed = source_content_hash(&raw.blob);
            let key = format!("{}@{}", envelope.metadata.name, envelope.metadata.version);
            if computed != raw.content_address {
                return Err(PrxError::HashMismatch {
                    key: format!("{key} (raw content address)"),
                    expected: raw.content_address.clone(),
                    found: computed,
                });
            }
            if computed != envelope.metadata.source_sha256 {
                return Err(PrxError::HashMismatch {
                    key: format!("{key} (raw vs metadata)"),
                    expected: envelope.metadata.source_sha256.clone(),
                    found: computed,
                });
            }
            Ok(raw.blob.clone())
        }
        RoundTripFidelity::ByteExactGraphFaithful => Err(PrxError::SourceNotReconstructible {
            reason: "byte-exact WN-LMF graph→source regeneration (write_wordnet + WN-LMF \
                     canonicalization) is not yet implemented"
                .to_string(),
        }),
    }
}

/// Verify the source-identity leg: reconstruct the source and bind it to the
/// trusted `SourcePin` (`praxis.lock` `[hashes]`). Mirrors OWL `verify_source_leg`.
fn wn_verify_source_leg(
    envelope: &WordNetPrxEnvelope,
    source_pin: &str,
    key: &str,
) -> Result<(), PrxError> {
    match envelope.mode {
        RoundTripFidelity::RawBytesComplementFloor => {
            let source_bytes = wn_reconstruct_source(envelope)?;
            wn_verify_content_address(&source_bytes, source_pin, key)
        }
        RoundTripFidelity::ByteExactGraphFaithful => Ok(()),
    }
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
    archive_pin: &str,
    source_pin: &str,
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
    archive_pin: &str,
    source_pin: &str,
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
        .ok_or_else(|| PrxError::NoArchivePin { key: key.clone() })?
        .to_string();
    let source_pin = lock_hashes()
        .get(&key)
        .ok_or_else(|| PrxError::NoLockPin { key: key.clone() })?
        .clone();
    wn_admit_validated(&rkyv_bytes, envelope, &archive_pin, &source_pin)
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
            match rel.rel_type {
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
/// `(name, version, url)`. Parses via [`read_wordnet`],
/// projects with `wn_builder_to_owned`, attaches the OMV/PROV-O metadata,
/// and carries the exact source bytes as the `RawBytesComplementFloor` raw
/// leaf (content-addressed to the UNZIPPED bytes the reader consumed —
/// matching OWL and USC).
pub fn build_wordnet_envelope(
    source: &[u8],
    name: &str,
    version: &str,
    url: &str,
) -> Result<WordNetPrxEnvelope, PrxError> {
    let text = core::str::from_utf8(source)
        .map_err(|e| PrxError::Read(format!("source is not UTF-8: {e}")))?;
    let wn = read_wordnet(text).map_err(|e| PrxError::Read(format!("{e}")))?;

    let number_of_synsets = wn.synsets.len() as u64;
    // A sense is one (lemma, synset) pairing recorded under a LexicalEntry
    // (ISO 24613 LMF; Fellbaum 1998).
    let number_of_senses = wn.entries.iter().map(|e| e.senses.len()).sum::<usize>() as u64;

    let data = wn_builder_to_owned(&wn);
    let source_sha256 = source_content_hash(source);
    let metadata = WnPrxMetadata {
        name: name.to_string(),
        version: version.to_string(),
        lexicon_uri: WN_LMF_NAMESPACE_URI.to_string(),
        source_url: url.to_string(),
        source_sha256: source_sha256.clone(),
        number_of_synsets,
        number_of_senses,
    };
    Ok(WordNetPrxEnvelope {
        metadata,
        data,
        mode: RoundTripFidelity::RawBytesComplementFloor,
        raw: Some(RawSource {
            content_address: source_sha256,
            blob: source.to_vec(),
        }),
    })
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
        load_wordnet_prx_gz(&read_back, &archive_address, source_pin)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognitive::linguistics::english::ConceptId;

    /// A minimal but full-shape WN-LMF lexicon: `<LexicalResource>` wrapper,
    /// one `<Lexicon>`, a small dog/cat/mammal/animal taxonomy plus a verb
    /// and an adjective, with `<Sense>`s wiring lemmas to synsets and a
    /// `hypernym` chain. `read_wordnet` accepts it; emit projects it through
    /// `wn_builder_to_owned`. Mirrors `English::sample`'s inline fixture so
    /// the corpus-equality test has a known reference.
    const SAMPLE_WN_LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="test-en" label="Test English" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-dog-n"><Lemma writtenForm="dog" partOfSpeech="n"/><Sense id="dog-n-01" synset="s-dog"/><Form writtenForm="dogs"/></LexicalEntry>
    <LexicalEntry id="e-cat-n"><Lemma writtenForm="cat" partOfSpeech="n"/><Sense id="cat-n-01" synset="s-cat"/></LexicalEntry>
    <LexicalEntry id="e-mammal-n"><Lemma writtenForm="mammal" partOfSpeech="n"/><Sense id="mammal-n-01" synset="s-mammal"/></LexicalEntry>
    <LexicalEntry id="e-animal-n"><Lemma writtenForm="animal" partOfSpeech="n"/><Sense id="animal-n-01" synset="s-animal"/></LexicalEntry>
    <LexicalEntry id="e-run-v"><Lemma writtenForm="run" partOfSpeech="v"/><Sense id="run-v-01" synset="s-run"/></LexicalEntry>
    <LexicalEntry id="e-big-a"><Lemma writtenForm="big" partOfSpeech="a"/><Sense id="big-a-01" synset="s-big"/></LexicalEntry>
    <Synset id="s-dog" ili="i1" partOfSpeech="n"><Definition>a domesticated canine</Definition><SynsetRelation relType="hypernym" target="s-mammal"/></Synset>
    <Synset id="s-cat" ili="i2" partOfSpeech="n"><Definition>a small feline</Definition><SynsetRelation relType="hypernym" target="s-mammal"/></Synset>
    <Synset id="s-mammal" ili="i3" partOfSpeech="n"><Definition>warm-blooded vertebrate</Definition><SynsetRelation relType="hypernym" target="s-animal"/></Synset>
    <Synset id="s-animal" ili="i4" partOfSpeech="n"><Definition>a living organism</Definition></Synset>
    <Synset id="s-run" ili="i5" partOfSpeech="v"><Definition>move fast on foot</Definition></Synset>
    <Synset id="s-big" ili="i7" partOfSpeech="a"><Definition>of considerable size</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;

    const FX_NAME: &str = "english_wordnet";
    const FX_VERSION: &str = "2025";
    const FX_URL: &str = "https://github.com/globalwordnet/english-wordnet/releases/download/2025-edition/english-wordnet-2025.xml.gz";

    // ── envelope / gate round-trip + determinism ──────────────────────

    /// to_bytes ∘ from_bytes is identity (the GetPut leg, incl. the word
    /// index), and emitting twice gives byte-identical `.prx.gz` — proving
    /// the `word_index` sort makes the rkyv layout process-deterministic (no
    /// HashMap iteration order leaks into the bytes).
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
    #[test]
    fn wordnet_emit_then_load_equals_corpus() {
        let src = SAMPLE_WN_LMF.as_bytes();

        // The reference corpus: read_wordnet → English::from_wordnet.
        let wn = read_wordnet(SAMPLE_WN_LMF).expect("parse sample WN-LMF");
        let reference = English::from_wordnet(&wn);

        // The archived corpus: emit → load → from_codegen.
        let prx_gz = emit_wordnet_prx_gz(src, FX_NAME, FX_VERSION, FX_URL).expect("emit");
        let archive_pin = prx_archive_address(&prx_gz).expect("archive address");
        let source_pin = source_content_hash(src);
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

    // ── raw leaf: .prx → source byte-exact (BytesPlusView floor, #186) ─

    #[test]
    fn wordnet_raw_leaf_reconstructs_source_byte_exact() {
        let envelope =
            build_wordnet_envelope(SAMPLE_WN_LMF.as_bytes(), FX_NAME, FX_VERSION, FX_URL)
                .expect("build envelope");
        assert_eq!(envelope.mode, RoundTripFidelity::RawBytesComplementFloor);
        let src = wn_reconstruct_source(&envelope).expect("reconstruct");
        assert_eq!(
            src,
            SAMPLE_WN_LMF.as_bytes(),
            "wn_reconstruct_source must return the exact source bytes"
        );
        assert_eq!(
            source_content_hash(&src),
            envelope.metadata.source_sha256,
            "reconstructed bytes must hash to the pinned source content address"
        );
    }

    /// Fail-closed: a floor envelope with no raw complement cannot
    /// reconstruct its source — `wn_reconstruct_source` refuses `raw = None`
    /// rather than fabricating bytes.
    #[test]
    fn wordnet_reconstruct_refuses_missing_raw_leaf() {
        let mut envelope =
            build_wordnet_envelope(SAMPLE_WN_LMF.as_bytes(), FX_NAME, FX_VERSION, FX_URL)
                .expect("build envelope");
        envelope.raw = None;
        let err = wn_reconstruct_source(&envelope)
            .expect_err("floor envelope without raw leaf must be rejected");
        assert!(
            matches!(err, PrxError::SourceNotReconstructible { .. }),
            "got {err:?}"
        );
    }

    /// Fail-closed: a tampered raw blob (no longer hashing to its content
    /// address) is rejected rather than returning wrong bytes.
    #[test]
    fn wordnet_raw_leaf_rejects_tampered_blob() {
        let mut envelope =
            build_wordnet_envelope(SAMPLE_WN_LMF.as_bytes(), FX_NAME, FX_VERSION, FX_URL)
                .expect("build envelope");
        envelope.raw.as_mut().expect("raw leaf").blob.push(b'!');
        let err = wn_reconstruct_source(&envelope).expect_err("tampered blob must fail closed");
        assert!(matches!(err, PrxError::HashMismatch { .. }), "got {err:?}");
    }

    // ── load gate: teeth on a poisoned word_index ─────────────────────

    /// The MerkleRoot leg's reason to exist: an envelope carrying a genuine
    /// source label + honest raw leaf but ONE poisoned `word_index` entry is
    /// rejected. The source leg alone would pass (raw is honest); the
    /// MerkleRoot leg binds the whole installed node — including the word
    /// index — so poisoning it changes the content address and the gate
    /// refuses it. Mirrors OWL's `load_rejects_poisoned_data_under_honest_label`.
    #[test]
    fn wordnet_load_rejects_poisoned_word_index_under_honest_label() {
        let honest = build_wordnet_envelope(SAMPLE_WN_LMF.as_bytes(), FX_NAME, FX_VERSION, FX_URL)
            .expect("build envelope");
        let honest_archive_pin = source_content_hash(&wordnet_envelope_to_bytes(&honest).unwrap());
        let source_pin = honest.metadata.source_sha256.clone();

        // Same genuine source label + raw leaf, only one word_index entry poisoned.
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

    #[test]
    fn wordnet_load_rejects_corrupted_blob() {
        let any = "0".repeat(64);
        let garbage = gzip(b"not a valid WordNet rkyv envelope at all").expect("gzip");
        let err = load_wordnet_prx_gz(&garbage, &any, &any).expect_err("garbage rkyv must fail");
        assert!(matches!(err, PrxError::Rkyv(_)), "got {err:?}");
    }

    // ── lock gate: unpinned source fails closed ───────────────────────

    /// The lock-driven load path fails closed for a WordNet envelope whose
    /// `"{name}@{version}"` has no `[archive_signatures]` pin — the
    /// MerkleRoot pin is looked up first, so an unregistered lexicon is
    /// refused there (mirrors OWL `load_validation_rejects_unpinned_source`).
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

    // ── metadata grounding: OMV/PROV-O fields populated correctly ─────

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

    // ── full-corpus emit (gated on the bundled XML, graceful skip) ────

    /// The full Open English WordNet 2025 corpus (≈89 MB on disk) emits and
    /// round-trips through the gate, materializing a rich [`English`] whose
    /// `concept_count` matches `English::from_wordnet`. Gated behind the
    /// on-disk file with a graceful skip — a plain checkout that hasn't
    /// provisioned the data emits nothing here, the same graceful-skip
    /// doctrine `loaded_vocabularies` and the emitters use. Heavy, so it is
    /// the on-disk-only corroboration of the cheap sample round-trip above.
    #[test]
    fn wordnet_full_corpus_emit_then_load_matches_from_wordnet() {
        let path = workspace_root().join("crates/domains/data/wordnet/english-wordnet-2025.xml");
        let Ok(source) = std::fs::read(&path) else {
            return; // not provisioned on disk — skip gracefully
        };
        let text = core::str::from_utf8(&source).expect("WordNet XML is UTF-8");
        let wn = read_wordnet(text).expect("parse full WordNet");
        let reference = English::from_wordnet(&wn);

        let prx_gz = emit_wordnet_prx_gz(&source, FX_NAME, FX_VERSION, FX_URL).expect("emit");
        let archive_pin = prx_archive_address(&prx_gz).expect("archive address");
        let source_pin = source_content_hash(&source);
        let loaded =
            load_wordnet_prx_gz(&prx_gz, &archive_pin, &source_pin).expect("load + validate");

        assert!(
            loaded.concept_count() > 100_000,
            "real English WordNet is rich (>100k synsets); got {}",
            loaded.concept_count()
        );
        assert_eq!(
            loaded.concept_count(),
            reference.concept_count(),
            "full-corpus concept_count must survive the archive"
        );

        // A canonical lemma resolves to the same synsets pre/post-archive.
        let lref: Vec<ConceptId> = reference.lookup("dog").to_vec();
        let larch: Vec<ConceptId> = loaded.lookup("dog").to_vec();
        assert_eq!(
            lref.len(),
            larch.len(),
            "lookup('dog') sense count must survive the archive"
        );
    }
}
