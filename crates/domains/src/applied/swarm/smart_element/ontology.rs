//! SmartElement — the autonomic edge element: a MAPE-K manager that is
//! also a signed-estimate fusion peer, carrying a queryable local
//! ontology.
//!
//! Two established traditions are composed here; neither is extended:
//!
//! - **Kephart & Chess (2003)** *The Vision of Autonomic Computing*, IEEE
//!   Computer 36(1) — the autonomic element: a managed element governed by
//!   an autonomic manager that closes a MAPE-K loop over a knowledge base,
//!   exhibiting the four self-* properties (§2, Table 1; §3).
//! - **IEEE Std 1451.0-2007** (smart transducer interface) and **Lee
//!   (2000)** *IEEE 1451: A Standard in Support of Smart Transducer
//!   Networking*, IEEE IMTC — the smart transducer: a Transducer plus a
//!   Network Capable Application Processor (NCAP), carrying a
//!   self-describing Transducer Electronic Data Sheet (TEDS, §5).
//!
//! The **synthesis concept** — `SmartElement`, and its `SmartSensor` /
//! `SmartDriver` specialisations — is this codebase's own: an element that
//! simultaneously (a) closes a MAPE-K loop, (b) carries a queryable local
//! ontology (concretely a TEDS-like self-description), and (c) participates
//! as a fusion peer that signs its estimates and excludes equivocators.
//! Its three glosses say so explicitly, and its content is discharged by
//! the five axioms below plus the six cross-functors — the synthesis is
//! *grounded*, not merely asserted.
//!
//! The estimation mathematics and the autonomic loop are established
//! literature (see [`super::engine`]); the only novelty claimed is the
//! ontological composition.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::applied::dependability::ontology::DependabilityConcept;
use crate::applied::swarm::consensus::ontology::ConsensusConcept;
use crate::formal::systems::mape_k::ontology::{LoopIsClosed, MapeKConcept};

use super::consensus_functor::SmartElementToConsensus;
use super::dependability_functor::SmartElementToDependability;
use super::engine::{
    EQUIVOCATOR_PEER, HONEST_PEER, SmartElementAction, aggregate_trusted, apply, information_eq,
    smart_element_fixture, trusts,
};
use super::mape_k_functor::SmartElementToMapeK;

