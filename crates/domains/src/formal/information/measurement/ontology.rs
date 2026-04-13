use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::{Ontology, Quality};

// Measurement ontology — the science of quantification.
//
// Measurement is the process of experimentally obtaining quantity values
// that can reasonably be attributed to a quantity. Every measurement
// result MUST carry uncertainty — a bare number is not a measurement.
//
// Also models Stevens' scale types, which constrain what operations
// are permissible on measured data (you cannot take the mean of ordinal data).
//
// References:
// - VIM (JCGM 200:2012), "International Vocabulary of Metrology" —
//   measurand, measurement result, uncertainty, traceability
// - Stevens, "On the Theory of Scales of Measurement" (1946, Science) —
//   nominal, ordinal, interval, ratio scale types
// - Krantz, Luce, Suppes & Tversky, "Foundations of Measurement" (1971) —
//   measurement as homomorphism from empirical to numerical system
// - GUM (JCGM 100:2008), "Guide to the Expression of Uncertainty in
//   Measurement" — uncertainty propagation
// - QUDT (Quantities, Units, Dimensions, Types) — W3C ontology

/// Concepts in the measurement ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum MeasurementConcept {
    /// The specific quantity intended to be measured.
    /// VIM 2.3: "quantity intended to be measured."
    /// Not the same as Quantity — it's the target of a measurement process.
    Measurand,

    /// The process of obtaining quantity values.
    /// VIM 2.1: "process of experimentally obtaining one or more quantity
    /// values that can reasonably be attributed to a quantity."
    Measurement,

    /// The output: quantity values + uncertainty.
    /// VIM 2.9: "set of quantity values being attributed to a measurand,
    /// together with any other available relevant information."
    /// A result WITHOUT uncertainty is not a measurement result.
    Result,

    /// Non-negative parameter characterizing dispersion of values.
    /// VIM 2.26: NOT error — it's intrinsic to the measurement.
    /// GUM (2008): propagates through composition.
    Uncertainty,

    /// A reference quantity used as a standard.
    /// VIM 1.9: "real scalar quantity, defined and adopted by convention."
    Unit,

    /// The detailed description of how to measure.
    /// VIM 2.6: "measurement according to one or more measurement
    /// principles and a given measurement method."
    Procedure,

    /// The phenomenon serving as the basis of measurement.
    /// VIM 2.4: e.g., Doppler effect for velocity measurement.
    Principle,

    /// The chain linking a result to a reference standard.
    /// VIM 2.41: "property of a measurement result whereby the result
    /// can be related to a reference through a documented unbroken
    /// chain of calibrations."
    Traceability,

    /// The raw output of a measuring instrument before corrections.
    /// VIM 4.1: "quantity value provided by a measuring instrument."
    Indication,

    /// The type of scale: nominal, ordinal, interval, ratio.
    /// Stevens (1946): determines permissible statistics.
    ScaleType,
}

define_ontology! {
    pub MeasurementOntology for MeasurementCategory {
        concepts: MeasurementConcept,
        relation: MeasurementRelation,
        kind: MeasurementRelationKind,
        kinds: [
            /// Measurement targets a Measurand.
            Targets,
            /// Measurement produces a Result.
            Produces,
            /// Result carries Uncertainty (VIM: non-negotiable).
            Carries,
            /// Result is expressed in a Unit.
            ExpressedIn,
            /// Measurement follows a Procedure.
            Follows,
            /// Procedure is based on a Principle.
            BasedOn,
            /// Result has Traceability to a reference.
            TracesTo,
            /// Measurement yields an Indication (raw output).
            Yields,
            /// Indication is corrected to produce Result.
            CorrectedTo,
            /// Result has a ScaleType (determines permissible operations).
            HasScale,
        ],
        edges: [
            // The measurement process: Measurement targets Measurand, produces Result
            (Measurement, Measurand, Targets),
            (Measurement, Result, Produces),
            // Result MUST carry Uncertainty (VIM axiom)
            (Result, Uncertainty, Carries),
            // Result is expressed in Unit
            (Result, Unit, ExpressedIn),
            // Measurement follows Procedure based on Principle
            (Measurement, Procedure, Follows),
            (Procedure, Principle, BasedOn),
            // Result has Traceability
            (Result, Traceability, TracesTo),
            // Measurement yields Indication, corrected to Result
            (Measurement, Indication, Yields),
            (Indication, Result, CorrectedTo),
            // Result has ScaleType
            (Result, ScaleType, HasScale),
        ],
        composed: [
            // Measurement → Uncertainty (through Result)
            (Measurement, Uncertainty),
            // Measurement → Principle (through Procedure)
            (Measurement, Principle),
            // Measurement → Unit (through Result)
            (Measurement, Unit),
        ],
    }
}

