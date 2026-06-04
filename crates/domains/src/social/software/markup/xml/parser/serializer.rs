//! Symmetric serializer pairing with [`grammar`](super::grammar) —
//! `XmlDocument` → bytes.
//!
//! Each `serialize_<production>` function inverts the
//! correspondingly named parser function: when chained
//! `serialize(parse(s))` yields a byte stream that, after W3C XML
//! Canonicalization 1.1 (Boyer & Marcy 2008), is byte-identical to
//! `canonical(s)`. That is the PutGet law witness used by
//! [`XmlLens::assert_put_get_law`](
//! crate::formal::meta::well_behaved_lens::WellBehavedLens::assert_put_get_law).
//!
//! The serializer produces W3C XML 1.0 §3 well-formed output but
//! does NOT itself emit canonical form — canonicalization is the
//! lens's [`canonical`](
//! crate::formal::meta::well_behaved_lens::WellBehavedLens::canonical)
//! step, applied to the serializer's output before comparison.
//!
//! # Citations
//!
//! - **Bray et al. (2008)** XML 1.0 Fifth Edition §3 — element /
//!   attribute / content syntax.
//! - **Bray et al. (2008)** §4.6 — predefined entity references.
//! - **Boyer & Marcy (2008)** W3C XML Canonicalization 1.1 §3.5 —
//!   character escapes used in canonical form (so we emit forms
//!   that the canonicalizer can normalize without ambiguity).

#[allow(unused_imports)]
use alloc::{format, string::String, vec, vec::Vec};

use super::super::ontology::{
    XmlAttribute, XmlDoctype, XmlDocument, XmlElement, XmlExternalId, XmlName, XmlNamespace,
    XmlNode,
};
use super::source_syntax::{EmptyForm, SyntaxDecisions};

/// Top-level entry point: emit a [`XmlDocument`] as W3C XML 1.0
/// §2.1 `document` bytes.
pub fn serialize_document(doc: &XmlDocument) -> Vec<u8> {
    let mut out = String::new();
    write_xml_decl(&mut out, &doc.version, doc.encoding.as_deref());
    if let Some(doctype) = &doc.doctype {
        write_doctype(&mut out, doctype);
    }
    write_element(&mut out, &doc.root);
    out.into_bytes()
}

// ── Byte-exact serialization — the graph-faithful `put` ──────────────────────
//
// `serialize_document` above is the CANONICAL serializer (the floor's PutGet
// witness): it normalizes away byte-affecting choices C14N 1.1 erases — a
// childless element always self-closes, entity escaping is the canonical set.
// `serialize_document_exact` is its byte-exact sibling: it reproduces the ORIGINAL
// bytes from the Information Set DOM PLUS the recorded concrete-syntax decisions,
// with no stored raw source. This is the L0 byte kernel of the serialized reverse
// lens — the one place that is irreducibly imperative (a byte stream is not a
// category to fold through; Foster et al. 2007's `put` bottoms out in byte
// emission), and it is XML-family-wide, written once rather than per format.
//
// The residue types ([`EmptyForm`], [`NodeDecisions`], [`SyntaxDecisions`]) live
// in [`super::source_syntax`] so the reader (`grammar`) can CAPTURE the same
// decisions this writer HONOURS without either depending on the other.

/// Byte-exact serialization — exact source bytes from the Information Set DOM
/// PLUS the recorded [`SyntaxDecisions`], with no stored raw source. Like
/// [`serialize_document`] but honours the decisions; today the empty-element
/// form, so a childless element recorded `Explicit` round-trips as `<a></a>`
/// rather than the canonical `<a/>`. Where no decision is recorded it falls back
/// to the canonical serializer's choice.
pub fn serialize_document_exact(doc: &XmlDocument, decisions: &SyntaxDecisions) -> Vec<u8> {
    let prolog = decisions.prolog();
    let mut out = String::new();
    write_xml_decl(&mut out, &doc.version, doc.encoding.as_deref());
    // §2.8 [27] Misc* white-space after the XMLDecl, before the DOCTYPE-or-root
    // (empty for a canonical prolog — `SyntaxDecisions::default()` carries no
    // prolog white-space, so existing callers are unaffected).
    out.push_str(&prolog.after_xml_decl);
    if let Some(doctype) = &doc.doctype {
        write_doctype(&mut out, doctype);
        // §2.8 [27] Misc* white-space after the DOCTYPE, before the root.
        out.push_str(&prolog.after_doctype);
    }
    // Pre-order element counter — incremented at every element's ENTRY in
    // `write_element_exact`, mirroring the reader's capture counter so the two
    // walks agree index-for-index (root = 0).
    let mut index: usize = 0;
    write_element_exact(&mut out, &doc.root, &mut index, decisions);
    // §2.1 [1] `document ::= prolog element Misc*` — the trailing epilog Misc*
    // white-space after the root element's end-tag.
    out.push_str(&prolog.after_root);
    out.into_bytes()
}

