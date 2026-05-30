//! W3C XML Schema 1.1 conformance harness for the praxis-native XSD
//! reader.
//!
//! Mirrors the structure of the XML 1.0 conformance harness
//! ([`crate::social::software::markup::xml::parser::conformance`]) for
//! the XSD layer: a [`XsdConformanceCase`] taxonomy modelled on the
//! W3C **XML Schema Test Suite** (xsts) and a representative,
//! §-cited corpus the reader MUST handle correctly.
//!
//! ## What "conformance" means for a projector
//!
//! The praxis XSD reader ([`project_from_xml_document`]) is a
//! *structural projector*: it reads an `<xs:schema>` document and
//! projects its constructs onto [`XsdConcept`] ontology instances. It
//! is **not** an XSD *validator* — it does not enforce the
//! component-level validity rules (unique-particle-attribution, type
//! derivation legality, …) that the xsts `schemaTest` cases probe.
//! So the conformance categories are recast for a projector:
//!
//! - [`XsdCaseType::SchemaValid`] — a valid XSD 1.1 schema document.
//!   The reader MUST parse it as XML and project the expected
//!   constructs (its [`XsdConcept`]s appear in the instance).
//! - [`XsdCaseType::SchemaWellFormedButInvalid`] — a well-formed XML
//!   document that is *not* a valid XSD (e.g. a constraint violation).
//!   A validator would reject it; the structural projector still reads
//!   the constructs it recognises. This documents the projector /
//!   validator boundary honestly.
//! - [`XsdCaseType::NotWellFormed`] — not well-formed XML. The reader
//!   MUST reject it (the underlying XML 1.0 parse fails).
//!
//! The full xsts archive (thousands of testGroups with `testSet` /
//! `testGroup` metadata) is registered + run as a tracked praxis
//! source in deferred follow-up work, mirroring the XML 1.0
//! `M4.λ.1.d.b` plan; this in-repo canon is the always-run, CI-gated
//! representative.
//!
//! ## Citation
//!
//! - **Gao, Sperberg-McQueen & Thompson (2012)** W3C XML Schema 1.1
//!   Part 1: Structures; **Peterson et al. (2012)** Part 2: Datatypes
//!   — the constructs under test.
//! - **W3C XML Schema Working Group**, "XML Schema Test Suite",
//!   <https://www.w3.org/XML/2004/xml-schema-test-suite/> — the
//!   canonical corpus modelled here.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

use super::from_xml::project_from_xml_document;
use super::ontology::XsdConcept;
use crate::social::software::markup::xml::parser::grammar::parse_document;

/// xsts-derived category for an XSD conformance case (recast for the
/// structural projector — see the module docs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XsdCaseType {
    /// A valid XSD 1.1 schema. The reader must parse + project the
    /// expected constructs.
    SchemaValid,
    /// Well-formed XML but not a valid XSD. The structural projector
    /// still reads the recognised constructs (validator boundary).
    SchemaWellFormedButInvalid,
    /// Not well-formed XML. The reader must reject it.
    NotWellFormed,
}

/// One XSD conformance case.
#[derive(Debug, Clone)]
pub struct XsdConformanceCase {
    /// Stable identifier — used in failure messages.
    pub id: &'static str,
    /// The conformance category.
    pub case_type: XsdCaseType,
    /// The XSD 1.1 §-reference the case exercises.
    pub section: &'static str,
    /// Short prose description.
    pub description: &'static str,
    /// The schema-document bytes under test.
    pub source: &'static [u8],
    /// Constructs the projector must surface (for `SchemaValid` /
    /// `SchemaWellFormedButInvalid`); empty for `NotWellFormed`.
    pub expect_concepts: &'static [XsdConcept],
}

/// Outcome of running one [`XsdConformanceCase`].
#[derive(Debug, Clone)]
pub struct XsdCaseOutcome {
    /// The case identifier.
    pub case_id: &'static str,
    /// The expected category.
    pub expected: XsdCaseType,
    /// Whether the reader behaved as the category requires.
    pub passed: bool,
    /// Human-readable detail (which check failed).
    pub detail: String,
}

