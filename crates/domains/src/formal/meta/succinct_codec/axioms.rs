//! Runnable axioms of the [`SuccinctCodec`](super::ontology) ontology — each
//! `verify()` is a *predicate that runs*, exercising the real succinct `.prx`
//! codec (the bit-packing kernel
//! [`markup::xml::succinct`](crate::social::software::markup::xml::succinct) and
//! the codegen-interchange codec
//! [`OwnedCodegenData::to_succinct`](crate::social::software::markup::xml::owl::prx::OwnedCodegenData::to_succinct))
//! rather than asserting a doc-comment (North Star W3 slice 3;
//! `feedback_praxis_as_compiler_self_describing`).
//!
//! This is a DIFFERENT codec from [`super::super::canonical_codec`]: that one is
//! DAG-CBOR (a self-describing interchange form whose guarantee is a stable
//! content address); this one is the COMPACT WIRE format — bit-packed columns,
//! gap-coded monotone offsets, and a front-coded string dictionary — that the
//! `.cprx.gz` English corpus and the registry ship in. Its round-trip and its
//! two compaction invariants are covered by NO canonical-codec axiom (a
//! DAG-CBOR round-trip says nothing about gap-coded offsets or front-coding)
//! and, before this module, by exactly one on-disk integration test
//! (`compact_prx_gz_is_smaller_than_source_and_reasoning_equivalent`) — named in
//! no `ontology!`, invisible to the self-model graph.
//!
//! Gated on `feature = "prx"`, like the codec itself
//! ([`markup::xml::succinct`](crate::social::software::markup::xml) and
//! [`owl::prx`](crate::social::software::markup::xml::owl::prx) are both
//! `#[cfg(feature = "prx")]`): the machinery this ontology describes only exists
//! under `prx`, so the ontology that self-describes it is gated with it.
//!
//! Each axiom is a GENUINELY UNCOVERED, non-tautological, machine-checkable
//! fact:
//!
//! - `SuccinctCodecRoundTrip` — `from_succinct(to_succinct(d)) == d` over a
//!   WordNet+registry-shaped [`OwnedCodegenData`]. Exercises the WHOLE
//!   composition (shared dictionary, four bit-packed text columns, the
//!   `word_index` CSR, six edge tables) — the assembly logic the two
//!   column-level axioms below do not touch. A codec that dropped, reordered, or
//!   widened any column is falsified.
//! - `MonotoneOffsetsCompact` — for a monotone non-decreasing offset sequence,
//!   the gap-coded column (`put_ef`) round-trips exactly AND is STRICTLY smaller
//!   than the same values stored as an absolute bit-packed column (`put_cv`).
//!   The second leg is the teeth: a `put_ef` that stored absolute values would
//!   round-trip identically but fail compaction.
//! - `FrontCodingSharesPrefixes` — the front-coded dictionary (`put_dict_fc`) is
//!   lossless for ANY input order, AND for a sorted dictionary with shared
//!   prefixes it is STRICTLY smaller than the plain length-prefixed dictionary
//!   (`put_dict`). The compaction leg is the teeth: an implementation that did
//!   not elide the shared prefix would round-trip but not compact.
//! - `RawSourceDeflateTransport` — the raw-source envelope's RFC 1951 payload
//!   transport (`raw_source_prx`, format v2) round-trips byte-exactly, strictly
//!   compacts a compressible witness, and downgrades store-if-smaller to the
//!   identity envelope on incompressible bytes.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

use crate::social::software::markup::xml::owl::prx::OwnedCodegenData;
use crate::social::software::markup::xml::succinct::{
    get_dict_fc, get_ef, put_cv, put_dict, put_dict_fc, put_ef,
};

