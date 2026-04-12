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

    fn map_object(obj: &DiagnosticConcept) -> MetaConcept {
        match obj {
            DiagnosticConcept::Symptom => MetaConcept::Gap,
            DiagnosticConcept::Hypothesis => MetaConcept::Evaluation,
            DiagnosticConcept::Test => MetaConcept::Control,
            DiagnosticConcept::Evidence => MetaConcept::Monitoring,
            DiagnosticConcept::Diagnosis => MetaConcept::EpistemicAssessment,
            DiagnosticConcept::Residual => MetaConcept::Trace,
            DiagnosticConcept::FaultMode => MetaConcept::Gap,
            DiagnosticConcept::Severity => MetaConcept::Evaluation,
            DiagnosticConcept::Remedy => MetaConcept::Repair,
            DiagnosticConcept::TraceContext => MetaConcept::Trace,
        }
    }

    fn map_morphism(m: &DiagnosticRelation) -> MetaRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = match m.kind {
            DiagnosticRelationKind::Identity => MetaRelationKind::Identity,
            DiagnosticRelationKind::Triggers => MetaRelationKind::Detects,
            DiagnosticRelationKind::Generates => MetaRelationKind::Detects,
            DiagnosticRelationKind::Requires => MetaRelationKind::Decides,
            DiagnosticRelationKind::Produces => MetaRelationKind::Observes,
            DiagnosticRelationKind::Updates => MetaRelationKind::Observes,
            DiagnosticRelationKind::Confirms => MetaRelationKind::Assesses,
            DiagnosticRelationKind::Identifies => MetaRelationKind::Detects,
            DiagnosticRelationKind::HasSeverity => MetaRelationKind::Assesses,
            DiagnosticRelationKind::Prescribes => MetaRelationKind::Triggers,
            DiagnosticRelationKind::Contextualizes => MetaRelationKind::Observes,
            DiagnosticRelationKind::Composed => MetaRelationKind::Composed,
        };
        MetaRelation { from, to, kind }
    }
}

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
            MetaConcept::Gap
        );
    }

    #[test]
    fn remedy_is_repair() {
        assert_eq!(
            DiagnosticsToMetacognition::map_object(&DiagnosticConcept::Remedy),
            MetaConcept::Repair
        );
    }

    #[test]
    fn diagnosis_is_epistemic_assessment() {
        assert_eq!(
            DiagnosticsToMetacognition::map_object(&DiagnosticConcept::Diagnosis),
            MetaConcept::EpistemicAssessment
        );
    }
}
