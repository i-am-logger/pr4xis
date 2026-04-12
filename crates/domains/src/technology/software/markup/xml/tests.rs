use pr4xis::category::Category;
use pr4xis::category::entity::Entity;
use pr4xis::category::validate::check_category_laws;

use super::ontology::*;
use super::reader;

// =============================================================================
// XML Category tests
// =============================================================================

#[test]
fn xml_category_laws() {
    check_category_laws::<XmlCategory>().unwrap();
}

#[test]
fn xml_has_10_node_kinds() {
    assert_eq!(XmlNodeKind::variants().len(), 10);
}

#[test]
fn document_contains_element() {
    let m = XmlCategory::morphisms();
    assert!(
        m.iter()
            .any(|c| c.parent == XmlNodeKind::Document && c.child == XmlNodeKind::Element)
    );
}

#[test]
fn element_contains_cdata() {
    let m = XmlCategory::morphisms();
    assert!(
        m.iter()
            .any(|c| c.parent == XmlNodeKind::Element && c.child == XmlNodeKind::CData)
    );
}

// =============================================================================
// XML Symbol tests
// =============================================================================

#[test]
fn xml_special_chars() {
    let chars = XmlSymbols::special_chars();
    assert!(chars.iter().any(|(c, _)| *c == '<'));
    assert!(chars.iter().any(|(c, _)| *c == '>'));
    assert!(chars.iter().any(|(c, _)| *c == '&'));
}

#[test]
fn xml_entities() {
    let entities = XmlSymbols::entities();
    assert_eq!(entities.len(), 5);
    assert!(entities.contains(&("&lt;", '<')));
    assert!(entities.contains(&("&amp;", '&')));
}

// =============================================================================
// XML Reader tests — reading through ontological understanding
// =============================================================================

#[test]
fn read_simple_element() {
    let doc = reader::read_xml("<root/>").unwrap();
    assert_eq!(doc.root.name.local, "root");
    assert!(doc.root.children.is_empty());
}

#[test]
fn read_element_with_text() {
    let doc = reader::read_xml("<greeting>hello world</greeting>").unwrap();
    assert_eq!(doc.root.name.local, "greeting");
    assert_eq!(doc.root.children.len(), 1);
    match &doc.root.children[0] {
        XmlNode::Text(t) => assert_eq!(t, "hello world"),
        _ => panic!("expected text node"),
    }
}

#[test]
fn read_element_with_attributes() {
    let doc = reader::read_xml(r#"<div class="main" id="top"/>"#).unwrap();
    assert_eq!(doc.root.attributes.len(), 2);
    assert_eq!(doc.root.attributes[0].name.local, "class");
    assert_eq!(doc.root.attributes[0].value, "main");
    assert_eq!(doc.root.attributes[1].name.local, "id");
    assert_eq!(doc.root.attributes[1].value, "top");
}

#[test]
fn read_nested_elements() {
    let xml = r#"<root><child1/><child2><grandchild/></child2></root>"#;
    let doc = reader::read_xml(xml).unwrap();
    assert_eq!(doc.root.children.len(), 2);
    if let XmlNode::Element(child2) = &doc.root.children[1] {
        assert_eq!(child2.name.local, "child2");
        assert_eq!(child2.children.len(), 1);
    } else {
        panic!("expected element");
    }
}

#[test]
fn read_with_xml_declaration() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?><root/>"#;
    let doc = reader::read_xml(xml).unwrap();
    assert_eq!(doc.version, "1.0");
    assert_eq!(doc.encoding, Some("UTF-8".into()));
}

#[test]
fn read_with_comment() {
    let xml = "<root><!-- a comment --><child/></root>";
    let doc = reader::read_xml(xml).unwrap();
    assert_eq!(doc.root.children.len(), 2);
    match &doc.root.children[0] {
        XmlNode::Comment(c) => assert_eq!(c, " a comment "),
        _ => panic!("expected comment"),
    }
}

#[test]
fn read_with_cdata() {
    let xml = "<root><![CDATA[<not>xml</not>]]></root>";
    let doc = reader::read_xml(xml).unwrap();
    match &doc.root.children[0] {
        XmlNode::CData(t) => assert_eq!(t, "<not>xml</not>"),
        _ => panic!("expected CDATA"),
    }
}