/// A WordNet+registry-shaped witness [`OwnedCodegenData`]: three WordNet synsets
/// (shared IRI prefix, repeated `Synset` kind) plus two SPAR/cito object
/// properties (a second shared prefix), a `word_index` whose CSR has variable
/// per-word handle counts (`dog` → two senses), a populated `taxonomy` and
/// `references` table, and four empty edge tables. Small but structurally
/// complete: every column the codec serializes is non-degenerate, so the
/// round-trip actually exercises the dictionary sharing, the CSR offsets, and
/// the edge endpoint packing.
fn witness() -> OwnedCodegenData {
    OwnedCodegenData {
        entity_count: 5,
        entity_ids: alloc::vec![
            String::from("http://wordnet.princeton.edu/synset-animal-noun-1"),
            String::from("http://wordnet.princeton.edu/synset-canine-noun-1"),
            String::from("http://wordnet.princeton.edu/synset-dog-noun-1"),
            String::from("http://purl.org/spar/cito/cites"),
            String::from("http://purl.org/spar/cito/isCitedBy"),
        ],
        entity_kind: alloc::vec![
            String::from("Synset"),
            String::from("Synset"),
            String::from("Synset"),
            String::from("ObjectProperty"),
            String::from("ObjectProperty"),
        ],
        entity_labels: alloc::vec![
            String::from("animal"),
            String::from("canine"),
            String::from("dog"),
            String::from("cites"),
            String::from("is cited by"),
        ],
        entity_defs: alloc::vec![
            String::from("a living organism"),
            String::from("any member of the dog family"),
            String::from("a domesticated carnivorous mammal"),
            String::new(),
            String::new(),
        ],
        word_index: alloc::vec![
            (String::from("animal"), alloc::vec![0u64]),
            (String::from("canine"), alloc::vec![1u64]),
            (String::from("dog"), alloc::vec![1u64, 2u64]),
        ],
        taxonomy: alloc::vec![(2u64, 1u64), (1u64, 0u64)],
        mereology: Vec::new(),
        opposition: Vec::new(),
        equivalence: Vec::new(),
        causation: Vec::new(),
        references: alloc::vec![(3u64, 4u64)],
    }
}

/// The succinct codec is a total inverse pair over an [`OwnedCodegenData`]:
/// `from_succinct(to_succinct(d)) == d`. Checked over the WordNet+registry
/// `witness`, whose every column is non-degenerate, so a codec that dropped,
/// reordered, widened, or truncated any column — the shared front-coded
/// dictionary, the four bit-packed text columns, the `word_index` CSR, or any of
/// the six edge tables — is falsified. Distinct from
/// [`super::super::canonical_codec::axioms::CodecRoundTrip`]: that proves the
/// DAG-CBOR interchange form round-trips; this proves the COMPACT bit-packed
/// wire form does, a different codec over a different structure. This is a
/// lossless-compression inverse pair (`from_succinct ∘ to_succinct = id`), NOT
/// a lens law. Witten, Moffat & Bell (1999) *Managing Gigabytes* (lossless
/// coding).
pub struct SuccinctCodecRoundTrip;

