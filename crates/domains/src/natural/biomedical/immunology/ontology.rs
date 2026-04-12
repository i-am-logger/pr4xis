//! Immunology ontology for tissue repair and bioelectric healing.
//!
//! Models the inflammatory cascade, macrophage polarization (M1/M2), cytokine
//! signaling, and the causal chain from tissue injury to repair or fibrosis.
//!
//! The key scientific insight modeled here: mechanical stimulation (whole-body
//! vibration) promotes M1->M2 macrophage transition, shifting the immune response
//! from pro-inflammatory to pro-repair (Weinheimer-Haus 2014, Yu 2019).
//!
//! # References
//! - Weinheimer-Haus 2014: Low-intensity vibration improves angiogenesis and
//!   wound healing via shifts in macrophage polarization
//! - Yu 2019: Mechanical loading promotes tissue repair through immune modulation

use pr4xis::category::Entity;
use pr4xis::define_dense_category;
use pr4xis::ontology::reasoning::causation::{self, CausalDef};
use pr4xis::ontology::reasoning::opposition::{self, OppositionDef};
use pr4xis::ontology::reasoning::taxonomy::{self, TaxonomyDef};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Immunology Entity
// ---------------------------------------------------------------------------

/// Every entity in the immunology ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum ImmunologyEntity {
    // Cells
    MacrophageM1,
    MacrophageM2,
    Neutrophil,
    TCell,
    Monocyte,
    MastCell,
    Fibroblast,

    // States
    AcuteInflammation,
    ChronicInflammation,
    Resolution,
    Fibrosis,
    TissueRepair,

    // Signals (cytokines)
    ProInflammatoryCytokine,
    AntiInflammatoryCytokine,
    TNFAlpha,
    IL6,
    IL10,
    TGFBeta,

    // Abstract categories
    ImmuneCell,
    StromalCell,
    InflammatoryState,
    Cytokine,
}

// ---------------------------------------------------------------------------
// Taxonomy (is-a)
// ---------------------------------------------------------------------------

/// Subsumption hierarchy for immunology entities.
///
/// Cells -> ImmuneCell, states -> InflammatoryState, cytokines -> Cytokine.
/// TNFAlpha/IL6 -> ProInflammatoryCytokine -> Cytokine.
/// IL10/TGFBeta -> AntiInflammatoryCytokine -> Cytokine.
pub struct ImmunologyTaxonomy;

impl TaxonomyDef for ImmunologyTaxonomy {
    type Entity = ImmunologyEntity;

    fn relations() -> Vec<(ImmunologyEntity, ImmunologyEntity)> {
        use ImmunologyEntity::*;
        vec![
            // Cells is-a ImmuneCell
            (MacrophageM1, ImmuneCell),
            (MacrophageM2, ImmuneCell),
            (Neutrophil, ImmuneCell),
            (TCell, ImmuneCell),
            (Monocyte, ImmuneCell),
            (MastCell, ImmuneCell),
            (Fibroblast, StromalCell),
            // States is-a InflammatoryState
            (AcuteInflammation, InflammatoryState),
            (ChronicInflammation, InflammatoryState),
            (Resolution, InflammatoryState),
            (Fibrosis, InflammatoryState),
            (TissueRepair, InflammatoryState),
            // Specific cytokines is-a Pro/Anti-inflammatory
            (TNFAlpha, ProInflammatoryCytokine),
            (IL6, ProInflammatoryCytokine),
            (IL10, AntiInflammatoryCytokine),
            (TGFBeta, AntiInflammatoryCytokine),
            // Pro/Anti-inflammatory is-a Cytokine
            (ProInflammatoryCytokine, Cytokine),
            (AntiInflammatoryCytokine, Cytokine),
        ]
    }
}

// ---------------------------------------------------------------------------
// Causal Events
// ---------------------------------------------------------------------------

/// Events in the inflammatory causal chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum ImmunologyEvent {
    TissueInjury,
    NeutrophilRecruitment,
    AcuteInflammationOnset,
    MonocyteRecruitment,
    M1Polarization,
    ProInflammatoryResponse,
    M1ToM2Transition,
    AntiInflammatoryResponse,
    TissueRemodeling,
    RepairCompletion,
    ChronicStimulus,
    FailedResolution,
    FibrosisProgression,
    MechanicalStimulation,
}

