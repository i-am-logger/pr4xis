//! Constitution coverage — the self-binding meta-test.
//!
//! Every test in this crate declares the guarantee it witnesses via
//! `#[pr4xis::praxis_value(..)]`, which registers a
//! [`GuaranteeTag`](pr4xis::constitution::GuaranteeTag) into the
//! [`CONSTITUTION_TESTS`](pr4xis::constitution::CONSTITUTION_TESTS) registry at
//! link time. This module folds that registry into a per-guarantee partition
//! of the suite and asserts every guarantee has a witness. The numbers it
//! prints are the constitution's own coverage, re-derived by running the test:
//!
//! ```text
//! cargo test -p pr4xis-domains --lib -- constitution_coverage -- --nocapture
//! ```
//!
//! The classification is *declared*, not reverse-engineered from test names —
//! a domain test whose name says nothing about "verifiability" still counts
//! toward `Verifiable` because it carries the tag.
//!
//! This test asserts *partition* coverage (every guarantee has a witness) and
//! that the multi-witness path is exercised. *Completeness* — that no test
//! escapes a tag — is a separate gate (`scripts/constitution-gate.sh`) that
//! compares the registry length to the live `--list` count; it becomes a hard
//! CI gate once the suite is fully backfilled. Until then it reports the
//! untagged remainder rather than failing the build.

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::constitution::{
        CONSTITUTION_TESTS, EveryAxiomCarriesItsExplanation, ExtensiblePreservesEveryGuarantee,
        Guarantee, OntologyBaseIsConsistent, TestKind,
    };
    use pr4xis::ontology::Axiom;

    /// The `Consistent` guarantee, witnessed structurally: run the
    /// [`OntologyBaseIsConsistent`] axiom over the FULL `pr4xis-domains` axiom
    /// base (every domain axiom + the catalog's structural axioms register into
    /// this crate's binary). A single universal check over the whole corpus —
    /// not a sample — so `Consistent` is enforced, not stated.
    #[pr4xis::praxis_value(Consistent)]
    #[test]
    fn ontology_base_is_consistent() {
        assert!(
            OntologyBaseIsConsistent.verify().is_ok(),
            "the registered ontology base derives a contradiction — some axiom is refuted by the corpus",
        );
    }

    /// The `Explainable` guarantee, witnessed structurally over the full corpus:
    /// every registered axiom's verdict carries a complete explanation (name +
    /// what it proves + citation). The proof object IS the explanation, checked
    /// universally, so the reasoning path is enforced to be the answer.
    #[pr4xis::praxis_value(Explainable)]
    #[test]
    fn every_axiom_carries_its_explanation() {
        assert!(
            EveryAxiomCarriesItsExplanation.verify().is_ok(),
            "a registered axiom returns a verdict with an incomplete explanation (missing name, claim, or citation)",
        );
    }

    /// The `Extensible` guarantee, modeled and checked as the second-order
    /// property it is: composition preserves every answer-guarantee. The axiom
    /// verifies the meta-structure (Extensible points at all five via
    /// `Preserves`); the workspace's functor-law tests discharge it operationally.
    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn extensible_preserves_every_guarantee() {
        assert!(
            ExtensiblePreservesEveryGuarantee.verify().is_ok(),
            "Extensible does not point at every answer-guarantee via Preserves — the meta-structure is broken",
        );
    }

    /// Partition the tagged suite across the five guarantees and report each
    /// guarantee's breadth (primary count) and depth (how many of those
    /// witnesses are ∀-claim properties). Asserts every guarantee has a
    /// primary witness and that the secondary (multi-witness) path is
    /// exercised. Prints the breakdown (run with `--nocapture`).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn constitution_coverage() {
        let total = CONSTITUTION_TESTS.len();
        assert!(
            total > 0,
            "no tests carry #[pr4xis::praxis_value(..)] — the constitution has no witnesses",
        );

        let properties = CONSTITUTION_TESTS
            .iter()
            .filter(|t| t.kind == TestKind::Property)
            .count();
        println!(
            "\n=== Constitution coverage: {total} tagged tests ({properties} ∀-properties) ==="
        );

        let mut secondary_links = 0usize;
        for g in Guarantee::variants() {
            let primary = CONSTITUTION_TESTS.iter().filter(|t| t.primary == g).count();
            let props = CONSTITUTION_TESTS
                .iter()
                .filter(|t| t.primary == g && t.kind == TestKind::Property)
                .count();
            let secondary = CONSTITUTION_TESTS
                .iter()
                .filter(|t| t.secondary.contains(&g))
                .count();
            secondary_links += secondary;
            let pct = primary as f64 * 100.0 / total as f64;
            println!(
                "  {g:>13?}: {primary:>5} primary ({pct:>4.1}%, {props} ∀)  +{secondary} secondary"
            );
            assert!(
                primary > 0,
                "guarantee {g:?} has no test that primarily witnesses it",
            );
        }

        assert!(
            secondary_links > 0,
            "no test declares a secondary guarantee — the multi-witness (cover) path is unverified",
        );

        // Emit each tag's full path for the identity-based completeness gate
        // (scripts/constitution-gate.sh) to diff against `cargo test --list`.
        //
        // Prefer a FILE over stdout: scraping ~6.7k `--nocapture` lines through a
        // pipe drops lines non-deterministically (libtest output buffering), which
        // made the gate flaky. When `PRAXIS_CONSTITUTION_TAGS_OUT` is set the gate
        // gets the full set atomically; otherwise we still print for ad-hoc use.
        let lines: Vec<String> = CONSTITUTION_TESTS
            .iter()
            .map(|t| format!("{}::{}", t.module, t.name))
            .collect();
        if let Ok(path) = std::env::var("PRAXIS_CONSTITUTION_TAGS_OUT") {
            std::fs::write(&path, lines.join("\n")).expect("write constitution tags file");
        } else {
            for line in &lines {
                println!("PRAXIS_TAG {line}");
            }
        }
    }
}
