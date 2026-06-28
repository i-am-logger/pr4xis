//! Acoustics — wave-propagation ontology for biomedical bone-conduction work.
//!
//! Entities cover wave properties, acoustic impedance, conduction paths,
//! transducers, and biological media. Causal events trace the chain from an
//! electrical drive signal through transducer activation, surface oscillation
//! and wave generation into a medium, across an impedance boundary (which
//! branches into partial reflection and partial transmission), and on into
//! deep-tissue penetration via bone coupling.
//!
//! Per `feedback_one_ontology_per_module` the original split between
//! `AcousticsEntity` and `AcousticsCausalEvent` has been merged: events are
//! first-class concepts subsumed by the `AcousticEvent` umbrella.
//!
//! # Literature
//!
//! - **Stenfelt (2005)** "Bone-Conducted Sound: Physiological and Clinical
//!   Aspects", *Otology & Neurotology* 26(6), 1245–1261 — characterises
//!   the bone-conduction transmission path and the ~4000× air-vs-bone
//!   impedance mismatch that motivates bone-coupled transducers.
//! - **Stenfelt (2016)** "Model predictions for bone conduction perception
//!   in the human", *Hearing Research* 340, 135–143 — quantitative
//!   bone-conduction transmission model.
//! - **Gupta, Bhardwaj & Roy (2021)** "Acoustic impedance mismatch at
//!   bone-soft-tissue interfaces", review article — published impedance
//!   values for air, bone, soft tissue, and fluid (Pa·s/m).
//! - **Eeg-Olofsson, Stenfelt et al. (2008)** "Transmission of bone-
//!   conducted sound measured by cochlear vibrations", *International
//!   Journal of Audiology* 47(12), 761–769 — experimental confirmation
//!   that bone-coupled drive produces deep-tissue cochlear excitation.
//! - **Chang, Kim, Stenfelt & Brunskog (2016)** "A whole-head
//!   finite-element model for bone conduction", *Journal of the
//!   Acoustical Society of America* 140(3), 1635–1651 — FE-based
//!   end-to-end model of the conduction chain.