/// Run one case through the praxis XSD reader.
///
/// - `SchemaValid` / `SchemaWellFormedButInvalid`: the XML parse must
///   succeed and every `expect_concepts` entry must appear in the
///   projected instance.
/// - `NotWellFormed`: the XML parse must fail.
pub fn run_case(case: &XsdConformanceCase) -> XsdCaseOutcome {
    let parsed = parse_document(case.source);
    let (passed, detail) = match case.case_type {
        XsdCaseType::NotWellFormed => match parsed {
            Ok(_) => (
                false,
                "expected not-well-formed rejection, but parse succeeded".to_string(),
            ),
            Err(_) => (
                true,
                "rejected as not well-formed (as required)".to_string(),
            ),
        },
        XsdCaseType::SchemaValid | XsdCaseType::SchemaWellFormedButInvalid => match parsed {
            Err(e) => (
                false,
                format!("expected a successful parse, got error: {e}"),
            ),
            Ok(doc) => {
                let instance = project_from_xml_document(&doc);
                let missing: Vec<&XsdConcept> = case
                    .expect_concepts
                    .iter()
                    .filter(|c| !instance.components.contains(c))
                    .collect();
                if missing.is_empty() {
                    (true, "all expected constructs projected".to_string())
                } else {
                    (false, format!("projected instance missing {missing:?}"))
                }
            }
        },
    };
    XsdCaseOutcome {
        case_id: case.id,
        expected: case.case_type,
        passed,
        detail,
    }
}

/// Run the whole [`canon`], returning every outcome.
pub fn run_canon() -> Vec<XsdCaseOutcome> {
    canon().iter().map(run_case).collect()
}

