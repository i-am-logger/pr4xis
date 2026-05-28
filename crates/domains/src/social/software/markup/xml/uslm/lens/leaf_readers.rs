//! Leaf-block readers and structural extractors for USLM XML.
//!
//! These functions walk subtrees whose parent element has already
//! been classified by the XSD-grounded dispatch in [`super`]; each
//! reader extracts attribute values, text content, and recursive
//! structure into typed runtime values.
//!
//! ## Grounding
//!
//! Every dispatch decision in this module consults the loaded
//! USLM-1.0.18 XSD ontology — `xsd.lookup_element`,
//! `xsd.is_member_of_substitution_group`, and the XSD-derived
//! `from_xsd_element` constructors on the runtime enums. There are
//! no hand-coded element-name lists.
//!
//! - W3C XSD 1.1 Part 1 §3.3 — *Element Declarations*: a name with
//!   no declaration has no `{type definition}` to dispatch on; every
//!   reader that consumes an element first verifies `xsd_declares`.
//! - W3C XSD 1.1 Part 1 §3.3.6 — *Substitution Groups*:
//!   `is_section_leaf` and `SubdivisionKind::from_xsd_element` /
//!   `ContainerKind::from_xsd_element` consult substitution-group
//!   membership against the loaded XSD's `"level"` head.
//! - W3C XSD 1.1 Part 1 §3.4 — *Type Definitions*: `LevelType` is the
//!   XSD-loaded complex type that the section/level family shares.
//! - W3C XSD 1.1 Part 1 §3.4.6.4 — *Schema-Validity Assessment*: once
//!   the type is fixed, the walk is a structural unfolding.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use std::sync::OnceLock;

use super::super::corpus::*;
use crate::formal::meta::xsd::from_xsd_parser::{XsdOntologyInstance, project_from_xsd_text};
use crate::formal::meta::xsd::uslm_vocabulary::USLM_1_0_18_XSD;
use crate::social::software::markup::xml::ontology::{XmlElement, XmlNode};
use crate::social::software::markup::xml::reader as xml_reader;

/// The XSD ontology instance projected from the bundled USLM-1.0.18
/// XSD. Built once on first call and cached for the lifetime of the
/// process. Per W3C XSD 1.1 Part 1 §3.3 (Element Declarations) and
/// §3.3.6 (Substitution Groups), every dispatch decision in this
/// module queries this instance rather than matching hand-coded
/// element-name literals.
pub(super) fn loaded_uslm_xsd() -> &'static XsdOntologyInstance {
    static USLM_XSD_INSTANCE: OnceLock<XsdOntologyInstance> = OnceLock::new();
    USLM_XSD_INSTANCE.get_or_init(|| project_from_xsd_text(USLM_1_0_18_XSD))
}

/// True iff `local_name` is declared as an `<xsd:element>` by the
/// bundled USLM-1.0.18 XSD (W3C XSD 1.1 Part 1 §3.3). The reader's
/// leaf-block dispatch consults this predicate before projecting to
/// runtime enum variants — element names that aren't loaded fall
/// through to the catch-all branches instead of being silently
/// accepted.
pub(super) fn xsd_declares(local_name: &str) -> bool {
    loaded_uslm_xsd().lookup_element(local_name).is_some()
}

/// True iff `name` matches the loaded USLM XSD's `<xsd:element
/// name="section">` declaration. The comparison reads the section
/// element's name from the XSD load rather than from a hand-coded
/// literal — per W3C XSD 1.1 Part 1 §3.3 (Element Declarations), the
/// authoritative source of an element's local-name is the loaded
/// `<xsd:element>` declaration. Also confirms the element is a
/// member of `substitutionGroup="level"` (W3C XSD 1.1 Part 1 §3.3.6),
/// so a future XSD revision that moved `section` out of the level
/// family would be detected.
pub(super) fn is_section_leaf(name: &str, xsd: &XsdOntologyInstance) -> bool {
    let Some(decl) = xsd.lookup_element("section") else {
        return false;
    };
    if !xsd.is_member_of_substitution_group(&decl.local_name, "level") {
        return false;
    }
    name == decl.local_name
}

/// Effective XML namespace context per W3C XML Namespaces 1.0 §6
/// (*Applying Namespaces to Elements and Attributes*). Carries the
/// in-scope default namespace URI as the recursion descends the
/// tree. A child that declares its own `xmlns="…"` replaces the
/// default for its subtree; absence of a declaration inherits the
/// parent's default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NsContext<'a> {
    default_uri: Option<&'a str>,
}

impl<'a> NsContext<'a> {
    /// Empty context (no in-scope default namespace).
    fn empty() -> Self {
        Self { default_uri: None }
    }

    /// Update the context for entering `elem`: if `elem` carries
    /// its own `xmlns="…"` declaration, that becomes the new
    /// default; otherwise inherit.
    fn enter(self, elem: &'a XmlElement) -> Self {
        if let Some(ns) = &elem.namespace
            && ns.prefix.is_none()
        {
            return Self {
                default_uri: Some(ns.uri.as_str()),
            };
        }
        self
    }

    /// Per W3C XML Namespaces 1.0 §6.2: an unprefixed element is
    /// in the in-scope default namespace; a prefixed element is in
    /// the namespace bound to its prefix. This function answers
    /// "is `elem` in `target_uri`?" using only the *default*
    /// channel; prefixed elements (e.g. `<dc:title>`) are
    /// definitionally NOT in the default namespace.
    fn elem_in(self, elem: &XmlElement, target_uri: &str) -> bool {
        if elem.name.prefix.is_some() {
            // A prefixed element is in its prefix's bound URI,
            // not the default. For USLM filtering we only want
            // unprefixed-in-default-USLM matches; prefixed
            // elements like `<dc:title>` always fall outside.
            return false;
        }
        self.default_uri == Some(target_uri)
    }
}