use pr4xis::category::{Arrow, Category};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Acoustics",
    source: "Stenfelt (2005) Bone-Conducted Sound: Physiological and Clinical Aspects, Otology & Neurotology 26(6); Stenfelt (2016) Model predictions for bone conduction perception in the human, Hearing Research 340; Gupta et al. (2021) Acoustic impedance mismatch at bone-soft-tissue interfaces; Eeg-Olofsson et al. (2008) Transmission of bone-conducted sound measured by cochlear vibrations, Int. J. Audiology 47(12); Chang et al. (2016) A whole-head finite-element model for bone conduction, JASA 140(3)",

    concepts: [
        // === Wave properties ===
        SoundWave,
        AcousticPressure,
        AcousticIntensity,
        AcousticFrequency,
        AcousticWavelength,
        AcousticAmplitude,
        Waveform,

        // === Impedance ===
        AcousticImpedance,
        ImpedanceMismatch,
        ReflectionCoefficient,
        TransmissionCoefficient,

        // === Conduction paths ===
        AirConduction,
        BoneConduction,
        SoftTissueConduction,

        // === Transducers ===
        ElectroacousticTransducer,
        PiezoelectricTransducer,
        ElectromagneticTransducer,

        // === Media ===
        Air,
        Bone,
        SoftTissue,
        Fluid,

        // === Abstract umbrellas ===
        WaveProperty,
        ImpedanceProperty,
        ConductionPath,
        TransducerType,
        AcousticMedium,
        AcousticEvent,

        // === Causal events (merged from AcousticsCausalEvent) ===
        ElectricalSignalInput,
        TransducerActivation,
        SurfaceOscillation,
        AcousticWaveGeneration,
        MediumPropagation,
        ImpedanceBoundary,
        PartialReflection,
        PartialTransmission,
        BoneCoupledTransmission,
        DeepTissuePenetration,
    ],

    labels: {
        SoundWave: ("en", "Sound wave",
            "Stenfelt (2005): a propagating mechanical disturbance carrying acoustic energy through an elastic medium."),
        AcousticPressure: ("en", "Acoustic pressure",
            "Stenfelt (2005): the local pressure deviation from the ambient mean caused by a sound wave (Pa)."),
        AcousticIntensity: ("en", "Acoustic intensity",
            "Stenfelt (2005): time-averaged acoustic power per unit area normal to the direction of propagation (W/m²)."),
        AcousticFrequency: ("en", "Acoustic frequency",
            "Stenfelt (2005): the temporal repetition rate of a sound wave (Hz)."),
        AcousticWavelength: ("en", "Acoustic wavelength",
            "Stenfelt (2005): spatial period of a sound wave; λ = c/f."),
        AcousticAmplitude: ("en", "Acoustic amplitude",
            "Stenfelt (2005): peak pressure (or displacement / velocity) of a sound wave."),
        Waveform: ("en", "Waveform",
            "Stenfelt (2005): the time-domain shape of an acoustic signal."),

        AcousticImpedance: ("en", "Acoustic impedance",
            "Stenfelt (2005): Z = ρc; the product of medium density and sound speed (Pa·s/m)."),
        ImpedanceMismatch: ("en", "Impedance mismatch",
            "Gupta et al. (2021): the ratio of acoustic impedances across a boundary; large mismatch drives strong reflection."),
        ReflectionCoefficient: ("en", "Reflection coefficient",
            "Stenfelt (2005): the fraction of incident acoustic power reflected at an impedance boundary."),
        TransmissionCoefficient: ("en", "Transmission coefficient",
            "Stenfelt (2005): the fraction of incident acoustic power transmitted across an impedance boundary."),

        AirConduction: ("en", "Air conduction",
            "Stenfelt (2005): the canonical hearing path: ambient sound enters the external auditory meatus and drives the middle ear."),
        BoneConduction: ("en", "Bone conduction",
            "Stenfelt (2005, 2016): coupling of vibration directly into the skull bone to excite the cochlea, bypassing the air-tissue impedance mismatch."),
        SoftTissueConduction: ("en", "Soft tissue conduction",
            "Stenfelt (2016): propagation of acoustic energy through skin, fat, and muscle (intermediate efficiency)."),

        ElectroacousticTransducer: ("en", "Electroacoustic transducer",
            "Stenfelt (2005): a device converting electrical signals into acoustic vibration (umbrella)."),
        PiezoelectricTransducer: ("en", "Piezoelectric transducer",
            "Stenfelt (2005): an electroacoustic transducer using a piezoelectric element."),
        ElectromagneticTransducer: ("en", "Electromagnetic transducer",
            "Stenfelt (2005): an electroacoustic transducer using a coil and magnet."),

        Air: ("en", "Air",
            "Gupta et al. (2021): low-density gaseous medium with Z ≈ 415 Pa·s/m."),
        Bone: ("en", "Bone",
            "Gupta et al. (2021): mineralised tissue with high acoustic impedance (~7.4×10⁶ Pa·s/m for cortical bone)."),
        SoftTissue: ("en", "Soft tissue",
            "Gupta et al. (2021): skin/fat/muscle/organ tissues; Z ≈ 1.6×10⁶ Pa·s/m."),
        Fluid: ("en", "Fluid",
            "Gupta et al. (2021): water/body fluids; Z ≈ 1.5×10⁶ Pa·s/m."),

        WaveProperty: ("en", "Wave property",
            "Stenfelt (2005): umbrella for measurable properties of an acoustic wave."),
        ImpedanceProperty: ("en", "Impedance property",
            "Stenfelt (2005): umbrella for impedance-related quantities at boundaries."),
        ConductionPath: ("en", "Conduction path",
            "Stenfelt (2016): umbrella for the route by which acoustic energy travels to the cochlea."),
        TransducerType: ("en", "Transducer type",
            "Stenfelt (2005): umbrella for electroacoustic-transducer kinds."),
        AcousticMedium: ("en", "Acoustic medium",
            "Stenfelt (2005): umbrella for any continuum through which a sound wave can propagate."),
        AcousticEvent: ("en", "Acoustic event",
            "Stenfelt (2016); Chang et al. (2016): umbrella for time-extended processes in the acoustic conduction chain."),

        ElectricalSignalInput: ("en", "Electrical signal input",
            "Chang et al. (2016): the electrical drive applied to the transducer at the head of the conduction chain."),
        TransducerActivation: ("en", "Transducer activation",
            "Chang et al. (2016): conversion of electrical energy into mechanical vibration of the transducer surface."),
        SurfaceOscillation: ("en", "Surface oscillation",
            "Stenfelt (2005): mechanical oscillation of the transducer-tissue or transducer-bone interface."),
        AcousticWaveGeneration: ("en", "Acoustic wave generation",
            "Stenfelt (2005): launching of an acoustic wave into the contacting medium."),
        MediumPropagation: ("en", "Medium propagation",
            "Stenfelt (2005): travel of the acoustic wave through a bulk medium."),
        ImpedanceBoundary: ("en", "Impedance boundary",
            "Stenfelt (2005): the wave reaches an interface between two media of differing acoustic impedance."),
        PartialReflection: ("en", "Partial reflection",
            "Stenfelt (2005): a fraction of incident power is reflected at an impedance boundary."),
        PartialTransmission: ("en", "Partial transmission",
            "Stenfelt (2005): the complementary fraction of incident power crosses the boundary."),
        BoneCoupledTransmission: ("en", "Bone-coupled transmission",
            "Stenfelt (2016): vibration coupled directly into the skull bone for efficient transmission past air-tissue mismatch."),
        DeepTissuePenetration: ("en", "Deep tissue penetration",
            "Eeg-Olofsson et al. (2008): cochlear-level excitation following bone-coupled transmission."),
    },

    is_a: [
        // Wave properties
        (SoundWave, WaveProperty),
        (AcousticPressure, WaveProperty),
        (AcousticIntensity, WaveProperty),
        (AcousticFrequency, WaveProperty),
        (AcousticWavelength, WaveProperty),
        (AcousticAmplitude, WaveProperty),
        (Waveform, WaveProperty),

        // Impedance
        (AcousticImpedance, ImpedanceProperty),
        (ImpedanceMismatch, ImpedanceProperty),
        (ReflectionCoefficient, ImpedanceProperty),
        (TransmissionCoefficient, ImpedanceProperty),

        // Conduction paths
        (AirConduction, ConductionPath),
        (BoneConduction, ConductionPath),
        (SoftTissueConduction, ConductionPath),

        // Transducers
        (ElectroacousticTransducer, TransducerType),
        (PiezoelectricTransducer, ElectroacousticTransducer),
        (ElectromagneticTransducer, ElectroacousticTransducer),

        // Media
        (Air, AcousticMedium),
        (Bone, AcousticMedium),
        (SoftTissue, AcousticMedium),
        (Fluid, AcousticMedium),

        // Events under the AcousticEvent umbrella
        (ElectricalSignalInput, AcousticEvent),
        (TransducerActivation, AcousticEvent),
        (SurfaceOscillation, AcousticEvent),
        (AcousticWaveGeneration, AcousticEvent),
        (MediumPropagation, AcousticEvent),
        (ImpedanceBoundary, AcousticEvent),
        (PartialReflection, AcousticEvent),
        (PartialTransmission, AcousticEvent),
        (BoneCoupledTransmission, AcousticEvent),
        (DeepTissuePenetration, AcousticEvent),
    ],

    causes: [
        // Main chain (Chang et al. 2016 §2 — FE model of conduction):
        // electrical drive → transducer → oscillation → wave → propagation.
        (ElectricalSignalInput, TransducerActivation),
        (TransducerActivation, SurfaceOscillation),
        (SurfaceOscillation, AcousticWaveGeneration),
        (AcousticWaveGeneration, MediumPropagation),
        // Impedance boundary is a branch point: both reflection and
        // transmission occur (Stenfelt 2005 — reflection / transmission
        // coefficients sum to unity at a planar boundary).
        (MediumPropagation, ImpedanceBoundary),
        (ImpedanceBoundary, PartialReflection),
        (ImpedanceBoundary, PartialTransmission),
        // Bone-coupling pathway bypasses the air-tissue mismatch
        // (Stenfelt 2016; Eeg-Olofsson 2008).
        (PartialTransmission, BoneCoupledTransmission),
        (BoneCoupledTransmission, DeepTissuePenetration),
    ],

    opposes: [
        // AirConduction ↔ BoneConduction: complementary conduction paths
        // (Stenfelt 2005, 2016).
        (AirConduction, BoneConduction),
        (BoneConduction, AirConduction),
        // ReflectionCoefficient ↔ TransmissionCoefficient: power-balance
        // duals at a planar impedance boundary (R + T = 1).
        (ReflectionCoefficient, TransmissionCoefficient),
        (TransmissionCoefficient, ReflectionCoefficient),
    ],
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Acoustic impedance Z in Pa·s/m for each medium concept.
///
/// Values follow Gupta et al. (2021) and Stenfelt (2005):
/// - Air: ~415 Pa·s/m
/// - Bone (cortical): ~7.4×10⁶ Pa·s/m
/// - Soft tissue: ~1.6×10⁶ Pa·s/m
/// - Fluid (water/body fluids): ~1.5×10⁶ Pa·s/m
#[derive(Debug, Clone)]
pub struct ImpedanceValue;

