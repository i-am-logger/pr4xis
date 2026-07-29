use std::path::{Path, PathBuf};

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR set during builds");
    let out_dir = PathBuf::from(&out_dir);
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set"));

    // ---------- English / WordNet (the STORE BUNDLE `.stores.gz`, baked in) --
    // Emit the COMPLETE WordNet ontology as the nine BUILT store buffers —
    // build.rs runs `English::from_wordnet` ONCE, natively, over the succinct
    // decode (`emit_english_store_bundle_gz`), frames the buffers, gzips — and
    // bake the bundle into the wasm via `include_bytes!`. The runtime gunzips
    // and loads it through the fail-closed store-bundle content gate
    // (`load_english_store_bundle_gz_gated`, verified against the embedded
    // praxis.lock `[store_bundle_signatures]` pin) by PER-STORE VALIDATION
    // ALONE: no WordNet decode, no `from_wordnet`, no owned intermediate maps
    // in the browser — the former +348 MiB load transient (which wasm32's
    // never-shrinking linear memory paid FOREVER) collapses to ~the resident
    // cost. Same-toolchain by construction: this build.rs and the wasm runtime
    // compile from ONE Cargo.lock (same rkyv version/features; rkyv's archived
    // layout is little-endian and arch-independent, and both host and wasm32
    // are LE), which is exactly the `[store_bundle_signatures]` trust class.
    use pr4xis_domains::social::software::markup::xml::lmf::prx::{
        emit_english_store_bundle_gz, emit_english_store_bundle_gz_from_wordnet,
    };
    let wordnet_path = "../../crates/domains/data/wordnet/english-wordnet-2025.xml";
    let english_stores = out_dir.join("english.stores.gz");
    if Path::new(wordnet_path).exists() {
        println!("cargo:rerun-if-changed={}", wordnet_path);
        let source = std::fs::read(wordnet_path).expect("read WordNet XML");
        let bundle_gz =
            emit_english_store_bundle_gz(&source).expect("emit English store bundle at build time");
        eprintln!("Emitted english.stores.gz: {} bytes", bundle_gz.len());
        std::fs::write(&english_stores, bundle_gz).expect("write english.stores.gz");
    } else {
        println!("cargo:warning=WordNet XML not found at build time. English will be empty.");
        std::fs::write(
            &english_stores,
            emit_english_store_bundle_gz_from_wordnet(&empty_wordnet())
                .expect("emit empty English store bundle"),
        )
        .expect("write empty english.stores.gz");
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

    // ---------- On-demand USC archives (zero-copy fast path, task #21) ---
    // The SAME titles staged above, additionally pre-projected into a
    // content-addressed `.prx` Archive — `usc_archive` (project → apply the
    // usc_functor → append the grounding functor), run natively ONCE here
    // instead of once per page load in every visitor's browser. Served
    // alongside the raw XML at `/sources/<file>.cprx`; the runtime's
    // `Encoding::ContentAddressedArchive` loads it fail-closed against the
    // baked root with NO client-side USLM parse (the same kernel path
    // every embedded `.prx` already takes, here applied to a NETWORK-
    // FETCHED, on-demand title instead of a build-baked one). The raw XML
    // stays served too — a title whose build-time parse fails (or one not
    // yet re-staged) still has its honest, slower fallback.
    stage_usc_archives(&out_dir, &manifest_dir);

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
    // its `rkyv` local-cache bytes (`ArchiveLens::put_aligned`, task #21/#29
    // — same-toolchain, same-Cargo.lock, zero-copy), and bake both the bytes
    // and the archive's Merkle ROOT into the wasm. The browser loads these
    // bytes fail-closed against the baked root (re-deriving the root and
    // refusing on mismatch) via `Encoding::RkyvArchive` + `materialize_bytes`
    // — no owned decode/re-encode pass. This is NOT the legacy `.prx.gz`
    // envelope, and NOT the content-addressed DAG-CBOR wire form either
    // (reserved for genuinely cross-toolchain, long-lived identity, which
    // none of this crate's build-baked bytes need). A network-fetched or
    // user-uploaded `.prx` would flow through the SAME `load_ontology_prx`
    // path; embedding just removes the network from the demo.
    //
    // The generated `embedded_prx.rs` carries ONE `EMBEDDED_PRX` manifest of
    // build-baked ontologies — the LegalSources BASE (`default_loaded: true`,
    // installed by `Pr4xis::new` so the chat answers "is a statute a law" out of
    // the box) and the Dependability DEMO (`default_loaded: false`, a one-click
    // load). Neither is a privileged hardcode: `new()` iterates the manifest's
    // `default_loaded` entries through the SAME fail-closed `.prx` core a
    // fetched/uploaded `.prx` takes. Both go through the default, lexicalizing
    // `emit`, so each concept's Lemon label is a query surface by construction.
    emit_embedded_prx(&out_dir);
    emit_eager_residency(&out_dir);
}

