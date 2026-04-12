//! Bone conduction ontology.
//!
//! Models how vibration reaches the cochlea through bone rather than air.
//! Three primary mechanisms (Stenfelt 2011; Tonndorf 1966):
//!   1. Osseotympanic: ear canal wall vibration → eardrum vibration
//!   2. Inertial: skull vibration → ossicular chain inertia → oval window
//!   3. Compressional: skull vibration → cochlear wall compression → fluid motion
//!
//! Additional: distortional mode (skull deformation at low frequencies).
//!
//! Key references:
//! - Stenfelt & Goode 2005: comprehensive review of BC pathways
//! - Stenfelt 2011: relative contributions of each mechanism
//! - Tonndorf 1966: original classification of BC mechanisms
//! - von Békésy 1960: foundational impedance measurements
//! - Stenfelt 2015: inner ear compressional mechanism
//! - Reinfeldt et al. 2015: estimation of BC pathways

use pr4xis::category::Entity;
use pr4xis::define_dense_category;
use pr4xis::ontology::reasoning::causation::{self, CausalDef};
use pr4xis::ontology::reasoning::opposition::{self, OppositionDef};
use pr4xis::ontology::reasoning::taxonomy::{self, TaxonomyDef};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

/// Every entity in the bone conduction domain.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum BoneCondEntity {
    // Mechanisms
    OsseotympanicBC,
    InertialBC,
    CompressionalBC,
    DistortionalBC,
    // Physical processes
    SkullVibration,
    EarCanalWallVibration,
    OssicularInertia,
    CochlearWallCompression,
    FluidInertia,
    SkullDeformation,
    SoundRadiation,
    // Transducer types
    BoneAnchoredDevice,
    PercutaneousImplant,
    TranscutaneousDevice,
    SkinDriveTransducer,
    PiezoelectricTransducer,
    ElectromagneticTransducer,
    // Skull regions
    Mastoid,
    Forehead,
    TemporalBone,
    Vertex,
    Teeth,
    // BC-specific phenomena
    OcclusionEffect,
    TranscranialAttenuation,
    SkullResonance,
    ForceLevel,
    // Abstract categories
    BCMechanism,
    BCTransducer,
    ApplicationSite,
    BCPhenomenon,
}

// ---------------------------------------------------------------------------
// Taxonomy (is-a)
// ---------------------------------------------------------------------------

/// Subsumption hierarchy for bone conduction entities.
pub struct BoneCondTaxonomy;

impl TaxonomyDef for BoneCondTaxonomy {
    type Entity = BoneCondEntity;

    fn relations() -> Vec<(BoneCondEntity, BoneCondEntity)> {
        use BoneCondEntity::*;
        vec![
            // BC mechanisms
            (OsseotympanicBC, BCMechanism),
            (InertialBC, BCMechanism),
            (CompressionalBC, BCMechanism),
            (DistortionalBC, BCMechanism),
            // Transducer types
            (BoneAnchoredDevice, BCTransducer),
            (PercutaneousImplant, BCTransducer),
            (TranscutaneousDevice, BCTransducer),
            (SkinDriveTransducer, BCTransducer),
            (PiezoelectricTransducer, BCTransducer),
            (ElectromagneticTransducer, BCTransducer),
            // Application sites
            (Mastoid, ApplicationSite),
            (Forehead, ApplicationSite),
            (TemporalBone, ApplicationSite),
            (Vertex, ApplicationSite),
            (Teeth, ApplicationSite),
            // BC phenomena
            (OcclusionEffect, BCPhenomenon),
            (TranscranialAttenuation, BCPhenomenon),
            (SkullResonance, BCPhenomenon),
        ]
    }
}

// ---------------------------------------------------------------------------
// Causal graph
// ---------------------------------------------------------------------------

