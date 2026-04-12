//! Morphospace attractor landscape ontology.
//!
//! Models the esophageal tissue morphospace as a set of attractor states
//! with disease progression and repair pathways. Pure ontology -- no simulation.
//!
//! Key concepts:
//! - Attractor states: stable tissue configurations (healthy, inflamed, Barrett's, etc.)
//! - Disease progression: causal chains from acid damage to dysplasia
//! - Repair pathways: bioelectric and mechanical interventions
//! - Vmem ranges: membrane potential signatures for each attractor
//!
//! References:
//! - Fields & Levin 2022: Competency in Navigating Arbitrary Spaces
//! - Chernet & Levin 2013: Vmem manipulation suppresses tumors

use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::causation;
use pr4xis::ontology::reasoning::opposition;
use pr4xis::ontology::reasoning::taxonomy;
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

/// Every entity in the morphospace ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum MorphospaceEntity {
    // Attractors
    Healthy,
    Inflamed,
    Barretts,
    Dysplastic,
    Fibrotic,
    // Repair pathways
    BasalTurnover,
    BioelectricRepair,
    MechanicalStimulation,
    CombinedTherapy,
    // Properties
    PolarizedVmem,
    DepolarizedVmem,
    ConnectedNetwork,
    DisconnectedNetwork,
    // Abstract
    Attractor,
    RepairPathway,
    BioelectricState,
}

// ---------------------------------------------------------------------------
// Causal event
// ---------------------------------------------------------------------------

/// Events in disease progression and repair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum MorphospaceEvent {
    // Disease
    AcidDamage,
    ChronicInflammation,
    GapJunctionLoss,
    MetaplasticTransition,
    DysplasticTransition,
    FibroticRemodeling,
    // Repair
    AcidRemoval,
    BasalCellReplacement,
    VmemRepolarization,
    GapJunctionRestoration,
    PatternRecognition,
    MechanotransductionActivation,
    AutonomousRepair,
}

// ---------------------------------------------------------------------------
// Category + Reasoning (generated)
// ---------------------------------------------------------------------------

