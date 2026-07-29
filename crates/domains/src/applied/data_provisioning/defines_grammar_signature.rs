//! Grammar-staleness detection for `[compact_defines_signatures]` — a
//! named, file-level CODE dependency-closure fingerprint over the exact
//! source [`defines_pointers`](crate::social::judicial::statute_structure::grounding::defines_pointers)
//! (`compute_defines_overlay`) reaches, hashed with the SAME
//! `ContentAddress` / envelope-framing primitive every other `praxis.lock`
//! digest space already uses (`super::prx_envelope::{put_blob, put_varint}`,
//! `pr4xis_runtime::address::ContentAddress`) — no new codec, no new trust
//! primitive, applied to a different input class (Rust source bytes instead
//! of a fetched data artifact).
//!
//! ## Why this exists
//!
//! `[compact_defines_signatures]` (`registry::LockData::compact_defines_signatures`)
//! pins the OUTPUT of an hours-long, deliberately CI-excluded NLP-grammar
//! extraction (`crates/cli/src/main.rs`'s `--defines` gate, only run via
//! `pr4xis compile --defines --lock`). Every OTHER `praxis.lock` digest
//! space is either re-derived (and so re-verified) on every ordinary
//! `pr4xis compile` run, or is the direct output of a `WellBehavedLens`
//! whose own round-trip test re-runs the lens every test cycle — so a
//! code-drift-vs-pin mismatch surfaces "for free." `[compact_defines_signatures]`
//! is the one family where NOTHING re-derives the pinned value on a normal
//! test/CI cycle, so a grammar/parsing-code change can silently drift out
//! from under an already-emitted, still-content-address-valid
//! `.defines.cprx.gz` cache with nothing catching it. This module is that
//! missing check: it fingerprints the CODE, not the data, and is compared
//! against a lockfile pin (`[grammar_signatures]`,
//! `registry::LockData::grammar_signatures`) the SAME way every other digest
//! space is compared — [`registry::LockDigest::verifies`].
//!
//! ## What's NOT in the closure
//!
//! Deliberately does NOT include the three grammar TSVs
//! (`olia-ccg-categories.tsv`, `ccg-supertag-costs.tsv`,
//! `dash-punctuation.tsv`) — those are DATA, already tracked by their own
//! `[hashes]` / `[compact_archive_signatures]` registry pins
//! (`praxis.toml`'s `[sources.olia_ccg_categories]` /
//! `[sources.ccg_supertag_costs]` / `[sources.dash_punctuation]`). This
//! closure covers only the CODE half that has no content address anywhere
//! else in the system.
//!
//! File-level, not directory-level: `verbnet/` and `english/` are large
//! shared directories carrying lots of code unrelated to `defines_pointers`
//! (only `basic_transitive_theme_order` and `ENGLISH_ONTOLOGY`/`form_atom`
//! are actually reached). Directory-wide hashing would reproduce the exact
//! false-positive-coupling problem a whole-crate/`Cargo.lock` hash has, just
//! scoped slightly smaller — file-level is the correct boundary given this
//! codebase's existing "small precise thing" bias, even though it costs hand
//! maintenance.
//!
//! ## Maintenance contract
//!
//! When `defines_pointers`'s own call graph
//! (`crate::social::judicial::statute_structure::grounding`) — or
//! `compute_defines_overlay`'s SCAN-SET construction
//! (`crate::social::software::markup::xml::uslm::corpus::bridge`) — gains or
//! drops a module, update
//! [`DEFINES_GRAMMAR_CLOSURE_FILES`](crate::applied::data_provisioning::defines_grammar_signature::DEFINES_GRAMMAR_CLOSURE_FILES)
//! in the SAME commit/PR — enforced by review, the same discipline already
//! applied to
//! `praxis.toml`'s `[sources.*]` registrations, not by a separate mechanical
//! checker (none exists for either).

use std::path::Path;

use pr4xis_runtime::address::ContentAddress;

use super::prx_envelope::put_blob;

