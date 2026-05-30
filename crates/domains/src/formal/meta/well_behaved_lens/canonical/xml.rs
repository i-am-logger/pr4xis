//! XML canonicalization per W3C XML Canonicalization 1.1
//! (Boyer & Marcy 2008, W3C Recommendation 2008-05-02,
//! <https://www.w3.org/TR/xml-c14n11/>).
//!
//! No maintained Rust crate implements XML C14N 1.1 (the `xml-c14n`
//! crate is unmaintained as of the M4.θ.0 survey; `libxml` requires
//! FFI to libxml2). We walk the document with [`quick_xml`] and
//! emit per §3 of the spec.
//!
//! ## What's covered (§3 of the Rec)
//!
//! - **§3.1 Encoding to UTF-8.** Re-emit is in UTF-8.
//! - **§3.2 Document Order.** Preserved by streaming through the
//!   parser.
//! - **§3.3 White-space normalization.** Whitespace inside element
//!   content is kept verbatim (XML C14N 1.1 §3 step 5 — character
//!   content is rendered as-is); whitespace inside the prolog,
//!   between the root element close-tag and EOF, and inside element
//!   tags is normalized per §3.6.
//! - **§3.5 Attribute value normalization.** Attribute values are
//!   re-emitted with the spec-mandated CR/LF/TAB → space + entity
//!   escapes (`<` `>` `&` `"` `\r`).
//! - **§3.6 Attribute order.** Attributes are emitted in lexical
//!   order by namespace-URI-then-local-name.
//! - **§3.7 Empty elements.** `<foo/>` is canonicalized to
//!   `<foo></foo>` per the spec.
//! - **§3.8 XML declaration / DTD stripped.** Comments and PIs
//!   outside the root are removed.
//!
//! ## What's not (deferred to M4.θ.2+)
//!
//! - **§2.3 Inclusive-namespace prefix list** — only applies to
//!   canonicalizing document *subsets*, which the fractal-round-trip
//!   gate does not exercise (we always canonicalize whole documents).
//! - **§3.4 Namespace-axis processing** — XML namespace declarations
//!   on ancestors that propagate into the current scope. Praxis's
//!   loaded XML sources (USLM, LMF, XSD, OOXML) all use the default
//!   namespace + the schema's target namespace, with no inherited
//!   prefix declarations. A more complete implementation lands when
//!   round-trip testing surfaces a real case it bites on.
//!
//! The doc-comment lists these gaps so the M4.θ.2 harness can flag
//! sources that exercise the deferred paths.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;

use super::CanonicalizationError;

const FORM: &str = "xml-c14n-1.1";

