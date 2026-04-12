//! Biological organization ontology.
//!
//! Models the hierarchy: Cell → Tissue → Organ → Organism
//! using praxis taxonomy (is-a) and mereology (has-a).

use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::causation;
use pr4xis::ontology::reasoning::mereology;
use pr4xis::ontology::reasoning::opposition;
use pr4xis::ontology::reasoning::taxonomy;
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

/// Every biological entity in the esophageal repair domain.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum BiologicalEntity {
    // Cells
    SquamousEpithelial,
    ColumnarEpithelial,
    GobletCell,
    BasalStemCell,
    Fibroblast,
    MacrophageM1,
    MacrophageM2,
    Osteocyte,
    // Tissues
    SquamousEpithelium,
    ColumnarEpithelium,
    ConnectiveTissue,
    SmoothMuscle,
    NeuralTissue,
    BoneMatrix,
    // Organs
    Esophagus,
    Heart,
    Lung,
    Brain,
    Bone,
    // Abstract
    Cell,
    Tissue,
    Organ,
    Organism,
}

// ---------------------------------------------------------------------------
// Causal event
// ---------------------------------------------------------------------------

/// Events in the biological causal chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum BiologicalCausalEvent {
    StemCellDivision,
    CellDifferentiation,
    TissueFormation,
    OrganDevelopment,
    AcidDamage,
    InflammationOnset,
    MetaplasticChange,
    FibrosisOnset,
}

// ---------------------------------------------------------------------------
// Category + Reasoning (generated)
// ---------------------------------------------------------------------------

