//! Acoustics ontology.
//!
//! Entities: wave properties, impedance, conduction paths, transducers, media.
//! Taxonomy: property type hierarchy (wave, impedance, conduction, transducer, medium).
//! Causal graph: electrical signal -> transducer activation -> surface oscillation ->
//!   acoustic wave generation -> medium propagation -> impedance boundary ->
//!   partial reflection + partial transmission -> deep tissue penetration.
//!
//! Key references:
//! - Stenfelt 2005, 2016: bone conduction transmission physics
//! - Gupta 2021: acoustic impedance mismatch at bone-soft tissue interface
//! - Eeg-Olofsson 2008: bone-conducted sound measured by cochlear vibrations
//! - Chang 2016: whole-head finite-element model

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::causation;
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

/// Every entity in the acoustics domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
pub enum AcousticsEntity {
    // Wave properties
    SoundWave,
    AcousticPressure,
    AcousticIntensity,
    AcousticFrequency,
    AcousticWavelength,
    AcousticAmplitude,
    Waveform,
    // Impedance
    AcousticImpedance,
    ImpedanceMismatch,
    ReflectionCoefficient,
    TransmissionCoefficient,
    // Conduction paths
    AirConduction,
    BoneConduction,
    SoftTissueConduction,
    // Transduction
    ElectroacousticTransducer,
    PiezoelectricTransducer,
    ElectromagneticTransducer,
    // Media
    Air,
    Bone,
    SoftTissue,
    Fluid,
    // Abstract categories
    WaveProperty,
    ImpedanceProperty,
    ConductionPath,
    TransducerType,
    AcousticMedium,
}

// ---------------------------------------------------------------------------
// Causal event
// ---------------------------------------------------------------------------

/// Events in the acoustics causal chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
pub enum AcousticsCausalEvent {
    /// Electrical signal drives the transducer
    ElectricalSignalInput,
    /// Transducer converts electrical to mechanical energy
    TransducerActivation,
    /// Surface oscillation at the transducer-tissue interface
    SurfaceOscillation,
    /// Acoustic wave launched into medium
    AcousticWaveGeneration,
    /// Wave propagates through a medium
    MediumPropagation,
    /// Wave encounters impedance boundary between media
    ImpedanceBoundary,
    /// Partial reflection at impedance boundary
    PartialReflection,
    /// Partial transmission through impedance boundary
    PartialTransmission,
    /// Vibration coupled into bone for efficient transmission
    BoneCoupledTransmission,
    /// Wave penetrates deep tissue layers
    DeepTissuePenetration,
}

// ---------------------------------------------------------------------------
// Category + Reasoning (generated)
// ---------------------------------------------------------------------------

