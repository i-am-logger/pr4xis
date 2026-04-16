//! `RawHash` extractor — real implementation.
//!
//! The `ContentHash::RawHash` leaf. Computes a cryptographic hash over the
//! supplied bytes and compares against the declared hash in the `ClaimData`.
//! This is the Dolstra 2006 baseline scheme and the fallback for any source
//! without self-description.

use super::super::ontology::{ClaimData, IdentityClaim, VerificationResult};
use sha2::{Digest, Sha256};

/// Verify a `RawHash` claim against a byte slice.
///
/// Expects `claim.data` to be `ClaimData::Sha256(_)`. Computes sha256 over
/// `bytes` and compares the hex digest against the expected value.
/// Case-insensitive comparison (hex is case-insensitive).
///
/// Returns `Verified` if they match, `Mismatch` if they don't,
/// `Unverifiable` if the claim is not a `Sha256` variant.
pub fn verify(claim: &IdentityClaim, bytes: &[u8]) -> VerificationResult {
    let expected = match &claim.data {
        ClaimData::Sha256(hex) => hex,
        ClaimData::HashAlgorithm { .. } => {
            return VerificationResult::Unverifiable {
                reason: "HashAlgorithm variant not yet implemented; use Sha256 for now".into(),
            };
        }
        _ => {
            return VerificationResult::Unverifiable {
                reason: "RawHash extractor expected Sha256 ClaimData".into(),
            };
        }
    };

    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let actual = hex::encode(hasher.finalize());

    if actual.eq_ignore_ascii_case(expected) {
        VerificationResult::Verified(claim.clone())
    } else {
        VerificationResult::Mismatch {
            expected: expected.clone(),
            actual,
        }
    }
}
