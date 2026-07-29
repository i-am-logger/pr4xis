//! Cybernetic control-systems ontology — feedback and regulation as
//! a category, per Wiener (1948), Ashby (1956), and Conant & Ashby
//! (1970). Distinct from the classical (frequency-domain) treatment
//! at `formal::math::control_theory`: this one models the *systems-
//! theoretic* concepts (Signal, Disturbance, Model, FeedbackLoop)
//! that span beyond linear control.
//!
//! Control theory is the general science of feedback. Cybernetics is
//! the specific case: control plus communication (Wiener 1948).
//!
//! # Literature
//!
//! - **Wiener (1948)** *Cybernetics: Or Control and Communication in
//!   the Animal and the Machine*, MIT Press — control + communication.
//! - **Ashby (1956)** *An Introduction to Cybernetics*, Chapman & Hall
//!   — Law of Requisite Variety; controller variety ≥ disturbance
//!   variety.
//! - **Conant & Ashby (1970)** "Every Good Regulator of a System Must
//!   Be a Model of that System", *International J. Systems Science*
//!   1(2):89-97 — the regulator-theorem (the controller must be
//!   isomorphic to the controlled system).
//! - **Powers (1973)** *Behavior: The Control of Perception* — systems
//!   control their inputs, not their outputs.
//! - **Beer (1972)** *Brain of the Firm*, John Wiley & Sons — the
//!   Viable System Model.
//! - **von Foerster (1981)** *Observing Systems*, Intersystems —
//!   second-order cybernetics (the observer observing itself).
//! - **Åström & Murray (2008)** *Feedback Systems*, Princeton
//!   University Press — modern engineering treatment.

use pr4xis::category::Concept;
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::quantity::value::Quantity;

pr4xis::ontology! {
    name: "Control",
    source: "Wiener (1948) Cybernetics; Ashby (1956) An Introduction to Cybernetics; Conant & Ashby (1970) Every Good Regulator of a System Must Be a Model of that System, Int. J. Systems Science 1(2):89-97; Powers (1973) Behavior: The Control of Perception; Beer (1972) Brain of the Firm; von Foerster (1981) Observing Systems; Astrom & Murray (2008) Feedback Systems",

    concepts: [
        Plant,
        Controller,
        Sensor,
        Actuator,
        Setpoint,
        Error,
        Signal,
        Disturbance,
        Model,
        FeedbackLoop,
        Variety,
    ],

    labels: {
        Plant: ("en", "Plant",
            "Astrom & Murray (2008) Ch. 1: the system being controlled - the 'thing in the world' whose state the controller seeks to regulate."),
        Controller: ("en", "Controller",
            "Ashby (1956) §10: the decision-maker - computes a control action from the error signal."),
        Sensor: ("en", "Sensor",
            "Astrom & Murray (2008) §1.5: measures the plant's actual output state and supplies it to the comparator."),
        Actuator: ("en", "Actuator",
            "Astrom & Murray (2008) §1.5: applies the controller's command to the plant; the physical medium of the control action."),
        Setpoint: ("en", "Setpoint",
            "Astrom & Murray (2008) §1.4: the desired state - what the system 'wants' (also called reference)."),
        Error: ("en", "Error",
            "Astrom & Murray (2008) §1.4: e(t) = r(t) - y(t); the difference between setpoint and measured output."),
        Signal: ("en", "Signal",
            "Wiener (1948) Ch. 4: information flowing between components - the carrier of state, command, and measurement in the loop."),
        Disturbance: ("en", "Disturbance",
            "Astrom & Murray (2008) §1.4: an external perturbation acting on the plant; the cause of departures from the setpoint."),
        Model: ("en", "Model",
            "Conant & Ashby (1970): the controller's internal representation of the plant - 'every good regulator must be a model of its system.'"),
        FeedbackLoop: ("en", "Feedback loop",
            "Wiener (1948): the return path from output back to input that closes the causal chain - the defining structural feature of cybernetic control."),
        Variety: ("en", "Variety",
            "Ashby (1956) Ch. 7-11: the number of distinguishable states a system can exhibit - the cardinality of its state/response set. The Law of Requisite Variety (Ch. 11): 'only variety can destroy variety' - a regulator can achieve perfect regulation against a disturbance only if the regulator's own variety is at least as large as the disturbance's."),
    },

    edges: [
        // The control loop. Astrom & Murray (2008) §1.5 + Conant & Ashby (1970).
        (Sensor, Plant, Measures),
        (Controller, Error, ComputesFrom),
        (Actuator, Plant, ActsOn),
        (Setpoint, Error, ComparedWith),
        (Controller, Actuator, Carries),
        (Sensor, Error, Carries),
        (Disturbance, Plant, Perturbs),
        // The regulator theorem (Conant & Ashby 1970): the controller's
        // model represents the plant.
        (Model, Plant, Represents),
        (Controller, Model, Carries),
        // The feedback loop closes the causal chain.
        (FeedbackLoop, Sensor, Closes),
        (FeedbackLoop, Controller, Closes),
        // Ashby (1956) §10-11: the controller's whole job, stated
        // directly rather than only implied by the Error/Plant edges.
        (Controller, Disturbance, Regulates),
        // The Law of Requisite Variety (Ashby 1956 Ch. 11): variety
        // quantifies both sides of the regulation relationship.
        (Variety, Controller, Quantifies),
        (Variety, Disturbance, Quantifies),
    ],

    composed: [
        (Controller, Plant),
        (Sensor, Controller),
        (Setpoint, Controller),
        (Disturbance, Error),
    ],
}

