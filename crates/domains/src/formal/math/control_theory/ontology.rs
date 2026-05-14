//! Classical (frequency-domain) control-theory ontology — plant,
//! controller, sensor, actuator, reference, error, feedback — plus the
//! three core axioms: negative feedback stabilises, integral action
//! drives steady-state error to zero, and BIBO stability is decidable
//! from pole location.
//!
//! # Literature
//!
//! - **Åström & Murray (2008)** *Feedback Systems: An Introduction
//!   for Scientists and Engineers*, Princeton University Press — the
//!   modern textbook treatment of negative feedback, PID control, and
//!   stability margins.
//! - **Ogata (2010)** *Modern Control Engineering*, 5th ed. —
//!   classical frequency-domain control, root locus, BIBO stability
//!   via pole location.
//! - **Lyapunov (1892)** *The General Problem of the Stability of
//!   Motion* (translated, Taylor & Francis 1992) — Lyapunov stability;
//!   BIBO stability for LTI systems is characterised by the location
//!   of the characteristic equation's roots.

use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::control_theory::feedback;
use crate::formal::math::control_theory::pid::{PidController, PidGains};
use crate::formal::math::control_theory::stability;

pr4xis::ontology! {
    name: "ControlTheory",
    source: "Astrom & Murray (2008) Feedback Systems, Princeton University Press; Ogata (2010) Modern Control Engineering, 5th ed.; Lyapunov (1892) The General Problem of the Stability of Motion",

    concepts: [
        Plant,
        Controller,
        Sensor,
        Actuator,
        Reference,
        Error,
        Feedback,
    ],

    labels: {
        Plant: ("en", "Plant",
            "Astrom & Murray (2008) Ch. 1: the dynamical system being controlled, often written G(s) in the frequency domain."),
        Controller: ("en", "Controller",
            "Astrom & Murray (2008) Ch. 1: the algorithmic component C(s) that generates a control signal from the error signal."),
        Sensor: ("en", "Sensor",
            "Astrom & Murray (2008) §1.5: the component measuring the plant's output and producing the measurement signal y."),
        Actuator: ("en", "Actuator",
            "Astrom & Murray (2008) §1.5: the component delivering the controller's command signal to the plant."),
        Reference: ("en", "Reference",
            "Astrom & Murray (2008) §1.4: the desired output (setpoint) r against which the plant output is regulated."),
        Error: ("en", "Error",
            "Astrom & Murray (2008) §1.4: the signal e = r - y, the difference between reference and measured output that drives the controller."),
        Feedback: ("en", "Feedback",
            "Astrom & Murray (2008) §1.1: the return path that takes the plant's measured output back to the comparator with the reference; the defining structural feature of closed-loop control."),
    },

    has_a: [
        // Classical control-loop mereology (Astrom & Murray 2008 §1.5):
        // a closed control loop is built out of these components.
        (Feedback, Plant),
        (Feedback, Controller),
        (Feedback, Sensor),
        (Feedback, Actuator),
        (Feedback, Reference),
        (Feedback, Error),
    ],

    opposes: [
        // Reference / Error are the two ends of the comparator: the
        // error is the orthogonal complement of "tracking" the reference.
        (Reference, Error),
        (Error, Reference),
    ],
}

/// Quality: short symbolic description of each control-theory concept,
/// matching the textbook glossary in Åström & Murray (2008) Ch. 1.
#[derive(Debug, Clone)]
pub struct ConceptDescription;

impl Quality for ConceptDescription {
    type Individual = ControlTheoryConcept;
    type Value = &'static str;

    fn get(&self, c: &ControlTheoryConcept) -> Option<&'static str> {
        use ControlTheoryConcept as C;
        Some(match c {
            C::Plant => "the system being controlled, G(s)",
            C::Controller => "generates control signal from error, C(s)",
            C::Sensor => "measures plant output for feedback",
            C::Actuator => "applies control signal to plant",
            C::Reference => "desired output value (setpoint)",
            C::Error => "difference between reference and measured output: e = r - y",
            C::Feedback => "path from output back to input for closed-loop control",
        })
    }
}

impl Ontology for ControlTheoryOntology {
    type Cat = ControlTheoryCategory;
    type Qual = ConceptDescription;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(NegativeFeedbackStabilizes));
        axioms.push(Box::new(ErrorConvergesToZero));
        axioms.push(Box::new(BIBOStabilityDefinition));
        axioms
    }
}

// ---------------------------------------------------------------------------
// Domain axioms.
// ---------------------------------------------------------------------------

/// Negative-feedback gain reduction: for positive open-loop gain G
/// and feedback gain H, the closed-loop gain |G/(1 + GH)| is strictly
/// less than the open-loop |G|. Åström & Murray (2008) §1.2 — the
/// "feedback reduces sensitivity" principle.
pub struct NegativeFeedbackStabilizes;

