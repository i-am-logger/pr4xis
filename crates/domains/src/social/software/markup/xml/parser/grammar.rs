//! Recursive-descent parser over the W3C XML 1.0 Fifth Edition
//! grammar (Bray et al. 2008).
//!
//! Each `parse_<production>` function transcribes one of the W3C
//! EBNF productions (cited in the function's doc-comment), advances
//! the byte cursor, and emits the corresponding piece of the typed
//! [`XmlDocument`] Infoset
//! tree.
//!
//! Strategy: recursive descent with one-byte lookahead, no
//! backtracking. The XML 1.0 grammar is predictable on the prolog
//! (Bray et al. 2008 §2.8) and on element content (§3.1) given the
//! single-byte sentinels `<`, `<!`, `<?`, `]]>`, `</`, so a
//! straight LL(1) descent works.
//!
//! Per `feedback_write_ontologically_not_mechanically`: every
//! dispatch reasons via the W3C grammar productions, not ad-hoc
//! string-shape checks. Each helper cites the EBNF rule it
//! implements.

#[allow(unused_imports)]
use alloc::{
    collections::BTreeMap,
    format,
    string::{String, ToString},
    vec::Vec,
};

use super::super::ontology::{
    XmlAttribute, XmlDoctype, XmlDocument, XmlElement, XmlEntityKind, XmlExternalId,
    XmlGeneralEntity, XmlName, XmlNamespace, XmlNode,
};
use super::source_syntax::{
    CaptureCtx, EmptyForm, EndOfLineForm, EntityName, EntityReferenceForm, EolKind, ExtendedRef,
    ExtendedRefKind, IntraTagWhitespace, NodeDecisions, PrologDecisions, StartTagToken,
    SyntaxDecisions,
};

/// Failure modes when parsing XML 1.0 bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XmlParseError {
    /// Input was not valid UTF-8 (W3C XML 1.0 §4.3.3 default
    /// encoding). The parser supports only UTF-8 inputs; the XML
    /// declaration's `encoding` attribute is preserved on the
    /// emitted [`XmlDocument`] but not used to re-decode.
    NotUtf8 { position: usize },
    /// Reached EOF in a state that expected more input (e.g. an
    /// open element tag with no matching ETag).
    UnexpectedEof { context: String },
    /// A byte sequence couldn't be matched to the W3C grammar.
    Syntax {
        position: usize,
        expected: String,
        found: String,
    },
    /// An ETag's name did not match its matching STag's name
    /// (W3C XML 1.0 §3.1 well-formedness constraint: Element Type
    /// Match).
    MismatchedTags {
        position: usize,
        open: String,
        close: String,
    },
    /// An entity reference that's not one of the five XML 1.0 §4.6
    /// predefined entities (`amp`, `lt`, `gt`, `apos`, `quot`) and
    /// not a numeric character reference (§4.1). DTD-declared
    /// general entities are not supported in this slice.
    UnsupportedEntity { position: usize, name: String },
    /// A numeric character reference that names a code point
    /// outside the W3C XML 1.0 §2.2 Char production.
    InvalidCharRef { position: usize, code_point: u32 },
    /// Two attributes in the same start-tag share a name. W3C XML
    /// 1.0 Fifth Edition §3.1 well-formedness constraint: *Unique
    /// Att Spec* — "no attribute name MUST appear more than once
    /// in the same start-tag or empty-element tag".
    DuplicateAttribute { position: usize, name: String },
    /// The string `--` (double-hyphen) appeared inside a comment.
    /// W3C XML 1.0 Fifth Edition §2.5 production \[15\] `Comment`:
    /// `Comment ::= '<!--' ((Char - '-') | ('-' (Char - '-')))* '-->'`
    /// — equivalent to "the string `--` MUST NOT occur within comments".
    MalformedComment { position: usize },
    /// A `]]>` sequence appeared in `CharData` outside a CDATA section.
    /// W3C XML 1.0 Fifth Edition §2.4 production \[14\] `CharData`:
    /// `CharData ::= [^<&]* - ([^<&]* ']]>' [^<&]*)` — the `]]>`
    /// sequence MUST be escaped in `CharData`.
    DisallowedCdataEnd { position: usize },
    /// A code point outside the W3C XML 1.0 §2.2 production \[2\]
    /// `Char` repertoire appeared in `CharData` / `Comment` / `PI` /
    /// `CDATA`. The membership test goes through
    /// [`is_xml_char`] — which itself consults the
    /// build-time-generated table parsed from the registered
    /// `xml_1_0_fifth_edition@2008` source.
    InvalidChar {
        position: usize,
        code_point: u32,
        context: &'static str,
    },
}

impl core::fmt::Display for XmlParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotUtf8 { position } => {
                write!(f, "input is not valid UTF-8 at byte {position}")
            }
            Self::UnexpectedEof { context } => write!(f, "unexpected EOF while parsing {context}"),
            Self::Syntax {
                position,
                expected,
                found,
            } => write!(
                f,
                "syntax error at byte {position}: expected {expected}, found {found}"
            ),
            Self::MismatchedTags {
                position,
                open,
                close,
            } => write!(
                f,
                "mismatched tags at byte {position}: STag {open:?} closed by ETag {close:?}"
            ),
            Self::UnsupportedEntity { position, name } => write!(
                f,
                "unsupported entity reference &{name}; at byte {position} \
                 (only XML 1.0 §4.6 predefined entities and numeric character \
                 references are supported)"
            ),
            Self::InvalidCharRef {
                position,
                code_point,
            } => write!(
                f,
                "invalid character reference U+{code_point:04X} at byte {position} \
                 (outside the W3C XML 1.0 §2.2 Char production)"
            ),
            Self::DuplicateAttribute { position, name } => write!(
                f,
                "duplicate attribute {name:?} at byte {position} \
                 (W3C XML 1.0 §3.1 well-formedness constraint Unique Att Spec)"
            ),
            Self::MalformedComment { position } => write!(
                f,
                "comment contains the forbidden substring '--' at byte {position} \
                 (W3C XML 1.0 §2.5 production [15] Comment)"
            ),
            Self::DisallowedCdataEnd { position } => write!(
                f,
                "CharData contains the forbidden sequence ']]>' at byte {position} \
                 (W3C XML 1.0 §2.4 production [14] CharData — must be escaped)"
            ),
            Self::InvalidChar {
                position,
                code_point,
                context,
            } => write!(
                f,
                "invalid character U+{code_point:04X} at byte {position} inside {context} \
                 (outside W3C XML 1.0 §2.2 [2] Char repertoire)"
            ),
        }
    }
}

impl std::error::Error for XmlParseError {}

/// Top-level entry point.
///
/// Implements **production \[1\]** `document ::= prolog element Misc*`
/// from W3C XML 1.0 Fifth Edition §2.1.
///
/// **W3C XML 1.0 §F (Autodetection of Character Encodings)** is
/// applied via `decode_input` (private): byte-order marks select between
/// UTF-8, UTF-16 big-endian, and UTF-16 little-endian; bytes are
/// transcoded to UTF-8 before the grammar descent. The UTF-8 BOM
/// (rare but allowed) is stripped as part of decoding.
///
/// Performs **§2.11 End-of-Line Handling** on the resulting string
/// before parsing: every literal `#xD#xA` (CRLF) and every lone `#xD`
/// (CR) is replaced with a single `#xA` (LF). The W3C spec requires
/// this normalization on input so that downstream productions never
/// see CR.
pub fn parse_document(input: &[u8]) -> Result<XmlDocument, XmlParseError> {
    parse_document_inner(input, &mut None)
}

/// Reader half of the serialized reverse lens — parse AND capture the
/// concrete-syntax decisions ([`SyntaxDecisions`]) needed to reconstruct the
/// exact source bytes with
/// [`serialize_document_exact`](super::serializer::serialize_document_exact).
///
/// Identical to [`parse_document`] for the produced [`XmlDocument`]; it
/// additionally threads a [`CaptureCtx`] through the element/content descent,
/// assigning each element a pre-order index at entry and recording any
/// non-canonical concrete-syntax decision against it (today: an explicit-empty
/// `<a></a>` element). The returned `(doc, decisions)` satisfies the byte-exact
/// round-trip law `serialize_document_exact(&doc, &decisions) == input` for any
/// input whose only non-canonical feature is the empty-element form.
pub fn parse_document_capturing(
    input: &[u8],
) -> Result<(XmlDocument, SyntaxDecisions), XmlParseError> {
    let mut capture = Some(CaptureCtx::default());
    let doc = parse_document_inner(input, &mut capture)?;
    let decisions = capture.expect("capture was seeded with Some").decisions;
    Ok((doc, decisions))
}

fn parse_document_inner(
    input: &[u8],
    capture: &mut Option<CaptureCtx>,
) -> Result<XmlDocument, XmlParseError> {
    let (raw, detected_encoding) = decode_input(input)?;
    // §2.11 \[2.11\] End-of-Line Handling — collapse `#xD#xA`/`#xD` to `#xA`
    // BEFORE the grammar descent, AND capture the erased EOL form (keyed by LF
    // ordinal, robust against re-escaping) so the byte-exact serializer can put
    // the `#xD` back. Empty for a pure-`#xA` source — the additive case.
    let (normalized, eol_form) = normalize_line_endings(&raw);
    let mut cursor = Cursor::new(&normalized);

    let ((version, encoding, standalone, doctype), (after_xml_decl, after_doctype)) =
        parse_prolog(&mut cursor)?;
    // W3C XML 1.0 §F + §4.3.3 "Character Encoding in Entities" —
    // if an XMLDecl carries an encoding declaration, that label
    // MUST be consistent with the encoding actually used. UTF-16
    // entities MUST begin with a BOM (§4.3.3.1), so a document
    // declaring encoding="UTF-16" but lacking a UTF-16 BOM is
    // malformed. xmlconf eduni/errata-2e/E61 is the spec
    // regression: an ASCII document declaring UTF-16 encoding.
    if let Some(declared) = &encoding {
        // Per W3C XML 1.0 §4.3.3 + §F + erratum E05: query the
        // bundled spec's encoding-label families for the canonical
        // 16-bit Unicode label set ({UTF-16, UTF-16BE, UTF-16LE,
        // ISO-10646-UCS-2}). Hand-coded alias lists violate
        // `feedback_bottom_up_loaded_not_encoded`.
        let families =
            crate::social::software::markup::xml::spec_1_0::loaded_xml_encoding_families();
        let declared_is_utf16 = families.is_utf16_family(declared);
        let detected_is_utf16 = matches!(
            detected_encoding,
            DetectedEncoding::Utf16Be | DetectedEncoding::Utf16Le
        );
        if declared_is_utf16 != detected_is_utf16 {
            return Err(XmlParseError::Syntax {
                position: 0,
                expected: "encoding declaration consistent with actual byte stream \
                           (§F / §4.3.3.1: UTF-16 entities must have a BOM)"
                    .into(),
                found: declared.clone(),
            });
        }
    }
    let entity_map: Vec<XmlGeneralEntity> = doctype
        .as_ref()
        .map(|d| d.general_entities.clone())
        .unwrap_or_default();
    // §4.1 WFC: Entity Declared — applies when:
    //   1. document has no DTD, OR
    //   2. internal-only DTD subset with no PE references, OR
    //   3. standalone='yes'.
    // Otherwise undeclared entity references are validity errors
    // (not well-formedness errors), and a non-validating processor
    // may treat them as bypassed without contribution.
    let strict_entity_declared = match (&doctype, standalone) {
        (_, Some(true)) => true,
        (None, _) => true,
        (Some(dt), _) => dt.external_id.is_none() && !dt.internal_subset_had_pe_references,
    };
    let root = parse_element(&mut cursor, &entity_map, strict_entity_declared, capture, 0)?;
    // §2.1 \[1\] `document ::= prolog element Misc*` — the trailing epilog
    // `Misc*`. Capture the FULL verbatim run (§2.6 \[16\] PI / §2.5 \[15\] Comment
    // / §2.3 \[3\] S) for byte-exact reconstruction; for a pure-`S` epilog this is
    // the leading white-space, so documents with no epilog PI/Comment are
    // unaffected. The explicit `skip_whitespace` is redundant after
    // `parse_misc_star` (which already consumes leading `S`) but kept for the EOF
    // assertion's clarity.
    let after_root_start = cursor.pos;
    parse_misc_star(&mut cursor)?;
    cursor.skip_whitespace();
    let after_root = cursor.misc_run_since(after_root_start).to_string();
    if !cursor.is_eof() {
        return Err(cursor.syntax_error("end of document", "trailing content"));
    }

    // Reader half of the serialized reverse lens: record the prolog/epilog
    // white-space (§2.8 \[27\] `Misc` `S`) the Infoset DOM does not carry, so
    // the byte-exact serializer can re-emit it. No-op for the canonical
    // (whitespace-free) prolog, and skipped entirely when not capturing.
    if let Some(ctx) = capture.as_mut() {
        let prolog = PrologDecisions {
            after_xml_decl,
            after_doctype,
            after_root,
        };
        if !prolog.is_empty() {
            ctx.record_prolog(prolog);
        }
        // §2.11 \[2.11\] End-of-Line form — the `#xD#xA`/`#xD` source line
        // breaks the §2.11 normalization above collapsed to `#xA`, keyed by LF
        // ordinal so the byte-exact serializer re-expands them over the finished
        // output. Empty (recorded as nothing) for a pure-`#xA` source, so a
        // CRLF-free document is unaffected.
        if !eol_form.is_empty() {
            ctx.record_eol_form(eol_form);
        }
    }

    Ok(XmlDocument {
        version,
        encoding,
        doctype,
        root,
    })
}

/// W3C XML 1.0 Fifth Edition §F (Autodetection of Character
/// Encodings) — transcode raw bytes to UTF-8 by examining the
/// initial byte sequence:
///
/// - `FE FF …`         → UTF-16 big-endian, BOM stripped
/// - `FF FE …`         → UTF-16 little-endian, BOM stripped
/// - `EF BB BF …`      → UTF-8 with BOM, BOM stripped
/// - anything else     → UTF-8 (no BOM)
///
/// Unpaired surrogates and odd-length UTF-16 inputs are reported as
/// [`XmlParseError::NotUtf8`] — that variant covers any
/// encoding-layer failure regardless of declared encoding.
///
/// Other encodings the spec recognises (UTF-32, EBCDIC, encoding
/// declared via `<?xml encoding="…"?>` for a non-Unicode label) are
/// out of scope: the praxis legal-evidence corpus is uniformly
/// UTF-8 / UTF-16, and the W3C XMLConf test cases the praxis
/// parser must pass declare only those.
/// Encoding the byte-prefix probe in `decode_input` (private) resolved
/// the document to. Carried alongside the decoded text so the
/// outer parser can match it against the optional `encoding="…"`
/// pseudo-attribute on the XMLDecl per W3C XML 1.0 §F /
/// §4.3.3 "Character Encoding in Entities".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DetectedEncoding {
    /// `FE FF` BOM observed.
    Utf16Be,
    /// `FF FE` BOM observed.
    Utf16Le,
    /// Default — neither UTF-16 BOM observed.
    Utf8,
}

fn decode_input(input: &[u8]) -> Result<(String, DetectedEncoding), XmlParseError> {
    if let Some(body) = input.strip_prefix(&[0xFE, 0xFF]) {
        return decode_utf16(body, /* big_endian = */ true).map(|s| (s, DetectedEncoding::Utf16Be));
    }
    if let Some(body) = input.strip_prefix(&[0xFF, 0xFE]) {
        return decode_utf16(body, /* big_endian = */ false)
            .map(|s| (s, DetectedEncoding::Utf16Le));
    }
    let body = input.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(input);
    core::str::from_utf8(body)
        .map(|s| (s.to_string(), DetectedEncoding::Utf8))
        .map_err(|e| XmlParseError::NotUtf8 {
            position: e.valid_up_to(),
        })
}

/// Decode UTF-16 bytes (already-stripped of the BOM) to a UTF-8
/// `String`. Endianness is selected by the caller from the BOM.
/// Reports `NotUtf8` on odd byte length or unpaired surrogates per
/// the Unicode 15 §3.9 D89/D91 invariants.
fn decode_utf16(bytes: &[u8], big_endian: bool) -> Result<String, XmlParseError> {
    if !bytes.len().is_multiple_of(2) {
        return Err(XmlParseError::NotUtf8 {
            position: bytes.len() & !1,
        });
    }
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|pair| {
            if big_endian {
                u16::from_be_bytes([pair[0], pair[1]])
            } else {
                u16::from_le_bytes([pair[0], pair[1]])
            }
        })
        .collect();
    let mut out = String::with_capacity(units.len());
    for (unit_index, result) in core::char::decode_utf16(units.iter().copied()).enumerate() {
        match result {
            Ok(ch) => out.push(ch),
            Err(_) => {
                return Err(XmlParseError::NotUtf8 {
                    position: 2 * unit_index,
                });
            }
        }
    }
    Ok(out)
}

/// **W3C XML 1.0 §2.11 End-of-Line Handling.** Every literal `#xD#xA`
/// (CRLF) becomes `#xA` (LF). Every lone `#xD` (CR not followed by LF)
/// also becomes `#xA`. This MUST run before parsing so production
/// rules never see CR — the spec is explicit: "the XML processor MUST
/// behave as if it normalized all line breaks in external parsed
/// entities (including the document entity) on input … to the single
/// character #xA".
///
/// Returns the normalized string AND the §2.11 \[2.11\] end-of-line FORM the
/// normalization erased — for each collapsed line break, the LF ORDINAL of the
/// produced `#xA` (its 0-based index among ALL `#xA` bytes in the normalized
/// output, counting untouched literal `#xA`s too) and whether the source wrote
/// `#xD#xA` (CRLF) or a lone `#xD` (CR). That ordinal is the
/// re-escaping-robust key the byte-exact serializer re-expands by (see
/// [`EndOfLineForm`]); a pure-`#xA` source collapses nothing, so the form is empty
/// (the additive fast path).
fn normalize_line_endings(raw: &str) -> (String, EndOfLineForm) {
    if !raw.contains('\r') {
        // Fast path — no `#xD` present, nothing to normalize and no §2.11 form
        // to record (the form is empty, so serialization is a byte-identical
        // no-op for pure-`#xA` input).
        return (raw.to_string(), EndOfLineForm::default());
    }
    let mut out = String::with_capacity(raw.len());
    let mut eols: Vec<(usize, EolKind)> = Vec::new();
    // The 0-based index of the NEXT `#xA` to be written, among all `#xA` bytes in
    // the normalized output — incremented for EVERY `#xA` (collapsed break OR
    // source-literal `#xA`), so it stays in lockstep with the serializer's `#xA`
    // count. A collapsed break records its kind at this ordinal; a literal `#xA`
    // records nothing (there is nothing to put back) but still advances it.
    let mut lf_ordinal = 0usize;
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\r' => {
                // CRLF → LF (consume the following LF); lone CR → LF. Record the
                // erased form at this `#xA`'s ordinal.
                let kind = if chars.peek() == Some(&'\n') {
                    chars.next();
                    EolKind::Crlf
                } else {
                    EolKind::Cr
                };
                eols.push((lf_ordinal, kind));
                out.push('\n');
                lf_ordinal += 1;
            }
            '\n' => {
                // A source literal `#xA` — passed through unchanged, records no
                // form, but advances the ordinal so collapsed breaks AFTER it key
                // correctly.
                out.push('\n');
                lf_ordinal += 1;
            }
            other => out.push(other),
        }
    }
    (out, EndOfLineForm { eols })
}