#[test]
fn read_entity_unescaping() {
    let xml = "<root>a &lt; b &amp; c</root>";
    let doc = reader::read_xml(xml).unwrap();
    assert_eq!(doc.root.children[0].text_content(), "a < b & c");
}

#[test]
fn read_namespace() {
    let xml = r#"<root xmlns:ns="http://example.com"><ns:child/></root>"#;
    let doc = reader::read_xml(xml).unwrap();
    assert!(doc.root.namespace.is_some());
    let ns = doc.root.namespace.as_ref().unwrap();
    assert_eq!(ns.prefix, Some("ns".into()));
    assert_eq!(ns.uri, "http://example.com");

    if let XmlNode::Element(child) = &doc.root.children[0] {
        assert_eq!(child.name.prefix, Some("ns".into()));
        assert_eq!(child.name.local, "child");
    }
}

#[test]
fn xml_to_markup_conversion() {
    let doc = reader::read_xml("<root><child>text</child></root>").unwrap();
    let markup = doc.to_markup();
    assert_eq!(markup.text_content(), "text");
}

#[test]
fn find_all_elements() {
    let xml = r#"<root><item id="1"/><group><item id="2"/></group><item id="3"/></root>"#;
    let doc = reader::read_xml(xml).unwrap();
    let items = doc.find_all("item");
    assert_eq!(items.len(), 3);
}

#[test]
fn read_error_unclosed_tag() {
    let result = reader::read_xml("<root><unclosed>");
    assert!(result.is_err());
}

#[test]
fn xml_name_qualified() {
    let name = XmlName::with_prefix("ns", "element");
    assert_eq!(name.qualified(), "ns:element");

    let simple = XmlName::new("element");
    assert_eq!(simple.qualified(), "element");
}

// =============================================================================
// Property-based tests
// =============================================================================

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_xml_node_kind() -> impl Strategy<Value = XmlNodeKind> {
        prop_oneof![
            Just(XmlNodeKind::Document),
            Just(XmlNodeKind::Element),
            Just(XmlNodeKind::Attribute),
            Just(XmlNodeKind::Text),
            Just(XmlNodeKind::CData),
            Just(XmlNodeKind::Comment),
            Just(XmlNodeKind::ProcessingInstruction),
            Just(XmlNodeKind::XmlDeclaration),
            Just(XmlNodeKind::DocType),
            Just(XmlNodeKind::Namespace),
        ]
    }

    proptest! {
        /// Identity composition is idempotent.
        #[test]
        fn prop_identity_idempotent(kind in arb_xml_node_kind()) {
            let id = XmlCategory::identity(&kind);
            let composed = XmlCategory::compose(&id, &id);
            prop_assert_eq!(composed, Some(id));
        }

        /// Element can contain all content node kinds.
        #[test]
        fn prop_element_contains_content(kind in arb_xml_node_kind()) {
            if matches!(kind, XmlNodeKind::Element | XmlNodeKind::Attribute | XmlNodeKind::Text
                | XmlNodeKind::CData | XmlNodeKind::Comment | XmlNodeKind::ProcessingInstruction
                | XmlNodeKind::Namespace) {
                let m = XmlCategory::morphisms();
                let expected = XmlContains { parent: XmlNodeKind::Element, child: kind };
                prop_assert!(m.contains(&expected),
                    "Element should contain {:?}", kind);
            }
        }

        /// Entity references round-trip correctly.
        #[test]
        fn prop_entities_roundtrip(_dummy in 0..1i32) {
            for (entity, ch) in XmlSymbols::entities() {
                let xml = format!("<r>{}</r>", entity);
                let doc = reader::read_xml(&xml).unwrap();
                let text = doc.root.children[0].text_content();
                prop_assert_eq!(text, ch.to_string());
            }
        }

        /// Self-closing elements have no children.
        #[test]
        fn prop_self_closing_empty(name in "[a-z]{1,10}") {
            let xml = format!("<{}/>", name);
            let doc = reader::read_xml(&xml).unwrap();
            prop_assert!(doc.root.children.is_empty());
            prop_assert_eq!(doc.root.name.local, name);
        }
    }
}
