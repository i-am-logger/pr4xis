//! The ALL-SOURCES SOURCE round-trip — "compile and decompile and compare
//! hashes, all sources" (issue #15, STAGE 1 of the universal compiler).
//!
//! # What this asserts
//!
//! For EVERY registered source with a `.prx` consumer (OWL RDF/XML, USLM XML,
//! WN-LMF XML) whose source bytes are on disk:
//!
//! ```text
//!   hash(decompile(compile(source))) == hash(source)
//! ```
//!
//! i.e. the SOURCE bytes round-trip exactly through the universal compiler:
//! `source → compile → .prx.gz → decompile → source`. The comparison is a
//! real, SHA-256-witnessed byte-exact equality, and the bytes are additionally
//! checked byte-for-byte (`out == in`).
//!
//! It is the SOURCE round-trip, NOT the FORMAT round-trip (`prx_runtime_emit.rs`
//! checks the latter). praxis's graph-faithful `.prx` sources regenerate the
//! source from the typed ontology plus a content-addressed concrete-syntax
//! complement (NO stored raw blob), achieving
//! [`RoundTripFidelity::ByteExactGraphFaithful`]: `english_wordnet` (SLICE 3b),
//! `usc_title_1` (SLICE U6), and the flat SPAR OWL family `cito` / `biro` / `c4o`
//! / `doco`. Every other source still rides the universal FLOOR
//! ([`RoundTripFidelity::RawBytesComplementFloor`]): the `.prx.gz` stores the
//! exact source as a content-addressed constant complement and the decompile op
//! returns it only after the `sha256` honesty gate. The test asserts the
//! achieved tier PER SOURCE against the completeness meter's DECLARED tier (the
//! lens registration), so a tier regression in either direction fails loudly.
//!
//! # Graceful skip + non-vacuity
//!
//! USC and WordNet corpora are large and externally provisioned (`pr4xis
//! update`), so a plain checkout has only the bundled OWL `.owl` files on disk;
//! sources not provisioned are skipped (the same discipline as the harness and
//! the emit anchors). The test is NON-VACUOUS: it asserts it exercised at least
//! one source for each kind that WAS provisioned, and at least one source
//! overall — so a misconfiguration that silently routed nothing fails loudly.
//!
//! Requires `fetch` (the `.prx` load gate) + `codegen` (the OWL `read_owl` emit
//! path; USC + WordNet emit need only `prx`).

use std::path::PathBuf;

use pr4xis_domains::applied::data_provisioning::registry::data_sources;
use pr4xis_domains::formal::meta::well_behaved_lens::{
    DecompileKind, RoundTripFidelity, completeness_meter, declared_matches_achieved, decompile,
    floor_source_count,
};
use pr4xis_domains::social::software::markup::xml::lmf::prx::emit_wordnet_prx_gz;
use pr4xis_domains::social::software::markup::xml::owl::prx::emit_prx_gz as emit_owl_prx_gz;
use pr4xis_domains::social::software::markup::xml::uslm::corpus::prx::emit_usc_prx_gz;
use sha2::{Digest, Sha256};

