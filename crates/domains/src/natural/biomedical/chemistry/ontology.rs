//! Chemistry ontology: foundational chemistry of matter.
//!
//! Models states of matter, chemical bonding types, physical properties,
//! and solution components. Causal chains cover dissolution, acid-base
//! reactions, phase transitions, and diffusion.

use pr4xis::category::Entity;
use pr4xis::define_dense_category;
use pr4xis::ontology::reasoning::causation::{self, CausalDef};
use pr4xis::ontology::reasoning::opposition::{self, OppositionDef};
use pr4xis::ontology::reasoning::taxonomy::{self, TaxonomyDef};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

/// Every entity in the chemistry ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum ChemistryEntity {
    // States of matter
    Solid,
    Liquid,
    Gas,
    Plasma,
    Gel,
    Colloid,

    // Bonding
    IonicBond,
    CovalentBond,
    HydrogenBond,
    VanDerWaals,
    Metallic,

    // Properties
    PH,
    Concentration,
    Osmolarity,
    Temperature,
    Pressure,

    // Solutions
    Solvent,
    Solute,
    Electrolyte,
    Buffer,

    // Abstract
    StateOfMatter,
    ChemicalBond,
    PhysicalProperty,
    SolutionComponent,
}

// ---------------------------------------------------------------------------
// Taxonomy (is-a)
// ---------------------------------------------------------------------------

/// Subsumption hierarchy for chemistry entities.
///
/// States -> StateOfMatter with Gel -> Colloid -> Liquid sub-hierarchy.
/// Bonds -> ChemicalBond, properties -> PhysicalProperty,
/// solutions -> SolutionComponent.
pub struct ChemistryTaxonomy;

impl TaxonomyDef for ChemistryTaxonomy {
    type Entity = ChemistryEntity;

    fn relations() -> Vec<(ChemistryEntity, ChemistryEntity)> {
        use ChemistryEntity::*;
        vec![
            // States of matter is-a StateOfMatter
            (Solid, StateOfMatter),
            (Liquid, StateOfMatter),
            (Gas, StateOfMatter),
            (Plasma, StateOfMatter),
            (Colloid, StateOfMatter),
            // Gel is-a Colloid (a gel is a type of colloid)
            (Gel, Colloid),
            // Colloid is related to Liquid (colloids are liquid-based)
            (Colloid, Liquid),
            // Bonds is-a ChemicalBond
            (IonicBond, ChemicalBond),
            (CovalentBond, ChemicalBond),
            (HydrogenBond, ChemicalBond),
            (VanDerWaals, ChemicalBond),
            (Metallic, ChemicalBond),
            // Properties is-a PhysicalProperty
            (PH, PhysicalProperty),
            (Concentration, PhysicalProperty),
            (Osmolarity, PhysicalProperty),
            (Temperature, PhysicalProperty),
            (Pressure, PhysicalProperty),
            // Solutions is-a SolutionComponent
            (Solvent, SolutionComponent),
            (Solute, SolutionComponent),
            (Electrolyte, SolutionComponent),
            (Buffer, SolutionComponent),
        ]
    }
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

define_dense_category! {
    /// Discrete category over chemistry entities.
    pub ChemistryCategory {
        entity: ChemistryEntity,
        relation: ChemistryRelation,
    }
}

// ---------------------------------------------------------------------------
// Causal graph
// ---------------------------------------------------------------------------

/// Events in the chemistry causal chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum ChemistryCausalEvent {
    Dissolution,
    IonDissociation,
    ElectrolyteFormation,
    AcidBaseReaction,
    PHChange,
    ProteinDenaturation,
    TemperatureChange,
    PhaseTransition,
    ConcentrationGradient,
    Diffusion,
}

/// Causal graph for chemistry.
///
/// Dissolution -> IonDissociation -> ElectrolyteFormation
/// AcidBaseReaction -> PHChange -> ProteinDenaturation
/// TemperatureChange -> PhaseTransition
/// ConcentrationGradient -> Diffusion
pub struct ChemistryCauses;

impl CausalDef for ChemistryCauses {
    type Entity = ChemistryCausalEvent;

