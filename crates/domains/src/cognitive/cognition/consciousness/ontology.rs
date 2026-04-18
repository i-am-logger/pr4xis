//! Consciousness — C1 × C2 product (Dehaene, Lau & Kouider, Science 2017).
//!
//! Consciousness is TWO orthogonal dimensions, not one:
//!
//! C1 (Global Broadcasting): selection of information for global
//!    availability. GWT (Baars 1988). IIT (Tononi 2004) contributes
//!    the integration measure. This connects to the processing pipeline.
//!
//! C2 (Self-Monitoring): metacognitive monitoring of computations,
//!    leading to certainty/error. Higher-Order (Rosenthal 2005).
//!    This connects to metacognition.
//!
//! C1 and C2 are ORTHOGONAL. A system can have C1 without C2
//! (broadcasting without self-monitoring) or C2 without C1
//! (metacognitive assessment without global access).
//!
//! The consciousness ontology is the PRODUCT C1 × C2. Each component
//! has its own functor to downstream ontologies:
//!   π₁: Consciousness → Pipeline (via C1 broadcasting)
//!   π₂: Consciousness → Metacognition (via C2 self-monitoring)
//!
//! Source: Dehaene, Lau & Kouider, Science (2017);
//!         Tononi (2004, 2012); Baars (1988, 2005);
//!         Rosenthal (2005); Block (1995)

use pr4xis::category::Category;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "C1",
    source: "Dehaene, Lau & Kouider (2017); Baars (1988); Tononi (2004)",
    being: MentalObject,

    concepts: [
        GlobalWorkspace,
        Coalition,
        BroadcastMessage,
        UnconsciousProcessor,
        ConsciousAccess,
        Attention,
        IntegratedInformation,
    ],

    labels: {
        GlobalWorkspace: ("en", "Global workspace", "The shared broadcast medium (Baars 1988)."),
        Coalition: ("en", "Coalition", "A coalition of processors competing for access."),
        BroadcastMessage: ("en", "Broadcast message", "The message broadcast when a coalition wins."),
        UnconsciousProcessor: ("en", "Unconscious processor", "A processor operating outside the workspace."),
        ConsciousAccess: ("en", "Conscious access", "The act of entering the global workspace."),
        Attention: ("en", "Attention", "The spotlight selecting what enters (Baars theater metaphor)."),
        IntegratedInformation: ("en", "Integrated information Φ", "Degree of broadcasting integration (Tononi 2004)."),
    },

    edges: [
        (Attention, ConsciousAccess, Selects),
        (GlobalWorkspace, Coalition, HasComponent),
        (GlobalWorkspace, BroadcastMessage, HasComponent),
        (Coalition, BroadcastMessage, Broadcasts),
        (ConsciousAccess, BroadcastMessage, Broadcasts),
    ],
}

pr4xis::ontology! {
    name: "C2",
    source: "Dehaene, Lau & Kouider (2017); Rosenthal (2005); Tononi (2012); Block (1995)",
    being: MentalObject,

    concepts: [
        FirstOrderState,
        HigherOrderRepresentation,
        CauseEffectStructure,
        Repertoire,
        Mechanism,
        AccessConsciousness,
        PhenomenalConsciousness,
    ],

    labels: {
        FirstOrderState: ("en", "First-order state", "A representation of the world."),
        HigherOrderRepresentation: ("en", "Higher-order representation", "A representation OF a first-order state (Rosenthal 2005)."),
        CauseEffectStructure: ("en", "Cause-effect structure", "Qualitative character (IIT)."),
        Repertoire: ("en", "Repertoire", "The repertoire of possible states."),
        Mechanism: ("en", "Mechanism", "The mechanism generating integrated information."),
        AccessConsciousness: ("en", "Access consciousness", "Available for reasoning/report (Block 1995)."),
        PhenomenalConsciousness: ("en", "Phenomenal consciousness", "Subjective experience (Block 1995)."),
    },

    edges: [
        (HigherOrderRepresentation, FirstOrderState, Represents),
        (HigherOrderRepresentation, AccessConsciousness, Enables),
        (Mechanism, CauseEffectStructure, Generates),
        (CauseEffectStructure, Mechanism, HasComponent),
        (CauseEffectStructure, Repertoire, HasComponent),
        (CauseEffectStructure, PhenomenalConsciousness, Generates),
    ],
}

/// Which dimension a concept belongs to.
#[derive(Debug, Clone)]
pub struct Dimension;

impl Quality for Dimension {
    type Individual = C1Concept;
    type Value = &'static str;

    fn get(&self, _individual: &C1Concept) -> Option<&'static str> {
        Some("C1-Broadcasting")
    }
}

/// C2 dimension quality.
#[derive(Debug, Clone)]
pub struct C2Dimension;

impl Quality for C2Dimension {
    type Individual = C2Concept;
    type Value = &'static str;

    fn get(&self, _individual: &C2Concept) -> Option<&'static str> {
        Some("C2-SelfMonitoring")
    }
}

/// Attention selects ConsciousAccess (GWT core).
#[derive(Debug)]
pub struct AttentionCausesAccess;

impl Axiom for AttentionCausesAccess {
    fn description(&self) -> &str {
        "Attention selects ConsciousAccess (Baars 1988)"
    }
    fn holds(&self) -> bool {
        C1Category::morphisms().iter().any(|r| {
            r.from == C1Concept::Attention
                && r.to == C1Concept::ConsciousAccess
                && r.kind == C1RelationKind::Selects
        })
    }
}
pr4xis::register_axiom!(AttentionCausesAccess);

/// Higher-order represents first-order (Rosenthal 2005).
#[derive(Debug)]
pub struct HigherOrderRepresentsFirst;

impl Axiom for HigherOrderRepresentsFirst {
    fn description(&self) -> &str {
        "HigherOrderRepresentation represents FirstOrderState (Rosenthal 2005)"
    }
    fn holds(&self) -> bool {
        C2Category::morphisms().iter().any(|r| {
            r.from == C2Concept::HigherOrderRepresentation
                && r.to == C2Concept::FirstOrderState
                && r.kind == C2RelationKind::Represents
        })
    }
}
pr4xis::register_axiom!(HigherOrderRepresentsFirst);

impl Ontology for C1Ontology {
    type Cat = C1Category;
    type Qual = Dimension;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        C1Ontology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(AttentionCausesAccess)]
    }
}

impl Ontology for C2Ontology {
    type Cat = C2Category;
    type Qual = C2Dimension;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        C2Ontology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(HigherOrderRepresentsFirst)]
    }
}