/// The closure NAME `[grammar_signatures]` keys the defines-overlay
/// fingerprint under — a fixed name, not `"<name>@<version>"` (see
/// `registry::LockData::grammar_signatures`'s field doc for why: the
/// grammar is one piece of code shared by every USC title, not a per-source
/// pin).
pub const DEFINES_OVERLAY_CLOSURE_NAME: &str = "defines_overlay";

/// The exact CODE dependency closure of `compute_defines_overlay` /
/// [`defines_pointers`](crate::social::judicial::statute_structure::grounding::defines_pointers)
/// — the grammar/parsing source whose behavior `[compact_defines_signatures]`
/// pins are a cached OUTPUT of. Workspace-root-relative paths, in the
/// declared, STABLE framing order (never re-sorted) — see
/// [`defines_grammar_closure_bytes`].
pub const DEFINES_GRAMMAR_CLOSURE_FILES: &[&str] = &[
    "crates/domains/src/social/judicial/statute_structure/grounding.rs",
    // `compute_defines_overlay` ITSELF, plus the two side-table walks that
    // decide WHICH provision text is ever offered to `defines_pointers`
    // (`defines_prose_index`, `dangling_chapeau_reassembly_index`). The
    // overlay's content is a function of the scan set AND the grammar, so a
    // scan-set change moves `[compact_defines_signatures]` exactly as a
    // grammar change does. Omitted until 2026-07-25, when widening
    // `defines_prose_index` to index subdivision-less section bodies took
    // Title 1 from 6 pairs to 10 without moving this fingerprint at all —
    // the precise silent-drift this module exists to prevent.
    "crates/domains/src/social/software/markup/xml/uslm/corpus/bridge.rs",
    "crates/domains/src/cognitive/linguistics/lambek/tokenize.rs",
    "crates/domains/src/cognitive/linguistics/lambek/category_projection.rs",
    "crates/domains/src/cognitive/linguistics/lambek/reduce.rs",
    "crates/domains/src/cognitive/linguistics/lambek/supertag_costs.rs",
    "crates/domains/src/cognitive/linguistics/lambek/types.rs",
    "crates/domains/src/cognitive/linguistics/lambek/montague.rs",
    "crates/domains/src/cognitive/linguistics/morphology/lemmatizer.rs",
    "crates/domains/src/cognitive/linguistics/verbnet/ontology.rs",
    "crates/domains/src/cognitive/linguistics/lemon/lexicon.rs",
    "crates/domains/src/cognitive/linguistics/lemon/mint.rs",
    "crates/domains/src/cognitive/linguistics/english/bridge.rs",
    // The LEXICAL-ASSIGNMENT half of the grammar: which `LexicalEntry`s (and
    // which OLiA form-level marks on them) a surface resolves to is as much
    // an input to the derivation as the chart rules are. `tokenize.rs` above
    // only decides what to DO with the entries it is handed; these decide
    // what it is handed at all. Omitted until 2026-07-25, when adding the
    // past-participle form-class mark (`English::lexical_lookup_all`'s
    // participle block, `olia::form_level_class`,
    // `generation::is_past_participle_form_of`, and the AGID slot layout
    // `irregular.rs` reads) changed which spans yield pointers — 42 U.S.C.
    // § 1396b(l)(5)(A)/(B) among them — through files none of which the
    // closure named, the SAME silent-drift class the `bridge.rs` note above
    // records.
    "crates/domains/src/cognitive/linguistics/english/ontology.rs",
    "crates/domains/src/cognitive/linguistics/lexicon/olia.rs",
    "crates/domains/src/cognitive/linguistics/morphology/english/generation.rs",
    "crates/domains/src/cognitive/linguistics/morphology/english/irregular.rs",
];

