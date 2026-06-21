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

/// Hard-fail accessor for an absent corpus — the SINGLE source of truth for
/// "the corpus must be on disk".
///
/// The heavy corpora (USC titles, the 89 MB WordNet) are fetched in CI by
/// `pr4xis update`, never committed. A test that finds its corpus ABSENT must
/// fail loud — it cannot assert anything, so a silent skip would be a
/// false-green (PASS while testing nothing). This unwraps the loader's
/// `Option`, panicking with the exact `pr4xis update <corpus>` to run so the
/// operator knows what to fetch. Tests do not skip.
///
/// `corpus` is the registry source name (the `pr4xis update` argument), e.g.
/// `"usc_title_18"` or `"english_wordnet"`.
#[track_caller]
pub fn require<T>(opt: Option<T>, corpus: &str) -> T {
    opt.unwrap_or_else(|| {
        panic!(
            "corpus `{corpus}` not on disk — run `pr4xis update {corpus}` to fetch it; \
             tests do not skip"
        )
    })
}

/// Hard-fail when a corpus-set scan provisioned NOTHING on disk.
///
/// The sibling of [`require`] for the loop-aggregate gates that iterate the
/// data-source registry and `continue` past each absent title (so they
/// legitimately cover whatever IS on disk). If the loop measured ZERO sources
/// it asserted nothing — a false-green — so this panics naming the
/// `pr4xis update` family to fetch. `count` is the number of sources the loop
/// actually measured; `corpus_family` describes what to provision (e.g.
/// `"usc"` USC titles, or `"english_wordnet"`). Tests do not skip.
#[track_caller]
pub fn require_provisioned(count: usize, corpus_family: &str) {
    assert!(
        count > 0,
        "no `{corpus_family}` corpus provisioned on disk — run `pr4xis update {corpus_family}` \
         (or `pr4xis update --list`) to fetch it; tests do not skip"
    );
}

/// Borrow a per-file `LazyLock<Option<C>>` corpus fixture, hard-failing via
/// [`require`] when the corpus is absent.
///
/// Each `tests/title_*.rs` file owns a file-local `LazyLock<Option<UslmCorpus>>`
/// (the giant is parsed once per binary), so the borrow can't live in a plain
/// function — it must expand at the call site against that static. This macro
/// is that borrow, routed through the shared hard-fail so the skip cannot
/// reappear file-by-file. Pass the static and the registry corpus name:
///
/// ```text
/// let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
/// ```
#[macro_export]
macro_rules! corpus_or_fail {
    ($lazy:ident, $corpus:expr) => {
        $crate::require((&*$lazy).as_ref(), $corpus)
    };
}
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
/// titles are fetched (`pr4xis update`), not committed. Callers route the
/// `None` through [`require`] / [`corpus_or_fail!`] so an absent corpus
/// HARD-FAILS the test (it can assert nothing); the test layer never skips. A
/// file that IS present but fails to parse is a hard error: the on-disk corpus
/// is expected to be well-formed.
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
    /// The full English WordNet source, or `None` when not on disk. Callers
    /// route the `None` through [`require`] so an absent `english_wordnet`
    /// HARD-FAILS the test naming `pr4xis update english_wordnet`; tests never
    /// skip on absence.
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