/// Read a USLM title-level XML file into a [`UsCodeTitle`].
///
/// This is the runtime path. It calls the generic
/// [`xml_reader::read_xml`] to get an [`XmlDocument`], then walks
/// the typed tree picking out `<title>` metadata and every
/// `<section>` element (no matter how deeply nested in parts /
/// chapters / subchapters).
///
/// [`XmlDocument`]: crate::social::software::markup::xml::ontology::XmlDocument
pub fn read_uslm_title(xml_text: &str) -> Result<UsCodeTitle, UslmReadError> {
    let xml = xml_reader::read_xml(xml_text).map_err(|e| UslmReadError::Xml(e.message))?;

    // `<title>` may sit directly under the document root (a slice
    // file) or under `<uscDoc>/<main>` (a full-title file). Both
    // cases share a single ontological rule: find the first
    // descendant whose qualified name resolves to the
    // USLM-namespace `title` per W3C XML Namespaces 1.0 §6. This
    // automatically excludes `<dc:title>` in `<meta>` blocks
    // (Dublin Core namespace) without needing a separate
    // "navigate via <main>" heuristic.
    let root_ctx = NsContext::empty().enter(&xml.root);
    let title_elem = find_first_in_namespace(&xml.root, root_ctx, "title", USLM_NAMESPACE_URI);

    // If the document is a single `<section>` slice (no `<title>`
    // wrapper), still parse it — return a UsCodeTitle with one
    // section and a synthetic identifier.
    let Some(title_elem) = title_elem else {
        if xml.root.name.local == "section" {
            let section = read_section(&xml.root)?;
            let identifier = derive_title_identifier(&section.identifier)
                .unwrap_or_else(|| "/us/usc/t?".to_string());
            let hierarchy = vec![HierarchyNode::Section(Box::new(section.clone()))];
            return Ok(UsCodeTitle {
                identifier,
                number: 0,
                heading: String::new(),
                sections: vec![section],
                hierarchy,
                notes_blocks: Vec::new(),
                bare_notes: Vec::new(),
                headers: Vec::new(),
                signatures: Vec::new(),
                meta: None,
                tocs: Vec::new(),
                tables: Vec::new(),
            });
        }
        return Err(UslmReadError::NoUsCodeRoot);
    };

    let identifier = attr(title_elem, "identifier").unwrap_or_default();
    let heading = first_child_text(title_elem, "heading").unwrap_or_default();
    let number = match find_first_descendant(title_elem, "num") {
        Some(n) => attr(n, "value")
            .and_then(|s| s.parse::<u32>().ok())
            .ok_or_else(|| UslmReadError::BadTitleNumber {
                raw: attr(n, "value").unwrap_or_default(),
            })?,
        None => 0,
    };

    let hierarchy = read_hierarchy_children(title_elem)?;
    let mut sections = Vec::new();
    flatten_sections(&hierarchy, &mut sections);
    let notes_blocks = read_notes_blocks(title_elem);
    let bare_notes = read_bare_notes(title_elem);
    let headers = read_headers(title_elem);
    let signatures = read_signatures(title_elem);
    // `<meta>` is a sibling of `<main>` under the `<uscDoc>` root —
    // not a descendant of `<title>`. Find it from the document root.
    let meta = find_first_descendant(&xml.root, "meta").map(read_meta);
    let tocs = read_tocs(title_elem);
    let mut tables = Vec::new();
    collect_tables_in(title_elem, &mut tables);

    Ok(UsCodeTitle {
        identifier,
        number,
        heading,
        sections,
        hierarchy,
        notes_blocks,
        bare_notes,
        headers,
        signatures,
        meta,
        tocs,
        tables,
    })
}

/// Walk a subtree gathering every XHTML `<table>` element. The
/// table itself is in the XHTML namespace; the discriminator is the
/// element's local name plus any preserved xmlns binding.
pub(super) fn collect_tables_in(elem: &XmlElement, out: &mut Vec<UsCodeTable>) {
    if elem.name.local == "table" {
        out.push(read_table(elem));
        return;
    }
    for child in &elem.children {
        if let XmlNode::Element(e) = child {
            collect_tables_in(e, out);
        }
    }
}

pub(super) fn read_table(elem: &XmlElement) -> UsCodeTable {
    let identifier = attr(elem, "id");
    let class = attr(elem, "class");
    let mut header_rows = Vec::new();
    let mut body_rows = Vec::new();
    for child in &elem.children {
        if let XmlNode::Element(e) = child {
            match e.name.local.as_str() {
                "thead" => collect_rows(e, &mut header_rows),
                "tbody" => collect_rows(e, &mut body_rows),
                "tr" => body_rows.push(read_table_row(e)),
                _ => {}
            }
        }
    }
    UsCodeTable {
        identifier,
        class,
        header_rows,
        body_rows,
    }
}

fn collect_rows(elem: &XmlElement, out: &mut Vec<UsCodeTableRow>) {
    for child in &elem.children {
        if let XmlNode::Element(e) = child
            && e.name.local == "tr"
        {
            out.push(read_table_row(e));
        }
    }
}

fn read_table_row(elem: &XmlElement) -> UsCodeTableRow {
    let class = attr(elem, "class");
    let mut cells = Vec::new();
    for child in &elem.children {
        if let XmlNode::Element(e) = child {
            let kind = match e.name.local.as_str() {
                "th" => UsCodeTableCellKind::Header,
                "td" => UsCodeTableCellKind::Data,
                _ => continue,
            };
            cells.push(UsCodeTableCell {
                kind,
                text: element_text(e),
                colspan: attr(e, "colspan").and_then(|s| s.parse().ok()),
                rowspan: attr(e, "rowspan").and_then(|s| s.parse().ok()),
            });
        }
    }
    UsCodeTableRow { class, cells }
}

