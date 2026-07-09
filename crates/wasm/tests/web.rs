//! wasm-bindgen-test coverage for the in-browser runtime.
//!
//! Rust-native wasm tests — a real headless browser driven by webdriver
//! (no Node). `cargo test` won't build these (the crate is excluded from
//! the workspace + needs the wasm target). Run:
//!   wasm-pack test --headless --firefox
//!
//! The one typed `Pr4xis::load(name, encoding, version, root_hex, payload)`
//! takes the USLM XML as UTF-8 `payload` bytes under the `"uslm-title"`
//! encoding, so the core load path — authoritative USLM XML → live `UsCode` →
//! self-model catalog — is exercisable here without `fetch`/DOM. The full
//! download + Web-Worker + progress-UI click-through would be a
//! Rust-native browser-automation test (fantoccini/thirtyfour over
//! webdriver) — never a JS tool.

use pr4xis_wasm::Pr4xis;
use wasm_bindgen_test::*;

// Run these in the real headless browser, matching the harness `dev-test-wasm`
// drives (`wasm-pack test --headless --firefox`). WITHOUT this the suite collects
// 0 tests under `--firefox` (they silently never run), which is how the
// post-59eaf8d `loaded_section_count` regression stayed invisible. With it, the
// 13 acceptance tests below actually gate the wasm runtime.
wasm_bindgen_test_configure!(run_in_browser);

/// The always-loaded base: the embedded `LegalSources` ontology `Pr4xis::new`
/// installs so the chat can answer "is a statute a law" with no explicit load.
/// Every `loaded_section_count` assertion is RELATIVE to this base so the tests
/// neither hardcode the base's size nor regress if it changes.
fn base_concepts() -> usize {
    Pr4xis::new().loaded_section_count()
}

// A minimal full-shape USLM title: `<uscDoc>` wrapper, USLM namespace,
// one `<section>`. Mirrors the runtime reader's known-good fixture
// (`uslm::lens::tests::SAMPLE_TITLE`). The namespace is load-bearing —
// `read_uslm_title` finds the `<title>` by USLM-namespace resolution.
const SAMPLE_TITLE: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<uscDoc xmlns="http://xml.house.gov/schemas/uslm/1.0" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <meta>
    <dc:title>Title 18</dc:title>
    <dc:type>USCTitle</dc:type>
    <dc:publisher>OLRC</dc:publisher>
  </meta>
  <main>
    <title identifier="/us/usc/t18">
      <num value="18">Title 18</num>
      <heading>CRIMES AND CRIMINAL PROCEDURE</heading>
      <section identifier="/us/usc/t18/s1514A">
        <num value="1514A">§ 1514A.</num>
        <heading>Civil action to protect against retaliation in fraud cases</heading>
        <content>No company may discriminate.</content>
      </section>
    </title>
  </main>
</uscDoc>
"##;

/// The concepts a loaded `SAMPLE_TITLE` materializes atop the base. The USC
/// projection (`uslm::corpus::bridge`) emits one concept-node per section /
/// subdivision PLUS the single corpus-level United States Code ROOT node
/// (`CODE_ROOT_URN`, 1 U.S.C. § 204) it anchors the Code-type grounding on
/// (the "statutes compose" bridge — commit `c2058084`). `SAMPLE_TITLE` carries
/// exactly one `<section>` and no subdivisions, so it contributes one section
/// concept + the one Code root = two. Verified by the domains-level bridge test
/// `usc::corpus::bridge::tests::projects_every_section_and_subdivision_as_a_node`
/// (sections + subdivisions + 1). Was `+ 1` before the Code root existed.
const SAMPLE_TITLE_CONCEPTS: usize = 2;

#[wasm_bindgen_test]
fn constructs_with_embedded_english() {
    let p = Pr4xis::new();
    assert!(p.concept_count() > 0, "English is embedded at build time");
    assert!(p.word_count() > 0);
}

#[wasm_bindgen_test]
fn available_sources_lists_registered_titles() {
    let p = Pr4xis::new();
    let json = p.available_sources();
    assert!(
        json.contains("usc_title_18"),
        "the source manifest should list staged titles: {json}"
    );
    assert!(
        json.contains("\"url\""),
        "each source carries a download URL"
    );
}

#[wasm_bindgen_test]
fn self_describe_reports_the_source_catalog() {
    let p = Pr4xis::new();
    let json = p.self_describe();
    assert!(json.contains("\"sources\""), "self-model emits the catalog");
    assert!(
        json.contains("english_wordnet"),
        "English is a Loaded source"
    );
}

