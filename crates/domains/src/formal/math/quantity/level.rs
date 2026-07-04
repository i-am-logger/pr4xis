//! Logarithmic levels — decibel values as typed quantities relative to a
//! reference, NOT a bare `f64` and NOT a linear [`Quantity`](super::value::Quantity)
//! (a decibel is a *ratio on a logarithmic scale*, so it does not add or scale
//! linearly and does not carry an SI dimension the way a length does).
//!
//! # Literature
//!
//! - **IEC 80000-15** *Quantities and units — Part 15: Logarithmic and related
//!   quantities*: a *level* is the logarithm of the ratio of a quantity to a
//!   reference value of the same kind. **Power** quantities use `10·log₁₀`;
//!   **root-power (field)** quantities (sound pressure, voltage) use `20·log₁₀`.
//! - **ISO 80000-8** *Acoustics*: sound pressure level, dB re 20 µPa.
//! - **ISO 389** *Acoustics — Reference zero for the calibration of audiometric
//!   equipment*: the dB HL audiometric zero.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Whether a level measures a **power** quantity (`10·log₁₀`) or a **root-power
/// / field** quantity such as sound pressure or voltage (`20·log₁₀`).
/// IEC 80000-15.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelKind {
    /// Power / energy quantity — level = `10·log₁₀(P/P₀)`.
    Power,
    /// Root-power (field) quantity — level = `20·log₁₀(F/F₀)`.
    RootPower,
}

impl LevelKind {
    /// The decade factor: `10` for power, `20` for root-power (IEC 80000-15).
    pub fn decade_factor(self) -> f64 {
        match self {
            LevelKind::Power => 10.0,
            LevelKind::RootPower => 20.0,
        }
    }
}

pr4xis::ontology! {
    name: "LogarithmicLevelReference",
    source: "IEC 80000-15 Logarithmic and related quantities; ISO 80000-8 Acoustics; ISO 389 Reference zero for the calibration of audiometric equipment",

    concepts: [
        SoundPressureAir,
        HearingLevel,
        CarrierToNoiseDensity,
        PowerRatio,
        FieldRatio,
        Milliwatt,
    ],

    labels: {
        SoundPressureAir: ("en", "dB SPL (re 20 µPa)",
            "ISO 80000-8: sound pressure level in air, referenced to 20 µPa (nominal threshold of hearing). A root-power/field quantity — 20·log₁₀."),
        HearingLevel: ("en", "dB HL",
            "ISO 389: hearing level, referenced to the frequency-dependent audiometric zero (normal-hearing threshold). Root-power."),
        CarrierToNoiseDensity: ("en", "dB-Hz (C/N₀)",
            "Carrier-to-noise-density ratio — a power ratio per unit bandwidth; the standard measure of GNSS signal strength. Power."),
        PowerRatio: ("en", "dB (power ratio)",
            "IEC 80000-15: a dimensionless power ratio, 10·log₁₀(P/P₀), with no absolute reference."),
        FieldRatio: ("en", "dB (field ratio)",
            "IEC 80000-15: a dimensionless root-power/field ratio, 20·log₁₀(F/F₀)."),
        Milliwatt: ("en", "dBm (re 1 mW)",
            "Power level referenced to 1 milliwatt. A power quantity."),
    },
}

/// Quality: whether each reference measures a power or a root-power quantity —
/// this fixes the 10-vs-20-log decade factor (IEC 80000-15).
#[derive(Debug, Clone)]
pub struct ReferenceKind;

impl Quality for ReferenceKind {
    type Individual = LogarithmicLevelReferenceConcept;
    type Value = LevelKind;

    fn get(&self, r: &LogarithmicLevelReferenceConcept) -> Option<LevelKind> {
        use LogarithmicLevelReferenceConcept as R;
        Some(match r {
            R::SoundPressureAir | R::HearingLevel | R::FieldRatio => LevelKind::RootPower,
            R::CarrierToNoiseDensity | R::PowerRatio | R::Milliwatt => LevelKind::Power,
        })
    }
}

impl Ontology for LogarithmicLevelReferenceOntology {
    type Cat = LogarithmicLevelReferenceCategory;
    type Qual = ReferenceKind;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(PowerLevelDecadeIsTenDecibels));
        axioms.push(Box::new(FieldLevelDecadeIsTwentyDecibels));
        axioms.push(Box::new(LevelsAddWhenRatiosMultiply));
        axioms
    }
}

/// A logarithmic level — a decibel value relative to a typed reference
/// (IEC 80000-15). Carries its reference so the 10-vs-20-log factor and the
/// physical meaning are explicit; a dB figure is neither a bare `f64` nor a
/// linear `Quantity`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogarithmicLevel {
    pub decibels: f64,
    pub reference: LogarithmicLevelReferenceConcept,
}

