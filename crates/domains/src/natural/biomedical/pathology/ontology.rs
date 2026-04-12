//! Disease pathology ontology — pure science of disease classification and progression.
//!
//! Models the ontology of disease states, staging, classifications, and
//! pathological processes. This is general pathology science, not specific
//! to any organ system.
//!
//! Key scientific facts encoded:
//! - Normal tissue -> acute injury -> chronic adaptation -> metaplasia -> dysplasia -> neoplasia
//! - Chronic adaptation can also lead to fibrosis -> stricture (fibrotic pathway)
//! - Dysplasia progresses through low-grade -> high-grade -> neoplastic transformation
//! - Metaplasia is reversible with intervention (Levin's bioelectric approach)
//! - Neoplasia is irreversible once established
//! - Depolarized Vmem (-15 mV) correlates with dysplasia/neoplasia
//! - Normal tissue maintains polarized Vmem (-50 mV)
//!
//! Key references:
//! - Levin 2014: bioelectric correlates of neoplastic transformation
//! - Chernet & Levin 2013: repolarization suppresses tumors
//! - Binns et al. 2019: bioelectric reversal of metaplasia

use pr4xis::category::Entity;
use pr4xis::define_dense_category;
use pr4xis::ontology::reasoning::causation::{self, CausalDef};
use pr4xis::ontology::reasoning::opposition::{self, OppositionDef};
use pr4xis::ontology::reasoning::taxonomy::{self, TaxonomyDef};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Pathology Entity
// ---------------------------------------------------------------------------

/// Every entity in the pathology ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum PathologyEntity {
    // Disease states
    /// Healthy tissue with normal morphology and Vmem.
    Normal,
    /// Acute tissue damage — reversible.
    AcuteInjury,
    /// Sustained tissue damage with ongoing insult.
    ChronicInjury,
    /// Replacement of one differentiated cell type by another.
    Metaplasia,
    /// Abnormal cell morphology and organization — premalignant.
    Dysplasia,
    /// Uncontrolled cell growth — malignant transformation.
    Neoplasia,
    /// Excessive extracellular matrix deposition replacing functional tissue.
    Fibrosis,
    /// Luminal narrowing from fibrotic remodeling.
    Stricture,

    // Staging
    /// Low-grade dysplasia — mild architectural distortion.
    LowGrade,
    /// High-grade dysplasia — severe distortion approaching carcinoma in situ.
    HighGrade,

    // Classifications
    /// Non-progressive, no malignant potential.
    Benign,
    /// Capable of progressing to malignancy.
    Premalignant,
    /// Invasive neoplastic growth.
    Malignant,

    // Processes
    /// Acute inflammatory response to injury.
    Inflammation,
    /// Tissue remodeling in response to chronic stress.
    CellularAdaptation,
    /// Disordered proliferation with loss of normal architecture.
    AtypicalGrowth,
    /// Breach of basement membrane — hallmark of malignancy.
    Invasion,

    // Abstract categories
    /// Abstract: a disease state.
    DiseaseState,
    /// Abstract: a staging level.
    Stage,
    /// Abstract: a disease classification.
    Classification,
    /// Abstract: a pathological process.
    PathologicalProcess,
}

// ---------------------------------------------------------------------------
// Taxonomy (is-a)
// ---------------------------------------------------------------------------

/// Subsumption hierarchy for pathology entities.
pub struct PathologyTaxonomy;

impl TaxonomyDef for PathologyTaxonomy {
    type Entity = PathologyEntity;

