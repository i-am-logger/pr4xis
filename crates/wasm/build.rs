use std::path::{Path, PathBuf};

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR set during builds");
    let out_dir = PathBuf::from(&out_dir);
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set"));

    // ---------- English / WordNet (the compact `.prx.gz`, baked in) ------
    // Emit the COMPLETE WordNet ontology as the size-reduced `.prx.gz` and bake
    // it into the wasm via `include_bytes!`; the runtime gunzips and loads it
    // (`load_prx_gz` → `English::from_wordnet`) into the full typed graph.
    use pr4xis_domains::social::software::markup::xml::lmf::{compact_succinct, reader};
    let wordnet_path = "../../crates/domains/data/wordnet/english-wordnet-2025.xml";
    let english_prx = out_dir.join("english.prx.gz");
    if Path::new(wordnet_path).exists() {
        println!("cargo:rerun-if-changed={}", wordnet_path);
        let xml = std::fs::read_to_string(wordnet_path).expect("read WordNet XML");
        let wn = reader::read_wordnet(&xml).expect("parse WordNet XML at build time");
        let prx_gz = compact_succinct::emit_prx_gz(&wn);
        eprintln!(
            "Emitted english.prx.gz: {} bytes, {} synsets, {} entries",
            prx_gz.len(),
            wn.synsets.len(),
            wn.entries.len()
        );
        std::fs::write(&english_prx, prx_gz).expect("write english.prx.gz");
    } else {
        println!("cargo:warning=WordNet XML not found at build time. English will be empty.");
        std::fs::write(
            &english_prx,
            compact_succinct::emit_prx_gz(&empty_wordnet()),
        )
        .expect("write empty english.prx.gz");
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

    // ---------- The embedded new-format `.prx` demo ontology -------------
    // Project a REAL compiled domain ontology — the Avizienis et al. (2004)
    // Dependability taxonomy (`DependabilityCategory`, fully glossed) — into
    // a content-addressed `.prx` Archive, emit its canonical bytes, and bake
    // both the bytes and the archive's Merkle ROOT into the wasm. The browser
    // loads these bytes fail-closed against the baked root (re-deriving the
    // root and refusing on mismatch). This is the new content-addressed
    // Archive format, NOT the legacy `.prx.gz` envelope. A network-fetched or
    // user-uploaded `.prx` would flow through the SAME `load_ontology_prx`
    // path; embedding just removes the network from the demo.
    //
    // The generated `embedded_prx.rs` carries TWO embedded ontologies: the
    // Dependability DEMO (a browser one-click load) and the LegalSources BASE
    // (always loaded from `Pr4xis::new` so the chat answers "is a statute a law"
    // out of the box). Both go through the default, lexicalizing `emit`, so each
    // concept's Lemon label is a query surface by construction.
    emit_embedded_prx(&out_dir);
}

/// The Dependability demo ontology's runtime [`OntologyName`] — the const baked
/// into `lib.rs` so the load method and the test agree on it without restating
/// the string.
const DEMO_ONTOLOGY_NAME: &str = "Dependability";

/// The always-loaded LegalSources base ontology's runtime [`OntologyName`].
const LEGAL_SOURCES_ONTOLOGY_NAME: &str = "LegalSources";