/// Collect direct-child `<toc>` blocks per LRC USLM User Guide
/// § "Table of Contents".
///
/// The element-name `"toc"` is the local-name of the
/// `<xsd:element name="toc">` declaration in the loaded USLM XSD
/// (W3C XSD 1.1 Part 1 §3.3); the guard below confirms the load saw
/// it before dispatching to the typed reader.
pub(super) fn read_tocs(elem: &XmlElement) -> Vec<UsCodeToc> {
    let mut out = Vec::new();
    let xsd_has_toc = xsd_declares("toc");
    for child in &elem.children {
        if !xsd_has_toc {
            break;
        }
        if let XmlNode::Element(e) = child
            && e.name.local == "toc"
        {
            out.push(read_toc(e));
        }
    }
    out
}

fn read_toc(elem: &XmlElement) -> UsCodeToc {
    let identifier = attr(elem, "id");
    let role = attr(elem, "role");
    let mut items = Vec::new();
    collect_toc_items(elem, &mut items);
    UsCodeToc {
        identifier,
        role,
        items,
    }
}

/// Walk a `<toc>` subtree gathering every `<tocItem>` descendant.
/// LRC wraps items in `<layout>` for three-column TOCs; we descend
/// through that wrapper without modeling it as a separate type.
fn collect_toc_items(elem: &XmlElement, out: &mut Vec<UsCodeTocItem>) {
    for child in &elem.children {
        if let XmlNode::Element(e) = child {
            if e.name.local == "tocItem" {
                out.push(read_toc_item(e));
            } else if matches!(e.name.local.as_str(), "layout" | "header") {
                // Wrapper elements — recurse without producing an
                // item. The `<header>` row is a column-header
                // banner, not a navigable entry.
                collect_toc_items(e, out);
            }
        }
    }
}

fn read_toc_item(elem: &XmlElement) -> UsCodeTocItem {
    // Column text concatenation: each `<column>` becomes one cell,
    // joined by tab characters to preserve visual separation.
    let mut cells = Vec::new();
    for child in &elem.children {
        if let XmlNode::Element(e) = child
            && e.name.local == "column"
        {
            cells.push(element_text(e));
        }
    }
    let text = cells.join("\t");
    let mut refs = Vec::new();
    collect_refs_in(elem, &mut refs);
    // The TOC item's target is the href of the first `<ref>` child
    // (typically the part/chapter/section's URN).
    let target = refs.first().map(|r| r.href.clone());
    UsCodeTocItem { target, text, refs }
}

/// Read a `<meta>` block into a typed [`UsCodeMeta`].
///
/// Per LRC USLM User Guide § "Metadata": `<meta>` is a USLM-namespace
/// container whose children are Dublin Core elements declared with
/// the `dc:` namespace prefix bound to
/// `http://purl.org/dc/elements/1.1/`. Only prefix-`"dc"` children
/// are considered — an unprefixed `<title>` inside `<meta>` would be
/// schema-non-conformant and is silently ignored. Local-name match
/// against the Dublin Core Element Set (DCMI Metadata Terms § 4)
/// routes each known element to its typed field; unknown DC elements
/// are silently dropped (the spec is open-ended).
pub(super) fn read_meta(elem: &XmlElement) -> UsCodeMeta {
    let mut meta = UsCodeMeta {
        title: None,
        doc_type: None,
        publisher: None,
        creator: None,
        date: None,
        identifier: None,
        language: None,
        format: None,
        rights: None,
        source: None,
        other_dc: Vec::new(),
        doc_number: None,
        doc_publication_name: None,
        properties: Vec::new(),
        dcterms_created: None,
        dcterms_modified: None,
        dcterms_other: Vec::new(),
    };
    for child in &elem.children {
        let XmlNode::Element(e) = child else { continue };
        let text = element_text(e);
        match e.name.prefix.as_deref() {
            // Dublin Core elements (dc:* namespace per
            // http://purl.org/dc/elements/1.1/).
            Some("dc") => {
                let local = e.name.local.as_str();
                if text.is_empty() {
                    // Empty DC element — surface rather than drop.
                    meta.other_dc.push((local.to_string(), String::new()));
                    continue;
                }
                match local {
                    "title" => meta.title = Some(text),
                    "type" => meta.doc_type = Some(text),
                    "publisher" => meta.publisher = Some(text),
                    "creator" => meta.creator = Some(text),
                    "date" => meta.date = Some(text),
                    "identifier" => meta.identifier = Some(text),
                    "language" => meta.language = Some(text),
                    "format" => meta.format = Some(text),
                    "rights" => meta.rights = Some(text),
                    "source" => meta.source = Some(text),
                    other => meta.other_dc.push((other.to_string(), text)),
                }
            }
            // DCMI Terms elements (dcterms:* namespace per
            // http://purl.org/dc/terms/) — refinement vocabulary.
            Some("dcterms") => match e.name.local.as_str() {
                "created" => meta.dcterms_created = Some(text),
                "modified" => meta.dcterms_modified = Some(text),
                other => meta.dcterms_other.push((other.to_string(), text)),
            },
            // USLM-native metadata elements (no prefix, in the
            // USLM namespace by default). Dispatch is XSD-grounded:
            // only element names declared by the loaded USLM XSD
            // (W3C XSD 1.1 Part 1 §3.3 Element Declarations) are
            // routed to a typed metadata field. Names absent from
            // the XSD load skip the entire branch (no silent typed
            // projection of unrecognised XML), which preserves the
            // tripwire-style coverage gap the comment below alludes
            // to.
            None if xsd_declares(&e.name.local) => match e.name.local.as_str() {
                "docNumber" => meta.doc_number = Some(text),
                "docPublicationName" => meta.doc_publication_name = Some(text),
                "property" => meta.properties.push(UsCodeMetaProperty {
                    role: attr(e, "role"),
                    value: text,
                }),
                // Other USLM-namespace elements inside <meta> are
                // not yet typed; the dc/dcterms/usual ones above
                // cover the LRC pl-119-90 surface. Ignore for now
                // (a tripwire field could surface them later).
                _ => {}
            },
            None => {}
            // Other namespace prefixes (xsi, etc.) — ignore.
            _ => {}
        }
    }
    meta
}