impl Axiom for SuccinctCodecRoundTrip {
    fn verify(&self) -> Verdict {
        let d = witness();
        let back = OwnedCodegenData::from_succinct(&d.to_succinct());
        if back == d {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SuccinctCodecRoundTrip",
        "from_succinct(to_succinct(d)) == d over a WordNet+registry-shaped OwnedCodegenData (the compact bit-packed .prx wire form is a total inverse pair)",
        "Witten, Moffat & Bell (1999) Managing Gigabytes: Compressing and Indexing Documents and Images, 2nd ed. — lossless coding: from_succinct ∘ to_succinct = identity (a total inverse pair)"
    );
}

pr4xis::register_axiom!(SuccinctCodecRoundTrip, constructor);

/// The monotone-offset column both round-trips AND compacts. For a monotone
/// non-decreasing sequence (a CSR offset array), the gap-coded column
/// (`put_ef`, which stores consecutive gaps) decodes back to the exact sequence,
/// AND its encoded size is STRICTLY smaller than the same values stored as an
/// absolute bit-packed column (`put_cv`) — because the per-node gaps are small
/// (a narrow bit width) even when the cumulative offsets span a large range (a
/// wide bit width). This is gap/delta coding of a monotone integer sequence
/// (the `put_ef` NAME is historical — it does NOT implement Elias-Fano's
/// upper/lower-bit split + unary + select; it is plain gap coding). Non-
/// tautological on BOTH legs: the round-trip alone would pass for a codec that
/// stored absolute offsets, so the strict-inequality compaction leg is what
/// proves the gaps are actually elided. Witten, Moffat & Bell (1999) §3.3
/// (gap/delta coding of monotone integer sequences).
pub struct MonotoneOffsetsCompact;

impl Axiom for MonotoneOffsetsCompact {
    fn verify(&self) -> Verdict {
        // A monotone non-decreasing offset array: prefix sums of a repeated
        // small-length pattern, so the cumulative range is large (a wide
        // absolute width) while every gap stays in {1,2,3} (a narrow gap width).
        let pattern = [2usize, 1, 3, 1, 2];
        let mut offsets = alloc::vec![0usize];
        for _ in 0..24 {
            for &len in &pattern {
                let last = *offsets.last().unwrap_or(&0);
                offsets.push(last + len);
            }
        }

        // Leg 1 — lossless: gap-coded then prefix-summed recovers the sequence.
        let mut gap_coded = Vec::new();
        put_ef(&mut gap_coded, &offsets);
        let mut pos = 0usize;
        let decoded = get_ef(&gap_coded, &mut pos);
        let lossless = decoded == offsets;

        // Leg 2 — compaction: the gap-coded column is strictly smaller than the
        // same values as an absolute bit-packed column. This is the property a
        // `put_ef` that forgot to gap-code would fail.
        let mut absolute = Vec::new();
        put_cv(&mut absolute, &offsets);
        let compacts = gap_coded.len() < absolute.len();

        if lossless && compacts {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MonotoneOffsetsCompact",
        "a monotone offset column gap-codes (put_ef) losslessly AND strictly smaller than the absolute bit-packed column (put_cv)",
        "Witten, Moffat & Bell (1999) Managing Gigabytes: Compressing and Indexing Documents and Images, 2nd ed., §3.3 (gap/delta coding of monotone integer sequences)"
    );
}

pr4xis::register_axiom!(MonotoneOffsetsCompact, constructor);

/// The front-coded string dictionary is lossless for ANY input order AND
/// compacts a sorted dictionary with shared prefixes. `put_dict_fc` stores each
/// entry as `(shared-prefix-length, suffix)` against the PREVIOUS entry, so:
/// (1) it round-trips an arbitrarily-ordered dictionary exactly (the shared
/// count is computed, never assumed — losslessness does not depend on sorting);
/// and (2) for a SORTED dictionary whose neighbours share long prefixes (e.g.
/// IRIs under a common namespace) the encoding is STRICTLY smaller than the
/// plain length-prefixed dictionary (`put_dict`), because the shared prefix is
/// written once, not per entry. Non-tautological on BOTH legs: an implementation
/// that stored full strings would pass leg 1 (lossless) but fail leg 2
/// (compaction). Witten, Moffat & Bell (1999) §4.2 (front coding).
pub struct FrontCodingSharesPrefixes;

impl Axiom for FrontCodingSharesPrefixes {
    fn verify(&self) -> Verdict {
        // A sorted dictionary of IRIs under two shared namespaces — the shape
        // `to_succinct` produces after sorting+dedup of every text column.
        let sorted = alloc::vec![
            String::from("http://purl.org/spar/cito/agreesWith"),
            String::from("http://purl.org/spar/cito/cites"),
            String::from("http://purl.org/spar/cito/citesAsAuthority"),
            String::from("http://purl.org/spar/cito/isCitedBy"),
            String::from("http://wordnet.princeton.edu/synset-animal-noun-1"),
            String::from("http://wordnet.princeton.edu/synset-canine-noun-1"),
            String::from("http://wordnet.princeton.edu/synset-dog-noun-1"),
        ];

        // Leg 1 — lossless for ANY order: a deliberately UNSORTED permutation
        // round-trips exactly (front coding computes the shared count against
        // the previous entry, so correctness never depends on sort order).
        let unsorted = alloc::vec![
            sorted[3].clone(),
            sorted[0].clone(),
            sorted[6].clone(),
            sorted[1].clone(),
            sorted[4].clone(),
        ];
        let mut fc_unsorted = Vec::new();
        put_dict_fc(&mut fc_unsorted, &unsorted);
        let mut pos = 0usize;
        let lossless = get_dict_fc(&fc_unsorted, &mut pos) == unsorted;

        // Leg 2 — compaction: for the sorted, shared-prefix dictionary the
        // front-coded form is strictly smaller than the plain one.
        let mut fc = Vec::new();
        put_dict_fc(&mut fc, &sorted);
        let mut plain = Vec::new();
        put_dict(&mut plain, &sorted);
        let compacts = fc.len() < plain.len();

        if lossless && compacts {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "FrontCodingSharesPrefixes",
        "the front-coded dictionary (put_dict_fc) is lossless for any input order AND strictly smaller than the plain dictionary (put_dict) on a sorted shared-prefix dictionary",
        "Witten, Moffat & Bell (1999) Managing Gigabytes: Compressing and Indexing Documents and Images, 2nd ed., §4.2 (front coding)"
    );
}

pr4xis::register_axiom!(FrontCodingSharesPrefixes, constructor);

/// The raw-source envelope's DEFLATE transport is lossless, compacting, and
/// store-if-smaller — three legs over the REAL
/// [`raw_source_prx`](crate::applied::data_provisioning::raw_source_prx) codec
/// (the format-v2 envelope every committed byte-stream `.prx` ships in):
///
/// 1. **Lossless** (Deutsch 1996 RFC 1951; Witten, Moffat & Bell 1999 lossless
///    coding): `decode(encode(name, ver, bytes, Deflate)) == bytes` over a
///    compressible XML-shaped witness.
/// 2. **Compacting**: the `Deflate` envelope of that witness is STRICTLY
///    smaller than its `Identity` envelope — an encoder that tagged `Deflate`
///    but stored the bytes verbatim would round-trip and fail here.
/// 3. **Store-if-smaller**: over a fixed-seed high-entropy (incompressible)
///    witness, a REQUESTED `Deflate` emits an envelope byte-identical to the
///    `Identity` envelope — already-compressed upstream payloads (the `.tar.gz`
///    suites, the OOXML ZIP) can never grow. An encoder that compressed
///    unconditionally would fail here (DEFLATE expands incompressible input).
///
/// Non-tautological on every leg, and none is covered by the bit-packed-corpus
/// axioms above (a different codec over a different wire form).
pub struct RawSourceDeflateTransport;

impl Axiom for RawSourceDeflateTransport {
    fn verify(&self) -> Verdict {
        use crate::applied::data_provisioning::raw_source_prx::{
            PayloadEncoding, decode_raw_source, encode_raw_source,
        };

        // Leg 1 + 2 — a compressible XML-shaped witness (the shape of the real
        // XSD/DTD/LMF sources this envelope carries).
        let compressible: Vec<u8> = b"<xs:element name=\"statute\" type=\"uslm:StatuteType\"/>\n"
            .iter()
            .copied()
            .cycle()
            .take(4096)
            .collect();
        let deflated = encode_raw_source("witness", "1", &compressible, PayloadEncoding::Deflate);
        let identity = encode_raw_source("witness", "1", &compressible, PayloadEncoding::Identity);
        let lossless = matches!(
            decode_raw_source(&deflated),
            Ok((n, v, out)) if n == "witness" && v == "1" && out == compressible
        );
        let compacts = deflated.len() < identity.len();

        // Leg 3 — a fixed-seed xorshift64* high-entropy witness: requested
        // Deflate must downgrade to the Identity envelope, byte-identically.
        let mut x = 0x9E37_79B9_7F4A_7C15u64;
        let incompressible: Vec<u8> = core::iter::repeat_with(|| {
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            (x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 56) as u8
        })
        .take(2048)
        .collect();
        let downgraded =
            encode_raw_source("witness", "1", &incompressible, PayloadEncoding::Deflate)
                == encode_raw_source("witness", "1", &incompressible, PayloadEncoding::Identity);

        if lossless && compacts && downgraded {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "RawSourceDeflateTransport",
        "the raw-source envelope's DEFLATE transport round-trips byte-exactly, strictly compacts a compressible witness, and downgrades store-if-smaller to the identity envelope on incompressible bytes",
        "Deutsch, P. (1996) RFC 1951: DEFLATE Compressed Data Format Specification version 1.3; Witten, Moffat & Bell (1999) Managing Gigabytes, 2nd ed. — lossless coding"
    );
}

pr4xis::register_axiom!(RawSourceDeflateTransport, constructor);

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn all_succinct_codec_axioms_hold() {
        assert!(SuccinctCodecRoundTrip.verify().is_ok());
        assert!(MonotoneOffsetsCompact.verify().is_ok());
        assert!(FrontCodingSharesPrefixes.verify().is_ok());
        assert!(RawSourceDeflateTransport.verify().is_ok());
    }

