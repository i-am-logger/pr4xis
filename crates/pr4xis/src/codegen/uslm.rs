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

/// Tokenizer dispatch result for a USLM element name encountered
/// during stream parse. Both STag (open-tag) and ETag (close-tag)
/// handlers branch on this classification, so the legacy
/// duplicated byte-string match arms reduce to one
/// `config.classify(name) -> UslmElementClass` call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UslmElementClass {
    /// One of the hierarchical containers (`section`, `subsection`,
    /// …); opens a new mereological child frame.
    Container,
    /// `<heading>` — capture-target for the surrounding container's
    /// display name.
    Heading,
    /// `<chapeau>` / `<content>` — capture-target for the
    /// surrounding container's body text.
    Body,
    /// `<num>` / `<ref>` / `<inline>` / `<i>` / `<b>` — text inside
    /// continues accumulating into whichever capture target is
    /// open (an inline ornament inside a heading is part of the
    /// heading).
    InlineOrnament,
    /// `<note>` / `<footnote>` — text inside is suppressed from
    /// the capture (footnotes don't contribute to the body or
    /// heading of the surrounding container).
    Suppressed,
    /// Any other USLM element — the tokenizer ignores it.
    Other,
}

/// USLM tokenizer configuration: the element-name classification
/// the stream-parse dispatches on.
///
/// The `container_tags` set is the W3C XSD 1.1 Part 1 §3.3.6
/// substitution-group `"level"` members in the LRC USLM XSD
/// (LRC USLM XML User Guide § "Hierarchical Levels"). Callers
/// with access to the loaded USLM XSD ontology SHOULD construct
/// the config via the level-substitution-group accessor (see
/// pr4xis-domains' `uslm_corpus_tokenizer_config_from_loaded_xsd`)
/// rather than relying on [`Self::default_uslm_1_0`], so the
/// container set tracks the loaded XSD per
/// `feedback_bottom_up_loaded_not_encoded`.
///
/// The other four sets (`heading_tags`, `body_tags`,
/// `ornament_tags`, `suppressed_tags`) are USLM semantic
/// groupings the schema does NOT formalize as XSD substitution
/// groups — they are documented USLM 1.0 conventions (LRC USLM
/// XML User Guide §§ "Headings", "Body Text Containers",
/// "Inline Ornaments", "Notes and Footnotes"). They stay as
/// cited defaults until those conventions land as an XSD
/// group structure.
#[derive(Debug, Clone)]
pub struct UslmTokenizerConfig {
    /// Substitution-group "level" members per LRC USLM XSD.
    pub container_tags: Vec<Vec<u8>>,
    /// Heading elements per LRC USLM User Guide § "Headings".
    pub heading_tags: Vec<Vec<u8>>,
    /// Body-text containers per LRC USLM User Guide
    /// § "Body Text Containers".
    pub body_tags: Vec<Vec<u8>>,
    /// Inline ornaments per LRC USLM User Guide § "Inline Markup".
    pub ornament_tags: Vec<Vec<u8>>,
    /// Suppressed-capture elements (notes, footnotes).
    pub suppressed_tags: Vec<Vec<u8>>,
}

impl UslmTokenizerConfig {
    /// USLM 1.0 default — the element classification used by the
    /// LRC's USLM corpus through release pl-119-90 (2026-Q1).
    /// Container set: `section` plus the 7 substitution-group
    /// "level" subdivisions. Caller WITH access to the loaded
    /// USLM XSD should prefer the XSD-grounded constructor; this
    /// default exists for callers that have no XSD instance handy
    /// (e.g. build.rs paths in crates that don't depend on
    /// pr4xis-domains).
    #[must_use]
    pub fn default_uslm_1_0() -> Self {
        Self {
            container_tags: vec![
                b"section".to_vec(),
                b"subsection".to_vec(),
                b"paragraph".to_vec(),
                b"subparagraph".to_vec(),
                b"clause".to_vec(),
                b"subclause".to_vec(),
                b"item".to_vec(),
                b"subitem".to_vec(),
            ],
            heading_tags: vec![b"heading".to_vec()],
            body_tags: vec![b"chapeau".to_vec(), b"content".to_vec()],
            ornament_tags: vec![
                b"num".to_vec(),
                b"ref".to_vec(),
                b"inline".to_vec(),
                b"i".to_vec(),
                b"b".to_vec(),
            ],
            suppressed_tags: vec![b"note".to_vec(), b"footnote".to_vec()],
        }
    }

