use super::ontology::*;
use pr4xis::category::{Arrow, Category, FinitelyGenerated};

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_diagnostic() -> impl Strategy<Value = DiagnosticConcept> {
        proptest::sample::select(DiagnosticConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_identity_idempotent(c in arb_diagnostic()) {
            let id = DiagnosticCategory::identity(&c);
            prop_assert_eq!(DiagnosticCategory::compose(&id, &id), Some(id));
        }

        /// Every concept has an Identity self-morphism. The dense
        /// `Composed` kind was removed (#166) — per-concept composed
        /// self-morphisms are no longer emitted.
        #[test]
        fn prop_self_identity(c in arb_diagnostic()) {
            let m = DiagnosticCategory::morphisms();
            let has_identity = m
                .iter()
                .any(|r| r.source() == c && r.target() == c && r.kind() == DiagnosticRelationKind::Identity);
            prop_assert!(has_identity);
        }

        /// Reiter (1987): Symptom transitively reaches Diagnosis. The
        /// transitive-closure edge is produced by same-kind composition
        /// of Subsumption / Parthood / Causation; the canonical reading
        /// here is the explicit edge sequence Symptom→Hypothesis→Diagnosis
        /// each verified separately.
        #[test]
        fn prop_symptom_reaches_diagnosis_via_hypothesis(_dummy in 0..1i32) {
            let m = DiagnosticCategory::morphisms();
            prop_assert!(m.iter().any(|r|
                r.source() == DiagnosticConcept::Symptom
                && r.target() == DiagnosticConcept::Hypothesis));
            prop_assert!(m.iter().any(|r|
                r.source() == DiagnosticConcept::Hypothesis
                && r.target() == DiagnosticConcept::Diagnosis));
        }

        /// Gertler FDI: Residual always triggers Symptom.
        #[test]
        fn prop_residual_triggers_symptom(_dummy in 0..1i32) {
            let m = DiagnosticCategory::morphisms();
            prop_assert!(m.iter().any(|r|
                r.source() == DiagnosticConcept::Residual
                && r.target() == DiagnosticConcept::Symptom
                && r.kind() == DiagnosticRelationKind::Triggers));
        }

        /// MAPE-K (Kephart & Chess 2003): every Diagnosis has a Remedy,
        /// FaultMode, and Severity.
        #[test]
        fn prop_diagnosis_has_outputs(_dummy in 0..1i32) {
            let m = DiagnosticCategory::morphisms();
            prop_assert!(m.iter().any(|r| r.source() == DiagnosticConcept::Diagnosis
                && r.target() == DiagnosticConcept::Remedy));
            prop_assert!(m.iter().any(|r| r.source() == DiagnosticConcept::Diagnosis
                && r.target() == DiagnosticConcept::FaultMode));
            prop_assert!(m.iter().any(|r| r.source() == DiagnosticConcept::Diagnosis
                && r.target() == DiagnosticConcept::Severity));
        }

        /// Composition with identity preserves any morphism.
        #[test]
        fn prop_left_identity(c in arb_diagnostic()) {
            let m = DiagnosticCategory::morphisms();
            let id = DiagnosticCategory::identity(&c);
            for morph in m.iter().filter(|r| r.source() == c) {
                let composed = DiagnosticCategory::compose(&id, morph);
                prop_assert_eq!(
                    composed.as_ref().map(|r| (r.source(), r.target())),
                    Some((morph.source(), morph.target()))
                );
            }
        }

        /// Bayesian feedback: Evidence → Hypothesis → Test → Evidence loop.
        #[test]
        fn prop_diagnostic_feedback_loop(_dummy in 0..1i32) {
            let m = DiagnosticCategory::morphisms();
            prop_assert!(m.iter().any(|r| r.source() == DiagnosticConcept::Evidence
                && r.target() == DiagnosticConcept::Hypothesis));
            prop_assert!(m.iter().any(|r| r.source() == DiagnosticConcept::Hypothesis
                && r.target() == DiagnosticConcept::Test));
            prop_assert!(m.iter().any(|r| r.source() == DiagnosticConcept::Test
                && r.target() == DiagnosticConcept::Evidence));
        }
    }
}
