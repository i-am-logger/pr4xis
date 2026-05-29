//! wasm-bindgen-test coverage for the in-browser runtime.
//!
//! Rust-native wasm tests — a real headless browser driven by webdriver
//! (no Node). `cargo test` won't build these (the crate is excluded from
//! the workspace + needs the wasm target). Run:
//!   wasm-pack test --headless --firefox
//!
//! `Pr4xis::load_source` takes the USLM XML as a *string*, so the core
//! load path — authoritative USLM XML → live `UsCode` → self-model
//! catalog — is exercisable here without `fetch`/DOM. The full
//! download + Web-Worker + progress-UI click-through would be a
//! Rust-native browser-automation test (fantoccini/thirtyfour over
//! webdriver) — never a JS tool.

use pr4xis_wasm::Pr4xis;
use wasm_bindgen_test::*;

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
    assert_eq!(p.loaded_section_count(), 0, "nothing loaded initially");

    p.load_source("usc_title_18".to_string(), SAMPLE_TITLE)
        .expect("a well-formed USLM title parses");

    // The proof of "loaded into memory like English": a live UsCode with
    // real, queryable sections — not inert bytes.
    assert_eq!(p.loaded_section_count(), 1, "one section materialized");
    let json = p.self_describe();
    assert!(
        json.contains("usc_title_18"),
        "title is in the catalog: {json}"
    );
}

#[wasm_bindgen_test]
fn load_source_is_idempotent() {
    let mut p = Pr4xis::new();
    p.load_source("usc_title_18".to_string(), SAMPLE_TITLE)
        .unwrap();
    p.load_source("usc_title_18".to_string(), SAMPLE_TITLE)
        .unwrap();
    assert_eq!(
        p.loaded_section_count(),
        1,
        "reloading the same source replaces, not duplicates"
    );
}

#[wasm_bindgen_test]
fn load_source_rejects_malformed_xml() {
    let mut p = Pr4xis::new();
    assert!(
        p.load_source("usc_title_18".to_string(), "<not-a-uslm-doc/>")
            .is_err(),
        "a document with no USLM root must fail closed"
    );
    assert_eq!(p.loaded_section_count(), 0, "nothing loaded on failure");
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
