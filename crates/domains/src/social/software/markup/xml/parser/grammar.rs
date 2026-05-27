//! Recursive-descent parser over the W3C XML 1.0 Fifth Edition
//! grammar (Bray et al. 2008).
//!
//! Each `parse_<production>` function transcribes one of the W3C
//! EBNF productions (cited in the function's doc-comment), advances
//! the byte cursor, and emits the corresponding piece of the typed
//! [`XmlDocument`](super::super::ontology::XmlDocument) Infoset
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
    XmlAttribute, XmlDoctype, XmlDocument, XmlElement, XmlExternalId, XmlName, XmlNamespace,
    XmlNode,
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
    /// W3C XML 1.0 Fifth Edition §2.5 production [15] `Comment`:
    /// `Comment ::= '<!--' ((Char - '-') | ('-' (Char - '-')))* '-->'`
    /// — equivalent to "the string `--` MUST NOT occur within comments".
    MalformedComment { position: usize },
    /// A `]]>` sequence appeared in `CharData` outside a CDATA section.
    /// W3C XML 1.0 Fifth Edition §2.4 production [14] `CharData`:
    /// `CharData ::= [^<&]* - ([^<&]* ']]>' [^<&]*)` — the `]]>`
    /// sequence MUST be escaped in `CharData`.
    DisallowedCdataEnd { position: usize },
    /// A code point outside the W3C XML 1.0 §2.2 production [2]
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
/// Implements **production [1]** `document ::= prolog element Misc*`
/// from W3C XML 1.0 Fifth Edition §2.1.
///
/// **W3C XML 1.0 §F (Autodetection of Character Encodings)** is
/// applied via [`decode_input`]: byte-order marks select between
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
    let raw = decode_input(input)?;
    let normalized = normalize_line_endings(&raw);
    let mut cursor = Cursor::new(&normalized);

    let (version, encoding, doctype) = parse_prolog(&mut cursor)?;
    let entity_map: Vec<(String, String)> = doctype
        .as_ref()
        .map(|d| d.general_entities.clone())
        .unwrap_or_default();
    let root = parse_element(&mut cursor, &entity_map)?;
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
fn decode_input(input: &[u8]) -> Result<String, XmlParseError> {
    if let Some(body) = input.strip_prefix(&[0xFE, 0xFF]) {
        return decode_utf16(body, /* big_endian = */ true);
    }
    if let Some(body) = input.strip_prefix(&[0xFF, 0xFE]) {
        return decode_utf16(body, /* big_endian = */ false);
    }
    let body = input.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(input);
    core::str::from_utf8(body)
        .map(|s| s.to_string())
        .map_err(|e| XmlParseError::NotUtf8 {
            position: e.valid_up_to(),
        })
}

