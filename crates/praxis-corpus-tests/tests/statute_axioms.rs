//! Statute-lens axioms that run over the REAL U.S. Code corpus — lifted out of
//! the `pr4xis-domains` `#[cfg(test)]` modules.
//!
//! `BytesToStatuteOnRealTitle18.verify()` reads the actual ~12 MB USC Title 18
//! (P.L. 119-90) USLM bytes, focuses §1514A, projects it to a `Statute`, and
//! checks the byte-boundary GetPut law. That parse is heavy: under nextest it is
//! paid per process-isolated test; here all `#[test]`s run as threads in one
//! process. The axiom soft-passes when the corpus is not on disk (a checkout that
//! hasn't run `pr4xis update`), so a plain checkout passes; with the title
//! provisioned, any real lens-law or projection regression fails it.

use pr4xis::ontology::Axiom;
use pr4xis_domains::social::compliance::statutes::lens::BytesToStatuteOnRealTitle18;

/// The full `bytes ⇄ Statute` lens chain is well-behaved on the REAL USC
/// Title 18 (P.L. 119-90) bytes. Soft-passes if the bytes aren't on disk (the
/// committer didn't `pr4xis update`); fails on any real lens-law or projection
/// regression on present bytes.
#[test]
fn axiom_bytes_to_statute_on_real_title_18() {
    assert!(BytesToStatuteOnRealTitle18.verify().is_ok());
}
