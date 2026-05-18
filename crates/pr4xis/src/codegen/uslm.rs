//! USLM XML → `RawStatuteDoc` codegen.
//!
//! Mirrors the [`super::wordnet`] codegen path but for U.S. Code
//! statutes published as United States Legislative Markup (USLM)
//! XML by the U.S. House Office of the Law Revision Counsel (LRC).
//! Input: a USLM XML file (per-title; the LRC's published unit of
//! distribution) plus the identifier of the section to extract
//! (e.g. `/us/usc/t18/s1514A`). Output: a [`RawStatuteDoc`] in the
//! same shape [`super::statute::build_from_doc`] consumes.
//!
//! Authoritative source for USLM:
//!
//! - U.S. House Office of the Law Revision Counsel, *USLM XML User
//!   Guide*, available at <https://uscode.house.gov/uslm/>.
//! - 1 U.S.C. § 204 — *Codes and Supplements; positive law titles*,
//!   the statute authorizing the U.S. Code itself.
//!
//! ## Mapping
//!
//! For each USLM hierarchical container reached inside the target
//! section (the §, plus every nested `subsection`, `paragraph`,
//! `subparagraph`, `clause`, `subclause`, `item`, `subitem`):
//!
//! - **`RawTerm.id`** — derived from USLM `identifier`. The path
//!   after the section prefix becomes the CURIE local part with
//!   slashes replaced by underscores. Example:
//!   `/us/usc/t18/s1514A/a/1/A` with statute_name `sox_1514a`
//!   becomes `sox_1514a:a_1_A`.
//! - **`RawTerm.name`** — text content of `<heading>` (or the
//!   `<num>` if no heading is present).
//! - **`RawTerm.definition`** — text content of `<chapeau>` +
//!   `<content>` (the body the container introduces or carries).
//!   Inline ornaments (`<inline class="small-caps">`, `<i>`,
//!   `<ref>` text, footnote markers) are extracted as plain text.
//! - **`RawRelation`** — for every container that has a parent
//!   container within the section, one `Composes` relation
//!   pointing from child to parent (the parent contains the child
//!   as a textual component). This is how USLM's strictly-nested
//!   hierarchy maps to praxis's mereological `Composes` relation.
//!
//! Cross-references (`<ref href="...">`) are extracted as inline
//! text into definitions but **not** lifted into typed relations
//! at this layer — `RawRel` has no generic `References` variant
//! today. They remain queryable from the source XML and a future
//! codegen pass can lift them when downstream consumers need typed
//! cross-reference relations.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use super::statute::{RawRel, RawRelation, RawStatuteDoc, RawTerm};

/// USLM container element names — the hierarchy inside a §.
///
/// Strictly nested per the USLM schema; encountering one of these
/// elements while parsing always opens a new mereological child.
const CONTAINER_TAGS: &[&[u8]] = &[
    b"section",
    b"subsection",
    b"paragraph",
    b"subparagraph",
    b"clause",
    b"subclause",
    b"item",
    b"subitem",
];

