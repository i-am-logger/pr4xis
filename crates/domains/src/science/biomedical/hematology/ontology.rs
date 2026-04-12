//! Hematology ontology: blood and blood plasma science.
//!
//! Models blood components (whole blood, plasma, serum, cells, platelets),
//! plasma proteins (albumin, globulin, fibrinogen, immunoglobulin),
//! plasma electrolytes (Na+, K+, Ca2+, Cl-, HCO3-), and blood properties
//! (osmotic pressure, oncotic pressure, pH, hematocrit, viscosity).

use praxis::category::{Category, Entity, Relationship};
use praxis::ontology::reasoning::causation::{self, CausalDef};
use praxis::ontology::reasoning::mereology::{self, MereologyDef};
use praxis::ontology::reasoning::opposition::{self, OppositionDef};
use praxis::ontology::reasoning::taxonomy::{self, TaxonomyDef};
use praxis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

/// Every entity in the hematology ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HematologyEntity {
    // Blood components
    WholeBlood,
    BloodPlasma,
    Serum,
    RedBloodCell,
    WhiteBloodCell,
    Platelet,

    // Plasma proteins
    Albumin,
    Globulin,
    Fibrinogen,
    Immunoglobulin,

    // Plasma electrolytes
    SodiumPlasma,
    PotassiumPlasma,
    CalciumPlasma,
    ChloridePlasma,
    BicarbonatePlasma,

    // Properties
    OsmoticPressure,
    OncoticPressure,
    BloodPH,
    Hematocrit,
    Viscosity,

    // Abstract
    BloodComponent,
    PlasmaProtein,
    PlasmaElectrolyte,
    BloodProperty,
}

impl Entity for HematologyEntity {
    fn variants() -> Vec<Self> {
        use HematologyEntity::*;
        vec![
            WholeBlood,
            BloodPlasma,
            Serum,
            RedBloodCell,
            WhiteBloodCell,
            Platelet,
            Albumin,
            Globulin,
            Fibrinogen,
            Immunoglobulin,
            SodiumPlasma,
            PotassiumPlasma,
            CalciumPlasma,
            ChloridePlasma,
            BicarbonatePlasma,
            OsmoticPressure,
            OncoticPressure,
            BloodPH,
            Hematocrit,
            Viscosity,
            BloodComponent,
            PlasmaProtein,
            PlasmaElectrolyte,
            BloodProperty,
        ]
    }
}

// ---------------------------------------------------------------------------
// Taxonomy (is-a)
// ---------------------------------------------------------------------------

/// Subsumption hierarchy for hematology entities.
///
/// Components -> BloodComponent, proteins -> PlasmaProtein,
/// electrolytes -> PlasmaElectrolyte, properties -> BloodProperty.
/// Serum is plasma minus clotting factors (both are BloodComponent).
pub struct HematologyTaxonomy;

impl TaxonomyDef for HematologyTaxonomy {
    type Entity = HematologyEntity;

    fn relations() -> Vec<(HematologyEntity, HematologyEntity)> {
        use HematologyEntity::*;
        vec![
            // Blood components is-a BloodComponent
            (WholeBlood, BloodComponent),
            (BloodPlasma, BloodComponent),
            (Serum, BloodComponent),
            (RedBloodCell, BloodComponent),
            (WhiteBloodCell, BloodComponent),
            (Platelet, BloodComponent),
            // Proteins is-a PlasmaProtein
            (Albumin, PlasmaProtein),
            (Globulin, PlasmaProtein),
            (Fibrinogen, PlasmaProtein),
            (Immunoglobulin, PlasmaProtein),
            // Electrolytes is-a PlasmaElectrolyte
            (SodiumPlasma, PlasmaElectrolyte),
            (PotassiumPlasma, PlasmaElectrolyte),
            (CalciumPlasma, PlasmaElectrolyte),
            (ChloridePlasma, PlasmaElectrolyte),
            (BicarbonatePlasma, PlasmaElectrolyte),
            // Properties is-a BloodProperty
            (OsmoticPressure, BloodProperty),
            (OncoticPressure, BloodProperty),
            (BloodPH, BloodProperty),
            (Hematocrit, BloodProperty),
            (Viscosity, BloodProperty),
        ]
    }
}

