//! Registered axioms for the praxis-native XSD-document →
//! [`super::super::from_xsd_parser::XsdOntologyInstance`] projection.
//!
//! Each axiom asserts a property the projection MUST satisfy per
//! the W3C XSD 1.1 Part 1 specification. Per
//! `feedback_high_test_coverage`, axioms are the third layer of
//! test depth alongside unit tests (`super::tests`) and proptest
//! property coverage.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

use super::super::from_xsd_parser::{DerivationMethod, SchemaImportInfo, SchemaIncludeInfo};
use super::super::ontology::XsdConcept;
use super::project_from_xml_document;
use crate::social::software::markup::xml::parser::grammar::parse_document;

/// **Axiom SchemaCompositionProjected.** Per W3C XSD 1.1 Part 1
/// §4.2, the `<xs:import>`, `<xs:include>`, `<xs:redefine>`, and
/// `<xs:override>` schema-composition directives MUST appear on
/// the projected [`super::super::from_xsd_parser::XsdOntologyInstance`] when present in the
/// source XSD. Asserts the projection captures all four kinds.
pub struct XsdSchemaCompositionProjected;

impl Axiom for XsdSchemaCompositionProjected {
    fn verify(&self) -> Verdict {
        let xsd = br#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:import namespace="http://example.org/a" schemaLocation="a.xsd"/>
  <xs:include schemaLocation="b.xsd"/>
  <xs:redefine schemaLocation="c.xsd"/>
  <xs:override schemaLocation="d.xsd"/>
</xs:schema>"#;
        let doc = match parse_document(xsd) {
            Ok(d) => d,
            Err(_) => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        let instance = project_from_xml_document(&doc);
        let expected_import = SchemaImportInfo {
            namespace: Some("http://example.org/a".into()),
            schema_location: Some("a.xsd".into()),
        };
        let expected_include = SchemaIncludeInfo {
            schema_location: "b.xsd".into(),
        };
        let all_present = instance.imports == vec![expected_import]
            && instance.includes == vec![expected_include]
            && instance.redefines.len() == 1
            && instance.overrides.len() == 1;
        if all_present {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "XsdSchemaCompositionProjected",
        "the projection captures all four W3C XSD 1.1 §4.2 schema-composition directives (import / include / redefine / override)",
        "Gao, Sperberg-McQueen & Thompson (2012) W3C XML Schema 1.1 Part 1 §4.2 Schemas and Namespaces"
    );
}

pr4xis::register_axiom!(
    XsdSchemaCompositionProjected,
    "W3C XSD 1.1 Part 1 (2012) §4.2 Schemas and Namespaces"
);

/// **Axiom AnnotationProjected.** Per W3C XSD 1.1 Part 1 §3.15,
/// `<xs:annotation>` MAY appear on any schema component and MUST
/// be projected through to the loaded ontology so downstream
/// tooling can surface its documentation. Asserts both top-level
/// and nested (inside `<xs:element>`) annotations are captured
/// with their `<xs:documentation>` payload.
pub struct XsdAnnotationProjected;

impl Axiom for XsdAnnotationProjected {
    fn verify(&self) -> Verdict {
        let xsd = br#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:annotation>
    <xs:documentation>top-level</xs:documentation>
  </xs:annotation>
  <xs:element name="foo">
    <xs:annotation>
      <xs:documentation>nested</xs:documentation>
    </xs:annotation>
  </xs:element>
</xs:schema>"#;
        let doc = match parse_document(xsd) {
            Ok(d) => d,
            Err(_) => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        let instance = project_from_xml_document(&doc);
        let docs: Vec<&str> = instance
            .annotations
            .iter()
            .flat_map(|a| a.documentation.iter().map(String::as_str))
            .collect();
        if docs.contains(&"top-level") && docs.contains(&"nested") {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "XsdAnnotationProjected",
        "<xs:annotation>/<xs:documentation> blocks are captured by the projection at every nesting level",
        "Gao, Sperberg-McQueen & Thompson (2012) W3C XML Schema 1.1 Part 1 §3.15 Annotations"
    );
}

pr4xis::register_axiom!(
    XsdAnnotationProjected,
    "W3C XSD 1.1 Part 1 (2012) §3.15 Annotations"
);

/// **Axiom TypeDerivationProjected.** Per W3C XSD 1.1 Part 1
/// §3.4.6, a complex type derives from a base type by
/// `<xs:restriction>` or `<xs:extension>`, each carrying a
/// `base="…"` reference. Asserts the projection captures both the
/// derivation concept and the base-type edge.
pub struct XsdTypeDerivationProjected;

impl Axiom for XsdTypeDerivationProjected {
    fn verify(&self) -> Verdict {
        let xsd = br#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:complexType name="d">
    <xs:complexContent>
      <xs:extension base="Base"/>
    </xs:complexContent>
  </xs:complexType>
</xs:schema>"#;
        let doc = match parse_document(xsd) {
            Ok(d) => d,
            Err(_) => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        let instance = project_from_xml_document(&doc);
        let has_concepts = instance.components.contains(&XsdConcept::ComplexContent)
            && instance.components.contains(&XsdConcept::Extension);
        let has_base = instance
            .derivations
            .iter()
            .any(|d| d.method == DerivationMethod::Extension && d.base.as_deref() == Some("Base"));
        if has_concepts && has_base {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "XsdTypeDerivationProjected",
        "the projection captures <xs:restriction>/<xs:extension> derivation concepts + their base= edge per W3C XSD 1.1 Part 1 §3.4.6",
        "Gao, Sperberg-McQueen & Thompson (2012) W3C XML Schema 1.1 Part 1 §3.4.6 Constraints on Complex Type Definition Schema Components"
    );
}

pr4xis::register_axiom!(
    XsdTypeDerivationProjected,
    "W3C XSD 1.1 Part 1 (2012) §3.4.6 type derivation"
);

/// **Axiom ModelGroupsProjected.** Per W3C XSD 1.1 Part 1 §3.8,
/// the three compositors `<xs:sequence>` / `<xs:choice>` /
/// `<xs:all>` and the §3.10 `<xs:any>` wildcard are projected as
/// their respective concepts.
pub struct XsdModelGroupsProjected;

impl Axiom for XsdModelGroupsProjected {
    fn verify(&self) -> Verdict {
        let xsd = br#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:complexType name="t">
    <xs:sequence>
      <xs:choice/>
      <xs:any/>
    </xs:sequence>
  </xs:complexType>
  <xs:complexType name="u"><xs:all/></xs:complexType>
</xs:schema>"#;
        let doc = match parse_document(xsd) {
            Ok(d) => d,
            Err(_) => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        let c = &project_from_xml_document(&doc).components;
        if c.contains(&XsdConcept::Sequence)
            && c.contains(&XsdConcept::Choice)
            && c.contains(&XsdConcept::AllGroup)
            && c.contains(&XsdConcept::Wildcard)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "XsdModelGroupsProjected",
        "the projection captures the §3.8 sequence/choice/all compositors + the §3.10 any wildcard",
        "Gao, Sperberg-McQueen & Thompson (2012) W3C XML Schema 1.1 Part 1 §3.8 Model Group Definitions, §3.10 Wildcards"
    );
}

pr4xis::register_axiom!(
    XsdModelGroupsProjected,
    "W3C XSD 1.1 Part 1 (2012) §3.8, §3.10"
);

/// **Axiom ConstrainingFacetsProjected.** Per W3C XSD 1.1 Part 2
/// §4.3, the constraining facets restrict a simple type's value
/// space. Asserts a representative sample (length §4.3.1, pattern
/// §4.3.4, enumeration §4.3.5, maxInclusive §4.3.7, explicitTimezone
/// §4.3.14, assertion §4.3.13) projects to its facet concept.
pub struct XsdConstrainingFacetsProjected;

impl Axiom for XsdConstrainingFacetsProjected {
    fn verify(&self) -> Verdict {
        let xsd = br#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:simpleType name="t">
    <xs:restriction base="xs:string">
      <xs:length value="5"/>
      <xs:pattern value="x"/>
      <xs:enumeration value="x"/>
      <xs:maxInclusive value="9"/>
      <xs:explicitTimezone value="required"/>
      <xs:assertion test="true()"/>
    </xs:restriction>
  </xs:simpleType>
</xs:schema>"#;
        let doc = match parse_document(xsd) {
            Ok(d) => d,
            Err(_) => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        let c = &project_from_xml_document(&doc).components;
        let all = [
            XsdConcept::LengthFacet,
            XsdConcept::PatternFacet,
            XsdConcept::EnumerationFacet,
            XsdConcept::MaxInclusiveFacet,
            XsdConcept::ExplicitTimezoneFacet,
            XsdConcept::AssertionFacet,
        ]
        .iter()
        .all(|f| c.contains(f));
        if all {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "XsdConstrainingFacetsProjected",
        "the projection captures the W3C XSD 1.1 Part 2 §4.3 constraining facets (length / pattern / enumeration / range / explicitTimezone / assertion)",
        "Peterson, Gao, Akhmedov, Malhotra, Biron & Sperberg-McQueen (2012) W3C XML Schema 1.1 Part 2: Datatypes §4.3"
    );
}

pr4xis::register_axiom!(
    XsdConstrainingFacetsProjected,
    "W3C XSD 1.1 Part 2 (2012) §4.3 Constraining Facets"
);

/// **Axiom IdentityConstraintsProjected.** Per W3C XSD 1.1 Part 1
/// §3.11, the identity-constraint categories (`<xs:key>` /
/// `<xs:keyref>` / `<xs:unique>`) and their XPath sub-parts
/// (`<xs:selector>` / `<xs:field>`) project to their concepts.
pub struct XsdIdentityConstraintsProjected;

impl Axiom for XsdIdentityConstraintsProjected {
    fn verify(&self) -> Verdict {
        let xsd = br#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="r">
    <xs:key name="k"><xs:selector xpath="a"/><xs:field xpath="@id"/></xs:key>
    <xs:unique name="u"><xs:selector xpath="b"/><xs:field xpath="@x"/></xs:unique>
    <xs:keyref name="f" refer="k"><xs:selector xpath="c"/><xs:field xpath="@r"/></xs:keyref>
  </xs:element>
</xs:schema>"#;
        let doc = match parse_document(xsd) {
            Ok(d) => d,
            Err(_) => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        let c = &project_from_xml_document(&doc).components;
        let all = [
            XsdConcept::Key,
            XsdConcept::Unique,
            XsdConcept::KeyRef,
            XsdConcept::Selector,
            XsdConcept::Field,
        ]
        .iter()
        .all(|k| c.contains(k));
        if all {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "XsdIdentityConstraintsProjected",
        "the projection captures the W3C XSD 1.1 Part 1 §3.11 identity-constraint categories (key/keyref/unique) and XPath sub-parts (selector/field)",
        "Gao, Sperberg-McQueen & Thompson (2012) W3C XML Schema 1.1 Part 1 §3.11 Identity-constraint Definitions"
    );
}

pr4xis::register_axiom!(
    XsdIdentityConstraintsProjected,
    "W3C XSD 1.1 Part 1 (2012) §3.11 Identity-constraint Definitions"
);

/// **Axiom Xsd11ContentAdditionsProjected.** XSD 1.1 introduced three
/// complex-type content constructs absent from XSD 1.0: `<xs:assert>`
/// (Part 1 §3.13 — an XPath 2.0 boolean test on each instance),
/// `<xs:openContent>` (§3.4.2.2 — wildcard content beyond the declared
/// particles) and the schema-level `<xs:defaultOpenContent>`
/// (§3.16.2). The projection captures all three.
pub struct Xsd11ContentAdditionsProjected;

impl Axiom for Xsd11ContentAdditionsProjected {
    fn verify(&self) -> Verdict {
        let xsd = br#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:defaultOpenContent mode="interleave"><xs:any/></xs:defaultOpenContent>
  <xs:complexType name="t">
    <xs:openContent mode="suffix"><xs:any/></xs:openContent>
    <xs:sequence><xs:element name="a" type="xs:string"/></xs:sequence>
    <xs:assert test="@x &gt; 0"/>
  </xs:complexType>
</xs:schema>"#;
        let doc = match parse_document(xsd) {
            Ok(d) => d,
            Err(_) => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        let c = &project_from_xml_document(&doc).components;
        let all = [
            XsdConcept::Assert,
            XsdConcept::OpenContent,
            XsdConcept::DefaultOpenContent,
        ]
        .iter()
        .all(|k| c.contains(k));
        if all {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "Xsd11ContentAdditionsProjected",
        "the projection captures the XSD 1.1 complex-type content additions: assert (Part 1 §3.13), openContent (§3.4.2.2) and defaultOpenContent (§3.16.2)",
        "Gao, Sperberg-McQueen & Thompson (2012) W3C XML Schema 1.1 Part 1 §§3.13, 3.4.2.2, 3.16.2"
    );
}

pr4xis::register_axiom!(
    Xsd11ContentAdditionsProjected,
    "W3C XSD 1.1 Part 1 (2012) §§3.13, 3.4.2.2, 3.16.2 — XSD 1.1 content additions"
);

#[cfg(test)]
mod axiom_tests {
    use super::*;

    #[test]
    fn schema_composition_axiom_holds() {
        assert!(XsdSchemaCompositionProjected.verify().is_ok());
    }

    #[test]
    fn identity_constraints_axiom_holds() {
        assert!(XsdIdentityConstraintsProjected.verify().is_ok());
    }

    #[test]
    fn annotation_axiom_holds() {
        assert!(XsdAnnotationProjected.verify().is_ok());
    }

    #[test]
    fn type_derivation_axiom_holds() {
        assert!(XsdTypeDerivationProjected.verify().is_ok());
    }

    #[test]
    fn model_groups_axiom_holds() {
        assert!(XsdModelGroupsProjected.verify().is_ok());
    }

    #[test]
    fn constraining_facets_axiom_holds() {
        assert!(XsdConstrainingFacetsProjected.verify().is_ok());
    }

    #[test]
    fn xsd11_content_additions_axiom_holds() {
        assert!(Xsd11ContentAdditionsProjected.verify().is_ok());
    }
}
