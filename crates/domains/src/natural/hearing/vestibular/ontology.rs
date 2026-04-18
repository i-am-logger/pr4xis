//! Vestibular system ontology.
//!
//! Models the balance and spatial orientation system that shares
//! the inner ear with the cochlea. Uses the same hair cell transduction
//! machinery but detects head rotation and linear acceleration.
//!
//! Key references:
//! - Goldberg et al. 2012: The Vestibular System
//! - Angelaki & Cullen 2008: multisensory integration
//! - Rabbitt et al. 2004: semicircular canal biomechanics
//! - Fernandez & Goldberg 1971: vestibular afferent physiology
//! - Hudspeth & Corey 1977: hair cell transduction in bullfrog sacculus

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::causation;
use pr4xis::ontology::reasoning::mereology;
use pr4xis::ontology::reasoning::taxonomy;
use pr4xis::ontology::{Axiom, Ontology, Quality};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Concept)]
pub enum VestibularEntity {
    LateralCanal,
    AnteriorCanal,
    PosteriorCanal,
    Ampulla,
    Cupula,
    CrisaAmpullaris,
    Utricle,
    Saccule,
    Macula,
    Otoconia,
    OtolithMembrane,
    StriolarRegion,
    ExtrastriolarRegion,
    TypeIHairCell,
    TypeIIHairCell,
    CalyxEnding,
    BoutonEnding,
    VestibularNerve,
    ScarpaGanglion,
    VestibularNuclei,
    MedialVestibularNucleus,
    LateralVestibularNucleus,
    SuperiorVestibularNucleus,
    CerebellumVestibular,
    VestibuloOcularReflex,
    VestibuloSpinalReflex,
    VestibuloColicReflex,
    AngularAcceleration,
    LinearAcceleration,
    GravityVector,
    HeadTilt,
    BPPV,
    VestibularNeuritis,
    Vertigo,
    SemicircularCanal,
    OtolithOrgan,
    VestibularHairCell,
    VestibularReflex,
    VestibularStimulus,
    VestibularDisorder,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Concept)]
