//! USC corpus codegen — walks every registered USC title XML and
//! emits a single `CodegenData<UsCode>` static populated with one
//! entity per `<section>`, plus a parallel `USC_SECTION_AUX` table
//! carrying the subdivision tree + Composes-relation graph per
//! section.
//!
//! Parallel to [`super::wordnet`] for English. Where the WordNet
//! codegen produces an `OntologyBuilder` with one entity per synset,
//! this codegen produces an `OntologyBuilder` with one entity per USC
//! section. The flat section data flows through
//! [`super::generate::generate_rust`] as the standard `CodegenData<P>`
//! transport; the subdivision depth is emitted by a tailored emitter
//! below as `pub static USC_SECTION_AUX: &[UscSectionAux]`.
//!
//! ## Mapping
//!
//! For every USC `<section>` encountered across the registered title
//! XMLs:
//!
//! - `EntityDef.id` — the section's USLM URN string verbatim, e.g.
//!   `/us/usc/t18/s1514A`. Grammar-validated against the LRC USLM
//!   identifier conventions (§V) by virtue of being read straight
//!   out of the LRC-published XML; the runtime functor
//!   [`UsCode::from_codegen`] const-constructs an `Identifier` from
//!   it without re-validation.
//! - `EntityDef.label` — section heading text (`<heading>`),
//!   whitespace-collapsed.
//! - `EntityDef.definition` — section body text, formed by joining
//!   the chapeau / content text of every container nested inside the
//!   section. Empty if the section has no chapeau/content (rare —
//!   placeholder reservations like `[Reserved]`).
//! - `pos` — the string `"section"` (USLM element name); the
//!   `entity_kind` slice of `CodegenData` carries this through.
//!
//! Each section also produces one [`SectionAux`] record holding the
//! full subdivision tree (subsection / paragraph / subparagraph /
//! clause / subclause / item / subitem — every member of the loaded
//! USLM XSD's `substitutionGroup="level"` family per W3C XSD 1.1
//! Part 1 §3.3.6, intersected with the level-below-section subset
//! per [`SubdivisionKind`]) and the Composes-edge list joining
//! parent to child.
//!
//! ## Literature
//!
//! - U.S. House Office of the Law Revision Counsel, *USLM XML User
//!   Guide (USLM-1.0.18.xsd)*. <https://uscode.house.gov/uslm/>.
//! - 1 U.S.C. § 204 — *Codes and Supplements; positive law titles*.
//! - W3C, *XML Schema 1.1 Part 1: Structures* §3.3.6 (Substitution
//!   Groups). <https://www.w3.org/TR/xmlschema11-1/#cElement_Declarations>.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use super::builder::{EntityDef, GenerateConfig, OntologyBuilder};
use super::generate::generate_rust;
use super::uslm::ParseError;

/// Project a USLM subdivision element name onto the runtime
/// [`SubdivisionKind`] enum variant. Returns the variant identifier
/// (PascalCase, used in emitted code). `None` for non-subdivision
/// tags. Mirrors `SubdivisionKind::parse` but emits the variant
/// name as a string for the codegen output.
///
/// Codegen emission is a build-time step and doesn't have a loaded
/// `XsdOntologyInstance` available; the runtime functor
/// (`SubdivisionKind::from_xsd_element`) re-validates each name
/// against the loaded USLM XSD ontology (W3C XSD 1.1 Part 1 §3.3.6
/// substitution-group membership) when the data is materialised.
fn subdivision_kind_variant(tag: &[u8]) -> Option<&'static str> {
    Some(match tag {
        b"subsection" => "Subsection",
        b"paragraph" => "Paragraph",
        b"subparagraph" => "Subparagraph",
        b"clause" => "Clause",
        b"subclause" => "Subclause",
        b"item" => "Item",
        b"subitem" => "Subitem",
        _ => return None,
    })
}

/// In-memory subdivision node — the build-time mirror of the runtime
/// `UscSubdivision`. Owned strings here (codegen output is a single
/// string emission so the data lives only for the duration of the
/// build).
#[derive(Debug, Clone)]
struct SubdivisionDoc {
    urn: String,
    kind_variant: &'static str,
    num: String,
    heading: Option<String>,
    chapeau: Option<String>,
    content: Option<String>,
    children: Vec<SubdivisionDoc>,
}

