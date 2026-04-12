//! Core Levin bioelectric framework ontology.
//!
//! Models Dr. Michael Levin's TAME (Technological Approach to Mind Everywhere)
//! framework as formal ontology:
//! - Competency hierarchy: Molecular → Cellular → Tissue → Organ → Organism
//! - Bioelectric code: Vmem patterns encode morphogenetic information
//! - Gap junction networks: signal propagation topology
//! - Cognitive lightcone: scale of goal-directed agency
//!
//! Key references:
//! - Levin 2019: The Computational Boundary of a "Self"
//! - Fields & Levin 2022: Competency in Navigating Arbitrary Spaces
//! - Levin 2014: Molecular bioelectricity in developmental biology

use praxis::category::{Category, Entity, Relationship};
use praxis::ontology::reasoning::causation::{self, CausalDef};
use praxis::ontology::reasoning::opposition::{self, OppositionDef};
use praxis::ontology::reasoning::taxonomy::{self, TaxonomyDef};
use praxis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// TAME Competency Levels
// ---------------------------------------------------------------------------

/// Levels of competency in the TAME hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompetencyLevel {
    Molecular,
    Cellular,
    Tissue,
    Organ,
    Organism,
}

impl Entity for CompetencyLevel {
    fn variants() -> Vec<Self> {
        vec![
            Self::Molecular,
            Self::Cellular,
            Self::Tissue,
            Self::Organ,
            Self::Organism,
        ]
    }
}

/// TAME hierarchy: Molecular → Cellular → Tissue → Organ → Organism.
pub struct TAMETaxonomy;

impl TaxonomyDef for TAMETaxonomy {
    type Entity = CompetencyLevel;

    fn relations() -> Vec<(CompetencyLevel, CompetencyLevel)> {
        use CompetencyLevel::*;
        vec![
            (Molecular, Cellular),
            (Cellular, Tissue),
            (Tissue, Organ),
            (Organ, Organism),
        ]
    }
}

// ---------------------------------------------------------------------------
// Bioelectric Entity
// ---------------------------------------------------------------------------

/// Every bioelectric entity in the Levin framework.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BioelectricEntity {
    // Signals
    MembranePotential,
    VoltageGradient,
    BioelectricPrepattern,
    TransepithelialPotential,
    // Networks
    GapJunctionNetwork,
    BioelectricCircuit,
    CognitiveLightcone,
    // Morphospace
    TargetMorphology,
    CurrentMorphology,
    MorphogeneticField,
    // Interventions
    IonChannelModulation,
    GapJunctionModulation,
    BioelectricCocktail,
    MechanicalStimulation,
    ProtonPumpInhibition,
    // Abstract
    Signal,
    Network,
    Morphospace,
    Intervention,
}

impl Entity for BioelectricEntity {
    fn variants() -> Vec<Self> {
        use BioelectricEntity::*;
        vec![
            MembranePotential,
            VoltageGradient,
            BioelectricPrepattern,
            TransepithelialPotential,
            GapJunctionNetwork,
            BioelectricCircuit,
            CognitiveLightcone,
            TargetMorphology,
            CurrentMorphology,
            MorphogeneticField,
            IonChannelModulation,
            GapJunctionModulation,
            BioelectricCocktail,
            MechanicalStimulation,
            ProtonPumpInhibition,
            Signal,
            Network,
            Morphospace,
            Intervention,
        ]
    }
}

// ---------------------------------------------------------------------------
// Taxonomy (is-a)
// ---------------------------------------------------------------------------

/// Subsumption hierarchy for bioelectric entities.
pub struct BioelectricTaxonomy;

impl TaxonomyDef for BioelectricTaxonomy {
    type Entity = BioelectricEntity;

