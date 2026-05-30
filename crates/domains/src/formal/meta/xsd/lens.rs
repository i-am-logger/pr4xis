//! `XsdSchemaLens` — the [`WellBehavedLens`] binding the praxis XSD
//! reader's byte hop into the round-trip harness.
//!
//! Per W3C XSD 1.1 Part 1 §2.5 + §3.16 (Gao, Sperberg-McQueen &
//! Thompson 2012), an XSD schema document is an XML document whose
//! root is `<xs:schema>`. This lens composes
//! `crate::social::software::markup::xml::parser::grammar::parse_document`
//! (the XML 1.0 parser, Bray et al. 2008) with
//! [`super::from_xml::project_from_xml_document`] (the XSD projector)
//! to read raw schema bytes into an [`XsdSchema`] = `(instance,
//! complement)` pair. Per Bancilhon & Spyratos 1981 Theorem 3 the
//! complement carrying the original bytes makes put-get hold byte-
//! canonically — see [`UslmXmlLens`](crate::social::software::markup::xml::uslm::UslmXmlLens)
//! for the same shape applied to USLM.
//!
//! ## Citation
//!
//! - **Gao, Sperberg-McQueen & Thompson (2012)** *W3C XML Schema 1.1
//!   Part 1: Structures*, W3C Recommendation 5 April 2012 — §2.5 and
//!   §3.16 on schema documents.
//! - **Bray, Paoli, Sperberg-McQueen, Maler & Yergeau (2008)** *XML
//!   1.0 (Fifth Edition)*, W3C Recommendation 26 November 2008 — the
//!   syntactic substrate read by the byte hop.
//! - **Foster, Greenwald, Moore, Pierce & Schmitt (2007)** "Combinators
//!   for Bidirectional Tree Transformations", *ACM TOPLAS* 29(3) §2.2
//!   — the well-behaved-lens laws.
//! - **Bancilhon & Spyratos (1981)** "Update Semantics of Relational
//!   Views", *ACM TODS* 6(4) Theorem 3 — constant-complement view
//!   update.
//! - **Boyer & Marcy (2008)** *Canonical XML Version 1.1*, W3C
//!   Recommendation 2 May 2008 — the canonical form used by
//!   [`XsdSchemaLens::canonical`].

#[allow(unused_imports)]
use alloc::{format, string::String, vec::Vec};
use core::fmt;

use super::from_xml::project_from_xml_document;
use super::from_xsd_parser::XsdOntologyInstance;
use crate::formal::meta::well_behaved_lens::{WellBehavedLens, canonical::xml as xml_canonical};
use crate::social::software::markup::xml::parser::grammar::{XmlParseError, parse_document};

/// The byte-anchored view of a schema document — the typed
/// [`XsdOntologyInstance`] projection plus the original bytes as the
/// constant complement (Bancilhon & Spyratos 1981 Theorem 3).
///
/// Holding the complement constant across put-without-modification is
/// what makes the well-behaved-lens PutGet law hold byte-canonically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XsdSchema {
    /// The projected typed view — every recognised
    /// [`crate::formal::meta::xsd::ontology::XsdConcept`] instance the
    /// schema document declares (W3C XSD 1.1 Part 1 §2.2 + §2.5 +
    /// §3.16).
    pub instance: XsdOntologyInstance,
    /// The complement: the original source bytes from which `instance`
    /// was derived. Per Bancilhon & Spyratos 1981 Theorem 3, holding
    /// the complement constant recovers the source verbatim on
    /// put-without-modification.
    pub complement: Vec<u8>,
}

/// Error of [`XsdSchemaLens::get`] / [`XsdSchemaLens::put`].
#[derive(Debug, Clone)]
pub enum XsdSchemaLensError {
    /// Input was not valid UTF-8.
    NotUtf8(String),
    /// The XML 1.0 layer rejected the bytes (well-formedness failure).
    XmlParse(String),
    /// Canonicalization (W3C C14N 1.1) failed.
    Canonical(String),
}

impl fmt::Display for XsdSchemaLensError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XsdSchemaLensError::NotUtf8(m) => write!(f, "xsd schema lens: not valid UTF-8: {m}"),
            XsdSchemaLensError::XmlParse(m) => write!(f, "xsd schema lens: XML parse failed: {m}"),
            XsdSchemaLensError::Canonical(m) => {
                write!(f, "xsd schema lens: canonicalization failed: {m}")
            }
        }
    }
}

impl From<XmlParseError> for XsdSchemaLensError {
    fn from(e: XmlParseError) -> Self {
        XsdSchemaLensError::XmlParse(format!("{e}"))
    }
}

/// The byte-anchored [`WellBehavedLens`] binding `bytes ⇆ XsdSchema`.
///
/// `get(bytes)` runs the XML 1.0 parser and then the XSD projector
/// over the resulting tree, retaining the original bytes as the
/// complement. `put(target)` returns the complement, satisfying the
/// constant-complement PutGet law byte-canonically (Bancilhon &
/// Spyratos 1981 Theorem 3). `canonical` routes through the praxis
/// XML C14N library so the round-trip harness can pin a stable
/// signature in `praxis.lock`.
pub struct XsdSchemaLens;