/// Decode UTF-16 bytes (already-stripped of the BOM) to a UTF-8
/// `String`. Endianness is selected by the caller from the BOM.
/// Reports `NotUtf8` on odd byte length or unpaired surrogates per
/// the Unicode 15 §3.9 D89/D91 invariants.
fn decode_utf16(bytes: &[u8], big_endian: bool) -> Result<String, XmlParseError> {
    if bytes.len() % 2 != 0 {
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
    let mut unit_index = 0usize;
    for result in core::char::decode_utf16(units.iter().copied()) {
        match result {
            Ok(ch) => out.push(ch),
            Err(_) => {
                return Err(XmlParseError::NotUtf8 {
                    position: 2 * unit_index,
                });
            }
        }
        unit_index += 1;
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

    /// W3C XML 1.0 §2.3 production [3] S — whitespace.
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

/// W3C XML 1.0 §2.8 production [22] `prolog`:
/// `prolog ::= XMLDecl? Misc* (doctypedecl Misc*)?`.
///
/// Returns `(version, encoding, doctype)`. The doctype, if present,
/// is projected to a typed [`XmlDoctype`] carrying the root-element
/// name, any `ExternalID`, and the inline general entity
/// declarations parsed from the internal subset (§4.2 GEDecl).
fn parse_prolog(
    c: &mut Cursor<'_>,
) -> Result<(String, Option<String>, Option<XmlDoctype>), XmlParseError> {
    c.skip_whitespace();
    let (version, encoding) = if c.starts_with("<?xml") {
        parse_xml_decl(c)?
    } else {
        ("1.0".into(), None)
    };
    parse_misc_star(c)?;
    let doctype = if c.starts_with("<!DOCTYPE") {
        let dt = parse_doctype(c)?;
        parse_misc_star(c)?;
        Some(dt)
    } else {
        None
    };
    Ok((version, encoding, doctype))
}

/// W3C XML 1.0 §2.8 production [23] `XMLDecl`:
/// `XMLDecl ::= '<?xml' VersionInfo EncodingDecl? SDDecl? S? '?>'`.
fn parse_xml_decl(c: &mut Cursor<'_>) -> Result<(String, Option<String>), XmlParseError> {
    c.consume("<?xml")?;
    c.require_whitespace("XMLDecl VersionInfo")?;
    c.consume("version")?;
    c.skip_whitespace();
    c.consume("=")?;
    c.skip_whitespace();
    let version = parse_quoted(c)?;
    c.skip_whitespace();

    let encoding = if c.starts_with("encoding") {
        c.consume("encoding")?;
        c.skip_whitespace();
        c.consume("=")?;
        c.skip_whitespace();
        let enc = parse_quoted(c)?;
        c.skip_whitespace();
        Some(enc)
    } else {
        None
    };

    if c.starts_with("standalone") {
        c.consume("standalone")?;
        c.skip_whitespace();
        c.consume("=")?;
        c.skip_whitespace();
        let _sa = parse_quoted(c)?;
        c.skip_whitespace();
    }

    c.consume("?>")?;
    Ok((version, encoding))
}

/// W3C XML 1.0 §3.1 production [10] `AttValue`'s quoted form,
/// reused for XMLDecl attributes by §2.8 (since their grammar
/// admits the same `' ... '` or `" ... "` form). At this point in
/// XMLDecl, references aren't permitted (§2.8 production [24]
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

/// W3C XML 1.0 §2.8 production [27] `Misc`:
/// `Misc ::= Comment | PI | S`.
///
/// Consumes zero or more `Misc` items before/between/after the
/// document element. Comments and PIs in the prolog/epilog are
/// dropped from the [`XmlDocument`] — the Infoset preserves them
/// only inside element content, not at the document level (Cowan
/// & Tobin 2004 §2.1 only requires document-level *children* for
/// the root element).
fn parse_misc_star(c: &mut Cursor<'_>) -> Result<(), XmlParseError> {
    loop {
        c.skip_whitespace();
        if c.starts_with("<!--") {
            skip_comment(c)?;
        } else if c.starts_with("<?") {
            skip_pi(c)?;
        } else {
            break;
        }
    }
    Ok(())
}

/// W3C XML 1.0 §2.5 production [15] `Comment`:
/// `Comment ::= '<!--' ((Char - '-') | ('-' (Char - '-')))* '-->'`.
fn skip_comment(c: &mut Cursor<'_>) -> Result<(), XmlParseError> {
    c.consume("<!--")?;
    let rest = c.rest();
    let end = rest
        .find("-->")
        .ok_or_else(|| XmlParseError::UnexpectedEof {
            context: "comment".into(),
        })?;
    c.pos += end + 3;
    Ok(())
}

/// W3C XML 1.0 §2.6 production [16] `PI`:
/// `PI ::= '<?' PITarget (S (Char* - (Char* '?>' Char*)))? '?>'`.
fn skip_pi(c: &mut Cursor<'_>) -> Result<(), XmlParseError> {
    c.consume("<?")?;
    let rest = c.rest();
    let end = rest
        .find("?>")
        .ok_or_else(|| XmlParseError::UnexpectedEof {
            context: "processing instruction".into(),
        })?;
    c.pos += end + 2;
    Ok(())
}

/// W3C XML 1.0 §2.8 production [28] `doctypedecl`:
/// `doctypedecl ::= '<!DOCTYPE' S Name (S ExternalID)? S? ('[' intSubset ']' S?)? '>'`.
///
/// Projects the declaration to a typed [`XmlDoctype`] carrying the
/// root-element name, any `ExternalID` (§4.2.2 [75]), and inline
/// general entity declarations (§4.2 [70] GEDecl) parsed from the
/// internal subset. Element-type declarations (§3.2 [45]),
/// attribute-list declarations (§3.3 [52]), notation declarations
/// (§4.7 [82]), parameter entity declarations (§4.2 [72]), and
/// parameter entity references (§4.1 [69]) within the internal
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

    let mut general_entities: Vec<(String, String)> = Vec::new();
    if c.starts_with("[") {
        c.consume("[")?;
        parse_internal_subset(c, &mut general_entities)?;
        c.consume("]")?;
        c.skip_whitespace();
    }

    c.consume(">")?;
    Ok(XmlDoctype {
        root_name: name.qualified(),
        external_id,
        general_entities,
    })
}

/// W3C XML 1.0 §4.2.2 production [75] `ExternalID`:
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
        let public_id = parse_quoted(c)?;
        c.require_whitespace("ExternalID SystemLiteral")?;
        let system_literal = parse_quoted(c)?;
        Ok(XmlExternalId::Public {
            public_id,
            system_literal,
        })
    }
}