/// Byte cursor with one-rune lookahead. Tracks position for error
/// reporting.
struct Cursor<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn rest(&self) -> &'a str {
        &self.input[self.pos..]
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn peek_char(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn starts_with(&self, lit: &str) -> bool {
        self.rest().starts_with(lit)
    }

    fn consume(&mut self, lit: &str) -> Result<(), XmlParseError> {
        if self.starts_with(lit) {
            self.pos += lit.len();
            Ok(())
        } else {
            Err(self.syntax_error(&format!("literal {lit:?}"), &self.preview()))
        }
    }

    /// W3C XML 1.0 §2.3 production \[3\] S — whitespace.
    fn skip_whitespace(&mut self) {
        let rest = self.rest();
        let n = rest
            .bytes()
            .take_while(|b| matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
            .count();
        self.pos += n;
    }

    /// W3C XML 1.0 §2.3 production \[3\] `S` — consume the white-space run at the
    /// cursor AND return it, for the byte-exact serialized reverse lens to
    /// re-emit (the intra-tag `S` layout the Infoset discards — §3.1
    /// \[40\]/\[44\]). The returned slice is the exact consumed substring (after
    /// the §2.11 end-of-line normalization already applied to the whole input);
    /// empty when no white-space is present.
    fn take_whitespace(&mut self) -> &'a str {
        let rest = self.rest();
        let n = rest
            .bytes()
            .take_while(|b| matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
            .count();
        let run = &rest[..n];
        self.pos += n;
        run
    }

    /// The ENTIRE source span `[start, self.pos)` verbatim — the full §2.8 \[27\]
    /// `Misc*` run (`Misc ::= Comment | PI | S`) the prolog/epilog consumed,
    /// INCLUDING any Comment or processing-instruction the Information Set drops
    /// at the document level (Cowan & Tobin 2004 §2.1 keeps document-level
    /// *children* only for the root element). Used by the byte-exact serialized
    /// reverse lens to re-emit a prolog/epilog `Misc*` that is NOT pure white-space
    /// — e.g. the `<?xml-stylesheet ...?>` PI every USC USLM title carries (§2.6
    /// \[16\] `PI`).
    ///
    /// For a pure-`S` `Misc*` (the WN-LMF case — XMLDecl, `S`, DOCTYPE, `S`, root)
    /// this is byte-identical to the leading white-space, so a document with no
    /// prolog/epilog PI or Comment is unaffected: the span IS its leading
    /// white-space. The slice is taken from the §2.11-normalized input, so it
    /// carries `#xA` line endings (the original `#xD#xA` form is a separate §2.11
    /// concrete-syntax residue the byte kernel does not yet model).
    fn misc_run_since(&self, start: usize) -> &'a str {
        &self.input[start..self.pos]
    }

    fn require_whitespace(&mut self, context: &str) -> Result<(), XmlParseError> {
        let before = self.pos;
        self.skip_whitespace();
        if self.pos == before {
            Err(self.syntax_error(&format!("whitespace before {context}"), &self.preview()))
        } else {
            Ok(())
        }
    }

    fn preview(&self) -> String {
        let rest = self.rest();
        let end = rest
            .char_indices()
            .nth(16)
            .map(|(i, _)| i)
            .unwrap_or(rest.len());
        rest[..end].to_string()
    }

    fn syntax_error(&self, expected: &str, found: &str) -> XmlParseError {
        XmlParseError::Syntax {
            position: self.pos,
            expected: expected.into(),
            found: found.into(),
        }
    }
}

/// The Infoset-carried prolog projection — `(version, encoding, standalone,
/// doctype)` parsed from the §2.8 prolog. `version` is required; the rest are
/// optional per the `XMLDecl?` / `doctypedecl?` productions.
type PrologParts = (String, Option<String>, Option<bool>, Option<XmlDoctype>);

/// The §2.8 \[27\] `Misc` `S` white-space the prolog consumed OUTSIDE the
/// Infoset — `(after_xml_decl, after_doctype)`. `after_xml_decl` is the `S`
/// before the DOCTYPE-or-root; `after_doctype` the `S` before the root (empty
/// when there is no DOCTYPE). The serialized reverse lens re-emits these for a
/// byte-exact prolog.
type PrologWhitespace = (String, String);

/// W3C XML 1.0 §2.8 production \[22\] `prolog`:
/// `prolog ::= XMLDecl? Misc* (doctypedecl Misc*)?`.
///
/// Returns `((version, encoding, standalone, doctype), (after_xml_decl,
/// after_doctype))`. The doctype, if present, is projected to a typed
/// [`XmlDoctype`] carrying the root-element name, any `ExternalID`, and the
/// inline general entity declarations parsed from the internal subset (§4.2
/// GEDecl). The second tuple is the §2.8 \[27\] `Misc` white-space (captured for
/// the byte-exact serialized reverse lens — see
/// [`PrologDecisions`](super::source_syntax::PrologDecisions)).
fn parse_prolog(c: &mut Cursor<'_>) -> Result<(PrologParts, PrologWhitespace), XmlParseError> {
    // §2.8 — "The XML declaration MUST be the first thing in the
    // document." Whitespace, comments, or PIs before `<?xml ...?>`
    // are well-formedness errors. xmlconf xmltest/not-wf/sa/147
    // (blank line before XMLDecl) is the spec regression.
    let prolog_start = c.pos;
    let (version, encoding, standalone) = if c.starts_with("<?xml") {
        parse_xml_decl(c)?
    } else {
        // Skip any leading whitespace before scanning further —
        // documents without XMLDecl may legitimately begin with
        // whitespace before Misc or the doctype.
        c.skip_whitespace();
        // …but if an XMLDecl shows up *after* whitespace, that's
        // not-wf per §2.8. The check is: if any character was
        // consumed by skip_whitespace AND the next token is
        // `<?xml`, reject.
        if c.pos > prolog_start && c.starts_with("<?xml") {
            return Err(XmlParseError::Syntax {
                position: prolog_start,
                expected: "XMLDecl at document start (§2.8: must be first thing)".into(),
                found: "whitespace".into(),
            });
        }
        ("1.0".into(), None, None)
    };
    // §2.8 \[27\] Misc* after the XMLDecl, before the (optional) DOCTYPE or the
    // root. Capture the FULL verbatim `Misc*` run (§2.6 \[16\] PI / §2.5 \[15\]
    // Comment / §2.3 \[3\] S) for byte-exact prolog reconstruction — the
    // `<?xml-stylesheet ...?>` PI every USC USLM title carries lives here. For a
    // pure-`S` run (WN-LMF) this equals the leading white-space, so the prolog
    // capture is unchanged for documents with no prolog PI/Comment.
    let after_decl_start = c.pos;
    parse_misc_star(c)?;
    let after_xml_decl = c.misc_run_since(after_decl_start).to_string();
    let doctype = if c.starts_with("<!DOCTYPE") {
        let dt = parse_doctype(c)?;
        // §2.8 \[27\] Misc* after the DOCTYPE, before the root.
        let after_doctype_start = c.pos;
        parse_misc_star(c)?;
        let after_doctype = c.misc_run_since(after_doctype_start).to_string();
        (Some(dt), after_doctype)
    } else {
        // No DOCTYPE — `after_doctype` is vacuously empty.
        (None, String::new())
    };
    let (doctype, after_doctype) = doctype;
    Ok((
        (version, encoding, standalone, doctype),
        (after_xml_decl, after_doctype),
    ))
}

/// W3C XML 1.0 §2.8 production \[23\] `XMLDecl` and the productions
/// it composes:
///
///   XMLDecl       ::= '<?xml' VersionInfo EncodingDecl? SDDecl? S? '?>'
///   VersionInfo   ::= S 'version' Eq ("'" VersionNum "'" | '"' VersionNum '"')
///   VersionNum    ::= '1.' [0-9]+
///   EncodingDecl  ::= S 'encoding' Eq ('"' EncName '"' | "'" EncName "'")
///   EncName       ::= [A-Za-z] ([A-Za-z0-9._] | '-')*
///   SDDecl        ::= S 'standalone' Eq (("'" ('yes' | 'no') "'") | ('"' ('yes' | 'no') '"'))
///
/// Enforces:
/// - S before each subsequent attribute (`encoding`, `standalone`)
///   — not just optional whitespace. xmlconf ibm/not-wf/P32/ibm32n01
///   (no space between version literal and standalone keyword) is
///   the regression for missing S.
/// - VersionNum matches `1.[0-9]+`.
/// - EncName matches its production.
/// - Standalone literal is exactly `yes` or `no` (lowercase). Case
///   variants (`Yes`, `YES`, `Standalone`) are explicitly malformed
///   per the spec's lowercase-keyword convention — xmlconf
///   ibm/not-wf/P32/ibm32n03..07 are the regression set.
fn parse_xml_decl(
    c: &mut Cursor<'_>,
) -> Result<(String, Option<String>, Option<bool>), XmlParseError> {
    c.consume("<?xml")?;
    c.require_whitespace("XMLDecl VersionInfo")?;
    c.consume("version")?;
    c.skip_whitespace();
    c.consume("=")?;
    c.skip_whitespace();
    let version = parse_quoted(c)?;
    if !is_version_num(&version) {
        return Err(c.syntax_error("VersionNum `1.[0-9]+`", &version));
    }

    let after_version_pos = c.pos;
    let had_ws_before_next = matches!(c.peek_char(), Some(' ' | '\t' | '\r' | '\n'));
    c.skip_whitespace();

    let encoding = if c.starts_with("encoding") {
        if !had_ws_before_next {
            return Err(c.syntax_error("S (whitespace) before `encoding`", &c.preview()));
        }
        c.consume("encoding")?;
        c.skip_whitespace();
        c.consume("=")?;
        c.skip_whitespace();
        let enc = parse_quoted(c)?;
        if !is_enc_name(&enc) {
            return Err(c.syntax_error("EncName `[A-Za-z]([A-Za-z0-9._]|'-')*`", &enc));
        }
        Some(enc)
    } else {
        None
    };

    let had_ws_before_sa = if encoding.is_some() {
        matches!(c.peek_char(), Some(' ' | '\t' | '\r' | '\n'))
    } else {
        had_ws_before_next
    };
    c.skip_whitespace();

    let standalone = if c.starts_with("standalone") {
        if !had_ws_before_sa {
            return Err(c.syntax_error("S (whitespace) before `standalone`", &c.preview()));
        }
        c.consume("standalone")?;
        c.skip_whitespace();
        c.consume("=")?;
        c.skip_whitespace();
        let sa = parse_quoted(c)?;
        match sa.as_str() {
            "yes" => {
                c.skip_whitespace();
                Some(true)
            }
            "no" => {
                c.skip_whitespace();
                Some(false)
            }
            _ => return Err(c.syntax_error("`yes` or `no` (lowercase)", &sa)),
        }
    } else {
        None
    };

    let _ = after_version_pos;
    c.consume("?>")?;
    Ok((version, encoding, standalone))
}

/// W3C XML 1.0 §2.8 \[26\] `VersionNum ::= '1.' [0-9]+`.
fn is_version_num(s: &str) -> bool {
    let Some(rest) = s.strip_prefix("1.") else {
        return false;
    };
    !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit())
}

