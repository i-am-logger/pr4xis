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
//! - Fernández & Goldberg 1971: vestibular afferent physiology
//! - Hudspeth & Corey 1977: hair cell transduction in bullfrog sacculus

use pr4xis::category::{Category, Entity, Relationship};
use pr4xis::ontology::reasoning::causation::{self, CausalDef};
use pr4xis::ontology::reasoning::mereology::{self, MereologyDef};
use pr4xis::ontology::reasoning::opposition::{self, OppositionDef};
use pr4xis::ontology::reasoning::taxonomy::{self, TaxonomyDef};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VestibularEntity {
    // Semicircular canals
    LateralCanal,
    AnteriorCanal,
    PosteriorCanal,
    Ampulla,
    Cupula,
    CrisaAmpullaris,
    // Otolith organs
    Utricle,
    Saccule,
    Macula,
    Otoconia,
    OtolithMembrane,
    StriolarRegion,
    ExtrastriolarRegion,
    // Hair cells
    TypeIHairCell,
    TypeIIHairCell,
    CalyxEnding,
    BoutonEnding,
    // Neural
    VestibularNerve,
    ScarpaGanglion,
    VestibularNuclei,
    MedialVestibularNucleus,
    LateralVestibularNucleus,
    SuperiorVestibularNucleus,
    CerebellumVestibular,
    // Reflexes
    VestibuloOcularReflex,
    VestibuloSpinalReflex,
    VestibuloColicReflex,
    // Stimuli
    AngularAcceleration,
    LinearAcceleration,
    GravityVector,
    HeadTilt,
    // Disorders
    BPPV,
    VestibularNeuritis,
    Vertigo,
    // Abstract categories
    SemicircularCanal,
    OtolithOrgan,
    VestibularHairCell,
    VestibularReflex,
    VestibularStimulus,
    VestibularDisorder,
}

impl Entity for VestibularEntity {
    fn variants() -> Vec<Self> {
        use VestibularEntity::*;
        vec![
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
        ]
    }
}

// ---------------------------------------------------------------------------
// Taxonomy
// ---------------------------------------------------------------------------

pub struct VestibularTaxonomy;

impl TaxonomyDef for VestibularTaxonomy {
    type Entity = VestibularEntity;

    fn relations() -> Vec<(VestibularEntity, VestibularEntity)> {
        use VestibularEntity::*;
        vec![
            // Canals
            (LateralCanal, SemicircularCanal),
            (AnteriorCanal, SemicircularCanal),
            (PosteriorCanal, SemicircularCanal),
            // Otolith organs
            (Utricle, OtolithOrgan),
            (Saccule, OtolithOrgan),
            // Hair cells
            (TypeIHairCell, VestibularHairCell),
            (TypeIIHairCell, VestibularHairCell),
            // Reflexes
            (VestibuloOcularReflex, VestibularReflex),
            (VestibuloSpinalReflex, VestibularReflex),
            (VestibuloColicReflex, VestibularReflex),
            // Stimuli
            (AngularAcceleration, VestibularStimulus),
            (LinearAcceleration, VestibularStimulus),
            (GravityVector, VestibularStimulus),
            (HeadTilt, VestibularStimulus),
            // Disorders
            (BPPV, VestibularDisorder),
            (VestibularNeuritis, VestibularDisorder),
            (Vertigo, VestibularDisorder),
        ]
    }
}

// ---------------------------------------------------------------------------
// Mereology
// ---------------------------------------------------------------------------

pub struct VestibularMereology;

impl MereologyDef for VestibularMereology {
    type Entity = VestibularEntity;

    fn relations() -> Vec<(VestibularEntity, VestibularEntity)> {
        use VestibularEntity::*;
        vec![
            // Each canal has an ampulla
            (LateralCanal, Ampulla),
            (AnteriorCanal, Ampulla),
            (PosteriorCanal, Ampulla),
            // Ampulla contains crista and cupula
            (Ampulla, CrisaAmpullaris),
            (Ampulla, Cupula),
            // Crista contains hair cells
            (CrisaAmpullaris, TypeIHairCell),
            (CrisaAmpullaris, TypeIIHairCell),
            // Otolith organs contain macula
            (Utricle, Macula),
            (Saccule, Macula),
            // Macula contains otoconia and hair cells
            (Macula, Otoconia),
            (Macula, OtolithMembrane),
            (Macula, TypeIHairCell),
            (Macula, TypeIIHairCell),
            (Macula, StriolarRegion),
            (Macula, ExtrastriolarRegion),
        ]
    }
}

