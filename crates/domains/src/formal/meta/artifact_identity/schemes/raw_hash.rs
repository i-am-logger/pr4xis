//! `RawHash` extractor — real implementation.
//!
//! The `ContentHash::RawHash` leaf. Computes a cryptographic hash over the
//! supplied bytes and compares against the declared hash in the `ClaimData`.
//! This is the Dolstra 2006 baseline scheme and the fallback for any source
//! without self-description.

use super::super::ontology::{ClaimData, HashAlgorithm, IdentityClaim, VerificationResult};
use alloc::string::String;
use sha2::{Digest, Sha256, Sha512};

/// Hex digest of `bytes` under a named [`HashAlgorithm`] — the W3C SRI
/// multi-algorithm integrity primitive. The enum admits only strong
/// functions (SHA-256 / SHA-512 from FIPS 180-4, BLAKE3 from Aumasson et al.
/// 2020); weak functions (MD5, SHA-1) are *unrepresentable*, so "refuse weak
/// algorithms" is a type invariant, not a runtime branch.
pub fn hash_hex(algorithm: HashAlgorithm, bytes: &[u8]) -> String {
    match algorithm {
        HashAlgorithm::Sha256 => {
            let mut h = Sha256::new();
            h.update(bytes);
            hex::encode(h.finalize())
        }
        HashAlgorithm::Sha512 => {
            let mut h = Sha512::new();
            h.update(bytes);
            hex::encode(h.finalize())
        }
        HashAlgorithm::Blake3 => hex::encode(blake3::hash(bytes).as_bytes()),
    }
}

/// Verify a `RawHash` claim against a byte slice.
///
/// Handles both the legacy [`ClaimData::Sha256`] shorthand and the
/// multi-algorithm [`ClaimData::HashAlgorithm`]: it recomputes the digest
/// under the claim's named algorithm via [`hash_hex`] and compares against
/// the expected value (hex is case-insensitive).
///
/// Returns `Verified` if they match, `Mismatch` if they don't,
/// `Unverifiable` if the claim is not a hash variant.
pub fn verify(claim: &IdentityClaim, bytes: &[u8]) -> VerificationResult {
    let (algorithm, expected) = match &claim.data {
        ClaimData::Sha256(hex) => (HashAlgorithm::Sha256, hex),
        ClaimData::HashAlgorithm {
            algorithm,
            digest_hex,
        } => (*algorithm, digest_hex),
        _ => {
            return VerificationResult::Unverifiable {
                reason: "RawHash extractor expected Sha256 or HashAlgorithm ClaimData".into(),
            };
        }
    };

    let actual = hash_hex(algorithm, bytes);
    if actual.eq_ignore_ascii_case(expected) {
        VerificationResult::Verified(claim.clone())
    } else {
        VerificationResult::Mismatch {
            expected: expected.clone(),
            actual,
        }
    }
}