define_ontology! {
    /// Morphospace attractor landscape ontology.
    pub MorphospaceOntologyMeta for MorphospaceCategory {
        entity: MorphospaceEntity,
        relation: MorphospaceRelation,

        taxonomy: MorphospaceTaxonomy [
            (Healthy, Attractor),
            (Inflamed, Attractor),
            (Barretts, Attractor),
            (Dysplastic, Attractor),
            (Fibrotic, Attractor),
            (BasalTurnover, RepairPathway),
            (BioelectricRepair, RepairPathway),
            (MechanicalStimulation, RepairPathway),
            (CombinedTherapy, RepairPathway),
            (PolarizedVmem, BioelectricState),
            (DepolarizedVmem, BioelectricState),
            (ConnectedNetwork, BioelectricState),
            (DisconnectedNetwork, BioelectricState),
        ],

        causation: DiseaseProgressionCauses for MorphospaceEvent [
            (AcidDamage, ChronicInflammation),
            (ChronicInflammation, GapJunctionLoss),
            (GapJunctionLoss, FibroticRemodeling),
            (ChronicInflammation, MetaplasticTransition),
            (MetaplasticTransition, DysplasticTransition),
            (AcidRemoval, BasalCellReplacement),
            (BasalCellReplacement, AutonomousRepair),
            (VmemRepolarization, PatternRecognition),
            (PatternRecognition, AutonomousRepair),
            (GapJunctionRestoration, PatternRecognition),
            (MechanotransductionActivation, VmemRepolarization),
            (MechanotransductionActivation, GapJunctionRestoration),
        ],

        opposition: MorphospaceOpposition [
            (Healthy, Dysplastic),
            (PolarizedVmem, DepolarizedVmem),
            (ConnectedNetwork, DisconnectedNetwork),
            (BasalTurnover, BioelectricRepair),
        ],
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Membrane potential range for an attractor state.
#[derive(Debug, Clone, PartialEq)]
pub struct VmemRange {
    pub min: f64,
    pub max: f64,
}

/// Quality: Vmem range for each attractor state.
#[derive(Debug, Clone)]
pub struct AttractorVmemRange;

impl Quality for AttractorVmemRange {
    type Individual = MorphospaceEntity;
    type Value = VmemRange;

    fn get(&self, individual: &MorphospaceEntity) -> Option<VmemRange> {
        use MorphospaceEntity::*;
        match individual {
            Healthy => Some(VmemRange {
                min: -70.0,
                max: -40.0,
            }),
            Inflamed => Some(VmemRange {
                min: -40.0,
                max: -28.0,
            }),
            Barretts => Some(VmemRange {
                min: -28.0,
                max: -18.0,
            }),
            Dysplastic => Some(VmemRange {
                min: -18.0,
                max: 0.0,
            }),
            Fibrotic => Some(VmemRange {
                min: -35.0,
                max: -20.0,
            }),
            _ => None,
        }
    }
}

/// Quality: disease severity (0 = healthy, higher = worse).
#[derive(Debug, Clone)]
pub struct DiseaseSeverity;

impl Quality for DiseaseSeverity {
    type Individual = MorphospaceEntity;
    type Value = u32;

    fn get(&self, individual: &MorphospaceEntity) -> Option<u32> {
        use MorphospaceEntity::*;
        match individual {
            Healthy => Some(0),
            Inflamed => Some(1),
            Barretts => Some(2),
            Fibrotic => Some(2),
            Dysplastic => Some(3),
            _ => None,
        }
    }
}

/// Quality: does this repair pathway require gap junctions?
#[derive(Debug, Clone)]
pub struct PathwayRequiresGJ;

impl Quality for PathwayRequiresGJ {
    type Individual = MorphospaceEntity;
    type Value = bool;

    fn get(&self, individual: &MorphospaceEntity) -> Option<bool> {
        use MorphospaceEntity::*;
        match individual {
            BasalTurnover => Some(false),
            BioelectricRepair => Some(true),
            MechanicalStimulation => Some(false),
            CombinedTherapy => Some(true),
            _ => None,
        }
    }
}

/// Quality: is this repair pathway hardware-accessible?
#[derive(Debug, Clone)]
pub struct PathwayIsHardwareAccessible;

impl Quality for PathwayIsHardwareAccessible {
    type Individual = MorphospaceEntity;
    type Value = bool;

    fn get(&self, individual: &MorphospaceEntity) -> Option<bool> {
        use MorphospaceEntity::*;
        match individual {
            BasalTurnover => Some(false),
            BioelectricRepair => Some(false),
            MechanicalStimulation => Some(true),
            CombinedTherapy => Some(false),
            _ => None,
        }
    }
}

/// Axiom: morphospace opposition is symmetric.
pub struct MorphospaceOppositionSymmetric;

impl Axiom for MorphospaceOppositionSymmetric {
    fn description(&self) -> &str {
        "morphospace opposition is symmetric"
    }

    fn holds(&self) -> bool {
        opposition::Symmetric::<MorphospaceOpposition>::new().holds()
    }
}

/// Axiom: morphospace opposition is irreflexive (nothing opposes itself).
pub struct MorphospaceOppositionIrreflexive;

impl Axiom for MorphospaceOppositionIrreflexive {
    fn description(&self) -> &str {
        "morphospace opposition is irreflexive"
    }

    fn holds(&self) -> bool {
        opposition::Irreflexive::<MorphospaceOpposition>::new().holds()
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Morphospace taxonomy is a DAG.
pub struct MorphospaceTaxonomyIsDAG;

impl Axiom for MorphospaceTaxonomyIsDAG {
    fn description(&self) -> &str {
        "morphospace taxonomy is a directed acyclic graph"
    }

    fn holds(&self) -> bool {
        taxonomy::NoCycles::<MorphospaceTaxonomy>::new().holds()
    }
}

/// All attractor states have Vmem ranges.
pub struct AllAttractorsHaveVmemRanges;

impl Axiom for AllAttractorsHaveVmemRanges {
    fn description(&self) -> &str {
        "all attractor states have Vmem ranges"
    }

    fn holds(&self) -> bool {
        use MorphospaceEntity::*;
        let vmem = AttractorVmemRange;
        [Healthy, Inflamed, Barretts, Dysplastic, Fibrotic]
            .iter()
            .all(|a| vmem.get(a).is_some())
    }
}

/// Healthy is the most polarized attractor (most negative Vmem min).
pub struct HealthyIsMostPolarized;

impl Axiom for HealthyIsMostPolarized {
    fn description(&self) -> &str {
        "healthy attractor has the most polarized Vmem"
    }

    fn holds(&self) -> bool {
        use MorphospaceEntity::*;
        let vmem = AttractorVmemRange;
        let healthy = vmem.get(&Healthy).unwrap();
        [Inflamed, Barretts, Dysplastic, Fibrotic].iter().all(|a| {
            let range = vmem.get(a).unwrap();
            healthy.min < range.min
        })
    }
}

/// Severity increases with depolarization along the main disease axis.
pub struct SeverityIncreasesWithDepolarization;

impl Axiom for SeverityIncreasesWithDepolarization {
    fn description(&self) -> &str {
        "disease severity increases with depolarization"
    }

    fn holds(&self) -> bool {
        use MorphospaceEntity::*;
        let sev = DiseaseSeverity;
        let vmem = AttractorVmemRange;
        // Check pairs along main axis: Healthy < Inflamed < Barretts < Dysplastic
        let pairs = [
            (Healthy, Inflamed),
            (Inflamed, Barretts),
            (Barretts, Dysplastic),
        ];
        pairs.iter().all(|(a, b)| {
            let sa = sev.get(a).unwrap();
            let sb = sev.get(b).unwrap();
            let va = vmem.get(a).unwrap();
            let vb = vmem.get(b).unwrap();
            sa < sb && va.max < vb.max
        })
    }
}

/// Disease progression causal graph is asymmetric.
pub struct DiseaseProgressionIsAsymmetric;

impl Axiom for DiseaseProgressionIsAsymmetric {
    fn description(&self) -> &str {
        "disease progression is asymmetric"
    }

    fn holds(&self) -> bool {
        causation::Asymmetric::<DiseaseProgressionCauses>::new().holds()
    }
}

/// Acid damage transitively causes dysplasia.
pub struct AcidCausesDysplasia;

impl Axiom for AcidCausesDysplasia {
    fn description(&self) -> &str {
        "acid damage transitively causes dysplastic transition"
    }

    fn holds(&self) -> bool {
        use MorphospaceEvent::*;
        let effects = causation::effects_of::<DiseaseProgressionCauses>(&AcidDamage);
        effects.contains(&DysplasticTransition)
    }
}

/// Mechanical stimulation activation causes repair.
pub struct MechanicalStimulationCausesRepair;

impl Axiom for MechanicalStimulationCausesRepair {
    fn description(&self) -> &str {
        "mechanotransduction activation causes autonomous repair"
    }

    fn holds(&self) -> bool {
        use MorphospaceEvent::*;
        let effects =
            causation::effects_of::<DiseaseProgressionCauses>(&MechanotransductionActivation);
        effects.contains(&AutonomousRepair)
    }
}

/// Acid removal causes repair.
pub struct AcidRemovalCausesRepair;

impl Axiom for AcidRemovalCausesRepair {
    fn description(&self) -> &str {
        "acid removal causes autonomous repair"
    }

    fn holds(&self) -> bool {
        use MorphospaceEvent::*;
        let effects = causation::effects_of::<DiseaseProgressionCauses>(&AcidRemoval);
        effects.contains(&AutonomousRepair)
    }
}

/// Two-mechanism GJ requirement: BioelectricRepair requires GJ,
/// MechanicalStimulation does not.
pub struct TwoMechanismGJRequirement;

impl Axiom for TwoMechanismGJRequirement {
    fn description(&self) -> &str {
        "bioelectric repair requires GJ, mechanical stimulation does not"
    }

    fn holds(&self) -> bool {
        use MorphospaceEntity::*;
        let gj = PathwayRequiresGJ;
        gj.get(&BioelectricRepair) == Some(true) && gj.get(&MechanicalStimulation) == Some(false)
    }
}

/// Only MechanicalStimulation is hardware-accessible.
pub struct OnlyMechanicalIsHardwareAccessible;

impl Axiom for OnlyMechanicalIsHardwareAccessible {
    fn description(&self) -> &str {
        "only mechanical stimulation is hardware-accessible"
    }

    fn holds(&self) -> bool {
        let hw = PathwayIsHardwareAccessible;
        let pathways: Vec<MorphospaceEntity> = MorphospaceEntity::variants()
            .into_iter()
            .filter(|e| {
                taxonomy::is_a::<MorphospaceTaxonomy>(e, &MorphospaceEntity::RepairPathway)
                    && *e != MorphospaceEntity::RepairPathway
            })
            .collect();
        let hw_accessible: Vec<&MorphospaceEntity> = pathways
            .iter()
            .filter(|e| hw.get(e) == Some(true))
            .collect();
        hw_accessible.len() == 1 && *hw_accessible[0] == MorphospaceEntity::MechanicalStimulation
    }
}

/// There are exactly 5 attractor states.
pub struct FiveAttractorStates;

impl Axiom for FiveAttractorStates {
    fn description(&self) -> &str {
        "there are exactly 5 attractor states"
    }

    fn holds(&self) -> bool {
        let descendants =
            taxonomy::descendants::<MorphospaceTaxonomy>(&MorphospaceEntity::Attractor);
        descendants.len() == 5
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// Top-level ontology tying together the morphospace category, qualities, and axioms.
pub struct MorphospaceOntology;

impl Ontology for MorphospaceOntology {
    type Cat = MorphospaceCategory;
    type Qual = DiseaseSeverity;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        MorphospaceOntologyMeta::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(AllAttractorsHaveVmemRanges),
            Box::new(HealthyIsMostPolarized),
            Box::new(SeverityIncreasesWithDepolarization),
            Box::new(AcidCausesDysplasia),
            Box::new(MechanicalStimulationCausesRepair),
            Box::new(AcidRemovalCausesRepair),
            Box::new(TwoMechanismGJRequirement),
            Box::new(OnlyMechanicalIsHardwareAccessible),
            Box::new(FiveAttractorStates),
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

    // -- Axiom tests --

    #[test]
    fn test_morphospace_taxonomy_is_dag() {
        assert!(
            MorphospaceTaxonomyIsDAG.holds(),
            "{}",
            MorphospaceTaxonomyIsDAG.description()
        );
    }

    #[test]
    fn test_all_attractors_have_vmem_ranges() {
        assert!(
            AllAttractorsHaveVmemRanges.holds(),
            "{}",
            AllAttractorsHaveVmemRanges.description()
        );
    }

    #[test]
    fn test_healthy_is_most_polarized() {
        assert!(
            HealthyIsMostPolarized.holds(),
            "{}",
            HealthyIsMostPolarized.description()
        );
    }

    #[test]
    fn test_severity_increases_with_depolarization() {
        assert!(
            SeverityIncreasesWithDepolarization.holds(),
            "{}",
            SeverityIncreasesWithDepolarization.description()
        );
    }

    #[test]
    fn test_disease_progression_is_asymmetric() {
        assert!(
            DiseaseProgressionIsAsymmetric.holds(),
            "{}",
            DiseaseProgressionIsAsymmetric.description()
        );
    }

    #[test]
    fn test_acid_causes_dysplasia() {
        assert!(
            AcidCausesDysplasia.holds(),
            "{}",
            AcidCausesDysplasia.description()
        );
    }

    #[test]
    fn test_mechanical_stimulation_causes_repair() {
        assert!(
            MechanicalStimulationCausesRepair.holds(),
            "{}",
            MechanicalStimulationCausesRepair.description()
        );
    }

    #[test]
    fn test_acid_removal_causes_repair() {
        assert!(
            AcidRemovalCausesRepair.holds(),
            "{}",
            AcidRemovalCausesRepair.description()
        );
    }

    #[test]
    fn test_two_mechanism_gj_requirement() {
        assert!(
            TwoMechanismGJRequirement.holds(),
            "{}",
            TwoMechanismGJRequirement.description()
        );
    }

    #[test]
    fn test_only_mechanical_is_hardware_accessible() {
        assert!(
            OnlyMechanicalIsHardwareAccessible.holds(),
            "{}",
            OnlyMechanicalIsHardwareAccessible.description()
        );
    }

    #[test]
    fn test_five_attractor_states() {
        assert!(
            FiveAttractorStates.holds(),
            "{}",
            FiveAttractorStates.description()
        );
    }

    // -- Category law tests --

    #[test]
    fn test_morphospace_category_laws() {
        check_category_laws::<MorphospaceCategory>().unwrap();
    }

    #[test]
    fn test_morphospace_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<MorphospaceTaxonomy>>().unwrap();
    }

    #[test]
    fn test_causal_category_laws() {
        check_category_laws::<CausalCategory<DiseaseProgressionCauses>>().unwrap();
    }

    // -- Repair pathway classification --

    #[test]
    fn test_repair_pathways_classified() {
        use MorphospaceEntity::*;
        for pathway in [
            BasalTurnover,
            BioelectricRepair,
            MechanicalStimulation,
            CombinedTherapy,
        ] {
            assert!(
                taxonomy::is_a::<MorphospaceTaxonomy>(&pathway, &RepairPathway),
                "{:?} should be a RepairPathway",
                pathway
            );
        }
    }

    // -- Vmem range tests --

    #[test]
    fn test_healthy_vmem_range() {
        let range = AttractorVmemRange.get(&MorphospaceEntity::Healthy).unwrap();
        assert_eq!(range.min, -70.0);
        assert_eq!(range.max, -40.0);
    }

    #[test]
    fn test_dysplastic_vmem_range() {
        let range = AttractorVmemRange
            .get(&MorphospaceEntity::Dysplastic)
            .unwrap();
        assert_eq!(range.min, -18.0);
        assert_eq!(range.max, 0.0);
    }

    // -- Causal chain tests --

    #[test]
    fn test_acid_damage_chain() {
        use MorphospaceEvent::*;
        let effects = causation::effects_of::<DiseaseProgressionCauses>(&AcidDamage);
        assert!(effects.contains(&ChronicInflammation));
        assert!(effects.contains(&GapJunctionLoss));
        assert!(effects.contains(&FibroticRemodeling));
        assert!(effects.contains(&MetaplasticTransition));
        assert!(effects.contains(&DysplasticTransition));
    }

    #[test]
    fn test_mechanotransduction_repair_chain() {
        use MorphospaceEvent::*;
        let effects =
            causation::effects_of::<DiseaseProgressionCauses>(&MechanotransductionActivation);
        assert!(effects.contains(&VmemRepolarization));
        assert!(effects.contains(&GapJunctionRestoration));
        assert!(effects.contains(&PatternRecognition));
        assert!(effects.contains(&AutonomousRepair));
    }

    #[test]
    fn test_entity_count() {
        assert_eq!(MorphospaceEntity::variants().len(), 16);
    }

    #[test]
    fn test_event_count() {
        assert_eq!(MorphospaceEvent::variants().len(), 13);
    }

    #[test]
    fn test_ontology_validates() {
        MorphospaceOntology::validate().unwrap();
    }

    // -- Opposition tests --

    #[test]
    fn test_morphospace_opposition_symmetric() {
        assert!(
            MorphospaceOppositionSymmetric.holds(),
            "{}",
            MorphospaceOppositionSymmetric.description()
        );
    }

    #[test]
    fn test_morphospace_opposition_irreflexive() {
        assert!(
            MorphospaceOppositionIrreflexive.holds(),
            "{}",
            MorphospaceOppositionIrreflexive.description()
        );
    }

    #[test]
    fn test_healthy_opposes_dysplastic() {
        use MorphospaceEntity::*;
        assert!(opposition::are_opposed::<MorphospaceOpposition>(
            &Healthy,
            &Dysplastic
        ));
        assert!(opposition::are_opposed::<MorphospaceOpposition>(
            &Dysplastic,
            &Healthy
        ));
    }

    #[test]
    fn test_polarized_opposes_depolarized() {
        use MorphospaceEntity::*;
        assert!(opposition::are_opposed::<MorphospaceOpposition>(
            &PolarizedVmem,
            &DepolarizedVmem
        ));
    }

    #[test]
    fn test_connected_opposes_disconnected() {
        use MorphospaceEntity::*;
        assert!(opposition::are_opposed::<MorphospaceOpposition>(
            &ConnectedNetwork,
            &DisconnectedNetwork
        ));
    }

    #[test]
    fn test_basal_turnover_opposes_bioelectric_repair() {
        use MorphospaceEntity::*;
        assert!(opposition::are_opposed::<MorphospaceOpposition>(
            &BasalTurnover,
            &BioelectricRepair
        ));
    }

    #[test]
    fn test_healthy_does_not_oppose_inflamed() {
        use MorphospaceEntity::*;
        assert!(!opposition::are_opposed::<MorphospaceOpposition>(
            &Healthy, &Inflamed
        ));
    }

    #[test]
    fn test_morphospace_opposites_query() {
        use MorphospaceEntity::*;
        let opps = opposition::opposites::<MorphospaceOpposition>(&Healthy);
        assert_eq!(opps, vec![Dysplastic]);
    }

    // -- Property-based tests (proptest) --

    use proptest::prelude::*;

    fn arb_attractor() -> impl Strategy<Value = MorphospaceEntity> {
        prop::sample::select(vec![
            MorphospaceEntity::Healthy,
            MorphospaceEntity::Inflamed,
            MorphospaceEntity::Barretts,
            MorphospaceEntity::Dysplastic,
            MorphospaceEntity::Fibrotic,
        ])
    }

    /// For attractors along the main disease axis (excluding Fibrotic which overlaps).
    fn arb_main_axis_attractor() -> impl Strategy<Value = MorphospaceEntity> {
        prop::sample::select(vec![
            MorphospaceEntity::Healthy,
            MorphospaceEntity::Inflamed,
            MorphospaceEntity::Barretts,
            MorphospaceEntity::Dysplastic,
        ])
    }

    proptest! {
        /// For any attractor with a VmemRange, min < max.
        #[test]
        fn prop_attractor_vmem_range_min_lt_max(attractor in arb_attractor()) {
            let range = AttractorVmemRange.get(&attractor).unwrap();
            prop_assert!(
                range.min < range.max,
                "VmemRange for {:?}: min ({}) should be < max ({})",
                attractor,
                range.min,
                range.max
            );
        }

        /// For any pair of main-axis attractors, if severity(A) < severity(B),
        /// then VmemRange of A has lower max than B (monotonicity).
        /// Fibrotic is excluded since its Vmem range overlaps with the main axis.
        #[test]
        fn prop_severity_vmem_monotonicity(
            a in arb_main_axis_attractor(),
            b in arb_main_axis_attractor(),
        ) {
            let sev_a = DiseaseSeverity.get(&a).unwrap();
            let sev_b = DiseaseSeverity.get(&b).unwrap();
            if sev_a < sev_b {
                let vmem_a = AttractorVmemRange.get(&a).unwrap();
                let vmem_b = AttractorVmemRange.get(&b).unwrap();
                prop_assert!(
                    vmem_a.max < vmem_b.max,
                    "Severity monotonicity: {:?} (sev={}, max={}) should have lower Vmem max than {:?} (sev={}, max={})",
                    a, sev_a, vmem_a.max,
                    b, sev_b, vmem_b.max
                );
            }
        }
    }

    // -- Literature axioms --

    /// Gralnek 2006: PPI heals ~70% at 4 weeks via acid removal -> basal repair pathway.
    /// BasalTurnover is GJ-independent.
    #[test]
    fn test_literature_gralnek_2006_ppi_basal_turnover_gj_independent() {
        use MorphospaceEntity::*;
        assert_eq!(
            PathwayRequiresGJ.get(&BasalTurnover),
            Some(false),
            "Gralnek 2006: PPI heals ~70% at 4 weeks via acid removal, \
             activating the basal turnover repair pathway which is GJ-independent"
        );
    }

    /// Levin 2015 PMID:26610482: gap junction blockade induces different
    /// species-specific head anatomies -- proving bistability in morphospace.
    /// Bistability is a PatternConcept in regeneration; GJModulation causes
    /// pattern change (connecting morphospace to bioelectric interventions).
    #[test]
    fn test_literature_levin_2015_gj_blockade_bistability() {
        use MorphospaceEntity::*;
        use MorphospaceEvent::*;
        // GapJunctionLoss (representing GJ blockade) transitively causes
        // remodeling, proving that GJ modulation alters morphospace state
        let effects = causation::effects_of::<DiseaseProgressionCauses>(&GapJunctionLoss);
        assert!(
            effects.contains(&FibroticRemodeling),
            "Levin 2015: gap junction blockade causes morphological change \
             (fibrotic remodeling), demonstrating bistability in morphospace"
        );
        // There are at least 2 distinct attractor states, proving bistability
        let attractors = taxonomy::descendants::<MorphospaceTaxonomy>(&Attractor);
        assert!(
            attractors.len() >= 2,
            "Levin 2015: morphospace must have at least 2 attractor states \
             to support bistability (head anatomy variants)"
        );
    }
}
