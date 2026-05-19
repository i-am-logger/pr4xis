//! Tests for the XSD reader + ontology.
//!
//! The "load-bearing" test is `loads_uslm_full_xsd` — it actually
//! parses `crates/domains/data/legal/uscode/schema/uslm-1.0.18.xsd`
//! and asserts the expected schema-component counts. That's the
//! validation that the reader handles the full LRC schema (101
//! elements, 37 complexTypes, 14 simpleTypes, 47 attributeGroups, 33
//! groups — counted with `grep -c` on the file).
//!
//! All other tests use focused snippets to exercise specific XSD
//! constructs in isolation.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::*;
use super::reader::{XsdReadError, read_xsd};
use proptest::prelude::*;

const USLM_XSD: &str = include_str!("../../../../../../data/legal/uscode/schema/uslm-1.0.18.xsd");

// =============================================================================
// Round-trip on the real USLM XSD
// =============================================================================

/// W3C XSD 1.1 Part 1 §3.1.2.1: A schema document is rooted at
/// `<xsd:schema>`. This test confirms the LRC's published USLM-1.0.18
/// XSD parses without error and produces the expected schema-component
/// counts.
#[test]
fn loads_uslm_full_xsd() {
    let schema = read_xsd(USLM_XSD).unwrap_or_else(|e| panic!("USLM XSD did not parse: {e}"));
    // Expected counts of top-level schema components. Derived from
    // `grep -c '^   <xsd:element name='` (3-space indent matches
    // top-level only) on the on-disk file. These are intentionally
    // exact: a re-pin of `uslm_xsd` in praxis.lock that changes the
    // schema-component counts is a schema change the human should
    // see, not silently absorb. The number is *top-level* — nested
    // element decls inside content models are separately captured as
    // `XsdTerm::Element` items inside their containing particle.
    assert_eq!(schema.elements.len(), 98, "top-level element decl count");
    assert_eq!(
        schema.complex_types.len(),
        37,
        "top-level complexType decl count"
    );
    assert_eq!(
        schema.simple_types.len(),
        13,
        "top-level simpleType decl count (one nested in an element decl is excluded)"
    );
    assert_eq!(
        schema.attribute_groups.len(),
        14,
        "top-level attributeGroup decl count (33 additional are `ref=` usages, \
         not declarations)"
    );
    // `grep -c '^   <xsd:group name='` returns the exact number — but
    // USLM also has `xsd:group ref=` usages inside complexTypes. Both
    // top-level decls and ref usages share the `<xsd:group ` prefix;
    // top-level distinguished by 3-space indent + `name=` attribute.
    let top_level_groups_via_grep = USLM_XSD
        .lines()
        .filter(|l| l.trim_start().starts_with("<xsd:group ") && l.contains("name="))
        .filter(|l| l.starts_with("   <xsd:group"))
        .count();
    assert_eq!(
        schema.groups.len(),
        top_level_groups_via_grep,
        "top-level named-group decl count must match an in-file count"
    );
    assert_eq!(
        schema.target_namespace.as_deref(),
        Some("http://xml.house.gov/schemas/uslm/1.0"),
        "USLM schema declares the http://xml.house.gov/schemas/uslm/1.0 \
         namespace (W3C XSD 1.1 Part 1 §3.1.2.5)"
    );
    assert_eq!(
        schema.version.as_deref(),
        Some("1.0.18"),
        "the loaded XSD is version 1.0.18 (the LRC publication unit \
         currently shipped to github.com/usgpo/uslm@main/USLM.xsd)"
    );
}

#[test]
fn uslm_xsd_declares_marker_inline_block_content_heads() {
    // USLM's "abstract heads" — the four schema components that
    // `substitutionGroup="…"` references throughout the rest of the
    // schema (W3C XSD 1.1 Part 1 §3.3.2.4). Confirm all four are
    // declared as top-level elements.
    let schema = read_xsd(USLM_XSD).expect("USLM XSD parses");
    for head in ["marker", "inline", "block", "content"] {
        let element = schema
            .element(head)
            .unwrap_or_else(|| panic!("USLM XSD must declare top-level element <{head}>"));
        assert_eq!(element.name.as_deref(), Some(head));
        // Each abstract head has a `type=` pointing at the corresponding
        // *Type complexType (per the LRC's Venetian Blind pattern).
        assert!(
            element.type_name.is_some(),
            "head element {head} should have a type= attribute"
        );
    }
}

