// Consciousness — C1 × C2 product (Dehaene, Lau & Kouider, Science 2017).
//
// Consciousness is TWO orthogonal dimensions, not one:
//
// C1 (Global Broadcasting): selection of information for global
//    availability. GWT (Baars 1988). IIT (Tononi 2004) contributes
//    the integration measure. This connects to the processing pipeline.
//
// C2 (Self-Monitoring): metacognitive monitoring of computations,
//    leading to certainty/error. Higher-Order (Rosenthal 2005).
//    This connects to metacognition.
//
// C1 and C2 are ORTHOGONAL. A system can have C1 without C2
// (broadcasting without self-monitoring) or C2 without C1
// (metacognitive assessment without global access).
//
// The consciousness ontology is the PRODUCT C1 × C2. Each component
// has its own functor to downstream ontologies:
//   π₁: Consciousness → Pipeline (via C1 broadcasting)
//   π₂: Consciousness → Metacognition (via C2 self-monitoring)
//
// Source: Dehaene, Lau & Kouider, Science (2017);
//         Tononi (2004, 2012); Baars (1988, 2005);
//         Rosenthal (2005); Block (1995)

use pr4xis::category::{Category, Entity};
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// C1 — Global Broadcasting (GWT + IIT).
/// Selection of information for global availability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum C1Concept {
    /// The global workspace — shared broadcast medium (Baars 1988).
    GlobalWorkspace,
    /// A coalition of processors competing for access.
    Coalition,
    /// The message broadcast when a coalition wins.
    BroadcastMessage,
    /// A processor operating outside the workspace.
    UnconsciousProcessor,
    /// The act of entering the global workspace.
    ConsciousAccess,
    /// The spotlight selecting what enters (Baars: theater metaphor).
    Attention,
    /// Integrated information Φ — degree of broadcasting integration (Tononi).
    IntegratedInformation,
}

define_ontology! {
    /// C1 — Global Broadcasting (Dehaene 2017: consciousness in the first sense).
    pub C1Ontology for C1Category {
        concepts: C1Concept,
        relation: C1Relation,
        kind: C1RelationKind,
        kinds: [
            /// Attention selects for conscious access.
            Selects,
            /// Coalition broadcasts message.
            Broadcasts,
            /// Workspace has components.
            HasComponent,
        ],
        edges: [
            (Attention, ConsciousAccess, Selects),
            (GlobalWorkspace, Coalition, HasComponent),
            (GlobalWorkspace, BroadcastMessage, HasComponent),
            (Coalition, BroadcastMessage, Broadcasts),
            (ConsciousAccess, BroadcastMessage, Broadcasts),
        ],
        composed: [
            (Attention, BroadcastMessage),
            (GlobalWorkspace, UnconsciousProcessor),
        ],

        being: MentalObject,
        source: "Dehaene, Lau & Kouider (2017); Baars (1988); Tononi (2004)",
    }
}

/// C2 — Self-Monitoring (Higher-Order + Metacognitive).
/// Monitoring of computations, certainty/error judgments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum C2Concept {
    /// A first-order state — representation of the world.
    FirstOrderState,
    /// A higher-order representation OF a first-order state (Rosenthal).
    HigherOrderRepresentation,
    /// Cause-effect structure — qualitative character (IIT).
    CauseEffectStructure,
    /// The repertoire of possible states.
    Repertoire,
    /// The mechanism generating integrated information.
    Mechanism,
    /// Access consciousness — available for reasoning/report (Block).
    AccessConsciousness,
    /// Phenomenal consciousness — subjective experience (Block).
    PhenomenalConsciousness,
}

define_ontology! {
    /// C2 — Self-Monitoring (Dehaene 2017: consciousness in the second sense).
    pub C2Ontology for C2Category {
        concepts: C2Concept,
        relation: C2Relation,
        kind: C2RelationKind,
        kinds: [
            /// Higher-order represents first-order.
            Represents,
            /// Mechanism generates information.
            Generates,
            /// Structure has components.
            HasComponent,
            /// Access enables reporting.
            Enables,
        ],
        edges: [
            (HigherOrderRepresentation, FirstOrderState, Represents),
            (HigherOrderRepresentation, AccessConsciousness, Enables),
            (Mechanism, CauseEffectStructure, Generates),
            (CauseEffectStructure, Mechanism, HasComponent),
            (CauseEffectStructure, Repertoire, HasComponent),
            (CauseEffectStructure, PhenomenalConsciousness, Generates),
        ],
        composed: [
            (HigherOrderRepresentation, PhenomenalConsciousness),
            (Mechanism, Repertoire),
            (Mechanism, PhenomenalConsciousness),
        ],

        being: MentalObject,
        source: "Dehaene, Lau & Kouider (2017); Rosenthal (2005); Tononi (2012); Block (1995)",
    }
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
