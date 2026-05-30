//! XHTML 1.0 XSD → Rust ontology types codegen.
//!
//! Loads the W3C XHTML 1.0 Strict XML schema (published as
//! `xhtml1-strict.xsd` by the W3C HTML Working Group) and emits
//! typed Rust source describing every `xsd:element`,
//! `xsd:complexType`, `xsd:simpleType`, `xsd:attributeGroup`, and
//! `xsd:group` it defines. The heavy lifting — XSD parsing, name
//! resolution, substitution-group dispatch, type-graph construction
//! — is done by the [`xsd-parser`] crate by Sebastian Bergmann;
//! this module is the thin glue that wires xsd-parser into the
//! praxis codegen pipeline.
//!
//! Per "bottom-up loaded, never encoded": every Tier 2+ ontology
//! type derives from a registered authoritative source, not from
//! Rust enum variants written by hand. The XHTML 1.0 Strict XSD is
//! that authoritative source; this module is the loader.
//!
//! [`xsd-parser`]: https://docs.rs/xsd-parser/1.5.2
//!
//! ## Citations
//!
//! - Pemberton, S. et al. (eds.) (2002) *XHTML 1.0: The Extensible
//!   HyperText Markup Language (Second Edition)*, W3C Recommendation
//!   26 January 2000, revised 1 August 2002.
//!   <https://www.w3.org/TR/xhtml1/>. §A.1 Document Type Definitions
//!   (the companion XSD at
//!   <https://www.w3.org/2002/08/xhtml/xhtml1-strict.xsd> is a
//!   faithful XML Schema rendering of the §A.1.1 Strict DTD).
//! - Gao, S., Sperberg-McQueen, C. M., and Thompson, H. S. (eds.)
//!   *W3C XML Schema Definition Language (XSD) 1.1 Part 1:
//!   Structures*, W3C Recommendation 5 April 2012.
//!   <https://www.w3.org/TR/xmlschema11-1/>.
//! - Bergmann, S. *xsd-parser: Rust code generator for XML schema
//!   files*, v1.5.2, MIT-licensed.
//!   <https://github.com/Bergmann89/xsd-parser>.
//!
//! ## Pipeline
//!
//! 1. Read the XSD file from disk.
//! 2. Strip the `schemaLocation` attribute from the single upstream
//!    `<xsd:import>` of `http://www.w3.org/2001/xml.xsd`. Per the
//!    praxis-lock model (sources hash-pinned and bundled, not
//!    network-fetched), we provide a synthetic `xml.xsd` stub
//!    defining only the attributes XHTML 1.0 Strict references
//!    (`xml:lang`, `xml:space`).
//! 3. Hand the schemas to `xsd-parser`'s `generate()` and write
//!    the resulting Rust source to the supplied path.
//!
//! ## Known post-processing
//!
//! XHTML 1.0 Strict's `i18n` attribute group carries both
//! `<xs:attribute name="lang">` (the XHTML `lang` attribute) and
//! `<xs:attribute ref="xml:lang">` (the namespaced `xml:lang`
//! attribute, backwards-compatible per Pemberton et al. 2002 §3.3).
//! xsd-parser projects both to a `pub lang:` field on the same
//! struct, which Rust rejects (E0124). [`postprocess_xhtml_collisions`]
//! renames the second occurrence to `lang_xml`, preserving a 1-to-1
//! mapping between the XSD's declared attributes and Rust fields.

use std::path::Path;

use xsd_parser::{
    Config,
    config::{GeneratorFlags, InterpreterFlags, OptimizerFlags, ParserFlags, Resolver, Schema},
    generate,
};

/// Minimal stub for the `http://www.w3.org/XML/1998/namespace`
/// schema. XHTML 1.0 Strict imports `xml.xsd` only to reference
/// `xml:lang` and `xml:space`; providing this stub is sufficient
/// for the xsd-parser interpreter without fetching the upstream
/// schema at build time.
///
/// The stub follows the W3C-published `xml.xsd` shape (a subset).
const XML_XSD_STUB: &str = r#"<?xml version="1.0"?>
<xs:schema targetNamespace="http://www.w3.org/XML/1998/namespace"
   xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:attribute name="base" type="xs:anyURI"/>
  <xs:attribute name="lang" type="xs:string"/>
  <xs:attribute name="space">
    <xs:simpleType>
      <xs:restriction base="xs:NCName">
        <xs:enumeration value="default"/>
        <xs:enumeration value="preserve"/>
      </xs:restriction>
    </xs:simpleType>
  </xs:attribute>
  <xs:attribute name="id" type="xs:ID"/>
</xs:schema>
"#;

/// Errors returned by [`generate_xhtml_schema_source`].
#[derive(Debug)]
pub enum XhtmlSchemaCodegenError {
    /// Could not read the XSD file from disk.
    ReadXsd(String, std::io::Error),
    /// `xsd-parser` failed to parse or generate. The string is the
    /// crate's own error message.
    Generate(String),
}

impl core::fmt::Display for XhtmlSchemaCodegenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ReadXsd(p, e) => write!(f, "read XSD {p}: {e}"),
            Self::Generate(msg) => write!(f, "xsd-parser codegen failed: {msg}"),
        }
    }
}

