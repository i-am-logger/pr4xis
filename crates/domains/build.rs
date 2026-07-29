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
//! The manifest is read from the workspace-root `praxis.toml`/`praxis.lock`
//! when present (the live source of truth), else from the committed registry
//! MANIFEST `.prx` (`data/registry/praxis-registry.prx`) — the SAME committed
//! artifact the runtime registry loads (`registry_prx::load_registry_manifest`).
//! So the PUBLISHED crate, unpacked with no workspace root, builds its codegen
//! from the `.prx`, never from a raw-TOML snapshot (it ships none). build.rs
//! emits NO `praxis_embed.rs` `&str` const — the runtime registry is sourced
//! from the `.prx` directly.
//!
//! Citation: 1 U.S.C. § 204 (Code authority); LRC, *USLM XML User
//! Guide* §V (USC URN hierarchy); W3C XML Schema 1.1 Part 1 (Gao,
//! Sperberg-McQueen & Thompson 2012).

use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

// The ONE `.prx` envelope codec — LEB128 framing, the raw-source + registry
// envelope grammars, the DEFLATE decompression-bomb guard, and the blake3
// content-address gate helpers — shared byte-for-byte with the runtime load path
// (`applied::data_provisioning::{raw_source_prx, registry_prx}`). `#[path]`-
// included (NOT a separate crate) so the build script and the lib compile the
// SAME source: the two envelope decoders are literally one decoder now, not a
// hand-kept mirror. The build script uses a subset (decode + blake3 gate; not the
// encoders), hence `allow(dead_code)`.
#[path = "src/applied/data_provisioning/prx_envelope.rs"]
#[allow(dead_code)]
mod prx_envelope;

// ---------------------------------------------------------------------------
// praxis.toml parsing — minimal, build-time only
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize)]
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

#[derive(Debug, Default, Deserialize)]
struct RawLockFile {
    #[serde(default)]
    #[allow(dead_code)]
    hashes: HashMap<String, String>,
    /// The committed-`.prx` content-address pins, keyed `"{name}@{version}"`,
    /// values `"blake3:<hex>"`. The build-side mirror of the runtime
    /// `[compact_archive_signatures]` gate (`raw_source_prx::load_raw_source`):
    /// the phase-2c XML schema sources are read at COMPILE time, so the gate
    /// that the runtime applies to the committed `.prx` is applied here too,
    /// against the SAME pin, before the decoded bytes feed codegen.
    #[serde(default, rename = "compact_archive_signatures")]
    compact_archive_signatures: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// Build-side committed-`.prx` decode + content-address gate
// ---------------------------------------------------------------------------
//
// The phase-2c XML schema / spec sources (`xml.xsd`, `xml-infoset.xhtml`,
// `xml_1_0_fifth_edition-2008.xml`) are consumed at COMPILE time by the codegen
// writers below. Their raw bytes are fetch-only (`pr4xis update`) and ship in NO
// crate; only the content-addressed committed `.prx` is committed. So — exactly
// like every runtime `include_str!` site that phase 2 repointed to
// `raw_source_prx::raw_source_text_embedded` — these build-time readers decode
// the committed `.prx` envelope and gate it against the SAME
// `[compact_archive_signatures]` pin before the bytes reach codegen.
//
// The envelope decode + DEFLATE inflate + bomb guard is NOT restated here: it is
// the shared `prx_envelope` module (`#[path]`-included above), the ONE decoder
// the runtime also runs. `prx_envelope::blake3_gated_raw_source_text` verifies
// the `blake3:`-tagged pin, decodes the envelope, and returns the source text.
// This thin wrapper adds only the two build-side concerns that cannot live in
// the `no_std` shared codec: reading the `.prx` off disk (with the graceful
// `Ok(None)` "not on disk" skip a fresh checkout relies on) and looking the pin
// up in the build-parsed `RawLockFile`.
//
// Known deliberate asymmetry vs the runtime `load_raw_source`: a MISSING pin is
// a graceful `Ok(None)` skip at runtime (absence is legal on a fresh checkout;
// the accessor panics later by name) but a hard `Err` here, because at build
// time the `.prx` is already on disk, so an unpinned committed artifact is a
// configuration defect. (The runtime gate discharges its claim through the
// multi-algorithm `raw_hash::verify` path; the build script carries only the
// `blake3` hash dep, so `blake3_gated_raw_source_text` refuses any non-`blake3:`
// pin BY NAME — same single-source guard, one place.)

/// Decode a committed raw-source `.prx` into its source text, gated against the
/// trusted `[compact_archive_signatures]` pin for `"{name}@{version}"`. Reads the
/// `.prx` off disk (graceful `Ok(None)` when it is not there — a fresh checkout
/// that hasn't run `pr4xis compile`), looks the pin up, and delegates the whole
/// blake3-gate + envelope decode + UTF-8 to the shared
/// [`prx_envelope::blake3_gated_raw_source_text`]. Fail-closed: a missing pin, an
/// address mismatch, or a malformed envelope is an `Err`, no bytes returned.
fn decode_committed_prx_gated(
    prx_path: &std::path::Path,
    name: &str,
    version: &str,
    lock: &RawLockFile,
) -> Result<Option<String>, String> {
    let Ok(prx) = std::fs::read(prx_path) else {
        return Ok(None); // committed .prx not on disk — graceful skip.
    };
    let key = format!("{name}@{version}");
    let pin = lock
        .compact_archive_signatures
        .get(&key)
        .ok_or_else(|| format!("no praxis.lock [compact_archive_signatures] pin for `{key}`"))?;
    prx_envelope::blake3_gated_raw_source_text(&prx, pin, &key).map(Some)
}

/// The committed `.prx` path beside a raw source path — swap the final extension
/// for `.prx` (mirrors `raw_source_prx::raw_prx_path`). `foo/bar-1.0.xsd` →
/// `foo/bar-1.0.prx`.
fn committed_prx_path(raw_path: &std::path::Path) -> PathBuf {
    raw_path.with_extension("prx")
}

// The committed registry MANIFEST `.prx` is decoded when the workspace-root
// `praxis.toml` / `praxis.lock` are ABSENT (the published crate unpacked under
// `target/package/`), gated against the BAKED root before any bytes are decoded —
// the SAME `prx_envelope::PRAXIS_REGISTRY_ROOT_HEX` the runtime
// `load_registry_manifest` uses (there is no longer a build/runtime twin to
// drift). `prx_envelope::blake3_gated_registry` performs the gate + decode; see
// its call in `main` below.

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

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR is always set during builds");
    let out_dir = PathBuf::from(out_dir);