impl Quality for ImpedanceValue {
    type Individual = AcousticsConcept;
    type Value = f64;

    fn get(&self, c: &AcousticsConcept) -> Option<f64> {
        use AcousticsConcept::*;
        match c {
            Air => Some(415.0),
            Bone => Some(7_400_000.0),
            SoftTissue => Some(1_600_000.0),
            Fluid => Some(1_500_000.0),
            _ => None,
        }
    }
}

/// Transmission-efficiency band for a conduction path.
///
/// Stenfelt (2005, 2016): bone conduction efficiently bypasses the
/// air-tissue impedance mismatch (High); air conduction suffers the
/// mismatch (Low); soft-tissue conduction lies in between.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Efficiency {
    High,
    Medium,
    Low,
}

/// Quality: transmission efficiency for conduction paths.
#[derive(Debug, Clone)]
pub struct TransmissionEfficiency;

impl Quality for TransmissionEfficiency {
    type Individual = AcousticsConcept;
    type Value = Efficiency;

    fn get(&self, c: &AcousticsConcept) -> Option<Efficiency> {
        use AcousticsConcept::*;
        match c {
            BoneConduction => Some(Efficiency::High),
            AirConduction => Some(Efficiency::Low),
            SoftTissueConduction => Some(Efficiency::Medium),
            _ => None,
        }
    }
}

