//! The `WellBehavedLens` trait — signature-of-understanding for
//! loaded sources, grounded in the bidirectional-transformation
//! literature.
//!
//! ## Lens laws
//!
//! Foster, Greenwald, Moore, Pierce & Schmitt 2007 §2.2 define a
//! *well-behaved lens* as a pair of total functions
//!
//!   `get : S → T`
//!   `put : T → S`
//!
//! between a source `S` and a target `T`, satisfying three laws:
//!
//!   - **GetPut:** `get(put(t)) = t` for every `t ∈ T`.
//!   - **PutGet:** `put(get(s)) = s` for every `s ∈ S` (up to the
//!     equivalence used to compare sources).
//!   - **PutPut:** `put(t') = put(t')` after `put(t)` — repeated
//!     application is idempotent in source space.
//!
//! In Praxis the source `S` is the set of byte streams of a given
//! source kind (a USLM XML file, a WordNet LMF XML file, a JSON
//! document, a `praxis.toml` manifest, …); the target `T` is the
//! Praxis ontology that types `S` after parsing. The source-space
//! equivalence used by PutGet is *canonical-form equality*: two byte
//! streams represent the same source iff their canonical forms are
//! byte-identical.
//!
//! ## Categorical statement
//!
//! Foster et al.'s lenses are equivalent to an *adjunction*
//! `get ⊣ put` (Mac Lane *Categories for the Working Mathematician*,
//! 2nd ed., §IV.1). The minimum guarantee an adjunction gives is
//! *natural* unit/counit — not that they are identities. When the
//! adjunction reaches the stronger *equivalence-of-categories* status
//! (Mac Lane §IV.4 Theorem 1), unit and counit compose to identity
//! up to natural isomorphism — the PutGet law in lens vocabulary.
//!
//! `WellBehavedLens` is the runtime witness that the adjunction
//! between a specific `S` and its `O` reaches equivalence-of-
//! categories status. For every byte stream `b ∈ S` it asserts the
//! PutGet law:
//!
//!   sig(b) == sig(put(get(b)))
//!
//! where `sig = SHA-256 ∘ canonical` and `canonical` is the
//! source kind's published canonical form (W3C XML C14N 1.1 for
//! XML, RFC 8785 JCS for JSON, Unicode TR #15 NFKC for plain text,
//! W3C REC-rdf-canon-20240521 for RDF, etc.). A hash mismatch is concrete evidence
//! that some structural detail of `b` is not yet reflected in `O`
//! — an ontology gap.
//!
//! ## What this module ships
//!
//! - The [`WellBehavedLens`] trait itself (this file).
//! - The [`LensLawFailure`] error type that carries the input and
//!   round-tripped digests + an optional structured diff.
//! - The [`crate::formal::meta::well_behaved_lens::canonical`]
//!   library of per-source canonical-form implementations.
//!
//! Running the PutGet check against every loaded source is the
//! responsibility of [`super::harness`], which iterates the
//! `linkme`-distributed slice of registered lenses, calls
//! [`WellBehavedLens::assert_put_get_law`] on each, and verifies the
//! resulting signature against `praxis.lock`'s
//! `[canonical_signatures]` section.
//!
//! ## Citations
//!
//! - **Foster, J. N.; Greenwald, M. B.; Moore, J. T.; Pierce, B. C.;
//!   Schmitt, A. (2007)** — "Combinators for Bidirectional Tree
//!   Transformations: A Linguistic Approach to the View Update
//!   Problem", *ACM Transactions on Programming Languages and
//!   Systems* 29(3) Article 17, §2.2 (well-behaved-lens laws).
//! - **Mac Lane (1998)** — *Categories for the Working
//!   Mathematician*, Springer GTM 5, 2nd ed., §IV.1 + §IV.4.

use alloc::{string::String, vec::Vec};
use core::fmt;

use sha2::{Digest, Sha256};