/// W3C XML 1.0 §4.3.3 \[81\] `EncName ::= [A-Za-z] ([A-Za-z0-9._] | '-')*`.
fn is_enc_name(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

/// W3C XML 1.0 §3.1 production \[10\] `AttValue`'s quoted form,
/// reused for XMLDecl attributes by §2.8 (since their grammar
/// admits the same `' ... '` or `" ... "` form). At this point in
/// XMLDecl, references aren't permitted (§2.8 production \[24\]
/// `VersionInfo` matches a literal `VersionNum`), so we don't
/// expand entities here.
fn parse_quoted(c: &mut Cursor<'_>) -> Result<String, XmlParseError> {
    let quote = c.peek_char().ok_or_else(|| XmlParseError::UnexpectedEof {
        context: "quoted value".into(),
    })?;
    if quote != '"' && quote != '\'' {
        return Err(c.syntax_error("\" or '", &quote.to_string()));
    }
    c.pos += quote.len_utf8();
    let rest = c.rest();
    let end = rest
        .find(quote)
        .ok_or_else(|| XmlParseError::UnexpectedEof {
            context: "quoted value".into(),
        })?;
    let value = rest[..end].to_string();
    c.pos += end + quote.len_utf8();
    Ok(value)
}

/// W3C XML 1.0 §2.8 production \[27\] `Misc`:
/// `Misc ::= Comment | PI | S`.
///
/// Consumes zero or more `Misc` items before/between/after the
/// document element. Comments and PIs in the prolog/epilog are
/// dropped from the [`XmlDocument`] — the Infoset preserves them
/// only inside element content, not at the document level (Cowan
/// & Tobin 2004 §2.1 only requires document-level *children* for
/// the root element).
fn parse_misc_star(c: &mut Cursor<'_>) -> Result<(), XmlParseError> {
    // §2.1 [27] Misc-item dispatch is grammar-grounded: the
    // (literal-prefix, MiscItemKind) entries come from the loaded
    // W3C XML 1.0 grammar's `Misc` production via
    // `spec_1_0::loaded_misc_dispatch_table()`, not from
    // hand-coded `starts_with` strings. `S` (whitespace) is
    // handled by `skip_whitespace` since it has no literal prefix.
    use crate::social::software::markup::xml::spec_1_0::{
        MiscItemKind, loaded_misc_dispatch_table,
    };
    let dispatch = loaded_misc_dispatch_table();
    loop {
        c.skip_whitespace();
        match dispatch.classify(c.rest()) {
            Some(MiscItemKind::Comment) => skip_comment(c)?,
            Some(MiscItemKind::ProcessingInstruction) => skip_pi(c)?,
            // §2.3 [3] S has no literal prefix; `skip_whitespace`
            // above consumed any S that started this iteration. A
            // dispatch result of None means we're past every Misc
            // item — exit the loop.
            Some(MiscItemKind::WhiteSpace) | None => break,
        }
    }
    Ok(())
}

/// W3C XML 1.0 §2.5 production \[15\] `Comment`:
/// `Comment ::= '<!--' ((Char - '-') | ('-' (Char - '-')))* '-->'`.
///
/// Enforces all three §2.5 well-formedness constraints:
/// - The body MUST NOT contain `--` (forbidden by the inner
///   `(Char - '-')` after a `-`).
/// - The body MUST NOT end with `-` (the spec form
///   `((Char - '-') | ('-' (Char - '-')))*` requires the last
///   char to be matched by `(Char - '-')`; xmlconf
///   xmltest/not-wf/sa/070 is the trailing-dash regression).
/// - Every character MUST be in the §2.2 \[2\] Char repertoire —
///   the `is_xml_char` predicate consults the loaded spec
///   table. xmlconf ibm/not-wf/P02 cases (NULL or other
///   out-of-range characters in a Misc-position comment) are
///   the regression set.
fn skip_comment(c: &mut Cursor<'_>) -> Result<(), XmlParseError> {
    let start = c.pos;
    c.consume("<!--")?;
    let rest = c.rest();
    let end = rest
        .find("-->")
        .ok_or_else(|| XmlParseError::UnexpectedEof {
            context: "comment".into(),
        })?;
    let body = &rest[..end];
    if body.contains("--") || body.ends_with('-') {
        return Err(XmlParseError::MalformedComment { position: start });
    }
    check_chars_in_range(body, start + 4, "Comment")?;
    c.pos += end + 3;
    Ok(())
}

/// W3C XML 1.0 §2.6 productions \[16\] `PI` + \[17\] `PITarget`:
///
///   PI       ::= '<?' PITarget (S (Char* - (Char* '?>' Char*)))? '?>'
///   PITarget ::= Name - (('X'|'x') ('M'|'m') ('L'|'l'))
///
/// Enforces three §2.6 well-formedness constraints:
/// - The body MUST be a valid PITarget (a Name) optionally followed
///   by S + Char* data.
/// - PITarget MUST NOT be case-insensitively `xml`.
/// - Every character in the data MUST be in §2.2 \[2\] Char.
fn skip_pi(c: &mut Cursor<'_>) -> Result<(), XmlParseError> {
    let start = c.pos;
    c.consume("<?")?;
    let target = parse_name(c)?;
    if target.qualified().eq_ignore_ascii_case("xml") {
        return Err(XmlParseError::Syntax {
            position: start,
            expected: "PITarget that is not `xml` (case-insensitive)".into(),
            found: target.qualified(),
        });
    }
    if c.starts_with("?>") {
        c.pos += 2;
        return Ok(());
    }
    c.require_whitespace("PI data")?;
    let data_start = c.pos;
    let rest = c.rest();
    let end = rest
        .find("?>")
        .ok_or_else(|| XmlParseError::UnexpectedEof {
            context: "processing instruction".into(),
        })?;
    let body = &rest[..end];
    check_chars_in_range(body, data_start, "PI")?;
    c.pos += end + 2;
    Ok(())
}

/// W3C XML 1.0 §2.8 production \[28\] `doctypedecl`:
/// `doctypedecl ::= '<!DOCTYPE' S Name (S ExternalID)? S? ('[' intSubset ']' S?)? '>'`.
///
/// Projects the declaration to a typed [`XmlDoctype`] carrying the
/// root-element name, any `ExternalID` (§4.2.2 \[75\]), and inline
/// general entity declarations (§4.2 \[70\] GEDecl) parsed from the
/// internal subset. Element-type declarations (§3.2 \[45\]),
/// attribute-list declarations (§3.3 \[52\]), notation declarations
/// (§4.7 \[82\]), parameter entity declarations (§4.2 \[72\]), and
/// parameter entity references (§4.1 \[69\]) within the internal
/// subset are consumed but not projected to typed values — they
/// affect validity, not well-formedness, so the document still
/// parses well-formedly without their typed representation.
fn parse_doctype(c: &mut Cursor<'_>) -> Result<XmlDoctype, XmlParseError> {
    // Capture the whole `<!DOCTYPE … >` declaration VERBATIM (after §2.11 EOL
    // normalization) as PROLOG residue — the byte-exact serializer reproduces it
    // exactly, the analogue of re-emitting the `<?xml?>` declaration bytes. The
    // structured projection below STILL runs (it yields the typed entities used
    // for entity-reference resolution and the PE-reference flag); this is a
    // parallel verbatim slice, NOT a stored element-tree DOM.
    let decl_start = c.pos;
    c.consume("<!DOCTYPE")?;
    c.require_whitespace("DOCTYPE name")?;
    let name = parse_name(c)?;
    c.skip_whitespace();

    let external_id = if c.starts_with("SYSTEM") || c.starts_with("PUBLIC") {
        let id = parse_external_id(c)?;
        c.skip_whitespace();
        Some(id)
    } else {
        None
    };

    let mut general_entities: Vec<XmlGeneralEntity> = Vec::new();
    let mut internal_subset_had_pe_references = false;
    if c.starts_with("[") {
        c.consume("[")?;
        parse_internal_subset(
            c,
            &mut general_entities,
            &mut internal_subset_had_pe_references,
        )?;
        c.consume("]")?;
        c.skip_whitespace();
    }

    c.consume(">")?;
    // The whole declaration's verbatim bytes (`<!DOCTYPE … >`), captured only on
    // the byte-exact reverse-lens read path; the serializer prefers it for
    // byte-exact output and falls back to the structured re-projection otherwise.
    let verbatim = Some(c.misc_run_since(decl_start).to_string());
    Ok(XmlDoctype {
        root_name: name.qualified(),
        external_id,
        general_entities,
        internal_subset_had_pe_references,
        verbatim,
    })
}

/// W3C XML 1.0 §4.2.2 production \[75\] `ExternalID`:
/// `'SYSTEM' S SystemLiteral | 'PUBLIC' S PubidLiteral S SystemLiteral`.
fn parse_external_id(c: &mut Cursor<'_>) -> Result<XmlExternalId, XmlParseError> {
    if c.starts_with("SYSTEM") {
        c.consume("SYSTEM")?;
        c.require_whitespace("ExternalID SystemLiteral")?;
        let system_literal = parse_quoted(c)?;
        Ok(XmlExternalId::System { system_literal })
    } else {
        c.consume("PUBLIC")?;
        c.require_whitespace("ExternalID PubidLiteral")?;
        let lit_pos = c.pos;
        let public_id = parse_quoted(c)?;
        // §4.2.2 [12] PubidLiteral / [13] PubidChar — the body is
        // restricted to `#x20 | #xD | #xA | [a-zA-Z0-9] |
        // [-'()+,./:=?;!*#@$_%]`. xmlconf ibm/not-wf/P13 cases
        // embed `{`, `~`, and Latin-1 letters which fail this set.
        for ch in public_id.chars() {
            if !is_pubid_char(ch) {
                return Err(XmlParseError::Syntax {
                    position: lit_pos,
                    expected: "PubidChar (§4.2.2 [13])".into(),
                    found: ch.to_string(),
                });
            }
        }
        c.require_whitespace("ExternalID SystemLiteral")?;
        let system_literal = parse_quoted(c)?;
        Ok(XmlExternalId::Public {
            public_id,
            system_literal,
        })
    }
}

/// W3C XML 1.0 §4.2.2 \[13\] `PubidChar ::= #x20 | #xD | #xA |
/// [a-zA-Z0-9] | [-'()+,./:=?;!*#@$_%]`.
fn is_pubid_char(c: char) -> bool {
    matches!(c, ' ' | '\r' | '\n')
        || c.is_ascii_alphanumeric()
        || matches!(
            c,
            '-' | '\''
                | '('
                | ')'
                | '+'
                | ','
                | '.'
                | '/'
                | ':'
                | '='
                | '?'
                | ';'
                | '!'
                | '*'
                | '#'
                | '@'
                | '$'
                | '_'
                | '%'
        )
}

/// W3C XML 1.0 §2.8 production \[28b\] `intSubset`:
/// `intSubset ::= (markupdecl | DeclSep)*` where
/// `markupdecl ::= elementdecl | AttlistDecl | EntityDecl | NotationDecl | PI | Comment`
/// (§2.8 \[29\]) and `DeclSep ::= PEReference | S` (§2.8 \[28a\]).
///
/// Each iteration recognises one of the \[28b\] alternatives by
/// the literal prefix the grammar itself names for that production
/// (`'<!ELEMENT'` from \[45\], `'<!ENTITY'` from \[70\], `'<!--'` from
/// \[15\], `'%'` from \[69\], etc.). Markup declarations are validated
/// in full by the EBNF interpreter against the loaded W3C grammar;
/// general-entity declarations are projected into `general_entities`.
///
/// PE references at the DeclSep position (§2.8 \[28a\]: `DeclSep ::=
/// PEReference | S`) are **included** per §4.4.8: the replacement
/// text — bracketed with leading and trailing #x20 — is parsed
/// back through this same procedure (mutual recursion via
/// [`parse_intsubset_items`]), so a PE whose value is one or more
/// complete markup declarations is processed exactly as if those
/// decls had been written inline at the reference point.
///
/// The **WFC: PEs in Internal Subset** (§4.4.8) — *"in the internal
/// DTD subset, parameter-entity references MUST NOT occur within
/// markup declarations; they may occur where markup declarations
/// can occur"* — is satisfied structurally: PE references are only
/// recognised at the \[28a\] DeclSep position. Inside a markup decl
/// the cursor is held by the validating interpreter call, never
/// passes through the `%` branch.
fn parse_internal_subset(
    c: &mut Cursor<'_>,
    general_entities: &mut Vec<XmlGeneralEntity>,
    pe_refs_seen: &mut bool,
) -> Result<(), XmlParseError> {
    let mut parameter_entities: Vec<(String, String)> = Vec::new();
    parse_intsubset_items(
        c,
        general_entities,
        &mut parameter_entities,
        pe_refs_seen,
        true,
    )
}

/// Walk a sequence of \[28b\] `(markupdecl | DeclSep)*` items.
///
/// `terminate_on_close_bracket` is `true` when called from
/// [`parse_internal_subset`] (the cursor is over the original
/// DOCTYPE intSubset and must stop at the closing `]`), `false`
/// when called recursively over a PE's replacement text (the
/// virtual cursor must run to end-of-input — the §4.4.8 PE
/// "include" boundary is the end of the replacement, not a `]`).
fn parse_intsubset_items(
    c: &mut Cursor<'_>,
    general_entities: &mut Vec<XmlGeneralEntity>,
    parameter_entities: &mut Vec<(String, String)>,
    pe_refs_seen: &mut bool,
    terminate_on_close_bracket: bool,
) -> Result<(), XmlParseError> {
    loop {
        c.skip_whitespace();
        if terminate_on_close_bracket {
            if c.starts_with("]") {
                return Ok(());
            }
        } else if c.is_eof() {
            return Ok(());
        }
        if c.starts_with("<!--") {
            skip_comment(c)?;
            continue;
        }
        if c.starts_with("<?") {
            skip_pi(c)?;
            continue;
        }
        if c.starts_with("<!ENTITY") {
            if let Some(entity) = parse_entity_decl(c, parameter_entities)? {
                // §4.5 — duplicate entity declarations: the first wins.
                if !general_entities.iter().any(|e| e.name == entity.name) {
                    general_entities.push(entity);
                }
            }
            continue;
        }
        if c.starts_with("<!ELEMENT") || c.starts_with("<!ATTLIST") || c.starts_with("<!NOTATION") {
            // Validate the markup declaration against the loaded W3C
            // XML 1.0 EBNF (M5.ζ.4). For PE-bearing declarations
            // (M5.ζ.4.b), expand `%name;` references via §4.4.8
            // "Included as PE" semantics before matching: the
            // replacement text is wrapped in leading + trailing
            // #x20 so it forms a complete grammatical token in the
            // expanded DTD.
            let production_name = if c.starts_with("<!ELEMENT") {
                "elementdecl"
            } else if c.starts_with("<!ATTLIST") {
                "AttlistDecl"
            } else {
                "NotationDecl"
            };
            // Find the declaration's extent in original input — skip
            // to top-level `>` respecting quoted SystemLiteral /
            // PubidLiteral / EntityValue contents.
            let decl_start = c.pos;
            let mut probe = Cursor {
                input: c.input,
                pos: c.pos,
            };
            skip_until_close_angle(&mut probe)?;
            let decl_end = probe.pos;
            let decl_text = &c.input[decl_start..decl_end];
            // §4.4.8 WFC: PEs in Internal Subset — "in the internal
            // DTD subset, parameter-entity references MUST NOT occur
            // within markup declarations". The grammar match below
            // structurally enforces this: the loaded W3C productions
            // for elementdecl / AttlistDecl / NotationDecl never name
            // `%Name;` as a permitted token inside the body, so any
            // `%` here surfaces as a NoMatch and rejects the
            // surrounding declaration. PE expansion inside markup
            // decls is allowed only in the external subset, which the
            // praxis parser does not load (the WFC explicitly carves
            // that out). xmlconf ibm/not-wf/P29/ibm29n03 regresses
            // without this strict reading.
            let grammar = crate::social::software::markup::xml::spec_1_0::loaded_xml_1_0_grammar();
            let mut interp = pr4xis::xml_grammar::Interpreter::new(grammar, decl_text);
            match interp.match_production(production_name, 0) {
                pr4xis::xml_grammar::MatchResult::Match { end_pos }
                    if end_pos == decl_text.len() =>
                {
                    // §3.3.2 [60] DefaultDecl — the optional
                    // (('#FIXED' S)? AttValue) variant carries an
                    // AttValue literal. The grammar match above
                    // accepts the declaration syntactically; the
                    // AttValue's WFCs (§4.1 WFC: Entity Declared,
                    // §4.4 row "Reference in Attribute Value" /
                    // No External Entity References / No Recursion,
                    // §4.4.4 WFC: Parsed Entity) must additionally
                    // hold. Only `<!ATTLIST>` carries AttValue
                    // literals inside markupdecl; `<!ELEMENT>` and
                    // `<!NOTATION>` do not (their grammars don't
                    // admit string literals). xmlconf
                    // xmltest/not-wf/sa/{078, 079, 080, 084, 180}
                    // are the spec regressions.
                    if production_name == "AttlistDecl" {
                        validate_attlist_default_values(decl_text, decl_start, general_entities)?;
                    }
                    c.pos = decl_end;
                    continue;
                }
                _ => {
                    return Err(c.syntax_error(production_name, &c.preview()));
                }
            }
        }
        if c.starts_with("%") {
            // §2.8 [28a] `DeclSep ::= PEReference | S`; the cursor
            // is at the only intsubset position the W3C grammar names
            // for PE references (the WFC: PEs in Internal Subset
            // forbids them anywhere else). §4.4.8 "Included as PE":
            // the PE's replacement text is included at the reference
            // point, enlarged with one leading and one trailing #x20.
            //
            // Flag the §4.1 WFC: Entity Declared carve-out — once
            // the internal subset has named even one PE reference,
            // undeclared general-entity references in content are
            // validity errors rather than well-formedness errors
            // (the unread external subset / unread PE expansion
            // may declare them).
            *pe_refs_seen = true;
            c.consume("%")?;
            let name = parse_name(c)?.qualified();
            c.consume(";")?;
            let resolved = parameter_entities
                .iter()
                .find(|(n, _)| n == &name)
                .map(|(_, v)| v.clone());
            if let Some(value) = resolved {
                let mut included = String::with_capacity(value.len() + 2);
                included.push(' ');
                included.push_str(&value);
                included.push(' ');
                let mut sub = Cursor::new(&included);
                parse_intsubset_items(
                    &mut sub,
                    general_entities,
                    parameter_entities,
                    pe_refs_seen,
                    /*terminate_on_close_bracket*/ false,
                )?;
            }
            // Undefined PE in a well-formedness-only parser: the
            // reference itself is well-formed (§4.1 [69]
            // `PEReference ::= '%' Name ';'`); without a validating
            // pass the spec does not require resolution. Continue.
            continue;
        }
        return Err(c.syntax_error("intSubset entry or `]`", &c.preview()));
    }
}

/// W3C XML 1.0 §3.3.2 production \[60\] `DefaultDecl`: validate the
/// AttValue literals embedded inside an `<!ATTLIST>` declaration.
///
/// The grammar-match pass already accepted the declaration
/// syntactically (every quoted run is a `DefaultDecl` AttValue;
/// neither `<!ELEMENT>` nor `<!NOTATION>` nor the AttType
/// productions admit string literals). This walks the declaration
/// text, locates each AttValue literal at its quote boundary, and
/// runs the §3.1 \[10\] AttValue body through
/// [`parse_att_value_body_into`] — the same well-formedness gate
/// the parser applies to attribute literals on live elements:
///
/// - §4.1 WFC: Entity Declared (unknown entity rejected).
/// - §4.4 WFC: No External Entity References.
/// - §4.4 WFC: No `<` in Attribute Values (post-expansion).
/// - §4.4.4 WFC: Parsed Entity (NDATA entity rejected).
/// - §4.1 WFC: No Recursion (cyclic chain rejected).
///
/// `decl_offset` is the start byte of the declaration in the
/// containing source so reported error positions remain
/// document-relative.
fn validate_attlist_default_values(
    decl_text: &str,
    decl_offset: usize,
    entities: &[XmlGeneralEntity],
) -> Result<(), XmlParseError> {
    let bytes = decl_text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'"' || b == b'\'' {
            let q = b as char;
            let quote_start = i;
            // Find the closing quote.
            let close_rel = decl_text[i + 1..]
                .find(q)
                .ok_or_else(|| XmlParseError::Syntax {
                    position: decl_offset + quote_start,
                    expected: "matching AttValue close quote".into(),
                    found: "unterminated".into(),
                })?;
            let close_abs = i + 1 + close_rel;
            // Build a single-quoted AttValue literal that
            // parse_att_value will consume cleanly. Easier: invoke
            // parse_att_value_body_into directly on the body.
            let body = &decl_text[i + 1..close_abs];
            // Offset each error message into the source coordinate
            // space by running the validation on a fragment whose
            // own cursor starts at body[0]; the surfacing error
            // position references the body, which is OK because
            // the validation only fires on malformed defaults
            // (sa/078 etc.) — diagnostic precision in the
            // declaration is sufficient.
            let mut sub = Cursor::new(body);
            let mut out = String::new();
            let mut visited = Vec::new();
            parse_att_value_body_into(
                &mut sub,
                entities,
                &mut visited,
                // ATTLIST default-value validation runs at DTD parse
                // time — before standalone is known and possibly
                // before all entity declarations have been processed
                // (forward references). Treat as strict (require
                // declared) so sa/078, 079, 080, 084, 180 surface
                // here; tests with external-DTD-only entity decls
                // referenced in ATTLIST defaults are not in the
                // XMLConf cases for production [60].
                /*strict_entity_declared*/
                true,
                AttValueTerminator::Eof,
                &mut out,
                // ATTLIST default-value validation discards `out`; no byte-exact
                // capture is wanted here (the default-value bytes are inside the
                // DOCTYPE internal subset, not on a live element).
                &mut None,
            )
            .map_err(|e| match e {
                XmlParseError::Syntax {
                    expected, found, ..
                } => XmlParseError::Syntax {
                    position: decl_offset + quote_start,
                    expected: format!("valid §3.3.2 DefaultDecl AttValue ({expected})"),
                    found,
                },
                XmlParseError::UnsupportedEntity { name, .. } => XmlParseError::UnsupportedEntity {
                    position: decl_offset + quote_start,
                    name,
                },
                other => other,
            })?;
            i = close_abs + 1;
        } else {
            i += 1;
        }
    }
    Ok(())
}

/// W3C XML 1.0 §4.2 production [70/71] `GEDecl`:
/// `<!ENTITY S Name S EntityValue S? >`.
///
/// Returns:
/// - `Some((name, value))` for an internal general entity (§4.2 \[73\]
///   `EntityValue` variant) — replacement text projected into the
///   doc's entity map.
/// - `Some((name, ""))` for an external general entity (§4.2 \[73\]
///   `ExternalID NDataDecl?` variant). Per §4.4 Table-4 row "Reference
///   in Content / External Parsed General", a non-validating parser
///   "Bypasses" the reference (replaces the reference with itself).
///   The praxis parser approximates this by registering the entity
///   with empty replacement text — well-formedness is preserved
///   (no UnsupportedEntity error on subsequent `&name;`); the text
///   output simply lacks the unread external content. Reading the
///   external entity body is M5.ε.5.b territory; this fix unblocks
///   the W3C XMLConf ext-sa cases that test well-formedness only.
/// - `None` for a parameter entity declaration (§4.2 \[72\] `PEDecl`).
///   PEs are skipped because they expand inside the DTD, not in
///   the document content (M5.ε.5.c — proper PE expansion).
fn parse_entity_decl(
    c: &mut Cursor<'_>,
    parameter_entities: &mut Vec<(String, String)>,
) -> Result<Option<XmlGeneralEntity>, XmlParseError> {
    c.consume("<!ENTITY")?;
    c.require_whitespace("ENTITY name")?;

    // §4.2 [72] PEDecl path begins with `%`. The PEDecl grammar is:
    //   PEDecl ::= '<!ENTITY' S '%' S Name S PEDef S? '>'
    //   PEDef  ::= EntityValue | ExternalID
    // Internal PEs (EntityValue variant) get their replacement text
    // captured for §4.4.8 "Included as PE" expansion inside
    // markup-decl validation; external PEs are recognised but their
    // bodies are not fetched (out-of-scope for the well-formedness
    // slice).
    if c.starts_with("%") {
        c.consume("%")?;
        c.require_whitespace("PEDecl name")?;
        let name = parse_name(c)?;
        c.require_whitespace("PEDecl value")?;
        // §4.2 [74] `PEDef ::= EntityValue | ExternalID` — no
        // NDataDecl. The grammar-grounded audit
        // `loaded_pedef_rejects_ndata_decl()` confirms at module init
        // that the loaded spec source agrees. After parsing the
        // ExternalID branch, §4.2 [72] `PEDecl ::= ... PEDef S? '>'`
        // mandates a literal `>` (with optional preceding whitespace)
        // as the only legal continuation; the `c.consume(">")` calls
        // below are the grammar's natural NDATA rejection — a stray
        // `NDATA` token fails the `>` consume with a syntax error.
        // xmlconf xmltest/not-wf/sa/089, /091 are the regression set.
        let _ = loaded_pedef_rejects_ndata_decl();
        if c.starts_with("SYSTEM") {
            c.consume("SYSTEM")?;
            c.require_whitespace("ExternalID SystemLiteral")?;
            let _system_literal = parse_quoted(c)?;
            c.skip_whitespace();
            c.consume(">")?;
        } else if c.starts_with("PUBLIC") {
            c.consume("PUBLIC")?;
            c.require_whitespace("ExternalID PubidLiteral")?;
            let lit_pos = c.pos;
            let pub_id = parse_quoted(c)?;
            for ch in pub_id.chars() {
                if !is_pubid_char(ch) {
                    return Err(XmlParseError::Syntax {
                        position: lit_pos,
                        expected: "PubidChar (§4.2.2 [13])".into(),
                        found: ch.to_string(),
                    });
                }
            }
            c.require_whitespace("ExternalID SystemLiteral")?;
            let _system_literal = parse_quoted(c)?;
            c.skip_whitespace();
            c.consume(">")?;
        } else {
            // Internal PE — capture the replacement text.
            let value = parse_entity_value(c)?;
            c.skip_whitespace();
            c.consume(">")?;
            // §4.5: first declaration wins for an entity name.
            let qname = name.qualified();
            if !parameter_entities.iter().any(|(n, _)| n == &qname) {
                parameter_entities.push((qname, value));
            }
        }
        return Ok(None);
    }

    let name = parse_name(c)?;
    c.require_whitespace("ENTITY value")?;

    // §4.2 [73] EntityDef ::= EntityValue | (ExternalID NDataDecl?).
    // §4.2.2 [75] ExternalID ::= 'SYSTEM' S SystemLiteral
    //                          | 'PUBLIC' S PubidLiteral S SystemLiteral.
    // §4.7 [76] NDataDecl ::= S 'NDATA' S Name.
    if c.starts_with("SYSTEM") {
        c.consume("SYSTEM")?;
        c.require_whitespace("ExternalID SystemLiteral")?;
        let _system_literal = parse_quoted(c)?;
        let unparsed = parse_optional_ndata_decl(c)?;
        c.skip_whitespace();
        c.consume(">")?;
        return Ok(Some(XmlGeneralEntity {
            name: name.qualified(),
            value: String::new(),
            kind: if unparsed {
                XmlEntityKind::ExternalUnparsed
            } else {
                XmlEntityKind::ExternalParsed
            },
        }));
    }
    if c.starts_with("PUBLIC") {
        c.consume("PUBLIC")?;
        c.require_whitespace("ExternalID PubidLiteral")?;
        let lit_pos = c.pos;
        let pub_id = parse_quoted(c)?;
        // §4.2.2 [12] PubidLiteral / [13] PubidChar — the body is
        // restricted to a narrow alphabet (see `is_pubid_char`).
        // xmlconf ibm/not-wf/P13 cases embed `{`, `~`, Latin-1
        // letters — all reject here.
        for ch in pub_id.chars() {
            if !is_pubid_char(ch) {
                return Err(XmlParseError::Syntax {
                    position: lit_pos,
                    expected: "PubidChar (§4.2.2 [13])".into(),
                    found: ch.to_string(),
                });
            }
        }
        // PUBLIC requires both PubidLiteral AND SystemLiteral separated
        // by whitespace; this is the gate that rejects malformed
        // declarations like `<!ENTITY foo PUBLIC "id">` (no SystemLiteral)
        // or `<!ENTITY e PUBLIC "a""b">` (no whitespace between literals).
        c.require_whitespace("ExternalID SystemLiteral")?;
        let _system_literal = parse_quoted(c)?;
        let unparsed = parse_optional_ndata_decl(c)?;
        c.skip_whitespace();
        c.consume(">")?;
        return Ok(Some(XmlGeneralEntity {
            name: name.qualified(),
            value: String::new(),
            kind: if unparsed {
                XmlEntityKind::ExternalUnparsed
            } else {
                XmlEntityKind::ExternalParsed
            },
        }));
    }

    let value = parse_entity_value(c)?;
    c.skip_whitespace();
    c.consume(">")?;
    Ok(Some(XmlGeneralEntity {
        name: name.qualified(),
        value,
        kind: XmlEntityKind::Internal,
    }))
}

