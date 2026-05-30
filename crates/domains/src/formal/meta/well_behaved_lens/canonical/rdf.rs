//! RDF canonicalization stub — W3C Recommendation
//! "RDF Dataset Canonicalization" (REC-rdf-canon-20240521,
//! Longley, Kellogg & Yamamoto 2024,
//! <https://www.w3.org/TR/rdf-canon/>).
//!
//! No maintained Rust crate implements the W3C RDFC-1.0 algorithm
//! at the time of the M4.θ.0 survey (the Recommendation landed
//! May 2024 and the Rust ecosystem's RDF stack — `oxrdf`,
//! `sophia`, `rio` — has hashing-based isomorphism support but not
//! the deterministic blank-node-labeling algorithm of RDFC-1.0).
//! The citation is registered in `citations.toml` so the M4.θ.2
//! test harness can detect any RDF source kind and emit a clear
//! "RDF canonicalization not yet implemented" message rather than
//! silently passing.
//!
//! When a viable crate appears (or we implement the canonicalization
//! algorithm in §4 of the W3C Recommendation) this stub is
//! replaced with the working implementation.

use alloc::vec::Vec;

use super::CanonicalizationError;

const FORM: &str = "rdf-canon-rec-20240521";

/// Stub: returns an "unimplemented" error.
///
/// The error is structured rather than a panic so the round-trip
/// harness can surface "RDF canonicalization not yet implemented"
/// to operators when an RDF source kind is encountered.
pub fn canonicalize(_bytes: &[u8]) -> Result<Vec<u8>, CanonicalizationError> {
    Err(CanonicalizationError::new(
        FORM,
        "W3C RDF Dataset Canonicalization (REC-rdf-canon-20240521) is registered but \
         not yet implemented; no maintained Rust crate currently exists. Tracking \
         under M4.θ.0.",
    ))
}