/// Read every path in `files` (resolved against `workspace_root`) and frame
/// it `blob(path) blob(bytes)` — the SAME `put_blob` LEB128
/// length-prefixed-blob framing the raw-source envelope uses
/// (`prx_envelope::encode_raw_source`), applied in the caller's declared
/// order. Framing the PATH alongside the bytes means a file rename/move
/// inside the closure is detected too, not just an in-place content edit —
/// for free, from the existing idiom, not a new property invented for this
/// check.
///
/// Shared by every named CODE-dependency-closure fingerprint in
/// `[grammar_signatures]` — [`defines_grammar_closure_bytes`] (this
/// module's own `"defines_overlay"` closure) and
/// [`super::corpus_parse_signature::corpus_parse_closure_bytes`] (the
/// `"corpus_parse"` closure) both call this rather than re-implementing the
/// same framing loop.
///
/// `Err` if any listed file is missing or unreadable — fail-closed: a
/// closure member that can't be read can't be fingerprinted, so the caller
/// must not silently treat that as "unchanged."
pub fn closure_bytes(workspace_root: &Path, files: &[&str]) -> std::io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    for rel in files {
        let bytes = std::fs::read(workspace_root.join(rel)).map_err(|e| {
            std::io::Error::new(e.kind(), format!("code closure member `{rel}`: {e}"))
        })?;
        put_blob(&mut buf, rel.as_bytes());
        put_blob(&mut buf, &bytes);
    }
    Ok(buf)
}

/// Read every [`DEFINES_GRAMMAR_CLOSURE_FILES`] path — the
/// `"defines_overlay"` closure specifically. See [`closure_bytes`] for the
/// shared framing this (and every other named closure) uses.
pub fn defines_grammar_closure_bytes(workspace_root: &Path) -> std::io::Result<Vec<u8>> {
    closure_bytes(workspace_root, DEFINES_GRAMMAR_CLOSURE_FILES)
}

