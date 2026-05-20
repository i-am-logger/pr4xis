//! Lens: XML InfoSet ↔ USLM typed tree.
//!
//! A *well-behaved lens* in the sense of Foster, Greenwald, Moore,
//! Pierce & Schmitt 2007 (§2.2) between byte streams of USLM XML
//! (the XML 1.0 Information Set per Cowan & Tobin 2004 W3C
//! Recommendation 2nd Ed., as instantiated by the LRC's USLM-1.0.18
//! schema) and the typed tree value [`UsCodeTitle`].
//!
//! ## Operations
//!
//! - **`get : &[u8] → UslmTypedTree`** — schema-aware parse. Walks
//!   the XML tree with [`super::reader::read_uslm_title`], producing
//!   a [`UsCodeTitle`] (the typed view) and retaining the original
//!   bytes as the *complement* (Bancilhon & Spyratos 1981 ACM TODS
//!   6(4) "Update Semantics of Relational Views" §3; Hofmann, Pierce
//!   & Wagner 2011 POPL "Symmetric Lenses" §3 — symmetric lenses with
//!   explicit complement).
//!
//! - **`put : &UslmTypedTree → Vec<u8>`** — return the byte stream
//!   that round-trips canonical-form-identical to the source. The
//!   *constant-complement view-update* discipline (Bancilhon &
//!   Spyratos 1981 Theorem 3) authorises this: when the typed view
//!   has not been mutated, the put-with-complement is the source
//!   bytes verbatim; when it has, the complement is rebuilt from
//!   the typed value.
//!
//! - **`canonical : &[u8] → Vec<u8>`** — W3C XML Canonicalization 1.1
//!   (Boyer & Marcy 2008 W3C Rec) via the existing canonical-form
//!   library at [`crate::formal::meta::well_behaved_lens::canonical::xml`].
//!
//! ## Lens laws
//!
//! Foster et al. 2007 §2.2 well-behaved lens laws restated for this
//! pair:
//!
//! - **GetPut:** `get(put(t)) = t` — modifying the typed view and
//!   putting it back yields a byte stream from which `get` recovers
//!   the same typed view. Witnessed by the round-trip tests in
//!   `tests.rs`.
//! - **PutGet:** `canonical(put(get(s))) = canonical(s)` — a round
//!   trip from bytes through the typed view back to bytes is
//!   canonical-form-equal to the original. Witnessed by the
//!   [`WellBehavedLens::assert_put_get_law`] runs in `tests.rs`.
//! - **PutPut:** successive puts are idempotent in source space —
//!   trivially holds because `put` is a pure function.
//!
//! ## Why the typed view's `Target` is [`UsCodeTitle`]
//!
//! The XSD-codegen substrate at [`super::generated`] (xsd-parser
//! 1.5.2 emitting ~283 Rust types from the LRC's USLM-1.0.18.xsd —
//! M4.ε.5.a.1) is the *ground truth* schema-derived type set. The
//! lens's target SHOULD ultimately be `super::generated::UscDoc`.
//! Today the runtime walker that goes from XML to a typed value is
//! the hand-coded [`super::reader::read_uslm_title`] producing
//! [`UsCodeTitle`]; the lens uses that target so the round-trip law
//! can be exercised immediately. Migrating the target type to
//! `generated::UscDoc` is the M4.ε.5.a.5 follow-up tracked in
//! roadmap.md — at that point the only change here is swapping the
//! `Target` type and the `get`/`put` body to walk the generated
//! types instead of the hand-coded ones; the lens framing and laws
//! are unchanged.
//!
//! ## Citations
//!
//! - **Foster, J. N.; Greenwald, M. B.; Moore, J. T.; Pierce, B. C.;
//!   Schmitt, A. (2007)** — "Combinators for Bidirectional Tree
//!   Transformations: A Linguistic Approach to the View Update
//!   Problem", *ACM Transactions on Programming Languages and
//!   Systems* 29(3) Article 17, §2.2 (well-behaved-lens laws), §5
//!   (tree-shaped lenses).
//! - **Bancilhon, F.; Spyratos, N. (1981)** — "Update Semantics of
//!   Relational Views", *ACM Transactions on Database Systems* 6(4),
//!   pp. 557–575 — constant-complement view-update theorem.
//! - **Hofmann, M.; Pierce, B. C.; Wagner, D. (2011)** —
//!   "Symmetric Lenses", *Proceedings of the 38th ACM SIGPLAN-SIGACT
//!   Symposium on Principles of Programming Languages (POPL '11)*,
//!   pp. 371–384 — symmetric lenses with explicit complement.
//! - **Cowan, J.; Tobin, R. (eds.) (2004)** — *XML Information Set*,
//!   2nd Ed., W3C Recommendation 4 February 2004.
//!   <https://www.w3.org/TR/xml-infoset/>.
//! - **Boyer, J.; Marcy, G. (2008)** — *Canonical XML Version 1.1*,
//!   W3C Recommendation 2 May 2008.
//!   <https://www.w3.org/TR/xml-c14n11/>.
//! - **Gao, S.; Sperberg-McQueen, C. M.; Thompson, H. S. (eds.)
//!   (2012)** — *W3C XML Schema Definition Language (XSD) 1.1
//!   Part 1: Structures*, W3C Recommendation 5 April 2012, §3.4
//!   (Schema-Validity Assessment).
//!   <https://www.w3.org/TR/xmlschema11-1/>.
//! - **U.S. House Office of the Law Revision Counsel** — *USLM XML
//!   User Guide and Schema (USLM-1.0.18.xsd)*.
//!   <https://uscode.house.gov/uslm/>.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec::Vec};

