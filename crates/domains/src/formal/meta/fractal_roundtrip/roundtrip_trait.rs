//! The `FractalRoundTrip` trait — signature-of-understanding for
//! loaded sources.
//!
//! ## Categorical statement
//!
//! Let `S` be the category of byte streams encoding a particular
//! source kind (a USLM XML file, a WordNet LMF XML file, a JSON
//! document, a `praxis.toml` manifest, …) and let `O` be the Praxis
//! ontology that types `S` after parsing.
//!
//! Praxis declares an *adjunction* `parse ⊣ reemit`, i.e. a pair of
//! functors `parse: S → O` and `reemit: O → S` with unit and counit
//! natural transformations satisfying the triangle identities (Mac
//! Lane *Categories for the Working Mathematician*, 2nd ed., §IV.1).
//! The minimum guarantee an adjunction gives is *natural* unit/counit
//! — not that they are identities.
//!
//! When the adjunction is in fact an *equivalence of categories*
//! (Mac Lane §IV.4 Theorem 1), unit and counit compose to identity
//! up to natural isomorphism — every object on each side has a
//! mirror on the other, and the round-trip is faithful.
//!
//! `FractalRoundTrip` is the runtime witness that the adjunction
//! between a specific `S` and its `O` reaches equivalence-of-
//! categories status. For every byte stream `b ∈ S` it asserts
//!
//!   sig(b) == sig(reemit(parse(b)))
//!
//! where `sig = SHA-256 ∘ canonical` and `canonical` is the
//! source kind's published canonical form (W3C XML C14N 1.1 for
//! XML, RFC 8785 JCS for JSON, Unicode TR #15 NFKC for plain text,
//! RFC 9595 for RDF, etc.). A hash mismatch is concrete evidence
//! that some structural detail of `b` is not yet reflected in `O`
//! — an ontology gap.
//!
//! ## What this module ships
//!
//! - The [`FractalRoundTrip`] trait itself (this file).
//! - The [`FractalRoundTripFailure`] error type that carries the
//!   input and round-tripped digests + an optional structured diff.
//! - The [`crate::formal::meta::fractal_roundtrip::canonical`]
//!   library of per-source canonical-form implementations.
//!
//! It does *not* run round-trip against any specific loaded source.
//! That lands in the M4.θ.2 test harness, which iterates the
//! sources manifest and asserts `FractalRoundTrip::assert_round_trip`
//! for each.

use alloc::{string::String, vec::Vec};
use core::fmt;

use sha2::{Digest, Sha256};

/// The signature-of-understanding trait.
///
/// A source kind implementing `FractalRoundTrip` declares that
/// Praxis can parse a byte stream of that kind, reconstruct it from
/// the parsed ontology instance, and produce a byte stream whose
/// canonical form has identical SHA-256 to the input's canonical
/// form.
///
/// Per the module-level doc-comment, this is the runtime witness of
/// the equivalence-of-categories status of the `parse ⊣ reemit`
/// adjunction.
pub trait FractalRoundTrip {
    /// The parsed-source type returned by [`parse`](Self::parse).
    type Source;
    /// Error type for parse / reemit / canonicalize.
    type Error: fmt::Display;

    /// Parse a byte stream into the ontology instance.
    fn parse(bytes: &[u8]) -> Result<Self::Source, Self::Error>;

    /// Re-emit the parsed source as bytes.
    fn reemit(source: &Self::Source) -> Result<Vec<u8>, Self::Error>;

    /// Canonicalize the input bytes per the source kind's published
    /// canonical-form specification. Two byte streams are taken to
    /// represent the same source iff their canonical forms are
    /// byte-identical.
    fn canonical(bytes: &[u8]) -> Result<Vec<u8>, Self::Error>;

    /// Parse + re-emit chained. Default implementation; impls may
    /// override only when there is a more efficient direct route
    /// that still goes through the ontology.
    fn round_trip(bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        let parsed = Self::parse(bytes)?;
        Self::reemit(&parsed)
    }

    /// SHA-256 of the canonical form. The *signature*.
    fn signature(bytes: &[u8]) -> Result<[u8; 32], Self::Error> {
        let c = Self::canonical(bytes)?;
        let mut h = Sha256::new();
        h.update(&c);
        Ok(h.finalize().into())
    }

    /// The round-trip assertion: `sig(input) == sig(round_trip(input))`.
    ///
    /// Used by the M4.θ.2 fractal round-trip test harness to
    /// verify each registered source kind.
    fn assert_round_trip(bytes: &[u8]) -> Result<(), FractalRoundTripFailure> {
        let input_sig = Self::signature(bytes).map_err(|e| FractalRoundTripFailure {
            stage: FailureStage::CanonicalizeInput,
            message: alloc::format!("{}", e),
            input_digest: None,
            roundtrip_digest: None,
        })?;
        let round_tripped = Self::round_trip(bytes).map_err(|e| FractalRoundTripFailure {
            stage: FailureStage::RoundTrip,
            message: alloc::format!("{}", e),
            input_digest: Some(input_sig),
            roundtrip_digest: None,
        })?;
        let rt_sig = Self::signature(&round_tripped).map_err(|e| FractalRoundTripFailure {
            stage: FailureStage::CanonicalizeRoundTrip,
            message: alloc::format!("{}", e),
            input_digest: Some(input_sig),
            roundtrip_digest: None,
        })?;
        if input_sig == rt_sig {
            Ok(())
        } else {
            Err(FractalRoundTripFailure {
                stage: FailureStage::DigestMismatch,
                message: String::from(
                    "canonical-form SHA-256 of input != canonical-form SHA-256 of round-trip; \
                     ontology does not yet capture the full structure of the source",
                ),
                input_digest: Some(input_sig),
                roundtrip_digest: Some(rt_sig),
            })
        }
    }
}

/// Structured failure from [`FractalRoundTrip::assert_round_trip`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FractalRoundTripFailure {
    /// Which stage of the assertion failed.
    pub stage: FailureStage,
    /// Human-readable description of the failure.
    pub message: String,
    /// SHA-256 of the canonical form of the input bytes (when
    /// canonicalization of the input succeeded).
    pub input_digest: Option<[u8; 32]>,
    /// SHA-256 of the canonical form of the round-tripped bytes
    /// (only set on `DigestMismatch`).
    pub roundtrip_digest: Option<[u8; 32]>,
}

impl fmt::Display for FractalRoundTripFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "FractalRoundTrip failure ({:?}): {}",
            self.stage, self.message
        )?;
        if let Some(d) = self.input_digest {
            write!(f, "; input_sig=")?;
            for b in d {
                write!(f, "{:02x}", b)?;
            }
        }
        if let Some(d) = self.roundtrip_digest {
            write!(f, "; roundtrip_sig=")?;
            for b in d {
                write!(f, "{:02x}", b)?;
            }
        }
        Ok(())
    }
}

/// Which step of `assert_round_trip` failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureStage {
    /// Canonicalizing the input bytes failed.
    CanonicalizeInput,
    /// Parse-then-re-emit failed (parse or reemit).
    RoundTrip,
    /// Canonicalizing the round-tripped bytes failed.
    CanonicalizeRoundTrip,
    /// Round-trip completed but the two canonical digests differ —
    /// the ontology does not yet capture the full structure of the
    /// source.
    DigestMismatch,
}