pr4xis::ontology! {
    name: "SmartElement",
    source: "Kephart & Chess (2003) IEEE Computer 36(1); IEEE Std 1451.0-2007; Lee (2000) IMTC",

    concepts: [
        // === The smart transducer (IEEE 1451.0-2007; Lee 2000) ===
        Transducer,
        Teds,
        Ncap,

        // === The autonomic element (Kephart & Chess 2003 §3) ===
        ManagedElement,
        AutonomicManager,
        LocalOntology,

        // === The synthesis: autonomic edge elements ===
        SmartElement,
        SmartSensor,
        SmartDriver,

        // === The self-* properties (Kephart & Chess 2003 §2, Table 1) ===
        SelfStarProperty,
        SelfConfiguration,
        SelfHealing,
        SelfOptimization,
        SelfProtection,
    ],

    labels: {
        Transducer: ("en", "Transducer", "IEEE Std 1451.0-2007 (smart transducer interface): the physical element that senses or actuates - the point where the digital system meets the physical world."),
        Teds: ("en", "TEDS", "IEEE Std 1451.0-2007 §5, Transducer Electronic Data Sheet: the self-describing metadata a smart transducer carries - the concrete, standardized anchor for 'carries a queryable self-description'."),
        Ncap: ("en", "NCAP", "IEEE Std 1451.0-2007; Lee (2000) 'IEEE 1451: A Standard in Support of Smart Transducer Networking' IEEE IMTC: the Network Capable Application Processor - the network-facing processor that operates the transducer and exposes it to the network."),
        ManagedElement: ("en", "Managed element", "Kephart & Chess (2003) 'The Vision of Autonomic Computing' IEEE Computer 36(1) §3: the resource an autonomic manager governs - hardware, software, or an application."),
        AutonomicManager: ("en", "Autonomic manager", "Kephart & Chess (2003) §3: the component that closes a MAPE-K loop (Monitor / Analyze / Plan / Execute over Knowledge) over a managed element."),
        LocalOntology: ("en", "Local ontology", "The queryable knowledge a smart element reasons over at the edge - Kephart & Chess (2003) §3's Knowledge base made local; concretely a TEDS-like self-description (IEEE 1451.0-2007 §5) the element can answer questions from."),
        SmartElement: ("en", "Smart element", "SYNTHESIS CONCEPT of this codebase: an autonomic edge element that self-manages (closes a MAPE-K loop, Kephart & Chess 2003 §3), carries a queryable local ontology (a TEDS-like self-description, IEEE 1451.0-2007 §5), and participates as a signed-estimate fusion peer (Olfati-Saber, Fax & Murray 2007). Composed from Kephart & Chess's autonomic element, IEEE 1451's smart transducer, and the consensus peer; grounded by the axioms and functors of this ontology, not asserted."),
        SmartSensor: ("en", "Smart sensor", "A SmartElement whose transducer senses - IEEE Std 1451's 'smart transducer' names exactly this (a transducer with an NCAP and TEDS); Lee (2000). The sensing specialisation of the synthesis element."),
        SmartDriver: ("en", "Smart driver", "SYNTHESIS CONCEPT: a SmartElement that operates a transducer/device and hosts the autonomic loop - the operating-system device driver (Corbet, Rubini & Kroah-Hartman 2005) made autonomic (Kephart & Chess 2003); its closest IEEE 1451 analog is the NCAP that drives a transducer. Grounded by the driver functor (SmartDriver -> Driver)."),
        SelfStarProperty: ("en", "Self-* property", "Kephart & Chess (2003) §2: the abstract parent of the four self-management properties an autonomic system exhibits."),
        SelfConfiguration: ("en", "Self-configuration", "Kephart & Chess (2003) §2, Table 1: automatic configuration of components following high-level policies - the system installs and adapts itself."),
        SelfHealing: ("en", "Self-healing", "Kephart & Chess (2003) §2, Table 1: automatic detection, diagnosis, and repair of localized problems - the system finds and fixes its own faults."),
        SelfOptimization: ("en", "Self-optimization", "Kephart & Chess (2003) §2, Table 1: automatic monitoring and tuning of resources to meet end-user needs - the system continually seeks to improve its own operation."),
        SelfProtection: ("en", "Self-protection", "Kephart & Chess (2003) §2, Table 1: automatic defence against malicious attacks or cascading failures, and anticipation of problems by early warning - the system protects itself, here by excluding equivocating peers."),
    },

    is_a: [
        // The two element specialisations (this codebase's synthesis).
        (SmartSensor, SmartElement),
        (SmartDriver, SmartElement),
        // The four self-* properties (Kephart & Chess 2003 §2, Table 1).
        (SelfConfiguration, SelfStarProperty),
        (SelfHealing, SelfStarProperty),
        (SelfOptimization, SelfStarProperty),
        (SelfProtection, SelfStarProperty),
    ],

    has_a: [
        // The autonomic element (Kephart & Chess 2003 §3): a smart element
        // has a manager and a local knowledge base.
        (SmartElement, AutonomicManager),
        (SmartElement, LocalOntology),
        // The smart transducer (IEEE 1451.0-2007): a sensing element and an
        // NCAP each carry a transducer.
        (SmartSensor, Transducer),
        (Ncap, Transducer),
    ],

    edges: [
        // Kephart & Chess (2003) §3: the element carries its Knowledge.
        (SmartElement, LocalOntology, Carries),
        // Kephart & Chess (2003) §2: the element exhibits the self-* properties.
        (SmartElement, SelfStarProperty, Exhibits),
        // IEEE Std 1451.0-2007 (NCAP role): the smart driver operates the transducer.
        (SmartDriver, Transducer, Operates),
        // IEEE Std 1451.0-2007 §5: the transducer is described by its TEDS.
        (Transducer, Teds, DescribedBy),
        // Kephart & Chess (2003) §3: the manager governs a managed element.
        (AutonomicManager, ManagedElement, Manages),
    ],
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// The autonomic self-* property a concept denotes — Kephart & Chess
/// (2003) §2, Table 1: the closed set of four self-management properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutonomicProperty {
    /// Self-configuration: automatic component configuration under policy.
    Configuration,
    /// Self-healing: automatic fault detection, diagnosis, and repair.
    Healing,
    /// Self-optimization: automatic resource monitoring and tuning.
    Optimization,
    /// Self-protection: automatic defence and equivocator exclusion.
    Protection,
}