/// Emit `EAGER_RESIDENT: &[&str]` — the sources this demonstrator fetches at
/// startup rather than waiting to be asked for.
///
/// THE THIRD RESIDENCY STATE. `EMBEDDED_PRX` already partitions baked-in
/// ontologies into resident (`default_loaded: true`) and one-click
/// (`default_loaded: false`). Everything fetched over the network was a single
/// undifferentiated "on demand" class — which left the demonstrator booting
/// without the corpus it exists to reason over, so a reviewer's first question
/// met an abstention that the engine had the capability to answer and simply
/// had not been given the data for. Naming eager residency makes that a
/// declared property of the deployment rather than an accident of what someone
/// remembered to click.
///
/// It is a DEPLOYMENT decision, not a registry one, which is why it lives here
/// and not in `praxis.toml`: the registry says what a source IS and where it
/// came from; which sources this particular page ships resident is a property
/// of this page. Curated and documented, in the same idiom as the corpus
/// ratchet's floors and the heavy-corpus MUST_PASS list — the runtime reads it
/// as data, so no name is ever typed into the page's JavaScript.
///
/// The six SPAR/OLiA vocabularies total ~0.5 MB and cost nothing to hold.
/// Title 42 is the expensive one at ~35 MB, and it earns its place: it carries
/// Medicare, Medicaid, the Older Americans Act and the HCBS waiver authorities
/// — the statutory base for both tracks' corpora. Its cost is disclosed on the
/// page before the transfer starts, not discovered during it.
fn emit_eager_residency(out_dir: &Path) {
    const EAGER: &[&str] = &[
        // Published OWL vocabularies — small, and they ground the citation,
        // document-structure and provenance vocabulary the trace reports.
        "biro",
        "c4o",
        "cito",
        "doco",
        "olia",
        "prov_o",
        // The statutory base both caregiver tracks are built on.
        "usc_title_42",
    ];
    let mut src = String::from(
        "// @generated by crates/wasm/build.rs — do not edit.\n\
         /// Registry names this deployment fetches at startup. The page looks\n\
         /// each up in the catalogs it already holds to learn HOW to load it,\n\
         /// so no source name is hard-coded in the page itself.\n\
         pub static EAGER_RESIDENT: &[&str] = &[\n",
    );
    for name in EAGER {
        src.push_str(&format!("    {name:?},\n"));
    }
    src.push_str("];\n");
    std::fs::write(out_dir.join("eager_residency.rs"), src).expect("write eager residency");
}

/// The Dependability demo ontology's runtime [`OntologyName`] — the const baked
/// into `lib.rs` so the load method and the test agree on it without restating
/// the string.
const DEMO_ONTOLOGY_NAME: &str = "Dependability";

/// The always-loaded LegalSources base ontology's runtime [`OntologyName`].
const LEGAL_SOURCES_ONTOLOGY_NAME: &str = "LegalSources";

/// The always-loaded caregiving chat lexicon's runtime [`OntologyName`]
/// (Caregiver AI Challenge Track 1).
const CAREGIVING_LEXICON_ONTOLOGY_NAME: &str = "caregiving_lexicon";

/// The always-loaded HCBS-compliance chat lexicon's runtime [`OntologyName`]
/// (Caregiver AI Challenge Track 2).
const HCBS_COMPLIANCE_LEXICON_ONTOLOGY_NAME: &str = "hcbs_compliance_lexicon";

