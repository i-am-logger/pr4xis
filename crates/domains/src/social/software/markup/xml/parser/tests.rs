//! Tests for the literature-grounded XML 1.0 parser + serializer
//! + lens pair.

#[allow(unused_imports)]
use alloc::{string::String, string::ToString, vec, vec::Vec};

use super::super::ontology::{
    XmlAttribute, XmlDocument, XmlElement, XmlEntityKind, XmlExternalId, XmlName, XmlNamespace,
    XmlNode,
};
use super::grammar::{XmlParseError, parse_document};
use super::lens::XmlLens;
use crate::formal::meta::well_behaved_lens::WellBehavedLens;

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parses_minimal_empty_element() {
    let xml = r#"<?xml version="1.0"?><root/>"#;
    let doc = parse_document(xml.as_bytes()).unwrap();
    assert_eq!(doc.version, "1.0");
    assert_eq!(doc.encoding, None);
    assert_eq!(doc.root.name, XmlName::new("root"));
    assert!(doc.root.children.is_empty());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parses_element_with_text_content() {
    let xml = r#"<?xml version="1.0"?><greeting>hello</greeting>"#;
    let doc = parse_document(xml.as_bytes()).unwrap();
    assert_eq!(doc.root.children.len(), 1);
    assert_eq!(doc.root.children[0], XmlNode::Text("hello".into()));
}

#[pr4xis::praxis_value(Verifiable)]
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

#[pr4xis::praxis_value(Verifiable)]
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

#[pr4xis::praxis_value(Verifiable)]
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

#[pr4xis::praxis_value(Verifiable)]
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

#[pr4xis::praxis_value(Verifiable)]
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

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn expands_numeric_character_references() {
    let xml = r#"<?xml version="1.0"?><r>&#65;&#x42;</r>"#;
    let doc = parse_document(xml.as_bytes()).unwrap();
    assert_eq!(doc.root.children[0], XmlNode::Text("AB".into()));
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn rejects_undeclared_entity() {
    let xml = r#"<?xml version="1.0"?><r>&unknown;</r>"#;
    match parse_document(xml.as_bytes()) {
        Err(XmlParseError::UnsupportedEntity { name, .. }) => assert_eq!(name, "unknown"),
        other => panic!("expected UnsupportedEntity, got {other:?}"),
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parses_cdata_section() {
    let xml = r#"<?xml version="1.0"?><r><![CDATA[<not> &an; entity]]></r>"#;
    let doc = parse_document(xml.as_bytes()).unwrap();
    assert_eq!(
        doc.root.children[0],
        XmlNode::CData("<not> &an; entity".into())
    );
}

#[pr4xis::praxis_value(Verifiable)]
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

#[pr4xis::praxis_value(Verifiable)]
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

#[pr4xis::praxis_value(Honest)]
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

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn skips_doctype_with_internal_subset() {
    let xml = r#"<?xml version="1.0"?><!DOCTYPE foo [<!ELEMENT bar (#PCDATA)>]><foo/>"#;
    let doc = parse_document(xml.as_bytes()).unwrap();
    assert_eq!(doc.root.name, XmlName::new("foo"));
}

// =============================================================================
// W3C XML 1.0 §4 Physical Structures — productions 45-69 + §4.2.2 ExternalID.
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parses_doctype_name_only() {
    let xml = br#"<?xml version="1.0"?><!DOCTYPE foo><foo/>"#;
    let doc = parse_document(xml).unwrap();
    let dt = doc.doctype.as_ref().expect("doctype expected");
    assert_eq!(dt.root_name, "foo");
    assert!(dt.external_id.is_none());
    assert!(dt.general_entities.is_empty());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parses_doctype_system_external_id() {
    // W3C XML 1.0 §4.2.2 [75] ExternalID — SYSTEM form.
    let xml = br#"<?xml version="1.0"?><!DOCTYPE foo SYSTEM "foo.dtd"><foo/>"#;
    let doc = parse_document(xml).unwrap();
    let dt = doc.doctype.as_ref().unwrap();
    assert_eq!(
        dt.external_id,
        Some(XmlExternalId::System {
            system_literal: "foo.dtd".into()
        })
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parses_doctype_public_external_id() {
    // W3C XML 1.0 §4.2.2 [75] ExternalID — PUBLIC form.
    let xml =
        br#"<?xml version="1.0"?><!DOCTYPE foo PUBLIC "-//Example//DTD//EN" "foo.dtd"><foo/>"#;
    let doc = parse_document(xml).unwrap();
    let dt = doc.doctype.as_ref().unwrap();
    assert_eq!(
        dt.external_id,
        Some(XmlExternalId::Public {
            public_id: "-//Example//DTD//EN".into(),
            system_literal: "foo.dtd".into(),
        })
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parses_internal_subset_general_entity_declaration() {
    // W3C XML 1.0 §4.2 [70/71] GEDecl: `<!ENTITY name "value">`.
    let xml = br#"<?xml version="1.0"?><!DOCTYPE foo [<!ENTITY hello "world">]><foo>&hello;</foo>"#;
    let doc = parse_document(xml).unwrap();
    let dt = doc.doctype.as_ref().unwrap();
    assert_eq!(dt.general_entities.len(), 1);
    assert_eq!(dt.general_entities[0].name, "hello");
    assert_eq!(dt.general_entities[0].value, "world");
    assert_eq!(dt.general_entities[0].kind, XmlEntityKind::Internal);
    // The reference in content was resolved.
    assert_eq!(doc.root.children[0], XmlNode::Text("world".into()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn resolves_declared_general_entity_in_attribute_value() {
    // §4.4.3 general-entity replacement applies inside attribute
    // values too (§3.3.3 says character + entity references
    // contribute unchanged).
    let xml = br#"<?xml version="1.0"?><!DOCTYPE foo [<!ENTITY tag "abc">]><foo bar="&tag;"/>"#;
    let doc = parse_document(xml).unwrap();
    assert_eq!(doc.root.attributes[0].value, "abc");
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn duplicate_entity_declaration_first_wins() {
    // §4.5 — "If the same entity is declared more than once, the
    // first declaration encountered is binding".
    let xml = br#"<?xml version="1.0"?><!DOCTYPE foo [<!ENTITY x "first"><!ENTITY x "second">]><foo>&x;</foo>"#;
    let doc = parse_document(xml).unwrap();
    let dt = doc.doctype.as_ref().unwrap();
    assert_eq!(dt.general_entities.len(), 1);
    assert_eq!(dt.general_entities[0].name, "x");
    assert_eq!(dt.general_entities[0].value, "first");
    assert_eq!(doc.root.children[0], XmlNode::Text("first".into()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn external_entity_declaration_registers_name_with_empty_replacement_text() {
    // §4.2 [73] ExternalID variant — we accept the declaration AND
    // register the name in `general_entities` with empty replacement
    // text. Per W3C XML 1.0 §4.4 Table-4 row "Reference in Content /
    // External Parsed General", a non-validating parser "Bypasses"
    // the reference — the praxis parser approximates this by
    // registering the name so subsequent `&ext;` references parse
    // as well-formed (no UnsupportedEntity error); the empty
    // replacement text reflects that we have not read the external
    // body. This is the well-formedness slice; reading external
    // entity bodies is deferred.
    let xml = br#"<?xml version="1.0"?><!DOCTYPE foo [<!ENTITY ext SYSTEM "ext.dtd">]><foo/>"#;
    let doc = parse_document(xml).unwrap();
    let dt = doc.doctype.as_ref().unwrap();
    assert_eq!(dt.general_entities.len(), 1);
    assert_eq!(dt.general_entities[0].name, "ext");
    assert_eq!(dt.general_entities[0].value, "");
    assert_eq!(dt.general_entities[0].kind, XmlEntityKind::ExternalParsed);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parameter_entity_declaration_skipped() {
    // §4.2 [72] PEDecl — recognized syntactically, not projected.
    let xml = br#"<?xml version="1.0"?><!DOCTYPE foo [<!ENTITY % p "value">]><foo/>"#;
    let doc = parse_document(xml).unwrap();
    let dt = doc.doctype.as_ref().unwrap();
    assert!(dt.general_entities.is_empty());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn attlist_with_fixed_default_decl_parses() {
    // §3.3 [60] DefaultDecl ::= '#REQUIRED' | '#IMPLIED'
    //                         | (('#FIXED' S)? AttValue)
    // xmlconf ibm/valid/P60/ibm60v01.xml — `<!ATTLIST three chapter
    // CDATA #FIXED "JavaBeans">` is the spec's regression for the
    // optional `('#FIXED' S)?` branch followed by an `AttValue`.
    let xml = br#"<?xml version="1.0"?>
<!DOCTYPE doc [
<!ELEMENT doc EMPTY>
<!ATTLIST doc chapter CDATA #FIXED "JavaBeans">
]><doc/>"#;
    assert!(parse_document(xml).is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn attlist_with_default_attvalue_only_parses() {
    // §3.3 [60] DefaultDecl — the bare `AttValue` form (no #FIXED).
    // xmlconf ibm/valid/P60/ibm60v01.xml — `<!ATTLIST four chapter
    // CDATA 'defualt'>`.
    let xml = br#"<?xml version="1.0"?>
<!DOCTYPE doc [
<!ELEMENT doc EMPTY>
<!ATTLIST doc chapter CDATA 'default'>
]><doc/>"#;
    assert!(parse_document(xml).is_ok());
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn external_unparsed_entity_reference_in_content_is_rejected() {
    // §4.4.4 WFC: Parsed Entity — "an entity reference MUST NOT
    // contain the name of an unparsed entity. Unparsed entities
    // may be referred to only in attribute values declared to be
    // of type ENTITY or ENTITIES" (which require validation, out
    // of scope). xmlconf xmltest/not-wf/sa/083 is the spec
    // regression.
    let xml = br#"<!DOCTYPE r [
<!NOTATION jpg SYSTEM "image/jpeg">
<!ENTITY pic SYSTEM "p.jpg" NDATA jpg>
]><r>&pic;</r>"#;
    assert!(parse_document(xml).is_err());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn external_unparsed_entity_kind_is_classified() {
    // Locked-in: an `<!ENTITY name SYSTEM "uri" NDATA n>` decl
    // registers as `XmlEntityKind::ExternalUnparsed`.
    let xml = br#"<!DOCTYPE r [
<!NOTATION jpg SYSTEM "image/jpeg">
<!ENTITY pic SYSTEM "p.jpg" NDATA jpg>
]><r/>"#;
    let doc = parse_document(xml).unwrap();
    let dt = doc.doctype.as_ref().unwrap();
    let pic = dt
        .general_entities
        .iter()
        .find(|e| e.name == "pic")
        .unwrap();
    assert_eq!(pic.kind, XmlEntityKind::ExternalUnparsed);
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn external_parsed_entity_reference_in_attribute_value_is_rejected() {
    // §3.1 + §4.4 row "Reference in Attribute Value" / WFC: No
    // External Entity References — references to external parsed
    // entities are forbidden in attribute values. xmlconf
    // ibm/not-wf/P41/ibm41n10 + xmltest/sa/081 regress.
    let xml = br#"<!DOCTYPE r [
<!ELEMENT r EMPTY>
<!ATTLIST r a CDATA #IMPLIED>
<!ENTITY ext SYSTEM "ext.txt">
]><r a="&ext;"/>"#;
    assert!(parse_document(xml).is_err());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn internal_entity_reference_in_attribute_value_resolves() {
    // Sanity: the §3.1 + §4.4 attribute-value reference WFCs are
    // *only* about external entities — internal entities still
    // resolve normally.
    let xml = br#"<!DOCTYPE r [
<!ELEMENT r EMPTY>
<!ATTLIST r a CDATA #IMPLIED>
<!ENTITY x "hello">
]><r a="&x;"/>"#;
    let doc = parse_document(xml).unwrap();
    assert_eq!(doc.root.attributes[0].value, "hello");
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn directly_recursive_entity_reference_is_rejected() {
    // §4.1 WFC: No Recursion — "A parsed entity MUST NOT contain
    // a recursive reference to itself, either directly or
    // indirectly." Direct self-reference: `&e;` resolves to text
    // that contains `&e;`. xmlconf xmltest/not-wf/sa/071 — single
    // self-referential internal entity — is the spec regression.
    let xml = br#"<!DOCTYPE r [<!ENTITY e "&e;">]><r>&e;</r>"#;
    assert!(parse_document(xml).is_err());
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn indirectly_recursive_entity_reference_is_rejected() {
    // §4.1 WFC: No Recursion (indirect cycle). xmlconf
    // xmltest/not-wf/sa/075 and sa/079 are three- and four-step
    // cycles; we test the minimal two-step indirect cycle.
    let xml = br#"<!DOCTYPE r [
<!ENTITY a "&b;">
<!ENTITY b "&a;">
]><r>&a;</r>"#;
    assert!(parse_document(xml).is_err());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn nested_internal_entity_reference_expands_recursively() {
    // §4.4.3 "Included" — a referenced internal parsed entity's
    // replacement text is itself processed for entity references.
    // Sanity test that paired with the WFC: No Recursion check we
    // still expand legitimate nested entities.
    let xml = br#"<!DOCTYPE r [
<!ENTITY inner "world">
<!ENTITY outer "hello &inner;">
]><r>&outer;</r>"#;
    let doc = parse_document(xml).unwrap();
    let text = match &doc.root.children[0] {
        crate::social::software::markup::xml::ontology::XmlNode::Text(t) => t,
        other => panic!("expected text node, got {other:?}"),
    };
    assert_eq!(text, "hello world");
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn parameter_entity_with_ndata_is_rejected_external_system() {
    // §4.2 [74] PEDef ::= EntityValue | ExternalID — note the
    // absence of NDataDecl. Parameter entities (§4.2 [72] PEDecl)
    // are parsed by definition; NDATA is general-entity-only
    // (§4.2 [73] EntityDef). xmlconf xmltest/not-wf/sa/089 is
    // the spec regression.
    let xml = br#"<!DOCTYPE doc [
<!ENTITY % foo SYSTEM "foo.xml" NDATA bar>
]>
<doc></doc>"#;
    assert!(parse_document(xml).is_err());
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn parameter_entity_with_ndata_is_rejected_external_public() {
    // §4.2 [74] PEDef + §4.2.2 [75] ExternalID PUBLIC variant +
    // forbidden NDataDecl. xmlconf xmltest/not-wf/sa/091.
    let xml = br#"<!DOCTYPE doc [
<!NOTATION n SYSTEM "n">
<!ENTITY % foo PUBLIC "pid" "foo.xml" NDATA n>
]>
<doc></doc>"#;
    assert!(parse_document(xml).is_err());
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn entity_value_with_literal_percent_is_rejected() {
    // W3C XML 1.0 §4.3.2 [9] EntityValue body alternation
    // `[^%&"]` excludes literal `%` — the spec says "the `%`
    // character must be escaped using a numeric character
    // reference or a parameter entity reference". xmlconf
    // ibm/not-wf/P09/ibm09n01 — `<!ENTITY x "Snow%Man">` — is
    // the spec regression.
    let xml = br#"<!DOCTYPE d [<!ENTITY x "Snow%Man">]><d/>"#;
    assert!(parse_document(xml).is_err());
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn entity_value_with_invalid_name_in_reference_is_rejected() {
    // §4.1 [68] EntityRef ::= '&' Name ';' — Name starts with
    // NameStartChar (digits excluded). `&49;` is malformed even
    // inside an EntityValue where references are bypassed: the
    // *syntax* still has to match. xmlconf ibm/not-wf/P66/ibm66n03.
    let xml = br#"<!DOCTYPE d [<!ENTITY x "ref: &49;">]><d/>"#;
    assert!(parse_document(xml).is_err());
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn user_entity_expansion_with_lt_in_attribute_is_rejected() {
    // §4.4 Table 4 WFC: No `<` in Attribute Values — when a
    // user-declared general entity is referenced from an
    // attribute value, the replacement text MUST NOT contain
    // `<`. xmlconf ibm/not-wf/P60/ibm60n07: `<!ENTITY x
    // "<Introduction">` then `attr="&x;"`.
    let xml = br#"<!DOCTYPE r [<!ELEMENT r EMPTY>
<!ATTLIST r a CDATA #REQUIRED>
<!ENTITY x "<bad">
]><r a="&x;"/>"#;
    assert!(parse_document(xml).is_err());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn predefined_lt_entity_in_attribute_is_allowed() {
    // §4.6 — `&lt;` is the sanctioned way to bring `<` into an
    // attribute value (its replacement text is the character
    // reference `&#60;`, not a literal `<`). The §4.4 WFC must
    // not over-fire and reject this.
    let xml = br#"<r a="&lt;"/>"#;
    let doc = parse_document(xml).unwrap();
    assert_eq!(doc.root.attributes[0].value, "<");
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn numeric_charref_to_lt_in_attribute_is_allowed() {
    // Same: `&#60;` resolves to `<` and is allowed in attribute
    // values (the WFC scopes to general-entity expansions only).
    let xml = br#"<r a="&#60;"/>"#;
    let doc = parse_document(xml).unwrap();
    assert_eq!(doc.root.attributes[0].value, "<");
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn entity_value_with_out_of_range_char_is_rejected() {
    // §4.3.2 [9] EntityValue body alternation `([^%&"] | …)` is
    // §2.2 [2] Char minus the literal-delimiters. A literal NUL
    // (#x0) or other XML 1.0 control-byte embedded in the value
    // is malformed even though it's not %, &, or the closing
    // quote. xmlconf ibm/xml-1.1/not-wf/P02 cases (1.1-only
    // control chars in EntityValue) regress here.
    let xml = b"<!DOCTYPE doc [<!ENTITY e \"bad\x01char\">]><doc/>";
    assert!(parse_document(xml).is_err());
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn att_value_with_out_of_range_char_is_rejected() {
    // §3.1 [10] AttValue body alternation `([^<&"] | …)` — same
    // shape, same Char restriction.
    let xml = b"<doc a=\"bad\x01char\"/>";
    assert!(parse_document(xml).is_err());
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn pi_in_content_with_xml_target_is_rejected() {
    // §2.6 [17] PITarget excludes `xml` case-insensitive. The
    // content-level PI parser (parse_pi_node) needs the same
    // gate as the Misc-level skip_pi.
    let xml = b"<doc><?xMl bad?></doc>";
    assert!(parse_document(xml).is_err());
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn xml_decl_rejects_missing_space_before_standalone() {
    // §2.8 [32] SDDecl ::= S 'standalone' Eq ...
    // — the leading S is required, not optional whitespace.
    // xmlconf ibm/not-wf/P32/ibm32n01 regresses here
    // (`version="1.0"standalone="yes"`).
    let xml = br#"<?xml version="1.0"standalone="yes"?><doc/>"#;
    assert!(parse_document(xml).is_err());
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn xml_decl_rejects_non_lowercase_standalone_keyword_and_value() {
    // §2.8 [32] SDDecl — the keyword `standalone` is lowercase
    // and the value is exactly `yes` or `no` (lowercase).
    // xmlconf ibm/not-wf/P32/ibm32n03..07 regress.
    assert!(parse_document(br#"<?xml version="1.0" Standalone="yes"?><doc/>"#).is_err());
    assert!(parse_document(br#"<?xml version="1.0" standalone="Yes"?><doc/>"#).is_err());
    assert!(parse_document(br#"<?xml version="1.0" standalone="YES"?><doc/>"#).is_err());
    assert!(parse_document(br#"<?xml version="1.0" standalone="No"?><doc/>"#).is_err());
    // Sanity: the lowercase forms parse.
    assert!(parse_document(br#"<?xml version="1.0" standalone="yes"?><doc/>"#).is_ok());
    assert!(parse_document(br#"<?xml version="1.0" standalone="no"?><doc/>"#).is_ok());
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn xml_decl_rejects_malformed_version_num() {
    // §2.8 [26] VersionNum ::= '1.' [0-9]+
    // xmlconf cases that put non-numeric junk after `1.` — or omit
    // the major-version `1.` prefix entirely — must reject.
    assert!(parse_document(br#"<?xml version="1.a"?><doc/>"#).is_err());
    assert!(parse_document(br#"<?xml version="2.0"?><doc/>"#).is_err());
    assert!(parse_document(br#"<?xml version="1."?><doc/>"#).is_err());
    // Sanity: `1.0` and `1.1` (5e accepts any `1.x`) both parse.
    assert!(parse_document(br#"<?xml version="1.0"?><doc/>"#).is_ok());
    assert!(parse_document(br#"<?xml version="1.1"?><doc/>"#).is_ok());
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn misc_position_comment_with_out_of_range_char_is_rejected() {
    // W3C XML 1.0 §2.5 [15] + §2.2 [2] Char — comment bodies must
    // be in the Char repertoire even at Misc positions (after the
    // doctype, after the root). xmlconf ibm/not-wf/P02 cases
    // (NULL inside comments) regress here.
    let xml = b"<?xml version=\"1.0\"?>\n<doc/>\n<!-- bad \x00 -->\n";
    assert!(parse_document(xml).is_err());
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn misc_position_pi_with_xml_target_is_rejected() {
    // W3C XML 1.0 §2.6 [17] PITarget — `PITarget ::= Name -
    // (('X'|'x') ('M'|'m') ('L'|'l'))`. A PI whose target is
    // case-insensitively `xml` is malformed (the only reserved
    // `<?xml ... ?>` form is the §2.8 XMLDecl, syntactically
    // distinct and consumed by `parse_xml_decl`).
    assert!(parse_document(b"<doc/>\n<?XmL nope?>").is_err());
    assert!(parse_document(b"<doc/>\n<?xml-stylesheet href=\"x\"?>").is_ok());
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn comment_ending_with_trailing_dash_is_rejected() {
    // W3C XML 1.0 §2.5 production [15] Comment —
    // `Comment ::= '<!--' ((Char - '-') | ('-' (Char - '-')))* '-->'`
    // The body alternation forbids a trailing `-` (the would-be
    // last char of body must be matched by `(Char - '-')`).
    // xmlconf xmltest/not-wf/sa/070.xml — `<!-- ... --->` (three
    // dashes before `>`) — is the spec's regression test.
    assert!(parse_document(b"<!-- foo --->\n<doc></doc>").is_err());
    // Inside content — the in-element comment parser must also catch it.
    assert!(parse_document(b"<doc><!-- bar ---></doc>").is_err());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parameter_entity_at_decl_sep_includes_a_general_entity_decl() {
    // §2.8 [28b] intSubset / [28a] DeclSep — when a `%name;` appears
    // between markup declarations, the PE is *included*: its
    // replacement text is parsed at the reference point per §4.4.8
    // "Included as PE", bracketed with leading/trailing #x20. A PE
    // whose value is a complete `<!ENTITY ...>` general-entity decl
    // therefore contributes that decl to the intsubset.
    let xml = br#"<?xml version="1.0"?><!DOCTYPE foo [
        <!ENTITY % chunk '<!ENTITY g "hello">'>
        %chunk;
    ]><foo>&g;</foo>"#;
    let doc = parse_document(xml).unwrap();
    let dt = doc.doctype.as_ref().unwrap();
    assert_eq!(dt.general_entities.len(), 1);
    assert_eq!(dt.general_entities[0].name, "g");
    assert_eq!(dt.general_entities[0].value, "hello");
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn parameter_entity_inside_markup_decl_in_internal_subset_is_rejected() {
    // W3C XML 1.0 §4.4.8 WFC: PEs in Internal Subset — *"in the
    // internal DTD subset, parameter-entity references MUST NOT
    // occur within markup declarations; they may occur where markup
    // declarations can occur"*. xmlconf ibm/not-wf/P29/ibm29n02.xml
    // is the spec's regression test for this constraint.
    let xml = br#"<?xml version="1.0"?><!DOCTYPE foo [
        <!ENTITY % parameterE "leopard EMPTY>">
        <!ELEMENT %parameterE;
    ]><foo/>"#;
    assert!(parse_document(xml).is_err());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn element_type_declaration_in_subset_skipped() {
    // §3.2 [45] elementdecl — affects validity, not well-formedness.
    let xml = br#"<?xml version="1.0"?><!DOCTYPE foo [<!ELEMENT foo EMPTY>]><foo/>"#;
    let doc = parse_document(xml).unwrap();
    assert_eq!(doc.root.name, XmlName::new("foo"));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn attribute_list_declaration_in_subset_skipped() {
    // §3.3 [52] AttlistDecl — same: validity, not well-formedness.
    let xml = br#"<?xml version="1.0"?><!DOCTYPE foo [<!ATTLIST foo a CDATA #IMPLIED>]><foo/>"#;
    let doc = parse_document(xml).unwrap();
    assert_eq!(doc.root.name, XmlName::new("foo"));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn notation_declaration_in_subset_skipped() {
    // §4.7 [82] NotationDecl — same: validity, not well-formedness.
    let xml = br#"<?xml version="1.0"?><!DOCTYPE foo [<!NOTATION gif SYSTEM "image/gif">]><foo/>"#;
    let doc = parse_document(xml).unwrap();
    assert_eq!(doc.root.name, XmlName::new("foo"));
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn doctype_round_trips_through_lens() {
    // Foster et al. 2007 §3, Definition 3.2 GetPut on a document with DOCTYPE.
    let xml = br#"<?xml version="1.0"?><!DOCTYPE foo [<!ENTITY x "y">]><foo>&x;</foo>"#;
    let parsed = XmlLens::get(xml).unwrap();
    let serialized = XmlLens::put(&parsed).unwrap();
    let reparsed = XmlLens::get(&serialized).unwrap();
    assert_eq!(parsed, reparsed);
}

// =============================================================================
// W3C XML 1.0 §2.11 End-of-Line Handling (production-quality conformance).
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn normalizes_crlf_line_endings_in_content() {
    // §2.11: "the XML processor MUST behave as if it normalized all
    // line breaks … on input … to the single character #xA".
    let xml = b"<?xml version=\"1.0\"?><r>line1\r\nline2</r>";
    let doc = parse_document(xml).unwrap();
    assert_eq!(doc.root.children[0], XmlNode::Text("line1\nline2".into()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn normalizes_lone_cr_line_endings_in_content() {
    let xml = b"<?xml version=\"1.0\"?><r>line1\rline2</r>";
    let doc = parse_document(xml).unwrap();
    assert_eq!(doc.root.children[0], XmlNode::Text("line1\nline2".into()));
}

// =============================================================================
// W3C XML 1.0 §3.3.3 Attribute-Value Normalization.
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn normalizes_literal_whitespace_in_attribute_values() {
    // §3.3.3 step 3.1.4: literal whitespace chars in attribute
    // values become a single #x20 (space) each. CRLF was already
    // normalized to LF by §2.11; the LF then becomes a space.
    let xml = b"<?xml version=\"1.0\"?><r a=\"x\ty\nz\"/>";
    let doc = parse_document(xml).unwrap();
    assert_eq!(doc.root.attributes[0].value, "x y z");
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn preserves_character_references_in_attribute_values() {
    // §3.3.3 step 3.1.1: characters from references contribute
    // unchanged — so &#xA; resolves to LF inside the attribute,
    // distinct from a literal LF which would normalize to space.
    let xml = b"<?xml version=\"1.0\"?><r a=\"x&#xA;y\"/>";
    let doc = parse_document(xml).unwrap();
    assert_eq!(doc.root.attributes[0].value, "x\ny");
}

// =============================================================================
// W3C XML 1.0 §3.1 well-formedness constraint: Unique Att Spec.
// =============================================================================

#[pr4xis::praxis_value(Honest)]
#[test]
fn rejects_duplicate_attribute_names_in_start_tag() {
    let xml = b"<?xml version=\"1.0\"?><r a=\"1\" a=\"2\"/>";
    match parse_document(xml) {
        Err(XmlParseError::DuplicateAttribute { name, .. }) => {
            assert_eq!(name, "a");
        }
        other => panic!("expected DuplicateAttribute, got {other:?}"),
    }
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn rejects_duplicate_attribute_with_namespace_prefix() {
    let xml = b"<?xml version=\"1.0\"?><r xmlns:p=\"http://x\" p:a=\"1\" p:a=\"2\"/>";
    match parse_document(xml) {
        Err(XmlParseError::DuplicateAttribute { name, .. }) => {
            assert_eq!(name, "p:a");
        }
        other => panic!("expected DuplicateAttribute, got {other:?}"),
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn accepts_same_local_name_with_different_prefix() {
    // Per the wf constraint, "name" means the qualified-name string.
    // `p:a` and `q:a` are distinct names.
    let xml = b"<?xml version=\"1.0\"?><r xmlns:p=\"http://x\" xmlns:q=\"http://y\" p:a=\"1\" q:a=\"2\"/>";
    let doc = parse_document(xml).unwrap();
    let has_p_a = doc
        .root
        .attributes
        .iter()
        .any(|a| a.name.qualified() == "p:a");
    let has_q_a = doc
        .root
        .attributes
        .iter()
        .any(|a| a.name.qualified() == "q:a");
    assert!(
        has_p_a && has_q_a,
        "both p:a and q:a accepted as distinct names"
    );
}

// =============================================================================
// Round-trip / lens-law tests (Foster et al. 2007 §3, Definition 3.2).
// =============================================================================

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn round_trip_minimal_empty_element() {
    let xml = b"<?xml version=\"1.0\"?><root/>";
    let parsed = XmlLens::get(xml).unwrap();
    let serialized = XmlLens::put(&parsed).unwrap();
    assert_eq!(serialized, xml);
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn round_trip_with_attributes_and_namespace() {
    let xml = b"<?xml version=\"1.0\"?><root xmlns=\"http://x\" a=\"1\"/>";
    let parsed = XmlLens::get(xml).unwrap();
    let serialized = XmlLens::put(&parsed).unwrap();
    assert_eq!(serialized, xml);
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn get_put_law_holds_for_simple_doc() {
    // GetPut: parse, re-serialize, re-parse — typed value matches.
    let xml = b"<?xml version=\"1.0\"?><a><b>hi</b><c x=\"1\"/></a>";
    let t1 = XmlLens::get(xml).unwrap();
    let bytes2 = XmlLens::put(&t1).unwrap();
    let t2 = XmlLens::get(&bytes2).unwrap();
    assert_eq!(t1, t2);
}

#[pr4xis::praxis_value(Deterministic)]
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

#[pr4xis::praxis_value(Deterministic)]
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

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn lens_serializes_synthesised_typed_value() {
    // Hand-build an XmlDocument and verify the serializer's output
    // re-parses to the same typed value (Foster et al. 2007 GetPut).
    let doc = XmlDocument {
        version: "1.0".into(),
        encoding: None,
        doctype: None,
        root: XmlElement {
            name: XmlName::new("synth"),
            namespace: None,
            namespaces: Vec::new(),
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
// Each property names a Foster et al. 2007 §3, Definition 3.2 lens law and a W3C XML 1.0
// §-reference. proptest cases generate synthetic XmlDocument values built
// from the grammar's productions and check the law holds.

mod property {
    use super::*;
    use proptest::prelude::*;

    /// W3C XML 1.0 §2.3 production \[4\] / \[4a\] NameStartChar / NameChar —
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

    /// W3C XML 1.0 §3.1 production \[10\] AttValue — characters allowed in
    /// an attribute value (excluding `<` and `&`, both must be escaped).
    /// Also excludes literal whitespace (`#x9`, `#xA`, `#xD`); those
    /// trigger §3.3.3 attribute-value normalization (whitespace → space)
    /// which breaks naive byte-for-byte round-trip. The dedicated
    /// `property_attr_literal_whitespace_normalized` test exercises
    /// the normalization path separately.
    fn arb_att_value() -> impl Strategy<Value = String> {
        proptest::collection::vec(
            any::<char>().prop_filter("XML 1.0 §2.2 Char minus '<' '&' and whitespace", |c| {
                let cp = *c as u32;
                let in_char = matches!(cp, 0x20..=0xD7FF | 0xE000..=0xFFFD | 0x10000..=0x10FFFF);
                in_char && *c != '<' && *c != '&'
            }),
            0..32,
        )
        .prop_map(|chars| chars.into_iter().collect())
    }

    /// W3C XML 1.0 §2.4 production \[14\] CharData — element content
    /// text minus the markup delimiters `<` and `&`. Also excludes
    /// lone `#xD` (CR) since §2.11 End-of-Line Handling normalizes
    /// any CR sequence to LF on input, breaking byte-for-byte
    /// round-trip; the dedicated `property_crlf_normalized_to_lf`
    /// test covers that path.
    fn arb_char_data() -> impl Strategy<Value = String> {
        proptest::collection::vec(
            any::<char>().prop_filter("XML 1.0 §2.4 CharData chars (no CR)", |c| {
                let cp = *c as u32;
                let in_char =
                    matches!(cp, 0x9 | 0xA | 0x20..=0xD7FF | 0xE000..=0xFFFD | 0x10000..=0x10FFFF);
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
                    doctype: None,
                    root: XmlElement {
                        name: XmlName::new(&root_name),
                        namespace: None,
                        namespaces: Vec::new(),
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
        /// Foster et al. 2007 §3, Definition 3.2 GetPut law:
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
                doctype: None,
                root: XmlElement {
                    name: XmlName::new("r"),
                    namespace: None,
                    namespaces: Vec::new(),
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
                doctype: None,
                root: XmlElement {
                    name: XmlName::new("r"),
                    namespace: None,
                    namespaces: Vec::new(),
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

        /// W3C XML 1.0 §2.11 End-of-Line Handling property: every
        /// CRLF or lone CR in the document body is normalized to LF
        /// on parse. We synthesize a CRLF-containing payload, parse,
        /// and assert no CR remains in the resulting text content.
        #[test]
        fn property_crlf_normalized_to_lf(parts in proptest::collection::vec(
            "[a-zA-Z]{0,5}", 1..6
        )) {
            let crlf_text: String = parts.join("\r\n");
            let xml = format!(
                "<?xml version=\"1.0\"?><r>{}</r>",
                crlf_text.replace(['<', '&'], "")
            );
            let doc = parse_document(xml.as_bytes()).unwrap();
            if let Some(XmlNode::Text(t)) = doc.root.children.first() {
                prop_assert!(!t.contains('\r'), "text contained CR after §2.11 normalization: {t:?}");
            }
        }

        /// W3C XML 1.0 §3.3.3 Attribute-Value Normalization property:
        /// every literal tab / newline in an attribute value is
        /// normalized to a single space on parse. Carriage returns
        /// are handled by §2.11 before the §3.3.3 path runs.
        #[test]
        fn property_attr_literal_whitespace_normalized(
            prefix in "[a-z]{0,3}",
            suffix in "[a-z]{0,3}",
            ws_idx in 0usize..2,
        ) {
            let ws = match ws_idx {
                0 => '\t',
                _ => '\n',
            };
            let value = format!("{prefix}{ws}{suffix}");
            let xml = format!("<?xml version=\"1.0\"?><r a=\"{value}\"/>");
            let doc = parse_document(xml.as_bytes()).unwrap();
            let normalized = &doc.root.attributes[0].value;
            let expected = format!("{prefix} {suffix}");
            prop_assert_eq!(normalized.clone(), expected);
        }

        /// W3C XML 1.0 §3.1 Unique Att Spec property: any two
        /// attributes sharing a qualified name in the same start-tag
        /// MUST be rejected as ill-formed.
        #[test]
        fn property_duplicate_attribute_rejected(name in arb_ascii_name()) {
            let xml = format!("<?xml version=\"1.0\"?><r {name}=\"1\" {name}=\"2\"/>");
            let result = parse_document(xml.as_bytes());
            let is_dup = matches!(result, Err(XmlParseError::DuplicateAttribute { .. }));
            prop_assert!(is_dup, "expected DuplicateAttribute, got {result:?}");
        }

        /// W3C XML 1.0 §4.4.3 property: every legal entity name +
        /// value pair declared in the internal subset resolves
        /// correctly when referenced from content.
        #[test]
        fn property_declared_general_entity_resolves(
            name in arb_ascii_name(),
            value in arb_char_data(),
        ) {
            // Skip predefined entity names — they take precedence.
            prop_assume!(!matches!(name.as_str(), "amp" | "lt" | "gt" | "apos" | "quot"));
            // Stay clear of quote chars in the value (would break the
            // entity declaration), `&` (would be parsed as a Reference
            // start, not literal data), and `%` (§4.3.2 `[^%&"]`
            // excludes literal `%` from EntityValue — the spec
            // requires it escaped via numeric char ref or PE ref).
            prop_assume!(
                !value.contains('"')
                    && !value.contains('&')
                    && !value.contains('%')
            );
            let xml = format!(
                "<?xml version=\"1.0\"?><!DOCTYPE r [<!ENTITY {name} \"{value}\">]><r>&{name};</r>"
            );
            let doc = parse_document(xml.as_bytes()).unwrap();
            if value.is_empty() {
                prop_assert!(doc.root.children.is_empty());
            } else {
                prop_assert_eq!(doc.root.children[0].clone(), XmlNode::Text(value));
            }
        }
    }

    pr4xis::register_praxis_value!(property_get_put_law, Deterministic);
    pr4xis::register_praxis_value!(property_mismatched_tags_rejected, Honest);
    pr4xis::register_praxis_value!(property_predefined_entities_always_expand, Verifiable);
    pr4xis::register_praxis_value!(property_char_data_round_trip, Deterministic);
    pr4xis::register_praxis_value!(property_attr_value_round_trip, Deterministic);
    pr4xis::register_praxis_value!(property_crlf_normalized_to_lf, Verifiable);
    pr4xis::register_praxis_value!(property_attr_literal_whitespace_normalized, Verifiable);
    pr4xis::register_praxis_value!(property_duplicate_attribute_rejected, Honest);
    pr4xis::register_praxis_value!(property_declared_general_entity_resolves, Verifiable);
}
