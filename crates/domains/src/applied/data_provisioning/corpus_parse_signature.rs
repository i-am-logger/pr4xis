//! Base-corpus-parse staleness detection for `.prx-cache/usc/*.prx.gz` /
//! `.prx-cache/usc-compact/*.cprx.gz` — a named, file-level CODE
//! dependency-closure fingerprint over the exact source
//! [`read_uslm_title`](crate::social::software::markup::xml::uslm::lens::read_uslm_title)
//! reaches, using the SAME `ContentAddress`/envelope-framing primitive and
//! the SAME `[grammar_signatures]` lock table
//! [`defines_grammar_signature`](super::defines_grammar_signature) already
//! established (a different NAME key in that same map, not a new table).
//!
//! ## Why this exists
//!
//! `loaded()` (`social::software::markup::xml::uslm::corpus::loaded`) loads
//! each USC title from a compiled `.prx.gz`/`.cprx.gz` archive when one is
//! present on disk, admitted through the fail-closed `praxis.lock`
//! content-address gate — but that gate verifies only that the CACHED BYTES
//! match their OWN pin, exactly like `[compact_defines_signatures]` did
//! before [`defines_grammar_signature`](super::defines_grammar_signature)
//! existed. It says nothing about whether the PARSING CODE that produced
//! those bytes
//! (`leaf_readers.rs`'s `read_section`/`read_subdivision`, the
//! `UsCodeMixed::prose_text`/`plain_text` projections in `runtime_types.rs`
//! they call, or the underlying XML tokenizer/parser) has since changed.
//!
//! Confirmed a REAL, measured defect this closes, not a hypothetical one:
//! this session, a footnote-exclusion revert to
//! `runtime_types.rs::NonProseSubtreeKind` had ZERO effect on three
//! consecutive `defines_pointers_corpus_ratchet` runs — each silently
//! re-served a `.prx-cache/usc/usc_title_42-pl-119-90.prx.gz` snapshot
//! baked HOURS before the revert, with nothing anywhere detecting the
//! mismatch. The cache's own content-address gate passed cleanly (the same
//! stale run produced both the cache and its pin), exactly the blind spot
//! [`defines_grammar_signature`](super::defines_grammar_signature) already
//! closes for the DIFFERENT `.defines.cprx.gz` cache — this module closes
//! the SAME class of gap for the base structural corpus.
//!
//! ## What's NOT in the closure
//!
//! `kinds.rs` (`InlineKind`) is deliberately excluded: it governs typed
//! inline-RUN metadata (`UsCodeInlineRun`, small-caps/italic styling) for
//! the byte-exact writer path, not the flat `lexical`/`prose_text` string
//! content `defines_pointers`/`cites_pointers`/`denotes_pointers` actually
//! read — see `NonProseSubtreeKind`'s own doc for why its exclusion logic
//! lives in `runtime_types.rs` rather than `kinds.rs` (the dependency
//! points the other way).
//!
//! ## Maintenance contract
//!
//! When `read_uslm_title`'s own call graph gains or drops a module that
//! affects the FLAT TEXT content a parsed `UsCodeSection`/`UsCodeSubdivision`
//! carries (not just byte-exact-writer-only metadata), update
//! [`CORPUS_PARSE_CLOSURE_FILES`] in the SAME commit/PR — the same
//! discipline
//! [`defines_grammar_signature::DEFINES_GRAMMAR_CLOSURE_FILES`](super::defines_grammar_signature::DEFINES_GRAMMAR_CLOSURE_FILES)
//! already documents.

use std::path::Path;

use pr4xis_runtime::address::ContentAddress;

use super::defines_grammar_signature::closure_bytes;

/// The closure NAME `[grammar_signatures]` keys the base-corpus-parse
/// fingerprint under — a fixed name, sharing the SAME lock table
/// [`DEFINES_OVERLAY_CLOSURE_NAME`](super::defines_grammar_signature::DEFINES_OVERLAY_CLOSURE_NAME)
/// uses, under a different key.
pub const CORPUS_PARSE_CLOSURE_NAME: &str = "corpus_parse";

/// The exact CODE dependency closure of
/// [`read_uslm_title`](crate::social::software::markup::xml::uslm::lens::read_uslm_title)
/// as it affects the FLAT TEXT content baked into a cached
/// `.prx.gz`/`.cprx.gz` archive — workspace-root-relative paths, in the
/// declared, STABLE framing order (never re-sorted) — see
/// [`corpus_parse_closure_bytes`].
pub const CORPUS_PARSE_CLOSURE_FILES: &[&str] = &[
    "crates/domains/src/social/software/markup/xml/uslm/lens/leaf_readers.rs",
    "crates/domains/src/social/software/markup/xml/uslm/corpus/runtime_types.rs",
    "crates/domains/src/social/software/markup/xml/uslm/corpus/mod.rs",
    "crates/domains/src/social/software/markup/xml/uslm/corpus/prx.rs",
    "crates/domains/src/social/software/markup/xml/ontology.rs",
    "crates/domains/src/social/software/markup/xml/parser/grammar.rs",
    "crates/domains/src/social/software/markup/xml/reader.rs",
];