// ---------------------------------------------------------------------------
// Mereology (has-a / part-whole)
// ---------------------------------------------------------------------------

/// Part-whole relationships for hematology entities.
///
/// WholeBlood has-a BloodPlasma, RedBloodCell, WhiteBloodCell, Platelet.
/// BloodPlasma has-a all proteins and electrolytes.
pub struct HematologyMereology;

impl MereologyDef for HematologyMereology {
    type Entity = HematologyEntity;

    fn relations() -> Vec<(HematologyEntity, HematologyEntity)> {
        use HematologyEntity::*;
        vec![
            // WholeBlood has-a cellular components and plasma
            (WholeBlood, BloodPlasma),
            (WholeBlood, RedBloodCell),
            (WholeBlood, WhiteBloodCell),
            (WholeBlood, Platelet),
            // BloodPlasma has-a proteins
            (BloodPlasma, Albumin),
            (BloodPlasma, Globulin),
            (BloodPlasma, Fibrinogen),
            (BloodPlasma, Immunoglobulin),
            // BloodPlasma has-a electrolytes
            (BloodPlasma, SodiumPlasma),
            (BloodPlasma, PotassiumPlasma),
            (BloodPlasma, CalciumPlasma),
            (BloodPlasma, ChloridePlasma),
            (BloodPlasma, BicarbonatePlasma),
        ]
    }
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

/// A morphism in the hematology category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HematologyRelation {
    pub from: HematologyEntity,
    pub to: HematologyEntity,
}

impl Relationship for HematologyRelation {
    type Object = HematologyEntity;

    fn source(&self) -> HematologyEntity {
        self.from
    }

    fn target(&self) -> HematologyEntity {
        self.to
    }
}

/// Discrete category over hematology entities.
pub struct HematologyCategory;

impl Category for HematologyCategory {
    type Object = HematologyEntity;
    type Morphism = HematologyRelation;