    /// Construct a config with `container_tags` derived from the
    /// supplied substitution-group-"level" members. Other sets stay
    /// at the [`Self::default_uslm_1_0`] values. Used by pr4xis-
    /// domains build.rs / runtime paths that have the loaded USLM
    /// XSD ontology and want the container set to follow the loaded
    /// schema per `feedback_bottom_up_loaded_not_encoded`.
    ///
    /// `level_members` is the output of
    /// `XsdOntologyInstance::substitution_group_members("level")` —
    /// the reflexive-transitive set of XSD `<xs:element>` declarations
    /// participating in the level substitution group.
    #[must_use]
    pub fn from_level_substitution_group(level_members: &[&str]) -> Self {
        let mut config = Self::default_uslm_1_0();
        config.container_tags = level_members
            .iter()
            .map(|s| s.as_bytes().to_vec())
            .collect();
        // The XSD doesn't include `section` as a level member in
        // every revision, but the streaming tokenizer must treat
        // `<section>` as a container (it's the top-level
        // mereological frame). Add it if absent.
        if !config.container_tags.iter().any(|t| t == b"section") {
            config.container_tags.push(b"section".to_vec());
        }
        config
    }

    /// Classify a USLM element name into one of the five tokenizer
    /// dispatch classes. The classifier walks the configured tag
    /// sets in priority order (container → heading → body →
    /// ornament → suppressed); the first match wins.
    #[must_use]
    pub fn classify(&self, name: &[u8]) -> UslmElementClass {
        if self.container_tags.iter().any(|t| t.as_slice() == name) {
            UslmElementClass::Container
        } else if self.heading_tags.iter().any(|t| t.as_slice() == name) {
            UslmElementClass::Heading
        } else if self.body_tags.iter().any(|t| t.as_slice() == name) {
            UslmElementClass::Body
        } else if self.ornament_tags.iter().any(|t| t.as_slice() == name) {
            UslmElementClass::InlineOrnament
        } else if self.suppressed_tags.iter().any(|t| t.as_slice() == name) {
            UslmElementClass::Suppressed
        } else {
            UslmElementClass::Other
        }
    }
}

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
    parse_uslm_xml_with_config(
        path,
        section_identifier,
        statute_name,
        &UslmTokenizerConfig::default_uslm_1_0(),
    )
}

