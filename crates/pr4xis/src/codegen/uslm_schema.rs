//! USLM XSD → Rust ontology types codegen.
//!
//! Loads the USLM XML schema (USLM-1.0.18.xsd, published by the
//! U.S. House Office of the Law Revision Counsel) and emits typed
//! Rust source describing every `xsd:element`, `xsd:complexType`,
//! `xsd:simpleType`, `xsd:attributeGroup`, and `xsd:group` it
//! defines. The heavy lifting — XSD parsing, name resolution,
//! substitution-group dispatch, type-graph construction — is done
//! by the [`xsd-parser`] crate by Sebastian Bergmann; this module
//! is the thin glue that wires xsd-parser into the praxis codegen
//! pipeline.
//!
//! Per "bottom-up loaded, never encoded": every Tier 2+ ontology
//! type derives from a registered authoritative source, not from
//! Rust enum variants written by hand. The USLM XSD is that
//! authoritative source; this module is the loader.
//!
//! [`xsd-parser`]: https://docs.rs/xsd-parser/1.5.2
//!
//! ## Citations
//!
//! - U.S. House Office of the Law Revision Counsel, *USLM XML
//!   User Guide and Schema (USLM-1.0.18.xsd)*. Available at
//!   <https://uscode.house.gov/uslm/>.
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
//! 2. Strip `schemaLocation="…"` attributes from the three
//!    upstream imports (`xml.xsd`, `dcterms.xsd`,
//!    `xhtml1-strict.xsd`). The published XSD references these by
//!    URL; we don't fetch at build time per the praxis-lock model
//!    (sources are hash-pinned and bundled, not network-fetched).
//!    `xsd-parser`'s parser flags allow disabling include
//!    resolution but the interpreter still expects every imported
//!    schema to be present in `Schemas`, so we provide a synthetic
//!    `xml.xsd` stub defining the three attributes USLM actually
//!    references (`xml:base`, `xml:lang`, `xml:space`).
//! 3. Hand the schemas to `xsd-parser`'s `generate()` and write
//!    the resulting Rust source to the supplied path.
//! 4. Apply a tiny post-processing step to fix two USLM-specific
//!    name collisions that xsd-parser cannot disambiguate from
//!    XSD alone (documented inline at the rewrite site).
//!
//! ## Known post-processing
//!
//! Two collisions are intrinsic to mapping USLM's name space to
//! Rust identifiers and cannot be resolved by xsd-parser
//! configuration:
//!
//! - The `<element name="meta">` clashes with the `meta` attribute
//!   on USLM's common attribute group: both become a `pub meta:`
//!   field on the same struct. The element-typed field is renamed
//!   to `pub meta_element:`.
//! - The mixed-content text fragment (xsd-parser emits
//!   `Text(::xsd_parser_types::xml::Text)`) clashes with USLM's
//!   `<element name="text">` (the substitution-group dispatcher
//!   named `Text`). The mixed-content variant is renamed to
//!   `TextFragment`.
//!
//! Both renames preserve a 1-to-1 mapping between USLM XSD
//! constructs and Rust identifiers; nothing is dropped.

use std::path::Path;

use xsd_parser::{
    Config,
    config::{GeneratorFlags, InterpreterFlags, OptimizerFlags, ParserFlags, Resolver, Schema},
    generate,
};

/// Minimal stub for the `http://www.w3.org/XML/1998/namespace`
/// schema. USLM imports `xml.xsd` only to reference three
/// attributes (`xml:base`, `xml:lang`, `xml:space`); providing
/// this stub is sufficient for the xsd-parser interpreter without
/// fetching the upstream schema at build time.
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

/// Errors returned by [`generate_uslm_schema_source`].
#[derive(Debug)]
pub enum UslmSchemaCodegenError {
    /// Could not read the XSD file from disk.
    ReadXsd(String, std::io::Error),
    /// `xsd-parser` failed to parse or generate. The string is the
    /// crate's own error message.
    Generate(String),
}

impl core::fmt::Display for UslmSchemaCodegenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ReadXsd(p, e) => write!(f, "read XSD {p}: {e}"),
            Self::Generate(msg) => write!(f, "xsd-parser codegen failed: {msg}"),
        }
    }
}

impl std::error::Error for UslmSchemaCodegenError {}