pub enum VestibularCausalEvent {
    HeadRotation,
    EndolymphFlow,
    CupulaDeflection,
    CanalHairCellActivation,
    HeadLinearMotion,
    OtoconiaShear,
    MaculaHairCellActivation,
    VestibularAfferentFiring,
    VestibularNucleiProcessing,
    VORActivation,
    EyeMovementCompensation,
    PosturalAdjustment,
}
define_ontology! {
    /// Discrete category over vestibular entities.
    pub VestibularOntology for VestibularCategory {
        entity: VestibularEntity, relation: VestibularRelation,
        being: PhysicalEndurant,
        source: "Goldberg et al. (2012); Angelaki & Cullen (2008)",
        taxonomy: VestibularTaxonomy [
            (LateralCanal, SemicircularCanal), (AnteriorCanal, SemicircularCanal), (PosteriorCanal, SemicircularCanal),
            (Utricle, OtolithOrgan), (Saccule, OtolithOrgan),
            (TypeIHairCell, VestibularHairCell), (TypeIIHairCell, VestibularHairCell),
            (VestibuloOcularReflex, VestibularReflex), (VestibuloSpinalReflex, VestibularReflex), (VestibuloColicReflex, VestibularReflex),
            (AngularAcceleration, VestibularStimulus), (LinearAcceleration, VestibularStimulus), (GravityVector, VestibularStimulus), (HeadTilt, VestibularStimulus),
            (BPPV, VestibularDisorder), (VestibularNeuritis, VestibularDisorder), (Vertigo, VestibularDisorder),
        ],
        mereology: VestibularMereology [
            (LateralCanal, Ampulla), (AnteriorCanal, Ampulla), (PosteriorCanal, Ampulla),
            (Ampulla, CrisaAmpullaris), (Ampulla, Cupula),
            (CrisaAmpullaris, TypeIHairCell), (CrisaAmpullaris, TypeIIHairCell),
            (Utricle, Macula), (Saccule, Macula),
            (Macula, Otoconia), (Macula, OtolithMembrane), (Macula, TypeIHairCell), (Macula, TypeIIHairCell), (Macula, StriolarRegion), (Macula, ExtrastriolarRegion),
        ],
        causation: VestibularCausalGraph for VestibularCausalEvent [
            (HeadRotation, EndolymphFlow), (EndolymphFlow, CupulaDeflection), (CupulaDeflection, CanalHairCellActivation),
            (HeadLinearMotion, OtoconiaShear), (OtoconiaShear, MaculaHairCellActivation),
            (CanalHairCellActivation, VestibularAfferentFiring), (MaculaHairCellActivation, VestibularAfferentFiring),
            (VestibularAfferentFiring, VestibularNucleiProcessing), (VestibularNucleiProcessing, VORActivation), (VORActivation, EyeMovementCompensation), (VestibularNucleiProcessing, PosturalAdjustment),
        ],
        opposition: VestibularOpposition [ (AngularAcceleration, LinearAcceleration), (TypeIHairCell, TypeIIHairCell), (VestibuloOcularReflex, VestibuloSpinalReflex) ],
    }
}
#[derive(Debug, Clone)]
pub struct TimeConstant;
impl Quality for TimeConstant {
    type Individual = VestibularEntity;
    type Value = f64;
    fn get(&self, individual: &VestibularEntity) -> Option<f64> {
        use VestibularEntity::*;
        match individual {
            Cupula => Some(6.0),
            LateralCanal => Some(6.0),
            VestibularNuclei => Some(17.0),
            _ => None,
        }
    }
}
#[derive(Debug, Clone)]
pub struct VORGain;
impl Quality for VORGain {
    type Individual = VestibularEntity;
    type Value = f64;
    fn get(&self, individual: &VestibularEntity) -> Option<f64> {
        match individual {
            VestibularEntity::VestibuloOcularReflex => Some(1.0),
            _ => None,
        }
    }
}
#[derive(Debug, Clone)]
pub struct CanalSensitivity;
impl Quality for CanalSensitivity {
    type Individual = VestibularEntity;
    type Value = &'static str;
    fn get(&self, individual: &VestibularEntity) -> Option<&'static str> {
        use VestibularEntity::*;
        match individual {
            LateralCanal => Some("horizontal/yaw"),
            AnteriorCanal => Some("sagittal/pitch"),
            PosteriorCanal => Some("coronal/roll"),
            _ => None,
        }
    }
}

pub struct ThreeCanals;
impl Axiom for ThreeCanals {
    fn description(&self) -> &str {
        "three semicircular canals are classified"
    }
    fn holds(&self) -> bool {
        use VestibularEntity::*;
        [LateralCanal, AnteriorCanal, PosteriorCanal]
            .iter()
            .all(|c| taxonomy::is_a::<VestibularTaxonomy>(c, &SemicircularCanal))
    }
}
pr4xis::register_axiom!(ThreeCanals);
pub struct TwoOtolithOrgans;
impl Axiom for TwoOtolithOrgans {
    fn description(&self) -> &str {
        "utricle and saccule are otolith organs"
    }
    fn holds(&self) -> bool {
        use VestibularEntity::*;
        taxonomy::is_a::<VestibularTaxonomy>(&Utricle, &OtolithOrgan)
            && taxonomy::is_a::<VestibularTaxonomy>(&Saccule, &OtolithOrgan)
    }
}
pr4xis::register_axiom!(TwoOtolithOrgans);
pub struct RotationCausesVOR;
impl Axiom for RotationCausesVOR {
    fn description(&self) -> &str {
        "head rotation transitively causes eye movement compensation"
    }
    fn holds(&self) -> bool {
        use VestibularCausalEvent::*;
        causation::effects_of::<VestibularCausalGraph>(&HeadRotation)
            .contains(&EyeMovementCompensation)
    }
}
pr4xis::register_axiom!(RotationCausesVOR);
pub struct CanalsContainHairCells;
impl Axiom for CanalsContainHairCells {
    fn description(&self) -> &str {
        "semicircular canals transitively contain hair cells"
    }
    fn holds(&self) -> bool {
        use VestibularEntity::*;
        let parts = mereology::parts_of::<VestibularMereology>(&LateralCanal);
        parts.contains(&TypeIHairCell) && parts.contains(&TypeIIHairCell)
    }
}
pr4xis::register_axiom!(CanalsContainHairCells);
pub struct ThreeDistinctCanalPlanes;
impl Axiom for ThreeDistinctCanalPlanes {
    fn description(&self) -> &str {
        "each semicircular canal is sensitive to a distinct plane"
    }
    fn holds(&self) -> bool {
        use VestibularEntity::*;
        let lat = CanalSensitivity.get(&LateralCanal).unwrap();
        let ant = CanalSensitivity.get(&AnteriorCanal).unwrap();
        let post = CanalSensitivity.get(&PosteriorCanal).unwrap();
        lat != ant && ant != post && lat != post
    }
}
pr4xis::register_axiom!(ThreeDistinctCanalPlanes);
pub struct VORGainIsUnity;
impl Axiom for VORGainIsUnity {
    fn description(&self) -> &str {
        "ideal VOR gain is 1.0"
    }
    fn holds(&self) -> bool {
        VORGain.get(&VestibularEntity::VestibuloOcularReflex) == Some(1.0)
    }
}
pr4xis::register_axiom!(VORGainIsUnity);

