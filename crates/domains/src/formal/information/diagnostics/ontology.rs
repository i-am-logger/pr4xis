use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::{Ontology, Quality};

// Diagnostics ontology — the universal diagnostic cycle.
//
// Every domain of diagnosis follows the same pattern:
//   Observation → Hypothesis → Test → Conclusion
//
// Medical: Symptom → Differential → Lab test → Diagnosis
// Automotive (OBD): DTC → Fault mode → Drive cycle → Repair
// Control (FDI): Residual → Fault signature → Structured residual → Isolation
// Software: Trace → Bug hypothesis → Test case → Fix
// Self-reflection: Monitoring → Gap → Repair attempt → Resolution
//
// This ontology formalizes the pattern and connects to existing ontologies
// via functors (Metacognition, Control, PROV, Measurement).
//
// References:
// - Reiter, "A Theory of Diagnosis from First Principles" (1987, AI)
// - Gertler, "Fault Detection and Diagnosis in Engineering Systems" (1998)
// - ISO 13374:2003 — Condition monitoring (6-layer processing)
// - Kephart & Chess, "The Vision of Autonomic Computing" (2003, IEEE) — MAPE-K
// - Kalman, "On the General Theory of Control Systems" (1960) — observability
// - Conant & Ashby, "Every Good Regulator Must Be a Model" (1970)
// - Smith, "Reflection and Semantics in a Procedural Language" (1982, MIT)
// - Maes, "Computational Reflection" (1987, OOPSLA)

/// Concepts in the diagnostic cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum DiagnosticConcept {
    /// Observable deviation from expected behavior.
    /// ISO 13374: State Detection. Reiter (1987): OBS.
    /// FDI: residual r(t) ≠ 0. Medical: presenting symptom.
    Symptom,

    /// Candidate explanation for the symptom.
    /// Reiter (1987): a minimal set D such that SD ∪ D is consistent with OBS.
    /// Medical: differential diagnosis. FDI: fault signature.
    Hypothesis,

    /// Action to discriminate between hypotheses.
    /// FDI: structured residual test. Medical: lab test.
    /// Categorically: a morphism that maps hypotheses to evidence.
    Test,

    /// Result of a test — supports or refutes a hypothesis.
    /// Bayesian: likelihood ratio update. OBD: freeze frame data.
    Evidence,

    /// Confirmed explanation — the minimal consistent subset.
    /// Reiter (1987): diagnosis. Medical: ICD code. OBD: confirmed DTC.
    Diagnosis,

    /// Quantitative deviation signal: actual minus expected.
    /// Gertler/Isermann FDI: r(t) = y(t) - ŷ(t).
    /// Kalman: innovation sequence. Control: error signal.
    Residual,

    /// The specific way a component can fail.
    /// FMEA: failure mode. OBD: DTC category. ISO 13374: fault class.
    FaultMode,

    /// Impact classification of the diagnosed fault.
    /// FMEA: severity × occurrence × detection = RPN.
    /// ISO 13374: Health Assessment level.
    Severity,

    /// Prescribed corrective action.
    /// MAPE-K: Execute phase. Medical: treatment plan.
    /// Metacognition: Repair morphism.
    Remedy,

    /// The observability context linking diagnosis to its trace.
    /// OpenTelemetry: SpanContext. PROV: the Activity chain.
    /// Connects the diagnostic process to the data it analyzed.
    TraceContext,
}