    println!("cargo:rerun-if-changed=build.rs");

    // The manifest the codegen writers walk for source versions/kinds. PREFER the
    // workspace-root `praxis.toml` / `praxis.lock` — the live source of truth that
    // `pr4xis update` / `compile` rewrites. When the workspace root is ABSENT (the
    // crate is unpacked under `target/package/` for `cargo publish --verify`, or
    // pulled from crates.io) decode the committed registry MANIFEST `.prx`
    // (`data/registry/praxis-registry.prx`) — the SAME committed artifact the
    // runtime registry loads — so the PUBLISHED crate reads its registered-source
    // manifest from the `.prx`, never from a raw-TOML snapshot (it ships none).
    // No empty fallback: the `.prx` is committed in-repo, so one source always
    // resolves; a genuinely-missing `.prx` is a build defect, surfaced.
    let registry_prx_path = PathBuf::from(&manifest_dir).join("data/registry/praxis-registry.prx");
    println!("cargo:rerun-if-changed={}", registry_prx_path.display());
    if manifest_path.exists() {
        println!("cargo:rerun-if-changed={}", manifest_path.display());
    }
    if lock_path.exists() {
        println!("cargo:rerun-if-changed={}", lock_path.display());
    }

    let (manifest_text, lock_text) = match (
        std::fs::read_to_string(&manifest_path),
        std::fs::read_to_string(&lock_path),
    ) {
        // Workspace root present — the live source of truth.
        (Ok(toml), Ok(lock)) => (toml, lock),
        // No workspace root (published/unpacked crate): decode the committed
        // registry `.prx`. This is what makes `cargo publish --verify` build a
        // NON-HOLLOW crate — the registry comes from the `.prx`, not empty consts.
        _ => {
            let prx = std::fs::read(&registry_prx_path).unwrap_or_else(|e| {
                panic!(
                    "neither workspace-root praxis.toml/.lock nor committed registry .prx \
                     `{}` is readable ({e}) — the registry manifest is unavailable",
                    registry_prx_path.display()
                )
            });
            prx_envelope::blake3_gated_registry(&prx).unwrap_or_else(|e| {
                panic!(
                    "committed registry .prx `{}` is malformed: {e}",
                    registry_prx_path.display()
                )
            })
        }
    };

    // `registry.rs` no longer `include!`s a `$OUT_DIR/praxis_embed.rs`: the
    // runtime registry loads the manifest from the committed `.prx` directly
    // (`registry_prx::load_registry_manifest`), so build.rs emits no raw-TOML
    // `&str` const. The manifest parsed here is build-time-only, driving the XML
    // codegen writers below.
    let manifest: RawManifest = toml::from_str(&manifest_text)
        .expect("parse praxis.toml (workspace root or registry .prx)");
    // The parsed lock carries the `[compact_archive_signatures]` pins the
    // phase-2c writers below gate their committed `.prx` against. Recovered from
    // the workspace-root `praxis.lock` or the committed registry `.prx` (same as
    // the manifest above) — so the writers always have the pins to gate against,
    // even in the published/unpacked crate.
    let lock: RawLockFile =
        toml::from_str(&lock_text).expect("parse praxis.lock (workspace root or registry .prx)");