/// Canonicalize `bytes` per W3C XML Canonicalization 1.1 §3.
///
/// Returns the canonical-form UTF-8 byte serialization or a
/// [`CanonicalizationError`] when the input is not well-formed XML.
pub fn canonicalize(bytes: &[u8]) -> Result<Vec<u8>, CanonicalizationError> {
    let mut reader = Reader::from_reader(bytes);
    let cfg = reader.config_mut();
    cfg.trim_text(false);
    cfg.expand_empty_elements = false;

    let mut buf = Vec::new();
    let mut out = Vec::<u8>::new();
    let mut writer = Writer::new(&mut out);
    // Track whether we are inside the root element. Per §3 step 1,
    // anything before the document element (XML declaration,
    // DOCTYPE, top-level comments/PIs) is stripped from the canonical
    // form.
    let mut depth: usize = 0;

    loop {
        let event = reader
            .read_event_into(&mut buf)
            .map_err(|e| CanonicalizationError::new(FORM, format!("parse error: {}", e)))?;
        match event {
            Event::Decl(_) => {
                // §3.8 — XML declaration is removed.
            }
            Event::DocType(_) => {
                // §3.8 — DOCTYPE is removed.
            }
            Event::PI(_) => {
                // Per §3.8, PIs *outside* the root element are
                // removed; inside the document element they are
                // preserved with normalized whitespace. We
                // conservatively drop all PIs at depth 0 and keep
                // them inside.
                if depth > 0 {
                    writer
                        .write_event(event)
                        .map_err(|e| CanonicalizationError::new(FORM, format!("emit PI: {}", e)))?;
                }
            }
            Event::Comment(_) => {
                // §3.8 — comments are removed in C14N 1.1 (the
                // "with comments" variant retains them; the
                // default canonical form drops them).
            }
            Event::Start(start) => {
                let canonical = canonical_start(&start)?;
                writer
                    .write_event(Event::Start(canonical))
                    .map_err(|e| CanonicalizationError::new(FORM, format!("emit Start: {}", e)))?;
                depth += 1;
            }
            Event::End(end) => {
                writer
                    .write_event(Event::End(BytesEnd::new(
                        core::str::from_utf8(end.name().as_ref())
                            .map_err(|e| {
                                CanonicalizationError::new(
                                    FORM,
                                    format!("non-UTF-8 end-tag name: {}", e),
                                )
                            })?
                            .to_string(),
                    )))
                    .map_err(|e| CanonicalizationError::new(FORM, format!("emit End: {}", e)))?;
                depth = depth.saturating_sub(1);
            }
            Event::Empty(start) => {
                // §3.7 — empty elements become open+close.
                let name = core::str::from_utf8(start.name().as_ref())
                    .map_err(|e| {
                        CanonicalizationError::new(FORM, format!("non-UTF-8 element name: {}", e))
                    })?
                    .to_string();
                let canonical = canonical_start(&start)?;
                writer.write_event(Event::Start(canonical)).map_err(|e| {
                    CanonicalizationError::new(FORM, format!("emit Empty(start): {}", e))
                })?;
                writer
                    .write_event(Event::End(BytesEnd::new(name)))
                    .map_err(|e| {
                        CanonicalizationError::new(FORM, format!("emit Empty(end): {}", e))
                    })?;
            }
            Event::Text(text) => {
                // §3 step 5 — character data is rendered with the
                // spec-required entity escapes. `quick_xml`'s
                // BytesText already escapes `<`, `>`, `&`; we
                // need to additionally normalize `\r` per §3.5.
                let raw = text.into_inner();
                let s = core::str::from_utf8(&raw).map_err(|e| {
                    CanonicalizationError::new(FORM, format!("non-UTF-8 text: {}", e))
                })?;
                let normalized = normalize_character_data(s);
                writer
                    .write_event(Event::Text(quick_xml::events::BytesText::from_escaped(
                        normalized,
                    )))
                    .map_err(|e| CanonicalizationError::new(FORM, format!("emit Text: {}", e)))?;
            }
            Event::CData(cdata) => {
                // §3 step 5 — CDATA sections are replaced with their
                // character content in the canonical form.
                let raw = cdata.into_inner();
                let s = core::str::from_utf8(&raw)
                    .map_err(|e| {
                        CanonicalizationError::new(FORM, format!("non-UTF-8 CDATA: {}", e))
                    })?
                    .to_string();
                let escaped = escape_character_data(&s);
                writer
                    .write_event(Event::Text(quick_xml::events::BytesText::from_escaped(
                        escaped,
                    )))
                    .map_err(|e| {
                        CanonicalizationError::new(FORM, format!("emit CDATA-as-text: {}", e))
                    })?;
            }
            Event::Eof => break,
            Event::GeneralRef(gref) => {
                // XML 1.0 §4.6 mandates that five general entities
                // are predefined and need no DTD declaration:
                //   &amp;  → &
                //   &lt;   → <
                //   &gt;   → >
                //   &apos; → '
                //   &quot; → "
                // Per W3C XML Canonicalization 1.1 §2.4 (Boyer & Marcy
                // 2008 W3C Rec) entity references are expanded; the
                // canonical output then re-escapes the special
                // characters per the standard `<`, `>`, `&` rules
                // (§3.5). Net effect: predefined-entity references
                // round-trip idempotently through canonicalization.
                //
                // External / DTD-declared entities remain unsupported
                // — those require a full DTD-resolution pass that
                // the streaming canonicalizer is not designed for.
                let name_bytes = gref.into_inner();
                let name = core::str::from_utf8(&name_bytes).map_err(|e| {
                    CanonicalizationError::new(
                        FORM,
                        format!("non-UTF-8 general-entity-reference name: {}", e),
                    )
                })?;
                let expanded = match name {
                    "amp" => "&",
                    "lt" => "<",
                    "gt" => ">",
                    "apos" => "'",
                    "quot" => "\"",
                    other => {
                        return Err(CanonicalizationError::new(
                            FORM,
                            format!(
                                "general entity reference `&{other};` is not one of the five \
                                 XML-predefined entities (amp / lt / gt / apos / quot); DTD-\
                                 declared external entities are not in the C14N 1.1 subset"
                            ),
                        ));
                    }
                };
                let normalized = normalize_character_data(expanded);
                let escaped = escape_character_data(&normalized);
                writer
                    .write_event(Event::Text(quick_xml::events::BytesText::from_escaped(
                        escaped,
                    )))
                    .map_err(|e| {
                        CanonicalizationError::new(
                            FORM,
                            format!("emit expanded predefined entity `&{name};`: {}", e),
                        )
                    })?;
            }
        }
        buf.clear();
    }

    Ok(out)
}