/// Quality: characteristic frequency range (min_hz, max_hz).
///
/// - Audible range: 20–20 000 Hz (Stenfelt 2005).
/// - Therapeutic bone-conduction window: 20–120 Hz (Stenfelt 2016 model
///   predictions — low-frequency bone-conduction is dominated by inertial
///   skull motion).
#[derive(Debug, Clone)]
pub struct FrequencyRange;

impl Quality for FrequencyRange {
    type Individual = AcousticsConcept;
    type Value = (f64, f64);

    fn get(&self, c: &AcousticsConcept) -> Option<(f64, f64)> {
        use AcousticsConcept::*;
        match c {
            SoundWave => Some((20.0, 20_000.0)),
            AcousticFrequency => Some((20.0, 20_000.0)),
            Waveform => Some((20.0, 120.0)),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Ontology + domain axioms
// ---------------------------------------------------------------------------

impl Ontology for AcousticsOntology {
    type Cat = AcousticsCategory;
    type Qual = ImpedanceValue;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(BoneImpedanceFarExceedsAir));
        axioms.push(Box::new(BoneImpedanceExceedsSoftTissue));
        axioms.push(Box::new(BoneConductionHighEfficiency));
        axioms.push(Box::new(AirConductionLowEfficiency));
        axioms.push(Box::new(ElectricalSignalCausesDeepPenetration));
        axioms.push(Box::new(ImpedanceBoundaryCausesBranch));
        axioms
    }
}

/// Helper: does a `Causation` edge (direct or transitive) exist from
/// `cause` to `effect`?
///
/// The macro emits same-kind transitive closure (per OBO-RO
/// `transitive_over`); a single morphisms() scan therefore suffices to
/// detect any-length causal reachability.
fn causes(cause: AcousticsConcept, effect: AcousticsConcept) -> bool {
    AcousticsCategory::morphisms().iter().any(|m| {
        m.kind() == AcousticsRelationKind::Causation && m.source() == cause && m.target() == effect
    })
}

/// Axiom: bone impedance far exceeds air impedance (Stenfelt 2005,
/// Gupta et al. 2021 — ~4000× mismatch).
pub struct BoneImpedanceFarExceedsAir;

impl Axiom for BoneImpedanceFarExceedsAir {
    fn verify(&self) -> Verdict {
        let bone_z = ImpedanceValue.get(&AcousticsConcept::Bone);
        let air_z = ImpedanceValue.get(&AcousticsConcept::Air);
        let ok = match (bone_z, air_z) {
            (Some(b), Some(a)) if a > 0.0 => b / a > 1000.0,
            _ => false,
        };
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "BoneImpedanceFarExceedsAir",
        "Bone acoustic impedance exceeds air impedance by more than three orders of magnitude (~4000×)",
        "Stenfelt (2005) Bone-Conducted Sound: Physiological and Clinical Aspects; Gupta et al. (2021)"
    );
}

pr4xis::register_axiom!(
    BoneImpedanceFarExceedsAir,
    "Stenfelt (2005) Bone-Conducted Sound: Physiological and Clinical Aspects; Gupta et al. (2021)"
);

/// Axiom: bone impedance > soft-tissue impedance (Gupta et al. 2021 —
/// 7.4×10⁶ vs 1.6×10⁶ Pa·s/m).
pub struct BoneImpedanceExceedsSoftTissue;

impl Axiom for BoneImpedanceExceedsSoftTissue {
    fn verify(&self) -> Verdict {
        let bone_z = ImpedanceValue.get(&AcousticsConcept::Bone);
        let soft_z = ImpedanceValue.get(&AcousticsConcept::SoftTissue);
        let ok = match (bone_z, soft_z) {
            (Some(b), Some(s)) => b > s,
            _ => false,
        };
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "BoneImpedanceExceedsSoftTissue",
        "Bone acoustic impedance exceeds that of soft tissue (~7.4×10⁶ vs ~1.6×10⁶ Pa·s/m)",
        "Gupta et al. (2021) Acoustic impedance mismatch at bone-soft-tissue interfaces"
    );
}

pr4xis::register_axiom!(
    BoneImpedanceExceedsSoftTissue,
    "Gupta et al. (2021) Acoustic impedance mismatch at bone-soft-tissue interfaces"
);

/// Axiom: bone-conduction transmission efficiency is High (Stenfelt 2005,
/// 2016 — bypasses the air-tissue impedance mismatch).
pub struct BoneConductionHighEfficiency;

impl Axiom for BoneConductionHighEfficiency {
    fn verify(&self) -> Verdict {
        if TransmissionEfficiency.get(&AcousticsConcept::BoneConduction) == Some(Efficiency::High) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "BoneConductionHighEfficiency",
        "Bone conduction has high transmission efficiency by bypassing the air-tissue impedance mismatch",
        "Stenfelt (2005), Stenfelt (2016) Model predictions for bone conduction perception"
    );
}

pr4xis::register_axiom!(
    BoneConductionHighEfficiency,
    "Stenfelt (2005); Stenfelt (2016) Model predictions for bone conduction perception"
);

/// Axiom: air-conduction transmission efficiency is Low (Gupta et al. 2021 —
/// ~4000× air-tissue mismatch reflects most incident power).
pub struct AirConductionLowEfficiency;

impl Axiom for AirConductionLowEfficiency {
    fn verify(&self) -> Verdict {
        if TransmissionEfficiency.get(&AcousticsConcept::AirConduction) == Some(Efficiency::Low) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AirConductionLowEfficiency",
        "Air conduction has low transmission efficiency due to the air-tissue impedance mismatch",
        "Gupta et al. (2021) Acoustic impedance mismatch at bone-soft-tissue interfaces"
    );
}

pr4xis::register_axiom!(
    AirConductionLowEfficiency,
    "Gupta et al. (2021) Acoustic impedance mismatch at bone-soft-tissue interfaces"
);

/// Axiom: end-to-end causal reachability from electrical signal input to
/// deep-tissue penetration (Chang et al. 2016 FE model; Eeg-Olofsson 2008
/// cochlear-vibration measurements confirm the chain).
pub struct ElectricalSignalCausesDeepPenetration;

impl Axiom for ElectricalSignalCausesDeepPenetration {
    fn verify(&self) -> Verdict {
        if causes(
            AcousticsConcept::ElectricalSignalInput,
            AcousticsConcept::DeepTissuePenetration,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ElectricalSignalCausesDeepPenetration",
        "Electrical signal input transitively causes deep tissue penetration via the full conduction chain",
        "Chang et al. (2016) Whole-head FE model for bone conduction, JASA 140(3); Eeg-Olofsson et al. (2008)"
    );
}

pr4xis::register_axiom!(
    ElectricalSignalCausesDeepPenetration,
    "Chang et al. (2016) Whole-head FE model for bone conduction, JASA 140(3); Eeg-Olofsson et al. (2008)"
);

/// Axiom: an impedance boundary causes both partial reflection and partial
/// transmission (Stenfelt 2005 — R + T = 1 at a planar boundary).
pub struct ImpedanceBoundaryCausesBranch;

impl Axiom for ImpedanceBoundaryCausesBranch {
    fn verify(&self) -> Verdict {
        if causes(
            AcousticsConcept::ImpedanceBoundary,
            AcousticsConcept::PartialReflection,
        ) && causes(
            AcousticsConcept::ImpedanceBoundary,
            AcousticsConcept::PartialTransmission,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ImpedanceBoundaryCausesBranch",
        "An impedance boundary branches into both partial reflection and partial transmission (R + T = 1)",
        "Stenfelt (2005) Bone-Conducted Sound: Physiological and Clinical Aspects"
    );
}

pr4xis::register_axiom!(
    ImpedanceBoundaryCausesBranch,
    "Stenfelt (2005) Bone-Conducted Sound: Physiological and Clinical Aspects"
);

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
    fn concept_count() {
        // 7 wave properties + 4 impedance + 3 conduction paths + 3 transducers
        // + 4 media + 6 abstract umbrellas + 10 events = 37.
        assert_eq!(AcousticsConcept::variants().len(), 37);
    }

