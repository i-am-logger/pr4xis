use super::ontology::*;
use pr4xis::category::{Arrow, Category, Concept};

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_measurement() -> impl Strategy<Value = MeasurementConcept> {
        proptest::sample::select(MeasurementConcept::variants())
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

        /// Every concept has an Identity self-morphism. The dense
        /// `Composed` kind was removed (#166).
        #[test]
        fn prop_self_identity(c in arb_measurement()) {
            let m = MeasurementCategory::morphisms();
            prop_assert!(m.iter().any(|r| r.source() == c
                && r.target() == c
                && r.kind() == MeasurementRelationKind::Identity));
        }

        /// VIM 2.9: Result MUST carry Uncertainty.
        #[test]
        fn prop_result_carries_uncertainty(_dummy in 0..1i32) {
            let m = MeasurementCategory::morphisms();
            prop_assert!(m.iter().any(|r|
                r.source() == MeasurementConcept::Result
                && r.target() == MeasurementConcept::Uncertainty
                && r.kind() == MeasurementRelationKind::Carries));
        }

        /// Stevens (1946): scale hierarchy.
        #[test]
        fn prop_scale_hierarchy(s in arb_scale()) {
            if s.permits_ratio() {
                prop_assert!(s.permits_mean());
            }
            if s.permits_mean() {
                prop_assert!(s.permits_median());
            }
        }

        #[test]
        fn prop_nominal_is_weakest(_dummy in 0..1i32) {
            prop_assert!(!ScaleKind::Nominal.permits_mean());
            prop_assert!(!ScaleKind::Nominal.permits_median());
            prop_assert!(!ScaleKind::Nominal.permits_ratio());
        }

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
            for morph in m.iter().filter(|r| r.source() == c) {
                let composed = MeasurementCategory::compose(&id, morph);
                prop_assert_eq!(
                    composed.as_ref().map(|r| (r.source(), r.target())),
                    Some((morph.source(), morph.target()))
                );
            }
        }
    }
}
