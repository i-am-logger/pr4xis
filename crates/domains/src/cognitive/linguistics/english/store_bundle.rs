//! The English STORE BUNDLE — the nine BUILT store buffers of a loaded
//! [`English`], framed into one container, so a consumer can assemble the full
//! `English` by per-store VALIDATION alone: no WordNet decode, no
//! [`from_wordnet`](English::from_wordnet), no owned intermediate maps.
//!
//! # Why
//!
//! The wasm (and native fast-path) load transient used to BE the permanent
//! footprint: `english_load_owned` gunzipped the succinct `.prx.gz`, decoded
//! the WordNet, and ran `from_wordnet` — building EVERY owned `HashMap`/`Vec`
//! intermediate before transcoding them into the nine packed store buffers
//! (measured +348.10 MiB peak over the ~43 MiB resident). wasm32 linear memory
//! never shrinks, so the browser paid that peak forever. This bundle moves the
//! `from_wordnet` transcode to EMIT time (`pr4xis compile`, the wasm
//! `build.rs`): what ships is the nine buffers themselves, and the load leg is
//! gunzip → content gate → split frames → per-store validate → assemble.
//!
//! # The frame
//!
//! The same dependency-free LEB128 framing discipline as
//! [`raw_source_prx`](crate::applied::data_provisioning::raw_source_prx) and
//! the registry envelope:
//!
//! ```text
//! varint  format_version          (= 1)
//! varint  store_count             (= 9)
//! per store, in the FIXED canonical order:
//!   blob    name                  (varint len + UTF-8 name)
//!   varint  col_count             (0 for a dict / rich store; the family
//!                                  stores carry one entry per label because
//!                                  the family buffer stores no header)
//!   col_count × (varint row_count, varint edge_count)
//!   blob    payload               (varint len + the store's buffer bytes)
//! ```
//!
//! The reader is fail-closed and TOTAL: an unknown format version, a wrong
//! store count, an out-of-order / unknown store name, a truncated or
//! overrunning blob, or trailing bytes is a typed [`StoreBundleError`] — never
//! a panic and never an allocation sized from an unvalidated field beyond the
//! buffer's own length.
//!
//! # Trust boundary (why every store re-validates)
//!
//! The bundle rides behind a `praxis.lock` content pin
//! (`[store_bundle_signatures]`), but the per-store validation is NOT waived:
//! the five packed CSR stores enter through their fail-closed
//! `from_untrusted_buf` (the sealed PackedCsr entry), and the four rich `rkyv`
//! stores are `bytecheck`-validated once (`from_validated_buf`) before any
//! `access_unchecked` read — the same validate-once discipline their trusted
//! `build` path applies. A tampered bundle is refused twice over: by the
//! content gate, and (defense in depth) by the store validators.
//!
//! # Portability class — same-toolchain ONLY, never the published wire
//!
//! Four of the nine buffers are `rkyv` envelopes, so the bundle inherits the
//! `[archive_signatures]` trust class (praxis.lock STEP-0 doctrine): a
//! per-toolchain BUILD-OUTPUT pin, not the source's durable identity. It is
//! safe embedded in the wasm binary and in the native `.prx-cache` — emitter
//! and consumer compile from one lockstep (same rkyv version/features; rkyv's
//! archived layout is little-endian and arch-independent, and both shipped
//! targets are LE) — but it is WIRE-UNSAFE as a published portable artifact.
//! The succinct `.cprx.gz` stays the portable wire.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use rkyv::util::AlignedVec;

use super::concept_store::ConceptStore;
use super::function_word_store::FunctionWordStore;
use super::morphology_store::MorphologyStore;
use super::ontology::English;
use super::relation_store::RelationStore;
use super::synset_index::SynsetIndex;
use super::taxonomy_store::TaxonomyStore;
use super::verb_transitivity_index::VerbTransitivityIndex;
use super::word_index::WordIndex;
use super::writing_system_store::WritingSystemStore;

/// The bundle frame format version — bumped on any layout change so an old
/// reader refuses a new bundle instead of guessing.
const STORE_BUNDLE_FORMAT_VERSION: u64 = 1;

/// The nine store names, in the FIXED canonical frame order (the [`English`]
/// field order). The reader enforces this exact sequence — the bundle is a
/// closed, deterministic layout, not a self-describing container.
pub const STORE_NAMES: [&str; 9] = [
    "concepts",
    "word_index",
    "taxonomy",
    "relations",
    "synset_index",
    "function_words",
    "verb_transitivity",
    "writing",
    "morphology",
];

