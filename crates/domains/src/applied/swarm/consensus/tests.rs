//! Property-based tests for the Consensus ontology, engine, and functors.

#![cfg(test)]

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use proptest::prelude::*;

use super::engine::{
    FIXTURE_PEER_COUNT, NUMERICAL_SLACK, PUSH_SUM_ROUNDS, average, average_consensus_step,
    path_graph_p3, push_sum_initial, push_sum_round, ratio_invariant, stable_step_size,
};
use super::ontology::{
    ConsensusCategory, ConsensusConcept, ConsensusOntology, RequiresConnectivity, TrustState,
};
use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::ontology::{Ontology, Quality};

fn arb_concept() -> impl Strategy<Value = ConsensusConcept> {
    proptest::sample::select(ConsensusConcept::variants())
}

/// Arbitrary bounded peer values for the three-peer fixtures — the
/// bound keeps the floating-point conservation slack meaningful.
fn arb_values() -> impl Strategy<Value = Vec<f64>> {
    proptest::collection::vec(-100.0..100.0f64, FIXTURE_PEER_COUNT)
}

proptest! {
    /// RequiresConnectivity is defined exactly on the two concrete
    /// protocols (Fiedler 1973; Xiao & Boyd 2004) and nowhere else.
    #[test]
    fn prop_requires_connectivity_exactly_on_protocols(c in arb_concept()) {
        use ConsensusConcept as C;
        let is_protocol = matches!(c, C::AverageConsensus | C::GossipAveraging);
        prop_assert_eq!(RequiresConnectivity.get(&c).is_some(), is_protocol);
    }

    /// TrustState is defined exactly on the two trust standings
    /// (Lamport, Shostak & Pease 1982) and nowhere else.
    #[test]
    fn prop_trust_state_exactly_on_trust_concepts(c in arb_concept()) {
        use ConsensusConcept as C;
        let is_standing = matches!(c, C::TrustedNeighbor | C::DistrustedPeer);
        prop_assert_eq!(TrustState.get(&c).is_some(), is_standing);
    }

    /// The average-consensus step preserves the average for every
    /// initial condition — OSFM (2007) sec II: the sum is invariant
    /// because each pairwise pull cancels over the symmetric graph.
    #[test]
    fn prop_consensus_step_preserves_average(values in arb_values()) {
        let topology = path_graph_p3();
        let step = stable_step_size(&topology).value;
        let next = average_consensus_step(&values, &topology, step);
        prop_assert!((average(&next).value - average(&values).value).abs() <= NUMERICAL_SLACK);
    }

    /// Every iterate stays inside the convex hull of the previous values
    /// for every initial condition — OSFM (2007) sec II: under the
    /// stable step each new value is a convex combination.
    #[test]
    fn prop_consensus_step_stays_in_hull(values in arb_values()) {
        let topology = path_graph_p3();
        let step = stable_step_size(&topology).value;
        let lo = values.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let next = average_consensus_step(&values, &topology, step);
        for v in next {
            prop_assert!(v >= lo - NUMERICAL_SLACK && v <= hi + NUMERICAL_SLACK);
        }
    }

    /// Push-sum conserves the ratio invariant for every initial
    /// condition and along the whole deterministic schedule — Kempe,
    /// Dobra & Gehrke (2003), mass conservation.
    #[test]
    fn prop_push_sum_conserves_mass(values in arb_values()) {
        let topology = path_graph_p3();
        let mut state = push_sum_initial(&values);
        let expected = ratio_invariant(&state).value;
        for round in 0..PUSH_SUM_ROUNDS {
            state = push_sum_round(&state, &topology, round);
            prop_assert!((ratio_invariant(&state).value - expected).abs() <= NUMERICAL_SLACK);
        }
    }

    /// Every arrow of the category carries a non-empty name.
    #[test]
    fn prop_every_arrow_is_named(_seed in any::<u32>()) {
        for m in ConsensusCategory::morphisms() {
            prop_assert!(!m.meta().name.as_str().is_empty());
        }
    }

    /// Structural + domain axioms all discharge, regardless of the
    /// sampling that drives the test.
    #[test]
    fn prop_all_axioms_hold(_seed in 0..16u32) {
        for axiom in ConsensusOntology::axioms() {
            if let Err(c) = axiom.verify() {
                prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
            }
        }
    }
}

pr4xis::register_praxis_value!(prop_requires_connectivity_exactly_on_protocols, Verifiable);
pr4xis::register_praxis_value!(prop_trust_state_exactly_on_trust_concepts, Verifiable);
pr4xis::register_praxis_value!(prop_consensus_step_preserves_average, Verifiable);
pr4xis::register_praxis_value!(prop_consensus_step_stays_in_hull, Verifiable);
pr4xis::register_praxis_value!(prop_push_sum_conserves_mass, Verifiable);
pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
pr4xis::register_praxis_value!(prop_all_axioms_hold, Verifiable);

/// An equivocator among honest peers is the only peer flagged — the
/// detection is targeted, not collective (LSP 1982: the proof names the
/// inconsistent reporter).
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn only_the_equivocator_is_flagged() {
    use super::engine::{
        FIXTURE_INITIAL_VALUES, PeerId, equivocation_round_p3, equivocators, honest_round,
    };
    let topology = path_graph_p3();
    let dishonest = equivocation_round_p3(&FIXTURE_INITIAL_VALUES, &topology);
    assert_eq!(equivocators(&dishonest), vec![PeerId(1)]);
    let honest = honest_round(&FIXTURE_INITIAL_VALUES, &topology);
    assert!(equivocators(&honest).is_empty());
}
