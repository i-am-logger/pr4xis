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

// =============================================================================
// Property-based tests (proptest) — Layer 3 of the three-layer test depth
// per `feedback_high_test_coverage`.
// =============================================================================
//
// Each property names a Foster et al. 2007 §2.2 lens law and a W3C XML 1.0
// §-reference. proptest cases generate synthetic XmlDocument values built
// from the grammar's productions and check the law holds.

mod property {
    use super::*;
    use proptest::prelude::*;

    /// W3C XML 1.0 §2.3 production [4] / [4a] NameStartChar / NameChar —
    /// ASCII subset (the part of the grammar most likely to round-trip
    /// without ambient Unicode mapping concerns).
    fn arb_ascii_name() -> impl Strategy<Value = String> {
        proptest::collection::vec(any::<u8>().prop_map(|b| b as char), 1..16)
            .prop_filter("starts with NameStartChar, rest are NameChar", |chars| {
                if chars.is_empty() {
                    return false;
                }
                let first = chars[0];
                let starts_ok = first.is_ascii_alphabetic() || first == '_';
                let rest_ok = chars[1..]
                    .iter()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_'));
                starts_ok && rest_ok
            })
            .prop_map(|chars| chars.into_iter().collect())
    }

    /// W3C XML 1.0 §3.1 production [10] AttValue — characters allowed in
    /// an attribute value (excluding `<` and `&`, both must be escaped).
    fn arb_att_value() -> impl Strategy<Value = String> {
        proptest::collection::vec(
            any::<char>().prop_filter("XML 1.0 §2.2 Char minus '<' '&'", |c| {
                let cp = *c as u32;
                let in_char = matches!(cp, 0x9 | 0xA | 0xD | 0x20..=0xD7FF | 0xE000..=0xFFFD | 0x10000..=0x10FFFF);
                in_char && *c != '<' && *c != '&'
            }),
            0..32,
        )
        .prop_map(|chars| chars.into_iter().collect())
    }

    /// W3C XML 1.0 §2.4 production [14] CharData — element content
    /// text minus the markup delimiters `<` and `&`.
    fn arb_char_data() -> impl Strategy<Value = String> {
        proptest::collection::vec(
            any::<char>().prop_filter("XML 1.0 §2.4 CharData chars", |c| {
                let cp = *c as u32;
                let in_char = matches!(cp, 0x9 | 0xA | 0xD | 0x20..=0xD7FF | 0xE000..=0xFFFD | 0x10000..=0x10FFFF);
                in_char && *c != '<' && *c != '&'
            }),
            0..32,
        )
        .prop_map(|chars| {
            let s: String = chars.into_iter().collect();
            // CharData §2.4 forbids the literal sequence `]]>`. Strip it to
            // keep generation in-grammar.
            s.replace("]]>", "")
        })
    }

    /// Build a simple [`XmlDocument`] with one element + one attribute +
    /// optional text content. Sufficient for exercising the lens laws
    /// without recursive complexity.
    fn arb_simple_document() -> impl Strategy<Value = XmlDocument> {
        (
            arb_ascii_name(),
            arb_ascii_name(),
            arb_att_value(),
            arb_char_data(),
        )
            .prop_map(|(root_name, attr_name, attr_value, text)| {
                let mut children: Vec<XmlNode> = Vec::new();
                if !text.is_empty() {
                    children.push(XmlNode::Text(text));
                }
                XmlDocument {
                    version: "1.0".into(),
                    encoding: None,
                    root: XmlElement {
                        name: XmlName::new(&root_name),
                        namespace: None,
                        attributes: vec![XmlAttribute {
                            name: XmlName::new(&attr_name),
                            value: attr_value,
                        }],
                        children,
                    },
                }
            })
    }

    proptest! {
        /// Foster et al. 2007 §2.2 GetPut law:
        /// for any well-formed typed value `t`, `get(put(t)) = t`.
        ///
        /// We generate `XmlDocument` values via [`arb_simple_document`],
        /// serialize, re-parse, and assert structural equality.
        #[test]
        fn property_get_put_law(doc in arb_simple_document()) {
            let bytes = XmlLens::put(&doc).unwrap();
            let parsed = XmlLens::get(&bytes).unwrap();
            prop_assert_eq!(parsed, doc);
        }

        /// W3C XML 1.0 §3.1 well-formedness constraint: Element Type Match.
        /// Random open/close pairs that mismatch must be rejected by
        /// [`parse_document`].
        #[test]
        fn property_mismatched_tags_rejected(
            a in arb_ascii_name(),
            b in arb_ascii_name(),
        ) {
            prop_assume!(a != b);
            let xml = format!("<?xml version=\"1.0\"?><{a}></{b}>");
            let result = parse_document(xml.as_bytes());
            let is_mismatch = matches!(result, Err(XmlParseError::MismatchedTags { .. }));
            prop_assert!(is_mismatch, "expected MismatchedTags, got {result:?}");
        }

        /// W3C XML 1.0 §4.6 — for every one of the 5 predefined entity
        /// references, the parser MUST recognise and expand it to the
        /// corresponding character. We assert this on each entity in
        /// isolation; tested as a property to make the universally-
        /// quantified claim explicit.
        #[test]
        fn property_predefined_entities_always_expand(idx in 0usize..5) {
            let (entity, expected) = match idx {
                0 => ("amp", '&'),
                1 => ("lt", '<'),
                2 => ("gt", '>'),
                3 => ("apos", '\''),
                _ => ("quot", '"'),
            };
            let xml = format!("<?xml version=\"1.0\"?><r>&{entity};</r>");
            let doc = parse_document(xml.as_bytes()).unwrap();
            prop_assert_eq!(doc.root.children[0].clone(), XmlNode::Text(expected.to_string()));
        }

        /// Property: every legal CharData payload survives a put+get
        /// round-trip in element content (escape forms are normalised
        /// to the original code points). This covers the inverse of
        /// `write_escaped_char_data` (W3C XML 1.0 §4.6 + C14N 1.1 §3.5).
        #[test]
        fn property_char_data_round_trip(text in arb_char_data()) {
            let doc = XmlDocument {
                version: "1.0".into(),
                encoding: None,
                root: XmlElement {
                    name: XmlName::new("r"),
                    namespace: None,
                    attributes: Vec::new(),
                    children: if text.is_empty() {
                        Vec::new()
                    } else {
                        vec![XmlNode::Text(text.clone())]
                    },
                },
            };
            let bytes = XmlLens::put(&doc).unwrap();
            let parsed = XmlLens::get(&bytes).unwrap();
            if text.is_empty() {
                prop_assert!(parsed.root.children.is_empty());
            } else {
                prop_assert_eq!(parsed.root.children[0].clone(), XmlNode::Text(text));
            }
        }

        /// Property: every legal AttValue payload survives a put+get
        /// round-trip on an attribute. Covers `write_escaped_attr_value`
        /// (W3C XML 1.0 §3.3.3 attribute-value normalization).
        #[test]
        fn property_attr_value_round_trip(value in arb_att_value()) {
            let doc = XmlDocument {
                version: "1.0".into(),
                encoding: None,
                root: XmlElement {
                    name: XmlName::new("r"),
                    namespace: None,
                    attributes: vec![XmlAttribute {
                        name: XmlName::new("a"),
                        value: value.clone(),
                    }],
                    children: Vec::new(),
                },
            };
            let bytes = XmlLens::put(&doc).unwrap();
            let parsed = XmlLens::get(&bytes).unwrap();
            prop_assert_eq!(parsed.root.attributes[0].value.clone(), value);
        }
    }
}