/// Causal graph for the inflammatory cascade.
///
/// Normal healing path:
///   TissueInjury -> NeutrophilRecruitment -> AcuteInflammationOnset
///   -> MonocyteRecruitment -> M1Polarization -> ProInflammatoryResponse
///   -> M1ToM2Transition -> AntiInflammatoryResponse -> TissueRemodeling
///   -> RepairCompletion
///
/// Pathological path:
///   ChronicStimulus -> FailedResolution -> FibrosisProgression
///
/// Vibration intervention (Weinheimer-Haus 2014):
///   MechanicalStimulation -> M1ToM2Transition
pub struct InflammationCauses;

impl CausalDef for InflammationCauses {
    type Entity = ImmunologyEvent;

    fn relations() -> Vec<(ImmunologyEvent, ImmunologyEvent)> {
        use ImmunologyEvent::*;
        vec![
            // Normal healing cascade
            (TissueInjury, NeutrophilRecruitment),
            (NeutrophilRecruitment, AcuteInflammationOnset),
            (AcuteInflammationOnset, MonocyteRecruitment),
            (MonocyteRecruitment, M1Polarization),
            (M1Polarization, ProInflammatoryResponse),
            (ProInflammatoryResponse, M1ToM2Transition),
            (M1ToM2Transition, AntiInflammatoryResponse),
            (AntiInflammatoryResponse, TissueRemodeling),
            (TissueRemodeling, RepairCompletion),
            // Pathological path
            (ChronicStimulus, FailedResolution),
            (FailedResolution, FibrosisProgression),
            // Vibration intervention (Weinheimer-Haus 2014)
            (MechanicalStimulation, M1ToM2Transition),
        ]
    }
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

define_dense_category! {
    /// Discrete category over immunology entities.
    pub ImmunologyCategory {
        entity: ImmunologyEntity,
        relation: ImmunologyRelation,
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Macrophage polarization state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolarizationValue {
    M1Classical,
    M2Alternative,
    Unpolarized,
    NotApplicable,
}

/// Time scale of inflammatory processes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeScaleValue {
    Hours,
    Days,
    Weeks,
}

/// Quality: is this entity pro-inflammatory?
#[derive(Debug, Clone)]
pub struct IsProInflammatory;

impl Quality for IsProInflammatory {
    type Individual = ImmunologyEntity;
    type Value = bool;

    fn get(&self, individual: &ImmunologyEntity) -> Option<bool> {
        use ImmunologyEntity::*;
        match individual {
            MacrophageM1 => Some(true),
            MacrophageM2 => Some(false),
            Neutrophil => Some(true),
            MastCell => Some(true),
            TNFAlpha => Some(true),
            IL6 => Some(true),
            IL10 => Some(false),
            TGFBeta => Some(false),
            ProInflammatoryCytokine => Some(true),
            AntiInflammatoryCytokine => Some(false),
            AcuteInflammation => Some(true),
            ChronicInflammation => Some(true),
            Resolution => Some(false),
            TissueRepair => Some(false),
            Fibrosis => Some(false),
            _ => None,
        }
    }
}

/// Quality: is this entity pro-repair?
#[derive(Debug, Clone)]
pub struct IsProRepair;

impl Quality for IsProRepair {
    type Individual = ImmunologyEntity;
    type Value = bool;

    fn get(&self, individual: &ImmunologyEntity) -> Option<bool> {
        use ImmunologyEntity::*;
        match individual {
            MacrophageM1 => Some(false),
            MacrophageM2 => Some(true),
            Fibroblast => Some(true),
            IL10 => Some(true),
            TGFBeta => Some(true),
            TNFAlpha => Some(false),
            IL6 => Some(false),
            AntiInflammatoryCytokine => Some(true),
            ProInflammatoryCytokine => Some(false),
            TissueRepair => Some(true),
            Resolution => Some(true),
            Fibrosis => Some(false),
            AcuteInflammation => Some(false),
            ChronicInflammation => Some(false),
            _ => None,
        }
    }
}

/// Quality: macrophage polarization state.
#[derive(Debug, Clone)]
pub struct PolarizationState;

impl Quality for PolarizationState {
    type Individual = ImmunologyEntity;
    type Value = PolarizationValue;

    fn get(&self, individual: &ImmunologyEntity) -> Option<PolarizationValue> {
        use ImmunologyEntity::*;
        match individual {
            MacrophageM1 => Some(PolarizationValue::M1Classical),
            MacrophageM2 => Some(PolarizationValue::M2Alternative),
            Monocyte => Some(PolarizationValue::Unpolarized),
            Neutrophil | TCell | MastCell | Fibroblast => Some(PolarizationValue::NotApplicable),
            _ => None,
        }
    }
}

/// Quality: time scale of inflammatory states.
#[derive(Debug, Clone)]
pub struct TimeScale;

impl Quality for TimeScale {
    type Individual = ImmunologyEntity;
    type Value = TimeScaleValue;

    fn get(&self, individual: &ImmunologyEntity) -> Option<TimeScaleValue> {
        use ImmunologyEntity::*;
        match individual {
            AcuteInflammation => Some(TimeScaleValue::Hours),
            Resolution => Some(TimeScaleValue::Days),
            TissueRepair => Some(TimeScaleValue::Days),
            ChronicInflammation => Some(TimeScaleValue::Weeks),
            Fibrosis => Some(TimeScaleValue::Weeks),
            _ => None,
        }
    }
}

/// Quality: is this event/transition modulable by vibration?
///
/// Based on Weinheimer-Haus 2014: whole-body vibration shifts macrophage
/// polarization from M1 (pro-inflammatory) to M2 (pro-repair).
#[derive(Debug, Clone)]
pub struct IsModulableByVibration;

impl Quality for IsModulableByVibration {
    type Individual = ImmunologyEvent;
    type Value = bool;

    fn get(&self, individual: &ImmunologyEvent) -> Option<bool> {
        use ImmunologyEvent::*;
        match individual {
            M1ToM2Transition => Some(true),
            AntiInflammatoryResponse => Some(true),
            TissueRemodeling => Some(true),
            RepairCompletion => Some(true),
            TissueInjury => Some(false),
            NeutrophilRecruitment => Some(false),
            AcuteInflammationOnset => Some(false),
            MonocyteRecruitment => Some(false),
            M1Polarization => Some(false),
            ProInflammatoryResponse => Some(false),
            ChronicStimulus => Some(false),
            FailedResolution => Some(false),
            FibrosisProgression => Some(false),
            MechanicalStimulation => Some(false),
        }
    }
}

// ---------------------------------------------------------------------------
// Opposition (semantic contrasts)
// ---------------------------------------------------------------------------

/// Opposition pairs in the immunology domain.
///
/// - MacrophageM1 ↔ MacrophageM2: pro-inflammatory vs pro-repair phenotypes
/// - AcuteInflammation ↔ Resolution: onset vs resolution of inflammation
/// - ChronicInflammation ↔ TissueRepair: pathological persistence vs healing
/// - ProInflammatoryCytokine ↔ AntiInflammatoryCytokine: opposing signaling classes
/// - TNFAlpha ↔ IL10: canonical pro- vs anti-inflammatory cytokines
pub struct ImmunologyOpposition;

impl OppositionDef for ImmunologyOpposition {
    type Entity = ImmunologyEntity;

    fn pairs() -> Vec<(ImmunologyEntity, ImmunologyEntity)> {
        use ImmunologyEntity::*;
        vec![
            (MacrophageM1, MacrophageM2),
            (AcuteInflammation, Resolution),
            (ChronicInflammation, TissueRepair),
            (ProInflammatoryCytokine, AntiInflammatoryCytokine),
            (TNFAlpha, IL10),
        ]
    }
}

/// Axiom: immunology opposition is symmetric.
pub struct ImmunologyOppositionSymmetric;

impl Axiom for ImmunologyOppositionSymmetric {
    fn description(&self) -> &str {
        "immunology opposition is symmetric"
    }

    fn holds(&self) -> bool {
        opposition::Symmetric::<ImmunologyOpposition>::new().holds()
    }
}

/// Axiom: immunology opposition is irreflexive (nothing opposes itself).
pub struct ImmunologyOppositionIrreflexive;

impl Axiom for ImmunologyOppositionIrreflexive {
    fn description(&self) -> &str {
        "immunology opposition is irreflexive"
    }

    fn holds(&self) -> bool {
        opposition::Irreflexive::<ImmunologyOpposition>::new().holds()
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Axiom: immunology taxonomy is a DAG (no cycles).
pub struct ImmunologyTaxonomyIsDAG;

impl Axiom for ImmunologyTaxonomyIsDAG {
    fn description(&self) -> &str {
        "immunology taxonomy is a directed acyclic graph"
    }

    fn holds(&self) -> bool {
        taxonomy::NoCycles::<ImmunologyTaxonomy>::new().holds()
    }
}

/// Axiom: causal graph is asymmetric (if A causes B, B does not cause A).
pub struct CausalAsymmetry;

impl Axiom for CausalAsymmetry {
    fn description(&self) -> &str {
        "inflammatory causation is asymmetric"
    }

    fn holds(&self) -> bool {
        causation::Asymmetric::<InflammationCauses>::new().holds()
    }
}

/// Axiom: no event directly causes itself.
pub struct CausalNoSelfCausation;

impl Axiom for CausalNoSelfCausation {
    fn description(&self) -> &str {
        "no inflammatory event directly causes itself"
    }

    fn holds(&self) -> bool {
        causation::NoSelfCausation::<InflammationCauses>::new().holds()
    }
}

/// Axiom: tissue injury transitively causes repair completion (normal healing).
pub struct InjuryCausesRepair;

impl Axiom for InjuryCausesRepair {
    fn description(&self) -> &str {
        "tissue injury transitively causes repair completion (normal healing path)"
    }

    fn holds(&self) -> bool {
        use ImmunologyEvent::*;
        let effects = causation::effects_of::<InflammationCauses>(&TissueInjury);
        effects.contains(&RepairCompletion)
    }
}

/// Axiom: chronic stimulus causes fibrosis, not repair.
pub struct ChronicStimulusCausesFibrosis;

impl Axiom for ChronicStimulusCausesFibrosis {
    fn description(&self) -> &str {
        "chronic stimulus causes fibrosis progression (pathological path)"
    }

    fn holds(&self) -> bool {
        use ImmunologyEvent::*;
        let effects = causation::effects_of::<InflammationCauses>(&ChronicStimulus);
        effects.contains(&FibrosisProgression) && !effects.contains(&RepairCompletion)
    }
}

/// Axiom: mechanical stimulation causes M1->M2 transition (Weinheimer-Haus 2014).
pub struct VibrationCausesM1ToM2;

impl Axiom for VibrationCausesM1ToM2 {
    fn description(&self) -> &str {
        "mechanical stimulation causes M1-to-M2 transition (Weinheimer-Haus 2014)"
    }

    fn holds(&self) -> bool {
        use ImmunologyEvent::*;
        let effects = causation::effects_of::<InflammationCauses>(&MechanicalStimulation);
        effects.contains(&M1ToM2Transition)
    }
}

/// Axiom: M1 is pro-inflammatory and not pro-repair; M2 is pro-repair and not
/// pro-inflammatory. They are mutually exclusive phenotypes.
pub struct M1M2MutuallyExclusive;

impl Axiom for M1M2MutuallyExclusive {
    fn description(&self) -> &str {
        "M1 is pro-inflammatory (not pro-repair), M2 is pro-repair (not pro-inflammatory)"
    }

    fn holds(&self) -> bool {
        use ImmunologyEntity::*;
        let pi = IsProInflammatory;
        let pr = IsProRepair;
        pi.get(&MacrophageM1) == Some(true)
            && pr.get(&MacrophageM1) == Some(false)
            && pi.get(&MacrophageM2) == Some(false)
            && pr.get(&MacrophageM2) == Some(true)
    }
}

/// Axiom: pro-inflammatory and anti-inflammatory cytokines are disjoint branches.
pub struct CytokineBranchesDisjoint;

impl Axiom for CytokineBranchesDisjoint {
    fn description(&self) -> &str {
        "pro-inflammatory and anti-inflammatory cytokines are disjoint taxonomy branches"
    }

    fn holds(&self) -> bool {
        use ImmunologyEntity::*;
        // TNFAlpha and IL6 are pro-inflammatory, not anti-inflammatory
        taxonomy::is_a::<ImmunologyTaxonomy>(&TNFAlpha, &ProInflammatoryCytokine)
            && taxonomy::is_a::<ImmunologyTaxonomy>(&IL6, &ProInflammatoryCytokine)
            && !taxonomy::is_a::<ImmunologyTaxonomy>(&TNFAlpha, &AntiInflammatoryCytokine)
            && !taxonomy::is_a::<ImmunologyTaxonomy>(&IL6, &AntiInflammatoryCytokine)
            // IL10 and TGFBeta are anti-inflammatory, not pro-inflammatory
            && taxonomy::is_a::<ImmunologyTaxonomy>(&IL10, &AntiInflammatoryCytokine)
            && taxonomy::is_a::<ImmunologyTaxonomy>(&TGFBeta, &AntiInflammatoryCytokine)
            && !taxonomy::is_a::<ImmunologyTaxonomy>(&IL10, &ProInflammatoryCytokine)
            && !taxonomy::is_a::<ImmunologyTaxonomy>(&TGFBeta, &ProInflammatoryCytokine)
    }
}

/// Axiom: M1->M2 transition eventually leads to repair completion.
pub struct M1ToM2LeadsToRepair;

impl Axiom for M1ToM2LeadsToRepair {
    fn description(&self) -> &str {
        "M1-to-M2 transition eventually leads to repair completion"
    }

    fn holds(&self) -> bool {
        use ImmunologyEvent::*;
        let effects = causation::effects_of::<InflammationCauses>(&M1ToM2Transition);
        effects.contains(&RepairCompletion)
    }
}

/// Axiom: all immune cells are classified under ImmuneCell in the taxonomy.
pub struct AllImmuneCellsClassified;

impl Axiom for AllImmuneCellsClassified {
    fn description(&self) -> &str {
        "all concrete immune cells are classified under ImmuneCell; Fibroblast is StromalCell"
    }

    fn holds(&self) -> bool {
        use ImmunologyEntity::*;
        let immune = [
            MacrophageM1,
            MacrophageM2,
            Neutrophil,
            TCell,
            Monocyte,
            MastCell,
        ];
        let stromal = [Fibroblast];
        immune
            .iter()
            .all(|c| taxonomy::is_a::<ImmunologyTaxonomy>(c, &ImmuneCell))
            && stromal
                .iter()
                .all(|c| taxonomy::is_a::<ImmunologyTaxonomy>(c, &StromalCell))
    }
}

/// Axiom: acute inflammation operates on Hours time scale,
/// chronic inflammation on Weeks.
pub struct InflammationTimeScales;

impl Axiom for InflammationTimeScales {
    fn description(&self) -> &str {
        "acute inflammation is Hours, chronic inflammation is Weeks"
    }

    fn holds(&self) -> bool {
        use ImmunologyEntity::*;
        let ts = TimeScale;
        ts.get(&AcuteInflammation) == Some(TimeScaleValue::Hours)
            && ts.get(&ChronicInflammation) == Some(TimeScaleValue::Weeks)
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// Top-level ontology tying together the immunology category, qualities, and axioms.
pub struct ImmunologyOntology;

impl Ontology for ImmunologyOntology {
    type Cat = ImmunologyCategory;
    type Qual = IsProInflammatory;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(ImmunologyTaxonomyIsDAG),
            Box::new(CausalAsymmetry),
            Box::new(CausalNoSelfCausation),
            Box::new(InjuryCausesRepair),
            Box::new(ChronicStimulusCausesFibrosis),
            Box::new(VibrationCausesM1ToM2),
            Box::new(M1M2MutuallyExclusive),
            Box::new(CytokineBranchesDisjoint),
            Box::new(M1ToM2LeadsToRepair),
            Box::new(AllImmuneCellsClassified),
            Box::new(InflammationTimeScales),
            Box::new(ImmunologyOppositionSymmetric),
            Box::new(ImmunologyOppositionIrreflexive),
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
    fn test_taxonomy_is_dag() {
        assert!(
            ImmunologyTaxonomyIsDAG.holds(),
            "{}",
            ImmunologyTaxonomyIsDAG.description()
        );
    }

    #[test]
    fn test_causal_asymmetry() {
        assert!(CausalAsymmetry.holds(), "{}", CausalAsymmetry.description());
    }

    #[test]
    fn test_causal_no_self_causation() {
        assert!(
            CausalNoSelfCausation.holds(),
            "{}",
            CausalNoSelfCausation.description()
        );
    }

    #[test]
    fn test_injury_causes_repair() {
        assert!(
            InjuryCausesRepair.holds(),
            "{}",
            InjuryCausesRepair.description()
        );
    }

    #[test]
    fn test_chronic_stimulus_causes_fibrosis() {
        assert!(
            ChronicStimulusCausesFibrosis.holds(),
            "{}",
            ChronicStimulusCausesFibrosis.description()
        );
    }

    #[test]
    fn test_vibration_causes_m1_to_m2() {
        assert!(
            VibrationCausesM1ToM2.holds(),
            "{}",
            VibrationCausesM1ToM2.description()
        );
    }

    #[test]
    fn test_m1_m2_mutually_exclusive() {
        assert!(
            M1M2MutuallyExclusive.holds(),
            "{}",
            M1M2MutuallyExclusive.description()
        );
    }

    #[test]
    fn test_cytokine_branches_disjoint() {
        assert!(
            CytokineBranchesDisjoint.holds(),
            "{}",
            CytokineBranchesDisjoint.description()
        );
    }

    #[test]
    fn test_m1_to_m2_leads_to_repair() {
        assert!(
            M1ToM2LeadsToRepair.holds(),
            "{}",
            M1ToM2LeadsToRepair.description()
        );
    }

    #[test]
    fn test_all_immune_cells_classified() {
        assert!(
            AllImmuneCellsClassified.holds(),
            "{}",
            AllImmuneCellsClassified.description()
        );
    }

    #[test]
    fn test_inflammation_time_scales() {
        assert!(
            InflammationTimeScales.holds(),
            "{}",
            InflammationTimeScales.description()
        );
    }

    // -- Opposition tests --

    #[test]
    fn test_immunology_opposition_symmetric() {
        assert!(
            ImmunologyOppositionSymmetric.holds(),
            "{}",
            ImmunologyOppositionSymmetric.description()
        );
    }

    #[test]
    fn test_immunology_opposition_irreflexive() {
        assert!(
            ImmunologyOppositionIrreflexive.holds(),
            "{}",
            ImmunologyOppositionIrreflexive.description()
        );
    }

    #[test]
    fn test_m1_opposes_m2() {
        use ImmunologyEntity::*;
        assert!(opposition::are_opposed::<ImmunologyOpposition>(
            &MacrophageM1,
            &MacrophageM2
        ));
        assert!(opposition::are_opposed::<ImmunologyOpposition>(
            &MacrophageM2,
            &MacrophageM1
        ));
    }

    #[test]
    fn test_acute_inflammation_opposes_resolution() {
        use ImmunologyEntity::*;
        assert!(opposition::are_opposed::<ImmunologyOpposition>(
            &AcuteInflammation,
            &Resolution
        ));
    }

    #[test]
    fn test_chronic_inflammation_opposes_tissue_repair() {
        use ImmunologyEntity::*;
        assert!(opposition::are_opposed::<ImmunologyOpposition>(
            &ChronicInflammation,
            &TissueRepair
        ));
    }

    #[test]
    fn test_cytokine_classes_opposed() {
        use ImmunologyEntity::*;
        assert!(opposition::are_opposed::<ImmunologyOpposition>(
            &ProInflammatoryCytokine,
            &AntiInflammatoryCytokine
        ));
    }

    #[test]
    fn test_tnf_alpha_opposes_il10() {
        use ImmunologyEntity::*;
        assert!(opposition::are_opposed::<ImmunologyOpposition>(
            &TNFAlpha, &IL10
        ));
    }

    #[test]
    fn test_m1_does_not_oppose_neutrophil() {
        use ImmunologyEntity::*;
        assert!(!opposition::are_opposed::<ImmunologyOpposition>(
            &MacrophageM1,
            &Neutrophil
        ));
    }

    #[test]
    fn test_immunology_opposites_query() {
        use ImmunologyEntity::*;
        let opps = opposition::opposites::<ImmunologyOpposition>(&MacrophageM1);
        assert_eq!(opps, vec![MacrophageM2]);
    }

    // -- Category law tests --

    #[test]
    fn test_immunology_category_laws() {
        check_category_laws::<ImmunologyCategory>().unwrap();
    }

    #[test]
    fn test_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<ImmunologyTaxonomy>>().unwrap();
    }

    #[test]
    fn test_causal_category_laws() {
        check_category_laws::<CausalCategory<InflammationCauses>>().unwrap();
    }

    // -- Taxonomy tests --

    #[test]
    fn test_entity_count() {
        assert_eq!(ImmunologyEntity::variants().len(), 22);
    }

    #[test]
    fn test_event_count() {
        assert_eq!(ImmunologyEvent::variants().len(), 14);
    }

    #[test]
    fn test_cells_are_classified() {
        use ImmunologyEntity::*;
        // Immune cells
        for cell in [
            MacrophageM1,
            MacrophageM2,
            Neutrophil,
            TCell,
            Monocyte,
            MastCell,
        ] {
            assert!(
                taxonomy::is_a::<ImmunologyTaxonomy>(&cell, &ImmuneCell),
                "{:?} should be an ImmuneCell",
                cell
            );
        }
        // Stromal cells (Fibroblast is NOT an immune cell)
        assert!(taxonomy::is_a::<ImmunologyTaxonomy>(
            &Fibroblast,
            &StromalCell
        ));
        assert!(!taxonomy::is_a::<ImmunologyTaxonomy>(
            &Fibroblast,
            &ImmuneCell
        ));
    }

    #[test]
    fn test_cytokines_are_cytokines() {
        use ImmunologyEntity::*;
        for cytokine in [TNFAlpha, IL6, IL10, TGFBeta] {
            assert!(
                taxonomy::is_a::<ImmunologyTaxonomy>(&cytokine, &Cytokine),
                "{:?} should be a Cytokine",
                cytokine
            );
        }
    }

    #[test]
    fn test_states_are_inflammatory_states() {
        use ImmunologyEntity::*;
        for state in [
            AcuteInflammation,
            ChronicInflammation,
            Resolution,
            Fibrosis,
            TissueRepair,
        ] {
            assert!(
                taxonomy::is_a::<ImmunologyTaxonomy>(&state, &InflammatoryState),
                "{:?} should be an InflammatoryState",
                state
            );
        }
    }

    #[test]
    fn test_pro_inflammatory_not_anti_inflammatory() {
        use ImmunologyEntity::*;
        assert!(!taxonomy::is_a::<ImmunologyTaxonomy>(
            &TNFAlpha,
            &AntiInflammatoryCytokine
        ));
        assert!(!taxonomy::is_a::<ImmunologyTaxonomy>(
            &IL6,
            &AntiInflammatoryCytokine
        ));
    }

    #[test]
    fn test_anti_inflammatory_not_pro_inflammatory() {
        use ImmunologyEntity::*;
        assert!(!taxonomy::is_a::<ImmunologyTaxonomy>(
            &IL10,
            &ProInflammatoryCytokine
        ));
        assert!(!taxonomy::is_a::<ImmunologyTaxonomy>(
            &TGFBeta,
            &ProInflammatoryCytokine
        ));
    }

    // -- Causal chain tests --

    #[test]
    fn test_full_healing_cascade() {
        use ImmunologyEvent::*;
        let effects = causation::effects_of::<InflammationCauses>(&TissueInjury);
        for event in [
            NeutrophilRecruitment,
            AcuteInflammationOnset,
            MonocyteRecruitment,
            M1Polarization,
            ProInflammatoryResponse,
            M1ToM2Transition,
            AntiInflammatoryResponse,
            TissueRemodeling,
            RepairCompletion,
        ] {
            assert!(
                effects.contains(&event),
                "TissueInjury should transitively cause {:?}",
                event
            );
        }
    }

    #[test]
    fn test_fibrosis_path_does_not_reach_repair() {
        use ImmunologyEvent::*;
        let effects = causation::effects_of::<InflammationCauses>(&ChronicStimulus);
        assert!(effects.contains(&FibrosisProgression));
        assert!(!effects.contains(&RepairCompletion));
    }

    #[test]
    fn test_mechanical_stimulation_reaches_repair() {
        use ImmunologyEvent::*;
        let effects = causation::effects_of::<InflammationCauses>(&MechanicalStimulation);
        assert!(effects.contains(&M1ToM2Transition));
        assert!(effects.contains(&RepairCompletion));
    }

    // -- Polarization consistency tests --

    #[test]
    fn test_polarization_states() {
        use ImmunologyEntity::*;
        let ps = PolarizationState;
        assert_eq!(ps.get(&MacrophageM1), Some(PolarizationValue::M1Classical));
        assert_eq!(
            ps.get(&MacrophageM2),
            Some(PolarizationValue::M2Alternative)
        );
        assert_eq!(ps.get(&Monocyte), Some(PolarizationValue::Unpolarized));
        assert_eq!(ps.get(&Neutrophil), Some(PolarizationValue::NotApplicable));
    }

    #[test]
    fn test_m1_inflammatory_m2_repair_consistency() {
        use ImmunologyEntity::*;
        let pi = IsProInflammatory;
        let pr = IsProRepair;
        // M1: pro-inflammatory=true, pro-repair=false
        assert_eq!(pi.get(&MacrophageM1), Some(true));
        assert_eq!(pr.get(&MacrophageM1), Some(false));
        // M2: pro-inflammatory=false, pro-repair=true
        assert_eq!(pi.get(&MacrophageM2), Some(false));
        assert_eq!(pr.get(&MacrophageM2), Some(true));
    }

    #[test]
    fn test_vibration_modulable_events() {
        use ImmunologyEvent::*;
        let vm = IsModulableByVibration;
        assert_eq!(vm.get(&M1ToM2Transition), Some(true));
        assert_eq!(vm.get(&TissueInjury), Some(false));
        assert_eq!(vm.get(&ChronicStimulus), Some(false));
    }

    #[test]
    fn test_immune_cell_descendants_count() {
        let descendants =
            taxonomy::descendants::<ImmunologyTaxonomy>(&ImmunologyEntity::ImmuneCell);
        assert_eq!(descendants.len(), 6); // M1, M2, Neutrophil, TCell, Monocyte, MastCell
    }

    #[test]
    fn test_cytokine_descendants_count() {
        let descendants = taxonomy::descendants::<ImmunologyTaxonomy>(&ImmunologyEntity::Cytokine);
        // ProInflammatoryCytokine, AntiInflammatoryCytokine, TNFAlpha, IL6, IL10, TGFBeta
        assert_eq!(descendants.len(), 6);
    }

    #[test]
    fn test_ontology_validates() {
        ImmunologyOntology::validate().unwrap();
    }

    // -- Property-based tests (proptest) --

    use proptest::prelude::*;

    fn arb_immunology_entity() -> impl Strategy<Value = ImmunologyEntity> {
        (0..ImmunologyEntity::variants().len()).prop_map(|i| ImmunologyEntity::variants()[i])
    }

    proptest! {
        /// For any ImmunologyEntity that is an ImmuneCell, IsProInflammatory and
        /// IsProRepair are never both true simultaneously (M1/M2 mutual exclusivity).
        #[test]
        fn prop_immune_cell_m1_m2_mutual_exclusivity(entity in arb_immunology_entity()) {
            if taxonomy::is_a::<ImmunologyTaxonomy>(
                &entity,
                &ImmunologyEntity::ImmuneCell,
            ) && entity != ImmunologyEntity::ImmuneCell
            {
                let pro_inflam = IsProInflammatory.get(&entity);
                let pro_repair = IsProRepair.get(&entity);
                if let (Some(true), Some(true)) = (pro_inflam, pro_repair) {
                    prop_assert!(
                        false,
                        "ImmuneCell {:?} cannot be both pro-inflammatory and pro-repair",
                        entity
                    );
                }
            }
        }
    }

    // -- Literature axioms --

    /// Weinheimer-Haus 2014: 45 Hz WBV shifted macrophage polarization M1->M2
    /// in diabetic mouse wounds. MechanicalStimulation causes M1ToM2Transition.
    #[test]
    fn test_literature_weinheimer_haus_2014_wbv_m1_to_m2() {
        use ImmunologyEvent::*;
        let effects = causation::effects_of::<InflammationCauses>(&MechanicalStimulation);
        assert!(
            effects.contains(&M1ToM2Transition),
            "Weinheimer-Haus 2014: 45 Hz whole-body vibration must cause \
             M1-to-M2 macrophage polarization shift in diabetic mouse wounds"
        );
        // The transition should also eventually lead to repair
        assert!(
            effects.contains(&RepairCompletion),
            "Weinheimer-Haus 2014: WBV-induced M1->M2 shift should \
             transitively lead to repair completion"
        );
    }

    /// Yu 2019 PMID:31247969: WBV induced omental macrophage polarization shift,
    /// confirming vibration modulates immune response.
    #[test]
    fn test_literature_yu_2019_wbv_immune_modulation() {
        use ImmunologyEvent::*;
        // MechanicalStimulation causally reaches anti-inflammatory response
        let effects = causation::effects_of::<InflammationCauses>(&MechanicalStimulation);
        assert!(
            effects.contains(&AntiInflammatoryResponse),
            "Yu 2019: WBV must transitively cause anti-inflammatory response \
             (omental macrophage polarization shift)"
        );
        // The M1ToM2Transition event is modulable by vibration
        assert_eq!(
            IsModulableByVibration.get(&M1ToM2Transition),
            Some(true),
            "Yu 2019: M1-to-M2 transition must be modulable by vibration"
        );
    }
}