define_ontology! {
    /// Biological organization ontology: Cell -> Tissue -> Organ -> Organism.
    pub BiologyOntologyMeta for BiologyCategory {
        entity: BiologicalEntity,
        relation: BiologicalRelation,

        taxonomy: BiologicalTaxonomy [
            (SquamousEpithelial, Cell),
            (ColumnarEpithelial, Cell),
            (GobletCell, Cell),
            (BasalStemCell, Cell),
            (Fibroblast, Cell),
            (MacrophageM1, Cell),
            (MacrophageM2, Cell),
            (Osteocyte, Cell),
            (SquamousEpithelium, Tissue),
            (ColumnarEpithelium, Tissue),
            (ConnectiveTissue, Tissue),
            (SmoothMuscle, Tissue),
            (NeuralTissue, Tissue),
            (BoneMatrix, Tissue),
            (Esophagus, Organ),
            (Heart, Organ),
            (Lung, Organ),
            (Brain, Organ),
            (Bone, Organ),
        ],

        mereology: BiologicalMereology [
            (Organism, Esophagus),
            (Organism, Heart),
            (Organism, Lung),
            (Organism, Brain),
            (Organism, Bone),
            (Esophagus, SquamousEpithelium),
            (Esophagus, ConnectiveTissue),
            (Esophagus, SmoothMuscle),
            (Esophagus, NeuralTissue),
            (Bone, BoneMatrix),
            (Bone, ConnectiveTissue),
            (SquamousEpithelium, SquamousEpithelial),
            (SquamousEpithelium, BasalStemCell),
            (ColumnarEpithelium, ColumnarEpithelial),
            (ColumnarEpithelium, GobletCell),
            (ConnectiveTissue, Fibroblast),
            (BoneMatrix, Osteocyte),
        ],

        causation: BiologicalCausalGraph for BiologicalCausalEvent [
            (StemCellDivision, CellDifferentiation),
            (CellDifferentiation, TissueFormation),
            (TissueFormation, OrganDevelopment),
            (AcidDamage, InflammationOnset),
            (InflammationOnset, MetaplasticChange),
            (InflammationOnset, FibrosisOnset),
        ],

        opposition: BiologicalOpposition [
            (SquamousEpithelial, ColumnarEpithelial),
            (MacrophageM1, MacrophageM2),
            (Cell, Organism),
        ],
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// The level of biological organization an entity belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrganizationLevel {
    Cellular,
    TissueLevel,
    OrganLevel,
    OrganismLevel,
    Abstract,
}

/// Quality: what organizational level does this entity represent?
#[derive(Debug, Clone)]
pub struct OrganizationLevelQuality;

impl Quality for OrganizationLevelQuality {
    type Individual = BiologicalEntity;
    type Value = OrganizationLevel;

    fn get(&self, individual: &BiologicalEntity) -> Option<OrganizationLevel> {
        use BiologicalEntity::*;
        use OrganizationLevel::*;
        Some(match individual {
            SquamousEpithelial | ColumnarEpithelial | GobletCell | BasalStemCell | Fibroblast
            | MacrophageM1 | MacrophageM2 | Osteocyte => Cellular,
            SquamousEpithelium | ColumnarEpithelium | ConnectiveTissue | SmoothMuscle
            | NeuralTissue | BoneMatrix => TissueLevel,
            Esophagus | Heart | Lung | Brain | Bone => OrganLevel,
            Organism => OrganismLevel,
            Cell | Tissue | Organ => Abstract,
        })
    }
}

/// Quality: is this entity proliferative (capable of division)?
#[derive(Debug, Clone)]
pub struct IsProliferative;

impl Quality for IsProliferative {
    type Individual = BiologicalEntity;
    type Value = bool;

    fn get(&self, individual: &BiologicalEntity) -> Option<bool> {
        use BiologicalEntity::*;
        Some(matches!(
            individual,
            BasalStemCell | MacrophageM1 | MacrophageM2 | Fibroblast
        ))
    }
}

/// Quality: is this entity mechanosensitive (responds to mechanical forces)?
#[derive(Debug, Clone)]
pub struct IsMechanosensitive;

impl Quality for IsMechanosensitive {
    type Individual = BiologicalEntity;
    type Value = bool;

    fn get(&self, individual: &BiologicalEntity) -> Option<bool> {
        use BiologicalEntity::*;
        Some(matches!(
            individual,
            SquamousEpithelial | ColumnarEpithelial | Osteocyte | BoneMatrix
        ))
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// The taxonomy has no cycles (is a DAG).
pub struct TaxonomyIsDAG;

impl Axiom for TaxonomyIsDAG {
    fn description(&self) -> &str {
        "biological taxonomy is a directed acyclic graph"
    }

    fn holds(&self) -> bool {
        taxonomy::NoCycles::<BiologicalTaxonomy>::new().holds()
    }
}

/// The taxonomy is antisymmetric: if A is-a B then B is not a A.
pub struct TaxonomyIsAntisymmetric;

impl Axiom for TaxonomyIsAntisymmetric {
    fn description(&self) -> &str {
        "biological taxonomy is antisymmetric"
    }

    fn holds(&self) -> bool {
        taxonomy::Antisymmetric::<BiologicalTaxonomy>::new().holds()
    }
}

/// The mereology has no cycles (is a DAG).
pub struct MereologyIsDAG;

impl Axiom for MereologyIsDAG {
    fn description(&self) -> &str {
        "biological mereology is a directed acyclic graph"
    }

    fn holds(&self) -> bool {
        mereology::NoCycles::<BiologicalMereology>::new().holds()
    }
}

/// All concrete cell types are subsumed by the abstract Cell entity.
pub struct AllCellsAreCell;

impl Axiom for AllCellsAreCell {
    fn description(&self) -> &str {
        "all cell types are subsumed by Cell"
    }

    fn holds(&self) -> bool {
        use BiologicalEntity::*;
        let cells = [
            SquamousEpithelial,
            ColumnarEpithelial,
            GobletCell,
            BasalStemCell,
            Fibroblast,
            MacrophageM1,
            MacrophageM2,
            Osteocyte,
        ];
        cells
            .iter()
            .all(|c| taxonomy::is_a::<BiologicalTaxonomy>(c, &Cell))
    }
}

/// The esophagus contains squamous epithelium (mereology).
pub struct EsophagusHasSquamousEpithelium;

impl Axiom for EsophagusHasSquamousEpithelium {
    fn description(&self) -> &str {
        "esophagus has squamous epithelium as a part"
    }

    fn holds(&self) -> bool {
        use BiologicalEntity::*;
        let parts = mereology::parts_of::<BiologicalMereology>(&Esophagus);
        parts.contains(&SquamousEpithelium)
    }
}

/// Squamous epithelium contains both squamous epithelial cells and basal stem cells.
pub struct EpitheliumHasStemCells;

impl Axiom for EpitheliumHasStemCells {
    fn description(&self) -> &str {
        "squamous epithelium contains both squamous epithelial and basal stem cells"
    }

    fn holds(&self) -> bool {
        use BiologicalEntity::*;
        let parts = mereology::parts_of::<BiologicalMereology>(&SquamousEpithelium);
        parts.contains(&SquamousEpithelial) && parts.contains(&BasalStemCell)
    }
}

/// All four non-abstract organization levels are represented.
pub struct AllLevelsRepresented;

impl Axiom for AllLevelsRepresented {
    fn description(&self) -> &str {
        "all four non-abstract organization levels are represented"
    }

    fn holds(&self) -> bool {
        use OrganizationLevel::*;
        let quality = OrganizationLevelQuality;
        let all = BiologicalEntity::variants();
        let levels: Vec<OrganizationLevel> = all.iter().filter_map(|e| quality.get(e)).collect();
        [Cellular, TissueLevel, OrganLevel, OrganismLevel]
            .iter()
            .all(|target| levels.contains(target))
    }
}

/// Axiom: basal stem cells differentiate into squamous epithelial cells.
/// This is the fundamental esophageal repair mechanism.
/// Hooper 1956; Piedrafita et al. 2020.
pub struct StemCellDifferentiation;

impl Axiom for StemCellDifferentiation {
    fn description(&self) -> &str {
        "basal stem cells and squamous epithelial cells coexist in squamous epithelium (differentiation pathway)"
    }
    fn holds(&self) -> bool {
        use BiologicalEntity::*;
        // Both cell types exist in the same tissue (mereology)
        let parts = mereology::parts_of::<BiologicalMereology>(&SquamousEpithelium);
        parts.contains(&BasalStemCell) && parts.contains(&SquamousEpithelial)
            // And BasalStemCell is proliferative while SquamousEpithelial is not
            && IsProliferative.get(&BasalStemCell) == Some(true)
            && IsProliferative.get(&SquamousEpithelial) == Some(false)
    }
}

/// Mechanosensitivity spans multiple organization levels (multi-scale property).
pub struct MechanosensitivityIsMultiscale;

impl Axiom for MechanosensitivityIsMultiscale {
    fn description(&self) -> &str {
        "mechanosensitive entities exist at both cellular and tissue levels"
    }

    fn holds(&self) -> bool {
        let mechano = IsMechanosensitive;
        let level = OrganizationLevelQuality;
        let mechano_entities: Vec<BiologicalEntity> = BiologicalEntity::variants()
            .into_iter()
            .filter(|e| mechano.get(e) == Some(true))
            .collect();
        let has_cellular = mechano_entities
            .iter()
            .any(|e| level.get(e) == Some(OrganizationLevel::Cellular));
        let has_tissue = mechano_entities
            .iter()
            .any(|e| level.get(e) == Some(OrganizationLevel::TissueLevel));
        has_cellular && has_tissue
    }
}

/// Axiom: biological opposition is symmetric.
pub struct BiologicalOppositionSymmetric;

impl Axiom for BiologicalOppositionSymmetric {
    fn description(&self) -> &str {
        "biological opposition is symmetric"
    }

    fn holds(&self) -> bool {
        opposition::Symmetric::<BiologicalOpposition>::new().holds()
    }
}

/// Axiom: biological opposition is irreflexive (nothing opposes itself).
pub struct BiologicalOppositionIrreflexive;

impl Axiom for BiologicalOppositionIrreflexive {
    fn description(&self) -> &str {
        "biological opposition is irreflexive"
    }

    fn holds(&self) -> bool {
        opposition::Irreflexive::<BiologicalOpposition>::new().holds()
    }
}

/// Axiom: biological causal graph is asymmetric.
pub struct BiologicalCausalAsymmetric;

impl Axiom for BiologicalCausalAsymmetric {
    fn description(&self) -> &str {
        "biological causal graph is asymmetric"
    }

    fn holds(&self) -> bool {
        causation::Asymmetric::<BiologicalCausalGraph>::new().holds()
    }
}

/// Axiom: no biological event directly causes itself.
pub struct BiologicalCausalNoSelfCausation;

impl Axiom for BiologicalCausalNoSelfCausation {
    fn description(&self) -> &str {
        "no biological event directly causes itself"
    }

    fn holds(&self) -> bool {
        causation::NoSelfCausation::<BiologicalCausalGraph>::new().holds()
    }
}

// ---------------------------------------------------------------------------
// Cross-domain equivalence axioms
// ---------------------------------------------------------------------------

/// Axiom: immunology->biology functor preserves MacrophageM1 identity.
/// MacrophageM1 in immunology IS MacrophageM1 in biology (cross-domain equivalence).
pub struct MacrophageM1CrossDomainEquivalence;

impl Axiom for MacrophageM1CrossDomainEquivalence {
    fn description(&self) -> &str {
        "MacrophageM1 is the same entity in immunology and biology (functor maps identity)"
    }

    fn holds(&self) -> bool {
        use crate::natural::biomedical::immunology::biology_functor::ImmunologyToBiology;
        use crate::natural::biomedical::immunology::ontology::ImmunologyEntity;
        use pr4xis::category::Functor;
        ImmunologyToBiology::map_object(&ImmunologyEntity::MacrophageM1)
            == BiologicalEntity::MacrophageM1
    }
}

/// Axiom: immunology->biology functor preserves MacrophageM2 identity.
/// MacrophageM2 in immunology IS MacrophageM2 in biology (cross-domain equivalence).
pub struct MacrophageM2CrossDomainEquivalence;

impl Axiom for MacrophageM2CrossDomainEquivalence {
    fn description(&self) -> &str {
        "MacrophageM2 is the same entity in immunology and biology (functor maps identity)"
    }

    fn holds(&self) -> bool {
        use crate::natural::biomedical::immunology::biology_functor::ImmunologyToBiology;
        use crate::natural::biomedical::immunology::ontology::ImmunologyEntity;
        use pr4xis::category::Functor;
        ImmunologyToBiology::map_object(&ImmunologyEntity::MacrophageM2)
            == BiologicalEntity::MacrophageM2
    }
}

/// Axiom: immunology->biology functor preserves Fibroblast identity.
/// Fibroblast in immunology IS Fibroblast in biology (cross-domain equivalence).
pub struct FibroblastCrossDomainEquivalence;

impl Axiom for FibroblastCrossDomainEquivalence {
    fn description(&self) -> &str {
        "Fibroblast is the same entity in immunology and biology (functor maps identity)"
    }

    fn holds(&self) -> bool {
        use crate::natural::biomedical::immunology::biology_functor::ImmunologyToBiology;
        use crate::natural::biomedical::immunology::ontology::ImmunologyEntity;
        use pr4xis::category::Functor;
        ImmunologyToBiology::map_object(&ImmunologyEntity::Fibroblast)
            == BiologicalEntity::Fibroblast
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// Top-level ontology tying together the biology category, qualities, and axioms.
pub struct BiologyOntology;

impl Ontology for BiologyOntology {
    type Cat = BiologyCategory;
    type Qual = OrganizationLevelQuality;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(TaxonomyIsDAG),
            Box::new(TaxonomyIsAntisymmetric),
            Box::new(MereologyIsDAG),
            Box::new(AllCellsAreCell),
            Box::new(EsophagusHasSquamousEpithelium),
            Box::new(EpitheliumHasStemCells),
            Box::new(AllLevelsRepresented),
            Box::new(MechanosensitivityIsMultiscale),
            Box::new(BiologicalOppositionSymmetric),
            Box::new(BiologicalOppositionIrreflexive),
            Box::new(BiologicalCausalAsymmetric),
            Box::new(BiologicalCausalNoSelfCausation),
            Box::new(MacrophageM1CrossDomainEquivalence),
            Box::new(MacrophageM2CrossDomainEquivalence),
            Box::new(FibroblastCrossDomainEquivalence),
            Box::new(StemCellDifferentiation),
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
    use pr4xis::ontology::reasoning::mereology::MereologyCategory;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;

    // -- Axiom tests --

    #[test]
    fn test_taxonomy_is_dag() {
        assert!(TaxonomyIsDAG.holds(), "{}", TaxonomyIsDAG.description());
    }

    #[test]
    fn test_taxonomy_is_antisymmetric() {
        assert!(
            TaxonomyIsAntisymmetric.holds(),
            "{}",
            TaxonomyIsAntisymmetric.description()
        );
    }

    #[test]
    fn test_mereology_is_dag() {
        assert!(MereologyIsDAG.holds(), "{}", MereologyIsDAG.description());
    }

    #[test]
    fn test_all_cells_are_cell() {
        assert!(AllCellsAreCell.holds(), "{}", AllCellsAreCell.description());
    }

    #[test]
    fn test_esophagus_has_squamous_epithelium() {
        assert!(
            EsophagusHasSquamousEpithelium.holds(),
            "{}",
            EsophagusHasSquamousEpithelium.description()
        );
    }

    #[test]
    fn test_epithelium_has_stem_cells() {
        assert!(
            EpitheliumHasStemCells.holds(),
            "{}",
            EpitheliumHasStemCells.description()
        );
    }

    #[test]
    fn test_all_levels_represented() {
        assert!(
            AllLevelsRepresented.holds(),
            "{}",
            AllLevelsRepresented.description()
        );
    }

    #[test]
    fn test_mechanosensitivity_is_multiscale() {
        assert!(
            MechanosensitivityIsMultiscale.holds(),
            "{}",
            MechanosensitivityIsMultiscale.description()
        );
    }

    // -- Category law tests --

    #[test]
    fn test_biology_category_laws() {
        check_category_laws::<BiologyCategory>().unwrap();
    }

    #[test]
    fn test_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<BiologicalTaxonomy>>().unwrap();
    }

    #[test]
    fn test_mereology_category_laws() {
        check_category_laws::<MereologyCategory<BiologicalMereology>>().unwrap();
    }

    // -- Taxonomy inheritance tests --

    #[test]
    fn test_squamous_epithelial_is_a_cell() {
        assert!(taxonomy::is_a::<BiologicalTaxonomy>(
            &BiologicalEntity::SquamousEpithelial,
            &BiologicalEntity::Cell
        ));
    }

    #[test]
    fn test_osteocyte_is_a_cell() {
        assert!(taxonomy::is_a::<BiologicalTaxonomy>(
            &BiologicalEntity::Osteocyte,
            &BiologicalEntity::Cell
        ));
    }

    #[test]
    fn test_esophagus_is_a_organ() {
        assert!(taxonomy::is_a::<BiologicalTaxonomy>(
            &BiologicalEntity::Esophagus,
            &BiologicalEntity::Organ
        ));
    }

    #[test]
    fn test_cell_is_not_tissue() {
        assert!(!taxonomy::is_a::<BiologicalTaxonomy>(
            &BiologicalEntity::Cell,
            &BiologicalEntity::Tissue
        ));
    }

    // -- Mereology transitivity tests --

    #[test]
    fn test_organism_transitively_contains_squamous_epithelial() {
        let parts = mereology::parts_of::<BiologicalMereology>(&BiologicalEntity::Organism);
        assert!(
            parts.contains(&BiologicalEntity::SquamousEpithelial),
            "organism should transitively contain squamous epithelial cells"
        );
    }

    #[test]
    fn test_esophagus_transitively_contains_basal_stem_cell() {
        let parts = mereology::parts_of::<BiologicalMereology>(&BiologicalEntity::Esophagus);
        assert!(
            parts.contains(&BiologicalEntity::BasalStemCell),
            "esophagus should transitively contain basal stem cells"
        );
    }

    #[test]
    fn test_bone_contains_osteocyte_transitively() {
        let parts = mereology::parts_of::<BiologicalMereology>(&BiologicalEntity::Bone);
        assert!(
            parts.contains(&BiologicalEntity::Osteocyte),
            "bone should transitively contain osteocytes via bone matrix"
        );
    }

    // -- Quality tests --

    #[test]
    fn test_basal_stem_cell_is_proliferative() {
        assert_eq!(
            IsProliferative.get(&BiologicalEntity::BasalStemCell),
            Some(true)
        );
    }

    #[test]
    fn test_osteocyte_is_not_proliferative() {
        assert_eq!(
            IsProliferative.get(&BiologicalEntity::Osteocyte),
            Some(false)
        );
    }

    #[test]
    fn test_osteocyte_is_mechanosensitive() {
        assert_eq!(
            IsMechanosensitive.get(&BiologicalEntity::Osteocyte),
            Some(true)
        );
    }

    #[test]
    fn test_bone_matrix_is_mechanosensitive() {
        assert_eq!(
            IsMechanosensitive.get(&BiologicalEntity::BoneMatrix),
            Some(true)
        );
    }

    #[test]
    fn test_macrophage_not_mechanosensitive() {
        assert_eq!(
            IsMechanosensitive.get(&BiologicalEntity::MacrophageM1),
            Some(false)
        );
    }

    #[test]
    fn test_organization_level_cell() {
        assert_eq!(
            OrganizationLevelQuality.get(&BiologicalEntity::Fibroblast),
            Some(OrganizationLevel::Cellular)
        );
    }

    #[test]
    fn test_organization_level_tissue() {
        assert_eq!(
            OrganizationLevelQuality.get(&BiologicalEntity::SmoothMuscle),
            Some(OrganizationLevel::TissueLevel)
        );
    }

    #[test]
    fn test_organization_level_organ() {
        assert_eq!(
            OrganizationLevelQuality.get(&BiologicalEntity::Esophagus),
            Some(OrganizationLevel::OrganLevel)
        );
    }

    #[test]
    fn test_organization_level_organism() {
        assert_eq!(
            OrganizationLevelQuality.get(&BiologicalEntity::Organism),
            Some(OrganizationLevel::OrganismLevel)
        );
    }

    #[test]
    fn test_stem_cell_differentiation() {
        assert!(
            StemCellDifferentiation.holds(),
            "{}",
            StemCellDifferentiation.description()
        );
    }

    #[test]
    fn test_ontology_validates() {
        BiologyOntology::validate().unwrap();
    }

    fn arb_bio_entity() -> impl Strategy<Value = BiologicalEntity> {
        (0..BiologicalEntity::variants().len()).prop_map(|i| BiologicalEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_organization_level_total(entity in arb_bio_entity()) {
            // Every entity has a defined organization level
            prop_assert!(OrganizationLevelQuality.get(&entity).is_some());
        }

        #[test]
        fn prop_taxonomy_is_a_reflexive(entity in arb_bio_entity()) {
            // Every entity is-a itself
            prop_assert!(taxonomy::is_a::<BiologicalTaxonomy>(&entity, &entity));
        }
    }

    // -- Opposition tests --

    #[test]
    fn test_biological_opposition_symmetric() {
        assert!(
            BiologicalOppositionSymmetric.holds(),
            "{}",
            BiologicalOppositionSymmetric.description()
        );
    }

    #[test]
    fn test_biological_opposition_irreflexive() {
        assert!(
            BiologicalOppositionIrreflexive.holds(),
            "{}",
            BiologicalOppositionIrreflexive.description()
        );
    }

    #[test]
    fn test_squamous_opposes_columnar() {
        use BiologicalEntity::*;
        assert!(opposition::are_opposed::<BiologicalOpposition>(
            &SquamousEpithelial,
            &ColumnarEpithelial
        ));
        assert!(opposition::are_opposed::<BiologicalOpposition>(
            &ColumnarEpithelial,
            &SquamousEpithelial
        ));
    }

    #[test]
    fn test_m1_opposes_m2() {
        use BiologicalEntity::*;
        assert!(opposition::are_opposed::<BiologicalOpposition>(
            &MacrophageM1,
            &MacrophageM2
        ));
    }

    #[test]
    fn test_cell_opposes_organism() {
        use BiologicalEntity::*;
        assert!(opposition::are_opposed::<BiologicalOpposition>(
            &Cell, &Organism
        ));
        assert!(opposition::are_opposed::<BiologicalOpposition>(
            &Organism, &Cell
        ));
    }

    #[test]
    fn test_cell_does_not_oppose_tissue() {
        use BiologicalEntity::*;
        assert!(!opposition::are_opposed::<BiologicalOpposition>(
            &Cell, &Tissue
        ));
    }

    #[test]
    fn test_biological_opposites_query() {
        use BiologicalEntity::*;
        let opps = opposition::opposites::<BiologicalOpposition>(&SquamousEpithelial);
        assert_eq!(opps, vec![ColumnarEpithelial]);
    }

    // -- Causal graph tests --

    #[test]
    fn test_biological_causal_asymmetric() {
        assert!(
            BiologicalCausalAsymmetric.holds(),
            "{}",
            BiologicalCausalAsymmetric.description()
        );
    }

    #[test]
    fn test_biological_causal_no_self_causation() {
        assert!(
            BiologicalCausalNoSelfCausation.holds(),
            "{}",
            BiologicalCausalNoSelfCausation.description()
        );
    }

    #[test]
    fn test_causal_category_laws() {
        check_category_laws::<CausalCategory<BiologicalCausalGraph>>().unwrap();
    }

    #[test]
    fn test_stem_cell_division_causes_organ_development() {
        use BiologicalCausalEvent::*;
        let effects = causation::effects_of::<BiologicalCausalGraph>(&StemCellDivision);
        assert!(effects.contains(&OrganDevelopment));
    }

    #[test]
    fn test_acid_damage_causes_metaplastic_change() {
        use BiologicalCausalEvent::*;
        let effects = causation::effects_of::<BiologicalCausalGraph>(&AcidDamage);
        assert!(effects.contains(&MetaplasticChange));
    }

    #[test]
    fn test_inflammation_causes_fibrosis() {
        use BiologicalCausalEvent::*;
        let effects = causation::effects_of::<BiologicalCausalGraph>(&InflammationOnset);
        assert!(effects.contains(&FibrosisOnset));
    }

    #[test]
    fn test_biological_causal_event_count() {
        assert_eq!(BiologicalCausalEvent::variants().len(), 8);
    }

    // -- Cross-domain equivalence tests --

    #[test]
    fn test_macrophage_m1_cross_domain_equivalence() {
        assert!(
            MacrophageM1CrossDomainEquivalence.holds(),
            "{}",
            MacrophageM1CrossDomainEquivalence.description()
        );
    }

    #[test]
    fn test_macrophage_m2_cross_domain_equivalence() {
        assert!(
            MacrophageM2CrossDomainEquivalence.holds(),
            "{}",
            MacrophageM2CrossDomainEquivalence.description()
        );
    }

    #[test]
    fn test_fibroblast_cross_domain_equivalence() {
        assert!(
            FibroblastCrossDomainEquivalence.holds(),
            "{}",
            FibroblastCrossDomainEquivalence.description()
        );
    }
}