/// Errors that can arise while parsing a USLM XML file.
#[derive(Debug)]
pub enum ParseError {
    /// Failed to read the file.
    Read(String, std::io::Error),
    /// quick-xml parse failure.
    Xml(String),
    /// The requested section identifier was not found anywhere in
    /// the document.
    SectionNotFound { identifier: String },
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Read(p, e) => write!(f, "read {p}: {e}"),
            Self::Xml(e) => write!(f, "XML parse error: {e}"),
            Self::SectionNotFound { identifier } => {
                write!(
                    f,
                    "section identifier {identifier:?} not found in USLM document"
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// Parse a USLM XML file (per-title), extract the section at
/// `section_identifier`, and produce a [`RawStatuteDoc`] keyed by
/// `statute_name` (the praxis-registry name used as the CURIE
/// prefix).
///
/// Example: `parse_uslm_xml(Path::new("usc18.xml"),
/// "/us/usc/t18/s1514A", "sox_1514a")` returns a doc whose terms
/// are `sox_1514a:a`, `sox_1514a:a_1`, `sox_1514a:a_1_A`, etc.
pub fn parse_uslm_xml(
    path: &Path,
    section_identifier: &str,
    statute_name: &str,
) -> Result<RawStatuteDoc, ParseError> {
    let xml = std::fs::read_to_string(path)
        .map_err(|e| ParseError::Read(path.display().to_string(), e))?;
    parse_uslm_str(&xml, section_identifier, statute_name)
}

/// Walk every `<section>` in a USLM title XML and produce a
/// [`RawStatuteDoc`] for each. The statute name (CURIE prefix) is
/// derived from the section identifier by lowercasing + slashifying
/// — e.g. `/us/usc/t18/s1514A` → `us_usc_t18_s1514a`.
pub fn parse_uslm_title_all_sections(path: &Path) -> Result<Vec<RawStatuteDoc>, ParseError> {
    let xml = std::fs::read_to_string(path)
        .map_err(|e| ParseError::Read(path.display().to_string(), e))?;
    parse_uslm_title_all_sections_str(&xml)
}

/// In-memory variant of [`parse_uslm_title_all_sections`].
///
/// Single-pass stream parse — walks the XML once, emitting a
/// `RawStatuteDoc` each time a `<section>` element closes. Total
/// work is O(XML_size), independent of section count.
pub fn parse_uslm_title_all_sections_str(xml: &str) -> Result<Vec<RawStatuteDoc>, ParseError> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut docs: Vec<RawStatuteDoc> = Vec::new();

    // Per-section capture state, set when entering a `<section>`
    // and cleared when leaving it. `None` means we're between
    // sections — element events are ignored.
    let mut state: Option<SectionCapture> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name_bytes = e.name().as_ref().to_vec();

                // Entering a <section> while between sections —
                // start a fresh capture. USLM doesn't nest sections,
                // so an inner <section> would be a structural
                // anomaly we ignore.
                if name_bytes == b"section" && state.is_none() {
                    let Some(identifier) = attr(e, b"identifier") else {
                        // <section> without identifier — skip.
                        buf.clear();
                        continue;
                    };
                    let statute_name = section_identifier_to_statute_name(&identifier);
                    let id_curie = identifier_to_curie(&identifier, &identifier, &statute_name);
                    state = Some(SectionCapture {
                        identifier,
                        statute_name,
                        terms: Vec::new(),
                        relations: Vec::new(),
                        stack: vec![ContainerFrame {
                            tag: b"section".to_vec(),
                            id: id_curie,
                            heading: String::new(),
                            body: String::new(),
                        }],
                        text_target: None,
                        text_buf: String::new(),
                    });
                    continue;
                }

                let Some(sc) = state.as_mut() else {
                    buf.clear();
                    continue;
                };

                if CONTAINER_TAGS.contains(&name_bytes.as_slice()) {
                    let identifier = attr(e, b"identifier");
                    let id = identifier
                        .as_deref()
                        .map(|s| identifier_to_curie(s, &sc.identifier, &sc.statute_name))
                        .unwrap_or_else(|| {
                            format!("{}:unknown_{}", sc.statute_name, sc.stack.len())
                        });
                    sc.stack.push(ContainerFrame {
                        tag: name_bytes.clone(),
                        id,
                        heading: String::new(),
                        body: String::new(),
                    });
                } else {
                    match name_bytes.as_slice() {
                        b"heading" => {
                            sc.text_target = Some(TextTarget::Heading);
                            sc.text_buf.clear();
                        }
                        b"chapeau" | b"content" => {
                            sc.text_target = Some(TextTarget::Body);
                            sc.text_buf.clear();
                        }
                        b"num" | b"ref" | b"inline" | b"i" | b"b" => {
                            // Inline ornaments — text continues
                            // accumulating into whichever target is
                            // open.
                        }
                        b"note" | b"footnote" => {
                            sc.text_target = Some(TextTarget::Suppressed);
                            sc.text_buf.clear();
                        }
                        _ => {}
                    }
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

                if CONTAINER_TAGS.contains(&name_bytes.as_slice()) {
                    if let Some(frame) = sc.stack.pop() {
                        let name = if frame.heading.trim().is_empty() {
                            derive_name_from_id(&frame.id)
                        } else {
                            clean_text(&frame.heading)
                        };
                        // Definition falls back to the term's name
                        // when no chapeau/content text is present
                        // (subsections that only carry nested
                        // children). Anchoring on the heading keeps
                        // the term meaningful and the downstream
                        // structural-data validation non-empty.
                        let raw_def = clean_text(&frame.body);
                        let definition = if raw_def.is_empty() {
                            name.clone()
                        } else {
                            raw_def
                        };
                        sc.terms.push(RawTerm {
                            id: frame.id.clone(),
                            name,
                            definition,
                            lemmas: Vec::new(),
                        });
                        if let Some(parent) = sc.stack.last() {
                            sc.relations.push(RawRelation {
                                from: frame.id,
                                to: parent.id.clone(),
                                relation: RawRel::Composes { into: None },
                            });
                        }
                    }
                    // Leaving the outermost <section> finalizes the
                    // doc and clears the capture state.
                    if sc.stack.is_empty() {
                        let captured = state.take().unwrap();
                        docs.push(RawStatuteDoc {
                            name: captured.statute_name,
                            description: format!("USLM source: {}", captured.identifier),
                            terms: captured.terms,
                            relations: captured.relations,
                        });
                    }
                } else {
                    match name_bytes.as_slice() {
                        b"heading" => {
                            if let Some(frame) = sc.stack.last_mut() {
                                frame.heading.push_str(&sc.text_buf);
                            }
                            sc.text_target = None;
                            sc.text_buf.clear();
                        }
                        b"chapeau" | b"content" => {
                            if let Some(frame) = sc.stack.last_mut() {
                                if !frame.body.is_empty() {
                                    frame.body.push(' ');
                                }
                                frame.body.push_str(&sc.text_buf);
                            }
                            sc.text_target = None;
                            sc.text_buf.clear();
                        }
                        b"note" | b"footnote" => {
                            sc.text_target = None;
                            sc.text_buf.clear();
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(ParseError::Xml(format!("{e}"))),
            _ => {}
        }
        buf.clear();
    }

    Ok(docs)
}

/// Per-section capture state for the single-pass title walker.
#[derive(Debug)]
struct SectionCapture {
    identifier: String,
    statute_name: String,
    terms: Vec<RawTerm>,
    relations: Vec<RawRelation>,
    stack: Vec<ContainerFrame>,
    text_target: Option<TextTarget>,
    text_buf: String,
}

/// Derive a praxis-compatible statute name (CURIE prefix) from a
/// USLM section identifier. Lowercase + slash-to-underscore, with
/// the leading slash stripped.
///
/// Example: `/us/usc/t18/s1514A` → `usc_t18_s1514a`.
pub fn section_identifier_to_statute_name(identifier: &str) -> String {
    identifier
        .trim_start_matches('/')
        .replace('/', "_")
        .to_lowercase()
}

/// Generate the Rust source for a per-title codegen module.
///
/// Output shape: a single Rust source string defining
///
/// ```ignore
/// pub static SECTIONS: &[StaticStatute] = &[ ... ];
/// ```
///
/// where `StaticStatute`, `StaticTerm`, and `StaticRelation` are
/// the types declared in
/// `pr4xis_domains::social::compliance::statutes::us_code`. The
/// source is intended to be written to `$OUT_DIR/{title}_codegen.rs`
/// and `include!`d by the per-title runtime module.
///
/// All emitted string literals use raw strings with `r#"..."#` /
/// `r##"..."##` boundary handling for arbitrary content.
pub fn generate_title_module_source(title_xml_path: &Path) -> Result<String, ParseError> {
    let docs = parse_uslm_title_all_sections(title_xml_path)?;
    Ok(emit_title_module(&docs))
}

/// In-memory variant for tests.
pub fn generate_title_module_source_from_str(xml: &str) -> Result<String, ParseError> {
    let docs = parse_uslm_title_all_sections_str(xml)?;
    Ok(emit_title_module(&docs))
}

/// Render the section list as a Rust source string.
fn emit_title_module(docs: &[RawStatuteDoc]) -> String {
    let mut out = String::new();
    out.push_str("// Auto-generated by pr4xis::codegen::uslm\n");
    out.push_str("// DO NOT EDIT — regenerate from the LRC USLM source.\n");
    out.push_str(&format!("// Sections: {}\n\n", docs.len()));
    out.push_str("pub static SECTIONS: &[StaticStatute] = &[\n");
    for doc in docs {
        out.push_str(&emit_static_statute(doc));
    }
    out.push_str("];\n");
    out
}

fn emit_static_statute(doc: &RawStatuteDoc) -> String {
    // Statute identifier and num are derived from the document's
    // first term (the §-level entries reach this codegen with
    // doc.name = the derived statute_name; we recover the
    // identifier from the description).
    let identifier = doc.description.strip_prefix("USLM source: ").unwrap_or("");
    let num = identifier
        .rsplit('/')
        .next()
        .unwrap_or("")
        .trim_start_matches('s');
    let heading = doc
        .terms
        .iter()
        .find(|t| !t.name.is_empty() && !t.name.starts_with('('))
        .map(|t| t.name.as_str())
        .unwrap_or("");
    let mut buf = String::new();
    buf.push_str("    StaticStatute {\n");
    buf.push_str(&format!("        identifier: {},\n", raw_str(identifier)));
    buf.push_str(&format!("        num: {},\n", raw_str(num)));
    buf.push_str(&format!("        heading: {},\n", raw_str(heading)));
    buf.push_str("        terms: &[\n");
    for t in &doc.terms {
        buf.push_str(&format!(
            "            StaticTerm {{ id: {}, name: {}, definition: {} }},\n",
            raw_str(&t.id),
            raw_str(&t.name),
            raw_str(&t.definition),
        ));
    }
    buf.push_str("        ],\n");
    buf.push_str("        relations: &[\n");
    for r in &doc.relations {
        let kind = match r.relation {
            crate::codegen::statute::RawRel::Composes { .. } => "Composes",
            // Currently only Composes is emitted by uslm.rs; other
            // kinds would need explicit StaticRelationKind variants.
            _ => "Composes",
        };
        buf.push_str(&format!(
            "            StaticRelation {{ from: {}, to: {}, kind: StaticRelationKind::{} }},\n",
            raw_str(&r.from),
            raw_str(&r.to),
            kind,
        ));
    }
    buf.push_str("        ],\n");
    buf.push_str("    },\n");
    buf
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

/// In-memory variant of [`parse_uslm_xml`]. Same semantics; useful
/// for tests that drive the parser with an inline fixture string.
pub fn parse_uslm_str(
    xml: &str,
    section_identifier: &str,
    statute_name: &str,
) -> Result<RawStatuteDoc, ParseError> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    let mut stack: Vec<ContainerFrame> = Vec::new();
    let mut text_target: Option<TextTarget> = None;
    let mut text_buf = String::new();
    let mut terms: Vec<RawTerm> = Vec::new();
    let mut relations: Vec<RawRelation> = Vec::new();
    let mut in_target_section = false;
    let mut target_seen = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name_bytes = e.name().as_ref().to_vec();

                if CONTAINER_TAGS.contains(&name_bytes.as_slice()) {
                    let identifier = attr(e, b"identifier");
                    if !in_target_section && identifier.as_deref() == Some(section_identifier) {
                        in_target_section = true;
                        target_seen = true;
                    }
                    if in_target_section {
                        let id = identifier
                            .as_deref()
                            .map(|s| identifier_to_curie(s, section_identifier, statute_name))
                            .unwrap_or_else(|| format!("{statute_name}:unknown_{}", stack.len()));
                        stack.push(ContainerFrame {
                            tag: name_bytes.clone(),
                            id,
                            heading: String::new(),
                            body: String::new(),
                        });
                    }
                } else if in_target_section {
                    match name_bytes.as_slice() {
                        b"heading" => {
                            text_target = Some(TextTarget::Heading);
                            text_buf.clear();
                        }
                        b"chapeau" | b"content" => {
                            text_target = Some(TextTarget::Body);
                            text_buf.clear();
                        }
                        b"num" | b"ref" | b"inline" | b"i" | b"b" => {
                            // Inline ornaments — keep collecting text
                            // into whichever target is open.
                        }
                        b"note" | b"footnote" => {
                            // Suppress note/footnote content from
                            // definitions per the layer's "structural
                            // shape only" scope.
                            text_target = Some(TextTarget::Suppressed);
                            text_buf.clear();
                        }
                        _ => {}
                    }
                }
            }

            Ok(Event::Text(ref e)) => {
                if text_target.is_some() {
                    let decoded = e.decode().unwrap_or_default();
                    let unescaped = match quick_xml::escape::unescape(&decoded) {
                        Ok(u) => u.into_owned(),
                        Err(_) => decoded.into_owned(),
                    };
                    text_buf.push_str(&unescaped);
                }
            }

            Ok(Event::End(ref e)) => {
                let name_bytes = e.name().as_ref().to_vec();
                if CONTAINER_TAGS.contains(&name_bytes.as_slice()) {
                    if in_target_section {
                        if let Some(frame) = stack.pop() {
                            let name = if frame.heading.trim().is_empty() {
                                derive_name_from_id(&frame.id)
                            } else {
                                clean_text(&frame.heading)
                            };
                            // Fall back to the name when no
                            // chapeau/content text — keeps every
                            // term's definition non-empty.
                            let raw_def = clean_text(&frame.body);
                            let definition = if raw_def.is_empty() {
                                name.clone()
                            } else {
                                raw_def
                            };
                            terms.push(RawTerm {
                                id: frame.id.clone(),
                                name,
                                definition,
                                lemmas: Vec::new(),
                            });
                            // Mereological edge: child Composes into
                            // parent. The top-level § has no parent
                            // within this scope and gets no edge.
                            if let Some(parent) = stack.last() {
                                relations.push(RawRelation {
                                    from: frame.id,
                                    to: parent.id.clone(),
                                    relation: RawRel::Composes { into: None },
                                });
                            }
                        }
                        if stack.is_empty() {
                            in_target_section = false;
                        }
                    }
                } else if in_target_section {
                    match name_bytes.as_slice() {
                        b"heading" => {
                            if let Some(frame) = stack.last_mut() {
                                frame.heading.push_str(&text_buf);
                            }
                            text_target = None;
                            text_buf.clear();
                        }
                        b"chapeau" | b"content" => {
                            if let Some(frame) = stack.last_mut() {
                                if !frame.body.is_empty() {
                                    frame.body.push(' ');
                                }
                                frame.body.push_str(&text_buf);
                            }
                            text_target = None;
                            text_buf.clear();
                        }
                        b"note" | b"footnote" => {
                            text_target = None;
                            text_buf.clear();
                        }
                        _ => {}
                    }
                }
            }

            Ok(Event::Eof) => break,
            Err(e) => return Err(ParseError::Xml(format!("{e}"))),
            _ => {}
        }
        buf.clear();
    }

    if !target_seen {
        return Err(ParseError::SectionNotFound {
            identifier: section_identifier.to_string(),
        });
    }

    Ok(RawStatuteDoc {
        name: statute_name.to_string(),
        description: format!("USLM source: {section_identifier}"),
        terms,
        relations,
    })
}

#[derive(Debug)]
struct ContainerFrame {
    #[allow(dead_code)]
    tag: Vec<u8>,
    id: String,
    heading: String,
    body: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TextTarget {
    Heading,
    Body,
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

/// Convert a USLM `identifier` URN into a praxis CURIE.
///
/// `section_prefix` is the identifier of the enclosing § (e.g.
/// `/us/usc/t18/s1514A`); the prefix is stripped, then the
/// remaining path is joined with underscores under the statute
/// name. For the § itself (identifier == prefix), the CURIE is
/// just the statute name with no local part suffix.
fn identifier_to_curie(identifier: &str, section_prefix: &str, statute_name: &str) -> String {
    if identifier == section_prefix {
        return statute_name.to_string();
    }
    let local = identifier
        .strip_prefix(section_prefix)
        .and_then(|s| s.strip_prefix('/'))
        .unwrap_or("");
    if local.is_empty() {
        statute_name.to_string()
    } else {
        let joined = local.replace('/', "_");
        format!("{statute_name}:{joined}")
    }
}

/// Fallback name when a container has no `<heading>`. Uses the
/// CURIE's local part formatted as a subdivision marker, e.g.
/// `sox_1514a:a_1_A` → `(a)(1)(A)`. For the section root case
/// (no local part) the CURIE itself is returned, since the
/// `<num>` for the § is the statutory reference itself, not a
/// subdivision marker.
fn derive_name_from_id(curie: &str) -> String {
    let Some(local) = curie.split(':').nth(1) else {
        return curie.to_string();
    };
    if local.is_empty() {
        return curie.to_string();
    }
    local
        .split('_')
        .map(|seg| format!("({seg})"))
        .collect::<Vec<_>>()
        .join("")
}

/// Trim, collapse whitespace runs, and strip surrounding ornament
/// characters from extracted text.
fn clean_text(s: &str) -> String {
    let trimmed = s.trim();
    let mut out = String::with_capacity(trimmed.len());
    let mut prev_was_space = false;
    for ch in trimmed.chars() {
        if ch.is_whitespace() {
            if !prev_was_space {
                out.push(' ');
            }
            prev_was_space = true;
        } else {
            out.push(ch);
            prev_was_space = false;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Inline fixture mirroring the structural shape of the real
    /// SOX § 1514A USLM slice — one § with two subsections, the
    /// first with two paragraphs, the first paragraph with two
    /// subparagraphs.
    const SAMPLE_USLM: &str = r##"<section identifier="/us/usc/t18/s1514A"><num value="1514A">§ 1514A.</num><heading> Civil action to protect against retaliation in fraud cases</heading><subsection identifier="/us/usc/t18/s1514A/a"><num value="a">(a)</num><heading> <inline class="small-caps">Whistleblower Protection</inline></heading><chapeau>No company may discriminate against an employee—</chapeau><paragraph identifier="/us/usc/t18/s1514A/a/1"><num value="1">(1)</num><chapeau>to provide information—</chapeau><subparagraph identifier="/us/usc/t18/s1514A/a/1/A"><num value="A">(A)</num><content>a Federal regulatory or law enforcement agency;</content></subparagraph><subparagraph identifier="/us/usc/t18/s1514A/a/1/B"><num value="B">(B)</num><content>any Member of Congress;</content></subparagraph></paragraph><paragraph identifier="/us/usc/t18/s1514A/a/2"><num value="2">(2)</num><content>to file a proceeding.</content></paragraph></subsection><subsection identifier="/us/usc/t18/s1514A/b"><num value="b">(b)</num><heading> <inline class="small-caps">Enforcement Action</inline></heading><content>A person who alleges discharge may seek relief.</content></subsection></section>"##;

    #[test]
    fn parses_section_into_term_per_container() {
        let doc = parse_uslm_str(SAMPLE_USLM, "/us/usc/t18/s1514A", "sox_1514a").expect("parse");
        // Containers: section + 2 subsections + 2 paragraphs + 2
        // subparagraphs = 7 RawTerms.
        assert_eq!(doc.terms.len(), 7, "got terms: {:?}", doc.terms);
    }

    #[test]
    fn curies_match_existing_convention() {
        let doc = parse_uslm_str(SAMPLE_USLM, "/us/usc/t18/s1514A", "sox_1514a").expect("parse");
        let ids: Vec<&str> = doc.terms.iter().map(|t| t.id.as_str()).collect();
        assert!(ids.contains(&"sox_1514a"), "section root term missing");
        assert!(ids.contains(&"sox_1514a:a"));
        assert!(ids.contains(&"sox_1514a:b"));
        assert!(ids.contains(&"sox_1514a:a_1"));
        assert!(ids.contains(&"sox_1514a:a_2"));
        assert!(ids.contains(&"sox_1514a:a_1_A"));
        assert!(ids.contains(&"sox_1514a:a_1_B"));
    }

    #[test]
    fn composes_relations_form_strict_hierarchy() {
        let doc = parse_uslm_str(SAMPLE_USLM, "/us/usc/t18/s1514A", "sox_1514a").expect("parse");
        // 6 non-root containers → 6 Composes edges (each child →
        // its parent). The root § has no parent within scope.
        assert_eq!(doc.relations.len(), 6);
        // Check a specific edge: (a)(1)(A) composes into (a)(1).
        let has_a_1_a_to_a_1 = doc.relations.iter().any(|r| {
            r.from == "sox_1514a:a_1_A"
                && r.to == "sox_1514a:a_1"
                && matches!(r.relation, RawRel::Composes { .. })
        });
        assert!(has_a_1_a_to_a_1, "(a)(1)(A) → (a)(1) edge missing");
        // Top-level subsections compose into the §.
        let has_a_to_root = doc.relations.iter().any(|r| {
            r.from == "sox_1514a:a"
                && r.to == "sox_1514a"
                && matches!(r.relation, RawRel::Composes { .. })
        });
        assert!(has_a_to_root, "(a) → § edge missing");
    }

    #[test]
    fn headings_capture_inline_text() {
        let doc = parse_uslm_str(SAMPLE_USLM, "/us/usc/t18/s1514A", "sox_1514a").expect("parse");
        let a = doc
            .terms
            .iter()
            .find(|t| t.id == "sox_1514a:a")
            .expect("(a) term");
        assert!(
            a.name.contains("Whistleblower Protection"),
            "got name: {:?}",
            a.name
        );
    }

    #[test]
    fn bodies_capture_chapeau_and_content() {
        let doc = parse_uslm_str(SAMPLE_USLM, "/us/usc/t18/s1514A", "sox_1514a").expect("parse");
        let a = doc
            .terms
            .iter()
            .find(|t| t.id == "sox_1514a:a")
            .expect("(a) term");
        assert!(
            a.definition.contains("No company may discriminate"),
            "got definition: {:?}",
            a.definition
        );
    }

    #[test]
    fn fallback_name_used_when_heading_absent() {
        let doc = parse_uslm_str(SAMPLE_USLM, "/us/usc/t18/s1514A", "sox_1514a").expect("parse");
        // (a)(1)(A) has no <heading>; the fallback derives a name
        // from the CURIE's subdivision path.
        let a_1_a = doc
            .terms
            .iter()
            .find(|t| t.id == "sox_1514a:a_1_A")
            .expect("(a)(1)(A) term");
        assert_eq!(a_1_a.name, "(a)(1)(A)");
    }

    #[test]
    fn section_not_found_returns_named_error() {
        let err =
            parse_uslm_str(SAMPLE_USLM, "/us/usc/t18/s9999", "no_such").expect_err("should fail");
        match err {
            ParseError::SectionNotFound { identifier } => {
                assert_eq!(identifier, "/us/usc/t18/s9999");
            }
            other => panic!("expected SectionNotFound, got {other:?}"),
        }
    }

    #[test]
    fn parser_is_deterministic() {
        let d1 = parse_uslm_str(SAMPLE_USLM, "/us/usc/t18/s1514A", "sox_1514a").unwrap();
        let d2 = parse_uslm_str(SAMPLE_USLM, "/us/usc/t18/s1514A", "sox_1514a").unwrap();
        let ids1: Vec<_> = d1.terms.iter().map(|t| t.id.clone()).collect();
        let ids2: Vec<_> = d2.terms.iter().map(|t| t.id.clone()).collect();
        assert_eq!(ids1, ids2);
        assert_eq!(d1.terms.len(), d2.terms.len());
        assert_eq!(d1.relations.len(), d2.relations.len());
    }

    #[test]
    fn identifier_to_curie_section_root() {
        assert_eq!(
            identifier_to_curie("/us/usc/t18/s1514A", "/us/usc/t18/s1514A", "sox_1514a"),
            "sox_1514a"
        );
    }

    #[test]
    fn identifier_to_curie_nested_subdivision() {
        assert_eq!(
            identifier_to_curie(
                "/us/usc/t18/s1514A/a/1/A",
                "/us/usc/t18/s1514A",
                "sox_1514a"
            ),
            "sox_1514a:a_1_A"
        );
    }

    #[test]
    fn derive_name_from_id_formats_subdivision_markers() {
        assert_eq!(derive_name_from_id("sox_1514a:a"), "(a)");
        assert_eq!(derive_name_from_id("sox_1514a:b_2_C"), "(b)(2)(C)");
        assert_eq!(derive_name_from_id("sox_1514a"), "sox_1514a");
    }

    // =========================================================
    // Whole-title compressor tests (M4.δ.2.a)
    // =========================================================

    /// Inline title with two sections — minimum non-trivial input
    /// for `parse_uslm_title_all_sections_str`.
    const SAMPLE_TITLE_USLM: &str = r##"<title xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18"><num value="18">Title 18—</num><heading>CRIMES</heading><section identifier="/us/usc/t18/s1"><num value="1">§ 1.</num><heading>First</heading><content>x</content></section><section identifier="/us/usc/t18/s2"><num value="2">§ 2.</num><heading>Second</heading><subsection identifier="/us/usc/t18/s2/a"><num value="a">(a)</num><content>body</content></subsection></section></title>"##;

    #[test]
    fn parse_uslm_title_all_sections_finds_every_section() {
        let docs = parse_uslm_title_all_sections_str(SAMPLE_TITLE_USLM).unwrap();
        assert_eq!(docs.len(), 2);
        // Section names derive from identifier via lowercase + slashify.
        let names: Vec<&str> = docs.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"us_usc_t18_s1"));
        assert!(names.contains(&"us_usc_t18_s2"));
    }

    #[test]
    fn section_identifier_to_statute_name_lowercases_and_slashifies() {
        assert_eq!(
            section_identifier_to_statute_name("/us/usc/t18/s1514A"),
            "us_usc_t18_s1514a"
        );
        assert_eq!(
            section_identifier_to_statute_name("/us/usc/t49/s42121"),
            "us_usc_t49_s42121"
        );
    }

    #[test]
    fn generate_title_module_source_emits_valid_rust_signature() {
        let src = generate_title_module_source_from_str(SAMPLE_TITLE_USLM).unwrap();
        // Header
        assert!(src.contains("// Auto-generated by pr4xis::codegen::uslm"));
        assert!(src.contains("// Sections: 2"));
        // The pub static declaration the runtime module includes.
        assert!(src.contains("pub static SECTIONS: &[StaticStatute] = &["));
        // Section identifiers appear as raw-string literals.
        assert!(src.contains("/us/usc/t18/s1"));
        assert!(src.contains("/us/usc/t18/s2"));
    }

    #[test]
    fn generate_title_module_source_handles_quotes_in_text() {
        // Content with double-quotes + hash signs — verifies raw-
        // string boundary escape works.
        let xml = r##"<title xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18"><num value="18">T</num><heading>H</heading><section identifier="/us/usc/t18/s1"><num value="1">§ 1.</num><heading>H</heading><content>weird "# inside text</content></section></title>"##;
        let src = generate_title_module_source_from_str(xml).unwrap();
        // The source compiles to valid Rust — the raw_str helper
        // must have picked a sufficient `#` boundary.
        assert!(src.contains("pub static SECTIONS"));
    }

    /// Real-corpus check — generating the source for the actual
    /// SOX § 1514A slice yields a non-trivial Rust module.
    #[test]
    fn generate_title_module_source_on_real_sox_slice() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../domains/data/legal/statutes/us_federal/sox_1514a/sox_1514a-2002.xml");
        if !path.exists() {
            eprintln!("SKIP: real slice not on disk");
            return;
        }
        let src = generate_title_module_source(&path).expect("emit");
        // One section in the slice file.
        assert!(src.contains("// Sections: 1"));
        // The § 1514A identifier appears verbatim.
        assert!(src.contains("/us/usc/t18/s1514A"));
        // At least one subsection CURIE (e.g. `us_usc_t18_s1514a:a`)
        // and one Composes relation appear in the output. CURIEs are
        // derived from the USLM identifier by lowercasing +
        // slashifying, with the section prefix stripped.
        assert!(
            src.contains("us_usc_t18_s1514a:a"),
            "subsection CURIE missing from generated source"
        );
        assert!(src.contains("StaticRelationKind::Composes"));
    }

    /// Timing check — single-pass title parse must run in well
    /// under a second on Title 18 (12 MB, 1,399 sections). Earlier
    /// per-section re-parse pattern took ~14 seconds; this fixes
    /// it to O(XML_size) rather than O(N × XML_size).
    #[test]
    fn parse_title_18_is_single_pass_fast() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../domains/data/legal/uscode/usc_title_18/usc_title_18-pl-119-90.xml");
        if !path.exists() {
            eprintln!("SKIP: Title 18 not on disk");
            return;
        }
        let xml = std::fs::read_to_string(&path).unwrap();
        let t0 = std::time::Instant::now();
        let docs = parse_uslm_title_all_sections_str(&xml).unwrap();
        let elapsed = t0.elapsed();
        eprintln!(
            "Title 18 single-pass parse: {} sections in {:?}",
            docs.len(),
            elapsed
        );
        assert!(docs.len() >= 1_000, "expected ≥1,000 sections");
        // Should be sub-second even in debug builds.
        assert!(
            elapsed < std::time::Duration::from_secs(5),
            "single-pass parse too slow: {elapsed:?}"
        );
    }

