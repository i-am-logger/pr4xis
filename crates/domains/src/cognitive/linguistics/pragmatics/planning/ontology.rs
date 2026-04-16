// Speech Act Planning — plan-based theory of speech acts.
//
// Speech acts are operators with epistemic preconditions and effects
// on the hearer's mental state. Planning selects which speech acts to
// perform based on communicative goals and the current epistemic state.
//
// Cohen & Perrault (1979): speech acts as STRIPS-like plan operators.
// Appelt (1985): KAMP planner — intensional logic of knowledge and action.
// Stalnaker (2002): assertion updates the Common Ground.
// Bratman (1987): BDI — Belief + Desire + Intention → Plan.
// Jakobson (1960): phatic function — social ritual, not information.
//
// This ontology bridges Epistemics → Response:
//   Epistemics (what I know) → Planning (what to say) → Response (how to say it)
//
// Source: Cohen & Perrault (1979); Appelt (1985);
//         Stalnaker (2002); Bratman (1987); Jakobson (1960)

use pr4xis::category::{Category, Entity};
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Concepts in the Speech Act Planning ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum PlanningConcept {
    // === BDI architecture (Bratman 1987) ===
    /// What the speaker believes about the world and the hearer.
    /// Maps from Epistemics: KnownKnown, KnownUnknown, etc.
    Belief,
    /// What the speaker wants to achieve in the hearer's mind.
    /// "I want the hearer to know that dogs are mammals."
    Desire,
    /// Commitment to a specific plan of speech acts.
    Intention,

    // === Plan-based speech acts (Cohen & Perrault 1979) ===
    /// A speech act as a plan operator: preconditions + effects.
    /// Precondition: speaker believes p. Effect: hearer believes p.
    SpeechActOperator,
    /// What must hold for the speech act to be appropriate.
    /// Epistemic: speaker knows p. Social: speaker has authority.
    Precondition,
    /// What changes after the speech act is performed.
    /// Epistemic: hearer now believes p. Social: commitment created.
    Effect,
    /// A sequence of speech acts achieving a communicative goal.
    Plan,

    // === Common Ground (Stalnaker 2002) ===
    /// The set of mutually accepted propositions.
    CommonGround,
    /// How an assertion changes the common ground.
    /// Assertion: removes worlds where p is false.
    CommonGroundUpdate,

    // === Communicative functions (Jakobson 1960) ===
    /// The communicative goal — what function the utterance serves.
    CommunicativeGoal,
    /// Informative: transfer knowledge (referential function).
    InformativeGoal,
    /// Phatic: maintain social contact (phatic function).
    /// "How are you?" — not seeking information.
    PhaticGoal,
    /// Directive: get hearer to do something (conative function).
    DirectiveGoal,
    /// Expressive: express speaker's attitude (emotive function).
    ExpressiveGoal,
}

define_ontology! {
    /// Speech Act Planning — BDI + plan-based speech acts.
    pub PlanningOntology for PlanningCategory {
        concepts: PlanningConcept,
        relation: PlanningRelation,
        kind: PlanningRelationKind,
        kinds: [
            /// BDI: belief + desire produce intention.
            Produces,
            /// Planning: intention selects plan.
            Selects,
            /// Planning: plan consists of operators.
            ConsistsOf,
            /// Operator: has precondition.
            HasPrecondition,
            /// Operator: has effect.
            HasEffect,
            /// Stalnaker: speech act updates common ground.
            Updates,
            /// Goal: specialization.
            Specializes,
        ],
        edges: [
            // BDI: Belief + Desire → Intention → Plan
            (Belief, Intention, Produces),
            (Desire, Intention, Produces),
            (Intention, Plan, Selects),
            // Plan consists of SpeechActOperators
            (Plan, SpeechActOperator, ConsistsOf),
            // Operator has Precondition and Effect
            (SpeechActOperator, Precondition, HasPrecondition),
            (SpeechActOperator, Effect, HasEffect),
            // Effect updates CommonGround
            (Effect, CommonGroundUpdate, Updates),
            (CommonGroundUpdate, CommonGround, Updates),
            // Goal specializations (Jakobson functions)
            (InformativeGoal, CommunicativeGoal, Specializes),
            (PhaticGoal, CommunicativeGoal, Specializes),
            (DirectiveGoal, CommunicativeGoal, Specializes),
            (ExpressiveGoal, CommunicativeGoal, Specializes),
            // Goal drives Desire
            (CommunicativeGoal, Desire, Produces),
        ],
        composed: [
            // BDI chain: Belief → Plan
            (Belief, Plan),
            (Desire, Plan),
            // Goal → Plan (through Desire → Intention)
            (CommunicativeGoal, Plan),
            (CommunicativeGoal, Intention),
            // Plan → CommonGround (through Operator → Effect → Update)
            (Plan, CommonGroundUpdate),
            (Plan, CommonGround),
            (SpeechActOperator, CommonGround),
        ],

        being: Process,
        source: "Cohen & Perrault (1979); Appelt (1985); Stalnaker (2002); Bratman (1987); Jakobson (1960)",
    }
}

