//! Tests for [`super::synthesizer`]. Three layers per
//! `feedback_high_test_coverage`.

use super::*;
use crate::formal::analytical_methods::FormalContext;
use crate::formal::doctrine_discovery::discover;
use pr4xis::ontology::Axiom;
use proptest::prelude::*;

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

// =============================================================================
// Layer 1 — pinpoint cases.
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn synthesize_returns_total_object_map() {
    let disc = discover(&canonical_context());
    let synth = synthesize(&disc);
    assert_eq!(synth.object_count(), 4);
    for g in 0..4 {
        assert!(synth.cluster_of(g).is_some());
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn every_assignment_is_in_cluster_range() {
    let disc = discover(&canonical_context());
    let synth = synthesize(&disc);
    let n = synth.cluster_count();
    for &ci in synth.object_map() {
        assert!(ci < n, "cluster {ci} out of range {n}");
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn synthesizer_assigns_tightest_concept() {
    // Hand-verifiable: dog has all four attributes, so its tightest
    // concept is the ({dog}, all four) singleton concept.
    let disc = discover(&canonical_context());
    let synth = synthesize(&disc);
    let dog_cluster = synth.cluster_of(1).unwrap();
    let assigned = &disc.fibration.lattice.concepts[dog_cluster];
    assert!(assigned.extent.contains(&1));
    // No strictly-smaller-extent concept also contains dog.
    for (idx, c) in disc.fibration.lattice.concepts.iter().enumerate() {
        if idx == dog_cluster {
            continue;
        }
        if c.extent.contains(&1) {
            assert!(c.extent.len() >= assigned.extent.len());
        }
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn map_identity_matches_cluster_of() {
    let disc = discover(&canonical_context());
    let synth = synthesize(&disc);
    for g in 0..synth.object_count() {
        assert_eq!(synth.map_identity(g), synth.cluster_of(g));
    }
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn synthesize_is_deterministic() {
    let disc = discover(&canonical_context());
    let a = synthesize(&disc);
    let b = synthesize(&disc);
    assert_eq!(a.object_map(), b.object_map());
    assert_eq!(a.cluster_count(), b.cluster_count());
}

#[pr4xis::praxis_value(Extensible)]
#[test]
fn laws_verified_on_canonical_context() {
    let disc = discover(&canonical_context());
    let synth = synthesize(&disc);
    assert!(synth.laws_verified());
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn empty_context_yields_empty_synthesis() {
    let ctx: FormalContext<&str, &str> = FormalContext::from_matrix(vec![], vec![], vec![]);
    let disc = discover(&ctx);
    let synth = synthesize(&disc);
    assert_eq!(synth.object_count(), 0);
}

// =============================================================================
// Layer 2 — registered axioms.
// =============================================================================

#[pr4xis::praxis_value(Extensible)]
#[test]
fn axiom_synthesized_functor_preserves_identity_holds() {
    assert!(SynthesizedFunctorPreservesIdentity.verify().is_ok());
}

#[pr4xis::praxis_value(Extensible)]
#[test]
fn axiom_synthesized_functor_preserves_composition_holds() {
    assert!(SynthesizedFunctorPreservesComposition.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_cluster_assignment_is_tightest_fit_holds() {
    assert!(ClusterAssignmentIsTightestFit.verify().is_ok());
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn axiom_synthesizer_is_deterministic_holds() {
    assert!(SynthesizerIsDeterministic.verify().is_ok());
}

// =============================================================================
// Layer 3 — property tests over arbitrary small contexts.
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
    /// Every object gets a cluster assignment.
    #[test]
    fn property_object_map_total(ctx in arb_context()) {
        let disc = discover(&ctx);
        let synth = synthesize(&disc);
        prop_assert_eq!(synth.object_count(), ctx.objects().len());
        for g in 0..synth.object_count() {
            prop_assert!(synth.cluster_of(g).is_some());
        }
    }

    /// Every assignment is within cluster range.
    #[test]
    fn property_assignments_within_range(ctx in arb_context()) {
        let disc = discover(&ctx);
        let synth = synthesize(&disc);
        let n = synth.cluster_count();
        for &ci in synth.object_map() {
            prop_assert!(ci < n);
        }
    }

    /// Object g is in the extent of its assigned cluster.
    #[test]
    fn property_object_in_assigned_extent(ctx in arb_context()) {
        let disc = discover(&ctx);
        let synth = synthesize(&disc);
        for (g, &ci) in synth.object_map().iter().enumerate() {
            prop_assert!(disc.fibration.lattice.concepts[ci].extent.contains(&g));
        }
    }

    /// Tightest-fit: no smaller-extent concept also contains the
    /// object.
    #[test]
    fn property_assignment_is_tightest_fit(ctx in arb_context()) {
        let disc = discover(&ctx);
        let synth = synthesize(&disc);
        let lat = &disc.fibration.lattice;
        for (g, &ci) in synth.object_map().iter().enumerate() {
            let assigned = &lat.concepts[ci];
            for (idx, c) in lat.concepts.iter().enumerate() {
                if idx == ci {
                    continue;
                }
                if c.extent.contains(&g) && c.extent.len() < assigned.extent.len() {
                    prop_assert!(false,
                        "object {g} has tighter concept at index {idx}");
                }
            }
        }
    }

    /// Determinism: synthesize twice → same output.
    #[test]
    fn property_synthesize_deterministic(ctx in arb_context()) {
        let disc = discover(&ctx);
        let a = synthesize(&disc);
        let b = synthesize(&disc);
        prop_assert_eq!(a.object_map(), b.object_map());
    }
}

pr4xis::register_praxis_value!(property_object_map_total, Verifiable);
pr4xis::register_praxis_value!(property_assignments_within_range, Verifiable);
pr4xis::register_praxis_value!(property_object_in_assigned_extent, Verifiable);
pr4xis::register_praxis_value!(property_assignment_is_tightest_fit, Verifiable);
pr4xis::register_praxis_value!(property_synthesize_deterministic, Deterministic);