/// In-memory section-aux record — the build-time mirror of the
/// runtime `UscSectionAux`. The `relations` field holds
/// `(child_urn, parent_urn)` pairs across the whole tree.
#[derive(Debug, Clone)]
struct SectionAux {
    urn: String,
    subdivisions: Vec<SubdivisionDoc>,
    relations: Vec<(String, String)>,
}

/// Walk every registered USC title XML in `title_xml_paths`, extract
/// one `<section>` per entity, and emit the generated Rust source
/// string for `CodegenData<UsCode>` plus the matching
/// `USC_SECTION_AUX` table carrying the subdivision tree + Composes
/// graph per section.
///
/// Skips files that don't exist (callers should filter the input
/// slice ahead of time if they want a hard error). Section ordering
/// in the output matches input-file order, then USLM document order
/// within each file.
pub fn generate_usc_corpus_source(
    title_xml_paths: &[&Path],
    config: &GenerateConfig,
) -> Result<String, ParseError> {
    let mut builder = OntologyBuilder::new();
    let mut all_aux: Vec<SectionAux> = Vec::new();
    for path in title_xml_paths {
        let xml = std::fs::read_to_string(path)
            .map_err(|e| ParseError::Read(path.display().to_string(), e))?;
        extract_sections(&xml, &mut builder, &mut all_aux)?;
    }
    let mut out = generate_rust(&builder, config);
    out.push('\n');
    let path_prefix = corpus_path_prefix(config);
    out.push_str(&emit_section_aux_table(&all_aux, &path_prefix));
    Ok(out)
}

/// In-memory variant of [`generate_usc_corpus_source`] for tests.
pub fn generate_usc_corpus_source_from_strs(
    xmls: &[&str],
    config: &GenerateConfig,
) -> Result<String, ParseError> {
    let mut builder = OntologyBuilder::new();
    let mut all_aux: Vec<SectionAux> = Vec::new();
    for xml in xmls {
        extract_sections(xml, &mut builder, &mut all_aux)?;
    }
    let mut out = generate_rust(&builder, config);
    out.push('\n');
    let path_prefix = corpus_path_prefix(config);
    out.push_str(&emit_section_aux_table(&all_aux, &path_prefix));
    Ok(out)
}

/// Derive the Rust path prefix used by emitted struct literals from
/// the config's marker path. The marker is the fully-qualified path
/// to `UsCode` in the consumer crate (e.g.
/// `"crate::social::software::markup::xml::uslm::corpus::UsCode"` for
/// pr4xis-domains internal, or
/// `"pr4xis_domains::social::software::markup::xml::uslm::corpus::UsCode"`
/// for external cli/wasm consumers); the prefix is everything up to
/// the final `::UsCode` segment so the aux types resolve from the
/// same module.
fn corpus_path_prefix(config: &GenerateConfig) -> String {
    let marker = config.entity_marker_path.as_str();
    marker
        .rsplit_once("::")
        .map(|(prefix, _)| prefix.to_string())
        .unwrap_or_else(|| marker.to_string())
}

