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
        std::fs::write(out_dir.join("english_codegen.rs"), code)
            .expect("failed to write generated English module");
        eprintln!(
            "Generated English: {} entities, {} relations",
            builder.entity_count(),
            builder.relation_count()
        );
    } else {
        println!("cargo:warning=WordNet XML not found at build time. English will be empty.");
    }

    // ---------- On-demand sources (Async staging — downloaded XML) ------
    // Stage each registered USC title's authoritative USLM XML for serving
    // at `/sources/<file>`, and emit the manifest the runtime exposes so
    // the host can download it (with progress) and parse it into a live
    // `UsCode` — the same materialization English gets, only at runtime.
    // No build-time parse / no derived blob: the served document IS the
    // authoritative §204 source. The set is derived from disk, not
    // hardcoded.
    stage_source_documents(&out_dir, &manifest_dir);

    // ---------- OWL ontology vocabularies (dual-load: .prx.gz OR source) -
    // Emit the ontology manifest the runtime exposes so the host can load a
    // registered OWL vocabulary two ways: its `.prx.gz` (served from
    // `/ontologies/`, produced by the release CI emitter, #256) or its
    // authoritative `.owl` source (staged here to `/sources/`). The lock
    // pin is baked in — the wasm runtime has no filesystem to read
    // praxis.lock from, so the `.prx.gz` source-hash gate validates against
    // this embedded pin. Registry-driven, never hardcoded.
    stage_ontology_vocabularies(&out_dir, &manifest_dir);
}

/// Stage each registered OWL `OntologyVocabulary`'s bundled `.owl` to
/// `<crate>/sources/<name>-<version>.owl` (served at `/sources/<file>`) and
/// emit `ontologies_manifest.rs` into `OUT_DIR`:
/// `(name, version, prx_url, source_url, lock_pin_sha256)`.
///
/// The set + pins come from the live registry (`data_sources()` filtered to
/// `SourceTaxonomyConcept::OntologyVocabulary`, paired with
/// `lock_hashes()["{name}@{version}"]`). build.rs runs natively, so it can
/// reach both. The `.prx.gz` artifacts themselves are produced by the
/// release CI (`emit_prx_gz`, #256) into Pages `/ontologies/`; the manifest
/// points the host there. A vocabulary whose `.owl` is not on disk, or whose
/// `name@version` carries no lock pin, is skipped — it cannot be served or
/// hash-validated.
fn stage_ontology_vocabularies(out_dir: &Path, manifest_dir: &Path) {
    use pr4xis_domains::applied::data_provisioning::registry::{data_sources, lock_hashes};
    use pr4xis_domains::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;

    let workspace_root = manifest_dir.join("../..");
    let sources_dir = manifest_dir.join("sources");
    std::fs::create_dir_all(&sources_dir).expect("create sources dir");

    let pins = lock_hashes();
    let mut manifest: Vec<(String, String, String, String, String)> = Vec::new();

    for entry in data_sources() {
        if entry.kind != SourceTaxonomyConcept::OntologyVocabulary {
            continue;
        }
        let key = format!("{}@{}", entry.name, entry.version);
        let Some(pin) = pins.get(&key) else {
            // Unpinned — the .prx.gz source-hash gate could not validate it.
            println!("cargo:warning=ontology {key} has no praxis.lock pin; skipping.");
            continue;
        };

        let owl_path = workspace_root.join(entry.local_path());
        if !owl_path.exists() {
            // Registered but not bundled — nothing to serve.
            continue;
        }
        println!("cargo:rerun-if-changed={}", owl_path.display());

        let owl_file = format!("{}-{}.owl", entry.name, entry.version);
        let dst = sources_dir.join(&owl_file);
        std::fs::copy(&owl_path, &dst).unwrap_or_else(|e| panic!("stage {owl_file}: {e}"));

        // The .prx.gz lives under /ontologies/ (release CI emitter, #256);
        // the .owl source under /sources/ (staged just above).
        let prx_url = format!("./ontologies/{}-{}.prx.gz", entry.name, entry.version);
        let source_url = format!("./sources/{owl_file}");
        eprintln!("Registered OWL vocabulary {key}: source -> {source_url}, prx -> {prx_url}");
        manifest.push((
            entry.name.clone(),
            entry.version.clone(),
            prx_url,
            source_url,
            pin.clone(),
        ));
    }

    manifest.sort();
    write_ontologies_manifest(out_dir, &manifest);
}

