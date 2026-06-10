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

use pr4xis_domains::social::software::markup::xml::lmf::WordNet;
use pr4xis_domains::social::software::markup::xml::lmf::reader::read_wordnet;
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

/// Absolute path to the workspace root (the praxis repo root).
///
/// The data-source registry reports each source's `local_path()` relative to
/// the workspace root, so corpus-wide gates resolve sources via this.
pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
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

/// One on-disk WN-LMF source, parsed once: the raw bytes (kept for gzip-size /
/// digest / byte-exact comparisons and for the path-based codegen parser)
/// alongside the parsed [`WordNet`].
pub struct WnSource {
    /// The registry name (`"english_wordnet"` or `"us_legal_lexicon"`).
    pub name: &'static str,
    /// The resolved on-disk path (the codegen parser takes a `&Path`).
    pub path: PathBuf,
    /// The raw WN-LMF XML bytes.
    pub source: Vec<u8>,
    /// The parsed WordNet ontology.
    pub wn: WordNet,
}

/// The WN-LMF corpus parsed once: the 89 MB `english_wordnet` and the small
/// `us_legal_lexicon`, whichever are on disk. The WordNet producer/round-trip
/// tests share this so the 89 MB parse is paid once for the whole test binary.
pub struct WnCorpus {
    /// The on-disk sources, in registry order (`us_legal_lexicon` then
    /// `english_wordnet`), skipping any absent on a fresh checkout.
    pub sources: Vec<WnSource>,
}

impl WnCorpus {
    /// The full English WordNet source, or `None` when not on disk.
    pub fn english(&self) -> Option<&WnSource> {
        self.sources.iter().find(|s| s.name == "english_wordnet")
    }
}

/// Load and parse every on-disk WN-LMF source under `crates/domains/data`.
///
/// A source absent on disk is skipped (the 89 MB WordNet is fetched, not
/// committed); a source that IS present but fails to parse is a hard error.
pub fn load_wordnet_corpus() -> WnCorpus {
    const SPECS: [(&str, &str); 2] = [
        ("us_legal_lexicon", "legal-text/us_legal_lexicon.xml"),
        ("english_wordnet", "wordnet/english-wordnet-2025.xml"),
    ];
    let mut sources = Vec::new();
    for (name, rel) in SPECS {
        let path = domains_data_dir().join(rel);
        let Ok(source) = std::fs::read(&path) else {
            continue;
        };
        let text = std::str::from_utf8(&source).expect("on-disk WN-LMF must be UTF-8");
        let wn = read_wordnet(text).expect("on-disk WN-LMF corpus must parse");
        sources.push(WnSource {
            name,
            path,
            source,
            wn,
        });
    }
    WnCorpus { sources }
}