/// W3C XML 1.0 §2.8 production [28b] `intSubset`:
/// `intSubset ::= (markupdecl | DeclSep)*` where
/// `markupdecl ::= elementdecl | AttlistDecl | EntityDecl | NotationDecl | PI | Comment`
/// (§2.8 [29]) and `DeclSep ::= PEReference | S` (§2.8 [28a]).
///
/// We project `<!ENTITY name "value">` general entity declarations
/// (§4.2 [70]/[71]) into the entity map. Other markup declarations
/// are consumed structurally but not projected — they don't affect
/// well-formedness.
fn parse_internal_subset(
    c: &mut Cursor<'_>,
    general_entities: &mut Vec<(String, String)>,
) -> Result<(), XmlParseError> {
    // §4.4 Table-4: parameter entities are recognised in the DTD
    // ("Included as PE"). We capture their replacement text from
    // PEDecls and substitute `%name;` references in subsequent
    // markup-decls before grammar validation (M5.ζ.4.b).
    let mut parameter_entities: Vec<(String, String)> = Vec::new();
    loop {
        c.skip_whitespace();
        if c.starts_with("]") {
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
            if let Some((name, value)) = parse_entity_decl(c, &mut parameter_entities)? {
                // §4.5 — duplicate entity declarations: the first wins.
                if !general_entities.iter().any(|(n, _)| n == &name) {
                    general_entities.push((name, value));
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
            let expanded_owned;
            let to_match: &str = if decl_text.contains('%') && !parameter_entities.is_empty() {
                expanded_owned = expand_pe_references(decl_text, &parameter_entities);
                &expanded_owned
            } else {
                decl_text
            };
            let grammar = crate::social::software::markup::xml::spec_1_0::loaded_xml_1_0_grammar();
            let mut interp = pr4xis::xml_grammar::Interpreter::new(grammar, to_match);
            match interp.match_production(production_name, 0) {
                pr4xis::xml_grammar::MatchResult::Match { end_pos }
                    if end_pos == to_match.len() =>
                {
                    c.pos = decl_end;
                    continue;
                }
                _ => {
                    return Err(c.syntax_error(production_name, &c.preview()));
                }
            }
        }
        if c.starts_with("%") {
            // §4.1 [69] PEReference — `%name;`. Top-level PE refs in
            // intSubset are consumed; their replacement effect on
            // surrounding markup-decls is handled by the per-decl
            // expansion above.
            skip_pe_reference(c)?;
            continue;
        }
        return Err(c.syntax_error("intSubset entry or `]`", &c.preview()));
    }
}

/// W3C XML 1.0 §4.4.8 "Included as PE": substitute every `%name;`
/// reference in `text` with the corresponding PE's replacement
/// text, surrounded by a leading and trailing space (#x20) so the
/// replacement forms a complete grammatical token in the expanded
/// DTD.
///
/// References to undefined names are left in place — the spec calls
/// this a validity-error situation, not a well-formedness violation;
/// the subsequent markup-decl match will reject the expanded text
/// if the unresolved `%name;` makes it ungrammatical.
fn expand_pe_references(text: &str, pes: &[(String, String)]) -> String {
    let mut out = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let after_percent = &text[i + 1..];
            if let Some(semi_rel) = after_percent.find(';') {
                let candidate = &after_percent[..semi_rel];
                let looks_like_name = !candidate.is_empty()
                    && candidate
                        .chars()
                        .all(|ch| !ch.is_whitespace() && ch != '%' && ch != ';');
                if looks_like_name {
                    if let Some((_, value)) = pes.iter().find(|(n, _)| n == candidate) {
                        out.push(' ');
                        out.push_str(value);
                        out.push(' ');
                        i += 1 + semi_rel + 1;
                        continue;
                    }
                }
            }
        }
        let ch = text[i..].chars().next().expect("non-empty by loop guard");
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// W3C XML 1.0 §4.2 production [70/71] `GEDecl`:
/// `<!ENTITY S Name S EntityValue S? >`.
///
/// Returns:
/// - `Some((name, value))` for an internal general entity (§4.2 [73]
///   `EntityValue` variant) — replacement text projected into the
///   doc's entity map.
/// - `Some((name, ""))` for an external general entity (§4.2 [73]
///   `ExternalID NDataDecl?` variant). Per §4.4 Table-4 row "Reference
///   in Content / External Parsed General", a non-validating parser
///   "Bypasses" the reference (replaces the reference with itself).
///   The praxis parser approximates this by registering the entity
///   with empty replacement text — well-formedness is preserved
///   (no UnsupportedEntity error on subsequent `&name;`); the text
///   output simply lacks the unread external content. Reading the
///   external entity body is M5.ε.5.b territory; this fix unblocks
///   the W3C XMLConf ext-sa cases that test well-formedness only.
/// - `None` for a parameter entity declaration (§4.2 [72] `PEDecl`).
///   PEs are skipped because they expand inside the DTD, not in
///   the document content (M5.ε.5.c — proper PE expansion).
fn parse_entity_decl(
    c: &mut Cursor<'_>,
    parameter_entities: &mut Vec<(String, String)>,
) -> Result<Option<(String, String)>, XmlParseError> {
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
        if c.starts_with("SYSTEM") || c.starts_with("PUBLIC") {
            // External PE — skip past `>` (respecting quoted literals).
            skip_until_close_angle(c)?;
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
        parse_optional_ndata_decl(c)?;
        c.skip_whitespace();
        c.consume(">")?;
        return Ok(Some((name.qualified(), String::new())));
    }
    if c.starts_with("PUBLIC") {
        c.consume("PUBLIC")?;
        c.require_whitespace("ExternalID PubidLiteral")?;
        let _pub_id = parse_quoted(c)?;
        // PUBLIC requires both PubidLiteral AND SystemLiteral separated
        // by whitespace; this is the gate that rejects malformed
        // declarations like `<!ENTITY foo PUBLIC "id">` (no SystemLiteral)
        // or `<!ENTITY e PUBLIC "a""b">` (no whitespace between literals).
        c.require_whitespace("ExternalID SystemLiteral")?;
        let _system_literal = parse_quoted(c)?;
        parse_optional_ndata_decl(c)?;
        c.skip_whitespace();
        c.consume(">")?;
        return Ok(Some((name.qualified(), String::new())));
    }

    let value = parse_entity_value(c)?;
    c.skip_whitespace();
    c.consume(">")?;
    Ok(Some((name.qualified(), value)))
}

/// §4.7 [76] `NDataDecl ::= S 'NDATA' S Name` — optional in
/// general-entity declarations marking an unparsed entity.
fn parse_optional_ndata_decl(c: &mut Cursor<'_>) -> Result<(), XmlParseError> {
    let save = c.pos;
    c.skip_whitespace();
    if c.starts_with("NDATA") {
        c.consume("NDATA")?;
        c.require_whitespace("NDataDecl Name")?;
        let _ = parse_name(c)?;
        Ok(())
    } else {
        // Not an NDataDecl — restore the cursor so the caller's
        // own skip_whitespace + `>` consume can run.
        c.pos = save;
        Ok(())
    }
}

/// W3C XML 1.0 §4.3.2 production [9] `EntityValue`:
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
                // pass through as literal. §4.5 calls this the
                // "literal entity value" before bypassing.
                let start = c.pos;
                let semi = c
                    .rest()
                    .find(';')
                    .ok_or_else(|| XmlParseError::UnexpectedEof {
                        context: "EntityValue Reference".into(),
                    })?;
                let literal = &c.input[start..start + semi + 1];
                out.push_str(literal);
                c.pos += semi + 1;
            }
        } else {
            out.push(ch);
            c.pos += ch.len_utf8();
        }
    }
}

