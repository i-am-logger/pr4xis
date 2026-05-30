//! `DtdLens` — the [`WellBehavedLens`] binding the praxis DTD parser's
//! byte hop into the round-trip harness. Parallel to
//! [`crate::formal::meta::xsd::lens::XsdSchemaLens`] for XSD.
//!
//! Per W3C XML 1.0 Fifth Edition §2.8 + §3 + §4, a DTD is a sequence
//! of markup declarations. This lens composes [`super::parser::parse_dtd`]
//! (the declarations scanner) with the byte stream to read raw DTD
//! bytes into a [`DtdSchema`] = `(declarations, complement)` pair.
//! Per Bancilhon & Spyratos 1981 Theorem 3 the complement carrying
//! the original bytes makes put-get hold byte-canonically — see
//! [`crate::formal::meta::xsd::lens::XsdSchemaLens`] for the same
//! shape applied to XSD.
//!
//! ## Citation
//!
//! - **Bray, Paoli, Sperberg-McQueen, Maler & Yergeau (2008)** *XML
//!   1.0 (Fifth Edition)*, W3C Recommendation 26 November 2008 — the
//!   syntactic substrate of DTDs.
//! - **Foster, Greenwald, Moore, Pierce & Schmitt (2007)** "Combinators
//!   for Bidirectional Tree Transformations", *ACM TOPLAS* 29(3) §2.2
//!   — the well-behaved-lens laws.
//! - **Bancilhon & Spyratos (1981)** "Update Semantics of Relational
//!   Views", *ACM TODS* 6(4) Theorem 3 — constant-complement view
//!   update.

#[allow(unused_imports)]
use alloc::{format, string::String, vec::Vec};
use core::fmt;

use super::parser::{DtdDecl, parse_dtd};
use crate::formal::meta::well_behaved_lens::WellBehavedLens;

/// The byte-anchored view of a DTD — the parsed declarations plus
/// the original bytes as the constant complement (Bancilhon &
/// Spyratos 1981 Theorem 3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DtdSchema {
    /// Every parsed declaration in document order — typed
    /// [`super::ontology::DtdConcept`] + name + body per
    /// [`DtdDecl`].
    pub declarations: Vec<DtdDecl>,
    /// The complement: the original source bytes from which
    /// `declarations` was derived. Per Bancilhon & Spyratos 1981
    /// Theorem 3, holding the complement constant recovers the
    /// source verbatim on put-without-modification.
    pub complement: Vec<u8>,
}

impl DtdSchema {
    /// All declarations.
    #[must_use]
    pub fn declarations(&self) -> &[DtdDecl] {
        &self.declarations
    }

    /// Iterate declarations of a particular concept kind.
    pub fn of_kind(&self, kind: super::ontology::DtdConcept) -> impl Iterator<Item = &DtdDecl> {
        self.declarations.iter().filter(move |d| d.kind == kind)
    }
}

/// Error of [`DtdLens::get`] / [`DtdLens::put`].
#[derive(Debug, Clone)]
pub enum DtdLensError {
    /// Input was not valid UTF-8.
    NotUtf8(String),
    /// Canonicalization failed (none currently — DTDs use their
    /// source bytes verbatim).
    Canonical(String),
}

impl fmt::Display for DtdLensError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DtdLensError::NotUtf8(m) => write!(f, "dtd lens: not valid UTF-8: {m}"),
            DtdLensError::Canonical(m) => write!(f, "dtd lens: canonicalization failed: {m}"),
        }
    }
}

/// The byte-anchored [`WellBehavedLens`] binding `bytes ⇆ DtdSchema`.
///
/// `get(bytes)` runs [`parse_dtd`] over the bytes, retaining the
/// originals as the complement. `put(target)` returns the
/// complement — constant-complement PutGet (Bancilhon & Spyratos
/// 1981 Theorem 3). `canonical` returns the bytes unchanged: DTDs
/// don't have a published canonicalisation form (W3C C14N is XML-
/// specific, not DTD), so the source bytes ARE their own canonical
/// form for round-trip purposes.
pub struct DtdLens;

impl WellBehavedLens for DtdLens {
    type Target = DtdSchema;
    type Error = DtdLensError;