#[wasm_bindgen_test]
fn load_source_materializes_a_live_usc() {
    let mut p = Pr4xis::new();
    assert_eq!(
        p.loaded_section_count(),
        base_concepts(),
        "only the always-loaded base is present"
    );

    p.load(
        "usc_title_18".to_string(),
        "uslm-title",
        None,
        None,
        Some(SAMPLE_TITLE.as_bytes().to_vec()),
    )
    .expect("a well-formed USLM title parses");

    // The proof of "loaded into memory like English": a live UsCode with
    // real, queryable sections — not inert bytes. One section concept + the
    // corpus-level Code root the projection anchors (see SAMPLE_TITLE_CONCEPTS).
    assert_eq!(
        p.loaded_section_count(),
        base_concepts() + SAMPLE_TITLE_CONCEPTS,
        "one section + the corpus-level Code root materialized atop the base"
    );
    let json = p.self_describe();
    assert!(
        json.contains("usc_title_18"),
        "title is in the catalog: {json}"
    );
}

#[wasm_bindgen_test]
fn load_source_is_idempotent() {
    let mut p = Pr4xis::new();
    p.load(
        "usc_title_18".to_string(),
        "uslm-title",
        None,
        None,
        Some(SAMPLE_TITLE.as_bytes().to_vec()),
    )
    .unwrap();
    p.load(
        "usc_title_18".to_string(),
        "uslm-title",
        None,
        None,
        Some(SAMPLE_TITLE.as_bytes().to_vec()),
    )
    .unwrap();
    assert_eq!(
        p.loaded_section_count(),
        base_concepts() + SAMPLE_TITLE_CONCEPTS,
        "reloading the same source replaces, not duplicates"
    );
}

#[wasm_bindgen_test]
fn load_source_rejects_malformed_xml() {
    let mut p = Pr4xis::new();
    assert!(
        p.load(
            "usc_title_18".to_string(),
            "uslm-title",
            None,
            None,
            Some(b"<not-a-uslm-doc/>".to_vec()),
        )
        .is_err(),
        "a document with no USLM root must fail closed"
    );
    assert_eq!(
        p.loaded_section_count(),
        base_concepts(),
        "a failed load leaves only the always-loaded base"
    );
}

#[wasm_bindgen_test]
fn chat_returns_a_response_envelope() {
    let p = Pr4xis::new();
    let json = p.chat("is a dog a mammal");
    assert!(
        json.contains("\"response\""),
        "chat returns a presentation: {json}"
    );
}

// ── OWL vocabulary dual-load (#257) ──────────────────────────────────
//
// A minimal CiTO-shaped OWL/RDF vocabulary in the `rdf:Description` +
// `rdf:type` serialization the SPAR ontologies use (RDF 1.1 XML Syntax
// §2.4 / §2.13). Two object properties, one subsuming the other
// (`citesAsEvidence rdfs:subPropertyOf cites`), plus one class — enough to
// prove `read_owl → from_owl_ontology` materialises a live vocabulary with
// the subsumption edge intact. Source-load only (no codegen needed in the
// wasm test build), so the `.prx.gz` emitter is not exercised here.
const SAMPLE_OWL: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<rdf:RDF xmlns="http://purl.org/spar/cito/"
         xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
         xmlns:owl="http://www.w3.org/2002/07/owl#">
  <rdf:Description rdf:about="http://purl.org/spar/cito/">
    <rdf:type rdf:resource="http://www.w3.org/2002/07/owl#Ontology"/>
  </rdf:Description>
  <rdf:Description rdf:about="http://purl.org/spar/cito/Citation">
    <rdf:type rdf:resource="http://www.w3.org/2002/07/owl#Class"/>
    <rdfs:label>citation</rdfs:label>
  </rdf:Description>
  <rdf:Description rdf:about="http://purl.org/spar/cito/cites">
    <rdf:type rdf:resource="http://www.w3.org/2002/07/owl#ObjectProperty"/>
    <rdfs:label>cites</rdfs:label>
  </rdf:Description>
  <rdf:Description rdf:about="http://purl.org/spar/cito/citesAsEvidence">
    <rdf:type rdf:resource="http://www.w3.org/2002/07/owl#ObjectProperty"/>
    <rdfs:label>cites as evidence</rdfs:label>
    <rdfs:subPropertyOf rdf:resource="http://purl.org/spar/cito/cites"/>
  </rdf:Description>
