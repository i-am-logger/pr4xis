//! Tests for [`super::project_from_xml_document`] —
//! the praxis-native XSD-document → XsdOntologyInstance projection.

#[allow(unused_imports)]
use alloc::{string::String, string::ToString, vec, vec::Vec};

use super::super::from_xsd_parser::project_from_xsd_text;
use super::super::ontology::XsdConcept;
use super::project_from_xml_document;
use crate::social::software::markup::xml::parser::grammar::parse_document;

#[test]
fn projects_a_single_xsd_element_declaration() {
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="foo" type="xs:string"/>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let instance = project_from_xml_document(&doc);
    assert_eq!(instance.elements.len(), 1);
    assert_eq!(instance.elements[0].local_name, "foo");
    assert_eq!(instance.elements[0].type_ref.as_deref(), Some("xs:string"));
    assert_eq!(instance.named.len(), 1);
    assert_eq!(instance.named[0].concept, XsdConcept::ElementDeclaration);
    assert_eq!(instance.named[0].local_name, "foo");
}

#[test]
fn projects_six_xsd_declaration_kinds() {
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="el" type="xs:string"/>
  <xs:complexType name="ct"/>
  <xs:simpleType name="st"/>
  <xs:attributeGroup name="ag"/>
  <xs:group name="g"/>
  <xs:attribute name="att" type="xs:string"/>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let instance = project_from_xml_document(&doc);

    let names_by_concept: Vec<(XsdConcept, &str)> = instance
        .named
        .iter()
        .map(|n| (n.concept, n.local_name.as_str()))
        .collect();
    assert_eq!(
        names_by_concept,
        vec![
            (XsdConcept::ElementDeclaration, "el"),
            (XsdConcept::ComplexTypeDefinition, "ct"),
            (XsdConcept::SimpleTypeDefinition, "st"),
            (XsdConcept::AttributeGroup, "ag"),
            (XsdConcept::ModelGroup, "g"),
            (XsdConcept::AttributeDeclaration, "att"),
        ]
    );
}

#[test]
fn projects_substitution_group_head_on_element_declarations() {
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="block" type="xs:string"/>
  <xs:element name="note" type="xs:string" substitutionGroup="block"/>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let instance = project_from_xml_document(&doc);
    let note = instance
        .elements
        .iter()
        .find(|e| e.local_name == "note")
        .unwrap();
    assert_eq!(note.substitution_group_head.as_deref(), Some("block"));
}

#[test]
fn ignores_ref_only_declarations() {
    // W3C XSD 1.1 Part 1 §3.3.3 — an `<xsd:element ref="…">` is a
    // *reference* to an existing declaration, not a new one. We
    // project only declarations carrying `name=` (Part 1 §3.3.1).
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="root" type="xs:string"/>
  <xs:element ref="root"/>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let instance = project_from_xml_document(&doc);
    assert_eq!(instance.elements.len(), 1);
}

#[test]
fn dispatches_on_namespace_uri_not_prefix() {
    // The XSD namespace can be bound to any prefix per W3C XML
    // Namespaces 1.0 §6. Here it's bound to `foo:` instead of the
    // conventional `xs:` / `xsd:`.
    let xsd = r#"<?xml version="1.0"?>
<foo:schema xmlns:foo="http://www.w3.org/2001/XMLSchema">
  <foo:element name="e" type="foo:string"/>
</foo:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let instance = project_from_xml_document(&doc);
    assert_eq!(
        instance.elements.len(),
        1,
        "unconventional prefix still resolves to XSD URI"
    );
    assert_eq!(instance.elements[0].local_name, "e");
}

#[test]
fn skips_non_xsd_namespace_elements() {
    // Elements outside the XSD namespace are not schema components,
    // even if their local name happens to match a recognized one.
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           xmlns:other="http://example.org/other">
  <xs:element name="real"/>
  <other:element name="fake"/>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let instance = project_from_xml_document(&doc);
    let names: Vec<&str> = instance
        .elements
        .iter()
        .map(|e| e.local_name.as_str())
        .collect();
    assert_eq!(names, vec!["real"]);
}

#[test]
fn handles_default_namespace_binding_for_xsd() {
    // XSD can be the default namespace too — common in some
    // hand-authored schemas.
    let xsd = r#"<?xml version="1.0"?>
<schema xmlns="http://www.w3.org/2001/XMLSchema">
  <element name="e" type="string"/>
</schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let instance = project_from_xml_document(&doc);
    assert_eq!(instance.elements.len(), 1);
    assert_eq!(instance.elements[0].local_name, "e");
}

#[test]
fn projects_real_uslm_xsd_via_praxis_xml() {
    // Read the bundled USLM 1.0.18 XSD through praxis-xml + the
    // praxis-native XSD reader, then check the projection picks up
    // a representative sample of USLM declarations.
    let xsd_bytes = include_bytes!("../../../../../data/legal/uscode/schema/uslm-1.0.18.xsd");
    let doc = parse_document(xsd_bytes).expect("USLM XSD must parse via praxis-xml");
    let instance = project_from_xml_document(&doc);

    // The schema declares dozens of elements, complex types, simple
    // types, attribute groups, and groups. A simple cardinality
    // check ensures the projection actually walked the tree.
    assert!(
        instance.elements.len() > 50,
        "expected at least 50 element declarations in USLM XSD, got {}",
        instance.elements.len()
    );

    // Representative USLM element names that must be present.
    let names: Vec<&str> = instance
        .elements
        .iter()
        .map(|e| e.local_name.as_str())
        .collect();
    for expected in [
        "uscDoc",
        "section",
        "title",
        "subsection",
        "paragraph",
        "meta",
    ] {
        assert!(
            names.contains(&expected),
            "missing USLM element declaration {expected:?}"
        );
    }
}

#[test]
fn matches_text_scanner_on_simple_schema() {
    // The praxis-native projection and the legacy text-scanner
    // projection should agree on schemas they both handle.
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="a" type="xs:string"/>
  <xs:element name="b" type="xs:int" substitutionGroup="a"/>
  <xs:complexType name="c"/>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let via_xml = project_from_xml_document(&doc);
    let via_text = project_from_xsd_text(xsd);
    assert_eq!(via_xml.elements, via_text.elements);
    assert_eq!(via_xml.named, via_text.named);
}