// ---------------------------------------------------------------------------
// Causal graph
// ---------------------------------------------------------------------------

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

impl Entity for VestibularCausalEvent {
    fn variants() -> Vec<Self> {
        use VestibularCausalEvent::*;
        vec![
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
        ]
    }
}

pub struct VestibularCausalGraph;

impl CausalDef for VestibularCausalGraph {
    type Entity = VestibularCausalEvent;

    fn relations() -> Vec<(VestibularCausalEvent, VestibularCausalEvent)> {
        use VestibularCausalEvent::*;
        vec![
            // Canal pathway: rotation → endolymph → cupula → hair cell
            (HeadRotation, EndolymphFlow),
            (EndolymphFlow, CupulaDeflection),
            (CupulaDeflection, CanalHairCellActivation),
            // Otolith pathway: linear motion → otoconia → macula hair cell
            (HeadLinearMotion, OtoconiaShear),
            (OtoconiaShear, MaculaHairCellActivation),
            // Both converge on vestibular nerve
            (CanalHairCellActivation, VestibularAfferentFiring),
            (MaculaHairCellActivation, VestibularAfferentFiring),
            // Central processing
            (VestibularAfferentFiring, VestibularNucleiProcessing),
            // Reflex outputs
            (VestibularNucleiProcessing, VORActivation),
            (VORActivation, EyeMovementCompensation),
            (VestibularNucleiProcessing, PosturalAdjustment),
        ]
    }
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VestibularRelation {
    pub from: VestibularEntity,
    pub to: VestibularEntity,
}

impl Relationship for VestibularRelation {
    type Object = VestibularEntity;
    fn source(&self) -> VestibularEntity {
        self.from
    }
    fn target(&self) -> VestibularEntity {
        self.to
    }
}

pub struct VestibularCategory;

impl Category for VestibularCategory {
    type Object = VestibularEntity;
    type Morphism = VestibularRelation;

