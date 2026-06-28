//! Acoustics — physics of sound: waves, propagation media, acoustic
//! phenomena, and the source→receiver causal chain.
//!
//! # Literature
//!
//! - **Kinsler et al. (2000)** *Fundamentals of Acoustics* (4th ed.), Wiley
//!   — wave equations, media, impedance, reflection/refraction/diffraction.
//! - **Pierce (2019)** *Acoustics: An Introduction to Its Physical
//!   Principles and Applications* — comprehensive linear acoustics text.
//! - **Stenfelt & Goode (2005)** "Bone-Conducted Sound: Physiological and
//!   Clinical Aspects", *Otology & Neurotology* 26(6):1245-1261 — bone vs
//!   air conduction impedance values.
//! - **von Bekesy (1960)** *Experiments in Hearing*, McGraw-Hill — early
//!   impedance and travelling-wave measurements.
//! - **Mow & Huiskes (2005)** *Basic Orthopaedic Biomechanics &
//!   Mechano-Biology* (3rd ed.) — cartilage acoustic properties.
//!
//! # Design
//!
//! Per `feedback_one_ontology_per_module`, the dual-enum (entities +
//! parallel `AcousticCausalEvent`) has been merged into a single concept
//! list. Events are first-class concepts under the umbrella
//! `AcousticEvent`, attached to entities via Causation edges. This keeps
//! the source→receiver chain (Kinsler 2000) as named morphisms rather
//! than a separate hand-rolled category.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Acoustics",
    source: "Kinsler et al. (2000) Fundamentals of Acoustics 4th ed.; Pierce (2019) Acoustics; Stenfelt & Goode (2005) Otology & Neurotology 26(6):1245; von Bekesy (1960) Experiments in Hearing; Mow & Huiskes (2005) Basic Orthopaedic Biomechanics",

    concepts: [
        // Wave properties
        Frequency, Amplitude, Wavelength, Phase, Intensity,
        // Wave types
        SoundWave, LongitudinalWave, TransverseWave, ShearWave,
        // Propagation media
        Air, Water, CorticalBone, CancellousBone, SoftTissue, Cartilage, Fluid,
        // Acoustic phenomena
        Resonance, Reflection, Refraction, Diffraction, Absorption, Attenuation,
        ImpedanceMismatch,
        // Abstract categories
        Wave, Medium, WaveProperty, AcousticPhenomenon, Solid, BoneTissue,
        // Causal events (Kinsler 2000 §1)
        SourceVibration, MediumCoupling, WavePropagation, BoundaryEncounter,
        ImpedanceTransition, EnergyReflection, EnergyTransmission, EnergyAbsorption,
        WaveAttenuation, ResonantAmplification, ReceiverExcitation,
        AcousticEvent,
    ],

    labels: {
        Frequency: ("en", "Frequency",
            "Kinsler et al. (2000) §1.3: number of cycles per second of a periodic acoustic wave (Hz)."),
        Amplitude: ("en", "Amplitude",
            "Kinsler et al. (2000) §1.3: peak pressure deviation of an acoustic wave from ambient."),
        Wavelength: ("en", "Wavelength",
            "Kinsler et al. (2000) §1.3: spatial period of the wave — distance between successive equal-phase points."),
        Phase: ("en", "Phase",
            "Kinsler et al. (2000) §1.3: angular offset of a sinusoidal component, modulo 2π."),
        Intensity: ("en", "Intensity",
            "Kinsler et al. (2000) §5.10: time-averaged acoustic power per unit area (W/m²)."),
        SoundWave: ("en", "Sound wave",
            "Kinsler et al. (2000) §5.1: a longitudinal pressure wave in a fluid medium."),
        LongitudinalWave: ("en", "Longitudinal wave",
            "Kinsler et al. (2000) §5.1: particle motion parallel to propagation direction."),
        TransverseWave: ("en", "Transverse wave",
            "Kinsler et al. (2000) §6.1: particle motion perpendicular to propagation; supported only in solids."),
        ShearWave: ("en", "Shear wave",
            "Kinsler et al. (2000) §6.1: transverse wave in a solid medium, restored by shear elasticity."),
        Air: ("en", "Air",
            "Kinsler et al. (2000) Table 5.1: c ≈ 343 m/s @ 20°C, Z₀ ≈ 413 Pa·s/m."),
        Water: ("en", "Water",
            "Kinsler et al. (2000) Table 5.1: c ≈ 1480 m/s @ 20°C."),
        CorticalBone: ("en", "Cortical bone",
            "Stenfelt & Goode (2005): dense outer bone; c ≈ 4080 m/s, Z ≈ 7.38·10⁶ Pa·s/m."),
        CancellousBone: ("en", "Cancellous bone",
            "Stenfelt & Goode (2005): porous trabecular bone; c ≈ 1800 m/s."),
        SoftTissue: ("en", "Soft tissue",
            "Kinsler et al. (2000): acoustically near-water; c ≈ 1540 m/s."),
        Cartilage: ("en", "Cartilage",
            "Mow & Huiskes (2005): c ≈ 1665 m/s."),
        Fluid: ("en", "Fluid",
            "Kinsler et al. (2000) §5.1: a medium with no shear elasticity — supports only longitudinal waves."),
        Resonance: ("en", "Resonance",
            "Kinsler et al. (2000) §3.2: amplitude amplification at a system's natural frequency."),
        Reflection: ("en", "Reflection",
            "Kinsler et al. (2000) §6.3: wave energy returned at an impedance boundary."),
        Refraction: ("en", "Refraction",
            "Kinsler et al. (2000) §6.6: change in propagation direction at a boundary."),
        Diffraction: ("en", "Diffraction",
            "Kinsler et al. (2000) §6.10: wave bending around obstacles smaller than λ."),
        Absorption: ("en", "Absorption",
            "Kinsler et al. (2000) §7.5: dissipation of acoustic energy as heat."),
        Attenuation: ("en", "Attenuation",
            "Kinsler et al. (2000) §7.5: decrease in wave amplitude with distance."),
        ImpedanceMismatch: ("en", "Impedance mismatch",
            "Kinsler et al. (2000) §6.3: difference in Z = ρc across a boundary causing reflection."),
        Wave: ("en", "Wave",
            "Kinsler et al. (2000) §5.1: a disturbance propagating through a medium."),
        Medium: ("en", "Medium",
            "Kinsler et al. (2000) §5.1: a material substrate that supports wave propagation."),
        WaveProperty: ("en", "Wave property",
            "Kinsler et al. (2000) §1.3: a scalar parameter characterizing a wave (frequency, amplitude, …)."),
        AcousticPhenomenon: ("en", "Acoustic phenomenon",
            "Kinsler et al. (2000) §6: emergent boundary or propagation effect."),
        Solid: ("en", "Solid",
            "Kinsler et al. (2000) §6.1: a medium with non-zero shear modulus."),
        BoneTissue: ("en", "Bone tissue",
            "Stenfelt & Goode (2005): mineralized solid biological tissue."),
        SourceVibration: ("en", "Source vibration",
            "Kinsler et al. (2000) §7.1: the originating mechanical oscillation that excites the medium."),
        MediumCoupling: ("en", "Medium coupling",
            "Kinsler et al. (2000) §7.1: transfer of source motion into propagating wave energy."),
        WavePropagation: ("en", "Wave propagation",
            "Kinsler et al. (2000) §5.5: outward travel of acoustic energy through the medium."),
        BoundaryEncounter: ("en", "Boundary encounter",
            "Kinsler et al. (2000) §6.3: arrival of a wave at an impedance discontinuity."),
        ImpedanceTransition: ("en", "Impedance transition",
            "Kinsler et al. (2000) §6.3: the boundary-driven splitting of energy into reflected and transmitted parts."),
        EnergyReflection: ("en", "Energy reflection",
            "Kinsler et al. (2000) §6.3: return of acoustic energy at a Z mismatch."),
        EnergyTransmission: ("en", "Energy transmission",
            "Kinsler et al. (2000) §6.3: passage of acoustic energy across a boundary."),
        EnergyAbsorption: ("en", "Energy absorption",
            "Kinsler et al. (2000) §7.5: thermalisation of acoustic energy at the boundary or in the medium."),
        WaveAttenuation: ("en", "Wave attenuation",
            "Kinsler et al. (2000) §7.5: progressive loss of amplitude during propagation."),
        ResonantAmplification: ("en", "Resonant amplification",
            "Kinsler et al. (2000) §3.2: build-up of amplitude at the receiver's natural frequency."),
        ReceiverExcitation: ("en", "Receiver excitation",
            "Kinsler et al. (2000) §7.1: the terminal event — coupling of acoustic energy into a receiver."),
        AcousticEvent: ("en", "Acoustic event",
            "Kinsler et al. (2000) §7.1: any perdurant in the source→receiver acoustic chain (umbrella concept)."),
    },

    is_a: [
        // Wave types
        (SoundWave, Wave), (LongitudinalWave, Wave),
        (TransverseWave, Wave), (ShearWave, Wave),
        (SoundWave, LongitudinalWave),
        // Media
        (Air, Medium), (Water, Medium), (CorticalBone, Medium),
        (CancellousBone, Medium), (SoftTissue, Medium),
        (Cartilage, Medium), (Fluid, Medium),
        (CorticalBone, BoneTissue), (CancellousBone, BoneTissue),
        (BoneTissue, Solid), (Cartilage, Solid),
        (Air, Fluid), (Water, Fluid),
        (Solid, Medium),
        // Wave properties
        (Frequency, WaveProperty), (Amplitude, WaveProperty),
        (Wavelength, WaveProperty), (Phase, WaveProperty), (Intensity, WaveProperty),
        // Acoustic phenomena
        (Resonance, AcousticPhenomenon), (Reflection, AcousticPhenomenon),
        (Refraction, AcousticPhenomenon), (Diffraction, AcousticPhenomenon),
        (Absorption, AcousticPhenomenon), (Attenuation, AcousticPhenomenon),
        (ImpedanceMismatch, AcousticPhenomenon),
        // Events
        (SourceVibration, AcousticEvent), (MediumCoupling, AcousticEvent),
        (WavePropagation, AcousticEvent), (BoundaryEncounter, AcousticEvent),
        (ImpedanceTransition, AcousticEvent), (EnergyReflection, AcousticEvent),
        (EnergyTransmission, AcousticEvent), (EnergyAbsorption, AcousticEvent),
        (WaveAttenuation, AcousticEvent), (ResonantAmplification, AcousticEvent),
        (ReceiverExcitation, AcousticEvent),
    ],

    has_a: [
        // A sound wave has-a frequency, amplitude, wavelength, phase, intensity
        (SoundWave, Frequency), (SoundWave, Amplitude), (SoundWave, Wavelength),
        (SoundWave, Phase), (SoundWave, Intensity),
    ],

    causes: [
        // Kinsler et al. (2000) §7.1: source → medium → propagation → boundary → receiver
        (SourceVibration, MediumCoupling),
        (MediumCoupling, WavePropagation),
        (WavePropagation, BoundaryEncounter),
        (WavePropagation, WaveAttenuation),
        (BoundaryEncounter, ImpedanceTransition),
        (ImpedanceTransition, EnergyReflection),
        (ImpedanceTransition, EnergyTransmission),
        (ImpedanceTransition, EnergyAbsorption),
        (EnergyTransmission, ResonantAmplification),
        (EnergyTransmission, ReceiverExcitation),
        (ResonantAmplification, ReceiverExcitation),
    ],

    opposes: [
        (Reflection, Refraction), (Refraction, Reflection),
        (Absorption, Resonance), (Resonance, Absorption),
        (LongitudinalWave, TransverseWave), (TransverseWave, LongitudinalWave),
    ],
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Speed of sound (m/s). Kinsler et al. (2000) Table 5.1; bone values from
/// Stenfelt & Goode (2005).
#[derive(Debug, Clone)]
pub struct SpeedOfSound;
impl Quality for SpeedOfSound {
    type Individual = AcousticsConcept;
    type Value = f64;
    fn get(&self, individual: &AcousticsConcept) -> Option<f64> {
        use AcousticsConcept::*;
        match individual {
            Air => Some(343.0),
            Water => Some(1480.0),
            CorticalBone => Some(4080.0),
            CancellousBone => Some(1800.0),
            SoftTissue => Some(1540.0),
            Cartilage => Some(1665.0),
            _ => None,
        }
    }
}