/// Read every [`CORPUS_PARSE_CLOSURE_FILES`] path (resolved against
/// `workspace_root`) and frame it the SAME `blob(path) blob(bytes)` way
/// [`defines_grammar_signature::defines_grammar_closure_bytes`](super::defines_grammar_signature::defines_grammar_closure_bytes)
/// does — see [`closure_bytes`], the shared framing primitive both modules
/// call.
///
/// `Err` if any listed file is missing or unreadable — fail-closed: a
/// closure member that can't be read can't be fingerprinted, so the caller
/// must not silently treat that as "unchanged."
pub fn corpus_parse_closure_bytes(workspace_root: &Path) -> std::io::Result<Vec<u8>> {
    closure_bytes(workspace_root, CORPUS_PARSE_CLOSURE_FILES)
}

/// The content address (BLAKE3 hex, [`ContentAddress::of`]) of
/// [`corpus_parse_closure_bytes`] — the value written to, and compared
/// against, `praxis.lock`'s `[grammar_signatures]."corpus_parse"` pin.
pub fn corpus_parse_closure_address(workspace_root: &Path) -> std::io::Result<String> {
    Ok(ContentAddress::of(&corpus_parse_closure_bytes(workspace_root)?).to_hex())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::applied::data_provisioning::registry::lock_grammar_signature;

    /// The workspace root — the SAME discovery
    /// `defines_grammar_signature`'s own test helper uses.
    fn workspace_root_for_test() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(std::path::PathBuf::from)
            .expect("crates/domains has two ancestor dirs")
    }

    /// STALENESS GUARD (normal suite, cheap): re-derive the corpus-parse
    /// closure address from the ~5 small `.rs` files on disk — milliseconds
    /// — and compare against the `praxis.lock`
    /// `[grammar_signatures]."corpus_parse"` pin. HARD-FAILS (no skip) when
    /// the pin is stale, catching exactly the class of bug this module's
    /// own doc comment cites: a `.prx-cache/usc/*.prx.gz` archive baked
    /// before a `leaf_readers.rs`/`runtime_types.rs` edit, silently served
    /// forever after because its OWN content-address gate still passes.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn corpus_parse_signature_matches_current_source() {
        let workspace_root = workspace_root_for_test();
        let fresh =
            corpus_parse_closure_bytes(&workspace_root).expect("read CORPUS_PARSE_CLOSURE_FILES");
        let Some(pin) = lock_grammar_signature(CORPUS_PARSE_CLOSURE_NAME) else {
            // No pin recorded: nothing to be stale relative to yet.
            return;
        };
        assert!(
            pin.verifies(&fresh),
            "praxis.lock [grammar_signatures].\"corpus_parse\" is STALE: the pinned \
             base-corpus-parse fingerprint no longer matches CORPUS_PARSE_CLOSURE_FILES \
             on disk — parsing code changed since the cached .prx.gz/.cprx.gz archives \
             were last built. Delete the stale .prx-cache/usc/ and .prx-cache/usc-compact/ \
             entries (or regenerate them) and re-pin [grammar_signatures].\"corpus_parse\" \
             in praxis.lock."
        );
    }

    /// NO FALSE POSITIVES: a source change OUTSIDE the declared closure
    /// must never move the fingerprint — the same specificity proof
    /// `defines_grammar_signature` already establishes for its own closure.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn corpus_parse_closure_address_ignores_files_outside_the_closure() {
        let real_root = workspace_root_for_test();

        let build_tree = |extra_unrelated_content: &[u8]| -> tempfile::TempDir {
            let dir = tempfile::tempdir().expect("tempdir");
            for rel in CORPUS_PARSE_CLOSURE_FILES {
                let src = real_root.join(rel);
                let dst = dir.path().join(rel);
                std::fs::create_dir_all(dst.parent().expect("closure path has a parent"))
                    .expect("create closure member's parent dir");
                std::fs::copy(&src, &dst).expect("copy real closure member into temp tree");
            }
            let unrelated = dir
                .path()
                .join("crates/domains/src/social/software/markup/xml/uslm/lens/NOT_IN_CLOSURE.rs");
            std::fs::create_dir_all(unrelated.parent().expect("has a parent"))
                .expect("create unrelated file's parent dir");
            std::fs::write(&unrelated, extra_unrelated_content).expect("write unrelated file");
            dir
        };

        let tree_a = build_tree(b"// sibling file, version A\n");
        let tree_b = build_tree(b"// sibling file, version B -- CONTENT CHANGED\n");

        let addr_a = corpus_parse_closure_address(tree_a.path()).expect("hash tree A");
        let addr_b = corpus_parse_closure_address(tree_b.path()).expect("hash tree B");
        assert_eq!(
            addr_a, addr_b,
            "a file OUTSIDE CORPUS_PARSE_CLOSURE_FILES must never affect the fingerprint, \
             even when it sits in the same directory as a real closure member"
        );
    }

    /// SENSITIVITY: editing a closure MEMBER — the case the staleness guard
    /// exists to catch — DOES move the fingerprint.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn corpus_parse_closure_address_is_sensitive_to_closure_member_edits() {
        let real_root = workspace_root_for_test();

        let build_tree = |first_file_extra_bytes: &[u8]| -> tempfile::TempDir {
            let dir = tempfile::tempdir().expect("tempdir");
            for (i, rel) in CORPUS_PARSE_CLOSURE_FILES.iter().enumerate() {
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

        let addr_a = corpus_parse_closure_address(tree_a.path()).expect("hash tree A");
        let addr_b = corpus_parse_closure_address(tree_b.path()).expect("hash tree B");
        assert_ne!(
            addr_a, addr_b,
            "editing a closure MEMBER must change the fingerprint"
        );
    }
}