fn write_element_exact(
    out: &mut String,
    el: &XmlElement,
    index: &mut usize,
    decisions: &SyntaxDecisions,
) {
    // Take this element's pre-order index AT ENTRY, before descending into
    // children — matching the reader, where `parse_element` claims its index at
    // entry too.
    let my_index = *index;
    *index += 1;
    out.push('<');
    write_name(out, &el.name);
    if el.namespaces.is_empty() {
        if let Some(ns) = &el.namespace {
            write_namespace_decl(out, ns);
        }
    } else {
        for ns in &el.namespaces {
            write_namespace_decl(out, ns);
        }
    }
    for attr in &el.attributes {
        write_attribute(out, attr);
    }
    if el.children.is_empty() {
        // The empty-element form is a recorded decision; default to the
        // canonical self-closing form when none is recorded.
        let explicit = matches!(
            decisions.get(my_index).and_then(|d| d.empty_form),
            Some(EmptyForm::Explicit)
        );
        if explicit {
            out.push_str("></");
            write_name(out, &el.name);
            out.push('>');
        } else {
            out.push_str("/>");
        }
    } else {
        out.push('>');
        for child in &el.children {
            write_node_exact(out, child, index, decisions);
        }
        out.push_str("</");
        write_name(out, &el.name);
        out.push('>');
    }
}

fn write_node_exact(
    out: &mut String,
    node: &XmlNode,
    index: &mut usize,
    decisions: &SyntaxDecisions,
) {
    match node {
        XmlNode::Element(el) => write_element_exact(out, el, index, decisions),
        // Non-element nodes render as in the canonical serializer; their own
        // byte-exact residue (comment white-space, entity style) lands in
        // follow-up decisions. They occupy no pre-order ELEMENT slot, so the
        // counter is untouched.
        other => write_node(out, other),
    }
}

/// W3C XML 1.0 §2.8 production \[28\] `doctypedecl` — inverse of
/// [`grammar::parse_doctype`](super::grammar). Re-emits the root
/// element name, optional `ExternalID`, and inline general entity
/// declarations (`<!ENTITY name "value">`) the parser projected.
fn write_doctype(out: &mut String, doctype: &XmlDoctype) {
    out.push_str("<!DOCTYPE ");
    out.push_str(&doctype.root_name);
    if let Some(id) = &doctype.external_id {
        match id {
            XmlExternalId::System { system_literal } => {
                out.push_str(" SYSTEM \"");
                out.push_str(system_literal);
                out.push('"');
            }
            XmlExternalId::Public {
                public_id,
                system_literal,
            } => {
                out.push_str(" PUBLIC \"");
                out.push_str(public_id);
                out.push_str("\" \"");
                out.push_str(system_literal);
                out.push('"');
            }
        }
    }
    if !doctype.general_entities.is_empty() {
        out.push_str(" [");
        for entity in &doctype.general_entities {
            out.push_str("<!ENTITY ");
            out.push_str(&entity.name);
            out.push_str(" \"");
            write_escaped_entity_value(out, &entity.value);
            out.push_str("\">");
        }
        out.push(']');
    }
    out.push('>');
}

/// W3C XML 1.0 §4.3.2 production \[9\] `EntityValue` — escape the
/// minimum required for the value to be re-readable inside a
/// double-quoted entity declaration: `&` → `&amp;`, `"` → `&quot;`.
fn write_escaped_entity_value(out: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
}

/// W3C XML 1.0 §2.8 production \[23\] `XMLDecl`. Re-emitted with the
/// exact version + encoding the [`XmlDocument`] carries.
fn write_xml_decl(out: &mut String, version: &str, encoding: Option<&str>) {
    out.push_str("<?xml version=\"");
    out.push_str(version);
    out.push('"');
    if let Some(enc) = encoding {
        out.push_str(" encoding=\"");
        out.push_str(enc);
        out.push('"');
    }
    out.push_str("?>");
}

/// W3C XML 1.0 §3 production \[39\] `element`. Emits the
/// `EmptyElemTag` short form when the element has no children, the
/// `STag content ETag` long form otherwise.
fn write_element(out: &mut String, el: &XmlElement) {
    out.push('<');
    write_name(out, &el.name);
    // Namespaces in XML 1.0 (Bray, Hollander, Layman & Tobin 2009) §3
    // allows any number of `xmlns` / `xmlns:prefix` declarations on a
    // single element. Emit every declaration the element carries; when
    // `namespaces` is empty fall back to the single-slot `namespace`
    // for parity with elements built via legacy constructors.
    if el.namespaces.is_empty() {
        if let Some(ns) = &el.namespace {
            write_namespace_decl(out, ns);
        }
    } else {
        for ns in &el.namespaces {
            write_namespace_decl(out, ns);
        }
    }
    for attr in &el.attributes {
        write_attribute(out, attr);
    }
    if el.children.is_empty() {
        out.push_str("/>");
    } else {
        out.push('>');
        for child in &el.children {
            write_node(out, child);
        }
        out.push_str("</");
        write_name(out, &el.name);
        out.push('>');
    }
}

