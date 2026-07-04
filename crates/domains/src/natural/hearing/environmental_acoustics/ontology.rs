//! Environmental acoustics — room acoustics, noise exposure, and
//! soundscape ecology.
//!
//! # Literature
//!
//! - **Kuttruff (2009)** *Room Acoustics* (5th ed.), Spon Press.
//! - **Sabine (1922)** *Collected Papers on Acoustics* — RT60 = 0.161 V/A.
//! - **OSHA (29 CFR 1910.95)** — Occupational Noise Exposure: 90 dBA TWA,
//!   5 dB exchange rate.
//! - **NIOSH (1998)** *Criteria for a Recommended Standard: Occupational
//!   Noise Exposure* — 85 dBA TWA, 3 dB exchange rate.
//! - **ISO 3382-1:2009** — Acoustics: Measurement of room acoustic
//!   parameters.
//! - **Schafer (1977)** *The Soundscape: Our Sonic Environment and the
//!   Tuning of the World*.

use pr4xis::ontology::{Axiom, Ontology, Quality, QualityKind};

use crate::formal::math::quantity::level::{
    LogarithmicLevel, LogarithmicLevelReferenceConcept as Ref,
};
use crate::formal::math::quantity::unit::SECOND;
use crate::formal::math::quantity::value::Quantity;

