//! USC corpus codegen — walks every registered USC title XML and
//! emits a single `CodegenData<UsCode>` static populated with one
//! entity per `<section>`.
//!
//! Parallel to [`super::wordnet`] for English. Where the WordNet
//! codegen produces an `OntologyBuilder` with one entity per synset,
//! this codegen produces an `OntologyBuilder` with one entity per USC
//! section. Both flow through [`super::generate::generate_rust`] so
//! the emitted code reuses the same `CodegenData<P>` transport.
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
//! No taxonomy / mereology / opposition / equivalence / causation /
//! references / word_index edges are emitted in the first cut. See
//! the parent task's scope notes.
//!
//! ## Literature
//!
//! - U.S. House Office of the Law Revision Counsel, *USLM XML User
//!   Guide (USLM-1.0.15.xsd)*. <https://uscode.house.gov/uslm/>.
//! - 1 U.S.C. § 204 — *Codes and Supplements; positive law titles*.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use super::builder::{EntityDef, GenerateConfig, OntologyBuilder};
use super::generate::generate_rust;
use super::uslm::ParseError;

/// Walk every registered USC title XML in `title_xml_paths`, extract
/// one `<section>` per entity, and emit the generated Rust source
/// string for `CodegenData<UsCode>`.
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
    for path in title_xml_paths {
        let xml = std::fs::read_to_string(path)
            .map_err(|e| ParseError::Read(path.display().to_string(), e))?;
        extract_sections_into_builder(&xml, &mut builder)?;
    }
    Ok(generate_rust(&builder, config))
}

/// In-memory variant of [`generate_usc_corpus_source`] for tests.
pub fn generate_usc_corpus_source_from_strs(
    xmls: &[&str],
    config: &GenerateConfig,
) -> Result<String, ParseError> {
    let mut builder = OntologyBuilder::new();
    for xml in xmls {
        extract_sections_into_builder(xml, &mut builder)?;
    }
    Ok(generate_rust(&builder, config))
}

/// Single-pass walker — for each `<section>` element in the XML,
/// append one `EntityDef` to `builder`. Heading and body text are
/// accumulated from `<heading>`, `<chapeau>`, and `<content>`
/// children at any nesting depth within the section.
///
/// `<note>` / `<footnote>` content is suppressed from the body so
/// editorial annotations don't pollute the section text.
fn extract_sections_into_builder(
    xml: &str,
    builder: &mut OntologyBuilder,
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

                if name_bytes == b"section" && state.is_none() {
                    let Some(identifier) = attr(e, b"identifier") else {
                        buf.clear();
                        continue;
                    };
                    state = Some(SectionCapture {
                        identifier,
                        heading: String::new(),
                        body: String::new(),
                        text_target: None,
                        text_buf: String::new(),
                        note_depth: 0,
                    });
                    continue;
                }

                let Some(sc) = state.as_mut() else {
                    buf.clear();
                    continue;
                };

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
                    b"chapeau" | b"content" if sc.note_depth == 0 => {
                        sc.text_target = Some(TextTarget::Body);
                        sc.text_buf.clear();
                    }
                    // Inline ornaments — text flows into whichever
                    // accumulator is open.
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

                if name_bytes == b"section" {
                    let captured = state.take().unwrap();
                    let heading = clean_text(&captured.heading);
                    let body = clean_text(&captured.body);
                    let mut entity = EntityDef::new(&captured.identifier, &heading).pos("section");
                    if !body.is_empty() {
                        entity = entity.definition(&body);
                    }
                    builder.add_entity(entity);
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
                        if !sc.heading.is_empty() {
                            sc.heading.push(' ');
                        }
                        sc.heading.push_str(&sc.text_buf);
                        sc.text_target = None;
                        sc.text_buf.clear();
                    }
                    b"chapeau" | b"content" if sc.note_depth == 0 => {
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

    #[test]
    fn emits_one_entity_per_section_across_multiple_titles() {
        let mut builder = OntologyBuilder::new();
        extract_sections_into_builder(SAMPLE_TITLE_USLM, &mut builder).expect("parse");
        assert_eq!(builder.entity_count(), 2);
        let ids: Vec<&str> = builder.entities.iter().map(|e| e.id.as_str()).collect();
        assert!(ids.contains(&"/us/usc/t18/s1"));
        assert!(ids.contains(&"/us/usc/t18/s2"));
    }

    #[test]
    fn section_headings_captured() {
        let mut builder = OntologyBuilder::new();
        extract_sections_into_builder(SAMPLE_TITLE_USLM, &mut builder).expect("parse");
        let labels: Vec<&str> = builder.entities.iter().map(|e| e.label.as_str()).collect();
        assert!(labels.contains(&"First"));
        assert!(labels.contains(&"Second"));
    }

    #[test]
    fn section_bodies_concatenate_chapeau_and_content_text() {
        let mut builder = OntologyBuilder::new();
        extract_sections_into_builder(SAMPLE_TITLE_USLM, &mut builder).expect("parse");
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
        extract_sections_into_builder(SAMPLE_TITLE_USLM, &mut builder).expect("parse");
        for e in &builder.entities {
            assert_eq!(e.pos.as_deref(), Some("section"));
        }
    }

    #[test]
    fn no_relations_emitted_in_first_cut() {
        let mut builder = OntologyBuilder::new();
        extract_sections_into_builder(SAMPLE_TITLE_USLM, &mut builder).expect("parse");
        assert_eq!(builder.relation_count(), 0);
        assert_eq!(builder.word_index.len(), 0);
    }

    #[test]
    fn generate_source_emits_codegen_data_marker() {
        let src = generate_usc_corpus_source_from_strs(&[SAMPLE_TITLE_USLM], &cfg()).expect("emit");
        assert!(src.contains("pub static CODEGEN_DATA"));
        assert!(
            src.contains("pr4xis_domains::social::software::markup::xml::uslm::corpus::UsCode")
        );
        // Section URNs reach the emitted source.
        assert!(src.contains("/us/usc/t18/s1"));
        assert!(src.contains("/us/usc/t18/s2"));
    }

    #[test]
    fn notes_are_suppressed_from_body() {
        let xml = r##"<title xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18"><section identifier="/us/usc/t18/s1"><heading>Hed</heading><content>keep this</content><note>drop this</note></section></title>"##;
        let mut builder = OntologyBuilder::new();
        extract_sections_into_builder(xml, &mut builder).expect("parse");
        let s = &builder.entities[0];
        let body = s.definitions.first().cloned().unwrap_or_default();
        assert!(body.contains("keep this"));
        assert!(!body.contains("drop this"), "got: {body:?}");
    }

    #[test]
    fn empty_input_yields_empty_corpus() {
        let mut builder = OntologyBuilder::new();
        extract_sections_into_builder(
            r##"<title xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18"><heading>X</heading></title>"##,
            &mut builder,
        )
        .expect("parse");
        assert_eq!(builder.entity_count(), 0);
    }
}