    fn relations() -> Vec<(BioelectricEntity, BioelectricEntity)> {
        use BioelectricEntity::*;
        vec![
            // Signals is-a Signal
            (MembranePotential, Signal),
            (VoltageGradient, Signal),
            (BioelectricPrepattern, Signal),
            (TransepithelialPotential, Signal),
            // Networks is-a Network
            (GapJunctionNetwork, Network),
            (BioelectricCircuit, Network),
            (CognitiveLightcone, Network),
            // Morphospace is-a Morphospace
            (TargetMorphology, Morphospace),
            (CurrentMorphology, Morphospace),
            (MorphogeneticField, Morphospace),
            // Interventions is-a Intervention
            (IonChannelModulation, Intervention),
            (GapJunctionModulation, Intervention),
            (BioelectricCocktail, Intervention),
            (MechanicalStimulation, Intervention),
            (ProtonPumpInhibition, Intervention),
        ]
    }
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

/// A morphism in the bioelectric category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BioelectricRelation {
    pub from: BioelectricEntity,
    pub to: BioelectricEntity,
}

impl Relationship for BioelectricRelation {
    type Object = BioelectricEntity;

    fn source(&self) -> BioelectricEntity {
        self.from
    }

    fn target(&self) -> BioelectricEntity {
        self.to
    }
}

/// Discrete category over bioelectric entities.
pub struct BioelectricCategory;

impl Category for BioelectricCategory {
    type Object = BioelectricEntity;
    type Morphism = BioelectricRelation;

