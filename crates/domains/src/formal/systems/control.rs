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
    ],

    composed: [
        (Controller, Plant),
        (Sensor, Controller),
        (Setpoint, Controller),
        (Disturbance, Error),
    ],
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

impl Ontology for ControlOntology {
    type Cat = ControlCategory;
    type Qual = OnFeedbackLoop;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>()
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
    fn ten_concepts() {
        assert_eq!(ControlConcept::variants().len(), 10);
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
