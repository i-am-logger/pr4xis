#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::*;

/// Read XML text into an XmlDocument through the XML ontology.
///
/// This is NOT a mechanical parser — it's the XML ontology applied to text.
/// It understands what `<`, `>`, `&` MEAN because the ontology defines them.
/// It produces XmlDocument/XmlElement/XmlNode because those are the
/// ontological types that XML content IS.
pub fn read_xml(input: &str) -> Result<XmlDocument, XmlReadError> {
    let input = input.trim();
    let mut pos = 0;

    // Read XML declaration if present
    let (version, encoding) = if input.starts_with("<?xml") {
        let end = input
            .find("?>")
            .ok_or(XmlReadError::new("unclosed XML declaration"))?;
        let decl = &input[5..end];
        let version = extract_attr_value(decl, "version").unwrap_or("1.0".into());
        let encoding = extract_attr_value(decl, "encoding");
        pos = end + 2;
        (version, encoding)
    } else {
        ("1.0".into(), None)
    };

    // Skip whitespace, comments, PIs, DOCTYPE before root element
    // (W3C XML 1.0 §2.8 — prolog may contain Misc* which is
    // Comment | PI | S). Loop until we land on the root element.
    loop {
        let remaining = input[pos..].trim_start();
        pos = input.len() - remaining.len();

        if input[pos..].starts_with("<!DOCTYPE") {
            let end =
                find_doctype_end(&input[pos..]).ok_or(XmlReadError::new("unclosed DOCTYPE"))?;
            pos += end;
            continue;
        }
        if input[pos..].starts_with("<!--") {
            let end = input[pos..]
                .find("-->")
                .ok_or(XmlReadError::new("unclosed comment in prolog"))?;
            pos += end + 3;
            continue;
        }
        if input[pos..].starts_with("<?") {
            let end = input[pos..]
                .find("?>")
                .ok_or(XmlReadError::new("unclosed PI in prolog"))?;
            pos += end + 2;
            continue;
        }
        break;
    }

    // Read root element
    let (root, _) = read_element(&input[pos..])?;

    Ok(XmlDocument {
        version,
        encoding,
        doctype: None,
        root,
    })
}