define_ontology! {
    /// Acoustics ontology: wave properties, impedance, conduction, transduction, media.
    pub AcousticsOntologyMeta for AcousticsCategory {
        entity: AcousticsEntity,
        relation: AcousticsRelation,
        being: Quality,
        source: "citings pending",

        taxonomy: AcousticsTaxonomy [
            // Wave properties is-a WaveProperty
            (SoundWave, WaveProperty),
            (AcousticPressure, WaveProperty),
            (AcousticIntensity, WaveProperty),
            (AcousticFrequency, WaveProperty),
            (AcousticWavelength, WaveProperty),
            (AcousticAmplitude, WaveProperty),
            (Waveform, WaveProperty),
            // Impedance is-a ImpedanceProperty
            (AcousticImpedance, ImpedanceProperty),
            (ImpedanceMismatch, ImpedanceProperty),
            (ReflectionCoefficient, ImpedanceProperty),
            (TransmissionCoefficient, ImpedanceProperty),
            // Conduction paths is-a ConductionPath
            (AirConduction, ConductionPath),
            (BoneConduction, ConductionPath),
            (SoftTissueConduction, ConductionPath),
            // Transducers is-a TransducerType
            (ElectroacousticTransducer, TransducerType),
            (PiezoelectricTransducer, TransducerType),
            (ElectromagneticTransducer, TransducerType),
            // Subtypes within transducers
            (PiezoelectricTransducer, ElectroacousticTransducer),
            (ElectromagneticTransducer, ElectroacousticTransducer),
            // Media is-a AcousticMedium
            (Air, AcousticMedium),
            (Bone, AcousticMedium),
            (SoftTissue, AcousticMedium),
            (Fluid, AcousticMedium),
        ],

        causation: AcousticsCauses for AcousticsCausalEvent [
            // Main chain: electrical -> transducer -> oscillation -> wave
            (ElectricalSignalInput, TransducerActivation),
            (TransducerActivation, SurfaceOscillation),
            (SurfaceOscillation, AcousticWaveGeneration),
            // Wave propagation through medium
            (AcousticWaveGeneration, MediumPropagation),
            // Impedance boundary: branch point (both reflection and transmission)
            (MediumPropagation, ImpedanceBoundary),
            (ImpedanceBoundary, PartialReflection),
            (ImpedanceBoundary, PartialTransmission),
            // Bone conduction path (bypasses air mismatch)
            (PartialTransmission, BoneCoupledTransmission),
            (BoneCoupledTransmission, DeepTissuePenetration),
        ],

        opposition: AcousticsOpposition [
            (AirConduction, BoneConduction),
            (ReflectionCoefficient, TransmissionCoefficient),
        ],
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: acoustic impedance value in Pa.s/m for acoustic media.
///
/// Published values (Duck 1990, Stenfelt 2005):
/// - Air: ~415 Pa.s/m
/// - Bone: ~7,400,000 Pa.s/m (cortical bone)
/// - Soft tissue: ~1,600,000 Pa.s/m
/// - Fluid (water/body fluids): ~1,500,000 Pa.s/m
#[derive(Debug, Clone)]
pub struct ImpedanceValue;

impl Quality for ImpedanceValue {
    type Individual = AcousticsEntity;
    type Value = f64;

    fn get(&self, individual: &AcousticsEntity) -> Option<f64> {
        use AcousticsEntity::*;
        match individual {
            Air => Some(415.0),
            Bone => Some(7_400_000.0),
            SoftTissue => Some(1_600_000.0),
            Fluid => Some(1_500_000.0),
            _ => None,
        }
    }
}

/// Transmission efficiency through a conduction path.
///
/// - BoneConduction: High -- bypasses the air-tissue impedance mismatch
///   (Stenfelt 2005, 2016)
/// - AirConduction: Low -- suffers ~4000x impedance mismatch at air-tissue
///   boundary (Gupta 2021)
/// - SoftTissueConduction: Medium -- moderate attenuation
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
    type Individual = AcousticsEntity;
    type Value = Efficiency;

    fn get(&self, individual: &AcousticsEntity) -> Option<Efficiency> {
        use AcousticsEntity::*;
        match individual {
            BoneConduction => Some(Efficiency::High),
            AirConduction => Some(Efficiency::Low),
            SoftTissueConduction => Some(Efficiency::Medium),
            _ => None,
        }
    }
}

/// Quality: relevant frequency range (min_hz, max_hz).
///
/// - Audible: 20-20,000 Hz (human hearing range)
/// - Therapeutic: 20-120 Hz (bone conduction therapeutic window)
/// - Ultrasound: >20,000 Hz
#[derive(Debug, Clone)]
pub struct FrequencyRange;

impl Quality for FrequencyRange {
    type Individual = AcousticsEntity;
    type Value = (f64, f64);

    fn get(&self, individual: &AcousticsEntity) -> Option<(f64, f64)> {
        use AcousticsEntity::*;
        match individual {
            SoundWave => Some((20.0, 20_000.0)),         // audible range
            AcousticFrequency => Some((20.0, 20_000.0)), // general audible
            Waveform => Some((20.0, 120.0)),             // therapeutic range
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Axiom: bone impedance >> air impedance (~4000x mismatch, Stenfelt 2005).
pub struct BoneImpedanceFarExceedsAir;

impl Axiom for BoneImpedanceFarExceedsAir {
    fn description(&self) -> &str {
        "bone impedance far exceeds air impedance (~4000x mismatch, Stenfelt 2005)"
    }

    fn holds(&self) -> bool {
        let bone_z = ImpedanceValue.get(&AcousticsEntity::Bone);
        let air_z = ImpedanceValue.get(&AcousticsEntity::Air);
        match (bone_z, air_z) {
            (Some(b), Some(a)) => b / a > 1000.0, // conservative: at least 1000x
            _ => false,
        }
    }
}
pr4xis::register_axiom!(BoneImpedanceFarExceedsAir);

/// Axiom: bone impedance > soft tissue impedance (7.4M vs 1.6M).
pub struct BoneImpedanceExceedsSoftTissue;

impl Axiom for BoneImpedanceExceedsSoftTissue {
    fn description(&self) -> &str {
        "bone impedance exceeds soft tissue impedance (7.4M vs 1.6M Pa.s/m)"
    }

    fn holds(&self) -> bool {
        let bone_z = ImpedanceValue.get(&AcousticsEntity::Bone);
        let soft_z = ImpedanceValue.get(&AcousticsEntity::SoftTissue);
        match (bone_z, soft_z) {
            (Some(b), Some(s)) => b > s,
            _ => false,
        }
    }
}
pr4xis::register_axiom!(BoneImpedanceExceedsSoftTissue);

/// Axiom: bone conduction has high transmission efficiency (bypasses air-tissue mismatch).
pub struct BoneConductionHighEfficiency;

impl Axiom for BoneConductionHighEfficiency {
    fn description(&self) -> &str {
        "bone conduction has high transmission efficiency (bypasses air-tissue mismatch)"
    }

    fn holds(&self) -> bool {
        TransmissionEfficiency.get(&AcousticsEntity::BoneConduction) == Some(Efficiency::High)
    }
}
pr4xis::register_axiom!(BoneConductionHighEfficiency);

/// Axiom: air conduction has low efficiency (suffers from impedance mismatch).
pub struct AirConductionLowEfficiency;

impl Axiom for AirConductionLowEfficiency {
    fn description(&self) -> &str {
        "air conduction has low transmission efficiency (suffers from impedance mismatch)"
    }

    fn holds(&self) -> bool {
        TransmissionEfficiency.get(&AcousticsEntity::AirConduction) == Some(Efficiency::Low)
    }
}
pr4xis::register_axiom!(AirConductionLowEfficiency);

/// Axiom: electrical signal transitively causes deep tissue penetration.
pub struct ElectricalSignalCausesDeepPenetration;

impl Axiom for ElectricalSignalCausesDeepPenetration {
    fn description(&self) -> &str {
        "electrical signal transitively causes deep tissue penetration \
         (full chain: signal -> transducer -> oscillation -> wave -> \
         propagation -> boundary -> transmission -> bone coupling -> penetration)"
    }

    fn holds(&self) -> bool {
        use AcousticsCausalEvent::*;
        let effects = causation::effects_of::<AcousticsCauses>(&ElectricalSignalInput);
        effects.contains(&DeepTissuePenetration)
    }
}
pr4xis::register_axiom!(ElectricalSignalCausesDeepPenetration);

/// Axiom: impedance boundary causes both reflection AND transmission (branch point).
pub struct ImpedanceBoundaryCausesBranch;

impl Axiom for ImpedanceBoundaryCausesBranch {
    fn description(&self) -> &str {
        "impedance boundary causes both partial reflection and partial transmission"
    }

    fn holds(&self) -> bool {
        use AcousticsCausalEvent::*;
        let effects = causation::effects_of::<AcousticsCauses>(&ImpedanceBoundary);
        effects.contains(&PartialReflection) && effects.contains(&PartialTransmission)
    }
}
pr4xis::register_axiom!(ImpedanceBoundaryCausesBranch);

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// Top-level acoustics ontology tying together category, qualities, and axioms.
pub struct AcousticsOntology;

impl Ontology for AcousticsOntology {
    type Cat = AcousticsCategory;
    type Qual = ImpedanceValue;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        AcousticsOntologyMeta::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(BoneImpedanceFarExceedsAir),
            Box::new(BoneImpedanceExceedsSoftTissue),
            Box::new(BoneConductionHighEfficiency),
            Box::new(AirConductionLowEfficiency),
            Box::new(ElectricalSignalCausesDeepPenetration),
            Box::new(ImpedanceBoundaryCausesBranch),
        ]
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;
    use pr4xis::ontology::reasoning::causation::CausalDef;
    use pr4xis::ontology::reasoning::opposition;
    use pr4xis::ontology::reasoning::taxonomy;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;

    // -- Axiom tests --

    #[test]
    fn test_bone_impedance_far_exceeds_air() {
        assert!(
            BoneImpedanceFarExceedsAir.holds(),
            "{}",
            BoneImpedanceFarExceedsAir.description()
        );
    }

    #[test]
    fn test_bone_impedance_exceeds_soft_tissue() {
        assert!(
            BoneImpedanceExceedsSoftTissue.holds(),
            "{}",
            BoneImpedanceExceedsSoftTissue.description()
        );
    }

    #[test]
    fn test_bone_conduction_high_efficiency() {
        assert!(
            BoneConductionHighEfficiency.holds(),
            "{}",
            BoneConductionHighEfficiency.description()
        );
    }

    #[test]
    fn test_air_conduction_low_efficiency() {
        assert!(
            AirConductionLowEfficiency.holds(),
            "{}",
            AirConductionLowEfficiency.description()
        );
    }

    #[test]
    fn test_electrical_signal_causes_deep_penetration() {
        assert!(
            ElectricalSignalCausesDeepPenetration.holds(),
            "{}",
            ElectricalSignalCausesDeepPenetration.description()
        );
    }

    #[test]
    fn test_impedance_boundary_causes_branch() {
        assert!(
            ImpedanceBoundaryCausesBranch.holds(),
            "{}",
            ImpedanceBoundaryCausesBranch.description()
        );
    }

    // -- Category law tests --

    #[test]
    fn test_acoustics_category_laws() {
        check_category_laws::<AcousticsCategory>().unwrap();
    }

    #[test]
    fn test_acoustics_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<AcousticsTaxonomy>>().unwrap();
    }

    #[test]
    fn test_acoustics_causal_category_laws() {
        use pr4xis::ontology::reasoning::causation::CausalCategory;
        check_category_laws::<CausalCategory<AcousticsCauses>>().unwrap();
    }

    // -- Taxonomy tests --

    #[test]
    fn test_wave_properties_are_wave() {
        use AcousticsEntity::*;
        for entity in [
            SoundWave,
            AcousticPressure,
            AcousticIntensity,
            AcousticFrequency,
            AcousticWavelength,
            AcousticAmplitude,
            Waveform,
        ] {
            assert!(
                taxonomy::is_a::<AcousticsTaxonomy>(&entity, &WaveProperty),
                "{:?} should be a WaveProperty",
                entity
            );
        }
    }

    #[test]
    fn test_impedance_properties_are_impedance() {
        use AcousticsEntity::*;
        for entity in [
            AcousticImpedance,
            ImpedanceMismatch,
            ReflectionCoefficient,
            TransmissionCoefficient,
        ] {
            assert!(
                taxonomy::is_a::<AcousticsTaxonomy>(&entity, &ImpedanceProperty),
                "{:?} should be an ImpedanceProperty",
                entity
            );
        }
    }

    #[test]
    fn test_conduction_paths_are_conduction() {
        use AcousticsEntity::*;
        for entity in [AirConduction, BoneConduction, SoftTissueConduction] {
            assert!(
                taxonomy::is_a::<AcousticsTaxonomy>(&entity, &ConductionPath),
                "{:?} should be a ConductionPath",
                entity
            );
        }
    }

    #[test]
    fn test_transducers_are_transducer_type() {
        use AcousticsEntity::*;
        for entity in [
            ElectroacousticTransducer,
            PiezoelectricTransducer,
            ElectromagneticTransducer,
        ] {
            assert!(
                taxonomy::is_a::<AcousticsTaxonomy>(&entity, &TransducerType),
                "{:?} should be a TransducerType",
                entity
            );
        }
    }

    #[test]
    fn test_piezo_and_em_are_electroacoustic() {
        use AcousticsEntity::*;
        assert!(taxonomy::is_a::<AcousticsTaxonomy>(
            &PiezoelectricTransducer,
            &ElectroacousticTransducer
        ));
        assert!(taxonomy::is_a::<AcousticsTaxonomy>(
            &ElectromagneticTransducer,
            &ElectroacousticTransducer
        ));
    }

    #[test]
    fn test_media_are_acoustic_medium() {
        use AcousticsEntity::*;
        for entity in [Air, Bone, SoftTissue, Fluid] {
            assert!(
                taxonomy::is_a::<AcousticsTaxonomy>(&entity, &AcousticMedium),
                "{:?} should be an AcousticMedium",
                entity
            );
        }
    }

    #[test]
    fn test_entity_count() {
        assert_eq!(AcousticsEntity::variants().len(), 26);
    }

    #[test]
    fn test_causal_event_count() {
        assert_eq!(AcousticsCausalEvent::variants().len(), 10);
    }

    // -- Opposition tests --

    #[test]
    fn test_air_opposes_bone_conduction() {
        use AcousticsEntity::*;
        assert!(opposition::are_opposed::<AcousticsOpposition>(
            &AirConduction,
            &BoneConduction
        ));
        assert!(opposition::are_opposed::<AcousticsOpposition>(
            &BoneConduction,
            &AirConduction
        ));
    }

    #[test]
    fn test_reflection_opposes_transmission() {
        use AcousticsEntity::*;
        assert!(opposition::are_opposed::<AcousticsOpposition>(
            &ReflectionCoefficient,
            &TransmissionCoefficient
        ));
    }

    #[test]
    fn test_air_does_not_oppose_soft_tissue_conduction() {
        use AcousticsEntity::*;
        assert!(!opposition::are_opposed::<AcousticsOpposition>(
            &AirConduction,
            &SoftTissueConduction
        ));
    }

    // -- Quality tests --

    #[test]
    fn test_air_impedance_value() {
        let z = ImpedanceValue.get(&AcousticsEntity::Air);
        assert_eq!(z, Some(415.0));
    }

    #[test]
    fn test_bone_impedance_value() {
        let z = ImpedanceValue.get(&AcousticsEntity::Bone);
        assert_eq!(z, Some(7_400_000.0));
    }

    #[test]
    fn test_soft_tissue_impedance_value() {
        let z = ImpedanceValue.get(&AcousticsEntity::SoftTissue);
        assert_eq!(z, Some(1_600_000.0));
    }

    #[test]
    fn test_fluid_impedance_value() {
        let z = ImpedanceValue.get(&AcousticsEntity::Fluid);
        assert_eq!(z, Some(1_500_000.0));
    }

    #[test]
    fn test_bone_conduction_efficiency() {
        assert_eq!(
            TransmissionEfficiency.get(&AcousticsEntity::BoneConduction),
            Some(Efficiency::High)
        );
    }

    #[test]
    fn test_air_conduction_efficiency() {
        assert_eq!(
            TransmissionEfficiency.get(&AcousticsEntity::AirConduction),
            Some(Efficiency::Low)
        );
    }

    #[test]
    fn test_soft_tissue_conduction_efficiency() {
        assert_eq!(
            TransmissionEfficiency.get(&AcousticsEntity::SoftTissueConduction),
            Some(Efficiency::Medium)
        );
    }

    #[test]
    fn test_sound_wave_frequency_range() {
        let range = FrequencyRange.get(&AcousticsEntity::SoundWave);
        assert_eq!(range, Some((20.0, 20_000.0)));
    }

    #[test]
    fn test_waveform_therapeutic_range() {
        let range = FrequencyRange.get(&AcousticsEntity::Waveform);
        assert_eq!(range, Some((20.0, 120.0)));
    }

    // -- Causal chain tests --

    #[test]
    fn test_electrical_signal_causes_transducer_activation() {
        use AcousticsCausalEvent::*;
        let effects = causation::effects_of::<AcousticsCauses>(&ElectricalSignalInput);
        assert!(effects.contains(&TransducerActivation));
    }

    #[test]
    fn test_full_chain_to_deep_penetration() {
        use AcousticsCausalEvent::*;
        let effects = causation::effects_of::<AcousticsCauses>(&ElectricalSignalInput);
        assert!(effects.contains(&TransducerActivation));
        assert!(effects.contains(&SurfaceOscillation));
        assert!(effects.contains(&AcousticWaveGeneration));
        assert!(effects.contains(&MediumPropagation));
        assert!(effects.contains(&ImpedanceBoundary));
        assert!(effects.contains(&PartialReflection));
        assert!(effects.contains(&PartialTransmission));
        assert!(effects.contains(&BoneCoupledTransmission));
        assert!(effects.contains(&DeepTissuePenetration));
    }

    #[test]
    fn test_impedance_boundary_branches() {
        use AcousticsCausalEvent::*;
        let effects = causation::effects_of::<AcousticsCauses>(&ImpedanceBoundary);
        assert!(effects.contains(&PartialReflection));
        assert!(effects.contains(&PartialTransmission));
    }

    // -- Ontology validation --

    #[test]
    fn test_ontology_validates() {
        AcousticsOntology::validate().unwrap();
    }

    // -- Property-based tests (proptest) --

    use proptest::prelude::*;

    fn arb_acoustics_entity() -> impl Strategy<Value = AcousticsEntity> {
        (0..AcousticsEntity::variants().len()).prop_map(|i| AcousticsEntity::variants()[i])
    }

    fn arb_acoustics_causal_event() -> impl Strategy<Value = AcousticsCausalEvent> {
        (0..AcousticsCausalEvent::variants().len())
            .prop_map(|i| AcousticsCausalEvent::variants()[i])
    }

    proptest! {
        /// For any AcousticsEntity, taxonomy is-a is reflexive.
        #[test]
        fn prop_taxonomy_is_a_reflexive(entity in arb_acoustics_entity()) {
            prop_assert!(
                taxonomy::is_a::<AcousticsTaxonomy>(&entity, &entity),
                "is-a should be reflexive for {:?}",
                entity
            );
        }

        /// ImpedanceValue for media is always positive.
        #[test]
        fn prop_impedance_always_positive(entity in arb_acoustics_entity()) {
            if let Some(z) = ImpedanceValue.get(&entity) {
                prop_assert!(z > 0.0, "impedance must be positive for {:?}", entity);
            }
        }

        /// FrequencyRange min < max when defined.
        #[test]
        fn prop_frequency_range_valid(entity in arb_acoustics_entity()) {
            if let Some((min, max)) = FrequencyRange.get(&entity) {
                prop_assert!(min < max, "frequency range min must be < max for {:?}", entity);
            }
        }

        /// No causal event causes itself (checked via proptest).
        #[test]
        fn prop_no_self_causation(event in arb_acoustics_causal_event()) {
            let relations = AcousticsCauses::relations();
            let self_caused = relations.iter().any(|(a, b)| *a == event && *b == event);
            prop_assert!(
                !self_caused,
                "event {:?} should not directly cause itself",
                event
            );
        }
    }
}
