//! Canonical codec — DAG-CBOR, the deterministic encoding the content address
//! is computed over.
//!
//! Content-addressing REQUIRES a single canonical encoding so that identical
//! data → identical bytes → the identical [`ContentAddress`], reproducibly
//! across implementations and toolchains. DAG-CBOR is that form (sorted map
//! keys, shortest-form integers, no indefinite-length items). `rkyv` — used
//! elsewhere as a local zero-copy cache — is explicitly NOT the address codec:
//! its byte layout is version/feature/target-bound, so committing or sharing
//! rkyv bytes is a cross-toolchain liability. The *address* is over this form.
//!
//! Determinism here rests on the encoded values being order-stable by
//! construction (the runtime's serialized structures use `BTreeMap`/`Vec`, and
//! the morphism/closure rows are sorted before encoding), so the canonical
//! bytes do not depend on iteration order.
//!
//! Citation: IPLD DAG-CBOR codec specification
//! (<https://ipld.io/specs/codecs/dag-cbor/>); RFC 8949 (CBOR) §4.2
//! (deterministically encoded CBOR).

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::address::ContentAddress;

/// Errors from the canonical (DAG-CBOR) codec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecError {
    /// The value could not be encoded as DAG-CBOR.
    Encode(String),
    /// The bytes could not be decoded as DAG-CBOR (malformed, or not the
    /// expected shape).
    Decode(String),
}

impl core::fmt::Display for CodecError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CodecError::Encode(e) => write!(f, "DAG-CBOR encode: {e}"),
            CodecError::Decode(e) => write!(f, "DAG-CBOR decode: {e}"),
        }
    }
}

impl std::error::Error for CodecError {}

/// Encode `value` to its canonical DAG-CBOR bytes — the form the content
/// address is taken over. Equal (order-stable) values produce equal bytes.
pub fn canonical_encode<T: Serialize>(value: &T) -> Result<Vec<u8>, CodecError> {
    serde_ipld_dagcbor::to_vec(value).map_err(|e| CodecError::Encode(e.to_string()))
}

/// Decode `bytes` (canonical DAG-CBOR) back into a value — the inverse of
/// [`canonical_encode`].
pub fn canonical_decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, CodecError> {
    serde_ipld_dagcbor::from_slice(bytes).map_err(|e| CodecError::Decode(e.to_string()))
}

/// The content address of `value`: hash its canonical DAG-CBOR encoding. This
/// is the bridge from "a definition" (any serializable value) to its place in
/// the Merkle-DAG — `address_of(definition)` is the node's identity.
pub fn address_of<T: Serialize>(value: &T) -> Result<ContentAddress, CodecError> {
    Ok(ContentAddress::of(&canonical_encode(value)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn encoding_is_deterministic() {
        let mut m = BTreeMap::new();
        m.insert("a".to_string(), 1u32);
        m.insert("b".to_string(), 2u32);
        assert_eq!(canonical_encode(&m).unwrap(), canonical_encode(&m).unwrap());
    }

    // `canonical_decode` is the substrate's untrusted-input boundary — the `.prx`
    // loader (and the wasm/web demo) deserialize ontologies through it. Honest at
    // this boundary is totality: an adversarial archive must be REFUSED with an
    // error, never drive an unbounded allocation from an attacker-declared length
    // (the allocation-bomb DoS class). These feed a length prefix claiming 2^64-1
    // items with no payload; a decoder that pre-allocates would OOM/abort, a
    // robust one reads to EOF and returns Err.

    #[test]
    fn decode_refuses_huge_array_length_without_oom() {
        // DAG-CBOR array header (major type 4), 8-byte length = u64::MAX.
        let adversarial = [0x9b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        assert!(canonical_decode::<Vec<u32>>(&adversarial).is_err());
    }

    #[test]
    fn decode_refuses_huge_byte_string_length_without_oom() {
        // DAG-CBOR byte-string header (major type 2), 8-byte length = u64::MAX.
        let adversarial = [0x5b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        assert!(canonical_decode::<Vec<u8>>(&adversarial).is_err());
    }

    #[test]
    fn decode_refuses_huge_map_length_without_oom() {
        // DAG-CBOR map header (major type 5), 8-byte length = u64::MAX.
        let adversarial = [0xbb, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        assert!(canonical_decode::<BTreeMap<String, u32>>(&adversarial).is_err());
    }

    #[test]
    fn equal_values_share_an_address() {
        let v = ("functor", vec!["A", "B"], 3u8);
        assert_eq!(address_of(&v).unwrap(), address_of(&v).unwrap());
    }

    #[test]
    fn distinct_values_get_distinct_addresses() {
        assert_ne!(address_of(&"A").unwrap(), address_of(&"B").unwrap());
    }

    #[test]
    fn structurally_equal_maps_address_equal_regardless_of_build_order() {
        // BTreeMap is sorted by construction, so two maps with the same
        // entries built in different orders are the same value — and DAG-CBOR
        // gives them the same address. (The runtime's structures are sorted
        // before encoding for exactly this reason.)
        let mut m1 = BTreeMap::new();
        m1.insert("z", 1u32);
        m1.insert("a", 2u32);
        let mut m2 = BTreeMap::new();
        m2.insert("a", 2u32);
        m2.insert("z", 1u32);
        assert_eq!(address_of(&m1).unwrap(), address_of(&m2).unwrap());
    }

    #[test]
    fn decode_inverts_encode() {
        let v: Vec<(String, u32)> = vec![("a".into(), 1), ("b".into(), 2)];
        let bytes = canonical_encode(&v).unwrap();
        let back: Vec<(String, u32)> = canonical_decode(&bytes).unwrap();
        assert_eq!(v, back);
    }
}