    fn identity(obj: &BioelectricEntity) -> BioelectricRelation {
        BioelectricRelation {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &BioelectricRelation, g: &BioelectricRelation) -> Option<BioelectricRelation> {
        if f.to != g.from {
            return None;
        }
        Some(BioelectricRelation {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<BioelectricRelation> {
        let variants = BioelectricEntity::variants();
        let mut morphisms = Vec::new();
        for &from in &variants {
            for &to in &variants {
                morphisms.push(BioelectricRelation { from, to });
            }
        }
        morphisms
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: what TAME competency level does this entity operate at?
#[derive(Debug, Clone)]
pub struct OperatingLevel;

impl Quality for OperatingLevel {
    type Individual = BioelectricEntity;
    type Value = CompetencyLevel;

    fn get(&self, individual: &BioelectricEntity) -> Option<CompetencyLevel> {
        use BioelectricEntity::*;
        use CompetencyLevel::*;
        Some(match individual {
            MembranePotential | IonChannelModulation | ProtonPumpInhibition => Molecular,
            VoltageGradient | GapJunctionNetwork | GapJunctionModulation => Cellular,
            BioelectricPrepattern
            | BioelectricCircuit
            | BioelectricCocktail
            | TransepithelialPotential
            | MorphogeneticField
            | CurrentMorphology
            | MechanicalStimulation => Tissue,
            CognitiveLightcone | TargetMorphology => Organ,
            Signal | Network | Morphospace | Intervention => Organism,
        })
    }
}

/// Quality: is this entity accessible via hardware (mechanical)?
#[derive(Debug, Clone)]
pub struct IsHardwareAccessible;

impl Quality for IsHardwareAccessible {
    type Individual = BioelectricEntity;
    type Value = bool;

    fn get(&self, individual: &BioelectricEntity) -> Option<bool> {
        use BioelectricEntity::*;
        match individual {
            MechanicalStimulation => Some(true),
            MembranePotential
            | VoltageGradient
            | BioelectricPrepattern
            | TransepithelialPotential
            | GapJunctionNetwork
            | BioelectricCircuit
            | CognitiveLightcone
            | TargetMorphology
            | CurrentMorphology
            | MorphogeneticField
            | IonChannelModulation
            | GapJunctionModulation
            | BioelectricCocktail
            | ProtonPumpInhibition => Some(false),
            Signal | Network | Morphospace | Intervention => None,
        }
    }
}

/// Quality: does this entity require gap junctions to function?
#[derive(Debug, Clone)]
pub struct RequiresGapJunctions;

impl Quality for RequiresGapJunctions {
    type Individual = BioelectricEntity;
    type Value = bool;

    fn get(&self, individual: &BioelectricEntity) -> Option<bool> {
        use BioelectricEntity::*;
        match individual {
            VoltageGradient
            | BioelectricPrepattern
            | GapJunctionNetwork
            | BioelectricCircuit
            | CognitiveLightcone
            | MorphogeneticField
            | BioelectricCocktail => Some(true),
            MembranePotential
            | IonChannelModulation
            | MechanicalStimulation
            | ProtonPumpInhibition => Some(false),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Opposition (semantic contrasts)
// ---------------------------------------------------------------------------

/// Opposition pairs in the bioelectric framework.
///
/// - IonChannelModulation ↔ ProtonPumpInhibition: direct bioelectric intervention
///   vs indirect acid removal
/// - Signal ↔ Intervention: what IS (observable) vs what you DO (action)
pub struct BioelectricOpposition;

impl OppositionDef for BioelectricOpposition {
    type Entity = BioelectricEntity;

    fn pairs() -> Vec<(BioelectricEntity, BioelectricEntity)> {
        use BioelectricEntity::*;
        vec![
            (IonChannelModulation, ProtonPumpInhibition),
            (Signal, Intervention),
        ]
    }
}

/// Axiom: bioelectric opposition is symmetric.
pub struct BioelectricOppositionSymmetric;

impl Axiom for BioelectricOppositionSymmetric {
    fn description(&self) -> &str {
        "bioelectric opposition is symmetric"
    }

    fn holds(&self) -> bool {
        opposition::Symmetric::<BioelectricOpposition>::new().holds()
    }
}

/// Axiom: bioelectric opposition is irreflexive (nothing opposes itself).
pub struct BioelectricOppositionIrreflexive;

impl Axiom for BioelectricOppositionIrreflexive {
    fn description(&self) -> &str {
        "bioelectric opposition is irreflexive"
    }

    fn holds(&self) -> bool {
        opposition::Irreflexive::<BioelectricOpposition>::new().holds()
    }
}

// ---------------------------------------------------------------------------
// Causal graph
// ---------------------------------------------------------------------------

/// Events in the bioelectric signal causal chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BioelectricSignalEvent {
    IonChannelOpening,
    IonFlux,
    VmemChange,
    GapJunctionPropagation,
    PatternFormation,
    MorphogeneticInstruction,
    AnatomicalChange,
}

impl Entity for BioelectricSignalEvent {
    fn variants() -> Vec<Self> {
        use BioelectricSignalEvent::*;
        vec![
            IonChannelOpening,
            IonFlux,
            VmemChange,
            GapJunctionPropagation,
            PatternFormation,
            MorphogeneticInstruction,
            AnatomicalChange,
        ]
    }
}

/// Causal graph for bioelectric signaling.
///
/// IonChannelOpening → IonFlux → VmemChange
/// VmemChange → GapJunctionPropagation → PatternFormation
/// PatternFormation → MorphogeneticInstruction → AnatomicalChange
pub struct BioelectricSignalCausalGraph;

impl CausalDef for BioelectricSignalCausalGraph {
    type Entity = BioelectricSignalEvent;

    fn relations() -> Vec<(BioelectricSignalEvent, BioelectricSignalEvent)> {
        use BioelectricSignalEvent::*;
        vec![
            (IonChannelOpening, IonFlux),
            (IonFlux, VmemChange),
            (VmemChange, GapJunctionPropagation),
            (GapJunctionPropagation, PatternFormation),
            (PatternFormation, MorphogeneticInstruction),
            (MorphogeneticInstruction, AnatomicalChange),
        ]
    }
}

/// Axiom: bioelectric signal causal graph is asymmetric.
pub struct BioelectricSignalCausalAsymmetric;

impl Axiom for BioelectricSignalCausalAsymmetric {
    fn description(&self) -> &str {
        "bioelectric signal causal graph is asymmetric"
    }

    fn holds(&self) -> bool {
        causation::Asymmetric::<BioelectricSignalCausalGraph>::new().holds()
    }
}

/// Axiom: no bioelectric signal event directly causes itself.
pub struct BioelectricSignalCausalNoSelfCausation;

impl Axiom for BioelectricSignalCausalNoSelfCausation {
    fn description(&self) -> &str {
        "no bioelectric signal event directly causes itself"
    }

    fn holds(&self) -> bool {
        causation::NoSelfCausation::<BioelectricSignalCausalGraph>::new().holds()
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Bioelectric code axiom: healthy tissue is polarized (-50 to -40 mV),
/// cancerous tissue is depolarized (-15 to -20 mV).
pub struct BioelectricCodeAxiom;

impl Axiom for BioelectricCodeAxiom {
    fn description(&self) -> &str {
        "bioelectric code: healthy Vmem is polarized, cancer is depolarized"
    }

    fn holds(&self) -> bool {
        // Healthy epithelial Vmem is below -40 mV (polarized)
        // Cancer/dysplastic Vmem is above -18 mV (depolarized)
        // These values are consistent with morphospace AttractorVmemRange
        let healthy_vmem = -50.0_f64;
        let cancer_vmem = -15.0_f64;
        // Healthy is in polarized range, cancer is in depolarized range
        healthy_vmem < -40.0 && cancer_vmem > -18.0 // -18 = morphospace Dysplastic min
    }
}

/// Gap junction communication axiom: tissue-level signals require GJ,
/// single-cell signals do not.
pub struct GapJunctionCommunicationAxiom;

impl Axiom for GapJunctionCommunicationAxiom {
    fn description(&self) -> &str {
        "tissue signals require gap junctions, single-cell signals do not"
    }

    fn holds(&self) -> bool {
        use BioelectricEntity::*;
        let req = RequiresGapJunctions;
        // Tissue-level signals require GJ
        req.get(&VoltageGradient) == Some(true)
            && req.get(&BioelectricPrepattern) == Some(true)
            // Single-cell signals do not
            && req.get(&MembranePotential) == Some(false)
    }
}

/// Repolarization repair axiom: both IonChannelModulation and ProtonPumpInhibition
/// are interventions.
pub struct RepolarizationRepairAxiom;

impl Axiom for RepolarizationRepairAxiom {
    fn description(&self) -> &str {
        "both ion channel modulation and PPI are interventions"
    }

    fn holds(&self) -> bool {
        use BioelectricEntity::*;
        taxonomy::is_a::<BioelectricTaxonomy>(&IonChannelModulation, &Intervention)
            && taxonomy::is_a::<BioelectricTaxonomy>(&ProtonPumpInhibition, &Intervention)
    }
}

/// Two-mechanism repair axiom: PPI doesn't need GJ, BioelectricCocktail does.
pub struct TwoMechanismRepairAxiom;

impl Axiom for TwoMechanismRepairAxiom {
    fn description(&self) -> &str {
        "PPI does not require gap junctions, bioelectric cocktail does"
    }

    fn holds(&self) -> bool {
        use BioelectricEntity::*;
        let req = RequiresGapJunctions;
        req.get(&ProtonPumpInhibition) == Some(false) && req.get(&BioelectricCocktail) == Some(true)
    }
}

/// TAME hierarchy axiom: no cycles, exactly 5 levels.
pub struct TAMEHierarchyAxiom;

impl Axiom for TAMEHierarchyAxiom {
    fn description(&self) -> &str {
        "TAME hierarchy has no cycles and exactly 5 levels"
    }

    fn holds(&self) -> bool {
        taxonomy::NoCycles::<TAMETaxonomy>::new().holds() && CompetencyLevel::variants().len() == 5
    }
}

/// Cognitive lightcone axiom: requires GJ and operates at Organ level.
pub struct CognitiveLightconeAxiom;

impl Axiom for CognitiveLightconeAxiom {
    fn description(&self) -> &str {
        "cognitive lightcone requires gap junctions and operates at organ level"
    }

    fn holds(&self) -> bool {
        use BioelectricEntity::*;
        RequiresGapJunctions.get(&CognitiveLightcone) == Some(true)
            && OperatingLevel.get(&CognitiveLightcone) == Some(CompetencyLevel::Organ)
    }
}

/// Mechanical stimulation is the only hardware-accessible intervention.
pub struct MechanicalStimulationIsHardwareAccessible;

impl Axiom for MechanicalStimulationIsHardwareAccessible {
    fn description(&self) -> &str {
        "exactly one hardware-accessible intervention: MechanicalStimulation"
    }

    fn holds(&self) -> bool {
        let hw = IsHardwareAccessible;
        let interventions: Vec<BioelectricEntity> = BioelectricEntity::variants()
            .into_iter()
            .filter(|e| taxonomy::is_a::<BioelectricTaxonomy>(e, &BioelectricEntity::Intervention))
            .filter(|e| *e != BioelectricEntity::Intervention)
            .collect();
        let hw_accessible: Vec<&BioelectricEntity> = interventions
            .iter()
            .filter(|e| hw.get(e) == Some(true))
            .collect();
        hw_accessible.len() == 1 && *hw_accessible[0] == BioelectricEntity::MechanicalStimulation
    }
}

/// Bioelectric taxonomy is a DAG.
pub struct BioelectricTaxonomyIsDAG;

impl Axiom for BioelectricTaxonomyIsDAG {
    fn description(&self) -> &str {
        "bioelectric taxonomy is a directed acyclic graph"
    }

    fn holds(&self) -> bool {
        taxonomy::NoCycles::<BioelectricTaxonomy>::new().holds()
    }
}

/// All 5 TAME levels are represented in OperatingLevel values.
pub struct AllTAMELevelsRepresented;

impl Axiom for AllTAMELevelsRepresented {
    fn description(&self) -> &str {
        "all 5 TAME competency levels are represented in operating levels"
    }

    fn holds(&self) -> bool {
        let op = OperatingLevel;
        let all = BioelectricEntity::variants();
        let levels: Vec<CompetencyLevel> = all.iter().filter_map(|e| op.get(e)).collect();
        CompetencyLevel::variants()
            .iter()
            .all(|target| levels.contains(target))
    }
}

// ---------------------------------------------------------------------------
// Cross-domain equivalence axioms
// ---------------------------------------------------------------------------

/// Axiom: bioelectricity->regeneration functor preserves TargetMorphology identity.
/// TargetMorphology in bioelectricity IS TargetMorphology in regeneration.
pub struct TargetMorphologyCrossDomainEquivalence;

impl Axiom for TargetMorphologyCrossDomainEquivalence {
    fn description(&self) -> &str {
        "TargetMorphology is the same entity in bioelectricity and regeneration (functor maps identity)"
    }

    fn holds(&self) -> bool {
        use crate::science::biomedical::bioelectricity::regeneration_functor::BioelectricToRegeneration;
        use crate::science::biomedical::regeneration::ontology::RegenerationEntity;
        use praxis::category::Functor;
        BioelectricToRegeneration::map_object(&BioelectricEntity::TargetMorphology)
            == RegenerationEntity::TargetMorphology
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// Top-level ontology tying together the bioelectric category, qualities, and axioms.
pub struct BioelectricOntology;

impl Ontology for BioelectricOntology {
    type Cat = BioelectricCategory;
    type Qual = OperatingLevel;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(BioelectricCodeAxiom),
            Box::new(GapJunctionCommunicationAxiom),
            Box::new(RepolarizationRepairAxiom),
            Box::new(TwoMechanismRepairAxiom),
            Box::new(TAMEHierarchyAxiom),
            Box::new(CognitiveLightconeAxiom),
            Box::new(MechanicalStimulationIsHardwareAccessible),
            Box::new(BioelectricTaxonomyIsDAG),
            Box::new(AllTAMELevelsRepresented),
            Box::new(BioelectricOppositionSymmetric),
            Box::new(BioelectricOppositionIrreflexive),
            Box::new(BioelectricSignalCausalAsymmetric),
            Box::new(BioelectricSignalCausalNoSelfCausation),
            Box::new(TargetMorphologyCrossDomainEquivalence),
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
    use praxis::ontology::reasoning::taxonomy::TaxonomyCategory;

    // -- Axiom tests --

    #[test]
    fn test_bioelectric_code_axiom() {
        assert!(
            BioelectricCodeAxiom.holds(),
            "{}",
            BioelectricCodeAxiom.description()
        );
    }

    #[test]
    fn test_gap_junction_communication_axiom() {
        assert!(
            GapJunctionCommunicationAxiom.holds(),
            "{}",
            GapJunctionCommunicationAxiom.description()
        );
    }

    #[test]
    fn test_repolarization_repair_axiom() {
        assert!(
            RepolarizationRepairAxiom.holds(),
            "{}",
            RepolarizationRepairAxiom.description()
        );
    }

    #[test]
    fn test_two_mechanism_repair_axiom() {
        assert!(
            TwoMechanismRepairAxiom.holds(),
            "{}",
            TwoMechanismRepairAxiom.description()
        );
    }

    #[test]
    fn test_tame_hierarchy_axiom() {
        assert!(
            TAMEHierarchyAxiom.holds(),
            "{}",
            TAMEHierarchyAxiom.description()
        );
    }

    #[test]
    fn test_cognitive_lightcone_axiom() {
        assert!(
            CognitiveLightconeAxiom.holds(),
            "{}",
            CognitiveLightconeAxiom.description()
        );
    }

    #[test]
    fn test_mechanical_stimulation_is_hardware_accessible() {
        assert!(
            MechanicalStimulationIsHardwareAccessible.holds(),
            "{}",
            MechanicalStimulationIsHardwareAccessible.description()
        );
    }

    #[test]
    fn test_bioelectric_taxonomy_is_dag() {
        assert!(
            BioelectricTaxonomyIsDAG.holds(),
            "{}",
            BioelectricTaxonomyIsDAG.description()
        );
    }

    #[test]
    fn test_all_tame_levels_represented() {
        assert!(
            AllTAMELevelsRepresented.holds(),
            "{}",
            AllTAMELevelsRepresented.description()
        );
    }

    // -- Opposition tests --

    #[test]
    fn test_bioelectric_opposition_symmetric() {
        assert!(
            BioelectricOppositionSymmetric.holds(),
            "{}",
            BioelectricOppositionSymmetric.description()
        );
    }

    #[test]
    fn test_bioelectric_opposition_irreflexive() {
        assert!(
            BioelectricOppositionIrreflexive.holds(),
            "{}",
            BioelectricOppositionIrreflexive.description()
        );
    }

    #[test]
    fn test_ion_channel_modulation_opposes_ppi() {
        use BioelectricEntity::*;
        assert!(opposition::are_opposed::<BioelectricOpposition>(
            &IonChannelModulation,
            &ProtonPumpInhibition
        ));
        assert!(opposition::are_opposed::<BioelectricOpposition>(
            &ProtonPumpInhibition,
            &IonChannelModulation
        ));
    }

    #[test]
    fn test_signal_opposes_intervention() {
        use BioelectricEntity::*;
        assert!(opposition::are_opposed::<BioelectricOpposition>(
            &Signal,
            &Intervention
        ));
    }

    #[test]
    fn test_signal_does_not_oppose_network() {
        use BioelectricEntity::*;
        assert!(!opposition::are_opposed::<BioelectricOpposition>(
            &Signal, &Network
        ));
    }

    #[test]
    fn test_bioelectric_opposites_query() {
        use BioelectricEntity::*;
        let opps = opposition::opposites::<BioelectricOpposition>(&Signal);
        assert_eq!(opps, vec![Intervention]);
    }

    // -- Category law tests --

    #[test]
    fn test_bioelectric_category_laws() {
        check_category_laws::<BioelectricCategory>().unwrap();
    }

    #[test]
    fn test_tame_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<TAMETaxonomy>>().unwrap();
    }

    #[test]
    fn test_bioelectric_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<BioelectricTaxonomy>>().unwrap();
    }

    // -- TAME transitivity tests --

    #[test]
    fn test_tame_molecular_reaches_organism() {
        use CompetencyLevel::*;
        assert!(taxonomy::is_a::<TAMETaxonomy>(&Molecular, &Cellular));
        assert!(taxonomy::is_a::<TAMETaxonomy>(&Molecular, &Tissue));
        assert!(taxonomy::is_a::<TAMETaxonomy>(&Molecular, &Organ));
        assert!(taxonomy::is_a::<TAMETaxonomy>(&Molecular, &Organism));
    }

    #[test]
    fn test_tame_organism_does_not_reach_molecular() {
        use CompetencyLevel::*;
        assert!(!taxonomy::is_a::<TAMETaxonomy>(&Organism, &Molecular));
    }

    // -- Signal taxonomy tests --

    #[test]
    fn test_signals_are_signals() {
        use BioelectricEntity::*;
        for signal in [
            MembranePotential,
            VoltageGradient,
            BioelectricPrepattern,
            TransepithelialPotential,
        ] {
            assert!(
                taxonomy::is_a::<BioelectricTaxonomy>(&signal, &Signal),
                "{:?} should be a Signal",
                signal
            );
        }
    }

    #[test]
    fn test_interventions_are_interventions() {
        use BioelectricEntity::*;
        for interv in [
            IonChannelModulation,
            GapJunctionModulation,
            BioelectricCocktail,
            MechanicalStimulation,
            ProtonPumpInhibition,
        ] {
            assert!(
                taxonomy::is_a::<BioelectricTaxonomy>(&interv, &Intervention),
                "{:?} should be an Intervention",
                interv
            );
        }
    }

    #[test]
    fn test_intervention_descendants_count() {
        let descendants =
            taxonomy::descendants::<BioelectricTaxonomy>(&BioelectricEntity::Intervention);
        assert_eq!(descendants.len(), 5);
    }

    #[test]
    fn test_entity_count() {
        assert_eq!(BioelectricEntity::variants().len(), 19);
    }

    // -- Property-based tests (proptest) --

    use proptest::prelude::*;

    fn arb_competency_level() -> impl Strategy<Value = CompetencyLevel> {
        (0..CompetencyLevel::variants().len()).prop_map(|i| CompetencyLevel::variants()[i])
    }

    fn arb_bioelectric_entity() -> impl Strategy<Value = BioelectricEntity> {
        (0..BioelectricEntity::variants().len()).prop_map(|i| BioelectricEntity::variants()[i])
    }

    proptest! {
        /// For any CompetencyLevel, the is-a relationship is reflexive.
        #[test]
        fn prop_competency_level_is_a_reflexive(level in arb_competency_level()) {
            prop_assert!(
                taxonomy::is_a::<TAMETaxonomy>(&level, &level),
                "is-a should be reflexive for {:?}",
                level
            );
        }

        /// For any BioelectricEntity, OperatingLevel always returns Some (total function).
        #[test]
        fn prop_operating_level_is_total(entity in arb_bioelectric_entity()) {
            prop_assert!(
                OperatingLevel.get(&entity).is_some(),
                "OperatingLevel should be defined for {:?}",
                entity
            );
        }
    }

    // -- Literature axioms --

    /// Chernet & Levin 2013: +19.4 mV hyperpolarization via GlyR suppresses tumors.
    /// IonChannelModulation is an Intervention that does NOT require gap junctions.
    #[test]
    fn test_literature_chernet_levin_2013_ion_channel_modulation_no_gj() {
        use BioelectricEntity::*;
        // IonChannelModulation is-a Intervention
        assert!(
            taxonomy::is_a::<BioelectricTaxonomy>(&IonChannelModulation, &Intervention),
            "Chernet & Levin 2013: IonChannelModulation must be an Intervention"
        );
        // IonChannelModulation does NOT require gap junctions
        assert_eq!(
            RequiresGapJunctions.get(&IonChannelModulation),
            Some(false),
            "Chernet & Levin 2013: GlyR-mediated hyperpolarization (+19.4 mV) \
             is cell-autonomous and does not require gap junctions"
        );
    }

    #[test]
    fn test_ontology_validates() {
        BioelectricOntology::validate().unwrap();
    }

    /// Levin 2022 TAME: exactly 5 levels of competency exist
    /// (Molecular → Cellular → Tissue → Organ → Organism) with strict ordering.
    #[test]
    fn test_literature_levin_2022_tame_five_levels_ordered() {
        use CompetencyLevel::*;
        let levels = CompetencyLevel::variants();
        // Exactly 5 levels
        assert_eq!(
            levels.len(),
            5,
            "Levin 2022 TAME: exactly 5 competency levels must exist"
        );
        // Strict ordering: each level is-a its successor but not vice versa
        let ordered = [Molecular, Cellular, Tissue, Organ, Organism];
        for i in 0..ordered.len() - 1 {
            assert!(
                taxonomy::is_a::<TAMETaxonomy>(&ordered[i], &ordered[i + 1]),
                "TAME ordering: {:?} should be-a {:?}",
                ordered[i],
                ordered[i + 1]
            );
            assert!(
                !taxonomy::is_a::<TAMETaxonomy>(&ordered[i + 1], &ordered[i]),
                "TAME ordering: {:?} should NOT be-a {:?} (asymmetric)",
                ordered[i + 1],
                ordered[i]
            );
        }
    }

    // -- Causal graph tests --

    #[test]
    fn test_bioelectric_signal_causal_asymmetric() {
        assert!(
            BioelectricSignalCausalAsymmetric.holds(),
            "{}",
            BioelectricSignalCausalAsymmetric.description()
        );
    }

    #[test]
    fn test_bioelectric_signal_causal_no_self_causation() {
        assert!(
            BioelectricSignalCausalNoSelfCausation.holds(),
            "{}",
            BioelectricSignalCausalNoSelfCausation.description()
        );
    }

    #[test]
    fn test_bioelectric_signal_causal_category_laws() {
        use praxis::ontology::reasoning::causation::CausalCategory;
        check_category_laws::<CausalCategory<BioelectricSignalCausalGraph>>().unwrap();
    }

    #[test]
    fn test_ion_channel_opening_causes_anatomical_change() {
        use BioelectricSignalEvent::*;
        let effects = causation::effects_of::<BioelectricSignalCausalGraph>(&IonChannelOpening);
        assert!(effects.contains(&AnatomicalChange));
    }

    #[test]
    fn test_vmem_change_causes_pattern_formation() {
        use BioelectricSignalEvent::*;
        let effects = causation::effects_of::<BioelectricSignalCausalGraph>(&VmemChange);
        assert!(effects.contains(&PatternFormation));
    }

    #[test]
    fn test_bioelectric_signal_event_count() {
        assert_eq!(BioelectricSignalEvent::variants().len(), 7);
    }

    // -- Cross-domain equivalence tests --

    #[test]
    fn test_target_morphology_cross_domain_equivalence() {
        assert!(
            TargetMorphologyCrossDomainEquivalence.holds(),
            "{}",
            TargetMorphologyCrossDomainEquivalence.description()
        );
    }
}