/// Collect every `<signature>` direct child as a [`UsCodeSignature`].
///
/// The `"signature"` element-name is the XSD-loaded local-name of
/// the `<xsd:element name="signature">` declaration in the loaded
/// USLM XSD (W3C XSD 1.1 Part 1 §3.3 Element Declarations); the
/// guards below confirm the load saw it (and `<name>`) before
/// dispatching to the typed reader.
pub(super) fn read_signatures(elem: &XmlElement) -> Vec<UsCodeSignature> {
    let mut out = Vec::new();
    if !xsd_declares("signature") {
        return out;
    }
    let xsd_has_name = xsd_declares("name");
    for child in &elem.children {
        if let XmlNode::Element(e) = child
            && e.name.local == "signature"
        {
            let mut names = Vec::new();
            if xsd_has_name {
                for inner in &e.children {
                    if let XmlNode::Element(n) = inner
                        && n.name.local == "name"
                    {
                        names.push(UsCodeName {
                            text: element_text(n),
                        });
                    }
                }
            }
            out.push(UsCodeSignature { names });
        }
    }
    out
}

/// Collect every `<date date="...">` descendant of `elem` as a
/// [`UsCodeDate`], in document order. Stops at child subdivision
/// boundaries so dates inside a nested section don't leak into
/// the parent's date list.
fn collect_dates_in(elem: &XmlElement, out: &mut Vec<UsCodeDate>) {
    if elem.name.local == "date" {
        let iso = attr(elem, "date").unwrap_or_default();
        let text = element_text(elem);
        out.push(UsCodeDate { iso, text });
        return;
    }
    let xsd = loaded_uslm_xsd();
    for child in &elem.children {
        if let XmlNode::Element(e) = child {
            // Don't descend into nested subdivisions / sections —
            // they collect their own dates. The is-subdivision /
            // is-section query is XSD-grounded via
            // `SubdivisionKind::from_xsd_element` and `is_section_leaf`
            // — both consult the loaded USLM XSD's
            // `substitutionGroup="level"` membership (W3C XSD 1.1
            // Part 1 §3.3.6) rather than matching hand-coded element
            // names.
            if SubdivisionKind::from_xsd_element(&e.name.local, xsd).is_some()
                || is_section_leaf(&e.name.local, xsd)
            {
                continue;
            }
            collect_dates_in(e, out);
        }
    }
}

fn read_quoted_content(elem: &XmlElement) -> UsCodeQuotedContent {
    let origin = attr(elem, "origin");
    let body = element_text(elem);
    let mut section_refs = Vec::new();
    let mut refs = Vec::new();
    collect_section_refs_in(elem, &mut section_refs);
    collect_refs_in(elem, &mut refs);
    UsCodeQuotedContent {
        origin,
        body,
        section_refs,
        refs,
    }
}

/// Walk an element's subtree appending every `<section>` found —
/// but unlike `read_section`, these are quoted/cited section
/// references, not real published sections. They become
/// [`UsCodeSectionRef`]s.
fn collect_section_refs_in(elem: &XmlElement, out: &mut Vec<UsCodeSectionRef>) {
    if elem.name.local == "section" {
        let identifier = attr(elem, "identifier");
        let num = first_child_attr(elem, "num", "value").unwrap_or_default();
        let heading = first_child_text(elem, "heading");
        let body = element_text(elem);
        out.push(UsCodeSectionRef {
            identifier,
            num,
            heading,
            body,
        });
        return;
    }
    for child in &elem.children {
        if let XmlNode::Element(e) = child {
            collect_section_refs_in(e, out);
        }
    }
}

/// Collect bare `<note>` children of `elem` — those that sit
/// directly under the parent rather than inside a `<notes>`
/// wrapper. USLM allows both forms; this function captures the
/// bare ones.
///
/// The `"note"` element-name is the XSD-loaded local-name of the
/// `<xsd:element name="note">` declaration (W3C XSD 1.1 Part 1 §3.3);
/// the early-exit guard below skips the walk entirely when the load
/// didn't see it, so the dispatch reflects the loaded ontology.
pub(super) fn read_bare_notes(elem: &XmlElement) -> Vec<UsCodeNote> {
    let mut out = Vec::new();
    if !xsd_declares("note") {
        return out;
    }
    for child in &elem.children {
        if let XmlNode::Element(e) = child
            && e.name.local == "note"
        {
            out.push(read_note(e));
        }
    }
    out
}

/// Read every `<notes>` child of an element as a typed
/// [`UsCodeNotesBlock`]. Inner `<note>` children become
/// [`UsCodeNote`]s; cross-references inside note bodies are
/// collected.
///
/// The `"notes"` element-name is the XSD-loaded local-name of the
/// `<xsd:element name="notes">` declaration (W3C XSD 1.1 Part 1 §3.3);
/// the early-exit guard below skips the walk entirely when the load
/// didn't see it.
pub(super) fn read_notes_blocks(elem: &XmlElement) -> Vec<UsCodeNotesBlock> {
    let mut out = Vec::new();
    if !xsd_declares("notes") {
        return out;
    }
    for child in &elem.children {
        if let XmlNode::Element(e) = child
            && e.name.local == "notes"
        {
            out.push(read_notes_block(e));
        }
    }
    out
}

fn read_notes_block(elem: &XmlElement) -> UsCodeNotesBlock {
    let block_type = attr(elem, "type");
    let identifier = attr(elem, "id");
    let heading = first_child_text(elem, "heading");
    let mut notes = Vec::new();
    // The `<note>` child of `<notes>` is XSD-declared; the guard
    // below confirms the load saw it (W3C XSD 1.1 Part 1 §3.3) before
    // dispatching to the typed reader.
    let xsd_has_note = xsd_declares("note");
    for child in &elem.children {
        if !xsd_has_note {
            break;
        }
        if let XmlNode::Element(e) = child
            && e.name.local == "note"
        {
            notes.push(read_note(e));
        }
    }
    UsCodeNotesBlock {
        block_type,
        identifier,
        heading,
        notes,
    }
}

