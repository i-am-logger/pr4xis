//! U.S. Code `.prx` archive-anchor gates — lifted out of the `pr4xis-domains`
//! `#[cfg(test)]` modules.
//!
//! Both gates re-emit each on-disk USC title's `.prx` (the standard envelope and
//! the compact codec) and assert the freshly-derived MerkleRoot / compact address
//! equals the committed `praxis.lock` pin. That re-emit is heavy: under nextest
//! it is paid once per process-isolated test; here all `#[test]`s run as threads
//! in one process, so each title is emitted once for the whole binary. The 16 MB
//! `ANCHOR_EMIT_SIZE_CAP` is preserved — it still bounds which titles are emitted
//! so the giants (Title 42 ≈ 108 MB) are anchored by the full-corpus
//! `pr4xis compile` CI step and `loaded()`'s fail-closed gate, not re-emitted
//! here. USC titles are externally provisioned (`pr4xis update`); CI provisions
//! them, so each gate HARD-FAILS (via `require_provisioned`) if NONE are on disk
//! within the cap — tests do not skip.

use pr4xis_domains::applied::data_provisioning::registry::{
    LockDigest, data_sources, lock_archive_signature, lock_compact_archive_signature,
    lock_compact_defines_signature,
};
use pr4xis_domains::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
use pr4xis_domains::social::software::markup::xml::owl::prx::prx_archive_address;
use pr4xis_domains::social::software::markup::xml::uslm::corpus::prx::{
    compact_prx_archive_address, compact_usc_defines_archive_address,
    emit_compact_usc_defines_prx_gz, emit_compact_usc_prx_gz, emit_usc_prx_gz,
};
use praxis_corpus_tests::{require_provisioned, workspace_root};

/// Skip titles whose XML exceeds this — keeps the per-test emit bounded while
/// still covering the footnote-heading titles (18 ≈ 12 MB, 28 ≈ 8 MB) whose
/// archive shifts with the `prose_text` heading projection. The larger titles
/// (5/49/15/42) are anchored by the full-corpus `pr4xis compile` CI step and by
/// `loaded()`'s fail-closed prx gate; each title's emit→load round-trip is
/// exercised by `usc_emit_then_load_equals_corpus`.
const ANCHOR_EMIT_SIZE_CAP: u64 = 16 * 1024 * 1024;

/// Every emitted USC `.prx` archive's MerkleRoot content address equals its
/// `praxis.lock` `[archive_signatures]` pin — the invariant the lock-driven load
/// gate enforces. A fresh `emit_usc_prx_gz` for every on-disk pinned title
/// within the cap re-derives EXACTLY that pin, so a stale or wrong pin is caught
/// for that title — including the footnote-heading titles whose archive shifts
/// with the `prose_text` projection.
///
/// `[archive_signatures]` is a SHARED keyspace; this anchor owns the
/// `UsCodeTitle` partition (the OWL anchor owns `OntologyVocabulary`, the WordNet
/// anchor owns `Language`).
#[test]
fn usc_archive_anchors_match_lock() {
    let root = workspace_root();
    let mut checked = 0usize;
    for entry in data_sources() {
        if entry.kind != SourceTaxonomyConcept::UsCodeTitle {
            continue;
        }
        let Some(pinned) = lock_archive_signature(&entry.name, &entry.version) else {
            continue; // not pinned — nothing to anchor
        };
        let path = root.join(entry.local_path());
        let Ok(meta) = std::fs::metadata(&path) else {
            continue; // not provisioned this run — covered when on disk
        };
        if meta.len() > ANCHOR_EMIT_SIZE_CAP {
            continue; // too large for the per-test budget — see the const doc
        }
        let src = std::fs::read(&path).expect("read pinned USC title");
        let prx_gz = emit_usc_prx_gz(&src, &entry.name, &entry.version, &entry.url)
            .expect("emit pinned USC title");
        let addr = prx_archive_address(&prx_gz).expect("derive MerkleRoot");
        assert_eq!(
            &LockDigest::address(addr),
            pinned,
            "{}@{} .prx MerkleRoot must equal its [archive_signatures] pin",
            entry.name,
            entry.version
        );
        checked += 1;
    }
    // A pinned title within the cap MUST be on disk — with none, the loop
    // asserted nothing (a false-green). CI provisions via `pr4xis update`.
    require_provisioned(checked, "usc");
}