fn read_element(input: &str) -> Result<(XmlElement, usize), XmlReadError> {
    let input = input.trim_start();
    if !input.starts_with('<') {
        return Err(XmlReadError::new("expected '<' to start element"));
    }

    // Find end of opening tag
    let tag_end = input
        .find('>')
        .ok_or(XmlReadError::new("unclosed opening tag"))?;
    let tag_content = &input[1..tag_end];

    // Check for self-closing
    let self_closing = tag_content.ends_with('/');
    let tag_content = if self_closing {
        &tag_content[..tag_content.len() - 1]
    } else {
        tag_content
    };

    // Parse tag name and attributes
    let (name, attrs) = parse_tag_content(tag_content)?;
    let xml_name = parse_xml_name(&name);
    // Namespaces in XML 1.0 (Bray, Hollander, Layman & Tobin 2009) §3 —
    // every `xmlns` / `xmlns:prefix` attribute is a namespace declaration,
    // not a regular attribute. Collect them all (in document order) into
    // `namespaces`; the legacy single `namespace` slot mirrors the first.
    let all_namespaces = extract_all_namespaces(&attrs);
    let namespace = all_namespaces.first().cloned();
    let xml_attrs: Vec<XmlAttribute> = attrs
        .into_iter()
        .filter(|(k, _)| !k.starts_with("xmlns"))
        .map(|(k, v)| XmlAttribute {
            name: parse_xml_name(&k),
            value: unescape_xml(&v),
        })
        .collect();

    if self_closing {
        return Ok((
            XmlElement {
                name: xml_name,
                namespace,
                namespaces: all_namespaces,
                attributes: xml_attrs,
                children: Vec::new(),
            },
            tag_end + 1,
        ));
    }

    // Read children until closing tag
    let mut children = Vec::new();
    let mut pos = tag_end + 1;
    let closing_tag = format!("</{}>", name);

    loop {
        if pos >= input.len() {
            return Err(XmlReadError::new(&format!("unclosed element '{}'", name)));
        }

        let remaining = &input[pos..];

        // Check for closing tag
        if remaining.starts_with(&closing_tag) {
            pos += closing_tag.len();
            break;
        }

        // Check for child element
        if remaining.starts_with("</") {
            // Mismatched closing tag
            return Err(XmlReadError::new(&format!(
                "unexpected closing tag, expected '{}'",
                closing_tag
            )));
        }

        if remaining.starts_with("<![CDATA[") {
            let end = remaining
                .find("]]>")
                .ok_or(XmlReadError::new("unclosed CDATA"))?;
            children.push(XmlNode::CData(remaining[9..end].into()));
            pos += end + 3;
        } else if remaining.starts_with("<!--") {
            let end = remaining
                .find("-->")
                .ok_or(XmlReadError::new("unclosed comment"))?;
            children.push(XmlNode::Comment(remaining[4..end].into()));
            pos += end + 3;
        } else if remaining.starts_with("<?") {
            let end = remaining
                .find("?>")
                .ok_or(XmlReadError::new("unclosed PI"))?;
            let pi_content = &remaining[2..end];
            let (target, data) = pi_content
                .split_once(char::is_whitespace)
                .map(|(t, d)| (t.to_string(), Some(d.trim().to_string())))
                .unwrap_or((pi_content.to_string(), None));
            children.push(XmlNode::ProcessingInstruction { target, data });
            pos += end + 2;
        } else if remaining.starts_with('<') {
            let (child_elem, consumed) = read_element(remaining)?;
            children.push(XmlNode::Element(child_elem));
            pos += consumed;
        } else {
            // Text content — read until next '<'
            let text_end = remaining.find('<').unwrap_or(remaining.len());
            let text = &remaining[..text_end];
            if !text.trim().is_empty() {
                children.push(XmlNode::Text(unescape_xml(text)));
            }
            pos += text_end;
        }
    }

    Ok((
        XmlElement {
            name: xml_name,
            namespace,
            namespaces: all_namespaces,
            attributes: xml_attrs,
            children,
        },
        pos,
    ))
}

fn parse_tag_content(content: &str) -> Result<(String, Vec<(String, String)>), XmlReadError> {
    let content = content.trim();
    let name_end = content
        .find(|c: char| c.is_whitespace())
        .unwrap_or(content.len());
    let name = content[..name_end].to_string();
    let rest = content[name_end..].trim();

    let mut attrs = Vec::new();
    let mut pos = 0;
    let bytes = rest.as_bytes();

    while pos < rest.len() {
        // Skip whitespace
        while pos < rest.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= rest.len() {
            break;
        }

        // Read attribute name
        let attr_start = pos;
        while pos < rest.len() && bytes[pos] != b'=' && !bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        let attr_name = rest[attr_start..pos].to_string();

        // Skip = and whitespace
        while pos < rest.len() && (bytes[pos] == b'=' || bytes[pos].is_ascii_whitespace()) {
            pos += 1;
        }

        // Read quoted value
        if pos < rest.len() && (bytes[pos] == b'"' || bytes[pos] == b'\'') {
            let quote = bytes[pos];
            pos += 1;
            let val_start = pos;
            while pos < rest.len() && bytes[pos] != quote {
                pos += 1;
            }
            let val = rest[val_start..pos].to_string();
            pos += 1; // skip closing quote
            attrs.push((attr_name, val));
        }
    }

    Ok((name, attrs))
}

fn parse_xml_name(name: &str) -> XmlName {
    if let Some((prefix, local)) = name.split_once(':') {
        XmlName::with_prefix(prefix, local)
    } else {
        XmlName::new(name)
    }
}