/// Acoustic impedance Z = ρc (Pa·s/m = rayl). Kinsler et al. (2000);
/// Stenfelt & Goode (2005).
#[derive(Debug, Clone)]
pub struct AcousticImpedance;
impl Quality for AcousticImpedance {
    type Individual = AcousticsConcept;
    type Value = f64;
    fn get(&self, individual: &AcousticsConcept) -> Option<f64> {
        use AcousticsConcept::*;
        match individual {
            Air => Some(413.0),
            Water => Some(1.48e6),
            CorticalBone => Some(7.38e6),
            CancellousBone => Some(1.44e6),
            SoftTissue => Some(1.63e6),
            Cartilage => Some(1.83e6),
            _ => None,
        }
    }
}

/// Whether the medium supports shear waves. Kinsler et al. (2000) §6.1.
#[derive(Debug, Clone)]
pub struct SupportsShearWaves;
impl Quality for SupportsShearWaves {
    type Individual = AcousticsConcept;
    type Value = bool;
    fn get(&self, individual: &AcousticsConcept) -> Option<bool> {
        use AcousticsConcept::*;
        match individual {
            Air | Water => Some(false),
            CorticalBone | CancellousBone | Cartilage => Some(true),
            SoftTissue => Some(false),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediumState {
    Gas,
    Liquid,
    SolidState,
}

#[derive(Debug, Clone)]
pub struct MediumPhase;
impl Quality for MediumPhase {
    type Individual = AcousticsConcept;
    type Value = MediumState;
    fn get(&self, individual: &AcousticsConcept) -> Option<MediumState> {
        use AcousticsConcept::*;
        match individual {
            Air => Some(MediumState::Gas),
            Water => Some(MediumState::Liquid),
            CorticalBone | CancellousBone | Cartilage => Some(MediumState::SolidState),
            SoftTissue => Some(MediumState::Liquid),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers — kinded-morphism queries.
// ---------------------------------------------------------------------------

fn effects_of(cause: AcousticsConcept) -> Vec<AcousticsConcept> {
    use pr4xis::category::{Arrow, Category};
    AcousticsCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == AcousticsRelationKind::Causation && m.source() == cause)
        .map(|m| m.target())
        .collect()
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

pub struct BoneFasterThanAir;
impl Axiom for BoneFasterThanAir {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use AcousticsConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let s = SpeedOfSound;
        if s.get(&CorticalBone).unwrap_or(0.0) > s.get(&Air).unwrap_or(0.0) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "BoneFasterThanAir",
        "speed of sound in cortical bone exceeds speed in air",
        "Stenfelt & Goode (2005) Otology & Neurotology 26(6):1245"
    );
}
pr4xis::register_axiom!(
    BoneFasterThanAir,
    "Stenfelt & Goode (2005) Otology & Neurotology 26(6):1245"
);

pub struct BoneAirImpedanceMismatch;
impl Axiom for BoneAirImpedanceMismatch {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use AcousticsConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let z = AcousticImpedance;
        let bone = z.get(&CorticalBone).unwrap_or(0.0);
        let air = z.get(&Air).unwrap_or(1.0);
        if bone / air > 1000.0 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "BoneAirImpedanceMismatch",
        "bone acoustic impedance is at least 1000x air impedance",
        "Stenfelt & Goode (2005) Otology & Neurotology 26(6):1245"
    );
}
pr4xis::register_axiom!(
    BoneAirImpedanceMismatch,
    "Stenfelt & Goode (2005) Otology & Neurotology 26(6):1245"
);

pub struct SoftTissueMatchesWater;
impl Axiom for SoftTissueMatchesWater {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use AcousticsConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let z = AcousticImpedance;
        let tissue = z.get(&SoftTissue).unwrap_or(0.0);
        let water = z.get(&Water).unwrap_or(1.0);
        let r = tissue / water;
        if (0.85..=1.15).contains(&r) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "SoftTissueMatchesWater",
        "soft tissue impedance is within 15% of water",
        "Kinsler et al. (2000) Fundamentals of Acoustics 4th ed."
    );
}
pr4xis::register_axiom!(
    SoftTissueMatchesWater,
    "Kinsler et al. (2000) Fundamentals of Acoustics 4th ed."
);

pub struct OnlySolidsHaveShearWaves;
impl Axiom for OnlySolidsHaveShearWaves {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use AcousticsConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let s = SupportsShearWaves;
        let ok = s.get(&Air) == Some(false)
            && s.get(&Water) == Some(false)
            && s.get(&CorticalBone) == Some(true)
            && s.get(&CancellousBone) == Some(true)
            && s.get(&Cartilage) == Some(true);
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "OnlySolidsHaveShearWaves",
        "only solid media support shear waves",
        "Kinsler et al. (2000) Fundamentals of Acoustics §6.1"
    );
}
pr4xis::register_axiom!(
    OnlySolidsHaveShearWaves,
    "Kinsler et al. (2000) Fundamentals of Acoustics §6.1"
);

pub struct SourceCausesReceiver;
impl Axiom for SourceCausesReceiver {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use AcousticsConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if effects_of(SourceVibration).contains(&ReceiverExcitation) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "SourceCausesReceiver",
        "source vibration transitively causes receiver excitation",
        "Kinsler et al. (2000) Fundamentals of Acoustics §7.1"
    );
}
pr4xis::register_axiom!(
    SourceCausesReceiver,
    "Kinsler et al. (2000) Fundamentals of Acoustics §7.1"
);

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for AcousticsOntology {
    type Cat = AcousticsCategory;
    type Qual = SpeedOfSound;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(BoneFasterThanAir));
        axioms.push(Box::new(BoneAirImpedanceMismatch));
        axioms.push(Box::new(SoftTissueMatchesWater));
        axioms.push(Box::new(OnlySolidsHaveShearWaves));
        axioms.push(Box::new(SourceCausesReceiver));
        axioms
    }
}