impl LogarithmicLevel {
    pub fn new(decibels: f64, reference: LogarithmicLevelReferenceConcept) -> Self {
        Self {
            decibels,
            reference,
        }
    }

    /// The level's power/field kind (IEC 80000-15).
    pub fn kind(&self) -> LevelKind {
        ReferenceKind
            .get(&self.reference)
            .unwrap_or(LevelKind::Power)
    }

    /// Linear ratio to the reference: `10^(dB/10)` for power, `10^(dB/20)` for field.
    pub fn linear_ratio(&self) -> f64 {
        10f64.powf(self.decibels / self.kind().decade_factor())
    }

    /// Build a level from a linear ratio to the reference (inverse of [`Self::linear_ratio`]).
    pub fn from_ratio(ratio: f64, reference: LogarithmicLevelReferenceConcept) -> Self {
        let factor = ReferenceKind
            .get(&reference)
            .unwrap_or(LevelKind::Power)
            .decade_factor();
        Self {
            decibels: factor * ratio.log10(),
            reference,
        }
    }

    /// Combine two levels of the **same** reference — decibels add. IEC 80000-15:
    /// levels add exactly when the underlying ratios multiply (the logarithm
    /// homomorphism `(ℝ, +) → (ℝ⁺, ×)`). Returns `None` for mismatched references
    /// (adding a dB-SPL to a dB-HL is meaningless).
    pub fn combine(&self, other: &LogarithmicLevel) -> Option<LogarithmicLevel> {
        if self.reference == other.reference {
            Some(LogarithmicLevel {
                decibels: self.decibels + other.decibels,
                reference: self.reference,
            })
        } else {
            None
        }
    }
}

/// Axiom: a power ratio of 10× is exactly 10 dB (`10·log₁₀ 10 = 10`), and the
/// round trip `from_ratio`→`linear_ratio` recovers the ratio. IEC 80000-15.
pub struct PowerLevelDecadeIsTenDecibels;

impl Axiom for PowerLevelDecadeIsTenDecibels {
    fn verify(&self) -> Verdict {
        let ten_db =
            LogarithmicLevel::from_ratio(10.0, LogarithmicLevelReferenceConcept::PowerRatio);
        let ok = (ten_db.decibels - 10.0).abs() < 1e-9
            && (LogarithmicLevel::new(10.0, LogarithmicLevelReferenceConcept::PowerRatio)
                .linear_ratio()
                - 10.0)
                .abs()
                < 1e-9;
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PowerLevelDecadeIsTenDecibels",
        "a 10× power ratio is 10 dB (10·log₁₀ 10 = 10); from_ratio/linear_ratio round-trip holds",
        "IEC 80000-15 Logarithmic and related quantities"
    );
}
pr4xis::register_axiom!(
    PowerLevelDecadeIsTenDecibels,
    "IEC 80000-15 Logarithmic and related quantities"
);

/// Axiom: a root-power (field) ratio of 10× is 20 dB, and a ~6.0206 dB field
/// level is a factor-of-2 ratio (`20·log₁₀ 2 ≈ 6.0206`). IEC 80000-15.
pub struct FieldLevelDecadeIsTwentyDecibels;

impl Axiom for FieldLevelDecadeIsTwentyDecibels {
    fn verify(&self) -> Verdict {
        let twenty = LogarithmicLevel::new(20.0, LogarithmicLevelReferenceConcept::FieldRatio);
        let six = LogarithmicLevel::new(
            20.0 * 2f64.log10(),
            LogarithmicLevelReferenceConcept::FieldRatio,
        );
        let ok =
            (twenty.linear_ratio() - 10.0).abs() < 1e-9 && (six.linear_ratio() - 2.0).abs() < 1e-9;
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "FieldLevelDecadeIsTwentyDecibels",
        "a 10× field ratio is 20 dB and a 6.0206 dB field level is a factor-of-2 ratio (20·log₁₀)",
        "IEC 80000-15 Logarithmic and related quantities"
    );
}
pr4xis::register_axiom!(
    FieldLevelDecadeIsTwentyDecibels,
    "IEC 80000-15 Logarithmic and related quantities"
);

/// Axiom: levels of the same reference add exactly when their linear ratios
/// multiply — the logarithm homomorphism `(ℝ, +) → (ℝ⁺, ×)`. IEC 80000-15.
pub struct LevelsAddWhenRatiosMultiply;

