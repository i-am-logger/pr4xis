//! Registered axioms for the praxis-native XML 1.0 parser.
//!
//! Each axiom names a property the parser/serializer pair MUST
//! satisfy per the published W3C standards and is verified at test
//! time via [`pr4xis::ontology::Axiom::verify`]. Per
//! `feedback_high_test_coverage`, the parser's three-layer test
//! depth is unit tests + property tests + registered axioms;
//! this file holds the third layer.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

use super::super::ontology::{XmlAttribute, XmlDocument, XmlElement, XmlName, XmlNode};
use super::grammar::{XmlParseError, parse_document};
use super::lens::XmlLens;
use crate::formal::meta::well_behaved_lens::WellBehavedLens;

/// **Axiom GetPut.** For every hand-built well-formed
/// [`XmlDocument`], `XmlLens::get(XmlLens::put(t)) = t` (Foster,
/// Greenwald, Moore, Pierce & Schmitt 2007 ACM TOPLAS 29(3) §2.2).
///
/// Verified by serializing a representative fixture and asserting
/// that the re-parsed value equals the original. Property-based
/// coverage of this law over random `XmlDocument`s lives in
/// [`super::tests::property::property_get_put_law`].
pub struct XmlLensGetPutLaw;

impl Axiom for XmlLensGetPutLaw {
    fn verify(&self) -> Verdict {
        let doc = XmlDocument {
            version: "1.0".into(),
            encoding: None,
            root: XmlElement {
                name: XmlName::new("axiom_witness"),
                namespace: None,
                attributes: vec![XmlAttribute {
                    name: XmlName::new("k"),
                    value: "v".into(),
                }],
                children: vec![XmlNode::Text("hello".into())],
            },
        };
        let bytes = match XmlLens::put(&doc) {
            Ok(b) => b,
            Err(_) => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        let parsed = match XmlLens::get(&bytes) {
            Ok(p) => p,
            Err(_) => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        if parsed == doc {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "XmlLensGetPutLaw",
        "XmlLens::get(XmlLens::put(t)) = t for well-formed typed values",
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) ACM TOPLAS 29(3) §2.2"
    );
}

pr4xis::register_axiom!(
    XmlLensGetPutLaw,
    "Foster et al. (2007) ACM TOPLAS 29(3) §2.2"
);

/// **Axiom PutGet.** For every byte stream of a well-formed XML
/// document, `canonical(put(get(s))) = canonical(s)` (Foster et
/// al. 2007 §2.2). The W3C XML Canonicalization 1.1 normalisation
/// (Boyer & Marcy 2008) is the equivalence we measure against.
pub struct XmlLensPutGetLaw;

impl Axiom for XmlLensPutGetLaw {
    fn verify(&self) -> Verdict {
        let xml = b"<?xml version=\"1.0\"?><r a=\"1\">hello&amp;world</r>";
        let parsed = match XmlLens::get(xml) {
            Ok(p) => p,
            Err(_) => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        let round = match XmlLens::put(&parsed) {
            Ok(b) => b,
            Err(_) => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        let canonical_source = match XmlLens::canonical(xml) {
            Ok(c) => c,
            Err(_) => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        let canonical_round = match XmlLens::canonical(&round) {
            Ok(c) => c,
            Err(_) => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        if canonical_source == canonical_round {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "XmlLensPutGetLaw",
        "canonical(put(get(s))) == canonical(s) under W3C C14N 1.1",
        "Foster et al. (2007) §2.2; Boyer & Marcy (2008) W3C XML Canonicalization 1.1"
    );
}

pr4xis::register_axiom!(
    XmlLensPutGetLaw,
    "Foster et al. (2007) §2.2; Boyer & Marcy (2008) W3C C14N 1.1"
);

/// **Axiom PredefinedEntities.** Per W3C XML 1.0 Fifth Edition
/// §4.6, every conforming XML processor MUST recognise the five
/// predefined entity references (`amp`, `lt`, `gt`, `apos`, `quot`)
/// and expand them to the corresponding characters. Asserts that
/// our parser does so on a single document containing all five.
pub struct XmlPredefinedEntitiesAreRecognised;

impl Axiom for XmlPredefinedEntitiesAreRecognised {
    fn verify(&self) -> Verdict {
        let xml = b"<?xml version=\"1.0\"?><r>&amp;&lt;&gt;&apos;&quot;</r>";
        let parsed = match parse_document(xml) {
            Ok(p) => p,
            Err(_) => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        let expected = XmlNode::Text("&<>'\"".to_string());
        if parsed.root.children.first() == Some(&expected) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "XmlPredefinedEntitiesAreRecognised",
        "the five W3C XML 1.0 §4.6 predefined entities are recognised by the parser",
        "Bray, Paoli, Sperberg-McQueen, Maler & Yergeau (2008) W3C XML 1.0 Fifth Edition §4.6"
    );
}

pr4xis::register_axiom!(
    XmlPredefinedEntitiesAreRecognised,
    "W3C XML 1.0 Fifth Edition (2008) §4.6"
);

/// **Axiom ElementTypeMatch.** Per W3C XML 1.0 Fifth Edition §3
/// "Logical Structures", well-formedness constraint *Element
/// Type Match*: every STag's `Name` MUST match its matching ETag's
/// `Name`. Asserts that our parser rejects mismatched-tag inputs
/// with the typed [`XmlParseError::MismatchedTags`] error.
pub struct XmlElementTypeMatchEnforced;

impl Axiom for XmlElementTypeMatchEnforced {
    fn verify(&self) -> Verdict {
        let xml = b"<?xml version=\"1.0\"?><a></b>";
        match parse_document(xml) {
            Err(XmlParseError::MismatchedTags { .. }) => {
                Ok(Box::new(SimpleProof::new(self.meta())))
            }
            _ => Err(Box::new(SimpleCounterexample::new(self.meta()))),
        }
    }

    pr4xis::axiom_meta!(
        "XmlElementTypeMatchEnforced",
        "the parser rejects inputs that violate the W3C XML 1.0 §3 Element Type Match well-formedness constraint",
        "Bray, Paoli, Sperberg-McQueen, Maler & Yergeau (2008) W3C XML 1.0 Fifth Edition §3"
    );
}

pr4xis::register_axiom!(
    XmlElementTypeMatchEnforced,
    "W3C XML 1.0 Fifth Edition (2008) §3 Element Type Match"
);

#[cfg(test)]
mod axiom_tests {
    use super::*;

    #[test]
    fn get_put_axiom_holds() {
        assert!(XmlLensGetPutLaw.verify().is_ok());
    }

    #[test]
    fn put_get_axiom_holds() {
        assert!(XmlLensPutGetLaw.verify().is_ok());
    }

    #[test]
    fn predefined_entities_axiom_holds() {
        assert!(XmlPredefinedEntitiesAreRecognised.verify().is_ok());
    }

    #[test]
    fn element_type_match_axiom_holds() {
        assert!(XmlElementTypeMatchEnforced.verify().is_ok());
    }
}