/// Which self-* property a concept denotes — Kephart & Chess (2003) §2,
/// Table 1. `Some` for exactly the four self-* concepts; `None`
/// everywhere else (including the abstract `SelfStarProperty` parent,
/// which denotes no single property).
#[derive(Debug, Clone)]
pub struct SelfStarKind;

impl Quality for SelfStarKind {
    type Individual = SmartElementConcept;
    type Value = AutonomicProperty;

    fn get(&self, c: &SmartElementConcept) -> Option<AutonomicProperty> {
        use AutonomicProperty as A;
        use SmartElementConcept as C;
        match c {
            C::SelfConfiguration => Some(A::Configuration),
            C::SelfHealing => Some(A::Healing),
            C::SelfOptimization => Some(A::Optimization),
            C::SelfProtection => Some(A::Protection),
            _ => None,
        }
    }
}

/// The MAPE-K phase each self-* property most exercises — Kephart & Chess
/// (2003). The value space is the EXISTING [`MapeKConcept`] (reused, not
/// redefined), and each assignment is grounded in K&C's own §2 / Table 1
/// descriptions:
///
/// - `SelfHealing` → `Analyze`: healing is *detect + diagnose* a localized
///   problem — the Analyze phase's diagnosis function (K&C Table 1).
/// - `SelfOptimization` → `Plan`: optimization *decides* how to retune
///   resources to meet needs — the Plan phase (K&C Table 1).
/// - `SelfConfiguration` → `Execute`: configuration *installs and adapts*
///   components under policy — carrying out the plan, the Execute phase
///   (K&C Table 1).
/// - `SelfProtection` → `Monitor`: protection *continuously watches* for
///   attacks and gives early warning — the Monitor phase's observation
///   function (K&C Table 1).
///
/// `None` for every concept that is not a self-* property.
#[derive(Debug, Clone)]
pub struct MapeKPhaseFocus;

impl Quality for MapeKPhaseFocus {
    type Individual = SmartElementConcept;
    type Value = MapeKConcept;

    fn get(&self, c: &SmartElementConcept) -> Option<MapeKConcept> {
        use SmartElementConcept as C;
        match c {
            C::SelfHealing => Some(MapeKConcept::Analyze),
            C::SelfOptimization => Some(MapeKConcept::Plan),
            C::SelfConfiguration => Some(MapeKConcept::Execute),
            C::SelfProtection => Some(MapeKConcept::Monitor),
            _ => None,
        }
    }
}

/// Whether a concept is a fusion peer — one that signs and gossips
/// estimates and participates in consensus-on-information (Olfati-Saber,
/// Fax & Murray 2007). `Some(true)` for the three Smart* concepts; `None`
/// elsewhere. One of the three smartness predicates the axioms verify.
#[derive(Debug, Clone)]
pub struct IsFusionPeer;

impl Quality for IsFusionPeer {
    type Individual = SmartElementConcept;
    type Value = bool;

    fn get(&self, c: &SmartElementConcept) -> Option<bool> {
        use SmartElementConcept as C;
        match c {
            C::SmartElement | C::SmartSensor | C::SmartDriver => Some(true),
            _ => None,
        }
    }
}

/// Whether a concept closes a MAPE-K loop — Kephart & Chess (2003) §3.
/// `Some(true)` for the three Smart* concepts; `None` elsewhere. One of
/// the three smartness predicates the axioms verify.
#[derive(Debug, Clone)]
pub struct HasClosedLoop;

impl Quality for HasClosedLoop {
    type Individual = SmartElementConcept;
    type Value = bool;

    fn get(&self, c: &SmartElementConcept) -> Option<bool> {
        use SmartElementConcept as C;
        match c {
            C::SmartElement | C::SmartSensor | C::SmartDriver => Some(true),
            _ => None,
        }
    }
}

/// Whether a concept carries a queryable local ontology — a TEDS-like
/// self-description it can answer questions from (IEEE Std 1451.0-2007 §5;
/// Kephart & Chess 2003 §3 Knowledge). `Some(true)` for the three Smart*
/// concepts; `None` elsewhere. One of the three smartness predicates the
/// axioms verify.
#[derive(Debug, Clone)]
pub struct HasQueryableOntology;

impl Quality for HasQueryableOntology {
    type Individual = SmartElementConcept;
    type Value = bool;

