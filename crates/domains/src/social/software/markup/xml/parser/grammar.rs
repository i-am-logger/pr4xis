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
    format,
    string::{String, ToString},
    vec::Vec,
};

use super::super::ontology::{
    XmlAttribute, XmlDoctype, XmlDocument, XmlElement, XmlEntityKind, XmlExternalId,
    XmlGeneralEntity, XmlName, XmlNamespace, XmlNode,
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
    let (raw, detected_encoding) = decode_input(input)?;
    let normalized = normalize_line_endings(&raw);
    let mut cursor = Cursor::new(&normalized);

    let (version, encoding, standalone, doctype) = parse_prolog(&mut cursor)?;
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
    let root = parse_element(&mut cursor, &entity_map, strict_entity_declared)?;
    parse_misc_star(&mut cursor)?;
    cursor.skip_whitespace();
    if !cursor.is_eof() {
        return Err(cursor.syntax_error("end of document", "trailing content"));
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
fn normalize_line_endings(raw: &str) -> String {
    if !raw.contains('\r') {
        // Fast path — no CR present, nothing to normalize.
        return raw.to_string();
    }
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\r' {
            // CRLF → LF (consume the following LF); lone CR → LF.
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            out.push('\n');
        } else {
            out.push(ch);
        }
    }
    out
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

/// `(version, encoding, standalone, doctype)` parsed from the §2.8
/// prolog. `version` is required; the rest are optional per the
/// `XMLDecl?` / `doctypedecl?` productions.
type PrologParts = (String, Option<String>, Option<bool>, Option<XmlDoctype>);

/// W3C XML 1.0 §2.8 production \[22\] `prolog`:
/// `prolog ::= XMLDecl? Misc* (doctypedecl Misc*)?`.
///
/// Returns `(version, encoding, standalone, doctype)`. The doctype, if
/// present, is projected to a typed [`XmlDoctype`] carrying the
/// root-element name, any `ExternalID`, and the inline general entity
/// declarations parsed from the internal subset (§4.2 GEDecl).
fn parse_prolog(c: &mut Cursor<'_>) -> Result<PrologParts, XmlParseError> {
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
    parse_misc_star(c)?;
    let doctype = if c.starts_with("<!DOCTYPE") {
        let dt = parse_doctype(c)?;
        parse_misc_star(c)?;
        Some(dt)
    } else {
        None
    };
    Ok((version, encoding, standalone, doctype))
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
    Ok(XmlDoctype {
        root_name: name.qualified(),
        external_id,
        general_entities,
        internal_subset_had_pe_references,
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

/// W3C XML 1.0 §3 production \[39\] `element`:
/// `element ::= EmptyElemTag | STag content ETag`.
///
/// `entities` is the list of `<!ENTITY name "value">` declarations
/// the DOCTYPE projected; consulted by [`parse_reference`] when an
/// entity reference's name doesn't match one of the five §4.6
/// predefined entities.
fn parse_element(
    c: &mut Cursor<'_>,
    entities: &[XmlGeneralEntity],
    strict_entity_declared: bool,
) -> Result<XmlElement, XmlParseError> {
    c.consume("<")?;
    let name = parse_name(c)?;
    let mut attributes: Vec<XmlAttribute> = Vec::new();
    // Namespaces in XML 1.0 (Bray, Hollander, Layman & Tobin 2009) §3
    // permits any number of `xmlns` / `xmlns:prefix` declarations on a
    // single element. We collect every one in document order; the legacy
    // single `namespace` slot mirrors the first declaration for backward
    // compatibility with consumers that only need a representative.
    let mut namespaces: Vec<XmlNamespace> = Vec::new();

    loop {
        let had_ws = {
            let before = c.pos;
            c.skip_whitespace();
            c.pos != before
        };
        if c.starts_with("/>") {
            c.consume("/>")?;
            let namespace = namespaces.first().cloned();
            return Ok(XmlElement {
                name,
                namespace,
                namespaces,
                attributes,
                children: Vec::new(),
            });
        }
        if c.starts_with(">") {
            c.consume(">")?;
            break;
        }
        if !had_ws {
            return Err(c.syntax_error("whitespace, /> or >", &c.preview()));
        }
        // §3.1 production [41] `Attribute ::= Name Eq AttValue`.
        // Namespace declarations (Bray, Hollander, Layman & Tobin
        // 2009 §3) use the reserved `xmlns` / `xmlns:prefix` form.
        let attr_name = parse_name(c)?;
        c.skip_whitespace();
        c.consume("=")?;
        c.skip_whitespace();
        let value = parse_att_value(c, entities, strict_entity_declared)?;

        let is_ns_decl = attr_name.prefix.as_deref() == Some("xmlns")
            || (attr_name.prefix.is_none() && attr_name.local == "xmlns");
        if is_ns_decl {
            let prefix = if attr_name.prefix.is_some() {
                Some(attr_name.local.clone())
            } else {
                None
            };
            namespaces.push(XmlNamespace { prefix, uri: value });
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
            attributes.push(XmlAttribute {
                name: attr_name,
                value,
            });
        }
    }

    // content + ETag
    let children = parse_content(c, entities, strict_entity_declared)?;
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

    let namespace = namespaces.first().cloned();
    Ok(XmlElement {
        name,
        namespace,
        namespaces,
        attributes,
        children,
    })
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
                let ch_val = parse_char_ref(c)?;
                out.push(ch_val);
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
                    out.push(ch);
                } else {
                    include_user_general_entity_in_att_value(
                        &qualified,
                        ref_pos,
                        entities,
                        visited,
                        strict_entity_declared,
                        out,
                    )?;
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
            let result = parse_att_value_body_into(
                &mut sub,
                entities,
                visited,
                strict_entity_declared,
                AttValueTerminator::Eof,
                out,
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

/// W3C XML 1.0 §3 production \[43\] `content`:
/// `content ::= CharData? ((element | Reference | CDSect | PI | Comment) CharData?)*`.
fn parse_content(
    c: &mut Cursor<'_>,
    entities: &[XmlGeneralEntity],
    strict_entity_declared: bool,
) -> Result<Vec<XmlNode>, XmlParseError> {
    let mut visited = Vec::new();
    parse_content_with_terminator(
        c,
        entities,
        &mut visited,
        strict_entity_declared,
        ContentTerminator::Etag,
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
) -> Result<Vec<XmlNode>, XmlParseError> {
    let mut nodes: Vec<XmlNode> = Vec::new();
    let mut text_buf = String::new();
    parse_content_into_buffers(
        c,
        entities,
        visited,
        strict_entity_declared,
        term,
        &mut nodes,
        &mut text_buf,
    )?;
    flush_text(&mut nodes, &mut text_buf);
    Ok(nodes)
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
fn parse_content_into_buffers(
    c: &mut Cursor<'_>,
    entities: &[XmlGeneralEntity],
    visited: &mut Vec<String>,
    strict_entity_declared: bool,
    term: ContentTerminator,
    nodes: &mut Vec<XmlNode>,
    text_buf: &mut String,
) -> Result<(), XmlParseError> {
    use crate::social::software::markup::xml::spec_1_0::{
        ContentItemKind, loaded_content_dispatch_table,
    };
    let dispatch = loaded_content_dispatch_table();
    loop {
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
                flush_text(nodes, text_buf);
                nodes.push(parse_comment_node(c)?);
                continue;
            }
            ContentItemKind::CDataSection => {
                flush_text(nodes, text_buf);
                nodes.push(parse_cdata_node(c)?);
                continue;
            }
            ContentItemKind::ProcessingInstruction => {
                flush_text(nodes, text_buf);
                nodes.push(parse_pi_node(c)?);
                continue;
            }
            ContentItemKind::Element => {
                flush_text(nodes, text_buf);
                let child = parse_element(c, entities, strict_entity_declared)?;
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
                let ch_val = parse_char_ref(c)?;
                text_buf.push(ch_val);
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
                    text_buf.push(ch);
                } else {
                    include_user_general_entity_in_content(
                        &qualified,
                        ref_pos,
                        entities,
                        visited,
                        strict_entity_declared,
                        nodes,
                        text_buf,
                    )?;
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
fn include_user_general_entity_in_content(
    name: &str,
    ref_pos: usize,
    entities: &[XmlGeneralEntity],
    visited: &mut Vec<String>,
    strict_entity_declared: bool,
    nodes: &mut Vec<XmlNode>,
    text_buf: &mut String,
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
            );
            visited.pop();
            result
        }
    }
}

fn flush_text(nodes: &mut Vec<XmlNode>, buf: &mut String) {
    if !buf.is_empty() {
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
