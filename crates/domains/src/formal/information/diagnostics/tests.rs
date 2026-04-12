use super::ontology::*;
use pr4xis::category::Category;

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_diagnostic() -> impl Strategy<Value = DiagnosticConcept> {
        prop_oneof![
            Just(DiagnosticConcept::Symptom),
            Just(DiagnosticConcept::Hypothesis),
            Just(DiagnosticConcept::Test),
            Just(DiagnosticConcept::Evidence),
            Just(DiagnosticConcept::Diagnosis),
            Just(DiagnosticConcept::Residual),
            Just(DiagnosticConcept::FaultMode),
            Just(DiagnosticConcept::Severity),
            Just(DiagnosticConcept::Remedy),
            Just(DiagnosticConcept::TraceContext),
        ]
    }

    proptest! {
        #[test]
        fn prop_identity_idempotent(c in arb_diagnostic()) {
            let id = DiagnosticCategory::identity(&c);
            prop_assert_eq!(DiagnosticCategory::compose(&id, &id), Some(id));
        }

        /// Every concept has both Identity and Composed self-morphisms.
        #[test]
        fn prop_self_morphisms(c in arb_diagnostic()) {
            let m = DiagnosticCategory::morphisms();
            let has_identity = m.iter().any(|r| r.from == c && r.to == c && r.kind == DiagnosticRelationKind::Identity);
            let has_composed = m.iter().any(|r| r.from == c && r.to == c && r.kind == DiagnosticRelationKind::Composed);
            prop_assert!(has_identity);
            prop_assert!(has_composed);
        }

        /// Reiter (1987): Symptom always reaches Diagnosis through the cycle.
        #[test]
        fn prop_symptom_reaches_diagnosis(_dummy in 0..1i32) {
            let m = DiagnosticCategory::morphisms();
            prop_assert!(m.iter().any(|r|
                r.from == DiagnosticConcept::Symptom
                && r.to == DiagnosticConcept::Diagnosis));
        }

        /// Gertler FDI: Residual always triggers Symptom.
        #[test]
        fn prop_residual_triggers_symptom(_dummy in 0..1i32) {
            let m = DiagnosticCategory::morphisms();
            prop_assert!(m.iter().any(|r|
                r.from == DiagnosticConcept::Residual
                && r.to == DiagnosticConcept::Symptom
                && r.kind == DiagnosticRelationKind::Triggers));
        }

        /// MAPE-K: every Diagnosis has a Remedy, FaultMode, and Severity.
        #[test]
        fn prop_diagnosis_has_outputs(_dummy in 0..1i32) {
            let m = DiagnosticCategory::morphisms();
            prop_assert!(m.iter().any(|r| r.from == DiagnosticConcept::Diagnosis && r.to == DiagnosticConcept::Remedy));
            prop_assert!(m.iter().any(|r| r.from == DiagnosticConcept::Diagnosis && r.to == DiagnosticConcept::FaultMode));
            prop_assert!(m.iter().any(|r| r.from == DiagnosticConcept::Diagnosis && r.to == DiagnosticConcept::Severity));
        }

        /// Composition with identity preserves any morphism.
        #[test]
        fn prop_left_identity(c in arb_diagnostic()) {
            let m = DiagnosticCategory::morphisms();
            let id = DiagnosticCategory::identity(&c);
            for morph in m.iter().filter(|r| r.from == c) {
                let composed = DiagnosticCategory::compose(&id, morph);
                prop_assert_eq!(composed.as_ref().map(|r| (r.from, r.to)), Some((morph.from, morph.to)));
            }
        }

        /// Bayesian feedback: Evidence → Hypothesis → Test → Evidence loop.
        #[test]
        fn prop_diagnostic_feedback_loop(_dummy in 0..1i32) {
            let m = DiagnosticCategory::morphisms();
            prop_assert!(m.iter().any(|r| r.from == DiagnosticConcept::Evidence && r.to == DiagnosticConcept::Hypothesis));
            prop_assert!(m.iter().any(|r| r.from == DiagnosticConcept::Hypothesis && r.to == DiagnosticConcept::Test));
            prop_assert!(m.iter().any(|r| r.from == DiagnosticConcept::Test && r.to == DiagnosticConcept::Evidence));
        }
    }
}
