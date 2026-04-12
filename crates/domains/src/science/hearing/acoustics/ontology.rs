//! Acoustics ontology.
//!
//! Models the physics of sound: waves, propagation media, acoustic phenomena.
//! Taxonomy: wave type hierarchy, medium hierarchy.
//! Mereology: sound wave has-a frequency, amplitude, wavelength.
//! Causal graph: source vibration → medium coupling → wave propagation → reception.
//!
//! Key references:
//! - Kinsler et al. 2000: Fundamentals of Acoustics (4th ed.)
//! - Pierce 2019: Acoustics: An Introduction to Its Physical Principles
//! - Stenfelt & Goode 2005: bone vs air conduction impedance
//! - von Békésy 1960: Experiments in Hearing

use praxis::category::{Category, Entity, Relationship};
use praxis::ontology::reasoning::causation::{self, CausalDef};
use praxis::ontology::reasoning::mereology::{self, MereologyDef};
use praxis::ontology::reasoning::opposition::{self, OppositionDef};
use praxis::ontology::reasoning::taxonomy::{self, TaxonomyDef};
use praxis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

/// Every entity in the acoustics domain.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AcousticEntity {
    // Wave properties
    Frequency,
    Amplitude,
    Wavelength,
    Phase,
    Intensity,
    // Wave types
    SoundWave,
    LongitudinalWave,
    TransverseWave,
    ShearWave,
    // Propagation media
    Air,
    Water,
    CorticalBone,
    CancellousBone,
    SoftTissue,
    Cartilage,
    Fluid,
    // Acoustic phenomena
    Resonance,
    Reflection,
    Refraction,
    Diffraction,
    Absorption,
    Attenuation,
    ImpedanceMismatch,
    // Abstract categories
    Wave,
    Medium,
    WaveProperty,
    AcousticPhenomenon,
    Solid,
    BoneTissue,
}

impl Entity for AcousticEntity {
    fn variants() -> Vec<Self> {
        use AcousticEntity::*;
        vec![
            Frequency,
            Amplitude,
            Wavelength,
            Phase,
            Intensity,
            SoundWave,
            LongitudinalWave,
            TransverseWave,
            ShearWave,
            Air,
            Water,
            CorticalBone,
            CancellousBone,
            SoftTissue,
            Cartilage,
            Fluid,
            Resonance,
            Reflection,
            Refraction,
            Diffraction,
            Absorption,
            Attenuation,
            ImpedanceMismatch,
            Wave,
            Medium,
            WaveProperty,
            AcousticPhenomenon,
            Solid,
            BoneTissue,
        ]
    }
}

// ---------------------------------------------------------------------------
// Taxonomy (is-a)
// ---------------------------------------------------------------------------

/// Subsumption hierarchy for acoustic entities.
pub struct AcousticTaxonomy;

impl TaxonomyDef for AcousticTaxonomy {
    type Entity = AcousticEntity;

    fn relations() -> Vec<(AcousticEntity, AcousticEntity)> {
        use AcousticEntity::*;
        vec![
            // Wave types is-a Wave
            (SoundWave, Wave),
            (LongitudinalWave, Wave),
            (TransverseWave, Wave),
            (ShearWave, Wave),
            // Sound wave is specifically longitudinal (in fluids)
            (SoundWave, LongitudinalWave),
            // Media is-a Medium
            (Air, Medium),
            (Water, Medium),
            (CorticalBone, Medium),
            (CancellousBone, Medium),
            (SoftTissue, Medium),
            (Cartilage, Medium),
            (Fluid, Medium),
            // Bone subtypes
            (CorticalBone, BoneTissue),
            (CancellousBone, BoneTissue),
            (BoneTissue, Solid),
            (Cartilage, Solid),
            // Fluid media
            (Air, Fluid),
            (Water, Fluid),
            // Solid is-a Medium
            (Solid, Medium),
            (Fluid, Medium),
            // Wave properties
            (Frequency, WaveProperty),
            (Amplitude, WaveProperty),
            (Wavelength, WaveProperty),
            (Phase, WaveProperty),
            (Intensity, WaveProperty),
            // Acoustic phenomena
            (Resonance, AcousticPhenomenon),
            (Reflection, AcousticPhenomenon),
            (Refraction, AcousticPhenomenon),
            (Diffraction, AcousticPhenomenon),
            (Absorption, AcousticPhenomenon),
            (Attenuation, AcousticPhenomenon),
            (ImpedanceMismatch, AcousticPhenomenon),
        ]
    }
}