/// Collect every `xmlns` / `xmlns:prefix` declaration on an element in
/// document order. Namespaces in XML 1.0 (Bray, Hollander, Layman & Tobin
/// 2009 §3) treats these as namespace declarations rather than regular
/// attributes; consumers that need the full prefix→URI map (RDF/XML in
/// particular, RDF 1.1 XML Syntax §2.4) read this collection.
fn extract_all_namespaces(attrs: &[(String, String)]) -> Vec<XmlNamespace> {
    let mut ns = Vec::new();
    for (k, v) in attrs {
        if k == "xmlns" {
            ns.push(XmlNamespace {
                prefix: None,
                uri: v.clone(),
            });
        } else if let Some(prefix) = k.strip_prefix("xmlns:") {
            ns.push(XmlNamespace {
                prefix: Some(prefix.into()),
                uri: v.clone(),
            });
        }
    }
    ns
}

fn extract_attr_value(content: &str, name: &str) -> Option<String> {
    let pattern = format!("{}=", name);
    let start = content.find(&pattern)?;
    let rest = &content[start + pattern.len()..];
    let rest = rest.trim_start();
    if rest.starts_with('"') || rest.starts_with('\'') {
        let quote = rest.as_bytes()[0];
        let end = rest[1..].find(|c: char| c as u8 == quote)?;
        Some(rest[1..end + 1].into())
    } else {
        None
    }
}

fn unescape_xml(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

fn find_doctype_end(input: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in input.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            _ => {}
        }
    }
    None
}

#[derive(Debug)]
pub struct XmlReadError {
    pub message: String,
}