/// Emit the two embedded `.prx` ontologies and ONE generated `embedded_prx.rs`
/// module carrying both constant sets (bytes path + trusted Merkle root hex +
/// ontology name each). The runtime loads each fail-closed against its baked root.
///
/// build.rs runs natively, so it can use the `emit` feature (which deps the
/// compile-time `pr4xis` category model) to project the live `Category` —
/// exactly the projection a `pr4xis compile` would perform, done here at build
/// time and frozen into the binary.
///
/// Both ontologies use the default, lexicalizing [`emit`], which mints each
/// concept's Lemon label as an `ontolex:Form` query surface whenever that label
/// differs from the Rust identifier. LegalSources needs this (`LegalSource` → "law",
/// `Precedent` → "case law" — "law" is the word a person types); Dependability gets
/// it for free (`CorrectService` → "correct service"), so the demo is label-
/// queryable by construction too. A concept whose label equals its identifier mints
/// no redundant Form, so identifier-grounded surfaces are unchanged.
fn emit_embedded_prx(out_dir: &Path) {
    use pr4xis_domains::applied::dependability::ontology::DependabilityCategory;
    use pr4xis_domains::social::judicial::legal_sources::ontology::LegalSourcesCategory;
    use pr4xis_runtime::emit::emit;
    use pr4xis_runtime::load;

    // --- The Dependability demo (default lexicalizing `emit`) ---
    let archive = emit::<DependabilityCategory>();
    let root = archive
        .root()
        .expect("the emitted Dependability archive has a derivable Merkle root");
    let bytes = load::emit(&archive).expect("the Dependability archive encodes to canonical .prx");
    let prx_path = out_dir.join("dependability.prx");
    std::fs::write(&prx_path, &bytes).expect("write embedded demo .prx");
    eprintln!(
        "Emitted embedded demo .prx: {} ({} nodes, {} bytes), root {}",
        DEMO_ONTOLOGY_NAME,
        archive.nodes.len(),
        bytes.len(),
        root.to_hex()
    );

    // --- The LegalSources base (default lexicalizing `emit`) ---
    let legal = emit::<LegalSourcesCategory>();
    let legal_root = legal
        .root()
        .expect("the emitted LegalSources archive has a derivable Merkle root");
    let legal_bytes =
        load::emit(&legal).expect("the LegalSources archive encodes to canonical .prx");
    let legal_path = out_dir.join("legal_sources.prx");
    std::fs::write(&legal_path, &legal_bytes).expect("write embedded LegalSources .prx");
    eprintln!(
        "Emitted embedded base .prx: {} ({} nodes, {} bytes), root {}",
        LEGAL_SOURCES_ONTOLOGY_NAME,
        legal.nodes.len(),
        legal_bytes.len(),
        legal_root.to_hex()
    );

    // Generate the ONE module the wasm includes: for each embedded ontology, the
    // bytes (by path), the trusted root hex (the fail-closed pin), and the name.
    let module = format!(
        "/// The embedded demo `.prx` — the Avizienis et al. (2004) Dependability\n\
         /// taxonomy projected to a content-addressed Archive at build time.\n\
         pub static EMBEDDED_DEMO_PRX: &[u8] = include_bytes!({prx_path:?});\n\
         /// The trusted Merkle root of [`EMBEDDED_DEMO_PRX`] (lowercase hex). The\n\
         /// runtime re-derives the root from the bytes and refuses to load on a\n\
         /// mismatch — the fail-closed pin, derived from the SAME archive whose\n\
         /// bytes are embedded above.\n\
         pub const EMBEDDED_DEMO_PRX_ROOT_HEX: &str = {root_hex:?};\n\
         /// The runtime ontology name the embedded demo `.prx` materializes under.\n\
         pub const EMBEDDED_DEMO_ONTOLOGY_NAME: &str = {name:?};\n\
         \n\
         /// The embedded LegalSources BASE `.prx` — the LKIF-Core-grounded formal\n\
         /// sources-of-law taxonomy, LEXICALIZED (each concept's Lemon label minted\n\
         /// as an `ontolex:Form` surface) so \"law\"/\"case law\" ground for chat.\n\
         /// Loaded at `Pr4xis::new` as an always-present base.\n\
         pub static EMBEDDED_LEGAL_SOURCES_PRX: &[u8] = include_bytes!({legal_path:?});\n\
         /// The trusted Merkle root of [`EMBEDDED_LEGAL_SOURCES_PRX`] (lowercase hex),\n\
         /// the fail-closed pin re-derived from the same archive whose bytes are above.\n\
         pub const EMBEDDED_LEGAL_SOURCES_PRX_ROOT_HEX: &str = {legal_root_hex:?};\n\
         /// The runtime ontology name the embedded LegalSources `.prx` materializes under.\n\
         pub const EMBEDDED_LEGAL_SOURCES_ONTOLOGY_NAME: &str = {legal_name:?};\n",
        prx_path = prx_path,
        root_hex = root.to_hex(),
        name = DEMO_ONTOLOGY_NAME,
        legal_path = legal_path,
        legal_root_hex = legal_root.to_hex(),
        legal_name = LEGAL_SOURCES_ONTOLOGY_NAME,
    );
    std::fs::write(out_dir.join("embedded_prx.rs"), module).expect("write embedded_prx module");
}

/// Stage each registered OWL `OntologyVocabulary`'s bundled `.owl` to
/// `<crate>/sources/<name>-<version>.owl` (served at `/sources/<file>`) and
/// emit `ontologies_manifest.rs` into `OUT_DIR`:
/// `(name, version, prx_url, source_url, lock_pin)`.
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
            // The tagged wire form (`<algorithm>:<hex>`) — the same lowering
            // praxis.lock itself carries.
            pin.to_string(),
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

/// An empty WordNet, so an `english.prx.gz` always exists for `include_bytes!`
/// even when the corpus is absent at build time.
fn empty_wordnet() -> pr4xis_domains::social::software::markup::xml::lmf::ontology::WordNet {
    use pr4xis_domains::social::software::markup::xml::lmf::ontology::{LexiconMetadata, WordNet};
    WordNet {
        lexicon: LexiconMetadata {
            id: None,
            label: None,
            language: None,
            email: None,
            license: None,
            version: None,
            url: None,
            citation: None,
            logo: None,
            status: None,
            confidence_score: None,
            dc: Vec::new(),
        },
        synsets: Vec::new(),
        entries: Vec::new(),
        syntactic_behaviours: Vec::new(),
    }
}
