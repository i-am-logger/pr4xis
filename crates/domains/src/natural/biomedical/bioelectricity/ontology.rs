//! Core Levin bioelectric framework ontology.
//!
//! Models Dr. Michael Levin's TAME (Technological Approach to Mind Everywhere)
//! framework as formal ontology:
//! - Bioelectric code: Vmem patterns encode morphogenetic information
//! - Gap junction networks: signal propagation topology
//! - Cognitive lightcone: scale of goal-directed agency
//!
//! The TAME competency hierarchy (Molecular → Organism) and the bioelectric
//! signal causal chain (IonChannelOpening → AnatomicalChange) live in sibling
//! modules (`tame`, `event`) to keep each ontology focused on one kind of
//! concept. See #118 for the dual-enum cleanup.
//!
//! Key references:
//! - Levin 2019: The Computational Boundary of a "Self"
//! - Fields & Levin 2022: Competency in Navigating Arbitrary Spaces
//! - Levin 2014: Molecular bioelectricity in developmental biology

use pr4xis::ontology::reasoning::opposition;
use pr4xis::ontology::reasoning::taxonomy;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pub use crate::natural::biomedical::bioelectricity::event::{
    BioelectricSignalCausalGraph, BioelectricSignalEvent,
};
pub use crate::natural::biomedical::bioelectricity::tame::{CompetencyLevel, TAMETaxonomy};

pr4xis::ontology! {
    name: "Bioelectric",
    source: "Levin (2019); Fields & Levin (2022)",
    being: AbstractObject,

    concepts: [
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
        // Abstract categories
        Signal,
        Network,
        Morphospace,
        Intervention,
    ],

    labels: {
        MembranePotential: ("en", "Membrane potential", "Single-cell membrane potential (Vmem). Difference in electric potential across the plasma membrane."),
        VoltageGradient: ("en", "Voltage gradient", "Spatial gradient of Vmem across a tissue."),
        BioelectricPrepattern: ("en", "Bioelectric prepattern", "Vmem pattern that prefigures anatomical structure before gene expression."),
        TransepithelialPotential: ("en", "Transepithelial potential", "Vmem across an epithelial layer; drives wound-healing currents."),
        GapJunctionNetwork: ("en", "Gap junction network", "Network of cells connected by connexin channels."),
        BioelectricCircuit: ("en", "Bioelectric circuit", "Functional pattern of electric coupling across cells."),
        CognitiveLightcone: ("en", "Cognitive lightcone", "Spatiotemporal extent of goal-directed agency in a bioelectric system."),
        TargetMorphology: ("en", "Target morphology", "Goal state in morphospace — the anatomical pattern tissue navigates toward."),
        CurrentMorphology: ("en", "Current morphology", "Present anatomical state of tissue in morphospace."),
        MorphogeneticField: ("en", "Morphogenetic field", "Field of bioelectric influence guiding tissue toward the target."),
        IonChannelModulation: ("en", "Ion channel modulation", "Intervention that changes Vmem by opening/closing ion channels. Cell-autonomous — does not require gap junctions."),
        GapJunctionModulation: ("en", "Gap junction modulation", "Intervention that changes cell coupling by gating connexins."),
        BioelectricCocktail: ("en", "Bioelectric cocktail", "Combined intervention targeting multiple ion channels simultaneously."),
        MechanicalStimulation: ("en", "Mechanical stimulation", "Hardware-accessible intervention using physical force."),
        ProtonPumpInhibition: ("en", "Proton pump inhibition", "Intervention targeting V-ATPase proton pumps."),
        Signal: ("en", "Signal (abstract)", "Abstract bioelectric signal — root of the Signal taxonomy."),
        Network: ("en", "Network (abstract)", "Abstract bioelectric network — root of the Network taxonomy."),
        Morphospace: ("en", "Morphospace (abstract)", "Abstract morphospace concept — root of the Morphospace taxonomy."),
        Intervention: ("en", "Intervention (abstract)", "Abstract intervention — root of the Intervention taxonomy."),
    },

    is_a: [
        (MembranePotential, Signal),
        (VoltageGradient, Signal),
        (BioelectricPrepattern, Signal),
        (TransepithelialPotential, Signal),
        (GapJunctionNetwork, Network),
        (BioelectricCircuit, Network),
        (CognitiveLightcone, Network),
        (TargetMorphology, Morphospace),
        (CurrentMorphology, Morphospace),
        (MorphogeneticField, Morphospace),
        (IonChannelModulation, Intervention),
        (GapJunctionModulation, Intervention),
        (BioelectricCocktail, Intervention),
        (MechanicalStimulation, Intervention),
        (ProtonPumpInhibition, Intervention),
    ],

    opposes: [
        (IonChannelModulation, ProtonPumpInhibition),
        (Signal, Intervention),
    ],
}