    fn get(bytes: &[u8]) -> Result<Self::Target, Self::Error> {
        // Validate UTF-8 up front so callers see a structural error
        // rather than silently parsing zero declarations.
        core::str::from_utf8(bytes).map_err(|e| DtdLensError::NotUtf8(format!("{e}")))?;
        let declarations = parse_dtd(bytes);
        Ok(DtdSchema {
            declarations,
            complement: bytes.to_vec(),
        })
    }

    fn put(target: &Self::Target) -> Result<Vec<u8>, Self::Error> {
        Ok(target.complement.clone())
    }

    fn canonical(bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        // No W3C-published canonical form for DTDs (C14N is XML-only).
        // Round-trip identity is therefore byte-equality on the source
        // bytes themselves.
        Ok(bytes.to_vec())
    }
}

// =============================================================================
// Round-trip harness registration — the bundled WN-LMF 1.3 DTD is the
// first DTD entry in the praxis source registry.
// =============================================================================

crate::register_lens!(WN_LMF_DTD_LENS, "wn_lmf_dtd", "1.3", DtdLens);

#[cfg(test)]
mod tests {
    use super::super::ontology::DtdConcept;
    use super::*;

    #[test]
    fn get_then_put_returns_original_bytes() {
        let bytes = b"<!ELEMENT root EMPTY>".to_vec();
        let target = <DtdLens as WellBehavedLens>::get(&bytes).expect("parse");
        let back = <DtdLens as WellBehavedLens>::put(&target).expect("put");
        assert_eq!(back, bytes);
    }

    #[test]
    fn get_parses_element_declarations() {
        let bytes = b"<!ELEMENT root (#PCDATA)>\n<!ATTLIST root id ID #REQUIRED>".to_vec();
        let target = <DtdLens as WellBehavedLens>::get(&bytes).expect("parse");
        assert_eq!(target.declarations.len(), 2);
        assert_eq!(target.of_kind(DtdConcept::ElementDecl).count(), 1);
        assert_eq!(target.of_kind(DtdConcept::AttListDecl).count(), 1);
    }

    #[test]
    fn put_get_law_holds() {
        let bytes = b"<!ELEMENT a (b)+>\n<!ELEMENT b EMPTY>\n".to_vec();
        assert!(<DtdLens as WellBehavedLens>::assert_put_get_law(&bytes).is_ok());
    }

    #[test]
    fn parses_real_wn_lmf_dtd() {
        // The bundled WN-LMF 1.3 DTD round-trips through the lens.
        let bytes = crate::social::software::markup::xml::lmf::WN_LMF_1_3_DTD
            .as_bytes()
            .to_vec();
        let target = <DtdLens as WellBehavedLens>::get(&bytes).expect("parse WN-LMF");
        // Expect at least the canonical six elements + their AttLists.
        let elements: Vec<_> = target
            .of_kind(DtdConcept::ElementDecl)
            .map(|d| d.name.clone())
            .collect();
        assert!(elements.contains(&"LexicalResource".to_string()));
        assert!(elements.contains(&"Lexicon".to_string()));
        assert!(elements.contains(&"LexicalEntry".to_string()));
        assert!(elements.contains(&"Synset".to_string()));
        assert!(elements.contains(&"Sense".to_string()));
        assert!(target.of_kind(DtdConcept::AttListDecl).count() >= 5);
    }

    proptest::proptest! {
        /// Robustness: for arbitrary byte streams, `get` either
        /// returns a `DtdSchema` (UTF-8 valid + the parser scans
        /// recognised declarations) or a typed [`DtdLensError`].
        /// Never panics.
        #[test]
        fn prop_get_never_panics_on_arbitrary_bytes(
            bytes in proptest::collection::vec(proptest::prelude::any::<u8>(), 0..512)
        ) {
            let _ = <DtdLens as WellBehavedLens>::get(&bytes);
        }

        /// When `get` succeeds, `put` returns the source bytes
        /// byte-canonically (constant-complement). The invariant
        /// holds for every UTF-8 input — the DTD parser is a
        /// projector that drops unrecognised content silently;
        /// the complement preserves the source verbatim.
        #[test]
        fn prop_get_put_canonical_on_success(
            bytes in proptest::collection::vec(proptest::prelude::any::<u8>(), 0..512)
        ) {
            if let Ok(target) = <DtdLens as WellBehavedLens>::get(&bytes) {
                let back = <DtdLens as WellBehavedLens>::put(&target)
                    .expect("put always succeeds on a successful get");
                proptest::prop_assert_eq!(back, bytes);
            }
        }
    }
}
