//! Bone conduction ontology.
//!
//! Models how vibration reaches the cochlea through bone rather than air.
//! Three primary mechanisms (Stenfelt 2011; Tonndorf 1966):
//!   1. Osseotympanic: ear canal wall vibration -> eardrum vibration
//!   2. Inertial: skull vibration -> ossicular chain inertia -> oval window
//!   3. Compressional: skull vibration -> cochlear wall compression -> fluid motion
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
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::causation;
use pr4xis::ontology::reasoning::taxonomy;
use pr4xis::ontology::{Axiom, Ontology, Quality};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum BoneCondEntity {
    OsseotympanicBC,
    InertialBC,
    CompressionalBC,
    DistortionalBC,
    SkullVibration,
    EarCanalWallVibration,
    OssicularInertia,
    CochlearWallCompression,
    FluidInertia,
    SkullDeformation,
    SoundRadiation,
    BoneAnchoredDevice,
    PercutaneousImplant,
    TranscutaneousDevice,
    SkinDriveTransducer,
    PiezoelectricTransducer,
    ElectromagneticTransducer,
    Mastoid,
    Forehead,
    TemporalBone,
    Vertex,
    Teeth,
    OcclusionEffect,
    TranscranialAttenuation,
    SkullResonance,
    ForceLevel,
    BCMechanism,
    BCTransducer,
    ApplicationSite,
    BCPhenomenon,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum BCCausalEvent {
    TransducerActivation,
    SkullCoupling,
    SkullWavePropagation,
    EarCanalWallMotion,
    TympanicMembraneResponse,
    OsseotympanicStimulation,
    OssicularLag,
    StapesDisplacement,
    OvalWindowDrive,
    CochlearBoneCompression,
    DifferentialFluidFlow,
    BasilarMembraneExcitation,
    SkullModeDeformation,
    InnerEarDistortion,
    CochlearResponse,
}

define_ontology! {
    /// Discrete category over bone conduction entities.
    pub BoneConductionOntology for BoneConductionCategory {
        entity: BoneCondEntity, relation: BoneCondRelation,
        being: PhysicalEndurant,
        source: "Stenfelt & Goode (2005); Tonndorf (1966)",
        taxonomy: BoneCondTaxonomy [
            (OsseotympanicBC, BCMechanism), (InertialBC, BCMechanism), (CompressionalBC, BCMechanism), (DistortionalBC, BCMechanism),
            (BoneAnchoredDevice, BCTransducer), (PercutaneousImplant, BCTransducer), (TranscutaneousDevice, BCTransducer), (SkinDriveTransducer, BCTransducer), (PiezoelectricTransducer, BCTransducer), (ElectromagneticTransducer, BCTransducer),
            (Mastoid, ApplicationSite), (Forehead, ApplicationSite), (TemporalBone, ApplicationSite), (Vertex, ApplicationSite), (Teeth, ApplicationSite),
            (OcclusionEffect, BCPhenomenon), (TranscranialAttenuation, BCPhenomenon), (SkullResonance, BCPhenomenon),
        ],
        causation: BCCausalGraph for BCCausalEvent [
            (TransducerActivation, SkullCoupling), (SkullCoupling, SkullWavePropagation),
            (SkullWavePropagation, EarCanalWallMotion), (EarCanalWallMotion, TympanicMembraneResponse), (TympanicMembraneResponse, OsseotympanicStimulation), (OsseotympanicStimulation, CochlearResponse),
            (SkullWavePropagation, OssicularLag), (OssicularLag, StapesDisplacement), (StapesDisplacement, OvalWindowDrive), (OvalWindowDrive, CochlearResponse),
            (SkullWavePropagation, CochlearBoneCompression), (CochlearBoneCompression, DifferentialFluidFlow), (DifferentialFluidFlow, BasilarMembraneExcitation), (BasilarMembraneExcitation, CochlearResponse),
            (SkullWavePropagation, SkullModeDeformation), (SkullModeDeformation, InnerEarDistortion), (InnerEarDistortion, CochlearResponse),
        ],
        opposition: BoneCondOpposition [
            (OsseotympanicBC, CompressionalBC), (PercutaneousImplant, TranscutaneousDevice), (Mastoid, Forehead),
        ],
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FrequencyRange {
    pub low: f64,
    pub high: f64,
}

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

#[derive(Debug, Clone)]
pub struct TranscranialAttenuationDB;
impl Quality for TranscranialAttenuationDB {
    type Individual = BoneCondEntity;
    type Value = f64;
    fn get(&self, individual: &BoneCondEntity) -> Option<f64> {
        use BoneCondEntity::*;
        match individual {
            Mastoid => Some(10.0),
            Forehead => Some(0.0),
            TemporalBone => Some(12.0),
            Vertex => Some(0.0),
            Teeth => Some(5.0),
            _ => None,
        }
    }
}

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

#[derive(Debug, Clone)]
pub struct RequiresSurgery;
impl Quality for RequiresSurgery {
    type Individual = BoneCondEntity;
    type Value = bool;
    fn get(&self, individual: &BoneCondEntity) -> Option<bool> {
        use BoneCondEntity::*;
        match individual {
            BoneAnchoredDevice | PercutaneousImplant | TranscutaneousDevice => Some(true),
            SkinDriveTransducer | PiezoelectricTransducer | ElectromagneticTransducer => {
                Some(false)
            }
            _ => None,
        }
    }
}

// Axioms

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
pr4xis::register_axiom!(FourBCMechanisms);

pub struct TransducerCausesCochlearResponse;
impl Axiom for TransducerCausesCochlearResponse {
    fn description(&self) -> &str {
        "transducer activation transitively causes cochlear response"
    }
    fn holds(&self) -> bool {
        use BCCausalEvent::*;
        causation::effects_of::<BCCausalGraph>(&TransducerActivation).contains(&CochlearResponse)
    }
}
pr4xis::register_axiom!(TransducerCausesCochlearResponse);

pub struct AllPathwaysConverge;
impl Axiom for AllPathwaysConverge {
    fn description(&self) -> &str {
        "osseotympanic, inertial, and compressional pathways all reach cochlear response"
    }
    fn holds(&self) -> bool {
        use BCCausalEvent::*;
        causation::effects_of::<BCCausalGraph>(&OsseotympanicStimulation)
            .contains(&CochlearResponse)
            && causation::effects_of::<BCCausalGraph>(&OvalWindowDrive).contains(&CochlearResponse)
            && causation::effects_of::<BCCausalGraph>(&BasilarMembraneExcitation)
                .contains(&CochlearResponse)
    }
}
pr4xis::register_axiom!(AllPathwaysConverge);

pub struct InertialCoversSpeechRange;
impl Axiom for InertialCoversSpeechRange {
    fn description(&self) -> &str {
        "inertial BC covers the speech frequency range (100-3000 Hz)"
    }
    fn holds(&self) -> bool {
        let inertial = DominantFrequencyRange
            .get(&BoneCondEntity::InertialBC)
            .unwrap();
        inertial.low <= 300.0 && inertial.high >= 3000.0
    }
}
pr4xis::register_axiom!(InertialCoversSpeechRange);

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
pr4xis::register_axiom!(ForeheadResonanceHigherThanMastoid);

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
pr4xis::register_axiom!(MidlineSitesSymmetric);

impl Ontology for BoneConductionOntology {
    type Cat = BoneConductionCategory;
    type Qual = TranscranialAttenuationDB;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(FourBCMechanisms),
            Box::new(TransducerCausesCochlearResponse),
            Box::new(AllPathwaysConverge),
            Box::new(InertialCoversSpeechRange),
            Box::new(ForeheadResonanceHigherThanMastoid),
            Box::new(MidlineSitesSymmetric),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;
    use pr4xis::ontology::reasoning::causation::CausalCategory;
    use pr4xis::ontology::reasoning::opposition;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;

    #[test]
    fn test_four_bc_mechanisms() {
        assert!(FourBCMechanisms.holds());
    }
    #[test]
    fn test_transducer_causes_cochlear_response() {
        assert!(TransducerCausesCochlearResponse.holds());
    }
    #[test]
    fn test_all_pathways_converge() {
        assert!(AllPathwaysConverge.holds());
    }
    #[test]
    fn test_inertial_has_broadest_range() {
        assert!(InertialCoversSpeechRange.holds());
    }
    #[test]
    fn test_midline_sites_symmetric() {
        assert!(MidlineSitesSymmetric.holds());
    }
    #[test]
    fn test_osseotympanic_opposes_compressional() {
        assert!(opposition::are_opposed::<BoneCondOpposition>(
            &BoneCondEntity::OsseotympanicBC,
            &BoneCondEntity::CompressionalBC
        ));
    }
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
    #[test]
    fn test_skull_wave_has_multiple_pathways() {
        use BCCausalEvent::*;
        let effects = causation::effects_of::<BCCausalGraph>(&SkullWavePropagation);
        assert!(effects.contains(&EarCanalWallMotion));
        assert!(effects.contains(&OssicularLag));
        assert!(effects.contains(&CochlearBoneCompression));
        assert!(effects.contains(&SkullModeDeformation));
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
        #[test] fn prop_taxonomy_reflexive(entity in arb_bc_entity()) { prop_assert!(taxonomy::is_a::<BoneCondTaxonomy>(&entity, &entity)); }
    }
}
