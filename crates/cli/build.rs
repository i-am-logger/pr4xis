use std::path::Path;

fn main() {
    let wordnet_path = "../../crates/domains/data/wordnet/english-wordnet-2025.xml";

    if !Path::new(wordnet_path).exists() {
        // WordNet not available — CLI will load at runtime instead
        println!("cargo:warning=WordNet XML not found at build time. CLI will load at runtime.");
        return;
    }

    println!("cargo:rerun-if-changed={}", wordnet_path);

    let path = Path::new(wordnet_path);
    let builder = pr4xis::codegen::wordnet::parse_wordnet_xml(path)
        .expect("failed to parse WordNet XML at build time");

    let config = pr4xis::codegen::GenerateConfig::new("english_codegen", "ConceptId")
        .taxonomy("EnglishTaxonomy")
        .equivalence("EnglishEquivalence")
        .opposition("EnglishOpposition")
        .mereology("EnglishMereology");

    let code = builder.generate(&config);

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("english_codegen.rs");
    std::fs::write(&out_path, code).expect("failed to write generated English module");

    eprintln!(
        "Generated English: {} entities, {} relations",
        builder.entity_count(),
        builder.relation_count()
    );
}