// ---------------------------------------------------------------------------
// Mereology (has-a / part-whole)
// ---------------------------------------------------------------------------

/// Part-whole relationships for acoustic entities.
pub struct AcousticMereology;

impl MereologyDef for AcousticMereology {
    type Entity = AcousticEntity;

    fn relations() -> Vec<(AcousticEntity, AcousticEntity)> {
        use AcousticEntity::*;
        vec![
            // A sound wave has-a frequency, amplitude, wavelength, phase, intensity
            (SoundWave, Frequency),
            (SoundWave, Amplitude),
            (SoundWave, Wavelength),
            (SoundWave, Phase),
            (SoundWave, Intensity),
        ]
    }
}

// ---------------------------------------------------------------------------
// Causal graph
// ---------------------------------------------------------------------------

/// Causal events in acoustic wave propagation.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AcousticCausalEvent {
    SourceVibration,
    MediumCoupling,
    WavePropagation,
    BoundaryEncounter,
    ImpedanceTransition,
    EnergyReflection,
    EnergyTransmission,
    EnergyAbsorption,
    WaveAttenuation,
    ResonantAmplification,
    ReceiverExcitation,
}

impl Entity for AcousticCausalEvent {
    fn variants() -> Vec<Self> {
        use AcousticCausalEvent::*;
        vec![
            SourceVibration,
            MediumCoupling,
            WavePropagation,
            BoundaryEncounter,
            ImpedanceTransition,
            EnergyReflection,
            EnergyTransmission,
            EnergyAbsorption,
            WaveAttenuation,
            ResonantAmplification,
            ReceiverExcitation,
        ]
    }
}

/// Causal graph for acoustic wave propagation.
pub struct AcousticCausalGraph;

impl CausalDef for AcousticCausalGraph {
    type Entity = AcousticCausalEvent;

    fn relations() -> Vec<(AcousticCausalEvent, AcousticCausalEvent)> {
        use AcousticCausalEvent::*;
        vec![
            // Source vibration couples into medium
            (SourceVibration, MediumCoupling),
            // Medium coupling initiates wave propagation
            (MediumCoupling, WavePropagation),
            // Propagation encounters boundaries
            (WavePropagation, BoundaryEncounter),
            // Propagation attenuates with distance
            (WavePropagation, WaveAttenuation),
            // Boundary creates impedance transition
            (BoundaryEncounter, ImpedanceTransition),
            // Impedance transition causes reflection and transmission
            (ImpedanceTransition, EnergyReflection),
            (ImpedanceTransition, EnergyTransmission),
            (ImpedanceTransition, EnergyAbsorption),
            // Transmitted energy can resonate
            (EnergyTransmission, ResonantAmplification),
            // Transmitted energy excites receiver
            (EnergyTransmission, ReceiverExcitation),
            // Resonance amplifies receiver excitation
            (ResonantAmplification, ReceiverExcitation),
        ]
    }
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

/// A morphism in the acoustics category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcousticRelation {
    pub from: AcousticEntity,
    pub to: AcousticEntity,
}

impl Relationship for AcousticRelation {
    type Object = AcousticEntity;

    fn source(&self) -> AcousticEntity {
        self.from
    }

    fn target(&self) -> AcousticEntity {
        self.to
    }
}

/// Discrete category over acoustic entities.
pub struct AcousticsCategory;

impl Category for AcousticsCategory {
    type Object = AcousticEntity;
    type Morphism = AcousticRelation;

