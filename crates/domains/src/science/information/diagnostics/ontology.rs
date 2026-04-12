use pr4xis::category::Category;
use pr4xis::category::entity::Entity;
use pr4xis::category::relationship::Relationship;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl Entity for DiagnosticConcept {
    fn variants() -> Vec<Self> {
        vec![
            Self::Symptom,
            Self::Hypothesis,
            Self::Test,
            Self::Evidence,
            Self::Diagnosis,
            Self::Residual,
            Self::FaultMode,
            Self::Severity,
            Self::Remedy,
            Self::TraceContext,
        ]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiagnosticRelation {
    pub from: DiagnosticConcept,
    pub to: DiagnosticConcept,
    pub kind: DiagnosticRelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticRelationKind {
    Identity,
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
    Composed,
}

impl Relationship for DiagnosticRelation {
    type Object = DiagnosticConcept;
    fn source(&self) -> DiagnosticConcept {
        self.from
    }
    fn target(&self) -> DiagnosticConcept {
        self.to
    }
}

pub struct DiagnosticCategory;

impl Category for DiagnosticCategory {
    type Object = DiagnosticConcept;
    type Morphism = DiagnosticRelation;

    fn identity(obj: &DiagnosticConcept) -> DiagnosticRelation {
        DiagnosticRelation {
            from: *obj,
            to: *obj,
            kind: DiagnosticRelationKind::Identity,
        }
    }

    fn compose(f: &DiagnosticRelation, g: &DiagnosticRelation) -> Option<DiagnosticRelation> {
        if f.to != g.from {
            return None;
        }
        if f.kind == DiagnosticRelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == DiagnosticRelationKind::Identity {
            return Some(f.clone());
        }
        Some(DiagnosticRelation {
            from: f.from,
            to: g.to,
            kind: DiagnosticRelationKind::Composed,
        })
    }

    fn morphisms() -> Vec<DiagnosticRelation> {
        use DiagnosticConcept as C;
        use DiagnosticRelationKind as R;
        let mut m = Vec::new();

        for c in DiagnosticConcept::variants() {
            m.push(DiagnosticRelation {
                from: c,
                to: c,
                kind: R::Identity,
            });
        }

        // The diagnostic cycle: Observation → Hypothesis → Test → Conclusion
        m.push(DiagnosticRelation {
            from: C::Residual,
            to: C::Symptom,
            kind: R::Triggers,
        });
        m.push(DiagnosticRelation {
            from: C::Symptom,
            to: C::Hypothesis,
            kind: R::Generates,
        });
        m.push(DiagnosticRelation {
            from: C::Hypothesis,
            to: C::Test,
            kind: R::Requires,
        });
        m.push(DiagnosticRelation {
            from: C::Test,
            to: C::Evidence,
            kind: R::Produces,
        });
        m.push(DiagnosticRelation {
            from: C::Evidence,
            to: C::Hypothesis,
            kind: R::Updates,
        });
        m.push(DiagnosticRelation {
            from: C::Hypothesis,
            to: C::Diagnosis,
            kind: R::Confirms,
        });

        // Diagnosis outputs
        m.push(DiagnosticRelation {
            from: C::Diagnosis,
            to: C::FaultMode,
            kind: R::Identifies,
        });
        m.push(DiagnosticRelation {
            from: C::Diagnosis,
            to: C::Severity,
            kind: R::HasSeverity,
        });
        m.push(DiagnosticRelation {
            from: C::Diagnosis,
            to: C::Remedy,
            kind: R::Prescribes,
        });

        // TraceContext links diagnosis to observability data
        m.push(DiagnosticRelation {
            from: C::TraceContext,
            to: C::Symptom,
            kind: R::Contextualizes,
        });
        m.push(DiagnosticRelation {
            from: C::TraceContext,
            to: C::Evidence,
            kind: R::Contextualizes,
        });

        // Composed: full diagnostic chain
        m.push(DiagnosticRelation {
            from: C::Residual,
            to: C::Diagnosis,
            kind: R::Composed,
        });
        m.push(DiagnosticRelation {
            from: C::Symptom,
            to: C::Diagnosis,
            kind: R::Composed,
        });
        m.push(DiagnosticRelation {
            from: C::Symptom,
            to: C::Remedy,
            kind: R::Composed,
        });
        m.push(DiagnosticRelation {
            from: C::TraceContext,
            to: C::Diagnosis,
            kind: R::Composed,
        });

        for c in DiagnosticConcept::variants() {
            m.push(DiagnosticRelation {
                from: c,
                to: c,
                kind: R::Composed,
            });
        }

        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws_hold() {
        check_category_laws::<DiagnosticCategory>().unwrap();
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