/// Whether a concept is BDI-structural vs goal-type vs planning.
#[derive(Debug, Clone)]
pub struct ConceptRole;

impl Quality for ConceptRole {
    type Individual = PlanningConcept;
    type Value = &'static str;

    fn get(&self, individual: &PlanningConcept) -> Option<&'static str> {
        Some(match individual {
            PlanningConcept::Belief | PlanningConcept::Desire | PlanningConcept::Intention => "BDI",
            PlanningConcept::SpeechActOperator
            | PlanningConcept::Precondition
            | PlanningConcept::Effect
            | PlanningConcept::Plan => "Planning",
            PlanningConcept::CommonGround | PlanningConcept::CommonGroundUpdate => "Stalnaker",
            PlanningConcept::CommunicativeGoal
            | PlanningConcept::InformativeGoal
            | PlanningConcept::PhaticGoal
            | PlanningConcept::DirectiveGoal
            | PlanningConcept::ExpressiveGoal => "Jakobson",
        })
    }
}

/// BDI: Belief + Desire produce Intention (Bratman 1987).
#[derive(Debug)]
pub struct BdiProducesIntention;

impl Axiom for BdiProducesIntention {
    fn description(&self) -> &str {
        "Belief and Desire both produce Intention (Bratman 1987 BDI)"
    }
    fn holds(&self) -> bool {
        let m = PlanningCategory::morphisms();
        let belief_produces = m.iter().any(|r| {
            r.from == PlanningConcept::Belief
                && r.to == PlanningConcept::Intention
                && r.kind == PlanningRelationKind::Produces
        });
        let desire_produces = m.iter().any(|r| {
            r.from == PlanningConcept::Desire
                && r.to == PlanningConcept::Intention
                && r.kind == PlanningRelationKind::Produces
        });
        belief_produces && desire_produces
    }
}

/// Speech act effects update Common Ground (Stalnaker 2002).
#[derive(Debug)]
pub struct EffectUpdatesCommonGround;

impl Axiom for EffectUpdatesCommonGround {
    fn description(&self) -> &str {
        "Effect updates CommonGround via CommonGroundUpdate (Stalnaker 2002)"
    }
    fn holds(&self) -> bool {
        let m = PlanningCategory::morphisms();
        m.iter().any(|r| {
            r.from == PlanningConcept::Effect
                && r.to == PlanningConcept::CommonGroundUpdate
                && r.kind == PlanningRelationKind::Updates
        })
    }
}

/// All Jakobson functions specialize CommunicativeGoal.
#[derive(Debug)]
pub struct GoalsSpecialize;

impl Axiom for GoalsSpecialize {
    fn description(&self) -> &str {
        "Informative, Phatic, Directive, Expressive specialize CommunicativeGoal (Jakobson 1960)"
    }
    fn holds(&self) -> bool {
        let m = PlanningCategory::morphisms();
        let goals = [
            PlanningConcept::InformativeGoal,
            PlanningConcept::PhaticGoal,
            PlanningConcept::DirectiveGoal,
            PlanningConcept::ExpressiveGoal,
        ];
        goals.iter().all(|g| {
            m.iter().any(|r| {
                r.from == *g
                    && r.to == PlanningConcept::CommunicativeGoal
                    && r.kind == PlanningRelationKind::Specializes
            })
        })
    }
}

impl Ontology for PlanningOntology {
    type Cat = PlanningCategory;
    type Qual = ConceptRole;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        PlanningOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(BdiProducesIntention),
            Box::new(EffectUpdatesCommonGround),
            Box::new(GoalsSpecialize),
        ]
    }
}