pr4xis::ontology! {
    name: "Environment",
    source: "Kuttruff (2009) Room Acoustics 5th ed.; Sabine (1922) Collected Papers on Acoustics; OSHA 29 CFR 1910.95; NIOSH (1998) Occupational Noise Exposure; ISO 3382-1:2009; Schafer (1977) The Soundscape",

    concepts: [
        ReverberationTime, RT60, EarlyDecayTime, Clarity, Definition,
        SpeechTransmissionIndex, CenterTime, LateralFraction,
        SoundAbsorption, AbsorptionCoefficient, SoundDiffusion,
        SoundInsulation, TransmissionLoss, FlankingTransmission,
        SoundPressureLevel, AWeighting, CWeighting, EquivalentContinuousLevel,
        PeakSoundLevel, SoundExposureLevel, NoiseDose, TimeWeightedAverage,
        OSHALimit, NIOSHLimit, ExchangeRate, PermissibleExposureLimit, ActionLevel,
        Soundscape, Keynote, SoundSignal, Soundmark, BackgroundNoise,
        SpeechRoom, MusicHall, WorshipSpace,
        SoundLevelMeter, Dosimeter, CalibrationSource,
        RoomParameter, AcousticProperty, NoiseMeasure, NoiseStandard,
        SoundscapeElement, MeasurementDevice, RoomType,
        // Events
        NoiseSourceEvent, SoundPropagation, WorkerExposure, DoseAccumulation,
        ThresholdShift, HearingDamageRisk, RoomReverberation,
        SpeechIntelligibilityReduction, EnvironmentEvent,
    ],

    labels: {
        ReverberationTime: ("en", "Reverberation time",
            "Sabine (1922): time for 60 dB sound-energy decay after source-off."),
        RT60: ("en", "RT60",
            "Sabine (1922): standard 60 dB reverberation time."),
        EarlyDecayTime: ("en", "Early decay time",
            "ISO 3382-1:2009: T over the 0..-10 dB decay range, extrapolated to 60 dB."),
        Clarity: ("en", "Clarity",
            "ISO 3382-1:2009: C80 — ratio of early (<80 ms) to late energy in dB."),
        Definition: ("en", "Definition",
            "ISO 3382-1:2009: D50 — ratio of early (<50 ms) to total energy."),
        SpeechTransmissionIndex: ("en", "Speech transmission index",
            "ISO 3382-1:2009: 0..1 metric for predicted speech intelligibility."),
        CenterTime: ("en", "Center time",
            "ISO 3382-1:2009: T_s — temporal centre of the impulse response."),
        LateralFraction: ("en", "Lateral fraction",
            "ISO 3382-1:2009: LF — fraction of early energy from lateral directions."),
        SoundAbsorption: ("en", "Sound absorption",
            "Kuttruff (2009) §3: conversion of acoustic energy into heat in a material."),
        AbsorptionCoefficient: ("en", "Absorption coefficient",
            "Kuttruff (2009) §3: α — fraction of incident energy absorbed."),
        SoundDiffusion: ("en", "Sound diffusion",
            "Kuttruff (2009) §6: scattering of sound to create uniform energy distribution."),
        SoundInsulation: ("en", "Sound insulation",
            "Kuttruff (2009) §9: building-element rejection of transmitted sound."),
        TransmissionLoss: ("en", "Transmission loss",
            "Kuttruff (2009) §9: dB attenuation through a partition."),
        FlankingTransmission: ("en", "Flanking transmission",
            "Kuttruff (2009) §9: sound bypassing a partition via adjacent paths."),
        SoundPressureLevel: ("en", "Sound pressure level",
            "Kinsler et al. (2000) §5.10: 20 log10(p/p_ref), p_ref = 20 µPa."),
        AWeighting: ("en", "A-weighting",
            "IEC 61672-1: human-hearing-approximating frequency weighting."),
        CWeighting: ("en", "C-weighting",
            "IEC 61672-1: nearly-flat weighting for peak / high-level measurements."),
        EquivalentContinuousLevel: ("en", "Equivalent continuous level",
            "OSHA 29 CFR 1910.95: L_eq — energy-equivalent steady level."),
        PeakSoundLevel: ("en", "Peak sound level",
            "IEC 61672-1: instantaneous-pressure peak SPL."),
        SoundExposureLevel: ("en", "Sound exposure level",
            "ISO 1996-1: SEL — single-event total exposure normalised to 1 s."),
        NoiseDose: ("en", "Noise dose",
            "OSHA 29 CFR 1910.95: cumulative noise exposure as % of PEL."),
        TimeWeightedAverage: ("en", "Time-weighted average",
            "OSHA 29 CFR 1910.95: TWA — 8-hour equivalent exposure."),
        OSHALimit: ("en", "OSHA limit",
            "OSHA 29 CFR 1910.95: 90 dBA / 8 hr TWA."),
        NIOSHLimit: ("en", "NIOSH limit",
            "NIOSH (1998): 85 dBA / 8 hr TWA."),
        ExchangeRate: ("en", "Exchange rate",
            "OSHA/NIOSH: dB increase that halves the allowed exposure time."),
        PermissibleExposureLimit: ("en", "Permissible exposure limit",
            "OSHA 29 CFR 1910.95: regulatory ceiling on noise exposure."),
        ActionLevel: ("en", "Action level",
            "OSHA 29 CFR 1910.95: 85 dBA TWA triggering hearing-conservation programme."),
        Soundscape: ("en", "Soundscape",
            "Schafer (1977): the acoustic environment as perceived."),
        Keynote: ("en", "Keynote",
            "Schafer (1977): omnipresent background sound establishing acoustic context."),
        SoundSignal: ("en", "Sound signal",
            "Schafer (1977): foregrounded sound carrying information."),
        Soundmark: ("en", "Soundmark",
            "Schafer (1977): community-identifying sound."),
        BackgroundNoise: ("en", "Background noise",
            "Schafer (1977): residual ambient noise level."),
        SpeechRoom: ("en", "Speech room",
            "Kuttruff (2009) §9: room optimised for speech clarity (short RT60)."),
        MusicHall: ("en", "Music hall",
            "Kuttruff (2009) §9: concert hall, RT60 ~ 1.5-2 s."),
        WorshipSpace: ("en", "Worship space",
            "Kuttruff (2009) §9: church / cathedral with long RT60."),
        SoundLevelMeter: ("en", "Sound level meter",
            "IEC 61672-1: instrument measuring SPL with prescribed weighting."),
        Dosimeter: ("en", "Dosimeter",
            "OSHA 29 CFR 1910.95: wearable noise-dose accumulator."),
        CalibrationSource: ("en", "Calibration source",
            "IEC 60942: reference acoustic level source for SLM calibration."),
        RoomParameter: ("en", "Room parameter",
            "ISO 3382-1:2009: umbrella for room-acoustic metrics."),
        AcousticProperty: ("en", "Acoustic property",
            "Kuttruff (2009): umbrella for material/element acoustic descriptors."),
        NoiseMeasure: ("en", "Noise measure",
            "ISO 1996-1: umbrella for environmental-noise metrics."),
        NoiseStandard: ("en", "Noise standard",
            "OSHA / NIOSH: umbrella for regulatory limits."),
        SoundscapeElement: ("en", "Soundscape element",
            "Schafer (1977): umbrella for soundscape-ontology categories."),
        MeasurementDevice: ("en", "Measurement device",
            "IEC 61672-1: umbrella for acoustic instruments."),
        RoomType: ("en", "Room type",
            "Kuttruff (2009): umbrella for room functional categories."),
        NoiseSourceEvent: ("en", "Noise source",
            "OSHA 29 CFR 1910.95: event — the originating noise generator."),
        SoundPropagation: ("en", "Sound propagation",
            "Kinsler et al. (2000): event — acoustic energy travelling through the environment."),
        WorkerExposure: ("en", "Worker exposure",
            "OSHA 29 CFR 1910.95: event — sound reaching the worker."),
        DoseAccumulation: ("en", "Dose accumulation",
            "OSHA 29 CFR 1910.95: event — cumulative dose increasing over the shift."),
        ThresholdShift: ("en", "Threshold shift",
            "NIOSH (1998): event — temporary or permanent hearing-threshold elevation."),
        HearingDamageRisk: ("en", "Hearing damage risk",
            "NIOSH (1998): terminal event — elevated probability of NIHL."),
        RoomReverberation: ("en", "Room reverberation",
            "Kuttruff (2009): event — sound persisting in the room after source-off."),
        SpeechIntelligibilityReduction: ("en", "Speech intelligibility reduction",
            "ISO 3382-3:2012: event — STI reduction from reverberation."),
        EnvironmentEvent: ("en", "Environment event",
            "Schafer (1977): umbrella concept for any environmental-acoustics perdurant."),
    },

    is_a: [
        (ReverberationTime, RoomParameter), (RT60, ReverberationTime),
        (EarlyDecayTime, RoomParameter), (Clarity, RoomParameter),
        (Definition, RoomParameter), (SpeechTransmissionIndex, RoomParameter),
        (CenterTime, RoomParameter), (LateralFraction, RoomParameter),
        (SoundAbsorption, AcousticProperty), (AbsorptionCoefficient, AcousticProperty),
        (SoundDiffusion, AcousticProperty), (SoundInsulation, AcousticProperty),
        (TransmissionLoss, AcousticProperty), (FlankingTransmission, AcousticProperty),
        (SoundPressureLevel, NoiseMeasure), (AWeighting, NoiseMeasure),
        (CWeighting, NoiseMeasure), (EquivalentContinuousLevel, NoiseMeasure),
        (PeakSoundLevel, NoiseMeasure), (SoundExposureLevel, NoiseMeasure),
        (NoiseDose, NoiseMeasure), (TimeWeightedAverage, NoiseMeasure),
        (OSHALimit, NoiseStandard), (NIOSHLimit, NoiseStandard),
        (ExchangeRate, NoiseStandard), (PermissibleExposureLimit, NoiseStandard),
        (ActionLevel, NoiseStandard),
        (Keynote, SoundscapeElement), (SoundSignal, SoundscapeElement),
        (Soundmark, SoundscapeElement), (BackgroundNoise, SoundscapeElement),
        (SpeechRoom, RoomType), (MusicHall, RoomType), (WorshipSpace, RoomType),
        (SoundLevelMeter, MeasurementDevice), (Dosimeter, MeasurementDevice),
        (CalibrationSource, MeasurementDevice),
        (NoiseSourceEvent, EnvironmentEvent), (SoundPropagation, EnvironmentEvent),
        (WorkerExposure, EnvironmentEvent), (DoseAccumulation, EnvironmentEvent),
        (ThresholdShift, EnvironmentEvent), (HearingDamageRisk, EnvironmentEvent),
        (RoomReverberation, EnvironmentEvent),
        (SpeechIntelligibilityReduction, EnvironmentEvent),
    ],

    causes: [
        (NoiseSourceEvent, SoundPropagation),
        (SoundPropagation, WorkerExposure),
        (WorkerExposure, DoseAccumulation),
        (DoseAccumulation, ThresholdShift),
        (ThresholdShift, HearingDamageRisk),
        (SoundPropagation, RoomReverberation),
        (RoomReverberation, SpeechIntelligibilityReduction),
    ],

    opposes: [
        (SoundAbsorption, SoundDiffusion), (SoundDiffusion, SoundAbsorption),
        (AWeighting, CWeighting), (CWeighting, AWeighting),
    ],
}