    fn get(&self, c: &SmartElementConcept) -> Option<bool> {
        use SmartElementConcept as C;
        match c {
            C::SmartElement | C::SmartSensor | C::SmartDriver => Some(true),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The three Smart* concepts the smartness predicates and functor axioms
/// quantify over.
fn smart_concepts() -> [SmartElementConcept; 3] {
    [
        SmartElementConcept::SmartElement,
        SmartElementConcept::SmartSensor,
        SmartElementConcept::SmartDriver,
    ]
}

fn direct_children_of(parent: SmartElementConcept) -> Vec<SmartElementConcept> {
    SmartElementCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == SmartElementRelationKind::Subsumption && m.target() == parent)
        .map(|m| m.source())
        .collect()
}

fn kinded_edge_exists(
    from: SmartElementConcept,
    to: SmartElementConcept,
    kind: SmartElementRelationKind,
) -> bool {
    SmartElementCategory::morphisms()
        .iter()
        .any(|m| m.source() == from && m.target() == to && m.kind() == kind)
}

fn verdict_from(axiom: &dyn Axiom, ok: bool) -> pr4xis::logic::proof::Verdict {
    use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
    if ok {
        Ok(Box::new(SimpleProof::new(axiom.meta())))
    } else {
        Err(Box::new(SimpleCounterexample::new(axiom.meta())))
    }
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

/// Kephart & Chess (2003) §3: a smart element is a full autonomic element.
/// The `SmartElementToMapeK` functor's image over all SmartElement
/// concepts covers every MAPE-K phase and the knowledge substrate —
/// `{Monitor, Analyze, Plan, Execute, Knowledge}` — AND the target
/// ontology's own `LoopIsClosed` axiom verifies (the four phases form a
/// closed cycle). Together: the element runs a complete, closed MAPE-K
/// loop, not a partial or open one.
pub struct SmartClosesMapeKLoop;

impl Axiom for SmartClosesMapeKLoop {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::Functor;
        let image: Vec<MapeKConcept> = SmartElementConcept::variants()
            .into_iter()
            .map(|c| SmartElementToMapeK::map_object(&c))
            .collect();
        let covers_all = [
            MapeKConcept::Monitor,
            MapeKConcept::Analyze,
            MapeKConcept::Plan,
            MapeKConcept::Execute,
            MapeKConcept::Knowledge,
        ]
        .iter()
        .all(|phase| image.contains(phase));
        // The target ontology's own loop-closure axiom, invoked directly.
        let loop_closed = LoopIsClosed.verify().is_ok();
        verdict_from(self, covers_all && loop_closed)
    }

    pr4xis::axiom_meta!(
        "SmartClosesMapeKLoop",
        "the SmartElementToMapeK image covers all of {Monitor, Analyze, Plan, Execute, Knowledge} and the target MAPE-K ontology's LoopIsClosed axiom verifies",
        "Kephart & Chess (2003) IEEE Computer 36(1) sec 3"
    );
}
pr4xis::register_axiom!(
    SmartClosesMapeKLoop,
    "Kephart & Chess (2003) IEEE Computer 36(1) sec 3"
);

/// Olfati-Saber, Fax & Murray (2007); Kephart & Chess (2003): a smart
/// element is a fusion peer. The `SmartElementToConsensus` functor maps
/// `SmartElement`, `SmartSensor`, and `SmartDriver` to `Peer`, and the
/// `IsFusionPeer` predicate is `Some(true)` for all three.
pub struct SmartIsFusionPeer;

impl Axiom for SmartIsFusionPeer {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::Functor;
        let maps_to_peer = smart_concepts()
            .iter()
            .all(|c| SmartElementToConsensus::map_object(c) == ConsensusConcept::Peer);
        let flagged = smart_concepts()
            .iter()
            .all(|c| IsFusionPeer.get(c) == Some(true));
        verdict_from(self, maps_to_peer && flagged)
    }

    pr4xis::axiom_meta!(
        "SmartIsFusionPeer",
        "SmartElementToConsensus maps SmartElement, SmartSensor and SmartDriver to Peer, and IsFusionPeer is Some(true) for all three",
        "Olfati-Saber, Fax & Murray (2007) Proc. IEEE 95(1); Kephart & Chess (2003)"
    );
}
pr4xis::register_axiom!(
    SmartIsFusionPeer,
    "Olfati-Saber, Fax & Murray (2007) Proc. IEEE 95(1); Kephart & Chess (2003)"
);

/// IEEE Std 1451.0-2007 §5; Kephart & Chess (2003): a smart element
/// carries a queryable local ontology. The `Carries` edge
/// (`SmartElement → LocalOntology`) exists, `HasQueryableOntology` is
/// `Some(true)` for the three Smart* concepts, and the standardized
/// self-description anchor is present: `Transducer` is `DescribedBy`
/// `Teds`.
pub struct SmartCarriesQueryableOntology;

impl Axiom for SmartCarriesQueryableOntology {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        let carries = kinded_edge_exists(
            SmartElementConcept::SmartElement,
            SmartElementConcept::LocalOntology,
            SmartElementRelationKind::Carries,
        );
        let queryable = smart_concepts()
            .iter()
            .all(|c| HasQueryableOntology.get(c) == Some(true));
        let described_by_teds = kinded_edge_exists(
            SmartElementConcept::Transducer,
            SmartElementConcept::Teds,
            SmartElementRelationKind::DescribedBy,
        );
        verdict_from(self, carries && queryable && described_by_teds)
    }

