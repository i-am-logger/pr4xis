//! Tests for [`super::classification_fibration`]. Three layers per
//! `feedback_high_test_coverage`.

use proptest::prelude::*;

use super::super::fca::FormalContext;
use super::{
    ConceptLatticeFibration, FibrationIsLinnaeanMonotone, PopulatedRanksAreContiguous,
    ProjectionIsTotal,
};
use crate::formal::classification::ontology::ClassificationConcept;
use pr4xis::ontology::Axiom;

// =============================================================================
// Layer 1 — structural sanity on hand-verifiable contexts.
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
fn projection_assigns_rank_to_every_concept() {
    let ctx = canonical_context();
    let fib = ConceptLatticeFibration::from_context(&ctx);
    assert_eq!(fib.ranks.len(), fib.lattice.concepts.len());
    for i in 0..fib.lattice.concepts.len() {
        assert!(fib.rank_of(i).is_some());
    }
}

#[pr4xis::praxis_value(Verifiable, Extensible)]
#[test]
fn bottom_concept_gets_species_rank() {
    let ctx = canonical_context();
    let fib = ConceptLatticeFibration::from_context(&ctx);
    let bot = fib.lattice.bottom().unwrap();
    let rank = fib.rank_of(bot).unwrap();
    // The lowest extent maps to the lowest Linnaean rank.
    assert_eq!(rank, ClassificationConcept::Species);
}

#[pr4xis::praxis_value(Verifiable, Extensible)]
#[test]
fn top_concept_gets_highest_rank_present() {
    let ctx = canonical_context();
    let fib = ConceptLatticeFibration::from_context(&ctx);
    let top = fib.lattice.top().unwrap();
    let rank = fib.rank_of(top).unwrap();
    let populated = fib.populated_ranks();
    // The top concept's rank is the maximum among populated ranks.
    let top_order = linnaean_order_for_test(rank);
    let max_populated = populated
        .iter()
        .map(|&c| linnaean_order_for_test(c))
        .max()
        .unwrap();
    assert_eq!(top_order, max_populated);
}