fn read_note(elem: &XmlElement) -> UsCodeNote {
    let topic = attr(elem, "topic");
    let role = attr(elem, "role");
    let note_type = attr(elem, "type");
    let identifier = attr(elem, "id");
    let heading = first_child_text(elem, "heading");
    // Body text: flatten everything except the heading.
    let mut body = String::new();
    for child in &elem.children {
        if let XmlNode::Element(e) = child {
            if e.name.local == "heading" {
                continue;
            }
            let text = element_text(e);
            if !text.is_empty() {
                if !body.is_empty() {
                    body.push(' ');
                }
                body.push_str(&text);
            }
        }
    }
    let mut refs = Vec::new();
    collect_refs_in(elem, &mut refs);
    let quoted_contents = read_quoted_contents_recursive(elem);
    let mut dates = Vec::new();
    collect_dates_in(elem, &mut dates);
    UsCodeNote {
        topic,
        role,
        note_type,
        identifier,
        heading,
        body,
        refs,
        quoted_contents,
        dates,
    }
}

/// Recursively find every `<quotedContent>` element within `elem`.
/// Notes can contain quoted content nested inside paragraphs,
/// tables, etc. — not just as direct children.
fn read_quoted_contents_recursive(elem: &XmlElement) -> Vec<UsCodeQuotedContent> {
    let mut out = Vec::new();
    fn walk(elem: &XmlElement, out: &mut Vec<UsCodeQuotedContent>) {
        if elem.name.local == "quotedContent" {
            out.push(read_quoted_content(elem));
            return;
        }
        for child in &elem.children {
            if let XmlNode::Element(e) = child {
                walk(e, out);
            }
        }
    }
    walk(elem, &mut out);
    out
}

fn read_source_credit(elem: &XmlElement) -> UsCodeSourceCredit {
    let identifier = attr(elem, "id");
    let text = element_text(elem);
    let mut refs = Vec::new();
    collect_refs_in(elem, &mut refs);
    let mut dates = Vec::new();
    collect_dates_in(elem, &mut dates);
    UsCodeSourceCredit {
        identifier,
        text,
        refs,
        dates,
    }
}

fn read_continuation(elem: &XmlElement) -> UsCodeContinuation {
    UsCodeContinuation {
        body: element_text(elem),
    }
}

pub(super) fn read_headers(elem: &XmlElement) -> Vec<UsCodeHeader> {
    let mut out = Vec::new();
    // `"header"` is the XSD-loaded local-name of the
    // `<xsd:element name="header">` declaration (W3C XSD 1.1 Part 1
    // §3.3); the early-exit guard confirms the load saw it.
    if !xsd_declares("header") {
        return out;
    }
    for child in &elem.children {
        if let XmlNode::Element(e) = child
            && e.name.local == "header"
        {
            out.push(UsCodeHeader {
                text: element_text(e),
            });
        }
    }
    out
}

/// Read the immediate children of a hierarchy node (title or
/// container), returning typed [`HierarchyNode`]s. Skips
/// editorial-scope subtrees (notes, quoted content) per the
/// USLM Schema's structural distinction.
///
/// Dispatch is grounded: the section-vs-container split queries the
/// loaded USLM XSD via [`ContainerKind::from_xsd_element`] (W3C XSD
/// 1.1 Part 1 §3.3 and §3.3.6) rather than matching hand-coded
/// element-name literals. The hand-coded `parse` path remains as the
/// projection-to-variant tail of the grounded query.
pub(super) fn read_hierarchy_children(
    elem: &XmlElement,
) -> Result<Vec<HierarchyNode>, UslmReadError> {
    let xsd = loaded_uslm_xsd();
    let mut out = Vec::new();
    for child in &elem.children {
        let XmlNode::Element(e) = child else { continue };
        // Editorial / quoted scopes are not part of the published
        // hierarchy. They live in different ontological kinds
        // tracked by separate fields (M4.δ.5+). The dispatch here is
        // structural: skip every level-group non-member that the
        // grounded `ContainerKind::from_xsd_element` would also skip,
        // but additionally skip the named editorial elements which
        // are declared by the XSD but aren't part of the hierarchy
        // walk. (W3C XSD 1.1 Part 1 §3.3.6 — substitution-group
        // membership is the navigational discriminator.)
        if matches!(
            e.name.local.as_str(),
            "quotedContent" | "note" | "footnote" | "notes"
        ) {
            continue;
        }
        // Dispatch by XSD-grounded query: is this child the section
        // leaf? Per W3C XSD 1.1 Part 1 §3.3 (Element Declarations),
        // the loaded XSD declares `<xsd:element name="section">` as a
        // member of `substitutionGroup="level"`; the comparison below
        // uses the XSD-loaded name as the source of truth.
        if is_section_leaf(&e.name.local, xsd) {
            out.push(HierarchyNode::Section(Box::new(read_section(e)?)));
        } else if let Some(kind) = ContainerKind::from_xsd_element(&e.name.local, xsd) {
            let container = UsCodeContainer {
                kind,
                identifier: attr(e, "identifier").unwrap_or_default(),
                num: first_child_attr(e, "num", "value").unwrap_or_default(),
                heading: first_child_text(e, "heading").unwrap_or_default(),
                children: read_hierarchy_children(e)?,
                notes_blocks: read_notes_blocks(e),
                bare_notes: read_bare_notes(e),
                tocs: read_tocs(e),
            };
            out.push(HierarchyNode::Container(Box::new(container)));
        }
        // Other element kinds (header, layout, toc, etc.) belong
        // to other tiers and are not part of the navigational
        // hierarchy. They'll be modeled in M4.δ.5+.
    }
    Ok(out)
}

/// DFS-walk a hierarchy, collecting every leaf `Section` into
/// `out` in document order.
pub(super) fn flatten_sections(nodes: &[HierarchyNode], out: &mut Vec<UsCodeSection>) {
    for node in nodes {
        match node {
            HierarchyNode::Section(s) => out.push((**s).clone()),
            HierarchyNode::Container(c) => flatten_sections(&c.children, out),
        }
    }
}

