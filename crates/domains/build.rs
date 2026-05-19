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
                    equivalence: &[], causation: &[], references: &[] };\n";
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
    match ct {
        Some("Pdf") => emit_statute_via_pdf(name, src, workspace_root, lock, out_dir),
        Some("UslmXml") => emit_statute_via_uslm(name, src, workspace_root, out_dir),
        // XmlLmf is handled by the wasm crate's build.rs;
        // AdobeGlyphList ships as include_str!. Plaintext / Json /
        // Video / Audio / Binary have no codegen yet — they decode
        // lazily at runtime.
        _ => {}
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

/// Codegen for sources whose canonical content type is USLM XML —
/// the whole-title path. Parses the on-disk title XML, emits every
/// `<section>` as a `StaticStatute` entry in a `pub static SECTIONS`
/// array. The runtime module at
/// `social::compliance::statutes::us_code::{name}` `include!`s the
/// output to expose `section()` / `all_sections()` accessors.
fn emit_statute_via_uslm(
    name: &str,
    src: &RawSource,
    workspace_root: &std::path::Path,
    out_dir: &std::path::Path,
) {
    let xml_path = expected_usc_title_path(workspace_root, name, &src.version);
    if !xml_path.exists() {
        println!(
            "cargo:warning=USC title `{name}@{version}` XML not on disk at \
             {path}; emitting empty codegen stub. Run `pr4xis update {name}` to \
             fetch.",
            version = src.version,
            path = xml_path.display(),
        );
        let stub = format!(
            "// Stub: USC title `{name}@{version}` XML not on disk.\n\
             pub static SECTIONS: &[StaticStatute] = &[];\n",
            version = src.version,
        );
        let out_path = out_dir.join(format!("{name}_codegen.rs"));
        std::fs::write(&out_path, stub).expect("write title stub");
        return;
    }
    println!("cargo:rerun-if-changed={}", xml_path.display());
    let source = pr4xis::codegen::uslm::generate_title_module_source(&xml_path)
        .expect("generate title module source");
    let out_path = out_dir.join(format!("{name}_codegen.rs"));
    let section_count = source.matches("StaticStatute {").count();
    std::fs::write(&out_path, source).expect("write title codegen");
    eprintln!(
        "Generated USC title `{name}`: {section_count} sections -> {}",
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