    /// The compaction legs have teeth: front coding is not a CONSTANT win, it is
    /// a shared-prefix win. A dictionary of single characters — no shared
    /// prefixes — is NOT smaller front-coded than plain (the per-entry
    /// `shared = 0` varint is pure overhead). If `FrontCodingSharesPrefixes`
    /// passed for such a dictionary, its compaction leg would be vacuous; this
    /// proves it can fail on a real absence of prefix sharing.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn front_coding_compaction_needs_shared_prefixes() {
        let no_shared = alloc::vec![
            String::from("a"),
            String::from("b"),
            String::from("c"),
            String::from("d"),
        ];
        let mut fc = Vec::new();
        put_dict_fc(&mut fc, &no_shared);
        let mut plain = Vec::new();
        put_dict(&mut plain, &no_shared);
        assert!(
            fc.len() >= plain.len(),
            "with no shared prefixes, front coding must not be smaller than plain \
             (fc={}, plain={}) — else the compaction axiom would be vacuous",
            fc.len(),
            plain.len(),
        );
    }

    /// The round-trip axiom has teeth: the codec actually CARRIES the data, so
    /// it is not satisfiable by a degenerate/constant encoder. Two witnesses
    /// differing in exactly one edge each round-trip to THEMSELVES, and their
    /// encodings DIFFER — a constant encoder would collapse them and fail the
    /// round-trip for one. This proves `SuccinctCodecRoundTrip` can distinguish
    /// real values, not just pass vacuously.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn round_trip_distinguishes_distinct_witnesses() {
        let d = witness();
        let mut d2 = witness();
        d2.references = alloc::vec![(4u64, 3u64)]; // the one edge, reversed.
        assert_ne!(d, d2, "the two witnesses must genuinely differ");
        assert_eq!(OwnedCodegenData::from_succinct(&d.to_succinct()), d);
        assert_eq!(OwnedCodegenData::from_succinct(&d2.to_succinct()), d2);
        assert_ne!(
            d.to_succinct(),
            d2.to_succinct(),
            "distinct values must encode differently — a constant encoder is falsified"
        );
    }

