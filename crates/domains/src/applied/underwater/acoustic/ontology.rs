//! Acoustic positioning system types.
//!
//! Source: Milne (1983), *Underwater Acoustic Positioning Systems*

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality, QualityKind};

use super::engine::{mackenzie_sound_speed, range_from_travel_time};
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::unit::METER;
use crate::formal::math::quantity::value::{Quantity, QuantityRange};
use crate::formal::math::temporal::duration::Duration;

pr4xis::ontology! {
    name: "Acoustic",
    source: "Milne (1983); Kinsey et al. (2006)",

    concepts: [USBL, LBL, SBL],

    labels: {
        USBL: ("en", "Ultra-Short Baseline", "Ultra-Short Baseline: single transceiver with multiple elements."),
        LBL: ("en", "Long Baseline", "Long Baseline: array of transponders on the seabed."),
        SBL: ("en", "Short Baseline", "Short Baseline: hull-mounted array of hydrophones."),
    },
}

/// How an acoustic positioning system's 1σ accuracy scales — the typed model
/// the prose `"0.1–1% of slant range"` / `"0.01–0.1 m"` encoded.
///
/// USBL and SBL accuracy is a dimensionless **fraction of the slant range**, so
/// absolute error grows with distance; LBL accuracy is an **absolute** length
/// bounded within the transponder baseline, independent of range (Milne 1983;
/// Kinsey et al. 2006). Both carry a typed [`QuantityRange`], so a downstream
/// consumer can compute an actual error bound (`fraction × range`) instead of
/// parsing English.
#[derive(Debug, Clone, PartialEq)]
pub enum AccuracyModel {
    /// A dimensionless fraction of slant range (absolute error = fraction × range).
    FractionOfSlantRange(QuantityRange),
    /// An absolute positional accuracy (a length), independent of range.
    Absolute(QuantityRange),
}

/// Quality: the [`AccuracyModel`] of each positioning system.
#[derive(Debug, Clone)]
pub struct PositioningAccuracy;

impl Quality for PositioningAccuracy {
    type Individual = AcousticConcept;
    type Value = AccuracyModel;
    const KIND: QualityKind = QualityKind::Physical;

    fn get(&self, system: &AcousticConcept) -> Option<AccuracyModel> {
        // 0.1%–1% of slant range, as a dimensionless fraction.
        let slant_fraction = || {
            AccuracyModel::FractionOfSlantRange(QuantityRange {
                min: Quantity::dimensionless(0.001),
                max: Quantity::dimensionless(0.01),
            })
        };
        Some(match system {
            AcousticConcept::USBL => slant_fraction(),
            AcousticConcept::SBL => slant_fraction(),
            // 0.01–0.1 m absolute, within the transponder baseline.
            AcousticConcept::LBL => AccuracyModel::Absolute(QuantityRange {
                min: Quantity::from_unit(0.01, &METER),
                max: Quantity::from_unit(0.1, &METER),
            }),
        })
    }
}

/// Axiom: sound speed in water is always positive.
pub struct SoundSpeedPositive;

impl Axiom for SoundSpeedPositive {
    fn verify(&self) -> Verdict {
        // Mackenzie (1981) "Nine-term equation for sound speed in the
        // oceans" — evaluate the real nine-term equation over a grid
        // spanning the oceanographic ranges of temperature [-2, 35] C,
        // salinity [0, 40] PSU, and depth [0, 8000] m, and confirm the
        // computed sound speed c is strictly positive at every node. A
        // non-positive (or NaN) result at any node refutes the axiom.
        let temperatures = [-2.0, 0.0, 5.0, 15.0, 25.0, 35.0];
        let salinities = [0.0, 10.0, 25.0, 35.0, 40.0];
        let depths = [0.0, 100.0, 1000.0, 4000.0, 8000.0];
        for &t in &temperatures {
            for &s in &salinities {
                for &d in &depths {
                    let c = mackenzie_sound_speed(
                        Quantity::from_unit(t, &unit::CELSIUS),
                        Quantity::from_unit(s, &unit::PSU),
                        Quantity::from_unit(d, &METER),
                    )
                    .value;
                    if c <= 0.0 || c.is_nan() {
                        return Err(Box::new(SimpleCounterexample::new(self.meta())));
                    }
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "SoundSpeedPositive",
        "sound speed in water is strictly positive (typically 1400-1600 m/s)",
        "Mackenzie (1981) Nine-term equation for sound speed in the oceans, JASA 70(3)"
    );
}
pr4xis::register_axiom!(
    SoundSpeedPositive,
    "Mackenzie (1981) Nine-term equation for sound speed in the oceans, JASA 70(3)"
);

/// Axiom: acoustic range measurements are non-negative.
pub struct RangeNonNegative;

impl Axiom for RangeNonNegative {
    fn verify(&self) -> Verdict {
        // Range = c · t_two_way / 2. Compute the real range function over
        // non-negative two-way travel times and positive sound speeds;
        // every result must be >= 0. A negative range at any node refutes
        // the axiom.
        let travel_times = [0.0, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0];
        let sound_speeds = [1400.0, 1500.0, 1540.0, 1600.0];
        for &t in &travel_times {
            for &c in &sound_speeds {
                let range = range_from_travel_time(
                    Duration::from_seconds(t),
                    Quantity::from_unit(c, &unit::METER_PER_SECOND),
                )
                .value;
                if range < 0.0 {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "RangeNonNegative",
        "acoustic range measurements are non-negative",
        "Milne (1983) Underwater Acoustic Positioning Systems"
    );
}
pr4xis::register_axiom!(
    RangeNonNegative,
    "Milne (1983) Underwater Acoustic Positioning Systems"
);

impl Ontology for AcousticOntology {
    type Cat = AcousticCategory;
    type Qual = PositioningAccuracy;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(SoundSpeedPositive));
        axioms.push(Box::new(RangeNonNegative));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<AcousticCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        AcousticOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn positioning_accuracy_is_typed() {
        use crate::formal::math::quantity::dimension::Dimension;
        let q = PositioningAccuracy;
        // LBL: an absolute LENGTH accuracy within the transponder baseline.
        match q.get(&AcousticConcept::LBL) {
            Some(AccuracyModel::Absolute(range)) => {
                assert_eq!(range.dimension(), Dimension::LENGTH);
                assert!(range.contains(&Quantity::from_unit(0.05, &METER)));
            }
            other => panic!("LBL should be Absolute LENGTH accuracy, got {other:?}"),
        }
        // USBL / SBL: a dimensionless fraction of slant range.
        for sys in [AcousticConcept::USBL, AcousticConcept::SBL] {
            match q.get(&sys) {
                Some(AccuracyModel::FractionOfSlantRange(range)) => {
                    assert!(range.dimension().is_dimensionless());
                }
                other => panic!("{sys:?} should be FractionOfSlantRange, got {other:?}"),
            }
        }
    }
}