</rdf:RDF>
"##;

#[wasm_bindgen_test]
fn available_ontologies_lists_registered_owl_vocabularies() {
    let p = Pr4xis::new();
    let json = p.available_ontologies();
    assert!(
        json.contains("\"ontologies\""),
        "the ontology manifest names the list: {json}"
    );
    // The bundled SPAR/OLiA vocabularies are registered; cito is among them.
    assert!(json.contains("cito"), "cito must be offered: {json}");
    // Each entry carries both load routes.
    assert!(
        json.contains("prx_url") && json.contains("source_url"),
        "each ontology offers .prx.gz and source routes: {json}"
    );
    // The lock pin is a validation secret — never surfaced to the host.
    assert!(
        !json.contains("\"lock_pin\"") && !json.contains("\"pin\""),
        "the embedded lock pin must not leak into the catalog: {json}"
    );
}

#[wasm_bindgen_test]
fn load_owl_source_materializes_a_live_vocabulary() {
    let mut p = Pr4xis::new();
    assert_eq!(
        p.loaded_section_count(),
        base_concepts(),
        "only the always-loaded base is present"
    );

    p.load(
        "cito".to_string(),
        "owl-source",
        None,
        None,
        Some(SAMPLE_OWL.as_bytes().to_vec()),
    )
    .expect("a well-formed OWL vocabulary parses");

    // 1 class + 2 object properties = 3 queryable entities — live, not
    // inert bytes.
    assert_eq!(
        p.loaded_section_count(),
        base_concepts() + 3,
        "one class + two object properties materialized atop the base"
    );
    let json = p.self_describe();
    assert!(json.contains("cito"), "cito is in the catalog: {json}");
}

#[wasm_bindgen_test]
fn load_owl_source_is_idempotent() {
    let mut p = Pr4xis::new();
    p.load(
        "cito".to_string(),
        "owl-source",
        None,
        None,
        Some(SAMPLE_OWL.as_bytes().to_vec()),
    )
    .unwrap();
    p.load(
        "cito".to_string(),
        "owl-source",
        None,
        None,
        Some(SAMPLE_OWL.as_bytes().to_vec()),
    )
    .unwrap();
    assert_eq!(
        p.loaded_section_count(),
        base_concepts() + 3,
        "reloading the same vocabulary replaces, not duplicates"
    );
}

#[wasm_bindgen_test]
fn load_owl_source_rejects_malformed_xml() {
    let mut p = Pr4xis::new();
    assert!(
        p.load(
            "cito".to_string(),
            "owl-source",
            None,
            None,
            Some(b"<<<not xml".to_vec()),
        )
        .is_err(),
        "malformed OWL must fail closed"
    );
    assert_eq!(
        p.loaded_section_count(),
        base_concepts(),
        "a failed load leaves only the always-loaded base"
    );
}

#[wasm_bindgen_test]
fn load_prx_rejects_unregistered_vocabulary() {
    let mut p = Pr4xis::new();
    // No embedded [archive_signatures]/[hashes] pin for an unknown name →
    // the gate has nothing to validate against and refuses.
    assert!(
        p.load(
            "not_a_registered_vocab".to_string(),
            "owl-prx-gz",
            Some("9.9.9".to_string()),
            None,
            Some(b"irrelevant bytes".to_vec()),
        )
        .is_err(),
        "an unpinned vocabulary cannot be validated, so the load refuses"
    );
    assert_eq!(
        p.loaded_section_count(),
        base_concepts(),
        "a failed load leaves only the always-loaded base"
    );
}

#[wasm_bindgen_test]
fn load_prx_rejects_corrupt_blob_for_registered_vocabulary() {
    let mut p = Pr4xis::new();
    // cito is registered (has a lock pin), but these bytes are not a valid
    // gzip/rkyv `.prx.gz` — the fail-closed gate rejects before installing.
    assert!(
        p.load(
            "cito".to_string(),
            "owl-prx-gz",
            Some("2.8.1".to_string()),
            None,
            Some(b"definitely not a valid prx.gz envelope".to_vec()),
        )
        .is_err(),
        "a corrupt .prx.gz for a registered vocabulary must fail closed"
    );
    assert_eq!(
        p.loaded_section_count(),
        base_concepts(),
        "a failed load leaves only the always-loaded base"
    );
}