    pr4xis::axiom_meta!(
        "SmartCarriesQueryableOntology",
        "the Carries edge SmartElement -> LocalOntology exists, HasQueryableOntology is Some(true) for the three Smart* concepts, and Transducer is DescribedBy Teds",
        "IEEE Std 1451.0-2007 sec 5; Kephart & Chess (2003)"
    );
}
pr4xis::register_axiom!(
    SmartCarriesQueryableOntology,
    "IEEE Std 1451.0-2007 sec 5; Kephart & Chess (2003)"
);

/// Kephart & Chess (2003) §2, Table 1: the self-* properties are exactly
/// four. The Subsumption-children of `SelfStarProperty` are exactly
/// `{SelfConfiguration, SelfHealing, SelfOptimization, SelfProtection}`
/// (set equality — a fifth child would break this axiom).
pub struct SelfStarComplete;

impl Axiom for SelfStarComplete {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        let children = direct_children_of(SmartElementConcept::SelfStarProperty);
        let expected = [
            SmartElementConcept::SelfConfiguration,
            SmartElementConcept::SelfHealing,
            SmartElementConcept::SelfOptimization,
            SmartElementConcept::SelfProtection,
        ];
        let set_equal =
            children.len() == expected.len() && expected.iter().all(|c| children.contains(c));
        verdict_from(self, set_equal)
    }

    pr4xis::axiom_meta!(
        "SelfStarComplete",
        "the Subsumption children of SelfStarProperty are exactly {SelfConfiguration, SelfHealing, SelfOptimization, SelfProtection}",
        "Kephart & Chess (2003) IEEE Computer 36(1)"
    );
}
pr4xis::register_axiom!(
    SelfStarComplete,
    "Kephart & Chess (2003) IEEE Computer 36(1)"
);

/// Avizienis et al. (2004); Lamport, Shostak & Pease (1982): self-
/// protection is fault handling that excludes equivocators. Structurally,
/// via `SmartElementToDependability`, `SelfProtection` maps to
/// `FaultHandling` (Avizienis §5.2: diagnosis, isolation,
/// reconfiguration). Operationally, on the engine fixture a smart element
/// observing a peer's equivocation moves it to distrusted and excludes it
/// from the fusion neighbourhood *before* the next aggregation — and that
/// exclusion changes the fused posterior (non-vacuity).
pub struct SelfProtectionExcludesEquivocators;

impl Axiom for SelfProtectionExcludesEquivocators {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::Functor;

        // Structural half: SelfProtection lands on the fault-handling side.
        let maps_to_fault_handling =
            SmartElementToDependability::map_object(&SmartElementConcept::SelfProtection)
                == DependabilityConcept::FaultHandling;

        // Operational half: the exclusion-before-aggregation experiment.
        let Some(fixture) = smart_element_fixture() else {
            return verdict_from(self, false);
        };
        // Naive aggregation still counts the equivocator.
        let naive = aggregate_trusted(&fixture);
        // Self-protection observes the equivocation, then aggregates.
        let observed = apply(
            &fixture,
            &SmartElementAction::ObserveEquivocation {
                peer: EQUIVOCATOR_PEER,
            },
        );
        let excluded = !trusts(&observed, EQUIVOCATOR_PEER) && trusts(&observed, HONEST_PEER);
        let after = aggregate_trusted(&observed);

        // The excluded aggregate is exactly local ⊕ the honest neighbour.
        let expected = fixture
            .neighborhood
            .iter()
            .find(|n| n.peer == HONEST_PEER)
            .map(|n| fixture.local_estimate.fuse(&n.contribution));
        let honest_only = matches!(&expected, Some(e) if information_eq(&after, e));
        // Non-vacuity: exclusion actually changed the fused posterior.
        let non_vacuous = !information_eq(&naive, &after);

