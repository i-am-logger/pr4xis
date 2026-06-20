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
// This is ONE build-side decoder shared by all three writers (not three), the
// build-script mirror of `raw_source_prx::load_raw_source_prx_gated`: the
// envelope framing is the same dependency-free LEB128 layout
// (`put_blob(name) put_blob(version) put_blob(bytes)`), and the content address
// is `blake3` hex (matching `pr4xis_runtime::address::ContentAddress::of`), which
// the build-dep `blake3` re-derives here.

/// Read one LEB128 length-prefixed blob from `buf` at `*pos`, advancing `*pos`.
/// Fully bounds-checked — a truncated envelope is an `Err`, never a panic
/// (mirrors `raw_source_prx::get_blob`).
fn prx_get_blob<'a>(buf: &'a [u8], pos: &mut usize) -> Result<&'a [u8], String> {
    let mut len: u64 = 0;
    let mut shift = 0u32;
    loop {
        let b = *buf
            .get(*pos)
            .ok_or_else(|| "committed .prx varint runs past end of buffer".to_string())?;
        *pos += 1;
        len |= u64::from(b & 0x7f) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 64 {
            return Err("committed .prx varint length overflow".to_string());
        }
    }
    let len = len as usize;
    let end = pos
        .checked_add(len)
        .filter(|&e| e <= buf.len())
        .ok_or_else(|| "committed .prx blob runs past end of buffer".to_string())?;
    let b = &buf[*pos..end];
    *pos = end;
    Ok(b)
}

/// Decode a committed raw-source `.prx` envelope into its source bytes, AFTER
/// verifying the envelope's `blake3` content address equals the trusted
/// `[compact_archive_signatures]` pin for `"{name}@{version}"`. Fail-closed: a
/// missing pin, an address mismatch, or a malformed envelope is an `Err` and
/// NO bytes are returned — the build-side twin of the runtime fail-closed gate.
///
/// Returns `Ok(None)` only when the committed `.prx` is **not on disk** (a fresh
/// checkout that hasn't run `pr4xis compile` — the same graceful skip the
/// runtime loader and the existing writers' "source not on disk" branch take,
/// which makes the writer fall through to its commented stub).
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
    let expected_hex = pin.strip_prefix("blake3:").unwrap_or(pin);
    let found_hex = blake3::hash(&prx).to_hex().to_string();
    if found_hex != expected_hex {
        return Err(format!(
            "committed .prx for `{key}` hash mismatch: praxis.lock pins {expected_hex}, \
             archive carries {found_hex} — refusing to feed codegen"
        ));
    }
    let mut pos = 0usize;
    let _name = prx_get_blob(&prx, &mut pos)?;
    let _version = prx_get_blob(&prx, &mut pos)?;
    let blob = prx_get_blob(&prx, &mut pos)?;
    let text = String::from_utf8(blob.to_vec())
        .map_err(|e| format!("committed .prx for `{key}` payload is not UTF-8: {e}"))?;
    Ok(Some(text))
}

/// The committed `.prx` path beside a raw source path — swap the final extension
/// for `.prx` (mirrors `raw_source_prx::raw_prx_path`). `foo/bar-1.0.xsd` →
/// `foo/bar-1.0.prx`.
fn committed_prx_path(raw_path: &std::path::Path) -> PathBuf {
    raw_path.with_extension("prx")
}

/// The trusted content address (blake3 hex) of the committed registry MANIFEST
/// `.prx` — the registry root. The build-side mirror of the runtime
/// `registry_prx::PRAXIS_REGISTRY_ROOT_HEX` baked const (the two MUST stay in
/// lockstep; both are regenerated by
/// `cargo test -p pr4xis-domains -- --ignored regenerate_praxis_registry_prx`).
/// The registry `.prx` is decoded precisely WHEN the workspace `praxis.lock` is
/// absent, so it cannot chain its pin from the lock it populates — it gates
/// against this BAKED root instead, exactly as the runtime
/// `load_registry_manifest` does.
const PRAXIS_REGISTRY_ROOT_HEX: &str =
    "c36a853c3296167bab1cec69a9b1b0919ac393de8023de7d5530ba925f104b0b";

/// Decode the committed registry MANIFEST `.prx`
/// (`crates/domains/data/registry/praxis-registry.prx`) into its
/// `(praxis.toml text, praxis.lock text)` — the build-side twin of the runtime
/// `registry_prx::decode_registry`. Two LEB128 length-prefixed blobs:
/// `put_blob(toml) put_blob(lock)`. Fail-closed on a truncated / malformed
/// envelope. Used when the workspace-root `praxis.toml` / `praxis.lock` are
/// ABSENT — the published crate unpacked under `target/package/` with no
/// workspace root — so the build reads the SAME committed `.prx` the runtime
/// registry loads, never an embedded raw-TOML snapshot.
///
/// CONTENT-ADDRESS GATED: the envelope's `blake3` digest must equal the BAKED
/// [`PRAXIS_REGISTRY_ROOT_HEX`] root before any bytes are decoded — the
/// build-side twin of the runtime `load_registry_manifest` baked-root gate. A
/// tampered or stale registry `.prx` is REFUSED (`Err`), never fed to codegen.
/// Unlike `decode_committed_prx_gated` (which chains from the workspace lock),
/// this gate's anchor is the baked root, because this path runs precisely when
/// that lock is unavailable.
fn decode_registry_prx(prx: &[u8]) -> Result<(String, String), String> {
    let found_hex = blake3::hash(prx).to_hex().to_string();
    if found_hex != PRAXIS_REGISTRY_ROOT_HEX {
        return Err(format!(
            "registry .prx root mismatch: baked PRAXIS_REGISTRY_ROOT_HEX is \
             {PRAXIS_REGISTRY_ROOT_HEX}, archive carries {found_hex} — refusing to feed codegen"
        ));
    }
    let mut pos = 0usize;
    let toml = prx_get_blob(prx, &mut pos)?;
    let lock = prx_get_blob(prx, &mut pos)?;
    let toml = String::from_utf8(toml.to_vec())
        .map_err(|e| format!("registry .prx praxis.toml payload is not UTF-8: {e}"))?;
    let lock = String::from_utf8(lock.to_vec())
        .map_err(|e| format!("registry .prx praxis.lock payload is not UTF-8: {e}"))?;
    Ok((toml, lock))
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
            decode_registry_prx(&prx).unwrap_or_else(|e| {
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