#[derive(Debug, Clone)]
pub struct RegulatoryLimitDB;
impl Quality for RegulatoryLimitDB {
    type Individual = EnvironmentConcept;
    type Value = LogarithmicLevel;
    const KIND: QualityKind = QualityKind::Physical;
    fn get(&self, individual: &EnvironmentConcept) -> Option<LogarithmicLevel> {
        use EnvironmentConcept::*;
        // Regulatory A-weighted noise-exposure ceiling — a dB SPL level in air
        // (re 20 µPa), OSHA 29 CFR 1910.95 / NIOSH (1998). A sound-pressure
        // level is logarithmic, not a linear Quantity.
        let db = match individual {
            OSHALimit => 90.0,
            NIOSHLimit => 85.0,
            PermissibleExposureLimit => 90.0,
            ActionLevel => 85.0,
            _ => return None,
        };
        Some(LogarithmicLevel::new(db, Ref::SoundPressureAir))
    }
}

#[derive(Debug, Clone)]
pub struct ExchangeRateDB;
impl Quality for ExchangeRateDB {
    type Individual = EnvironmentConcept;
    type Value = LogarithmicLevel;
    const KIND: QualityKind = QualityKind::Physical;
    fn get(&self, individual: &EnvironmentConcept) -> Option<LogarithmicLevel> {
        use EnvironmentConcept::*;
        // Exchange rate — the dB increment that halves the permitted exposure
        // time. Energy-based (equal-energy vs. equal-effect), so a power-ratio
        // dB (10·log₁₀), IEC 80000-15. Logarithmic, not a linear Quantity.
        let db = match individual {
            OSHALimit => 5.0,
            NIOSHLimit => 3.0,
            _ => return None,
        };
        Some(LogarithmicLevel::new(db, Ref::PowerRatio))
    }
}

