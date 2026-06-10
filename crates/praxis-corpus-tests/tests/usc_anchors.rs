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
//! here. USC titles are externally provisioned (`pr4xis update`), so a plain
//! checkout has none on disk and both gates skip gracefully.

use pr4xis_domains::applied::data_provisioning::registry::{
    LockDigest, data_sources, lock_archive_signature, lock_compact_archive_signature,
};
use pr4xis_domains::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
use pr4xis_domains::social::software::markup::xml::owl::prx::prx_archive_address;
use pr4xis_domains::social::software::markup::xml::uslm::corpus::prx::{
    compact_prx_archive_address, emit_compact_usc_prx_gz, emit_usc_prx_gz,
};
use praxis_corpus_tests::workspace_root;

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
    for entry in data_sources() {
        if entry.kind != SourceTaxonomyConcept::UsCodeTitle {
            continue;
        }
        let Some(pinned) = lock_archive_signature(&entry.name, &entry.version) else {
            continue; // not pinned — nothing to anchor
        };
        let path = root.join(entry.local_path());
        let Ok(meta) = std::fs::metadata(&path) else {
            continue; // not provisioned on disk — skip gracefully
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
    }
}

/// COMPACT ARCHIVE ANCHOR — for every on-disk title within the budget that has a
/// `[compact_archive_signatures]` pin, a fresh `emit_compact_usc_prx_gz`
/// re-derives EXACTLY that pin. The portable (toolchain-independent) compact
/// sibling of `usc_archive_anchors_match_lock`; keeps the committed compact pins
/// honest — a stale or wrong pin (or a codec change that shifts the bytes) fails
/// closed here. Skips titles > the cap and titles not on disk.
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
    if checked == 0 {
        eprintln!("compact anchor: no pinned on-disk USC title within the cap — skipped");
    }
}
