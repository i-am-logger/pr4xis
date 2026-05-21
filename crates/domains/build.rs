//! Build-time codegen for Static-store sources.
//!
//! For every `[sources.<name>]` entry in workspace-root `praxis.toml`
//! with `type = "Statute"` (and a matching `[structural."<name>@<version>"]`
//! block in `praxis.lock`), this build script runs the praxis statute
//! codegen and writes `$OUT_DIR/<name>_codegen.rs`. The corresponding
//! source module under `social::compliance::statutes::<name>::` includes
//! it via `include!(concat!(env!("OUT_DIR"), "/<name>_codegen.rs"))`.
//!
//! Mirrors the wordnet codegen pattern in `crates/wasm/build.rs`. Skips
//! gracefully if praxis.toml or praxis.lock is missing (downstream
//! consumers of pr4xis-domains as a published crate hit that branch
//! until the published-crate-bundles-its-own-praxis.toml story lands).
//!
//! The structural source of truth is `praxis.lock`; the manifest only
//! enumerates which entries need codegen. Drift between the lock's
//! structural data and the canonical sha256 in `[hashes]` is caught at
//! runtime by `LockManifestAgreement`.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

#[path = "build_helpers/extract_pdf.rs"]
mod extract_pdf;

use extract_pdf::{PdfExtractOutcome, escape_for_raw_string, extract_pdf_to_text};

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
    #[serde(default)]
    structural: HashMap<String, RawStructural>,
}

#[derive(Debug, Deserialize)]
struct RawStructural {
    #[serde(default)]
    description: String,
    #[serde(default)]
    terms: Vec<RawTerm>,
    #[serde(default)]
    relations: Vec<RawRelation>,
}