#[derive(Debug, Clone)]
pub struct IdealRT60Seconds;
impl Quality for IdealRT60Seconds {
    type Individual = EnvironmentConcept;
    type Value = Quantity;
    const KIND: QualityKind = QualityKind::Physical;
    fn get(&self, individual: &EnvironmentConcept) -> Option<Quantity> {
        use EnvironmentConcept::*;
        let seconds = match individual {
            SpeechRoom => 0.5,
            MusicHall => 1.5,
            WorshipSpace => 2.0,
            _ => return None,
        };
        Some(Quantity::from_unit(seconds, &SECOND))
    }
}

fn effects_of(cause: EnvironmentConcept) -> Vec<EnvironmentConcept> {
    use pr4xis::category::{Arrow, Category};
    EnvironmentCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == EnvironmentRelationKind::Causation && m.source() == cause)
        .map(|m| m.target())
        .collect()
}

pub struct SpeechRoomShortestRT60;
impl Axiom for SpeechRoomShortestRT60 {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use EnvironmentConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        // All three RT60 values share the SECOND unit, so a bare `.value`
        // compare is dimensionally sound.
        let s = IdealRT60Seconds
            .get(&SpeechRoom)
            .map(|q| q.value)
            .unwrap_or(0.0);
        let m = IdealRT60Seconds
            .get(&MusicHall)
            .map(|q| q.value)
            .unwrap_or(0.0);
        let w = IdealRT60Seconds
            .get(&WorshipSpace)
            .map(|q| q.value)
            .unwrap_or(0.0);
        if s < m && m < w {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "SpeechRoomShortestRT60",
        "speech rooms have shortest ideal RT60",
        "Kuttruff (2009) Room Acoustics §9"
    );
}
pr4xis::register_axiom!(SpeechRoomShortestRT60, "Kuttruff (2009) Room Acoustics §9");