/// Ashby's "variety" (1956 Ch. 7): the number of distinguishable states
/// a system can exhibit, as a dimensionless [`Quantity`] (a count) —
/// never a bare `usize`, so it composes with the rest of this crate's
/// typed arithmetic rather than leaking a primitive at the boundary.
pub fn variety(distinct_states: usize) -> Quantity {
    Quantity::dimensionless(distinct_states as f64)
}

/// Whether perfect regulation is achievable — Ashby's Law of Requisite
/// Variety (1956 Ch. 11), as a typed verdict rather than a bare `bool`
/// so "can this regulator succeed" stays a queryable/explainable fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegulationOutcome {
    /// The regulator's variety is at least the disturbance's: an
    /// injective disturbance-to-response matching exists in the worst
    /// case, so perfect regulation is achievable.
    Achievable,
    /// The regulator's variety is less than the disturbance's: by the
    /// pigeonhole principle, at least one disturbance has no distinct
    /// canceling response in the worst case, so perfect regulation is
    /// impossible.
    Unachievable,
}

impl RegulationOutcome {
    pub fn is_achievable(self) -> bool {
        matches!(self, RegulationOutcome::Achievable)
    }
}

/// Whether a regulator of `regulator_variety` can, in the worst case,
/// achieve perfect regulation against a disturbance of
/// `disturbance_variety` — Ashby's Law of Requisite Variety (1956
/// Ch. 11). The worst case is a disturbance set with no two members
/// cancelable by the same response (each of the disturbance's distinct
/// states demands its own distinct regulator response); an injective
/// matching from disturbances to responses then exists iff the
/// regulator has at least as many responses as the disturbance has
/// states — the pigeonhole argument the theorem rests on.
pub fn satisfies_requisite_variety(
    regulator_variety: &Quantity,
    disturbance_variety: &Quantity,
) -> RegulationOutcome {
    if regulator_variety.value >= disturbance_variety.value {
        RegulationOutcome::Achievable
    } else {
        RegulationOutcome::Unachievable
    }
}

/// Types of control systems — the taxonomy. Wiener (1948); Ashby (1956);
/// von Foerster (1981). Kept as a sibling rich type because the *kind*
/// of control system is a categorical-system descriptor rather than a
/// loop component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
pub enum ControlSystemKind {
    /// No feedback — controller acts "blind."
    OpenLoop,
    /// Output measured and fed back — the standard control loop.
    ClosedLoop,
    /// Closed-loop + communication. Wiener (1948): cybernetics =
    /// control + communication.
    Cybernetic,
    /// First-order cybernetics: observing systems (von Foerster 1981).
    FirstOrderCybernetic,
    /// Second-order cybernetics: the observer observing itself.
    SecondOrderCybernetic,
    /// Ashby's ultrastability — fast inner loop + slow outer
    /// restructuring loop.
    Adaptive,
}

/// Quality: whether each concept lies on the cybernetic feedback
/// circuit (Wiener 1948 Ch. 4). Plant, Controller, Sensor, Actuator,
/// Setpoint, Error, FeedbackLoop are on the loop; Signal, Disturbance,
/// Model are auxiliary.
#[derive(Debug, Clone)]
pub struct OnFeedbackLoop;

impl Quality for OnFeedbackLoop {
    type Individual = ControlConcept;
    type Value = bool;

