//! The `WellBehavedLens` trait — signature-of-understanding for
//! loaded sources, grounded in the bidirectional-transformation
//! literature.
//!
//! ## Lens laws
//!
//! Foster, Greenwald, Moore, Pierce & Schmitt 2007 §3, Definition 3.2 define a
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
//! where `sig = address ∘ canonical` and `canonical` is the
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
//!   Systems* 29(3) Article 17, §3, Definition 3.2 (well-behaved-lens laws).
//! - **Mac Lane (1998)** — *Categories for the Working
//!   Mathematician*, Springer GTM 5, 2nd ed., §IV.1 + §IV.4.

use alloc::{string::String, vec::Vec};
use core::fmt;

use pr4xis_runtime::address::{ContentAddress, HashAlgorithm, hash_hex};

/// The round-trip fidelity a [`WellBehavedLens`] guarantees — which
/// PutGet law the harness holds it to (M4.ι / #186).
///
/// A *rich* type, not a bool: byte-exactness and the raw-bytes floor
/// are genuinely different guarantees with different witnesses, and
/// further fidelity levels extend this enum rather than adding flags.
///
/// rkyv-serializable so a content-addressed archive (`.prx`) can record
/// the source lens's fidelity directly — the archive's reconstruction
/// mode IS this grade, not a parallel enum. The rkyv derives are gated on
/// `feature = "prx"` (where `rkyv` is linked and the archive that consumes
/// them exists); without `prx` the grade is still a first-class value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum RoundTripFidelity {
    /// `put(get(b)) == b` byte-for-byte, reconstructed from the typed
    /// ontology graph alone with NO constant-complement side-channel.
    /// The target for every source with a graph-faithful concrete
    /// syntax (plain text, XML/USLM/XSD/DTD, OWL). Held to
    /// [`WellBehavedLens::assert_byte_exact_law`].
    ByteExactGraphFaithful,
    /// Byte-identity recovered via a stored raw-bytes complement
    /// (Bancilhon & Spyratos 1981, constant-complement view update),
    /// the universal FLOOR for sources with no published canonical
    /// form (PDF/binary — ISO 32000-2 publishes none). Held to the
    /// canonical [`WellBehavedLens::assert_put_get_law`].
    RawBytesComplementFloor,
}

/// The signature-of-understanding trait — Foster et al. 2007
/// well-behaved lens specialized to byte-stream sources.
///
/// A source kind implementing `WellBehavedLens` declares that Praxis
/// can `get` (parse) a byte stream of that kind, reconstruct it from
/// the parsed ontology instance via `put` (re-emit), and produce a
/// byte stream whose canonical form has an identical content digest to the
/// input's canonical form.
///
/// Per the module-level doc-comment, this is the runtime witness of
/// the PutGet law for the `get ⊣ put` adjunction.
pub trait WellBehavedLens {
    /// The ontology-target type returned by [`get`](Self::get).
    type Target;
    /// Error type for `get` / `put` / `canonical`.
    type Error: fmt::Display;

    /// The round-trip fidelity this lens guarantees. Defaults to
    /// [`RoundTripFidelity::RawBytesComplementFloor`] so existing
    /// lenses are unaffected until migrated to graph-faithful
    /// byte-exactness (M4.ι / #186); the byte-exact migration sets
    /// this to [`RoundTripFidelity::ByteExactGraphFaithful`].
    const FIDELITY: RoundTripFidelity = RoundTripFidelity::RawBytesComplementFloor;

    /// Parse a byte stream into the ontology instance (Foster et al.
    /// 2007 §3, Definition 3.2, `get : S → T`).
    fn get(bytes: &[u8]) -> Result<Self::Target, Self::Error>;

    /// Re-emit the ontology instance as bytes (Foster et al. 2007
    /// §3, Definition 3.2, `put : T → S`).
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

    /// Content address of the canonical form. The *signature*.
    fn signature(bytes: &[u8]) -> Result<[u8; 32], Self::Error> {
        let c = Self::canonical(bytes)?;
        Ok(*ContentAddress::of(&c).as_bytes())
    }

