//! The uniform **decompile** op — STAGE 1 of the universal compiler's
//! `.prx → source` leg (the lens *put*).
//!
//! # What this module is
//!
//! `pr4xis compile` turns a registered source (OWL RDF/XML, USLM XML, or
//! WN-LMF XML) into a content-addressed `.prx.gz` archive. This module is the
//! single inverse operation: given the `.prx.gz` bytes of any of those source
//! kinds, [`decompile`] regenerates the original source bytes AND reports the
//! [`RoundTripFidelity`] tier the regeneration was achieved at.
//!
//! It is a thin, honest *router*, not a new reconstructor: it dispatches to the
//! per-leaf reconstruct already proved byte-exact at the floor —
//! [`owl::prx::reconstruct_source`], [`uslm::corpus::prx::usc_reconstruct_source`],
//! and [`lmf::prx::wn_reconstruct_source`] — and surfaces the
//! [`RoundTripFidelity`] those leaves carry in their envelope's `mode` field.
//!
//! # The fidelity it reports — and what it is NOT (yet)
//!
//! Every source compiled today rides the universal FLOOR,
//! [`RoundTripFidelity::RawBytesComplementFloor`]: the `.prx.gz` stores the
//! exact source bytes as a content-addressed *constant complement* (Bancilhon &
//! Spyratos 1981), and the reconstruct leaf returns them only after a
//! `sha256(blob) == content_address == metadata.source_sha256` honesty gate
//! (NIST FIPS 180-4 §6.2; Dolstra 2006 content-addressing). The byte-exactness
//! is therefore real and cryptographically witnessed — but it is *floor*, not
//! *graph-faithful*: the bytes come from a stored side-channel, not from
//! re-emitting the typed ontology graph alone.
//!
//! [`RoundTripFidelity::ByteExactGraphFaithful`] — reconstructing the source
//! from the graph with no stored complement (`write_owl` + RDFC #258 for OWL,
//! `write_uslm` for USC, `write_wordnet` for WordNet) — is STAGE 2. This op is
//! the single entry point STAGE 2 upgrades per source: when a leaf's
//! reconstruct learns to regenerate from `data`, [`decompile`] reports
//! `ByteExactGraphFaithful` for it with no caller change.
//!
//! # Routing — registry-aware, not byte-sniffing
//!
//! The three `.prx` envelope shapes are distinct rkyv layouts with no shared
//! peekable header, so [`decompile`] takes the source *kind* explicitly
//! ([`DecompileKind`]) — exactly the dispatch dimension the registry, the load
//! gate, and the emitters already use. A caller that holds a registered source
//! resolves its [`crate::applied::data_provisioning::ontology::ContentType`]
//! (via the [`RegistryEntry`](crate::applied::data_provisioning::ontology::RegistryEntry))
//! and maps it with [`DecompileKind::from_content_type`].
//!
//! # Citations
//!
//! - **Foster, Greenwald, Moore, Pierce & Schmitt (2007)** — "Combinators for
//!   Bidirectional Tree Transformations", *ACM TOPLAS* 29(3) §2.2 (the lens
//!   *put* leg this op realises).
//! - **Bancilhon & Spyratos (1981)** — "Update semantics of relational views",
//!   *ACM TODS* 6(4) (the constant-complement the FLOOR stores).
//! - **NIST FIPS 180-4 (2015)** §6.2 (SHA-256) + **Dolstra (2006)** *Purely
//!   Functional Software Deployment* §3 (content-addressing the honesty gate
//!   re-verifies).

use alloc::vec::Vec;
use core::fmt;

use super::lens_trait::RoundTripFidelity;
use crate::applied::data_provisioning::ontology::ContentType;
use crate::social::software::markup::xml::{lmf, owl, uslm};

/// The source kinds the universal compiler can decompile today — the closed set
/// of `.prx` leaves with a registered reconstruct.
///
/// A *rich* type, not a `ContentType` re-used directly: `ContentType` ranges
/// over every byte format praxis decodes (PDF, ZIP, glyph lists, …), most of
/// which have no `.prx` consumer. `DecompileKind` is exactly the three that do,
/// so [`decompile`] is total over it — no "unsupported kind" runtime arm hiding
/// inside the router. New leaves extend this enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecompileKind {
    /// W3C OWL 2 / RDF-XML vocabulary. Reconstruct:
    /// [`owl::prx::reconstruct_source`]. Graph-faithful gap: `write_owl_exact`
    /// + RDF Dataset Canonicalization (#258).
    Owl,
    /// United States Legislative Markup XML (a U.S. Code title). Reconstruct:
    /// [`uslm::corpus::prx::usc_reconstruct_source`]. Graph-faithful gap:
    /// `write_uslm`.
    UsCode,
    /// WN-LMF XML (Open English WordNet). Reconstruct:
    /// [`lmf::prx::wn_reconstruct_source`]. Graph-faithful gap: `write_wordnet`.
    WordNet,
}