/// Single-pass walker — for each `<section>` element in the XML,
/// append one `EntityDef` to `builder` and one [`SectionAux`] to
/// `aux_out`. Heading and body text are accumulated from
/// `<heading>`, `<chapeau>`, and `<content>` children at any nesting
/// depth within the section. The subdivision tree is built from
/// every level-group element below the section per
/// `SUBDIVISION_TAGS`.
///
/// `<note>` / `<footnote>` content is suppressed from the body so
/// editorial annotations don't pollute the section text.
fn extract_sections(
    xml: &str,
    builder: &mut OntologyBuilder,
    aux_out: &mut Vec<SectionAux>,
) -> Result<(), ParseError> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    // Per-section capture state. `None` between sections — element
    // events are still walked but text accumulators ignore them.
    let mut state: Option<SectionCapture> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name_bytes = e.name().as_ref().to_vec();

                // Open a new section capture.
                if name_bytes == b"section" && state.is_none() {
                    let Some(identifier) = attr(e, b"identifier") else {
                        buf.clear();
                        continue;
                    };
                    state = Some(SectionCapture::new(identifier));
                    continue;
                }

                let Some(sc) = state.as_mut() else {
                    buf.clear();
                    continue;
                };

                // Open a subdivision frame for any level-below-section
                // element. The subdivision's `num` is carried by a
                // nested `<num value="...">` element (the marker
                // text), not by an attribute on the subdivision
                // element itself — see USLM XSD `num` element vs
                // `value` attribute distinction.
                if let Some(kind_variant) = subdivision_kind_variant(&name_bytes) {
                    let urn = attr(e, b"identifier").unwrap_or_default();
                    sc.sub_stack.push(SubdivisionFrame {
                        urn,
                        kind_variant,
                        num: String::new(),
                        heading: String::new(),
                        chapeau: String::new(),
                        content: String::new(),
                        children: Vec::new(),
                    });
                    continue;
                }

                // `<num value="a">(a)</num>` — capture the marker
                // value into whichever subdivision frame is currently
                // open. Only honour the FIRST `<num>` seen inside a
                // subdivision (USLM nests `<num>` only once at the
                // top of each level container; deeper `<num>` belongs
                // to a child subdivision opened separately).
                if name_bytes == b"num"
                    && let Some(frame) = sc.sub_stack.last_mut()
                    && frame.num.is_empty()
                    && let Some(v) = attr(e, b"value")
                {
                    frame.num = v;
                }

                match name_bytes.as_slice() {
                    b"note" | b"footnote" => {
                        // Track nesting so a `<note>` inside a `<note>`
                        // still keeps suppression on.
                        sc.note_depth += 1;
                        sc.text_target = Some(TextTarget::Suppressed);
                        sc.text_buf.clear();
                    }
                    b"heading" if sc.note_depth == 0 => {
                        sc.text_target = Some(TextTarget::Heading);
                        sc.text_buf.clear();
                    }
                    b"chapeau" if sc.note_depth == 0 => {
                        sc.text_target = Some(TextTarget::Chapeau);
                        sc.text_buf.clear();
                    }
                    b"content" if sc.note_depth == 0 => {
                        sc.text_target = Some(TextTarget::Content);
                        sc.text_buf.clear();
                    }
                    // Inline ornaments and `<num>` — text flows into
                    // whichever accumulator is open.
                    b"num" | b"ref" | b"inline" | b"i" | b"b" | b"sup" | b"sub" | b"span"
                    | b"a" => {}
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                if let Some(sc) = state.as_mut()
                    && sc.text_target.is_some()
                {
                    let decoded = e.decode().unwrap_or_default();
                    let unescaped = match quick_xml::escape::unescape(&decoded) {
                        Ok(u) => u.into_owned(),
                        Err(_) => decoded.into_owned(),
                    };
                    sc.text_buf.push_str(&unescaped);
                }
            }
            Ok(Event::End(ref e)) => {
                let name_bytes = e.name().as_ref().to_vec();
                let Some(sc) = state.as_mut() else {
                    buf.clear();
                    continue;
                };

                // Closing the outermost <section>: finalise and emit.
                if name_bytes == b"section" {
                    let captured = state.take().unwrap();
                    let SectionCapture {
                        identifier,
                        heading,
                        body,
                        sub_stack: _, // empty by invariant
                        aux_subs,
                        aux_relations,
                        ..
                    } = captured;
                    let heading = clean_text(&heading);
                    let body = clean_text(&body);
                    let mut entity = EntityDef::new(&identifier, &heading).pos("section");
                    if !body.is_empty() {
                        entity = entity.definition(&body);
                    }
                    builder.add_entity(entity);
                    aux_out.push(SectionAux {
                        urn: identifier,
                        subdivisions: aux_subs,
                        relations: aux_relations,
                    });
                    continue;
                }

                // Closing a subdivision frame: pop, finalize, and
                // attach to its parent (or the section's top-level
                // subdivisions list).
                if subdivision_kind_variant(&name_bytes).is_some() {
                    if let Some(frame) = sc.sub_stack.pop() {
                        let SubdivisionFrame {
                            urn,
                            kind_variant,
                            num,
                            heading,
                            chapeau,
                            content,
                            children,
                        } = frame;
                        // Compute the parent URN for the
                        // Composes-edge: either the immediate parent
                        // subdivision (if the stack is non-empty after
                        // popping) or the section URN.
                        let parent_urn = sc
                            .sub_stack
                            .last()
                            .map(|p| p.urn.clone())
                            .unwrap_or_else(|| sc.identifier.clone());
                        if !urn.is_empty() && !parent_urn.is_empty() {
                            sc.aux_relations.push((urn.clone(), parent_urn));
                        }
                        let doc = SubdivisionDoc {
                            urn,
                            kind_variant,
                            num,
                            heading: nonempty(clean_text(&heading)),
                            chapeau: nonempty(clean_text(&chapeau)),
                            content: nonempty(clean_text(&content)),
                            children,
                        };
                        if let Some(parent_frame) = sc.sub_stack.last_mut() {
                            parent_frame.children.push(doc);
                        } else {
                            sc.aux_subs.push(doc);
                        }
                    }
                    continue;
                }

                match name_bytes.as_slice() {
                    b"note" | b"footnote" => {
                        if sc.note_depth > 0 {
                            sc.note_depth -= 1;
                        }
                        if sc.note_depth == 0 {
                            sc.text_target = None;
                        }
                        sc.text_buf.clear();
                    }
                    b"heading" if sc.note_depth == 0 => {
                        // Heading text goes to whatever frame is open.
                        // If a subdivision frame is open, the heading
                        // belongs to it; otherwise to the section
                        // heading accumulator.
                        if let Some(frame) = sc.sub_stack.last_mut() {
                            if !frame.heading.is_empty() {
                                frame.heading.push(' ');
                            }
                            frame.heading.push_str(&sc.text_buf);
                        } else {
                            if !sc.heading.is_empty() {
                                sc.heading.push(' ');
                            }
                            sc.heading.push_str(&sc.text_buf);
                        }
                        sc.text_target = None;
                        sc.text_buf.clear();
                    }
                    b"chapeau" if sc.note_depth == 0 => {
                        // Chapeau text goes to the open subdivision
                        // frame (if any); also accumulate into the
                        // section body for the flat section text.
                        if let Some(frame) = sc.sub_stack.last_mut() {
                            if !frame.chapeau.is_empty() {
                                frame.chapeau.push(' ');
                            }
                            frame.chapeau.push_str(&sc.text_buf);
                        }
                        if !sc.body.is_empty() {
                            sc.body.push(' ');
                        }
                        sc.body.push_str(&sc.text_buf);
                        sc.text_target = None;
                        sc.text_buf.clear();
                    }
                    b"content" if sc.note_depth == 0 => {
                        if let Some(frame) = sc.sub_stack.last_mut() {
                            if !frame.content.is_empty() {
                                frame.content.push(' ');
                            }
                            frame.content.push_str(&sc.text_buf);
                        }
                        if !sc.body.is_empty() {
                            sc.body.push(' ');
                        }
                        sc.body.push_str(&sc.text_buf);
                        sc.text_target = None;
                        sc.text_buf.clear();
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(ParseError::Xml(format!("{e}"))),
            _ => {}
        }
        buf.clear();
    }

    Ok(())
}

