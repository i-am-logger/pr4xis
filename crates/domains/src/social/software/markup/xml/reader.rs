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
    let remaining = input[pos..].trim_start();
    pos = input.len() - remaining.len();

    // Skip DOCTYPE if present
    if input[pos..].starts_with("<!DOCTYPE") {
        let end = find_doctype_end(&input[pos..]).ok_or(XmlReadError::new("unclosed DOCTYPE"))?;
        pos += end;
    }

    let remaining = input[pos..].trim_start();
    pos = input.len() - remaining.len();

    // Read root element
    let (root, _) = read_element(&input[pos..])?;

    Ok(XmlDocument {
        version,
        encoding,
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
    let namespace = extract_namespace(&attrs);
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

fn extract_namespace(attrs: &[(String, String)]) -> Option<XmlNamespace> {
    for (k, v) in attrs {
        if k == "xmlns" {
            return Some(XmlNamespace {
                prefix: None,
                uri: v.clone(),
            });
        }
        if let Some(prefix) = k.strip_prefix("xmlns:") {
            return Some(XmlNamespace {
                prefix: Some(prefix.into()),
                uri: v.clone(),
            });
        }
    }
    None
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