/// Why a store bundle was refused — fail-closed, every variant names the
/// violated invariant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreBundleError {
    /// The frame is structurally malformed (truncated varint/blob, trailing
    /// bytes, overflow) — the message names the exact fault.
    Malformed(String),
    /// The bundle declares a format version this build does not read.
    UnsupportedVersion { declared: u64 },
    /// The bundle does not carry exactly the nine canonical stores.
    WrongStoreCount { declared: u64 },
    /// A frame's store name is not the canonical name expected at its slot.
    UnexpectedStore { expected: &'static str, got: String },
    /// A store's payload failed its own fail-closed validation
    /// (`from_untrusted_buf` / the `bytecheck` pass) — the store is named.
    StoreValidation { store: &'static str, detail: String },
}

impl core::fmt::Display for StoreBundleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            StoreBundleError::Malformed(m) => write!(f, "english store bundle: {m}"),
            StoreBundleError::UnsupportedVersion { declared } => write!(
                f,
                "english store bundle: unsupported format version {declared} (this build reads \
                 version {STORE_BUNDLE_FORMAT_VERSION}) — refusing to guess at the layout"
            ),
            StoreBundleError::WrongStoreCount { declared } => write!(
                f,
                "english store bundle: {declared} store frame(s) declared, expected {}",
                STORE_NAMES.len()
            ),
            StoreBundleError::UnexpectedStore { expected, got } => write!(
                f,
                "english store bundle: frame names store {got:?}, expected {expected:?} at this \
                 slot (the canonical order is fixed)"
            ),
            StoreBundleError::StoreValidation { store, detail } => write!(
                f,
                "english store bundle: store {store:?} failed validation: {detail}"
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for StoreBundleError {}

// ── LEB128 framing (the raw_source_prx discipline) ───────────────────────────

/// Append a LEB128 varint.
fn put_varint(out: &mut Vec<u8>, mut n: u64) {
    loop {
        let b = (n & 0x7f) as u8;
        n >>= 7;
        if n == 0 {
            out.push(b);
            break;
        }
        out.push(b | 0x80);
    }
}

/// Append `bytes` length-prefixed (LEB128 varint length + raw bytes).
fn put_blob(out: &mut Vec<u8>, bytes: &[u8]) {
    put_varint(out, bytes.len() as u64);
    out.extend_from_slice(bytes);
}

/// Read one LEB128 varint with full bounds checking — a truncated buffer is
/// `Err`, never a panic.
fn get_varint(buf: &[u8], pos: &mut usize) -> Result<u64, StoreBundleError> {
    let mut n: u64 = 0;
    let mut shift = 0u32;
    loop {
        let b = *buf.get(*pos).ok_or_else(|| {
            StoreBundleError::Malformed("varint runs past end of buffer".to_string())
        })?;
        *pos += 1;
        n |= ((b & 0x7f) as u64) << shift;
        if b & 0x80 == 0 {
            return Ok(n);
        }
        shift += 7;
        if shift >= 64 {
            return Err(StoreBundleError::Malformed(
                "varint length overflow".to_string(),
            ));
        }
    }
}

/// Read one length-prefixed blob with full bounds checking — a truncated
/// bundle is `Err`, never a panic and never an over-read.
fn get_blob<'a>(buf: &'a [u8], pos: &mut usize) -> Result<&'a [u8], StoreBundleError> {
    let len = get_varint(buf, pos)? as usize;
    let end = pos
        .checked_add(len)
        .filter(|&e| e <= buf.len())
        .ok_or_else(|| StoreBundleError::Malformed("blob runs past end of buffer".to_string()))?;
    let b = &buf[*pos..end];
    *pos = end;
    Ok(b)
}

// ── one parsed frame (shared by the decoder and the byte-identity tests) ─────

/// One parsed store frame, borrowing the bundle buffer — the structural view
/// [`store_bundle_frames`] exposes so a keystone test can compare a store's
/// bytes with the reference build's, per store, without assembling an
/// `English`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleFrame<'a> {
    /// The canonical store name (one of [`STORE_NAMES`]).
    pub name: &'a str,
    /// The per-column `(row_count, edge_count)` table — empty for a dict /
    /// rich store, one entry per label for a family store.
    pub cols: Vec<(u64, u64)>,
    /// The store's buffer bytes, verbatim.
    pub payload: &'a [u8],
}