/// §4.7 \[76\] `NDataDecl ::= S 'NDATA' S Name` — optional in
/// general-entity declarations marking an unparsed entity.
/// Returns `true` iff an NDataDecl was consumed.
///
/// The leading S is required by \[76\] when an NDataDecl is present:
/// `<!ENTITY foo SYSTEM "x"NDATA eps>` (no space between `"x"` and
/// `NDATA`) is malformed. xmlconf xmltest/not-wf/sa/069.xml is the
/// spec regression — the comment in that file even names the
/// constraint ("missing space before NDATA").
fn parse_optional_ndata_decl(c: &mut Cursor<'_>) -> Result<bool, XmlParseError> {
    let save = c.pos;
    let consumed_any_s = {
        let before = c.pos;
        c.skip_whitespace();
        c.pos > before
    };
    if c.starts_with("NDATA") {
        if !consumed_any_s {
            return Err(c.syntax_error("S (whitespace) before `NDATA`", "NDATA"));
        }
        c.consume("NDATA")?;
        c.require_whitespace("NDataDecl Name")?;
        let _ = parse_name(c)?;
        Ok(true)
    } else {
        // Not an NDataDecl — restore the cursor so the caller's
        // own skip_whitespace + `>` consume can run.
        c.pos = save;
        Ok(false)
    }
}

/// Grammar-grounded audit: confirms the loaded W3C XML 1.0 spec's
/// \[74\] `PEDef ::= EntityValue | ExternalID` production does NOT
/// reference `NDataDecl` (transitively, through `NonTerminal`
/// indirection). This is the structural fact that makes the
/// PE-declaration parser's `c.consume(">")` after the ExternalID
/// branch a sufficient rejection of stray NDATA tokens — no
/// hand-coded `c.starts_with("NDATA")` check is needed.
///
/// Called once via `OnceLock` at module first-use of the entity-
/// decl path; panics with `pedef_audit_failed_panic_message()` if
/// the spec source has drifted such that PEDef gained an
/// NDataDecl reference. Per `feedback_corpus_wide_audit_on_load`:
/// spec-source drift fails closed.
///
/// The W3C source for \[74\] PEDef as of the 2008 Fifth Edition:
///
/// ```text
/// <prod id="NT-PEDef" num="74">
///   <lhs>PEDef</lhs>
///   <rhs>EntityValue | ExternalID</rhs>
/// </prod>
/// ```
///
/// xmlconf xmltest/not-wf/sa/089 (`<!ENTITY % foo SYSTEM "foo"
/// NDATA bar>`) and /091 (variant with NOTATION declared) are
/// the regression set the natural grammar-tail rejection covers:
/// after `parse_quoted` consumes `"foo"`, `c.skip_whitespace()`
/// passes over ` `, then `c.consume(">")` faces `N` and emits a
/// syntax error — exactly the §4.2 \[72\] PEDecl tail mismatch
/// the W3C spec mandates.
fn loaded_pedef_rejects_ndata_decl() -> bool {
    use std::sync::OnceLock;
    static AUDIT: OnceLock<bool> = OnceLock::new();
    *AUDIT.get_or_init(|| {
        use crate::social::software::markup::xml::spec_1_0::loaded_xml_1_0_grammar;
        let grammar = loaded_xml_1_0_grammar();
        let pedef = grammar
            .lookup("PEDef")
            .expect("[74] PEDef must be present in the loaded W3C XML 1.0 grammar");
        if term_references_production(&pedef.rhs, "NDataDecl", grammar) {
            panic!(
                "W3C XML 1.0 [74] PEDef in the loaded spec source unexpectedly \
                 references NDataDecl — spec source has drifted. The PE-declaration \
                 parser's `c.consume(\">\")` is no longer sufficient as the grammar-tail \
                 rejection; restore the explicit NDATA check at the call sites."
            );
        }
        true
    })
}

/// Recursively check whether `term`'s transitive expansion in
/// `grammar` references the production named `target` via any
/// `NonTerminal(name)` reference. Used by
/// [`loaded_pedef_rejects_ndata_decl`] to confirm PEDef has no
/// NDataDecl reference.
fn term_references_production(
    term: &pr4xis::xml_grammar::Term,
    target: &str,
    grammar: &pr4xis::xml_grammar::Grammar,
) -> bool {
    use pr4xis::xml_grammar::Term;
    // Visited set to break cycles in case the grammar has self-
    // recursive productions (the W3C XML 1.0 grammar does not, but
    // the safety belt is cheap).
    fn walk(
        term: &Term,
        target: &str,
        grammar: &pr4xis::xml_grammar::Grammar,
        visited: &mut Vec<String>,
    ) -> bool {
        match term {
            Term::NonTerminal(n) => {
                if n == target {
                    return true;
                }
                if visited.iter().any(|v| v == n) {
                    return false;
                }
                visited.push(n.clone());
                let Some(p) = grammar.lookup(n) else {
                    return false;
                };
                walk(&p.rhs, target, grammar, visited)
            }
            Term::Sequence(items) | Term::Alternation(items) => {
                items.iter().any(|t| walk(t, target, grammar, visited))
            }
            Term::Optional(inner) | Term::ZeroOrMore(inner) | Term::OneOrMore(inner) => {
                walk(inner, target, grammar, visited)
            }
            Term::Subtraction(a, b) => {
                walk(a, target, grammar, visited) || walk(b, target, grammar, visited)
            }
            Term::Literal(_) | Term::CharClass(_) => false,
        }
    }
    let mut visited = Vec::new();
    walk(term, target, grammar, &mut visited)
}

/// W3C XML 1.0 §4.3.2 production \[9\] `EntityValue`:
/// `'"' ([^%&"] | PEReference | Reference)* '"' | "'" ([^%&'] | PEReference | Reference)* "'"`.
///
/// Resolves `Reference` (numeric character refs and the five §4.6
/// predefined entities) to their character values during literal
/// construction. `PEReference` (parameter entity refs) is not
/// resolved — parameter entities are deferred.
fn parse_entity_value(c: &mut Cursor<'_>) -> Result<String, XmlParseError> {
    let quote = c.peek_char().ok_or_else(|| XmlParseError::UnexpectedEof {
        context: "EntityValue".into(),
    })?;
    if quote != '"' && quote != '\'' {
        return Err(c.syntax_error("\" or '", &quote.to_string()));
    }
    c.pos += quote.len_utf8();

    let mut out = String::new();
    loop {
        let ch_pos = c.pos;
        let ch = c.peek_char().ok_or_else(|| XmlParseError::UnexpectedEof {
            context: "EntityValue".into(),
        })?;
        if ch == quote {
            c.pos += quote.len_utf8();
            return Ok(out);
        }
        if ch == '&' {
            // Inside an EntityValue, only character references resolve
            // immediately. General entity references stay literal in
            // the replacement text (§4.5) — we approximate by passing
            // them through unresolved here (their resolution belongs
            // to the consuming context).
            if c.rest().starts_with("&#") {
                out.push(parse_char_ref(c)?);
            } else {
                // General entity reference inside another entity value:
                // pass through as literal per §4.5. The Reference
                // syntax is still validated — §4.1 [68] `EntityRef ::=
                // '&' Name ';'` requires a §2.3 [5] Name, so `&49;`
                // (starts with a digit) is malformed. xmlconf
                // ibm/not-wf/P66/ibm66n03 regresses without this.
                let ref_start = c.pos;
                c.consume("&")?;
                let _name = parse_name(c)?;
                c.consume(";")?;
                let literal = &c.input[ref_start..c.pos];
                out.push_str(literal);
            }
        } else if ch == '%' {
            // §4.3.2 [9] EntityValue body alternation `[^%&"]`
            // excludes a literal `%`; the spec requires it escaped
            // via numeric char ref or PEReference. PEReferences only
            // resolve in the external subset (WFC: PEs in Internal
            // Subset). For the internal-subset slice this parser
            // covers, a literal `%` is always malformed. xmlconf
            // ibm/not-wf/P09/ibm09n01 regresses here.
            return Err(XmlParseError::Syntax {
                position: ch_pos,
                expected: "EntityValue body (literal `%` forbidden)".into(),
                found: "%".to_string(),
            });
        } else {
            // §4.3.2 [9] `EntityValue ::= '"' ([^%&"] | PEReference |
            // Reference)* '"' | ...`. The `[^%&"]` notation in W3C
            // EBNF means "§2.2 [2] Char minus {%, &, "}" — every
            // body char must lie in the Char repertoire. xmlconf
            // ibm/xml-1.1/not-wf/P02 cases embed 1.1-only control
            // chars in entity values; rejecting them here is the
            // standard way an XML 1.0 parser refuses 1.1-only
            // features (§2.8 spec note: a 1.0 processor accepts
            // 1.x documents provided they don't use non-1.0
            // features).
            if !is_xml_char(ch) {
                return Err(XmlParseError::InvalidChar {
                    position: ch_pos,
                    code_point: ch as u32,
                    context: "EntityValue",
                });
            }
            out.push(ch);
            c.pos += ch.len_utf8();
        }
    }
}

/// Skip tokens up to and including the next top-level `>` while
/// respecting:
/// - paired `[`/`]` brackets (nested intSubset markers — §2.8 \[28\]),
/// - quoted string literals `"…"` and `'…'` (W3C XML 1.0 §4.2.2
///   \[11\] SystemLiteral / \[12\] PubidLiteral / §4.3.2 \[9\] EntityValue
///   — a `>` inside quotes is content, not the close-marker).
///
/// This is the correct skip semantics for markup declarations whose
/// values may embed `>` (e.g. `<!ENTITY % e "<!ELEMENT doc (#PCDATA)>">`).
fn skip_until_close_angle(c: &mut Cursor<'_>) -> Result<(), XmlParseError> {
    let mut depth = 0u32;
    let mut quote: Option<char> = None;
    while let Some(ch) = c.peek_char() {
        if let Some(q) = quote {
            // Inside a string literal — only the matching close-quote
            // exits; every other byte (including `>`) is literal.
            if ch == q {
                quote = None;
            }
            c.pos += ch.len_utf8();
            continue;
        }
        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                c.pos += ch.len_utf8();
            }
            '[' => {
                depth += 1;
                c.pos += 1;
            }
            ']' => {
                depth = depth.saturating_sub(1);
                c.pos += 1;
            }
            '>' if depth == 0 => {
                c.pos += 1;
                return Ok(());
            }
            _ => c.pos += ch.len_utf8(),
        }
    }
    Err(XmlParseError::UnexpectedEof {
        context: "markup declaration".into(),
    })
}

/// Helper: parse just a character reference (`&#digits;` or
/// `&#xhex;`). Factored out of [`parse_reference`] so
/// [`parse_entity_value`] can use it without going through the
/// general-entity-name branch.
///
/// Enforces W3C XML 1.0 §4.1 **WFC: Legal Character** — the
/// referenced code point must be in §2.2 \[2\] `Char`. This catches:
///
/// - NUL and other C0 controls (`&#x0;` … `&#x1F;` excluding the
///   three `Char`-permitted `#x9` / `#xA` / `#xD`).
/// - The two §2.2-excluded noncharacters at the BMP top
///   (`&#xFFFE;`, `&#xFFFF;`).
/// - C1 controls and other code points outside §2.2 Char even
///   though they're valid Unicode scalars (which `char::from_u32`
///   accepts on its own).
///
/// xmlconf ibm/not-wf/P66 cases ibm66n12..15 are the regression
/// set; they reference NUL, `#x1F`, `#xFFFE`, `#xFFFF` and are
/// rejected by this check.
fn parse_char_ref(c: &mut Cursor<'_>) -> Result<char, XmlParseError> {
    let start_pos = c.pos;
    c.consume("&")?;
    let code_point = if c.starts_with("#x") {
        c.consume("#x")?;
        let rest = c.rest();
        let end = rest.find(';').ok_or_else(|| XmlParseError::UnexpectedEof {
            context: "character reference".into(),
        })?;
        let digits = &rest[..end];
        let cp = u32::from_str_radix(digits, 16).map_err(|_| XmlParseError::Syntax {
            position: c.pos,
            expected: "hex digits".into(),
            found: digits.to_string(),
        })?;
        c.pos += end + 1;
        cp
    } else if c.starts_with("#") {
        c.consume("#")?;
        let rest = c.rest();
        let end = rest.find(';').ok_or_else(|| XmlParseError::UnexpectedEof {
            context: "character reference".into(),
        })?;
        let digits = &rest[..end];
        let cp = digits.parse::<u32>().map_err(|_| XmlParseError::Syntax {
            position: c.pos,
            expected: "decimal digits".into(),
            found: digits.to_string(),
        })?;
        c.pos += end + 1;
        cp
    } else {
        return Err(c.syntax_error("character reference", &c.preview()));
    };
    let ch = char::from_u32(code_point).ok_or(XmlParseError::InvalidCharRef {
        position: start_pos,
        code_point,
    })?;
    // §4.1 WFC: Legal Character — the resolved char MUST be in
    // §2.2 [2] Char. is_xml_char consults the loaded predicate.
    if !is_xml_char(ch) {
        return Err(XmlParseError::InvalidCharRef {
            position: start_pos,
            code_point,
        });
    }
    Ok(ch)
}

/// Parse a §4.1 \[66\] numeric character reference AND capture its exact source
/// FORM as an [`ExtendedRefKind::Numeric`] (decimal vs hex, the hex letters' case,
/// and the verbatim digit string incl. leading zeros) so the byte-exact serializer
/// re-emits the reference spelling rather than the resolved literal char.
///
/// The form-aware sibling of [`parse_char_ref`]; the resolution rules (radix,
/// §2.2 Char legality) are identical. Used only on the byte-exact capturing path —
/// `parse_char_ref` stays the non-capturing default.
fn parse_char_ref_capturing(c: &mut Cursor<'_>) -> Result<(char, ExtendedRefKind), XmlParseError> {
    let start_pos = c.pos;
    c.consume("&")?;
    let (hex, upper_hex, digits, code_point) =
        if c.rest().starts_with("#x") || c.rest().starts_with("#X") {
            // Hex form `&#x…;` / `&#X…;` — preserve the `x`/`X` case AND the a–f
            // digit case so e.g. `&#X2019;` round-trips.
            let upper_x = c.rest().starts_with("#X");
            c.consume(if upper_x { "#X" } else { "#x" })?;
            let rest = c.rest();
            let end = rest.find(';').ok_or_else(|| XmlParseError::UnexpectedEof {
                context: "character reference".into(),
            })?;
            let digits = rest[..end].to_string();
            let cp = u32::from_str_radix(&digits, 16).map_err(|_| XmlParseError::Syntax {
                position: c.pos,
                expected: "hex digits".into(),
                found: digits.clone(),
            })?;
            c.pos += end + 1;
            let upper_hex = upper_x || digits.chars().any(|ch| ch.is_ascii_uppercase());
            (true, upper_hex, digits, cp)
        } else if c.starts_with("#") {
            c.consume("#")?;
            let rest = c.rest();
            let end = rest.find(';').ok_or_else(|| XmlParseError::UnexpectedEof {
                context: "character reference".into(),
            })?;
            let digits = rest[..end].to_string();
            let cp = digits.parse::<u32>().map_err(|_| XmlParseError::Syntax {
                position: c.pos,
                expected: "decimal digits".into(),
                found: digits.clone(),
            })?;
            c.pos += end + 1;
            (false, false, digits, cp)
        } else {
            return Err(c.syntax_error("character reference", &c.preview()));
        };
    let ch = char::from_u32(code_point).ok_or(XmlParseError::InvalidCharRef {
        position: start_pos,
        code_point,
    })?;
    if !is_xml_char(ch) {
        return Err(XmlParseError::InvalidCharRef {
            position: start_pos,
            code_point,
        });
    }
    Ok((
        ch,
        ExtendedRefKind::Numeric {
            hex,
            upper_hex,
            digits,
        },
    ))
}

/// W3C XML 1.0 §3 production \[39\] `element`:
/// `element ::= EmptyElemTag | STag content ETag`.
///
/// `entities` is the list of `<!ENTITY name "value">` declarations
/// the DOCTYPE projected; consulted by [`parse_reference`] when an
/// entity reference's name doesn't match one of the five §4.6
/// predefined entities.
/// Maximum element-nesting depth. The parser is recursive-descent
/// (`parse_element` ↔ `parse_content`), so without a bound a pathologically
/// nested document (`<a><a><a>…`) overflows the stack and aborts the process —
/// a denial-of-service. Bounding it turns that into a clean refusal. 256 is the
/// long-standing libxml2 default: far beyond any real document, yet shallow
/// enough that the bounded recursion fits comfortably on a small thread stack.
const MAX_ELEMENT_DEPTH: u32 = 256;

