//! Auditory anatomy ontology.
//!
//! Models the structural hierarchy of the human auditory system:
//! Ear → Outer/Middle/Inner → individual structures → cell types.
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

use praxis::category::{Category, Entity, Relationship};
use praxis::ontology::reasoning::mereology::{self, MereologyDef};
use praxis::ontology::reasoning::opposition::{self, OppositionDef};
use praxis::ontology::reasoning::taxonomy::{self, TaxonomyDef};
use praxis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

/// Every anatomical entity in the auditory system.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AuditoryEntity {
    // Outer ear
    Pinna,
    EarCanal,
    TympanicMembrane,
    // Middle ear
    Malleus,
    Incus,
    Stapes,
    OvalWindow,
    RoundWindow,
    EustachianTube,
    TensorTympani,
    Stapedius,
    // Inner ear — cochlea
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
    // Inner ear — vestibular
    Vestibule,
    SemicircularCanals,
    // Cell types
    InnerHairCell,
    OuterHairCell,
    SupportingCell,
    SpiralGanglionNeuron,
    // Neural pathway
    AuditoryNerve,
    CochlearNucleus,
    SuperiorOlivaryComplex,
    InferiorColliculus,
    MedialGeniculateBody,
    AuditoryCortex,
    // Abstract categories
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

impl Entity for AuditoryEntity {
    fn variants() -> Vec<Self> {
        use AuditoryEntity::*;
        vec![
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
        ]
    }
}

// ---------------------------------------------------------------------------
// Taxonomy (is-a)
// ---------------------------------------------------------------------------

/// Subsumption hierarchy for auditory anatomy.
pub struct AuditoryTaxonomy;

impl TaxonomyDef for AuditoryTaxonomy {
    type Entity = AuditoryEntity;

    fn relations() -> Vec<(AuditoryEntity, AuditoryEntity)> {
        use AuditoryEntity::*;
        vec![
            // Ear divisions
            (OuterEar, Ear),
            (MiddleEar, Ear),
            (InnerEar, Ear),
            // Outer ear structures
            (Pinna, OuterEar),
            (EarCanal, OuterEar),
            (TympanicMembrane, OuterEar),
            // Ossicles is-a MiddleEar component
            (Ossicle, MiddleEar),
            (Malleus, Ossicle),
            (Incus, Ossicle),
            (Stapes, Ossicle),
            // Middle ear muscles
            (TensorTympani, MiddleEar),
            (Stapedius, MiddleEar),
            // Windows
            (OvalWindow, MiddleEar),
            (RoundWindow, MiddleEar),
            (EustachianTube, MiddleEar),
            // Inner ear structures
            (Cochlea, InnerEar),
            (Vestibule, InnerEar),
            (SemicircularCanals, InnerEar),
            // Cochlear membranes
            (BasilarMembrane, CochlearMembrane),
            (TectorialMembrane, CochlearMembrane),
            (ReissnersMembrane, CochlearMembrane),
            // Cochlear fluids
            (Endolymph, CochlearFluid),
            (Perilymph, CochlearFluid),
            // Hair cells
            (InnerHairCell, HairCell),
            (OuterHairCell, HairCell),
            // Auditory nuclei
            (CochlearNucleus, AuditoryNucleus),
            (SuperiorOlivaryComplex, AuditoryNucleus),
            (InferiorColliculus, AuditoryNucleus),
            (MedialGeniculateBody, AuditoryNucleus),
        ]
    }
}

// ---------------------------------------------------------------------------
// Mereology (has-a / part-whole)
// ---------------------------------------------------------------------------

/// Part-whole relationships for auditory anatomy.
pub struct AuditoryMereology;

impl MereologyDef for AuditoryMereology {
    type Entity = AuditoryEntity;

    fn relations() -> Vec<(AuditoryEntity, AuditoryEntity)> {
        use AuditoryEntity::*;
        vec![
            // Ear has all divisions
            (Ear, OuterEar),
            (Ear, MiddleEar),
            (Ear, InnerEar),
            // Outer ear composition
            (OuterEar, Pinna),
            (OuterEar, EarCanal),
            (OuterEar, TympanicMembrane),
            // Middle ear composition
            (MiddleEar, Malleus),
            (MiddleEar, Incus),
            (MiddleEar, Stapes),
            (MiddleEar, OvalWindow),
            (MiddleEar, RoundWindow),
            (MiddleEar, EustachianTube),
            (MiddleEar, TensorTympani),
            (MiddleEar, Stapedius),
            // Inner ear composition
            (InnerEar, Cochlea),
            (InnerEar, Vestibule),
            (InnerEar, SemicircularCanals),
            // Cochlea composition
            (Cochlea, BasilarMembrane),
            (Cochlea, OrganOfCorti),
            (Cochlea, TectorialMembrane),
            (Cochlea, ScalaVestibuli),
            (Cochlea, ScalaMedia),
            (Cochlea, ScalaTympani),
            (Cochlea, ReissnersMembrane),
            (Cochlea, StriVascularis),
            // Scala fluids
            (ScalaVestibuli, Perilymph),
            (ScalaTympani, Perilymph),
            (ScalaMedia, Endolymph),
            // Organ of Corti composition
            (OrganOfCorti, InnerHairCell),
            (OrganOfCorti, OuterHairCell),
            (OrganOfCorti, SupportingCell),
            // Neural pathway (Cochlea → nerve → brain)
            (Cochlea, SpiralGanglionNeuron),
        ]
    }
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

/// A morphism in the anatomy category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditoryRelation {
    pub from: AuditoryEntity,
    pub to: AuditoryEntity,
}

