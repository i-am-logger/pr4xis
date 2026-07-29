//! Content-addressing — the one primitive the runtime GROUNDS.
//!
//! A [`ContentAddress`] is the BLAKE3 hash (Aumasson, O'Connor, Neves &
//! Wilcox-O'Hearn 2020, "BLAKE3: one function, fast everywhere") of a
//! CANONICAL byte encoding of a definition. BLAKE3 is a sound tree-mode hash
//! (Gunsing, CRYPTO 2022, under the Bertoni–Daemen sound-tree-hashing
//! conditions), so the same function supports incrementally verified
//! streaming (Bao — O'Connor) without changing what an address means.
//! Everything else about the `.prx` format is learned from the meta-`.prx`;
//! this is the bottom of the reflexive tower — it is what "reference" and
//! "agreement" MEAN. Two peers agree a definition is the same iff they hash
//! the same canonical bytes to the same address.
//!
//! The canonical ENCODING (which bytes are fed in) is the codec layer's concern
//! — the target is a multihash-tagged DAG-CBOR canonical form — and the
//! *definition* fed in is the concept's `morphisms_from` closure + axioms +
//! lexical entry, NOT its name (definition-bearing addressing, which closes the
//! G5 wire gap). This primitive is deliberately encoding-agnostic: it grounds
//! only the hash, so the codec and the definition-encoding can be chosen and
//! evolved without changing what identity *means*.

use sha2::{Digest, Sha256, Sha512};

/// The hash functions a content address (or an integrity claim over one) may
/// name. The enum admits only strong functions — BLAKE3 (Aumasson, O'Connor,
/// Neves & Wilcox-O'Hearn 2020) and SHA-256 / SHA-512 (NIST FIPS 180-4) — so
/// weak functions (MD5, SHA-1)
/// are *unrepresentable*: "refuse weak algorithms" is a type invariant, not a
/// runtime branch. Praxis EMITS addresses under exactly one algorithm per
/// format epoch ([`ADDRESS_ALGORITHM`]); it VERIFIES claims under any variant
/// here (the W3C SRI verify-many discipline).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HashAlgorithm {
    /// NIST FIPS 180-4 §6.2.
    Sha256,
    /// NIST FIPS 180-4 §6.4.
    Sha512,
    /// Aumasson, O'Connor, Neves & Wilcox-O'Hearn (2020).
    Blake3,
}

/// The ONE algorithm praxis emits content addresses under in this format
/// epoch. Changing it is a deliberate format-version event (every pin and
/// known-answer constant re-blesses), never a per-address choice: the
/// fail-closed gates re-derive addresses from the bytes they admit, so the
/// algorithm comes from verifier policy — a payload never selects its own
/// verifier.
pub const ADDRESS_ALGORITHM: HashAlgorithm = HashAlgorithm::Blake3;

/// Lowercase-hex digest of `bytes` under a named [`HashAlgorithm`] — the
/// multi-algorithm verify leg. [`ContentAddress::of`] is the emit leg and
/// always uses [`ADDRESS_ALGORITHM`].
pub fn hash_hex(algorithm: HashAlgorithm, bytes: &[u8]) -> String {
    fn hex(bytes: &[u8]) -> String {
        use std::fmt::Write;
        let mut s = String::with_capacity(bytes.len() * 2);
        for b in bytes {
            write!(s, "{b:02x}").expect("writing to a String is infallible");
        }
        s
    }
    match algorithm {
        HashAlgorithm::Sha256 => hex(&Sha256::digest(bytes)),
        HashAlgorithm::Sha512 => hex(&Sha512::digest(bytes)),
        HashAlgorithm::Blake3 => hex(blake3::hash(bytes).as_bytes()),
    }
}

/// The content address of a canonical byte encoding: a BLAKE3 digest
/// (Aumasson, O'Connor, Neves & Wilcox-O'Hearn 2020).
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
        Self(*blake3::hash(canonical_bytes).as_bytes())
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

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn address_is_deterministic() {
        assert_eq!(ContentAddress::of(b"praxis"), ContentAddress::of(b"praxis"));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn different_bytes_yield_different_address() {
        assert_ne!(ContentAddress::of(b"a"), ContentAddress::of(b"b"));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn hex_round_trips() {
        let a = ContentAddress::of(b"the ground");
        let hex = a.to_hex();
        assert_eq!(hex.len(), 64);
        assert_eq!(ContentAddress::from_hex(&hex), Some(a));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn from_hex_rejects_malformed() {
        assert_eq!(ContentAddress::from_hex("xyz"), None); // wrong length
        assert_eq!(ContentAddress::from_hex(&"z".repeat(64)), None); // non-hex
        assert_eq!(ContentAddress::from_hex(&"a".repeat(63)), None); // off-by-one
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn matches_blake3_known_answer() {
        // Official BLAKE3 empty-input vector (BLAKE3 team test_vectors.json,
        // input_len = 0): af1349b9...3262.
        assert_eq!(
            ContentAddress::of(b"").to_hex(),
            "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
        );
    }
}