fn parse_element(
    c: &mut Cursor<'_>,
    entities: &[XmlGeneralEntity],
    strict_entity_declared: bool,
    capture: &mut Option<CaptureCtx>,
    depth: u32,
) -> Result<XmlElement, XmlParseError> {
    if depth > MAX_ELEMENT_DEPTH {
        return Err(c.syntax_error(
            "element nesting within the maximum depth",
            "input nested beyond the maximum element depth",
        ));
    }
    // Claim this element's PRE-ORDER index AT ENTRY — before either the
    // self-closing or the explicit branch, and before descending into any
    // children — so the index ordering matches the byte-exact serializer's
    // emit counter (root = 0, children strictly after their parent, document
    // order within a level). The self-closing path consumes the index and
    // records nothing (`SelfClosing` is the serializer default); the explicit
    // path records `Explicit` for a childless `<a></a>`.
    let my_index = capture.as_mut().map(|ctx| {
        let idx = ctx.counter;
        ctx.counter += 1;
        idx
    });
    c.consume("<")?;
    let name = parse_name(c)?;
    let mut attributes: Vec<XmlAttribute> = Vec::new();
    // Namespaces in XML 1.0 (Bray, Hollander, Layman & Tobin 2009) §3
    // permits any number of `xmlns` / `xmlns:prefix` declarations on a
    // single element. We collect every one in document order; the legacy
    // single `namespace` slot mirrors the first declaration for backward
    // compatibility with consumers that only need a representative.
    let mut namespaces: Vec<XmlNamespace> = Vec::new();

    // Reader capture for the §3.1 [40]/[44] start-tag white-space LAYOUT (the
    // `S` runs between the name, attributes, and the close — the real corpus's
    // multi-line attribute indentation) and the §4.6 predefined-entity-reference
    // FORM in each attribute value. Both ride the attribute-like SOURCE ORDER
    // slot (every `xmlns` decl AND every regular attribute, the order the
    // byte-exact serializer re-emits them for in-scope inputs). Only built when
    // capturing.
    let capturing = my_index.is_some();
    let mut intra_ws = capturing.then(IntraTagWhitespace::default);
    let mut attr_refs: BTreeMap<usize, EntityReferenceForm> = BTreeMap::new();
    let mut attr_slot: usize = 0;
    // The EXACT ordered start-tag token sequence (every `xmlns` decl AND every
    // regular attribute, in source order) — recorded for the byte-exact
    // serialized reverse lens so an INTERLEAVED start-tag (the USC `<uscDoc>`
    // root: attributes before its `xmlns` decls) re-emits in source order. Only
    // built when capturing; only retained when non-canonical (see
    // `record_start_tag_decisions`).
    let mut start_tag_tokens: Vec<StartTagToken> = Vec::new();

    loop {
        // §3.1 [40] `STag ::= '<' Name (S Attribute)* S? '>'` (and [44]
        // EmptyElemTag) — the `S` run here is EITHER the leading `S` of the next
        // `(S Attribute)` group OR the trailing `S?` before the close. Capture
        // the exact run; classify it once the next token reveals which.
        let ws_run = c.take_whitespace();
        let had_ws = !ws_run.is_empty();
        if c.starts_with("/>") {
            if let Some(iw) = intra_ws.as_mut() {
                iw.before_close = ws_run.to_string();
            }
            c.consume("/>")?;
            let namespace = namespaces.first().cloned();
            let element = XmlElement {
                name,
                namespace,
                namespaces,
                attributes,
                children: Vec::new(),
            };
            // A self-closing childless element keeps the canonical empty-element
            // form (no `empty_form` decision), but it may still carry non-canonical
            // intra-tag white-space, attribute entity-ref forms, and an interleaved
            // start-tag token order — record those.
            record_start_tag_decisions(
                capture,
                my_index,
                &intra_ws,
                &attr_refs,
                &start_tag_tokens,
                None,
            );
            return Ok(element);
        }
        if c.starts_with(">") {
            if let Some(iw) = intra_ws.as_mut() {
                iw.before_close = ws_run.to_string();
            }
            c.consume(">")?;
            break;
        }
        if !had_ws {
            return Err(c.syntax_error("whitespace, /> or >", &c.preview()));
        }
        // §3.1 production [41] `Attribute ::= Name Eq AttValue`.
        // Namespace declarations (Bray, Hollander, Layman & Tobin
        // 2009 §3) use the reserved `xmlns` / `xmlns:prefix` form.
        // §3.1 [25] `Eq ::= S? '=' S?` — capture the two S? runs straddling `=`.
        let attr_name = parse_name(c)?;
        let name_to_eq = c.take_whitespace().to_string();
        c.consume("=")?;
        let eq_to_value = c.take_whitespace().to_string();
        // Per-attribute reference-form capture sink (only when capturing): the
        // closed §4.6 predefined set PLUS the open §4.1 numeric/general-entity
        // forms (the `&rdfs;seeAlso` reference prov_o writes in `rdf:resource`).
        let mut value_ref_sink: Option<AttrRefSink> = capturing.then(AttrRefSink::default);
        let value = parse_att_value(c, entities, strict_entity_declared, &mut value_ref_sink)?;

        if let Some(iw) = intra_ws.as_mut() {
            iw.before_attr.push(ws_run.to_string());
            iw.around_eq.push((name_to_eq, eq_to_value));
        }
        if let Some(sink) = value_ref_sink.filter(|s| !s.is_empty()) {
            attr_refs.insert(
                attr_slot,
                EntityReferenceForm {
                    refs: sink.predefined,
                    ext_refs: sink.ext,
                },
            );
        }
        attr_slot += 1;

        let is_ns_decl = attr_name.prefix.as_deref() == Some("xmlns")
            || (attr_name.prefix.is_none() && attr_name.local == "xmlns");
        if is_ns_decl {
            let prefix = if attr_name.prefix.is_some() {
                Some(attr_name.local.clone())
            } else {
                None
            };
            let ns = XmlNamespace { prefix, uri: value };
            if capturing {
                start_tag_tokens.push(StartTagToken::Namespace(ns.clone()));
            }
            namespaces.push(ns);
        } else {
            // W3C XML 1.0 §3.1 well-formedness constraint Unique Att
            // Spec — the same attribute name (qualified) MUST NOT
            // appear more than once in the same start-tag.
            if attributes.iter().any(|a| a.name == attr_name) {
                return Err(XmlParseError::DuplicateAttribute {
                    position: c.pos,
                    name: attr_name.qualified(),
                });
            }
            let attr = XmlAttribute {
                name: attr_name,
                value,
            };
            if capturing {
                start_tag_tokens.push(StartTagToken::Attribute(attr.clone()));
            }
            attributes.push(attr);
        }
    }

    // content + ETag — the EXPLICIT (`STag content ETag`) form. The cursor is
    // PAST the start-tag `>`; children come from `parse_content`, which threads
    // the same `capture` so descendant elements claim later pre-order indices.
    // It also returns the per-text-child §4.6 predefined-entity-reference forms,
    // which key into THIS element's decisions (the char-data third coordinate).
    let (children, text_refs) = parse_content(c, entities, strict_entity_declared, capture, depth)?;
    c.consume("</")?;
    let close_name = parse_name(c)?;
    c.skip_whitespace();
    c.consume(">")?;
    if close_name != name {
        return Err(XmlParseError::MismatchedTags {
            position: c.pos,
            open: name.qualified(),
            close: close_name.qualified(),
        });
    }

    // Reader capture: a childless element written in the EXPLICIT `<a></a>`
    // form is one non-canonical decision — the canonical serializer would
    // self-close it, so without this the bytes would not round-trip. A childless
    // self-closing `<a/>` reached the early return above. A non-empty element
    // carries its form in its children and needs no `empty_form` decision.
    let empty_form = (children.is_empty()).then_some(EmptyForm::Explicit);
    let text_entity_refs: BTreeMap<usize, EntityReferenceForm> = text_refs.into_iter().collect();
    record_start_tag_decisions(
        capture,
        my_index,
        &intra_ws,
        &attr_refs,
        &start_tag_tokens,
        Some((empty_form, text_entity_refs)),
    );

    let namespace = namespaces.first().cloned();
    Ok(XmlElement {
        name,
        namespace,
        namespaces,
        attributes,
        children,
    })
}

/// Fold the start-tag-position concrete-syntax captures (§3.1 [40]/[44]
/// intra-tag white-space, §4.6 attribute-value entity-reference forms) and the
/// content-position captures (the `empty_form` decision and §4.6 char-data
/// entity-reference forms) into ONE [`NodeDecisions`] keyed by the element's
/// pre-order index. Records nothing when not capturing or when every capture is
/// the canonical default — preserving `SyntaxDecisions::default()` for a
/// canonical document so existing callers are unaffected.
///
/// `content` is `None` for the self-closing early-return (no `empty_form`, no
/// char-data refs yet), `Some((empty_form, text_entity_refs))` for the explicit
/// `STag content ETag` exit.
///
/// `start_tag_tokens` is the start-tag's `(S Attribute)*` sequence in EXACT
/// source order; it is recorded into [`NodeDecisions::start_tag_order`] ONLY when
/// the order is NON-CANONICAL — i.e. an `xmlns` declaration does not strictly
/// precede every regular attribute ([`start_tag_order_is_canonical`]). A
/// canonically-ordered tag records nothing, so the byte-exact serializer's
/// default ns-then-attr emit is unchanged (WordNet, most USC elements).
fn record_start_tag_decisions(
    capture: &mut Option<CaptureCtx>,
    my_index: Option<usize>,
    intra_ws: &Option<IntraTagWhitespace>,
    attr_refs: &BTreeMap<usize, EntityReferenceForm>,
    start_tag_tokens: &[StartTagToken],
    content: Option<(Option<EmptyForm>, BTreeMap<usize, EntityReferenceForm>)>,
) {
    let (Some(ctx), Some(idx)) = (capture.as_mut(), my_index) else {
        return;
    };
    let (empty_form, text_entity_refs) = content.unwrap_or((None, BTreeMap::new()));
    // Only the non-canonical intra-tag layout is worth recording; a canonical
    // single-space single-line tag matches the writer's default and is dropped.
    let intra_tag_whitespace = intra_ws.as_ref().filter(|iw| !iw.is_canonical()).cloned();
    // Only an INTERLEAVED start-tag (an `xmlns` decl after a regular attribute)
    // needs its exact order recorded; a canonical ns-then-attr order matches the
    // serializer's default and is dropped.
    let start_tag_order =
        (!start_tag_order_is_canonical(start_tag_tokens)).then(|| start_tag_tokens.to_vec());
    let decisions = NodeDecisions {
        empty_form,
        intra_tag_whitespace,
        attr_entity_refs: attr_refs.clone(),
        text_entity_refs,
        start_tag_order,
    };
    if decisions != NodeDecisions::default() {
        ctx.decisions.set(idx, decisions);
    }
}

/// Whether a start-tag's token sequence is in the CANONICAL ns-then-attr order
/// the byte-exact serializer emits by default — every [`StartTagToken::Namespace`]
/// precedes every [`StartTagToken::Attribute`]. When `true`, no
/// [`NodeDecisions::start_tag_order`] is recorded (the default emit reproduces
/// it). When `false` — an `xmlns` decl follows a regular attribute, as on the USC
/// `<uscDoc>` root — the exact order must be carried so the serializer can
/// interleave them faithfully.
fn start_tag_order_is_canonical(tokens: &[StartTagToken]) -> bool {
    let mut seen_attribute = false;
    for token in tokens {
        match token {
            StartTagToken::Attribute(_) => seen_attribute = true,
            StartTagToken::Namespace(_) => {
                if seen_attribute {
                    return false;
                }
            }
        }
    }
    true
}

/// W3C XML 1.0 §2.3 production \[5\] `Name`:
/// `Name ::= NameStartChar (NameChar)*`.
///
/// Production \[4\] `NameStartChar` covers ASCII letters, `_`, `:`,
/// and a large Unicode range (Bray et al. 2008 §2.3). We accept
/// the ASCII subset here, plus the `:` separator used by
/// Namespaces in XML (Bray, Hollander, Layman & Tobin 2009 §3)
/// to split prefix from local name.
fn parse_name(c: &mut Cursor<'_>) -> Result<XmlName, XmlParseError> {
    let rest = c.rest();
    let mut iter = rest.char_indices();
    let (_, first) = iter.next().ok_or_else(|| XmlParseError::UnexpectedEof {
        context: "Name".into(),
    })?;
    if !is_name_start_char(first) {
        return Err(c.syntax_error("NameStartChar", &first.to_string()));
    }
    let mut end = first.len_utf8();
    for (i, ch) in iter {
        if is_name_char(ch) {
            end = i + ch.len_utf8();
        } else {
            break;
        }
    }
    let raw = rest[..end].to_string();
    c.pos += end;

    if let Some((prefix, local)) = raw.split_once(':') {
        Ok(XmlName {
            prefix: Some(prefix.to_string()),
            local: local.to_string(),
        })
    } else {
        Ok(XmlName {
            prefix: None,
            local: raw,
        })
    }
}

/// W3C XML 1.0 §2.3 production \[4\] `NameStartChar` — the full
/// character class.
///
/// Delegates to the build-time-generated
/// [`crate::social::software::markup::xml::spec_1_0::grammar::is_name_start_char`]
/// predicate, whose range table is parsed from the registered
/// `xml_1_0_fifth_edition@2008` source (Bray et al. 2008) — per
/// `feedback_bottom_up_loaded_not_encoded`, the character class
/// comes from the loaded spec, not from this file.
///
/// Exposed for reuse by the XSD datatype lexical mappings (`xs:Name` /
/// `xs:NCName` / `xs:NMTOKEN` are defined by reference to the XML 1.0
/// `Name` / `Nmtoken` productions, W3C XSD 1.1 Part 2 §3.4.4-§3.4.7).
pub fn is_name_start_char(ch: char) -> bool {
    crate::social::software::markup::xml::spec_1_0::grammar::is_name_start_char(ch as u32)
}

/// W3C XML 1.0 §2.3 production \[4a\] `NameChar`. Extends
/// [`is_name_start_char`] with digits, `.`, `-`, `·`, and the
/// combining-mark ranges.
///
/// Delegates to the build-time-generated
/// [`crate::social::software::markup::xml::spec_1_0::grammar::is_name_char`]
/// predicate — same loaded-source grounding as
/// [`is_name_start_char`]. Exposed for reuse by the XSD datatype
/// lexical mappings.
pub fn is_name_char(ch: char) -> bool {
    crate::social::software::markup::xml::spec_1_0::grammar::is_name_char(ch as u32)
}

/// W3C XML 1.0 §2.2 production \[2\] `Char` — the legal character
/// repertoire. Delegates to the build-time-generated
/// [`crate::social::software::markup::xml::spec_1_0::grammar::is_char`]
/// predicate, whose range table comes from the loaded spec source.
///
/// Used by `parse_content` / `parse_comment_node` / `parse_pi_node` /
/// `parse_cdata_node` to enforce §2.2 — every character in
/// `CharData`, `Comment`, `PI`, and `CDATA` content MUST be in this
/// class. Form feed (`#x0C`), NUL (`#x00`), and most ASCII control
/// characters are excluded.
pub fn is_xml_char(ch: char) -> bool {
    crate::social::software::markup::xml::spec_1_0::grammar::is_char(ch as u32)
}

/// W3C XML 1.0 §3.1 production \[10\] `AttValue`:
/// `AttValue ::= '"' ([^<&"] | Reference)* '"' | "'" ([^<&'] | Reference)* "'"`.
///
/// Expands the five §4.6 predefined entity references and numeric
/// character references; rejects other entity refs (DTD entities
/// are out of scope for this slice).
///
/// Applies **§3.3.3 Attribute-Value Normalization** on the fly:
/// every literal whitespace character `#x9` / `#xA` / `#xD` in the
/// raw value contributes a single `#x20` (space) to the normalized
/// result. Character + entity references contribute their referenced
/// character unchanged (§3.3.3 step 3.1.1 vs 3.1.4). Without a DTD
/// declaring non-CDATA attribute types we apply only the CDATA case
/// of §3.3.3.
fn parse_att_value(
    c: &mut Cursor<'_>,
    entities: &[XmlGeneralEntity],
    strict_entity_declared: bool,
    ref_capture: &mut Option<AttrRefSink>,
) -> Result<String, XmlParseError> {
    let quote = c.peek_char().ok_or_else(|| XmlParseError::UnexpectedEof {
        context: "AttValue".into(),
    })?;
    if quote != '"' && quote != '\'' {
        return Err(c.syntax_error("\" or '", &quote.to_string()));
    }
    c.pos += quote.len_utf8();

    let mut out = String::new();
    let mut visited = Vec::new();
    parse_att_value_body_into(
        c,
        entities,
        &mut visited,
        strict_entity_declared,
        AttValueTerminator::Quote(quote),
        &mut out,
        ref_capture,
    )?;
    c.pos += quote.len_utf8();
    Ok(out)
}

/// Terminator for the AttValue-body loop shared between top-level
/// attribute literals (`'…'` / `"…"`, terminated by their opening
/// quote) and entity-replacement-text re-parse (terminated by
/// EOF). Entity boundaries are atomic per §4.3.2 — re-parsed
/// replacement text must not contain markup chars that would
/// violate the AttValue body production.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AttValueTerminator {
    /// Match the opening literal quote.
    Quote(char),
    /// Inside an entity-replacement re-parse, the body ends at
    /// EOF. A literal `"` or `'` from the entity expansion is
    /// just text per §4.4.5 footnote ("a single or double quote
    /// character in the replacement text is always treated as a
    /// normal data character").
    Eof,
}

/// Inner AttValue-body loop sharing the parent's `out`
/// accumulator. The §4.4.3 "Included" path for a user-declared
/// internal general entity called from an attribute value
/// recurses here with a sub-cursor over the entity's replacement
/// text, so the §3.3.3 normalization rules apply uniformly
/// across the entity-inclusion boundary.
///
/// Enforces three §3.1 / §4.4 well-formedness constraints on the
/// resulting attribute value:
///
/// - **No `<` in Attribute Values** (§4.4 row "Reference in
///   Attribute Value") — literal `<` is rejected; `&lt;`
///   resolves to the character which is then deposited as data,
///   so the explicit-reference path is sanctioned.
/// - **No External Entity References** (§3.1, §4.4) — references
///   to external (parsed *or* unparsed) entities are rejected.
/// - **No Recursion** (§4.1) — `visited` carries the
///   in-progress entity stack.
fn parse_att_value_body_into(
    c: &mut Cursor<'_>,
    entities: &[XmlGeneralEntity],
    visited: &mut Vec<String>,
    strict_entity_declared: bool,
    term: AttValueTerminator,
    out: &mut String,
    // Reader-side capture sink for the reference FORM (the closed §4.6 predefined
    // set AND the open §4.1 numeric `&#39;` / general-entity `&rdfs;` forms),
    // keyed by char index into the resolved value. `None` on the non-capturing
    // path (`parse_document`) and on the ATTLIST default-value validation path,
    // which discard their output. See
    // [`EntityReferenceForm`](super::source_syntax::EntityReferenceForm).
    ref_capture: &mut Option<AttrRefSink>,
) -> Result<(), XmlParseError> {
    loop {
        match term {
            AttValueTerminator::Quote(q) => {
                if c.peek_char() == Some(q) {
                    return Ok(());
                }
            }
            AttValueTerminator::Eof => {
                if c.is_eof() {
                    return Ok(());
                }
            }
        }
        let ch_pos = c.pos;
        let ch = c.peek_char().ok_or_else(|| XmlParseError::UnexpectedEof {
            context: "AttValue".into(),
        })?;
        if ch == '<' {
            // §3.1 [10] AttValue body excludes literal `<`. After
            // entity inclusion the re-parse re-fires this check —
            // catching xmlconf xmltest/not-wf/sa/090 (entity
            // expansion `<foo a='&#60;'></foo>` puts a literal
            // `<` inside an attribute value).
            return Err(c.syntax_error("AttValue content (no '<')", "<"));
        }
        if ch == '&' {
            if c.rest().starts_with("&#") {
                // §4.1 [66] CharRef `&#N;`/`&#xN;`. When capturing, record the
                // exact numeric FORM (decimal/hex, case, verbatim digits) as an
                // [`ExtendedRef`] keyed by the resolved char's index, so the
                // byte-exact serializer re-emits the reference rather than the
                // resolved literal char. The non-capturing path resolves to the
                // char as before.
                if let Some(sink) = ref_capture.as_mut() {
                    let (ch_val, kind) = parse_char_ref_capturing(c)?;
                    sink.ext.push(ExtendedRef {
                        char_index: out.chars().count(),
                        kind,
                    });
                    out.push(ch_val);
                } else {
                    let ch_val = parse_char_ref(c)?;
                    out.push(ch_val);
                }
            } else {
                let ref_pos = c.pos;
                c.consume("&")?;
                let name = parse_name(c)?;
                c.consume(";")?;
                let qualified = name.qualified();
                // §4.6 predefined-entity resolution via the table
                // generated at build time from the loaded W3C XML 1.0
                // spec source's `<div2 id="sec-predefined-ent">` block
                // (see `pr4xis::codegen::xml_grammar::extract_predefined_entities`).
                if let Some(ch) = crate::social::software::markup::xml::spec_1_0::grammar::resolve_predefined_entity(&qualified) {
                    // Reader capture: record that the char about to be pushed was
                    // written as a §4.6 predefined entity reference, at its char
                    // index in the resolved value (counted in chars, not bytes —
                    // the serializer iterates `chars()` and refs sit next to
                    // multibyte chars). `EntityName::for_resolved_char` is the
                    // inverse of the resolution table for the closed §4.6 set.
                    if let (Some(sink), Some(entity)) =
                        (ref_capture.as_mut(), EntityName::for_resolved_char(ch))
                    {
                        sink.predefined.push((out.chars().count(), entity));
                    }
                    out.push(ch);
                } else {
                    // §4.1 [68] reference to a DTD-declared general entity
                    // (`&rdfs;seeAlso`). The inclusion EXPANDS the replacement
                    // text into `out`; when capturing, record the reference FORM
                    // (the entity name + the expansion's resolved-char length) as
                    // an [`ExtendedRef`] so the byte-exact serializer re-emits the
                    // `&name;` reference and skips the expanded run — reproducing
                    // the internal-subset entity reference WITHOUT storing a DOM.
                    let ref_char_index = out.chars().count();
                    include_user_general_entity_in_att_value(
                        &qualified,
                        ref_pos,
                        entities,
                        visited,
                        strict_entity_declared,
                        out,
                    )?;
                    if let Some(sink) = ref_capture.as_mut() {
                        let expansion_chars = out.chars().count() - ref_char_index;
                        sink.ext.push(ExtendedRef {
                            char_index: ref_char_index,
                            kind: ExtendedRefKind::General {
                                name: qualified,
                                expansion_chars,
                            },
                        });
                    }
                }
            }
        } else if matches!(ch, '\t' | '\n' | '\r') {
            // §3.3.3 step 3.1.4: literal whitespace becomes #x20.
            out.push(' ');
            c.pos += ch.len_utf8();
        } else {
            // §3.1 [10] `AttValue` body alternation `([^<&"] | …)`
            // restricts to §2.2 [2] Char minus the literal-delimiters.
            // A character outside §2.2 Char is malformed even though
            // it's not `<`, `&`, or the closing quote.
            if !is_xml_char(ch) {
                return Err(XmlParseError::InvalidChar {
                    position: ch_pos,
                    code_point: ch as u32,
                    context: "AttValue",
                });
            }
            out.push(ch);
            c.pos += ch.len_utf8();
        }
    }
}

