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
    XmlAttribute, XmlDocument, XmlElement, XmlName, XmlNamespace, XmlNode,
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
        }
    }
}

impl std::error::Error for XmlParseError {}

/// Top-level entry point.
///
/// Implements **production [1]** `document ::= prolog element Misc*`
/// from W3C XML 1.0 Fifth Edition §2.1.
pub fn parse_document(input: &[u8]) -> Result<XmlDocument, XmlParseError> {
    let s = core::str::from_utf8(input).map_err(|e| XmlParseError::NotUtf8 {
        position: e.valid_up_to(),
    })?;
    let mut cursor = Cursor::new(s);

    let (version, encoding) = parse_prolog(&mut cursor)?;
    let root = parse_element(&mut cursor)?;
    parse_misc_star(&mut cursor)?;
    cursor.skip_whitespace();
    if !cursor.is_eof() {
        return Err(cursor.syntax_error("end of document", "trailing content"));
    }

    Ok(XmlDocument {
        version,
        encoding,
        root,
    })
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
/// Returns `(version, encoding)` from the XMLDecl (or W3C-mandated
/// defaults if absent: version `1.0`, encoding `None`). DOCTYPE
/// declarations are skipped without entity expansion — the
/// well-formedness constraints we enforce don't require DTD
/// resolution.
fn parse_prolog(c: &mut Cursor<'_>) -> Result<(String, Option<String>), XmlParseError> {
    c.skip_whitespace();
    let (version, encoding) = if c.starts_with("<?xml") {
        parse_xml_decl(c)?
    } else {
        ("1.0".into(), None)
    };
    parse_misc_star(c)?;
    if c.starts_with("<!DOCTYPE") {
        skip_doctype(c)?;
        parse_misc_star(c)?;
    }
    Ok((version, encoding))
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

/// W3C XML 1.0 §2.8 production [28] `doctypedecl`. We don't expand
/// DTD bodies in this slice; the entire DOCTYPE construct is
/// skipped between `<!DOCTYPE` and the matching `>`, accounting
/// for an optional internal subset `[ ... ]`.
fn skip_doctype(c: &mut Cursor<'_>) -> Result<(), XmlParseError> {
    c.consume("<!DOCTYPE")?;
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
        context: "DOCTYPE declaration".into(),
    })
}

/// W3C XML 1.0 §3 production [39] `element`:
/// `element ::= EmptyElemTag | STag content ETag`.
fn parse_element(c: &mut Cursor<'_>) -> Result<XmlElement, XmlParseError> {
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
        let value = parse_att_value(c)?;

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
            attributes.push(XmlAttribute {
                name: attr_name,
                value,
            });
        }
    }

    // content + ETag
    let children = parse_content(c)?;
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

/// W3C XML 1.0 §2.3 production [4] `NameStartChar`. Subset used
/// here covers the characters that USC titles + USLM schema
/// content actually use.
fn is_name_start_char(ch: char) -> bool {
    matches!(ch, 'A'..='Z' | 'a'..='z' | '_' | ':')
        || matches!(ch as u32, 0xC0..=0xD6 | 0xD8..=0xF6 | 0xF8..=0x2FF
            | 0x370..=0x37D | 0x37F..=0x1FFF | 0x200C..=0x200D
            | 0x2070..=0x218F | 0x2C00..=0x2FEF | 0x3001..=0xD7FF
            | 0xF900..=0xFDCF | 0xFDF0..=0xFFFD | 0x10000..=0xEFFFF)
}

/// W3C XML 1.0 §2.3 production [4a] `NameChar`. Extends
/// [`is_name_start_char`] with digits, `.`, `-`, `·`, and the
/// combining-mark ranges.
fn is_name_char(ch: char) -> bool {
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
fn parse_att_value(c: &mut Cursor<'_>) -> Result<String, XmlParseError> {
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
            out.push(parse_reference(c)?);
        } else {
            out.push(ch);
            c.pos += ch.len_utf8();
        }
    }
}

/// W3C XML 1.0 §3 production [43] `content`:
/// `content ::= CharData? ((element | Reference | CDSect | PI | Comment) CharData?)*`.
fn parse_content(c: &mut Cursor<'_>) -> Result<Vec<XmlNode>, XmlParseError> {
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
            let child = parse_element(c)?;
            nodes.push(XmlNode::Element(child));
            continue;
        }
        let ch = c.peek_char().ok_or_else(|| XmlParseError::UnexpectedEof {
            context: "element content".into(),
        })?;
        if ch == '&' {
            text_buf.push(parse_reference(c)?);
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
fn parse_reference(c: &mut Cursor<'_>) -> Result<char, XmlParseError> {
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
        let name = parse_name(c)?;
        c.consume(";")?;
        match (name.prefix.as_deref(), name.local.as_str()) {
            (None, "amp") => Ok('&'),
            (None, "lt") => Ok('<'),
            (None, "gt") => Ok('>'),
            (None, "apos") => Ok('\''),
            (None, "quot") => Ok('"'),
            _ => Err(XmlParseError::UnsupportedEntity {
                position: start_pos,
                name: name.qualified(),
            }),
        }
    }
}