    /// Lowercase-hex digest of the canonical form under a NAMED
    /// [`HashAlgorithm`] — the verify-many leg of the canonical signature.
    /// The harness dispatches this under the algorithm the `praxis.lock`
    /// pin names (one verify leg, [`hash_hex`]); [`Self::signature`] is the
    /// emit leg (always [`pr4xis_runtime::address::ADDRESS_ALGORITHM`]).
    fn signature_hex(bytes: &[u8], algorithm: HashAlgorithm) -> Result<String, Self::Error> {
        let c = Self::canonical(bytes)?;
        Ok(hash_hex(algorithm, &c))
    }

    /// Run the PutGet law (Foster et al. 2007 §3, Definition 3.2):
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
                    "canonical-form digest of input != canonical-form digest of put(get(input)); \
                     ontology does not yet capture the full structure of the source (PutGet law violated)",
                ),
                input_digest: Some(input_sig),
                roundtrip_digest: Some(rt_sig),
            })
        }
    }

    /// Run the strict **byte-exact** PutGet law: `put(get(b)) == b` as
    /// raw byte vectors. This is the equivalence-of-categories counit
    /// at *byte identity* (Mac Lane §IV.4 Theorem 1), strictly
    /// stronger than [`assert_put_get_law`](Self::assert_put_get_law),
    /// whose counit is only canonical-form identity.
    ///
    /// It requires the ontology graph to reconstruct the original
    /// bytes exactly — no canonicalization, no constant-complement
    /// side-channel. A mismatch is concrete evidence that a
    /// byte-affecting *concrete-syntax* decision is not yet a
    /// first-class node in the ontology (M4.ι / #186).
    ///
    /// Only lenses declaring
    /// [`RoundTripFidelity::ByteExactGraphFaithful`] are held to this
    /// law; the harness routes
    /// [`RoundTripFidelity::RawBytesComplementFloor`] sources to
    /// [`assert_put_get_law`](Self::assert_put_get_law) instead.
    fn assert_byte_exact_law(bytes: &[u8]) -> Result<(), LensLawFailure> {
        let round_tripped = Self::apply_put_after_get(bytes).map_err(|e| LensLawFailure {
            stage: FailureStage::PutAfterGet,
            message: alloc::format!("{}", e),
            input_digest: Some(raw_digest(bytes)),
            roundtrip_digest: None,
        })?;
        if round_tripped.as_slice() == bytes {
            Ok(())
        } else {
            Err(LensLawFailure {
                stage: FailureStage::ByteMismatch,
                message: String::from(
                    "put(get(input)) != input byte-for-byte; the ontology graph does not \
                     reconstruct the original bytes (byte-exact PutGet law violated — a \
                     concrete-syntax decision is not captured)",
                ),
                input_digest: Some(raw_digest(bytes)),
                roundtrip_digest: Some(raw_digest(&round_tripped)),
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
    /// Content digest of the input bytes — the *canonical form* under the
    /// PutGet law ([`FailureStage::DigestMismatch`]), or the *raw
    /// bytes* under the byte-exact law ([`FailureStage::ByteMismatch`]).
    pub input_digest: Option<[u8; 32]>,
    /// Content digest of the round-tripped bytes — canonical form
    /// (`DigestMismatch`) or raw bytes (`ByteMismatch`). Set only when
    /// the round-trip produced output to compare.
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
    /// Byte-exact round-trip completed but `put(get(b)) != b`
    /// byte-for-byte — a concrete-syntax decision is not yet a
    /// first-class node in the ontology (byte-exact PutGet law
    /// violated, M4.ι / #186).
    ByteMismatch,
}

/// Content digest of raw bytes (no canonicalization). Fingerprints inputs and
/// outputs in [`WellBehavedLens::assert_byte_exact_law`] failures,
/// where the comparison is on the original byte stream rather than a
/// canonical form.
fn raw_digest(bytes: &[u8]) -> [u8; 32] {
    *ContentAddress::of(bytes).as_bytes()
}