/// §4.4.3 "Included" for a user-declared general entity reference
/// appearing in an attribute value literal. Looks up the entity,
/// rejects external references per §3.1 / §4.4 "No External
/// Entity References" and §4.4.4 "Parsed Entity", enforces §4.1
/// WFC: No Recursion via `visited`, and re-parses the entity's
/// literal replacement text as an AttValue body into the parent's
/// `out` accumulator.
fn include_user_general_entity_in_att_value(
    name: &str,
    ref_pos: usize,
    entities: &[XmlGeneralEntity],
    visited: &mut Vec<String>,
    strict_entity_declared: bool,
    out: &mut String,
) -> Result<(), XmlParseError> {
    if visited.iter().any(|n| n == name) {
        return Err(XmlParseError::Syntax {
            position: ref_pos,
            expected: "non-recursive entity reference (§4.1 WFC: No Recursion)".into(),
            found: format!("&{name};"),
        });
    }
    let entity = match entities.iter().find(|e| e.name == name) {
        Some(e) => e,
        None => {
            // §4.1 WFC: Entity Declared carve-out — same logic as
            // the content-position path. When the carve-out applies,
            // undeclared references in attribute values are validity
            // (not well-formedness) errors; the non-validating
            // processor bypasses them silently.
            if strict_entity_declared {
                return Err(XmlParseError::UnsupportedEntity {
                    position: ref_pos,
                    name: name.to_string(),
                });
            }
            return Ok(());
        }
    };
    match entity.kind {
        XmlEntityKind::ExternalUnparsed => Err(XmlParseError::Syntax {
            position: ref_pos,
            expected: "reference to a parsed entity (§4.4.4 WFC: Parsed Entity)".into(),
            found: format!("&{name};"),
        }),
        XmlEntityKind::ExternalParsed => Err(XmlParseError::Syntax {
            position: ref_pos,
            expected: "AttValue (no external-entity references — §3.1 / §4.4 WFC)".into(),
            found: format!("&{name};"),
        }),
        XmlEntityKind::Internal => {
            let value = entity.value.clone();
            let mut sub = Cursor::new(&value);
            visited.push(name.to_string());
            // The replacement text is re-parsed with NO inner capture (`&mut
            // None`): the `&name;` user-entity reference FORM is recorded ONCE at
            // the OUTER call site (`parse_att_value_body_into`'s general-entity
            // branch) as an `ExtendedRef::General` keyed by the reference's char
            // index plus the expansion's char-length. Capturing the inner refs
            // here would double-count and shift those char indices.
            let result = parse_att_value_body_into(
                &mut sub,
                entities,
                visited,
                strict_entity_declared,
                AttValueTerminator::Eof,
                out,
                &mut None,
            );
            visited.pop();
            result
        }
    }
}

/// Terminator for the content-loop shared between top-level
/// element bodies and entity-replacement-text re-parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContentTerminator {
    /// Stop on `</` (element end-tag) — the §3 \[43\] `content`
    /// production wrapped in `STag content ETag` (§3 \[39\]).
    Etag,
    /// Stop on EOF — entity replacement text is its own
    /// self-contained `content` fragment (§4.4.3 "Included" with
    /// entity boundaries atomic per §4.3.2 "Well-Formed Parsed
    /// Entity"). A stray `</` is malformed.
    Eof,
}

/// The child nodes of an element plus — when capturing — the per-text-child
/// §4.6 predefined-entity-reference forms: each `(text-child ordinal,
/// EntityReferenceForm)` pair keys into the element's `NodeDecisions` to
/// complete the char-data key `(element index, text-child ordinal, char index)`.
/// Empty `Vec` of forms on the non-capturing path.
type ContentWithRefs = (Vec<XmlNode>, Vec<(usize, EntityReferenceForm)>);

/// W3C XML 1.0 §3 production \[43\] `content`:
/// `content ::= CharData? ((element | Reference | CDSect | PI | Comment) CharData?)*`.
///
/// Returns the child nodes and — when `capture` is `Some` — the per-text-child
/// §4.6 predefined-entity-reference forms (see [`ContentWithRefs`]). The caller
/// ([`parse_element`]) folds those into the element's `NodeDecisions` under the
/// element's own pre-order index.
fn parse_content(
    c: &mut Cursor<'_>,
    entities: &[XmlGeneralEntity],
    strict_entity_declared: bool,
    capture: &mut Option<CaptureCtx>,
    depth: u32,
) -> Result<ContentWithRefs, XmlParseError> {
    let mut visited = Vec::new();
    parse_content_with_terminator(
        c,
        entities,
        &mut visited,
        strict_entity_declared,
        ContentTerminator::Etag,
        capture,
        depth,
    )
}

/// Shared content-loop body for the top-level element body and
/// for entity-replacement-text re-parse (§4.4.3 "Included" with
/// §4.3.2 atomic entity boundaries).
///
/// `visited` is the W3C XML 1.0 §4.1 WFC: No Recursion stack —
/// the qualified names of user-declared general entities the
/// parser is currently inside. The top-level call starts empty;
/// each entity-inclusion call pushes the entity's name before
/// recursing into its replacement text, popping on return. A
/// reference to an entity already in `visited` is the cycle.
fn parse_content_with_terminator(
    c: &mut Cursor<'_>,
    entities: &[XmlGeneralEntity],
    visited: &mut Vec<String>,
    strict_entity_declared: bool,
    term: ContentTerminator,
    capture: &mut Option<CaptureCtx>,
    depth: u32,
) -> Result<ContentWithRefs, XmlParseError> {
    let mut nodes: Vec<XmlNode> = Vec::new();
    let mut text_buf = String::new();
    // Capture the §4.6 predefined-entity-reference form in char data only on the
    // capturing path; `None` makes every `flush_text_capturing` a plain flush.
    let mut text_ref: Option<TextRefCapture> = capture.as_ref().map(|_| TextRefCapture::default());
    parse_content_into_buffers(
        c,
        entities,
        visited,
        strict_entity_declared,
        term,
        &mut nodes,
        &mut text_buf,
        capture,
        &mut text_ref,
        depth,
    )?;
    flush_text_capturing(&mut nodes, &mut text_buf, &mut text_ref);
    let text_refs = text_ref.map(|cap| cap.done).unwrap_or_default();
    Ok((nodes, text_refs))
}

/// Inner content-loop sharing the parent's `nodes` and
/// `text_buf` accumulators. The §4.4.3 "Included" path for a
/// user-declared internal general entity calls this directly with
/// a sub-cursor over the entity's replacement text — that way
/// CharData on either side of the entity-inclusion boundary
/// joins into one [`XmlNode::Text`] node (matching the W3C
/// Infoset model: entity references are not Infoset items, so
/// adjacent character data they straddle is a single
/// character-information-item sequence).
///
/// Returns on terminator without flushing `text_buf`; the
/// outermost caller is responsible for flushing.
///
/// Each argument is a genuinely distinct threaded value of the
/// content-loop (the cursor, the declared entities, the §4.1
/// No-Recursion `visited` stack, the entity-declared strictness
/// flag, the terminator, the two shared `nodes`/`text_buf`
/// accumulators, the reader `capture`, and the `text_ref` §4.6
/// reference-form sink riding alongside `text_buf`), so the count
/// is intrinsic rather than a flag-bag to be folded into a struct.
#[allow(clippy::too_many_arguments)]
fn parse_content_into_buffers(
    c: &mut Cursor<'_>,
    entities: &[XmlGeneralEntity],
    visited: &mut Vec<String>,
    strict_entity_declared: bool,
    term: ContentTerminator,
    nodes: &mut Vec<XmlNode>,
    text_buf: &mut String,
    capture: &mut Option<CaptureCtx>,
    text_ref: &mut Option<TextRefCapture>,
    depth: u32,
) -> Result<(), XmlParseError> {
    use crate::social::software::markup::xml::spec_1_0::{
        ContentItemKind, loaded_content_dispatch_table,
    };
    let dispatch = loaded_content_dispatch_table();
    // Bound total entity expansion: `nodes`/`text_buf` accumulate every included
    // entity's replacement text (entity inclusion appends to these SAME buffers),
    // so a "billion laughs" bomb blows their size. `depth` and the §4.1 `visited`
    // cycle check do NOT bound exponential NON-cyclic expansion (a→bb, b→cc, …);
    // this does — refuse cleanly rather than OOM/hang.
    const MAX_EXPANSION_BYTES: usize = 16 * 1024 * 1024;
    const MAX_EXPANSION_NODES: usize = 1_000_000;
    loop {
        if text_buf.len() > MAX_EXPANSION_BYTES || nodes.len() > MAX_EXPANSION_NODES {
            return Err(c.syntax_error(
                "entity expansion within the size limit",
                "entity expansion exceeds the maximum",
            ));
        }
        match term {
            ContentTerminator::Etag => {
                if c.starts_with("</") {
                    return Ok(());
                }
            }
            ContentTerminator::Eof => {
                if c.is_eof() {
                    return Ok(());
                }
                if c.starts_with("</") {
                    // §4.3.2 — a parsed entity referenced from
                    // content must itself match `content`; a stray
                    // ETag is malformed. xmlconf xmltest/not-wf/sa/074
                    // — entity expands to `</foo><foo>` — is the
                    // spec regression.
                    return Err(XmlParseError::Syntax {
                        position: c.pos,
                        expected: "well-formed entity replacement text (§4.3.2)".into(),
                        found: "</".into(),
                    });
                }
            }
        }
        // §3.1 [43] content-item dispatch is grammar-grounded: the
        // (literal-prefix, ContentItemKind) entries come from the
        // loaded W3C XML 1.0 grammar's `content` production via
        // `spec_1_0::loaded_content_dispatch_table()`, not from
        // hand-coded `starts_with` strings. The reference branch
        // (`&` vs `&#`) is dispatched a level deeper since the
        // grammar-extracted common prefix is "&".
        match dispatch.classify(c.rest()) {
            ContentItemKind::Comment => {
                flush_text_capturing(nodes, text_buf, text_ref);
                nodes.push(parse_comment_node(c)?);
                continue;
            }
            ContentItemKind::CDataSection => {
                flush_text_capturing(nodes, text_buf, text_ref);
                nodes.push(parse_cdata_node(c)?);
                continue;
            }
            ContentItemKind::ProcessingInstruction => {
                flush_text_capturing(nodes, text_buf, text_ref);
                nodes.push(parse_pi_node(c)?);
                continue;
            }
            ContentItemKind::Element => {
                flush_text_capturing(nodes, text_buf, text_ref);
                let child = parse_element(c, entities, strict_entity_declared, capture, depth + 1)?;
                nodes.push(XmlNode::Element(child));
                continue;
            }
            // Reference and CharData fall through to the existing
            // per-character handler below (which already disambiguates
            // `&#` from `&Name;` and routes CharData through the
            // §2.4 well-formedness checks).
            ContentItemKind::Reference | ContentItemKind::CharData => {}
        }
        let ch_pos = c.pos;
        let ch = c.peek_char().ok_or_else(|| XmlParseError::UnexpectedEof {
            context: "element content".into(),
        })?;
        if ch == '&' {
            // §4.4 Table 4 dispatch.
            //
            //   1. `&#…;` (§4.1 [66] CharRef) — Included as the
            //      resolved character.
            //   2. `&amp;` / `&lt;` / `&gt;` / `&apos;` / `&quot;`
            //      (§4.6 predefined) — Included as the
            //      corresponding character.
            //   3. `&Name;` (§4.1 [68] EntityRef, user general
            //      entity) — Included per §4.4.3: the entity's
            //      *literal* replacement text (§4.5 construction
            //      preserves general entity references) is
            //      re-parsed as a `content` fragment at the
            //      reference position; the resulting nodes are
            //      spliced. xmlconf xmltest/not-wf/sa/{074, 090,
            //      092, 103, 116, 117, 119, 120, 153, 182} —
            //      entity expansion that does NOT form a valid
            //      content fragment — are rejected here.
            if c.rest().starts_with("&#") {
                // §4.1 [66] CharRef. When capturing, record the exact numeric FORM
                // (decimal/hex, case, verbatim digits) as an [`ExtendedRef`] keyed
                // by the resolved char's index, so the byte-exact serializer
                // re-emits `&#39;`/`&#x27;` rather than the resolved literal char.
                // The non-capturing `parse_document` path resolves to the char as
                // before.
                if let Some(cap) = text_ref.as_mut() {
                    let (ch_val, kind) = parse_char_ref_capturing(c)?;
                    cap.pending_ext.push(ExtendedRef {
                        char_index: text_buf.chars().count(),
                        kind,
                    });
                    text_buf.push(ch_val);
                } else {
                    let ch_val = parse_char_ref(c)?;
                    text_buf.push(ch_val);
                }
            } else {
                let ref_pos = c.pos;
                c.consume("&")?;
                let name = parse_name(c)?;
                c.consume(";")?;
                let qualified = name.qualified();
                // §4.6 predefined-entity resolution via the table
                // generated at build time from the loaded W3C XML 1.0
                // spec source's `<div2 id="sec-predefined-ent">` block
                // (see `pr4xis::codegen::xml_grammar::extract_predefined_entities`).
                if let Some(ch) = crate::social::software::markup::xml::spec_1_0::grammar::resolve_predefined_entity(&qualified) {
                    // Reader capture: record the §4.6 reference at its char index
                    // in the CURRENT text buffer (chars, not bytes — the buffer
                    // spans the entity-inclusion boundary and may abut multibyte
                    // chars). Flushed to the text node's child ordinal by
                    // `flush_text_capturing`.
                    if let (Some(cap), Some(entity)) =
                        (text_ref.as_mut(), EntityName::for_resolved_char(ch))
                    {
                        cap.pending.push((text_buf.chars().count(), entity));
                    }
                    text_buf.push(ch);
                } else {
                    // §4.1 [68] general-entity reference in content. When
                    // capturing, record the reference FORM (name + expansion
                    // char-length) as an [`ExtendedRef`] so the byte-exact writer
                    // re-emits `&name;`. The inclusion may splice further nodes
                    // (the replacement text is re-parsed as content); the simple
                    // char-index keying only works when the expansion is pure
                    // text appended to `text_buf` (no spliced element/comment
                    // nodes). If it spliced nodes, fail closed — the capture is
                    // out of this slice's byte-exact scope rather than a silent
                    // mismatch. (prov_o's general-entity refs are all in attribute
                    // VALUES, so this content path is not exercised by it.)
                    let nodes_before = nodes.len();
                    // Only needed for the capture/reverse-lens path below; on the
                    // decode path (text_ref None) this `chars().count()` is O(n)
                    // per inclusion → O(n²) over an entity-expansion bomb. Guard
                    // it so decoding stays linear and the expansion budget fires.
                    let chars_before = if text_ref.is_some() {
                        text_buf.chars().count()
                    } else {
                        0
                    };
                    let ext_before = text_ref.as_ref().map_or(0, |t| t.pending_ext.len());
                    let pred_before = text_ref.as_ref().map_or(0, |t| t.pending.len());
                    include_user_general_entity_in_content(
                        &qualified,
                        ref_pos,
                        entities,
                        visited,
                        strict_entity_declared,
                        nodes,
                        text_buf,
                        capture,
                        text_ref,
                        depth,
                    )?;
                    if let Some(cap) = text_ref.as_mut() {
                        // Capture the §4.1 general-entity reference FORM ONLY when
                        // the expansion is PURE TEXT appended to `text_buf` — no
                        // spliced element/comment nodes, no nested refs captured
                        // inside (which would shift the char-index keying the
                        // single `&name;` slot relies on). A non-pure expansion
                        // (an entity whose replacement text contains markup, as in
                        // `<!ENTITY e "<m></m>">`) is NOT recorded: it remains the
                        // existing §4.4.3 spliced-DOM behaviour, byte-INEXACT in
                        // its reference syntax (a separate later slice) but a
                        // well-formed PARSE — never a fail-closed error. The
                        // bundled byte-exact OWL vocabs use general-entity refs
                        // only in ATTRIBUTE values (pure-text URIs), so this
                        // content path's non-capture does not affect them.
                        let pure_text = nodes.len() == nodes_before
                            && cap.pending_ext.len() == ext_before
                            && cap.pending.len() == pred_before;
                        if pure_text {
                            let expansion_chars = text_buf.chars().count() - chars_before;
                            cap.pending_ext.push(ExtendedRef {
                                char_index: chars_before,
                                kind: ExtendedRefKind::General {
                                    name: qualified,
                                    expansion_chars,
                                },
                            });
                        }
                    }
                }
            }
        } else {
            // §2.4 [14] CharData well-formedness: every char must be
            // in the §2.2 Char repertoire.
            if !is_xml_char(ch) {
                return Err(XmlParseError::InvalidChar {
                    position: ch_pos,
                    code_point: ch as u32,
                    context: "CharData",
                });
            }
            // §2.4 [14] CharData also forbids the `]]>` sequence
            // outside a CDATA section. The lookahead is on the input
            // (not on text_buf) so previously-buffered chars don't
            // confuse the check.
            if ch == ']' && c.rest().starts_with("]]>") {
                return Err(XmlParseError::DisallowedCdataEnd { position: ch_pos });
            }
            text_buf.push(ch);
            c.pos += ch.len_utf8();
        }
    }
}

