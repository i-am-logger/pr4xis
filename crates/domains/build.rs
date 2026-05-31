//! Build-time codegen for the U.S. Code USLM XML corpus.
//!
//! Walks workspace-root `praxis.toml` for `[sources.<name>]` entries
//! with `type = "UsCodeTitle"`. For each on-disk title XML, parses
//! the USLM and emits the aggregated codegen output consumed at
//! runtime by `social::software::markup::xml::uslm::corpus::loaded()`.
//!
//! Statutes (individual USC sections — e.g. 18 U.S.C. § 1514A,
//! 49 U.S.C. § 42121) are slices of the loaded title corpus,
//! addressable by USLM URN via `UsCode::section_by_urn`. They are
//! never separate `[sources.*]` entries.
//!
//! Skips gracefully if praxis.toml or praxis.lock is missing
//! (downstream consumers of pr4xis-domains as a published crate hit
//! that branch until the published-crate-bundles-its-own-praxis.toml
//! story lands).
//!
//! Citation: 1 U.S.C. § 204 (Code authority); LRC, *USLM XML User
//! Guide* §V (USC URN hierarchy); W3C XML Schema 1.1 Part 1 (Gao,
//! Sperberg-McQueen & Thompson 2012).

use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

// ---------------------------------------------------------------------------
// praxis.toml parsing — minimal, build-time only
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct RawManifest {
    #[serde(default)]
    sources: HashMap<String, RawSource>,
}

#[derive(Debug, Deserialize)]
struct RawSource {
    version: String,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    #[allow(dead_code)]
    url: String,
    #[serde(default)]
    #[allow(dead_code)]
    description: Option<String>,
}