define_ontology! {
    pub DiagnosticOntology for DiagnosticCategory {
        concepts: DiagnosticConcept,
        relation: DiagnosticRelation,
        kind: DiagnosticRelationKind,
        kinds: [
            /// Residual triggers Symptom detection (FDI: r(t) ≠ 0).
            Triggers,
            /// Symptom generates Hypothesis (abductive inference).
            Generates,
            /// Hypothesis requires Test (to discriminate).
            Requires,
            /// Test produces Evidence.
            Produces,
            /// Evidence supports or refutes Hypothesis (Bayesian update).
            Updates,
            /// Hypothesis confirmed as Diagnosis (Reiter: minimal consistent).
            Confirms,
            /// Diagnosis identifies FaultMode.
            Identifies,
            /// Diagnosis has Severity.
            HasSeverity,
            /// Diagnosis prescribes Remedy.
            Prescribes,
            /// TraceContext contextualizes Symptom (links to observability data).
            Contextualizes,
        ],
        edges: [
            // The diagnostic cycle: Observation → Hypothesis → Test → Conclusion
            (Residual, Symptom, Triggers),
            (Symptom, Hypothesis, Generates),
            (Hypothesis, Test, Requires),
            (Test, Evidence, Produces),
            (Evidence, Hypothesis, Updates),
            (Hypothesis, Diagnosis, Confirms),
            // Diagnosis outputs
            (Diagnosis, FaultMode, Identifies),
            (Diagnosis, Severity, HasSeverity),
            (Diagnosis, Remedy, Prescribes),
            // TraceContext links diagnosis to observability data
            (TraceContext, Symptom, Contextualizes),
            (TraceContext, Evidence, Contextualizes),
        ],
        composed: [
            // Full diagnostic chain
            (Residual, Diagnosis),
            (Symptom, Diagnosis),
            (Symptom, Remedy),
            (TraceContext, Diagnosis),
        ],
    }
}

/// Diagnostic status quality — the status associated with each concept.
#[derive(Debug, Clone)]
pub struct DiagnosticStatusQuality;

impl Quality for DiagnosticStatusQuality {
    type Individual = DiagnosticConcept;
    type Value = DiagnosticStatus;

    fn get(&self, individual: &DiagnosticConcept) -> Option<DiagnosticStatus> {
        match individual {
            DiagnosticConcept::Residual => Some(DiagnosticStatus::Healthy),
            DiagnosticConcept::Symptom => Some(DiagnosticStatus::Investigating),
            DiagnosticConcept::Hypothesis => Some(DiagnosticStatus::Investigating),
            DiagnosticConcept::Test => Some(DiagnosticStatus::Investigating),
            DiagnosticConcept::Evidence => Some(DiagnosticStatus::Investigating),
            DiagnosticConcept::Diagnosis => Some(DiagnosticStatus::Diagnosed),
            DiagnosticConcept::Remedy => Some(DiagnosticStatus::Remediated),
            _ => None,
        }
    }
}

impl Ontology for DiagnosticOntology {
    type Cat = DiagnosticCategory;
    type Qual = DiagnosticStatusQuality;

    fn structural_axioms() -> Vec<Box<dyn pr4xis::ontology::Axiom>> {
        Self::generated_structural_axioms()
    }
}

/// Observability level — can the system's state be reconstructed from output?
///
/// Kalman (1960): observability = full rank of [C; CA; CA²; ...; CA^(n-1)].
/// Applied to tracing: does the trace contain enough information?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservabilityLevel {
    /// All internal state reconstructible from output.
    /// Kalman: observability matrix has full rank.
    FullyObservable,
    /// Some state reconstructible, some hidden.
    /// Partial rank. Some events are unobservable.
    PartiallyObservable,
    /// State cannot be reconstructed from output.
    /// The trace is insufficient for diagnosis.
    Unobservable,
}

