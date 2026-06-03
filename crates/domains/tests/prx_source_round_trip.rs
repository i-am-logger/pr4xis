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
//! checks the latter). It passes today via the universal FLOOR
//! ([`RoundTripFidelity::RawBytesComplementFloor`]): the `.prx.gz` stores the
//! exact source as a content-addressed constant complement and the decompile
//! op returns it only after the `sha256` honesty gate. The test additionally
//! asserts the achieved tier IS `RawBytesComplementFloor` — the honest STAGE-1
//! statement that this is floor-via-stored-complement, not yet graph-faithful.
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

        // Honest STAGE-1 statement: the achieved tier is the stored-complement
        // FLOOR, not graph-faithful. (When STAGE 2 lands a graph-faithful
        // writer for a kind, this assertion is what flips — deliberately, with
        // the writer.)
        assert_eq!(
            fidelity,
            RoundTripFidelity::RawBytesComplementFloor,
            "{}@{}: expected the stored-complement FLOOR tier in STAGE 1",
            entry.name,
            entry.version
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

    // The meter must be HONEST about STAGE 1: every source that achieves a tier
    // today achieves the stored-complement FLOOR, and carries a named gap to
    // graph-faithfulness. (No graph-faithful writer exists yet, so no row may
    // claim `ByteExactGraphFaithful`.)
    for r in &meter {
        if let Some(RoundTripFidelity::ByteExactGraphFaithful) = r.achieved {
            panic!(
                "{}: a source claims ByteExactGraphFaithful, but STAGE 1 ships no \
                 graph-faithful writer — this would be an over-claim",
                r.source
            );
        }
        // A provisioned floor source must name its gap (the per-source writer).
        if r.is_floor_via_stored_complement() {
            assert!(
                r.graph_faithful_gap.is_some(),
                "{}: a floor source must name its gap to graph-faithfulness",
                r.source
            );
        }
    }

    // The floor count is the remaining gap; it does not block this test's green.
    let floor = floor_source_count(&meter);
    eprintln!(
        "completeness meter: {} rows, {floor} on the stored-complement floor (the remaining gap)",
        meter.len()
    );
}