    fn relations() -> Vec<(ChemistryCausalEvent, ChemistryCausalEvent)> {
        use ChemistryCausalEvent::*;
        vec![
            (Dissolution, IonDissociation),
            (IonDissociation, ElectrolyteFormation),
            (AcidBaseReaction, PHChange),
            (PHChange, ProteinDenaturation),
            (TemperatureChange, PhaseTransition),
            (ConcentrationGradient, Diffusion),
        ]
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Whether an entity conducts electricity.
#[derive(Debug, Clone)]
pub struct ConductsElectricity;

impl Quality for ConductsElectricity {
    type Individual = ChemistryEntity;
    type Value = bool;

    fn get(&self, individual: &ChemistryEntity) -> Option<bool> {
        use ChemistryEntity::*;
        Some(match individual {
            Electrolyte | Plasma => true,
            Solid | Gel | Colloid => false, // depends on type, default false
            Gas | Liquid => false,
            _ => false,
        })
    }
}

/// Whether an entity is aqueous (water-based).
#[derive(Debug, Clone)]
pub struct IsAqueous;

impl Quality for IsAqueous {
    type Individual = ChemistryEntity;
    type Value = bool;

    fn get(&self, individual: &ChemistryEntity) -> Option<bool> {
        use ChemistryEntity::*;
        Some(matches!(individual, Liquid | Gel | Colloid))
    }
}

/// Bond strength classification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BondStrengthLevel {
    Strong,
    Moderate,
    Weak,
}

/// Quality: how strong is a chemical bond?
#[derive(Debug, Clone)]
pub struct BondStrength;

impl Quality for BondStrength {
    type Individual = ChemistryEntity;
    type Value = BondStrengthLevel;

