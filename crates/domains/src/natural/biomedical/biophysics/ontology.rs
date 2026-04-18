//! Biophysics ontology.
//!
//! Entities: mechanical properties, wave physics, piezoelectricity,
//! membrane biophysics, and biological media.
//! Taxonomy: property type hierarchy (mechanical, wave, piezoelectric, medium).
//! Causal graph: vibration -> wave propagation -> tissue deformation ->
//!   mechanotransduction + piezoelectric charge generation.
//!
//! Key references:
//! - Fukada & Yasuda 1957: piezoelectricity of bone (collagen)
//! - Duck 1990: acoustic properties of biological tissue
//! - Cowin & Doty 2007: tissue mechanics

use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::causation;
use pr4xis::ontology::reasoning::opposition;
use pr4xis::ontology::reasoning::taxonomy;
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

/// Every biophysical entity in the domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum BiophysicsEntity {
    // Mechanical properties
    Viscoelasticity,
    Elasticity,
    Viscosity,
    StiffnessModulus,
    StrainRate,
    MechanicalStress,
    MechanicalStrain,
    // Wave physics
    MechanicalWave,
    AcousticImpedance,
    Attenuation,
    Frequency,
    Wavelength,
    ResonanceFrequency,
    // Piezoelectricity
    PiezoelectricEffect,
    DirectPiezoelectric,
    ConversePiezoelectric,
    CollagenPiezoelectricity,
    // Membrane biophysics
    MembraneCapacitance,
    MembraneTension,
    CellDeformation,
    // Medium
    BoneMatrix,
    SoftTissue,
    FluidMedium,
    CellMembrane,
    // Abstract
    MechanicalProperty,
    WaveProperty,
    PiezoelectricProperty,
    BiologicalMedium,
}

// ---------------------------------------------------------------------------
// Causal event
// ---------------------------------------------------------------------------

/// Events in the biophysics causal chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum BiophysicsCausalEvent {
    /// External vibration applied to tissue
    ExternalVibration,
    /// Wave propagates through medium
    WavePropagation,
    /// Tissue undergoes mechanical deformation
    TissueDeformation,
    /// Cell membrane experiences strain
    CellMembraneStrain,
    /// Mechanotransducer channel activates
    MechanotransducerActivation,
    /// Bone conduction transmits vibration
    BoneConductionTransmission,
    /// Piezoelectric charge generated from deformation
    PiezoelectricChargeGeneration,
    /// Local electric field from piezoelectric effect
    LocalElectricField,
    /// Impedance mismatch at tissue boundary
    ImpedanceMismatch,
    /// Wave reflects at impedance boundary
    WaveReflection,
}

// ---------------------------------------------------------------------------
// Category + Reasoning (generated)
// ---------------------------------------------------------------------------

