//! Auditory anatomy ontology.
//!
//! Models the structural hierarchy of the human auditory system:
//! Ear -> Outer/Middle/Inner -> individual structures -> cell types.
//!
//! Taxonomy: structure-type hierarchy (e.g., Malleus is-a Ossicle).
//! Mereology: part-whole (e.g., MiddleEar has-a Malleus).
//!
//! Key references:
//! - Pickles 2012: An Introduction to the Physiology of Hearing
//! - Raphael & Altschuler 2003: Structure and innervation of the cochlea
//! - Dallos et al. 1996: The Cochlea (Springer Handbook)
//! - von Békésy 1960: Experiments in Hearing
//! - Hudspeth 2014: Integrating the active process of hair cells

use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::mereology;
use pr4xis::ontology::reasoning::taxonomy;
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

/// Every anatomical entity in the auditory system.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum AuditoryEntity {
    Pinna,
    EarCanal,
    TympanicMembrane,
    Malleus,
    Incus,
    Stapes,
    OvalWindow,
    RoundWindow,
    EustachianTube,
    TensorTympani,
    Stapedius,
    Cochlea,
    BasilarMembrane,
    OrganOfCorti,
    TectorialMembrane,
    ScalaVestibuli,
    ScalaMedia,
    ScalaTympani,
    Endolymph,
    Perilymph,
    StriVascularis,
    ReissnersMembrane,
    Vestibule,
    SemicircularCanals,
    InnerHairCell,
    OuterHairCell,
    SupportingCell,
    SpiralGanglionNeuron,
    AuditoryNerve,
    CochlearNucleus,
    SuperiorOlivaryComplex,
    InferiorColliculus,
    MedialGeniculateBody,
    AuditoryCortex,
    Ear,
    OuterEar,
    MiddleEar,
    InnerEar,
    Ossicle,
    HairCell,
    CochlearFluid,
    CochlearMembrane,
    AuditoryNucleus,
}

// ---------------------------------------------------------------------------
// Ontology (define_ontology! macro)
// ---------------------------------------------------------------------------

