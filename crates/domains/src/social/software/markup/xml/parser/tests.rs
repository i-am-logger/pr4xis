//! Tests for the literature-grounded XML 1.0 parser + serializer
//! + lens pair.

#[allow(unused_imports)]
use alloc::{string::String, string::ToString, vec, vec::Vec};

use super::super::ontology::{
    XmlAttribute, XmlDocument, XmlElement, XmlName, XmlNamespace, XmlNode,
};
use super::grammar::{XmlParseError, parse_document};
use super::lens::XmlLens;
use crate::formal::meta::well_behaved_lens::WellBehavedLens;

#[test]
fn parses_minimal_empty_element() {
    let xml = r#"<?xml version="1.0"?><root/>"#;
    let doc = parse_document(xml.as_bytes()).unwrap();
    assert_eq!(doc.version, "1.0");
    assert_eq!(doc.encoding, None);
    assert_eq!(doc.root.name, XmlName::new("root"));
    assert!(doc.root.children.is_empty());
}

#[test]
fn parses_element_with_text_content() {
    let xml = r#"<?xml version="1.0"?><greeting>hello</greeting>"#;
    let doc = parse_document(xml.as_bytes()).unwrap();
    assert_eq!(doc.root.children.len(), 1);
    assert_eq!(doc.root.children[0], XmlNode::Text("hello".into()));
}

#[test]
fn parses_nested_elements() {
    let xml = r#"<?xml version="1.0"?><a><b><c>x</c></b></a>"#;
    let doc = parse_document(xml.as_bytes()).unwrap();
    let b = match &doc.root.children[0] {
        XmlNode::Element(el) => el,
        other => panic!("expected element, got {other:?}"),
    };
    assert_eq!(b.name, XmlName::new("b"));
    let c = match &b.children[0] {
        XmlNode::Element(el) => el,
        other => panic!("expected element, got {other:?}"),
    };
    assert_eq!(c.name, XmlName::new("c"));
    assert_eq!(c.children[0], XmlNode::Text("x".into()));
}

#[test]
fn parses_attributes() {
    let xml = r#"<?xml version="1.0"?><e a="1" b="two"/>"#;
    let doc = parse_document(xml.as_bytes()).unwrap();
    assert_eq!(
        doc.root.attributes,
        vec![
            XmlAttribute {
                name: XmlName::new("a"),
                value: "1".into(),
            },
            XmlAttribute {
                name: XmlName::new("b"),
                value: "two".into(),
            },
        ]
    );
}

#[test]
fn parses_default_namespace_declaration() {
    let xml = r#"<?xml version="1.0"?><root xmlns="http://example.org/ns"/>"#;
    let doc = parse_document(xml.as_bytes()).unwrap();
    assert_eq!(
        doc.root.namespace,
        Some(XmlNamespace {
            prefix: None,
            uri: "http://example.org/ns".into(),
        })
    );
    assert!(doc.root.attributes.is_empty(), "xmlns is not an attribute");
}

#[test]
fn parses_prefixed_namespace_declaration() {
    let xml = r#"<?xml version="1.0"?><root xmlns:dc="http://purl.org/dc/"/>"#;
    let doc = parse_document(xml.as_bytes()).unwrap();
    assert_eq!(
        doc.root.namespace,
        Some(XmlNamespace {
            prefix: Some("dc".into()),
            uri: "http://purl.org/dc/".into(),
        })
    );
}

#[test]
fn expands_predefined_entities_in_text() {
    // W3C XML 1.0 §4.6 predefined entities.
    let xml = r#"<?xml version="1.0"?><r>a &amp; b &lt; c &gt; d &apos;e&apos; &quot;f&quot;</r>"#;
    let doc = parse_document(xml.as_bytes()).unwrap();
    assert_eq!(
        doc.root.children[0],
        XmlNode::Text("a & b < c > d 'e' \"f\"".into())
    );
}

#[test]
fn expands_numeric_character_references() {
    let xml = r#"<?xml version="1.0"?><r>&#65;&#x42;</r>"#;
    let doc = parse_document(xml.as_bytes()).unwrap();
    assert_eq!(doc.root.children[0], XmlNode::Text("AB".into()));
}

#[test]
fn rejects_undeclared_entity() {
    let xml = r#"<?xml version="1.0"?><r>&unknown;</r>"#;
    match parse_document(xml.as_bytes()) {
        Err(XmlParseError::UnsupportedEntity { name, .. }) => assert_eq!(name, "unknown"),
        other => panic!("expected UnsupportedEntity, got {other:?}"),
    }
}

#[test]
fn parses_cdata_section() {
    let xml = r#"<?xml version="1.0"?><r><![CDATA[<not> &an; entity]]></r>"#;
    let doc = parse_document(xml.as_bytes()).unwrap();
    assert_eq!(
        doc.root.children[0],
        XmlNode::CData("<not> &an; entity".into())
    );
}

