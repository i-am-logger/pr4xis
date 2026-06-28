//! Diagnostics — the universal diagnostic cycle.
//!
//! Every diagnostic domain follows the same pattern: Observation →
//! Hypothesis → Test → Conclusion. The category's morphisms encode the
//! Reiter (1987) abductive loop, the Gertler (1998) FDI residual input,
//! the MAPE-K (Kephart & Chess 2003) remedy output, and the
//! observability-data link via TraceContext.
//!
//! # Literature
//!
//! - **Reiter (1987)** "A Theory of Diagnosis from First Principles",
//!   *Artificial Intelligence* 32(1):57-95 — diagnosis as minimal
//!   consistent subset; OBS / SD / D.
//! - **Gertler (1998)** *Fault Detection and Diagnosis in Engineering
//!   Systems*, Marcel Dekker — residuals r(t) = y(t) − ŷ(t); structured
//!   residuals for isolation.
//! - **ISO 13374:2003** *Condition monitoring and diagnostics of
//!   machines — Data processing, communication and presentation*, ISO —
//!   six-layer processing (data acquisition, state detection, health
//!   assessment, prognostic assessment, advisory generation).
//! - **Kephart & Chess (2003)** "The Vision of Autonomic Computing",
//!   *IEEE Computer* 36(1):41-50 — MAPE-K (Monitor / Analyze / Plan /
//!   Execute / Knowledge).
//! - **Kalman (1960)** "On the General Theory of Control Systems",
//!   *IFAC World Congress* — observability rank criterion.
//! - **Conant & Ashby (1970)** "Every Good Regulator of a System Must
//!   Be a Model of That System", *International Journal of Systems
//!   Science* 1(2):89-97.
//! - **Smith (1982)** *Reflection and Semantics in a Procedural Language*
//!   (PhD thesis, MIT) — computational reflection.
//! - **Maes (1987)** "Computational Reflection", *OOPSLA* — reflective
//!   diagnosis as repair morphism.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Diagnostic",
    source: "Reiter (1987) A Theory of Diagnosis from First Principles, AI 32(1):57-95; Gertler (1998) Fault Detection and Diagnosis in Engineering Systems; ISO 13374:2003 Condition monitoring and diagnostics of machines; Kephart & Chess (2003) The Vision of Autonomic Computing, IEEE Computer 36(1):41-50; Kalman (1960) On the General Theory of Control Systems; Conant & Ashby (1970) Every Good Regulator of a System Must Be a Model of That System, Int. J. Systems Science 1(2):89-97",

    concepts: [
        Symptom,
        Hypothesis,
        Test,
        Evidence,
        Diagnosis,
        Residual,
        FaultMode,
        Severity,
        Remedy,
        TraceContext,
    ],

    labels: {
        Symptom: ("en", "Symptom",
            "Reiter (1987) OBS; ISO 13374:2003 State Detection - observable deviation from expected behaviour."),
        Hypothesis: ("en", "Hypothesis",
            "Reiter (1987): a candidate minimal set D such that SD ∪ D is consistent with OBS."),
        Test: ("en", "Test",
            "Gertler (1998) §3: action to discriminate between hypotheses - a structured residual or medical lab test."),
        Evidence: ("en", "Evidence",
            "Bayesian: likelihood-ratio update over hypotheses produced by a test."),
        Diagnosis: ("en", "Diagnosis",
            "Reiter (1987): the confirmed minimal consistent subset - in medicine an ICD code, in OBD a confirmed DTC."),
        Residual: ("en", "Residual",
            "Gertler (1998) r(t) = y(t) - y_hat(t); Kalman innovation sequence - quantitative deviation signal."),
        FaultMode: ("en", "Fault mode",
            "FMEA: the specific way a component can fail; ISO 13374:2003 fault class."),
        Severity: ("en", "Severity",
            "FMEA: severity x occurrence x detection = RPN; ISO 13374:2003 Health Assessment level."),
        Remedy: ("en", "Remedy",
            "Kephart & Chess (2003) MAPE-K Execute phase - prescribed corrective action."),
        TraceContext: ("en", "Trace context",
            "OpenTelemetry SpanContext; W3C PROV-O Activity chain - links the diagnostic process to its observability data."),
    },

    edges: [
        // Gertler (1998) FDI: residual triggers symptom detection.
        (Residual, Symptom, Triggers),
        // Reiter (1987) abductive step: symptom generates hypothesis.
        (Symptom, Hypothesis, Generates),
        // Reiter (1987): hypothesis requires test for discrimination.
        (Hypothesis, Test, Requires),
        // Test produces evidence.
        (Test, Evidence, Produces),
        // Bayesian loop: evidence updates hypothesis.
        (Evidence, Hypothesis, Updates),
        // Reiter (1987): hypothesis confirmed as diagnosis.
        (Hypothesis, Diagnosis, Confirms),
        // Diagnosis outputs.
        (Diagnosis, FaultMode, Identifies),
        (Diagnosis, Severity, HasSeverity),
        // MAPE-K (Kephart & Chess 2003): diagnosis prescribes remedy.
        (Diagnosis, Remedy, Prescribes),
        // OpenTelemetry / PROV: trace context contextualizes observations.
        (TraceContext, Symptom, Contextualizes),
        (TraceContext, Evidence, Contextualizes),
    ],
}