impl Ontology for VestibularOntology {
    type Cat = VestibularCategory;
    type Qual = TimeConstant;
    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }
    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(ThreeCanals),
            Box::new(TwoOtolithOrgans),
            Box::new(RotationCausesVOR),
            Box::new(CanalsContainHairCells),
            Box::new(ThreeDistinctCanalPlanes),
            Box::new(VORGainIsUnity),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;
    use pr4xis::ontology::reasoning::causation::CausalCategory;
    use pr4xis::ontology::reasoning::mereology::MereologyCategory;
    use pr4xis::ontology::reasoning::opposition;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;
    #[test]
    fn test_three_canals() {
        assert!(ThreeCanals.holds());
    }
    #[test]
    fn test_two_otoliths() {
        assert!(TwoOtolithOrgans.holds());
    }
    #[test]
    fn test_rotation_causes_vor() {
        assert!(RotationCausesVOR.holds());
    }
    #[test]
    fn test_canals_contain_hair_cells() {
        assert!(CanalsContainHairCells.holds());
    }
    #[test]
    fn test_vor_gain_unity() {
        assert!(VORGainIsUnity.holds());
    }
    #[test]
    fn test_angular_opposes_linear_acceleration() {
        assert!(opposition::are_opposed::<VestibularOpposition>(
            &VestibularEntity::AngularAcceleration,
            &VestibularEntity::LinearAcceleration
        ));
    }
    #[test]
    fn test_category_laws() {
        check_category_laws::<VestibularCategory>().unwrap();
    }
    #[test]
    fn test_taxonomy_laws() {
        check_category_laws::<TaxonomyCategory<VestibularTaxonomy>>().unwrap();
    }
    #[test]
    fn test_mereology_laws() {
        check_category_laws::<MereologyCategory<VestibularMereology>>().unwrap();
    }
    #[test]
    fn test_causal_laws() {
        check_category_laws::<CausalCategory<VestibularCausalGraph>>().unwrap();
    }
    #[test]
    fn test_three_distinct_canal_planes() {
        assert!(ThreeDistinctCanalPlanes.holds());
    }
    #[test]
    fn test_lateral_canal_sensitivity() {
        assert_eq!(
            CanalSensitivity.get(&VestibularEntity::LateralCanal),
            Some("horizontal/yaw")
        );
    }
    #[test]
    fn test_posterior_canal_sensitivity() {
        assert_eq!(
            CanalSensitivity.get(&VestibularEntity::PosteriorCanal),
            Some("coronal/roll")
        );
    }
    #[test]
    fn test_cupula_time_constant() {
        assert_eq!(TimeConstant.get(&VestibularEntity::Cupula), Some(6.0));
    }
    #[test]
    fn test_entity_count() {
        assert_eq!(VestibularEntity::variants().len(), 40);
    }
    #[test]
    fn test_ontology_validates() {
        VestibularOntology::validate().unwrap();
    }
    fn arb_entity() -> impl Strategy<Value = VestibularEntity> {
        (0..VestibularEntity::variants().len()).prop_map(|i| VestibularEntity::variants()[i])
    }
    proptest! { #[test] fn prop_taxonomy_reflexive(entity in arb_entity()) { prop_assert!(taxonomy::is_a::<VestibularTaxonomy>(&entity, &entity)); } }
}