/// Emit the two embedded `.prx` ontologies and ONE generated `embedded_prx.rs`
/// module carrying a single `EMBEDDED_PRX: &[EmbeddedOntology]` manifest — each
/// entry the bytes (by path), the trusted Merkle root hex, the runtime ontology
/// name, and a `default_loaded` residency flag. The runtime loads each
/// fail-closed against its baked root through ONE `.prx` core; `Pr4xis::new`
/// iterates the `default_loaded` entries, so no embedded ontology is a
/// privileged hardcode in `new()`.
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
    use pr4xis_domains::applied::data_provisioning::lexicon_provenance::lexicon_runtime_ontology;
    use pr4xis_domains::applied::dependability::ontology::DependabilityCategory;
    use pr4xis_domains::social::care::{caregiving_lexicon, hcbs_compliance_lexicon};
    use pr4xis_domains::social::judicial::legal_sources::ontology::LegalSourcesCategory;
    use pr4xis_runtime::emit::emit;
    use pr4xis_runtime::lens::archive_lens::ArchiveLens;

    // --- The Dependability demo (default lexicalizing `emit`) ---
    let archive = emit::<DependabilityCategory>();
    let root = archive
        .root()
        .expect("the emitted Dependability archive has a derivable Merkle root");
    let buf = ArchiveLens::put_aligned(&archive);
    let bytes = buf.as_slice();
    let prx_path = out_dir.join("dependability.prx");
    std::fs::write(&prx_path, bytes).expect("write embedded demo .prx");
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
    let legal_buf = ArchiveLens::put_aligned(&legal);
    let legal_bytes = legal_buf.as_slice();
    let legal_path = out_dir.join("legal_sources.prx");
    std::fs::write(&legal_path, legal_bytes).expect("write embedded LegalSources .prx");
    eprintln!(
        "Emitted embedded base .prx: {} ({} nodes, {} bytes), root {}",
        LEGAL_SOURCES_ONTOLOGY_NAME,
        legal.nodes.len(),
        legal_bytes.len(),
        legal_root.to_hex()
    );

    // --- The two Caregiver AI Challenge chat lexicons (default: loaded) ---
    // These are NOT compile-time `Category` taxonomies (there is no
    // `emit::<T>()` for them) — they are WN-LMF definitional lexicons
    // materialized through the SAME generic bridge the CLI and the corpus
    // tests use (`lexicon_runtime_ontology`, `crates/domains/data/care/*.xml`
    // embedded as compact WN-LMF bytes in the domains crate already).
    // Projecting through `to_owned_archive()` here, at build time, lets both
    // lexicons join the manifest and load through the ONE fail-closed `.prx`
    // core every other embedded/fetched/uploaded ontology takes — no second
    // load path for the browser.
    let caregiving = lexicon_runtime_ontology(
        "caregiving_lexicon",
        "2026",
        caregiving_lexicon::CAREGIVING_LEXICON_PRX,
    )
    .expect("the embedded caregiving_lexicon WN-LMF bytes materialize");
    let caregiving_archive = caregiving
        .to_owned_archive()
        .expect("the materialized caregiving_lexicon ontology round-trips to an owned Archive");
    let caregiving_root = caregiving_archive
        .root()
        .expect("the emitted caregiving_lexicon archive has a derivable Merkle root");
    let caregiving_buf = ArchiveLens::put_aligned(&caregiving_archive);
    let caregiving_bytes = caregiving_buf.as_slice();
    let caregiving_path = out_dir.join("caregiving_lexicon.prx");
    std::fs::write(&caregiving_path, caregiving_bytes)
        .expect("write embedded caregiving_lexicon .prx");
    eprintln!(
        "Emitted embedded lexicon .prx: {} ({} nodes, {} bytes), root {}",
        CAREGIVING_LEXICON_ONTOLOGY_NAME,
        caregiving_archive.nodes.len(),
        caregiving_bytes.len(),
        caregiving_root.to_hex()
    );

    let hcbs = lexicon_runtime_ontology(
        "hcbs_compliance_lexicon",
        "2026",
        hcbs_compliance_lexicon::HCBS_COMPLIANCE_LEXICON_PRX,
    )
    .expect("the embedded hcbs_compliance_lexicon WN-LMF bytes materialize");
    let hcbs_archive = hcbs.to_owned_archive().expect(
        "the materialized hcbs_compliance_lexicon ontology round-trips to an owned Archive",
    );
    let hcbs_root = hcbs_archive
        .root()
        .expect("the emitted hcbs_compliance_lexicon archive has a derivable Merkle root");
    let hcbs_buf = ArchiveLens::put_aligned(&hcbs_archive);
    let hcbs_bytes = hcbs_buf.as_slice();
    let hcbs_path = out_dir.join("hcbs_compliance_lexicon.prx");
    std::fs::write(&hcbs_path, hcbs_bytes).expect("write embedded hcbs_compliance_lexicon .prx");
    eprintln!(
        "Emitted embedded lexicon .prx: {} ({} nodes, {} bytes), root {}",
        HCBS_COMPLIANCE_LEXICON_ONTOLOGY_NAME,
        hcbs_archive.nodes.len(),
        hcbs_bytes.len(),
        hcbs_root.to_hex()
    );

    // Generate the ONE module the wasm includes: a single `EMBEDDED_PRX`
    // manifest of `EmbeddedOntology` entries (bytes by path + trusted root hex +
    // name + `default_loaded`). The runtime loads each fail-closed against its
    // baked root through the same `.prx` core; `Pr4xis::new` iterates the
    // `default_loaded` entries. The LegalSources base is `default_loaded: true`
    // (always installed at construction); the Dependability demo is
    // `default_loaded: false` (an on-demand one-click load).
    let module = format!(
        "/// One build-baked embedded `.prx` ontology — a compiled domain `Category`\n\
         /// (or, for the two lexicons, a materialized WN-LMF ontology) projected to\n\
         /// its `rkyv` local-cache bytes at build time — plus how the runtime\n\
         /// provisions it. Every entry (base or demo) loads through the SAME\n\
         /// fail-closed `.prx` core a network-fetched or user-uploaded `.prx` takes.\n\
         pub struct EmbeddedOntology {{\n\
         \x20   /// The runtime ontology name this `.prx` materializes under.\n\
         \x20   pub name: &'static str,\n\
         \x20   /// The `rkyv` local-cache Archive bytes (baked via `include_bytes!`).\n\
         \x20   pub bytes: &'static [u8],\n\
         \x20   /// The trusted Merkle root (lowercase hex) the fail-closed load re-derives\n\
         \x20   /// from the bytes and checks against, refusing on mismatch.\n\
         \x20   pub root_hex: &'static str,\n\
         \x20   /// Residency: `true` means `Pr4xis::new` installs it as an always-present\n\
         \x20   /// base (no network, no explicit load); `false` means an on-demand load.\n\
         \x20   pub default_loaded: bool,\n\
         }}\n\
         \n\
         /// The embedded `.prx` manifest — the single source of truth for which\n\
         /// build-baked ontologies exist and how each is provisioned. `Pr4xis::new`\n\
         /// iterates the `default_loaded` entries through the one fail-closed core, so\n\
         /// no embedded ontology is a privileged hardcode in `new()`. The LegalSources\n\
         /// BASE is LEXICALIZED (each concept's Lemon label minted as an `ontolex:Form`\n\
         /// surface) so \"law\"/\"case law\" ground for chat; the two Caregiver AI\n\
         /// Challenge chat lexicons (caregiving_lexicon, hcbs_compliance_lexicon) are\n\
         /// likewise `default_loaded: true` bases, materialized through the generic\n\
         /// WN-LMF bridge rather than a compile-time `Category`; the Dependability DEMO\n\
         /// is the Avizienis et al. (2004) taxonomy the UI offers as a one-click load.\n\
         pub static EMBEDDED_PRX: &[EmbeddedOntology] = &[\n\
         \x20   EmbeddedOntology {{\n\
         \x20       name: {legal_name:?},\n\
         \x20       bytes: include_bytes!({legal_path:?}),\n\
         \x20       root_hex: {legal_root_hex:?},\n\
         \x20       default_loaded: true,\n\
         \x20   }},\n\
         \x20   EmbeddedOntology {{\n\
         \x20       name: {caregiving_name:?},\n\
         \x20       bytes: include_bytes!({caregiving_path:?}),\n\
         \x20       root_hex: {caregiving_root_hex:?},\n\
         \x20       default_loaded: true,\n\
         \x20   }},\n\
         \x20   EmbeddedOntology {{\n\
         \x20       name: {hcbs_name:?},\n\
         \x20       bytes: include_bytes!({hcbs_path:?}),\n\
         \x20       root_hex: {hcbs_root_hex:?},\n\
         \x20       default_loaded: true,\n\
         \x20   }},\n\
         \x20   EmbeddedOntology {{\n\
         \x20       name: {name:?},\n\
         \x20       bytes: include_bytes!({prx_path:?}),\n\
         \x20       root_hex: {root_hex:?},\n\
         \x20       default_loaded: false,\n\
         \x20   }},\n\
         ];\n",
        prx_path = prx_path,
        root_hex = root.to_hex(),
        name = DEMO_ONTOLOGY_NAME,
        legal_path = legal_path,
        legal_root_hex = legal_root.to_hex(),
        legal_name = LEGAL_SOURCES_ONTOLOGY_NAME,
        caregiving_path = caregiving_path,
        caregiving_root_hex = caregiving_root.to_hex(),
        caregiving_name = CAREGIVING_LEXICON_ONTOLOGY_NAME,
        hcbs_path = hcbs_path,
        hcbs_root_hex = hcbs_root.to_hex(),
        hcbs_name = HCBS_COMPLIANCE_LEXICON_ONTOLOGY_NAME,
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

/// Project each staged USC title's authoritative XML into its `rkyv`
/// local-cache bytes (task #21's zero-copy on-demand path) and emit
/// `usc_archives_manifest.rs`: `(name, version, url, bytes, root_hex)`.
///
/// Reuses `usc_archive` — the EXACT function `Encoding::UslmTitle`'s runtime
/// decode arm calls after its own (much more expensive) client-side
/// `read_uslm_title` parse — so the projection this emits is byte-identical
/// (by content) to what a browser materializes today, just computed once,
/// natively, at build time instead of once per page load.
///
/// Deliberately `ArchiveLens::put_aligned`, NOT `pr4xis_runtime::load::emit`
/// (the content-addressed DAG-CBOR wire form `emit_embedded_prx` uses for
/// the ALWAYS-BAKED-IN ontologies): this build.rs and the wasm runtime it
/// serves already compile from ONE `Cargo.lock` (same rkyv version/features
/// — see the crate's own `rkyv` dependency comment), exactly the trust class
/// the English store bundle (`emit_english_store_bundle_gz`) already relies
/// on for the SAME reason. DAG-CBOR exists for cross-toolchain, long-lived
/// content addressing; a same-build browser fetch has no such need, and
/// paying it would cost the browser an extra owned decode-then-re-encode
/// pass on top of the one `materialize_bytes` already needs for its root/
/// closure bookkeeping. The content-address ROOT is still `Archive::root()`
/// — a pure function of node/connection content, independent of which wire
/// form carries it (DAG-CBOR or rkyv), so the trust guarantee is unchanged.
///
/// A title whose XML fails to parse at build time is skipped with a
/// `cargo:warning`, not a hard failure: its raw-XML entry in
/// `sources_manifest.rs` (staged by `stage_source_documents` above) remains
/// the honest fallback, so a build never breaks over one malformed title.
fn stage_usc_archives(out_dir: &Path, manifest_dir: &Path) {
    use pr4xis_domains::social::software::markup::xml::uslm::corpus::UsCode;
    use pr4xis_domains::social::software::markup::xml::uslm::corpus::bridge::{
        apply_defines_overlay, usc_archive,
    };
    use pr4xis_domains::social::software::markup::xml::uslm::corpus::prx::load_usc_defines_overlay_from_disk;
    use pr4xis_domains::social::software::markup::xml::uslm::lens::read_uslm_title;
    use pr4xis_runtime::lens::archive_lens::ArchiveLens;

    let uscode_dir = manifest_dir.join("../../crates/domains/data/legal/uscode");
    let sources_dir = manifest_dir.join("sources");
    std::fs::create_dir_all(&sources_dir).expect("create sources dir");

    // The cached `defines` overlay (task #6, "statutory definition" grounding
    // — "the term X means Y" -> a `defines` edge onto the defining node),
    // merged into EVERY staged title's archive below via `apply_defines_
    // overlay` BEFORE serialization — not a wasm-runtime concern at all: the
    // browser only ever loads pre-staged `.rprx` bytes through the generic
    // `Encoding::RkyvArchive` path (`wasm/src/lib.rs`), which has no notion
    // of "defines" and does not need one once the edges are already baked
    // in here. `load_usc_defines_overlay_from_disk` combines every
    // registered title's pairs into ONE list; `apply_defines_overlay`
    // silently skips any pair whose URN doesn't name a node in the
    // CURRENT title's archive (its own doc), so handing the whole combined
    // list to each per-title merge below is correct, not wasteful — no
    // per-title filtering needed. FAIL-CLOSED like every other embedded
    // artifact in this build: a stale `.defines.cprx.gz` (grammar/parsing
    // code changed since the cache was computed) panics here, at build
    // time, rather than silently shipping a wasm bundle with stale
    // definitions to the browser.
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/wasm has two ancestor dirs up to the workspace root");
    let defines_overlay = load_usc_defines_overlay_from_disk(workspace_root);
    eprintln!(
        "Loaded defines overlay for wasm USC staging: {} (urn, term) pairs",
        defines_overlay.len()
    );

    // The SAME overlay, emitted for the RUNTIME to use on the raw-XML load
    // path. Baking it into the staged `.rprx` alone left the two USC routes
    // materially different: "Load (fast)" produced a title carrying `defines`
    // edges, "Load raw XML" produced one without, and only `defines` edges
    // grounded into English populate `ComposedReasoner`'s definition index.
    // So the raw route reported success, turned the card green, changed the
    // state CID — and left every statutory definition in that title
    // unanswerable. A source that loads but that chat cannot consult is the
    // orphaned-mechanism failure this codebase treats as a defect.
    //
    // Emitted here rather than recomputed in the browser deliberately:
    // deriving these pairs takes ~1.5h for Title 42 (see
    // `usc_runtime_ontology_with_defines`'s own doc). The pairs are already
    // in hand at build time, so the runtime just applies them. Handing the
    // WHOLE list to any single title is correct — `apply_defines_overlay`
    // skips pairs whose URN names no node in that archive.
    {
        // An EMPTY overlay is a broken build, not a lean one.
        //
        // Every deployed wasm shipped this way and nothing noticed: the loader
        // returned an empty vec when the cache was absent, this emitted
        // `&[]` without complaint, and the page then loaded Title 42 — all
        // ~35 MB of it, eagerly — into an engine that could not answer a
        // single definition from it. The capability the surrounding comment
        // describes was absent from precisely the artifact other people run.
        //
        // The overlays are committed, so an empty list here means the build is
        // reading the wrong tree or the registry lost its USC titles. Either
        // way, saying so at build time costs one line and saves shipping a
        // demonstrator that is quietly worse than the one under test.
        assert!(
            !defines_overlay.is_empty(),
            "USC_DEFINES_OVERLAY would be emitted EMPTY, which silently \
             disables every statutory-definition answer in the shipped wasm. \
             The overlays are committed under .prx-cache/usc-defines-compact/ \
             — check that the workspace root resolved correctly and that the \
             registry still carries UsCodeTitle sources."
        );
        let mut src = String::from(
            "// @generated by crates/wasm/build.rs — do not edit.\n\
             /// Every cached `(urn, term)` statutory-definition pair across the\n\
             /// registered U.S. Code titles, baked so the raw-USLM load path can\n\
             /// apply the same `defines` overlay the pre-projected archives carry.\n\
             /// Pairs a title does not contain are skipped by `apply_defines_overlay`.\n\
             pub static USC_DEFINES_OVERLAY: &[(&str, &str)] = &[\n",
        );
        for (urn, term) in &defines_overlay {
            src.push_str(&format!("    ({urn:?}, {term:?}),\n"));
        }
        src.push_str("];\n");
        std::fs::write(out_dir.join("usc_defines_overlay.rs"), src)
            .expect("write usc defines overlay");
    }

    let mut manifest: Vec<(String, String, String, u64, String)> = Vec::new();
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
            let Some(xml_path) = find_title_xml(&dir, &name) else {
                continue;
            };
            // Shares the SAME rerun trigger `stage_source_documents` already
            // declares for this file — an extra `rerun-if-changed` on an
            // already-declared path is harmless (cargo just accumulates them).
            println!("cargo:rerun-if-changed={}", xml_path.display());
            let stem = xml_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            let version = stem
                .strip_prefix(&format!("{name}-"))
                .unwrap_or(stem)
                .to_string();

            let xml = match std::fs::read_to_string(&xml_path) {
                Ok(xml) => xml,
                Err(e) => {
                    println!(
                        "cargo:warning=USC title {name}@{version}: read failed ({e}); no compact archive staged."
                    );
                    continue;
                }
            };
            let title = match read_uslm_title(&xml) {
                Ok(title) => title,
                Err(e) => {
                    println!(
                        "cargo:warning=USC title {name}@{version}: parse failed ({e:?}); no compact archive staged."
                    );
                    continue;
                }
            };
            let usc = UsCode::from_uslm_titles_owned(vec![title]);
            let archive = apply_defines_overlay(usc_archive(&usc), &defines_overlay);
            let root = archive
                .root()
                .expect("a projected USC archive has a derivable Merkle root");
            let buf = ArchiveLens::put_aligned(&archive);
            let bytes = buf.as_slice();

            let file = format!("{name}-{version}.rprx");
            let dst = sources_dir.join(&file);
            std::fs::write(&dst, bytes).unwrap_or_else(|e| panic!("write {file}: {e}"));
            eprintln!(
                "Emitted rkyv USC archive {name}@{version}: {} nodes, {} bytes -> sources/{file}, root {}",
                archive.nodes.len(),
                bytes.len(),
                root.to_hex()
            );
            // The smallest staged title ALSO gets a compile-time-embedded
            // browser-test fixture (see `emit_usc_title_1_test_fixture`):
            // wasm32-in-browser tests (`wasm-pack test --headless --firefox`)
            // have no filesystem/network, so `include_bytes!` at native
            // compile time is the only way to hand them real, build.rs-
            // produced bytes — the exact technique every embedded `.prx`
            // (LegalSources, the two caregiver lexicons) already relies on.
            if name == "usc_title_1" {
                emit_usc_title_1_test_fixture(out_dir, &dst, &xml_path, root.to_hex());
            }
            manifest.push((
                name,
                version,
                format!("./sources/{file}"),
                bytes.len() as u64,
                root.to_hex(),
            ));
        }
    }

    manifest.sort();
    write_usc_archives_manifest(out_dir, &manifest);
}