/// The signature-of-understanding trait — Foster et al. 2007
/// well-behaved lens specialized to byte-stream sources.
///
/// A source kind implementing `WellBehavedLens` declares that Praxis
/// can `get` (parse) a byte stream of that kind, reconstruct it from
/// the parsed ontology instance via `put` (re-emit), and produce a
/// byte stream whose canonical form has identical SHA-256 to the
/// input's canonical form.
///
/// Per the module-level doc-comment, this is the runtime witness of
/// the PutGet law for the `get ⊣ put` adjunction.
pub trait WellBehavedLens {
    /// The ontology-target type returned by [`get`](Self::get).
    type Target;
    /// Error type for `get` / `put` / `canonical`.
    type Error: fmt::Display;

    /// Parse a byte stream into the ontology instance (Foster et al.
    /// 2007 §2.2, `get : S → T`).
    fn get(bytes: &[u8]) -> Result<Self::Target, Self::Error>;

    /// Re-emit the ontology instance as bytes (Foster et al. 2007
    /// §2.2, `put : T → S`).
    fn put(target: &Self::Target) -> Result<Vec<u8>, Self::Error>;

    /// Canonicalize the input bytes per the source kind's published
    /// canonical-form specification. Two byte streams are taken to
    /// represent the same source iff their canonical forms are
    /// byte-identical.
    fn canonical(bytes: &[u8]) -> Result<Vec<u8>, Self::Error>;

    /// `get` followed by `put` — apply the lens round-trip. Default
    /// implementation; impls may override only when there is a more
    /// efficient direct route that still goes through the ontology.
    fn apply_put_after_get(bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        let parsed = Self::get(bytes)?;
        Self::put(&parsed)
    }

    /// SHA-256 of the canonical form. The *signature*.
    fn signature(bytes: &[u8]) -> Result<[u8; 32], Self::Error> {
        let c = Self::canonical(bytes)?;
        let mut h = Sha256::new();
        h.update(&c);
        Ok(h.finalize().into())
    }

    /// Run the PutGet law (Foster et al. 2007 §2.2):
    /// `canonical(put(get(s))) == canonical(s)`.
    ///
    /// Used by the M4.θ.2 round-trip test harness to verify each
    /// registered source kind.
    fn assert_put_get_law(bytes: &[u8]) -> Result<(), LensLawFailure> {
        let input_sig = Self::signature(bytes).map_err(|e| LensLawFailure {
            stage: FailureStage::CanonicalizeInput,
            message: alloc::format!("{}", e),
            input_digest: None,
            roundtrip_digest: None,
        })?;
        let round_tripped = Self::apply_put_after_get(bytes).map_err(|e| LensLawFailure {
            stage: FailureStage::PutAfterGet,
            message: alloc::format!("{}", e),
            input_digest: Some(input_sig),
            roundtrip_digest: None,
        })?;
        let rt_sig = Self::signature(&round_tripped).map_err(|e| LensLawFailure {
            stage: FailureStage::CanonicalizeRoundTrip,
            message: alloc::format!("{}", e),
            input_digest: Some(input_sig),
            roundtrip_digest: None,
        })?;
        if input_sig == rt_sig {
            Ok(())
        } else {
            Err(LensLawFailure {
                stage: FailureStage::DigestMismatch,
                message: String::from(
                    "canonical-form SHA-256 of input != canonical-form SHA-256 of put(get(input)); \
                     ontology does not yet capture the full structure of the source (PutGet law violated)",
                ),
                input_digest: Some(input_sig),
                roundtrip_digest: Some(rt_sig),
            })
        }
    }
}

/// Structured failure from [`WellBehavedLens::assert_put_get_law`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LensLawFailure {
    /// Which stage of the PutGet-law assertion failed.
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

impl fmt::Display for LensLawFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "WellBehavedLens PutGet-law failure ({:?}): {}",
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

/// Which step of `assert_put_get_law` failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureStage {
    /// Canonicalizing the input bytes failed.
    CanonicalizeInput,
    /// `put(get(_))` failed (either `get` or `put`).
    PutAfterGet,
    /// Canonicalizing the round-tripped bytes failed.
    CanonicalizeRoundTrip,
    /// Round-trip completed but the two canonical digests differ —
    /// the ontology does not yet capture the full structure of the
    /// source (PutGet law violated).
    DigestMismatch,
}