impl std::error::Error for XhtmlSchemaCodegenError {}

/// Load `xsd_path` and emit Rust source describing every XHTML type.
/// Suitable for writing into `$OUT_DIR/xhtml_schema_generated.rs`
/// from a build script and `include!`'ing from a module.
///
/// The output references `xsd_parser_types` symbols
/// (`::xsd_parser_types::xml::Text`, `Mixed`, `AnyElement`,
/// `AnyAttributes`) for mixed-content / wildcard handling; the
/// downstream crate must depend on `xsd-parser-types` so the
/// generated code resolves.
///
/// # Errors
///
/// Returns [`XhtmlSchemaCodegenError`] if the XSD can't be read or
/// xsd-parser rejects the schema.
pub fn generate_xhtml_schema_source(xsd_path: &Path) -> Result<String, XhtmlSchemaCodegenError> {
    let xsd = std::fs::read_to_string(xsd_path)
        .map_err(|e| XhtmlSchemaCodegenError::ReadXsd(xsd_path.display().to_string(), e))?;

    let xsd_stripped = strip_schema_location_attrs(&xsd);

    let mut config = Config::default();
    config.parser.resolver = vec![Resolver::File];
    config.parser.flags = ParserFlags::DEFAULT_NAMESPACES
        | ParserFlags::GENERATE_PREFIXES
        | ParserFlags::ALTERNATIVE_PREFIXES;
    config.parser.schemas = vec![
        Schema::Schema(XML_XSD_STUB.to_owned()),
        Schema::Schema(xsd_stripped),
    ];

    config.interpreter.flags = InterpreterFlags::BUILDIN_TYPES
        | InterpreterFlags::DEFAULT_TYPEDEFS
        | InterpreterFlags::WITH_XS_ANY_TYPE
        | InterpreterFlags::WITH_XS_ANY_SIMPLE_TYPE;

    config.optimizer.flags = OptimizerFlags::all() - OptimizerFlags::REMOVE_DUPLICATES;

    config.generator.flags = GeneratorFlags::FLATTEN_CONTENT
        | GeneratorFlags::USE_MODULES
        | GeneratorFlags::BUILD_IN_ABSOLUTE_PATHS
        | GeneratorFlags::ABSOLUTE_PATHS_INSTEAD_USINGS
        | GeneratorFlags::MIXED_TYPE_SUPPORT;

    // XHTML 1.0 Strict uses `<xs:complexType name="...">` heavily
    // (FormCtrl, InlSpecial, Inline, Block, ...). Adding the same
    // `Item` postfix as USLM keeps the two generated trees naming-
    // consistent and avoids any future collision with xsd-parser's
    // built-in type aliases.
    config.generator.type_postfix.type_ = "Item".to_string();

    let tokens = generate(config).map_err(|e| XhtmlSchemaCodegenError::Generate(e.to_string()))?;
    let raw_source = tokens.to_string();
    Ok(postprocess_xhtml_collisions(&raw_source))
}

/// Fix the XHTML-specific identifier collisions xsd-parser cannot
/// disambiguate from XSD alone. The `i18n` attribute group on every
/// XHTML element carries both `<xs:attribute name="lang">` (the
/// XHTML 1.0 `lang` attribute) AND `<xs:attribute ref="xml:lang">`
/// (the namespaced `xml:lang` attribute, backwards-compatible per
/// Pemberton et al. 2002 §3.3) — xsd-parser projects both to a
/// `pub lang:` field on the same struct, which is rejected by
/// Rust (E0124, "field already declared").
///
/// We resolve the collision by renaming the second occurrence of
/// the field declaration *within each struct body* to `lang_xml`,
/// preserving a 1-to-1 mapping between XHTML XSD constructs and
/// Rust identifiers. The dedup is per-struct (matched by walking
/// `pub struct ... { ... }` blocks), so unrelated structs that
/// each have a single `lang` field stay unchanged.
fn postprocess_xhtml_collisions(src: &str) -> String {
    // The xsd-parser output is dense (single line of tokens
    // separated by spaces). We can't reliably split by lines.
    // Instead, walk through the source and, for every span between
    // a `pub struct <Name> {` opener and its matching `}`, rewrite
    // the second `pub lang :` declaration to `pub lang_xml :`.
    //
    // The brace counting is on the *opening* run of the struct
    // body — generic parameter lists in xsd-parser output use `<>`,
    // not `{}`, so a simple curly-brace nesting counter handles
    // nested types correctly.
    let mut out = String::with_capacity(src.len());
    let mut rest = src;
    while let Some(idx) = rest.find("pub struct ") {
        // Emit everything up to and including the struct keyword.
        out.push_str(&rest[..idx]);
        rest = &rest[idx..];
        // Find the opening `{` of the struct body.
        let Some(brace_open) = rest.find('{') else {
            out.push_str(rest);
            return out;
        };
        // Find the matching `}`.
        let body_start = brace_open + 1;
        let mut depth = 1;
        let mut body_end = body_start;
        for (i, b) in rest.as_bytes()[body_start..].iter().enumerate() {
            match *b {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        body_end = body_start + i;
                        break;
                    }
                }
                _ => {}
            }
        }
        // Slice the struct header + opener and the body.
        out.push_str(&rest[..body_start]);
        let body = &rest[body_start..body_end];
        out.push_str(&rewrite_lang_collision_in_body(body));
        // Continue after the body (including the closing brace).
        rest = &rest[body_end..];
    }
    out.push_str(rest);
    out
}