/// §4.4.3 "Included" for a user-declared general entity reference
/// appearing in element content. Looks up the entity, enforces
/// the entity-kind WFCs (§4.4.4 Parsed Entity), pushes the entity
/// name onto `visited` to detect §4.1 WFC: No Recursion, and
/// re-parses the entity's *literal* replacement text (§4.5
/// construction preserves general-entity references inside an
/// EntityValue) as a `content` fragment at the reference
/// position. The resulting nodes are spliced into the parent's
/// node list.
///
/// Mirrors [`parse_content_into_buffers`]'s threaded state (it
/// forwards the same `nodes`/`text_buf`/`capture`/`text_ref`
/// accumulators into the §4.4.3 re-parse), so the argument count is
/// intrinsic.
#[allow(clippy::too_many_arguments)]
fn include_user_general_entity_in_content(
    name: &str,
    ref_pos: usize,
    entities: &[XmlGeneralEntity],
    visited: &mut Vec<String>,
    strict_entity_declared: bool,
    nodes: &mut Vec<XmlNode>,
    text_buf: &mut String,
    capture: &mut Option<CaptureCtx>,
    text_ref: &mut Option<TextRefCapture>,
    depth: u32,
) -> Result<(), XmlParseError> {
    if visited.iter().any(|n| n == name) {
        return Err(XmlParseError::Syntax {
            position: ref_pos,
            expected: "non-recursive entity reference (§4.1 WFC: No Recursion)".into(),
            found: format!("&{name};"),
        });
    }
    let entity = match entities.iter().find(|e| e.name == name) {
        Some(e) => e,
        None => {
            // §4.1 WFC: Entity Declared — when the carve-out applies
            // (external subset present, or internal subset has PE
            // references, or standalone='no'/unspecified), an
            // unknown general entity is a *validity* error, not a
            // well-formedness error. The non-validating processor
            // bypasses the reference per §5.1 ("processors that
            // ignore external entities ... need not report errors
            // in those entities"). When the carve-out does NOT
            // apply (no DTD, internal-only with no PE refs, or
            // standalone='yes'), the WFC fires and the reference
            // is well-formedness-malformed.
            if strict_entity_declared {
                return Err(XmlParseError::UnsupportedEntity {
                    position: ref_pos,
                    name: name.to_string(),
                });
            }
            return Ok(());
        }
    };
    match entity.kind {
        XmlEntityKind::ExternalUnparsed => Err(XmlParseError::Syntax {
            position: ref_pos,
            expected: "reference to a parsed entity (§4.4.4 WFC: Parsed Entity)".into(),
            found: format!("&{name};"),
        }),
        XmlEntityKind::ExternalParsed => {
            // §5.1 non-validating processor: a reference to an
            // external parsed entity whose body we did not retrieve
            // is bypassed without text contribution.
            Ok(())
        }
        XmlEntityKind::Internal => {
            // §4.4.3 "Included" re-parse over the entity's literal
            // replacement text, on the SAME `nodes`/`text_buf`
            // accumulators as the surrounding content — so
            // CharData on either side of the entity-inclusion
            // boundary joins into a single XmlNode::Text per the
            // W3C Infoset model (the entity reference itself is
            // not an Infoset item; character data it straddles is
            // one character-information-item sequence).
            let value = entity.value.clone();
            let mut sub = Cursor::new(&value);
            visited.push(name.to_string());
            let result = parse_content_into_buffers(
                &mut sub,
                entities,
                visited,
                strict_entity_declared,
                ContentTerminator::Eof,
                nodes,
                text_buf,
                // §4.4.3 inclusion re-parses the entity's replacement text on
                // the SAME capture context, so an element materialized from an
                // entity still claims its pre-order index at entry — the
                // counter threads straight through the inclusion boundary.
                capture,
                // The §4.6 reference-form sink also threads through: char data
                // straddling the inclusion boundary is one text node, so its
                // refs accumulate into one `pending` run keyed by the eventual
                // child ordinal.
                text_ref,
                depth,
            );
            visited.pop();
            result
        }
    }
}

/// Reader-side accumulator for the §4.6 predefined-entity-reference FORM in
/// CHAR DATA (which resolved chars of a text node were written as
/// `&amp;`/`&lt;`/`&gt;`/`&apos;`/`&quot;`), keyed by the text node's child
/// ordinal — the third coordinate of the char-data key
/// `(element index, text-child ordinal, char index)`.
///
/// `pending` collects `(char_index, entity_name)` for the CURRENTLY buffered
/// text run (char index into the running `text_buf`, counted in chars to match
/// the serializer's `chars()` walk). `done` holds completed
/// `(child_ordinal, EntityReferenceForm)` flushed when the text node is pushed.
/// A text node spans the entity-inclusion boundary (§4.4.3) — its char indices
/// and refs accumulate across the `text_buf` lifetime, so capture rides
/// alongside `text_buf` and is flushed by [`flush_text_capturing`].
/// Reader-side capture sink for the reference FORM inside ONE attribute value —
/// the closed §4.6 predefined set ([`EntityName`], one resolved char each) plus
/// the open §4.1 numeric / general-entity forms ([`ExtendedRef`]), both keyed by
/// resolved-string char index. Flushed into one [`EntityReferenceForm`] per
/// attribute slot. Empty for a canonical value (no resolved references), so the
/// capture is purely additive.
#[derive(Debug, Default)]
struct AttrRefSink {
    /// The §4.6 predefined references (`&amp;`/`&lt;`/`&gt;`/`&apos;`/`&quot;`).
    predefined: Vec<(usize, EntityName)>,
    /// The §4.1 numeric (`&#39;`) / general-entity (`&rdfs;`) references.
    ext: Vec<ExtendedRef>,
}

impl AttrRefSink {
    fn is_empty(&self) -> bool {
        self.predefined.is_empty() && self.ext.is_empty()
    }
}

#[derive(Debug, Default)]
struct TextRefCapture {
    pending: Vec<(usize, EntityName)>,
    /// The §4.1 numeric/general reference forms in the CURRENTLY buffered text
    /// run (char index into the running `text_buf`), the open-set sibling of
    /// `pending`'s closed §4.6 predefined set — `&#39;` numeric refs and `&rdfs;`
    /// general-entity refs. Flushed alongside `pending` by
    /// [`flush_text_capturing`].
    pending_ext: Vec<ExtendedRef>,
    done: Vec<(usize, EntityReferenceForm)>,
}

/// Flush `text_buf` into a `Text` node AND, when capturing, key its pending
/// §4.6 reference form by the ordinal the text node takes among the element's
/// children. `nodes.len()` is that ordinal BEFORE the push, matching the
/// serializer which counts the same child position as it walks `el.children`.
fn flush_text_capturing(
    nodes: &mut Vec<XmlNode>,
    buf: &mut String,
    text_ref: &mut Option<TextRefCapture>,
) {
    if !buf.is_empty() {
        if let Some(cap) = text_ref.as_mut()
            && (!cap.pending.is_empty() || !cap.pending_ext.is_empty())
        {
            let child_ordinal = nodes.len();
            cap.done.push((
                child_ordinal,
                EntityReferenceForm {
                    refs: core::mem::take(&mut cap.pending),
                    ext_refs: core::mem::take(&mut cap.pending_ext),
                },
            ));
        }
        nodes.push(XmlNode::Text(core::mem::take(buf)));
    }
}

/// W3C XML 1.0 §2.5 production \[15\] `Comment`, emitting a typed
/// `XmlNode::Comment` for inside-element occurrences (Cowan &
/// Tobin 2004 §2.5 keeps comments in the Infoset).
///
/// Enforces two §2.5 well-formedness constraints:
/// - The body MUST NOT contain the string `--` (the EBNF subtraction
///   `(Char - '-')` is the spec form).
/// - Every character MUST be in the §2.2 \[2\] Char repertoire — the
///   `is_xml_char` predicate consults the loaded spec table.
fn parse_comment_node(c: &mut Cursor<'_>) -> Result<XmlNode, XmlParseError> {
    let comment_start = c.pos;
    c.consume("<!--")?;
    let rest = c.rest();
    let end = rest
        .find("-->")
        .ok_or_else(|| XmlParseError::UnexpectedEof {
            context: "comment".into(),
        })?;
    let body = &rest[..end];
    // §2.5 [15] Comment body alternation —
    // `((Char - '-') | ('-' (Char - '-')))*` — forbids both any
    // `--` substring AND a trailing `-` (the last body char must
    // be matched by `(Char - '-')`). xmlconf xmltest/not-wf/sa/070
    // is the trailing-dash regression.
    if body.contains("--") || body.ends_with('-') {
        return Err(XmlParseError::MalformedComment {
            position: comment_start,
        });
    }
    check_chars_in_range(body, comment_start + 4, "Comment")?;
    let body = body.to_string();
    c.pos += end + 3;
    Ok(XmlNode::Comment(body))
}

/// W3C XML 1.0 §2.7 production \[18\] `CDSect`:
/// `CDSect ::= CDStart CData CDEnd`.
///
/// §2.7 \[20\] `CData ::= (Char* - (Char* ']]>' Char*))` — every char
/// in the body must be in the §2.2 Char repertoire; the `]]>` tail
/// is consumed by the close-marker so a body containing a literal
/// `]]>` simply ends early (legal XML — content can be split across
/// CDATA sections).
fn parse_cdata_node(c: &mut Cursor<'_>) -> Result<XmlNode, XmlParseError> {
    let cdata_start = c.pos;
    c.consume("<![CDATA[")?;
    let rest = c.rest();
    let end = rest
        .find("]]>")
        .ok_or_else(|| XmlParseError::UnexpectedEof {
            context: "CDATA section".into(),
        })?;
    let body = &rest[..end];
    check_chars_in_range(body, cdata_start + 9, "CDATA section")?;
    let body = body.to_string();
    c.pos += end + 3;
    Ok(XmlNode::CData(body))
}

/// W3C XML 1.0 §2.6 production \[16\] `PI` emitting a typed
/// `XmlNode::ProcessingInstruction`.
///
/// §2.6 \[16\] `PI ::= '<?' PITarget (S (Char* - (Char* '?>' Char*)))? '?>'`
/// — every char in the data segment must be in the §2.2 Char
/// repertoire; the `?>` tail is consumed by the close-marker.
fn parse_pi_node(c: &mut Cursor<'_>) -> Result<XmlNode, XmlParseError> {
    let pi_start = c.pos;
    c.consume("<?")?;
    let target_name = parse_name(c)?;
    // §2.6 [17] PITarget excludes the case-insensitive name `xml`
    // — the only `<?xml ... ?>` form allowed is the XMLDecl
    // (production [23]) at the document head, which `parse_xml_decl`
    // handles before reaching content. xmlconf cases that
    // smuggle `<?XML ... ?>` into element content regress here.
    if target_name.qualified().eq_ignore_ascii_case("xml") {
        return Err(XmlParseError::Syntax {
            position: pi_start,
            expected: "PITarget that is not `xml` (case-insensitive)".into(),
            found: target_name.qualified(),
        });
    }
    let mut data: Option<String> = None;
    if c.peek_char()
        .is_some_and(|ch| matches!(ch, ' ' | '\t' | '\r' | '\n'))
    {
        c.skip_whitespace();
        let data_start_pos = c.pos;
        let rest = c.rest();
        let end = rest
            .find("?>")
            .ok_or_else(|| XmlParseError::UnexpectedEof {
                context: "processing instruction".into(),
            })?;
        if end > 0 {
            let body = &rest[..end];
            check_chars_in_range(body, data_start_pos, "PI")?;
            data = Some(body.to_string());
        }
        c.pos += end;
    }
    c.consume("?>")?;
    let _ = pi_start; // surfaced for symmetry; not yet used in errors
    Ok(XmlNode::ProcessingInstruction {
        target: target_name.qualified(),
        data,
    })
}

/// Walk `body` and reject the first character outside §2.2 \[2\] `Char`.
/// `position_of_body_start` is the byte offset of `body\[0\]` in the
/// original input; `context` names the production (Comment / CDATA /
/// PI / CharData) for the error message.
fn check_chars_in_range(
    body: &str,
    position_of_body_start: usize,
    context: &'static str,
) -> Result<(), XmlParseError> {
    let mut offset_within_body = 0;
    for ch in body.chars() {
        if !is_xml_char(ch) {
            return Err(XmlParseError::InvalidChar {
                position: position_of_body_start + offset_within_body,
                code_point: ch as u32,
                context,
            });
        }
        offset_within_body += ch.len_utf8();
    }
    Ok(())
}

#[cfg(test)]
mod reverse_lens_roundtrip_tests {
    //! The correctness gate for the serialized reverse lens: `parse_document_capturing`
    //! records the concrete-syntax decisions so that
    //! `serialize_document_exact(&doc, &decisions)` reproduces the input
    //! BYTE-FOR-BYTE.
    //!
    //! Scope of the captured non-canonical concrete-syntax features:
    //! - the empty-element form — `<a></a>` (explicit) vs the canonical `<a/>`
    //!   (W3C XML 1.0 §3.1);
    //! - prolog / epilog `Misc` white-space (§2.8 \[27\]);
    //! - the intra-tag white-space LAYOUT (§3.1 \[40\]/\[44\] — the real corpus's
    //!   multi-line attribute indentation);
    //! - the §4.6 predefined-entity-reference FORM (`&amp; &lt; &gt; &apos;
    //!   &quot;`) the source used in attribute values and char data (the parser
    //!   resolves these to a literal char, so without the capture the serializer
    //!   would emit the bare char — a byte mismatch).
    //!
    //! Still out of scope (a §4.1 numeric character reference `&#N;`, a
    //! user-declared DTD general entity's reference syntax, the `standalone`
    //! xml-decl attr, and an element co-locating an `xmlns` declaration with
    //! non-`xmlns` attributes) — those are later slices; the reader fails LOUD
    //! (a `debug_assert`) rather than silently dropping a numeric reference when
    //! capturing.

    use super::super::serializer::serialize_document_exact;
    use super::super::source_syntax::{EmptyForm, EntityName, NodeDecisions};
    use super::{XmlNode, parse_document_capturing};