    fn identity(obj: &AcousticEntity) -> AcousticRelation {
        AcousticRelation {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &AcousticRelation, g: &AcousticRelation) -> Option<AcousticRelation> {
        if f.to != g.from {
            return None;
        }
        Some(AcousticRelation {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<AcousticRelation> {
        let variants = AcousticEntity::variants();
        let mut morphisms = Vec::new();
        for &from in &variants {
            for &to in &variants {
                morphisms.push(AcousticRelation { from, to });
            }
        }
        morphisms
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Speed of sound in a given medium (m/s).
///
/// Values from Kinsler et al. 2000, Table 5.1; bone values from
/// Stenfelt & Goode 2005.
#[derive(Debug, Clone)]
pub struct SpeedOfSound;

impl Quality for SpeedOfSound {
    type Individual = AcousticEntity;
    type Value = f64;

    fn get(&self, individual: &AcousticEntity) -> Option<f64> {
        use AcousticEntity::*;
        match individual {
            Air => Some(343.0),             // at 20C, 1 atm
            Water => Some(1480.0),          // at 20C
            CorticalBone => Some(4080.0),   // Stenfelt & Goode 2005
            CancellousBone => Some(1800.0), // varies widely, ~1500-2000
            SoftTissue => Some(1540.0),     // similar to water
            Cartilage => Some(1665.0),      // Mow & Huiskes 2005
            _ => None,
        }
    }
}

/// Acoustic impedance Z = rho * c (Pa*s/m = Rayl).
///
/// Critical for understanding energy transfer at boundaries.
/// Stenfelt & Goode 2005; Kinsler et al. 2000.
#[derive(Debug, Clone)]
pub struct AcousticImpedance;

impl Quality for AcousticImpedance {
    type Individual = AcousticEntity;
    type Value = f64;

    fn get(&self, individual: &AcousticEntity) -> Option<f64> {
        use AcousticEntity::*;
        match individual {
            Air => Some(413.0),             // 1.2 kg/m3 * 343 m/s
            Water => Some(1.48e6),          // 1000 * 1480
            CorticalBone => Some(7.38e6),   // 1810 * 4080 (Stenfelt 2005)
            CancellousBone => Some(1.44e6), // ~800 * 1800
            SoftTissue => Some(1.63e6),     // 1060 * 1540
            Cartilage => Some(1.83e6),      // 1100 * 1665
            _ => None,
        }
    }
}

/// Whether a medium supports shear waves (transverse waves).
/// Solids support both longitudinal and shear; fluids only longitudinal.
#[derive(Debug, Clone)]
pub struct SupportsShearWaves;

impl Quality for SupportsShearWaves {
    type Individual = AcousticEntity;
    type Value = bool;

    fn get(&self, individual: &AcousticEntity) -> Option<bool> {
        use AcousticEntity::*;
        match individual {
            Air | Water => Some(false),
            CorticalBone | CancellousBone | Cartilage => Some(true),
            SoftTissue => Some(false), // effectively a fluid for acoustic purposes
            _ => None,
        }
    }
}

/// The medium state: solid, liquid, or gas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediumState {
    Gas,
    Liquid,
    SolidState,
}

/// Quality: physical state of the medium.
#[derive(Debug, Clone)]
pub struct MediumPhase;

impl Quality for MediumPhase {
    type Individual = AcousticEntity;
    type Value = MediumState;

    fn get(&self, individual: &AcousticEntity) -> Option<MediumState> {
        use AcousticEntity::*;
        match individual {
            Air => Some(MediumState::Gas),
            Water => Some(MediumState::Liquid),
            CorticalBone | CancellousBone | Cartilage => Some(MediumState::SolidState),
            SoftTissue => Some(MediumState::Liquid), // acoustically liquid-like
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Opposition
// ---------------------------------------------------------------------------

/// Opposition pairs in acoustics.
///
/// - Reflection vs Transmission: energy splits at boundary
/// - Absorption vs Resonance: energy damped vs amplified
/// - LongitudinalWave vs TransverseWave: compression vs shear
pub struct AcousticOpposition;

impl OppositionDef for AcousticOpposition {
    type Entity = AcousticEntity;

    fn pairs() -> Vec<(AcousticEntity, AcousticEntity)> {
        use AcousticEntity::*;
        vec![
            (Reflection, Refraction),
            (Absorption, Resonance),
            (LongitudinalWave, TransverseWave),
        ]
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// The taxonomy has no cycles.
pub struct AcousticTaxonomyIsDAG;

impl Axiom for AcousticTaxonomyIsDAG {
    fn description(&self) -> &str {
        "acoustic taxonomy is a directed acyclic graph"
    }

    fn holds(&self) -> bool {
        taxonomy::NoCycles::<AcousticTaxonomy>::new().holds()
    }
}

/// The taxonomy is antisymmetric.
pub struct AcousticTaxonomyIsAntisymmetric;

impl Axiom for AcousticTaxonomyIsAntisymmetric {
    fn description(&self) -> &str {
        "acoustic taxonomy is antisymmetric"
    }

    fn holds(&self) -> bool {
        taxonomy::Antisymmetric::<AcousticTaxonomy>::new().holds()
    }
}

/// The mereology has no cycles.
pub struct AcousticMereologyIsDAG;

impl Axiom for AcousticMereologyIsDAG {
    fn description(&self) -> &str {
        "acoustic mereology is a directed acyclic graph"
    }

    fn holds(&self) -> bool {
        mereology::NoCycles::<AcousticMereology>::new().holds()
    }
}

/// Sound speed in bone > sound speed in air (fundamental to bone conduction).
///
/// Stenfelt & Goode 2005: cortical bone ~4080 m/s vs air 343 m/s.
pub struct BoneFasterThanAir;

impl Axiom for BoneFasterThanAir {
    fn description(&self) -> &str {
        "speed of sound in cortical bone exceeds speed in air"
    }

    fn holds(&self) -> bool {
        use AcousticEntity::*;
        let s = SpeedOfSound;
        s.get(&CorticalBone).unwrap() > s.get(&Air).unwrap()
    }
}

/// Impedance mismatch: bone impedance >> air impedance (~18,000:1).
///
/// This ratio explains why air conduction is inefficient for coupling
/// into bone, and why bone conduction transducers bypass this barrier.
/// Stenfelt & Goode 2005.
pub struct BoneAirImpedanceMismatch;

impl Axiom for BoneAirImpedanceMismatch {
    fn description(&self) -> &str {
        "bone acoustic impedance is at least 1000x air impedance"
    }

    fn holds(&self) -> bool {
        use AcousticEntity::*;
        let z = AcousticImpedance;
        let bone_z = z.get(&CorticalBone).unwrap();
        let air_z = z.get(&Air).unwrap();
        bone_z / air_z > 1000.0
    }
}

/// Soft tissue impedance is close to water (within 15%).
///
/// This is why ultrasound works well through soft tissue but not through
/// bone or air. Kinsler et al. 2000.
pub struct SoftTissueMatchesWater;

impl Axiom for SoftTissueMatchesWater {
    fn description(&self) -> &str {
        "soft tissue impedance is within 15% of water"
    }

    fn holds(&self) -> bool {
        use AcousticEntity::*;
        let z = AcousticImpedance;
        let tissue_z = z.get(&SoftTissue).unwrap();
        let water_z = z.get(&Water).unwrap();
        let ratio = tissue_z / water_z;
        (0.85..=1.15).contains(&ratio)
    }
}

/// Only solids support shear waves; fluids do not.
///
/// Kinsler et al. 2000, Ch. 6.
pub struct OnlySolidsHaveShearWaves;

impl Axiom for OnlySolidsHaveShearWaves {
    fn description(&self) -> &str {
        "only solid media support shear waves"
    }

    fn holds(&self) -> bool {
        use AcousticEntity::*;
        let shear = SupportsShearWaves;
        // Fluids: no shear
        shear.get(&Air) == Some(false)
            && shear.get(&Water) == Some(false)
            // Solids: shear
            && shear.get(&CorticalBone) == Some(true)
            && shear.get(&CancellousBone) == Some(true)
            && shear.get(&Cartilage) == Some(true)
    }
}

/// Causal graph is asymmetric.
pub struct AcousticCausalGraphIsAsymmetric;

impl Axiom for AcousticCausalGraphIsAsymmetric {
    fn description(&self) -> &str {
        "acoustic causal graph is asymmetric"
    }

    fn holds(&self) -> bool {
        causation::Asymmetric::<AcousticCausalGraph>::new().holds()
    }
}

/// No event causes itself.
pub struct AcousticCausalGraphNoSelfCause;

impl Axiom for AcousticCausalGraphNoSelfCause {
    fn description(&self) -> &str {
        "no acoustic event causes itself"
    }

    fn holds(&self) -> bool {
        causation::NoSelfCausation::<AcousticCausalGraph>::new().holds()
    }
}

/// Source vibration transitively causes receiver excitation.
pub struct SourceCausesReceiver;

impl Axiom for SourceCausesReceiver {
    fn description(&self) -> &str {
        "source vibration transitively causes receiver excitation"
    }

    fn holds(&self) -> bool {
        use AcousticCausalEvent::*;
        let effects = causation::effects_of::<AcousticCausalGraph>(&SourceVibration);
        effects.contains(&ReceiverExcitation)
    }
}

/// Opposition is symmetric.
pub struct AcousticOppositionSymmetric;

impl Axiom for AcousticOppositionSymmetric {
    fn description(&self) -> &str {
        "acoustic opposition is symmetric"
    }

    fn holds(&self) -> bool {
        opposition::Symmetric::<AcousticOpposition>::new().holds()
    }
}

/// Opposition is irreflexive.
pub struct AcousticOppositionIrreflexive;

impl Axiom for AcousticOppositionIrreflexive {
    fn description(&self) -> &str {
        "acoustic opposition is irreflexive"
    }

    fn holds(&self) -> bool {
        opposition::Irreflexive::<AcousticOpposition>::new().holds()
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// Top-level acoustics ontology.
pub struct AcousticsOntology;

impl Ontology for AcousticsOntology {
    type Cat = AcousticsCategory;
    type Qual = SpeedOfSound;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(AcousticTaxonomyIsDAG),
            Box::new(AcousticTaxonomyIsAntisymmetric),
            Box::new(AcousticMereologyIsDAG),
            Box::new(BoneFasterThanAir),
            Box::new(BoneAirImpedanceMismatch),
            Box::new(SoftTissueMatchesWater),
            Box::new(OnlySolidsHaveShearWaves),
            Box::new(AcousticCausalGraphIsAsymmetric),
            Box::new(AcousticCausalGraphNoSelfCause),
            Box::new(SourceCausesReceiver),
            Box::new(AcousticOppositionSymmetric),
            Box::new(AcousticOppositionIrreflexive),
        ]
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use praxis::category::validate::check_category_laws;
    use praxis::ontology::reasoning::causation::CausalCategory;
    use praxis::ontology::reasoning::mereology::MereologyCategory;
    use praxis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;

    // -- Axiom tests --

    #[test]
    fn test_taxonomy_is_dag() {
        assert!(
            AcousticTaxonomyIsDAG.holds(),
            "{}",
            AcousticTaxonomyIsDAG.description()
        );
    }

    #[test]
    fn test_taxonomy_is_antisymmetric() {
        assert!(
            AcousticTaxonomyIsAntisymmetric.holds(),
            "{}",
            AcousticTaxonomyIsAntisymmetric.description()
        );
    }

    #[test]
    fn test_mereology_is_dag() {
        assert!(
            AcousticMereologyIsDAG.holds(),
            "{}",
            AcousticMereologyIsDAG.description()
        );
    }

    #[test]
    fn test_bone_faster_than_air() {
        assert!(
            BoneFasterThanAir.holds(),
            "{}",
            BoneFasterThanAir.description()
        );
    }

    #[test]
    fn test_bone_air_impedance_mismatch() {
        assert!(
            BoneAirImpedanceMismatch.holds(),
            "{}",
            BoneAirImpedanceMismatch.description()
        );
    }

    #[test]
    fn test_soft_tissue_matches_water() {
        assert!(
            SoftTissueMatchesWater.holds(),
            "{}",
            SoftTissueMatchesWater.description()
        );
    }

    #[test]
    fn test_only_solids_have_shear_waves() {
        assert!(
            OnlySolidsHaveShearWaves.holds(),
            "{}",
            OnlySolidsHaveShearWaves.description()
        );
    }

    #[test]
    fn test_causal_graph_asymmetric() {
        assert!(
            AcousticCausalGraphIsAsymmetric.holds(),
            "{}",
            AcousticCausalGraphIsAsymmetric.description()
        );
    }

    #[test]
    fn test_causal_graph_no_self_cause() {
        assert!(
            AcousticCausalGraphNoSelfCause.holds(),
            "{}",
            AcousticCausalGraphNoSelfCause.description()
        );
    }

    #[test]
    fn test_source_causes_receiver() {
        assert!(
            SourceCausesReceiver.holds(),
            "{}",
            SourceCausesReceiver.description()
        );
    }

    #[test]
    fn test_opposition_symmetric() {
        assert!(
            AcousticOppositionSymmetric.holds(),
            "{}",
            AcousticOppositionSymmetric.description()
        );
    }

    #[test]
    fn test_opposition_irreflexive() {
        assert!(
            AcousticOppositionIrreflexive.holds(),
            "{}",
            AcousticOppositionIrreflexive.description()
        );
    }

    // -- Category law tests --

    #[test]
    fn test_acoustics_category_laws() {
        check_category_laws::<AcousticsCategory>().unwrap();
    }

    #[test]
    fn test_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<AcousticTaxonomy>>().unwrap();
    }

    #[test]
    fn test_mereology_category_laws() {
        check_category_laws::<MereologyCategory<AcousticMereology>>().unwrap();
    }

    #[test]
    fn test_causal_category_laws() {
        check_category_laws::<CausalCategory<AcousticCausalGraph>>().unwrap();
    }

    // -- Taxonomy tests --

    #[test]
    fn test_sound_wave_is_a_wave() {
        assert!(taxonomy::is_a::<AcousticTaxonomy>(
            &AcousticEntity::SoundWave,
            &AcousticEntity::Wave
        ));
    }

    #[test]
    fn test_sound_wave_is_longitudinal() {
        assert!(taxonomy::is_a::<AcousticTaxonomy>(
            &AcousticEntity::SoundWave,
            &AcousticEntity::LongitudinalWave
        ));
    }

    #[test]
    fn test_cortical_bone_is_a_medium() {
        assert!(taxonomy::is_a::<AcousticTaxonomy>(
            &AcousticEntity::CorticalBone,
            &AcousticEntity::Medium
        ));
    }

    #[test]
    fn test_cortical_bone_is_solid() {
        assert!(taxonomy::is_a::<AcousticTaxonomy>(
            &AcousticEntity::CorticalBone,
            &AcousticEntity::Solid
        ));
    }

    #[test]
    fn test_air_is_fluid() {
        assert!(taxonomy::is_a::<AcousticTaxonomy>(
            &AcousticEntity::Air,
            &AcousticEntity::Fluid
        ));
    }

    // -- Mereology tests --

    #[test]
    fn test_sound_wave_has_frequency() {
        let parts = mereology::parts_of::<AcousticMereology>(&AcousticEntity::SoundWave);
        assert!(parts.contains(&AcousticEntity::Frequency));
    }

    #[test]
    fn test_sound_wave_has_amplitude() {
        let parts = mereology::parts_of::<AcousticMereology>(&AcousticEntity::SoundWave);
        assert!(parts.contains(&AcousticEntity::Amplitude));
    }

    // -- Quality tests --

    #[test]
    fn test_speed_of_sound_air() {
        assert_eq!(SpeedOfSound.get(&AcousticEntity::Air), Some(343.0));
    }

    #[test]
    fn test_speed_of_sound_cortical_bone() {
        assert_eq!(
            SpeedOfSound.get(&AcousticEntity::CorticalBone),
            Some(4080.0)
        );
    }

    #[test]
    fn test_impedance_values_ordered() {
        let z = AcousticImpedance;
        let air = z.get(&AcousticEntity::Air).unwrap();
        let water = z.get(&AcousticEntity::Water).unwrap();
        let bone = z.get(&AcousticEntity::CorticalBone).unwrap();
        assert!(air < water, "air impedance should be less than water");
        assert!(water < bone, "water impedance should be less than bone");
    }

    #[test]
    fn test_ontology_validates() {
        AcousticsOntology::validate().unwrap();
    }

    fn arb_acoustic_entity() -> impl Strategy<Value = AcousticEntity> {
        (0..AcousticEntity::variants().len()).prop_map(|i| AcousticEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_taxonomy_reflexive(entity in arb_acoustic_entity()) {
            prop_assert!(taxonomy::is_a::<AcousticTaxonomy>(&entity, &entity));
        }

        #[test]
        fn prop_media_with_speed_have_impedance(entity in arb_acoustic_entity()) {
            if SpeedOfSound.get(&entity).is_some() {
                prop_assert!(AcousticImpedance.get(&entity).is_some());
            }
        }
    }
}