pub struct NIOSHStricterThanOSHA;
impl Axiom for NIOSHStricterThanOSHA {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use EnvironmentConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        // Both limits are dB SPL (SoundPressureAir), so compare decibels.
        let n = RegulatoryLimitDB
            .get(&NIOSHLimit)
            .map(|l| l.decibels)
            .unwrap_or(0.0);
        let o = RegulatoryLimitDB
            .get(&OSHALimit)
            .map(|l| l.decibels)
            .unwrap_or(0.0);
        if n < o {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "NIOSHStricterThanOSHA",
        "NIOSH limit (85 dBA) is stricter than OSHA (90 dBA)",
        "NIOSH (1998) Occupational Noise Exposure Recommended Criteria"
    );
}
pr4xis::register_axiom!(
    NIOSHStricterThanOSHA,
    "NIOSH (1998) Occupational Noise Exposure Recommended Criteria"
);

pub struct NIOSHUsesEqualEnergy;
impl Axiom for NIOSHUsesEqualEnergy {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use EnvironmentConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        // Both exchange rates are power-ratio dB (PowerRatio), so compare decibels.
        let n = ExchangeRateDB
            .get(&NIOSHLimit)
            .map(|l| l.decibels)
            .unwrap_or(0.0);
        let o = ExchangeRateDB
            .get(&OSHALimit)
            .map(|l| l.decibels)
            .unwrap_or(0.0);
        if n < o {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "NIOSHUsesEqualEnergy",
        "NIOSH uses 3 dB exchange rate, stricter than OSHA's 5 dB",
        "NIOSH (1998) Occupational Noise Exposure Recommended Criteria"
    );
}
pr4xis::register_axiom!(
    NIOSHUsesEqualEnergy,
    "NIOSH (1998) Occupational Noise Exposure Recommended Criteria"
);

pub struct NoiseCausesHearingDamage;
impl Axiom for NoiseCausesHearingDamage {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use EnvironmentConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if effects_of(NoiseSourceEvent).contains(&HearingDamageRisk) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "NoiseCausesHearingDamage",
        "noise source transitively causes hearing damage risk",
        "NIOSH (1998) Occupational Noise Exposure Recommended Criteria"
    );
}
pr4xis::register_axiom!(
    NoiseCausesHearingDamage,
    "NIOSH (1998) Occupational Noise Exposure Recommended Criteria"
);

impl Ontology for EnvironmentOntology {
    type Cat = EnvironmentCategory;
    type Qual = RegulatoryLimitDB;
    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut a = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        a.push(Box::new(SpeechRoomShortestRT60));
        a.push(Box::new(NIOSHStricterThanOSHA));
        a.push(Box::new(NIOSHUsesEqualEnergy));
        a.push(Box::new(NoiseCausesHearingDamage));
        a
    }
}

// Back-compat aliases.
pub use EnvironmentCategory as EnvironmentalAcousticsCategory;
pub use EnvironmentConcept as EnvironmentEntity;
pub use EnvironmentOntology as EnvironmentalAcousticsOntology;
pub use EnvironmentRelationKind as EnvironmentalAcousticsCategoryRelationKind;

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<EnvironmentCategory>();
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        EnvironmentOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn niosh_stricter_than_osha() {
        assert!(NIOSHStricterThanOSHA.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn niosh_equal_energy() {
        assert!(NIOSHUsesEqualEnergy.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn speech_room_shortest() {
        assert!(SpeechRoomShortestRT60.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn noise_causes_damage() {
        assert!(NoiseCausesHearingDamage.verify().is_ok());
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in EnvironmentCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }
        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in EnvironmentOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
}