impl Relationship for AuditoryRelation {
    type Object = AuditoryEntity;

    fn source(&self) -> AuditoryEntity {
        self.from
    }

    fn target(&self) -> AuditoryEntity {
        self.to
    }
}

/// Discrete category over auditory entities.
pub struct AnatomyCategory;

impl Category for AnatomyCategory {
    type Object = AuditoryEntity;
    type Morphism = AuditoryRelation;

    fn identity(obj: &AuditoryEntity) -> AuditoryRelation {
        AuditoryRelation {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &AuditoryRelation, g: &AuditoryRelation) -> Option<AuditoryRelation> {
        if f.to != g.from {
            return None;
        }
        Some(AuditoryRelation {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<AuditoryRelation> {
        let variants = AuditoryEntity::variants();
        let mut morphisms = Vec::new();
        for &from in &variants {
            for &to in &variants {
                morphisms.push(AuditoryRelation { from, to });
            }
        }
        morphisms
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Anatomical region classification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnatomicalRegion {
    External,
    MiddleEarRegion,
    InnerEarRegion,
    Neural,
    Abstract,
}

/// Quality: which anatomical region does this entity belong to?
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

/// Quality: is this structure mechanically vibrating during sound transduction?
///
/// Structures that physically move to transmit acoustic energy.
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

/// Quality: approximate characteristic frequency (Hz) at the structure.
///
/// Represents the tonotopic position or resonant frequency.
/// - Pinna resonance ~2700 Hz (Shaw 1974)
/// - Ear canal resonance ~3000 Hz (quarter-wave, ~2.7 cm length)
/// - Middle ear best transmission ~1000 Hz (Rosowski 1996)
#[derive(Debug, Clone)]
pub struct CharacteristicFrequency;

impl Quality for CharacteristicFrequency {
    type Individual = AuditoryEntity;
    type Value = f64;

    fn get(&self, individual: &AuditoryEntity) -> Option<f64> {
        use AuditoryEntity::*;
        match individual {
            Pinna => Some(2700.0),            // Shaw 1974
            EarCanal => Some(3000.0),         // quarter-wave resonance
            TympanicMembrane => Some(1000.0), // best transmission range
            Stapes => Some(1000.0),           // middle ear peak
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Opposition
// ---------------------------------------------------------------------------

/// Opposition pairs in auditory anatomy.
///
/// - OuterEar vs InnerEar: external sound collection vs internal transduction
/// - Endolymph vs Perilymph: high-K+ vs low-K+ cochlear fluid
/// - InnerHairCell vs OuterHairCell: sensory transduction vs cochlear amplifier
pub struct AnatomyOpposition;

impl OppositionDef for AnatomyOpposition {
    type Entity = AuditoryEntity;

    fn pairs() -> Vec<(AuditoryEntity, AuditoryEntity)> {
        use AuditoryEntity::*;
        vec![
            (OuterEar, InnerEar),
            (Endolymph, Perilymph),
            (InnerHairCell, OuterHairCell),
        ]
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Taxonomy is a DAG.
pub struct AnatomyTaxonomyIsDAG;

impl Axiom for AnatomyTaxonomyIsDAG {
    fn description(&self) -> &str {
        "auditory anatomy taxonomy is a directed acyclic graph"
    }

    fn holds(&self) -> bool {
        taxonomy::NoCycles::<AuditoryTaxonomy>::new().holds()
    }
}

/// Taxonomy is antisymmetric.
pub struct AnatomyTaxonomyIsAntisymmetric;

impl Axiom for AnatomyTaxonomyIsAntisymmetric {
    fn description(&self) -> &str {
        "auditory anatomy taxonomy is antisymmetric"
    }

    fn holds(&self) -> bool {
        taxonomy::Antisymmetric::<AuditoryTaxonomy>::new().holds()
    }
}

/// Mereology is a DAG.
pub struct AnatomyMereologyIsDAG;

impl Axiom for AnatomyMereologyIsDAG {
    fn description(&self) -> &str {
        "auditory anatomy mereology is a directed acyclic graph"
    }

    fn holds(&self) -> bool {
        mereology::NoCycles::<AuditoryMereology>::new().holds()
    }
}

/// Three ossicles form the ossicular chain.
///
/// Pickles 2012, Ch. 2.
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

/// Cochlea contains both inner and outer hair cells (via Organ of Corti).
///
/// Raphael & Altschuler 2003.
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

/// Ear transitively contains all hair cells.
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

/// Cochlea contains three scalae (fluid compartments).
///
/// Dallos et al. 1996; Pickles 2012.
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

/// All four anatomical regions are represented.
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

/// Opposition is symmetric.
pub struct AnatomyOppositionSymmetric;

impl Axiom for AnatomyOppositionSymmetric {
    fn description(&self) -> &str {
        "auditory anatomy opposition is symmetric"
    }

    fn holds(&self) -> bool {
        opposition::Symmetric::<AnatomyOpposition>::new().holds()
    }
}

/// Opposition is irreflexive.
pub struct AnatomyOppositionIrreflexive;

impl Axiom for AnatomyOppositionIrreflexive {
    fn description(&self) -> &str {
        "auditory anatomy opposition is irreflexive"
    }

    fn holds(&self) -> bool {
        opposition::Irreflexive::<AnatomyOpposition>::new().holds()
    }
}

/// Both IHC and OHC are mechanically active.
///
/// Hudspeth 2014.
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
// Ontology
// ---------------------------------------------------------------------------

/// Top-level auditory anatomy ontology.
pub struct AnatomyOntology;

impl Ontology for AnatomyOntology {
    type Cat = AnatomyCategory;
    type Qual = RegionQuality;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(AnatomyTaxonomyIsDAG),
            Box::new(AnatomyTaxonomyIsAntisymmetric),
            Box::new(AnatomyMereologyIsDAG),
            Box::new(ThreeOssicles),
            Box::new(CochleaContainsHairCells),
            Box::new(EarContainsHairCells),
            Box::new(CochleaHasThreeScalae),
            Box::new(AllRegionsRepresented),
            Box::new(HairCellsAreMechanicallyActive),
            Box::new(AnatomyOppositionSymmetric),
            Box::new(AnatomyOppositionIrreflexive),
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
    use praxis::ontology::reasoning::mereology::MereologyCategory;
    use praxis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;

    // -- Axiom tests --

    #[test]
    fn test_taxonomy_is_dag() {
        assert!(
            AnatomyTaxonomyIsDAG.holds(),
            "{}",
            AnatomyTaxonomyIsDAG.description()
        );
    }

    #[test]
    fn test_taxonomy_is_antisymmetric() {
        assert!(
            AnatomyTaxonomyIsAntisymmetric.holds(),
            "{}",
            AnatomyTaxonomyIsAntisymmetric.description()
        );
    }

    #[test]
    fn test_mereology_is_dag() {
        assert!(
            AnatomyMereologyIsDAG.holds(),
            "{}",
            AnatomyMereologyIsDAG.description()
        );
    }

    #[test]
    fn test_three_ossicles() {
        assert!(ThreeOssicles.holds(), "{}", ThreeOssicles.description());
    }

    #[test]
    fn test_cochlea_contains_hair_cells() {
        assert!(
            CochleaContainsHairCells.holds(),
            "{}",
            CochleaContainsHairCells.description()
        );
    }

    #[test]
    fn test_ear_contains_hair_cells() {
        assert!(
            EarContainsHairCells.holds(),
            "{}",
            EarContainsHairCells.description()
        );
    }

    #[test]
    fn test_cochlea_has_three_scalae() {
        assert!(
            CochleaHasThreeScalae.holds(),
            "{}",
            CochleaHasThreeScalae.description()
        );
    }

    #[test]
    fn test_all_regions_represented() {
        assert!(
            AllRegionsRepresented.holds(),
            "{}",
            AllRegionsRepresented.description()
        );
    }

    #[test]
    fn test_hair_cells_mechanically_active() {
        assert!(
            HairCellsAreMechanicallyActive.holds(),
            "{}",
            HairCellsAreMechanicallyActive.description()
        );
    }

    // -- Opposition tests --

    #[test]
    fn test_opposition_symmetric() {
        assert!(
            AnatomyOppositionSymmetric.holds(),
            "{}",
            AnatomyOppositionSymmetric.description()
        );
    }

    #[test]
    fn test_opposition_irreflexive() {
        assert!(
            AnatomyOppositionIrreflexive.holds(),
            "{}",
            AnatomyOppositionIrreflexive.description()
        );
    }

    #[test]
    fn test_outer_ear_opposes_inner_ear() {
        assert!(opposition::are_opposed::<AnatomyOpposition>(
            &AuditoryEntity::OuterEar,
            &AuditoryEntity::InnerEar
        ));
    }

    // -- Category law tests --

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

    // -- Taxonomy tests --

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

    // -- Mereology tests --

    #[test]
    fn test_ear_contains_cochlea_transitively() {
        let parts = mereology::parts_of::<AuditoryMereology>(&AuditoryEntity::Ear);
        assert!(
            parts.contains(&AuditoryEntity::Cochlea),
            "ear should transitively contain cochlea"
        );
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

    // -- Quality tests --

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
        #[test]
        fn prop_taxonomy_reflexive(entity in arb_auditory_entity()) {
            prop_assert!(taxonomy::is_a::<AuditoryTaxonomy>(&entity, &entity));
        }

        #[test]
        fn prop_region_is_total(entity in arb_auditory_entity()) {
            prop_assert!(RegionQuality.get(&entity).is_some());
        }
    }
}
