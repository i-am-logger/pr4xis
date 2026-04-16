//! `GitObjectSha` extractor — stub.
//!
//! Returns `Unverifiable` until a real implementation lands. The axiom
//! `EverySchemeHasAnExtractor` still holds because this function exists;
//! `VerificationFailClosed` ensures pipelines reject artifacts that claim
//! this scheme until the real extractor is wired up.

use super::super::ontology::{IdentityClaim, VerificationResult};

/// Stub verifier. Always returns `Unverifiable`.
pub fn verify(_claim: &IdentityClaim, _bytes: &[u8]) -> VerificationResult {
    VerificationResult::Unverifiable {
        reason: "GitObjectSha scheme not yet implemented".into(),
    }
}