define_ontology! {
    /// Biophysics ontology: mechanics, waves, piezoelectricity, media.
    pub BiophysicsOntologyMeta for BiophysicsCategory {
        entity: BiophysicsEntity,
        relation: BiophysicsRelation,
        being: Quality,
        source: "Fukada & Yasuda (1957); Duck (1990)",

        taxonomy: BiophysicsTaxonomy [
            (Viscoelasticity, MechanicalProperty),
            (Elasticity, MechanicalProperty),
            (Viscosity, MechanicalProperty),
            (StiffnessModulus, MechanicalProperty),
            (StrainRate, MechanicalProperty),
            (MechanicalStress, MechanicalProperty),
            (MechanicalStrain, MechanicalProperty),
            (MechanicalWave, WaveProperty),
            (AcousticImpedance, WaveProperty),
            (Attenuation, WaveProperty),
            (Frequency, WaveProperty),
            (Wavelength, WaveProperty),
            (ResonanceFrequency, WaveProperty),
            (PiezoelectricEffect, PiezoelectricProperty),
            (DirectPiezoelectric, PiezoelectricProperty),
            (ConversePiezoelectric, PiezoelectricProperty),
            (CollagenPiezoelectricity, PiezoelectricProperty),
            (DirectPiezoelectric, PiezoelectricEffect),
            (ConversePiezoelectric, PiezoelectricEffect),
            (MembraneCapacitance, MechanicalProperty),
            (MembraneTension, MechanicalProperty),
            (CellDeformation, MechanicalProperty),
            (BoneMatrix, BiologicalMedium),
            (SoftTissue, BiologicalMedium),
            (FluidMedium, BiologicalMedium),
            (CellMembrane, BiologicalMedium),
        ],

        causation: BiophysicsCauses for BiophysicsCausalEvent [
            (ExternalVibration, WavePropagation),
            (WavePropagation, TissueDeformation),
            (TissueDeformation, CellMembraneStrain),
            (CellMembraneStrain, MechanotransducerActivation),
            (ExternalVibration, BoneConductionTransmission),
            (BoneConductionTransmission, WavePropagation),
            (TissueDeformation, PiezoelectricChargeGeneration),
            (PiezoelectricChargeGeneration, LocalElectricField),
            (ImpedanceMismatch, WaveReflection),
        ],

        opposition: BiophysicsOpposition [
            (DirectPiezoelectric, ConversePiezoelectric),
            (Elasticity, Viscosity),
            (MechanicalStress, MechanicalStrain),
        ],
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: acoustic impedance value in MRayls for biological media.
#[derive(Debug, Clone)]
pub struct AcousticImpedanceValue;

impl Quality for AcousticImpedanceValue {
    type Individual = BiophysicsEntity;
    type Value = f64;

    fn get(&self, individual: &BiophysicsEntity) -> Option<f64> {
        use BiophysicsEntity::*;
        match individual {
            BoneMatrix => Some(7.4),   // ~7.4 MRayls
            SoftTissue => Some(1.6),   // ~1.6 MRayls
            FluidMedium => Some(1.5),  // ~1.5 MRayls
            CellMembrane => Some(1.6), // ~1.6 MRayls (similar to soft tissue)
            _ => None,
        }
    }
}

/// Quality: is this entity piezoelectric?
#[derive(Debug, Clone)]
pub struct IsPiezoelectric;

impl Quality for IsPiezoelectric {
    type Individual = BiophysicsEntity;
    type Value = bool;

    fn get(&self, individual: &BiophysicsEntity) -> Option<bool> {
        use BiophysicsEntity::*;
        match individual {
            CollagenPiezoelectricity => Some(true),
            BoneMatrix => Some(true), // collagen content makes bone piezoelectric
            CellMembrane => Some(false),
            SoftTissue => Some(false),
            FluidMedium => Some(false),
            _ => None,
        }
    }
}

/// Quality: does this medium transmit vibration?
#[derive(Debug, Clone)]
pub struct TransmitsVibration;

impl Quality for TransmitsVibration {
    type Individual = BiophysicsEntity;
    type Value = bool;

    fn get(&self, individual: &BiophysicsEntity) -> Option<bool> {
        use BiophysicsEntity::*;
        match individual {
            BoneMatrix => Some(true),   // efficiently (low attenuation)
            SoftTissue => Some(true),   // with attenuation
            FluidMedium => Some(true),  // acoustic transmission
            CellMembrane => Some(true), // transmits to cell interior
            _ => None,
        }
    }
}

/// Quality: relevant frequency range (min_hz, max_hz) for wave entities.
#[derive(Debug, Clone)]
pub struct FrequencyRange;

impl Quality for FrequencyRange {
    type Individual = BiophysicsEntity;
    type Value = (f64, f64);

    fn get(&self, individual: &BiophysicsEntity) -> Option<(f64, f64)> {
        use BiophysicsEntity::*;
        match individual {
            MechanicalWave => Some((1.0, 200.0)), // 1-200 Hz general mechanical
            ResonanceFrequency => Some((20.0, 120.0)), // 20-120 Hz therapeutic range
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Axiom: biophysics taxonomy is a DAG.
pub struct BiophysicsTaxonomyIsDAG;

impl Axiom for BiophysicsTaxonomyIsDAG {
    fn description(&self) -> &str {
        "biophysics taxonomy is a directed acyclic graph"
    }

    fn holds(&self) -> bool {
        taxonomy::NoCycles::<BiophysicsTaxonomy>::new().holds()
    }
}
pr4xis::register_axiom!(BiophysicsTaxonomyIsDAG);

/// Axiom: biophysics causal graph is asymmetric.
pub struct BiophysicsCausalAsymmetric;

impl Axiom for BiophysicsCausalAsymmetric {
    fn description(&self) -> &str {
        "biophysics causal graph is asymmetric"
    }

    fn holds(&self) -> bool {
        causation::Asymmetric::<BiophysicsCauses>::new().holds()
    }
}
pr4xis::register_axiom!(BiophysicsCausalAsymmetric);

/// Axiom: ExternalVibration transitively causes MechanotransducerActivation.
pub struct VibrationCausesMechanotransduction;

impl Axiom for VibrationCausesMechanotransduction {
    fn description(&self) -> &str {
        "external vibration transitively causes mechanotransducer activation \
         (full chain: vibration -> wave -> deformation -> strain -> activation)"
    }

    fn holds(&self) -> bool {
        use BiophysicsCausalEvent::*;
        let effects = causation::effects_of::<BiophysicsCauses>(&ExternalVibration);
        effects.contains(&MechanotransducerActivation)
    }
}
pr4xis::register_axiom!(VibrationCausesMechanotransduction);

/// Axiom: PiezoelectricChargeGeneration follows TissueDeformation.
/// Fukada & Yasuda 1957: mechanical stress on bone generates electric charge.
pub struct PiezoelectricFollowsDeformation;

impl Axiom for PiezoelectricFollowsDeformation {
    fn description(&self) -> &str {
        "piezoelectric charge generation follows tissue deformation \
         (Fukada & Yasuda 1957)"
    }

    fn holds(&self) -> bool {
        use BiophysicsCausalEvent::*;
        let effects = causation::effects_of::<BiophysicsCauses>(&TissueDeformation);
        effects.contains(&PiezoelectricChargeGeneration)
    }
}
pr4xis::register_axiom!(PiezoelectricFollowsDeformation);

/// Axiom: BoneMatrix is piezoelectric (collagen content).
pub struct BoneMatrixIsPiezoelectric;

impl Axiom for BoneMatrixIsPiezoelectric {
    fn description(&self) -> &str {
        "bone matrix is piezoelectric due to collagen content"
    }

    fn holds(&self) -> bool {
        IsPiezoelectric.get(&BiophysicsEntity::BoneMatrix) == Some(true)
    }
}
pr4xis::register_axiom!(BoneMatrixIsPiezoelectric);

/// Axiom: BoneMatrix impedance > SoftTissue impedance (impedance mismatch = reflection).
pub struct BoneImpedanceGreaterThanSoftTissue;

impl Axiom for BoneImpedanceGreaterThanSoftTissue {
    fn description(&self) -> &str {
        "bone matrix acoustic impedance exceeds soft tissue impedance \
         (impedance mismatch causes wave reflection)"
    }

    fn holds(&self) -> bool {
        let bone_z = AcousticImpedanceValue.get(&BiophysicsEntity::BoneMatrix);
        let soft_z = AcousticImpedanceValue.get(&BiophysicsEntity::SoftTissue);
        match (bone_z, soft_z) {
            (Some(b), Some(s)) => b > s,
            _ => false,
        }
    }
}
pr4xis::register_axiom!(BoneImpedanceGreaterThanSoftTissue);

/// Axiom: ImpedanceMismatch causes WaveReflection.
pub struct ImpedanceMismatchCausesReflection;

impl Axiom for ImpedanceMismatchCausesReflection {
    fn description(&self) -> &str {
        "impedance mismatch causes wave reflection"
    }

    fn holds(&self) -> bool {
        use BiophysicsCausalEvent::*;
        let effects = causation::effects_of::<BiophysicsCauses>(&ImpedanceMismatch);
        effects.contains(&WaveReflection)
    }
}
pr4xis::register_axiom!(ImpedanceMismatchCausesReflection);

/// Axiom: biophysics opposition is symmetric.
pub struct BiophysicsOppositionSymmetric;

impl Axiom for BiophysicsOppositionSymmetric {
    fn description(&self) -> &str {
        "biophysics opposition is symmetric"
    }

    fn holds(&self) -> bool {
        opposition::Symmetric::<BiophysicsOpposition>::new().holds()
    }
}
pr4xis::register_axiom!(BiophysicsOppositionSymmetric);

/// Axiom: biophysics opposition is irreflexive.
pub struct BiophysicsOppositionIrreflexive;

impl Axiom for BiophysicsOppositionIrreflexive {
    fn description(&self) -> &str {
        "biophysics opposition is irreflexive"
    }

    fn holds(&self) -> bool {
        opposition::Irreflexive::<BiophysicsOpposition>::new().holds()
    }
}
pr4xis::register_axiom!(BiophysicsOppositionIrreflexive);

/// Axiom: no biophysics causal event directly causes itself.
pub struct BiophysicsCausalNoSelfCausation;

impl Axiom for BiophysicsCausalNoSelfCausation {
    fn description(&self) -> &str {
        "no biophysics causal event directly causes itself"
    }

    fn holds(&self) -> bool {
        causation::NoSelfCausation::<BiophysicsCauses>::new().holds()
    }
}
pr4xis::register_axiom!(BiophysicsCausalNoSelfCausation);

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// Top-level biophysics ontology tying together category, qualities, and axioms.
pub struct BiophysicsOntology;

impl Ontology for BiophysicsOntology {
    type Cat = BiophysicsCategory;
    type Qual = AcousticImpedanceValue;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        BiophysicsOntologyMeta::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(VibrationCausesMechanotransduction),
            Box::new(PiezoelectricFollowsDeformation),
            Box::new(BoneMatrixIsPiezoelectric),
            Box::new(BoneImpedanceGreaterThanSoftTissue),
            Box::new(ImpedanceMismatchCausesReflection),
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
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;

    // -- Axiom tests --

    #[test]
    fn test_taxonomy_is_dag() {
        assert!(
            BiophysicsTaxonomyIsDAG.holds(),
            "{}",
            BiophysicsTaxonomyIsDAG.description()
        );
    }

    #[test]
    fn test_causal_asymmetric() {
        assert!(
            BiophysicsCausalAsymmetric.holds(),
            "{}",
            BiophysicsCausalAsymmetric.description()
        );
    }

    #[test]
    fn test_causal_no_self_causation() {
        assert!(
            BiophysicsCausalNoSelfCausation.holds(),
            "{}",
            BiophysicsCausalNoSelfCausation.description()
        );
    }

    #[test]
    fn test_vibration_causes_mechanotransduction() {
        assert!(
            VibrationCausesMechanotransduction.holds(),
            "{}",
            VibrationCausesMechanotransduction.description()
        );
    }

    #[test]
    fn test_piezoelectric_follows_deformation() {
        assert!(
            PiezoelectricFollowsDeformation.holds(),
            "{}",
            PiezoelectricFollowsDeformation.description()
        );
    }

    #[test]
    fn test_bone_matrix_is_piezoelectric() {
        assert!(
            BoneMatrixIsPiezoelectric.holds(),
            "{}",
            BoneMatrixIsPiezoelectric.description()
        );
    }

    #[test]
    fn test_bone_impedance_greater_than_soft_tissue() {
        assert!(
            BoneImpedanceGreaterThanSoftTissue.holds(),
            "{}",
            BoneImpedanceGreaterThanSoftTissue.description()
        );
    }

    #[test]
    fn test_impedance_mismatch_causes_reflection() {
        assert!(
            ImpedanceMismatchCausesReflection.holds(),
            "{}",
            ImpedanceMismatchCausesReflection.description()
        );
    }

    #[test]
    fn test_opposition_symmetric() {
        assert!(
            BiophysicsOppositionSymmetric.holds(),
            "{}",
            BiophysicsOppositionSymmetric.description()
        );
    }

    #[test]
    fn test_opposition_irreflexive() {
        assert!(
            BiophysicsOppositionIrreflexive.holds(),
            "{}",
            BiophysicsOppositionIrreflexive.description()
        );
    }

    // -- Category law tests --

    #[test]
    fn test_biophysics_category_laws() {
        check_category_laws::<BiophysicsCategory>().unwrap();
    }

    #[test]
    fn test_biophysics_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<BiophysicsTaxonomy>>().unwrap();
    }

    #[test]
    fn test_biophysics_causal_category_laws() {
        use pr4xis::ontology::reasoning::causation::CausalCategory;
        check_category_laws::<CausalCategory<BiophysicsCauses>>().unwrap();
    }

    // -- Taxonomy tests --

    #[test]
    fn test_mechanical_properties_are_mechanical() {
        use BiophysicsEntity::*;
        for entity in [
            Viscoelasticity,
            Elasticity,
            Viscosity,
            StiffnessModulus,
            StrainRate,
            MechanicalStress,
            MechanicalStrain,
            MembraneCapacitance,
            MembraneTension,
            CellDeformation,
        ] {
            assert!(
                taxonomy::is_a::<BiophysicsTaxonomy>(&entity, &MechanicalProperty),
                "{:?} should be a MechanicalProperty",
                entity
            );
        }
    }

    #[test]
    fn test_wave_properties_are_wave() {
        use BiophysicsEntity::*;
        for entity in [
            MechanicalWave,
            AcousticImpedance,
            Attenuation,
            Frequency,
            Wavelength,
            ResonanceFrequency,
        ] {
            assert!(
                taxonomy::is_a::<BiophysicsTaxonomy>(&entity, &WaveProperty),
                "{:?} should be a WaveProperty",
                entity
            );
        }
    }

    #[test]
    fn test_piezoelectric_are_piezoelectric() {
        use BiophysicsEntity::*;
        for entity in [
            PiezoelectricEffect,
            DirectPiezoelectric,
            ConversePiezoelectric,
            CollagenPiezoelectricity,
        ] {
            assert!(
                taxonomy::is_a::<BiophysicsTaxonomy>(&entity, &PiezoelectricProperty),
                "{:?} should be a PiezoelectricProperty",
                entity
            );
        }
    }

    #[test]
    fn test_direct_and_converse_are_piezoelectric_effect() {
        use BiophysicsEntity::*;
        assert!(taxonomy::is_a::<BiophysicsTaxonomy>(
            &DirectPiezoelectric,
            &PiezoelectricEffect
        ));
        assert!(taxonomy::is_a::<BiophysicsTaxonomy>(
            &ConversePiezoelectric,
            &PiezoelectricEffect
        ));
    }

    #[test]
    fn test_media_are_biological_medium() {
        use BiophysicsEntity::*;
        for entity in [BoneMatrix, SoftTissue, FluidMedium, CellMembrane] {
            assert!(
                taxonomy::is_a::<BiophysicsTaxonomy>(&entity, &BiologicalMedium),
                "{:?} should be a BiologicalMedium",
                entity
            );
        }
    }

    #[test]
    fn test_entity_count() {
        assert_eq!(BiophysicsEntity::variants().len(), 28);
    }

    #[test]
    fn test_causal_event_count() {
        assert_eq!(BiophysicsCausalEvent::variants().len(), 10);
    }

    // -- Opposition tests --

    #[test]
    fn test_direct_opposes_converse_piezoelectric() {
        use BiophysicsEntity::*;
        assert!(opposition::are_opposed::<BiophysicsOpposition>(
            &DirectPiezoelectric,
            &ConversePiezoelectric
        ));
        assert!(opposition::are_opposed::<BiophysicsOpposition>(
            &ConversePiezoelectric,
            &DirectPiezoelectric
        ));
    }

    #[test]
    fn test_elasticity_opposes_viscosity() {
        use BiophysicsEntity::*;
        assert!(opposition::are_opposed::<BiophysicsOpposition>(
            &Elasticity,
            &Viscosity
        ));
    }

    #[test]
    fn test_stress_opposes_strain() {
        use BiophysicsEntity::*;
        assert!(opposition::are_opposed::<BiophysicsOpposition>(
            &MechanicalStress,
            &MechanicalStrain
        ));
    }

    #[test]
    fn test_elasticity_does_not_oppose_stress() {
        use BiophysicsEntity::*;
        assert!(!opposition::are_opposed::<BiophysicsOpposition>(
            &Elasticity,
            &MechanicalStress
        ));
    }

    // -- Quality tests --

    #[test]
    fn test_bone_impedance_value() {
        let z = AcousticImpedanceValue.get(&BiophysicsEntity::BoneMatrix);
        assert_eq!(z, Some(7.4));
    }

    #[test]
    fn test_soft_tissue_impedance_value() {
        let z = AcousticImpedanceValue.get(&BiophysicsEntity::SoftTissue);
        assert_eq!(z, Some(1.6));
    }

    #[test]
    fn test_fluid_medium_impedance_value() {
        let z = AcousticImpedanceValue.get(&BiophysicsEntity::FluidMedium);
        assert_eq!(z, Some(1.5));
    }

    #[test]
    fn test_bone_is_piezoelectric() {
        assert_eq!(
            IsPiezoelectric.get(&BiophysicsEntity::BoneMatrix),
            Some(true)
        );
    }

    #[test]
    fn test_soft_tissue_not_piezoelectric() {
        assert_eq!(
            IsPiezoelectric.get(&BiophysicsEntity::SoftTissue),
            Some(false)
        );
    }

    #[test]
    fn test_all_media_transmit_vibration() {
        use BiophysicsEntity::*;
        for medium in [BoneMatrix, SoftTissue, FluidMedium, CellMembrane] {
            assert_eq!(
                TransmitsVibration.get(&medium),
                Some(true),
                "{:?} should transmit vibration",
                medium
            );
        }
    }

    #[test]
    fn test_mechanical_wave_frequency_range() {
        let range = FrequencyRange.get(&BiophysicsEntity::MechanicalWave);
        assert_eq!(range, Some((1.0, 200.0)));
    }

    #[test]
    fn test_resonance_frequency_range() {
        let range = FrequencyRange.get(&BiophysicsEntity::ResonanceFrequency);
        assert_eq!(range, Some((20.0, 120.0)));
    }

    // -- Causal chain tests --

    #[test]
    fn test_external_vibration_causes_wave_propagation() {
        use BiophysicsCausalEvent::*;
        let effects = causation::effects_of::<BiophysicsCauses>(&ExternalVibration);
        assert!(effects.contains(&WavePropagation));
    }

    #[test]
    fn test_vibration_full_chain_to_activation() {
        use BiophysicsCausalEvent::*;
        let effects = causation::effects_of::<BiophysicsCauses>(&ExternalVibration);
        assert!(effects.contains(&WavePropagation));
        assert!(effects.contains(&TissueDeformation));
        assert!(effects.contains(&CellMembraneStrain));
        assert!(effects.contains(&MechanotransducerActivation));
    }

    #[test]
    fn test_vibration_bone_conduction_path() {
        use BiophysicsCausalEvent::*;
        let effects = causation::effects_of::<BiophysicsCauses>(&ExternalVibration);
        assert!(effects.contains(&BoneConductionTransmission));
    }

    #[test]
    fn test_tissue_deformation_causes_piezoelectric_charge() {
        use BiophysicsCausalEvent::*;
        let effects = causation::effects_of::<BiophysicsCauses>(&TissueDeformation);
        assert!(effects.contains(&PiezoelectricChargeGeneration));
        assert!(effects.contains(&LocalElectricField));
    }

    // -- Ontology validation --

    #[test]
    fn test_ontology_validates() {
        BiophysicsOntology::validate().unwrap();
    }

    // -- Property-based tests (proptest) --

    use proptest::prelude::*;

    fn arb_biophysics_entity() -> impl Strategy<Value = BiophysicsEntity> {
        (0..BiophysicsEntity::variants().len()).prop_map(|i| BiophysicsEntity::variants()[i])
    }

    fn arb_biophysics_causal_event() -> impl Strategy<Value = BiophysicsCausalEvent> {
        (0..BiophysicsCausalEvent::variants().len())
            .prop_map(|i| BiophysicsCausalEvent::variants()[i])
    }

    proptest! {
        /// For any BiophysicsEntity, taxonomy is-a is reflexive.
        #[test]
        fn prop_taxonomy_is_a_reflexive(entity in arb_biophysics_entity()) {
            prop_assert!(
                taxonomy::is_a::<BiophysicsTaxonomy>(&entity, &entity),
                "is-a should be reflexive for {:?}",
                entity
            );
        }

        /// AcousticImpedanceValue for media is always positive.
        #[test]
        fn prop_impedance_always_positive(entity in arb_biophysics_entity()) {
            if let Some(z) = AcousticImpedanceValue.get(&entity) {
                prop_assert!(z > 0.0, "impedance must be positive for {:?}", entity);
            }
        }

        /// FrequencyRange min < max when defined.
        #[test]
        fn prop_frequency_range_valid(entity in arb_biophysics_entity()) {
            if let Some((min, max)) = FrequencyRange.get(&entity) {
                prop_assert!(min < max, "frequency range min must be < max for {:?}", entity);
            }
        }

        /// No causal event causes itself (checked via proptest).
        #[test]
        fn prop_no_self_causation(event in arb_biophysics_causal_event()) {
            let relations = BiophysicsCauses::relations();
            let self_caused = relations.iter().any(|(a, b)| *a == event && *b == event);
            prop_assert!(
                !self_caused,
                "event {:?} should not directly cause itself",
                event
            );
        }
    }
}
