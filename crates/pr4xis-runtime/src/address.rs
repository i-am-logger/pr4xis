//! Content-addressing — the one primitive the runtime GROUNDS.
//!
//! A [`ContentAddress`] is the SHA-256 (NIST FIPS 180-4 §6.2) of a CANONICAL
//! byte encoding of a definition. Everything else about the `.prx` format is
//! learned from the meta-`.prx`; this is the bottom of the reflexive tower —
//! it is what "reference" and "agreement" MEAN. Two peers agree a definition is
//! the same iff they hash the same canonical bytes to the same address.
//!
//! The canonical ENCODING (which bytes are fed in) is the codec layer's concern
//! — the target is a multihash-tagged DAG-CBOR canonical form — and the
//! *definition* fed in is the concept's `morphisms_from` closure + axioms +
//! lexical entry, NOT its name (definition-bearing addressing, which closes the
//! G5 wire gap). This primitive is deliberately encoding-agnostic: it grounds
//! only the hash, so the codec and the definition-encoding can be chosen and
//! evolved without changing what identity *means*.

use sha2::{Digest, Sha256};

/// The content address of a canonical byte encoding: a SHA-256 digest.
///
/// The runtime never trusts a self-asserted address — it re-derives the address
/// from the bytes it is about to admit and compares (the fail-closed load gate);
/// a mismatch is rejected. `Ord` so addresses key the Merkle-DAG's `BTreeMap`s
/// deterministically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContentAddress([u8; 32]);

impl ContentAddress {
    /// Ground primitive: the content address of `canonical_bytes`. This is the
    /// ONE computation the runtime grounds — `.prx` identity bottoms out here.
    pub fn of(canonical_bytes: &[u8]) -> Self {
        Self(Sha256::digest(canonical_bytes).into())
    }

    /// The raw 32-byte digest.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Lowercase hex (64 chars) — the form `praxis.lock` pins use, so a
    /// `ContentAddress` and a committed pin compare directly.
    pub fn to_hex(&self) -> String {
        use std::fmt::Write;
        let mut s = String::with_capacity(64);
        for b in &self.0 {
            // Infallible: writing to a `String` never errors.
            write!(s, "{b:02x}").expect("writing to a String is infallible");
        }
        s
    }

    /// Parse a 64-character lowercase-hex digest. `None` if the length is wrong
    /// or any character is not a hex digit.
    pub fn from_hex(hex: &str) -> Option<Self> {
        if hex.len() != 64 {
            return None;
        }
        let mut out = [0u8; 32];
        for (byte, pair) in out.iter_mut().zip(hex.as_bytes().chunks_exact(2)) {
            let hi = (pair[0] as char).to_digit(16)?;
            let lo = (pair[1] as char).to_digit(16)?;
            *byte = ((hi << 4) | lo) as u8;
        }
        Some(Self(out))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_is_deterministic() {
        assert_eq!(ContentAddress::of(b"praxis"), ContentAddress::of(b"praxis"));
    }

    #[test]
    fn different_bytes_yield_different_address() {
        assert_ne!(ContentAddress::of(b"a"), ContentAddress::of(b"b"));
    }

    #[test]
    fn hex_round_trips() {
        let a = ContentAddress::of(b"the ground");
        let hex = a.to_hex();
        assert_eq!(hex.len(), 64);
        assert_eq!(ContentAddress::from_hex(&hex), Some(a));
    }

    #[test]
    fn from_hex_rejects_malformed() {
        assert_eq!(ContentAddress::from_hex("xyz"), None); // wrong length
        assert_eq!(ContentAddress::from_hex(&"z".repeat(64)), None); // non-hex
        assert_eq!(ContentAddress::from_hex(&"a".repeat(63)), None); // off-by-one
    }

    #[test]
    fn matches_sha256_known_answer() {
        // NIST KAT: SHA-256("") = e3b0c442...b855.
        assert_eq!(
            ContentAddress::of(b"").to_hex(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