impl DecompileKind {
    /// Map a registry [`ContentType`] to the decompile leaf that handles it, or
    /// `None` for a content type with no `.prx` consumer (PDF, ZIP, plaintext,
    /// …). The single place the `ContentType → DecompileKind` correspondence
    /// lives, so the CLI and the all-sources test agree by construction.
    #[must_use]
    pub fn from_content_type(ct: ContentType) -> Option<Self> {
        match ct {
            ContentType::Owl => Some(DecompileKind::Owl),
            ContentType::UslmXml => Some(DecompileKind::UsCode),
            ContentType::XmlLmf => Some(DecompileKind::WordNet),
            _ => None,
        }
    }

    /// The work remaining to lift this leaf from
    /// [`RoundTripFidelity::RawBytesComplementFloor`] (stored complement) to
    /// [`RoundTripFidelity::ByteExactGraphFaithful`] (regenerate from the graph
    /// alone) — named per source so the [`super::completeness`] meter can state
    /// the gap honestly. Returns the byte-exact writer + canonicalization that
    /// STAGE 2 must build.
    #[must_use]
    pub fn graph_faithful_gap(self) -> &'static str {
        match self {
            DecompileKind::Owl => "write_owl_exact + RDFC #258",
            DecompileKind::UsCode => "write_uslm",
            DecompileKind::WordNet => "write_wordnet",
        }
    }
}

/// Failure from [`decompile`]. Wraps the per-leaf
/// [`PrxError`](owl::prx::PrxError) (gunzip / rkyv-validate / honesty-gate
/// failure) and tags which leaf raised it, so a caller knows *which* source
/// kind's `.prx` was malformed without re-deriving it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecompileError {
    /// The leaf that was dispatched to.
    pub kind: DecompileKind,
    /// The underlying `.prx` load/reconstruct failure (gzip, rkyv bytecheck,
    /// or the `sha256` honesty gate refusing a tampered complement).
    pub source: owl::prx::PrxError,
}

impl fmt::Display for DecompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "decompile {:?}: {}", self.kind, self.source)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DecompileError {}

/// **The uniform decompile op** — the single `.prx.gz → source` entry point of
/// the universal compiler.
///
/// Routes `prx_gz` to the reconstruct leaf for `kind`, returning BOTH the
/// regenerated source bytes AND the [`RoundTripFidelity`] tier the
/// reconstruction was achieved at (read off the envelope's `mode`, never
/// asserted by this op). Today every source returns its bytes at
/// [`RoundTripFidelity::RawBytesComplementFloor`] — real, sha256-witnessed
/// byte-exactness from the stored constant complement; the tier makes clear it
/// is floor-not-graph-faithful (see the module doc).
///
/// Each leaf:
/// 1. `gunzip` the `.prx.gz` (RFC 1952),
/// 2. `*_envelope_from_bytes` — bytecheck-validated rkyv decode (a corrupt blob
///    fails closed),
/// 3. `*_reconstruct_source` — returns the stored complement only after the
///    `sha256(blob) == content_address == metadata.source_sha256` honesty gate.
///
/// # Errors
///
/// [`DecompileError`] if the gzip layer, the rkyv bytecheck, or the honesty
/// gate rejects the archive.
pub fn decompile(
    prx_gz: &[u8],
    kind: DecompileKind,
) -> Result<(Vec<u8>, RoundTripFidelity), DecompileError> {
    let err = |source| DecompileError { kind, source };
    match kind {
        DecompileKind::Owl => {
            let rkyv_bytes = owl::prx::gunzip(prx_gz).map_err(err)?;
            let envelope = owl::prx::envelope_from_bytes(&rkyv_bytes).map_err(err)?;
            let source = owl::prx::reconstruct_source(&envelope).map_err(err)?;
            Ok((source, envelope.mode))
        }
        DecompileKind::UsCode => {
            let rkyv_bytes = owl::prx::gunzip(prx_gz).map_err(err)?;
            let envelope = uslm::corpus::prx::usc_envelope_from_bytes(&rkyv_bytes).map_err(err)?;
            let source = uslm::corpus::prx::usc_reconstruct_source(&envelope).map_err(err)?;
            Ok((source, envelope.mode))
        }
        DecompileKind::WordNet => {
            let rkyv_bytes = owl::prx::gunzip(prx_gz).map_err(err)?;
            let envelope = lmf::prx::wordnet_envelope_from_bytes(&rkyv_bytes).map_err(err)?;
            let source = lmf::prx::wn_reconstruct_source(&envelope).map_err(err)?;
            Ok((source, envelope.mode))
        }
    }
}
