use pr4xis::category::Functor;

use super::ontology::*;
use crate::formal::systems::control::*;

// Diagnostics → Control functor.
//
// Proof that fault detection and isolation IS control theory (Gertler 1998).
// The residual r(t) = y(t) - ŷ(t) IS the error signal in the control loop.
// The diagnostic process IS the controller applied to itself.
//
// The mapping:
//   Symptom     → Error (deviation from expected — the error signal)
//   Hypothesis  → Model (the hypothesized system model)
//   Test        → Sensor (measurement to discriminate)
//   Evidence    → Signal (information from measurement)
//   Diagnosis   → Controller (the confirmed diagnosis drives the fix)
//   Residual    → Error (actual minus expected — IS the control error)
//   FaultMode   → Disturbance (the fault IS a disturbance)
//   Severity    → Setpoint (how far from desired state)
//   Remedy      → Actuator (the corrective action)
//   TraceContext → FeedbackLoop (the diagnostic loop IS feedback)

pub struct DiagnosticsToControl;

impl Functor for DiagnosticsToControl {
    type Source = DiagnosticCategory;
    type Target = ControlCategory;

    fn map_object(obj: &DiagnosticConcept) -> ControlConcept {
        match obj {
            DiagnosticConcept::Symptom => ControlConcept::Error,
            DiagnosticConcept::Hypothesis => ControlConcept::Model,
            DiagnosticConcept::Test => ControlConcept::Sensor,
            DiagnosticConcept::Evidence => ControlConcept::Signal,
            DiagnosticConcept::Diagnosis => ControlConcept::Controller,
            DiagnosticConcept::Residual => ControlConcept::Error,
            DiagnosticConcept::FaultMode => ControlConcept::Disturbance,
            DiagnosticConcept::Severity => ControlConcept::Setpoint,
            DiagnosticConcept::Remedy => ControlConcept::Actuator,
            DiagnosticConcept::TraceContext => ControlConcept::FeedbackLoop,
        }
    }

    fn map_morphism(m: &DiagnosticRelation) -> ControlRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = match m.kind {
            DiagnosticRelationKind::Identity => ControlRelationKind::Identity,
            DiagnosticRelationKind::Triggers => ControlRelationKind::Carries,
            DiagnosticRelationKind::Generates => ControlRelationKind::Represents,
            DiagnosticRelationKind::Requires => ControlRelationKind::Measures,
            DiagnosticRelationKind::Produces => ControlRelationKind::Carries,
            DiagnosticRelationKind::Updates => ControlRelationKind::Carries,
            DiagnosticRelationKind::Confirms => ControlRelationKind::ComputesFrom,
            DiagnosticRelationKind::Identifies => ControlRelationKind::Perturbs,
            DiagnosticRelationKind::HasSeverity => ControlRelationKind::ComparedWith,
            DiagnosticRelationKind::Prescribes => ControlRelationKind::ActsOn,
            DiagnosticRelationKind::Contextualizes => ControlRelationKind::Closes,
            DiagnosticRelationKind::Composed => ControlRelationKind::Composed,
        };
        ControlRelation { from, to, kind }
    }
}
pr4xis::register_functor!(DiagnosticsToControl);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws() {
        check_functor_laws::<DiagnosticsToControl>().unwrap();
    }

    #[test]
    fn residual_is_error() {
        // Gertler FDI: residual r(t) = y(t) - ŷ(t) IS the error signal
        assert_eq!(
            DiagnosticsToControl::map_object(&DiagnosticConcept::Residual),
            ControlConcept::Error
        );
    }

    #[test]
    fn remedy_is_actuator() {
        assert_eq!(
            DiagnosticsToControl::map_object(&DiagnosticConcept::Remedy),
            ControlConcept::Actuator
        );
    }
}