/// Emit `usc_title_1_test_fixture.rs` — `include_bytes!`-embedded rkyv
/// archive + raw XML for `usc_title_1` (the smallest staged title), plus its
/// root hex. `crates/wasm/tests/web.rs` `include!`s this to exercise the
/// `"rkyv-archive"` load path over REAL build.rs-produced bytes without any
/// runtime filesystem/network access (unavailable in a headless-browser
/// wasm32 test) — everything is resolved at native compile time.
fn emit_usc_title_1_test_fixture(
    out_dir: &Path,
    rprx_path: &Path,
    xml_path: &Path,
    root_hex: String,
) {
    let src = format!(
        "/// Compile-time-embedded `usc_title_1` rkyv archive + its raw XML —\n\
         /// a browser-runtime test fixture (task #21). wasm32-in-browser\n\
         /// tests have no filesystem/network, so this is `include_bytes!`'d\n\
         /// at native compile time instead of fetched at test runtime.\n\
         pub static USC_TITLE_1_RPRX: &[u8] = include_bytes!({rprx_path:?});\n\
         pub static USC_TITLE_1_ROOT_HEX: &str = {root_hex:?};\n\
         pub static USC_TITLE_1_XML: &[u8] = include_bytes!({xml_path:?});\n"
    );
    std::fs::write(out_dir.join("usc_title_1_test_fixture.rs"), src)
        .expect("write usc_title_1_test_fixture.rs");
}