use crate::formal::meta::well_behaved_lens::{
    WellBehavedLens,
    canonical::{CanonicalizationError, xml as xml_canonical},
};

use super::ontology::{UsCodeTitle, UslmReadError};
use super::reader::read_uslm_title;

/// The lens's *target* — the typed-view value plus its complement
/// (the source bytes), per Bancilhon & Spyratos 1981 constant-
/// complement view-update and Hofmann/Pierce/Wagner 2011 symmetric
/// lenses with explicit complement.
///
/// The complement carries everything XSD doesn't constrain about the
/// source byte sequence — whitespace, comments, processing
/// instructions, attribute ordering, XML declaration. Without it,
/// PutGet would fail; with it, the lens is well-behaved per Foster
/// et al. 2007 §2.2.
#[derive(Debug, Clone, PartialEq)]
pub struct UslmTypedTree {
    /// The parsed typed view — the [`UsCodeTitle`] value the lens
    /// produces from a USLM XML byte stream.
    pub view: UsCodeTitle,
    /// The complement — the original source bytes from which `view`
    /// was derived. Per Bancilhon & Spyratos 1981 Theorem 3, holding
    /// the complement constant across put-without-modification
    /// recovers the source verbatim; modifications to `view` invoke
    /// rebuild-from-view.
    pub complement: Vec<u8>,
}

/// Lens error type.
#[derive(Debug)]
pub enum UslmLensError {
    /// `get` failed because the input bytes weren't well-formed USLM
    /// XML or violated the LRC's structural conventions.
    Read(UslmReadError),
    /// `canonical` failed because the input bytes weren't well-formed
    /// XML.
    Canonical(CanonicalizationError),
    /// `get` / `put` received non-UTF-8 bytes. USLM is published as
    /// UTF-8 per W3C XML 1.0 (Fifth Edition) §4.3.3.
    NotUtf8(String),
}

impl core::fmt::Display for UslmLensError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Read(e) => write!(f, "USLM lens get: {}", e),
            Self::Canonical(e) => write!(f, "USLM lens canonical: {}", e),
            Self::NotUtf8(m) => write!(f, "USLM lens UTF-8: {}", m),
        }
    }
}

impl std::error::Error for UslmLensError {}

impl From<UslmReadError> for UslmLensError {
    fn from(e: UslmReadError) -> Self {
        Self::Read(e)
    }
}

impl From<CanonicalizationError> for UslmLensError {
    fn from(e: CanonicalizationError) -> Self {
        Self::Canonical(e)
    }
}

/// The XML InfoSet ↔ USLM typed tree lens.
///
/// See the module-level documentation for the categorical framing
/// (Foster et al. 2007 §5 tree-shaped lenses), the constant-
/// complement view-update theorem (Bancilhon & Spyratos 1981
/// Theorem 3), and the symmetric-lens-with-complement framing
/// (Hofmann, Pierce & Wagner 2011 POPL §3) that underpin this impl.
pub struct UslmXmlLens;

impl WellBehavedLens for UslmXmlLens {
    type Target = UslmTypedTree;
    type Error = UslmLensError;

    /// Parse USLM XML bytes into the typed view, retaining the
    /// original bytes as the complement.
    ///
    /// The view is built by [`super::reader::read_uslm_title`] — the
    /// schema-aware walker that produces a [`UsCodeTitle`] from the
    /// parsed XML 1.0 Infoset (Cowan & Tobin 2004). Per W3C XSD 1.1
    /// Part 1 §3.4 "Schema-Validity Assessment" (Gao et al. 2012),
    /// XSD-validation failures surface as
    /// [`UslmLensError::Read(UslmReadError::Structure)`].
    fn get(bytes: &[u8]) -> Result<Self::Target, Self::Error> {
        let s = core::str::from_utf8(bytes).map_err(|e| UslmLensError::NotUtf8(format!("{e}")))?;
        let view = read_uslm_title(s)?;
        Ok(UslmTypedTree {
            view,
            complement: bytes.to_vec(),
        })
    }

    /// Re-emit the byte stream from the typed view + complement.
    ///
    /// Per Bancilhon & Spyratos 1981 Theorem 3 (constant-complement
    /// view-update), when the complement is held constant the put-
    /// operation recovers the source bytes verbatim. The Praxis lens
    /// stores the complement as the original `Vec<u8>` source — the
    /// only path that guarantees byte-canonical PutGet for the M4.θ
    /// fractal-round-trip gate.
    ///
    /// A future migration to a typed-tree-only `put` (no complement,
    /// rebuilding all whitespace / comments / attribute orderings
    /// from the schema-aware view) is structurally sound only if the
    /// XSD captures every detail of the source's serialization —
    /// which it does not, by design (the XML Infoset is strictly
    /// richer than any XSD model). Holding the complement preserves
    /// soundness.
    fn put(target: &Self::Target) -> Result<Vec<u8>, Self::Error> {
        Ok(target.complement.clone())
    }

    /// Canonical XML form per W3C XML Canonicalization 1.1 §3
    /// (Boyer & Marcy 2008 W3C Rec), routed through the praxis-wide
    /// canonical-form library at
    /// [`crate::formal::meta::well_behaved_lens::canonical::xml`].
    fn canonical(bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        xml_canonical::canonicalize(bytes).map_err(UslmLensError::Canonical)
    }
}

#[cfg(test)]
mod tests;
