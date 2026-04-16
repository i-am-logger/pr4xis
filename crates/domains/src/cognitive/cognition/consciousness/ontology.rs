// Consciousness — IIT + GWT + Higher-Order Theories.
//
// Three complementary theories of consciousness, unified categorically:
//
// IIT (Tononi 2004, 2008, 2012): consciousness = integrated information (Φ).
// A system is conscious to the degree it integrates information beyond its
// parts. Five axioms: intrinsic existence, composition, information,
// integration, exclusion. Five postulates map axioms to physical properties.
//
// GWT (Baars 1988, 2005): consciousness = global broadcasting.
// Specialized unconscious processors compete for access to a shared
// global workspace. The winning coalition broadcasts its content to all
// other processors. Attention is the spotlight.
//
// Higher-Order (Rosenthal 2005, Lau & Rosenthal 2011): consciousness =
// having a representation OF a representation. A first-order state
// becomes conscious when a higher-order state represents it.
//
// For pr4xis: consciousness governs what enters the system's awareness.
// When processing a query, not everything is "conscious" — the global
// workspace selects what's salient for the response. The metacognition
// ontology monitors; consciousness determines what monitoring ATTENDS to.
//
// Composes with:
// - Self-Model: AwarenessLevel IS a consciousness property
// - Metacognition: MetaLevel IS the higher-order representation
// - Epistemics: what enters awareness IS what we know we know
// - Distinction: the mark IS the first act of consciousness

use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Concepts in the Consciousness ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum ConsciousnessConcept {
    // === IIT (Tononi 2004, 2012) ===
    /// Integrated information (Φ) — the quantity of consciousness.
    /// How much a system integrates information beyond its parts.
    IntegratedInformation,
    /// A cause-effect structure — the qualitative character of experience.
    /// What it's like to be in a particular conscious state.
    CauseEffectStructure,
    /// A mechanism — a set of elements in a definite state.
    /// The physical substrate that generates Φ.
    Mechanism,
    /// The repertoire of possible states — past causes and future effects.
    Repertoire,

    // === GWT (Baars 1988, 2005) ===
    /// The global workspace — the shared broadcast medium.
    /// Content on the "stage" is conscious; everything else is not.
    GlobalWorkspace,
    /// A coalition of processors competing for workspace access.
    Coalition,
    /// The message broadcast to all processors when a coalition wins.
    BroadcastMessage,
    /// A specialized processor operating outside awareness.
    UnconsciousProcessor,
    /// The act of entering the global workspace.
    ConsciousAccess,
    /// The spotlight selecting what enters the workspace.
    Attention,

    // === Higher-Order Theories (Rosenthal 2005) ===
    /// A first-order state — a representation of the world.
    FirstOrderState,
    /// A higher-order representation OF a first-order state.
    /// This is what makes a state conscious (Rosenthal).
    HigherOrderRepresentation,

    // === Bridging concepts ===
    /// Access consciousness — information available for reasoning/report.
    /// Block (1995): functionally conscious.
    AccessConsciousness,
    /// Phenomenal consciousness — the subjective experience.
    /// Block (1995): "what it's like."
    PhenomenalConsciousness,
}

define_ontology! {
    /// Consciousness — IIT + GWT + Higher-Order.
    pub ConsciousnessOntology for ConsciousnessCategory {
        concepts: ConsciousnessConcept,
        relation: ConsciousnessRelation,

        being: MentalObject,
        source: "Tononi (2004, 2012); Baars (1988, 2005); Rosenthal (2005); Block (1995)",

        is_a: ConsciousnessTaxonomy [
            // IIT: Mechanism generates IntegratedInformation
            (Mechanism, IntegratedInformation),
            // GWT: ConsciousAccess is-a kind of Attention
            (ConsciousAccess, Attention),
            // Both access and phenomenal are kinds of consciousness
            (AccessConsciousness, ConsciousAccess),
            (PhenomenalConsciousness, CauseEffectStructure),
        ],

        has_a: ConsciousnessMereology [
            // GWT: GlobalWorkspace contains BroadcastMessage
            (GlobalWorkspace, BroadcastMessage),
            // GlobalWorkspace contains competing Coalitions
            (GlobalWorkspace, Coalition),
            // Coalition consists of UnconsciousProcessors
            (Coalition, UnconsciousProcessor),
            // IIT: CauseEffectStructure has Mechanisms
            (CauseEffectStructure, Mechanism),
            // CauseEffectStructure has Repertoire
            (CauseEffectStructure, Repertoire),
            // Higher-order: HigherOrderRepresentation wraps FirstOrderState
            (HigherOrderRepresentation, FirstOrderState),
        ],

        causes: ConsciousnessCausation for ConsciousnessConcept [
            // GWT: Attention causes ConsciousAccess
            (Attention, ConsciousAccess),
            // GWT: Coalition winning causes BroadcastMessage
            (Coalition, BroadcastMessage),
            // Higher-order: HigherOrderRepresentation causes AccessConsciousness
            (HigherOrderRepresentation, AccessConsciousness),
            // IIT: Integration causes CauseEffectStructure
            (IntegratedInformation, CauseEffectStructure),
        ],

        opposes: ConsciousnessOpposition [
            // Conscious vs unconscious processing
            (ConsciousAccess, UnconsciousProcessor),
            // Access vs phenomenal consciousness (Block's distinction)
            (AccessConsciousness, PhenomenalConsciousness),
            // First-order vs higher-order (Rosenthal's hierarchy)
            (FirstOrderState, HigherOrderRepresentation),
        ],
    }
}