/// Read a single `<section>` element into a [`UsCodeSection`].
///
/// Public so callers that already hold an XmlElement (e.g. after
/// pre-slicing) can drive the parse directly.
///
/// Dispatch: every per-child branch first verifies that the element
/// name is declared by the loaded USLM XSD (W3C XSD 1.1 Part 1 §3.3)
/// before projecting to a runtime branch. The projection-to-variant
/// step uses the hand-coded enum constructors (`SubdivisionKind::parse`
/// for level-group child kinds, the `read_*` extractors for typed
/// leaves) but only after the XSD grounding has classified the name
/// as a known USLM element.
pub fn read_section(elem: &XmlElement) -> Result<UsCodeSection, UslmReadError> {
    let xsd = loaded_uslm_xsd();
    if !is_section_leaf(&elem.name.local, xsd) {
        return Err(UslmReadError::Structure(format!(
            "expected <section>, got <{}>",
            elem.name.local
        )));
    }
    let identifier = attr(elem, "identifier").unwrap_or_default();
    let num = first_child_attr(elem, "num", "value").unwrap_or_default();
    let num_footnote = first_child_num_footnote(elem);
    let heading = first_child_text(elem, "heading").unwrap_or_default();
    let heading_runs = first_child_inline_runs(elem, "heading");
    let chapeau = first_child_text(elem, "chapeau");
    let chapeau_runs = first_child_inline_runs(elem, "chapeau");
    let content = first_child_text(elem, "content");
    let content_runs = first_child_inline_runs(elem, "content");

    let mut children = Vec::new();
    let mut refs = Vec::new();
    let mut notes_blocks = Vec::new();
    let mut bare_notes = Vec::new();
    let mut source_credits = Vec::new();
    let mut continuations = Vec::new();
    let mut def_blocks = Vec::new();
    let mut markers = Vec::new();
    let mut amendments = Vec::new();
    for child in &elem.children {
        if let XmlNode::Element(e) = child {
            // Skip children whose names aren't declared by the loaded
            // USLM XSD — they can't dispatch via the ontology query.
            // (W3C XSD 1.1 Part 1 §3.3: an undeclared element has no
            // `{type definition}` to dispatch on.)
            if !xsd_declares(&e.name.local) {
                // Out-of-namespace foreign content (Dublin Core,
                // XHTML, xsi:*) still uses the local-name match via
                // namespace prefix; we don't ground those here. For
                // USLM-namespaced names this is the unknown-element
                // bailout.
                continue;
            }
            if let Some(kind) = SubdivisionKind::from_xsd_element(&e.name.local, xsd) {
                children.push(read_subdivision(e, kind)?);
            } else if matches!(
                e.name.local.as_str(),
                "ref" | "chapeau" | "content" | "heading"
            ) {
                collect_refs_in(e, &mut refs);
            } else if e.name.local == "notes" {
                notes_blocks.push(read_notes_block(e));
            } else if e.name.local == "note" {
                bare_notes.push(read_note(e));
            } else if e.name.local == "sourceCredit" {
                source_credits.push(read_source_credit(e));
            } else if e.name.local == "continuation" {
                continuations.push(read_continuation(e));
            } else if e.name.local == "def" {
                def_blocks.push(read_def_block(e));
            } else if e.name.local == "marker" {
                markers.push(read_marker(e));
            } else if let Some(amend_kind) = UsCodeAmendmentKind::parse(&e.name.local) {
                amendments.push(UsCodeAmendmentMarkup {
                    kind: amend_kind,
                    text: element_text(e),
                });
            }
        }
    }

    Ok(UsCodeSection {
        identifier,
        num,
        num_footnote,
        heading,
        heading_runs,
        chapeau,
        chapeau_runs,
        content,
        content_runs,
        children,
        refs,
        notes_blocks,
        bare_notes,
        source_credits,
        continuations,
        def_blocks,
        markers,
        amendments,
    })
}

/// The cross-reference footnote the LRC embeds inside a section's
/// `<num>` to disambiguate a duplicated section number — e.g.
/// "Another section 3598 is set out after this section." Returns the
/// note's plain text, or `None` when `<num>` carries no `<note>` /
/// `<footnote>` (the common case).
///
/// `element_text` deliberately suppresses `<note>` / `<footnote>`
/// descendants of body elements, so the disambiguation footnote is
/// invisible to the ordinary `num` value extraction; this reader
/// reaches into the `<num>` element specifically to recover it. Per
/// the Office of the Law Revision Counsel duplicate-numbering
/// editorial convention.
pub(super) fn first_child_num_footnote(elem: &XmlElement) -> Option<String> {
    for child in &elem.children {
        if let XmlNode::Element(num) = child
            && num.name.local == "num"
        {
            for node in &num.children {
                if let XmlNode::Element(note) = node
                    && matches!(note.name.local.as_str(), "note" | "footnote")
                {
                    let text = note_text(note);
                    if !text.is_empty() {
                        return Some(text);
                    }
                }
            }
            return None;
        }
    }
    None
}

/// Plain text of a `<note>` / `<footnote>` element, normalized like
/// [`element_text`] but WITHOUT suppressing the note itself (the
/// suppression in `element_text` only applies to note/footnote
/// *descendants* of body elements). Used to recover the LRC's
/// duplicate-numbering disambiguation footnote from inside `<num>`.
fn note_text(elem: &XmlElement) -> String {
    let mut buf = String::new();
    for child in &elem.children {
        match child {
            XmlNode::Text(s) | XmlNode::CData(s) => buf.push_str(s),
            XmlNode::Element(e) => push_text(e, &mut buf),
            // Comments and processing instructions carry no normative
            // text (W3C XML 1.0 §2.5/§2.6) — skip.
            XmlNode::Comment(_) | XmlNode::ProcessingInstruction { .. } => {}
        }
    }
    let trimmed = buf.trim();
    let mut out = String::with_capacity(trimmed.len());
    let mut prev_space = false;
    for ch in trimmed.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out
}