// ---------------------------------------------------------------------------
// Backward-compatible type aliases for cross-functor migration.
// AcousticEntity / AcousticRelation are referenced by sibling functors.
// (Per `feedback_breaking_changes_ok` we could just rename, but several
// functors share these — keeping aliases reduces fanout churn.)
// ---------------------------------------------------------------------------

pub use AcousticsConcept as AcousticEntity;
pub use AcousticsRelation as AcousticRelation;
pub use AcousticsRelationKind as AcousticsCategoryRelationKind;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<AcousticsCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        AcousticsOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn bone_faster_than_air() {
        assert!(BoneFasterThanAir.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn bone_air_impedance_mismatch() {
        assert!(BoneAirImpedanceMismatch.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn soft_tissue_matches_water() {
        assert!(SoftTissueMatchesWater.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn only_solids_have_shear() {
        assert!(OnlySolidsHaveShearWaves.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn source_causes_receiver() {
        assert!(SourceCausesReceiver.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn impedance_values_ordered() {
        let z = AcousticImpedance;
        let air = z.get(&AcousticsConcept::Air).unwrap();
        let water = z.get(&AcousticsConcept::Water).unwrap();
        let bone = z.get(&AcousticsConcept::CorticalBone).unwrap();
        assert!(air < water);
        assert!(water < bone);
    }

    fn arb_concept() -> impl Strategy<Value = AcousticsConcept> {
        proptest::sample::select(AcousticsConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in AcousticsCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }
        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in AcousticsOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }
        #[test]
        fn prop_media_with_speed_have_impedance(c in arb_concept()) {
            if SpeedOfSound.get(&c).is_some() {
                prop_assert!(AcousticImpedance.get(&c).is_some());
            }
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_media_with_speed_have_impedance, Verifiable);
}
