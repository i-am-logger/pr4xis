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

/// The concepts the DERIVED title index contributes once any USC title is held.
///
/// Loading a title now also derives `usc_titles` — one definitional entry per
/// held title, so the title is answerable by its own citation ("what is title
/// 42"). That is a second mechanism firing on the same action, so a count taken
/// after a load is `base + title + index`, and every assertion below that reads
/// `base + SAMPLE_TITLE_CONCEPTS` alone was measuring the world before it
/// existed.
///
/// Read from the engine's own report of that ONE ontology rather than written
/// down: the assertions still pin what the TITLE contributed, and this cannot
/// drift when the index's own shape changes.
fn title_index_concepts(p: &Pr4xis) -> usize {
    let v: serde_json::Value =
        serde_json::from_str(&p.self_describe()).expect("self_describe returns JSON");
    v["ontologies"]
        .as_array()
        .map(|xs| {
            xs.iter()
                .filter(|o| o["name"].as_str() == Some("usc_titles"))
                .filter_map(|o| o["concepts"].as_u64())
                .sum::<u64>() as usize
        })
        .unwrap_or(0)
}

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
        base_concepts() + SAMPLE_TITLE_CONCEPTS + title_index_concepts(&p),
        "one section + the corpus-level Code root materialized atop the base, \
         plus the derived title index the same load mints"
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
        base_concepts() + SAMPLE_TITLE_CONCEPTS + title_index_concepts(&p),
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
    let mut p = Pr4xis::new();
    let json = p.chat("is a dog a mammal");
    assert!(
        json.contains("\"response\""),
        "chat returns a presentation: {json}"
    );
    // The turn echoes its own question. The page writes a downloadable
    // decision record straight from this envelope, and a record carrying the
    // reasoning but not the question it answers is not one a compliance
    // function can read months later.
    assert!(
        json.contains("\"question\""),
        "chat echoes the question it answered: {json}"
    );
}

/// A downloaded decision record must say WHICH ENGINE answered, and UNDER WHAT
/// KNOWLEDGE — otherwise it cannot be re-derived, which is the one thing it
/// exists for.
///
/// Both facts are stamped on the turn by the engine rather than fetched by the
/// page afterwards, and this session makes the difference material: eager
/// residency loads sources in the background while the reader asks questions,
/// so a `self_describe` issued after the answer can report a knowledge state
/// that did not produce it. The stamp is taken at the moment of the answer.
#[wasm_bindgen_test]
fn a_turn_stamps_the_knowledge_state_that_produced_it() {
    let mut p = Pr4xis::new();
    let before: serde_json::Value =
        serde_json::from_str(&p.chat("is a dog a mammal")).expect("chat returns JSON");
    assert!(
        before["engine_version"]
            .as_str()
            .is_some_and(|v| !v.is_empty()),
        "every turn names the engine build that answered it: {before}"
    );
    let cid_before = before["state_cid"].as_str().map(str::to_owned);
    assert!(
        cid_before.is_some(),
        "a fresh Pr4xis already carries the embedded bases, so it HAS a \
         knowledge state to fingerprint: {before}"
    );

    // Load something, and the fingerprint must move. A `state_cid` that
    // survives a load unchanged would be describing the engine rather than
    // its knowledge, and two records answering the same question under
    // different authorities would be indistinguishable.
    p.load(
        "cito".to_string(),
        "owl-source",
        None,
        None,
        Some(SAMPLE_OWL.as_bytes().to_vec()),
    )
    .expect("a well-formed OWL vocabulary parses");
    let after: serde_json::Value =
        serde_json::from_str(&p.chat("is a dog a mammal")).expect("chat returns JSON");
    assert_ne!(
        after["state_cid"].as_str().map(str::to_owned),
        cid_before,
        "loading an authority must change the knowledge fingerprint the record \
         carries, or the record cannot distinguish what it was answered from"
    );
}