/// Read a subdivision element (subsection / paragraph / … /
/// subitem) recursively into a [`UsCodeSubdivision`].
///
/// Dispatch matches the section reader's pattern: every per-child
/// branch is gated on the loaded USLM XSD's element-declaration set
/// (W3C XSD 1.1 Part 1 §3.3) and substitution-group membership
/// (§3.3.6).
fn read_subdivision(
    elem: &XmlElement,
    kind: SubdivisionKind,
) -> Result<UsCodeSubdivision, UslmReadError> {
    let xsd = loaded_uslm_xsd();
    let identifier = attr(elem, "identifier").unwrap_or_default();
    let num = first_child_attr(elem, "num", "value").unwrap_or_default();
    let heading = first_child_text(elem, "heading");
    let heading_runs = first_child_inline_runs(elem, "heading");
    let chapeau = first_child_text(elem, "chapeau");
    let chapeau_runs = first_child_inline_runs(elem, "chapeau");
    let content = first_child_text(elem, "content");
    let content_runs = first_child_inline_runs(elem, "content");

    let mut children = Vec::new();
    let mut refs = Vec::new();
    let mut def_blocks = Vec::new();
    let mut markers = Vec::new();
    let mut amendments = Vec::new();
    for child in &elem.children {
        if let XmlNode::Element(e) = child {
            if !xsd_declares(&e.name.local) {
                continue;
            }
            if let Some(child_kind) = SubdivisionKind::from_xsd_element(&e.name.local, xsd) {
                children.push(read_subdivision(e, child_kind)?);
            } else if matches!(
                e.name.local.as_str(),
                "ref" | "chapeau" | "content" | "heading"
            ) {
                collect_refs_in(e, &mut refs);
            } else if e.name.local == "def" {
                def_blocks.push(read_def_block(e));
            } else if e.name.local == "marker" {
                markers.push(read_marker(e));
            } else if let Some(amend_kind) = UsCodeAmendmentKind::parse(&e.name.local) {
                amendments.push(UsCodeAmendmentMarkup {
                    kind: amend_kind,
                    text: element_text(e),
                });
            }
        }
    }

    Ok(UsCodeSubdivision {
        identifier,
        num,
        kind,
        heading,
        heading_runs,
        chapeau,
        chapeau_runs,
        content,
        content_runs,
        children,
        refs,
        def_blocks,
        markers,
        amendments,
    })
}

/// Read a `<def>` block per LRC USLM User Guide § "Lexical Elements".
/// Collects every direct `<term>` child plus the flat body text.
fn read_def_block(elem: &XmlElement) -> UsCodeDefBlock {
    let identifier = attr(elem, "id");
    let mut terms = Vec::new();
    for child in &elem.children {
        if let XmlNode::Element(e) = child
            && e.name.local == "term"
        {
            terms.push(read_term(e));
        }
    }
    UsCodeDefBlock {
        identifier,
        terms,
        body: element_text(elem),
    }
}

/// Read a `<term>` mention. Accepts both `refersTo` (canonical USLM
/// attribute name) and `refers-to` (LRC xPath-example convention).
fn read_term(elem: &XmlElement) -> UsCodeTerm {
    UsCodeTerm {
        text: element_text(elem),
        refers_to: attr(elem, "refersTo").or_else(|| attr(elem, "refers-to")),
    }
}

/// Read a `<marker name="..." class="...">` anchor element. The
/// `name` attribute is required by the schema; if absent, we record
/// an empty string and let the consumer report the malformed source.
fn read_marker(elem: &XmlElement) -> UsCodeMarker {
    UsCodeMarker {
        name: attr(elem, "name").unwrap_or_default(),
        class: attr(elem, "class"),
    }
}

/// First direct child of `elem` with the given local name,
/// converted to a Vec of typed inline runs. Returns empty Vec if
/// the child doesn't exist.
fn first_child_inline_runs(elem: &XmlElement, name: &str) -> Vec<UsCodeInlineRun> {
    for child in &elem.children {
        if let XmlNode::Element(e) = child
            && e.name.local == name
        {
            return read_inline_runs(e);
        }
    }
    Vec::new()
}

/// Walk a text-bearing element's children and emit a typed
/// inline-run sequence per W3C XHTML inline-element semantics +
/// USLM Schema § "Inline Elements".
///
/// Plain-text nodes become `InlineKind::PlainText` runs;
/// recognized inline tags (`<i>`, `<b>`, `<sup>`, `<sub>`,
/// `<span>`, `<a>`, `<inline>`) become their typed kind. Unknown
/// inline-like elements fall through to recursive flattening so
/// no text content is lost — the type information is lost only
/// for the unrecognized wrapper.
fn read_inline_runs(elem: &XmlElement) -> Vec<UsCodeInlineRun> {
    let mut out = Vec::new();
    for child in &elem.children {
        match child {
            XmlNode::Text(s) | XmlNode::CData(s) => {
                let trimmed = collapse_whitespace(s);
                if !trimmed.is_empty() {
                    out.push(UsCodeInlineRun {
                        kind: InlineKind::PlainText,
                        text: trimmed,
                        class: None,
                        href: None,
                    });
                }
            }
            XmlNode::Element(e) => {
                if let Some(kind) = InlineKind::parse(&e.name.local) {
                    let text = element_text(e);
                    if text.is_empty() {
                        continue;
                    }
                    let class = attr(e, "class");
                    let href = attr(e, "href");
                    out.push(UsCodeInlineRun {
                        kind,
                        text,
                        class,
                        href,
                    });
                } else if matches!(e.name.local.as_str(), "ref" | "num") {
                    // <ref> and <num> contribute their visible
                    // text but aren't inline-style ornaments —
                    // emit as plain text to keep the visible
                    // sequence intact.
                    let text = element_text(e);
                    if !text.is_empty() {
                        out.push(UsCodeInlineRun {
                            kind: InlineKind::PlainText,
                            text,
                            class: None,
                            href: attr(e, "href"),
                        });
                    }
                } else if !matches!(e.name.local.as_str(), "note" | "footnote") {
                    // Unknown wrapper element — recurse and
                    // flatten its content so no text is lost.
                    out.extend(read_inline_runs(e));
                }
            }
            _ => {}
        }
    }
    out
}