#[derive(Debug)]
struct SectionCapture {
    identifier: String,
    heading: String,
    body: String,
    text_target: Option<TextTarget>,
    text_buf: String,
    /// Depth of nested `<note>` / `<footnote>` elements — anything
    /// inside is suppressed from heading/body.
    note_depth: u32,
    /// Stack of open subdivision frames. Each new
    /// `<subsection>`/`<paragraph>`/etc. pushes; closing pops and
    /// attaches to the parent frame (or the section top-level
    /// `aux_subs` if the stack is empty).
    sub_stack: Vec<SubdivisionFrame>,
    /// Completed top-level subdivisions for the section.
    aux_subs: Vec<SubdivisionDoc>,
    /// Composes-edge `(child_urn, parent_urn)` pairs accumulated
    /// across the whole tree.
    aux_relations: Vec<(String, String)>,
}

impl SectionCapture {
    fn new(identifier: String) -> Self {
        Self {
            identifier,
            heading: String::new(),
            body: String::new(),
            text_target: None,
            text_buf: String::new(),
            note_depth: 0,
            sub_stack: Vec::new(),
            aux_subs: Vec::new(),
            aux_relations: Vec::new(),
        }
    }
}

#[derive(Debug)]
struct SubdivisionFrame {
    urn: String,
    kind_variant: &'static str,
    num: String,
    heading: String,
    chapeau: String,
    content: String,
    children: Vec<SubdivisionDoc>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TextTarget {
    Heading,
    Chapeau,
    Content,
    Suppressed,
}

fn attr(e: &quick_xml::events::BytesStart, key: &[u8]) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == key {
            return Some(String::from_utf8_lossy(&attr.value).into_owned());
        }
    }
    None
}