/// The content address (BLAKE3 hex, [`ContentAddress::of`]) of
/// [`defines_grammar_closure_bytes`] — the value written to, and compared
/// against, `praxis.lock`'s `[grammar_signatures]."defines_overlay"` pin.
/// This is literally `ContentAddress::of(bytes).to_hex()` — the same
/// primitive every one of the other `praxis.lock` digest spaces already
/// uses (`registry.rs`), applied to a different input class (source bytes
/// instead of a fetched data artifact). No new hash algorithm, no new
/// codec, no new trust primitive.
pub fn defines_grammar_closure_address(workspace_root: &Path) -> std::io::Result<String> {
    Ok(ContentAddress::of(&defines_grammar_closure_bytes(workspace_root)?).to_hex())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::applied::data_provisioning::registry::lock_grammar_signature;

    /// The workspace root — the SAME discovery
    /// `registry_prx.rs`'s `workspace_manifest_paths` uses (`crates/domains`
    /// has two ancestor dirs up to the workspace root).
    fn workspace_root_for_test() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(std::path::PathBuf::from)
            .expect("crates/domains has two ancestor dirs")
    }

    /// STALENESS GUARD (normal suite, cheap): re-derive the grammar-closure
    /// address from the ~12 small `.rs` files on disk — milliseconds, a
    /// handful of file reads plus one BLAKE3 hash, NOT a re-run of the
    /// hours-long `compute_defines_overlay` extraction — and compare
    /// against the `praxis.lock` `[grammar_signatures]."defines_overlay"`
    /// pin. HARD-FAILS (no skip) when the pin is stale — the SAME
    /// discipline `committed_registry_prx_matches_workspace_root_manifest`
    /// applies at the registry layer, and
    /// `defines_grammar_closure_address_is_sensitive_to_closure_member_edits`
    /// below proves the fingerprint really does move when it should.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn defines_grammar_signature_matches_current_source() {
        let workspace_root = workspace_root_for_test();
        let fresh = defines_grammar_closure_bytes(&workspace_root)
            .expect("read DEFINES_GRAMMAR_CLOSURE_FILES");
        let Some(pin) = lock_grammar_signature(DEFINES_OVERLAY_CLOSURE_NAME) else {
            // No pin recorded at all: `[compact_defines_signatures]` is
            // empty (parse-time already enforces the pin must exist once it
            // isn't — `registry::parse_praxis_lock`), so there is nothing
            // to be stale relative to.
            return;
        };
        assert!(
            pin.verifies(&fresh),
            "praxis.lock [grammar_signatures].\"defines_overlay\" is STALE: the pinned \
             grammar fingerprint no longer matches DEFINES_GRAMMAR_CLOSURE_FILES on disk \
             — grammar/parsing code changed since the compact_defines_signatures overlay \
             was last computed. Re-run `pr4xis compile --defines --lock` (this is the \
             EXPENSIVE, hours-long regen — this test only detects that it's needed, it \
             does not run it) and commit the refreshed pins."
        );
    }

    /// NO FALSE POSITIVES: a source change OUTSIDE the declared closure —
    /// even one living in the SAME directory as a real closure member —
    /// must never move the fingerprint. The function reads only the
    /// explicitly enumerated `DEFINES_GRAMMAR_CLOSURE_FILES` paths, nothing
    /// implicit (no directory walk, no glob), proven here by constructing
    /// two otherwise-identical workspace trees that differ ONLY in a file
    /// that is NOT a closure member and asserting the derived address is
    /// identical.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn defines_grammar_closure_address_ignores_files_outside_the_closure() {
        let real_root = workspace_root_for_test();

        let build_tree = |extra_unrelated_content: &[u8]| -> tempfile::TempDir {
            let dir = tempfile::tempdir().expect("tempdir");
            for rel in DEFINES_GRAMMAR_CLOSURE_FILES {
                let src = real_root.join(rel);
                let dst = dir.path().join(rel);
                std::fs::create_dir_all(dst.parent().expect("closure path has a parent"))
                    .expect("create closure member's parent dir");
                std::fs::copy(&src, &dst).expect("copy real closure member into temp tree");
            }
            // An unrelated file, deliberately placed in the SAME directory
            // as a real closure member (`lambek/`) — the adversarial case
            // for a directory-scoped (rather than file-scoped) hash.
            let unrelated = dir
                .path()
                .join("crates/domains/src/cognitive/linguistics/lambek/NOT_IN_CLOSURE.rs");
            std::fs::write(&unrelated, extra_unrelated_content).expect("write unrelated file");
            dir
        };

        let tree_a = build_tree(b"// sibling file, version A\n");
        let tree_b = build_tree(b"// sibling file, version B -- CONTENT CHANGED\n");

        let addr_a = defines_grammar_closure_address(tree_a.path()).expect("hash tree A");
        let addr_b = defines_grammar_closure_address(tree_b.path()).expect("hash tree B");
        assert_eq!(
            addr_a, addr_b,
            "a file OUTSIDE DEFINES_GRAMMAR_CLOSURE_FILES must never affect the fingerprint, \
             even when it sits in the same directory as a real closure member"
        );
    }

    /// SENSITIVITY (the complement of the specificity proof above): editing
    /// a closure MEMBER — the case the staleness guard exists to catch —
    /// DOES move the fingerprint.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn defines_grammar_closure_address_is_sensitive_to_closure_member_edits() {
        let real_root = workspace_root_for_test();

        let build_tree = |first_file_extra_bytes: &[u8]| -> tempfile::TempDir {
            let dir = tempfile::tempdir().expect("tempdir");
            for (i, rel) in DEFINES_GRAMMAR_CLOSURE_FILES.iter().enumerate() {
                let src = real_root.join(rel);
                let dst = dir.path().join(rel);
                std::fs::create_dir_all(dst.parent().expect("closure path has a parent"))
                    .expect("create closure member's parent dir");
                let mut bytes = std::fs::read(&src).expect("read real closure member");
                if i == 0 {
                    bytes.extend_from_slice(first_file_extra_bytes);
                }
                std::fs::write(&dst, &bytes).expect("write closure member into temp tree");
            }
            dir
        };

        let tree_a = build_tree(b"");
        let tree_b = build_tree(b"// one appended comment line\n");

        let addr_a = defines_grammar_closure_address(tree_a.path()).expect("hash tree A");
        let addr_b = defines_grammar_closure_address(tree_b.path()).expect("hash tree B");
        assert_ne!(
            addr_a, addr_b,
            "editing a closure MEMBER must change the fingerprint"
        );
    }
}