/// Split a bundle into its nine frames, fail-closed — the shared structural
/// pass [`decode_store_bundle`] builds on. Enforces the format version, the
/// store count, the exact canonical name order, and total consumption (no
/// trailing bytes).
pub fn store_bundle_frames(bytes: &[u8]) -> Result<Vec<BundleFrame<'_>>, StoreBundleError> {
    let mut pos = 0usize;
    let version = get_varint(bytes, &mut pos)?;
    if version != STORE_BUNDLE_FORMAT_VERSION {
        return Err(StoreBundleError::UnsupportedVersion { declared: version });
    }
    let count = get_varint(bytes, &mut pos)?;
    if count != STORE_NAMES.len() as u64 {
        return Err(StoreBundleError::WrongStoreCount { declared: count });
    }
    let mut frames = Vec::with_capacity(STORE_NAMES.len());
    for expected in STORE_NAMES {
        let name = get_blob(bytes, &mut pos)?;
        let name = core::str::from_utf8(name)
            .map_err(|e| StoreBundleError::Malformed(format!("store name is not UTF-8: {e}")))?;
        if name != expected {
            return Err(StoreBundleError::UnexpectedStore {
                expected,
                got: name.to_string(),
            });
        }
        let col_count = get_varint(bytes, &mut pos)?;
        // The only families are the taxonomy (2 labels) and relations (27
        // labels); a declared column table beyond the label-set ceiling is a
        // forgery — bounded before the allocation is sized from it.
        if col_count > 64 {
            return Err(StoreBundleError::Malformed(format!(
                "store {name:?} declares {col_count} columns — beyond any label set"
            )));
        }
        let mut cols = Vec::with_capacity(col_count as usize);
        for _ in 0..col_count {
            let rc = get_varint(bytes, &mut pos)?;
            let ec = get_varint(bytes, &mut pos)?;
            cols.push((rc, ec));
        }
        let payload = get_blob(bytes, &mut pos)?;
        frames.push(BundleFrame {
            name: expected,
            cols,
            payload,
        });
    }
    if pos != bytes.len() {
        return Err(StoreBundleError::Malformed(format!(
            "{} trailing byte(s) after the last store frame",
            bytes.len() - pos
        )));
    }
    Ok(frames)
}

// ── encode ───────────────────────────────────────────────────────────────────

/// Frame one store: name, column table, payload.
fn put_frame(out: &mut Vec<u8>, name: &str, cols: &[(usize, usize)], payload: &[u8]) {
    put_blob(out, name.as_bytes());
    put_varint(out, cols.len() as u64);
    for &(rc, ec) in cols {
        put_varint(out, rc as u64);
        put_varint(out, ec as u64);
    }
    put_blob(out, payload);
}

/// Serialize a loaded [`English`]'s nine BUILT store buffers into one framed
/// bundle (uncompressed — the emit leg gzips and the content pin is taken over
/// THESE bytes, gzip-level-independent). The buffers are written verbatim, so
/// the emitted bundle is BY CONSTRUCTION the store set `from_wordnet` built.
#[must_use]
pub fn encode_store_bundle(english: &English) -> Vec<u8> {
    let mut out = Vec::new();
    put_varint(&mut out, STORE_BUNDLE_FORMAT_VERSION);
    put_varint(&mut out, STORE_NAMES.len() as u64);
    put_frame(
        &mut out,
        "concepts",
        &[],
        english.concepts_store().as_bytes(),
    );
    put_frame(&mut out, "word_index", &[], english.word_index.as_bytes());
    put_frame(
        &mut out,
        "taxonomy",
        &english.taxonomy_store().col_layout(),
        english.taxonomy_store().as_bytes(),
    );
    put_frame(
        &mut out,
        "relations",
        &english.relations_store().col_layout(),
        english.relations_store().as_bytes(),
    );
    put_frame(
        &mut out,
        "synset_index",
        &[],
        english.synset_index_store().as_bytes(),
    );
    put_frame(
        &mut out,
        "function_words",
        &[],
        english.function_words_store().as_bytes(),
    );
    put_frame(
        &mut out,
        "verb_transitivity",
        &[],
        english.verb_transitivity_store().as_bytes(),
    );
    put_frame(&mut out, "writing", &[], english.writing_store().as_bytes());
    put_frame(
        &mut out,
        "morphology",
        &[],
        english.morphology_store().as_bytes(),
    );
    out
}

// ── decode ───────────────────────────────────────────────────────────────────