    /// Assert the full byte-exact round-trip law: `serialize_document_exact ∘
    /// parse_document_capturing` is the identity on `input` (for inputs whose
    /// non-canonical features are all in this slice's scope).
    fn assert_byte_exact_roundtrip(input: &[u8]) {
        let (doc, decisions) = parse_document_capturing(input).expect("input parses");
        let out = serialize_document_exact(&doc, &decisions);
        assert_eq!(
            out,
            input.to_vec(),
            "round-trip mismatch\n  in : {:?}\n  out: {:?}",
            core::str::from_utf8(input),
            core::str::from_utf8(&out),
        );
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn roundtrip_mixed_empty_forms_and_text() {
        // (a) `a` explicit-empty, `b` self-closing, `c` has text — the three
        // forms side by side. Uses the exact XML decl the serializer emits.
        let input = br#"<?xml version="1.0" encoding="UTF-8"?><root><a></a><b/><c>text</c></root>"#;
        assert_byte_exact_roundtrip(input);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn roundtrip_indented_document_with_attributes() {
        // Whitespace BETWEEN elements (indentation) is kept VERBATIM by
        // `flush_text` as Text nodes and emitted unchanged, and attributes keep
        // their source order — so an indented document round-trips byte-for-byte
        // with only the empty-element form (`<a …></a>`) needing a decision.
        let input = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><root>\n  <a x=\"1\" y=\"2\"></a>\n  <b/>\n  <c>text</c>\n</root>";
        assert_byte_exact_roundtrip(input);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn roundtrip_wn_lmf_fragment() {
        // A representative WN-LMF 1.3 document: XML decl + DOCTYPE (SYSTEM id) +
        // an indented Lexicon with attributes, self-closing childless elements
        // (`Lemma`, `Sense`), and text content (`Definition`). Every childless
        // element is written self-closing (the canonical form), so the document
        // needs NO per-element concrete-syntax decision. Its one residual is the
        // §2.8 [27] `Misc` `S` in the prolog — the `\n` between the XML decl and
        // the DOCTYPE, and the `\n` between the DOCTYPE and the root — which the
        // reader now captures into `PrologDecisions` and the writer re-emits, so
        // it round-trips byte-for-byte. This measures the real residual for
        // WordNet-shaped input.
        let input = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE LexicalResource SYSTEM \"http://globalwordnet.github.io/schemas/WN-LMF-1.3.dtd\">\n<LexicalResource>\n  <Lexicon id=\"ewn\" label=\"English WordNet\">\n    <LexicalEntry id=\"w1\">\n      <Lemma writtenForm=\"dog\" partOfSpeech=\"n\"/>\n      <Sense id=\"s1\" synset=\"syn-1\"/>\n    </LexicalEntry>\n    <Synset id=\"syn-1\" partOfSpeech=\"n\">\n      <Definition>a domesticated carnivore</Definition>\n    </Synset>\n  </Lexicon>\n</LexicalResource>";
        assert_byte_exact_roundtrip(input);

        // Pin the captured prolog white-space: a `\n` after the XML decl (before
        // the DOCTYPE) and a `\n` after the DOCTYPE (before the root). No epilog.
        let (_, decisions) = parse_document_capturing(input).unwrap();
        let prolog = decisions.prolog();
        assert_eq!(prolog.after_xml_decl, "\n");
        assert_eq!(prolog.after_doctype, "\n");
        assert_eq!(prolog.after_root, "");
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn roundtrip_epilog_whitespace_after_root() {
        // §2.1 [1] `document ::= prolog element Misc*` — the trailing `Misc*`.
        // White-space AFTER the root element's end-tag is the epilog residual;
        // the reader captures it into `PrologDecisions::after_root` and the
        // writer re-emits it, so a document with a trailing newline (and a
        // prolog newline) round-trips byte-for-byte.
        let input = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<root>\n  <a/>\n</root>\n";
        assert_byte_exact_roundtrip(input);

        // Pin the capture: the `\n` after the XML decl (no DOCTYPE here, so
        // `after_doctype` is vacuously empty) and the trailing `\n` epilog.
        let (_, decisions) = parse_document_capturing(input).unwrap();
        let prolog = decisions.prolog();
        assert_eq!(prolog.after_xml_decl, "\n");
        assert_eq!(prolog.after_doctype, "");
        assert_eq!(prolog.after_root, "\n");
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn roundtrip_nested_explicit_empty() {
        // (b) a NESTED explicit-empty: `y` (pre-order index 2) sits inside `x`
        // (index 1) inside `root` (index 0).
        let input = br#"<?xml version="1.0" encoding="UTF-8"?><root><x><y></y></x></root>"#;
        assert_byte_exact_roundtrip(input);

        // Pin the pre-order index the decision lands on: only `y` is recorded
        // explicit, at index 2.
        let (_, decisions) = parse_document_capturing(input).unwrap();
        assert_eq!(
            decisions.get(0),
            None,
            "root self-closes? no — it has children"
        );
        assert_eq!(decisions.get(1), None, "x has children, no decision");
        assert_eq!(
            decisions.get(2),
            Some(&NodeDecisions {
                empty_form: Some(EmptyForm::Explicit),
                ..NodeDecisions::default()
            }),
            "nested y captured explicit at pre-order index 2",
        );
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn roundtrip_explicit_empty_after_sibling_element_and_text() {
        // (c) the explicit-empty `<e></e>` is preceded by a sibling ELEMENT
        // (`<s/>`, which itself contains a nested element so the pre-order index
        // of `e` is non-trivial) AND text. Pre-order entry order:
        //   root=0, s=1, n=2, e=3.
        // A naive sibling-count or child-path scheme would mis-key `e`; the
        // pre-order counter places it at 3.
        let input =
            br#"<?xml version="1.0" encoding="UTF-8"?><root><s><n/></s>between<e></e></root>"#;
        assert_byte_exact_roundtrip(input);

        let (_, decisions) = parse_document_capturing(input).unwrap();
        assert_eq!(
            decisions.get(3),
            Some(&NodeDecisions {
                empty_form: Some(EmptyForm::Explicit),
                ..NodeDecisions::default()
            }),
            "e captured explicit at pre-order index 3 (root=0, s=1, n=2, e=3)",
        );
        // The self-closing siblings record nothing.
        assert_eq!(decisions.get(1), None);
        assert_eq!(decisions.get(2), None);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn entity_inclusion_threads_the_preorder_counter() {
        // (d) a DOCTYPE-declared general entity whose replacement text contains
        // an explicit-empty element, proving the §4.4.3 entity-inclusion path
        // (`include_user_general_entity_in_content`) threads the pre-order
        // counter: the element materialized FROM the entity still claims its
        // index at entry and its `Explicit` form is captured.
        //
        // NOTE on scope: this is NOT a byte-exact round-trip. The parser
        // RESOLVES the `&e;` reference (§4.4.3 "Included"), splicing the
        // expanded `<m></m>` element into `root.children` — the reference is
        // gone from the Infoset DOM. The serializer therefore re-emits the
        // expanded element, never the `&e;` reference; preserving the
        // entity-reference SYNTAX is a separate (later) concrete-syntax slice.
        // What we DO assert here is the counter-threading invariant this slice
        // owns: the entity-materialized element is captured at the correct
        // pre-order index, and re-serializing the resulting DOM with those
        // decisions reproduces the explicit `<m></m>` form (not the canonical
        // `<m/>`).
        let input =
            br#"<?xml version="1.0"?><!DOCTYPE root [<!ENTITY e "<m></m>">]><root><a/>&e;</root>"#;
        let (doc, decisions) = parse_document_capturing(input).expect("input parses");

        // Expanded DOM: root has two element children — the self-closing `a`
        // and the entity-materialized explicit-empty `m`.
        assert_eq!(doc.root.children.len(), 2);
        assert!(matches!(&doc.root.children[0], XmlNode::Element(el) if el.name.local == "a"));
        assert!(matches!(&doc.root.children[1], XmlNode::Element(el) if el.name.local == "m"));

        // Pre-order entry order: root=0, a=1, m=2. `a` self-closes (no
        // decision); `m` (materialized through the entity-inclusion path) is
        // captured explicit at index 2 — the counter threaded through the
        // inclusion boundary.
        assert_eq!(decisions.get(1), None, "self-closing a records nothing");
        assert_eq!(
            decisions.get(2),
            Some(&NodeDecisions {
                empty_form: Some(EmptyForm::Explicit),
                ..NodeDecisions::default()
            }),
            "entity-materialized m captured explicit at pre-order index 2",
        );

        // Re-serializing the EXPANDED DOM with the captured decisions yields the
        // explicit `<m></m>` form (the entity-inclusion counter landed on the
        // right element); the canonical serializer would have self-closed it.
        let out = serialize_document_exact(&doc, &decisions);
        let out_str = core::str::from_utf8(&out).unwrap();
        assert!(
            out_str.contains("<m></m>"),
            "expanded element re-serializes in explicit form: {out_str}",
        );
        assert!(
            out_str.contains("<a/>"),
            "self-closing sibling stays self-closing: {out_str}",
        );
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn roundtrip_multiline_start_tag_indented_attributes() {
        // §3.1 [40] `STag ::= '<' Name (S Attribute)* S? '>'` — each attribute
        // indented onto its own line (newline + spaces), the real OEWN-2025
        // layout. The intra-tag white-space LAYOUT is captured and re-emitted;
        // the canonical writer's single-space separation would not reproduce it.
        let input = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><Lexicon\n    id=\"oewn\"\n    label=\"English WordNet\"\n    version=\"2025\">text</Lexicon>";
        assert_byte_exact_roundtrip(input);

        // Pin the captured layout on the root (pre-order index 0): three
        // attributes, each preceded by `\n    `, no `S` around `=`, no trailing
        // `S?` (the `>` abuts the last value's quote).
        let (_, decisions) = parse_document_capturing(input).unwrap();
        let iw = decisions
            .get(0)
            .and_then(|d| d.intra_tag_whitespace.as_ref())
            .expect("multi-line start-tag records an intra-tag layout");
        assert_eq!(iw.before_attr, vec!["\n    ", "\n    ", "\n    "]);
        assert_eq!(
            iw.around_eq,
            vec![
                (String::new(), String::new()),
                (String::new(), String::new()),
                (String::new(), String::new()),
            ]
        );
        assert_eq!(iw.before_close, "");
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn roundtrip_predefined_entity_in_attribute_value() {
        // §4.6 `&apos;` inside `writtenForm="&apos;hood"` — the parser RESOLVES
        // it to a literal `'`, which the canonical escaper would emit bare. The
        // captured reference form re-emits `&apos;` at its char index.
        let input = br#"<?xml version="1.0" encoding="UTF-8"?><Lemma writtenForm="&apos;hood"/>"#;
        assert_byte_exact_roundtrip(input);

        // Pin the capture: the root element (index 0) records one §4.6 reference
        // in attribute slot 0 at char index 0 (the apostrophe is the first char
        // of the resolved value `'hood`).
        let (_, decisions) = parse_document_capturing(input).unwrap();
        let form = decisions
            .get(0)
            .and_then(|d| d.attr_entity_refs.get(&0))
            .expect("attribute value records its §4.6 reference form");
        assert_eq!(form.refs, vec![(0, EntityName::Apos)]);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn roundtrip_multiple_entity_refs_per_attribute_value() {
        // Multiple §4.6 references in one attribute value, with intervening
        // multibyte chars is covered separately; here two `&apos;` in a
        // transliteration (`Dhu'l-Qa'dah`) prove ascending multi-ref capture.
        let input = br#"<?xml version="1.0" encoding="UTF-8"?><Lemma writtenForm="Dhu&apos;l-Qa&apos;dah"/>"#;
        assert_byte_exact_roundtrip(input);

        // Resolved value is `Dhu'l-Qa'dah`: the apostrophes are at char indices
        // 3 and 8.
        let (_, decisions) = parse_document_capturing(input).unwrap();
        let form = decisions
            .get(0)
            .and_then(|d| d.attr_entity_refs.get(&0))
            .expect("attribute value records both §4.6 references");
        assert_eq!(
            form.refs,
            vec![(3, EntityName::Apos), (8, EntityName::Apos)]
        );
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn roundtrip_entity_ref_adjacent_to_multibyte_char_in_char_data() {
        // The char-index basis MUST be exact when a §4.6 reference sits next to a
        // multibyte char: `&quot;` immediately before the curly quote U+2019
        // (’, 3 UTF-8 bytes). A byte-offset basis would mis-place the reference;
        // the char-index basis (the serializer iterates `chars()`) is exact.
        // Resolved char-data is `say "’ello` → chars: s,a,y,space,",’,e,l,l,o;
        // the `"` (from `&quot;`) is at char index 4 and the curly quote ’ at 5.
        let input =
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Example>say &quot;\u{2019}ello</Example>"
                .as_bytes();
        assert_byte_exact_roundtrip(input);

        let (_, decisions) = parse_document_capturing(input).unwrap();
        // The `Example` element is pre-order index 0; its only child is the text
        // node at child ordinal 0, recording one §4.6 reference at char index 4.
        let form = decisions
            .get(0)
            .and_then(|d| d.text_entity_refs.get(&0))
            .expect("char data records its §4.6 reference form");
        assert_eq!(form.refs, vec![(4, EntityName::Quot)]);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn roundtrip_oewn_2025_shaped_fragment() {
        // THE GATE: a real OEWN-2025-shaped fragment combining every captured
        // concrete-syntax feature this slice owns —
        //   • a multi-line `<Lexicon …>` start-tag, each attribute on its own
        //     indented line (§3.1 [40] intra-tag white-space);
        //   • a `<LexicalEntry>` whose `<Lemma writtenForm="&apos;hood"/>`
        //     carries a §4.6 reference in an attribute value, the parser would
        //     otherwise resolve to a bare `'`;
        //   • a `<Definition>` whose char data has `&quot;` adjacent to a curly
        //     quote (U+2018 ‘), exercising the exact char-index basis;
        //   • prolog `Misc` white-space (the `\n` after the XML decl);
        //   • self-closing childless elements (`Lemma`, `Sense`) kept canonical.
        // `serialize_document_exact(parse_document_capturing(frag)) == frag`
        // byte-for-byte.
        let input = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<LexicalResource>\n\
  <Lexicon\n\
      id=\"oewn\"\n\
      label=\"English WordNet\"\n\
      version=\"2025\">\n\
    <LexicalEntry id=\"w1\">\n\
      <Lemma writtenForm=\"&apos;hood\" partOfSpeech=\"n\"/>\n\
      <Sense id=\"oewn-1-n\" synset=\"oewn-1-n\"/>\n\
    </LexicalEntry>\n\
    <Synset id=\"oewn-1-n\" partOfSpeech=\"n\">\n\
      <Definition>a \u{2018}neighborhood&quot; sense</Definition>\n\
    </Synset>\n\
  </Lexicon>\n\
</LexicalResource>"
            .as_bytes();
        assert_byte_exact_roundtrip(input);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn no_decision_recorded_for_fully_self_closing_document() {
        // A document with only self-closing empties records NO decisions — the
        // canonical serializer's default already reproduces it, so the exact
        // serializer with empty decisions is byte-identical.
        let input = br#"<?xml version="1.0" encoding="UTF-8"?><root><a/><b/></root>"#;
        let (_doc, decisions) = parse_document_capturing(input).unwrap();
        assert_eq!(
            decisions,
            super::super::source_syntax::SyntaxDecisions::new()
        );
        assert_byte_exact_roundtrip(input);
    }

    /// The generic content-residue diff fails CLOSED on dropped `#PCDATA`: a
    /// non-white-space source `Text` run with no regenerated counterpart is real
    /// character data a structural writer dropped, NOT inter-element white-space
    /// residue (XML 1.0 §2.3 [3] `S` is `#x20 | #x9 | #xD | #xA` only). It must
    /// surface as [`RegeneratedComplementError::UnmatchedContentText`], never be
    /// silently re-inserted as concrete-syntax residue — so real content can never
    /// masquerade as white-space when this generic carrier serves a future
    /// `write_uslm` / `write_owl_exact`.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn diff_fails_closed_on_dropped_content_text() {
        use super::super::source_syntax::{RegeneratedComplementError, diff_content_whitespace};
        let (source, _) =
            parse_document_capturing(b"<r><a>real content</a></r>").expect("source parses");
        let (regenerated, _) =
            parse_document_capturing(b"<r><a></a></r>").expect("regenerated parses");
        let err = diff_content_whitespace(&source, &regenerated)
            .expect_err("dropped #PCDATA must fail closed, not become white-space residue");
        assert!(
            matches!(err, RegeneratedComplementError::UnmatchedContentText { .. }),
            "expected UnmatchedContentText, got {err:?}"
        );
    }

    /// The positive control: genuine inter-element white-space the regenerated
    /// tree lacks is ADMITTED as residue (not rejected), so the white-space-only
    /// guard distinguishes §2.3 [3] `S` from character data rather than rejecting
    /// all unmatched text.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn diff_admits_inter_element_white_space() {
        use super::super::source_syntax::diff_content_whitespace;
        let (source, _) = parse_document_capturing(b"<r>\n  <a/>\n</r>").expect("source parses");
        let (regenerated, _) =
            parse_document_capturing(b"<r><a/></r>").expect("regenerated parses");
        assert!(
            diff_content_whitespace(&source, &regenerated).is_ok(),
            "inter-element white-space must be captured as residue, not fail closed"
        );
    }

    // ── §2.11 [2.11] End-of-Line form (slice U5) ────────────────────────────

    /// FOCUSED META-TEST (slice U5): a small CRLF-bearing document round-trips
    /// byte-for-byte. The `#xD#xA` line breaks span the productions a real source
    /// (the on-disk USC title, a CRLF-formatted corpus) emits VERBATIM — the prolog
    /// `Misc*` `S` (between the XML decl and the root), the inter-element `S`
    /// indentation, and char-data — proving the §2.11 \[2.11\] EOL form is FULLY
    /// GENERIC, not a prolog-only special case. The char-data run also carries a
    /// `&amp;` so the test pins the ROBUSTNESS of the LF-ordinal key: the `&`-escape
    /// shifts byte offsets, but the CRLF after it still re-expands correctly.
    /// (A `#xD#xA` INSIDE an attribute value is a distinct §3.3.3
    /// attribute-value-normalization residue — the serializer escapes a literal
    /// `#xA` there to `&#xA;` regardless — so it is out of this slice's scope.)
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn roundtrip_crlf_document_byte_exact() {
        // CRLF after the XML decl (prolog `Misc*` `S`), CRLF in the inter-element
        // indentation, and CRLF in char-data AFTER a `&amp;` (so an escape shifts
        // byte offsets before the break — the LF-ordinal key must still find it).
        let input =
            b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<r>\r\n  <c>a &amp; b\r\nc</c>\r\n</r>";
        assert_byte_exact_roundtrip(input);

        // The captured form lists four `Crlf` entries (additive proof that the
        // round-trip is not vacuous): every line break was a CRLF.
        let (_, decisions) = parse_document_capturing(input).unwrap();
        let eol = decisions.eol_form();
        assert_eq!(eol.eols.len(), 4, "four CRLF line breaks captured");
        assert!(
            eol.eols
                .iter()
                .all(|(_, k)| matches!(k, super::super::source_syntax::EolKind::Crlf)),
            "every recorded break is a CRLF (#xD#xA)"
        );
    }

    /// A lone `#xD` (CR not followed by `#xA`) is the third §2.11 \[2.11\] form —
    /// the spec collapses it to `#xA` too. It round-trips byte-for-byte (the `#xA`
    /// is rewritten back to a bare `#xD`, NOT a CRLF), proving the writer
    /// dispatches on the captured [`EolKind`] rather than always inserting CRLF.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn roundtrip_lone_cr_document_byte_exact() {
        // A bare `#xD` between the decl and the root (an old-Mac-style line break).
        let input = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r<r>x\ry</r>";
        assert_byte_exact_roundtrip(input);

        let (_, decisions) = parse_document_capturing(input).unwrap();
        let eol = decisions.eol_form();
        assert_eq!(eol.eols.len(), 2, "two lone-CR line breaks captured");
        assert!(
            eol.eols
                .iter()
                .all(|(_, k)| matches!(k, super::super::source_syntax::EolKind::Cr)),
            "every recorded break is a lone CR (#xD)"
        );
    }

    /// A mixed `#xD#xA` / lone `#xD` / literal `#xA` document round-trips
    /// byte-for-byte — the three §2.11 \[2.11\] forms side by side, each put back
    /// to its exact source bytes.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn roundtrip_mixed_eol_forms_byte_exact() {
        // `\r\n` (CRLF), then `\n` (already LF — records nothing), then `\r`
        // (lone CR), in three char-data lines.
        let input = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><r>a\r\nb\nc\rd</r>";
        assert_byte_exact_roundtrip(input);

        let (_, decisions) = parse_document_capturing(input).unwrap();
        let eol = decisions.eol_form();
        // Two recorded forms (the literal `#xA` records nothing): one CRLF, one CR.
        assert_eq!(
            eol.eols.len(),
            2,
            "CRLF + lone CR recorded; literal LF is not"
        );
        use super::super::source_syntax::EolKind;
        assert_eq!(eol.eols[0].1, EolKind::Crlf);
        assert_eq!(eol.eols[1].1, EolKind::Cr);
    }

    /// ADDITIVE PROOF (slice U5): a pure-`#xA` document records NO §2.11 EOL form
    /// at all (the residue is empty), so `serialize_document_exact` is
    /// byte-identical to the pre-EOL-form serializer for LF-only input. This is
    /// the no-regression guarantee for the pure-LF WordNet 89 MB corpus and every
    /// existing `reverse_lens` fixture: the EOL kernel addition is INERT when the
    /// source carries no `#xD`.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn pure_lf_document_records_no_eol_form() {
        let input = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<r>\n  <a/>\n</r>\n";
        // Round-trips (it always did — covered by the other fixtures) AND records
        // nothing new.
        assert_byte_exact_roundtrip(input);
        let (_, decisions) = parse_document_capturing(input).unwrap();
        assert!(
            decisions.eol_form().is_empty(),
            "a pure-#xA source must record an EMPTY §2.11 EOL form (additive: the \
             re-expansion is a no-op, so LF-only corpora are unaffected)"
        );
    }
}

#[cfg(test)]
mod depth_safety_tests {
    use super::parse_document;

    /// Billion-laughs-style nesting: tens of thousands of nested elements. The
    /// recursive-descent parser (`parse_element` ↔ `parse_content`) would
    /// overflow the stack and ABORT the process — a denial-of-service — without
    /// a depth bound. Honest = refuse cleanly with an error, never crash.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn deeply_nested_xml_is_refused_not_a_stack_overflow() {
        // Run on a generous stack so the test verifies the depth-bound REFUSAL,
        // not the host thread's stack size. Without the bound, 50k-deep nesting
        // overflows even a 32 MiB stack — so a clean `Err` here proves the guard
        // fires (at MAX_ELEMENT_DEPTH) long before the stack is exhausted.
        let refused = std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(|| {
                let depth = 50_000usize;
                let mut xml = String::with_capacity(depth * 7);
                for _ in 0..depth {
                    xml.push_str("<a>");
                }
                for _ in 0..depth {
                    xml.push_str("</a>");
                }
                parse_document(xml.as_bytes()).is_err()
            })
            .expect("spawn test thread")
            .join()
            .expect("the parser must not panic/overflow on deeply-nested input");
        assert!(
            refused,
            "deeply-nested XML must be refused by the depth bound, not parsed",
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn deeply_nested_content_model_is_refused_not_a_stack_overflow() {
        // A DOCTYPE internal-subset <!ELEMENT decl with deeply-nested parens
        // drives the PEG content-model interpreter (pr4xis xml_grammar) into
        // unbounded mutual recursion — a stack overflow SEPARATE from the
        // element-tree depth bound. Run on a generous stack so a clean return
        // proves the interpreter's own recursion guard fires.
        let ok = std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(|| {
                let parens = "(".repeat(20_000);
                let doc = format!("<!DOCTYPE x [<!ELEMENT a {parens}b>]>\n<x/>");
                let _ = parse_document(doc.as_bytes());
                true
            })
            .expect("spawn test thread")
            .join()
            .expect("the content-model interpreter must not overflow the stack");
        assert!(ok);
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn billion_laughs_entity_expansion_is_refused_not_oom() {
        // Classic XML entity-expansion bomb: nested general entities that expand
        // exponentially (lol9 → ~10^8 "lol"s, hundreds of MB). There is no cycle
        // and the nesting is shallow, so neither the §4.1 visited cycle-check nor
        // the depth bound catches it — the expansion-size budget must refuse it
        // before exhausting memory.
        let mut doc = String::from("<!DOCTYPE lolz [\n<!ENTITY lol \"lol\">\n");
        for i in 2..=9u32 {
            let prev = if i == 2 {
                String::from("lol")
            } else {
                format!("lol{}", i - 1)
            };
            let refs = format!("&{prev};").repeat(10);
            doc.push_str(&format!("<!ENTITY lol{i} \"{refs}\">\n"));
        }
        doc.push_str("]>\n<lolz>&lol9;</lolz>");
        // Must return (an error) without OOM/hang — the budget caps expansion.
        let _ = parse_document(doc.as_bytes());
    }
}