/// IN THE BROWSER, the engine's own `duration_us` is 0 — and that is why the
/// page times the call itself.
///
/// `WasmSafeTimer` wraps `std::time::Instant`, which does not exist on
/// `wasm32-unknown-unknown`, so `elapsed_us()` is cfg'd to a literal 0 there.
/// Native builds measure correctly, which is exactly what makes this easy to
/// miss: every native test agrees the figure is real. The demonstrator's
/// footer once published that 0 as "0µs", a fabricated measurement on a page
/// whose whole pledge is that its numbers are computed live; `sendChat`
/// (docs/chat/chat-ui.js) now measures with `performance.now()` and falls
/// back to it whenever the engine's figure is not a positive number.
///
/// This test pins the PREMISE of that fix in the one environment where it
/// holds. If a future `WasmSafeTimer` learns to read a real browser clock,
/// this fails — and the fallback in `sendChat` becomes removable rather than
/// silently redundant.
#[wasm_bindgen_test]
fn engine_reports_no_duration_in_the_browser_so_the_page_must_measure() {
    let mut p = Pr4xis::new();
    let json = p.chat("is a dog a mammal");
    let v: serde_json::Value = serde_json::from_str(&json).expect("chat returns JSON");
    let us = v["duration_us"].as_u64();
    assert_eq!(
        us,
        Some(0),
        "on wasm32 the engine cannot read a clock, so it reports 0 and the \
         page measures the call itself (see docs/chat/chat-ui.js `sendChat`). \
         Got {us:?} — if this is now a real figure, the host-side fallback is \
         no longer needed."
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

// ---------------------------------------------------------------------------
// USC zero-copy on-demand archives (task #21) — a pre-projected `rkyv`
// local-cache archive per staged USC title, build.rs-emitted alongside the
// raw XML. The runtime loads it through `Encoding::RkyvArchive`
// (`materialize_bytes` + a re-derived-root check) — no client-side USLM
// parse, no owned DAG-CBOR decode/re-encode pass. In production the bytes
// are network-fetched, but a wasm32-in-browser test has no filesystem/
// network, so `usc_title_1_fixture` below embeds the SAME build.rs-produced
// bytes via `include_bytes!` at native compile time — mirroring exactly how
// the embedded LegalSources/lexicon `.prx`s already reach this test binary.
// ---------------------------------------------------------------------------

/// Compile-time-embedded `usc_title_1` rkyv archive + raw XML — see
/// `emit_usc_title_1_test_fixture` in build.rs.
mod usc_title_1_fixture {
    include!(concat!(env!("OUT_DIR"), "/usc_title_1_test_fixture.rs"));
}

#[wasm_bindgen_test]
fn available_usc_archives_lists_a_pre_projected_title() {
    let json = Pr4xis::new().available_usc_archives();
    assert!(
        json.contains("\"archives\""),
        "the usc archive manifest names the list: {json}"
    );
    assert!(
        json.contains("usc_title_1"),
        "a small staged title should have a pre-projected archive: {json}"
    );
    assert!(
        json.contains("\"root\"") && json.contains("\"url\"") && json.contains("\"bytes\""),
        "each entry carries its load coordinates: {json}"
    );
}

#[wasm_bindgen_test]
fn load_usc_archive_zero_copy_matches_the_raw_uslm_path() {
    // The fast path (a build-time-projected rkyv archive) must materialize
    // the SAME concept count as the slow path (client-side USLM parse) —
    // proving the build-time projection is an earlier computation of the
    // identical result, not a divergent shortcut.
    let mut fast = Pr4xis::new();
    fast.load(
        "usc_title_1".to_string(),
        "rkyv-archive",
        None,
        Some(usc_title_1_fixture::USC_TITLE_1_ROOT_HEX.to_string()),
        Some(usc_title_1_fixture::USC_TITLE_1_RPRX.to_vec()),
    )
    .expect("a pre-projected USC archive loads fail-closed against its own root");

    let mut slow = Pr4xis::new();
    slow.load(
        "usc_title_1".to_string(),
        "uslm-title",
        None,
        None,
        Some(usc_title_1_fixture::USC_TITLE_1_XML.to_vec()),
    )
    .expect("the same title's raw XML parses");

    assert_eq!(
        fast.loaded_section_count(),
        slow.loaded_section_count(),
        "the zero-copy rkyv archive and the raw-XML parse must materialize \
         the SAME concept count"
    );

    // A count is a weak witness: two ontologies with the same number of
    // concepts and different content pass it. The claim being made is that the
    // build-time projection is an EARLIER COMPUTATION OF THE IDENTICAL RESULT,
    // so compare what a reader would actually observe.
    //
    // Concepts AND morphisms, per source, from the catalog the page renders:
    // the generating edges are where a projection would most plausibly differ
    // while leaving the node count intact.
    let stat = |p: &Pr4xis| -> (u64, u64) {
        let v: serde_json::Value =
            serde_json::from_str(&p.self_describe()).expect("self_describe returns JSON");
        let row = v["ontologies"]
            .as_array()
            .expect("ontologies[]")
            .iter()
            .find(|o| o["name"].as_str() == Some("usc_title_1"))
            .expect("the loaded title appears in the catalog")
            .clone();
        (
            row["concepts"].as_u64().expect("concepts"),
            row["morphisms"].as_u64().expect("morphisms"),
        )
    };
    assert_eq!(
        stat(&fast),
        stat(&slow),
        "the two routes must agree on morphisms as well as concepts — a \
         projection that dropped edges would keep the node count and lose the \
         reasoning"
    );

    // And the observable that actually matters: the two routes must ANSWER the
    // same. This is the leg that would have caught the defines overlay reaching
    // one route and not the other — the counts matched throughout that defect,
    // because the overlay adds `defines` edges to an already-materialized node
    // set, so only an answer could tell the two apart.
    let mut base = Pr4xis::new();
    let mut answered_on_both = 0;
    let mut changed_by_the_load = 0;
    for q in ["what is a person", "what is an officer", "what is a vessel"] {
        let a: serde_json::Value = serde_json::from_str(&fast.chat(q)).expect("JSON");
        let b: serde_json::Value = serde_json::from_str(&slow.chat(q)).expect("JSON");
        assert_eq!(
            (a["outcome"].as_str(), a["response"].as_str()),
            (b["outcome"].as_str(), b["response"].as_str()),
            "the zero-copy and raw-XML routes must give the same answer to \
             {q:?} — same outcome, same words"
        );
        if a["outcome"].as_str() == Some("answered") {
            answered_on_both += 1;
        }
        // Against an engine with NO title loaded. Title 1 is the Dictionary
        // Act: it defines "person", "officer" and "vessel" for the whole Code,
        // so loading it must change what the engine says about at least one of
        // them.
        let n: serde_json::Value = serde_json::from_str(&base.chat(q)).expect("JSON");
        if n["response"].as_str() != a["response"].as_str() {
            changed_by_the_load += 1;
        }
    }
    // NON-VACUITY. Two routes that both abstain agree trivially, and an
    // equality satisfiable by universal silence tests nothing.
    assert!(
        answered_on_both > 0,
        "both routes abstained on every question, so the comparison above \
         proved nothing — it compared two silences"
    );
    // AND the load has to matter. Equality between two routes that each
    // contribute nothing would also be satisfiable; this is the claim the
    // whole design rests on — knowledge is LOADED, so loading it changes the
    // answer — and until now nothing anywhere asserted it of a USC title.
    assert!(
        changed_by_the_load > 0,
        "loading Title 1 changed no answer, so neither route is contributing \
         anything a reader could observe"
    );
}

#[wasm_bindgen_test]
fn load_usc_archive_rejects_a_wrong_root() {
    let mut p = Pr4xis::new();
    assert!(
        p.load(
            "usc_title_1".to_string(),
            "rkyv-archive",
            None,
            Some("0".repeat(64)),
            Some(usc_title_1_fixture::USC_TITLE_1_RPRX.to_vec()),
        )
        .is_err(),
        "a wrong Merkle root must be refused, not silently admitted"
    );
    assert_eq!(
        p.loaded_section_count(),
        base_concepts(),
        "a failed load leaves only the always-loaded base"
    );
}

// ── Unload ───────────────────────────────────────────────────────────
//
// `unload` had NO test anywhere in the repository. Every load encoding
// carried both a success and a fail-closed case; the inverse carried
// neither, and the only caller was `docs/worker.js`. Two branches were
// wholly uncovered: the `composed = None` short-circuit when the last
// runtime ontology leaves, and the `ground_loaded_set` error path, which
// cannot roll back and so degrades the engine to embedded-English-only.
//
// The assertion that matters is not "the count went down" — it is that
// the CAPABILITY went with it. A source whose removal leaves its terms
// still answerable was never really the thing answering them.

/// Is `name` in the LOADED set the self-model reports?
///
/// Not `json.contains(name)`: a source's name survives an unload in the
/// catalog (as `available`) and in the append-only load history, both by
/// design, so a substring test fails on correct behaviour. `ontologies` is the
/// live set.
fn loaded_names(json: &str) -> Vec<String> {
    let v: serde_json::Value = serde_json::from_str(json).expect("self_describe returns JSON");
    v["ontologies"]
        .as_array()
        .map(|xs| {
            xs.iter()
                .filter_map(|o| o["name"].as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn is_loaded(json: &str, name: &str) -> bool {
    loaded_names(json).iter().any(|n| n == name)
}

/// Does the engine still report a QUERYABLE capability for `name`? The loaded
/// set says the ontology is held; this says it can still be reasoned over.
fn has_capability(json: &str, name: &str) -> bool {
    let v: serde_json::Value = serde_json::from_str(json).expect("self_describe returns JSON");
    v["capabilities"]
        .as_array()
        .is_some_and(|xs| xs.iter().any(|c| c["ontology"].as_str() == Some(name)))
}

#[wasm_bindgen_test]
fn unload_restores_the_pre_load_state_and_takes_the_capability_with_it() {
    let mut p = Pr4xis::new();
    let base = base_concepts();

    p.load(
        "usc_title_18".to_string(),
        "uslm-title",
        None,
        None,
        Some(SAMPLE_TITLE.as_bytes().to_vec()),
    )
    .expect("a well-formed USLM title parses");
    assert_eq!(
        p.loaded_section_count(),
        base + SAMPLE_TITLE_CONCEPTS + title_index_concepts(&p)
    );
    assert!(is_loaded(&p.self_describe(), "usc_title_18"));

    assert!(
        p.unload("usc_title_18"),
        "unloading a loaded id reports true"
    );

    assert_eq!(
        p.loaded_section_count(),
        base,
        "the section count returns to the pre-load base"
    );
    // Asserted STRUCTURALLY, against the loaded set — not as the absence of
    // the string from the document. The name legitimately survives in two
    // places: the catalog still lists the source as `available` (that is the
    // knowledge boundary doing its job), and the load history is append-only
    // by design, because an unload is part of the record of what the session
    // did. A substring check contradicts both and fails on correct behaviour.
    assert!(
        !is_loaded(&p.self_describe(), "usc_title_18"),
        "the title left the LOADED set: {}",
        p.self_describe()
    );
    assert!(
        !has_capability(&p.self_describe(), "usc_title_18"),
        "and its capability went with it — a source whose removal leaves its \
         terms still answerable was never the thing answering them"
    );
}

/// A RESIDENT base refuses to unload, and the engine is what refuses.
///
/// The page already withholds the Unload control from a resident source, but
/// that made residency a UI convention: a second host, or a direct call to this
/// exported method, would have removed a base nothing can re-acquire — there is
/// no load act to re-run, because the reader never performed one. The three
/// caregiving/legal bases ship as part of what the deployment IS.
///
/// Asserted on the ENGINE's own answer rather than on a button, so it holds for
/// any caller.
/// A title is HELD because of what it contains, never because of what it is
/// called.
///
/// `SAMPLE_TITLE` is Title 18's USLM — `/us/usc/t18/...`. Loading it under the
/// name `usc_title_5` used to make the engine answer "what is title 5" with
/// Title 5's registered gloss (Government Organization and Employees): a
/// confident definition of a document it does not hold, in the most convincing
/// register available. The engine now reads the title out of the loaded nodes'
/// own URNs, so the name a caller supplies cannot make it claim knowledge.
#[wasm_bindgen_test]
fn a_mislabelled_title_does_not_make_the_engine_answer_about_it() {
    let mut p = Pr4xis::new();
    p.load(
        "usc_title_5".to_string(),
        "uslm-title",
        None,
        None,
        Some(SAMPLE_TITLE.as_bytes().to_vec()),
    )
    .expect("the document parses — the name is the only thing that is wrong");

    let answer = p.chat("what is title 5");
    let v: serde_json::Value = serde_json::from_str(&answer).expect("chat returns JSON");
    let response = v["response"].as_str().unwrap_or_default();
    assert!(
        !response.contains("Government Organization"),
        "the engine recited Title 5's registered gloss for content that is \
         Title 18 — it answered about a document it does not hold: {response}"
    );

    // And the converse, so this is not satisfiable by answering nothing ever:
    // the title actually present IS answerable.
    let held = p.chat("what is title 18");
    let hv: serde_json::Value = serde_json::from_str(&held).expect("chat returns JSON");
    assert_eq!(
        hv["outcome"].as_str(),
        Some("answered"),
        "the title whose content IS loaded must still answer: {held}"
    );
}

#[wasm_bindgen_test]
fn a_resident_base_refuses_to_unload_and_stays_answerable() {
    let mut p = Pr4xis::new();
    let base = base_concepts();

    // ASK THE ENGINE which source is resident rather than naming one here — a
    // hardcoded name would pin this test to today's manifest and would not
    // notice a base that stopped reporting its residency.
    let described = p.self_describe();
    let v: serde_json::Value =
        serde_json::from_str(&described).expect("self_describe returns JSON");
    let name = v["sources"]
        .as_array()
        .expect("sources[]")
        .iter()
        .find(|s| s["residency"].as_str() == Some("resident") && s["releasable"] == false)
        .and_then(|s| s["name"].as_str())
        .expect("a freshly-constructed engine reports at least one resident source")
        .to_owned();

    assert!(
        !p.unload(&name),
        "a resident base ({name}) must refuse to unload — it was never acquired \
         by a control act, so no control act releases it"
    );
    assert_eq!(
        p.loaded_section_count(),
        base,
        "and it must still be there: a refusal that removed it anyway would be \
         worse than allowing it"
    );
    assert!(
        is_loaded(&p.self_describe(), &name),
        "the base is still in the loaded set after the refusal"
    );
}

#[wasm_bindgen_test]
fn unload_of_an_unknown_id_is_a_no_op() {
    let mut p = Pr4xis::new();
    let before_sections = p.loaded_section_count();
    let before_ontologies = p.loaded_ontology_count();
    let before_describe = p.self_describe();

    assert!(
        !p.unload("no_such_source"),
        "unloading an id that was never loaded reports false"
    );

    // Every observable identical — an unknown id must not disturb the
    // loaded set, and in particular must not trigger a re-ground that
    // could fail and strand the engine without a reasoner.
    assert_eq!(p.loaded_section_count(), before_sections);
    assert_eq!(p.loaded_ontology_count(), before_ontologies);
    assert_eq!(p.self_describe(), before_describe);
}

#[wasm_bindgen_test]
fn unload_one_of_two_re_grounds_the_survivor() {
    // The single-ontology case above never exercises `ground_loaded_set`:
    // removing the only runtime ontology short-circuits to `composed =
    // None`. This one does, which is the branch that can fail and cannot
    // roll back.
    let mut p = Pr4xis::new();
    let base = base_concepts();

    p.load(
        "usc_title_18".to_string(),
        "uslm-title",
        None,
        None,
        Some(SAMPLE_TITLE.as_bytes().to_vec()),
    )
    .expect("title loads");
    p.load(
        "cito".to_string(),
        "owl-source",
        None,
        None,
        Some(SAMPLE_OWL.as_bytes().to_vec()),
    )
    .expect("vocabulary loads");
    let both = p.loaded_ontology_count();

    assert!(p.unload("cito"), "the vocabulary unloads");

    // The survivor is intact and still queryable — a re-ground that
    // silently dropped it would leave the count right and the capability
    // gone, which is precisely the failure this asserts against.
    assert_eq!(p.loaded_ontology_count(), both - 1);
    assert_eq!(
        p.loaded_section_count(),
        base + SAMPLE_TITLE_CONCEPTS + title_index_concepts(&p),
        "the surviving title kept every section it materialized, and its \
         derived index entry survived with it"
    );
    let json = p.self_describe();
    assert!(
        is_loaded(&json, "usc_title_18"),
        "survivor still in the loaded set: {json}"
    );
    assert!(
        has_capability(&json, "usc_title_18"),
        "and still answerable — the re-ground kept it"
    );
    assert!(
        !is_loaded(&json, "cito"),
        "the unloaded vocabulary left the loaded set: {json}"
    );
}