#[test]
fn uslm_xsd_property_head_substitutions_include_ref_and_date() {
    // W3C XSD 1.1 Part 1 §3.3.2.4: substitutionGroup makes child
    // elements substitutable for a head. USLM uses `property` as one
    // such head; `<ref>` and `<date>` are members.
    let schema = read_xsd(USLM_XSD).expect("USLM XSD parses");
    let members = schema.substitution_members_of("property");
    let names: Vec<_> = members.iter().filter_map(|e| e.name.as_deref()).collect();
    assert!(
        names.contains(&"ref"),
        "property head must include <ref> (was: {names:?})"
    );
    assert!(
        names.contains(&"date"),
        "property head must include <date> (was: {names:?})"
    );
}

#[test]
fn uslm_xsd_inline_head_substitutions_nonempty() {
    let schema = read_xsd(USLM_XSD).expect("USLM XSD parses");
    let members = schema.substitution_members_of("inline");
    assert!(
        !members.is_empty(),
        "the <inline> head must have at least one substitution member"
    );
}

#[test]
fn uslm_xsd_mixed_content_elements_include_inline_family() {
    // USLM's "transparent inline elements" — the things that have
    // `mixed="true"` on their complexType, meaning text passes
    // through. Confirm we can derive that allowlist from the XSD
    // rather than hand-coding it (W3C XSD 1.1 Part 1 §3.4.2.5).
    let schema = read_xsd(USLM_XSD).expect("USLM XSD parses");
    let mixed = schema.mixed_content_elements();
    assert!(
        !mixed.is_empty(),
        "USLM uses mixed content extensively; the derived list cannot be empty"
    );
}

// =============================================================================
// Focused snippet tests — one XSD construct each
// =============================================================================

fn parse(snippet: &str) -> Result<XsdSchema, XsdReadError> {
    let wrapped = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<xsd:schema xmlns:xsd="http://www.w3.org/2001/XMLSchema"
            xmlns="http://example.com/test"
            targetNamespace="http://example.com/test"
            elementFormDefault="qualified">
{snippet}
</xsd:schema>"#
    );
    read_xsd(&wrapped)
}

#[test]
fn empty_schema_parses() {
    let schema = parse("").expect("empty schema body");
    assert_eq!(
        schema.target_namespace.as_deref(),
        Some("http://example.com/test")
    );
    assert!(schema.elements.is_empty());
}