/// Observability level — Kalman (1960) applied to trace completeness.
/// Can the system's state be reconstructed from output?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservabilityLevel {
    /// All internal state reconstructible from output. Kalman rank
    /// criterion holds: [C; CA; CA²; ...; CA^(n-1)] has full rank.
    FullyObservable,
    /// Some state reconstructible, some hidden. Partial rank.
    PartiallyObservable,
    /// State cannot be reconstructed. Trace insufficient for diagnosis.
    Unobservable,
}

/// Diagnostic status — the current state of a diagnosis. ISO 13374:2003
/// Health Assessment level adapted to the abductive workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticStatus {
    Healthy,
    Investigating,
    Diagnosed,
    Remediated,
    Unknown,
}

/// Quality: which DiagnosticStatus each concept characterises in the
/// abductive workflow.
#[derive(Debug, Clone)]
pub struct DiagnosticStatusQuality;

impl Quality for DiagnosticStatusQuality {
    type Individual = DiagnosticConcept;
    type Value = DiagnosticStatus;

    fn get(&self, c: &DiagnosticConcept) -> Option<DiagnosticStatus> {
        use DiagnosticConcept as D;
        match c {
            D::Residual => Some(DiagnosticStatus::Healthy),
            D::Symptom | D::Hypothesis | D::Test | D::Evidence => {
                Some(DiagnosticStatus::Investigating)
            }
            D::Diagnosis => Some(DiagnosticStatus::Diagnosed),
            D::Remedy => Some(DiagnosticStatus::Remediated),
            _ => None,
        }
    }
}

impl Ontology for DiagnosticOntology {
    type Cat = DiagnosticCategory;
    type Qual = DiagnosticStatusQuality;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<DiagnosticCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        DiagnosticOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ten_concepts() {
        assert_eq!(DiagnosticConcept::variants().len(), 10);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn symptom_generates_hypothesis() {
        // Reiter (1987) §3.
        let m = DiagnosticCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == DiagnosticConcept::Symptom
            && r.target() == DiagnosticConcept::Hypothesis
            && r.kind() == DiagnosticRelationKind::Generates));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn hypothesis_requires_test() {
        let m = DiagnosticCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == DiagnosticConcept::Hypothesis
            && r.target() == DiagnosticConcept::Test
            && r.kind() == DiagnosticRelationKind::Requires));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_produces_evidence() {
        let m = DiagnosticCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == DiagnosticConcept::Test
            && r.target() == DiagnosticConcept::Evidence
            && r.kind() == DiagnosticRelationKind::Produces));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn evidence_updates_hypothesis() {
        let m = DiagnosticCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == DiagnosticConcept::Evidence
            && r.target() == DiagnosticConcept::Hypothesis
            && r.kind() == DiagnosticRelationKind::Updates));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn residual_triggers_symptom() {
        // Gertler (1998).
        let m = DiagnosticCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == DiagnosticConcept::Residual
            && r.target() == DiagnosticConcept::Symptom
            && r.kind() == DiagnosticRelationKind::Triggers));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn diagnosis_prescribes_remedy() {
        // Kephart & Chess (2003) MAPE-K Execute.
        let m = DiagnosticCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == DiagnosticConcept::Diagnosis
            && r.target() == DiagnosticConcept::Remedy
            && r.kind() == DiagnosticRelationKind::Prescribes));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn trace_context_contextualizes_symptom() {
        let m = DiagnosticCategory::morphisms();
        assert!(
            m.iter()
                .any(|r| r.source() == DiagnosticConcept::TraceContext
                    && r.target() == DiagnosticConcept::Symptom
                    && r.kind() == DiagnosticRelationKind::Contextualizes)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn observability_levels_distinct() {
        assert_ne!(
            ObservabilityLevel::FullyObservable,
            ObservabilityLevel::Unobservable
        );
        assert_ne!(
            ObservabilityLevel::PartiallyObservable,
            ObservabilityLevel::Unobservable
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn diagnostic_feedback_loop_present() {
        // Hypothesis → Test → Evidence → Hypothesis (Bayesian update).
        let m = DiagnosticCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == DiagnosticConcept::Hypothesis
            && r.target() == DiagnosticConcept::Test));
        assert!(
            m.iter().any(|r| r.source() == DiagnosticConcept::Test
                && r.target() == DiagnosticConcept::Evidence)
        );
        assert!(m.iter().any(|r| r.source() == DiagnosticConcept::Evidence
            && r.target() == DiagnosticConcept::Hypothesis));
    }

    fn arb_concept() -> impl Strategy<Value = DiagnosticConcept> {
        proptest::sample::select(DiagnosticConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in DiagnosticCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in DiagnosticOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }

        #[test]
        fn prop_diagnostic_status_total_on_workflow(c in arb_concept()) {
            use DiagnosticConcept as D;
            let v = DiagnosticStatusQuality.get(&c);
            let on_workflow = matches!(c,
                D::Residual | D::Symptom | D::Hypothesis | D::Test
                | D::Evidence | D::Diagnosis | D::Remedy);
            prop_assert_eq!(v.is_some(), on_workflow);
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_diagnostic_status_total_on_workflow, Verifiable);
}