/// The Stevens scale kind associated with each measurement concept.
#[derive(Debug, Clone)]
pub struct ScaleKindQuality;

impl Quality for ScaleKindQuality {
    type Individual = MeasurementConcept;
    type Value = ScaleKind;

    fn get(&self, individual: &MeasurementConcept) -> Option<ScaleKind> {
        match individual {
            MeasurementConcept::ScaleType => None, // meta-concept, not a measurement itself
            MeasurementConcept::Result => Some(ScaleKind::Ratio), // default: results are ratio-scale
            _ => None,
        }
    }
}

impl Ontology for MeasurementOntology {
    type Cat = MeasurementCategory;
    type Qual = ScaleKindQuality;

    fn structural_axioms() -> Vec<Box<dyn pr4xis::ontology::Axiom>> {
        Self::generated_structural_axioms()
    }
}

/// Stevens' scale types — a total order of measurement strength.
///
/// Each scale type admits a group of permissible transformations and
/// constrains which statistics are meaningful.
/// Stevens (1946, Science, Vol. 103, No. 2684).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScaleKind {
    /// Classification only. Permissible: any bijection.
    /// Statistics: mode, chi-square.
    /// Example: jersey numbers, postal codes.
    Nominal,

    /// Rank order. Permissible: monotone increasing functions.
    /// Statistics: median, percentile, Spearman correlation.
    /// Example: Mohs hardness, pain scales.
    Ordinal,

    /// Equal intervals, arbitrary zero. Permissible: y = ax + b (a > 0).
    /// Statistics: mean, standard deviation, Pearson correlation.
    /// Example: Celsius temperature, calendar dates.
    Interval,

    /// True zero, all arithmetic meaningful. Permissible: y = ax (a > 0).
    /// Statistics: geometric mean, coefficient of variation.
    /// Example: duration (seconds), mass (kg), throughput (ops/sec).
    Ratio,
}

impl ScaleKind {
    /// Can we compute a mean on this scale?
    /// Stevens (1946): mean requires at least interval scale.
    pub fn permits_mean(&self) -> bool {
        matches!(self, Self::Interval | Self::Ratio)
    }

    /// Can we compute a median on this scale?
    /// Stevens (1946): median requires at least ordinal scale.
    pub fn permits_median(&self) -> bool {
        !matches!(self, Self::Nominal)
    }

