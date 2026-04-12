use super::ontology::*;
use pr4xis::category::Category;

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_measurement() -> impl Strategy<Value = MeasurementConcept> {
        prop_oneof![
            Just(MeasurementConcept::Measurand),
            Just(MeasurementConcept::Measurement),
            Just(MeasurementConcept::Result),
            Just(MeasurementConcept::Uncertainty),
            Just(MeasurementConcept::Unit),
            Just(MeasurementConcept::Procedure),
            Just(MeasurementConcept::Principle),
            Just(MeasurementConcept::Traceability),
            Just(MeasurementConcept::Indication),
            Just(MeasurementConcept::ScaleType),
        ]
    }

    fn arb_scale() -> impl Strategy<Value = ScaleKind> {
        prop_oneof![
            Just(ScaleKind::Nominal),
            Just(ScaleKind::Ordinal),
            Just(ScaleKind::Interval),
            Just(ScaleKind::Ratio),
        ]
    }

    proptest! {
        #[test]
        fn prop_identity_idempotent(c in arb_measurement()) {
            let id = MeasurementCategory::identity(&c);
            prop_assert_eq!(MeasurementCategory::compose(&id, &id), Some(id));
        }

        /// Every concept has both Identity and Composed self-morphisms.
        #[test]
        fn prop_self_morphisms(c in arb_measurement()) {
            let m = MeasurementCategory::morphisms();
            let has_identity = m.iter().any(|r| r.from == c && r.to == c && r.kind == MeasurementRelationKind::Identity);
            let has_composed = m.iter().any(|r| r.from == c && r.to == c && r.kind == MeasurementRelationKind::Composed);
            prop_assert!(has_identity);
            prop_assert!(has_composed);
        }

        /// VIM 2.9: Result MUST carry Uncertainty (invariant).
        #[test]
        fn prop_result_carries_uncertainty(_dummy in 0..1i32) {
            let m = MeasurementCategory::morphisms();
            prop_assert!(m.iter().any(|r|
                r.from == MeasurementConcept::Result
                && r.to == MeasurementConcept::Uncertainty
                && r.kind == MeasurementRelationKind::Carries));
        }

        /// Stevens (1946): scale hierarchy — stronger scales permit all weaker operations.
        #[test]
        fn prop_scale_hierarchy(s in arb_scale()) {
            if s.permits_ratio() {
                prop_assert!(s.permits_mean());
            }
            if s.permits_mean() {
                prop_assert!(s.permits_median());
            }
        }

        /// Stevens (1946): Nominal permits neither mean nor ratio.
        #[test]
        fn prop_nominal_is_weakest(_dummy in 0..1i32) {
            prop_assert!(!ScaleKind::Nominal.permits_mean());
            prop_assert!(!ScaleKind::Nominal.permits_median());
            prop_assert!(!ScaleKind::Nominal.permits_ratio());
        }

        /// Stevens (1946): Ratio permits everything.
        #[test]
        fn prop_ratio_is_strongest(_dummy in 0..1i32) {
            prop_assert!(ScaleKind::Ratio.permits_mean());
            prop_assert!(ScaleKind::Ratio.permits_median());
            prop_assert!(ScaleKind::Ratio.permits_ratio());
        }

        /// Composition with identity preserves any morphism.
        #[test]
        fn prop_left_identity(c in arb_measurement()) {
            let m = MeasurementCategory::morphisms();
            let id = MeasurementCategory::identity(&c);
            for morph in m.iter().filter(|r| r.from == c) {
                let composed = MeasurementCategory::compose(&id, morph);
                prop_assert_eq!(composed.as_ref().map(|r| (r.from, r.to)), Some((morph.from, morph.to)));
            }
        }
    }
}