/// Whether a concept is from IIT vs GWT vs Higher-Order.
#[derive(Debug, Clone)]
pub struct TheoryOrigin;

impl Quality for TheoryOrigin {
    type Individual = ConsciousnessConcept;
    type Value = &'static str;

    fn get(&self, individual: &ConsciousnessConcept) -> Option<&'static str> {
        Some(match individual {
            ConsciousnessConcept::IntegratedInformation
            | ConsciousnessConcept::CauseEffectStructure
            | ConsciousnessConcept::Mechanism
            | ConsciousnessConcept::Repertoire => "IIT",
            ConsciousnessConcept::GlobalWorkspace
            | ConsciousnessConcept::Coalition
            | ConsciousnessConcept::BroadcastMessage
            | ConsciousnessConcept::UnconsciousProcessor
            | ConsciousnessConcept::ConsciousAccess
            | ConsciousnessConcept::Attention => "GWT",
            ConsciousnessConcept::FirstOrderState
            | ConsciousnessConcept::HigherOrderRepresentation => "Higher-Order",
            ConsciousnessConcept::AccessConsciousness
            | ConsciousnessConcept::PhenomenalConsciousness => "Block",
        })
    }
}

/// Attention causes ConsciousAccess (GWT core claim).
#[derive(Debug)]
pub struct AttentionCausesAccess;

impl Axiom for AttentionCausesAccess {
    fn description(&self) -> &str {
        "Attention causes ConsciousAccess (Baars 1988: spotlight metaphor)"
    }
    fn holds(&self) -> bool {
        use pr4xis::ontology::reasoning::causation::CausalDef;
        ConsciousnessCausation::relations().iter().any(|(c, e)| {
            *c == ConsciousnessConcept::Attention && *e == ConsciousnessConcept::ConsciousAccess
        })
    }
}

/// Conscious and unconscious processing are opposed (GWT).
#[derive(Debug)]
pub struct ConsciousUnconsciousOpposed;

impl Axiom for ConsciousUnconsciousOpposed {
    fn description(&self) -> &str {
        "ConsciousAccess and UnconsciousProcessor are opposed (GWT)"
    }
    fn holds(&self) -> bool {
        use pr4xis::ontology::reasoning::opposition::OppositionDef;
        ConsciousnessOpposition::pairs().iter().any(|(a, b)| {
            *a == ConsciousnessConcept::ConsciousAccess
                && *b == ConsciousnessConcept::UnconsciousProcessor
        })
    }
}

/// Integration produces cause-effect structure (IIT core).
#[derive(Debug)]
pub struct IntegrationProducesStructure;

impl Axiom for IntegrationProducesStructure {
    fn description(&self) -> &str {
        "IntegratedInformation causes CauseEffectStructure (Tononi 2012: IIT axiom)"
    }
    fn holds(&self) -> bool {
        use pr4xis::ontology::reasoning::causation::CausalDef;
        ConsciousnessCausation::relations().iter().any(|(c, e)| {
            *c == ConsciousnessConcept::IntegratedInformation
                && *e == ConsciousnessConcept::CauseEffectStructure
        })
    }
}

impl Ontology for ConsciousnessOntology {
    type Cat = ConsciousnessCategory;
    type Qual = TheoryOrigin;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        ConsciousnessOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(AttentionCausesAccess),
            Box::new(ConsciousUnconsciousOpposed),
            Box::new(IntegrationProducesStructure),
        ]
    }
}
