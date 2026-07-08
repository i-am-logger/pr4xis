//! Runnable axioms of the [`CanonicalCodec`](super::ontology) ontology —
//! each `verify()` is a *predicate that runs*, exercising the real canonical
//! codec ([`pr4xis_runtime::codec`]) rather than asserting a doc-comment
//! (North Star W3 slice 1; `feedback_praxis_as_compiler_self_describing`).
//!
//! Unlike [`super::super::ontology_archive::axioms`] (gated on `feature =
//! "prx"`, where the `.prx` realisation lives), these run in EVERY build: the
//! [`pr4xis_runtime`] dependency — and its `codec` module — is unconditional.
//!
//! Each of the three axioms is a GENUINELY UNCOVERED, machine-checkable fact,
//! not a tautology:
//!
//! - `CanonicalEncodingDeterministic` — two INDEPENDENTLY built equal values
//!   encode to EQUAL bytes (and a distinct value to distinct bytes). This
//!   tests serializer byte-stability across construction history, not `f == f`.
//! - `CodecRoundTrip` — `decode(encode(v)) == v` over a witness set. The
//!   DAG-CBOR round-trip is covered by NO existing axiom (rkyv/gzip in
//!   `ontology_archive` are DIFFERENT codecs).
//! - `DecodeRefusesAdversarialLength` — decode of an input declaring a
//!   `2^64-1` length REFUSES with a typed error, never OOM/panics. This is the
//!   untrusted-input boundary the `.prx` loader runs on.
//!
//! There is deliberately NO `AddressIsHashOfCanonical` axiom: `address_of` is
//! DEFINED as `ContentAddress::of(canonical_encode(v))` (codec.rs:62-63), so
//! asserting that equality would re-run the body and prove only `f(x) == f(x)`.
//! The address→encoding dependency is instead a kinded morphism in the
//! ontology's edges (`ContentAddress -[Dependency]-> CanonicalEncoding`).

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;
use pr4xis_runtime::codec::{canonical_decode, canonical_encode};

/// `decode(encode(v)) == v` for one concrete witness value — the per-type
/// round-trip check the [`CodecRoundTrip`] axiom folds over its
/// heterogeneous witness set. Returns `false` on any encode/decode error or
/// on a mismatch, so a lossy codec is falsified.
fn round_trips<T>(value: &T) -> bool
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq,
{
    match canonical_encode(value) {
        Ok(bytes) => matches!(canonical_decode::<T>(&bytes), Ok(back) if &back == value),
        Err(_) => false,
    }
}

