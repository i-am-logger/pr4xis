use pr4xis::category::Functor;

use super::ontology::*;
use crate::cognitive::cognition::metacognition::*;

// Diagnostics → Metacognition functor.
//
// Proof that diagnosis IS metacognition (Reiter 1987 + MAPE-K).
// The diagnostic cycle (Observation → Hypothesis → Test → Conclusion)
// IS the metacognitive cycle (Monitor → Evaluate → Control → Repair).
//
// The mapping:
//   Symptom     → Gap (something is wrong — metacognitive gap detected)
//   Hypothesis  → Evaluation (what might be wrong)
//   Test        → Control (decide what to check)
//   Evidence    → Monitoring (observed data)
//   Diagnosis   → EpistemicAssessment (confirmed understanding)
//   Residual    → Trace (the deviation signal IS the trace)
//   FaultMode   → Gap (specific type of gap)
//   Severity    → Evaluation (how serious)
//   Remedy      → Repair (the fix)
//   TraceContext → Trace (observability context)

pub struct DiagnosticsToMetacognition;

impl Functor for DiagnosticsToMetacognition {
    type Source = DiagnosticCategory;
    type Target = MetaCognitionCategory;

    fn map_object(obj: &DiagnosticConcept) -> MetaCognitionConcept {
        match obj {
            DiagnosticConcept::Symptom => MetaCognitionConcept::Gap,
            DiagnosticConcept::Hypothesis => MetaCognitionConcept::Evaluation,
            DiagnosticConcept::Test => MetaCognitionConcept::Control,
            DiagnosticConcept::Evidence => MetaCognitionConcept::Monitoring,
            DiagnosticConcept::Diagnosis => MetaCognitionConcept::EpistemicAssessment,
            DiagnosticConcept::Residual => MetaCognitionConcept::Trace,
            DiagnosticConcept::FaultMode => MetaCognitionConcept::Gap,
            DiagnosticConcept::Severity => MetaCognitionConcept::Evaluation,
            DiagnosticConcept::Remedy => MetaCognitionConcept::Repair,
            DiagnosticConcept::TraceContext => MetaCognitionConcept::Trace,
        }
    }

    fn map_morphism(m: &DiagnosticRelation) -> MetaCognitionRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = match m.kind {
            DiagnosticRelationKind::Identity => MetaCognitionRelationKind::Identity,
            DiagnosticRelationKind::Triggers => MetaCognitionRelationKind::Detects,
            DiagnosticRelationKind::Generates => MetaCognitionRelationKind::Detects,
            DiagnosticRelationKind::Requires => MetaCognitionRelationKind::Decides,
            DiagnosticRelationKind::Produces => MetaCognitionRelationKind::Observes,
            DiagnosticRelationKind::Updates => MetaCognitionRelationKind::Observes,
            DiagnosticRelationKind::Confirms => MetaCognitionRelationKind::Assesses,
            DiagnosticRelationKind::Identifies => MetaCognitionRelationKind::Detects,
            DiagnosticRelationKind::HasSeverity => MetaCognitionRelationKind::Assesses,
            DiagnosticRelationKind::Prescribes => MetaCognitionRelationKind::Triggers,
            DiagnosticRelationKind::Contextualizes => MetaCognitionRelationKind::Observes,
            DiagnosticRelationKind::Composed => MetaCognitionRelationKind::Composed,
        };
        MetaCognitionRelation { from, to, kind }
    }
}
pr4xis::register_functor!(DiagnosticsToMetacognition);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws() {
        check_functor_laws::<DiagnosticsToMetacognition>().unwrap();
    }

    #[test]
    fn symptom_is_gap() {
        assert_eq!(
            DiagnosticsToMetacognition::map_object(&DiagnosticConcept::Symptom),
            MetaCognitionConcept::Gap
        );
    }

    #[test]
    fn remedy_is_repair() {
        assert_eq!(
            DiagnosticsToMetacognition::map_object(&DiagnosticConcept::Remedy),
            MetaCognitionConcept::Repair
        );
    }

    #[test]
    fn diagnosis_is_epistemic_assessment() {
        assert_eq!(
            DiagnosticsToMetacognition::map_object(&DiagnosticConcept::Diagnosis),
            MetaCognitionConcept::EpistemicAssessment
        );
    }
}
