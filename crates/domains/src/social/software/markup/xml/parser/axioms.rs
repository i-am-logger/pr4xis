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
/// Greenwald, Moore, Pierce & Schmitt 2007 ACM TOPLAS 29(3) §3, Definition 3.2).
///
/// Verified by serializing a representative fixture and asserting
/// that the re-parsed value equals the original. Property-based
/// coverage of this law over random `XmlDocument`s lives in
/// the test-only `super::tests::property::property_get_put_law` proptest.
pub struct XmlLensGetPutLaw;

impl Axiom for XmlLensGetPutLaw {
    fn verify(&self) -> Verdict {
        let doc = XmlDocument {
            version: "1.0".into(),
            encoding: None,
            doctype: None,
            root: XmlElement {
                name: XmlName::new("axiom_witness"),
                namespace: None,
                namespaces: Vec::new(),
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
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) ACM TOPLAS 29(3) §3, Definition 3.2"
    );
}

pr4xis::register_axiom!(
    XmlLensGetPutLaw,
    "Foster et al. (2007) ACM TOPLAS 29(3) §3, Definition 3.2"
);

/// **Axiom PutGet.** For every byte stream of a well-formed XML
/// document, `canonical(put(get(s))) = canonical(s)` (Foster et
/// al. 2007 §3, Definition 3.2). The W3C XML Canonicalization 1.1 normalisation
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
        "Foster et al. (2007) §3, Definition 3.2; Boyer & Marcy (2008) W3C XML Canonicalization 1.1"
    );
}