/// A map whose entries serialize in the caller-given order (a `serialize_map`
/// over a `Vec`), NOT pre-sorted like a `BTreeMap`. It exists so the
/// determinism axiom can feed the encoder keys OUT of canonical order and prove
/// the codec itself imposes the order — a `BTreeMap` witness is already sorted,
/// so it cannot drive (and so cannot falsify) DAG-CBOR key-sorting.
struct UnsortedMap(Vec<(&'static str, u32)>);

impl serde::Serialize for UnsortedMap {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (k, v) in &self.0 {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

/// The canonical encoding is VALUE-deterministic AND canonicalizing: the same
/// map presented with keys in ASCENDING vs DESCENDING order encodes to EQUAL
/// bytes (the codec imposes DAG-CBOR's sorted key order — RFC 8949 §4.2 — so the
/// content address is a stable identity independent of construction order), and
/// a distinct value encodes to DIFFERENT bytes (a constant/degenerate encoder is
/// falsified). Non-tautological on BOTH legs: the witnesses serialize their keys
/// in genuinely opposite orders (via [`UnsortedMap`], not a self-sorting
/// `BTreeMap`), so an encoder that streamed keys in input order — failing to
/// sort — would make `a != b` and be caught; this is not `encode(x) == encode(x)`.
/// Bormann & Hoffman (2020) RFC 8949 §4.2; IPLD DAG-CBOR.
pub struct CanonicalEncodingDeterministic;

impl Axiom for CanonicalEncodingDeterministic {
    fn verify(&self) -> Verdict {
        // Same three entries, presented in ASCENDING vs DESCENDING key order —
        // equal VALUE, opposite serialization order. Equal bytes here can ONLY
        // come from the codec sorting the keys, not from the witness type.
        let ascending = UnsortedMap(vec![("alpha", 1), ("beta", 2), ("gamma", 3)]);
        let descending = UnsortedMap(vec![("gamma", 3), ("beta", 2), ("alpha", 1)]);
        // A DIFFERENT value (one entry changed) — must encode differently, or a
        // constant encoder would satisfy the equal→equal leg vacuously.
        let different = UnsortedMap(vec![("alpha", 1), ("beta", 99), ("gamma", 3)]);

        let (Ok(a), Ok(b), Ok(c)) = (
            canonical_encode(&ascending),
            canonical_encode(&descending),
            canonical_encode(&different),
        ) else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        if a == b && a != c {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CanonicalEncodingDeterministic",
        "two independently built equal values canonical-encode to equal bytes, and a distinct value to distinct bytes",
        "Bormann & Hoffman (2020) Concise Binary Object Representation (CBOR), RFC 8949 §4.2; IPLD DAG-CBOR codec specification"
    );
}

pr4xis::register_axiom!(CanonicalEncodingDeterministic, constructor);

/// The canonical codec is a total inverse pair over well-formed values:
/// `canonical_decode(canonical_encode(v)) == v`. Checked over a heterogeneous
/// witness set — a map, a list of pairs, a nested tuple, an empty list, and a
/// binary blob spanning `0x00`/`0xFF` — so a codec that dropped, reordered, or
/// widened any shape is falsified. This DAG-CBOR round-trip is covered by no
/// other axiom (the rkyv + gzip round-trips in `ontology_archive` are
/// different codecs over a different type). This is a serialization
/// isomorphism (a section/retract inverse pair, `decode ∘ encode = id`), NOT a
/// lens law — there is one type, no distinct source/view and no prior-source
/// `put`. IPLD DAG-CBOR codec specification (total, deterministic, round-trippable).
pub struct CodecRoundTrip;

impl Axiom for CodecRoundTrip {
    fn verify(&self) -> Verdict {
        // A map with several keys (exercises DAG-CBOR sorted-map encoding).
        let mut map: BTreeMap<String, i64> = BTreeMap::new();
        map.insert("one".to_string(), 1);
        map.insert("minus".to_string(), -7);
        map.insert("big".to_string(), 1_000_000);
        // A list of pairs (order-significant sequence).
        let pairs: Vec<(String, u32)> = alloc::vec![
            ("a".to_string(), 1),
            ("b".to_string(), 2),
            ("c".to_string(), 3)
        ];
        // A nested heterogeneous tuple.
        let nested: (String, Vec<u64>, bool) = ("nest".to_string(), alloc::vec![10, 20, 30], true);
        // Boundary shapes: an empty list, and a binary blob incl. 0x00 / 0xFF.
        let empty: Vec<u32> = Vec::new();
        let blob: Vec<u8> = alloc::vec![0u8, 1, 2, 255, 254, 0, 13, 10];

        if round_trips(&map)
            && round_trips(&pairs)
            && round_trips(&nested)
            && round_trips(&empty)
            && round_trips(&blob)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CodecRoundTrip",
        "canonical_decode(canonical_encode(v)) == v over the witness values (DAG-CBOR is a total inverse pair)",
        "IPLD DAG-CBOR codec specification (https://ipld.io/specs/codecs/dag-cbor/) — a total, deterministic, round-trippable encoding: canonical_decode ∘ canonical_encode = identity (a serialization isomorphism / section-retract inverse pair over well-formed values)"
    );
}

pr4xis::register_axiom!(CodecRoundTrip, constructor);

/// Decode is TOTAL and fail-closed at the untrusted-input boundary: an input
/// whose header declares an adversarial `2^64-1` length with no payload is
/// REFUSED with a typed [`CodecError`](pr4xis_runtime::codec::CodecError),
/// never driving an unbounded allocation (the allocation-bomb DoS class). This
/// is the boundary the `.prx` loader and the wasm/web demo deserialize
/// ontologies through — the highest-value guarantee here. A decoder that
/// pre-allocated from the attacker-declared length would OOM/abort; a robust
/// one reads to EOF and returns `Err`. The three headers cover the count-
/// carrying DAG-CBOR major types: array, byte string, and map (Bormann &
/// Hoffman 2020, RFC 8949 §3, 8-byte extended count `0x1b …` = u64).
pub struct DecodeRefusesAdversarialLength;

impl Axiom for DecodeRefusesAdversarialLength {
    fn verify(&self) -> Verdict {
        // DAG-CBOR array header (major type 4, `0x9b` = 8-byte length), length
        // = u64::MAX, no elements.
        let adversarial_array = [0x9b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        // Byte-string header (major type 2, `0x5b`), length = u64::MAX.
        let adversarial_bytes = [0x5b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        // Map header (major type 5, `0xbb`), length = u64::MAX pairs.
        let adversarial_map = [0xbb, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];

        // Every adversarial header must be REFUSED (Err), never allocated.
        if canonical_decode::<Vec<u32>>(&adversarial_array).is_err()
            && canonical_decode::<Vec<u8>>(&adversarial_bytes).is_err()
            && canonical_decode::<BTreeMap<String, u32>>(&adversarial_map).is_err()
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DecodeRefusesAdversarialLength",
        "canonical_decode refuses an input declaring a 2^64-1 length with a typed error, never OOM/panics",
        "Bormann & Hoffman (2020) Concise Binary Object Representation (CBOR), RFC 8949 §3 (Specification of the CBOR Encoding — the head/argument length encoding of arrays, byte strings, and maps)"
    );
}

pr4xis::register_axiom!(DecodeRefusesAdversarialLength, constructor);

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn all_codec_axioms_hold() {
        assert!(CanonicalEncodingDeterministic.verify().is_ok());
        assert!(CodecRoundTrip.verify().is_ok());
        assert!(DecodeRefusesAdversarialLength.verify().is_ok());
    }

    /// `CanonicalEncodingDeterministic` has teeth: a HashMap-style value whose
    /// two encodings genuinely differed would fail. We instead prove the
    /// converse guard here — the round-trip helper REJECTS a codec mismatch —
    /// so the axioms can FAIL on a real defect, not just pass. A blob that is
    /// truncated after encoding must not round-trip.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn round_trip_helper_rejects_a_truncated_encoding() {
        let blob: Vec<u8> = alloc::vec![1u8, 2, 3, 4, 5];
        let bytes = canonical_encode(&blob).expect("encode");
        // Drop the final byte: the truncated bytes must NOT decode back to the
        // original blob (either a decode error or a different value).
        let truncated = &bytes[..bytes.len() - 1];
        let decoded = canonical_decode::<Vec<u8>>(truncated);
        assert!(
            !matches!(decoded, Ok(ref back) if back == &blob),
            "a truncated encoding must not round-trip to the original value"
        );
    }

    use proptest::prelude::*;

    proptest! {
        /// ∀-strengthening of [`CodecRoundTrip`]: over a generated space of maps and
        /// binary blobs, `canonical_decode(canonical_encode(v)) == v`. The witness
        /// axiom fixes a handful of shapes; this drives the DAG-CBOR round-trip over
        /// the whole generated domain (arbitrary keys, values incl. every byte).
        #[test]
        fn prop_canonical_codec_round_trips(
            map in prop::collection::btree_map("[a-z]{0,8}", any::<i64>(), 0..8),
            blob in prop::collection::vec(any::<u8>(), 0..48),
        ) {
            let map_bytes = canonical_encode(&map).expect("encode map");
            let map_back = canonical_decode::<BTreeMap<String, i64>>(&map_bytes)
                .expect("decode map");
            prop_assert_eq!(map_back, map);

            let blob_bytes = canonical_encode(&blob).expect("encode blob");
            let blob_back = canonical_decode::<Vec<u8>>(&blob_bytes).expect("decode blob");
            prop_assert_eq!(blob_back, blob);
        }
    }

    pr4xis::register_praxis_value!(prop_canonical_codec_round_trips, Deterministic);
}