/// Skip tokens up to and including the next top-level `>` while
/// respecting:
/// - paired `[`/`]` brackets (nested intSubset markers — §2.8 [28]),
/// - quoted string literals `"…"` and `'…'` (W3C XML 1.0 §4.2.2
///   [11] SystemLiteral / [12] PubidLiteral / §4.3.2 [9] EntityValue
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

fn skip_pe_reference(c: &mut Cursor<'_>) -> Result<(), XmlParseError> {
    c.consume("%")?;
    let _name = parse_name(c)?;
    c.consume(";")?;
    Ok(())
}

/// Helper: parse just a character reference (`&#digits;` or
/// `&#xhex;`). Factored out of [`parse_reference`] so
/// [`parse_entity_value`] can use it without going through the
/// general-entity-name branch.
fn parse_char_ref(c: &mut Cursor<'_>) -> Result<char, XmlParseError> {
    let start_pos = c.pos;
    c.consume("&")?;
    if c.starts_with("#x") {
        c.consume("#x")?;
        let rest = c.rest();
        let end = rest.find(';').ok_or_else(|| XmlParseError::UnexpectedEof {
            context: "character reference".into(),
        })?;
        let digits = &rest[..end];
        let code_point = u32::from_str_radix(digits, 16).map_err(|_| XmlParseError::Syntax {
            position: c.pos,
            expected: "hex digits".into(),
            found: digits.to_string(),
        })?;
        c.pos += end + 1;
        char::from_u32(code_point).ok_or(XmlParseError::InvalidCharRef {
            position: start_pos,
            code_point,
        })
    } else if c.starts_with("#") {
        c.consume("#")?;
        let rest = c.rest();
        let end = rest.find(';').ok_or_else(|| XmlParseError::UnexpectedEof {
            context: "character reference".into(),
        })?;
        let digits = &rest[..end];
        let code_point = digits.parse::<u32>().map_err(|_| XmlParseError::Syntax {
            position: c.pos,
            expected: "decimal digits".into(),
            found: digits.to_string(),
        })?;
        c.pos += end + 1;
        char::from_u32(code_point).ok_or(XmlParseError::InvalidCharRef {
            position: start_pos,
            code_point,
        })
    } else {
        Err(c.syntax_error("character reference", &c.preview()))
    }
}