/// The representative, §-cited XSD 1.1 conformance corpus. One case
/// per major construct family plus the well-formed-but-invalid and
/// not-well-formed boundaries.
pub fn canon() -> Vec<XsdConformanceCase> {
    use XsdCaseType as T;
    use XsdConcept as C;
    vec![
        XsdConformanceCase {
            id: "element-attribute-decl",
            case_type: T::SchemaValid,
            section: "Part 1 §3.3 / §3.2 (element / attribute declarations)",
            description: "A schema declaring a global element and attribute.",
            source: br#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="a" type="xs:string"/>
  <xs:attribute name="b" type="xs:int"/>
</xs:schema>"#,
            expect_concepts: &[C::ElementDeclaration, C::AttributeDeclaration],
        },
        XsdConformanceCase {
            id: "complex-type-sequence",
            case_type: T::SchemaValid,
            section: "Part 1 §3.4 / §3.8.1 (complexType + sequence)",
            description: "A complex type with a sequence model group.",
            source: br#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:complexType name="t">
    <xs:sequence><xs:element name="a" type="xs:string"/></xs:sequence>
  </xs:complexType>
</xs:schema>"#,
            expect_concepts: &[C::ComplexTypeDefinition, C::Sequence],
        },
        XsdConformanceCase {
            id: "simple-type-restriction-facets",
            case_type: T::SchemaValid,
            section: "Part 2 §4.1.2 / §4.3 (simpleType restriction + facets)",
            description: "A simple type restricting string by length + pattern.",
            source: br#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:simpleType name="code">
    <xs:restriction base="xs:string">
      <xs:length value="5"/>
      <xs:pattern value="[A-Z]{5}"/>
    </xs:restriction>
  </xs:simpleType>
</xs:schema>"#,
            expect_concepts: &[
                C::SimpleTypeDefinition,
                C::Restriction,
                C::LengthFacet,
                C::PatternFacet,
            ],
        },
        XsdConformanceCase {
            id: "list-and-union",
            case_type: T::SchemaValid,
            section: "Part 2 §4.1.2 (list / union simple types)",
            description: "List and union simple-type varieties.",
            source: br#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:simpleType name="l"><xs:list itemType="xs:int"/></xs:simpleType>
  <xs:simpleType name="u"><xs:union memberTypes="xs:int xs:string"/></xs:simpleType>
</xs:schema>"#,
            expect_concepts: &[C::ListType, C::UnionType],
        },
        XsdConformanceCase {
            id: "model-groups-choice-all",
            case_type: T::SchemaValid,
            section: "Part 1 §3.8.1 (choice / all model groups)",
            description: "Choice and all compositors.",
            source: br#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:complexType name="c"><xs:choice><xs:element name="x" type="xs:int"/></xs:choice></xs:complexType>
  <xs:complexType name="d"><xs:all><xs:element name="y" type="xs:int"/></xs:all></xs:complexType>
</xs:schema>"#,
            expect_concepts: &[C::Choice, C::AllGroup],
        },
        XsdConformanceCase {
            id: "identity-constraints",
            case_type: T::SchemaValid,
            section: "Part 1 §3.11 (key / keyref / unique + selector / field)",
            description: "Identity constraints with XPath sub-parts.",
            source: br#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="r">
    <xs:key name="k"><xs:selector xpath="a"/><xs:field xpath="@id"/></xs:key>
    <xs:keyref name="f" refer="k"><xs:selector xpath="b"/><xs:field xpath="@r"/></xs:keyref>
    <xs:unique name="u"><xs:selector xpath="c"/><xs:field xpath="@x"/></xs:unique>
  </xs:element>
</xs:schema>"#,
            expect_concepts: &[C::Key, C::KeyRef, C::Unique, C::Selector, C::Field],
        },
        XsdConformanceCase {
            id: "schema-composition",
            case_type: T::SchemaValid,
            section: "Part 1 §4.2 (import / include / override)",
            description: "Schema-composition directives.",
            source: br#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:import namespace="urn:x" schemaLocation="x.xsd"/>
  <xs:include schemaLocation="y.xsd"/>
  <xs:override schemaLocation="z.xsd"/>
</xs:schema>"#,
            expect_concepts: &[C::SchemaImport, C::SchemaInclude, C::SchemaOverride],
        },
        XsdConformanceCase {
            id: "xsd-1-1-additions",
            case_type: T::SchemaValid,
            section: "Part 1 §3.13 / §3.4.2.2 / §3.16.2 (assert / openContent / defaultOpenContent)",
            description: "The XSD 1.1 complex-type content additions.",
            source: br#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:defaultOpenContent mode="interleave"><xs:any/></xs:defaultOpenContent>
  <xs:complexType name="t">
    <xs:openContent mode="suffix"><xs:any/></xs:openContent>
    <xs:sequence><xs:element name="a" type="xs:string"/></xs:sequence>
    <xs:assert test="@x &gt; 0"/>
  </xs:complexType>
</xs:schema>"#,
            expect_concepts: &[C::Assert, C::OpenContent, C::DefaultOpenContent],
        },
        XsdConformanceCase {
            id: "notation-declaration",
            case_type: T::SchemaValid,
            section: "Part 1 §3.14 (notation declarations)",
            description: "A schema declaring a single <xsd:notation> binding a name to a \
                          public / system identifier pair (the XSD analogue of an XML 1.0 \
                          NOTATION declaration, Bray et al. 2008 §4.7).",
            source: br#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:notation name="jpeg" public="image/jpeg" system="viewer.exe"/>
</xs:schema>"#,
            expect_concepts: &[C::NotationDeclaration],
        },
        XsdConformanceCase {
            id: "annotation",
            case_type: T::SchemaValid,
            section: "Part 1 §3.15 (annotation / documentation / appinfo)",
            description: "An annotation with documentation and appinfo. The projector \
                          surfaces the Annotation component and captures the appinfo / \
                          documentation text into the instance's `annotations` field.",
            source: br#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:annotation>
    <xs:documentation>doc</xs:documentation>
    <xs:appinfo>info</xs:appinfo>
  </xs:annotation>
</xs:schema>"#,
            expect_concepts: &[C::Annotation],
        },
        XsdConformanceCase {
            id: "wellformed-but-xsd-invalid",
            case_type: T::SchemaWellFormedButInvalid,
            section: "Part 1 §3.4.6.4 (a validator would reject; the projector reads it)",
            description: "Well-formed XML; an <xs:element> with both a `type` attribute \
                          and an inline <xs:complexType> is XSD-invalid, but the structural \
                          projector still recognises the element declaration.",
            source: br#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="bad" type="xs:string">
    <xs:complexType><xs:sequence/></xs:complexType>
  </xs:element>
</xs:schema>"#,
            expect_concepts: &[C::ElementDeclaration],
        },
        XsdConformanceCase {
            id: "not-wellformed-mismatched-tags",
            case_type: T::NotWellFormed,
            section: "W3C XML 1.0 §3.1 (Element Type Match) — the schema is not well-formed XML",
            description: "Mismatched start/end tags: not well-formed XML, must be rejected.",
            source: br#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="a"></xs:attribute>
</xs:schema>"#,
            expect_concepts: &[],
        },
        XsdConformanceCase {
            id: "not-wellformed-unclosed",
            case_type: T::NotWellFormed,
            section: "W3C XML 1.0 §2.1 (an unclosed root element is not well-formed)",
            description: "Unclosed <xs:schema>: not well-formed XML, must be rejected.",
            source: br#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="a" type="xs:string"/>"#,
            expect_concepts: &[],
        },
    ]
}