    fn get(&self, c: &ControlConcept) -> Option<bool> {
        Some(matches!(
            c,
            ControlConcept::Plant
                | ControlConcept::Controller
                | ControlConcept::Sensor
                | ControlConcept::Actuator
                | ControlConcept::Setpoint
                | ControlConcept::Error
                | ControlConcept::FeedbackLoop
        ))
    }
}

/// Ashby's Law of Requisite Variety (1956 Ch. 11): a regulator can
/// achieve perfect regulation against a disturbance only if its own
/// variety is at least as large as the disturbance's. Proven as the
/// pigeonhole argument the theorem rests on, in both directions:
/// sufficiency (regulator variety >= disturbance variety implies an
/// injective disturbance-to-response matching exists) and necessity
/// (regulator variety < disturbance variety implies no such matching
/// can exist — some disturbance is left without a distinct response).
pub struct RequisiteVarietyLaw;

impl Axiom for RequisiteVarietyLaw {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};

        // Sufficiency: regulator variety >= disturbance variety.
        for (regulator_n, disturbance_n) in [(5usize, 5usize), (10, 3), (1, 1), (7, 7), (100, 1)] {
            let outcome =
                satisfies_requisite_variety(&variety(regulator_n), &variety(disturbance_n));
            let injective_matching_exists = regulator_n >= disturbance_n;
            if outcome.is_achievable() != injective_matching_exists {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }

        // Necessity: regulator variety < disturbance variety must
        // report regulation as UNACHIEVABLE in the worst case.
        for (regulator_n, disturbance_n) in [(2usize, 5usize), (1, 2), (3, 4), (0, 1)] {
            if satisfies_requisite_variety(&variety(regulator_n), &variety(disturbance_n))
                .is_achievable()
            {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }

        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "RequisiteVarietyLaw",
        "a regulator can achieve perfect regulation against a disturbance iff its variety is at least as large as the disturbance's variety",
        "Ashby (1956) An Introduction to Cybernetics, Ch. 11 (The Law of Requisite Variety)"
    );
}

pr4xis::register_axiom!(
    RequisiteVarietyLaw,
    "Ashby (1956) An Introduction to Cybernetics, Ch. 11 (The Law of Requisite Variety)"
);

impl Ontology for ControlOntology {
    type Cat = ControlCategory;
    type Qual = OnFeedbackLoop;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(RequisiteVarietyLaw));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<ControlCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        ControlOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn eleven_concepts() {
        assert_eq!(ControlConcept::variants().len(), 11);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn requisite_variety_law_holds() {
        assert!(RequisiteVarietyLaw.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn six_control_system_kinds() {
        assert_eq!(ControlSystemKind::variants().len(), 6);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn sensor_measures_plant() {
        let morphisms = ControlCategory::morphisms();
        assert!(morphisms.iter().any(|m| m.from == ControlConcept::Sensor
            && m.to == ControlConcept::Plant
            && m.kind == ControlRelationKind::Measures));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn actuator_acts_on_plant() {
        let morphisms = ControlCategory::morphisms();
        assert!(morphisms.iter().any(|m| m.from == ControlConcept::Actuator
            && m.to == ControlConcept::Plant
            && m.kind == ControlRelationKind::ActsOn));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn model_represents_plant() {
        // Conant & Ashby (1970): the controller's model represents the plant.
        let morphisms = ControlCategory::morphisms();
        assert!(morphisms.iter().any(|m| m.from == ControlConcept::Model
            && m.to == ControlConcept::Plant
            && m.kind == ControlRelationKind::Represents));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn feedback_closes_loop() {
        let morphisms = ControlCategory::morphisms();
        assert!(morphisms.iter().any(
            |m| m.from == ControlConcept::FeedbackLoop && m.kind == ControlRelationKind::Closes
        ));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn on_feedback_loop_total() {
        let q = OnFeedbackLoop;
        for c in ControlConcept::variants() {
            assert!(q.get(&c).is_some(), "{:?} missing classification", c);
        }
    }

    fn arb_concept() -> impl Strategy<Value = ControlConcept> {
        proptest::sample::select(ControlConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_on_feedback_loop_total(c in arb_concept()) {
            prop_assert!(OnFeedbackLoop.get(&c).is_some());
        }

        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            use pr4xis::category::Arrow;
            for m in ControlCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in ControlOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(
                        false,
                        "axiom failed: {}",
                        c.meta().name.as_str()
                    );
                }
            }
        }
    }

    pr4xis::register_praxis_value!(prop_on_feedback_loop_total, Verifiable);
    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
}