// ---------------------------------------------------------------------------
// praxis.lock parsing — minimal, build-time only
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct RawLockFile {
    #[serde(default)]
    #[allow(dead_code)]
    hashes: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// Build entry point
// ---------------------------------------------------------------------------

fn main() {
    // CARGO_MANIFEST_DIR is crates/domains/. Workspace root is two levels up.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is always set during builds");
    let workspace_root: PathBuf = PathBuf::from(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/domains has at least two ancestor directories")
        .to_path_buf();

    let manifest_path = workspace_root.join("praxis.toml");
    let lock_path = workspace_root.join("praxis.lock");

    if !manifest_path.exists() || !lock_path.exists() {
        println!(
            "cargo:warning=praxis.toml or praxis.lock not found at workspace root \
             ({}); skipping statute codegen.",
            workspace_root.display()
        );
        return;
    }

    println!("cargo:rerun-if-changed={}", manifest_path.display());
    println!("cargo:rerun-if-changed={}", lock_path.display());
    println!("cargo:rerun-if-changed=build.rs");

    let manifest_text = std::fs::read_to_string(&manifest_path).expect("read praxis.toml");
    let lock_text = std::fs::read_to_string(&lock_path).expect("read praxis.lock");

    let manifest: RawManifest = toml::from_str(&manifest_text).expect("parse praxis.toml");
    let _lock: RawLockFile = toml::from_str(&lock_text).expect("parse praxis.lock");

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR is always set during builds");
    let out_dir = PathBuf::from(out_dir);

    let mut sorted_names: Vec<_> = manifest.sources.keys().cloned().collect();
    sorted_names.sort();

    // Per-source dispatch (currently a no-op): UsCodeTitle sources are
    // aggregated by `write_usc_corpus_codegen`; XSD sources have their
    // own writers below; Language is handled by the wasm crate's
    // build.rs. New ContentTypes that need per-source codegen add an
    // arm in `dispatch_codegen` and a writer.
    for name in &sorted_names {
        let src = &manifest.sources[name];
        let ct = content_type_for_kind(&src.kind);
        dispatch_codegen(name, src, ct, &workspace_root, &out_dir);
    }

    // After per-title codegen, emit a single aggregate
    // `CodegenData<UsCode>` static spanning every registered
    // UsCodeTitle whose XML is on disk. Mirrors the cli/wasm
    // build.rs `write_usc_corpus_codegen` block — duplicated here
    // so pr4xis-domains tests can materialise a real ~2770-section
    // corpus without depending on the cli's OUT_DIR. The runtime
    // module at `social::software::markup::xml::uslm::corpus`
    // exposes a cached `from_codegen_output()` test helper that
    // `include!`s this file.
    write_usc_corpus_codegen(&workspace_root, &manifest, &sorted_names, &out_dir);

    // (Removed: write_uslm_schema_codegen + write_xhtml_schema_codegen.
    // Both depended on Sebastian Bergmann's xsd-parser crate to emit
    // XSD-faithful Rust types. The runtime path uses the praxis-native
    // XSD projector (`formal::meta::xsd::from_xml::project_from_xml_document`)
    // for dispatch instead; the emitted Rust types were never wired up
    // at runtime. See chore/drop-lopdf-xsd-parser.)

    // M4.η.2 — XML 1.0 ontology grounding sources. Two outputs:
    //
    // - `xml_namespace_schema_generated.rs`: the four `xml:*`
    //   reserved attribute names loaded from the W3C xml.xsd at
    //   `crates/domains/data/markup-schemas/xml/xml.xsd`.
    // - `xml_infoset_generated.rs`: the 11 information items loaded
    //   from the W3C XML Information Set rec at
    //   `crates/domains/data/markup-schemas/xml/xml-infoset.xhtml`.
    //
    // Per "bottom-up loaded, never encoded", every name comes from
    // a registered authoritative source — not from hand-coded Rust
    // enum variants or string lists.
    write_xml_namespace_schema_codegen(&workspace_root, &manifest, &out_dir);
    write_xml_infoset_codegen(&workspace_root, &manifest, &out_dir);

    // M5.ε.2 — XML 1.0 grammar productions from the loaded spec
    // (`xml_1_0_fifth_edition@2008`, Bray et al. 2008). Parses the
    // 86 `<prod>` blocks and emits range tables + predicates for
    // §2.2 Char, §2.3 NameStartChar, §2.3 NameChar. The parser
    // (`parser::grammar`) includes the generated module instead of
    // hand-coding the code-point ranges as primitive `matches!`
    // arms, per `feedback_bottom_up_loaded_not_encoded`.
    write_xml_grammar_codegen(&workspace_root, &manifest, &out_dir);
}

/// Find the registered `xml_1_0_namespace_xsd` source in the praxis
/// manifest, resolve its bundled xml.xsd on disk, and invoke
/// `pr4xis::codegen::xml_schemas::generate_xml_namespace_schema_source`
/// to emit `$OUT_DIR/xml_namespace_schema_generated.rs`. On any
/// failure (missing entry, missing file, scan error) write a
/// commented stub so the runtime `include!` site always resolves.
fn write_xml_namespace_schema_codegen(
    workspace_root: &std::path::Path,
    manifest: &RawManifest,
    out_dir: &std::path::Path,
) {
    let out_path = out_dir.join("xml_namespace_schema_generated.rs");

    let Some(src) = manifest.sources.get("xml_1_0_namespace_xsd") else {
        let stub = "// Stub: `xml_1_0_namespace_xsd` source not registered in praxis.toml.\n\
             pub const XML_NAMESPACE_ATTRIBUTES: &[&str] = &[];\n";
        std::fs::write(&out_path, stub).expect("write xml_namespace_schema stub");
        return;
    };

    if src.kind != "XmlSchemaDefinition" {
        let stub = format!(
            "// Stub: `xml_1_0_namespace_xsd` is registered as kind {:?}, not XmlSchemaDefinition; \
             skipping XML namespace codegen.\n\
             pub const XML_NAMESPACE_ATTRIBUTES: &[&str] = &[];\n",
            src.kind,
        );
        std::fs::write(&out_path, stub).expect("write xml_namespace_schema stub");
        return;
    }

    // xml.xsd is bundled at a fixed name (not `<name>-<version>.xsd`
    // like the per-corpus XSDs) — the W3C-published file is just
    // `xml.xsd` and that's the convention every consumer follows.
    let xsd_path = workspace_root.join("crates/domains/data/markup-schemas/xml/xml.xsd");

    if !xsd_path.exists() {
        let stub = format!(
            "// Stub: XML namespace XSD not on disk at {}; skipping codegen.\n\
             pub const XML_NAMESPACE_ATTRIBUTES: &[&str] = &[];\n",
            xsd_path.display(),
        );
        std::fs::write(&out_path, stub).expect("write xml_namespace_schema stub");
        return;
    }

    println!("cargo:rerun-if-changed={}", xsd_path.display());

    match pr4xis::codegen::xml_schemas::generate_xml_namespace_schema_source(&xsd_path) {
        Ok(source) => {
            let attr_count = source.matches("    \"").count();
            std::fs::write(&out_path, source).expect("write xml_namespace_schema codegen");
            eprintln!(
                "Generated XML namespace XSD inventory: {attr_count} reserved attribute names -> {}",
                out_path.display(),
            );
        }
        Err(e) => {
            let stub = format!(
                "// XML namespace XSD codegen failed for {}: {}\n\
                 pub const XML_NAMESPACE_ATTRIBUTES: &[&str] = &[];\n",
                xsd_path.display(),
                e,
            );
            std::fs::write(&out_path, stub).expect("write xml_namespace_schema stub");
            println!("cargo:warning=XML namespace XSD codegen failed: {e}");
        }
    }
}

/// Find the registered `xml_infoset` source in the praxis manifest,
/// resolve its bundled XHTML on disk, and invoke
/// `pr4xis::codegen::xml_schemas::generate_xml_infoset_source` to
/// emit `$OUT_DIR/xml_infoset_generated.rs`. On any failure (missing
/// entry, missing file, scan error) write a commented stub.
fn write_xml_infoset_codegen(
    workspace_root: &std::path::Path,
    manifest: &RawManifest,
    out_dir: &std::path::Path,
) {
    let out_path = out_dir.join("xml_infoset_generated.rs");

    let stub_decl = "/// Stub variant of `InformationItemEntry` for the case where the \
                     bundled rec is missing.\n\
                     #[derive(Debug, Clone, Copy, PartialEq, Eq)]\n\
                     pub struct InformationItemEntry {\n    \
                        pub section: &'static str,\n    \
                        pub anchor: &'static str,\n    \
                        pub english_name: &'static str,\n    \
                        pub variant_ident: &'static str,\n\
                     }\n\
                     pub const XML_INFOSET_INFORMATION_ITEMS: &[InformationItemEntry] = &[];\n";

    let Some(src) = manifest.sources.get("xml_infoset") else {
        let stub =
            format!("// Stub: `xml_infoset` source not registered in praxis.toml.\n{stub_decl}",);
        std::fs::write(&out_path, stub).expect("write xml_infoset stub");
        return;
    };

    if src.kind != "ConceptualSpec" {
        let stub = format!(
            "// Stub: `xml_infoset` is registered as kind {:?}, not ConceptualSpec; \
             skipping infoset codegen.\n{stub_decl}",
            src.kind,
        );
        std::fs::write(&out_path, stub).expect("write xml_infoset stub");
        return;
    }

    let xhtml_path =
        workspace_root.join("crates/domains/data/markup-schemas/xml/xml-infoset.xhtml");

    if !xhtml_path.exists() {
        let stub = format!(
            "// Stub: XML Information Set rec not on disk at {}; skipping codegen.\n{stub_decl}",
            xhtml_path.display(),
        );
        std::fs::write(&out_path, stub).expect("write xml_infoset stub");
        return;
    }

    println!("cargo:rerun-if-changed={}", xhtml_path.display());

    match pr4xis::codegen::xml_schemas::generate_xml_infoset_source(&xhtml_path) {
        Ok(source) => {
            let item_count = source.matches("InformationItemEntry {").count();
            std::fs::write(&out_path, source).expect("write xml_infoset codegen");
            eprintln!(
                "Generated XML Information Set inventory: {item_count} information items -> {}",
                out_path.display(),
            );
        }
        Err(e) => {
            let stub = format!(
                "// XML Infoset codegen failed for {}: {}\n{stub_decl}",
                xhtml_path.display(),
                e,
            );
            std::fs::write(&out_path, stub).expect("write xml_infoset stub");
            println!("cargo:warning=XML Infoset codegen failed: {e}");
        }
    }
}

/// Find the registered `xml_1_0_fifth_edition` source in the praxis
/// manifest, resolve its bundled XML on disk, and invoke
/// `pr4xis::codegen::xml_grammar::generate_xml_grammar_source` to
/// emit `$OUT_DIR/xml_grammar_generated.rs`. On any failure (missing
/// entry, missing file, RHS parse error) write a commented stub
/// that satisfies the runtime `include!` site with zero-range
/// tables (`is_char` etc. return `false` for everything) — that
/// makes any consumer of the predicate report failure clearly
/// rather than silently using a fallback.
fn write_xml_grammar_codegen(
    workspace_root: &std::path::Path,
    manifest: &RawManifest,
    out_dir: &std::path::Path,
) {
    let out_path = out_dir.join("xml_grammar_generated.rs");

    // Stub declarations so consumers compile when the spec is absent.
    let stub_decl = "#[allow(dead_code)]\n\
                     pub const CHAR_RANGES: &[(u32, u32)] = &[];\n\
                     #[allow(dead_code)]\n\
                     pub const NAME_START_CHAR_RANGES: &[(u32, u32)] = &[];\n\
                     #[allow(dead_code)]\n\
                     pub const NAME_CHAR_RANGES: &[(u32, u32)] = &[];\n\
                     #[must_use]\n#[allow(dead_code)]\n\
                     pub fn is_char(_c: u32) -> bool { false }\n\
                     #[must_use]\n#[allow(dead_code)]\n\
                     pub fn is_name_start_char(_c: u32) -> bool { false }\n\
                     #[must_use]\n#[allow(dead_code)]\n\
                     pub fn is_name_char(_c: u32) -> bool { false }\n";

    let Some(src) = manifest.sources.get("xml_1_0_fifth_edition") else {
        let stub = format!(
            "// Stub: `xml_1_0_fifth_edition` source not registered in praxis.toml.\n{stub_decl}",
        );
        std::fs::write(&out_path, stub).expect("write xml_grammar stub");
        return;
    };

    if src.kind != "ConceptualSpec" {
        let stub = format!(
            "// Stub: `xml_1_0_fifth_edition` is registered as kind {:?}, not ConceptualSpec; \
             skipping grammar codegen.\n{stub_decl}",
            src.kind,
        );
        std::fs::write(&out_path, stub).expect("write xml_grammar stub");
        return;
    }

    let spec_path = workspace_root
        .join("crates/domains/data/markup-schemas/xml")
        .join(format!("xml_1_0_fifth_edition-{}.xml", src.version));

    if !spec_path.exists() {
        let stub = format!(
            "// Stub: XML 1.0 Fifth Edition spec not on disk at {}; skipping codegen.\n{stub_decl}",
            spec_path.display(),
        );
        std::fs::write(&out_path, stub).expect("write xml_grammar stub");
        return;
    }

    println!("cargo:rerun-if-changed={}", spec_path.display());

    match pr4xis::codegen::xml_grammar::generate_xml_grammar_source(&spec_path) {
        Ok(source) => {
            let range_count = source.matches("(0x").count();
            std::fs::write(&out_path, source).expect("write xml_grammar codegen");
            eprintln!(
                "Generated XML 1.0 grammar predicates: {range_count} code-point ranges -> {}",
                out_path.display(),
            );
        }
        Err(e) => {
            let stub = format!(
                "// XML grammar codegen failed for {}: {}\n{stub_decl}",
                spec_path.display(),
                e,
            );
            std::fs::write(&out_path, stub).expect("write xml_grammar stub");
            println!("cargo:warning=XML grammar codegen failed: {e}");
        }
    }
}

/// Walk every registered USC title's expected XML path, collect
/// the ones that exist on disk, and emit a `CodegenData<UsCode>`
/// static at `$OUT_DIR/usc_corpus_codegen.rs`. If no titles are
/// on disk, write an empty stub so the `include!` site always
/// resolves.
fn write_usc_corpus_codegen(
    workspace_root: &std::path::Path,
    manifest: &RawManifest,
    sorted_names: &[String],
    out_dir: &std::path::Path,
) {
    // M4.δ.7.a: build-time USC corpus codegen has been retired. The
    // runtime constructor `UsCode::from_uslm_titles_owned` in
    // `social/software/markup/xml/uslm/corpus/mod.rs` reads + parses
    // the registered title XMLs at first call to `loaded()`, mirroring
    // the WordNet pattern (`English::cached`). This eliminates the
    // ~85 MB aggregate Rust source that was hitting rustc's compile-
    // time memory ceiling, and unblocks arbitrary-sized titles
    // (Title 42 at 113 MB, etc.).
    //
    // We still emit a stub `usc_corpus_codegen.rs` because the runtime
    // `corpus/mod.rs` module includes it (the include is kept as a
    // soft compatibility boundary so `UsCode::sample()` and the
    // codegen-data-shape tests still see an empty `CODEGEN_DATA` /
    // `USC_SECTION_AUX`). A follow-up commit can delete the include
    // and the stub once those tests migrate to runtime sample fixtures.
    //
    // The `cargo:rerun-if-changed=` directives are still emitted so a
    // future rebuild picks up XML changes — even though the runtime
    // path doesn't use them, downstream tooling (incremental cargo,
    // rust-analyzer) benefits from accurate dependency tracking.
    for name in sorted_names {
        let src = &manifest.sources[name];
        if src.kind != "UsCodeTitle" {
            continue;
        }
        let xml_path = expected_usc_title_path(workspace_root, name, &src.version);
        if xml_path.exists() {
            println!("cargo:rerun-if-changed={}", xml_path.display());
        }
    }

    let out_path = out_dir.join("usc_corpus_codegen.rs");
    let stub = "// Empty stub — M4.δ.7.a retired build-time USC corpus codegen \
                in favor of runtime XML loading via `UsCode::loaded()`. \
                See docs/m4-delta-7-a-design.md.\n\
                pub static CODEGEN_DATA: pr4xis::codegen_data::CodegenData<\
                crate::social::software::markup::xml::uslm::corpus::UsCode> = \
                pr4xis::codegen_data::CodegenData { \
                entity_count: 0, entity_ids: &[], entity_kind: &[], \
                entity_labels: &[], entity_defs: &[], word_index: &[], \
                taxonomy: &[], mereology: &[], opposition: &[], \
                equivalence: &[], causation: &[], references: &[] };\n\
                pub static USC_SECTION_AUX: \
                &[crate::social::software::markup::xml::uslm::corpus::UscSectionAux] = &[];\n";
    std::fs::write(&out_path, stub).expect("write usc corpus stub");
    eprintln!(
        "pr4xis-domains USC corpus codegen: retired (M4.δ.7.a) — runtime loader in corpus/mod.rs handles all titles"
    );
}

/// Route a registered source to its codegen path based on its
/// canonical content type. Currently every active path is handled
/// outside this function (UsCodeTitle by `write_usc_corpus_codegen`,
/// XSD by the XSD writers, Language by the wasm crate's build.rs,
/// AdobeGlyphList via `include_str!`), so this is effectively a
/// no-op left as the dispatch hook for future per-source writers.
fn dispatch_codegen(
    _name: &str,
    _src: &RawSource,
    _ct: Option<&'static str>,
    _workspace_root: &std::path::Path,
    _out_dir: &std::path::Path,
) {
}

/// Kind → ContentType mapping (duplicates `canonical_encoding`).
/// Kept as a `&'static str` so build.rs doesn't drag in the
/// pr4xis-domains ContentType enum.
fn content_type_for_kind(kind: &str) -> Option<&'static str> {
    Some(match kind {
        "Language" => "XmlLmf",
        "UsCodeTitle" => "UslmXml",
        "ProceduralRule" | "CaseLaw" => "Pdf",
        "LegalLexicon" => "Json",
        "TypographicGlyphSet" => "AdobeGlyphList",
        // XmlSchemaDefinition / ConceptualSpec sources are routed by
        // their per-source writers (`write_xml_namespace_schema_codegen`,
        // `write_xml_infoset_codegen`, etc.), not by `dispatch_codegen`.
        // Abstract / non-leaf concepts also fall through.
        _ => return None,
    })
}

/// Resolve the expected on-disk USLM XML path for a registered
/// USC title. Mirrors the runtime `RegistryEntry::local_path()`
/// for the `UsCodeTitle` kind (`legal/uscode/<name>/<name>-<version>.xml`).
fn expected_usc_title_path(workspace_root: &std::path::Path, name: &str, version: &str) -> PathBuf {
    workspace_root
        .join("crates/domains/data/legal/uscode")
        .join(name)
        .join(format!("{name}-{version}.xml"))
}
