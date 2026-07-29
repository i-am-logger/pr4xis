//! Full-corpus equivalence gate for the runtime's `MaterializedClosure` — the
//! u32-CSR lazy engine — against the INDEPENDENT eager `ReachabilityClosure`
//! oracle, over a real loaded USC title at full scale.
//!
//! This is the `#[test]` driver for the registered, cited
//! [`MaterializedClosureMatchesEagerOracle`] axiom (lifted out of a raw
//! `assert_eq!` differential — its `verify()` reads the real Title 42 USLM
//! bytes, materializes the u32-CSR engine, and checks it answers every graded
//! query — image/reachable/reaches/chain/meet — identically to the eager Floyd
//! oracle both up the mereology and down the inverse fan-out). The test
//! `require()`-gates on the title's presence, so an unprovisioned checkout
//! hard-fails with the `pr4xis update usc` hint (the crate's "tests do not
//! skip" contract), never a silent pass.

use pr4xis::ontology::Axiom;
use pr4xis_domains::formal::meta::reach_laws::MaterializedClosureMatchesEagerOracle;
use praxis_corpus_tests::{load_uslm_corpus, require};

#[test]
fn materialized_closure_matches_eager_oracle_over_a_full_usc_title() {
    // Fail loud (not a silent skip) when Title 42 — the largest provisioned
    // title, the same corpus `MaterializedClosureMatchesEagerOracle::verify()`
    // reads internally — isn't on disk.
    require(
        load_uslm_corpus("legal/uscode/usc_title_42/usc_title_42-pl-119-90.xml"),
        "usc",
    );
    assert!(MaterializedClosureMatchesEagerOracle.verify().is_ok());
}