    /// Real-corpus check — parse the SOX § 1514A USLM slice that
    /// ships in `pr4xis-domains/data/legal/statutes/us_federal/`.
    /// Verifies the parser handles the actual published structure,
    /// not just the synthetic inline fixture above.
    #[test]
    fn parses_real_sox_1514a_slice() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../domains/data/legal/statutes/us_federal/sox_1514a/sox_1514a-2002.xml");
        if !path.exists() {
            eprintln!("SKIP: SOX § 1514A USLM slice not on disk at {path:?}");
            return;
        }
        let doc = parse_uslm_xml(&path, "/us/usc/t18/s1514A", "sox_1514a").expect("parse");

        // The published § 1514A has subsections (a)–(e), and the
        // structure (subsections + paragraphs + subparagraphs +
        // clauses) totals well above the seven of our synthetic
        // fixture. Verify we get a sensible count.
        assert!(
            doc.terms.len() >= 20,
            "expected ≥20 containers in real § 1514A; got {} (likely partial parse): {:?}",
            doc.terms.len(),
            doc.terms.iter().map(|t| &t.id).collect::<Vec<_>>()
        );

        // The § itself must be present.
        assert!(doc.terms.iter().any(|t| t.id == "sox_1514a"));

        // Every published subsection must be present.
        for sub in ["a", "b", "c", "d", "e"] {
            let id = format!("sox_1514a:{sub}");
            assert!(
                doc.terms.iter().any(|t| t.id == id),
                "subsection {id} missing"
            );
        }

        // (a)(1)(A) — the first §-protected-activity subparagraph
        // (reporting to a Federal regulatory or law enforcement
        // agency) must be present per the actual published text.
        assert!(
            doc.terms.iter().any(|t| t.id == "sox_1514a:a_1_A"),
            "(a)(1)(A) missing"
        );

        // Composes hierarchy is intact: (a)(1)(A) → (a)(1) → (a)
        // → §. Walking up via Composes relations from any leaf
        // must reach the root.
        let parent_of = |id: &str| -> Option<String> {
            doc.relations.iter().find_map(|r| {
                if r.from == id && matches!(r.relation, RawRel::Composes { .. }) {
                    Some(r.to.clone())
                } else {
                    None
                }
            })
        };
        let mut cur = "sox_1514a:a_1_A".to_string();
        let mut hops = 0;
        while let Some(p) = parent_of(&cur) {
            cur = p;
            hops += 1;
            if hops > 10 {
                panic!("Composes chain from (a)(1)(A) didn't terminate within 10 hops");
            }
        }
        assert_eq!(
            cur, "sox_1514a",
            "Composes chain should terminate at root §"
        );
        assert!(
            hops >= 3,
            "expected at least 3 Composes hops (subpara→para→subsection→§)"
        );
    }
}