/// Emit `AVAILABLE_ONTOLOGIES: &[(name, version, prx_url, source_url, lock_pin)]`
/// — the runtime's view of which registered OWL vocabularies are loadable,
/// each with the praxis.lock source-hash pin the `.prx.gz` gate validates
/// against (the wasm runtime has no filesystem to read the lock from).
fn write_ontologies_manifest(
    out_dir: &Path,
    manifest: &[(String, String, String, String, String)],
) {
    let mut src = String::from(
        "/// (registry name, version, served .prx.gz URL, served .owl source URL,\n\
         /// praxis.lock source-hash pin for `name@version`).\n\
         pub static AVAILABLE_ONTOLOGIES: &[(&str, &str, &str, &str, &str)] = &[\n",
    );
    for (name, version, prx_url, source_url, pin) in manifest {
        src.push_str(&format!(
            "    ({name:?}, {version:?}, {prx_url:?}, {source_url:?}, {pin:?}),\n"
        ));
    }
    src.push_str("];\n");
    std::fs::write(out_dir.join("ontologies_manifest.rs"), src).expect("write ontologies manifest");
}

/// Copy each on-disk USC title XML to `<crate>/sources/<name>-<version>.xml`
/// (served at `/sources/<file>`) and emit `sources_manifest.rs` into
/// `OUT_DIR`: `(name, version, url, byte size)`.
fn stage_source_documents(out_dir: &Path, manifest_dir: &Path) {
    let uscode_dir = manifest_dir.join("../../crates/domains/data/legal/uscode");
    let sources_dir = manifest_dir.join("sources");
    std::fs::create_dir_all(&sources_dir).expect("create sources dir");

    let mut manifest: Vec<(String, String, String, u64)> = Vec::new();
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
            let stem = xml.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
            let version = stem
                .strip_prefix(&format!("{name}-"))
                .unwrap_or(stem)
                .to_string();

            let file = format!("{name}-{version}.xml");
            let dst = sources_dir.join(&file);
            std::fs::copy(&xml, &dst).unwrap_or_else(|e| panic!("stage {file}: {e}"));
            let bytes = std::fs::metadata(&dst).map(|m| m.len()).unwrap_or(0);
            eprintln!("Staged {name}@{version}: {bytes} bytes -> sources/{file}");
            manifest.push((name, version, format!("./sources/{file}"), bytes));
        }
    } else {
        println!("cargo:warning=USC corpus dir not found; no sources staged.");
    }

    manifest.sort();
    write_sources_manifest(out_dir, &manifest);
}

/// The `<name>-<version>.xml` inside a `usc_title_*` directory, if present.
fn find_title_xml(dir: &Path, name: &str) -> Option<PathBuf> {
    std::fs::read_dir(dir).ok()?.flatten().find_map(|e| {
        let p = e.path();
        let fname = p.file_name()?.to_str()?;
        (p.extension().is_some_and(|x| x == "xml") && fname.starts_with(name)).then_some(p)
    })
}

/// Emit `AVAILABLE_SOURCES: &[(name, version, url, bytes)]` — the runtime's
/// view of which registered sources are downloadable, so the meta page can
/// offer a Load action (with a real download progress bar) only where the
/// authoritative document is served.
fn write_sources_manifest(out_dir: &Path, manifest: &[(String, String, String, u64)]) {
    let mut src = String::from(
        "/// (registry name, version, served URL, byte size of the source document).\n\
         pub static AVAILABLE_SOURCES: &[(&str, &str, &str, u64)] = &[\n",
    );
    for (name, version, url, bytes) in manifest {
        src.push_str(&format!("    ({name:?}, {version:?}, {url:?}, {bytes}),\n"));
    }
    src.push_str("];\n");
    std::fs::write(out_dir.join("sources_manifest.rs"), src).expect("write sources manifest");
}