#[test]
fn parses_comment_inside_element() {
    let xml = r#"<?xml version="1.0"?><r>before<!-- mid -->after</r>"#;
    let doc = parse_document(xml.as_bytes()).unwrap();
    assert_eq!(
        doc.root.children,
        vec![
            XmlNode::Text("before".into()),
            XmlNode::Comment(" mid ".into()),
            XmlNode::Text("after".into()),
        ]
    );
}

#[test]
fn parses_processing_instruction_inside_element() {
    let xml = r#"<?xml version="1.0"?><r>x<?stylesheet href="a.css"?>y</r>"#;
    let doc = parse_document(xml.as_bytes()).unwrap();
    assert_eq!(
        doc.root.children,
        vec![
            XmlNode::Text("x".into()),
            XmlNode::ProcessingInstruction {
                target: "stylesheet".into(),
                data: Some("href=\"a.css\"".into()),
            },
            XmlNode::Text("y".into()),
        ]
    );
}

#[test]
fn detects_mismatched_tags() {
    let xml = r#"<?xml version="1.0"?><a></b>"#;
    match parse_document(xml.as_bytes()) {
        Err(XmlParseError::MismatchedTags { open, close, .. }) => {
            assert_eq!(open, "a");
            assert_eq!(close, "b");
        }
        other => panic!("expected MismatchedTags, got {other:?}"),
    }
}

#[test]
fn skips_doctype_with_internal_subset() {
    let xml = r#"<?xml version="1.0"?><!DOCTYPE foo [<!ELEMENT bar (#PCDATA)>]><foo/>"#;
    let doc = parse_document(xml.as_bytes()).unwrap();
    assert_eq!(doc.root.name, XmlName::new("foo"));
}

// =============================================================================
// Round-trip / lens-law tests (Foster et al. 2007 §2.2).
// =============================================================================

#[test]
fn round_trip_minimal_empty_element() {
    let xml = b"<?xml version=\"1.0\"?><root/>";
    let parsed = XmlLens::get(xml).unwrap();
    let serialized = XmlLens::put(&parsed).unwrap();
    assert_eq!(serialized, xml);
}

#[test]
fn round_trip_with_attributes_and_namespace() {
    let xml = b"<?xml version=\"1.0\"?><root xmlns=\"http://x\" a=\"1\"/>";
    let parsed = XmlLens::get(xml).unwrap();
    let serialized = XmlLens::put(&parsed).unwrap();
    assert_eq!(serialized, xml);
}

#[test]
fn get_put_law_holds_for_simple_doc() {
    // GetPut: parse, re-serialize, re-parse — typed value matches.
    let xml = b"<?xml version=\"1.0\"?><a><b>hi</b><c x=\"1\"/></a>";
    let t1 = XmlLens::get(xml).unwrap();
    let bytes2 = XmlLens::put(&t1).unwrap();
    let t2 = XmlLens::get(&bytes2).unwrap();
    assert_eq!(t1, t2);
}

#[test]
fn put_get_law_holds_for_simple_doc() {
    // PutGet: canonical(put(get(s))) == canonical(s).
    let xml = b"<?xml version=\"1.0\"?><a><b>hi</b></a>";
    let t = XmlLens::get(xml).unwrap();
    let round = XmlLens::put(&t).unwrap();
    let canonical_source = XmlLens::canonical(xml).unwrap();
    let canonical_round = XmlLens::canonical(&round).unwrap();
    assert_eq!(canonical_source, canonical_round);
}

#[test]
fn put_get_law_with_entities_normalises_to_canonical_form() {
    // The literal `>` in input gets escaped to `&gt;` on put. C14N
    // 1.1 §3.5 then canonicalises both forms to the same byte
    // sequence, so PutGet still holds.
    let xml = b"<?xml version=\"1.0\"?><r>a &gt; b</r>";
    let t = XmlLens::get(xml).unwrap();
    let round = XmlLens::put(&t).unwrap();
    let canonical_source = XmlLens::canonical(xml).unwrap();
    let canonical_round = XmlLens::canonical(&round).unwrap();
    assert_eq!(canonical_source, canonical_round);
}

#[test]
fn lens_serializes_synthesised_typed_value() {
    // Hand-build an XmlDocument and verify the serializer's output
    // re-parses to the same typed value (Foster et al. 2007 GetPut).
    let doc = XmlDocument {
        version: "1.0".into(),
        encoding: None,
        root: XmlElement {
            name: XmlName::new("synth"),
            namespace: None,
            attributes: vec![XmlAttribute {
                name: XmlName::new("kind"),
                value: "test".into(),
            }],
            children: vec![XmlNode::Text("inner".into())],
        },
    };
    let bytes = XmlLens::put(&doc).unwrap();
    let parsed = XmlLens::get(&bytes).unwrap();
    assert_eq!(parsed, doc);
}