impl Axiom for NegativeFeedbackStabilizes {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let test_cases = [
            (1.0, 1.0),
            (10.0, 0.5),
            (100.0, 0.1),
            (5.0, 2.0),
            (0.5, 1.0),
        ];
        for &(g, h) in &test_cases {
            let cl = feedback::closed_loop_gain(g, h);
            if cl.abs() >= g.abs() + 1e-10 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            let s = feedback::sensitivity(g, h);
            if s >= 1.0 + 1e-10 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "NegativeFeedbackStabilizes",
        "negative feedback reduces gain: |G/(1+GH)| < |G| for GH > 0",
        "Astrom & Murray (2008) Feedback Systems §1.2"
    );
}

pr4xis::register_axiom!(
    NegativeFeedbackStabilizes,
    "Astrom & Murray (2008) Feedback Systems §1.2"
);

/// Integral-action zero steady-state error: a closed-loop with a PI
/// (or PID) controller driving a stable plant has its steady-state
/// tracking error converge to zero for a step reference. Follows from
/// the Final Value Theorem applied to the closed-loop transfer
/// function with a pole at the origin. Åström & Murray (2008) §11.1.
pub struct ErrorConvergesToZero;

impl Axiom for ErrorConvergesToZero {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let gains = PidGains::pi(1.0, 2.0);
        let dt = 0.01;
        let mut pid = PidController::new(gains, dt);
        let reference = 1.0;
        let mut output = 0.0;
        for _ in 0..5000 {
            let error = feedback::error_signal(reference, output);
            let control = pid.update(error);
            output = 0.95 * output + 0.05 * control;
        }
        if (reference - output).abs() < 0.01 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ErrorConvergesToZero",
        "stable closed-loop with integral action has zero steady-state error for a step",
        "Astrom & Murray (2008) Feedback Systems §11.1 (Final Value Theorem applied to PI control)"
    );
}

pr4xis::register_axiom!(
    ErrorConvergesToZero,
    "Astrom & Murray (2008) Feedback Systems §11.1 (Final Value Theorem applied to PI control)"
);

/// BIBO-stability characterisation: a LTI system is bounded-input
/// bounded-output stable iff every pole of its transfer function lies
/// strictly in the open left half-plane (negative real part). Ogata
/// (2010) §5.3; equivalent to Lyapunov stability for LTI systems.
pub struct BIBOStabilityDefinition;

impl Axiom for BIBOStabilityDefinition {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};

        let stable_poles = vec![vec![-1.0, -2.0], vec![-0.5, -0.1, -3.0], vec![-10.0]];
        for poles in &stable_poles {
            if !stability::is_bibo_stable(poles) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            if stability::classify_stability(poles)
                != stability::StabilityClass::AsymptoticallyStable
            {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }

        let unstable_poles = vec![vec![1.0, -2.0], vec![0.5], vec![-1.0, 3.0]];
        for poles in &unstable_poles {
            if stability::is_bibo_stable(poles) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            if stability::classify_stability(poles) != stability::StabilityClass::Unstable {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }

        let marginal_poles = vec![vec![0.0, -1.0], vec![0.0]];
        for poles in &marginal_poles {
            if stability::is_bibo_stable(poles) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            if stability::classify_stability(poles) != stability::StabilityClass::MarginallyStable {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "BIBOStabilityDefinition",
        "LTI BIBO stability iff every pole has negative real part",
        "Ogata (2010) Modern Control Engineering, 5th ed. §5.3"
    );
}

pr4xis::register_axiom!(
    BIBOStabilityDefinition,
    "Ogata (2010) Modern Control Engineering, 5th ed. §5.3"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Concept;
    use pr4xis::category::laws::assert_category_laws;
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<ControlTheoryCategory>();
    }

    #[test]
    fn ontology_validates() {
        ControlTheoryOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn seven_control_concepts() {
        assert_eq!(ControlTheoryConcept::variants().len(), 7);
    }

    #[test]
    fn concept_description_total() {
        let q = ConceptDescription;
        for c in ControlTheoryConcept::variants() {
            assert!(q.get(&c).is_some(), "{:?} missing description", c);
        }
    }

    #[test]
    fn negative_feedback_stabilizes_holds() {
        assert!(NegativeFeedbackStabilizes.verify().is_ok());
    }

    #[test]
    fn error_converges_to_zero_holds() {
        assert!(ErrorConvergesToZero.verify().is_ok());
    }

    #[test]
    fn bibo_stability_definition_holds() {
        assert!(BIBOStabilityDefinition.verify().is_ok());
    }

    fn arb_concept() -> impl Strategy<Value = ControlTheoryConcept> {
        proptest::sample::select(ControlTheoryConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_concept_description_total(c in arb_concept()) {
            prop_assert!(ConceptDescription.get(&c).is_some());
        }

        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            use pr4xis::category::{Arrow, Category};
            for m in ControlTheoryCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in ControlTheoryOntology::axioms() {
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
}
