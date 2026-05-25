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
        }
    }
}

impl std::error::Error for XmlParseError {}

/// Top-level entry point.
///
/// Implements **production [1]** `document ::= prolog element Misc*`
/// from W3C XML 1.0 Fifth Edition §2.1.
///
/// Strips an optional UTF-8 byte-order mark (BOM, U+FEFF encoded as
/// `EF BB BF`) at the start of the input per W3C XML 1.0 §F (Autodetection
/// of Character Encodings) — the BOM is allowed on UTF-8 streams and
/// is not part of the document content.
///
/// Performs **§2.11 End-of-Line Handling** on the resulting string
/// before parsing: every literal `#xD#xA` (CRLF) and every lone `#xD`
/// (CR) is replaced with a single `#xA` (LF). The W3C spec requires
/// this normalization on input so that downstream productions never
/// see CR.
pub fn parse_document(input: &[u8]) -> Result<XmlDocument, XmlParseError> {
    let input = input.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(input);
    let raw = core::str::from_utf8(input).map_err(|e| XmlParseError::NotUtf8 {
        position: e.valid_up_to(),
    })?;
    let normalized = normalize_line_endings(raw);
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
            if let Some((name, value)) = parse_entity_decl(c)? {
                // §4.5 — duplicate entity declarations: the first wins.
                if !general_entities.iter().any(|(n, _)| n == &name) {
                    general_entities.push((name, value));
                }
            }
            continue;
        }
        if c.starts_with("<!ELEMENT") || c.starts_with("<!ATTLIST") || c.starts_with("<!NOTATION") {
            // Consume the declaration up to its closing `>`; we do not
            // currently project these to typed values (validity, not
            // well-formedness).
            skip_markup_decl(c)?;
            continue;
        }
        if c.starts_with("%") {
            // §4.1 [69] PEReference — `%name;`. We don't resolve
            // parameter entities; consume the reference and continue.
            skip_pe_reference(c)?;
            continue;
        }
        return Err(c.syntax_error("intSubset entry or `]`", &c.preview()));
    }
}

/// W3C XML 1.0 §4.2 production [70/71] `GEDecl`:
/// `<!ENTITY S Name S EntityValue S? >`. Returns `Some((name,
/// value))` for a general entity declaration; returns `None` for a
/// parameter entity declaration (§4.2 [72]) and for external general
/// entities (§4.2 [73] ExternalID variant), both of which we
/// consume structurally without projection in this slice.
fn parse_entity_decl(c: &mut Cursor<'_>) -> Result<Option<(String, String)>, XmlParseError> {
    c.consume("<!ENTITY")?;
    c.require_whitespace("ENTITY name")?;

    // §4.2 [72] PEDecl path begins with `%`. We skip it.
    if c.starts_with("%") {
        skip_until_close_angle(c)?;
        return Ok(None);
    }

    let name = parse_name(c)?;
    c.require_whitespace("ENTITY value")?;

    // §4.2 [73] EntityDef ::= EntityValue | (ExternalID NDataDecl?).
    if c.starts_with("SYSTEM") || c.starts_with("PUBLIC") {
        // External entity — consume up to `>` without projection.
        skip_until_close_angle(c)?;
        return Ok(None);
    }

    let value = parse_entity_value(c)?;
    c.skip_whitespace();
    c.consume(">")?;
    Ok(Some((name.qualified(), value)))
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

/// Consume tokens up to and including the next `>` at top level
/// (paired square brackets nested within are ignored). Used to
/// skip element-type, attribute-list, and notation declarations
/// inside the internal subset whose typed projections we do not
/// yet emit.
fn skip_markup_decl(c: &mut Cursor<'_>) -> Result<(), XmlParseError> {
    skip_until_close_angle(c)
}

fn skip_until_close_angle(c: &mut Cursor<'_>) -> Result<(), XmlParseError> {
    let mut depth = 0u32;
    while let Some(ch) = c.peek_char() {
        match ch {
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
/// character class. Exposed for reuse by the XSD datatype lexical
/// mappings (`xs:Name` / `xs:NCName` / `xs:NMTOKEN` are defined by
/// reference to the XML 1.0 `Name` / `Nmtoken` productions, W3C XSD
/// 1.1 Part 2 §3.4.4-§3.4.7).
pub fn is_name_start_char(ch: char) -> bool {
    matches!(ch, 'A'..='Z' | 'a'..='z' | '_' | ':')
        || matches!(ch as u32, 0xC0..=0xD6 | 0xD8..=0xF6 | 0xF8..=0x2FF
            | 0x370..=0x37D | 0x37F..=0x1FFF | 0x200C..=0x200D
            | 0x2070..=0x218F | 0x2C00..=0x2FEF | 0x3001..=0xD7FF
            | 0xF900..=0xFDCF | 0xFDF0..=0xFFFD | 0x10000..=0xEFFFF)
}

/// W3C XML 1.0 §2.3 production [4a] `NameChar`. Extends
/// [`is_name_start_char`] with digits, `.`, `-`, `·`, and the
/// combining-mark ranges. Exposed for reuse by the XSD datatype
/// lexical mappings (see [`is_name_start_char`]).
pub fn is_name_char(ch: char) -> bool {
    is_name_start_char(ch)
        || matches!(ch, '-' | '.' | '0'..='9')
        || ch as u32 == 0xB7
        || matches!(ch as u32, 0x0300..=0x036F | 0x203F..=0x2040)
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
        let ch = c.peek_char().ok_or_else(|| XmlParseError::UnexpectedEof {
            context: "element content".into(),
        })?;
        if ch == '&' {
            text_buf.push_str(&parse_reference(c, entities)?);
        } else {
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
fn parse_comment_node(c: &mut Cursor<'_>) -> Result<XmlNode, XmlParseError> {
    c.consume("<!--")?;
    let rest = c.rest();
    let end = rest
        .find("-->")
        .ok_or_else(|| XmlParseError::UnexpectedEof {
            context: "comment".into(),
        })?;
    let body = rest[..end].to_string();
    c.pos += end + 3;
    Ok(XmlNode::Comment(body))
}

/// W3C XML 1.0 §2.7 production [18] `CDSect`:
/// `CDSect ::= CDStart CData CDEnd`.
fn parse_cdata_node(c: &mut Cursor<'_>) -> Result<XmlNode, XmlParseError> {
    c.consume("<![CDATA[")?;
    let rest = c.rest();
    let end = rest
        .find("]]>")
        .ok_or_else(|| XmlParseError::UnexpectedEof {
            context: "CDATA section".into(),
        })?;
    let body = rest[..end].to_string();
    c.pos += end + 3;
    Ok(XmlNode::CData(body))
}

/// W3C XML 1.0 §2.6 production [16] `PI` emitting a typed
/// `XmlNode::ProcessingInstruction`.
fn parse_pi_node(c: &mut Cursor<'_>) -> Result<XmlNode, XmlParseError> {
    c.consume("<?")?;
    let target_name = parse_name(c)?;
    let mut data: Option<String> = None;
    if c.peek_char()
        .is_some_and(|ch| matches!(ch, ' ' | '\t' | '\r' | '\n'))
    {
        c.skip_whitespace();
        let rest = c.rest();
        let end = rest
            .find("?>")
            .ok_or_else(|| XmlParseError::UnexpectedEof {
                context: "processing instruction".into(),
            })?;
        if end > 0 {
            data = Some(rest[..end].to_string());
        }
        c.pos += end;
    }
    c.consume("?>")?;
    Ok(XmlNode::ProcessingInstruction {
        target: target_name.qualified(),
        data,
    })
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
