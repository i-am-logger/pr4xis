//! Statute-lens axioms that run over the REAL U.S. Code corpus — lifted out of
//! the `pr4xis-domains` `#[cfg(test)]` modules.
//!
//! `BytesToStatuteOnRealTitle18.verify()` reads the actual ~12 MB USC Title 18
//! (P.L. 119-90) USLM bytes, focuses §1514A, projects it to a `Statute`, and
//! checks the byte-boundary GetPut law. That parse is heavy: under nextest it is
//! paid per process-isolated test; here all `#[test]`s run as threads in one
//! process. The test require()-gates on the title's presence: an absent corpus
//! hard-fails with the `pr4xis update` hint (the crate's "tests do not skip"
//! contract), never a silent pass; with the title provisioned, any real lens-law
//! or projection regression fails it.

use pr4xis::ontology::Axiom;
use pr4xis_domains::social::compliance::statutes::lens::BytesToStatuteOnRealTitle18;
use praxis_corpus_tests::{load_uslm_corpus, require};

/// The full `bytes ⇄ Statute` lens chain is well-behaved on the REAL USC
/// Title 18 (P.L. 119-90) bytes. require()-gates on the title's presence, so an
/// unprovisioned checkout hard-fails with the `pr4xis update usc_title_18` hint (no
/// false-green); fails on any real lens-law or projection regression on present
/// bytes.
#[test]
fn axiom_bytes_to_statute_on_real_title_18() {
    // Fail loud (not a silent skip) when the title isn't provisioned — the same
    // title `BytesToStatuteOnRealTitle18::verify()` reads internally. Matches the
    // sibling oracles and the crate's `require()` contract.
    require(
        load_uslm_corpus("legal/uscode/usc_title_18/usc_title_18-pl-119-90.xml"),
        "usc_title_18",
    );
    assert!(BytesToStatuteOnRealTitle18.verify().is_ok());
}