impl WellBehavedLens for XsdSchemaLens {
    type Target = XsdSchema;
    type Error = XsdSchemaLensError;

    fn get(bytes: &[u8]) -> Result<Self::Target, Self::Error> {
        let doc = parse_document(bytes)?;
        let instance = project_from_xml_document(&doc);
        Ok(XsdSchema {
            instance,
            complement: bytes.to_vec(),
        })
    }

    fn put(target: &Self::Target) -> Result<Vec<u8>, Self::Error> {
        Ok(target.complement.clone())
    }

    fn canonical(bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        xml_canonical::canonicalize(bytes)
            .map_err(|e| XsdSchemaLensError::Canonical(format!("{e}")))
    }
}

// =============================================================================
// Round-trip harness registrations — one entry per registered XSD
// source. The `RoundTripHarnessAllVerified` axiom picks these up at
// link time and runs the PutGet law + canonical-signature pin against
// the on-disk bytes.
// =============================================================================

crate::register_lens!(USLM_XSD_LENS, "uslm_xsd", "1.0.18", XsdSchemaLens);
crate::register_lens!(XHTML_1_0_XSD_LENS, "xhtml_1_0_xsd", "1.0", XsdSchemaLens);
crate::register_lens!(
    XML_1_0_NAMESPACE_XSD_LENS,
    "xml_1_0_namespace_xsd",
    "1.0",
    XsdSchemaLens
);
crate::register_lens!(
    XSD_META_SCHEMA_LENS,
    "xsd_meta_schema",
    "1.1",
    XsdSchemaLens
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_then_put_returns_original_bytes() {
        // GetPut on a minimal in-memory schema. Constant-complement
        // discipline means put(get(bytes)) == bytes byte-canonically.
        let bytes = br#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="a" type="xs:string"/>
</xs:schema>"#
            .to_vec();
        let target = <XsdSchemaLens as WellBehavedLens>::get(&bytes).expect("parse + project");
        let back = <XsdSchemaLens as WellBehavedLens>::put(&target).expect("put");
        assert_eq!(back, bytes);
    }

    #[test]
    fn get_projects_xsd_components() {
        // The projector emits a SchemaDocument and an
        // ElementDeclaration for this minimal schema (XSD 1.1 Part 1
        // §2.5 + §3.3).
        use crate::formal::meta::xsd::ontology::XsdConcept;
        let bytes = br#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="a" type="xs:string"/>
</xs:schema>"#
            .to_vec();
        let target = <XsdSchemaLens as WellBehavedLens>::get(&bytes).expect("parse + project");
        assert!(
            target
                .instance
                .components
                .contains(&XsdConcept::SchemaDocument)
        );
        assert!(
            target
                .instance
                .components
                .contains(&XsdConcept::ElementDeclaration)
        );
    }

    #[test]
    fn put_get_law_holds() {
        // The WellBehavedLens trait surface includes a convenience
        // law-checker the harness calls.
        let bytes = br#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:simpleType name="t"><xs:restriction base="xs:string"/></xs:simpleType>
</xs:schema>"#
            .to_vec();
        assert!(<XsdSchemaLens as WellBehavedLens>::assert_put_get_law(&bytes).is_ok());
    }

    #[test]
    fn not_wellformed_xml_is_rejected() {
        let bytes = br#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="a"></xs:attribute>
</xs:schema>"#
            .to_vec();
        assert!(matches!(
            <XsdSchemaLens as WellBehavedLens>::get(&bytes),
            Err(XsdSchemaLensError::XmlParse(_))
        ));
    }

    proptest::proptest! {
        /// Robustness: for arbitrary byte streams, `get` either
        /// returns a `XsdSchema` (the XML 1.0 parser accepts and the
        /// XSD projector projects) or a typed [`XsdSchemaLensError`].
        /// It never panics, never silently swallows malformed input.
        #[test]
        fn prop_get_never_panics_on_arbitrary_bytes(
            bytes in proptest::collection::vec(proptest::prelude::any::<u8>(), 0..512)
        ) {
            let _ = <XsdSchemaLens as WellBehavedLens>::get(&bytes);
        }

        /// When `get` succeeds on bytes, `put` of the resulting
        /// target returns the original bytes byte-canonically
        /// (Bancilhon & Spyratos 1981 constant-complement). The
        /// invariant holds for every successfully-parsed XSD
        /// schema document — not just the canon fixtures.
        #[test]
        fn prop_get_put_canonical_on_success(
            bytes in proptest::collection::vec(proptest::prelude::any::<u8>(), 0..512)
        ) {
            if let Ok(target) = <XsdSchemaLens as WellBehavedLens>::get(&bytes) {
                let back = <XsdSchemaLens as WellBehavedLens>::put(&target)
                    .expect("put always succeeds on a successful get");
                proptest::prop_assert_eq!(back, bytes);
            }
        }
    }
}