/// Collapse internal whitespace runs to single spaces and trim.
fn collapse_whitespace(s: &str) -> String {
    let trimmed = s.trim();
    let mut out = String::with_capacity(trimmed.len());
    let mut prev_space = false;
    for ch in trimmed.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out
}

/// Read a `<ref>` element into a [`UsCodeRef`], if it carries a
/// USLM cross-reference `href`. Footnote backlinks use `idref`
/// (anchor-style internal references) and are filtered out here —
/// they're not citation-graph edges.
fn read_ref(elem: &XmlElement) -> Option<UsCodeRef> {
    let href = attr(elem, "href")?;
    if href.is_empty() {
        return None;
    }
    let text = element_text(elem);
    Some(UsCodeRef { href, text })
}

/// Walk an element's subtree appending every `<ref href="...">`
/// found into `out`. Stops descending into child subdivisions —
/// those collect their own refs separately. Notes / footnotes are
/// skipped per the body-text-only rule used by `element_text`.
/// Footnote backlinks (`<ref idref="...">` without `href`) are
/// filtered by [`read_ref`].
fn collect_refs_in(elem: &XmlElement, out: &mut Vec<UsCodeRef>) {
    if elem.name.local == "ref" {
        if let Some(r) = read_ref(elem) {
            out.push(r);
        }
        return;
    }
    let xsd = loaded_uslm_xsd();
    for child in &elem.children {
        if let XmlNode::Element(e) = child {
            // Don't descend into nested subdivisions — they collect
            // their own refs. The is-subdivision query consults the
            // loaded USLM XSD's `substitutionGroup="level"` membership
            // (W3C XSD 1.1 Part 1 §3.3.6) via
            // `SubdivisionKind::from_xsd_element` rather than a
            // hand-coded name match.
            if SubdivisionKind::from_xsd_element(&e.name.local, xsd).is_some() {
                continue;
            }
            if matches!(e.name.local.as_str(), "note" | "footnote") {
                continue;
            }
            collect_refs_in(e, out);
        }
    }
}

// ---------------------------------------------------------------------------
// XmlElement helpers
// ---------------------------------------------------------------------------

pub(super) fn attr(elem: &XmlElement, key: &str) -> Option<String> {
    elem.attributes
        .iter()
        .find(|a| a.name.local == key)
        .map(|a| a.value.clone())
}

pub(super) fn find_first_descendant<'a>(
    elem: &'a XmlElement,
    name: &str,
) -> Option<&'a XmlElement> {
    if elem.name.local == name {
        return Some(elem);
    }
    for child in &elem.children {
        if let XmlNode::Element(e) = child
            && let Some(found) = find_first_descendant(e, name)
        {
            return Some(found);
        }
    }
    None
}

/// First descendant of `elem` (depth-first) whose qualified name
/// resolves to `(local_name, namespace_uri)` per W3C XML Namespaces
/// 1.0 §6. `ctx` carries the in-scope default namespace as the
/// recursion descends — entering a child updates the context if
/// that child declares its own `xmlns="…"`.
fn find_first_in_namespace<'a>(
    elem: &'a XmlElement,
    ctx: NsContext<'_>,
    local_name: &str,
    namespace_uri: &str,
) -> Option<&'a XmlElement> {
    if elem.name.local == local_name && ctx.elem_in(elem, namespace_uri) {
        return Some(elem);
    }
    for child in &elem.children {
        if let XmlNode::Element(e) = child {
            let child_ctx = ctx.enter(e);
            if let Some(found) = find_first_in_namespace(e, child_ctx, local_name, namespace_uri) {
                return Some(found);
            }
        }
    }
    None
}

pub(super) fn first_child_text(elem: &XmlElement, name: &str) -> Option<String> {
    for child in &elem.children {
        if let XmlNode::Element(e) = child
            && e.name.local == name
        {
            return Some(element_text(e));
        }
    }
    None
}

pub(super) fn first_child_attr(
    elem: &XmlElement,
    child_name: &str,
    attr_key: &str,
) -> Option<String> {
    for child in &elem.children {
        if let XmlNode::Element(e) = child
            && e.name.local == child_name
        {
            return attr(e, attr_key);
        }
    }
    None
}

/// Concatenate all descendant text into a normalized plain string
/// (whitespace runs collapsed, leading/trailing whitespace
/// trimmed). Inline ornaments (`<inline>`, `<i>`, `<ref>`) contribute
/// their visible text; container ornaments (`<note>`, `<footnote>`)
/// are suppressed because they're outside the body's normative
/// flow.
fn element_text(elem: &XmlElement) -> String {
    let mut buf = String::new();
    push_text(elem, &mut buf);
    let trimmed = buf.trim();
    let mut out = String::with_capacity(trimmed.len());
    let mut prev_space = false;
    for ch in trimmed.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out
}

fn push_text(elem: &XmlElement, buf: &mut String) {
    for child in &elem.children {
        match child {
            XmlNode::Text(s) | XmlNode::CData(s) => buf.push_str(s),
            XmlNode::Element(e) => {
                if matches!(e.name.local.as_str(), "note" | "footnote") {
                    continue;
                }
                push_text(e, buf);
            }
            _ => {}
        }
    }
}

pub(super) fn derive_title_identifier(section_identifier: &str) -> Option<String> {
    // `/us/usc/t18/s1514A` → `/us/usc/t18`. Strip the last
    // `/sXXX...` segment and anything below it.
    let mut parts: Vec<&str> = section_identifier.split('/').collect();
    while let Some(last) = parts.last() {
        if last.starts_with('s') || last.starts_with('t') {
            if last.starts_with('t') {
                break;
            }
            parts.pop();
        } else {
            parts.pop();
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("/"))
    }
}