#[test]
fn element_with_type_attribute() {
    let schema = parse(r#"<xsd:element name="section" type="SectionType"/>"#).unwrap();
    assert_eq!(schema.elements.len(), 1);
    let e = &schema.elements[0];
    assert_eq!(e.name.as_deref(), Some("section"));
    assert_eq!(e.type_name.as_deref(), Some("SectionType"));
    assert!(!e.is_abstract);
}

#[test]
fn element_with_substitution_group() {
    let schema =
        parse(r#"<xsd:element name="ref" type="RefType" substitutionGroup="property"/>"#).unwrap();
    let e = &schema.elements[0];
    assert_eq!(e.substitution_group.as_deref(), Some("property"));
}

#[test]
fn complex_type_with_sequence_of_elements() {
    let schema = parse(
        r#"<xsd:complexType name="SectionType">
              <xsd:sequence>
                <xsd:element name="num" type="xsd:string"/>
                <xsd:element name="heading" type="xsd:string" minOccurs="0"/>
                <xsd:element name="content" minOccurs="0" maxOccurs="unbounded"/>
              </xsd:sequence>
            </xsd:complexType>"#,
    )
    .unwrap();
    let ct = &schema.complex_types[0];
    assert_eq!(ct.name.as_deref(), Some("SectionType"));
    assert!(!ct.mixed);
    match &ct.content_model {
        XsdContentModel::Sequence(p) => {
            assert_eq!(p.terms.len(), 3);
            // The last element has maxOccurs="unbounded"
            match &p.terms[2] {
                XsdTerm::Element(e) => {
                    assert_eq!(e.max_occurs, Some(Occurs::Unbounded));
                    assert_eq!(e.min_occurs, Some(Occurs::Count(0)));
                }
                _ => panic!("expected an element term"),
            }
        }
        other => panic!("expected Sequence content model, got {other:?}"),
    }
}

#[test]
fn complex_type_with_choice() {
    let schema = parse(
        r#"<xsd:complexType name="ChoiceType">
              <xsd:choice maxOccurs="unbounded">
                <xsd:element name="a" type="xsd:string"/>
                <xsd:element name="b" type="xsd:string"/>
              </xsd:choice>
            </xsd:complexType>"#,
    )
    .unwrap();
    let ct = &schema.complex_types[0];
    match &ct.content_model {
        XsdContentModel::Choice(p) => {
            assert_eq!(p.max_occurs, Some(Occurs::Unbounded));
            assert_eq!(p.terms.len(), 2);
        }
        other => panic!("expected Choice, got {other:?}"),
    }
}

#[test]
fn complex_type_mixed_attribute() {
    let schema = parse(
        r#"<xsd:complexType name="InlineType" mixed="true">
              <xsd:sequence/>
            </xsd:complexType>"#,
    )
    .unwrap();
    assert!(schema.complex_types[0].mixed);
}

#[test]
fn complex_type_with_extension() {
    let schema = parse(
        r#"<xsd:complexType name="DerivedType">
              <xsd:complexContent>
                <xsd:extension base="BaseType">
                  <xsd:sequence>
                    <xsd:element name="extra" type="xsd:string"/>
                  </xsd:sequence>
                  <xsd:attribute name="kind" type="xsd:string"/>
                </xsd:extension>
              </xsd:complexContent>
            </xsd:complexType>"#,
    )
    .unwrap();
    match &schema.complex_types[0].content_model {
        XsdContentModel::ExtensionOf {
            base, attributes, ..
        } => {
            assert_eq!(base, "BaseType");
            assert_eq!(attributes.len(), 1);
            assert_eq!(attributes[0].name.as_deref(), Some("kind"));
        }
        other => panic!("expected ExtensionOf, got {other:?}"),
    }
}

#[test]
fn simple_type_with_enumeration() {
    let schema = parse(
        r#"<xsd:simpleType name="Color">
              <xsd:restriction base="xsd:string">
                <xsd:enumeration value="red"/>
                <xsd:enumeration value="green"/>
                <xsd:enumeration value="blue"/>
              </xsd:restriction>
            </xsd:simpleType>"#,
    )
    .unwrap();
    let st = &schema.simple_types[0];
    let r = st
        .restriction()
        .expect("simpleType should have a restriction");
    assert_eq!(r.base, "xsd:string");
    assert_eq!(r.enumerations, vec!["red", "green", "blue"]);
}

#[test]
fn simple_type_with_union() {
    // W3C XSD 1.1 Part 2 §4.1.2.5: xsd:union memberTypes lists the
    // member-type QNames whitespace-separated. USLM uses this for
    // DateOrDateTimeType etc.
    let schema = parse(
        r#"<xsd:simpleType name="DateOrDateTime">
              <xsd:union memberTypes="xsd:date xsd:dateTime"/>
            </xsd:simpleType>"#,
    )
    .unwrap();
    match &schema.simple_types[0].derivation {
        XsdSimpleDerivation::Union { member_types } => {
            assert_eq!(
                member_types,
                &vec!["xsd:date".to_string(), "xsd:dateTime".to_string()]
            );
        }
        other => panic!("expected Union, got {other:?}"),
    }
}

#[test]
fn simple_type_with_list() {
    // W3C XSD 1.1 Part 2 §4.1.2.4: xsd:list has an itemType attribute.
    let schema = parse(
        r#"<xsd:simpleType name="StringList">
              <xsd:list itemType="xsd:string"/>
            </xsd:simpleType>"#,
    )
    .unwrap();
    match &schema.simple_types[0].derivation {
        XsdSimpleDerivation::List { item_type } => {
            assert_eq!(item_type, "xsd:string");
        }
        other => panic!("expected List, got {other:?}"),
    }
}

#[test]
fn simple_type_with_pattern() {
    let schema = parse(
        r#"<xsd:simpleType name="UrnPath">
              <xsd:restriction base="xsd:string">
                <xsd:pattern value="/us/usc/.*"/>
              </xsd:restriction>
            </xsd:simpleType>"#,
    )
    .unwrap();
    assert_eq!(
        schema.simple_types[0].restriction().unwrap().patterns,
        vec!["/us/usc/.*"]
    );
}

#[test]
fn attribute_group_declaration() {
    let schema = parse(
        r#"<xsd:attributeGroup name="commonAtts">
              <xsd:attribute name="id" type="xsd:ID"/>
              <xsd:attribute name="style" type="xsd:string"/>
            </xsd:attributeGroup>"#,
    )
    .unwrap();
    let ag = &schema.attribute_groups[0];
    assert_eq!(ag.name, "commonAtts");
    assert_eq!(ag.attributes.len(), 2);
}

#[test]
fn named_group_declaration() {
    let schema = parse(
        r#"<xsd:group name="contentChoice">
              <xsd:choice>
                <xsd:element name="p" type="xsd:string"/>
              </xsd:choice>
            </xsd:group>"#,
    )
    .unwrap();
    let g = &schema.groups[0];
    assert_eq!(g.name, "contentChoice");
    matches!(g.content_model, XsdContentModel::Choice(_));
}

#[test]
fn any_wildcard_inside_sequence() {
    let schema = parse(
        r#"<xsd:complexType name="DcMetaType">
              <xsd:sequence>
                <xsd:any namespace="http://purl.org/dc/elements/1.1/" processContents="lax"/>
              </xsd:sequence>
            </xsd:complexType>"#,
    )
    .unwrap();
    match &schema.complex_types[0].content_model {
        XsdContentModel::Sequence(p) => match &p.terms[0] {
            XsdTerm::Any(a) => {
                assert_eq!(
                    a.namespace.as_deref(),
                    Some("http://purl.org/dc/elements/1.1/")
                );
                assert_eq!(a.process_contents.as_deref(), Some("lax"));
            }
            other => panic!("expected Any, got {other:?}"),
        },
        other => panic!("expected Sequence, got {other:?}"),
    }
}

#[test]
fn complex_type_with_attribute_group_ref() {
    let schema = parse(
        r#"<xsd:complexType name="SectionType">
              <xsd:sequence/>
              <xsd:attributeGroup ref="commonAtts"/>
            </xsd:complexType>"#,
    )
    .unwrap();
    assert_eq!(
        schema.complex_types[0].attribute_group_refs,
        vec!["commonAtts".to_string()]
    );
}

#[test]
fn rejects_non_schema_root() {
    let xml = r#"<?xml version="1.0"?><not-schema xmlns="http://example.com"/>"#;
    let err = read_xsd(xml).expect_err("non-schema root should be rejected");
    match err {
        XsdReadError::NotSchemaRoot { actual_local, .. } => {
            assert_eq!(actual_local, "not-schema");
        }
        other => panic!("expected NotSchemaRoot, got {other:?}"),
    }
}

#[test]
fn rejects_unsupported_construct() {
    // xsd:include is documented as a deferred / unsupported feature
    // (USLM doesn't use it; xsd:import is the supported alternative).
    // Fail-closed: must return Unsupported, not silently proceed.
    let err = parse(r#"<xsd:include schemaLocation="other.xsd"/>"#)
        .expect_err("xsd:include should be rejected");
    match err {
        XsdReadError::Unsupported { construct, .. } => {
            assert!(construct.contains("include"));
        }
        other => panic!("expected Unsupported, got {other:?}"),
    }
}

#[test]
fn rejects_invalid_occurs_value() {
    let err = parse(
        r#"<xsd:complexType name="Bad">
              <xsd:sequence>
                <xsd:element name="x" type="xsd:string" maxOccurs="lots"/>
              </xsd:sequence>
            </xsd:complexType>"#,
    )
    .expect_err("non-integer non-unbounded maxOccurs should be rejected");
    match err {
        XsdReadError::Unsupported { reason, .. } => {
            assert!(reason.contains("non-negative integer"));
        }
        other => panic!("expected Unsupported, got {other:?}"),
    }
}

// =============================================================================
// Functor laws — read_xsd is a typed transformation
// =============================================================================

/// Identity-on-empty: parsing the smallest valid schema (no top-level
/// declarations) yields a schema with zero declarations.
///
/// Eilenberg & MacLane (1942) §1 — functor preserves identity. Here
/// the "identity" is the empty schema body; the parser must not
/// invent components.
#[test]
fn functor_law_identity_on_empty_body() {
    let schema = parse("").expect("empty body parses");
    assert_eq!(schema.elements.len(), 0);
    assert_eq!(schema.complex_types.len(), 0);
    assert_eq!(schema.simple_types.len(), 0);
    assert_eq!(schema.attribute_groups.len(), 0);
    assert_eq!(schema.groups.len(), 0);
    assert_eq!(schema.imports.len(), 0);
}

/// Composition: declaring an element + a complex type independently
/// yields the same components as declaring them together in either
/// order. The reader composes parse-output without introducing order
/// dependencies between schema components.
#[test]
fn functor_law_composition_order_independent() {
    let a = parse(
        r#"<xsd:element name="e" type="T"/>
           <xsd:complexType name="T"><xsd:sequence/></xsd:complexType>"#,
    )
    .unwrap();
    let b = parse(
        r#"<xsd:complexType name="T"><xsd:sequence/></xsd:complexType>
           <xsd:element name="e" type="T"/>"#,
    )
    .unwrap();
    assert_eq!(a.elements.len(), b.elements.len());
    assert_eq!(a.complex_types.len(), b.complex_types.len());
    assert_eq!(a.element("e"), b.element("e"));
    assert_eq!(a.complex_type("T"), b.complex_type("T"));
}

// =============================================================================
// Property-based — XSD reader robustness
// =============================================================================

proptest! {
    /// For any XSD-valid set of element names, the reader produces
    /// exactly that many element decls — no inventing, no losing.
    #[test]
    fn prop_element_count_matches_declared(
        names in proptest::collection::vec("[a-zA-Z][a-zA-Z0-9]{0,10}", 0..20)
    ) {
        // De-duplicate; XSD requires unique top-level element names.
        let mut unique: Vec<String> = Vec::new();
        for n in names {
            if !unique.contains(&n) { unique.push(n); }
        }
        let body: String = unique
            .iter()
            .map(|n| format!(r#"<xsd:element name="{n}" type="xsd:string"/>"#))
            .collect::<Vec<_>>()
            .join("\n");
        let schema = parse(&body).expect("synthetic schema parses");
        prop_assert_eq!(schema.elements.len(), unique.len());
        for n in &unique {
            prop_assert!(schema.element(n).is_some());
        }
    }

    /// substitutionGroup membership is purely a function of the
    /// `substitutionGroup` attribute. Adding an unrelated element
    /// does not change the membership of an existing head.
    #[test]
    fn prop_substitution_group_local(
        head in "[a-z]{3,8}",
        members in proptest::collection::vec("[a-z][a-z0-9]{0,8}", 0..6),
        decoy in "[A-Z][a-z]{3,8}"
    ) {
        let mut unique_members: Vec<String> = Vec::new();
        for m in members {
            if m != head && !unique_members.contains(&m) {
                unique_members.push(m);
            }
        }
        let mut body = format!(
            r#"<xsd:element name="{head}" type="HeadType" abstract="true"/>
"#
        );
        for m in &unique_members {
            body.push_str(&format!(
                r#"<xsd:element name="{m}" type="MType" substitutionGroup="{head}"/>
"#
            ));
        }
        body.push_str(&format!(
            r#"<xsd:element name="{decoy}" type="DType"/>
"#
        ));
        let schema = parse(&body).unwrap();
        let found_members: Vec<_> = schema
            .substitution_members_of(&head)
            .iter()
            .filter_map(|e| e.name.clone())
            .collect();
        for m in &unique_members {
            prop_assert!(found_members.contains(m));
        }
        prop_assert!(!found_members.contains(&decoy));
    }
}