// Backward-compatibility re-export so existing call sites (Quality impls,
// axioms, functors) that reference `BioelectricEntity` keep compiling AND
// support glob imports (type aliases don't).
pub use BioelectricConcept as BioelectricEntity;

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
        use BioelectricConcept::*;
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
        use BioelectricConcept::*;
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
        use BioelectricConcept::*;
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
// Axioms
// ---------------------------------------------------------------------------

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
pr4xis::register_axiom!(BioelectricOppositionSymmetric);

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
pr4xis::register_axiom!(BioelectricOppositionIrreflexive);

/// Bioelectric code axiom: healthy tissue is polarized (-50 to -40 mV),
/// cancerous tissue is depolarized (-15 to -20 mV).
pub struct BioelectricCodeAxiom;

impl Axiom for BioelectricCodeAxiom {
    fn description(&self) -> &str {
        "bioelectric code: healthy Vmem is polarized, cancer is depolarized"
    }

    fn holds(&self) -> bool {
        let healthy_vmem = -50.0_f64;
        let cancer_vmem = -15.0_f64;
        healthy_vmem < -40.0 && cancer_vmem > -18.0
    }
}
pr4xis::register_axiom!(BioelectricCodeAxiom);

/// Gap junction communication axiom: tissue-level signals require GJ,
/// single-cell signals do not.
pub struct GapJunctionCommunicationAxiom;

impl Axiom for GapJunctionCommunicationAxiom {
    fn description(&self) -> &str {
        "tissue signals require gap junctions, single-cell signals do not"
    }

    fn holds(&self) -> bool {
        use BioelectricConcept::*;
        let req = RequiresGapJunctions;
        req.get(&VoltageGradient) == Some(true)
            && req.get(&BioelectricPrepattern) == Some(true)
            && req.get(&MembranePotential) == Some(false)
    }
}
pr4xis::register_axiom!(GapJunctionCommunicationAxiom);

/// Repolarization repair axiom: both IonChannelModulation and ProtonPumpInhibition
/// are interventions.
pub struct RepolarizationRepairAxiom;

impl Axiom for RepolarizationRepairAxiom {
    fn description(&self) -> &str {
        "both ion channel modulation and PPI are interventions"
    }

    fn holds(&self) -> bool {
        use BioelectricConcept::*;
        taxonomy::is_a::<BioelectricTaxonomy>(&IonChannelModulation, &Intervention)
            && taxonomy::is_a::<BioelectricTaxonomy>(&ProtonPumpInhibition, &Intervention)
    }
}
pr4xis::register_axiom!(RepolarizationRepairAxiom);

/// Two-mechanism repair axiom: PPI doesn't need GJ, BioelectricCocktail does.
pub struct TwoMechanismRepairAxiom;

impl Axiom for TwoMechanismRepairAxiom {
    fn description(&self) -> &str {
        "PPI does not require gap junctions, bioelectric cocktail does"
    }