/// W3C XML 1.0 §3 production [39] `element`:
/// `element ::= EmptyElemTag | STag content ETag`.
///
/// `entities` is the list of `<!ENTITY name "value">` declarations
/// the DOCTYPE projected; consulted by [`parse_reference`] when an
/// entity reference's name doesn't match one of the five §4.6
/// predefined entities.
fn parse_element(
    c: &mut Cursor<'_>,
    entities: &[(String, String)],
) -> Result<XmlElement, XmlParseError> {
    c.consume("<")?;
    let name = parse_name(c)?;
    let mut attributes: Vec<XmlAttribute> = Vec::new();
    let mut namespace: Option<XmlNamespace> = None;

    loop {
        let had_ws = {
            let before = c.pos;
            c.skip_whitespace();
            c.pos != before
        };
        if c.starts_with("/>") {
            c.consume("/>")?;
            return Ok(XmlElement {
                name,
                namespace,
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
        let value = parse_att_value(c, entities)?;

        let is_ns_decl = attr_name.prefix.as_deref() == Some("xmlns")
            || (attr_name.prefix.is_none() && attr_name.local == "xmlns");
        if is_ns_decl && namespace.is_none() {
            let prefix = if attr_name.prefix.is_some() {
                Some(attr_name.local.clone())
            } else {
                None
            };
            namespace = Some(XmlNamespace { prefix, uri: value });
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
    let children = parse_content(c, entities)?;
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

    Ok(XmlElement {
        name,
        namespace,
        attributes,
        children,
    })
}

/// W3C XML 1.0 §2.3 production [5] `Name`:
/// `Name ::= NameStartChar (NameChar)*`.
///
/// Production [4] `NameStartChar` covers ASCII letters, `_`, `:`,
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

/// W3C XML 1.0 §2.3 production [4] `NameStartChar` — the full
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

/// W3C XML 1.0 §2.3 production [4a] `NameChar`. Extends
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

/// W3C XML 1.0 §2.2 production [2] `Char` — the legal character
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

/// W3C XML 1.0 §3.1 production [10] `AttValue`:
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
    entities: &[(String, String)],
) -> Result<String, XmlParseError> {
    let quote = c.peek_char().ok_or_else(|| XmlParseError::UnexpectedEof {
        context: "AttValue".into(),
    })?;
    if quote != '"' && quote != '\'' {
        return Err(c.syntax_error("\" or '", &quote.to_string()));
    }
    c.pos += quote.len_utf8();

    let mut out = String::new();
    loop {
        let ch = c.peek_char().ok_or_else(|| XmlParseError::UnexpectedEof {
            context: "AttValue".into(),
        })?;
        if ch == quote {
            c.pos += quote.len_utf8();
            return Ok(out);
        }
        if ch == '<' {
            return Err(c.syntax_error("AttValue content (no '<')", "<"));
        }
        if ch == '&' {
            // Character + entity references contribute their referenced
            // character(s) unchanged (§3.3.3 step 3.1.1).
            out.push_str(&parse_reference(c, entities)?);
        } else if matches!(ch, '\t' | '\n' | '\r') {
            // §3.3.3 step 3.1.4: literal whitespace becomes #x20.
            out.push(' ');
            c.pos += ch.len_utf8();
        } else {
            out.push(ch);
            c.pos += ch.len_utf8();
        }
    }
}

/// W3C XML 1.0 §3 production [43] `content`:
/// `content ::= CharData? ((element | Reference | CDSect | PI | Comment) CharData?)*`.
fn parse_content(
    c: &mut Cursor<'_>,
    entities: &[(String, String)],
) -> Result<Vec<XmlNode>, XmlParseError> {
    let mut nodes: Vec<XmlNode> = Vec::new();
    let mut text_buf = String::new();

    loop {
        if c.starts_with("</") {
            flush_text(&mut nodes, &mut text_buf);
            return Ok(nodes);
        }
        if c.starts_with("<!--") {
            flush_text(&mut nodes, &mut text_buf);
            nodes.push(parse_comment_node(c)?);
            continue;
        }
        if c.starts_with("<![CDATA[") {
            flush_text(&mut nodes, &mut text_buf);
            nodes.push(parse_cdata_node(c)?);
            continue;
        }
        if c.starts_with("<?") {
            flush_text(&mut nodes, &mut text_buf);
            nodes.push(parse_pi_node(c)?);
            continue;
        }
        if c.starts_with("<") {
            flush_text(&mut nodes, &mut text_buf);
            let child = parse_element(c, entities)?;
            nodes.push(XmlNode::Element(child));
            continue;
        }
        let ch_pos = c.pos;
        let ch = c.peek_char().ok_or_else(|| XmlParseError::UnexpectedEof {
            context: "element content".into(),
        })?;
        if ch == '&' {
            text_buf.push_str(&parse_reference(c, entities)?);
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

fn flush_text(nodes: &mut Vec<XmlNode>, buf: &mut String) {
    if !buf.is_empty() {
        nodes.push(XmlNode::Text(core::mem::take(buf)));
    }
}

/// W3C XML 1.0 §2.5 production [15] `Comment`, emitting a typed
/// `XmlNode::Comment` for inside-element occurrences (Cowan &
/// Tobin 2004 §2.5 keeps comments in the Infoset).
///
/// Enforces two §2.5 well-formedness constraints:
/// - The body MUST NOT contain the string `--` (the EBNF subtraction
///   `(Char - '-')` is the spec form).
/// - Every character MUST be in the §2.2 [2] Char repertoire — the
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
    // §2.5: "--" must not occur within comments.
    if body.contains("--") {
        return Err(XmlParseError::MalformedComment {
            position: comment_start,
        });
    }
    check_chars_in_range(body, comment_start + 4, "Comment")?;
    let body = body.to_string();
    c.pos += end + 3;
    Ok(XmlNode::Comment(body))
}

/// W3C XML 1.0 §2.7 production [18] `CDSect`:
/// `CDSect ::= CDStart CData CDEnd`.
///
/// §2.7 [20] `CData ::= (Char* - (Char* ']]>' Char*))` — every char
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

/// W3C XML 1.0 §2.6 production [16] `PI` emitting a typed
/// `XmlNode::ProcessingInstruction`.
///
/// §2.6 [16] `PI ::= '<?' PITarget (S (Char* - (Char* '?>' Char*)))? '?>'`
/// — every char in the data segment must be in the §2.2 Char
/// repertoire; the `?>` tail is consumed by the close-marker.
fn parse_pi_node(c: &mut Cursor<'_>) -> Result<XmlNode, XmlParseError> {
    let pi_start = c.pos;
    c.consume("<?")?;
    let target_name = parse_name(c)?;
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

/// Walk `body` and reject the first character outside §2.2 [2] `Char`.
/// `position_of_body_start` is the byte offset of `body[0]` in the
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

/// W3C XML 1.0 §4.1 production [67] `Reference`:
/// `Reference ::= EntityRef | CharRef`. §4.6 defines the five
/// predefined entities (`amp`, `lt`, `gt`, `apos`, `quot`) every
/// XML processor must recognize. §4.1 productions [66] `CharRef`
/// covers numeric character references in decimal or hex.
///
/// `entities` is the list of `<!ENTITY name "value">` declarations
/// the DOCTYPE projected (W3C XML 1.0 §4.2 [70] GEDecl). When an
/// entity reference's name doesn't match a §4.6 predefined entity,
/// we consult this map per §4.4 "XML Processor Treatment of
/// Entities and References".
fn parse_reference(
    c: &mut Cursor<'_>,
    entities: &[(String, String)],
) -> Result<String, XmlParseError> {
    let start_pos = c.pos;
    if c.rest().starts_with("&#") {
        let ch = parse_char_ref(c)?;
        let mut s = String::new();
        s.push(ch);
        return Ok(s);
    }
    c.consume("&")?;
    let name = parse_name(c)?;
    c.consume(";")?;
    let qualified = name.qualified();
    match (name.prefix.as_deref(), name.local.as_str()) {
        (None, "amp") => Ok("&".into()),
        (None, "lt") => Ok("<".into()),
        (None, "gt") => Ok(">".into()),
        (None, "apos") => Ok("'".into()),
        (None, "quot") => Ok("\"".into()),
        _ => {
            // §4.4.3 — general entity references in content resolve to
            // their declared replacement text. We look up by qualified
            // name; declared entities are always unqualified per
            // §4.2 GEDecl.
            if let Some((_, value)) = entities.iter().find(|(n, _)| n == &qualified) {
                Ok(value.clone())
            } else {
                Err(XmlParseError::UnsupportedEntity {
                    position: start_pos,
                    name: qualified,
                })
            }
        }
    }
}