/// Workspace root — grandparent of `crates/domains/`, where registry
/// `local_path()`s resolve. Mirrors the emitters + the corpus loader.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let d = h.finalize();
    let mut s = String::with_capacity(64);
    for b in d {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Compile one source's bytes to a `.prx.gz` via the per-leaf emitter for its
/// kind — the same emit the CLI's `pr4xis compile` calls.
fn compile_for_kind(
    kind: DecompileKind,
    source: &[u8],
    name: &str,
    version: &str,
    url: &str,
) -> Vec<u8> {
    let r = match kind {
        DecompileKind::Owl => emit_owl_prx_gz(source, name, version, url),
        DecompileKind::UsCode => emit_usc_prx_gz(source, name, version, url),
        DecompileKind::WordNet => emit_wordnet_prx_gz(source, name, version, url),
    };
    r.unwrap_or_else(|e| panic!("compile {kind:?} {name}@{version}: {e}"))
}

/// The tier a source DECLARES in the completeness meter (its registered lens's
/// `WellBehavedLens::FIDELITY`) — the single source of truth for the expected
/// emit tier. A source with no lens declares the universal floor. Keyed by
/// `"{name}@{version}"`.
fn meter_declared_tier(name: &str, version: &str) -> RoundTripFidelity {
    let key = format!("{name}@{version}");
    completeness_meter()
        .iter()
        .find(|r| r.source == key)
        .map(|r| r.declared)
        .unwrap_or(RoundTripFidelity::RawBytesComplementFloor)
}

#[test]
fn all_sources_source_round_trip_byte_exact() {
    let root = workspace_root();

    // Per-kind exercised counts, so the test can prove non-vacuity for each
    // kind that was actually provisioned on disk.
    let mut owl = 0usize;
    let mut usc = 0usize;
    let mut wordnet = 0usize;

    for entry in data_sources() {
        // Only sources with a `.prx` consumer participate; the rest (PDF, ZIP,
        // …) have no compile/decompile leg and are correctly out of scope.
        let Some(kind) = DecompileKind::from_content_type(entry.content_type()) else {
            continue;
        };

        // Graceful skip — corpora not provisioned on this machine (USC/WordNet
        // are large + externally fetched). Same discipline as the harness +
        // emit anchors.
        let src_path = root.join(entry.local_path());
        let Ok(source) = std::fs::read(&src_path) else {
            continue;
        };

        // compile → .prx.gz → decompile → source.
        let prx_gz = compile_for_kind(kind, &source, &entry.name, &entry.version, &entry.url);
        let (reconstructed, fidelity) = decompile(&prx_gz, kind)
            .unwrap_or_else(|e| panic!("decompile {kind:?} {}@{}: {e}", entry.name, entry.version));

        // The SOURCE round-trip law: hashes match (and bytes are byte-exact).
        assert_eq!(
            sha256_hex(&reconstructed),
            sha256_hex(&source),
            "{}@{}: hash(decompile(compile(source))) != hash(source)",
            entry.name,
            entry.version
        );
        assert_eq!(
            reconstructed, source,
            "{}@{}: decompiled bytes are not byte-identical to the source",
            entry.name, entry.version
        );

        // The achieved tier, per SOURCE (not just per kind). Graph-faithful
        // `.prx` sources regenerate from the typed ontology + a concrete-syntax
        // complement (NO stored raw blob), carrying `ByteExactGraphFaithful`:
        // `english_wordnet` (SLICE 3b), `usc_title_1` (SLICE U6), and the flat
        // SPAR OWL family `cito` / `biro` / `c4o` / `doco`. The remaining OWL
        // (`prov_o`, `olia`) / USC sources still ride the stored-complement FLOOR
        // — their per-source byte-exact writers are the open gap. A WN-LMF source
        // the structural writer cannot yet
        // regenerate byte-exactly (`us_legal_lexicon`, whose child order the
        // DTD-ordered writer reorders) honestly degrades to the floor.
        //
        // The single source of truth for the EXPECTED tier is the completeness
        // meter's DECLARED tier (the lens registration), so the emit tier and the
        // meter cannot drift: a source declares graph-faithful iff a graph-
        // faithful lens is registered for it, else it declares the floor.
        let expected_tier = meter_declared_tier(&entry.name, &entry.version);
        assert_eq!(
            fidelity, expected_tier,
            "{}@{}: emitted round-trip tier disagrees with the completeness meter's declared tier",
            entry.name, entry.version
        );

        match kind {
            DecompileKind::Owl => owl += 1,
            DecompileKind::UsCode => usc += 1,
            DecompileKind::WordNet => wordnet += 1,
        }
    }

    // NON-VACUOUS: at least one source overall must have been exercised, and at
    // least one per kind that was provisioned. A plain checkout bundles the OWL
    // `.owl` files, so OWL must always be non-zero here; USC/WordNet are only
    // asserted-non-zero when their (externally provisioned) corpora are present.
    let total = owl + usc + wordnet;
    assert!(
        total >= 1,
        "the all-sources round-trip exercised NO source — registry/route misconfiguration"
    );
    assert!(
        owl >= 1,
        "expected at least one OWL source (the bundled `.owl` vocabularies) to round-trip"
    );

    // Per-kind non-vacuity when provisioned: if any USC/WordNet `.prx` consumer
    // is registered AND its source is on disk, it must have been exercised.
    assert_provisioned_kind_exercised(DecompileKind::UsCode, usc, &root);
    assert_provisioned_kind_exercised(DecompileKind::WordNet, wordnet, &root);

    eprintln!(
        "all-sources SOURCE round-trip: OWL={owl} USC={usc} WordNet={wordnet} (byte-exact at the floor)"
    );
}

/// If a `kind`'s `.prx` consumer is registered and its source is on disk, the
/// round-trip MUST have exercised it (`exercised >= 1`). Guards against a route
/// that silently skips a provisioned kind.
fn assert_provisioned_kind_exercised(
    kind: DecompileKind,
    exercised: usize,
    root: &std::path::Path,
) {
    let provisioned = data_sources()
        .iter()
        .filter(|e| DecompileKind::from_content_type(e.content_type()) == Some(kind))
        .any(|e| root.join(e.local_path()).exists());
    if provisioned {
        assert!(
            exercised >= 1,
            "{kind:?} has a provisioned source on disk but the round-trip exercised none"
        );
    }
}

/// The COMPLETENESS METER's anti-lie cross-check: the tier each source DECLARES
/// (its lens's `WellBehavedLens::FIDELITY`) must equal the tier it ACHIEVES (the
/// `RoundTripFidelity` the emitted `.prx` carries, via the harness verdict). A
/// source can therefore NEVER falsely claim graph-faithfulness.
#[test]
fn completeness_meter_declared_tier_matches_achieved() {
    let meter = completeness_meter();
    assert!(
        !meter.is_empty(),
        "the completeness meter must cover at least the registered lenses"
    );

    let liars = declared_matches_achieved(&meter);
    assert!(
        liars.is_empty(),
        "these sources declare a round-trip fidelity they do not achieve \
         (declared != achieved): {liars:?}"
    );

    // The meter must be HONEST per source. The graph-faithful sources in this
    // slice are: `english_wordnet` (WordNet, the FIRST — SLICE 3b), `usc_title_1`
    // (UsCode, SLICE U6), and the flat SPAR OWL family `cito` / `biro` / `c4o` /
    // `doco`. Each MAY claim `ByteExactGraphFaithful` and carries NO gap. EVERY
    // OTHER source — the two still-floor OWL vocabs (`prov_o`, `olia`) and every
    // other USC title (`usc_title_15/18/49`, …) — is still on the stored-complement
    // FLOOR: it may NOT claim graph-faithfulness, and a floor row must name its
    // per-source writer gap. The whitelist below has TEETH: it accepts ONLY those
    // named sources (each OWL prefix `@`-anchored), so a future over-claim — e.g.
    // `usc_title_15` leaking a graph-faithful tier from a title-agnostic emit, or
    // `prov_o` leaking one — still trips this assertion.
    for r in &meter {
        match r.declared {
            RoundTripFidelity::ByteExactGraphFaithful => {
                // The two legitimately graph-faithful sources in this slice. Note
                // `r.source` is the `"{name}@{version}"` key, so the `usc_title_1`
                // arm pins the EXACT source — `usc_title_15` (kind=UsCode) does
                // NOT match and is correctly rejected as an over-claim. The same
                // predicate is proven to keep its teeth by
                // `slice_guard_rejects_graph_faithful_over_claims`.
                assert!(
                    slice_allows_graph_faithful(r.kind, &r.source),
                    "{}: only english_wordnet (WordNet), usc_title_1 (UsCode), and the flat \
                     SPAR OWL family cito/biro/c4o/doco are graph-faithful in this slice; \
                     {:?} over-claims",
                    r.source,
                    r.kind
                );
                assert!(
                    r.graph_faithful_gap.is_none(),
                    "{}: a graph-faithful source carries NO gap",
                    r.source
                );
            }
            RoundTripFidelity::RawBytesComplementFloor => {
                // A floor source must NOT have achieved graph-faithfulness and
                // must name its gap (the per-source byte-exact writer).
                assert_ne!(
                    r.achieved,
                    Some(RoundTripFidelity::ByteExactGraphFaithful),
                    "{}: a floor source achieved graph-faithfulness without declaring it",
                    r.source
                );
                if r.is_floor_via_stored_complement() {
                    assert!(
                        r.graph_faithful_gap.is_some(),
                        "{}: a floor source must name its gap to graph-faithfulness",
                        r.source
                    );
                }
            }
        }
    }

    // The floor count is the remaining gap; it does not block this test's green.
    let floor = floor_source_count(&meter);
    eprintln!(
        "completeness meter: {} rows, {floor} on the stored-complement floor (the remaining gap)",
        meter.len()
    );
}

/// The graph-faithful WHITELIST used by
/// [`completeness_meter_declared_tier_matches_achieved`] — `kind == WordNet`, OR
/// `kind == UsCode && source` is exactly `usc_title_1@…`, OR `kind == Owl &&
/// source` is exactly one of the FLAT SPAR OWL vocabs `cito@…` / `biro@…` /
/// `c4o@…` / `doco@…` (the byte-exact OWL family; the remaining two OWL vocabs —
/// `prov_o` and `olia` — stay on the raw-bytes floor). Every OWL prefix is
/// `@`-anchored so a sibling vocab cannot leak a graph-faithful claim. Extracted
/// so the meta-test below can prove the guard keeps its teeth against an
/// over-claim leak.
fn slice_allows_graph_faithful(kind: DecompileKind, source: &str) -> bool {
    kind == DecompileKind::WordNet
        || (kind == DecompileKind::UsCode && source.starts_with("usc_title_1@"))
        || (kind == DecompileKind::Owl
            && (source.starts_with("cito@")
                || source.starts_with("biro@")
                || source.starts_with("c4o@")
                || source.starts_with("doco@")))
}

/// The over-claim guard in `completeness_meter_declared_tier_matches_achieved`
/// must keep its TEETH after SLICE U6 widened it from "WordNet only" to "WordNet
/// or usc_title_1": it accepts EXACTLY the two graph-faithful sources of this
/// slice and rejects every other source that might leak a graph-faithful tier —
/// most pointedly `usc_title_15` (also `kind == UsCode`, the precise title the
/// title-agnostic emit bug over-claimed). The `usc_title_1@` prefix is `@`-
/// anchored, so `usc_title_15@…` does NOT match it (position 11 is `5`, not `@`).
#[test]
fn slice_guard_rejects_graph_faithful_over_claims() {
    // The two legitimately graph-faithful sources are accepted.
    assert!(slice_allows_graph_faithful(
        DecompileKind::WordNet,
        "english_wordnet@2025"
    ));
    assert!(slice_allows_graph_faithful(
        DecompileKind::UsCode,
        "usc_title_1@pl-119-90"
    ));

    // A sibling USC title that LEAKS a graph-faithful claim is rejected — the
    // exact regression the title-agnostic `build_usc_envelope` produced.
    assert!(
        !slice_allows_graph_faithful(DecompileKind::UsCode, "usc_title_15@pl-119-90"),
        "usc_title_15 (kind=UsCode) must NOT be accepted as graph-faithful — \
         the @-anchored usc_title_1 prefix keeps the guard's teeth"
    );
    for leak in [
        "usc_title_18@pl-119-90",
        "usc_title_49@pl-119-90",
        "usc_title_5@pl-119-90",
    ] {
        assert!(
            !slice_allows_graph_faithful(DecompileKind::UsCode, leak),
            "{leak} must NOT be accepted as graph-faithful"
        );
    }

    // The flat SPAR OWL family is byte-exact — each accepted.
    for ok in ["cito@2.8.1", "biro@1.1.1", "c4o@1.2", "doco@1.3"] {
        assert!(
            slice_allows_graph_faithful(DecompileKind::Owl, ok),
            "{ok} (kind=Owl) is a flat byte-exact SPAR vocab and must be accepted"
        );
    }
    // The two STILL-FLOOR OWL vocabs are rejected — `prov_o` (striped but blocked
    // below the writer by §4.1 numeric character references, internal-subset DTD
    // entities, interspersed comments) and `olia` (internal-subset DTD entities).
    // The `@`-anchored prefixes keep the teeth so ONLY the flat four are
    // graph-faithful in this slice. `prov_o@`/`olia@` match no accepted prefix; a
    // `c4o`-lookalike that is NOT `@`-anchored (`c4ox@…`) is also rejected.
    for leak in [
        "prov_o@2013-04-30",
        "olia@2026-04-09",
        "c4ox@9.9",
        "biro_extra@1.1.1",
    ] {
        assert!(
            !slice_allows_graph_faithful(DecompileKind::Owl, leak),
            "{leak} (kind=Owl) must NOT be accepted — only cito/biro/c4o/doco are graph-faithful"
        );
    }
}