    /// Can we compute ratios (e.g., "twice as fast")?
    /// Stevens (1946): ratios require ratio scale.
    pub fn permits_ratio(&self) -> bool {
        matches!(self, Self::Ratio)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws_hold() {
        check_category_laws::<MeasurementCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        MeasurementOntology::validate().unwrap();
    }

    #[test]
    fn has_ten_concepts() {
        assert_eq!(MeasurementConcept::variants().len(), 10);
    }

    // --- VIM 2.9: Result MUST carry Uncertainty ---

    #[test]
    fn result_carries_uncertainty() {
        let m = MeasurementCategory::morphisms();
        assert!(m.iter().any(|r| r.from == MeasurementConcept::Result
            && r.to == MeasurementConcept::Uncertainty
            && r.kind == MeasurementRelationKind::Carries));
    }

    // --- VIM 2.1: Measurement produces Result ---

    #[test]
    fn measurement_produces_result() {
        let m = MeasurementCategory::morphisms();
        assert!(m.iter().any(|r| r.from == MeasurementConcept::Measurement
            && r.to == MeasurementConcept::Result
            && r.kind == MeasurementRelationKind::Produces));
    }

    // --- VIM 2.3: Measurement targets Measurand ---

    #[test]
    fn measurement_targets_measurand() {
        let m = MeasurementCategory::morphisms();
        assert!(m.iter().any(|r| r.from == MeasurementConcept::Measurement
            && r.to == MeasurementConcept::Measurand
            && r.kind == MeasurementRelationKind::Targets));
    }

    // --- VIM 2.41: Result has Traceability ---

    #[test]
    fn result_has_traceability() {
        let m = MeasurementCategory::morphisms();
        assert!(m.iter().any(|r| r.from == MeasurementConcept::Result
            && r.to == MeasurementConcept::Traceability
            && r.kind == MeasurementRelationKind::TracesTo));
    }

    // --- Krantz (1971): Measurement is a homomorphism ---
    // (Measurement maps empirical system to numerical system through Indication → Result)

    #[test]
    fn indication_corrected_to_result() {
        let m = MeasurementCategory::morphisms();
        assert!(m.iter().any(|r| r.from == MeasurementConcept::Indication
            && r.to == MeasurementConcept::Result
            && r.kind == MeasurementRelationKind::CorrectedTo));
    }

    // --- Stevens (1946): Scale types and permissible statistics ---

    #[test]
    fn scale_types_exist() {
        let _nominal = ScaleKind::Nominal;
        let _ordinal = ScaleKind::Ordinal;
        let _interval = ScaleKind::Interval;
        let _ratio = ScaleKind::Ratio;
    }

    #[test]
    fn nominal_permits_only_mode() {
        assert!(!ScaleKind::Nominal.permits_mean());
        assert!(!ScaleKind::Nominal.permits_median());
        assert!(!ScaleKind::Nominal.permits_ratio());
    }

    #[test]
    fn ordinal_permits_median_not_mean() {
        assert!(!ScaleKind::Ordinal.permits_mean());
        assert!(ScaleKind::Ordinal.permits_median());
        assert!(!ScaleKind::Ordinal.permits_ratio());
    }

    #[test]
    fn interval_permits_mean_not_ratio() {
        assert!(ScaleKind::Interval.permits_mean());
        assert!(ScaleKind::Interval.permits_median());
        assert!(!ScaleKind::Interval.permits_ratio());
    }

    #[test]
    fn ratio_permits_everything() {
        assert!(ScaleKind::Ratio.permits_mean());
        assert!(ScaleKind::Ratio.permits_median());
        assert!(ScaleKind::Ratio.permits_ratio());
    }

    // --- Stevens: Scale types form a hierarchy ---
    // Ratio ⊃ Interval ⊃ Ordinal ⊃ Nominal
    // Each stronger scale permits all operations of weaker scales.

    #[test]
    fn scale_hierarchy() {
        // Ratio permits everything Interval permits
        assert!(ScaleKind::Ratio.permits_mean());
        // Interval permits everything Ordinal permits
        assert!(ScaleKind::Interval.permits_median());
        // Ordinal permits everything Nominal permits
        // (Nominal permits mode, which all scales do — not modeled as method)
    }

    // --- Composition: Measurement → Uncertainty ---

    #[test]
    fn measurement_reaches_uncertainty() {
        let m = MeasurementCategory::morphisms();
        assert!(m.iter().any(|r| r.from == MeasurementConcept::Measurement
            && r.to == MeasurementConcept::Uncertainty));
    }

    // --- Result has ScaleType ---

    #[test]
    fn result_has_scale_type() {
        let m = MeasurementCategory::morphisms();
        assert!(m.iter().any(|r| r.from == MeasurementConcept::Result
            && r.to == MeasurementConcept::ScaleType
            && r.kind == MeasurementRelationKind::HasScale));
    }
}