    fn identity(obj: &VestibularEntity) -> VestibularRelation {
        VestibularRelation {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &VestibularRelation, g: &VestibularRelation) -> Option<VestibularRelation> {
        if f.to != g.from {
            return None;
        }
        Some(VestibularRelation {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<VestibularRelation> {
        let v = VestibularEntity::variants();
        v.iter()
            .flat_map(|&a| {
                v.iter()
                    .map(move |&b| VestibularRelation { from: a, to: b })
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Time constant (seconds) of the canal/cupula system.
///
/// Mechanical time constant: ~5-7 seconds (Rabbitt et al. 2004)
/// Neural time constant (velocity storage): ~15-20 seconds
#[derive(Debug, Clone)]
pub struct TimeConstant;

impl Quality for TimeConstant {
    type Individual = VestibularEntity;
    type Value = f64;

    fn get(&self, individual: &VestibularEntity) -> Option<f64> {
        use VestibularEntity::*;
        match individual {
            Cupula => Some(6.0), // seconds, mechanical
            LateralCanal => Some(6.0),
            VestibularNuclei => Some(17.0), // seconds, velocity storage
            _ => None,
        }
    }
}

/// VOR gain (eye velocity / head velocity). Ideally 1.0.
///
/// Leigh & Zee 2015: The Neurology of Eye Movements.
#[derive(Debug, Clone)]
pub struct VORGain;

impl Quality for VORGain {
    type Individual = VestibularEntity;
    type Value = f64;

    fn get(&self, individual: &VestibularEntity) -> Option<f64> {
        use VestibularEntity::*;
        match individual {
            VestibuloOcularReflex => Some(1.0), // ideal gain
            _ => None,
        }
    }
}

/// The plane of rotation to which each semicircular canal is most sensitive.
///
/// - LateralCanal: horizontal plane (yaw)
/// - AnteriorCanal: sagittal plane (pitch)
/// - PosteriorCanal: coronal plane (roll)
///
/// Goldberg et al. 2012, Ch. 2.
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

// ---------------------------------------------------------------------------
// Opposition
// ---------------------------------------------------------------------------

/// Opposition pairs in the vestibular system.
///
/// - AngularAcceleration vs LinearAcceleration: rotational vs translational
/// - TypeIHairCell vs TypeIIHairCell: Type I (calyx) vs Type II (bouton)
/// - VestibuloOcularReflex vs VestibuloSpinalReflex: eye vs posture compensation
pub struct VestibularOpposition;

impl OppositionDef for VestibularOpposition {
    type Entity = VestibularEntity;

    fn pairs() -> Vec<(VestibularEntity, VestibularEntity)> {
        use VestibularEntity::*;
        vec![
            (AngularAcceleration, LinearAcceleration),
            (TypeIHairCell, TypeIIHairCell),
            (VestibuloOcularReflex, VestibuloSpinalReflex),
        ]
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

pub struct VestibularTaxonomyIsDAG;
impl Axiom for VestibularTaxonomyIsDAG {
    fn description(&self) -> &str {
        "vestibular taxonomy is a DAG"
    }
    fn holds(&self) -> bool {
        taxonomy::NoCycles::<VestibularTaxonomy>::new().holds()
    }
}

pub struct VestibularMereologyIsDAG;
impl Axiom for VestibularMereologyIsDAG {
    fn description(&self) -> &str {
        "vestibular mereology is a DAG"
    }
    fn holds(&self) -> bool {
        mereology::NoCycles::<VestibularMereology>::new().holds()
    }
}

pub struct VestibularCausalIsAsymmetric;
impl Axiom for VestibularCausalIsAsymmetric {
    fn description(&self) -> &str {
        "vestibular causal graph is asymmetric"
    }
    fn holds(&self) -> bool {
        causation::Asymmetric::<VestibularCausalGraph>::new().holds()
    }
}

/// Three semicircular canals are classified.
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

/// Two otolith organs are classified.
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

/// Head rotation transitively causes eye movement compensation (VOR).
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

/// Canals contain hair cells (via ampulla → crista).
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

/// Opposition is symmetric.
pub struct VestibularOppositionSymmetric;
impl Axiom for VestibularOppositionSymmetric {
    fn description(&self) -> &str {
        "vestibular opposition is symmetric"
    }
    fn holds(&self) -> bool {
        opposition::Symmetric::<VestibularOpposition>::new().holds()
    }
}

/// Opposition is irreflexive.
pub struct VestibularOppositionIrreflexive;
impl Axiom for VestibularOppositionIrreflexive {
    fn description(&self) -> &str {
        "vestibular opposition is irreflexive"
    }
    fn holds(&self) -> bool {
        opposition::Irreflexive::<VestibularOpposition>::new().holds()
    }
}

/// Each canal has a distinct sensitivity plane (Goldberg et al. 2012).
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

/// VOR gain is unity.
pub struct VORGainIsUnity;
impl Axiom for VORGainIsUnity {
    fn description(&self) -> &str {
        "ideal VOR gain is 1.0"
    }
    fn holds(&self) -> bool {
        VORGain.get(&VestibularEntity::VestibuloOcularReflex) == Some(1.0)
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

pub struct VestibularOntology;

impl Ontology for VestibularOntology {
    type Cat = VestibularCategory;
    type Qual = TimeConstant;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(VestibularTaxonomyIsDAG),
            Box::new(VestibularMereologyIsDAG),
            Box::new(VestibularCausalIsAsymmetric),
            Box::new(ThreeCanals),
            Box::new(TwoOtolithOrgans),
            Box::new(RotationCausesVOR),
            Box::new(CanalsContainHairCells),
            Box::new(ThreeDistinctCanalPlanes),
            Box::new(VORGainIsUnity),
            Box::new(VestibularOppositionSymmetric),
            Box::new(VestibularOppositionIrreflexive),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;
    use pr4xis::ontology::reasoning::causation::CausalCategory;
    use pr4xis::ontology::reasoning::mereology::MereologyCategory;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;

    #[test]
    fn test_taxonomy_dag() {
        assert!(VestibularTaxonomyIsDAG.holds());
    }
    #[test]
    fn test_mereology_dag() {
        assert!(VestibularMereologyIsDAG.holds());
    }
    #[test]
    fn test_causal_asymmetric() {
        assert!(VestibularCausalIsAsymmetric.holds());
    }
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
    fn test_opposition_symmetric() {
        assert!(VestibularOppositionSymmetric.holds());
    }
    #[test]
    fn test_opposition_irreflexive() {
        assert!(VestibularOppositionIrreflexive.holds());
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
    proptest! {
        #[test]
        fn prop_taxonomy_reflexive(entity in arb_entity()) {
            prop_assert!(taxonomy::is_a::<VestibularTaxonomy>(&entity, &entity));
        }
    }
}
