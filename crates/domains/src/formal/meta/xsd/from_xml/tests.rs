//! Tests for [`super::project_from_xml_document`] —
//! the praxis-native XSD-document → XsdOntologyInstance projection.

#[allow(unused_imports)]
use alloc::{string::String, string::ToString, vec, vec::Vec};

use super::super::from_xsd_parser::{
    DerivationMethod, SchemaImportInfo, SchemaIncludeInfo, SchemaOverrideInfo, SchemaRedefineInfo,
    project_from_xsd_text,
};
use super::super::ontology::XsdConcept;
use super::project_from_xml_document;
use crate::social::software::markup::xml::parser::grammar::parse_document;

#[pr4xis::praxis_value(Verifiable)]
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

#[pr4xis::praxis_value(Verifiable)]
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

#[pr4xis::praxis_value(Verifiable)]
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

#[pr4xis::praxis_value(Honest)]
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

#[pr4xis::praxis_value(Verifiable)]
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

#[pr4xis::praxis_value(Honest)]
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

#[pr4xis::praxis_value(Verifiable)]
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

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn projects_real_uslm_xsd_via_praxis_xml() {
    // Read the bundled USLM 1.0.18 XSD through praxis-xml + the
    // praxis-native XSD reader, then check the projection picks up
    // a representative sample of USLM declarations.
    let xsd_bytes = crate::formal::meta::xsd::uslm_vocabulary::loaded_uslm_1_0_18_xsd().as_bytes();
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

// =============================================================================
// W3C XSD 1.1 Part 1 §4.2 Schema composition + §3.15 Annotations.
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn projects_xsd_import_directive() {
    // W3C XSD 1.1 Part 1 §4.2.6 — <xs:import namespace=... schemaLocation=...>.
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:import namespace="http://purl.org/dc/" schemaLocation="dc.xsd"/>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let instance = project_from_xml_document(&doc);
    assert_eq!(
        instance.imports,
        vec![SchemaImportInfo {
            namespace: Some("http://purl.org/dc/".into()),
            schema_location: Some("dc.xsd".into()),
        }]
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn projects_xsd_import_without_schema_location() {
    // §4.2.6.1: schemaLocation is a hint, not required.
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:import namespace="http://example.org/"/>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let instance = project_from_xml_document(&doc);
    assert_eq!(
        instance.imports,
        vec![SchemaImportInfo {
            namespace: Some("http://example.org/".into()),
            schema_location: None,
        }]
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn projects_xsd_include_directive() {
    // W3C XSD 1.1 Part 1 §4.2.3 — <xs:include schemaLocation=...>.
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:include schemaLocation="common.xsd"/>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let instance = project_from_xml_document(&doc);
    assert_eq!(
        instance.includes,
        vec![SchemaIncludeInfo {
            schema_location: "common.xsd".into(),
        }]
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn projects_xsd_redefine_directive() {
    // W3C XSD 1.1 Part 1 §4.2.4 — <xs:redefine schemaLocation=...>.
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:redefine schemaLocation="base.xsd"/>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let instance = project_from_xml_document(&doc);
    assert_eq!(
        instance.redefines,
        vec![SchemaRedefineInfo {
            schema_location: "base.xsd".into(),
        }]
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn projects_xsd_override_directive() {
    // W3C XSD 1.1 Part 1 §4.2.5 — <xs:override schemaLocation=...>.
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:override schemaLocation="base.xsd"/>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let instance = project_from_xml_document(&doc);
    assert_eq!(
        instance.overrides,
        vec![SchemaOverrideInfo {
            schema_location: "base.xsd".into(),
        }]
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn projects_top_level_annotation_with_documentation() {
    // W3C XSD 1.1 Part 1 §3.15 — <xs:annotation>/<xs:documentation>.
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:annotation>
    <xs:documentation>Top-level schema docs.</xs:documentation>
  </xs:annotation>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let instance = project_from_xml_document(&doc);
    assert_eq!(instance.annotations.len(), 1);
    assert_eq!(
        instance.annotations[0].documentation,
        vec!["Top-level schema docs.".to_string()]
    );
    assert!(instance.annotations[0].appinfo.is_empty());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn projects_annotation_with_appinfo() {
    // §3.15.1 — <xs:appinfo> machine-readable application info.
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:annotation>
    <xs:appinfo>app-data</xs:appinfo>
    <xs:documentation>human prose</xs:documentation>
  </xs:annotation>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let instance = project_from_xml_document(&doc);
    let ann = &instance.annotations[0];
    assert_eq!(ann.appinfo, vec!["app-data".to_string()]);
    assert_eq!(ann.documentation, vec!["human prose".to_string()]);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn projects_nested_annotation_on_element_declaration() {
    // §3.15 — annotations attach to any schema component. Here an
    // <xs:element> declaration carries its own <xs:annotation>.
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="foo">
    <xs:annotation>
      <xs:documentation>foo means bar</xs:documentation>
    </xs:annotation>
  </xs:element>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let instance = project_from_xml_document(&doc);
    // Element still projected.
    assert_eq!(instance.elements.len(), 1);
    assert_eq!(instance.elements[0].local_name, "foo");
    // Annotation captured.
    assert_eq!(instance.annotations.len(), 1);
    assert_eq!(
        instance.annotations[0].documentation,
        vec!["foo means bar".to_string()]
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn projects_real_uslm_xsd_annotations_and_imports() {
    // The bundled USLM 1.0.18 XSD has multiple <xs:annotation>
    // blocks and three <xs:import> directives (xml, dcterms, xhtml).
    let xsd_bytes = crate::formal::meta::xsd::uslm_vocabulary::loaded_uslm_1_0_18_xsd().as_bytes();
    let doc = parse_document(xsd_bytes).unwrap();
    let instance = project_from_xml_document(&doc);
    assert!(
        instance.annotations.len() >= 10,
        "USLM XSD has many annotations; got {}",
        instance.annotations.len()
    );
    let documentation_count: usize = instance
        .annotations
        .iter()
        .map(|a| a.documentation.len())
        .sum();
    assert!(
        documentation_count >= 10,
        "expected substantial documentation coverage; got {documentation_count}"
    );
}

// =============================================================================
// W3C XSD 1.1 Part 1 §3.4 type derivation + §3.8 model groups + §3.16 varieties.
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn projects_model_groups() {
    // §3.8 sequence / choice / all + §3.10 any wildcard.
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:complexType name="t">
    <xs:sequence>
      <xs:choice>
        <xs:element name="a" type="xs:string"/>
      </xs:choice>
      <xs:any/>
    </xs:sequence>
  </xs:complexType>
  <xs:complexType name="u">
    <xs:all/>
  </xs:complexType>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let instance = project_from_xml_document(&doc);
    assert!(instance.components.contains(&XsdConcept::Sequence));
    assert!(instance.components.contains(&XsdConcept::Choice));
    assert!(instance.components.contains(&XsdConcept::AllGroup));
    assert!(instance.components.contains(&XsdConcept::Wildcard));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn projects_complex_content_extension_with_base() {
    // §3.4.2 complexContent + §3.4.6 extension(base=...).
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:complexType name="derived">
    <xs:complexContent>
      <xs:extension base="BaseType">
        <xs:attribute name="extra" type="xs:string"/>
      </xs:extension>
    </xs:complexContent>
  </xs:complexType>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let instance = project_from_xml_document(&doc);
    assert!(instance.components.contains(&XsdConcept::ComplexContent));
    assert!(instance.components.contains(&XsdConcept::Extension));
    assert_eq!(instance.derivations.len(), 1);
    assert_eq!(instance.derivations[0].method, DerivationMethod::Extension);
    assert_eq!(instance.derivations[0].base.as_deref(), Some("BaseType"));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn projects_simple_content_restriction_with_base() {
    // §3.4.2 simpleContent + §3.4.6 restriction(base=...).
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:complexType name="r">
    <xs:simpleContent>
      <xs:restriction base="xs:string"/>
    </xs:simpleContent>
  </xs:complexType>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let instance = project_from_xml_document(&doc);
    assert!(instance.components.contains(&XsdConcept::SimpleContent));
    assert!(instance.components.contains(&XsdConcept::Restriction));
    assert_eq!(
        instance.derivations[0].method,
        DerivationMethod::Restriction
    );
    assert_eq!(instance.derivations[0].base.as_deref(), Some("xs:string"));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn projects_simple_type_list_and_union() {
    // §3.16 / Part 2 §4.1.2 — list + union varieties.
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:simpleType name="intlist">
    <xs:list itemType="xs:int"/>
  </xs:simpleType>
  <xs:simpleType name="strOrInt">
    <xs:union memberTypes="xs:string xs:int"/>
  </xs:simpleType>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let instance = project_from_xml_document(&doc);
    assert!(instance.components.contains(&XsdConcept::ListType));
    assert!(instance.components.contains(&XsdConcept::UnionType));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn projects_real_uslm_xsd_derivations() {
    // The bundled USLM XSD uses complexContent/simpleContent +
    // extension/restriction extensively for its type hierarchy.
    let xsd_bytes = crate::formal::meta::xsd::uslm_vocabulary::loaded_uslm_1_0_18_xsd().as_bytes();
    let doc = parse_document(xsd_bytes).unwrap();
    let instance = project_from_xml_document(&doc);
    assert!(
        instance.derivations.len() > 20,
        "USLM XSD derives many types; got {} derivations",
        instance.derivations.len()
    );
    assert!(
        instance.components.contains(&XsdConcept::Sequence),
        "USLM content models use xs:sequence"
    );
}

// =============================================================================
// W3C XSD 1.1 Part 2 §4.3 constraining facets.
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn projects_string_facets() {
    // §4.3.1–.6 — length/min/max/pattern/enumeration/whiteSpace.
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:simpleType name="t">
    <xs:restriction base="xs:string">
      <xs:length value="5"/>
      <xs:minLength value="1"/>
      <xs:maxLength value="9"/>
      <xs:pattern value="[a-z]+"/>
      <xs:enumeration value="x"/>
      <xs:whiteSpace value="collapse"/>
    </xs:restriction>
  </xs:simpleType>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let c = &project_from_xml_document(&doc).components;
    for facet in [
        XsdConcept::LengthFacet,
        XsdConcept::MinLengthFacet,
        XsdConcept::MaxLengthFacet,
        XsdConcept::PatternFacet,
        XsdConcept::EnumerationFacet,
        XsdConcept::WhiteSpaceFacet,
    ] {
        assert!(c.contains(&facet), "missing facet {facet:?}");
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn projects_numeric_range_and_digit_facets() {
    // §4.3.7–.12 — min/max inclusive/exclusive, total/fraction digits.
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:simpleType name="t">
    <xs:restriction base="xs:decimal">
      <xs:minInclusive value="0"/>
      <xs:maxInclusive value="100"/>
      <xs:minExclusive value="-1"/>
      <xs:maxExclusive value="101"/>
      <xs:totalDigits value="5"/>
      <xs:fractionDigits value="2"/>
    </xs:restriction>
  </xs:simpleType>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let c = &project_from_xml_document(&doc).components;
    for facet in [
        XsdConcept::MinInclusiveFacet,
        XsdConcept::MaxInclusiveFacet,
        XsdConcept::MinExclusiveFacet,
        XsdConcept::MaxExclusiveFacet,
        XsdConcept::TotalDigitsFacet,
        XsdConcept::FractionDigitsFacet,
    ] {
        assert!(c.contains(&facet), "missing facet {facet:?}");
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn projects_xsd11_facets() {
    // XSD 1.1 additions: §4.3.14 explicitTimezone, §4.3.13 assertion.
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:simpleType name="t">
    <xs:restriction base="xs:dateTime">
      <xs:explicitTimezone value="required"/>
      <xs:assertion test="true()"/>
    </xs:restriction>
  </xs:simpleType>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let c = &project_from_xml_document(&doc).components;
    assert!(c.contains(&XsdConcept::ExplicitTimezoneFacet));
    assert!(c.contains(&XsdConcept::AssertionFacet));
}

// =============================================================================
// W3C XSD 1.1 Part 1 §3.11 identity constraints.
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn projects_key_with_selector_and_field() {
    // §3.11.1 key + §3.11.2 selector/field.
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="root">
    <xs:key name="pk">
      <xs:selector xpath="row"/>
      <xs:field xpath="@id"/>
    </xs:key>
  </xs:element>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let c = &project_from_xml_document(&doc).components;
    assert!(c.contains(&XsdConcept::Key));
    assert!(c.contains(&XsdConcept::Selector));
    assert!(c.contains(&XsdConcept::Field));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn projects_keyref_and_unique() {
    // §3.11.1 keyref + unique.
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="root">
    <xs:unique name="u">
      <xs:selector xpath="a"/>
      <xs:field xpath="@x"/>
    </xs:unique>
    <xs:keyref name="fk" refer="pk">
      <xs:selector xpath="b"/>
      <xs:field xpath="@ref"/>
    </xs:keyref>
  </xs:element>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let c = &project_from_xml_document(&doc).components;
    assert!(c.contains(&XsdConcept::Unique));
    assert!(c.contains(&XsdConcept::KeyRef));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn projects_complex_type_assertion() {
    // §3.13 — `<xs:assert>` is an XSD 1.1 complex-type assertion.
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:complexType name="t">
    <xs:sequence><xs:element name="a" type="xs:int"/></xs:sequence>
    <xs:assert test="a &gt; 0"/>
  </xs:complexType>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let c = &project_from_xml_document(&doc).components;
    assert!(c.contains(&XsdConcept::Assert));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn projects_open_content_and_default_open_content() {
    // §3.4.2.2 openContent on a complex type + §3.16.2 schema-level
    // defaultOpenContent — both new in XSD 1.1.
    let xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:defaultOpenContent mode="interleave"><xs:any/></xs:defaultOpenContent>
  <xs:complexType name="t">
    <xs:openContent mode="suffix"><xs:any/></xs:openContent>
    <xs:sequence><xs:element name="a" type="xs:string"/></xs:sequence>
  </xs:complexType>
</xs:schema>"#;
    let doc = parse_document(xsd.as_bytes()).unwrap();
    let c = &project_from_xml_document(&doc).components;
    assert!(c.contains(&XsdConcept::OpenContent));
    assert!(c.contains(&XsdConcept::DefaultOpenContent));
}

#[pr4xis::praxis_value(Deterministic)]
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
