//! RDF canonicalization stub — IETF RFC 9595 (Longley & Sporny 2024,
//! <https://www.rfc-editor.org/rfc/rfc9595.html>).
//!
//! No maintained Rust crate implements RFC 9595's RDF Dataset
//! Canonicalization at the time of the M4.θ.0 survey (the RFC
//! landed 2024 and the Rust ecosystem's RDF stack — `oxrdf`,
//! `sophia`, `rio` — has hashing-based isomorphism support but not
//! the RFC 9595 deterministic blank-node-labeling algorithm). The
//! citation is registered in `citations.toml` so the M4.θ.2 test
//! harness can detect any RDF source kind and emit a clear
//! "RDF canonicalization not yet implemented" message rather than
//! silently passing.
//!
//! When a viable crate appears (or we implement the dataset-
//! canonicalization algorithm in §4 of the RFC) this stub is
//! replaced with the working implementation.

use alloc::vec::Vec;

use super::CanonicalizationError;

const FORM: &str = "rdf-rfc-9595";

/// Stub: returns an "unimplemented" error.
///
/// The error is structured rather than a panic so the round-trip
/// harness can surface "RDF canonicalization not yet implemented"
/// to operators when an RDF source kind is encountered.
pub fn canonicalize(_bytes: &[u8]) -> Result<Vec<u8>, CanonicalizationError> {
    Err(CanonicalizationError::new(
        FORM,
        "RFC 9595 RDF Dataset Canonicalization is registered but not yet implemented; \
         no maintained Rust crate currently exists. Tracking under M4.θ.0.",
    ))
}