/// Copy a frame payload into a fresh 16-aligned buffer — the alignment every
/// store validator requires (a frame slice carries no alignment guarantee).
fn aligned(payload: &[u8]) -> AlignedVec<16> {
    let mut buf = AlignedVec::<16>::with_capacity(payload.len());
    buf.extend_from_slice(payload);
    buf
}

/// The frame's column table as the `(row_count, edge_count)` usizes the family
/// validator takes — overflow-checked on 32-bit targets (wasm32), where a
/// forged 64-bit count must not truncate.
fn cols_usize(
    store: &'static str,
    cols: &[(u64, u64)],
) -> Result<Vec<(usize, usize)>, StoreBundleError> {
    cols.iter()
        .map(|&(rc, ec)| {
            let rc = usize::try_from(rc).map_err(|_| StoreBundleError::StoreValidation {
                store,
                detail: format!("row_count {rc} overflows this target"),
            })?;
            let ec = usize::try_from(ec).map_err(|_| StoreBundleError::StoreValidation {
                store,
                detail: format!("edge_count {ec} overflows this target"),
            })?;
            Ok((rc, ec))
        })
        .collect()
}

/// Assemble a full [`English`] from a framed store bundle — the load leg.
///
/// Splits the frames ([`store_bundle_frames`]), then per store runs its own
/// fail-closed validating entry — `from_untrusted_buf` for the five packed CSR
/// stores (the sealed PackedCsr byte entry), the `bytecheck`
/// `from_validated_buf` pass for the four rich `rkyv` stores — and assembles
/// the `English` directly (`English::from_stores`). NO WordNet decode, NO
/// `from_wordnet`, NO owned intermediate maps: the peak transient is the
/// bundle bytes plus the per-store buffer copies (≈ the resident cost), not
/// the former +348 MiB owned-map build.
pub fn decode_store_bundle(bytes: &[u8]) -> Result<English, StoreBundleError> {
    let frames = store_bundle_frames(bytes)?;
    // Indexed by the canonical order `store_bundle_frames` enforced.
    let f = |i: usize| -> &BundleFrame<'_> { &frames[i] };
    let csr_err = |store: &'static str| {
        move |e: crate::formal::meta::packed_csr::PackedCsrError| {
            StoreBundleError::StoreValidation {
                store,
                detail: e.to_string(),
            }
        }
    };
    let rich_err = |store: &'static str| {
        move |detail: String| StoreBundleError::StoreValidation { store, detail }
    };

    let concepts =
        ConceptStore::from_validated_buf(aligned(f(0).payload)).map_err(rich_err("concepts"))?;
    let word_index =
        WordIndex::from_untrusted_buf(aligned(f(1).payload)).map_err(csr_err("word_index"))?;
    let taxonomy = TaxonomyStore::from_untrusted_buf(
        aligned(f(2).payload),
        &cols_usize("taxonomy", &f(2).cols)?,
    )
    .map_err(csr_err("taxonomy"))?;
    let relations = RelationStore::from_untrusted_buf(
        aligned(f(3).payload),
        &cols_usize("relations", &f(3).cols)?,
    )
    .map_err(csr_err("relations"))?;
    let synset_index =
        SynsetIndex::from_untrusted_buf(aligned(f(4).payload)).map_err(csr_err("synset_index"))?;
    let function_words = FunctionWordStore::from_validated_buf(aligned(f(5).payload))
        .map_err(rich_err("function_words"))?;
    let verb_transitivity = VerbTransitivityIndex::from_untrusted_buf(aligned(f(6).payload))
        .map_err(csr_err("verb_transitivity"))?;
    let writing = WritingSystemStore::from_validated_buf(aligned(f(7).payload))
        .map_err(rich_err("writing"))?;
    let morphology = MorphologyStore::from_validated_buf(aligned(f(8).payload))
        .map_err(rich_err("morphology"))?;

    Ok(English::from_stores(
        concepts,
        word_index,
        taxonomy,
        relations,
        synset_index,
        function_words,
        verb_transitivity,
        writing,
        morphology,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A sample-corpus bundle round-trips: encode → decode assembles an
    /// `English` that answers identically (lookup / is-a / transitivity /
    /// function words / morphology / writing), and RE-ENCODING the decoded
    /// `English` reproduces the bundle BYTE-FOR-BYTE — the per-store
    /// byte-identity keystone at unit scale (the corpus-scale twin lives in
    /// `praxis-corpus-tests`).
    #[pr4xis::praxis_value(Verifiable, Deterministic)]
    #[test]
    fn bundle_round_trips_the_sample_english_byte_identically() {
        use crate::cognitive::linguistics::language::Language;

        let reference = English::sample();
        let bundle = encode_store_bundle(&reference);
        let loaded = decode_store_bundle(&bundle).expect("the fresh bundle decodes");

        // Per-store byte identity: re-encoding the decoded English writes the
        // SAME nine buffers (each store's bytes were carried verbatim).
        assert_eq!(
            encode_store_bundle(&loaded),
            bundle,
            "re-encoded bundle must be byte-identical (all 9 stores)"
        );

        // Behavior spot-checks across every store.
        assert_eq!(loaded.concept_count(), reference.concept_count());
        assert_eq!(loaded.lookup("dog"), reference.lookup("dog"));
        let dog = loaded.lookup("dog")[0];
        let animal = loaded.lookup("animal")[0];
        assert!(loaded.is_a(dog, animal), "dog is-a animal via the taxonomy");
        assert_eq!(
            loaded.concept_by_synset("s-dog").map(|c| c.id()),
            reference.concept_by_synset("s-dog").map(|c| c.id()),
        );
        assert_eq!(
            Language::lexical_lookup(&loaded, "see"),
            Language::lexical_lookup(&reference, "see"),
        );
        assert_eq!(
            Language::morphological_rules(&loaded),
            Language::morphological_rules(&reference),
        );
        assert_eq!(
            Language::writing_system(&loaded).name,
            Language::writing_system(&reference).name,
        );
        assert_eq!(
            Language::word_count(&loaded),
            Language::word_count(&reference),
        );
    }

    /// Determinism: two independent encodes of the same `English` are
    /// byte-identical (the pin + re-emit-equality strategy's precondition).
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn encode_is_deterministic() {
        let english = English::sample();
        assert_eq!(encode_store_bundle(&english), encode_store_bundle(&english));
    }

    /// ∀ single-byte mutation of a real bundle, the decode is FAIL-CLOSED:
    /// a typed `Err` or an `Ok` whose store contents still validate — never a
    /// panic (the hostile-bytes discipline of the sealed PackedCsr entry,
    /// applied to the whole container). Runs the full byte range of the small
    /// sample bundle deterministically at a byte stride, plus the gzip-level
    /// tamper leg lives with the content gate in `lmf::prx`.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn mutated_bundle_never_panics_and_is_refused_or_still_valid() {
        let bundle = encode_store_bundle(&English::sample());
        // Every byte for the small sample bundle — total, not sampled.
        for i in 0..bundle.len() {
            let mut bad = bundle.clone();
            bad[i] ^= 0x01;
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                decode_store_bundle(&bad).map(|_| ())
            }));
            match res {
                Ok(_) => {} // Err (refused) or Ok (mutation landed in a
                // still-valid region, e.g. a gloss byte) — both are sound;
                // the CONTENT GATE (the lock pin) is what refuses any
                // mutation outright, tested at the gated-load layer.
                Err(_) => panic!("decode_store_bundle PANICKED on byte {i} ^= 0x01"),
            }
        }
    }

    /// Truncations and trailing garbage are typed refusals.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn truncated_or_padded_bundles_are_refused() {
        let bundle = encode_store_bundle(&English::sample());
        for cut in [0, 1, bundle.len() / 2, bundle.len() - 1] {
            assert!(
                decode_store_bundle(&bundle[..cut]).is_err(),
                "a bundle truncated to {cut} bytes must be refused"
            );
        }
        let mut padded = bundle.clone();
        padded.push(0);
        assert!(
            matches!(
                decode_store_bundle(&padded),
                Err(StoreBundleError::Malformed(_))
            ),
            "trailing bytes must be refused"
        );
    }

    /// The frame splitter names the canonical stores in order.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn frames_carry_the_canonical_store_names_in_order() {
        let bundle = encode_store_bundle(&English::sample());
        let frames = store_bundle_frames(&bundle).expect("the fresh bundle splits");
        let names: Vec<&str> = frames.iter().map(|f| f.name).collect();
        assert_eq!(names, STORE_NAMES);
        // The two family stores carry their column tables; the rest carry none.
        assert_eq!(frames[2].cols.len(), 2, "taxonomy: one entry per Direction");
        assert_eq!(
            frames[3].cols.len(),
            27,
            "relations: one entry per RelationKind"
        );
        for i in [0usize, 1, 4, 5, 6, 7, 8] {
            assert!(
                frames[i].cols.is_empty(),
                "{} carries no column table",
                frames[i].name
            );
        }
    }
}