    fn holds(&self) -> bool {
        use BioelectricConcept::*;
        let req = RequiresGapJunctions;
        req.get(&ProtonPumpInhibition) == Some(false) && req.get(&BioelectricCocktail) == Some(true)
    }
}
pr4xis::register_axiom!(TwoMechanismRepairAxiom);

/// TAME hierarchy axiom: exactly 5 levels.
pub struct TAMEHierarchyAxiom;

impl Axiom for TAMEHierarchyAxiom {
    fn description(&self) -> &str {
        "TAME hierarchy has no cycles and exactly 5 levels"
    }

    fn holds(&self) -> bool {
        use pr4xis::category::Entity;
        taxonomy::NoCycles::<TAMETaxonomy>::new().holds() && CompetencyLevel::variants().len() == 5
    }
}
pr4xis::register_axiom!(TAMEHierarchyAxiom);

/// Cognitive lightcone axiom: requires GJ and operates at Organ level.
pub struct CognitiveLightconeAxiom;

impl Axiom for CognitiveLightconeAxiom {
    fn description(&self) -> &str {
        "cognitive lightcone requires gap junctions and operates at organ level"
    }

    fn holds(&self) -> bool {
        use BioelectricConcept::*;
        RequiresGapJunctions.get(&CognitiveLightcone) == Some(true)
            && OperatingLevel.get(&CognitiveLightcone) == Some(CompetencyLevel::Organ)
    }
}
pr4xis::register_axiom!(CognitiveLightconeAxiom);

/// Mechanical stimulation is the only hardware-accessible intervention.
pub struct MechanicalStimulationIsHardwareAccessible;

impl Axiom for MechanicalStimulationIsHardwareAccessible {
    fn description(&self) -> &str {
        "exactly one hardware-accessible intervention: MechanicalStimulation"
    }

