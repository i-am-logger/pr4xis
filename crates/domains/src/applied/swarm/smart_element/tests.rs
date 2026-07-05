//! Property-based tests for the SmartElement ontology, engine, and functors.

#![cfg(test)]

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use proptest::prelude::*;

use super::engine::{
    EQUIVOCATOR_PEER, HONEST_PEER, MAPE_PHASE_ORDER, MapePhase, SmartElementAction,
    aggregate_trusted, apply, information_eq, run_mape_cycle, smart_element_fixture, trusts,
};
use super::ontology::{
    HasClosedLoop, HasQueryableOntology, IsFusionPeer, MapeKPhaseFocus, SelfStarKind,
    SmartElementCategory, SmartElementConcept, SmartElementOntology,
};
use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::ontology::{Ontology, Quality};

fn arb_concept() -> impl Strategy<Value = SmartElementConcept> {
    proptest::sample::select(SmartElementConcept::variants())
}

/// The three Smart* concepts the smartness predicates are defined on.
fn is_smart(c: SmartElementConcept) -> bool {
    use SmartElementConcept as C;
    matches!(c, C::SmartElement | C::SmartSensor | C::SmartDriver)
}

/// The four self-* concepts `SelfStarKind` / `MapeKPhaseFocus` are defined on.
fn is_self_star(c: SmartElementConcept) -> bool {
    use SmartElementConcept as C;
    matches!(
        c,
        C::SelfConfiguration | C::SelfHealing | C::SelfOptimization | C::SelfProtection
    )
}

proptest! {
    /// The three smartness predicates (`IsFusionPeer`, `HasClosedLoop`,
    /// `HasQueryableOntology`) are each defined on exactly the three
    /// Smart* concepts and agree with each other — Kephart & Chess (2003);
    /// IEEE 1451.0-2007; Olfati-Saber, Fax & Murray (2007).
    #[test]
    fn prop_smartness_predicates_exactly_on_smart_concepts(c in arb_concept()) {
        let smart = is_smart(c);
        prop_assert_eq!(IsFusionPeer.get(&c), if smart { Some(true) } else { None });
        prop_assert_eq!(HasClosedLoop.get(&c), if smart { Some(true) } else { None });
        prop_assert_eq!(
            HasQueryableOntology.get(&c),
            if smart { Some(true) } else { None }
        );
    }

    /// `SelfStarKind` and `MapeKPhaseFocus` are defined on exactly the
    /// four self-* concepts (Kephart & Chess 2003 §2, Table 1).
    #[test]
    fn prop_self_star_qualities_exactly_on_self_star(c in arb_concept()) {
        let self_star = is_self_star(c);
        prop_assert_eq!(SelfStarKind.get(&c).is_some(), self_star);
        prop_assert_eq!(MapeKPhaseFocus.get(&c).is_some(), self_star);
    }

    /// Every arrow of the category carries a non-empty name.
    #[test]
    fn prop_every_arrow_is_named(_seed in any::<u32>()) {
        for m in SmartElementCategory::morphisms() {
            prop_assert!(!m.meta().name.as_str().is_empty());
        }
    }

    /// Structural + domain axioms all discharge, regardless of sampling.
    #[test]
    fn prop_all_axioms_hold(_seed in 0..16u32) {
        for axiom in SmartElementOntology::axioms() {
            if let Err(c) = axiom.verify() {
                prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
            }
        }
    }
}

pr4xis::register_praxis_value!(
    prop_smartness_predicates_exactly_on_smart_concepts,
    Verifiable
);
pr4xis::register_praxis_value!(prop_self_star_qualities_exactly_on_self_star, Verifiable);
pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
pr4xis::register_praxis_value!(prop_all_axioms_hold, Verifiable);

/// The loop-closure witness: a full MAPE cycle acts in the four phases in
/// exactly Kephart & Chess (2003) §3's order, and the situation's phase
/// wraps back to Monitor — the cycle closes.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn full_cycle_visits_phases_in_order_and_closes() {
    let fixture = smart_element_fixture().expect("fixture is non-singular");
    let (end, visited) = run_mape_cycle(&fixture);
    assert_eq!(
        visited,
        MAPE_PHASE_ORDER.to_vec(),
        "the cycle visits Monitor -> Analyze -> Plan -> Execute in order"
    );
    assert_eq!(
        end.phase,
        MapePhase::Monitor,
        "the loop closes back to Monitor after Execute"
    );
    // The Monitor+Analyze phases updated the knowledge base.
    assert!(end.knowledge.configured, "Monitor configured the element");
    assert!(end.knowledge.healthy, "Analyze diagnosed health");
}

/// Exclusion-before-aggregation, at the engine level: observing the
/// equivocator distrusts it, drops it from the trusted neighbourhood, and
/// the next aggregation ignores it — the honest neighbour still counts,
/// and the fused posterior differs from the naive one (Lamport, Shostak &
/// Pease 1982; Li et al. 2004 SUNDR).
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn observing_equivocation_excludes_before_aggregation() {
    let fixture = smart_element_fixture().expect("fixture is non-singular");
    assert!(trusts(&fixture, EQUIVOCATOR_PEER), "starts trusted");
    let naive = aggregate_trusted(&fixture);

    let observed = apply(
        &fixture,
        &SmartElementAction::ObserveEquivocation {
            peer: EQUIVOCATOR_PEER,
        },
    );
    assert!(!trusts(&observed, EQUIVOCATOR_PEER), "equivocator excluded");
    assert!(trusts(&observed, HONEST_PEER), "honest neighbour retained");

    let after = aggregate_trusted(&observed);
    assert!(
        !information_eq(&naive, &after),
        "exclusion changed the fused posterior (non-vacuity)"
    );
}

/// Gossiping with the equivocator after it is distrusted is a no-op — its
/// contribution never enters the local estimate.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn gossip_with_distrusted_peer_is_ignored() {
    let fixture = smart_element_fixture().expect("fixture is non-singular");
    let observed = apply(
        &fixture,
        &SmartElementAction::ObserveEquivocation {
            peer: EQUIVOCATOR_PEER,
        },
    );
    let gossiped = apply(
        &observed,
        &SmartElementAction::GossipEstimate {
            with: EQUIVOCATOR_PEER,
        },
    );
    assert!(
        information_eq(&observed.local_estimate, &gossiped.local_estimate),
        "a distrusted peer's contribution never enters the estimate"
    );
}
