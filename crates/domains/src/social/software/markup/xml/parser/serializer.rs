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
use super::source_syntax::{
    EmptyForm, EndOfLineForm, EntityReferenceForm, EolKind, IntraTagWhitespace, StartTagToken,
    SyntaxDecisions,
};
use alloc::collections::BTreeMap;

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
/// [`serialize_document`] but honours the decisions: the empty-element form (so a
/// childless element recorded `Explicit` round-trips as `<a></a>` rather than the
/// canonical `<a/>`), the intra-tag white-space, the §4.6 entity-reference form,
/// the prolog/epilog `Misc*`, and — as a FINAL pass over the finished bytes — the
/// §2.11 \[2.11\] end-of-line form (re-expanding each normalized `#xA` back to the
/// source `#xD#xA`/`#xD`). Where no decision is recorded it falls back to the
/// canonical serializer's choice; an empty EOL form leaves the bytes untouched.
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
    // §2.11 [2.11] End-of-Line Handling, INVERTED: the bytes so far carry the
    // §2.11-normalized `#xA` everywhere (the form the reader descended); put the
    // source `#xD#xA`/`#xD` back at each recorded LF ordinal. Empty form ⇒ no-op,
    // so a pure-`#xA` source is byte-identical to the pre-EOL-form serializer.
    expand_end_of_line_form(out.into_bytes(), decisions.eol_form())
}

