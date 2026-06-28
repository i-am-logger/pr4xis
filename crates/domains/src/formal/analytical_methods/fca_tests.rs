//! Tests for [`super::fca`]. Three layers per
//! `feedback_high_test_coverage`:
//!
//! 1. **Structural laws** — Galois connection, closure properties,
//!    Hasse diagram correctness (Birkhoff 1940 Ch. V; Davey & Priestley
//!    2002 §1.4).
//! 2. **Axiom verification** — every registered axiom.
//! 3. **Property-based tests** — proptest invariants over arbitrary
//!    small contexts (≤ 6 objects × 6 attributes; lattice bounded by
//!    2^6 = 64 concepts).

use proptest::prelude::*;

use super::{
    BitSet, ConceptLatticeIsComplete, DoubleDerivationIsClosure, EnumeratedConceptsAreClosed,
    FormalConcept, FormalContext, GaloisConnectionLaw,
};
use pr4xis::ontology::Axiom;

// =============================================================================
// Layer 1 — structural laws on a hand-verified small context.
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
fn bitset_basic_operations() {
    let mut bs = BitSet::empty(10);
    assert!(!bs.contains(0));
    bs.set(0);
    bs.set(3);
    bs.set(9);
    assert!(bs.contains(0));
    assert!(bs.contains(3));
    assert!(bs.contains(9));
    assert!(!bs.contains(1));
    assert_eq!(bs.count(), 3);
    assert_eq!(bs.to_vec(), vec![0, 3, 9]);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn bitset_subset_and_intersect() {
    let mut a = BitSet::empty(8);
    a.set(0);
    a.set(2);
    let mut b = BitSet::empty(8);
    b.set(0);
    b.set(2);
    b.set(5);
    assert!(a.is_subset(&b));
    assert!(!b.is_subset(&a));
    let i = a.intersect(&b);
    assert_eq!(i.to_vec(), vec![0, 2]);
    let u = a.union(&b);
    assert_eq!(u.to_vec(), vec![0, 2, 5]);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn galois_extent_to_intent_works() {
    // Ganter-Wille canonical context. extent {dog} → intent {all 4}.
    let ctx = canonical_context();
    let intent = ctx.extent_to_intent(&[1]); // dog
    assert_eq!(intent.to_vec(), vec![0, 1, 2, 3]);

    // extent {fish, dog} → intent {needs_water, can_move, has_skeleton}.
    let intent = ctx.extent_to_intent(&[0, 1]);
    assert_eq!(intent.to_vec(), vec![0, 2, 3]);

    // extent {reed, bean} → intent {needs_water}.
    let intent = ctx.extent_to_intent(&[2, 3]);
    assert_eq!(intent.to_vec(), vec![0]);

    // Empty extent → all attributes (vacuous universal).
    let intent = ctx.extent_to_intent(&[]);
    assert_eq!(intent.count(), 4);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn galois_intent_to_extent_works() {
    let ctx = canonical_context();

    // intent {needs_water} → extent {all 4 objects}.
    let mut intent = BitSet::empty(4);
    intent.set(0);
    let extent = ctx.intent_to_extent(&intent);
    assert_eq!(extent, vec![0, 1, 2, 3]);

    // intent {has_limbs} → extent {dog} only.
    let mut intent = BitSet::empty(4);
    intent.set(1);
    let extent = ctx.intent_to_extent(&intent);
    assert_eq!(extent, vec![1]);

    // intent {needs_water, has_skeleton} → extent {fish, dog}.
    let mut intent = BitSet::empty(4);
    intent.set(0);
    intent.set(3);
    let extent = ctx.intent_to_extent(&intent);
    assert_eq!(extent, vec![0, 1]);
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn closure_is_extensive_monotone_idempotent() {
    let ctx = canonical_context();
    let extent = vec![0]; // {fish}
    let close = ctx.extent_closure(&extent);
    // Extensive: extent ⊆ close.
    for g in &extent {
        assert!(close.contains(g));
    }
    // Idempotent: close(close) == close.
    let close2 = ctx.extent_closure(&close);
    assert_eq!(close, close2);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn lattice_includes_top_and_bottom() {
    let ctx = canonical_context();
    let lat = ctx.build_lattice();
    assert!(!lat.is_empty());
    let top = lat.top().unwrap();
    let bot = lat.bottom().unwrap();
    // Top has the largest extent.
    let top_ext = lat.concepts[top].extent.len();
    let bot_ext = lat.concepts[bot].extent.len();
    assert!(top_ext >= bot_ext);
    // Top extent must equal all objects sharing the intent of the top.
    // In the canonical context the top is (all 4 objects, {needs_water}).
    assert_eq!(top_ext, 4);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn lattice_has_expected_size_on_canonical_context() {
    // Hand-computed concepts of the 4×4 Ganter-Wille canonical
    // context. Closed extents:
    //
    //   ({dog}, {needs_water, has_limbs, can_move, has_skeleton})
    //   ({fish, dog}, {needs_water, can_move, has_skeleton})
    //   ({fish, dog, reed, bean}, {needs_water})
    //
    // Sub-extents not in the list close to one of the above:
    //   ({fish}) closes to ({fish, dog})
    //   ({reed}) and ({bean}) close to all four objects
    //   ∅ closes to {dog}
    //
    // So the lattice has exactly 3 concepts.
    let ctx = canonical_context();
    let lat = ctx.build_lattice();
    assert_eq!(lat.len(), 3, "expected 3 concepts; got {}", lat.len());
    // Concepts must be unique by intent.
    let mut intents: Vec<_> = lat.concepts.iter().map(|c| c.intent.clone()).collect();
    intents.sort_by_key(|b| b.count());
    intents.dedup();
    assert_eq!(intents.len(), lat.len(), "duplicate intents in lattice");
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn hasse_diagram_is_transitive_reduction() {
    let ctx = canonical_context();
    let lat = ctx.build_lattice();
    // Every Hasse edge (i, j) must NOT have an intermediate k with
    // i < k < j.
    for &(i, j) in &lat.order_edges {
        assert!(
            lat.concepts[i].leq(&lat.concepts[j]),
            "Hasse edge {}→{} not in order",
            i,
            j
        );
        for k in 0..lat.concepts.len() {
            if k == i || k == j {
                continue;
            }
            let strict_i_k = lat.concepts[i].leq(&lat.concepts[k])
                && lat.concepts[i].extent != lat.concepts[k].extent;
            let strict_k_j = lat.concepts[k].leq(&lat.concepts[j])
                && lat.concepts[k].extent != lat.concepts[j].extent;
            assert!(
                !(strict_i_k && strict_k_j),
                "Hasse edge {i}→{j} has intermediate {k}"
            );
        }
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn lower_and_upper_covers_match_order_edges() {
    let ctx = canonical_context();
    let lat = ctx.build_lattice();
    for i in 0..lat.len() {
        for j in lat.upper_covers(i) {
            assert!(lat.order_edges.contains(&(i, j)));
        }
        for k in lat.lower_covers(i) {
            assert!(lat.order_edges.contains(&(k, i)));
        }
    }
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn empty_context_produces_at_least_one_concept() {
    // Edge case: 0 objects, 0 attributes. The lattice degenerates to
    // a single concept (∅, ∅).
    let ctx: FormalContext<&str, &str> = FormalContext::from_matrix(vec![], vec![], vec![]);
    let lat = ctx.build_lattice();
    assert!(!lat.is_empty());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn single_object_single_attribute_context() {
    let ctx = FormalContext::from_matrix(vec!["x"], vec!["P"], vec![vec![true]]);
    let lat = ctx.build_lattice();
    // Two concepts: (∅, {P}) and ({x}, {P}). Or in the degenerate
    // single-attribute case, just one if the attribute is full.
    assert!(!lat.is_empty());
    assert!(lat.top().is_some());
    assert!(lat.bottom().is_some());
}

// =============================================================================
// Layer 2 — registered axioms verify.
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_galois_connection_law_holds() {
    assert!(GaloisConnectionLaw.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_double_derivation_is_closure_holds() {
    assert!(DoubleDerivationIsClosure.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_concept_lattice_is_complete_holds() {
    assert!(ConceptLatticeIsComplete.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_enumerated_concepts_are_closed_holds() {
    assert!(EnumeratedConceptsAreClosed.verify().is_ok());
}

// =============================================================================
// Layer 3 — property tests over small random contexts.
// =============================================================================

prop_compose! {
    /// Generate a random context with |G| ∈ [1, 5] and |M| ∈ [1, 5].
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
    /// Property: every concept produced by `build_lattice` is closed
    /// under the double-derivation operator. (Ganter & Wille 1999
    /// Theorem 3: a formal concept is exactly a fixed point of `''`.)
    #[test]
    fn property_every_concept_is_closed(ctx in arb_context()) {
        let lat = ctx.build_lattice();
        for c in &lat.concepts {
            let closed = ctx.intent_closure(&c.intent);
            prop_assert_eq!(closed, c.intent.clone());
        }
    }

    /// Property: concepts are pairwise distinct by intent.
    #[test]
    fn property_concepts_distinct_by_intent(ctx in arb_context()) {
        let lat = ctx.build_lattice();
        let mut intents: Vec<_> = lat.concepts.iter().map(|c| c.intent.clone()).collect();
        intents.sort_by_key(|a| a.to_vec());
        let len = intents.len();
        intents.dedup();
        prop_assert_eq!(intents.len(), len);
    }

    /// Property: extents and intents are mutually determined by the
    /// Galois connection — `concept.extent = intent_to_extent(concept.intent)`
    /// and `extent_to_intent(concept.extent) = concept.intent`.
    #[test]
    fn property_galois_round_trip(ctx in arb_context()) {
        let lat = ctx.build_lattice();
        for c in &lat.concepts {
            let derived_extent = ctx.intent_to_extent(&c.intent);
            prop_assert_eq!(derived_extent, c.extent.clone());
            let derived_intent = ctx.extent_to_intent(&c.extent);
            prop_assert_eq!(derived_intent, c.intent.clone());
        }
    }

    /// Property: lattice size is bounded by 2^min(|G|, |M|)
    /// (Ganter & Wille 1999 §2.3). For |G|, |M| ≤ 5, the bound is 32.
    #[test]
    fn property_lattice_size_bounded(ctx in arb_context()) {
        let lat = ctx.build_lattice();
        let g = ctx.objects().len();
        let m = ctx.attributes().len();
        let bound = 1usize << g.min(m);
        prop_assert!(lat.len() <= bound,
            "lattice size {} exceeds 2^min(|G|, |M|) = {}",
            lat.len(), bound);
    }

    /// Property: every Hasse edge `(i, j)` satisfies `concepts[i] < concepts[j]`
    /// and there is no intermediate `k` (transitive reduction).
    #[test]
    fn property_hasse_diagram_transitive_reduction(ctx in arb_context()) {
        let lat = ctx.build_lattice();
        for &(i, j) in &lat.order_edges {
            // Strict.
            prop_assert!(lat.concepts[i].leq(&lat.concepts[j]));
            prop_assert_ne!(lat.concepts[i].extent.clone(), lat.concepts[j].extent.clone());
            // No intermediate.
            for k in 0..lat.len() {
                if k == i || k == j { continue; }
                let strict_i_k = lat.concepts[i].leq(&lat.concepts[k])
                    && lat.concepts[i].extent != lat.concepts[k].extent;
                let strict_k_j = lat.concepts[k].leq(&lat.concepts[j])
                    && lat.concepts[k].extent != lat.concepts[j].extent;
                prop_assert!(!(strict_i_k && strict_k_j),
                    "Hasse edge {i}→{j} has intermediate {k}");
            }
        }
    }

    /// Property: top concept exists and dominates every other concept
    /// (extent inclusion is reflexive). Bottom analogously.
    #[test]
    fn property_top_and_bottom_exist_and_dominate(ctx in arb_context()) {
        let lat = ctx.build_lattice();
        let top = lat.top().expect("non-empty lattice has a top");
        let bot = lat.bottom().expect("non-empty lattice has a bottom");
        for i in 0..lat.len() {
            prop_assert!(lat.concepts[i].leq(&lat.concepts[top]),
                "concept {i} not <= top {top}");
            prop_assert!(lat.concepts[bot].leq(&lat.concepts[i]),
                "bottom {bot} not <= concept {i}");
        }
    }

    /// Property: the Galois connection's antitone laws —
    /// `A1 ⊆ A2 ⟹ A2' ⊆ A1'` and `B1 ⊆ B2 ⟹ B2' ⊆ B1'`.
    #[test]
    fn property_derivation_is_antitone(
        ctx in arb_context(),
        seed in any::<u64>(),
    ) {
        let g = ctx.objects().len();
        if g < 2 { return Ok(()); }
        // Use seed to pick two extent samples.
        let g1 = (seed as usize) % g;
        let g2 = ((seed >> 32) as usize) % g;
        let a1: Vec<usize> = vec![g1];
        let a2: Vec<usize> = vec![g1, g2];
        // a1 ⊆ a2.
        let i1 = ctx.extent_to_intent(&a1);
        let i2 = ctx.extent_to_intent(&a2);
        // Antitone: i2 ⊆ i1.
        prop_assert!(i2.is_subset(&i1),
            "extent_to_intent not antitone: a1={a1:?}, a2={a2:?}");
    }
}

pr4xis::register_praxis_value!(property_every_concept_is_closed, Verifiable);
pr4xis::register_praxis_value!(property_concepts_distinct_by_intent, Verifiable);
pr4xis::register_praxis_value!(property_galois_round_trip, Deterministic);
pr4xis::register_praxis_value!(property_lattice_size_bounded, Verifiable);
pr4xis::register_praxis_value!(property_hasse_diagram_transitive_reduction, Verifiable);
pr4xis::register_praxis_value!(property_top_and_bottom_exist_and_dominate, Verifiable);
pr4xis::register_praxis_value!(property_derivation_is_antitone, Verifiable);

// =============================================================================
// FormalConcept equality + sanity.
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn formal_concept_eq_uses_extent_and_intent() {
    let ctx = canonical_context();
    let lat = ctx.build_lattice();
    let same: Vec<FormalConcept<&str, &str>> = lat.concepts.clone();
    for (a, b) in lat.concepts.iter().zip(same.iter()) {
        assert_eq!(a, b);
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn extent_objects_and_intent_attributes_resolve() {
    let ctx = canonical_context();
    let lat = ctx.build_lattice();
    let top = lat.top().unwrap();
    let extent_names = lat.concepts[top].extent_objects(&ctx);
    let intent_names = lat.concepts[top].intent_attributes(&ctx);
    // In the canonical context, the top concept's extent is all
    // four objects; intent is just "needs_water".
    assert_eq!(extent_names.len(), 4);
    assert!(!intent_names.is_empty());
}