/// Inside a single struct body, rename the second (and subsequent)
/// `pub lang :` field-declaration to `pub lang_xml :`. The first
/// occurrence (the XHTML `lang` attribute) keeps its name; later
/// occurrences (the `xml:lang` reference) get the suffix.
fn rewrite_lang_collision_in_body(body: &str) -> String {
    const NEEDLE: &str = "pub lang :";
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    let mut seen = 0usize;
    while let Some(idx) = rest.find(NEEDLE) {
        out.push_str(&rest[..idx]);
        if seen == 0 {
            // Keep the first occurrence.
            out.push_str(NEEDLE);
        } else {
            // Rename the second-and-later occurrences.
            out.push_str("pub lang_xml :");
        }
        rest = &rest[idx + NEEDLE.len()..];
        seen += 1;
    }
    out.push_str(rest);
    out
}

/// Strip every `schemaLocation="..."` attribute. Keeps `namespace="..."`
/// intact so namespace-prefix bindings still work in any
/// `xsd:attribute ref="xml:lang"` cross-namespace reference XHTML
/// makes inside attribute declarations.
fn strip_schema_location_attrs(xsd: &str) -> String {
    let mut out = String::with_capacity(xsd.len());
    let mut rest = xsd;
    while let Some(idx) = rest.find("schemaLocation=") {
        out.push_str(&rest[..idx]);
        let mut start = out.len();
        while start > 0 && out.as_bytes()[start - 1].is_ascii_whitespace() {
            start -= 1;
        }
        out.truncate(start);
        let tail = &rest[idx + "schemaLocation=".len()..];
        let mut bytes = tail.bytes();
        let Some(quote) = bytes.next() else { break };
        if quote != b'"' && quote != b'\'' {
            out.push_str(&rest[idx..]);
            return out;
        }
        let mut end = 1;
        for b in tail.bytes().skip(1) {
            end += 1;
            if b == quote {
                break;
            }
        }
        rest = &tail[end..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cited: W3C XSD 1.1 Part 1 §4.2.3 — `schemaLocation` is an
    /// optional hint, not part of the schema's semantic content.
    /// Stripping it must not alter any element/type declaration.
    #[test]
    fn strip_attrs_preserves_imports_modulo_location() {
        let input = r#"<xs:import namespace="http://www.w3.org/XML/1998/namespace" schemaLocation="http://www.w3.org/2001/xml.xsd"/>"#;
        let stripped = strip_schema_location_attrs(input);
        assert!(stripped.contains("namespace=\"http://www.w3.org/XML/1998/namespace\""));
        assert!(!stripped.contains("schemaLocation"));
    }

    /// The strip pass is the identity on input without
    /// `schemaLocation` attributes.
    #[test]
    fn strip_attrs_identity_on_clean_input() {
        let input =
            "<?xml version=\"1.0\"?><xs:schema xmlns:xs=\"http://www.w3.org/2001/XMLSchema\"/>";
        let stripped = strip_schema_location_attrs(input);
        assert_eq!(stripped, input);
    }

    /// Collision rewrite: within a struct, the second `pub lang :`
    /// becomes `pub lang_xml :`; the first stays put.
    #[test]
    fn postprocess_lang_collision_rewrites_second_in_struct() {
        let input = "pub struct A { pub lang : Option<String> , pub other : i32 , pub lang : Option<String> , }";
        let out = postprocess_xhtml_collisions(input);
        assert!(out.contains("pub lang : Option<String>"));
        assert!(out.contains("pub lang_xml : Option<String>"));
    }

    /// Collision rewrite is idempotent.
    #[test]
    fn postprocess_lang_collision_is_idempotent() {
        let input = "pub struct A { pub lang : Option<String> , pub lang : Option<String> , }";
        let once = postprocess_xhtml_collisions(input);
        let twice = postprocess_xhtml_collisions(&once);
        assert_eq!(once, twice);
    }

    /// Single-occurrence struct is untouched.
    #[test]
    fn postprocess_lang_collision_no_op_when_no_collision() {
        let input = "pub struct A { pub lang : Option<String> , pub other : i32 , }";
        let out = postprocess_xhtml_collisions(input);
        assert_eq!(out, input);
    }

    /// Per-struct scope: two separate structs each with one `lang`
    /// field stay unchanged.
    #[test]
    fn postprocess_lang_collision_scoped_to_struct() {
        let input = "pub struct A { pub lang : Option<String> , } pub struct B { pub lang : Option<String> , }";
        let out = postprocess_xhtml_collisions(input);
        // Both `pub lang :` remain (one per struct).
        let count = out.matches("pub lang :").count();
        assert_eq!(count, 2);
        assert!(!out.contains("lang_xml"));
    }
}
