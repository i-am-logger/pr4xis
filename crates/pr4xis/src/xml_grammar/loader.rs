//! Load every W3C XML 1.0 production from the bundled spec bytes
//! into a typed [`Grammar`].
//!
//! Walks the spec's `<prod id="NT-X" num="N">` blocks, extracts
//! each LHS name + production number + RHS content, parses the RHS
//! via [`parse_rhs`], and adds the result to the grammar.
//!
//! This is the runtime analog of dryruby/ebnf's grammar-loader step.
//! Per `feedback_bottom_up_loaded_not_encoded`, the W3C grammar
//! arrives in this codebase via this function — not as hand-written
//! Rust rules.
//!
//! ## Citation
//!
//! - **Bray, T., Paoli, J., Sperberg-McQueen, C. M., Maler, E. &
//!   Yergeau, F.** (eds.) (2008) *Extensible Markup Language (XML)
//!   1.0 (Fifth Edition)*, W3C Recommendation 26 November 2008,
//!   xmlspec.dtd-format source. The `<prod>`/`<lhs>`/`<rhs>` element
//!   conventions used here are the ones the spec itself uses.

use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
};

use super::ast::{Grammar, Production};
use super::rhs_parser::{ParseRhsError, parse_rhs};

/// Errors returned by [`load_grammar`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadGrammarError {
    /// The text scan didn't find a closing tag where the spec
    /// declares one (`</lhs>`, `</rhs>`, `</prod>` etc.).
    Malformed {
        production: Option<String>,
        position: usize,
        what: String,
    },
    /// A particular production's RHS failed [`parse_rhs`].
    RhsParse {
        production: String,
        position: usize,
        cause: Box<ParseRhsError>,
    },
}