    // The early-return that used to bail when praxis.toml/lock were
    // missing emitted no $OUT_DIR files, which broke the runtime
    // `include!`s for xml_namespace_schema_generated.rs, xml_infoset_
    // generated.rs, xml_grammar_generated.rs, and usc_corpus_codegen.rs.
    // The writers below run unconditionally; each writes a stub when
    // its source isn't registered in the manifest.

    // Emit the (now-empty) `CodegenData<UsCode>` stub at
    // `$OUT_DIR/usc_corpus_codegen.rs` so the runtime `include!` in
    // `social::software::markup::xml::uslm::corpus` always resolves.
    // Build-time USC codegen was retired (M4.δ.7.a); the corpus loads at
    // runtime. (The former per-source `dispatch_codegen` hook was a no-op
    // feeding a drifted `content_type_for_kind` copy of the cited
    // `canonical_encoding` authority — both deleted.)
    write_usc_corpus_codegen(&out_dir);

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
    write_xml_namespace_schema_codegen(&workspace_root, &manifest, &lock, &out_dir);
    write_xml_infoset_codegen(&workspace_root, &manifest, &lock, &out_dir);

    // M5.ε.2 — XML 1.0 grammar productions from the loaded spec
    // (`xml_1_0_fifth_edition@2008`, Bray et al. 2008). Parses the
    // 86 `<prod>` blocks and emits range tables + predicates for
    // §2.2 Char, §2.3 NameStartChar, §2.3 NameChar. The parser
    // (`parser::grammar`) includes the generated module instead of
    // hand-coding the code-point ranges as primitive `matches!`
    // arms, per `feedback_bottom_up_loaded_not_encoded`.
    write_xml_grammar_codegen(&workspace_root, &manifest, &lock, &out_dir);
}

/// Find the registered `xml_1_0_namespace_xsd` source in the praxis
/// manifest, materialize its committed `xml.prx` through the build-side
/// `[compact_archive_signatures]` gate, and invoke
/// `pr4xis::codegen::xml_schemas::generate_xml_namespace_schema_from_source`
/// to emit `$OUT_DIR/xml_namespace_schema_generated.rs`. On any
/// failure (missing entry, missing/unpinned `.prx`, scan error) write a
/// commented stub so the runtime `include!` site always resolves.
///
/// The raw `xml.xsd` is fetch-only (`pr4xis update`) and ships in NO crate;
/// only the content-addressed committed `xml.prx` is committed and read here —
/// the build-time twin of every runtime `raw_source_text_embedded` site.
fn write_xml_namespace_schema_codegen(
    workspace_root: &std::path::Path,
    manifest: &RawManifest,
    lock: &RawLockFile,
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
    // `xml.xsd`, so its committed envelope is `xml.prx` beside it.
    let xsd_path = workspace_root.join("crates/domains/data/markup-schemas/xml/xml.xsd");
    let prx_path = committed_prx_path(&xsd_path);
    println!("cargo:rerun-if-changed={}", prx_path.display());

    let xsd = match decode_committed_prx_gated(
        &prx_path,
        "xml_1_0_namespace_xsd",
        &src.version,
        lock,
    ) {
        Ok(Some(text)) => text,
        Ok(None) => {
            let stub = format!(
                "// Stub: committed XML namespace XSD `.prx` not on disk at {}; skipping codegen.\n\
                 pub const XML_NAMESPACE_ATTRIBUTES: &[&str] = &[];\n",
                prx_path.display(),
            );
            std::fs::write(&out_path, stub).expect("write xml_namespace_schema stub");
            return;
        }
        Err(e) => {
            // A present-but-failing committed `.prx` is a defect (stale/poisoned
            // archive or missing pin) — fail the build, never feed unverified
            // bytes to codegen.
            panic!("XML namespace XSD committed .prx gate failed: {e}");
        }
    };

    match pr4xis::codegen::xml_schemas::generate_xml_namespace_schema_from_source(&xsd) {
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
                prx_path.display(),
                e,
            );
            std::fs::write(&out_path, stub).expect("write xml_namespace_schema stub");
            println!("cargo:warning=XML namespace XSD codegen failed: {e}");
        }
    }
}

