use std::path::{Path, PathBuf};

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR set during builds");
    let out_dir = PathBuf::from(&out_dir);
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set"));

    // ---------- English / WordNet (Embedded staging — baked in) ----------
    let wordnet_path = "../../crates/domains/data/wordnet/english-wordnet-2025.xml";
    if Path::new(wordnet_path).exists() {
        println!("cargo:rerun-if-changed={}", wordnet_path);
        let path = Path::new(wordnet_path);
        let builder = pr4xis::codegen::wordnet::parse_wordnet_xml(path)
            .expect("failed to parse WordNet XML at build time");
        let config = pr4xis::codegen::GenerateConfig::with_marker(
            "english_codegen",
            "ConceptId",
            "pr4xis_domains::cognitive::linguistics::english::English",
        );
        let code = builder.generate(&config);
        let out_path = out_dir.join("english_codegen.rs");
        std::fs::write(&out_path, code).expect("failed to write generated English module");
        eprintln!(
            "Generated English: {} entities, {} relations",
            builder.entity_count(),
            builder.relation_count()
        );
    } else {
        println!("cargo:warning=WordNet XML not found at build time. English will be empty.");
    }

    // ---------- On-demand sources (Async staging — rkyv archives) ----------
    // Every registered USC title XML on disk is projected into a
    // content-addressed `.rkyv` archive served at `/archives/<file>`. The
    // runtime fetches one on demand (`Pr4xis::load_source`) — no title is
    // baked into the binary, so the struct-literal codegen that hit
    // rustc's memory ceiling on the large titles is gone. The set is
    // derived from disk, not hardcoded.
    emit_usc_archives(&out_dir, &manifest_dir);
}

/// Project each on-disk USC title into an rkyv archive under
/// `<crate>/archives/`, and emit `archives_manifest.rs` (the catalog of
/// what's fetchable) into `OUT_DIR` for the runtime to expose.
fn emit_usc_archives(out_dir: &Path, manifest_dir: &Path) {
    let uscode_dir = manifest_dir.join("../../crates/domains/data/legal/uscode");
    let archives_dir = manifest_dir.join("archives");
    std::fs::create_dir_all(&archives_dir).expect("create archives dir");

    // Discover `usc_title_*/usc_title_*-<version>.xml` on disk.
    let mut manifest: Vec<(String, String, String)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&uscode_dir) {
        for entry in entries.flatten() {
            let dir = entry.path();
            let Some(name) = dir.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !dir.is_dir() || !name.starts_with("usc_title_") {
                continue;
            }
            let name = name.to_string();
            let Some(xml) = find_title_xml(&dir, &name) else {
                continue;
            };
            println!("cargo:rerun-if-changed={}", xml.display());
            // version = filename stem with the `<name>-` prefix stripped.
            let stem = xml.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
            let version = stem
                .strip_prefix(&format!("{name}-"))
                .unwrap_or(stem)
                .to_string();

            let builder = pr4xis::codegen::usc_corpus::build_usc_corpus(&[xml.as_path()])
                .unwrap_or_else(|e| panic!("build USC corpus for {name}: {e:?}"));
            let owned = builder.to_owned_codegen_data();
            let bytes = pr4xis::archive::to_archive_bytes(&owned)
                .unwrap_or_else(|e| panic!("rkyv-serialize {name}: {e}"));

            let file = format!("{name}-{version}.rkyv");
            std::fs::write(archives_dir.join(&file), &bytes)
                .unwrap_or_else(|e| panic!("write archive {file}: {e}"));
            eprintln!(
                "Archived {name}@{version}: {} sections, {} bytes -> archives/{file}",
                owned.entity_count,
                bytes.len()
            );
            manifest.push((name, version, file));
        }
    } else {
        println!("cargo:warning=USC corpus dir not found; no archives emitted.");
    }

    manifest.sort();
    write_archives_manifest(out_dir, &manifest);
}

/// The `<name>-<version>.xml` inside a `usc_title_*` directory, if present.
fn find_title_xml(dir: &Path, name: &str) -> Option<PathBuf> {
    std::fs::read_dir(dir).ok()?.flatten().find_map(|e| {
        let p = e.path();
        let fname = p.file_name()?.to_str()?;
        (p.extension().is_some_and(|x| x == "xml") && fname.starts_with(name)).then_some(p)
    })
}

/// Emit `AVAILABLE_ARCHIVES: &[(name, version, file)]` — the runtime's
/// view of which registered sources are fetchable as rkyv archives, so
/// the meta page can offer a Load action only where one exists.
fn write_archives_manifest(out_dir: &Path, manifest: &[(String, String, String)]) {
    let mut src = String::from(
        "/// (registry name, version, archive filename served at /archives/<file>).\n\
         pub static AVAILABLE_ARCHIVES: &[(&str, &str, &str)] = &[\n",
    );
    for (name, version, file) in manifest {
        src.push_str(&format!("    ({name:?}, {version:?}, {file:?}),\n"));
    }
    src.push_str("];\n");
    std::fs::write(out_dir.join("archives_manifest.rs"), src).expect("write archives manifest");
}