define_ontology! {
    /// Discrete category over auditory anatomy entities.
    pub AnatomyOntology for AnatomyCategory {
        entity: AuditoryEntity,
        relation: AuditoryRelation,

        taxonomy: AuditoryTaxonomy [
            (OuterEar, Ear), (MiddleEar, Ear), (InnerEar, Ear),
            (Pinna, OuterEar), (EarCanal, OuterEar), (TympanicMembrane, OuterEar),
            (Ossicle, MiddleEar), (Malleus, Ossicle), (Incus, Ossicle), (Stapes, Ossicle),
            (TensorTympani, MiddleEar), (Stapedius, MiddleEar),
            (OvalWindow, MiddleEar), (RoundWindow, MiddleEar), (EustachianTube, MiddleEar),
            (Cochlea, InnerEar), (Vestibule, InnerEar), (SemicircularCanals, InnerEar),
            (BasilarMembrane, CochlearMembrane), (TectorialMembrane, CochlearMembrane), (ReissnersMembrane, CochlearMembrane),
            (Endolymph, CochlearFluid), (Perilymph, CochlearFluid),
            (InnerHairCell, HairCell), (OuterHairCell, HairCell),
            (CochlearNucleus, AuditoryNucleus), (SuperiorOlivaryComplex, AuditoryNucleus),
            (InferiorColliculus, AuditoryNucleus), (MedialGeniculateBody, AuditoryNucleus),
        ],

        mereology: AuditoryMereology [
            (Ear, OuterEar), (Ear, MiddleEar), (Ear, InnerEar),
            (OuterEar, Pinna), (OuterEar, EarCanal), (OuterEar, TympanicMembrane),
            (MiddleEar, Malleus), (MiddleEar, Incus), (MiddleEar, Stapes),
            (MiddleEar, OvalWindow), (MiddleEar, RoundWindow), (MiddleEar, EustachianTube),
            (MiddleEar, TensorTympani), (MiddleEar, Stapedius),
            (InnerEar, Cochlea), (InnerEar, Vestibule), (InnerEar, SemicircularCanals),
            (Cochlea, BasilarMembrane), (Cochlea, OrganOfCorti), (Cochlea, TectorialMembrane),
            (Cochlea, ScalaVestibuli), (Cochlea, ScalaMedia), (Cochlea, ScalaTympani),
            (Cochlea, ReissnersMembrane), (Cochlea, StriVascularis),
            (ScalaVestibuli, Perilymph), (ScalaTympani, Perilymph), (ScalaMedia, Endolymph),
            (OrganOfCorti, InnerHairCell), (OrganOfCorti, OuterHairCell), (OrganOfCorti, SupportingCell),
            (Cochlea, SpiralGanglionNeuron),
        ],

        opposition: AnatomyOpposition [
            (OuterEar, InnerEar), (Endolymph, Perilymph), (InnerHairCell, OuterHairCell),
        ],
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnatomicalRegion {
    External,
    MiddleEarRegion,
    InnerEarRegion,
    Neural,
    Abstract,
}

#[derive(Debug, Clone)]
pub struct RegionQuality;
impl Quality for RegionQuality {
    type Individual = AuditoryEntity;
    type Value = AnatomicalRegion;
    fn get(&self, individual: &AuditoryEntity) -> Option<AnatomicalRegion> {
        use AnatomicalRegion::*;
        use AuditoryEntity::*;
        Some(match individual {
            Pinna | EarCanal | TympanicMembrane => External,
            Malleus | Incus | Stapes | OvalWindow | RoundWindow | EustachianTube
            | TensorTympani | Stapedius => MiddleEarRegion,
            Cochlea | BasilarMembrane | OrganOfCorti | TectorialMembrane | ScalaVestibuli
            | ScalaMedia | ScalaTympani | Endolymph | Perilymph | StriVascularis
            | ReissnersMembrane | Vestibule | SemicircularCanals | InnerHairCell
            | OuterHairCell | SupportingCell | SpiralGanglionNeuron => InnerEarRegion,
            AuditoryNerve
            | CochlearNucleus
            | SuperiorOlivaryComplex
            | InferiorColliculus
            | MedialGeniculateBody
            | AuditoryCortex => Neural,
            Ear | OuterEar | MiddleEar | InnerEar | Ossicle | HairCell | CochlearFluid
            | CochlearMembrane | AuditoryNucleus => Abstract,
        })
    }
}

#[derive(Debug, Clone)]
pub struct IsMechanicallyActive;
impl Quality for IsMechanicallyActive {
    type Individual = AuditoryEntity;
    type Value = bool;
    fn get(&self, individual: &AuditoryEntity) -> Option<bool> {
        use AuditoryEntity::*;
        Some(matches!(
            individual,
            TympanicMembrane
                | Malleus
                | Incus
                | Stapes
                | OvalWindow
                | RoundWindow
                | BasilarMembrane
                | TectorialMembrane
                | InnerHairCell
                | OuterHairCell
        ))
    }
}

#[derive(Debug, Clone)]
pub struct CharacteristicFrequency;
impl Quality for CharacteristicFrequency {
    type Individual = AuditoryEntity;
    type Value = f64;
    fn get(&self, individual: &AuditoryEntity) -> Option<f64> {
        use AuditoryEntity::*;
        match individual {
            Pinna => Some(2700.0),
            EarCanal => Some(3000.0),
            TympanicMembrane => Some(1000.0),
            Stapes => Some(1000.0),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

pub struct ThreeOssicles;
impl Axiom for ThreeOssicles {
    fn description(&self) -> &str {
        "malleus, incus, and stapes are ossicles"
    }
    fn holds(&self) -> bool {
        use AuditoryEntity::*;
        taxonomy::is_a::<AuditoryTaxonomy>(&Malleus, &Ossicle)
            && taxonomy::is_a::<AuditoryTaxonomy>(&Incus, &Ossicle)
            && taxonomy::is_a::<AuditoryTaxonomy>(&Stapes, &Ossicle)
    }
}

pub struct CochleaContainsHairCells;
impl Axiom for CochleaContainsHairCells {
    fn description(&self) -> &str {
        "cochlea transitively contains both inner and outer hair cells"
    }
    fn holds(&self) -> bool {
        use AuditoryEntity::*;
        let parts = mereology::parts_of::<AuditoryMereology>(&Cochlea);
        parts.contains(&InnerHairCell) && parts.contains(&OuterHairCell)
    }
}

pub struct EarContainsHairCells;
impl Axiom for EarContainsHairCells {
    fn description(&self) -> &str {
        "ear transitively contains inner and outer hair cells"
    }
    fn holds(&self) -> bool {
        use AuditoryEntity::*;
        let parts = mereology::parts_of::<AuditoryMereology>(&Ear);
        parts.contains(&InnerHairCell) && parts.contains(&OuterHairCell)
    }
}

pub struct CochleaHasThreeScalae;
impl Axiom for CochleaHasThreeScalae {
    fn description(&self) -> &str {
        "cochlea contains scala vestibuli, scala media, and scala tympani"
    }
    fn holds(&self) -> bool {
        use AuditoryEntity::*;
        let parts = mereology::parts_of::<AuditoryMereology>(&Cochlea);
        parts.contains(&ScalaVestibuli)
            && parts.contains(&ScalaMedia)
            && parts.contains(&ScalaTympani)
    }
}

pub struct AllRegionsRepresented;
impl Axiom for AllRegionsRepresented {
    fn description(&self) -> &str {
        "all four non-abstract anatomical regions are represented"
    }
    fn holds(&self) -> bool {
        use AnatomicalRegion::*;
        let quality = RegionQuality;
        let all = AuditoryEntity::variants();
        let regions: Vec<AnatomicalRegion> = all.iter().filter_map(|e| quality.get(e)).collect();
        [External, MiddleEarRegion, InnerEarRegion, Neural]
            .iter()
            .all(|target| regions.contains(target))
    }
}

pub struct HairCellsAreMechanicallyActive;
impl Axiom for HairCellsAreMechanicallyActive {
    fn description(&self) -> &str {
        "both inner and outer hair cells are mechanically active"
    }
    fn holds(&self) -> bool {
        use AuditoryEntity::*;
        IsMechanicallyActive.get(&InnerHairCell) == Some(true)
            && IsMechanicallyActive.get(&OuterHairCell) == Some(true)
    }
}

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for AnatomyOntology {
    type Cat = AnatomyCategory;
    type Qual = RegionQuality;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(ThreeOssicles),
            Box::new(CochleaContainsHairCells),
            Box::new(EarContainsHairCells),
            Box::new(CochleaHasThreeScalae),
            Box::new(AllRegionsRepresented),
            Box::new(HairCellsAreMechanicallyActive),
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
    use pr4xis::ontology::reasoning::mereology::MereologyCategory;
    use pr4xis::ontology::reasoning::opposition;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;

    #[test]
    fn test_three_ossicles() {
        assert!(ThreeOssicles.holds());
    }
    #[test]
    fn test_cochlea_contains_hair_cells() {
        assert!(CochleaContainsHairCells.holds());
    }
    #[test]
    fn test_ear_contains_hair_cells() {
        assert!(EarContainsHairCells.holds());
    }
    #[test]
    fn test_cochlea_has_three_scalae() {
        assert!(CochleaHasThreeScalae.holds());
    }
    #[test]
    fn test_all_regions_represented() {
        assert!(AllRegionsRepresented.holds());
    }
    #[test]
    fn test_hair_cells_mechanically_active() {
        assert!(HairCellsAreMechanicallyActive.holds());
    }

    #[test]
    fn test_outer_ear_opposes_inner_ear() {
        assert!(opposition::are_opposed::<AnatomyOpposition>(
            &AuditoryEntity::OuterEar,
            &AuditoryEntity::InnerEar
        ));
    }

    #[test]
    fn test_anatomy_category_laws() {
        check_category_laws::<AnatomyCategory>().unwrap();
    }
    #[test]
    fn test_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<AuditoryTaxonomy>>().unwrap();
    }
    #[test]
    fn test_mereology_category_laws() {
        check_category_laws::<MereologyCategory<AuditoryMereology>>().unwrap();
    }

    #[test]
    fn test_malleus_is_ossicle() {
        assert!(taxonomy::is_a::<AuditoryTaxonomy>(
            &AuditoryEntity::Malleus,
            &AuditoryEntity::Ossicle
        ));
    }
    #[test]
    fn test_inner_hair_cell_is_hair_cell() {
        assert!(taxonomy::is_a::<AuditoryTaxonomy>(
            &AuditoryEntity::InnerHairCell,
            &AuditoryEntity::HairCell
        ));
    }
    #[test]
    fn test_cochlea_is_inner_ear() {
        assert!(taxonomy::is_a::<AuditoryTaxonomy>(
            &AuditoryEntity::Cochlea,
            &AuditoryEntity::InnerEar
        ));
    }
    #[test]
    fn test_endolymph_is_cochlear_fluid() {
        assert!(taxonomy::is_a::<AuditoryTaxonomy>(
            &AuditoryEntity::Endolymph,
            &AuditoryEntity::CochlearFluid
        ));
    }

    #[test]
    fn test_ear_contains_cochlea_transitively() {
        let parts = mereology::parts_of::<AuditoryMereology>(&AuditoryEntity::Ear);
        assert!(parts.contains(&AuditoryEntity::Cochlea));
    }
    #[test]
    fn test_cochlea_contains_basilar_membrane() {
        let parts = mereology::parts_of::<AuditoryMereology>(&AuditoryEntity::Cochlea);
        assert!(parts.contains(&AuditoryEntity::BasilarMembrane));
    }
    #[test]
    fn test_cochlea_contains_organ_of_corti() {
        let parts = mereology::parts_of::<AuditoryMereology>(&AuditoryEntity::Cochlea);
        assert!(parts.contains(&AuditoryEntity::OrganOfCorti));
    }
    #[test]
    fn test_organ_of_corti_contains_ihc() {
        let parts = mereology::parts_of::<AuditoryMereology>(&AuditoryEntity::OrganOfCorti);
        assert!(parts.contains(&AuditoryEntity::InnerHairCell));
    }
    #[test]
    fn test_middle_ear_contains_stapes() {
        let parts = mereology::parts_of::<AuditoryMereology>(&AuditoryEntity::MiddleEar);
        assert!(parts.contains(&AuditoryEntity::Stapes));
    }

    #[test]
    fn test_pinna_is_external() {
        assert_eq!(
            RegionQuality.get(&AuditoryEntity::Pinna),
            Some(AnatomicalRegion::External)
        );
    }
    #[test]
    fn test_stapes_is_middle_ear_region() {
        assert_eq!(
            RegionQuality.get(&AuditoryEntity::Stapes),
            Some(AnatomicalRegion::MiddleEarRegion)
        );
    }
    #[test]
    fn test_cochlea_is_inner_ear_region() {
        assert_eq!(
            RegionQuality.get(&AuditoryEntity::Cochlea),
            Some(AnatomicalRegion::InnerEarRegion)
        );
    }
    #[test]
    fn test_auditory_cortex_is_neural() {
        assert_eq!(
            RegionQuality.get(&AuditoryEntity::AuditoryCortex),
            Some(AnatomicalRegion::Neural)
        );
    }
    #[test]
    fn test_tympanic_membrane_is_mechanically_active() {
        assert_eq!(
            IsMechanicallyActive.get(&AuditoryEntity::TympanicMembrane),
            Some(true)
        );
    }
    #[test]
    fn test_eustachian_tube_is_not_mechanically_active() {
        assert_eq!(
            IsMechanicallyActive.get(&AuditoryEntity::EustachianTube),
            Some(false)
        );
    }
    #[test]
    fn test_ear_canal_resonance() {
        assert_eq!(
            CharacteristicFrequency.get(&AuditoryEntity::EarCanal),
            Some(3000.0)
        );
    }
    #[test]
    fn test_entity_count() {
        assert_eq!(AuditoryEntity::variants().len(), 43);
    }
    #[test]
    fn test_ontology_validates() {
        AnatomyOntology::validate().unwrap();
    }

    fn arb_auditory_entity() -> impl Strategy<Value = AuditoryEntity> {
        (0..AuditoryEntity::variants().len()).prop_map(|i| AuditoryEntity::variants()[i])
    }
    proptest! {
        #[test] fn prop_taxonomy_reflexive(entity in arb_auditory_entity()) { prop_assert!(taxonomy::is_a::<AuditoryTaxonomy>(&entity, &entity)); }
        #[test] fn prop_region_is_total(entity in arb_auditory_entity()) { prop_assert!(RegionQuality.get(&entity).is_some()); }
    }
}