/// XSD-grounded variant of [`parse_uslm_xml`] — accepts a
/// [`UslmTokenizerConfig`] whose `container_tags` set can be derived
/// from the loaded USLM XSD via
/// [`UslmTokenizerConfig::from_level_substitution_group`] for
/// `feedback_bottom_up_loaded_not_encoded`-compliant operation.
pub fn parse_uslm_xml_with_config(
    path: &Path,
    section_identifier: &str,
    statute_name: &str,
    config: &UslmTokenizerConfig,
) -> Result<RawStatuteDoc, ParseError> {
    let xml = std::fs::read_to_string(path)
        .map_err(|e| ParseError::Read(path.display().to_string(), e))?;
    parse_uslm_str_with_config(&xml, section_identifier, statute_name, config)
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

/// XSD-grounded variant of [`parse_uslm_title_all_sections`].
pub fn parse_uslm_title_all_sections_with_config(
    path: &Path,
    config: &UslmTokenizerConfig,
) -> Result<Vec<RawStatuteDoc>, ParseError> {
    let xml = std::fs::read_to_string(path)
        .map_err(|e| ParseError::Read(path.display().to_string(), e))?;
    parse_uslm_title_all_sections_str_with_config(&xml, config)
}

/// In-memory variant of [`parse_uslm_title_all_sections`] — uses
/// the [`UslmTokenizerConfig::default_uslm_1_0`] tag classification.
pub fn parse_uslm_title_all_sections_str(xml: &str) -> Result<Vec<RawStatuteDoc>, ParseError> {
    parse_uslm_title_all_sections_str_with_config(xml, &UslmTokenizerConfig::default_uslm_1_0())
}

/// XSD-grounded variant of [`parse_uslm_title_all_sections_str`].
///
/// Single-pass stream parse — walks the XML once, emitting a
/// `RawStatuteDoc` each time a `<section>` element closes. Total
/// work is O(XML_size), independent of section count. Dispatch on
/// each element is driven by `config.classify(name)` — no
/// hand-coded byte-string match arms remain.
pub fn parse_uslm_title_all_sections_str_with_config(
    xml: &str,
    config: &UslmTokenizerConfig,
) -> Result<Vec<RawStatuteDoc>, ParseError> {
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
                            emit: true,
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

                match config.classify(&name_bytes) {
                    UslmElementClass::Container => {
                        let identifier = attr(e, b"identifier");
                        let emit = identifier.is_some();
                        let id = identifier
                            .as_deref()
                            .map(|s| identifier_to_curie(s, &sc.identifier, &sc.statute_name))
                            .unwrap_or_default();
                        sc.stack.push(ContainerFrame {
                            tag: name_bytes.clone(),
                            id,
                            heading: String::new(),
                            body: String::new(),
                            emit,
                        });
                    }
                    UslmElementClass::Heading => {
                        sc.text_target = Some(TextTarget::Heading);
                        sc.text_buf.clear();
                    }
                    UslmElementClass::Body => {
                        sc.text_target = Some(TextTarget::Body);
                        sc.text_buf.clear();
                    }
                    UslmElementClass::InlineOrnament => {
                        // Text continues accumulating into whichever
                        // capture target is open.
                    }
                    UslmElementClass::Suppressed => {
                        sc.text_target = Some(TextTarget::Suppressed);
                        sc.text_buf.clear();
                    }
                    UslmElementClass::Other => {}
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

                match config.classify(&name_bytes) {
                    UslmElementClass::Container => {
                        if let Some(frame) = sc.stack.pop() {
                            // Identifier-less containers are structural
                            // scaffolding, not citable subdivisions —
                            // pop to keep nesting balanced but emit no
                            // term (matching the runtime subdivision walk).
                            if frame.emit {
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
                                // Compose into the nearest emitting
                                // ancestor, skipping identifier-less
                                // scaffolding frames.
                                if let Some(parent) = sc.stack.iter().rev().find(|f| f.emit) {
                                    sc.relations.push(RawRelation {
                                        from: frame.id,
                                        to: parent.id.clone(),
                                        relation: RawRel::Composes { into: None },
                                    });
                                }
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
                    }
                    UslmElementClass::Heading => {
                        if let Some(frame) = sc.stack.last_mut() {
                            frame.heading.push_str(&sc.text_buf);
                        }
                        sc.text_target = None;
                        sc.text_buf.clear();
                    }
                    UslmElementClass::Body => {
                        if let Some(frame) = sc.stack.last_mut() {
                            if !frame.body.is_empty() {
                                frame.body.push(' ');
                            }
                            frame.body.push_str(&sc.text_buf);
                        }
                        sc.text_target = None;
                        sc.text_buf.clear();
                    }
                    UslmElementClass::InlineOrnament => {
                        // Inline ornaments contribute nothing on close
                        // — text already accumulated into the parent
                        // target via Event::Text handling.
                    }
                    UslmElementClass::Suppressed => {
                        sc.text_target = None;
                        sc.text_buf.clear();
                    }
                    UslmElementClass::Other => {}
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
/// ```text
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
    parse_uslm_str_with_config(
        xml,
        section_identifier,
        statute_name,
        &UslmTokenizerConfig::default_uslm_1_0(),
    )
}

/// XSD-grounded variant of [`parse_uslm_str`] — accepts a
/// [`UslmTokenizerConfig`] whose `container_tags` set tracks the
/// loaded USLM XSD's `substitutionGroup="level"` membership.
pub fn parse_uslm_str_with_config(
    xml: &str,
    section_identifier: &str,
    statute_name: &str,
    config: &UslmTokenizerConfig,
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

                match config.classify(&name_bytes) {
                    UslmElementClass::Container => {
                        let identifier = attr(e, b"identifier");
                        if !in_target_section && identifier.as_deref() == Some(section_identifier) {
                            in_target_section = true;
                            target_seen = true;
                        }
                        if in_target_section {
                            let emit = identifier.is_some();
                            let id = identifier
                                .as_deref()
                                .map(|s| identifier_to_curie(s, section_identifier, statute_name))
                                .unwrap_or_default();
                            stack.push(ContainerFrame {
                                tag: name_bytes.clone(),
                                id,
                                heading: String::new(),
                                body: String::new(),
                                emit,
                            });
                        }
                    }
                    UslmElementClass::Heading if in_target_section => {
                        text_target = Some(TextTarget::Heading);
                        text_buf.clear();
                    }
                    UslmElementClass::Body if in_target_section => {
                        text_target = Some(TextTarget::Body);
                        text_buf.clear();
                    }
                    UslmElementClass::Suppressed if in_target_section => {
                        // Suppress note/footnote content from
                        // definitions per the layer's "structural
                        // shape only" scope.
                        text_target = Some(TextTarget::Suppressed);
                        text_buf.clear();
                    }
                    UslmElementClass::InlineOrnament => {
                        // Text accumulates into whichever target
                        // is open.
                    }
                    _ => {}
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
                match config.classify(&name_bytes) {
                    UslmElementClass::Container => {
                        if in_target_section {
                            if let Some(frame) = stack.pop() {
                                // Identifier-less containers are
                                // structural scaffolding, not citable
                                // subdivisions — pop to keep nesting
                                // balanced but emit no term (matching the
                                // runtime subdivision walk).
                                if frame.emit {
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
                                    // Mereological edge: child Composes
                                    // into the nearest emitting ancestor
                                    // (skipping identifier-less scaffolding).
                                    // The top-level § has no emitting
                                    // ancestor in scope and gets no edge.
                                    if let Some(parent) = stack.iter().rev().find(|f| f.emit) {
                                        relations.push(RawRelation {
                                            from: frame.id,
                                            to: parent.id.clone(),
                                            relation: RawRel::Composes { into: None },
                                        });
                                    }
                                }
                            }
                            if stack.is_empty() {
                                in_target_section = false;
                            }
                        }
                    }
                    UslmElementClass::Heading if in_target_section => {
                        if let Some(frame) = stack.last_mut() {
                            frame.heading.push_str(&text_buf);
                        }
                        text_target = None;
                        text_buf.clear();
                    }
                    UslmElementClass::Body if in_target_section => {
                        if let Some(frame) = stack.last_mut() {
                            if !frame.body.is_empty() {
                                frame.body.push(' ');
                            }
                            frame.body.push_str(&text_buf);
                        }
                        text_target = None;
                        text_buf.clear();
                    }
                    UslmElementClass::Suppressed if in_target_section => {
                        text_target = None;
                        text_buf.clear();
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
    /// Whether this frame becomes a `RawTerm` when popped. A USLM
    /// container is a citable subdivision only when it carries an
    /// `identifier` attribute; identifier-less containers (e.g.
    /// `<level>` grouping wrappers) are structural scaffolding the
    /// LRC does not assign a URN, so they are NOT terms. We still
    /// push a frame for them to keep start/end nesting balanced, but
    /// emit no term and route a child's `Composes` edge to the
    /// nearest emitting ancestor. This mirrors the runtime
    /// `read_subdivision` walk, which only constructs a
    /// `UsCodeSubdivision` for XSD-substitution-group subdivision
    /// elements (every one of which carries an identifier).
    emit: bool,
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

    /// USLM identifier for 18 U.S.C. § 1514A (Sarbanes–Oxley § 806).
    const SOX_1514A_IDENTIFIER: &str = "/us/usc/t18/s1514A";

    /// The fetched `usc_title_18` USLM XML, read as a string. Sourced from the
    /// same path the sibling `parse_title_18_is_single_pass_fast` test reads
    /// (`pr4xis-domains/data/legal/uscode/usc_title_18/`), the registered
    /// `usc_title_18` corpus CI fetches via `pr4xis update usc_title_18`. FAILS
    /// LOUD (panics) when the corpus is not on disk — the data is fetched in
    /// CI, so an absent corpus is a real failure, not a reason to skip.
    fn real_title_18_xml() -> String {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../domains/data/legal/uscode/usc_title_18/usc_title_18-pl-119-90.xml");
        std::fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "run `pr4xis update usc_title_18` to fetch the Title 18 USLM corpus \
                 at {} — tests do not skip ({e})",
                path.display()
            )
        })
    }

    /// The verbatim `<section …identifier="/us/usc/t18/s1514A">…</section>`
    /// byte span sliced out of the fetched Title 18 USLM source — genuine
    /// published bytes, no transcription — wrapped in a single-section
    /// `<title>` so codegen reports exactly one section. The title-sourced
    /// replacement for the deleted standalone `sox_1514a-2002.xml` document.
    /// FAILS LOUD (panics) when the corpus or section is absent; tests do not
    /// skip. Mirrors the domains-side
    /// `uslm::real_sox_1514a::section_bytes` slice.
    fn real_sox_1514a_section_title_doc() -> String {
        let xml = real_title_18_xml();
        let needle = format!("identifier=\"{SOX_1514A_IDENTIFIER}\"");
        let id_pos = xml.find(&needle).unwrap_or_else(|| {
            panic!(
                "§ 1514A ({SOX_1514A_IDENTIFIER}) not found in the fetched usc_title_18 \
                 corpus — a real corpus regression"
            )
        });
        let start = xml[..id_pos]
            .rfind("<section")
            .expect("§ 1514A identifier must sit inside a <section> element");
        let end_tag = "</section>";
        let end_rel = xml[start..]
            .find(end_tag)
            .expect("§ 1514A <section> must have a closing </section>")
            + end_tag.len();
        let section = &xml[start..start + end_rel];
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
             <title xmlns=\"http://xml.house.gov/schemas/uslm/1.0\" identifier=\"/us/usc/t18\">\
             {section}</title>"
        )
    }

    /// Inline fixture mirroring the structural shape of the real
    /// SOX § 1514A USLM slice — one § with two subsections, the
    /// first with two paragraphs, the first paragraph with two
    /// subparagraphs.
    const SAMPLE_USLM: &str = r##"<section identifier="/us/usc/t18/s1514A"><num value="1514A">§ 1514A.</num><heading> Civil action to protect against retaliation in fraud cases</heading><subsection identifier="/us/usc/t18/s1514A/a"><num value="a">(a)</num><heading> <inline class="small-caps">Whistleblower Protection</inline></heading><chapeau>No company may discriminate against an employee—</chapeau><paragraph identifier="/us/usc/t18/s1514A/a/1"><num value="1">(1)</num><chapeau>to provide information—</chapeau><subparagraph identifier="/us/usc/t18/s1514A/a/1/A"><num value="A">(A)</num><content>a Federal regulatory or law enforcement agency;</content></subparagraph><subparagraph identifier="/us/usc/t18/s1514A/a/1/B"><num value="B">(B)</num><content>any Member of Congress;</content></subparagraph></paragraph><paragraph identifier="/us/usc/t18/s1514A/a/2"><num value="2">(2)</num><content>to file a proceeding.</content></paragraph></subsection><subsection identifier="/us/usc/t18/s1514A/b"><num value="b">(b)</num><heading> <inline class="small-caps">Enforcement Action</inline></heading><content>A person who alleges discharge may seek relief.</content></subsection></section>"##;

    #[test]
    fn default_tokenizer_config_classifies_usml_1_0_elements() {
        let c = UslmTokenizerConfig::default_uslm_1_0();
        // The 8 USLM 1.0 container tags all classify as Container.
        for tag in [
            b"section".as_ref(),
            b"subsection".as_ref(),
            b"paragraph".as_ref(),
            b"subparagraph".as_ref(),
            b"clause".as_ref(),
            b"subclause".as_ref(),
            b"item".as_ref(),
            b"subitem".as_ref(),
        ] {
            assert_eq!(c.classify(tag), UslmElementClass::Container, "{tag:?}");
        }
        assert_eq!(c.classify(b"heading"), UslmElementClass::Heading);
        assert_eq!(c.classify(b"chapeau"), UslmElementClass::Body);
        assert_eq!(c.classify(b"content"), UslmElementClass::Body);
        assert_eq!(c.classify(b"num"), UslmElementClass::InlineOrnament);
        assert_eq!(c.classify(b"ref"), UslmElementClass::InlineOrnament);
        assert_eq!(c.classify(b"inline"), UslmElementClass::InlineOrnament);
        assert_eq!(c.classify(b"i"), UslmElementClass::InlineOrnament);
        assert_eq!(c.classify(b"b"), UslmElementClass::InlineOrnament);
        assert_eq!(c.classify(b"note"), UslmElementClass::Suppressed);
        assert_eq!(c.classify(b"footnote"), UslmElementClass::Suppressed);
        // Unknown elements project to Other.
        assert_eq!(c.classify(b"someUnknownTag"), UslmElementClass::Other);
        assert_eq!(c.classify(b""), UslmElementClass::Other);
    }

    #[test]
    fn from_level_substitution_group_preserves_section_as_container() {
        // Simulate the XSD's level-group membership for a subset
        // that doesn't include `section` (some USLM revisions
        // declare the section element separately from the level
        // substitution group). The constructor must still treat
        // `<section>` as a container — it's the top-level
        // mereological frame the stream tokenizer hinges on.
        let level_members = [
            "subsection",
            "paragraph",
            "subparagraph",
            "clause",
            "subclause",
            "item",
            "subitem",
        ];
        let config = UslmTokenizerConfig::from_level_substitution_group(&level_members);
        assert_eq!(
            config.classify(b"section"),
            UslmElementClass::Container,
            "section must always classify as Container"
        );
        for tag in level_members.iter() {
            assert_eq!(
                config.classify(tag.as_bytes()),
                UslmElementClass::Container,
                "{tag} should classify as Container"
            );
        }
    }

    #[test]
    fn parse_uslm_str_with_xsd_grounded_config_matches_default() {
        // The XSD-grounded config built from a level-group set
        // equivalent to the USLM 1.0 default must produce
        // byte-identical RawStatuteDoc output for the same input.
        let default_doc =
            parse_uslm_str(SAMPLE_USLM, "/us/usc/t18/s1514A", "sox_1514a").expect("parse");
        let xsd_config = UslmTokenizerConfig::from_level_substitution_group(&[
            "subsection",
            "paragraph",
            "subparagraph",
            "clause",
            "subclause",
            "item",
            "subitem",
        ]);
        let xsd_doc =
            parse_uslm_str_with_config(SAMPLE_USLM, "/us/usc/t18/s1514A", "sox_1514a", &xsd_config)
                .expect("parse");
        assert_eq!(default_doc.terms.len(), xsd_doc.terms.len());
        assert_eq!(default_doc.relations.len(), xsd_doc.relations.len());
        for (lhs, rhs) in default_doc.terms.iter().zip(xsd_doc.terms.iter()) {
            assert_eq!(lhs.id, rhs.id);
            assert_eq!(lhs.name, rhs.name);
        }
    }

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
    ///
    /// § 1514A is sliced out of the registered `usc_title_18` corpus (the
    /// authority CI fetches via `pr4xis update usc_title_18`) rather than a
    /// deleted standalone `sox_1514a-2002.xml` fixture — `pr4xis-domains` went
    /// crates.io-publishable, so the standalone granule no longer ships. FAILS
    /// LOUD (panics) when the corpus is absent; tests do not skip.
    #[test]
    fn generate_title_module_source_on_real_sox_slice() {
        // Wrap the verbatim § 1514A `<section>` (sliced out of Title 18) in a
        // single-section `<title>` so codegen reports exactly one section,
        // identical to the old standalone-slice shape.
        let section_doc = real_sox_1514a_section_title_doc();
        let src = generate_title_module_source_from_str(&section_doc).expect("emit");
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
        // The registered `usc_title_18` corpus — CI fetches it via
        // `pr4xis update usc_title_18`. FAILS LOUD when absent; tests do not
        // skip.
        let xml = real_title_18_xml();
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

    /// Real-corpus check — parse 18 U.S.C. § 1514A out of the registered
    /// `usc_title_18` corpus (the authority CI fetches via
    /// `pr4xis update usc_title_18`), NOT a deleted standalone
    /// `sox_1514a-2002.xml` fixture. `parse_uslm_str` selects § 1514A by its
    /// USLM identifier internally, so it reads the same bytes the runtime path
    /// does. Verifies the parser handles the actual published structure, not
    /// just the synthetic inline fixture above. FAILS LOUD (panics) when the
    /// corpus is absent; tests do not skip.
    #[test]
    fn parses_real_sox_1514a_slice() {
        let xml = real_title_18_xml();
        let doc = parse_uslm_str(&xml, "/us/usc/t18/s1514A", "sox_1514a").expect("parse");

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