#[derive(Debug, Deserialize)]
struct RawTerm {
    id: String,
    name: String,
    #[serde(default)]
    definition: String,
    #[serde(default)]
    lemmas: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawRelation {
    from: String,
    to: String,
    relation: String,
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
    println!("cargo:rerun-if-changed=build_helpers/extract_pdf.rs");

    let manifest_text = std::fs::read_to_string(&manifest_path).expect("read praxis.toml");
    let lock_text = std::fs::read_to_string(&lock_path).expect("read praxis.lock");

    let manifest: RawManifest = toml::from_str(&manifest_text).expect("parse praxis.toml");
    let lock: RawLockFile = toml::from_str(&lock_text).expect("parse praxis.lock");

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR is always set during builds");
    let out_dir = PathBuf::from(out_dir);

    let mut sorted_names: Vec<_> = manifest.sources.keys().cloned().collect();
    sorted_names.sort();

    // Single ontological dispatch: every registered source is
    // routed by its ContentType (derived from the SourceTaxonomy
    // kind), not by a hand-edited string list of kinds. New
    // statute leaves are picked up automatically; new ContentTypes
    // need exactly one new arm in `dispatch_codegen`.
    for name in &sorted_names {
        let src = &manifest.sources[name];
        let ct = content_type_for_kind(&src.kind);
        dispatch_codegen(name, src, ct, &workspace_root, &lock, &out_dir);
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

    // M4.ε.5.a — XSD-grounded USLM ontology types. Drives the
    // `uslm::generated` runtime module from the registered
    // `uslm_xsd` source (USLM-1.0.18.xsd, bundled under
    // `crates/domains/data/legal/uscode/schema/`). Per "bottom-up
    // loaded, never encoded", these types replace the hand-coded
    // M4.δ.1–M4.δ.20 ontology — switch + delete come in follow-up
    // commits; the two trees coexist during M4.ε.5.a's add step.
    write_uslm_schema_codegen(&workspace_root, &manifest, &out_dir);

    // M4.η.1 — XSD-grounded HTML5 ontology types. Drives the
    // `social::software::markup::html::generated` runtime module
    // from the registered `xhtml_1_0_xsd` source (XHTML 1.0 Strict,
    // bundled under `crates/domains/data/markup-schemas/xhtml/`).
    // Per "bottom-up loaded, never encoded", element / attribute
    // names come from the W3C-published schema, not from hand-coded
    // Rust string lists.
    write_xhtml_schema_codegen(&workspace_root, &manifest, &out_dir);

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
}

/// Find the registered `uslm_xsd` source in the praxis manifest,
/// resolve its bundled XSD on disk, and invoke
/// `pr4xis::codegen::uslm_schema::generate_uslm_schema_source` to
/// emit `$OUT_DIR/uslm_schema_generated.rs`. On any failure
/// (missing entry, missing file, xsd-parser error) write a
/// commented stub so the runtime `include!` site always resolves.
fn write_uslm_schema_codegen(
    workspace_root: &std::path::Path,
    manifest: &RawManifest,
    out_dir: &std::path::Path,
) {
    let out_path = out_dir.join("uslm_schema_generated.rs");

    let Some(src) = manifest.sources.get("uslm_xsd") else {
        let stub = "// Stub: `uslm_xsd` source not registered in praxis.toml.\n";
        std::fs::write(&out_path, stub).expect("write uslm_schema stub");
        return;
    };

    if src.kind != "XmlSchemaDefinition" {
        let stub = format!(
            "// Stub: `uslm_xsd` is registered as kind {:?}, not XmlSchemaDefinition; \
             skipping XSD codegen.\n",
            src.kind,
        );
        std::fs::write(&out_path, stub).expect("write uslm_schema stub");
        return;
    }

    let xsd_path = workspace_root
        .join("crates/domains/data/legal/uscode/schema")
        .join(format!("uslm-{}.xsd", src.version));

    if !xsd_path.exists() {
        let stub = format!(
            "// Stub: USLM XSD not on disk at {}; skipping XSD codegen.\n",
            xsd_path.display(),
        );
        std::fs::write(&out_path, stub).expect("write uslm_schema stub");
        return;
    }

    println!("cargo:rerun-if-changed={}", xsd_path.display());

    match pr4xis::codegen::uslm_schema::generate_uslm_schema_source(&xsd_path) {
        Ok(source) => {
            let type_count = count_top_level_types(&source);
            std::fs::write(&out_path, source).expect("write uslm_schema codegen");
            eprintln!(
                "Generated USLM XSD ontology: {type_count} top-level types -> {}",
                out_path.display(),
            );
        }
        Err(e) => {
            // xsd-parser failure → emit a commented stub. The
            // runtime tree still compiles; downstream tests that
            // exercise the generated types will fail with clear
            // messages.
            let stub = format!(
                "// xsd-parser codegen failed for {}: {}\n\
                 // The runtime module will compile but contain no USLM types.\n",
                xsd_path.display(),
                e,
            );
            std::fs::write(&out_path, stub).expect("write uslm_schema stub");
            println!("cargo:warning=USLM XSD codegen failed: {e}");
        }
    }
}

/// Find the registered `xhtml_1_0_xsd` source in the praxis manifest,
/// resolve its bundled XSD on disk, and invoke
/// `pr4xis::codegen::xhtml_schema::generate_xhtml_schema_source` to
/// emit `$OUT_DIR/xhtml_schema_generated.rs`. On any failure
/// (missing entry, missing file, xsd-parser error) write a
/// commented stub so the runtime `include!` site always resolves.
fn write_xhtml_schema_codegen(
    workspace_root: &std::path::Path,
    manifest: &RawManifest,
    out_dir: &std::path::Path,
) {
    let out_path = out_dir.join("xhtml_schema_generated.rs");

    let Some(src) = manifest.sources.get("xhtml_1_0_xsd") else {
        let stub = "// Stub: `xhtml_1_0_xsd` source not registered in praxis.toml.\n";
        std::fs::write(&out_path, stub).expect("write xhtml_schema stub");
        return;
    };

    if src.kind != "XmlSchemaDefinition" {
        let stub = format!(
            "// Stub: `xhtml_1_0_xsd` is registered as kind {:?}, not XmlSchemaDefinition; \
             skipping XSD codegen.\n",
            src.kind,
        );
        std::fs::write(&out_path, stub).expect("write xhtml_schema stub");
        return;
    }

    let xsd_path = workspace_root
        .join("crates/domains/data/markup-schemas/xhtml")
        .join(format!("xhtml-{}-strict.xsd", src.version));

    if !xsd_path.exists() {
        let stub = format!(
            "// Stub: XHTML XSD not on disk at {}; skipping XSD codegen.\n",
            xsd_path.display(),
        );
        std::fs::write(&out_path, stub).expect("write xhtml_schema stub");
        return;
    }

    println!("cargo:rerun-if-changed={}", xsd_path.display());

    match pr4xis::codegen::xhtml_schema::generate_xhtml_schema_source(&xsd_path) {
        Ok(source) => {
            let type_count = count_top_level_types(&source);
            std::fs::write(&out_path, source).expect("write xhtml_schema codegen");
            eprintln!(
                "Generated XHTML XSD ontology: {type_count} top-level types -> {}",
                out_path.display(),
            );
        }
        Err(e) => {
            let stub = format!(
                "// xsd-parser codegen failed for {}: {}\n\
                 // The runtime module will compile but contain no XHTML types.\n",
                xsd_path.display(),
                e,
            );
            std::fs::write(&out_path, stub).expect("write xhtml_schema stub");
            println!("cargo:warning=XHTML XSD codegen failed: {e}");
        }
    }
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

/// Count `pub struct` / `pub enum` / `pub type` declarations at
/// the top level of a generated Rust module — best-effort
/// telemetry for the build log.
fn count_top_level_types(source: &str) -> usize {
    source
        .split_inclusive(|c: char| c == ';' || c == '}')
        .filter(|chunk| {
            chunk.contains("pub struct ")
                || chunk.contains("pub enum ")
                || chunk.contains("pub type ")
        })
        .count()
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
    let mut present_paths: Vec<PathBuf> = Vec::new();
    for name in sorted_names {
        let src = &manifest.sources[name];
        if src.kind != "UsCodeTitle" {
            continue;
        }
        let xml_path = expected_usc_title_path(workspace_root, name, &src.version);
        if xml_path.exists() {
            present_paths.push(xml_path);
        }
    }

    let out_path = out_dir.join("usc_corpus_codegen.rs");

    if present_paths.is_empty() {
        let stub = "// Stub: no USC title XML on disk.\n\
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
        return;
    }

    for p in &present_paths {
        println!("cargo:rerun-if-changed={}", p.display());
    }

    let paths: Vec<&std::path::Path> = present_paths.iter().map(|p| p.as_path()).collect();
    let config = pr4xis::codegen::GenerateConfig::with_marker(
        "usc_corpus_codegen",
        "UscEntityId",
        "crate::social::software::markup::xml::uslm::corpus::UsCode",
    );
    let source = pr4xis::codegen::usc_corpus::generate_usc_corpus_source(&paths, &config)
        .expect("generate USC corpus codegen");
    let section_count = source
        .lines()
        .find_map(|l| l.strip_prefix("// Entities: "))
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    std::fs::write(&out_path, source).expect("write usc corpus codegen");
    eprintln!(
        "Generated pr4xis-domains UsCode corpus: {section_count} sections -> {}",
        out_path.display()
    );
}

/// Route a registered source to its codegen path based on its
/// canonical content type. Mirrors
/// `applied::data_provisioning::ontology::canonical_encoding` —
/// duplicated here because build.rs cannot depend on the crate it
/// is building.
fn dispatch_codegen(
    name: &str,
    src: &RawSource,
    ct: Option<&'static str>,
    workspace_root: &std::path::Path,
    lock: &RawLockFile,
    out_dir: &std::path::Path,
) {
    // UslmXml titles are aggregated into the single
    // `usc_corpus_codegen.rs` static by `write_usc_corpus_codegen`;
    // no per-title module is emitted anymore (the legacy
    // `us_code/title_N.rs` shims were deleted in M4.ε.6). XmlLmf is
    // handled by the wasm crate's build.rs; AdobeGlyphList ships as
    // include_str!. Plaintext / Json / Video / Audio / Binary have
    // no codegen yet — they decode lazily at runtime.
    if let Some("Pdf") = ct {
        emit_statute_via_pdf(name, src, workspace_root, lock, out_dir);
    }
}

/// Kind → ContentType mapping (duplicates `canonical_encoding`).
/// Kept as a `&'static str` so build.rs doesn't drag in the
/// pr4xis-domains ContentType enum.
fn content_type_for_kind(kind: &str) -> Option<&'static str> {
    Some(match kind {
        "Language" => "XmlLmf",
        "Statute" | "UsFederalStatute" => "Pdf",
        "UsCodeTitle" => "UslmXml",
        "Regulation" | "ConstitutionalArticle" | "ProceduralRule" | "CaseLaw" => "Pdf",
        "LegalLexicon" => "Json",
        "TypographicGlyphSet" => "AdobeGlyphList",
        // Abstract / non-leaf concepts have no decoder.
        _ => return None,
    })
}

/// Codegen for sources whose canonical content type is PDF — the
/// existing `[structural.*]`-driven path that builds an OntologyBuilder
/// from the lock's hand-curated structural data and emits a
/// per-statute Concept module. Also emits a typed
/// `PdfBuildExtraction` const for the on-disk PDF.
fn emit_statute_via_pdf(
    name: &str,
    src: &RawSource,
    workspace_root: &std::path::Path,
    lock: &RawLockFile,
    out_dir: &std::path::Path,
) {
    let key = format!("{}@{}", name, src.version);
    let Some(structural) = lock.structural.get(&key) else {
        println!(
            "cargo:warning=Statute `{key}` has no [structural.*] block in \
             praxis.lock; skipping codegen for this entry."
        );
        return;
    };

    let mut code = generate_statute_module(name, structural);

    let pdf_path = expected_pdf_path(workspace_root, name, &src.version);
    if pdf_path.exists() {
        println!("cargo:rerun-if-changed={}", pdf_path.display());
    }
    let outcome = extract_pdf_to_text(&pdf_path);
    let extraction_const = emit_pdf_build_extraction_const(&outcome, &pdf_bytes_hash(&pdf_path));
    code.push_str("\n\n");
    code.push_str(&extraction_const);

    let out_path = out_dir.join(format!("{}_codegen.rs", name));
    std::fs::write(&out_path, code).expect("write generated statute module");

    eprintln!(
        "Generated statute `{name}`: {} terms, {} relations, extraction={} -> {}",
        structural.terms.len(),
        structural.relations.len(),
        match &outcome {
            PdfExtractOutcome::Extracted(_) => "Extracted",
            PdfExtractOutcome::NotOnDisk => "NotOnDisk",
            PdfExtractOutcome::ParseFailed(_) => "ParseFailed",
            PdfExtractOutcome::Encrypted => "Encrypted",
        },
        out_path.display()
    );
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

// ---------------------------------------------------------------------------
// Phase 7 helpers — typed PdfBuildExtraction emission
// ---------------------------------------------------------------------------

/// Resolve the expected on-disk PDF path for a registered
/// statute. Mirrors the runtime `RegistryEntry::local_path()` for
/// the `UsFederalStatute` content-type family — but the build
/// script reads the manifest as raw TOML, so the convention is
/// duplicated here (and tested at runtime via the data_provisioning
/// path-shape tests).
fn expected_pdf_path(workspace_root: &std::path::Path, name: &str, version: &str) -> PathBuf {
    workspace_root
        .join("crates/domains/data/legal/statutes/us_federal")
        .join(name)
        .join(format!("{name}-{version}.pdf"))
}

/// SHA-256 of the on-disk PDF as a lowercase hex string. Returns
/// `""` if the file isn't present.
fn pdf_bytes_hash(path: &std::path::Path) -> String {
    let Ok(bytes) = std::fs::read(path) else {
        return String::new();
    };
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Emit the typed const for the codegen module. The output
/// references `crate::applied::data_provisioning::build_extraction::PdfBuildExtraction`
/// — the absolute path lets the emitted code compile regardless
/// of which module-scope the `include!` lands in.
fn emit_pdf_build_extraction_const(outcome: &PdfExtractOutcome, hash: &str) -> String {
    let path = "crate::applied::data_provisioning::build_extraction::PdfBuildExtraction";
    match outcome {
        PdfExtractOutcome::Extracted(text) => format!(
            "pub const PDF_EXTRACTION: {path} = {path}::Extracted {{\n    \
             text: r#\"{}\"#,\n    \
             bytes_hash: \"{}\",\n}};",
            escape_for_raw_string(text),
            hash,
        ),
        PdfExtractOutcome::NotOnDisk => {
            format!("pub const PDF_EXTRACTION: {path} = {path}::NotOnDisk;")
        }
        PdfExtractOutcome::ParseFailed(detail) => format!(
            "pub const PDF_EXTRACTION: {path} = {path}::ParseFailed {{\n    \
             detail: r#\"{}\"#,\n}};",
            escape_for_raw_string(detail),
        ),
        PdfExtractOutcome::Encrypted => {
            format!("pub const PDF_EXTRACTION: {path} = {path}::Encrypted;")
        }
    }
}

// ---------------------------------------------------------------------------
// Codegen — feed structural data into pr4xis::codegen::statute
// ---------------------------------------------------------------------------

fn generate_statute_module(name: &str, structural: &RawStructural) -> String {
    use pr4xis::codegen::statute::{
        RawRelation as PrRawRelation, RawStatuteDoc, RawTerm as PrRawTerm, build_from_doc,
    };
    use pr4xis::codegen::{GenerateConfig, generate_rust};

    let terms: Vec<PrRawTerm> = structural
        .terms
        .iter()
        .map(|t| PrRawTerm {
            id: t.id.clone(),
            name: t.name.clone(),
            definition: t.definition.clone(),
            lemmas: t.lemmas.clone(),
        })
        .collect();

    let relations: Vec<PrRawRelation> = structural
        .relations
        .iter()
        .filter_map(|r| {
            parse_simple_relation(&r.relation).map(|rel| PrRawRelation {
                from: r.from.clone(),
                to: r.to.clone(),
                relation: rel,
            })
        })
        .collect();

    let doc = RawStatuteDoc {
        name: name.to_string(),
        description: structural.description.clone(),
        terms,
        relations,
    };

    let builder = build_from_doc(&doc);
    let pascal = to_pascal_case(name);
    let config = GenerateConfig::new(&format!("{name}_codegen"), &format!("{pascal}Id"));
    generate_rust(&builder, &config)
}

fn parse_simple_relation(s: &str) -> Option<pr4xis::codegen::statute::RawRel> {
    use pr4xis::codegen::statute::RawRel;
    Some(match s {
        "Requires" => RawRel::Requires,
        "SubtypeOf" => RawRel::SubtypeOf,
        "Contradicts" => RawRel::Contradicts,
        "Negates" => RawRel::Negates,
        "AlternativeTo" => RawRel::AlternativeTo,
        "AffirmativeDefenseTo" => RawRel::AffirmativeDefenseTo,
        "SafeHarborFor" => RawRel::SafeHarborFor,
        "ExhaustionRequiredFor" => RawRel::ExhaustionRequiredFor,
        "Precedes" => RawRel::Precedes { max_days: None },
        "Implies" => RawRel::Implies { consequence: None },
        "Composes" => RawRel::Composes { into: None },
        "Triggers" => RawRel::Triggers { obligation: None },
        "Rebuts" => RawRel::Rebuts { burden: None },
        _ => return None,
    })
}

fn to_pascal_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut capitalize_next = true;
    for ch in s.chars() {
        if ch == '_' || ch == '-' {
            capitalize_next = true;
            continue;
        }
        if capitalize_next {
            out.extend(ch.to_uppercase());
            capitalize_next = false;
        } else {
            out.push(ch);
        }
    }
    out
}