    fn relations() -> Vec<(PathologyEntity, PathologyEntity)> {
        use PathologyEntity::*;
        vec![
            // Disease states is-a DiseaseState
            (Normal, DiseaseState),
            (AcuteInjury, DiseaseState),
            (ChronicInjury, DiseaseState),
            (Metaplasia, DiseaseState),
            (Dysplasia, DiseaseState),
            (Neoplasia, DiseaseState),
            (Fibrosis, DiseaseState),
            (Stricture, DiseaseState),
            // Stages is-a Stage
            (LowGrade, Stage),
            (HighGrade, Stage),
            // Classifications is-a Classification
            (Benign, Classification),
            (Premalignant, Classification),
            (Malignant, Classification),
            // Processes is-a PathologicalProcess
            (Inflammation, PathologicalProcess),
            (CellularAdaptation, PathologicalProcess),
            (AtypicalGrowth, PathologicalProcess),
            (Invasion, PathologicalProcess),
        ]
    }
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

define_dense_category! {
    /// Discrete category over pathology entities.
    pub PathologyCategory {
        entity: PathologyEntity,
        relation: PathologyRelation,
    }
}

// ---------------------------------------------------------------------------
// Causal graph
// ---------------------------------------------------------------------------

/// Events in the disease progression causal chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum PathologyCausalEvent {
    /// Initial tissue damage from exogenous or endogenous insult.
    TissueInsult,
    /// Acute inflammatory and repair response.
    AcuteResponse,
    /// Chronic tissue remodeling under sustained insult.
    ChronicAdaptation,
    /// Cellular phenotype switch (e.g. squamous -> columnar).
    MetaplasticTransformation,
    /// Acquisition of dysplastic features.
    DysplasticProgression,
    /// Transition from dysplasia to carcinoma.
    NeoplasticTransformation,
    /// Excessive collagen deposition and scarring.
    FibroticRemodeling,
    /// Luminal narrowing from fibrosis.
    StrictureFormation,
    /// Mild dysplastic changes.
    LowGradeProgression,
    /// Severe dysplastic changes approaching carcinoma in situ.
    HighGradeProgression,
}

/// Causal graph for disease progression.
///
/// Main pathway: TissueInsult -> AcuteResponse -> ChronicAdaptation
///               -> MetaplasticTransformation -> DysplasticProgression
///               -> NeoplasticTransformation
///
/// Fibrotic branch: ChronicAdaptation -> FibroticRemodeling -> StrictureFormation
///
/// Dysplasia staging: DysplasticProgression -> LowGradeProgression
///                    -> HighGradeProgression -> NeoplasticTransformation
pub struct DiseaseProgressionCauses;

impl CausalDef for DiseaseProgressionCauses {
    type Entity = PathologyCausalEvent;

    fn relations() -> Vec<(PathologyCausalEvent, PathologyCausalEvent)> {
        use PathologyCausalEvent::*;
        vec![
            // Main progression pathway
            (TissueInsult, AcuteResponse),
            (AcuteResponse, ChronicAdaptation),
            (ChronicAdaptation, MetaplasticTransformation),
            (MetaplasticTransformation, DysplasticProgression),
            (DysplasticProgression, NeoplasticTransformation),
            // Fibrotic branch
            (ChronicAdaptation, FibroticRemodeling),
            (FibroticRemodeling, StrictureFormation),
            // Dysplasia staging branch
            (DysplasticProgression, LowGradeProgression),
            (LowGradeProgression, HighGradeProgression),
            (HighGradeProgression, NeoplasticTransformation),
        ]
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: is this disease state reversible?
#[derive(Debug, Clone)]
pub struct IsReversible;

impl Quality for IsReversible {
    type Individual = PathologyEntity;
    type Value = bool;

    fn get(&self, individual: &PathologyEntity) -> Option<bool> {
        use PathologyEntity::*;
        match individual {
            Normal => Some(true),        // trivially — already healthy
            AcuteInjury => Some(true),   // heals with removal of insult
            ChronicInjury => Some(true), // can resolve if insult removed
            Metaplasia => Some(true),    // reversible with intervention (Levin bioelectric)
            Dysplasia => Some(true),     // low-grade can regress; high-grade less so
            Neoplasia => Some(false),    // irreversible malignant transformation
            Fibrosis => Some(false),     // partially — scarring is largely permanent
            Stricture => Some(false),    // structural narrowing requires intervention
            _ => None,
        }
    }
}

/// Malignant potential classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MalignantPotentialLevel {
    None,
    Low,
    High,
    IsMalignant,
}

/// Quality: what is the malignant potential of this disease state?
#[derive(Debug, Clone)]
pub struct MalignantPotential;

impl Quality for MalignantPotential {
    type Individual = PathologyEntity;
    type Value = MalignantPotentialLevel;