/// Find the registered `xml_infoset` source in the praxis manifest,
/// materialize its committed `xml-infoset.prx` through the build-side
/// `[compact_archive_signatures]` gate, and invoke
/// `pr4xis::codegen::xml_schemas::generate_xml_infoset_from_source` to
/// emit `$OUT_DIR/xml_infoset_generated.rs`. On any failure (missing
/// entry, missing/unpinned `.prx`, scan error) write a commented stub.
///
/// The raw `xml-infoset.xhtml` is fetch-only and ships in NO crate; only the
/// content-addressed committed `.prx` is committed and read here.
fn write_xml_infoset_codegen(
    workspace_root: &std::path::Path,
    manifest: &RawManifest,
    lock: &RawLockFile,
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
    let prx_path = committed_prx_path(&xhtml_path);
    println!("cargo:rerun-if-changed={}", prx_path.display());

    let xhtml = match decode_committed_prx_gated(&prx_path, "xml_infoset", &src.version, lock) {
        Ok(Some(text)) => text,
        Ok(None) => {
            let stub = format!(
                "// Stub: committed XML Information Set `.prx` not on disk at {}; skipping \
                 codegen.\n{stub_decl}",
                prx_path.display(),
            );
            std::fs::write(&out_path, stub).expect("write xml_infoset stub");
            return;
        }
        Err(e) => panic!("XML Infoset committed .prx gate failed: {e}"),
    };

    match pr4xis::codegen::xml_schemas::generate_xml_infoset_from_source(&xhtml) {
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
                prx_path.display(),
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
    lock: &RawLockFile,
    out_dir: &std::path::Path,
) {
    let out_path = out_dir.join("xml_grammar_generated.rs");

    // Stub declarations so consumers compile when the spec is absent.
    // Must cover every symbol the runtime callers reference from
    // `spec_1_0::grammar::*` — `parser::grammar` invokes
    // `resolve_predefined_entity` in addition to the range predicates.
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
                     pub fn is_name_char(_c: u32) -> bool { false }\n\
                     #[must_use]\n#[allow(dead_code)]\n\
                     pub fn resolve_predefined_entity(_name: &str) -> Option<char> { None }\n";

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
    let prx_path = committed_prx_path(&spec_path);
    println!("cargo:rerun-if-changed={}", prx_path.display());

    let spec_bytes =
        match decode_committed_prx_gated(&prx_path, "xml_1_0_fifth_edition", &src.version, lock) {
            Ok(Some(text)) => text,
            Ok(None) => {
                let stub = format!(
                    "// Stub: committed XML 1.0 Fifth Edition spec `.prx` not on disk at {}; \
                     skipping codegen.\n{stub_decl}",
                    prx_path.display(),
                );
                std::fs::write(&out_path, stub).expect("write xml_grammar stub");
                return;
            }
            Err(e) => panic!("XML 1.0 grammar spec committed .prx gate failed: {e}"),
        };

    match pr4xis::codegen::xml_grammar::generate_xml_grammar_from_source(&spec_bytes) {
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
                prx_path.display(),
                e,
            );
            std::fs::write(&out_path, stub).expect("write xml_grammar stub");
            println!("cargo:warning=XML grammar codegen failed: {e}");
        }
    }
}

/// Emit the (now-empty) `CodegenData<UsCode>` stub at
/// `$OUT_DIR/usc_corpus_codegen.rs`. Build-time USC corpus codegen was
/// retired (M4.δ.7.a) — `UsCode::loaded()` reads + parses the registered
/// title XMLs at runtime (the WordNet `English::cached` pattern). The stub
/// is still emitted because the runtime `corpus/mod.rs` module `include!`s
/// it (a soft compatibility boundary so `UsCode::sample()` and the
/// codegen-data-shape tests still see an empty `CODEGEN_DATA` /
/// `USC_SECTION_AUX`).
fn write_usc_corpus_codegen(out_dir: &std::path::Path) {
    let out_path = out_dir.join("usc_corpus_codegen.rs");
    let stub = "// Empty stub — M4.δ.7.a retired build-time USC corpus codegen \
                in favor of runtime XML loading via `UsCode::loaded()`. \
                See the M4.δ.7.a design note.\n\
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

// Items 2 & 4 (audit-6): `dispatch_codegen` (an empty no-op hook),
// `content_type_for_kind` (a drifted `&'static str` copy of the cited
// `canonical_encoding` authority — it mapped `LegalLexicon => "Json"` where
// the authority says `XmlLmfLexicon`), and `expected_usc_title_path` (a
// hand mirror of `RegistryEntry::local_path`, used only to emit a
// `rerun-if-changed` for the retired build-time USC codegen) were all
// deleted. Item 3 (the envelope decode + content-address gate that this build
// script mirrored) is now the shared `prx_envelope` module, `#[path]`-included
// at the top of this file — one decoder for both build time and load time, no
// hand-kept mirror left.