impl Axiom for LevelsAddWhenRatiosMultiply {
    fn verify(&self) -> Verdict {
        use LogarithmicLevelReferenceConcept as R;
        let fixtures = [(3.0, 6.0), (10.0, -4.0), (0.0, 20.0), (-12.0, 12.0)];
        let ok = [R::PowerRatio, R::FieldRatio, R::CarrierToNoiseDensity]
            .iter()
            .all(|&r| {
                fixtures.iter().all(|&(da, db)| {
                    let a = LogarithmicLevel::new(da, r);
                    let b = LogarithmicLevel::new(db, r);
                    match a.combine(&b) {
                        Some(sum) => {
                            let product = a.linear_ratio() * b.linear_ratio();
                            (sum.linear_ratio() - product).abs() < 1e-6 * product
                        }
                        None => false,
                    }
                })
            });
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "LevelsAddWhenRatiosMultiply",
        "combining two levels of the same reference adds their decibels and multiplies their linear ratios — the logarithm homomorphism (ℝ,+)→(ℝ⁺,×)",
        "IEC 80000-15 Logarithmic and related quantities; ITU-R V.574-5 (use of the decibel and the neper)"
    );
}
pr4xis::register_axiom!(
    LevelsAddWhenRatiosMultiply,
    "IEC 80000-15 Logarithmic and related quantities; ITU-R V.574-5 (use of the decibel and the neper)"
);

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use proptest::prelude::*;

    fn any_reference() -> impl Strategy<Value = LogarithmicLevelReferenceConcept> {
        proptest::sample::select(LogarithmicLevelReferenceConcept::variants())
    }

    proptest! {
        /// A dB level and its linear ratio are mutual inverses (round-trip).
        #[test]
        fn db_ratio_round_trip(db in -100.0f64..100.0, r in any_reference()) {
            let level = LogarithmicLevel::new(db, r);
            let recovered = LogarithmicLevel::from_ratio(level.linear_ratio(), r);
            prop_assert!((recovered.decibels - db).abs() < 1e-6);
        }

        /// Building from a positive ratio and reading it back recovers the ratio.
        #[test]
        fn ratio_db_round_trip(ratio in 1e-3f64..1e3, r in any_reference()) {
            let level = LogarithmicLevel::from_ratio(ratio, r);
            prop_assert!((level.linear_ratio() - ratio).abs() < 1e-6 * ratio.max(1.0));
        }

        /// The dB → linear map is strictly increasing (more dB ⇒ larger ratio).
        #[test]
        fn more_decibels_more_ratio(db in -50.0f64..50.0, delta in 0.1f64..10.0, r in any_reference()) {
            let lo = LogarithmicLevel::new(db, r).linear_ratio();
            let hi = LogarithmicLevel::new(db + delta, r).linear_ratio();
            prop_assert!(hi > lo);
        }

        /// Zero decibels is always a unit ratio, for every reference.
        #[test]
        fn zero_db_is_unit_ratio(r in any_reference()) {
            prop_assert!((LogarithmicLevel::new(0.0, r).linear_ratio() - 1.0).abs() < 1e-12);
        }

        /// Combining same-reference levels adds dB and multiplies ratios
        /// (the log homomorphism (ℝ,+)→(ℝ⁺,×)).
        #[test]
        fn levels_add_ratios_multiply(da in -50.0f64..50.0, db in -50.0f64..50.0, r in any_reference()) {
            let a = LogarithmicLevel::new(da, r);
            let b = LogarithmicLevel::new(db, r);
            let sum = a.combine(&b).unwrap();
            let product = a.linear_ratio() * b.linear_ratio();
            prop_assert!((sum.linear_ratio() - product).abs() < 1e-6 * product);
        }
    }

    pr4xis::register_praxis_value!(db_ratio_round_trip, Verifiable);
    pr4xis::register_praxis_value!(ratio_db_round_trip, Verifiable);
    pr4xis::register_praxis_value!(more_decibels_more_ratio, Verifiable);
    pr4xis::register_praxis_value!(zero_db_is_unit_ratio, Verifiable);
    pr4xis::register_praxis_value!(levels_add_ratios_multiply, Verifiable);
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<LogarithmicLevelReferenceCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        LogarithmicLevelReferenceOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn decade_axioms_hold() {
        assert!(PowerLevelDecadeIsTenDecibels.verify().is_ok());
        assert!(FieldLevelDecadeIsTwentyDecibels.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn spl_is_root_power() {
        let spl = LogarithmicLevel::new(94.0, LogarithmicLevelReferenceConcept::SoundPressureAir);
        assert_eq!(spl.kind(), LevelKind::RootPower);
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn zero_db_is_unit_ratio() {
        for r in LogarithmicLevelReferenceConcept::variants() {
            let level = LogarithmicLevel::new(0.0, r);
            assert!((level.linear_ratio() - 1.0).abs() < 1e-12);
        }
    }
}