impl core::fmt::Display for LoadGrammarError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Malformed {
                production,
                position,
                what,
            } => match production {
                Some(name) => write!(
                    f,
                    "malformed production `{name}` at byte {position}: {what}"
                ),
                None => write!(f, "malformed spec at byte {position}: {what}"),
            },
            Self::RhsParse {
                production,
                position,
                cause,
            } => write!(
                f,
                "RHS parse error in production `{production}` near byte {position}: {cause}"
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for LoadGrammarError {}

/// Walk the spec bytes and build a [`Grammar`] from every `<prod>`
/// block. Productions appear in source order.
///
/// `spec_bytes` is the xmlspec.dtd-format XML source — the bytes of
/// the registered `xml_1_0_fifth_edition@2008` praxis source. XML
/// comments (W3C XML 1.0 §2.5) inside `<prod>` bodies are stripped
/// before extraction so that e.g. EntityDecl's commented-out
/// `<com>General entities</com>` annotation between alternation
/// branches doesn't confuse the lhs/rhs locator.
///
/// On encountering a malformed block or an unparseable RHS, returns
/// the first error. (A loose-collection mode that reports every
/// failure could be added later.)
pub fn load_grammar(spec_bytes: &str) -> Result<Grammar, LoadGrammarError> {
    let mut grammar = Grammar::new();
    let stripped = strip_xml_comments(spec_bytes);
    let spec_bytes: &str = &stripped;
    let mut cursor = 0;
    while let Some(rel) = spec_bytes[cursor..].find("<prod ") {
        let prod_start = cursor + rel;
        let close_open =
            spec_bytes[prod_start..]
                .find('>')
                .ok_or_else(|| LoadGrammarError::Malformed {
                    production: None,
                    position: prod_start,
                    what: "unterminated <prod> open tag".to_string(),
                })?;
        let open_attrs = &spec_bytes[prod_start + 6..prod_start + close_open];
        let num = extract_attr(open_attrs, "num").unwrap_or_default();
        let prod_close = spec_bytes[prod_start..].find("</prod>").ok_or_else(|| {
            LoadGrammarError::Malformed {
                production: None,
                position: prod_start,
                what: "missing </prod>".to_string(),
            }
        })?;
        let prod_body = &spec_bytes[prod_start + close_open + 1..prod_start + prod_close];

        let (lhs_name, rhs_content) = split_lhs_rhs(prod_body, prod_start)?;
        let rhs_term = parse_rhs(&rhs_content).map_err(|e| LoadGrammarError::RhsParse {
            production: lhs_name.clone(),
            position: prod_start,
            cause: Box::new(e),
        })?;

        grammar.add(Production {
            name: lhs_name,
            number: num,
            rhs: rhs_term,
        });

        cursor = prod_start + prod_close + "</prod>".len();
    }
    Ok(grammar)
}

/// Remove every W3C XML 1.0 §2.5 comment (`<!-- … -->`) from `s`.
/// The spec uses these inside `<prod>` bodies to attach human
/// annotations between alternation branches — and those comments
/// sometimes embed faux `</rhs>` / `<rhs>` markers that would
/// confuse the loader's `find` calls. Stripping them up front lets
/// the rest of the loader treat the input as if it were
/// comment-free.
fn strip_xml_comments(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find("<!--") {
        out.push_str(&rest[..start]);
        let after_open = &rest[start + "<!--".len()..];
        if let Some(end) = after_open.find("-->") {
            rest = &after_open[end + "-->".len()..];
        } else {
            // Unterminated comment — append everything from here and
            // bail out of the loop. The caller will surface this if
            // it matters.
            out.push_str(&rest[start..]);
            return out;
        }
    }
    out.push_str(rest);
    out
}

/// Pull `key="value"` (or `key='value'`) out of the attribute list
/// of an open tag. Returns the inner value with quotes stripped.
fn extract_attr(open_attrs: &str, key: &str) -> Option<String> {
    let needle = format!("{key}=");
    let idx = open_attrs.find(&needle)?;
    let after = &open_attrs[idx + needle.len()..];
    let quote = after.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let rest = &after[1..];
    let end = rest.find(quote)?;
    Some(rest[..end].to_string())
}

/// Locate the `<lhs ...>name</lhs>` and concatenated `<rhs ...>...</rhs>`
/// content inside a production body (the content between
/// `<prod ...>` and `</prod>`). Tolerates attributes on `<lhs>` /
/// `<rhs>`.
///
/// Multiple `<rhs>` blocks per production are concatenated into a
/// single RHS string separated by spaces. The W3C spec uses this
/// pattern (e.g. §3.2 [51] `Mixed`, §3.3.1 [54] `AttType`) to wrap
/// long alternations across lines; the second and subsequent
/// `<rhs>` blocks begin with `|` to continue the alternation.
fn split_lhs_rhs(body: &str, prod_pos: usize) -> Result<(String, String), LoadGrammarError> {
    let lhs_open = body
        .find("<lhs")
        .ok_or_else(|| LoadGrammarError::Malformed {
            production: None,
            position: prod_pos,
            what: "missing <lhs>".to_string(),
        })?;
    let after_lhs_open_tag =
        body[lhs_open + 4..]
            .find('>')
            .ok_or_else(|| LoadGrammarError::Malformed {
                production: None,
                position: prod_pos,
                what: "unterminated <lhs> open tag".to_string(),
            })?;
    let lhs_content_start = lhs_open + 4 + after_lhs_open_tag + 1;
    let lhs_content_end =
        body[lhs_content_start..]
            .find("</lhs>")
            .ok_or_else(|| LoadGrammarError::Malformed {
                production: None,
                position: prod_pos,
                what: "missing </lhs>".to_string(),
            })?;
    let lhs_name = body[lhs_content_start..lhs_content_start + lhs_content_end]
        .trim()
        .to_string();

    // Walk every `<rhs>...</rhs>` block in document order, concatenating
    // their contents separated by a single space. Productions like
    // §3.2 [51] `Mixed` use this pattern to split a long alternation
    // across two `<rhs>` blocks, the second beginning with `|`.
    let mut rhs_parts: Vec<String> = Vec::new();
    let mut cursor = lhs_content_start + lhs_content_end + "</lhs>".len();
    while let Some(rel) = body[cursor..].find("<rhs") {
        let rhs_open_abs = cursor + rel;
        let after_rhs_open_tag =
            body[rhs_open_abs + 4..]
                .find('>')
                .ok_or_else(|| LoadGrammarError::Malformed {
                    production: Some(lhs_name.clone()),
                    position: prod_pos,
                    what: "unterminated <rhs> open tag".to_string(),
                })?;
        let rhs_content_start = rhs_open_abs + 4 + after_rhs_open_tag + 1;
        let rhs_content_end = body[rhs_content_start..].find("</rhs>").ok_or_else(|| {
            LoadGrammarError::Malformed {
                production: Some(lhs_name.clone()),
                position: prod_pos,
                what: "missing </rhs>".to_string(),
            }
        })?;
        rhs_parts.push(body[rhs_content_start..rhs_content_start + rhs_content_end].to_string());
        cursor = rhs_content_start + rhs_content_end + "</rhs>".len();
    }
    if rhs_parts.is_empty() {
        return Err(LoadGrammarError::Malformed {
            production: Some(lhs_name),
            position: prod_pos,
            what: "no <rhs> blocks".to_string(),
        });
    }
    Ok((lhs_name, rhs_parts.join(" ")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pick_attrs(s: &str) -> &str {
        &s[6..s.find('>').unwrap()]
    }

    #[test]
    fn extracts_double_quoted_attribute() {
        let open = "<prod id=\"NT-Char\" num=\"2\">";
        assert_eq!(extract_attr(pick_attrs(open), "num").as_deref(), Some("2"));
        assert_eq!(
            extract_attr(pick_attrs(open), "id").as_deref(),
            Some("NT-Char")
        );
    }

    #[test]
    fn extracts_single_quoted_attribute() {
        // Some spec productions use single-quoted ids (line 2465's
        // ExternalDef).
        let open = "<prod id='NT-ExternalDef'>";
        assert_eq!(
            extract_attr(pick_attrs(open), "id").as_deref(),
            Some("NT-ExternalDef")
        );
    }

    #[test]
    fn loads_two_production_fixture() {
        // Synthetic fixture mirroring two real spec productions.
        let spec = "
            <prod id=\"NT-document\" num=\"1\">
                <lhs>document</lhs>
                <rhs>#x9 | #xA</rhs>
            </prod>
            <prod id=\"NT-Char\" num=\"2\">
                <lhs diff=\"chg\">Char</lhs>
                <rhs>#x9 | #xA | #xD | [#x20-#xD7FF]</rhs>
            </prod>
        ";
        let g = load_grammar(spec).expect("load");
        assert_eq!(g.len(), 2);
        let p1 = g.lookup("document").expect("document");
        assert_eq!(p1.number, "1");
        let p2 = g.lookup("Char").expect("Char");
        assert_eq!(p2.number, "2");
    }

    #[test]
    fn reports_rhs_parse_errors() {
        let spec = "<prod id=\"NT-Bad\" num=\"99\">
            <lhs>Bad</lhs>
            <rhs>?</rhs>
        </prod>";
        let err = load_grammar(spec).unwrap_err();
        match err {
            LoadGrammarError::RhsParse { production, .. } => assert_eq!(production, "Bad"),
            _ => panic!("expected RhsParse"),
        }
    }
}