        verdict_from(
            self,
            maps_to_fault_handling && excluded && honest_only && non_vacuous,
        )
    }

    pr4xis::axiom_meta!(
        "SelfProtectionExcludesEquivocators",
        "SelfProtection maps to FaultHandling via SmartElementToDependability, and on the engine a smart element observing a peer's equivocation excludes it from the fusion neighbourhood before the next aggregation (which changes the fused posterior)",
        "Avizienis, Laprie, Randell & Landwehr (2004) IEEE TDSC 1(1); Lamport, Shostak & Pease (1982) ACM TOPLAS 4(3)"
    );
}
pr4xis::register_axiom!(
    SelfProtectionExcludesEquivocators,
    "Avizienis, Laprie, Randell & Landwehr (2004) IEEE TDSC 1(1); Lamport, Shostak & Pease (1982) ACM TOPLAS 4(3)"
);

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for SmartElementOntology {
    type Cat = SmartElementCategory;
    type Qual = SelfStarKind;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(SmartClosesMapeKLoop));
        axioms.push(Box::new(SmartIsFusionPeer));
        axioms.push(Box::new(SmartCarriesQueryableOntology));
        axioms.push(Box::new(SelfStarComplete));
        axioms.push(Box::new(SelfProtectionExcludesEquivocators));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<SmartElementCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        SmartElementOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn fourteen_concepts() {
        assert_eq!(SmartElementConcept::variants().len(), 14);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn smart_closes_mape_k_loop_holds() {
        assert!(SmartClosesMapeKLoop.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn smart_is_fusion_peer_holds() {
        assert!(SmartIsFusionPeer.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn smart_carries_queryable_ontology_holds() {
        assert!(SmartCarriesQueryableOntology.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn self_star_complete_holds() {
        assert!(SelfStarComplete.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn self_protection_excludes_equivocators_holds() {
        assert!(SelfProtectionExcludesEquivocators.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn self_star_kind_classification() {
        use AutonomicProperty as A;
        use SmartElementConcept as C;
        assert_eq!(
            SelfStarKind.get(&C::SelfConfiguration),
            Some(A::Configuration)
        );
        assert_eq!(SelfStarKind.get(&C::SelfHealing), Some(A::Healing));
        assert_eq!(
            SelfStarKind.get(&C::SelfOptimization),
            Some(A::Optimization)
        );
        assert_eq!(SelfStarKind.get(&C::SelfProtection), Some(A::Protection));
        assert_eq!(
            SelfStarKind.get(&C::SelfStarProperty),
            None,
            "the abstract parent denotes no single self-* property"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn mape_k_phase_focus_reuses_mape_k_concepts() {
        use SmartElementConcept as C;
        // Grounded in Kephart & Chess (2003) Table 1 (see the quality doc).
        assert_eq!(
            MapeKPhaseFocus.get(&C::SelfHealing),
            Some(MapeKConcept::Analyze)
        );
        assert_eq!(
            MapeKPhaseFocus.get(&C::SelfOptimization),
            Some(MapeKConcept::Plan)
        );
        assert_eq!(
            MapeKPhaseFocus.get(&C::SelfConfiguration),
            Some(MapeKConcept::Execute)
        );
        assert_eq!(
            MapeKPhaseFocus.get(&C::SelfProtection),
            Some(MapeKConcept::Monitor)
        );
        assert_eq!(MapeKPhaseFocus.get(&C::SmartElement), None);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn three_smartness_predicates_agree_on_smart_concepts() {
        for c in smart_concepts() {
            assert_eq!(IsFusionPeer.get(&c), Some(true), "{c:?} is a fusion peer");
            assert_eq!(HasClosedLoop.get(&c), Some(true), "{c:?} closes a loop");
            assert_eq!(
                HasQueryableOntology.get(&c),
                Some(true),
                "{c:?} carries a queryable ontology"
            );
        }
        // None on a non-smart concept (the transducer is not itself smart).
        assert_eq!(IsFusionPeer.get(&SmartElementConcept::Transducer), None);
        assert_eq!(HasClosedLoop.get(&SmartElementConcept::Teds), None);
        assert_eq!(
            HasQueryableOntology.get(&SmartElementConcept::ManagedElement),
            None
        );
    }
}