fn linnaean_order_for_test(c: ClassificationConcept) -> u8 {
    use ClassificationConcept as C;
    match c {
        C::Species => 1,
        C::Genus => 2,
        C::Family => 3,
        C::Order => 4,
        C::Class => 5,
        C::Phylum => 6,
        C::Kingdom => 7,
        _ => 0,
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn fiber_returns_concepts_in_that_rank() {
    let ctx = canonical_context();
    let fib = ConceptLatticeFibration::from_context(&ctx);
    let species_fiber = fib.fiber(ClassificationConcept::Species);
    for i in species_fiber {
        assert_eq!(fib.rank_of(i), Some(ClassificationConcept::Species));
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn populated_ranks_are_sorted_linnaean() {
    let ctx = canonical_context();
    let fib = ConceptLatticeFibration::from_context(&ctx);
    let ranks = fib.populated_ranks();
    let orders: Vec<u8> = ranks.iter().map(|&c| linnaean_order_for_test(c)).collect();
    let mut sorted = orders.clone();
    sorted.sort_unstable();
    assert_eq!(orders, sorted);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn populated_ranks_are_contiguous_on_canonical() {
    let ctx = canonical_context();
    let fib = ConceptLatticeFibration::from_context(&ctx);
    let orders: Vec<u8> = fib
        .populated_ranks()
        .into_iter()
        .map(linnaean_order_for_test)
        .collect();
    // Contiguity: consecutive differences in [0, 1].
    for w in orders.windows(2) {
        assert!(w[1] - w[0] <= 1, "non-contiguous: {orders:?}");
    }
}

#[pr4xis::praxis_value(Verifiable, Extensible)]
#[test]
fn trivial_lattice_maps_to_kingdom() {
    // The "all bits true" 1×1 context degenerates to a single concept
    // (extent {0}, intent {0}). Depth 0, max_depth 0 — maps to Kingdom.
    let ctx = FormalContext::from_matrix(vec!["only"], vec!["everything"], vec![vec![true]]);
    let fib = ConceptLatticeFibration::from_context(&ctx);
    let any_rank = fib.rank_of(0).unwrap();
    assert_eq!(any_rank, ClassificationConcept::Kingdom);
}

// =============================================================================
// Layer 2 — registered axioms.
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_projection_is_total_holds() {
    assert!(ProjectionIsTotal.verify().is_ok());
}

#[pr4xis::praxis_value(Extensible)]
#[test]
fn axiom_fibration_is_linnaean_monotone_holds() {
    assert!(FibrationIsLinnaeanMonotone.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_populated_ranks_are_contiguous_holds() {
    assert!(PopulatedRanksAreContiguous.verify().is_ok());
}

// =============================================================================
// Layer 3 — property tests over arbitrary small contexts.
// =============================================================================

prop_compose! {
    fn arb_context()
        (g_count in 1usize..=5, m_count in 1usize..=5)
        (rows in prop::collection::vec(
            prop::collection::vec(any::<bool>(), m_count..=m_count),
            g_count..=g_count,
        ), g in Just(g_count), m in Just(m_count))
        -> FormalContext<usize, usize>
    {
        let _ = (g, m);
        let objects: Vec<usize> = (0..rows.len()).collect();
        let attributes: Vec<usize> = (0..rows[0].len()).collect();
        FormalContext::from_matrix(objects, attributes, rows)
    }
}

proptest! {
    /// Property: every fibration projection is total.
    #[test]
    fn property_projection_total(ctx in arb_context()) {
        let fib = ConceptLatticeFibration::from_context(&ctx);
        for i in 0..fib.lattice.concepts.len() {
            prop_assert!(fib.rank_of(i).is_some());
        }
    }

    /// Property: every Hasse edge respects Linnaean monotonicity —
    /// sub-concept's rank ≤ super-concept's rank.
    #[test]
    fn property_linnaean_monotonicity(ctx in arb_context()) {
        let fib = ConceptLatticeFibration::from_context(&ctx);
        for &(i, j) in &fib.lattice.order_edges {
            let ri = fib.rank_of(i).unwrap();
            let rj = fib.rank_of(j).unwrap();
            prop_assert!(linnaean_order_for_test(ri) <= linnaean_order_for_test(rj),
                "{i} (rank {ri:?}) → {j} (rank {rj:?}) violates monotonicity");
        }
    }

    /// Property: the populated-ranks set is always sorted ascending
    /// in Linnaean order.
    #[test]
    fn property_populated_ranks_sorted(ctx in arb_context()) {
        let fib = ConceptLatticeFibration::from_context(&ctx);
        let orders: Vec<u8> = fib
            .populated_ranks()
            .into_iter()
            .map(linnaean_order_for_test)
            .collect();
        let mut sorted = orders.clone();
        sorted.sort_unstable();
        prop_assert_eq!(orders, sorted);
    }

    /// Property: fibers partition the concept set — every concept is
    /// in exactly one fiber.
    #[test]
    fn property_fibers_partition(ctx in arb_context()) {
        let fib = ConceptLatticeFibration::from_context(&ctx);
        let total: usize = fib.populated_ranks()
            .iter()
            .map(|&r| fib.fiber(r).len())
            .sum();
        prop_assert_eq!(total, fib.lattice.concepts.len());
    }

    /// Property: the rank assignment is *deterministic* — running
    /// `from_context` twice on the same input produces the same ranks.
    #[test]
    fn property_assignment_deterministic(ctx in arb_context()) {
        let a = ConceptLatticeFibration::from_context(&ctx);
        let b = ConceptLatticeFibration::from_context(&ctx);
        prop_assert_eq!(a.ranks, b.ranks);
    }
}

pr4xis::register_praxis_value!(property_projection_total, Verifiable);
pr4xis::register_praxis_value!(property_linnaean_monotonicity, Extensible);
pr4xis::register_praxis_value!(property_populated_ranks_sorted, Verifiable);
pr4xis::register_praxis_value!(property_fibers_partition, Verifiable);
pr4xis::register_praxis_value!(property_assignment_deterministic, Deterministic);