    fn get(&self, individual: &ChemistryEntity) -> Option<BondStrengthLevel> {
        use ChemistryEntity::*;
        match individual {
            CovalentBond | IonicBond | Metallic => Some(BondStrengthLevel::Strong),
            HydrogenBond => Some(BondStrengthLevel::Moderate),
            VanDerWaals => Some(BondStrengthLevel::Weak),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Opposition
// ---------------------------------------------------------------------------

/// Opposition pairs in chemistry.
///
/// - Solvent ↔ Solute: the dissolving agent vs the dissolved substance
/// - IonicBond ↔ CovalentBond: electrostatic transfer vs shared electrons
pub struct ChemistryOpposition;

impl OppositionDef for ChemistryOpposition {
    type Entity = ChemistryEntity;

    fn pairs() -> Vec<(ChemistryEntity, ChemistryEntity)> {
        use ChemistryEntity::*;
        vec![(Solvent, Solute), (IonicBond, CovalentBond)]
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// The taxonomy has no cycles (is a DAG).
pub struct ChemistryTaxonomyIsDAG;

impl Axiom for ChemistryTaxonomyIsDAG {
    fn description(&self) -> &str {
        "chemistry taxonomy is a directed acyclic graph"
    }

    fn holds(&self) -> bool {
        taxonomy::NoCycles::<ChemistryTaxonomy>::new().holds()
    }
}

/// The taxonomy is antisymmetric.
pub struct ChemistryTaxonomyAntisymmetric;

impl Axiom for ChemistryTaxonomyAntisymmetric {
    fn description(&self) -> &str {
        "chemistry taxonomy is antisymmetric"
    }

    fn holds(&self) -> bool {
        taxonomy::Antisymmetric::<ChemistryTaxonomy>::new().holds()
    }
}

/// Causal graph is asymmetric.
pub struct ChemistryCausalAsymmetric;

impl Axiom for ChemistryCausalAsymmetric {
    fn description(&self) -> &str {
        "chemistry causal graph is asymmetric"
    }

    fn holds(&self) -> bool {
        causation::Asymmetric::<ChemistryCauses>::new().holds()
    }
}

/// No self-causation in chemistry.
pub struct ChemistryCausalNoSelfCausation;

impl Axiom for ChemistryCausalNoSelfCausation {
    fn description(&self) -> &str {
        "no chemistry event directly causes itself"
    }

    fn holds(&self) -> bool {
        causation::NoSelfCausation::<ChemistryCauses>::new().holds()
    }
}

/// Dissolution causes ion dissociation.
pub struct DissolutionCausesIonDissociation;

impl Axiom for DissolutionCausesIonDissociation {
    fn description(&self) -> &str {
        "dissolution causes ion dissociation"
    }

    fn holds(&self) -> bool {
        use ChemistryCausalEvent::*;
        let effects = causation::effects_of::<ChemistryCauses>(&Dissolution);
        effects.contains(&IonDissociation)
    }
}

/// Acid-base reaction causes pH change.
pub struct AcidBaseCausesPHChange;

impl Axiom for AcidBaseCausesPHChange {
    fn description(&self) -> &str {
        "acid-base reaction causes pH change"
    }

    fn holds(&self) -> bool {
        use ChemistryCausalEvent::*;
        let effects = causation::effects_of::<ChemistryCauses>(&AcidBaseReaction);
        effects.contains(&PHChange)
    }
}

/// Electrolytes conduct electricity.
pub struct ElectrolytesConductElectricity;

impl Axiom for ElectrolytesConductElectricity {
    fn description(&self) -> &str {
        "electrolytes conduct electricity"
    }

    fn holds(&self) -> bool {
        ConductsElectricity.get(&ChemistryEntity::Electrolyte) == Some(true)
    }
}

/// Opposition is symmetric.
pub struct ChemistryOppositionSymmetric;

impl Axiom for ChemistryOppositionSymmetric {
    fn description(&self) -> &str {
        "chemistry opposition is symmetric"
    }

    fn holds(&self) -> bool {
        opposition::Symmetric::<ChemistryOpposition>::new().holds()
    }
}

/// Opposition is irreflexive.
pub struct ChemistryOppositionIrreflexive;

impl Axiom for ChemistryOppositionIrreflexive {
    fn description(&self) -> &str {
        "chemistry opposition is irreflexive"
    }

    fn holds(&self) -> bool {
        opposition::Irreflexive::<ChemistryOpposition>::new().holds()
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// Top-level chemistry ontology.
pub struct ChemistryOntology;

impl Ontology for ChemistryOntology {
    type Cat = ChemistryCategory;
    type Qual = ConductsElectricity;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(ChemistryTaxonomyIsDAG),
            Box::new(ChemistryTaxonomyAntisymmetric),
            Box::new(ChemistryCausalAsymmetric),
            Box::new(ChemistryCausalNoSelfCausation),
            Box::new(DissolutionCausesIonDissociation),
            Box::new(AcidBaseCausesPHChange),
            Box::new(ElectrolytesConductElectricity),
            Box::new(ChemistryOppositionSymmetric),
            Box::new(ChemistryOppositionIrreflexive),
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
        assert!(
            ChemistryTaxonomyIsDAG.holds(),
            "{}",
            ChemistryTaxonomyIsDAG.description()
        );
    }

    #[test]
    fn test_taxonomy_antisymmetric() {
        assert!(
            ChemistryTaxonomyAntisymmetric.holds(),
            "{}",
            ChemistryTaxonomyAntisymmetric.description()
        );
    }

    #[test]
    fn test_causal_asymmetric() {
        assert!(
            ChemistryCausalAsymmetric.holds(),
            "{}",
            ChemistryCausalAsymmetric.description()
        );
    }

    #[test]
    fn test_causal_no_self_causation() {
        assert!(
            ChemistryCausalNoSelfCausation.holds(),
            "{}",
            ChemistryCausalNoSelfCausation.description()
        );
    }

    #[test]
    fn test_dissolution_causes_ion_dissociation() {
        assert!(
            DissolutionCausesIonDissociation.holds(),
            "{}",
            DissolutionCausesIonDissociation.description()
        );
    }

    #[test]
    fn test_acid_base_causes_ph_change() {
        assert!(
            AcidBaseCausesPHChange.holds(),
            "{}",
            AcidBaseCausesPHChange.description()
        );
    }

    #[test]
    fn test_electrolytes_conduct() {
        assert!(
            ElectrolytesConductElectricity.holds(),
            "{}",
            ElectrolytesConductElectricity.description()
        );
    }

    #[test]
    fn test_opposition_symmetric() {
        assert!(
            ChemistryOppositionSymmetric.holds(),
            "{}",
            ChemistryOppositionSymmetric.description()
        );
    }

    #[test]
    fn test_opposition_irreflexive() {
        assert!(
            ChemistryOppositionIrreflexive.holds(),
            "{}",
            ChemistryOppositionIrreflexive.description()
        );
    }

    // -- Category law tests --

    #[test]
    fn test_chemistry_category_laws() {
        check_category_laws::<ChemistryCategory>().unwrap();
    }

    #[test]
    fn test_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<ChemistryTaxonomy>>().unwrap();
    }

    #[test]
    fn test_causal_category_laws() {
        check_category_laws::<CausalCategory<ChemistryCauses>>().unwrap();
    }

    // -- Taxonomy tests --

    #[test]
    fn test_gel_is_a_colloid() {
        assert!(taxonomy::is_a::<ChemistryTaxonomy>(
            &ChemistryEntity::Gel,
            &ChemistryEntity::Colloid
        ));
    }

    #[test]
    fn test_colloid_is_a_liquid() {
        assert!(taxonomy::is_a::<ChemistryTaxonomy>(
            &ChemistryEntity::Colloid,
            &ChemistryEntity::Liquid
        ));
    }

    #[test]
    fn test_gel_transitively_is_a_liquid() {
        assert!(taxonomy::is_a::<ChemistryTaxonomy>(
            &ChemistryEntity::Gel,
            &ChemistryEntity::Liquid
        ));
    }

    #[test]
    fn test_ionic_bond_is_a_chemical_bond() {
        assert!(taxonomy::is_a::<ChemistryTaxonomy>(
            &ChemistryEntity::IonicBond,
            &ChemistryEntity::ChemicalBond
        ));
    }

    #[test]
    fn test_ph_is_a_physical_property() {
        assert!(taxonomy::is_a::<ChemistryTaxonomy>(
            &ChemistryEntity::PH,
            &ChemistryEntity::PhysicalProperty
        ));
    }

    #[test]
    fn test_electrolyte_is_a_solution_component() {
        assert!(taxonomy::is_a::<ChemistryTaxonomy>(
            &ChemistryEntity::Electrolyte,
            &ChemistryEntity::SolutionComponent
        ));
    }

    // -- Opposition tests --

    #[test]
    fn test_solvent_opposes_solute() {
        assert!(opposition::are_opposed::<ChemistryOpposition>(
            &ChemistryEntity::Solvent,
            &ChemistryEntity::Solute
        ));
        assert!(opposition::are_opposed::<ChemistryOpposition>(
            &ChemistryEntity::Solute,
            &ChemistryEntity::Solvent
        ));
    }

    #[test]
    fn test_ionic_opposes_covalent() {
        assert!(opposition::are_opposed::<ChemistryOpposition>(
            &ChemistryEntity::IonicBond,
            &ChemistryEntity::CovalentBond
        ));
    }

    // -- Quality tests --

    #[test]
    fn test_electrolyte_conducts() {
        assert_eq!(
            ConductsElectricity.get(&ChemistryEntity::Electrolyte),
            Some(true)
        );
    }

    #[test]
    fn test_plasma_conducts() {
        assert_eq!(
            ConductsElectricity.get(&ChemistryEntity::Plasma),
            Some(true)
        );
    }

    #[test]
    fn test_gas_does_not_conduct() {
        assert_eq!(ConductsElectricity.get(&ChemistryEntity::Gas), Some(false));
    }

    #[test]
    fn test_liquid_is_aqueous() {
        assert_eq!(IsAqueous.get(&ChemistryEntity::Liquid), Some(true));
    }

    #[test]
    fn test_gel_is_aqueous() {
        assert_eq!(IsAqueous.get(&ChemistryEntity::Gel), Some(true));
    }

    #[test]
    fn test_solid_not_aqueous() {
        assert_eq!(IsAqueous.get(&ChemistryEntity::Solid), Some(false));
    }

    #[test]
    fn test_covalent_bond_strong() {
        assert_eq!(
            BondStrength.get(&ChemistryEntity::CovalentBond),
            Some(BondStrengthLevel::Strong)
        );
    }

    #[test]
    fn test_hydrogen_bond_moderate() {
        assert_eq!(
            BondStrength.get(&ChemistryEntity::HydrogenBond),
            Some(BondStrengthLevel::Moderate)
        );
    }

    #[test]
    fn test_van_der_waals_weak() {
        assert_eq!(
            BondStrength.get(&ChemistryEntity::VanDerWaals),
            Some(BondStrengthLevel::Weak)
        );
    }

    // -- Causal chain tests --

    #[test]
    fn test_dissolution_transitively_causes_electrolyte_formation() {
        use ChemistryCausalEvent::*;
        let effects = causation::effects_of::<ChemistryCauses>(&Dissolution);
        assert!(effects.contains(&ElectrolyteFormation));
    }

    #[test]
    fn test_acid_base_transitively_causes_protein_denaturation() {
        use ChemistryCausalEvent::*;
        let effects = causation::effects_of::<ChemistryCauses>(&AcidBaseReaction);
        assert!(effects.contains(&ProteinDenaturation));
    }

    #[test]
    fn test_temperature_change_causes_phase_transition() {
        use ChemistryCausalEvent::*;
        let effects = causation::effects_of::<ChemistryCauses>(&TemperatureChange);
        assert!(effects.contains(&PhaseTransition));
    }

    #[test]
    fn test_concentration_gradient_causes_diffusion() {
        use ChemistryCausalEvent::*;
        let effects = causation::effects_of::<ChemistryCauses>(&ConcentrationGradient);
        assert!(effects.contains(&Diffusion));
    }

    #[test]
    fn test_causal_event_count() {
        assert_eq!(ChemistryCausalEvent::variants().len(), 10);
    }

    // -- Ontology validation --

    #[test]
    fn test_ontology_validates() {
        ChemistryOntology::validate().unwrap();
    }

    // -- Proptest --

    fn arb_chemistry_entity() -> impl Strategy<Value = ChemistryEntity> {
        (0..ChemistryEntity::variants().len()).prop_map(|i| ChemistryEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_taxonomy_is_a_reflexive(entity in arb_chemistry_entity()) {
            prop_assert!(taxonomy::is_a::<ChemistryTaxonomy>(&entity, &entity));
        }

        #[test]
        fn prop_conductivity_total(entity in arb_chemistry_entity()) {
            prop_assert!(ConductsElectricity.get(&entity).is_some());
        }

        #[test]
        fn prop_aqueous_total(entity in arb_chemistry_entity()) {
            prop_assert!(IsAqueous.get(&entity).is_some());
        }
    }
}