/// **Axiom XsdSchemaTestSuiteConformance.** The praxis XSD reader
/// handles every case in the hand-authored conformance [`canon`]
/// correctly: it projects the expected [`XsdConcept`]s for every
/// `SchemaValid` / `SchemaWellFormedButInvalid` case and rejects
/// every `NotWellFormed` case at the XML-parse boundary.
///
/// The canon mirrors the W3C **XML Schema Test Suite** (xsts)
/// categories, recast for a structural projector (see the module
/// docs). Per `feedback_corpus_wide_audit_on_load`, this axiom walks
/// every case through the reader at test time; the full xsts archive
/// is registered as a tracked praxis source in deferred follow-up
/// (M4.λ.2.e.b, mirroring the XML 1.0 `M4.λ.1.d.b` plan).
pub struct XsdProjectorPassesConformanceCanon;

impl Axiom for XsdProjectorPassesConformanceCanon {
    fn verify(&self) -> Verdict {
        let failures: Vec<String> = run_canon()
            .into_iter()
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
        "XsdProjectorPassesConformanceCanon",
        "the XSD reader projects the expected constructs for every valid / well-formed-but-invalid case and rejects every not-well-formed case (W3C XML Schema Test Suite categories)",
        "W3C XML Schema Working Group, XML Schema Test Suite (xsts); Gao, Sperberg-McQueen & Thompson (2012) W3C XML Schema 1.1 Part 1"
    );
}

pr4xis::register_axiom!(
    XsdProjectorPassesConformanceCanon,
    "W3C XML Schema Working Group, XML Schema Test Suite (xsts)"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canon_is_representative() {
        let cases = canon();
        // Every taxonomy category is exercised.
        assert!(
            cases
                .iter()
                .any(|c| c.case_type == XsdCaseType::SchemaValid)
        );
        assert!(
            cases
                .iter()
                .any(|c| c.case_type == XsdCaseType::SchemaWellFormedButInvalid)
        );
        assert!(
            cases
                .iter()
                .any(|c| c.case_type == XsdCaseType::NotWellFormed)
        );
        assert!(cases.len() >= 10);
    }

    #[test]
    fn every_case_passes() {
        for outcome in run_canon() {
            assert!(
                outcome.passed,
                "conformance case {} failed: {}",
                outcome.case_id, outcome.detail
            );
        }
    }

    #[test]
    fn conformance_axiom_holds() {
        assert!(XsdProjectorPassesConformanceCanon.verify().is_ok());
    }
}