/// Causal events in bone conduction hearing.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum BCCausalEvent {
    TransducerActivation,
    SkullCoupling,
    SkullWavePropagation,
    // Osseotympanic pathway
    EarCanalWallMotion,
    TympanicMembraneResponse,
    OsseotympanicStimulation,
    // Inertial pathway
    OssicularLag,
    StapesDisplacement,
    OvalWindowDrive,
    // Compressional pathway
    CochlearBoneCompression,
    DifferentialFluidFlow,
    BasilarMembraneExcitation,
    // Distortional pathway
    SkullModeDeformation,
    InnerEarDistortion,
    // Common endpoint
    CochlearResponse,
}

/// Causal graph for bone conduction pathways.
///
/// All three mechanisms originate from skull vibration but take
/// different pathways to reach the cochlea.
/// Stenfelt 2011; Tonndorf 1966.
pub struct BCCausalGraph;

impl CausalDef for BCCausalGraph {
    type Entity = BCCausalEvent;

    fn relations() -> Vec<(BCCausalEvent, BCCausalEvent)> {
        use BCCausalEvent::*;
        vec![
            // Common: transducer → skull
            (TransducerActivation, SkullCoupling),
            (SkullCoupling, SkullWavePropagation),
            // Osseotympanic pathway
            (SkullWavePropagation, EarCanalWallMotion),
            (EarCanalWallMotion, TympanicMembraneResponse),
            (TympanicMembraneResponse, OsseotympanicStimulation),
            (OsseotympanicStimulation, CochlearResponse),
            // Inertial pathway
            (SkullWavePropagation, OssicularLag),
            (OssicularLag, StapesDisplacement),
            (StapesDisplacement, OvalWindowDrive),
            (OvalWindowDrive, CochlearResponse),
            // Compressional pathway
            (SkullWavePropagation, CochlearBoneCompression),
            (CochlearBoneCompression, DifferentialFluidFlow),
            (DifferentialFluidFlow, BasilarMembraneExcitation),
            (BasilarMembraneExcitation, CochlearResponse),
            // Distortional pathway
            (SkullWavePropagation, SkullModeDeformation),
            (SkullModeDeformation, InnerEarDistortion),
            (InnerEarDistortion, CochlearResponse),
        ]
    }
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

define_dense_category! {
    /// Discrete category over bone conduction entities.
    pub BoneConductionCategory {
        entity: BoneCondEntity,
        relation: BoneCondRelation,
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Dominant frequency range (Hz) where each BC mechanism contributes most.
///
/// - Osseotympanic: < 1000 Hz (low frequencies, Stenfelt & Reinfeldt 2007)
/// - Inertial: 100-3000 Hz (broad range, Stenfelt 2011)
/// - Compressional: > 4000 Hz (high frequencies, Stenfelt 2015)
/// - Distortional: < 400 Hz (very low, skull modes, Stenfelt 2011)
#[derive(Debug, Clone, PartialEq)]
pub struct FrequencyRange {
    pub low: f64,
    pub high: f64,
}

/// Quality: dominant frequency range for each BC mechanism.
#[derive(Debug, Clone)]
pub struct DominantFrequencyRange;

impl Quality for DominantFrequencyRange {
    type Individual = BoneCondEntity;
    type Value = FrequencyRange;

    fn get(&self, individual: &BoneCondEntity) -> Option<FrequencyRange> {
        use BoneCondEntity::*;
        match individual {
            OsseotympanicBC => Some(FrequencyRange {
                low: 20.0,
                high: 1000.0,
            }),
            InertialBC => Some(FrequencyRange {
                low: 100.0,
                high: 3000.0,
            }),
            CompressionalBC => Some(FrequencyRange {
                low: 4000.0,
                high: 10000.0,
            }),
            DistortionalBC => Some(FrequencyRange {
                low: 20.0,
                high: 400.0,
            }),
            _ => None,
        }
    }
}

/// Transcranial attenuation at the application site (dB).
///
/// How much the signal is attenuated crossing the skull to the contralateral ear.
/// Stenfelt 2012: varies by frequency and site.
#[derive(Debug, Clone)]
pub struct TranscranialAttenuationDB;

impl Quality for TranscranialAttenuationDB {
    type Individual = BoneCondEntity;
    type Value = f64;

    fn get(&self, individual: &BoneCondEntity) -> Option<f64> {
        use BoneCondEntity::*;
        match individual {
            Mastoid => Some(10.0),      // ~10 dB average (Stenfelt 2012)
            Forehead => Some(0.0),      // ~0 dB (midline, symmetric)
            TemporalBone => Some(12.0), // ~12 dB
            Vertex => Some(0.0),        // ~0 dB (midline)
            Teeth => Some(5.0),         // ~5 dB (near midline)
            _ => None,
        }
    }
}

/// Skull resonance frequency (Hz) at each application site.
///
/// - Mastoid: ~200 Hz
/// - Forehead: ~800 Hz
///
/// Stenfelt 2011.
#[derive(Debug, Clone)]
pub struct SkullResonanceFrequency;

impl Quality for SkullResonanceFrequency {
    type Individual = BoneCondEntity;
    type Value = f64;

    fn get(&self, individual: &BoneCondEntity) -> Option<f64> {
        use BoneCondEntity::*;
        match individual {
            Mastoid => Some(200.0),
            Forehead => Some(800.0),
            _ => None,
        }
    }
}

/// Whether the application site requires surgical implantation.
#[derive(Debug, Clone)]
pub struct RequiresSurgery;

impl Quality for RequiresSurgery {
    type Individual = BoneCondEntity;
    type Value = bool;

    fn get(&self, individual: &BoneCondEntity) -> Option<bool> {
        use BoneCondEntity::*;
        match individual {
            BoneAnchoredDevice => Some(true),   // BAHA: percutaneous abutment
            PercutaneousImplant => Some(true),  // through-skin fixture
            TranscutaneousDevice => Some(true), // internal magnet
            SkinDriveTransducer => Some(false), // headband, no surgery
            PiezoelectricTransducer => Some(false),
            ElectromagneticTransducer => Some(false),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Opposition
// ---------------------------------------------------------------------------

/// Opposition pairs in bone conduction.
///
/// - OsseotympanicBC vs CompressionalBC: low-frequency vs high-frequency dominant
/// - PercutaneousImplant vs TranscutaneousDevice: through-skin vs across-skin
/// - Mastoid vs Forehead: lateral vs midline application
pub struct BoneCondOpposition;

impl OppositionDef for BoneCondOpposition {
    type Entity = BoneCondEntity;

    fn pairs() -> Vec<(BoneCondEntity, BoneCondEntity)> {
        use BoneCondEntity::*;
        vec![
            (OsseotympanicBC, CompressionalBC),
            (PercutaneousImplant, TranscutaneousDevice),
            (Mastoid, Forehead),
        ]
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Taxonomy is a DAG.
pub struct BCTaxonomyIsDAG;

impl Axiom for BCTaxonomyIsDAG {
    fn description(&self) -> &str {
        "bone conduction taxonomy is a directed acyclic graph"
    }

    fn holds(&self) -> bool {
        taxonomy::NoCycles::<BoneCondTaxonomy>::new().holds()
    }
}

/// Causal graph is asymmetric.
pub struct BCCausalGraphIsAsymmetric;

impl Axiom for BCCausalGraphIsAsymmetric {
    fn description(&self) -> &str {
        "bone conduction causal graph is asymmetric"
    }

    fn holds(&self) -> bool {
        causation::Asymmetric::<BCCausalGraph>::new().holds()
    }
}

/// No event causes itself.
pub struct BCCausalGraphNoSelfCause;

impl Axiom for BCCausalGraphNoSelfCause {
    fn description(&self) -> &str {
        "no bone conduction event causes itself"
    }

    fn holds(&self) -> bool {
        causation::NoSelfCausation::<BCCausalGraph>::new().holds()
    }
}

/// All four BC mechanisms are classified as BCMechanism.
pub struct FourBCMechanisms;

impl Axiom for FourBCMechanisms {
    fn description(&self) -> &str {
        "all four BC mechanisms (osseotympanic, inertial, compressional, distortional) are classified"
    }

    fn holds(&self) -> bool {
        use BoneCondEntity::*;
        [OsseotympanicBC, InertialBC, CompressionalBC, DistortionalBC]
            .iter()
            .all(|m| taxonomy::is_a::<BoneCondTaxonomy>(m, &BCMechanism))
    }
}

/// Transducer activation transitively causes cochlear response.
///
/// This is the fundamental claim of bone conduction hearing.
/// Stenfelt & Goode 2005.
pub struct TransducerCausesCochlearResponse;

impl Axiom for TransducerCausesCochlearResponse {
    fn description(&self) -> &str {
        "transducer activation transitively causes cochlear response"
    }

    fn holds(&self) -> bool {
        use BCCausalEvent::*;
        let effects = causation::effects_of::<BCCausalGraph>(&TransducerActivation);
        effects.contains(&CochlearResponse)
    }
}

/// All three main pathways converge on cochlear response.
///
/// Stenfelt 2011: osseotympanic, inertial, and compressional all reach cochlea.
pub struct AllPathwaysConverge;

impl Axiom for AllPathwaysConverge {
    fn description(&self) -> &str {
        "osseotympanic, inertial, and compressional pathways all reach cochlear response"
    }

    fn holds(&self) -> bool {
        use BCCausalEvent::*;
        let osseotympanic = causation::effects_of::<BCCausalGraph>(&OsseotympanicStimulation);
        let inertial = causation::effects_of::<BCCausalGraph>(&OvalWindowDrive);
        let compressional = causation::effects_of::<BCCausalGraph>(&BasilarMembraneExcitation);
        osseotympanic.contains(&CochlearResponse)
            && inertial.contains(&CochlearResponse)
            && compressional.contains(&CochlearResponse)
    }
}

/// Inertial BC covers the speech frequency range (100-3000 Hz).
///
/// This makes it the most important mechanism for speech perception
/// through bone conduction. Stenfelt 2011.
pub struct InertialCoversSpeechRange;

impl Axiom for InertialCoversSpeechRange {
    fn description(&self) -> &str {
        "inertial BC covers the speech frequency range (100-3000 Hz)"
    }

    fn holds(&self) -> bool {
        use BoneCondEntity::*;
        let freq = DominantFrequencyRange;
        let inertial = freq.get(&InertialBC).unwrap();
        // Speech frequencies: ~300-3000 Hz
        inertial.low <= 300.0 && inertial.high >= 3000.0
    }
}

/// Opposition is symmetric.
pub struct BCOppositionSymmetric;

impl Axiom for BCOppositionSymmetric {
    fn description(&self) -> &str {
        "bone conduction opposition is symmetric"
    }

    fn holds(&self) -> bool {
        opposition::Symmetric::<BoneCondOpposition>::new().holds()
    }
}

/// Opposition is irreflexive.
pub struct BCOppositionIrreflexive;

impl Axiom for BCOppositionIrreflexive {
    fn description(&self) -> &str {
        "bone conduction opposition is irreflexive"
    }

    fn holds(&self) -> bool {
        opposition::Irreflexive::<BoneCondOpposition>::new().holds()
    }
}

/// Forehead resonance frequency is higher than mastoid (Stenfelt 2011).
pub struct ForeheadResonanceHigherThanMastoid;

impl Axiom for ForeheadResonanceHigherThanMastoid {
    fn description(&self) -> &str {
        "forehead skull resonance frequency is higher than mastoid"
    }

    fn holds(&self) -> bool {
        use BoneCondEntity::*;
        SkullResonanceFrequency.get(&Forehead).unwrap()
            > SkullResonanceFrequency.get(&Mastoid).unwrap()
    }
}

/// Midline sites (forehead, vertex) have zero transcranial attenuation.
///
/// Stenfelt 2012.
pub struct MidlineSitesSymmetric;

impl Axiom for MidlineSitesSymmetric {
    fn description(&self) -> &str {
        "midline application sites have zero transcranial attenuation"
    }

    fn holds(&self) -> bool {
        use BoneCondEntity::*;
        TranscranialAttenuationDB.get(&Forehead) == Some(0.0)
            && TranscranialAttenuationDB.get(&Vertex) == Some(0.0)
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// Top-level bone conduction ontology.
pub struct BoneConductionOntology;

impl Ontology for BoneConductionOntology {
    type Cat = BoneConductionCategory;
    type Qual = TranscranialAttenuationDB;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(BCTaxonomyIsDAG),
            Box::new(BCCausalGraphIsAsymmetric),
            Box::new(BCCausalGraphNoSelfCause),
            Box::new(FourBCMechanisms),
            Box::new(TransducerCausesCochlearResponse),
            Box::new(AllPathwaysConverge),
            Box::new(InertialCoversSpeechRange),
            Box::new(ForeheadResonanceHigherThanMastoid),
            Box::new(MidlineSitesSymmetric),
            Box::new(BCOppositionSymmetric),
            Box::new(BCOppositionIrreflexive),
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
    use pr4xis::ontology::reasoning::causation::CausalCategory;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;

    // -- Axiom tests --

    #[test]
    fn test_taxonomy_is_dag() {
        assert!(BCTaxonomyIsDAG.holds(), "{}", BCTaxonomyIsDAG.description());
    }

    #[test]
    fn test_causal_graph_asymmetric() {
        assert!(
            BCCausalGraphIsAsymmetric.holds(),
            "{}",
            BCCausalGraphIsAsymmetric.description()
        );
    }

    #[test]
    fn test_causal_graph_no_self_cause() {
        assert!(
            BCCausalGraphNoSelfCause.holds(),
            "{}",
            BCCausalGraphNoSelfCause.description()
        );
    }

    #[test]
    fn test_four_bc_mechanisms() {
        assert!(
            FourBCMechanisms.holds(),
            "{}",
            FourBCMechanisms.description()
        );
    }

    #[test]
    fn test_transducer_causes_cochlear_response() {
        assert!(
            TransducerCausesCochlearResponse.holds(),
            "{}",
            TransducerCausesCochlearResponse.description()
        );
    }

    #[test]
    fn test_all_pathways_converge() {
        assert!(
            AllPathwaysConverge.holds(),
            "{}",
            AllPathwaysConverge.description()
        );
    }

    #[test]
    fn test_inertial_has_broadest_range() {
        assert!(
            InertialCoversSpeechRange.holds(),
            "{}",
            InertialCoversSpeechRange.description()
        );
    }

    #[test]
    fn test_midline_sites_symmetric() {
        assert!(
            MidlineSitesSymmetric.holds(),
            "{}",
            MidlineSitesSymmetric.description()
        );
    }

    // -- Opposition tests --

    #[test]
    fn test_opposition_symmetric() {
        assert!(
            BCOppositionSymmetric.holds(),
            "{}",
            BCOppositionSymmetric.description()
        );
    }

    #[test]
    fn test_opposition_irreflexive() {
        assert!(
            BCOppositionIrreflexive.holds(),
            "{}",
            BCOppositionIrreflexive.description()
        );
    }

    #[test]
    fn test_osseotympanic_opposes_compressional() {
        assert!(opposition::are_opposed::<BoneCondOpposition>(
            &BoneCondEntity::OsseotympanicBC,
            &BoneCondEntity::CompressionalBC
        ));
    }

    // -- Category law tests --

    #[test]
    fn test_bone_conduction_category_laws() {
        check_category_laws::<BoneConductionCategory>().unwrap();
    }

    #[test]
    fn test_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<BoneCondTaxonomy>>().unwrap();
    }

    #[test]
    fn test_causal_category_laws() {
        check_category_laws::<CausalCategory<BCCausalGraph>>().unwrap();
    }

    // -- Taxonomy tests --

    #[test]
    fn test_inertial_is_bc_mechanism() {
        assert!(taxonomy::is_a::<BoneCondTaxonomy>(
            &BoneCondEntity::InertialBC,
            &BoneCondEntity::BCMechanism
        ));
    }

    #[test]
    fn test_mastoid_is_application_site() {
        assert!(taxonomy::is_a::<BoneCondTaxonomy>(
            &BoneCondEntity::Mastoid,
            &BoneCondEntity::ApplicationSite
        ));
    }

    #[test]
    fn test_baha_is_transducer() {
        assert!(taxonomy::is_a::<BoneCondTaxonomy>(
            &BoneCondEntity::BoneAnchoredDevice,
            &BoneCondEntity::BCTransducer
        ));
    }

    // -- Causal chain tests --

    #[test]
    fn test_skull_wave_has_multiple_pathways() {
        use BCCausalEvent::*;
        let effects = causation::effects_of::<BCCausalGraph>(&SkullWavePropagation);
        // Should reach all pathway-specific events
        assert!(
            effects.contains(&EarCanalWallMotion),
            "should reach osseotympanic"
        );
        assert!(effects.contains(&OssicularLag), "should reach inertial");
        assert!(
            effects.contains(&CochlearBoneCompression),
            "should reach compressional"
        );
        assert!(
            effects.contains(&SkullModeDeformation),
            "should reach distortional"
        );
    }

    #[test]
    fn test_cochlear_response_has_multiple_causes() {
        use BCCausalEvent::*;
        let causes = causation::causes_of::<BCCausalGraph>(&CochlearResponse);
        assert!(causes.contains(&OsseotympanicStimulation));
        assert!(causes.contains(&OvalWindowDrive));
        assert!(causes.contains(&BasilarMembraneExcitation));
        assert!(causes.contains(&InnerEarDistortion));
    }

    // -- Quality tests --

    #[test]
    fn test_forehead_resonance_higher_than_mastoid() {
        assert!(ForeheadResonanceHigherThanMastoid.holds());
    }

    #[test]
    fn test_mastoid_resonance_frequency() {
        assert_eq!(
            SkullResonanceFrequency.get(&BoneCondEntity::Mastoid),
            Some(200.0)
        );
    }

    #[test]
    fn test_forehead_resonance_frequency() {
        assert_eq!(
            SkullResonanceFrequency.get(&BoneCondEntity::Forehead),
            Some(800.0)
        );
    }

    #[test]
    fn test_mastoid_attenuation() {
        assert_eq!(
            TranscranialAttenuationDB.get(&BoneCondEntity::Mastoid),
            Some(10.0)
        );
    }

    #[test]
    fn test_baha_requires_surgery() {
        assert_eq!(
            RequiresSurgery.get(&BoneCondEntity::BoneAnchoredDevice),
            Some(true)
        );
    }

    #[test]
    fn test_skin_drive_no_surgery() {
        assert_eq!(
            RequiresSurgery.get(&BoneCondEntity::SkinDriveTransducer),
            Some(false)
        );
    }

    #[test]
    fn test_entity_count() {
        assert_eq!(BoneCondEntity::variants().len(), 30);
    }

    #[test]
    fn test_ontology_validates() {
        BoneConductionOntology::validate().unwrap();
    }

    fn arb_bc_entity() -> impl Strategy<Value = BoneCondEntity> {
        (0..BoneCondEntity::variants().len()).prop_map(|i| BoneCondEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_taxonomy_reflexive(entity in arb_bc_entity()) {
            prop_assert!(taxonomy::is_a::<BoneCondTaxonomy>(&entity, &entity));
        }
    }
}