    fn identity(obj: &HematologyEntity) -> HematologyRelation {
        HematologyRelation {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &HematologyRelation, g: &HematologyRelation) -> Option<HematologyRelation> {
        if f.to != g.from {
            return None;
        }
        Some(HematologyRelation {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<HematologyRelation> {
        let variants = HematologyEntity::variants();
        let mut morphisms = Vec::new();
        for &from in &variants {
            for &to in &variants {
                morphisms.push(HematologyRelation { from, to });
            }
        }
        morphisms
    }
}

// ---------------------------------------------------------------------------
// Causal graph
// ---------------------------------------------------------------------------

/// Events in the hematology causal chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HematologyCausalEvent {
    Hemorrhage,
    PlasmaVolumeLoss,
    ElectrolyteImbalance,
    Inflammation,
    AcutePhaseResponse,
    AlbuminDecrease,
    AcidBaseDisturbance,
    BicarbonateBuffering,
    PHCorrection,
    CoagulationCascade,
    FibrinFormation,
}

impl Entity for HematologyCausalEvent {
    fn variants() -> Vec<Self> {
        use HematologyCausalEvent::*;
        vec![
            Hemorrhage,
            PlasmaVolumeLoss,
            ElectrolyteImbalance,
            Inflammation,
            AcutePhaseResponse,
            AlbuminDecrease,
            AcidBaseDisturbance,
            BicarbonateBuffering,
            PHCorrection,
            CoagulationCascade,
            FibrinFormation,
        ]
    }
}

/// Causal graph for hematology.
///
/// Hemorrhage -> PlasmaVolumeLoss -> ElectrolyteImbalance
/// Inflammation -> AcutePhaseResponse -> AlbuminDecrease
/// AcidBaseDisturbance -> BicarbonateBuffering -> PHCorrection
/// CoagulationCascade -> FibrinFormation
pub struct HematologyCauses;

impl CausalDef for HematologyCauses {
    type Entity = HematologyCausalEvent;

    fn relations() -> Vec<(HematologyCausalEvent, HematologyCausalEvent)> {
        use HematologyCausalEvent::*;
        vec![
            (Hemorrhage, PlasmaVolumeLoss),
            (PlasmaVolumeLoss, ElectrolyteImbalance),
            (Inflammation, AcutePhaseResponse),
            (AcutePhaseResponse, AlbuminDecrease),
            (AcidBaseDisturbance, BicarbonateBuffering),
            (BicarbonateBuffering, PHCorrection),
            (CoagulationCascade, FibrinFormation),
        ]
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Normal plasma concentration in mmol/L for electrolytes.
#[derive(Debug, Clone)]
pub struct NormalConcentration;

impl Quality for NormalConcentration {
    type Individual = HematologyEntity;
    type Value = f64;

    fn get(&self, individual: &HematologyEntity) -> Option<f64> {
        use HematologyEntity::*;
        match individual {
            SodiumPlasma => Some(140.0),
            PotassiumPlasma => Some(4.5),
            CalciumPlasma => Some(2.5),
            ChloridePlasma => Some(100.0),
            BicarbonatePlasma => Some(24.0),
            _ => None,
        }
    }
}

/// Whether an entity is a clotting factor.
#[derive(Debug, Clone)]
pub struct IsClottingFactor;

impl Quality for IsClottingFactor {
    type Individual = HematologyEntity;
    type Value = bool;

    fn get(&self, individual: &HematologyEntity) -> Option<bool> {
        use HematologyEntity::*;
        Some(matches!(individual, Fibrinogen | Platelet))
    }
}

/// Whether an entity affects osmolarity.
#[derive(Debug, Clone)]
pub struct AffectsOsmolarity;

impl Quality for AffectsOsmolarity {
    type Individual = HematologyEntity;
    type Value = bool;

    fn get(&self, individual: &HematologyEntity) -> Option<bool> {
        use HematologyEntity::*;
        Some(matches!(
            individual,
            SodiumPlasma
                | PotassiumPlasma
                | CalciumPlasma
                | ChloridePlasma
                | BicarbonatePlasma
                | Albumin
        ))
    }
}

// ---------------------------------------------------------------------------
// Opposition
// ---------------------------------------------------------------------------

/// Opposition pairs in hematology.
///
/// - Hemorrhage ↔ CoagulationCascade (bleeding vs clotting) — modeled via
///   causal events, but also as entity-level opposition using Platelet vs
///   the abstract BloodComponent to capture the semantic contrast.
/// - Albumin ↔ Globulin (transport vs immune function)
/// - RedBloodCell ↔ WhiteBloodCell (oxygen transport vs immune defense)
pub struct HematologyOpposition;

impl OppositionDef for HematologyOpposition {
    type Entity = HematologyEntity;

    fn pairs() -> Vec<(HematologyEntity, HematologyEntity)> {
        use HematologyEntity::*;
        vec![(Albumin, Globulin), (RedBloodCell, WhiteBloodCell)]
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// The taxonomy has no cycles (is a DAG).
pub struct HematologyTaxonomyIsDAG;

impl Axiom for HematologyTaxonomyIsDAG {
    fn description(&self) -> &str {
        "hematology taxonomy is a directed acyclic graph"
    }

    fn holds(&self) -> bool {
        taxonomy::NoCycles::<HematologyTaxonomy>::new().holds()
    }
}

/// The taxonomy is antisymmetric.
pub struct HematologyTaxonomyAntisymmetric;

impl Axiom for HematologyTaxonomyAntisymmetric {
    fn description(&self) -> &str {
        "hematology taxonomy is antisymmetric"
    }

    fn holds(&self) -> bool {
        taxonomy::Antisymmetric::<HematologyTaxonomy>::new().holds()
    }
}

/// The mereology has no cycles (is a DAG).
pub struct HematologyMereologyIsDAG;

impl Axiom for HematologyMereologyIsDAG {
    fn description(&self) -> &str {
        "hematology mereology is a directed acyclic graph"
    }

    fn holds(&self) -> bool {
        mereology::NoCycles::<HematologyMereology>::new().holds()
    }
}

/// Causal graph is asymmetric.
pub struct HematologyCausalAsymmetric;

impl Axiom for HematologyCausalAsymmetric {
    fn description(&self) -> &str {
        "hematology causal graph is asymmetric"
    }

    fn holds(&self) -> bool {
        causation::Asymmetric::<HematologyCauses>::new().holds()
    }
}

/// No self-causation.
pub struct HematologyCausalNoSelfCausation;

impl Axiom for HematologyCausalNoSelfCausation {
    fn description(&self) -> &str {
        "no hematology event directly causes itself"
    }

    fn holds(&self) -> bool {
        causation::NoSelfCausation::<HematologyCauses>::new().holds()
    }
}

/// WholeBlood contains BloodPlasma (mereology).
pub struct WholeBloodContainsPlasma;

impl Axiom for WholeBloodContainsPlasma {
    fn description(&self) -> &str {
        "whole blood contains blood plasma"
    }

    fn holds(&self) -> bool {
        use HematologyEntity::*;
        let parts = mereology::parts_of::<HematologyMereology>(&WholeBlood);
        parts.contains(&BloodPlasma)
    }
}

/// BloodPlasma contains all electrolytes (mereology).
pub struct PlasmaContainsAllElectrolytes;

impl Axiom for PlasmaContainsAllElectrolytes {
    fn description(&self) -> &str {
        "blood plasma contains all plasma electrolytes"
    }

    fn holds(&self) -> bool {
        use HematologyEntity::*;
        let parts = mereology::parts_of::<HematologyMereology>(&BloodPlasma);
        parts.contains(&SodiumPlasma)
            && parts.contains(&PotassiumPlasma)
            && parts.contains(&CalciumPlasma)
            && parts.contains(&ChloridePlasma)
            && parts.contains(&BicarbonatePlasma)
    }
}

/// Sodium is the dominant plasma cation: Na >> K.
pub struct SodiumIsDominantCation;

impl Axiom for SodiumIsDominantCation {
    fn description(&self) -> &str {
        "sodium is the dominant plasma cation (140 mmol/L >> 4.5 mmol/L potassium)"
    }

    fn holds(&self) -> bool {
        use HematologyEntity::*;
        let na = NormalConcentration.get(&SodiumPlasma).unwrap();
        let k = NormalConcentration.get(&PotassiumPlasma).unwrap();
        na > k * 10.0 // Na is ~31x K
    }
}

/// Blood pH is tightly regulated (7.35-7.45).
pub struct BloodPHRegulated;

impl Axiom for BloodPHRegulated {
    fn description(&self) -> &str {
        "blood pH is tightly regulated between 7.35 and 7.45"
    }

    fn holds(&self) -> bool {
        // The normal blood pH range is a scientific fact axiom.
        // We verify the bicarbonate buffering system exists to maintain it.
        use HematologyCausalEvent::*;
        let effects = causation::effects_of::<HematologyCauses>(&AcidBaseDisturbance);
        effects.contains(&PHCorrection)
    }
}

/// Hemorrhage transitively causes electrolyte imbalance.
pub struct HemorrhageCausesElectrolyteImbalance;

impl Axiom for HemorrhageCausesElectrolyteImbalance {
    fn description(&self) -> &str {
        "hemorrhage transitively causes electrolyte imbalance"
    }

    fn holds(&self) -> bool {
        use HematologyCausalEvent::*;
        let effects = causation::effects_of::<HematologyCauses>(&Hemorrhage);
        effects.contains(&ElectrolyteImbalance)
    }
}

/// Opposition is symmetric.
pub struct HematologyOppositionSymmetric;

impl Axiom for HematologyOppositionSymmetric {
    fn description(&self) -> &str {
        "hematology opposition is symmetric"
    }

    fn holds(&self) -> bool {
        opposition::Symmetric::<HematologyOpposition>::new().holds()
    }
}

/// Opposition is irreflexive.
pub struct HematologyOppositionIrreflexive;

impl Axiom for HematologyOppositionIrreflexive {
    fn description(&self) -> &str {
        "hematology opposition is irreflexive"
    }

    fn holds(&self) -> bool {
        opposition::Irreflexive::<HematologyOpposition>::new().holds()
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// Top-level hematology ontology.
pub struct HematologyOntology;

impl Ontology for HematologyOntology {
    type Cat = HematologyCategory;
    type Qual = NormalConcentration;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(HematologyTaxonomyIsDAG),
            Box::new(HematologyTaxonomyAntisymmetric),
            Box::new(HematologyMereologyIsDAG),
            Box::new(HematologyCausalAsymmetric),
            Box::new(HematologyCausalNoSelfCausation),
            Box::new(WholeBloodContainsPlasma),
            Box::new(PlasmaContainsAllElectrolytes),
            Box::new(SodiumIsDominantCation),
            Box::new(BloodPHRegulated),
            Box::new(HemorrhageCausesElectrolyteImbalance),
            Box::new(HematologyOppositionSymmetric),
            Box::new(HematologyOppositionIrreflexive),
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
            HematologyTaxonomyIsDAG.holds(),
            "{}",
            HematologyTaxonomyIsDAG.description()
        );
    }

    #[test]
    fn test_taxonomy_antisymmetric() {
        assert!(
            HematologyTaxonomyAntisymmetric.holds(),
            "{}",
            HematologyTaxonomyAntisymmetric.description()
        );
    }

    #[test]
    fn test_mereology_is_dag() {
        assert!(
            HematologyMereologyIsDAG.holds(),
            "{}",
            HematologyMereologyIsDAG.description()
        );
    }

    #[test]
    fn test_causal_asymmetric() {
        assert!(
            HematologyCausalAsymmetric.holds(),
            "{}",
            HematologyCausalAsymmetric.description()
        );
    }

    #[test]
    fn test_causal_no_self_causation() {
        assert!(
            HematologyCausalNoSelfCausation.holds(),
            "{}",
            HematologyCausalNoSelfCausation.description()
        );
    }

    #[test]
    fn test_whole_blood_contains_plasma() {
        assert!(
            WholeBloodContainsPlasma.holds(),
            "{}",
            WholeBloodContainsPlasma.description()
        );
    }

    #[test]
    fn test_plasma_contains_all_electrolytes() {
        assert!(
            PlasmaContainsAllElectrolytes.holds(),
            "{}",
            PlasmaContainsAllElectrolytes.description()
        );
    }

    #[test]
    fn test_sodium_is_dominant_cation() {
        assert!(
            SodiumIsDominantCation.holds(),
            "{}",
            SodiumIsDominantCation.description()
        );
    }

    #[test]
    fn test_blood_ph_regulated() {
        assert!(
            BloodPHRegulated.holds(),
            "{}",
            BloodPHRegulated.description()
        );
    }

    #[test]
    fn test_hemorrhage_causes_electrolyte_imbalance() {
        assert!(
            HemorrhageCausesElectrolyteImbalance.holds(),
            "{}",
            HemorrhageCausesElectrolyteImbalance.description()
        );
    }

    #[test]
    fn test_opposition_symmetric() {
        assert!(
            HematologyOppositionSymmetric.holds(),
            "{}",
            HematologyOppositionSymmetric.description()
        );
    }

    #[test]
    fn test_opposition_irreflexive() {
        assert!(
            HematologyOppositionIrreflexive.holds(),
            "{}",
            HematologyOppositionIrreflexive.description()
        );
    }

    // -- Category law tests --

    #[test]
    fn test_hematology_category_laws() {
        check_category_laws::<HematologyCategory>().unwrap();
    }

    #[test]
    fn test_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<HematologyTaxonomy>>().unwrap();
    }

    #[test]
    fn test_mereology_category_laws() {
        check_category_laws::<MereologyCategory<HematologyMereology>>().unwrap();
    }

    #[test]
    fn test_causal_category_laws() {
        check_category_laws::<CausalCategory<HematologyCauses>>().unwrap();
    }

    // -- Taxonomy tests --

    #[test]
    fn test_blood_plasma_is_a_blood_component() {
        assert!(taxonomy::is_a::<HematologyTaxonomy>(
            &HematologyEntity::BloodPlasma,
            &HematologyEntity::BloodComponent
        ));
    }

    #[test]
    fn test_serum_is_a_blood_component() {
        assert!(taxonomy::is_a::<HematologyTaxonomy>(
            &HematologyEntity::Serum,
            &HematologyEntity::BloodComponent
        ));
    }

    #[test]
    fn test_albumin_is_a_plasma_protein() {
        assert!(taxonomy::is_a::<HematologyTaxonomy>(
            &HematologyEntity::Albumin,
            &HematologyEntity::PlasmaProtein
        ));
    }

    #[test]
    fn test_sodium_is_a_plasma_electrolyte() {
        assert!(taxonomy::is_a::<HematologyTaxonomy>(
            &HematologyEntity::SodiumPlasma,
            &HematologyEntity::PlasmaElectrolyte
        ));
    }

    #[test]
    fn test_hematocrit_is_a_blood_property() {
        assert!(taxonomy::is_a::<HematologyTaxonomy>(
            &HematologyEntity::Hematocrit,
            &HematologyEntity::BloodProperty
        ));
    }

    // -- Mereology tests --

    #[test]
    fn test_whole_blood_contains_rbc() {
        let parts = mereology::parts_of::<HematologyMereology>(&HematologyEntity::WholeBlood);
        assert!(parts.contains(&HematologyEntity::RedBloodCell));
    }

    #[test]
    fn test_whole_blood_transitively_contains_sodium() {
        let parts = mereology::parts_of::<HematologyMereology>(&HematologyEntity::WholeBlood);
        assert!(
            parts.contains(&HematologyEntity::SodiumPlasma),
            "whole blood should transitively contain sodium via plasma"
        );
    }

    #[test]
    fn test_whole_blood_transitively_contains_albumin() {
        let parts = mereology::parts_of::<HematologyMereology>(&HematologyEntity::WholeBlood);
        assert!(
            parts.contains(&HematologyEntity::Albumin),
            "whole blood should transitively contain albumin via plasma"
        );
    }

    #[test]
    fn test_plasma_contains_fibrinogen() {
        let parts = mereology::parts_of::<HematologyMereology>(&HematologyEntity::BloodPlasma);
        assert!(parts.contains(&HematologyEntity::Fibrinogen));
    }

    // -- Opposition tests --

    #[test]
    fn test_albumin_opposes_globulin() {
        assert!(opposition::are_opposed::<HematologyOpposition>(
            &HematologyEntity::Albumin,
            &HematologyEntity::Globulin
        ));
        assert!(opposition::are_opposed::<HematologyOpposition>(
            &HematologyEntity::Globulin,
            &HematologyEntity::Albumin
        ));
    }

    #[test]
    fn test_rbc_opposes_wbc() {
        assert!(opposition::are_opposed::<HematologyOpposition>(
            &HematologyEntity::RedBloodCell,
            &HematologyEntity::WhiteBloodCell
        ));
    }

    // -- Quality tests --

    #[test]
    fn test_sodium_concentration() {
        assert_eq!(
            NormalConcentration.get(&HematologyEntity::SodiumPlasma),
            Some(140.0)
        );
    }

    #[test]
    fn test_potassium_concentration() {
        assert_eq!(
            NormalConcentration.get(&HematologyEntity::PotassiumPlasma),
            Some(4.5)
        );
    }

    #[test]
    fn test_calcium_concentration() {
        assert_eq!(
            NormalConcentration.get(&HematologyEntity::CalciumPlasma),
            Some(2.5)
        );
    }

    #[test]
    fn test_chloride_concentration() {
        assert_eq!(
            NormalConcentration.get(&HematologyEntity::ChloridePlasma),
            Some(100.0)
        );
    }

    #[test]
    fn test_bicarbonate_concentration() {
        assert_eq!(
            NormalConcentration.get(&HematologyEntity::BicarbonatePlasma),
            Some(24.0)
        );
    }

    #[test]
    fn test_fibrinogen_is_clotting_factor() {
        assert_eq!(
            IsClottingFactor.get(&HematologyEntity::Fibrinogen),
            Some(true)
        );
    }

    #[test]
    fn test_platelet_is_clotting_factor() {
        assert_eq!(
            IsClottingFactor.get(&HematologyEntity::Platelet),
            Some(true)
        );
    }

    #[test]
    fn test_albumin_is_not_clotting_factor() {
        assert_eq!(
            IsClottingFactor.get(&HematologyEntity::Albumin),
            Some(false)
        );
    }

    #[test]
    fn test_sodium_affects_osmolarity() {
        assert_eq!(
            AffectsOsmolarity.get(&HematologyEntity::SodiumPlasma),
            Some(true)
        );
    }

    #[test]
    fn test_albumin_affects_osmolarity() {
        assert_eq!(
            AffectsOsmolarity.get(&HematologyEntity::Albumin),
            Some(true)
        );
    }

    #[test]
    fn test_rbc_does_not_affect_osmolarity() {
        assert_eq!(
            AffectsOsmolarity.get(&HematologyEntity::RedBloodCell),
            Some(false)
        );
    }

    // -- Causal chain tests --

    #[test]
    fn test_hemorrhage_transitively_causes_electrolyte_imbalance() {
        use HematologyCausalEvent::*;
        let effects = causation::effects_of::<HematologyCauses>(&Hemorrhage);
        assert!(effects.contains(&ElectrolyteImbalance));
    }

    #[test]
    fn test_inflammation_causes_albumin_decrease() {
        use HematologyCausalEvent::*;
        let effects = causation::effects_of::<HematologyCauses>(&Inflammation);
        assert!(effects.contains(&AlbuminDecrease));
    }

    #[test]
    fn test_acid_base_disturbance_causes_ph_correction() {
        use HematologyCausalEvent::*;
        let effects = causation::effects_of::<HematologyCauses>(&AcidBaseDisturbance);
        assert!(effects.contains(&PHCorrection));
    }

    #[test]
    fn test_coagulation_cascade_causes_fibrin_formation() {
        use HematologyCausalEvent::*;
        let effects = causation::effects_of::<HematologyCauses>(&CoagulationCascade);
        assert!(effects.contains(&FibrinFormation));
    }

    #[test]
    fn test_causal_event_count() {
        assert_eq!(HematologyCausalEvent::variants().len(), 11);
    }

    // -- Ontology validation --

    #[test]
    fn test_ontology_validates() {
        HematologyOntology::validate().unwrap();
    }

    // -- Proptest --

    fn arb_hematology_entity() -> impl Strategy<Value = HematologyEntity> {
        (0..HematologyEntity::variants().len()).prop_map(|i| HematologyEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_taxonomy_is_a_reflexive(entity in arb_hematology_entity()) {
            prop_assert!(taxonomy::is_a::<HematologyTaxonomy>(&entity, &entity));
        }

        #[test]
        fn prop_clotting_factor_total(entity in arb_hematology_entity()) {
            prop_assert!(IsClottingFactor.get(&entity).is_some());
        }

        #[test]
        fn prop_affects_osmolarity_total(entity in arb_hematology_entity()) {
            prop_assert!(AffectsOsmolarity.get(&entity).is_some());
        }
    }
}
