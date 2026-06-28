//! Tests for [`super::rule_set`]. Three layers per
//! `feedback_high_test_coverage`: hand-verified pinpoint cases,
//! axiom verification, proptest property invariants.

use super::super::implication::{DeonticOperator, Implication};
use super::*;
use pr4xis::ontology::Axiom;
use proptest::prelude::*;

// =============================================================================
// Layer 1 — pinpoint cases on hand-verified small rule sets.
// =============================================================================

fn canonical_rule_set() -> RuleSet<&'static str> {
    RuleSet::from_rules(vec![
        Implication::assertoric(vec![], vec!["a"]),
        Implication::assertoric(vec!["b"], vec!["a"]),
        Implication::assertoric(vec!["b", "c"], vec!["a"]),
        Implication::obligation(vec![], vec!["d"]),
        Implication::prohibition(vec![], vec!["d"]),
        Implication::obligation(vec!["b"], vec!["e"]),
        Implication::prohibition(vec!["b"], vec!["e"]),
    ])
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn implication_subsumption_basic() {
    // r0: ⊤ ⇒ a   subsumes   r1: {b} ⇒ a
    let r0: Implication<&str> = Implication::assertoric(vec![], vec!["a"]);
    let r1: Implication<&str> = Implication::assertoric(vec!["b"], vec!["a"]);
    assert!(r0.subsumes(&r1));
    assert!(!r1.subsumes(&r0));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn implication_subsumption_reflexive() {
    let r: Implication<&str> = Implication::assertoric(vec!["x"], vec!["y"]);
    assert!(r.subsumes(&r));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn implication_subsumption_consequent_must_be_superset() {
    // r1: {a} ⇒ {x, y}   subsumes   r2: {a} ⇒ {x}
    let r1: Implication<&str> = Implication::assertoric(vec!["a"], vec!["x", "y"]);
    let r2: Implication<&str> = Implication::assertoric(vec!["a"], vec!["x"]);
    assert!(r1.subsumes(&r2));
    assert!(!r2.subsumes(&r1));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn implication_subsumption_respects_deontic_dimension() {
    // Same antecedent + consequent, different deontic — neither
    // subsumes the other.
    let o: Implication<&str> = Implication::obligation(vec!["a"], vec!["x"]);
    let f: Implication<&str> = Implication::prohibition(vec!["a"], vec!["x"]);
    assert!(!o.subsumes(&f));
    assert!(!f.subsumes(&o));
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn implication_normalize_idempotent() {
    let r: Implication<&str> = Implication::new(
        vec!["b", "a", "b"],
        vec!["y", "x"],
        DeonticOperator::Obligation,
    );
    let once = r.clone().normalize();
    let twice = once.clone().normalize();
    assert_eq!(once, twice);
    // Already sorted+deduped by `new`.
    assert_eq!(once.antecedent(), &["a", "b"]);
    assert_eq!(once.consequent(), &["x", "y"]);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn implication_conflict_classic_deontic() {
    // Op(d) vs Fp(d) — von Wright canonical conflict.
    let o: Implication<&str> = Implication::obligation(vec![], vec!["d"]);
    let f: Implication<&str> = Implication::prohibition(vec![], vec!["d"]);
    assert!(o.conflicts_with(&f));
    assert!(f.conflicts_with(&o));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn implication_no_conflict_when_targets_differ() {
    // Op(a) vs Fp(b) — different targets, no conflict.
    let o: Implication<&str> = Implication::obligation(vec![], vec!["a"]);
    let f: Implication<&str> = Implication::prohibition(vec![], vec!["b"]);
    assert!(!o.conflicts_with(&f));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn implication_no_conflict_when_antecedents_disjoint() {
    // Op(d) given x, Fp(d) given y — disjoint conditions, no conflict.
    let o: Implication<&str> = Implication::obligation(vec!["x"], vec!["d"]);
    let f: Implication<&str> = Implication::prohibition(vec!["y"], vec!["d"]);
    assert!(!o.conflicts_with(&f));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn implication_conflict_unconditional_rule_always_fires() {
    // Op(d) ⊤, Fp(d) given some condition — unconditional rule
    // always fires, so they conflict whenever the conditional one
    // could fire.
    let o: Implication<&str> = Implication::obligation(vec![], vec!["d"]);
    let f: Implication<&str> = Implication::prohibition(vec!["any"], vec!["d"]);
    assert!(o.conflicts_with(&f));
    assert!(f.conflicts_with(&o));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn rule_set_subsumption_order_identifies_strict_subsumers() {
    let rs = canonical_rule_set();
    let order = rs.subsumption_order();
    // r0 (⊤ ⇒ a) subsumes r1 ({b} ⇒ a) and r2 ({b, c} ⇒ a).
    assert!(order.contains(&(0, 1)));
    assert!(order.contains(&(0, 2)));
    // r1 ({b} ⇒ a) subsumes r2 ({b, c} ⇒ a).
    assert!(order.contains(&(1, 2)));
    // Not the other direction.
    assert!(!order.contains(&(1, 0)));
    assert!(!order.contains(&(2, 1)));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn rule_set_most_general_picks_unconditional() {
    let rs = canonical_rule_set();
    let mg = rs.most_general();
    // r0 is unconditional and subsumes the other assertoric rules
    // → it's a maximal element. r3, r4 are also maximal (only
    // assertoric subsumption removes things).
    assert!(mg.contains(&0));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn rule_set_conflicts_finds_deontic_pairs() {
    let rs = canonical_rule_set();
    let conf = rs.conflicts();
    // r3=Op(d), r4=Fp(d) → conflict.
    assert!(conf.contains(&(3, 4)));
    // r5=Op(e)|{b}, r6=Fp(e)|{b} → conflict (shared antecedent).
    assert!(conf.contains(&(5, 6)));
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn rule_set_normalize_dedupes() {
    let mut rs: RuleSet<&str> = RuleSet::from_rules(vec![
        Implication::assertoric(vec!["a"], vec!["x"]),
        Implication::assertoric(vec!["a"], vec!["x"]), // duplicate
    ]);
    rs.push(Implication::assertoric(vec!["a"], vec!["x"])); // also dup
    let n = rs.normalize();
    assert_eq!(n.len(), 1);
}

#[pr4xis::praxis_value(Verifiable, Deterministic, Extensible)]
#[test]
fn rule_set_canonical_basis_drops_strictly_subsumed() {
    let rs = canonical_rule_set();
    let basis = rs.canonical_basis();
    // The basis has at most as many rules as the original.
    assert!(basis.len() <= rs.len());
    // r2 ({b, c} ⇒ a) is strictly subsumed by r0 (⊤ ⇒ a) and r1 — out.
    let kept_text: Vec<_> = basis
        .rules()
        .iter()
        .map(|r| (r.antecedent().to_vec(), r.consequent().to_vec()))
        .collect();
    assert!(
        !kept_text.contains(&(vec!["b", "c"], vec!["a"])),
        "canonical basis must drop strictly-subsumed ({{b, c}} ⇒ a)"
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn empty_rule_set_operations_are_total() {
    let rs: RuleSet<&str> = RuleSet::new();
    assert!(rs.is_empty());
    assert_eq!(rs.subsumption_order(), vec![]);
    assert_eq!(rs.conflicts(), vec![]);
    assert_eq!(rs.most_general(), Vec::<usize>::new());
    assert_eq!(rs.canonical_basis().len(), 0);
}

// =============================================================================
// Layer 2 — registered axioms verify.
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_subsumption_reflexive_holds() {
    assert!(SubsumptionReflexive.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_subsumption_transitive_holds() {
    assert!(SubsumptionTransitive.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable, Deterministic)]
#[test]
fn axiom_normalization_idempotent_holds() {
    assert!(NormalizationIdempotent.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_conflict_symmetric_holds() {
    assert!(ConflictSymmetric.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_deontic_conflict_detected_holds() {
    assert!(DeonticConflictDetected.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable, Deterministic)]
#[test]
fn axiom_canonical_basis_is_subset_holds() {
    assert!(CanonicalBasisIsSubset.verify().is_ok());
}

// =============================================================================
// Layer 3 — property tests over arbitrary rules.
// =============================================================================

prop_compose! {
    /// Generate a small implication over a concept alphabet
    /// of 4 letters (a..d) so we get many shared concepts.
    fn arb_implication()(
        ant in prop::collection::vec(prop::sample::select(vec!["a", "b", "c", "d"]), 0..=4),
        con in prop::collection::vec(prop::sample::select(vec!["a", "b", "c", "d"]), 1..=3),
        d in prop::sample::select(vec![
            DeonticOperator::Assertoric,
            DeonticOperator::Obligation,
            DeonticOperator::Permission,
            DeonticOperator::Prohibition,
        ])
    ) -> Implication<&'static str> {
        Implication::new(ant, con, d)
    }
}

prop_compose! {
    fn arb_rule_set()(rules in prop::collection::vec(arb_implication(), 0..=6))
        -> RuleSet<&'static str>
    {
        RuleSet::from_rules(rules)
    }
}

proptest! {
    /// Reflexivity: every rule subsumes itself.
    #[test]
    fn property_subsumption_reflexive(r in arb_implication()) {
        prop_assert!(r.subsumes(&r));
    }

    /// Transitivity: R1 ≼ R2 ∧ R2 ≼ R3 → R1 ≼ R3.
    #[test]
    fn property_subsumption_transitive(
        r1 in arb_implication(),
        r2 in arb_implication(),
        r3 in arb_implication(),
    ) {
        if r1.subsumes(&r2) && r2.subsumes(&r3) {
            prop_assert!(r1.subsumes(&r3));
        }
    }

    /// Anti-symmetry up to equality: if R1 ≼ R2 and R2 ≼ R1, then
    /// R1 == R2.
    #[test]
    fn property_subsumption_antisymmetric_up_to_eq(
        r1 in arb_implication(),
        r2 in arb_implication(),
    ) {
        if r1.subsumes(&r2) && r2.subsumes(&r1) {
            prop_assert_eq!(r1, r2);
        }
    }

    /// Normalization is idempotent.
    #[test]
    fn property_normalization_idempotent(r in arb_implication()) {
        let once = r.clone().normalize();
        let twice = once.clone().normalize();
        prop_assert_eq!(once, twice);
    }

    /// Normalization preserves the rule's logical content — same
    /// `subsumes` relations to a witness.
    #[test]
    fn property_normalization_preserves_subsumption(
        r in arb_implication(),
        w in arb_implication(),
    ) {
        let n = r.clone().normalize();
        prop_assert_eq!(r.subsumes(&w), n.subsumes(&w));
        prop_assert_eq!(w.subsumes(&r), w.subsumes(&n));
    }

    /// Conflict is symmetric.
    #[test]
    fn property_conflict_symmetric(
        r1 in arb_implication(),
        r2 in arb_implication(),
    ) {
        prop_assert_eq!(r1.conflicts_with(&r2), r2.conflicts_with(&r1));
    }

    /// Conflict is irreflexive ON DEONTIC DIMENSION: a rule cannot
    /// conflict with itself (the deontic operator equals itself, and
    /// equal operators don't conflict in von Wright's square).
    #[test]
    fn property_conflict_irreflexive(r in arb_implication()) {
        prop_assert!(!r.conflicts_with(&r));
    }

    /// Subsumption order returned by RuleSet matches per-pair
    /// `subsumes`.
    #[test]
    fn property_rule_set_order_matches_pairwise(rs in arb_rule_set()) {
        let order = rs.subsumption_order();
        for i in 0..rs.len() {
            for j in 0..rs.len() {
                if i == j { continue; }
                let in_order = order.contains(&(i, j));
                let subsumes = rs.rules()[i].subsumes(&rs.rules()[j]);
                prop_assert_eq!(in_order, subsumes,
                    "({}, {}) in_order={}, subsumes={}", i, j, in_order, subsumes);
            }
        }
    }

    /// Conflict set returned by RuleSet matches per-pair
    /// `conflicts_with`, with `i < j` enforced.
    #[test]
    fn property_rule_set_conflicts_match_pairwise(rs in arb_rule_set()) {
        let conf = rs.conflicts();
        for &(i, j) in &conf {
            prop_assert!(i < j);
            prop_assert!(rs.rules()[i].conflicts_with(&rs.rules()[j]));
        }
        // Completeness: every (i < j) conflicting pair is in `conf`.
        for i in 0..rs.len() {
            for j in (i + 1)..rs.len() {
                if rs.rules()[i].conflicts_with(&rs.rules()[j]) {
                    prop_assert!(conf.contains(&(i, j)));
                }
            }
        }
    }

    /// Canonical basis is monotonically smaller-or-equal.
    #[test]
    fn property_canonical_basis_size(rs in arb_rule_set()) {
        let basis = rs.canonical_basis();
        prop_assert!(basis.len() <= rs.len());
    }

    /// RuleSet normalization is idempotent.
    #[test]
    fn property_rule_set_normalize_idempotent(rs in arb_rule_set()) {
        let once = rs.clone().normalize();
        let twice = once.clone().normalize();
        prop_assert_eq!(once, twice);
    }
}

pr4xis::register_praxis_value!(property_subsumption_reflexive, Verifiable);
pr4xis::register_praxis_value!(property_subsumption_transitive, Verifiable);
pr4xis::register_praxis_value!(property_subsumption_antisymmetric_up_to_eq, Verifiable);
pr4xis::register_praxis_value!(property_normalization_idempotent, Deterministic);
pr4xis::register_praxis_value!(
    property_normalization_preserves_subsumption,
    Deterministic,
    Verifiable
);
pr4xis::register_praxis_value!(property_conflict_symmetric, Verifiable);
pr4xis::register_praxis_value!(property_conflict_irreflexive, Verifiable);
pr4xis::register_praxis_value!(property_rule_set_order_matches_pairwise, Verifiable);
pr4xis::register_praxis_value!(property_rule_set_conflicts_match_pairwise, Verifiable);
pr4xis::register_praxis_value!(property_canonical_basis_size, Verifiable);
pr4xis::register_praxis_value!(property_rule_set_normalize_idempotent, Deterministic);