    // -- Domain axiom tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn bone_impedance_far_exceeds_air_axiom() {
        assert!(BoneImpedanceFarExceedsAir.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn bone_impedance_exceeds_soft_tissue_axiom() {
        assert!(BoneImpedanceExceedsSoftTissue.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn bone_conduction_high_efficiency_axiom() {
        assert!(BoneConductionHighEfficiency.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn air_conduction_low_efficiency_axiom() {
        assert!(AirConductionLowEfficiency.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn electrical_signal_causes_deep_penetration_axiom() {
        assert!(ElectricalSignalCausesDeepPenetration.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn impedance_boundary_causes_branch_axiom() {
        assert!(ImpedanceBoundaryCausesBranch.verify().is_ok());
    }

    // -- Subsumption / kind tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn wave_properties_subsume_under_wave_property() {
        let subs: Vec<_> = AcousticsCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == AcousticsRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for c in [
            AcousticsConcept::SoundWave,
            AcousticsConcept::AcousticPressure,
            AcousticsConcept::AcousticFrequency,
            AcousticsConcept::AcousticWavelength,
            AcousticsConcept::Waveform,
        ] {
            assert!(
                subs.contains(&(c, AcousticsConcept::WaveProperty)),
                "{:?} should be a WaveProperty",
                c
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn piezo_and_em_subsume_under_electroacoustic() {
        let subs: Vec<_> = AcousticsCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == AcousticsRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(subs.contains(&(
            AcousticsConcept::PiezoelectricTransducer,
            AcousticsConcept::ElectroacousticTransducer
        )));
        assert!(subs.contains(&(
            AcousticsConcept::ElectromagneticTransducer,
            AcousticsConcept::ElectroacousticTransducer
        )));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn events_subsume_under_acoustic_event() {
        let subs: Vec<_> = AcousticsCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == AcousticsRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for ev in [
            AcousticsConcept::ElectricalSignalInput,
            AcousticsConcept::TransducerActivation,
            AcousticsConcept::AcousticWaveGeneration,
            AcousticsConcept::ImpedanceBoundary,
            AcousticsConcept::DeepTissuePenetration,
        ] {
            assert!(
                subs.contains(&(ev, AcousticsConcept::AcousticEvent)),
                "{:?} should subsume under AcousticEvent",
                ev
            );
        }
    }

    // -- Causation-kind tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn full_causal_chain_to_deep_penetration() {
        for c in [
            AcousticsConcept::TransducerActivation,
            AcousticsConcept::SurfaceOscillation,
            AcousticsConcept::AcousticWaveGeneration,
            AcousticsConcept::MediumPropagation,
            AcousticsConcept::ImpedanceBoundary,
            AcousticsConcept::PartialTransmission,
            AcousticsConcept::BoneCoupledTransmission,
            AcousticsConcept::DeepTissuePenetration,
        ] {
            assert!(
                causes(AcousticsConcept::ElectricalSignalInput, c),
                "ElectricalSignalInput should transitively cause {:?}",
                c
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn impedance_boundary_branches() {
        assert!(causes(
            AcousticsConcept::ImpedanceBoundary,
            AcousticsConcept::PartialReflection,
        ));
        assert!(causes(
            AcousticsConcept::ImpedanceBoundary,
            AcousticsConcept::PartialTransmission,
        ));
    }

    // -- Opposition-kind tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn air_and_bone_conduction_oppose() {
        let opps: Vec<_> = AcousticsCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == AcousticsRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(
            AcousticsConcept::AirConduction,
            AcousticsConcept::BoneConduction
        )));
        assert!(opps.contains(&(
            AcousticsConcept::BoneConduction,
            AcousticsConcept::AirConduction
        )));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn reflection_and_transmission_oppose() {
        let opps: Vec<_> = AcousticsCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == AcousticsRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opps.contains(&(
            AcousticsConcept::ReflectionCoefficient,
            AcousticsConcept::TransmissionCoefficient
        )));
    }

    // -- Quality tests --

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn impedance_values_match_literature() {
        assert_eq!(ImpedanceValue.get(&AcousticsConcept::Air), Some(415.0));
        assert_eq!(
            ImpedanceValue.get(&AcousticsConcept::Bone),
            Some(7_400_000.0)
        );
        assert_eq!(
            ImpedanceValue.get(&AcousticsConcept::SoftTissue),
            Some(1_600_000.0)
        );
        assert_eq!(
            ImpedanceValue.get(&AcousticsConcept::Fluid),
            Some(1_500_000.0)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn transmission_efficiency_classes() {
        assert_eq!(
            TransmissionEfficiency.get(&AcousticsConcept::BoneConduction),
            Some(Efficiency::High)
        );
        assert_eq!(
            TransmissionEfficiency.get(&AcousticsConcept::AirConduction),
            Some(Efficiency::Low)
        );
        assert_eq!(
            TransmissionEfficiency.get(&AcousticsConcept::SoftTissueConduction),
            Some(Efficiency::Medium)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn frequency_range_audible_and_therapeutic() {
        assert_eq!(
            FrequencyRange.get(&AcousticsConcept::SoundWave),
            Some((20.0, 20_000.0))
        );
        assert_eq!(
            FrequencyRange.get(&AcousticsConcept::Waveform),
            Some((20.0, 120.0))
        );
    }

    // -- Proptests --

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
                    prop_assert!(
                        false,
                        "axiom failed: {}",
                        c.meta().name.as_str()
                    );
                }
            }
        }

        #[test]
        fn prop_subsumption_targets_valid(_seed in any::<u32>()) {
            let variants: Vec<_> = AcousticsConcept::variants();
            for m in AcousticsCategory::morphisms() {
                if m.kind() == AcousticsRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            let opposed: std::collections::HashSet<_> = AcousticsCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == AcousticsRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in opposed.iter() {
                prop_assert!(opposed.contains(&(*b, *a)),
                    "opposition not symmetric: {:?} → {:?} but not back", a, b);
            }
        }

        #[test]
        fn prop_impedance_positive_when_defined(c in arb_concept()) {
            if let Some(z) = ImpedanceValue.get(&c) {
                prop_assert!(z > 0.0, "impedance must be positive for {:?}", c);
            }
        }

        #[test]
        fn prop_frequency_range_valid(c in arb_concept()) {
            if let Some((lo, hi)) = FrequencyRange.get(&c) {
                prop_assert!(lo < hi, "frequency range min<max for {:?}", c);
            }
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_subsumption_targets_valid, Verifiable);
    pr4xis::register_praxis_value!(prop_opposition_is_symmetric, Verifiable);
    pr4xis::register_praxis_value!(prop_impedance_positive_when_defined, Verifiable);
    pr4xis::register_praxis_value!(prop_frequency_range_valid, Verifiable);
}