pr4xis::register_axiom!(
    XmlLensPutGetLaw,
    "Foster et al. (2007) §3, Definition 3.2; Boyer & Marcy (2008) W3C C14N 1.1"
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

/// **Axiom UniqueAttSpec.** Per W3C XML 1.0 Fifth Edition §3.1
/// well-formedness constraint *Unique Att Spec*: no attribute name
/// may appear more than once in the same start-tag. Asserts that
/// the parser rejects such inputs with the typed
/// [`XmlParseError::DuplicateAttribute`] error.
pub struct XmlUniqueAttSpecEnforced;

impl Axiom for XmlUniqueAttSpecEnforced {
    fn verify(&self) -> Verdict {
        let xml = b"<?xml version=\"1.0\"?><r a=\"1\" a=\"2\"/>";
        match parse_document(xml) {
            Err(XmlParseError::DuplicateAttribute { .. }) => {
                Ok(Box::new(SimpleProof::new(self.meta())))
            }
            _ => Err(Box::new(SimpleCounterexample::new(self.meta()))),
        }
    }

    pr4xis::axiom_meta!(
        "XmlUniqueAttSpecEnforced",
        "the parser rejects inputs that violate the W3C XML 1.0 §3.1 Unique Att Spec well-formedness constraint",
        "Bray, Paoli, Sperberg-McQueen, Maler & Yergeau (2008) W3C XML 1.0 Fifth Edition §3.1"
    );
}

pr4xis::register_axiom!(
    XmlUniqueAttSpecEnforced,
    "W3C XML 1.0 Fifth Edition (2008) §3.1 Unique Att Spec"
);

/// **Axiom EndOfLineNormalization.** Per W3C XML 1.0 Fifth Edition
/// §2.11 End-of-Line Handling, the XML processor MUST normalize
/// every CRLF and lone CR sequence to a single LF before the
/// productions consume the input. Asserts the parser does so by
/// reading a CRLF-bearing document and observing LF in the typed
/// content.
pub struct XmlEndOfLineNormalizationEnforced;

impl Axiom for XmlEndOfLineNormalizationEnforced {
    fn verify(&self) -> Verdict {
        let xml = b"<?xml version=\"1.0\"?><r>a\r\nb\rc</r>";
        let doc = match parse_document(xml) {
            Ok(d) => d,
            Err(_) => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        let text = match doc.root.children.first() {
            Some(XmlNode::Text(t)) => t.clone(),
            _ => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        if text == "a\nb\nc" {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "XmlEndOfLineNormalizationEnforced",
        "the parser normalizes CRLF and lone CR to LF per W3C XML 1.0 §2.11 before productions consume the input",
        "Bray, Paoli, Sperberg-McQueen, Maler & Yergeau (2008) W3C XML 1.0 Fifth Edition §2.11"
    );
}

pr4xis::register_axiom!(
    XmlEndOfLineNormalizationEnforced,
    "W3C XML 1.0 Fifth Edition (2008) §2.11 End-of-Line Handling"
);

/// **Axiom AttributeValueNormalization.** Per W3C XML 1.0 Fifth
/// Edition §3.3.3 step 3.1.4, literal whitespace characters (`#x9`,
/// `#xA`, `#xD`) inside an attribute value MUST be replaced by a
/// single `#x20` (space) in the value passed to the application.
/// Character references (§3.3.3 step 3.1.1) pass through unchanged.
pub struct XmlAttributeValueNormalizationEnforced;

impl Axiom for XmlAttributeValueNormalizationEnforced {
    fn verify(&self) -> Verdict {
        // Literal tab in the value normalizes to space; `&#xA;`
        // reference passes through as LF (distinct case).
        let xml = b"<?xml version=\"1.0\"?><r a=\"x\ty\" b=\"x&#xA;y\"/>";
        let doc = match parse_document(xml) {
            Ok(d) => d,
            Err(_) => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        let a = doc
            .root
            .attributes
            .iter()
            .find(|a| a.name.local == "a")
            .map(|a| a.value.as_str());
        let b = doc
            .root
            .attributes
            .iter()
            .find(|a| a.name.local == "b")
            .map(|a| a.value.as_str());
        if a == Some("x y") && b == Some("x\ny") {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "XmlAttributeValueNormalizationEnforced",
        "the parser normalizes attribute values per W3C XML 1.0 §3.3.3 step 3.1.4 (whitespace → space; character references pass through)",
        "Bray, Paoli, Sperberg-McQueen, Maler & Yergeau (2008) W3C XML 1.0 Fifth Edition §3.3.3"
    );
}

pr4xis::register_axiom!(
    XmlAttributeValueNormalizationEnforced,
    "W3C XML 1.0 Fifth Edition (2008) §3.3.3 Attribute-Value Normalization"
);

/// **Axiom GeneralEntityResolution.** Per W3C XML 1.0 Fifth Edition
/// §4.4.3 (Included), a general entity reference in content MUST
/// be replaced by the entity's declared replacement text. Asserts
/// that a `<!DOCTYPE>` with an inline `<!ENTITY name "value">`
/// declaration causes `&name;` in the body to expand to `value`.
pub struct XmlGeneralEntityResolutionEnforced;

impl Axiom for XmlGeneralEntityResolutionEnforced {
    fn verify(&self) -> Verdict {
        let xml = b"<?xml version=\"1.0\"?><!DOCTYPE r [<!ENTITY e \"hello\">]><r>&e;</r>";
        let doc = match parse_document(xml) {
            Ok(d) => d,
            Err(_) => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        let resolved = matches!(
            doc.root.children.first(),
            Some(XmlNode::Text(t)) if t == "hello"
        );
        if resolved {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "XmlGeneralEntityResolutionEnforced",
        "the parser resolves DOCTYPE-declared general entity references per W3C XML 1.0 §4.4.3",
        "Bray, Paoli, Sperberg-McQueen, Maler & Yergeau (2008) W3C XML 1.0 Fifth Edition §4.4.3"
    );
}

pr4xis::register_axiom!(
    XmlGeneralEntityResolutionEnforced,
    "W3C XML 1.0 Fifth Edition (2008) §4.4.3 General Entity Resolution"
);

/// **Axiom FirstEntityDeclarationWins.** Per W3C XML 1.0 Fifth
/// Edition §4.5 (Construction of Internal Entity Replacement
/// Text), "if the same entity is declared more than once, the
/// first declaration encountered is binding". Asserts that a
/// duplicate `<!ENTITY>` declaration in the internal subset
/// preserves the first-declared value.
pub struct XmlFirstEntityDeclarationBinds;

impl Axiom for XmlFirstEntityDeclarationBinds {
    fn verify(&self) -> Verdict {
        let xml = b"<?xml version=\"1.0\"?><!DOCTYPE r [<!ENTITY x \"first\"><!ENTITY x \"second\">]><r>&x;</r>";
        let doc = match parse_document(xml) {
            Ok(d) => d,
            Err(_) => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        let bound = matches!(
            doc.root.children.first(),
            Some(XmlNode::Text(t)) if t == "first"
        );
        if bound {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "XmlFirstEntityDeclarationBinds",
        "duplicate <!ENTITY> declarations keep the first-declared value per W3C XML 1.0 §4.5",
        "Bray, Paoli, Sperberg-McQueen, Maler & Yergeau (2008) W3C XML 1.0 Fifth Edition §4.5"
    );
}

pr4xis::register_axiom!(
    XmlFirstEntityDeclarationBinds,
    "W3C XML 1.0 Fifth Edition (2008) §4.5 Construction of Internal Entity Replacement Text"
);

/// **Axiom W3CConformance.** The parser passes every case in the
/// hand-authored W3C XML 1.0 conformance canon
/// (`super::conformance::canon_corpus`). The canon mirrors the
/// W3C XML Conformance Test Suite (XMLConf, W3C XML Test Suite
/// Working Group) categories — `valid`, `invalid`, `not-wf`,
/// `error` — with one representative case per major
/// well-formedness or syntactic rule the parser claims to enforce.
///
/// Per `feedback_corpus_wide_audit_on_load`, this axiom walks
/// every case through the parser at test time. Future work
/// (M4.λ.1.d.b) registers the full ~2000-case XMLConf archive as
/// a praxis source.
pub struct XmlParserPassesConformanceCanon;

impl Axiom for XmlParserPassesConformanceCanon {
    fn verify(&self) -> Verdict {
        let outcomes = super::conformance::run_canon_corpus();
        let failures: Vec<String> = outcomes
            .iter()
            .filter(|o| !o.passed)
            .map(|o| format!("{} ({:?}): {}", o.case_id, o.expected, o.detail))
            .collect();
        if failures.is_empty() {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "XmlParserPassesConformanceCanon",
        "the parser handles every hand-authored conformance case correctly (valid/invalid/not-wf/error categories per W3C XMLConf)",
        "W3C XML Test Suite Working Group, XML Test Suite; Bray et al. (2008) W3C XML 1.0 Fifth Edition"
    );
}

pr4xis::register_axiom!(
    XmlParserPassesConformanceCanon,
    "W3C XML Test Suite Working Group, XML Test Suite (XMLConf)"
);

#[cfg(test)]
mod axiom_tests {
    use super::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn get_put_axiom_holds() {
        assert!(XmlLensGetPutLaw.verify().is_ok());
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn put_get_axiom_holds() {
        assert!(XmlLensPutGetLaw.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn predefined_entities_axiom_holds() {
        assert!(XmlPredefinedEntitiesAreRecognised.verify().is_ok());
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn element_type_match_axiom_holds() {
        assert!(XmlElementTypeMatchEnforced.verify().is_ok());
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn unique_att_spec_axiom_holds() {
        assert!(XmlUniqueAttSpecEnforced.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn end_of_line_normalization_axiom_holds() {
        assert!(XmlEndOfLineNormalizationEnforced.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn attribute_value_normalization_axiom_holds() {
        assert!(XmlAttributeValueNormalizationEnforced.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn general_entity_resolution_axiom_holds() {
        assert!(XmlGeneralEntityResolutionEnforced.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn first_entity_declaration_wins_axiom_holds() {
        assert!(XmlFirstEntityDeclarationBinds.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn parser_passes_conformance_canon_axiom_holds() {
        let outcomes = super::super::conformance::run_canon_corpus();
        for o in &outcomes {
            assert!(
                o.passed,
                "conformance case {} (expected {:?}) failed: {}",
                o.case_id, o.expected, o.detail
            );
        }
        // Also exercise the axiom wrapper.
        assert!(XmlParserPassesConformanceCanon.verify().is_ok());
    }
}
