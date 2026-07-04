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