    fn holds(&self) -> bool {
        use pr4xis::category::Entity;
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
pr4xis::register_axiom!(MechanicalStimulationIsHardwareAccessible);

/// All 5 TAME levels are represented in OperatingLevel values.
pub struct AllTAMELevelsRepresented;

impl Axiom for AllTAMELevelsRepresented {
    fn description(&self) -> &str {
        "all 5 TAME competency levels are represented in operating levels"
    }

    fn holds(&self) -> bool {
        use pr4xis::category::Entity;
        let op = OperatingLevel;
        let all = BioelectricEntity::variants();
        let levels: Vec<CompetencyLevel> = all.iter().filter_map(|e| op.get(e)).collect();
        CompetencyLevel::variants()
            .iter()
            .all(|target| levels.contains(target))
    }
}
pr4xis::register_axiom!(AllTAMELevelsRepresented);

/// Axiom: bioelectricity->regeneration functor preserves TargetMorphology identity.
pub struct TargetMorphologyCrossDomainEquivalence;

impl Axiom for TargetMorphologyCrossDomainEquivalence {
    fn description(&self) -> &str {
        "TargetMorphology is the same entity in bioelectricity and regeneration (functor maps identity)"
    }

    fn holds(&self) -> bool {
        use crate::natural::biomedical::bioelectricity::regeneration_functor::BioelectricToRegeneration;
        use crate::natural::biomedical::regeneration::ontology::RegenerationEntity;
        use pr4xis::category::Functor;
        BioelectricToRegeneration::map_object(&BioelectricEntity::TargetMorphology)
            == RegenerationEntity::TargetMorphology
    }
}
pr4xis::register_axiom!(TargetMorphologyCrossDomainEquivalence);

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

impl Ontology for BioelectricOntology {
    type Cat = BioelectricCategory;
    type Qual = OperatingLevel;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(BioelectricCodeAxiom),
            Box::new(GapJunctionCommunicationAxiom),
            Box::new(RepolarizationRepairAxiom),
            Box::new(TwoMechanismRepairAxiom),
            Box::new(TAMEHierarchyAxiom),
            Box::new(CognitiveLightconeAxiom),
            Box::new(MechanicalStimulationIsHardwareAccessible),
            Box::new(AllTAMELevelsRepresented),
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
    use crate::natural::biomedical::bioelectricity::event::{CausalAsymmetric, NoSelfCausation};
    use pr4xis::category::Entity;
    use pr4xis::category::validate::check_category_laws;
    use pr4xis::ontology::reasoning::causation;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;

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
    fn test_all_tame_levels_represented() {
        assert!(
            AllTAMELevelsRepresented.holds(),
            "{}",
            AllTAMELevelsRepresented.description()
        );
    }

    #[test]
    fn test_bioelectric_opposition_symmetric() {
        assert!(BioelectricOppositionSymmetric.holds());
    }

    #[test]
    fn test_bioelectric_opposition_irreflexive() {
        assert!(BioelectricOppositionIrreflexive.holds());
    }

    #[test]
    fn test_ion_channel_modulation_opposes_ppi() {
        use BioelectricConcept::*;
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
        use BioelectricConcept::*;
        assert!(opposition::are_opposed::<BioelectricOpposition>(
            &Signal,
            &Intervention
        ));
    }

    #[test]
    fn test_signal_does_not_oppose_network() {
        use BioelectricConcept::*;
        assert!(!opposition::are_opposed::<BioelectricOpposition>(
            &Signal, &Network
        ));
    }

    #[test]
    fn test_bioelectric_opposites_query() {
        use BioelectricConcept::*;
        let opps = opposition::opposites::<BioelectricOpposition>(&Signal);
        assert_eq!(opps, vec![Intervention]);
    }

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

    #[test]
    fn test_signals_are_signals() {
        use BioelectricConcept::*;
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
        use BioelectricConcept::*;
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

    use proptest::prelude::*;

    fn arb_competency_level() -> impl Strategy<Value = CompetencyLevel> {
        (0..CompetencyLevel::variants().len()).prop_map(|i| CompetencyLevel::variants()[i])
    }

    fn arb_bioelectric_entity() -> impl Strategy<Value = BioelectricEntity> {
        (0..BioelectricEntity::variants().len()).prop_map(|i| BioelectricEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_competency_level_is_a_reflexive(level in arb_competency_level()) {
            prop_assert!(
                taxonomy::is_a::<TAMETaxonomy>(&level, &level),
                "is-a should be reflexive for {:?}",
                level
            );
        }

        #[test]
        fn prop_operating_level_is_total(entity in arb_bioelectric_entity()) {
            prop_assert!(
                OperatingLevel.get(&entity).is_some(),
                "OperatingLevel should be defined for {:?}",
                entity
            );
        }
    }

    #[test]
    fn test_literature_chernet_levin_2013_ion_channel_modulation_no_gj() {
        use BioelectricConcept::*;
        assert!(
            taxonomy::is_a::<BioelectricTaxonomy>(&IonChannelModulation, &Intervention),
            "Chernet & Levin 2013: IonChannelModulation must be an Intervention"
        );
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

    #[test]
    fn test_literature_levin_2022_tame_five_levels_ordered() {
        use CompetencyLevel::*;
        let levels = CompetencyLevel::variants();
        assert_eq!(
            levels.len(),
            5,
            "Levin 2022 TAME: exactly 5 competency levels must exist"
        );
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

    #[test]
    fn test_bioelectric_signal_causal_asymmetric() {
        assert!(CausalAsymmetric.holds());
    }

    #[test]
    fn test_bioelectric_signal_causal_no_self_causation() {
        assert!(NoSelfCausation.holds());
    }

    #[test]
    fn test_bioelectric_signal_causal_category_laws() {
        use pr4xis::ontology::reasoning::causation::CausalCategory;
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

    #[test]
    fn test_target_morphology_cross_domain_equivalence() {
        assert!(
            TargetMorphologyCrossDomainEquivalence.holds(),
            "{}",
            TargetMorphologyCrossDomainEquivalence.description()
        );
    }
}
