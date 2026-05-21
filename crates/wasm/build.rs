use std::path::{Path, PathBuf};

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR set during builds");
    let out_dir = PathBuf::from(&out_dir);

    // ---------- English / WordNet ----------
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
        println!("cargo:warning=WordNet XML not found at build time. CLI will load at runtime.");
    }

    // ---------- USC corpus ----------
    write_usc_corpus_codegen(&out_dir);
}

/// Walk every registered USC title XML on disk and emit a single
/// `CodegenData<UsCode>` static. Mirrors the English/WordNet codegen
/// block above. If no XML is present, write an empty stub so the
/// `include!()` site always finds the file.
fn write_usc_corpus_codegen(out_dir: &Path) {
    let candidates: [&str; 3] = [
        "../../crates/domains/data/legal/uscode/usc_title_18/usc_title_18-pl-119-90.xml",
        "../../crates/domains/data/legal/uscode/usc_title_28/usc_title_28-pl-119-90.xml",
        "../../crates/domains/data/legal/uscode/usc_title_49/usc_title_49-pl-119-90.xml",
    ];

    let out_path = out_dir.join("usc_codegen.rs");

    let present: Vec<PathBuf> = candidates
        .iter()
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .collect();

    if present.is_empty() {
        println!("cargo:warning=No USC title XML on disk; emitting empty UsCode codegen stub.");
        let stub = "pub static CODEGEN_DATA: pr4xis::codegen_data::CodegenData<\
                    pr4xis_domains::social::software::markup::xml::uslm::corpus::UsCode> = \
                    pr4xis::codegen_data::CodegenData { \
                    entity_count: 0, entity_ids: &[], entity_kind: &[], \
                    entity_labels: &[], entity_defs: &[], word_index: &[], \
                    taxonomy: &[], mereology: &[], opposition: &[], \
                    equivalence: &[], causation: &[], references: &[] };\n\
                    pub static USC_SECTION_AUX: \
                    &[pr4xis_domains::social::software::markup::xml::uslm::corpus::UscSectionAux] = &[];\n";
        std::fs::write(&out_path, stub).expect("write usc stub");
        return;
    }

    for p in &present {
        println!("cargo:rerun-if-changed={}", p.display());
    }

    let paths: Vec<&Path> = present.iter().map(|p| p.as_path()).collect();
    let config = pr4xis::codegen::GenerateConfig::with_marker(
        "usc_codegen",
        "UscEntityId",
        "pr4xis_domains::social::software::markup::xml::uslm::corpus::UsCode",
    );
    let source = pr4xis::codegen::usc_corpus::generate_usc_corpus_source(&paths, &config)
        .expect("generate USC corpus codegen");
    let section_count = source
        .lines()
        .find_map(|l| l.strip_prefix("// Entities: "))
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    std::fs::write(&out_path, source).expect("write usc codegen");
    eprintln!(
        "Generated UsCode corpus: {section_count} sections -> {}",
        out_path.display()
    );
}