    fn get(&self, individual: &PathologyEntity) -> Option<MalignantPotentialLevel> {
        use MalignantPotentialLevel::*;
        use PathologyEntity as P;
        match individual {
            P::Normal => Some(None),
            P::AcuteInjury => Some(None),
            P::ChronicInjury => Some(Low),
            P::Metaplasia => Some(Low),
            P::Dysplasia => Some(High),
            P::Neoplasia => Some(IsMalignant),
            P::Fibrosis => Some(None),
            P::Stricture => Some(None),
            _ => Option::None,
        }
    }
}

/// Quality: does this state require clinical intervention?
#[derive(Debug, Clone)]
pub struct RequiresIntervention;

impl Quality for RequiresIntervention {
    type Individual = PathologyEntity;
    type Value = bool;

    fn get(&self, individual: &PathologyEntity) -> Option<bool> {
        use PathologyEntity::*;
        match individual {
            Normal => Some(false),
            AcuteInjury => Some(false),  // typically self-resolving
            ChronicInjury => Some(true), // remove insult source
            Metaplasia => Some(true),    // surveillance + bioelectric intervention
            Dysplasia => Some(true),     // active treatment required
            Neoplasia => Some(true),     // definitive treatment required
            Fibrosis => Some(true),      // anti-fibrotic therapy
            Stricture => Some(true),     // dilation or surgical intervention
            _ => None,
        }
    }
}

/// Quality: bioelectric correlate (Vmem in mV) associated with each disease state.
/// Normal tissue is polarized (~-50 mV), dysplastic/neoplastic tissue is depolarized (~-15 mV).
#[derive(Debug, Clone)]
pub struct BioelectricCorrelate;

impl Quality for BioelectricCorrelate {
    type Individual = PathologyEntity;
    type Value = f64;

    fn get(&self, individual: &PathologyEntity) -> Option<f64> {
        use PathologyEntity::*;
        match individual {
            Normal => Some(-50.0),        // healthy polarized Vmem
            AcuteInjury => Some(-30.0),   // transient depolarization from injury
            ChronicInjury => Some(-25.0), // sustained partial depolarization
            Metaplasia => Some(-25.0),    // intermediate depolarization
            Dysplasia => Some(-15.0),     // depolarized — Levin's neoplastic signature
            Neoplasia => Some(-10.0),     // strongly depolarized
            Fibrosis => Some(-35.0),      // mild depolarization
            Stricture => Some(-35.0),     // structural — mild depolarization
            _ => None,
        }
    }
}

/// Quality: Barrett's esophagus staging mapping (relevant but general pathology concept).
/// Maps disease states to Barrett's staging terminology where applicable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarrettsStageLevel {
    /// No Barrett's — normal squamous epithelium.
    NoBarretts,
    /// Non-dysplastic Barrett's (intestinal metaplasia).
    NonDysplastic,
    /// Barrett's with low-grade dysplasia.
    BarrettsLGD,
    /// Barrett's with high-grade dysplasia.
    BarrettsHGD,
    /// Esophageal adenocarcinoma arising from Barrett's.
    Adenocarcinoma,
}

/// Quality: optional Barrett's esophagus stage for each disease state.
#[derive(Debug, Clone)]
pub struct BarrettsStage;

impl Quality for BarrettsStage {
    type Individual = PathologyEntity;
    type Value = BarrettsStageLevel;

    fn get(&self, individual: &PathologyEntity) -> Option<BarrettsStageLevel> {
        use BarrettsStageLevel::*;
        use PathologyEntity as P;
        match individual {
            P::Normal => Some(NoBarretts),
            P::Metaplasia => Some(NonDysplastic),
            P::Dysplasia => Some(BarrettsLGD), // default to LGD; staging refines
            P::Neoplasia => Some(Adenocarcinoma),
            P::LowGrade => Some(BarrettsLGD),
            P::HighGrade => Some(BarrettsHGD),
            _ => Option::None,
        }
    }
}

// ---------------------------------------------------------------------------
// Opposition (semantic contrasts)
// ---------------------------------------------------------------------------

/// Opposition pairs in pathology.
///
/// - Normal vs Neoplasia: health vs disease endpoint
/// - Benign vs Malignant: non-progressive vs invasive
/// - LowGrade vs HighGrade: mild vs severe dysplasia
/// - Inflammation vs CellularAdaptation: acute vs chronic response
pub struct PathologyOpposition;

impl OppositionDef for PathologyOpposition {
    type Entity = PathologyEntity;

