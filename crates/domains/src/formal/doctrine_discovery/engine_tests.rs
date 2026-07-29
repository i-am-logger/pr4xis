//! Tests for [`super::engine`]. Three layers per
//! `feedback_high_test_coverage`: pinpoint hand-verified cases,
//! axiom verification, proptest property invariants.

use super::*;
use crate::formal::analytical_methods::FormalContext;
use crate::formal::analytical_methods::fca::BitSet;
use pr4xis::ontology::Axiom;
use proptest::prelude::*;

// =============================================================================
// Layer 1 — pinpoint cases on the Ganter-Wille canonical context.
// =============================================================================

fn canonical_context() -> FormalContext<&'static str, &'static str> {
    FormalContext::from_matrix(
        vec!["fish", "dog", "reed", "bean"],
        vec!["needs_water", "has_limbs", "can_move", "has_skeleton"],
        vec![
            vec![true, false, true, true],
            vec![true, true, true, true],
            vec![true, false, false, false],
            vec![true, false, false, false],
        ],
    )
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn discover_returns_lattice_clusters() {
    // Per the FCA tests, the canonical context has 3 concepts.
    let disc = discover(&canonical_context());
    assert_eq!(disc.cluster_count().value as usize, 3);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn implication_count_is_positive_for_nontrivial_context() {
    // has_limbs in this context implies all of {needs_water,
    // can_move, has_skeleton} (only dog has limbs, dog has all four
    // attributes). So we expect at least one non-trivial implication.
    let disc = discover(&canonical_context());
    assert!(
        disc.implication_count() >= 1,
        "expected at least one non-trivial closure, got {}",
        disc.implication_count()
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn has_limbs_closure_implies_full_attribute_set() {
    // Hand-verifiable: among the four objects, only `dog` has
    // `has_limbs`. Dog also has `needs_water`, `can_move`,
    // `has_skeleton`. So `{has_limbs}'' = {has_limbs, needs_water,
    // can_move, has_skeleton}` — the implication
    // `{has_limbs} → all-four` is in the basis.
    let disc = discover(&canonical_context());
    let has_limbs_imp = disc
        .basis
        .rules()
        .iter()
        .find(|r| r.antecedent() == ["has_limbs"]);
    let imp = has_limbs_imp.expect("expected an implication with antecedent {has_limbs}");
    let consequent = imp.consequent();
    for attr in ["needs_water", "has_limbs", "can_move", "has_skeleton"] {
        assert!(
            consequent.contains(&attr),
            "expected `{attr}` in has_limbs closure; got {consequent:?}"
        );
    }
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn discover_is_deterministic() {
    let ctx = canonical_context();
    let a = discover(&ctx);
    let b = discover(&ctx);
    assert_eq!(a.cluster_count(), b.cluster_count());
    assert_eq!(a.implication_count(), b.implication_count());
    assert_eq!(a.subsumption_order, b.subsumption_order);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn empty_context_yields_minimal_discovery() {
    let ctx: FormalContext<&str, &str> = FormalContext::from_matrix(vec![], vec![], vec![]);
    let disc = discover(&ctx);
    // FCA emits at least the (∅, ∅) concept.
    assert!(disc.cluster_count().value >= 1.0);
    // No attributes → no implications.
    assert_eq!(disc.implication_count(), 0);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn attribute_extractor_closure_works() {
    // Use a closure as an AttributeExtractor — confirms the blanket
    // `impl<F: Fn(...) -> Vec<A>>` works.
    let objects = vec!["fish", "dog"];
    let extractor = |o: &&str| -> alloc::vec::Vec<&'static str> {
        match *o {
            "fish" => vec!["needs_water", "can_move"],
            "dog" => vec!["needs_water", "has_limbs", "can_move"],
            _ => vec![],
        }
    };
    // Build the context via the extractor.
    let attrs: alloc::vec::Vec<&'static str> = vec!["needs_water", "has_limbs", "can_move"];
    let ctx = FormalContext::from_predicate(objects, attrs, |o, a| {
        extractor.attributes_of(o).contains(a)
    });
    let disc = discover(&ctx);
    assert!(disc.cluster_count().value >= 1.0);
}

// =============================================================================
// Layer 2 — registered axioms verify.
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_discovered_clusters_match_lattice_holds() {
    assert!(DiscoveredClustersMatchLattice.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_every_implication_is_context_valid_holds() {
    assert!(EveryImplicationIsContextValid.verify().is_ok());
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn axiom_canonical_basis_is_subsumption_minimal_holds() {
    assert!(CanonicalBasisIsSubsumptionMinimal.verify().is_ok());
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn axiom_discovery_is_deterministic_holds() {
    assert!(DiscoveryIsDeterministic.verify().is_ok());
}

// =============================================================================
// Layer 3 — proptest properties over arbitrary small contexts.
// =============================================================================

prop_compose! {
    fn arb_context()
        (g_count in 1usize..=4, m_count in 1usize..=4)
        (rows in prop::collection::vec(
            prop::collection::vec(any::<bool>(), m_count..=m_count),
            g_count..=g_count,
        ), _g in Just(g_count), _m in Just(m_count))
        -> FormalContext<usize, usize>
    {
        let objects: alloc::vec::Vec<usize> = (0..rows.len()).collect();
        let attributes: alloc::vec::Vec<usize> = (0..rows[0].len()).collect();
        FormalContext::from_matrix(objects, attributes, rows)
    }
}

proptest! {
    /// Cluster count equals the concept-lattice size for the same
    /// context.
    #[test]
    fn property_cluster_count_equals_lattice_size(ctx in arb_context()) {
        let disc = discover(&ctx);
        let lat = ctx.build_lattice();
        prop_assert_eq!(disc.cluster_count().value as usize, lat.len());
    }

    /// Every emitted implication is *valid* in the context: its
    /// consequent is contained in its antecedent's closure.
    #[test]
    fn property_every_implication_is_valid(ctx in arb_context()) {
        let disc = discover(&ctx);
        let m_count = ctx.attributes().len();
        for imp in disc.basis.rules() {
            let mut x = BitSet::empty(m_count);
            for a in imp.antecedent() {
                let i = ctx.attributes().iter().position(|attr| attr == a)
                    .expect("antecedent attribute present");
                x.set(i);
            }
            let close = ctx.intent_closure(&x);
            for a in imp.consequent() {
                let i = ctx.attributes().iter().position(|attr| attr == a)
                    .expect("consequent attribute present");
                prop_assert!(close.contains(i),
                    "implication {imp:?} not valid in context");
            }
        }
    }

    /// Discovery is deterministic — equal contexts yield equal output.
    #[test]
    fn property_discovery_deterministic(ctx in arb_context()) {
        let a = discover(&ctx);
        let b = discover(&ctx);
        prop_assert_eq!(a.cluster_count(), b.cluster_count());
        prop_assert_eq!(a.implication_count(), b.implication_count());
        prop_assert_eq!(&a.subsumption_order, &b.subsumption_order);
    }

    /// The canonical basis size is bounded by the number of
    /// attributes — every emitted implication has a singleton
    /// antecedent from the |M| candidates.
    #[test]
    fn property_basis_size_bounded_by_attribute_count(ctx in arb_context()) {
        let disc = discover(&ctx);
        let m_count = ctx.attributes().len();
        prop_assert!(disc.implication_count() <= m_count,
            "|basis|={} exceeds |M|={}", disc.implication_count(), m_count);
    }

    /// Subsumption order is irreflexive — no (i, i) pair, since
    /// `RuleSet::subsumption_order` excludes reflexive pairs.
    #[test]
    fn property_subsumption_order_irreflexive(ctx in arb_context()) {
        let disc = discover(&ctx);
        for &(i, j) in &disc.subsumption_order {
            prop_assert_ne!(i, j);
        }
    }
}

pr4xis::register_praxis_value!(property_cluster_count_equals_lattice_size, Verifiable);
pr4xis::register_praxis_value!(property_every_implication_is_valid, Verifiable);
pr4xis::register_praxis_value!(property_discovery_deterministic, Deterministic);
pr4xis::register_praxis_value!(property_basis_size_bounded_by_attribute_count, Verifiable);
pr4xis::register_praxis_value!(property_subsumption_order_irreflexive, Verifiable);