/// Emit `AVAILABLE_USC_ARCHIVES: &[(name, version, url, bytes, root_hex)]` —
/// the runtime's view of which USC titles have a pre-projected, zero-copy-
/// loadable rkyv archive. A title absent here still appears in
/// `AVAILABLE_SOURCES` (the raw-XML fallback); the host UI offers the fast
/// route only where this manifest names one.
fn write_usc_archives_manifest(out_dir: &Path, manifest: &[(String, String, String, u64, String)]) {
    let mut src = String::from(
        "/// (registry name, version, served .rprx URL, byte size, trusted Merkle\n\
         /// root hex). The zero-copy on-demand path (task #21): a pre-projected\n\
         /// `rkyv` USC title archive — `Encoding::RkyvArchive` loads it fail-\n\
         /// closed against the baked root via `materialize_bytes`, no client-\n\
         /// side USLM parse and no owned DAG-CBOR decode/re-encode pass.\n\
         pub static AVAILABLE_USC_ARCHIVES: &[(&str, &str, &str, u64, &str)] = &[\n",
    );
    for (name, version, url, bytes, root_hex) in manifest {
        src.push_str(&format!(
            "    ({name:?}, {version:?}, {url:?}, {bytes}, {root_hex:?}),\n"
        ));
    }
    src.push_str("];\n");
    std::fs::write(out_dir.join("usc_archives_manifest.rs"), src)
        .expect("write usc archives manifest");
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
