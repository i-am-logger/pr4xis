//! Heavy-corpus test fixtures for praxis.
//!
//! The multi-hundred-MB U.S. Code titles (Title 42 is 113 MB) and the 89 MB
//! WordNet corpus are too large to re-parse per test. Under nextest, every
//! `#[test]` runs in its own OS process, so a process-local cache re-parses the
//! giant once per test; with ~6 Title-42 assertions that is 6× a 113 MB parse.
//!
//! These tests run under `cargo test` instead: all `#[test]`s in one test
//! binary are threads in ONE process, so a [`std::sync::LazyLock`] fixture
//! parses each giant exactly ONCE and every assertion borrows the shared,
//! immutable result. That mirrors praxis's own discipline — the `.prx` IS the
//! parse-once-immutable artifact — applied to the test suite.
//!
//! This crate is excluded from the default workspace (see the root
//! `Cargo.toml`) so the heavy lane never runs under `cargo test --workspace` /
//! nextest. CI runs it explicitly with
//! `cargo test --manifest-path crates/praxis-corpus-tests/Cargo.toml`.

use std::path::PathBuf;

use pr4xis_domains::social::software::markup::xml::uslm::{UsCodeTitle, read_uslm_title};

/// A USLM corpus parsed once: the raw XML (kept so the codegen path can re-read
/// it) alongside the parsed [`UsCodeTitle`].
pub struct UslmCorpus {
    /// The raw on-disk USLM XML.
    pub xml: String,
    /// The parsed title.
    pub title: UsCodeTitle,
}

/// Absolute path to `crates/domains/data`, where the giant corpora live.
///
/// Resolved relative to THIS crate's manifest so it is stable regardless of the
/// working directory the test runner is invoked from.
pub fn domains_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../domains/data")
}

/// Load and parse a USLM title from `crates/domains/data/<rel>`.
///
/// Returns `None` when the giant is not on disk — the multi-hundred-MB USC
/// titles are fetched (`pr4xis update`), not committed, so the corpus tests
/// skip gracefully on a fresh checkout. A file that IS present but fails to
/// parse is a hard error: the on-disk corpus is expected to be well-formed.
pub fn load_uslm_corpus(rel: &str) -> Option<UslmCorpus> {
    let path = domains_data_dir().join(rel);
    let xml = std::fs::read_to_string(&path).ok()?;
    let title = read_uslm_title(&xml).expect("on-disk USLM corpus must parse");
    Some(UslmCorpus { xml, title })
}