    fn pairs() -> Vec<(PathologyEntity, PathologyEntity)> {
        use PathologyEntity::*;
        vec![
            (Normal, Neoplasia),
            (Benign, Malignant),
            (LowGrade, HighGrade),
            (Inflammation, CellularAdaptation),
        ]
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Axiom: pathology taxonomy is a directed acyclic graph.
pub struct PathologyTaxonomyIsDAG;

impl Axiom for PathologyTaxonomyIsDAG {
    fn description(&self) -> &str {
        "pathology taxonomy is a directed acyclic graph"
    }

    fn holds(&self) -> bool {
        taxonomy::NoCycles::<PathologyTaxonomy>::new().holds()
    }
}

/// Axiom: disease progression causal graph is asymmetric.
pub struct DiseaseProgressionCausalAsymmetric;

impl Axiom for DiseaseProgressionCausalAsymmetric {
    fn description(&self) -> &str {
        "disease progression causal graph is asymmetric"
    }

    fn holds(&self) -> bool {
        causation::Asymmetric::<DiseaseProgressionCauses>::new().holds()
    }
}

/// Axiom: no disease progression event directly causes itself.
pub struct DiseaseProgressionNoSelfCausation;

impl Axiom for DiseaseProgressionNoSelfCausation {
    fn description(&self) -> &str {
        "no disease progression event directly causes itself"
    }

    fn holds(&self) -> bool {
        causation::NoSelfCausation::<DiseaseProgressionCauses>::new().holds()
    }
}

/// Axiom: tissue insult transitively causes neoplastic transformation (full progression).
pub struct TissueInsultCausesNeoplasia;

impl Axiom for TissueInsultCausesNeoplasia {
    fn description(&self) -> &str {
        "tissue insult transitively causes neoplastic transformation"
    }

    fn holds(&self) -> bool {
        use PathologyCausalEvent::*;
        let effects = causation::effects_of::<DiseaseProgressionCauses>(&TissueInsult);
        effects.contains(&NeoplasticTransformation)
    }
}

/// Axiom: tissue insult also causes stricture formation (fibrotic pathway).
pub struct TissueInsultCausesStricture;

impl Axiom for TissueInsultCausesStricture {
    fn description(&self) -> &str {
        "tissue insult transitively causes stricture formation (fibrotic pathway)"
    }

    fn holds(&self) -> bool {
        use PathologyCausalEvent::*;
        let effects = causation::effects_of::<DiseaseProgressionCauses>(&TissueInsult);
        effects.contains(&StrictureFormation)
    }
}

/// Axiom: dysplasia is premalignant.
pub struct DysplasiaIsPremalignant;

impl Axiom for DysplasiaIsPremalignant {
    fn description(&self) -> &str {
        "dysplasia has high malignant potential (premalignant)"
    }

    fn holds(&self) -> bool {
        MalignantPotential.get(&PathologyEntity::Dysplasia) == Some(MalignantPotentialLevel::High)
    }
}

/// Axiom: normal tissue has no malignant potential.
pub struct NormalHasNoMalignantPotential;

impl Axiom for NormalHasNoMalignantPotential {
    fn description(&self) -> &str {
        "normal tissue has no malignant potential"
    }

    fn holds(&self) -> bool {
        MalignantPotential.get(&PathologyEntity::Normal) == Some(MalignantPotentialLevel::None)
    }
}

/// Axiom: neoplasia is malignant.
pub struct NeoplasiaIsMalignant;

impl Axiom for NeoplasiaIsMalignant {
    fn description(&self) -> &str {
        "neoplasia is malignant"
    }

    fn holds(&self) -> bool {
        MalignantPotential.get(&PathologyEntity::Neoplasia)
            == Some(MalignantPotentialLevel::IsMalignant)
    }
}

/// Axiom: metaplasia is reversible (with intervention — Levin's bioelectric approach).
pub struct MetaplasiaIsReversible;

impl Axiom for MetaplasiaIsReversible {
    fn description(&self) -> &str {
        "metaplasia is reversible with intervention"
    }

    fn holds(&self) -> bool {
        IsReversible.get(&PathologyEntity::Metaplasia) == Some(true)
    }
}

/// Axiom: pathology opposition is symmetric.
pub struct PathologyOppositionSymmetric;

impl Axiom for PathologyOppositionSymmetric {
    fn description(&self) -> &str {
        "pathology opposition is symmetric"
    }

    fn holds(&self) -> bool {
        opposition::Symmetric::<PathologyOpposition>::new().holds()
    }
}

/// Axiom: pathology opposition is irreflexive (nothing opposes itself).
pub struct PathologyOppositionIrreflexive;

impl Axiom for PathologyOppositionIrreflexive {
    fn description(&self) -> &str {
        "pathology opposition is irreflexive"
    }

    fn holds(&self) -> bool {
        opposition::Irreflexive::<PathologyOpposition>::new().holds()
    }
}

/// Axiom: acute injury is reversible, neoplasia is not.
pub struct AcuteReversibleNeoplasiaIrreversible;

impl Axiom for AcuteReversibleNeoplasiaIrreversible {
    fn description(&self) -> &str {
        "acute injury is reversible but neoplasia is irreversible"
    }

    fn holds(&self) -> bool {
        IsReversible.get(&PathologyEntity::AcuteInjury) == Some(true)
            && IsReversible.get(&PathologyEntity::Neoplasia) == Some(false)
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// Top-level pathology ontology tying together category, qualities, and axioms.
pub struct PathologyOntology;

impl Ontology for PathologyOntology {
    type Cat = PathologyCategory;
    type Qual = IsReversible;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(PathologyTaxonomyIsDAG),
            Box::new(DiseaseProgressionCausalAsymmetric),
            Box::new(DiseaseProgressionNoSelfCausation),
            Box::new(TissueInsultCausesNeoplasia),
            Box::new(TissueInsultCausesStricture),
            Box::new(DysplasiaIsPremalignant),
            Box::new(NormalHasNoMalignantPotential),
            Box::new(NeoplasiaIsMalignant),
            Box::new(MetaplasiaIsReversible),
            Box::new(PathologyOppositionSymmetric),
            Box::new(PathologyOppositionIrreflexive),
            Box::new(AcuteReversibleNeoplasiaIrreversible),
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
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;

    // -- Axiom tests --

    #[test]
    fn test_pathology_taxonomy_is_dag() {
        assert!(
            PathologyTaxonomyIsDAG.holds(),
            "{}",
            PathologyTaxonomyIsDAG.description()
        );
    }

    #[test]
    fn test_disease_progression_causal_asymmetric() {
        assert!(
            DiseaseProgressionCausalAsymmetric.holds(),
            "{}",
            DiseaseProgressionCausalAsymmetric.description()
        );
    }

    #[test]
    fn test_disease_progression_no_self_causation() {
        assert!(
            DiseaseProgressionNoSelfCausation.holds(),
            "{}",
            DiseaseProgressionNoSelfCausation.description()
        );
    }

    #[test]
    fn test_tissue_insult_causes_neoplasia() {
        assert!(
            TissueInsultCausesNeoplasia.holds(),
            "{}",
            TissueInsultCausesNeoplasia.description()
        );
    }

    #[test]
    fn test_tissue_insult_causes_stricture() {
        assert!(
            TissueInsultCausesStricture.holds(),
            "{}",
            TissueInsultCausesStricture.description()
        );
    }

    #[test]
    fn test_dysplasia_is_premalignant() {
        assert!(
            DysplasiaIsPremalignant.holds(),
            "{}",
            DysplasiaIsPremalignant.description()
        );
    }

    #[test]
    fn test_normal_has_no_malignant_potential() {
        assert!(
            NormalHasNoMalignantPotential.holds(),
            "{}",
            NormalHasNoMalignantPotential.description()
        );
    }

    #[test]
    fn test_neoplasia_is_malignant() {
        assert!(
            NeoplasiaIsMalignant.holds(),
            "{}",
            NeoplasiaIsMalignant.description()
        );
    }

    #[test]
    fn test_metaplasia_is_reversible() {
        assert!(
            MetaplasiaIsReversible.holds(),
            "{}",
            MetaplasiaIsReversible.description()
        );
    }

    #[test]
    fn test_pathology_opposition_symmetric() {
        assert!(
            PathologyOppositionSymmetric.holds(),
            "{}",
            PathologyOppositionSymmetric.description()
        );
    }

    #[test]
    fn test_pathology_opposition_irreflexive() {
        assert!(
            PathologyOppositionIrreflexive.holds(),
            "{}",
            PathologyOppositionIrreflexive.description()
        );
    }

    #[test]
    fn test_acute_reversible_neoplasia_irreversible() {
        assert!(
            AcuteReversibleNeoplasiaIrreversible.holds(),
            "{}",
            AcuteReversibleNeoplasiaIrreversible.description()
        );
    }

    // -- Category law tests --

    #[test]
    fn test_pathology_category_laws() {
        check_category_laws::<PathologyCategory>().unwrap();
    }

    #[test]
    fn test_pathology_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<PathologyTaxonomy>>().unwrap();
    }

    // -- Opposition tests --

    #[test]
    fn test_normal_opposes_neoplasia() {
        use PathologyEntity::*;
        assert!(opposition::are_opposed::<PathologyOpposition>(
            &Normal, &Neoplasia
        ));
        assert!(opposition::are_opposed::<PathologyOpposition>(
            &Neoplasia, &Normal
        ));
    }

    #[test]
    fn test_benign_opposes_malignant() {
        use PathologyEntity::*;
        assert!(opposition::are_opposed::<PathologyOpposition>(
            &Benign, &Malignant
        ));
        assert!(opposition::are_opposed::<PathologyOpposition>(
            &Malignant, &Benign
        ));
    }

    #[test]
    fn test_lowgrade_opposes_highgrade() {
        use PathologyEntity::*;
        assert!(opposition::are_opposed::<PathologyOpposition>(
            &LowGrade, &HighGrade
        ));
    }

    #[test]
    fn test_inflammation_opposes_cellular_adaptation() {
        use PathologyEntity::*;
        assert!(opposition::are_opposed::<PathologyOpposition>(
            &Inflammation,
            &CellularAdaptation
        ));
    }

    #[test]
    fn test_normal_does_not_oppose_benign() {
        use PathologyEntity::*;
        assert!(!opposition::are_opposed::<PathologyOpposition>(
            &Normal, &Benign
        ));
    }

    // -- Causal chain tests --

    #[test]
    fn test_full_progression_chain() {
        use PathologyCausalEvent::*;
        let effects = causation::effects_of::<DiseaseProgressionCauses>(&TissueInsult);
        assert!(effects.contains(&AcuteResponse));
        assert!(effects.contains(&ChronicAdaptation));
        assert!(effects.contains(&MetaplasticTransformation));
        assert!(effects.contains(&DysplasticProgression));
        assert!(effects.contains(&NeoplasticTransformation));
    }

    #[test]
    fn test_fibrotic_pathway() {
        use PathologyCausalEvent::*;
        let effects = causation::effects_of::<DiseaseProgressionCauses>(&ChronicAdaptation);
        assert!(effects.contains(&FibroticRemodeling));
        assert!(effects.contains(&StrictureFormation));
    }

    #[test]
    fn test_dysplasia_staging_chain() {
        use PathologyCausalEvent::*;
        let effects = causation::effects_of::<DiseaseProgressionCauses>(&DysplasticProgression);
        assert!(effects.contains(&LowGradeProgression));
        assert!(effects.contains(&HighGradeProgression));
        assert!(effects.contains(&NeoplasticTransformation));
    }

    #[test]
    fn test_causal_event_count() {
        assert_eq!(PathologyCausalEvent::variants().len(), 10);
    }

    #[test]
    fn test_causal_category_laws() {
        use pr4xis::ontology::reasoning::causation::CausalCategory;
        check_category_laws::<CausalCategory<DiseaseProgressionCauses>>().unwrap();
    }

    // -- Taxonomy tests --

    #[test]
    fn test_disease_states_are_disease_states() {
        use PathologyEntity::*;
        for state in [
            Normal,
            AcuteInjury,
            ChronicInjury,
            Metaplasia,
            Dysplasia,
            Neoplasia,
            Fibrosis,
            Stricture,
        ] {
            assert!(
                taxonomy::is_a::<PathologyTaxonomy>(&state, &DiseaseState),
                "{:?} should be a DiseaseState",
                state
            );
        }
    }

    #[test]
    fn test_classifications_are_classifications() {
        use PathologyEntity::*;
        for cls in [Benign, Premalignant, Malignant] {
            assert!(
                taxonomy::is_a::<PathologyTaxonomy>(&cls, &Classification),
                "{:?} should be a Classification",
                cls
            );
        }
    }

    #[test]
    fn test_processes_are_processes() {
        use PathologyEntity::*;
        for proc in [Inflammation, CellularAdaptation, AtypicalGrowth, Invasion] {
            assert!(
                taxonomy::is_a::<PathologyTaxonomy>(&proc, &PathologicalProcess),
                "{:?} should be a PathologicalProcess",
                proc
            );
        }
    }

    #[test]
    fn test_disease_state_descendants_count() {
        let descendants =
            taxonomy::descendants::<PathologyTaxonomy>(&PathologyEntity::DiseaseState);
        assert_eq!(descendants.len(), 8);
    }

    #[test]
    fn test_entity_count() {
        assert_eq!(PathologyEntity::variants().len(), 21);
    }

    // -- Quality tests --

    #[test]
    fn test_bioelectric_correlate_normal_polarized() {
        let vmem = BioelectricCorrelate.get(&PathologyEntity::Normal).unwrap();
        assert!(
            vmem < -40.0,
            "normal tissue should be polarized (< -40 mV), got {}",
            vmem
        );
    }

    #[test]
    fn test_bioelectric_correlate_dysplasia_depolarized() {
        let vmem = BioelectricCorrelate
            .get(&PathologyEntity::Dysplasia)
            .unwrap();
        assert!(
            vmem > -20.0,
            "dysplastic tissue should be depolarized (> -20 mV), got {}",
            vmem
        );
    }

    #[test]
    fn test_bioelectric_correlate_neoplasia_depolarized() {
        let vmem = BioelectricCorrelate
            .get(&PathologyEntity::Neoplasia)
            .unwrap();
        assert!(
            vmem > -15.0,
            "neoplastic tissue should be strongly depolarized, got {}",
            vmem
        );
    }

    #[test]
    fn test_barrets_stage_normal() {
        assert_eq!(
            BarrettsStage.get(&PathologyEntity::Normal),
            Some(BarrettsStageLevel::NoBarretts)
        );
    }

    #[test]
    fn test_barrets_stage_metaplasia() {
        assert_eq!(
            BarrettsStage.get(&PathologyEntity::Metaplasia),
            Some(BarrettsStageLevel::NonDysplastic)
        );
    }

    #[test]
    fn test_barrets_stage_neoplasia() {
        assert_eq!(
            BarrettsStage.get(&PathologyEntity::Neoplasia),
            Some(BarrettsStageLevel::Adenocarcinoma)
        );
    }

    // -- Ontology validation --

    #[test]
    fn test_ontology_validates() {
        PathologyOntology::validate().unwrap();
    }

    // -- Property-based tests (proptest) --

    use proptest::prelude::*;

    fn arb_pathology_entity() -> impl Strategy<Value = PathologyEntity> {
        (0..PathologyEntity::variants().len()).prop_map(|i| PathologyEntity::variants()[i])
    }

    proptest! {
        /// For any disease state, malignant potential is defined.
        #[test]
        fn prop_malignant_potential_defined_for_disease_states(entity in arb_pathology_entity()) {
            if taxonomy::is_a::<PathologyTaxonomy>(&entity, &PathologyEntity::DiseaseState)
                && entity != PathologyEntity::DiseaseState
            {
                prop_assert!(
                    MalignantPotential.get(&entity).is_some(),
                    "MalignantPotential should be defined for disease state {:?}",
                    entity
                );
            }
        }

        /// Taxonomy is-a is reflexive for all entities.
        #[test]
        fn prop_taxonomy_reflexive(entity in arb_pathology_entity()) {
            prop_assert!(taxonomy::is_a::<PathologyTaxonomy>(&entity, &entity));
        }

        /// For any disease state, IsReversible is defined.
        #[test]
        fn prop_reversibility_defined_for_disease_states(entity in arb_pathology_entity()) {
            if taxonomy::is_a::<PathologyTaxonomy>(&entity, &PathologyEntity::DiseaseState)
                && entity != PathologyEntity::DiseaseState
            {
                prop_assert!(
                    IsReversible.get(&entity).is_some(),
                    "IsReversible should be defined for disease state {:?}",
                    entity
                );
            }
        }
    }
}