/// Diagnostic status — the current state of a diagnosis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticStatus {
    /// Normal operation — no symptoms detected.
    Healthy,
    /// Symptom detected — diagnosis in progress.
    Investigating,
    /// Diagnosis confirmed — remedy available.
    Diagnosed,
    /// Remedy applied — awaiting verification.
    Remediated,
    /// System in unknown state — insufficient observability.
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws_hold() {
        check_category_laws::<DiagnosticCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        DiagnosticOntology::validate().unwrap();
    }

    #[test]
    fn has_ten_concepts() {
        assert_eq!(DiagnosticConcept::variants().len(), 10);
    }

    // --- Reiter (1987): the diagnostic cycle ---

    #[test]
    fn symptom_generates_hypothesis() {
        let m = DiagnosticCategory::morphisms();
        assert!(m.iter().any(|r| r.from == DiagnosticConcept::Symptom
            && r.to == DiagnosticConcept::Hypothesis
            && r.kind == DiagnosticRelationKind::Generates));
    }

    #[test]
    fn hypothesis_requires_test() {
        let m = DiagnosticCategory::morphisms();
        assert!(m.iter().any(|r| r.from == DiagnosticConcept::Hypothesis
            && r.to == DiagnosticConcept::Test
            && r.kind == DiagnosticRelationKind::Requires));
    }

    #[test]
    fn test_produces_evidence() {
        let m = DiagnosticCategory::morphisms();
        assert!(m.iter().any(|r| r.from == DiagnosticConcept::Test
            && r.to == DiagnosticConcept::Evidence
            && r.kind == DiagnosticRelationKind::Produces));
    }

    #[test]
    fn evidence_updates_hypothesis() {
        let m = DiagnosticCategory::morphisms();
        assert!(m.iter().any(|r| r.from == DiagnosticConcept::Evidence
            && r.to == DiagnosticConcept::Hypothesis
            && r.kind == DiagnosticRelationKind::Updates));
    }

    #[test]
    fn hypothesis_confirms_diagnosis() {
        let m = DiagnosticCategory::morphisms();
        assert!(m.iter().any(|r| r.from == DiagnosticConcept::Hypothesis
            && r.to == DiagnosticConcept::Diagnosis
            && r.kind == DiagnosticRelationKind::Confirms));
    }

    // --- Gertler FDI: Residual triggers Symptom ---

    #[test]
    fn residual_triggers_symptom() {
        let m = DiagnosticCategory::morphisms();
        assert!(m.iter().any(|r| r.from == DiagnosticConcept::Residual
            && r.to == DiagnosticConcept::Symptom
            && r.kind == DiagnosticRelationKind::Triggers));
    }

    // --- MAPE-K: Diagnosis prescribes Remedy ---

    #[test]
    fn diagnosis_prescribes_remedy() {
        let m = DiagnosticCategory::morphisms();
        assert!(m.iter().any(|r| r.from == DiagnosticConcept::Diagnosis
            && r.to == DiagnosticConcept::Remedy
            && r.kind == DiagnosticRelationKind::Prescribes));
    }

    // --- Full chain: Symptom reaches Remedy ---

    #[test]
    fn symptom_reaches_remedy() {
        let m = DiagnosticCategory::morphisms();
        assert!(
            m.iter()
                .any(|r| r.from == DiagnosticConcept::Symptom && r.to == DiagnosticConcept::Remedy)
        );
    }

    // --- OpenTelemetry: TraceContext contextualizes ---

    #[test]
    fn trace_context_contextualizes_symptom() {
        let m = DiagnosticCategory::morphisms();
        assert!(m.iter().any(|r| r.from == DiagnosticConcept::TraceContext
            && r.to == DiagnosticConcept::Symptom
            && r.kind == DiagnosticRelationKind::Contextualizes));
    }

    #[test]
    fn trace_context_reaches_diagnosis() {
        let m = DiagnosticCategory::morphisms();
        assert!(
            m.iter().any(|r| r.from == DiagnosticConcept::TraceContext
                && r.to == DiagnosticConcept::Diagnosis)
        );
    }

    // --- Kalman: observability levels ---

    #[test]
    fn observability_levels_exist() {
        assert_ne!(
            ObservabilityLevel::FullyObservable,
            ObservabilityLevel::Unobservable
        );
        assert_ne!(
            ObservabilityLevel::PartiallyObservable,
            ObservabilityLevel::Unobservable
        );
    }

    // --- Diagnostic feedback loop: Evidence → Hypothesis (Bayesian update) ---

    #[test]
    fn diagnostic_feedback_loop() {
        // Evidence updates Hypothesis, and Hypothesis requires more Tests
        // This forms a loop: Hypothesis → Test → Evidence → Hypothesis
        let m = DiagnosticCategory::morphisms();
        assert!(m.iter().any(|r| r.from == DiagnosticConcept::Hypothesis
            && r.to == DiagnosticConcept::Test));
        assert!(
            m.iter()
                .any(|r| r.from == DiagnosticConcept::Test && r.to == DiagnosticConcept::Evidence)
        );
        assert!(m.iter().any(
            |r| r.from == DiagnosticConcept::Evidence && r.to == DiagnosticConcept::Hypothesis
        ));
    }
}