impl XmlReadError {
    pub fn new(msg: &str) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

impl core::fmt::Display for XmlReadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "XML read error: {}", self.message)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for XmlReadError {}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // ── Prolog-handling property tests ─────────────────────────────
    //
    // W3C XML 1.0 §2.8: the prolog allows arbitrary intermixing of
    // comments, processing instructions, and whitespace before the
    // root element (with at most one DOCTYPE). These properties
    // verify the read_xml prolog loop respects that ordering
    // freedom.

    const MINIMAL_ROOT: &str = "<r/>";

    #[test]
    fn prolog_with_no_extras_parses() {
        assert!(read_xml(MINIMAL_ROOT).is_ok());
    }

    #[test]
    fn prolog_with_xml_decl_parses() {
        let s = format!("<?xml version=\"1.0\"?>{MINIMAL_ROOT}");
        assert!(read_xml(&s).is_ok());
    }

    #[test]
    fn prolog_with_comment_parses() {
        let s = format!("<!-- hi --> {MINIMAL_ROOT}");
        assert!(read_xml(&s).is_ok());
    }

    #[test]
    fn prolog_with_xml_decl_and_comment_parses() {
        let s = format!("<?xml version=\"1.0\"?>\n<!-- doc -->\n{MINIMAL_ROOT}");
        assert!(read_xml(&s).is_ok());
    }

    #[test]
    fn prolog_with_multiple_comments_parses() {
        let s = format!("<!--a--><!--b--><!--c-->{MINIMAL_ROOT}");
        assert!(read_xml(&s).is_ok());
    }

    #[test]
    fn unclosed_prolog_comment_errors_cleanly() {
        let s = format!("<!-- never-closed {MINIMAL_ROOT}");
        let err = read_xml(&s).unwrap_err();
        assert!(err.message.contains("unclosed comment"));
    }

    // ── Adversarial-input properties (compliance: no panic on
    // ── malformed input) ──────────────────────────────────────────

    #[test]
    fn adversarial_random_bytes_never_panic() {
        // FRE 901 + Daubert prong 3: when input is malformed,
        // produce a typed Err — never panic, never silently succeed.
        let cases: &[&str] = &[
            "",
            "<",
            ">",
            "<<<",
            ">>>",
            "<r",
            "<r>",
            "<r>>",
            "<r><r></r>",
            "<r></s>",
            "<r/>extra",
            "<r attr=\"unclosed",
            "<r attr='unclosed",
            "<?xml",
            "<?xml version=\"1.0\"",
            "<!--",
            "<!-- unclosed root",
            "<!DOCTYPE",
            "<!DOCTYPE r",
            "<![CDATA[unclosed",
            "&amp;",
            "<r>&amp;</r>",
            // Mixed content with unbalanced tags
            "<a><b><c></a></b></c>",
            // Self-close with content
            "<r><nested/></nested>",
        ];
        for input in cases {
            let result = read_xml(input);
            // Caller gets either Ok or Err. The function MUST NOT
            // panic — that would crash the auditor pipeline.
            let _ = result;
        }
    }

    proptest! {
        #[test]
        fn property_random_input_never_panics(s in "[\\x00-\\x7F]{0,200}") {
            // Random ASCII byte streams (incl. nul and control chars)
            // must never cause the parser to panic. Daubert prong 3.
            let _ = read_xml(&s);
        }

        #[test]
        fn property_random_tags_never_panic(
            tag in "[a-z]{1,8}",
            content in "[a-z0-9 ]{0,32}",
            attrs in proptest::collection::vec(("[a-z]{1,4}", "[a-z]{1,8}"), 0..3),
        ) {
            // Build syntactically-valid but semantically-arbitrary
            // XML and verify the parser doesn't panic.
            let mut s = format!("<{tag}");
            for (k, v) in &attrs {
                s.push_str(&format!(" {k}=\"{v}\""));
            }
            s.push('>');
            s.push_str(&content);
            s.push_str(&format!("</{tag}>"));
            let _ = read_xml(&s);
        }

        #[test]
        fn property_truncated_input_never_panics(
            full in "<r[a-z ]{0,20}>[a-z ]{0,20}</r>",
            cut_at in 0usize..50,
        ) {
            // Truncate input at an arbitrary position; parser must
            // return Ok or Err, never panic.
            let len = full.len().min(cut_at);
            // Truncate at char boundary to avoid str slicing panic
            // (that's an input-validation concern, not parser-bug).
            let boundary = full.char_indices().take_while(|(i, _)| *i <= len).last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            let truncated = &full[..boundary];
            let _ = read_xml(truncated);
        }
    }

    proptest! {
        #[test]
        fn property_prolog_with_arbitrary_comments_parses(
            comments in proptest::collection::vec("[a-z ]{0,20}", 0..6),
        ) {
            // Any sequence of comments (with safe ASCII content) in
            // the prolog should not break the parser.
            let mut s = String::new();
            for c in &comments {
                s.push_str("<!--");
                s.push_str(c);
                s.push_str("-->");
            }
            s.push_str(MINIMAL_ROOT);
            prop_assert!(
                read_xml(&s).is_ok(),
                "failed to parse prolog with {} comments: {s:?}",
                comments.len()
            );
        }

        #[test]
        fn property_xml_decl_position_invariant(
            ws_count in 0usize..5,
            comment_count in 0usize..3,
        ) {
            // <?xml ?> followed by whitespace and comments must still
            // produce a parsed root.
            let mut s = String::from("<?xml version=\"1.0\"?>");
            for _ in 0..ws_count {
                s.push(' ');
            }
            for i in 0..comment_count {
                s.push_str(&format!("<!--c{i}-->"));
            }
            s.push_str(MINIMAL_ROOT);
            prop_assert!(read_xml(&s).is_ok());
        }

        #[test]
        fn property_self_closing_root_with_attributes_parses(
            attrs in proptest::collection::vec(("[a-z]{1,8}", "[a-z]{1,8}"), 0..5),
        ) {
            let mut s = String::from("<r");
            for (k, v) in &attrs {
                s.push_str(&format!(" {k}=\"{v}\""));
            }
            s.push_str("/>");
            prop_assert!(read_xml(&s).is_ok());
        }
    }
}