    use proptest::prelude::*;

    /// A generated [`OwnedCodegenData`] over which the succinct round-trip must
    /// hold. Every field is independent (the codec serializes each column on its
    /// own and reads it back to its own length), so an arbitrary well-formed value
    /// round-trips — the strategy just bounds sizes and value ranges to keep the
    /// bit-packed widths small. `entity_count` is an independent header (not derived
    /// from the vec lengths), handles/endpoints are small integers, and words may
    /// repeat or be empty — all faithfully preserved by the codec.
    fn owned_codegen_strategy() -> impl Strategy<Value = OwnedCodegenData> {
        let text_col = || prop::collection::vec("[a-z]{0,6}", 0..5);
        let edge_table = || prop::collection::vec((0u64..30, 0u64..30), 0..5);
        let word_index =
            prop::collection::vec(("[a-z]{0,6}", prop::collection::vec(0u64..30, 0..4)), 0..4);
        (
            0u64..40,
            (text_col(), text_col(), text_col(), text_col()),
            word_index,
            (
                edge_table(),
                edge_table(),
                edge_table(),
                edge_table(),
                edge_table(),
                edge_table(),
            ),
        )
            .prop_map(
                |(
                    entity_count,
                    (entity_ids, entity_kind, entity_labels, entity_defs),
                    word_index,
                    (taxonomy, mereology, opposition, equivalence, causation, references),
                )| OwnedCodegenData {
                    entity_count,
                    entity_ids,
                    entity_kind,
                    entity_labels,
                    entity_defs,
                    word_index,
                    taxonomy,
                    mereology,
                    opposition,
                    equivalence,
                    causation,
                    references,
                },
            )
    }

    proptest! {
        /// ∀-strengthening of [`SuccinctCodecRoundTrip`]: over the generated space
        /// of `OwnedCodegenData`, `from_succinct(to_succinct(d)) == d`. The witness
        /// axiom fixes one WordNet-shaped value; this drives the compact codec over
        /// arbitrary column contents (empty/duplicate strings, variable CSR runs,
        /// every edge table populated).
        #[test]
        fn prop_succinct_codec_round_trips(d in owned_codegen_strategy()) {
            let back = OwnedCodegenData::from_succinct(&d.to_succinct());
            prop_assert_eq!(back, d);
        }
    }

    pr4xis::register_praxis_value!(prop_succinct_codec_round_trips, Deterministic);
}
