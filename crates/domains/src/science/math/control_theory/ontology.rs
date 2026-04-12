use pr4xis::category::Entity;
use pr4xis::define_dense_category;
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::science::math::control_theory::feedback;
use crate::science::math::control_theory::pid::{PidController, PidGains};
use crate::science::math::control_theory::stability;

// ---------------------------------------------------------------------------
// Entity: control system concepts
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum ControlConcept {
    Plant,
    Controller,
    Sensor,
    Actuator,
    Reference,
    Error,
    Feedback,
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

define_dense_category! {
    /// Discrete category over control theory entities.
    pub ControlCategory {
        entity: ControlConcept,
        relation: ControlRelation,
    }
}

// ---------------------------------------------------------------------------
// Quality: concept descriptions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ConceptDescription;

impl Quality for ConceptDescription {
    type Individual = ControlConcept;
    type Value = &'static str;

    fn get(&self, c: &ControlConcept) -> Option<&'static str> {
        Some(match c {
            ControlConcept::Plant => "the system being controlled, G(s)",
            ControlConcept::Controller => "generates control signal from error, C(s)",
            ControlConcept::Sensor => "measures plant output for feedback",
            ControlConcept::Actuator => "applies control signal to plant",
            ControlConcept::Reference => "desired output value (setpoint)",
            ControlConcept::Error => "difference between reference and measured output: e = r - y",
            ControlConcept::Feedback => "path from output back to input for closed-loop control",
        })
    }
}

// ---------------------------------------------------------------------------
// Axioms — Åström & Murray (2008), Lyapunov (1892)
// ---------------------------------------------------------------------------

/// Negative feedback stabilizes: closed-loop gain is bounded when open-loop gain is finite.
pub struct NegativeFeedbackStabilizes;

impl Axiom for NegativeFeedbackStabilizes {
    fn description(&self) -> &str {
        "negative feedback reduces gain and stabilizes: |G/(1+GH)| < |G| for GH > 0"
    }

    fn holds(&self) -> bool {
        // For positive plant and feedback gains, closed-loop gain < open-loop gain
        let test_cases = [
            (1.0, 1.0),
            (10.0, 0.5),
            (100.0, 0.1),
            (5.0, 2.0),
            (0.5, 1.0),
        ];
        for &(g, h) in &test_cases {
            let cl = feedback::closed_loop_gain(g, h);
            // Closed-loop gain should be less than open-loop gain
            if cl.abs() >= g.abs() + 1e-10 {
                return false;
            }
            // Sensitivity should be < 1
            let s = feedback::sensitivity(g, h);
            if s >= 1.0 + 1e-10 {
                return false;
            }
        }
        true
    }
}

/// For a stable system with integral action, the steady-state error converges to zero.
///
/// A PI or PID controller with Ki > 0 drives the steady-state error to zero
/// for a step input (by the Final Value Theorem).
pub struct ErrorConvergesToZero;

impl Axiom for ErrorConvergesToZero {
    fn description(&self) -> &str {
        "stable system with integral action: steady-state error converges to zero"
    }

    fn holds(&self) -> bool {
        // Simulate a PI controller tracking a step reference
        let gains = PidGains::pi(1.0, 2.0);
        let dt = 0.01;
        let mut pid = PidController::new(gains, dt);

        let reference = 1.0;
        let mut output = 0.0;

        // Simple first-order plant: y[n+1] = 0.95*y[n] + 0.05*u[n]
        for _ in 0..5000 {
            let error = feedback::error_signal(reference, output);
            let control = pid.update(error);
            output = 0.95 * output + 0.05 * control;
        }

        // After convergence, error should be near zero
        let final_error = (reference - output).abs();
        final_error < 0.01
    }
}

/// BIBO stability definition: all poles with negative real parts means BIBO stable.
pub struct BIBOStabilityDefinition;

impl Axiom for BIBOStabilityDefinition {
    fn description(&self) -> &str {
        "BIBO stability: system is stable iff all poles have negative real parts"
    }

    fn holds(&self) -> bool {
        // Stable systems: all poles have Re < 0
        let stable_poles = vec![vec![-1.0, -2.0], vec![-0.5, -0.1, -3.0], vec![-10.0]];
        for poles in &stable_poles {
            if !stability::is_bibo_stable(poles) {
                return false;
            }
            if stability::classify_stability(poles)
                != stability::StabilityClass::AsymptoticallyStable
            {
                return false;
            }
        }

        // Unstable systems: at least one pole has Re > 0
        let unstable_poles = vec![vec![1.0, -2.0], vec![0.5], vec![-1.0, 3.0]];
        for poles in &unstable_poles {
            if stability::is_bibo_stable(poles) {
                return false;
            }
            if stability::classify_stability(poles) != stability::StabilityClass::Unstable {
                return false;
            }
        }

        // Marginally stable: some poles at Re = 0, none positive
        let marginal_poles = vec![vec![0.0, -1.0], vec![0.0]];
        for poles in &marginal_poles {
            if stability::is_bibo_stable(poles) {
                return false; // BIBO requires strictly negative
            }
            if stability::classify_stability(poles) != stability::StabilityClass::MarginallyStable {
                return false;
            }
        }

        true
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// The control theory ontology.
///
/// Founded on:
///   - Åström & Murray (2008). *Feedback Systems*. Princeton University Press.
///   - Ogata (2010). *Modern Control Engineering* (5th ed.).
///   - Lyapunov, A.M. (1892). "The General Problem of the Stability of Motion."
pub struct ControlTheoryOntology;

impl Ontology for ControlTheoryOntology {
    type Cat = ControlCategory;
    type Qual = ConceptDescription;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(NegativeFeedbackStabilizes),
            Box::new(ErrorConvergesToZero),
            Box::new(BIBOStabilityDefinition),
        ]
    }
}