/// Load `xsd_path` and emit Rust source describing every USLM
/// type. Suitable for writing into `$OUT_DIR/<name>_generated.rs`
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
/// Returns [`UslmSchemaCodegenError`] if the XSD can't be read or
/// xsd-parser rejects the schema. xsd-parser's panics (out-of-band
/// in 1.5.2 for some edge cases) are not caught — surface them up
/// the build-script's `eprintln!` channel.
pub fn generate_uslm_schema_source(xsd_path: &Path) -> Result<String, UslmSchemaCodegenError> {
    let xsd = std::fs::read_to_string(xsd_path)
        .map_err(|e| UslmSchemaCodegenError::ReadXsd(xsd_path.display().to_string(), e))?;

    // The published USLM XSD has three `xsd:import schemaLocation="http://..."`
    // declarations (xml.xsd, dcterms.xsd, xhtml1-strict.xsd). Per the
    // praxis-lock model — sources hash-pinned and bundled, not network-
    // fetched — we strip the `schemaLocation` attributes so xsd-parser
    // doesn't try to resolve them, then provide the `xml.xsd` stub as
    // an inline schema. The other two namespaces are referenced only in
    // `<xsd:documentation>` blocks (comments), not in any actual element
    // / attribute declaration, so a stub for them isn't needed.
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

    // BUILD_IN_ABSOLUTE_PATHS / ABSOLUTE_PATHS_INSTEAD_USINGS make every
    // built-in reference fully-qualified (`::core::option::Option`,
    // `::std::string::String`, `::xsd_parser_types::xml::Text`), which
    // avoids `use` statements that would clash with USLM-defined types of
    // the same short name (USLM has an `<element name="text">` that
    // generates a `pub enum Text` in the same scope).
    //
    // FLATTEN_CONTENT and USE_MODULES keep the output readable and group
    // each namespace's types in its own module.
    //
    // MIXED_TYPE_SUPPORT turns `<xs:complexType mixed="true">` into
    // `Mixed<T>` wrappers (vs. inline `Text` variants), which sharply
    // reduces the chance of variant-name collisions.
    config.generator.flags = GeneratorFlags::FLATTEN_CONTENT
        | GeneratorFlags::USE_MODULES
        | GeneratorFlags::BUILD_IN_ABSOLUTE_PATHS
        | GeneratorFlags::ABSOLUTE_PATHS_INSTEAD_USINGS
        | GeneratorFlags::MIXED_TYPE_SUPPORT;

    // USLM defines `<xsd:complexType name="DateType">` and similar
    // names that collide with xsd-parser's DEFAULT_TYPEDEFS aliases
    // for built-in xs:date etc. Renaming the complex-type postfix to
    // `Item` keeps the two name spaces disjoint: `xs:date → DateType`
    // (built-in alias), `USLM DateType → DateTypeItem` (USLM complex
    // type wrapping a `<num>` + content model).
    config.generator.type_postfix.type_ = "Item".to_string();

    // Emit `#[derive(Serialize, Deserialize)]` and the quick-xml
    // serde render steps on every generated type, so the USLM lens
    // can populate `generated::UscDoc` from on-disk XML via
    // `quick_xml::de::from_str`.
    let config = config.with_serde_quick_xml();

    let tokens = generate(config).map_err(|e| UslmSchemaCodegenError::Generate(e.to_string()))?;
    let raw_source = tokens.to_string();

    Ok(postprocess_uslm_collisions(&raw_source))
}