fn write_name(out: &mut String, name: &XmlName) {
    if let Some(prefix) = &name.prefix {
        out.push_str(prefix);
        out.push(':');
    }
    out.push_str(&name.local);
}

/// Namespaces in XML 1.0 (Bray, Hollander, Layman & Tobin 2009)
/// §3 — `xmlns` and `xmlns:prefix` attributes. Emitted alongside
/// the element name before regular attributes.
fn write_namespace_decl(out: &mut String, ns: &XmlNamespace) {
    out.push(' ');
    match &ns.prefix {
        Some(prefix) => {
            out.push_str("xmlns:");
            out.push_str(prefix);
        }
        None => out.push_str("xmlns"),
    }
    out.push_str("=\"");
    write_escaped_attr_value(out, &ns.uri);
    out.push('"');
}

fn write_attribute(out: &mut String, attr: &XmlAttribute) {
    out.push(' ');
    write_name(out, &attr.name);
    out.push_str("=\"");
    write_escaped_attr_value(out, &attr.value);
    out.push('"');
}

fn write_node(out: &mut String, node: &XmlNode) {
    match node {
        XmlNode::Element(el) => write_element(out, el),
        XmlNode::Text(t) => write_escaped_char_data(out, t),
        XmlNode::CData(t) => {
            out.push_str("<![CDATA[");
            out.push_str(t);
            out.push_str("]]>");
        }
        XmlNode::Comment(t) => {
            out.push_str("<!--");
            out.push_str(t);
            out.push_str("-->");
        }
        XmlNode::ProcessingInstruction { target, data } => {
            out.push_str("<?");
            out.push_str(target);
            if let Some(d) = data {
                out.push(' ');
                out.push_str(d);
            }
            out.push_str("?>");
        }
    }
}

/// W3C XML 1.0 §4.6 — escape the three characters that
/// `CharData` (§2.4 production \[14\]) cannot contain literally:
/// `&` → `&amp;`, `<` → `&lt;`. `>` is conditionally required
/// only after `]]` (the CDATA terminator); we always escape it
/// for safety. The escape forms here are exactly the ones C14N
/// 1.1 §3.5 normalizes to, so the canonicalizer's output is
/// stable.
fn write_escaped_char_data(out: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
}

/// W3C XML 1.0 §3.3.3 — attribute-value normalization defines
/// the characters that must be escaped inside `AttValue`. We
/// escape the C14N 1.1 §3.5 set (`&`, `<`, `"`, `\r`, `\n`, `\t`)
/// so the put output round-trips through canonicalization stably.
fn write_escaped_attr_value(out: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '"' => out.push_str("&quot;"),
            '\r' => out.push_str("&#xD;"),
            '\n' => out.push_str("&#xA;"),
            '\t' => out.push_str("&#x9;"),
            _ => out.push(ch),
        }
    }
}

#[cfg(test)]
mod exact_tests {
    use super::super::source_syntax::NodeDecisions;
    use super::*;

    fn doc(root: XmlElement) -> XmlDocument {
        XmlDocument {
            version: "1.0".into(),
            encoding: Some("UTF-8".into()),
            doctype: None,
            root,
        }
    }

    fn el(local: &str, children: Vec<XmlNode>) -> XmlElement {
        XmlElement {
            name: XmlName::new(local),
            namespace: None,
            namespaces: vec![],
            attributes: vec![],
            children,
        }
    }

    #[test]
    fn empty_element_form_is_honoured() {
        let d = doc(el("a", vec![]));
        // Canonical serialization always self-closes a childless element…
        assert_eq!(
            serialize_document(&d),
            br#"<?xml version="1.0" encoding="UTF-8"?><a/>"#.to_vec()
        );
        // …the byte-exact serializer honours an `Explicit` decision on it
        // (the root element is pre-order index 0)…
        let mut decisions = SyntaxDecisions::new();
        decisions.set(
            0,
            NodeDecisions {
                empty_form: Some(EmptyForm::Explicit),
            },
        );
        assert_eq!(
            serialize_document_exact(&d, &decisions),
            br#"<?xml version="1.0" encoding="UTF-8"?><a></a>"#.to_vec()
        );
        // …and with no decision recorded it matches the canonical form.
        assert_eq!(
            serialize_document_exact(&d, &SyntaxDecisions::new()),
            serialize_document(&d)
        );
    }

    #[test]
    fn nested_explicit_empty_child_keyed_by_index() {
        // `<root><a/></root>` with the child (pre-order index 1, after the root
        // at 0) recorded `Explicit`.
        let d = doc(el("root", vec![XmlNode::Element(el("a", vec![]))]));
        let mut decisions = SyntaxDecisions::new();
        decisions.set(
            1,
            NodeDecisions {
                empty_form: Some(EmptyForm::Explicit),
            },
        );
        assert_eq!(
            serialize_document_exact(&d, &decisions),
            br#"<?xml version="1.0" encoding="UTF-8"?><root><a></a></root>"#.to_vec()
        );
    }
}
