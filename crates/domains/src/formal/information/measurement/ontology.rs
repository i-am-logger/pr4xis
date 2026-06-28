//! Measurement — the science of quantification.
//!
//! Measurement is the process of experimentally obtaining quantity values
//! that can reasonably be attributed to a quantity. Every measurement
//! result MUST carry uncertainty — a bare number is not a measurement.
//!
//! # Literature
//!
//! - **JCGM 200:2012 (VIM)** *International Vocabulary of Metrology —
//!   Basic and General Concepts and Associated Terms*, BIPM — measurand
//!   (2.3), measurement (2.1), result (2.9), uncertainty (2.26),
//!   traceability (2.41), unit (1.9), indication (4.1).
//! - **Stevens (1946)** "On the Theory of Scales of Measurement",
//!   *Science* 103(2684):677-680 — nominal / ordinal / interval / ratio
//!   scale types; permissible-statistics hierarchy.
//! - **Krantz, Luce, Suppes & Tversky (1971)** *Foundations of
//!   Measurement, Volume I*, Academic Press — measurement as
//!   homomorphism from empirical to numerical system.
//! - **JCGM 100:2008 (GUM)** *Guide to the Expression of Uncertainty in
//!   Measurement*, BIPM — uncertainty propagation.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Measurement",
    source: "JCGM 200:2012 International Vocabulary of Metrology (VIM); Stevens (1946) On the Theory of Scales of Measurement, Science 103(2684):677-680; Krantz, Luce, Suppes & Tversky (1971) Foundations of Measurement Volume I; JCGM 100:2008 Guide to the Expression of Uncertainty in Measurement (GUM)",

    concepts: [
        Measurand,
        Measurement,
        Result,
        Uncertainty,
        Unit,
        Procedure,
        Principle,
        Traceability,
        Indication,
        ScaleType,
    ],

    labels: {
        Measurand: ("en", "Measurand",
            "VIM 2.3: the specific quantity intended to be measured."),
        Measurement: ("en", "Measurement",
            "VIM 2.1: the process of experimentally obtaining one or more quantity values that can reasonably be attributed to a quantity."),
        Result: ("en", "Measurement result",
            "VIM 2.9: the set of quantity values being attributed to a measurand together with any other available relevant information. A result without uncertainty is not a measurement result."),
        Uncertainty: ("en", "Measurement uncertainty",
            "VIM 2.26: non-negative parameter characterising the dispersion of values attributed to a measurand. GUM (2008): propagates through composition."),
        Unit: ("en", "Unit",
            "VIM 1.9: a real scalar quantity, defined and adopted by convention, used as a reference standard."),
        Procedure: ("en", "Measurement procedure",
            "VIM 2.6: detailed description of a measurement according to one or more measurement principles and a given method."),
        Principle: ("en", "Measurement principle",
            "VIM 2.4: the phenomenon serving as the basis of measurement (e.g., Doppler effect for velocity)."),
        Traceability: ("en", "Metrological traceability",
            "VIM 2.41: the property of a measurement result whereby it can be related to a reference through a documented unbroken chain of calibrations."),
        Indication: ("en", "Indication",
            "VIM 4.1: the quantity value provided by a measuring instrument before corrections."),
        ScaleType: ("en", "Scale type",
            "Stevens (1946): the kind of measurement scale (nominal / ordinal / interval / ratio) determining permissible statistics."),
    },

    edges: [
        // VIM 2.1 / 2.3: Measurement targets Measurand and produces Result.
        (Measurement, Measurand, Targets),
        (Measurement, Result, Produces),
        // VIM 2.9 / 2.26: Result MUST carry Uncertainty (non-negotiable).
        (Result, Uncertainty, Carries),
        // Result is expressed in a Unit (VIM 1.9).
        (Result, Unit, ExpressedIn),
        // Measurement follows a Procedure based on a Principle.
        (Measurement, Procedure, Follows),
        (Procedure, Principle, BasedOn),
        // VIM 2.41: Result has Traceability to a reference.
        (Result, Traceability, TracesTo),
        // VIM 4.1: Measurement yields Indication, corrected to Result.
        (Measurement, Indication, Yields),
        (Indication, Result, CorrectedTo),
        // Stevens (1946): Result has a ScaleType.
        (Result, ScaleType, HasScale),
    ],
}

/// Stevens' scale types — a total order of measurement strength
/// (Stevens 1946 *Science* 103(2684):677-680). Each admits a group of
/// permissible transformations and constrains which statistics are
/// meaningful.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScaleKind {
    /// Classification only. Permissible: any bijection. Statistics:
    /// mode, chi-square. Example: jersey numbers.
    Nominal,
    /// Rank order. Permissible: monotone increasing functions.
    /// Statistics: median, percentile, Spearman correlation. Example:
    /// Mohs hardness.
    Ordinal,
    /// Equal intervals, arbitrary zero. Permissible: y = ax + b (a > 0).
    /// Statistics: mean, standard deviation. Example: Celsius
    /// temperature.
    Interval,
    /// True zero, all arithmetic meaningful. Permissible: y = ax (a > 0).
    /// Statistics: geometric mean, coefficient of variation. Example:
    /// mass in kg.
    Ratio,
}

