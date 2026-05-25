//! Registered axioms for the praxis-native XSD-document →
//! [`XsdOntologyInstance`] projection.
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

use super::super::from_xsd_parser::{SchemaImportInfo, SchemaIncludeInfo};
use super::project_from_xml_document;
use crate::social::software::markup::xml::parser::grammar::parse_document;

/// **Axiom SchemaCompositionProjected.** Per W3C XSD 1.1 Part 1
/// §4.2, the `<xs:import>`, `<xs:include>`, `<xs:redefine>`, and
/// `<xs:override>` schema-composition directives MUST appear on
/// the projected [`XsdOntologyInstance`] when present in the
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

#[cfg(test)]
mod axiom_tests {
    use super::*;

    #[test]
    fn schema_composition_axiom_holds() {
        assert!(XsdSchemaCompositionProjected.verify().is_ok());
    }

    #[test]
    fn annotation_axiom_holds() {
        assert!(XsdAnnotationProjected.verify().is_ok());
    }
}
