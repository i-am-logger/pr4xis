//! Speech Act Planning — plan-based theory of speech acts.
//!
//! Speech acts are operators with epistemic preconditions and effects
//! on the hearer's mental state. Planning selects which speech acts to
//! perform based on communicative goals and the current epistemic state.
//!
//! Cohen & Perrault (1979): speech acts as STRIPS-like plan operators.
//! Appelt (1985): KAMP planner — intensional logic of knowledge and action.
//! Stalnaker (2002): assertion updates the Common Ground.
//! Bratman (1987): BDI — Belief + Desire + Intention → Plan.
//! Jakobson (1960): phatic function — social ritual, not information.
//!
//! This ontology bridges Epistemics → Response:
//!   Epistemics (what I know) → Planning (what to say) → Response (how to say it)
//!
//! Source: Cohen & Perrault (1979); Appelt (1985);
//!         Stalnaker (2002); Bratman (1987); Jakobson (1960)

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Category;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Planning",
    source: "Cohen & Perrault (1979); Appelt (1985); Stalnaker (2002); Bratman (1987); Jakobson (1960)",

    concepts: [
        // BDI architecture (Bratman 1987)
        Belief,
        Desire,
        Intention,
        // Plan-based speech acts (Cohen & Perrault 1979)
        SpeechActOperator,
        Precondition,
        Effect,
        Plan,
        // Common Ground (Stalnaker 2002)
        CommonGround,
        CommonGroundUpdate,
        // Communicative functions (Jakobson 1960)
        CommunicativeGoal,
        InformativeGoal,
        PhaticGoal,
        DirectiveGoal,
        ExpressiveGoal,
    ],

    labels: {
        Belief: ("en", "Belief", "What the speaker believes about the world and the hearer. Maps from Epistemics."),
        Desire: ("en", "Desire", "What the speaker wants to achieve in the hearer's mind."),
        Intention: ("en", "Intention", "Commitment to a specific plan of speech acts."),
        SpeechActOperator: ("en", "Speech act operator", "A speech act as a plan operator: preconditions + effects (Cohen & Perrault 1979)."),
        Precondition: ("en", "Precondition", "What must hold for the speech act to be appropriate."),
        Effect: ("en", "Effect", "What changes after the speech act is performed."),
        Plan: ("en", "Plan", "A sequence of speech acts achieving a communicative goal."),
        CommonGround: ("en", "Common ground", "The set of mutually accepted propositions (Stalnaker 2002)."),
        CommonGroundUpdate: ("en", "Common ground update", "How an assertion changes the common ground."),
        CommunicativeGoal: ("en", "Communicative goal", "The goal the utterance serves (Jakobson function)."),
        InformativeGoal: ("en", "Informative goal", "Transfer knowledge (referential function)."),
        PhaticGoal: ("en", "Phatic goal", "Maintain social contact. 'How are you?' — not seeking information."),
        DirectiveGoal: ("en", "Directive goal", "Get hearer to do something (conative function)."),
        ExpressiveGoal: ("en", "Expressive goal", "Express speaker's attitude (emotive function)."),
    },

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
}

/// The theoretical tradition a planning concept is drawn from.
///
/// A closed classification of the four literatures this ontology composes:
/// Bratman's BDI architecture, Cohen & Perrault's plan-based speech acts,
/// Stalnaker's Common Ground, and Jakobson's communicative functions.
///
/// Source: Cohen & Perrault (1979); Bratman (1987); Stalnaker (2002);
///         Jakobson (1960)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConceptTradition {
    /// Belief–Desire–Intention practical reasoning (Bratman 1987).
    Bdi,
    /// Plan-based theory of speech acts (Cohen & Perrault 1979).
    Planning,
    /// Common Ground / assertion update (Stalnaker 2002).
    Stalnaker,
    /// Communicative functions of language (Jakobson 1960).
    Jakobson,
}

/// Quality: the theoretical [`ConceptTradition`] each planning concept is drawn from.
#[derive(Debug, Clone)]
pub struct ConceptRole;

impl Quality for ConceptRole {
    type Individual = PlanningConcept;
    type Value = ConceptTradition;

    fn get(&self, individual: &PlanningConcept) -> Option<ConceptTradition> {
        Some(match individual {
            PlanningConcept::Belief | PlanningConcept::Desire | PlanningConcept::Intention => {
                ConceptTradition::Bdi
            }
            PlanningConcept::SpeechActOperator
            | PlanningConcept::Precondition
            | PlanningConcept::Effect
            | PlanningConcept::Plan => ConceptTradition::Planning,
            PlanningConcept::CommonGround | PlanningConcept::CommonGroundUpdate => {
                ConceptTradition::Stalnaker
            }
            PlanningConcept::CommunicativeGoal
            | PlanningConcept::InformativeGoal
            | PlanningConcept::PhaticGoal
            | PlanningConcept::DirectiveGoal
            | PlanningConcept::ExpressiveGoal => ConceptTradition::Jakobson,
        })
    }
}

/// BDI: Belief + Desire produce Intention (Bratman 1987).
#[derive(Debug)]
pub struct BdiProducesIntention;

impl Axiom for BdiProducesIntention {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
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
        if belief_produces && desire_produces {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "BdiProducesIntention",
        "Belief and Desire both produce Intention (Bratman 1987 BDI)",
        "Bratman (1987) Intention, Plans, and Practical Reason"
    );
}
pr4xis::register_axiom!(
    BdiProducesIntention,
    "Bratman (1987) Intention, Plans, and Practical Reason"
);

/// Speech act effects update Common Ground (Stalnaker 2002).
#[derive(Debug)]
pub struct EffectUpdatesCommonGround;

impl Axiom for EffectUpdatesCommonGround {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let m = PlanningCategory::morphisms();
        if m.iter().any(|r| {
            r.from == PlanningConcept::Effect
                && r.to == PlanningConcept::CommonGroundUpdate
                && r.kind == PlanningRelationKind::Updates
        }) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "EffectUpdatesCommonGround",
        "Effect updates CommonGround via CommonGroundUpdate (Stalnaker 2002)",
        "Stalnaker (2002) Common Ground, Linguistics and Philosophy 25(5–6)"
    );
}
pr4xis::register_axiom!(
    EffectUpdatesCommonGround,
    "Stalnaker (2002) Common Ground, Linguistics and Philosophy 25(5–6)"
);

/// All Jakobson functions specialize CommunicativeGoal.
#[derive(Debug)]
pub struct GoalsSpecialize;

impl Axiom for GoalsSpecialize {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let m = PlanningCategory::morphisms();
        let goals = [
            PlanningConcept::InformativeGoal,
            PlanningConcept::PhaticGoal,
            PlanningConcept::DirectiveGoal,
            PlanningConcept::ExpressiveGoal,
        ];
        if goals.iter().all(|g| {
            m.iter().any(|r| {
                r.from == *g
                    && r.to == PlanningConcept::CommunicativeGoal
                    && r.kind == PlanningRelationKind::Specializes
            })
        }) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "GoalsSpecialize",
        "Informative, Phatic, Directive, Expressive specialize CommunicativeGoal (Jakobson 1960)",
        "Jakobson (1960) Linguistics and Poetics, in Style in Language"
    );
}
pr4xis::register_axiom!(
    GoalsSpecialize,
    "Jakobson (1960) Linguistics and Poetics, in Style in Language"
);

impl Ontology for PlanningOntology {
    type Cat = PlanningCategory;
    type Qual = ConceptRole;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(BdiProducesIntention));
        axioms.push(Box::new(EffectUpdatesCommonGround));
        axioms.push(Box::new(GoalsSpecialize));
        axioms
    }
}