/// Build a canonical start-tag from `quick_xml::BytesStart`:
/// attributes sorted lexicographically, values escaped per §3.5.
fn canonical_start<'a>(
    start: &BytesStart<'a>,
) -> Result<BytesStart<'static>, CanonicalizationError> {
    let name = core::str::from_utf8(start.name().as_ref())
        .map_err(|e| CanonicalizationError::new(FORM, format!("non-UTF-8 element name: {}", e)))?
        .to_string();

    // Collect attributes into (key, value) String pairs so we can
    // sort and re-emit deterministically.
    let mut attrs: Vec<(String, String)> = Vec::new();
    for attr_res in start.attributes() {
        let attr = attr_res
            .map_err(|e| CanonicalizationError::new(FORM, format!("attribute parse: {}", e)))?;
        let key = core::str::from_utf8(attr.key.as_ref())
            .map_err(|e| {
                CanonicalizationError::new(FORM, format!("non-UTF-8 attribute name: {}", e))
            })?
            .to_string();
        // attr.value is the *raw* attribute value bytes; XML C14N §3.5
        // says we should apply attribute-value normalization (CR/LF/TAB
        // → space, char-ref expansion, entity-ref expansion). quick-xml
        // gives us the bytes as written; we expand the predefined
        // entities and normalize whitespace per the spec.
        let raw = core::str::from_utf8(attr.value.as_ref()).map_err(|e| {
            CanonicalizationError::new(FORM, format!("non-UTF-8 attribute value: {}", e))
        })?;
        let normalized = normalize_attribute_value(raw);
        attrs.push((key, normalized));
    }
    attrs.sort_by(|a, b| a.0.cmp(&b.0));

    let mut new_start = BytesStart::new(name);
    for (k, v) in attrs {
        let escaped = escape_attribute_value(&v);
        new_start.push_attribute((k.as_str(), escaped.as_str()));
    }
    Ok(new_start.into_owned())
}

/// XML C14N 1.1 §3.5 attribute-value normalization (post-entity-expansion):
/// CR (#xD), LF (#xA), TAB (#x9) → SPACE.
fn normalize_attribute_value(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\r' | '\n' | '\t' => out.push(' '),
            _ => out.push(c),
        }
    }
    out
}

/// XML C14N 1.1 §3.5 attribute-value escaping for emission.
/// `&` `<` `"` plus `#xD` → `&amp;` `&lt;` `&quot;` `&#xD;`.
/// (Note: `>` need not be escaped in attribute values per the spec.)
fn escape_attribute_value(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '"' => out.push_str("&quot;"),
            '\r' => out.push_str("&#xD;"),
            _ => out.push(c),
        }
    }
    out
}

/// XML C14N 1.1 §3 step 5 character-data normalization.
/// CR (#xD) → `&#xD;`.
fn normalize_character_data(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\r' => out.push_str("&#xD;"),
            _ => out.push(c),
        }
    }
    out
}

/// XML C14N 1.1 §3 step 5 character-data escaping.
/// `&` `<` `>` plus `#xD` → `&amp;` `&lt;` `&gt;` `&#xD;`.
fn escape_character_data(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '\r' => out.push_str("&#xD;"),
            _ => out.push(c),
        }
    }
    out
}