impl ScaleKind {
    /// Stevens (1946): mean requires at least interval scale.
    pub fn permits_mean(&self) -> bool {
        matches!(self, Self::Interval | Self::Ratio)
    }

    /// Stevens (1946): median requires at least ordinal scale.
    pub fn permits_median(&self) -> bool {
        !matches!(self, Self::Nominal)
    }

    /// Stevens (1946): ratios require ratio scale.
    pub fn permits_ratio(&self) -> bool {
        matches!(self, Self::Ratio)
    }
}

/// Quality: the Stevens scale kind associated with each measurement
/// concept. By VIM/GUM convention a Result is ratio-scale by default.
#[derive(Debug, Clone)]
pub struct ScaleKindQuality;

impl Quality for ScaleKindQuality {
    type Individual = MeasurementConcept;
    type Value = ScaleKind;

    fn get(&self, c: &MeasurementConcept) -> Option<ScaleKind> {
        match c {
            MeasurementConcept::Result => Some(ScaleKind::Ratio),
            _ => None,
        }
    }
}

impl Ontology for MeasurementOntology {
    type Cat = MeasurementCategory;
    type Qual = ScaleKindQuality;

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
        assert_category_laws::<MeasurementCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        MeasurementOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ten_concepts() {
        assert_eq!(MeasurementConcept::variants().len(), 10);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn result_carries_uncertainty() {
        // VIM 2.9 axiom — non-negotiable.
        let m = MeasurementCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == MeasurementConcept::Result
            && r.target() == MeasurementConcept::Uncertainty
            && r.kind() == MeasurementRelationKind::Carries));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn measurement_produces_result() {
        let m = MeasurementCategory::morphisms();
        assert!(
            m.iter()
                .any(|r| r.source() == MeasurementConcept::Measurement
                    && r.target() == MeasurementConcept::Result
                    && r.kind() == MeasurementRelationKind::Produces)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn result_has_traceability() {
        // VIM 2.41.
        let m = MeasurementCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == MeasurementConcept::Result
            && r.target() == MeasurementConcept::Traceability
            && r.kind() == MeasurementRelationKind::TracesTo));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn indication_corrected_to_result() {
        // VIM 4.1: instrument indication → corrected result.
        let m = MeasurementCategory::morphisms();
        assert!(
            m.iter()
                .any(|r| r.source() == MeasurementConcept::Indication
                    && r.target() == MeasurementConcept::Result
                    && r.kind() == MeasurementRelationKind::CorrectedTo)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn nominal_permits_only_mode() {
        assert!(!ScaleKind::Nominal.permits_mean());
        assert!(!ScaleKind::Nominal.permits_median());
        assert!(!ScaleKind::Nominal.permits_ratio());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ratio_permits_everything() {
        assert!(ScaleKind::Ratio.permits_mean());
        assert!(ScaleKind::Ratio.permits_median());
        assert!(ScaleKind::Ratio.permits_ratio());
    }

    fn arb_concept() -> impl Strategy<Value = MeasurementConcept> {
        proptest::sample::select(MeasurementConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in MeasurementCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in MeasurementOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }

        #[test]
        fn prop_scale_hierarchy(_seed in any::<u32>()) {
            // Stevens (1946): stronger scales permit all weaker operations.
            for s in [ScaleKind::Nominal, ScaleKind::Ordinal, ScaleKind::Interval, ScaleKind::Ratio] {
                if s.permits_ratio() { prop_assert!(s.permits_mean()); }
                if s.permits_mean() { prop_assert!(s.permits_median()); }
            }
        }

        #[test]
        fn prop_result_carries_uncertainty(_seed in any::<u32>()) {
            // VIM 2.9 invariant.
            let m = MeasurementCategory::morphisms();
            prop_assert!(m.iter().any(|r|
                r.source() == MeasurementConcept::Result
                && r.target() == MeasurementConcept::Uncertainty
                && r.kind() == MeasurementRelationKind::Carries));
        }

        #[test]
        fn prop_scale_quality_partial(c in arb_concept()) {
            // ScaleKindQuality is defined only on Result.
            let v = ScaleKindQuality.get(&c);
            prop_assert_eq!(v.is_some(), c == MeasurementConcept::Result);
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_scale_hierarchy, Verifiable);
    pr4xis::register_praxis_value!(prop_result_carries_uncertainty, Verifiable);
    pr4xis::register_praxis_value!(prop_scale_quality_partial, Honest, Verifiable);
}