/// COMPACT ARCHIVE ANCHOR — for every on-disk title within the budget that has a
/// `[compact_archive_signatures]` pin, a fresh `emit_compact_usc_prx_gz`
/// re-derives EXACTLY that pin. The portable (toolchain-independent) compact
/// sibling of `usc_archive_anchors_match_lock`; keeps the committed compact pins
/// honest — a stale or wrong pin (or a codec change that shifts the bytes) fails
/// closed here. Leaves out titles > the cap and titles not on disk this run, but
/// HARD-FAILS via `require_provisioned` if NONE are on disk within the cap.
#[test]
fn compact_usc_archive_anchors_match_lock() {
    let root = workspace_root();
    let mut checked = 0usize;
    for entry in data_sources() {
        if entry.kind != SourceTaxonomyConcept::UsCodeTitle {
            continue;
        }
        let Some(pinned) = lock_compact_archive_signature(&entry.name, &entry.version) else {
            continue;
        };
        let path = root.join(entry.local_path());
        let Ok(meta) = std::fs::metadata(&path) else {
            continue;
        };
        if meta.len() > ANCHOR_EMIT_SIZE_CAP {
            continue;
        }
        let src = std::fs::read(&path).expect("read pinned USC title");
        let cprx_gz = emit_compact_usc_prx_gz(&src).expect("emit compact");
        let addr = compact_prx_archive_address(&cprx_gz).expect("derive compact address");
        assert_eq!(
            &LockDigest::address(addr),
            pinned,
            "{}@{} compact .prx address must equal its \
             [compact_archive_signatures] pin (codec or source drift?)",
            entry.name,
            entry.version
        );
        checked += 1;
    }
    // A pinned title within the cap MUST be on disk — with none, the loop
    // asserted nothing (a false-green). CI provisions via `pr4xis update`.
    require_provisioned(checked, "usc");
}

/// DEFINES-OVERLAY ARCHIVE ANCHOR — for every on-disk title within the budget
/// that has a `[compact_defines_signatures]` pin, a fresh
/// `emit_compact_usc_defines_prx_gz` re-derives EXACTLY that pin. Unlike the two
/// anchors above, `[compact_defines_signatures]` is legitimately EMPTY until a
/// maintainer runs the rare, opt-in `pr4xis compile --defines --lock` (a
/// ~3.3-hour full-corpus grounding pass, see
/// `uslm::corpus::bridge::usc_runtime_ontology_with_defines`'s doc) — so this
/// gate soft-passes (asserts nothing) when nothing is pinned yet, UNLIKE the
/// hard-fail `require_provisioned` the structural anchors use. It gains real
/// teeth the moment the first defines pin is committed. `#[ignore]`d: even one
/// title's defines re-emit runs the full tokenize/chart/Montague pipeline over
/// every lexical-prose node (~26-40ms/node measured) — not cheap enough for a
/// routine `cargo test` run, so `ANCHOR_EMIT_SIZE_CAP` alone would not bound it
/// the way it does for the structural anchors; run explicitly via
/// `cargo test --ignored compact_usc_defines_archive_anchors_match_lock`.
#[test]
#[ignore]
fn compact_usc_defines_archive_anchors_match_lock() {
    use pr4xis_domains::cognitive::linguistics::english::ontology::english_load_owned;
    use pr4xis_domains::cognitive::linguistics::verbnet::store::verbnet_classes_loaded;

    let root = workspace_root();
    let lang = english_load_owned();
    let verbnet = verbnet_classes_loaded();
    let mut checked = 0usize;
    for entry in data_sources() {
        if entry.kind != SourceTaxonomyConcept::UsCodeTitle {
            continue;
        }
        let Some(pinned) = lock_compact_defines_signature(&entry.name, &entry.version) else {
            continue; // not pinned yet — the expected pre-`--defines` state
        };
        let path = root.join(entry.local_path());
        let Ok(src) = std::fs::read(&path) else {
            continue; // not provisioned this run — covered when on disk
        };
        let defines_gz = emit_compact_usc_defines_prx_gz(&src, &lang, verbnet)
            .expect("emit pinned USC defines overlay");
        let addr =
            compact_usc_defines_archive_address(&defines_gz).expect("derive defines address");
        assert_eq!(
            &LockDigest::address(addr),
            pinned,
            "{}@{} defines-overlay address must equal its \
             [compact_defines_signatures] pin (grammar or source drift?)",
            entry.name,
            entry.version
        );
        checked += 1;
    }
    if checked == 0 {
        eprintln!(
            "[compact_defines_signatures] has no pins yet — nothing to anchor \
             (run `pr4xis compile --defines --lock` to provision the first one)"
        );
    }
}