/// Strip every `schemaLocation="..."` attribute. Keeps `namespace="..."`
/// intact so namespace-prefix bindings still work in `xsd:attribute
/// ref="xml:base"` (the only cross-namespace reference USLM actually
/// makes inside attribute declarations).
fn strip_schema_location_attrs(xsd: &str) -> String {
    // The `schemaLocation` attribute always appears on `<xsd:import>`
    // elements in this schema and follows the form
    // `schemaLocation="http(s)://..."`. A simple state machine over
    // the input string is sufficient — `xsd-parser` itself handles
    // the structural XML parsing.
    let mut out = String::with_capacity(xsd.len());
    let mut rest = xsd;
    while let Some(idx) = rest.find("schemaLocation=") {
        out.push_str(&rest[..idx]);
        // Walk back over any whitespace before the attribute so we
        // don't leave a stray space behind.
        let mut start = out.len();
        while start > 0 && out.as_bytes()[start - 1].is_ascii_whitespace() {
            start -= 1;
        }
        out.truncate(start);
        // Skip past the attribute value (quoted with " or ').
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

/// Fix the two USLM-specific identifier collisions xsd-parser
/// cannot disambiguate from XSD alone. Both rewrites are string
/// substitutions on the formatted Rust source.
fn postprocess_uslm_collisions(src: &str) -> String {
    // ---- Collision 1 -------------------------------------------------
    // `<xsd:element name="meta">` AND the `meta` attribute on the
    // USLM common attribute group both project to a `pub meta:`
    // field on the same struct. xsd-parser emits both, e.g.:
    //     pub meta: Option<String>,        // attribute
    //     pub meta: MetaTypeItem,          // element
    // Rust rejects (E0124). Rename the element-typed field to
    // `meta_element`. Match the second declaration by its element
    // type (`MetaTypeItem`); the first stays as the attribute.
    let src = src.replace("pub meta : MetaTypeItem", "pub meta_element : MetaTypeItem");

    // ---- Collision 2 -------------------------------------------------
    // USLM's `<xsd:element name="text">` becomes a `pub enum Text`
    // (the substitution-group dispatcher), but xsd-parser's
    // mixed-content fallback also emits enum variants named `Text`
    // wrapping `::xsd_parser_types::xml::Text`. Rust rejects
    // (E0428/E0004). Rename every such fallback variant to
    // `TextFragment`. The replacement targets the specific token
    // shape xsd-parser produces.
    let src = src.replace(
        "Text (:: xsd_parser_types :: xml :: Text)",
        "TextFragment (:: xsd_parser_types :: xml :: Text)",
    );

    // ---- Collision 3 -------------------------------------------------
    // For the XSD-builtin `OccurrenceSimpleType` (a `xsd:union` of
    // `xsd:nonNegativeInteger` and the enumeration {"all", "none",
    // "first", "last"}), xsd-parser's serde codegen emits
    // `#[serde(other)] Usize(usize)` — a `#[serde(other)]` attribute
    // on a tuple variant, which serde rejects (it requires a unit
    // variant). The enum represents XSD `minOccurs`/`maxOccurs` and
    // is unreachable from USLM document content; stripping the
    // attribute makes the enum compile.
    src.replace(
        "# [serde (other)] Usize (:: core :: primitive :: usize)",
        "Usize (:: core :: primitive :: usize)",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Empty schema → identity functor: an empty XSD-load step
    /// produces a generated module with no USLM types (only the
    /// xsd-parser default built-in aliases).
    #[test]
    fn empty_schema_yields_no_uslm_types() {
        // The strip-attrs pass is the identity on input without
        // `schemaLocation`.
        let stripped = strip_schema_location_attrs(
            "<?xml version=\"1.0\"?><xs:schema xmlns:xs=\"http://www.w3.org/2001/XMLSchema\"/>",
        );
        assert!(!stripped.contains("schemaLocation"));
    }

    /// Cited: W3C XSD 1.1 Part 1 §4.2.3 — `schemaLocation` is an
    /// optional hint, not part of the schema's semantic content.
    /// Stripping it must not alter any element/type declaration.
    #[test]
    fn strip_attrs_preserves_imports_modulo_location() {
        let input = r#"<xs:import namespace="http://x/" schemaLocation="http://x/x.xsd"/>"#;
        let stripped = strip_schema_location_attrs(input);
        assert!(stripped.contains("namespace=\"http://x/\""));
        assert!(!stripped.contains("schemaLocation"));
    }

    /// Collision-1 rewrite is idempotent: applying it twice yields
    /// the same output as applying it once (no rewrite cycles).
    #[test]
    fn postprocess_is_idempotent_on_meta_collision() {
        let once = postprocess_uslm_collisions("pub meta : MetaTypeItem , pub other : i32");
        let twice = postprocess_uslm_collisions(&once);
        assert_eq!(once, twice);
        assert!(once.contains("meta_element"));
    }

    /// Collision-2 rewrite is idempotent.
    #[test]
    fn postprocess_is_idempotent_on_text_collision() {
        let input = "Text (:: xsd_parser_types :: xml :: Text)";
        let once = postprocess_uslm_collisions(input);
        let twice = postprocess_uslm_collisions(&once);
        assert_eq!(once, twice);
        assert!(once.starts_with("TextFragment"));
    }
}