/// Invert the §2.11 \[2.11\] End-of-Line normalization the reader applied: at the
/// `#xA` byte with each recorded LF ORDINAL, re-expand back to the source form
/// (`#xD#xA` for [`EolKind::Crlf`], `#xD` for [`EolKind::Cr`]).
///
/// Operates on the FINISHED `normalized` byte stream — the one
/// [`serialize_document_exact`] just built. The serializer emits every `#xA`
/// LITERALLY and in order (char-data does not escape `#xA`; the prolog/epilog
/// `Misc*` and inter-element `S` runs are verbatim), so the k-th `#xA` byte here
/// is the SAME line break the reader recorded at LF ordinal `k` — even though a
/// §4.6 `&`/`<`/`>` escape shifted byte offsets in between. Keying by ordinal (not
/// byte offset) is therefore robust against every other escaping decision, fully
/// generic (prolog, char-data, inter-element `S` all re-expand identically), and
/// additive: an empty form returns `normalized` unchanged, so a pure-`#xA` source
/// (WordNet) is untouched.
///
/// Walks the output byte-by-byte, counting `#xA`s; at a `#xA` whose ordinal is in
/// the recorded set, emits the source form instead. The recorded ordinals are
/// ascending (the reader pushed them in document order), so a single advancing
/// cursor over them suffices. The result length is `normalized.len()` plus the
/// count of `Crlf` breaks (one `#xD` re-inserted each; a `Cr` rewrites the `#xA`
/// in place, adding nothing).
fn expand_end_of_line_form(normalized: Vec<u8>, eol_form: &EndOfLineForm) -> Vec<u8> {
    if eol_form.is_empty() {
        // Additive fast path: nothing to re-expand, return the bytes by move.
        return normalized;
    }
    let recorded = eol_form.eols.as_slice();
    // One `#xD` re-inserted per `Crlf`; a `Cr` replaces `#xA` in place.
    let crlf_count = recorded
        .iter()
        .filter(|(_, kind)| matches!(kind, EolKind::Crlf))
        .count();
    let mut out = Vec::with_capacity(normalized.len() + crlf_count);
    let mut lf_ordinal = 0usize;
    let mut next = 0usize;
    for &byte in &normalized {
        if byte == b'\n' {
            if next < recorded.len() && recorded[next].0 == lf_ordinal {
                // This `#xA` is a §2.11-collapsed break — emit the source form.
                match recorded[next].1 {
                    // `#xD#xA` (CRLF): re-insert the `#xD`, then keep the `#xA`.
                    EolKind::Crlf => {
                        out.push(b'\r');
                        out.push(b'\n');
                    }
                    // lone `#xD` (CR): the source had no `#xA` here, only `#xD`.
                    EolKind::Cr => out.push(b'\r'),
                }
                next += 1;
            } else {
                // A source-literal `#xA` (no form to put back) — keep it.
                out.push(b'\n');
            }
            lf_ordinal += 1;
        } else {
            out.push(byte);
        }
    }
    // Every recorded ordinal must have landed on an `#xA` in the output — a
    // reader/writer LF-count disagreement (e.g. a recorded break inside an
    // attribute value, the out-of-scope §3.3.3 case) would leave entries
    // unconsumed. Guarded without panicking on the release path.
    debug_assert_eq!(
        next,
        recorded.len(),
        "every recorded §2.11 EOL ordinal must index an `#xA` in the serialized output \
         (an `#xA` inside an attribute value is the out-of-scope §3.3.3 residue)"
    );
    out
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
    let node_decisions = decisions.get(my_index);
    let intra_ws = node_decisions.and_then(|d| d.intra_tag_whitespace.as_ref());
    let attr_entity_refs = node_decisions.map(|d| &d.attr_entity_refs);

    let start_tag_order = node_decisions.and_then(|d| d.start_tag_order.as_deref());
    out.push('<');
    write_name(out, &el.name);
    write_start_tag_attributes(out, el, intra_ws, attr_entity_refs, start_tag_order);
    if el.children.is_empty() {
        // The empty-element form is a recorded decision; default to the
        // canonical self-closing form when none is recorded.
        let explicit = matches!(
            node_decisions.and_then(|d| d.empty_form),
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
        // Track the child ordinal so a `Text` child's §4.6 entity-reference
        // form (keyed by that ordinal in `NodeDecisions::text_entity_refs`)
        // re-emits at the right child — the same ordinal the reader assigned at
        // `flush_text_capturing`.
        for (child_ordinal, child) in el.children.iter().enumerate() {
            let text_refs = node_decisions.and_then(|d| d.text_entity_refs.get(&child_ordinal));
            write_node_exact(out, child, index, decisions, text_refs);
        }
        out.push_str("</");
        write_name(out, &el.name);
        out.push('>');
    }
}

/// Re-emit the `(S Attribute)* S?` portion of the start-tag (§3.1 \[40\]/\[44\])
/// — honouring the captured intra-tag white-space layout and per-attribute §4.6
/// entity-reference forms. By default it emits namespaces then attributes; when
/// `start_tag_order` is present (an INTERLEAVED start-tag — the USC `<uscDoc>`
/// root) it emits the tokens in that EXACT source order instead, and the per-slot
/// white-space / entity-ref runs key by that order. With no decision it falls
/// back to the canonical single-space separation, so canonical tags are
/// byte-identical to [`write_element`].
fn write_start_tag_attributes(
    out: &mut String,
    el: &XmlElement,
    intra_ws: Option<&IntraTagWhitespace>,
    attr_entity_refs: Option<&BTreeMap<usize, EntityReferenceForm>>,
    start_tag_order: Option<&[StartTagToken]>,
) {
    // The attribute-like emit sequence the reader keyed its per-slot captures by.
    // When `start_tag_order` carries the exact source-order token sequence (an
    // interleaved start-tag), use it verbatim; otherwise the canonical
    // ns-then-attr order — every `xmlns`/`xmlns:prefix` declaration then every
    // regular attribute (mirroring `write_element`'s order). `namespaces` falls
    // back to the single `namespace` slot when empty, exactly as the canonical
    // writer does.
    enum Slot<'a> {
        Ns(&'a XmlNamespace),
        Attr(&'a XmlAttribute),
    }
    let mut slots: Vec<Slot<'_>> = Vec::new();
    if let Some(order) = start_tag_order {
        for token in order {
            match token {
                StartTagToken::Namespace(ns) => slots.push(Slot::Ns(ns)),
                StartTagToken::Attribute(attr) => slots.push(Slot::Attr(attr)),
            }
        }
    } else {
        if el.namespaces.is_empty() {
            if let Some(ns) = &el.namespace {
                slots.push(Slot::Ns(ns));
            }
        } else {
            for ns in &el.namespaces {
                slots.push(Slot::Ns(ns));
            }
        }
        for attr in &el.attributes {
            slots.push(Slot::Attr(attr));
        }
    }

    // The captured intra-tag white-space, when present, carries one
    // `before_attr` / `around_eq` entry per emitted slot. A mismatched length
    // would mean the reader and writer disagree on the attribute-like sequence;
    // the `start_tag_order` complement keeps them aligned even when the source
    // interleaves `xmlns` declarations with regular attributes. Guard it.
    if let Some(iw) = intra_ws {
        debug_assert_eq!(
            iw.before_attr.len(),
            slots.len(),
            "intra-tag white-space slot count must match the emitted attribute-like \
             sequence (start_tag_order keeps interleaved tags aligned)"
        );
        debug_assert_eq!(iw.around_eq.len(), slots.len());
    }

    for (slot_index, slot) in slots.iter().enumerate() {
        // §3.1 [40]/[44] leading `S` of this `(S Attribute)` group: the captured
        // run, or the canonical single space when no layout was recorded.
        match intra_ws.and_then(|iw| iw.before_attr.get(slot_index)) {
            Some(run) => out.push_str(run),
            None => out.push(' '),
        }
        let (name_to_eq, eq_to_value) = intra_ws
            .and_then(|iw| iw.around_eq.get(slot_index))
            .map_or(("", ""), |(a, b)| (a.as_str(), b.as_str()));
        let value_refs = attr_entity_refs.and_then(|m| m.get(&slot_index));
        match slot {
            Slot::Ns(ns) => {
                // Namespaces in XML 1.0 (Bray, Hollander, Layman & Tobin 2009)
                // §3 — `xmlns` / `xmlns:prefix`. The §3.1 [25] `Eq` white-space
                // straddles the `=` exactly as for a regular attribute.
                match &ns.prefix {
                    Some(prefix) => {
                        out.push_str("xmlns:");
                        out.push_str(prefix);
                    }
                    None => out.push_str("xmlns"),
                }
                out.push_str(name_to_eq);
                out.push('=');
                out.push_str(eq_to_value);
                out.push('"');
                write_escaped_attr_value_exact(out, &ns.uri, value_refs);
                out.push('"');
            }
            Slot::Attr(attr) => {
                write_name(out, &attr.name);
                out.push_str(name_to_eq);
                out.push('=');
                out.push_str(eq_to_value);
                out.push('"');
                write_escaped_attr_value_exact(out, &attr.value, value_refs);
                out.push('"');
            }
        }
    }

    // §3.1 [40]/[44] trailing `S?` before the `>` or `/>` close. Empty for the
    // canonical tag; the captured run for a multi-line one.
    if let Some(iw) = intra_ws {
        out.push_str(&iw.before_close);
    }
}

fn write_node_exact(
    out: &mut String,
    node: &XmlNode,
    index: &mut usize,
    decisions: &SyntaxDecisions,
    text_refs: Option<&EntityReferenceForm>,
) {
    match node {
        XmlNode::Element(el) => write_element_exact(out, el, index, decisions),
        // Char data honours the captured §4.6 predefined-entity-reference form
        // (which resolved chars were written `&apos;`/`&quot;`/… in the source);
        // with none recorded it falls back to the canonical escaper.
        XmlNode::Text(t) => write_escaped_char_data_exact(out, t, text_refs),
        // Other non-element nodes render as in the canonical serializer; their
        // own byte-exact residue (comment white-space, CDATA) lands in follow-up
        // decisions. They occupy no pre-order ELEMENT slot, so the counter is
        // untouched.
        other => write_node(out, other),
    }
}

/// W3C XML 1.0 §2.8 production \[28\] `doctypedecl` — inverse of
/// [`grammar::parse_doctype`](super::grammar). Re-emits the root
/// element name, optional `ExternalID`, and inline general entity
/// declarations (`<!ENTITY name "value">`) the parser projected.
fn write_doctype(out: &mut String, doctype: &XmlDoctype) {
    // Byte-exact PROLOG residue: when the read path captured the whole
    // declaration verbatim (the `<!ENTITY>` internal-subset layout/white-space/
    // comments the structured projection erases), reproduce it exactly — the
    // analogue of re-emitting the `<?xml?>` declaration bytes. NOT a stored
    // element-tree DOM; the element backbone still regenerates from the graph.
    if let Some(verbatim) = &doctype.verbatim {
        out.push_str(verbatim);
        return;
    }
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
        // The byte-exact `verbatim` path returned early above; this structured
        // re-projection (canonical compact layout) is only for a synthetic
        // doctype built without capture.
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
        write_escaped_char_data_char(out, ch);
    }
}

/// One char of `CharData` in the canonical (C14N 1.1 §3.5) escaping.
fn write_escaped_char_data_char(out: &mut String, ch: char) {
    match ch {
        '&' => out.push_str("&amp;"),
        '<' => out.push_str("&lt;"),
        '>' => out.push_str("&gt;"),
        _ => out.push(ch),
    }
}

/// Byte-exact `CharData` escaper — honours the captured §4.6
/// predefined-entity-reference FORM the same way
/// [`write_escaped_attr_value_exact`] does for attribute values: at each char
/// index the reader recorded a reference for, re-emit the source `&name;` form
/// (so a `&quot;` adjacent to a curly quote round-trips). With no refs it is
/// byte-identical to [`write_escaped_char_data`].
fn write_escaped_char_data_exact(out: &mut String, s: &str, refs: Option<&EntityReferenceForm>) {
    write_with_entity_refs(out, s, refs, write_escaped_char_data_char);
}

/// Walk `s` by CHAR INDEX, re-emitting the captured §4.6 predefined-entity
/// reference at each recorded index and falling back to `canonical_char` (the
/// position-free canonical escaper) everywhere else. Shared by the attribute and
/// char-data byte-exact escapers; the only difference between them is the
/// canonical fallback. The recorded `refs` are in ascending char-index order
/// (the reader pushes them in resolution order), so a single advancing cursor
/// over them suffices.
fn write_with_entity_refs(
    out: &mut String,
    s: &str,
    refs: Option<&EntityReferenceForm>,
    canonical_char: fn(&mut String, char),
) {
    let recorded = refs.map(|r| r.refs.as_slice()).unwrap_or(&[]);
    let ext = refs.map(|r| r.ext_refs.as_slice()).unwrap_or(&[]);
    // Fast path: no reference forms at all — the canonical escaper verbatim. This
    // is the no-op additive case for every WordNet/cito/biro/c4o/doco value.
    if recorded.is_empty() && ext.is_empty() {
        for ch in s.chars() {
            canonical_char(out, ch);
        }
        return;
    }
    let mut next = 0usize; // cursor into the §4.6 predefined `recorded`
    let mut next_ext = 0usize; // cursor into the §4.1 numeric/general `ext`
    // §4.1 general-entity expansions span multiple resolved chars; when one is
    // re-emitted as `&name;`, skip the remaining `expansion_chars - 1` chars it
    // covers. `skip` counts those pending skips.
    let mut skip = 0usize;
    for (char_index, ch) in s.chars().enumerate() {
        if skip > 0 {
            // Inside a general-entity expansion already re-emitted as `&name;`.
            skip -= 1;
            continue;
        }
        if next < recorded.len() && recorded[next].0 == char_index {
            // §4.6 predefined entity reference — re-emit the exact reference text.
            // `resolved_char` is the inverse of the recorded char, asserted to
            // agree as a capture-integrity check.
            let entity = recorded[next].1;
            debug_assert_eq!(
                entity.resolved_char(),
                ch,
                "recorded §4.6 entity reference does not resolve to the char at its \
                 captured index — reader/writer char-index disagreement"
            );
            out.push_str(entity.reference());
            next += 1;
        } else if next_ext < ext.len() && ext[next_ext].char_index == char_index {
            // §4.1 numeric (`&#39;`) or general-entity (`&rdfs;`) reference.
            match &ext[next_ext].kind {
                super::source_syntax::ExtendedRefKind::Numeric {
                    hex,
                    upper_hex,
                    digits,
                } => {
                    if *hex {
                        out.push_str(if *upper_hex { "&#X" } else { "&#x" });
                    } else {
                        out.push_str("&#");
                    }
                    out.push_str(digits);
                    out.push(';');
                }
                super::source_syntax::ExtendedRefKind::General {
                    name,
                    expansion_chars,
                } => {
                    out.push('&');
                    out.push_str(name);
                    out.push(';');
                    // This char plus the next `expansion_chars - 1` are the
                    // entity's expansion — skip them (already emitted as `&name;`).
                    skip = expansion_chars.saturating_sub(1);
                }
            }
            next_ext += 1;
        } else {
            canonical_char(out, ch);
        }
    }
    debug_assert_eq!(
        next,
        recorded.len(),
        "every recorded §4.6 entity reference must land on a char index within the value"
    );
    debug_assert_eq!(
        next_ext,
        ext.len(),
        "every recorded §4.1 numeric/general reference must land on a char index within the value"
    );
}

/// W3C XML 1.0 §3.3.3 — attribute-value normalization defines
/// the characters that must be escaped inside `AttValue`. We
/// escape the C14N 1.1 §3.5 set (`&`, `<`, `"`, `\r`, `\n`, `\t`)
/// so the put output round-trips through canonicalization stably.
fn write_escaped_attr_value(out: &mut String, s: &str) {
    for ch in s.chars() {
        write_escaped_attr_char(out, ch);
    }
}

/// One char of an `AttValue` in the canonical (C14N 1.1 §3.5) escaping.
fn write_escaped_attr_char(out: &mut String, ch: char) {
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

/// Byte-exact `AttValue` escaper — honours the captured §4.6
/// predefined-entity-reference FORM (W3C XML 1.0 §4.6): at each char index the
/// reader recorded a reference for, re-emit the SOURCE form (`&apos;`, `&quot;`,
/// …) instead of the canonical escape, so a source `&apos;hood` round-trips
/// (the parser resolved it to a bare `'`, which the canonical escaper would
/// emit unescaped). Indices are char positions into the resolved value (the
/// reader counted chars, not bytes — refs sit next to multibyte chars). Every
/// other char uses the canonical escaper, so with no refs this is byte-identical
/// to [`write_escaped_attr_value`].
fn write_escaped_attr_value_exact(out: &mut String, s: &str, refs: Option<&EntityReferenceForm>) {
    write_with_entity_refs(out, s, refs, write_escaped_attr_char);
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

    #[pr4xis::praxis_value(Deterministic)]
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
                ..NodeDecisions::default()
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

    #[pr4xis::praxis_value(Deterministic)]
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
                ..NodeDecisions::default()
            },
        );
        assert_eq!(
            serialize_document_exact(&d, &decisions),
            br#"<?xml version="1.0" encoding="UTF-8"?><root><a></a></root>"#.to_vec()
        );
    }
}
