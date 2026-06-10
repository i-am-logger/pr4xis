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
//! real, content-address-witnessed byte-exact equality, and the bytes are additionally
//! checked byte-for-byte (`out == in`).
//!
//! It is the SOURCE round-trip, NOT the FORMAT round-trip (`prx_runtime_emit.rs`
//! checks the latter). A graph-faithful `.prx` source regenerates the source
//! from the typed ontology plus a content-addressed concrete-syntax complement
//! (NO stored raw blob), achieving
//! [`RoundTripFidelity::ByteExactGraphFaithful`]; every other source rides the
//! universal FLOOR ([`RoundTripFidelity::RawBytesComplementFloor`]): the
//! `.prx.gz` stores the exact source as a content-addressed constant complement
//! and the decompile op returns it only after the content-address honesty gate.
//!
//! # Source-agnostic by construction
//!
//! WHICH tier each source reaches is NOT enumerated here — it is read from the
//! lens registry (the single source of truth): a source declares graph-faithful
//! iff a graph-faithful lens is registered for it, else it declares the floor.
//! This test asserts the achieved tier PER SOURCE against that DECLARED tier
//! (via the completeness meter), so a tier regression in either direction fails
//! loudly — and flipping a new source to graph-faithful needs NO edit here, only
//! its lens registration. The over-claim teeth are likewise registry-derived:
//! every PROVISIONED graph-faithful source MUST be exercised and reconstructed
//! byte-exact below (the coverage assertion), so a source can never claim
//! graph-faithfulness without that claim being proven on its real bytes — no
//! hardcoded per-source whitelist, which would only rot as sources flip.
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
use pr4xis_runtime::address::ContentAddress;

/// Workspace root — grandparent of `crates/domains/`, where registry
/// `local_path()`s resolve. Mirrors the emitters + the corpus loader.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
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
        DecompileKind::UsCode => emit_usc_prx_gz(source, name, version, url, None),
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

    // The keys (`"{name}@{version}"`) of every graph-faithful source this run
    // actually exercised + reconstructed byte-exact. The source-agnostic
    // over-claim teeth: after the loop, EVERY provisioned graph-faithful source
    // must appear here (it was proven), so a graph-faithful claim can never skip
    // its byte-exact proof — replacing the old hardcoded per-source whitelist.
    let mut exercised_graph_faithful: Vec<String> = Vec::new();

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
            ContentAddress::of(&reconstructed).to_hex(),
            ContentAddress::of(&source).to_hex(),
            "{}@{}: hash(decompile(compile(source))) != hash(source)",
            entry.name,
            entry.version
        );
        assert_eq!(
            reconstructed, source,
            "{}@{}: decompiled bytes are not byte-identical to the source",
            entry.name, entry.version
        );

        // The achieved tier, per SOURCE (not just per kind). A graph-faithful
        // source regenerates from the typed ontology + a concrete-syntax
        // complement (NO stored raw blob), carrying `ByteExactGraphFaithful`; a
        // floor source rides the stored-complement `RawBytesComplementFloor`.
        // WHICH it is, is NOT spelled out here: the single source of truth for the
        // EXPECTED tier is the completeness meter's DECLARED tier (the lens
        // registration), so the emit tier and the meter cannot drift — a source
        // declares graph-faithful iff a graph-faithful lens is registered for it,
        // else the floor. (This is the registry gate that caught the title-
        // agnostic emit over-claiming a floor-registered USC title: emit tier !=
        // declared tier ⇒ this assertion fails, for ANY source, no name list.)
        let expected_tier = meter_declared_tier(&entry.name, &entry.version);
        assert_eq!(
            fidelity, expected_tier,
            "{}@{}: emitted round-trip tier disagrees with the completeness meter's declared tier",
            entry.name, entry.version
        );

        // Record the graph-faithful sources actually exercised this run, so the
        // source-agnostic coverage assertion below can prove none was skipped.
        if expected_tier == RoundTripFidelity::ByteExactGraphFaithful {
            exercised_graph_faithful.push(format!("{}@{}", entry.name, entry.version));
        }

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

    // The over-claim guard, SOURCE-AGNOSTIC. Every source the lens registry
    // DECLARES graph-faithful, whose bytes are provisioned on disk, MUST have
    // been exercised above and reconstructed byte-exact — so a graph-faithful
    // claim is always proven over its REAL bytes, never a hardcoded name list.
    // Flipping a new source needs no edit here; a source that declares
    // graph-faithful yet is silently skipped (or never proven) trips this.
    // A FLOOR source needs no such coverage: its stored-complement round-trip is
    // the generic floor mechanism, proven by any floor source on disk.
    for row in completeness_meter() {
        if row.declared != RoundTripFidelity::ByteExactGraphFaithful {
            continue;
        }
        let Some(entry) = data_sources()
            .iter()
            .find(|e| format!("{}@{}", e.name, e.version) == row.source)
        else {
            continue; // a graph-faithful lens with no data-source entry — out of scope here
        };
        if !root.join(entry.local_path()).exists() {
            continue; // not provisioned on this machine — graceful skip, nothing to prove
        }
        assert!(
            exercised_graph_faithful.contains(&row.source),
            "{}: the lens registry DECLARES it graph-faithful and its source is provisioned on \
             disk, but the all-sources round-trip did NOT exercise it — a graph-faithful claim \
             must be proven byte-exact over its real bytes, never skipped",
            row.source
        );
    }

    eprintln!(
        "all-sources SOURCE round-trip: OWL={owl} USC={usc} WordNet={wordnet} \
         (graph-faithful sources proven byte-exact; floor sources byte-exact via stored complement)"
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

    // The meter must be HONEST per source, SOURCE-AGNOSTICALLY: the tier each row
    // declares comes from the lens registry (a graph-faithful lens ⟹
    // graph-faithful, else the floor), so this test names NO source. Two
    // structural invariants hold for every row, whatever its source:
    //   * a graph-faithful row has closed its gap, so it carries NO gap; and
    //   * a floor row has NOT achieved graph-faithfulness, and names its gap.
    // The anti-lie (declared == achieved when provisioned) is the
    // `declared_matches_achieved` check above; a graph-faithful claim is PROVEN
    // true — byte-exact over real bytes, and never skipped — by
    // `all_sources_source_round_trip_byte_exact`'s coverage assertion, and a
    // byte-exact LAW violation is caught hard by the `RoundTripHarnessAllVerified`
    // CI-gate axiom. No hardcoded whitelist is needed (or wanted: it would rot as
    // sources flip).
    for r in &meter {
        match r.declared {
            RoundTripFidelity::ByteExactGraphFaithful => {
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