/// Trim and collapse internal whitespace runs.
fn clean_text(s: &str) -> String {
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

fn nonempty(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

// ---------------------------------------------------------------------
// Emission — `pub static USC_SECTION_AUX: &[UscSectionAux]`
// ---------------------------------------------------------------------

fn emit_section_aux_table(aux: &[SectionAux], prefix: &str) -> String {
    let mut out = String::new();
    out.push_str("// USC subdivision tree + Composes-relation graph per section.\n");
    out.push_str("// Generated alongside CODEGEN_DATA; consumed by\n");
    out.push_str("// `UsCode::from_codegen_with_aux` to attach subdivision data to\n");
    out.push_str("// each section after the flat section list is materialised.\n");
    out.push_str(&format!(
        "pub static USC_SECTION_AUX: &[{prefix}::UscSectionAux] = &[\n"
    ));
    for entry in aux {
        out.push_str(&emit_section_aux(entry, prefix));
    }
    out.push_str("];\n");
    out
}

fn emit_section_aux(entry: &SectionAux, prefix: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!("    {prefix}::UscSectionAux {{\n"));
    out.push_str(&format!("        urn: {},\n", raw_str(&entry.urn)));
    out.push_str("        subdivisions: &[\n");
    for sub in &entry.subdivisions {
        out.push_str(&emit_subdivision(sub, 12, prefix));
    }
    out.push_str("        ],\n");
    out.push_str("        relations: &[\n");
    for (from, to) in &entry.relations {
        out.push_str(&format!(
            "            {prefix}::UscComposesEdge {{ from_urn: {}, to_urn: {} }},\n",
            raw_str(from),
            raw_str(to),
        ));
    }
    out.push_str("        ],\n");
    out.push_str("    },\n");
    out
}

fn emit_subdivision(sub: &SubdivisionDoc, indent: usize, prefix: &str) -> String {
    let pad = " ".repeat(indent);
    // The Identifier-format path always sits two levels above the
    // corpus module (i.e. swap `social::software::markup::xml::uslm::corpus`
    // for `formal::meta::identifier_format`), regardless of whether
    // the prefix is `crate::...` or `pr4xis_domains::...`. Derive
    // the prefix root by trimming the corpus tail.
    let identifier_format_path = identifier_format_path(prefix);
    let mut out = String::new();
    out.push_str(&pad);
    out.push_str(&format!("{prefix}::UscSubdivision {{\n"));
    out.push_str(&pad);
    out.push_str(&format!(
        "    urn: {identifier_format_path}::Identifier::from_codegen_static({identifier_format_path}::ontology::IdentifierFormatConcept::UslmUrn, {}),\n",
        raw_str(&sub.urn),
    ));
    out.push_str(&pad);
    out.push_str(&format!(
        "    kind: {prefix}::SubdivisionKind::{},\n",
        sub.kind_variant,
    ));
    out.push_str(&pad);
    out.push_str(&format!("    num: {},\n", raw_str(&sub.num)));
    out.push_str(&pad);
    out.push_str(&format!(
        "    heading: {},\n",
        emit_option_str(sub.heading.as_deref())
    ));
    out.push_str(&pad);
    out.push_str(&format!(
        "    chapeau: {},\n",
        emit_option_str(sub.chapeau.as_deref())
    ));
    out.push_str(&pad);
    out.push_str(&format!(
        "    content: {},\n",
        emit_option_str(sub.content.as_deref())
    ));
    out.push_str(&pad);
    out.push_str("    children: &[\n");
    for child in &sub.children {
        out.push_str(&emit_subdivision(child, indent + 8, prefix));
    }
    out.push_str(&pad);
    out.push_str("    ],\n");
    out.push_str(&pad);
    out.push_str("},\n");
    out
}

/// Given the corpus prefix `<root>::social::software::markup::xml::uslm::corpus`,
/// derive the sibling `<root>::formal::meta::identifier_format` path
/// used to construct typed URN identifiers in emitted subdivision
/// literals. The shared `<root>` is either `crate` (when the
/// generated file is included inside `pr4xis-domains`) or
/// `pr4xis_domains` (when included by an external consumer such as
/// the cli or wasm crate).
fn identifier_format_path(corpus_prefix: &str) -> String {
    const CORPUS_TAIL: &str = "::social::software::markup::xml::uslm::corpus";
    let root = corpus_prefix.strip_suffix(CORPUS_TAIL).unwrap_or("crate");
    format!("{root}::formal::meta::identifier_format")
}

fn emit_option_str(s: Option<&str>) -> String {
    match s {
        Some(v) => format!("Some({})", raw_str(v)),
        None => "None".to_string(),
    }
}

/// Emit a Rust raw-string literal with `#`-padding sized so the
/// content can't forge the boundary. Picks the smallest `n` such
/// that `"#"*n` doesn't appear inside `s`.
fn raw_str(s: &str) -> String {
    let mut n = 1;
    loop {
        let boundary = format!("\"{}", "#".repeat(n));
        if !s.contains(&boundary) {
            break;
        }
        n += 1;
    }
    let hashes = "#".repeat(n);
    format!("r{hashes}\"{s}\"{hashes}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TITLE_USLM: &str = r##"<title xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18"><num value="18">Title 18—</num><heading>CRIMES</heading><section identifier="/us/usc/t18/s1"><num value="1">§ 1.</num><heading>First</heading><content>one body</content></section><section identifier="/us/usc/t18/s2"><num value="2">§ 2.</num><heading>Second</heading><subsection identifier="/us/usc/t18/s2/a"><num value="a">(a)</num><content>two body</content></subsection></section></title>"##;

    fn cfg() -> GenerateConfig {
        GenerateConfig::with_marker(
            "usc_codegen",
            "UscEntityId",
            "pr4xis_domains::social::software::markup::xml::uslm::corpus::UsCode",
        )
    }

    fn extract_into(xml: &str, b: &mut OntologyBuilder, a: &mut Vec<SectionAux>) {
        extract_sections(xml, b, a).expect("parse");
    }

    #[test]
    fn emits_one_entity_per_section_across_multiple_titles() {
        let mut builder = OntologyBuilder::new();
        let mut aux = Vec::new();
        extract_into(SAMPLE_TITLE_USLM, &mut builder, &mut aux);
        assert_eq!(builder.entity_count(), 2);
        let ids: Vec<&str> = builder.entities.iter().map(|e| e.id.as_str()).collect();
        assert!(ids.contains(&"/us/usc/t18/s1"));
        assert!(ids.contains(&"/us/usc/t18/s2"));
    }

    #[test]
    fn section_headings_captured() {
        let mut builder = OntologyBuilder::new();
        let mut aux = Vec::new();
        extract_into(SAMPLE_TITLE_USLM, &mut builder, &mut aux);
        let labels: Vec<&str> = builder.entities.iter().map(|e| e.label.as_str()).collect();
        assert!(labels.contains(&"First"));
        assert!(labels.contains(&"Second"));
    }

    #[test]
    fn section_bodies_concatenate_chapeau_and_content_text() {
        let mut builder = OntologyBuilder::new();
        let mut aux = Vec::new();
        extract_into(SAMPLE_TITLE_USLM, &mut builder, &mut aux);
        let s2 = builder
            .entities
            .iter()
            .find(|e| e.id == "/us/usc/t18/s2")
            .expect("§2 present");
        assert!(
            s2.definitions
                .first()
                .map(|d| d.contains("two body"))
                .unwrap_or(false),
            "expected nested subsection text to appear in §2 body, got {:?}",
            s2.definitions
        );
    }

    #[test]
    fn entity_kind_is_section() {
        let mut builder = OntologyBuilder::new();
        let mut aux = Vec::new();
        extract_into(SAMPLE_TITLE_USLM, &mut builder, &mut aux);
        for e in &builder.entities {
            assert_eq!(e.pos.as_deref(), Some("section"));
        }
    }

    #[test]
    fn no_word_index_in_first_cut() {
        let mut builder = OntologyBuilder::new();
        let mut aux = Vec::new();
        extract_into(SAMPLE_TITLE_USLM, &mut builder, &mut aux);
        assert_eq!(builder.word_index.len(), 0);
    }

    #[test]
    fn aux_table_one_entry_per_section() {
        let mut builder = OntologyBuilder::new();
        let mut aux = Vec::new();
        extract_into(SAMPLE_TITLE_USLM, &mut builder, &mut aux);
        assert_eq!(aux.len(), 2);
        let urns: Vec<&str> = aux.iter().map(|a| a.urn.as_str()).collect();
        assert!(urns.contains(&"/us/usc/t18/s1"));
        assert!(urns.contains(&"/us/usc/t18/s2"));
    }

    #[test]
    fn aux_subdivisions_capture_one_node_for_simple_subsection() {
        let mut builder = OntologyBuilder::new();
        let mut aux = Vec::new();
        extract_into(SAMPLE_TITLE_USLM, &mut builder, &mut aux);
        let s2 = aux
            .iter()
            .find(|a| a.urn == "/us/usc/t18/s2")
            .expect("§2 aux");
        assert_eq!(s2.subdivisions.len(), 1);
        assert_eq!(s2.subdivisions[0].urn, "/us/usc/t18/s2/a");
        assert_eq!(s2.subdivisions[0].kind_variant, "Subsection");
        assert_eq!(s2.subdivisions[0].num, "a");
    }

    #[test]
    fn aux_relations_form_child_to_parent_composes_edges() {
        let mut builder = OntologyBuilder::new();
        let mut aux = Vec::new();
        extract_into(SAMPLE_TITLE_USLM, &mut builder, &mut aux);
        let s2 = aux
            .iter()
            .find(|a| a.urn == "/us/usc/t18/s2")
            .expect("§2 aux");
        // One subsection → one Composes edge (subsection → section).
        assert_eq!(s2.relations.len(), 1);
        let (from, to) = &s2.relations[0];
        assert_eq!(from, "/us/usc/t18/s2/a");
        assert_eq!(to, "/us/usc/t18/s2");
    }

    #[test]
    fn nested_subdivisions_form_a_tree() {
        // Synthetic § 1514A-like fragment: subsection (a) with two
        // paragraphs, first paragraph with two subparagraphs.
        let xml = r##"<title xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18"><section identifier="/us/usc/t18/s1514A"><num value="1514A">§ 1514A.</num><heading>SOX</heading><subsection identifier="/us/usc/t18/s1514A/a"><num value="a">(a)</num><heading>Whistleblower</heading><chapeau>No company may discriminate—</chapeau><paragraph identifier="/us/usc/t18/s1514A/a/1"><num value="1">(1)</num><chapeau>to provide information—</chapeau><subparagraph identifier="/us/usc/t18/s1514A/a/1/A"><num value="A">(A)</num><content>a Federal agency;</content></subparagraph><subparagraph identifier="/us/usc/t18/s1514A/a/1/B"><num value="B">(B)</num><content>any Member of Congress;</content></subparagraph></paragraph><paragraph identifier="/us/usc/t18/s1514A/a/2"><num value="2">(2)</num><content>to file a proceeding.</content></paragraph></subsection></section></title>"##;
        let mut builder = OntologyBuilder::new();
        let mut aux = Vec::new();
        extract_into(xml, &mut builder, &mut aux);
        let s = aux
            .iter()
            .find(|a| a.urn == "/us/usc/t18/s1514A")
            .expect("§ 1514A aux");
        // Top-level: one subsection (a).
        assert_eq!(s.subdivisions.len(), 1);
        let a = &s.subdivisions[0];
        assert_eq!(a.urn, "/us/usc/t18/s1514A/a");
        assert_eq!(a.kind_variant, "Subsection");
        assert_eq!(a.heading.as_deref(), Some("Whistleblower"));
        assert!(
            a.chapeau
                .as_deref()
                .map(|c| c.contains("No company"))
                .unwrap_or(false)
        );
        // Subsection (a) has two paragraphs.
        assert_eq!(a.children.len(), 2);
        // First paragraph has two subparagraphs.
        let p1 = &a.children[0];
        assert_eq!(p1.urn, "/us/usc/t18/s1514A/a/1");
        assert_eq!(p1.kind_variant, "Paragraph");
        assert_eq!(p1.children.len(), 2);
        let sp_a = &p1.children[0];
        assert_eq!(sp_a.urn, "/us/usc/t18/s1514A/a/1/A");
        assert_eq!(sp_a.kind_variant, "Subparagraph");
        // Relation count: every non-section node has one edge to its
        // parent. 5 subdivisions (a, 1, A, B, 2) → 5 edges.
        assert_eq!(s.relations.len(), 5);
        // Spot-check the deepest edge:
        assert!(
            s.relations
                .iter()
                .any(|(f, t)| f == "/us/usc/t18/s1514A/a/1/A" && t == "/us/usc/t18/s1514A/a/1")
        );
        // Top-level edge: subsection (a) → section.
        assert!(
            s.relations
                .iter()
                .any(|(f, t)| f == "/us/usc/t18/s1514A/a" && t == "/us/usc/t18/s1514A")
        );
    }

    #[test]
    fn generate_source_emits_codegen_data_marker_and_aux_table() {
        let src = generate_usc_corpus_source_from_strs(&[SAMPLE_TITLE_USLM], &cfg()).expect("emit");
        assert!(src.contains("pub static CODEGEN_DATA"));
        assert!(
            src.contains("pr4xis_domains::social::software::markup::xml::uslm::corpus::UsCode")
        );
        // Section URNs reach the emitted source.
        assert!(src.contains("/us/usc/t18/s1"));
        assert!(src.contains("/us/usc/t18/s2"));
        // Aux table emitted with subdivision data.
        assert!(src.contains("pub static USC_SECTION_AUX"));
        assert!(src.contains("UscSectionAux"));
        assert!(src.contains("UscSubdivision"));
        assert!(src.contains("UscComposesEdge"));
        assert!(src.contains("SubdivisionKind::Subsection"));
    }

    #[test]
    fn notes_are_suppressed_from_body() {
        let xml = r##"<title xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18"><section identifier="/us/usc/t18/s1"><heading>Hed</heading><content>keep this</content><note>drop this</note></section></title>"##;
        let mut builder = OntologyBuilder::new();
        let mut aux = Vec::new();
        extract_into(xml, &mut builder, &mut aux);
        let s = &builder.entities[0];
        let body = s.definitions.first().cloned().unwrap_or_default();
        assert!(body.contains("keep this"));
        assert!(!body.contains("drop this"), "got: {body:?}");
    }

    #[test]
    fn empty_input_yields_empty_corpus() {
        let mut builder = OntologyBuilder::new();
        let mut aux = Vec::new();
        extract_into(
            r##"<title xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18"><heading>X</heading></title>"##,
            &mut builder,
            &mut aux,
        );
        assert_eq!(builder.entity_count(), 0);
        assert_eq!(aux.len(), 0);
    }

    #[test]
    fn subdivision_kind_variant_dispatches_every_kind() {
        assert_eq!(subdivision_kind_variant(b"subsection"), Some("Subsection"));
        assert_eq!(subdivision_kind_variant(b"paragraph"), Some("Paragraph"));
        assert_eq!(
            subdivision_kind_variant(b"subparagraph"),
            Some("Subparagraph")
        );
        assert_eq!(subdivision_kind_variant(b"clause"), Some("Clause"));
        assert_eq!(subdivision_kind_variant(b"subclause"), Some("Subclause"));
        assert_eq!(subdivision_kind_variant(b"item"), Some("Item"));
        assert_eq!(subdivision_kind_variant(b"subitem"), Some("Subitem"));
        assert_eq!(subdivision_kind_variant(b"section"), None);
        assert_eq!(subdivision_kind_variant(b"chapter"), None);
    }
}
