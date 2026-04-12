//! Biochemistry ontology.
//!
//! Entities: signaling molecules, biochemical processes, energy metabolites.
//! Taxonomy: molecule type hierarchy (signaling, process, metabolite).
//! Causal graph: calcium entry -> calmodulin -> CaMKII -> CREB -> gene expression.
//!
//! Key references:
//! - Bhatt 2000: CaMKII activation by calmodulin
//! - Sheng & Greenberg 1990: CREB phosphorylation and gene expression
//! - Bhargava 2012: cAMP/IP3 as second messengers
//! - Ignarro 1987: nitric oxide as signaling molecule
//! - Krebs 1957: ATP in phosphorylation cascades

use pr4xis::category::Entity;
use pr4xis::define_dense_category;
use pr4xis::ontology::reasoning::causation::{self, CausalDef};
use pr4xis::ontology::reasoning::opposition::{self, OppositionDef};
use pr4xis::ontology::reasoning::taxonomy::{self, TaxonomyDef};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

/// Every biochemistry entity in the bioelectric repair signaling domain.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum BiochemistryEntity {
    // Signaling molecules
    CalciumIon,
    Calmodulin,
    CaMKII,
    ProteinKinaseC,
    CREB,
    NitricOxide,
    // Second messengers
    CAMP,
    IP3,
    // Processes
    SignalTransduction,
    PhosphorylationCascade,
    GeneTranscription,
    ProteinSynthesis,
    SecondMessenger,
    // Metabolic
    ATP,
    ADP,
    Glycolysis,
    OxidativePhosphorylation,
    // Abstract categories
    SignalingMolecule,
    BiochemicalProcess,
    EnergyMetabolite,
}

// ---------------------------------------------------------------------------
// Taxonomy (is-a)
// ---------------------------------------------------------------------------

/// Subsumption hierarchy for biochemistry entities.
pub struct BiochemistryTaxonomy;

impl TaxonomyDef for BiochemistryTaxonomy {
    type Entity = BiochemistryEntity;

    fn relations() -> Vec<(BiochemistryEntity, BiochemistryEntity)> {
        use BiochemistryEntity::*;
        vec![
            // Signaling molecules is-a SignalingMolecule
            (CalciumIon, SignalingMolecule),
            (Calmodulin, SignalingMolecule),
            (CaMKII, SignalingMolecule),
            (CREB, SignalingMolecule),
            (NitricOxide, SignalingMolecule),
            (CAMP, SignalingMolecule),
            (IP3, SignalingMolecule),
            // Processes is-a BiochemicalProcess
            (SignalTransduction, BiochemicalProcess),
            (PhosphorylationCascade, BiochemicalProcess),
            (GeneTranscription, BiochemicalProcess),
            (ProteinSynthesis, BiochemicalProcess),
            (SecondMessenger, BiochemicalProcess),
            // Metabolites is-a EnergyMetabolite
            (ATP, EnergyMetabolite),
            (ADP, EnergyMetabolite),
            // Metabolic processes is-a BiochemicalProcess
            (Glycolysis, BiochemicalProcess),
            (OxidativePhosphorylation, BiochemicalProcess),
            // ProteinKinaseC is a signaling molecule
            (ProteinKinaseC, SignalingMolecule),
        ]
    }
}

// ---------------------------------------------------------------------------
// Causal graph
// ---------------------------------------------------------------------------

/// Causal events in biochemical signaling cascades relevant to bioelectric repair.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum BiochemistryCausalEvent {
    /// Calcium enters cell through ion channels
    CalciumEntry,
    /// Calmodulin binds calcium and becomes active
    CalmodulinActivation,
    /// CaMKII is phosphorylated by active calmodulin
    CaMKIIPhosphorylation,
    /// CREB transcription factor is activated
    CREBActivation,
    /// Gene expression changes downstream of CREB
    GeneExpressionChange,
    /// Protein synthesis altered by gene expression
    ProteinSynthesisChange,
    /// PKC activated by calcium/diacylglycerol
    PKCActivation,
    /// Downstream signaling from PKC
    DownstreamSignaling,
    /// Nitric oxide synthase activated by calcium
    NOSynthaseActivation,
    /// NO produced, causing vasodilation
    NOProduction,
    /// ATP hydrolyzed to ADP
    ATPHydrolysis,
    /// Energy released from ATP hydrolysis
    EnergyRelease,
}

/// Causal graph: biochemical signaling cascades.
///
/// Main pathway: CalciumEntry -> CalmodulinActivation -> CaMKIIPhosphorylation
///               -> CREBActivation -> GeneExpressionChange -> ProteinSynthesisChange
/// PKC branch:   CalciumEntry -> PKCActivation -> DownstreamSignaling
/// NO branch:    CalciumEntry -> NOSynthaseActivation -> NOProduction (vasodilation)
/// Energy:       ATPHydrolysis -> EnergyRelease
pub struct BiochemistryCauses;

impl CausalDef for BiochemistryCauses {
    type Entity = BiochemistryCausalEvent;

    fn relations() -> Vec<(BiochemistryCausalEvent, BiochemistryCausalEvent)> {
        use BiochemistryCausalEvent::*;
        vec![
            // Main calcium-calmodulin-CaMKII-CREB pathway
            (CalciumEntry, CalmodulinActivation),
            (CalmodulinActivation, CaMKIIPhosphorylation),
            (CaMKIIPhosphorylation, CREBActivation),
            (CREBActivation, GeneExpressionChange),
            (GeneExpressionChange, ProteinSynthesisChange),
            // PKC branch
            (CalciumEntry, PKCActivation),
            (PKCActivation, DownstreamSignaling),
            // Nitric oxide / vasodilation branch
            (CalciumEntry, NOSynthaseActivation),
            (NOSynthaseActivation, NOProduction),
            // Energy metabolism
            (ATPHydrolysis, EnergyRelease),
        ]
    }
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

define_dense_category! {
    /// Discrete category over biochemistry entities.
    ///
    /// Every entity pair has a unique morphism; composition is transitive.
    pub BiochemistryCategory {
        entity: BiochemistryEntity,
        relation: BiochemistryRelation,
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: is this entity a second messenger?
#[derive(Debug, Clone)]
pub struct IsSecondMessenger;

impl Quality for IsSecondMessenger {
    type Individual = BiochemistryEntity;
    type Value = bool;

    fn get(&self, individual: &BiochemistryEntity) -> Option<bool> {
        use BiochemistryEntity::*;
        match individual {
            CalciumIon | CAMP | IP3 | NitricOxide => Some(true),
            Calmodulin | CaMKII | ProteinKinaseC | CREB => Some(false),
            ATP | ADP => Some(false),
            _ => None,
        }
    }
}

/// Quality: is this entity a kinase?
#[derive(Debug, Clone)]
pub struct IsKinase;

impl Quality for IsKinase {
    type Individual = BiochemistryEntity;
    type Value = bool;

    fn get(&self, individual: &BiochemistryEntity) -> Option<bool> {
        use BiochemistryEntity::*;
        match individual {
            CaMKII | ProteinKinaseC => Some(true),
            CalciumIon | Calmodulin | CREB | NitricOxide | CAMP | IP3 => Some(false),
            ATP | ADP => Some(false),
            _ => None,
        }
    }
}

/// Quality: does this process require ATP?
#[derive(Debug, Clone)]
pub struct RequiresATP;

impl Quality for RequiresATP {
    type Individual = BiochemistryEntity;
    type Value = bool;

    fn get(&self, individual: &BiochemistryEntity) -> Option<bool> {
        use BiochemistryEntity::*;
        match individual {
            PhosphorylationCascade | ProteinSynthesis => Some(true),
            SignalTransduction | GeneTranscription | SecondMessenger => Some(false),
            Glycolysis | OxidativePhosphorylation => Some(false),
            _ => None,
        }
    }
}

/// Time scale of biochemical processes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeScale {
    /// Sub-second events (ion flux, channel gating)
    Milliseconds,
    /// Seconds-range events (enzyme activation)
    Seconds,
    /// Minutes-range events (signaling cascades)
    Minutes,
    /// Hours-range events (gene expression, protein synthesis)
    Hours,
}

/// Quality: what is the characteristic time scale of this process?
#[derive(Debug, Clone)]
pub struct ProcessTimeScale;

impl Quality for ProcessTimeScale {
    type Individual = BiochemistryEntity;
    type Value = TimeScale;

    fn get(&self, individual: &BiochemistryEntity) -> Option<TimeScale> {
        use BiochemistryEntity::*;
        match individual {
            SignalTransduction => Some(TimeScale::Milliseconds),
            PhosphorylationCascade => Some(TimeScale::Seconds),
            SecondMessenger => Some(TimeScale::Seconds),
            GeneTranscription => Some(TimeScale::Hours),
            ProteinSynthesis => Some(TimeScale::Hours),
            Glycolysis => Some(TimeScale::Minutes),
            OxidativePhosphorylation => Some(TimeScale::Minutes),
            _ => None,
        }
    }
}

/// Quality: is this process reversible on a short time scale?
#[derive(Debug, Clone)]
pub struct IsReversible;

impl Quality for IsReversible {
    type Individual = BiochemistryEntity;
    type Value = bool;

    fn get(&self, individual: &BiochemistryEntity) -> Option<bool> {
        use BiochemistryEntity::*;
        match individual {
            PhosphorylationCascade => Some(true), // phosphatases reverse kinases
            SignalTransduction => Some(true),     // signal can be terminated
            SecondMessenger => Some(true),        // degraded by phosphodiesterases
            GeneTranscription => Some(false),     // slow to reverse once initiated
            ProteinSynthesis => Some(false),      // proteins must be degraded
            Glycolysis => Some(false),            // irreversible committed steps
            OxidativePhosphorylation => Some(false), // directional electron transport
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Opposition (semantic contrasts)
// ---------------------------------------------------------------------------

/// Opposition pairs in the biochemistry domain.
///
/// - ATP <-> ADP: charged vs discharged energy currency
/// - Glycolysis <-> OxidativePhosphorylation: anaerobic vs aerobic
/// - PhosphorylationCascade <-> GeneTranscription: fast signaling vs slow expression
pub struct BiochemistryOpposition;

impl OppositionDef for BiochemistryOpposition {
    type Entity = BiochemistryEntity;

    fn pairs() -> Vec<(BiochemistryEntity, BiochemistryEntity)> {
        use BiochemistryEntity::*;
        vec![
            (ATP, ADP),
            (Glycolysis, OxidativePhosphorylation),
            (PhosphorylationCascade, GeneTranscription),
        ]
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// The biochemistry taxonomy has no cycles (is a DAG).
pub struct BiochemistryTaxonomyIsDAG;

impl Axiom for BiochemistryTaxonomyIsDAG {
    fn description(&self) -> &str {
        "biochemistry taxonomy is a directed acyclic graph"
    }

    fn holds(&self) -> bool {
        taxonomy::NoCycles::<BiochemistryTaxonomy>::new().holds()
    }
}

/// The causal graph is asymmetric: if A causes B then B does not cause A.
pub struct BiochemistryCausalAsymmetric;

impl Axiom for BiochemistryCausalAsymmetric {
    fn description(&self) -> &str {
        "biochemistry causal graph is asymmetric"
    }

    fn holds(&self) -> bool {
        causation::Asymmetric::<BiochemistryCauses>::new().holds()
    }
}

/// No event directly causes itself.
pub struct BiochemistryCausalNoSelfCause;

impl Axiom for BiochemistryCausalNoSelfCause {
    fn description(&self) -> &str {
        "no biochemical event directly causes itself"
    }

    fn holds(&self) -> bool {
        causation::NoSelfCausation::<BiochemistryCauses>::new().holds()
    }
}

/// CalciumEntry transitively causes GeneExpressionChange (the full signaling cascade).
pub struct CalciumEntryCausesGeneExpression;

impl Axiom for CalciumEntryCausesGeneExpression {
    fn description(&self) -> &str {
        "calcium entry transitively causes gene expression change"
    }

    fn holds(&self) -> bool {
        use BiochemistryCausalEvent::*;
        let effects = causation::effects_of::<BiochemistryCauses>(&CalciumEntry);
        effects.contains(&GeneExpressionChange)
    }
}

/// CalciumEntry also causes NOProduction (vasodilation pathway).
pub struct CalciumEntryCausesNOProduction;

impl Axiom for CalciumEntryCausesNOProduction {
    fn description(&self) -> &str {
        "calcium entry causes NO production via NOS activation"
    }

    fn holds(&self) -> bool {
        use BiochemistryCausalEvent::*;
        let effects = causation::effects_of::<BiochemistryCauses>(&CalciumEntry);
        effects.contains(&NOSynthaseActivation) && effects.contains(&NOProduction)
    }
}

/// Calcium is a second messenger.
pub struct CalciumIsSecondMessenger;

impl Axiom for CalciumIsSecondMessenger {
    fn description(&self) -> &str {
        "calcium ion is a second messenger"
    }

    fn holds(&self) -> bool {
        IsSecondMessenger.get(&BiochemistryEntity::CalciumIon) == Some(true)
    }
}

/// CaMKII is a kinase.
pub struct CaMKIIIsKinase;

impl Axiom for CaMKIIIsKinase {
    fn description(&self) -> &str {
        "CaMKII is a kinase"
    }

    fn holds(&self) -> bool {
        IsKinase.get(&BiochemistryEntity::CaMKII) == Some(true)
    }
}

/// PhosphorylationCascade requires ATP.
pub struct PhosphorylationRequiresATP;

impl Axiom for PhosphorylationRequiresATP {
    fn description(&self) -> &str {
        "phosphorylation cascade requires ATP"
    }

    fn holds(&self) -> bool {
        RequiresATP.get(&BiochemistryEntity::PhosphorylationCascade) == Some(true)
    }
}

/// Biochemistry opposition is symmetric.
pub struct BiochemistryOppositionSymmetric;

impl Axiom for BiochemistryOppositionSymmetric {
    fn description(&self) -> &str {
        "biochemistry opposition is symmetric"
    }

    fn holds(&self) -> bool {
        opposition::Symmetric::<BiochemistryOpposition>::new().holds()
    }
}

/// Biochemistry opposition is irreflexive (nothing opposes itself).
pub struct BiochemistryOppositionIrreflexive;

impl Axiom for BiochemistryOppositionIrreflexive {
    fn description(&self) -> &str {
        "biochemistry opposition is irreflexive"
    }

    fn holds(&self) -> bool {
        opposition::Irreflexive::<BiochemistryOpposition>::new().holds()
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// Top-level ontology tying together the biochemistry category, qualities, and axioms.
pub struct BiochemistryOntology;

impl Ontology for BiochemistryOntology {
    type Cat = BiochemistryCategory;
    type Qual = IsSecondMessenger;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(BiochemistryTaxonomyIsDAG),
            Box::new(BiochemistryCausalAsymmetric),
            Box::new(BiochemistryCausalNoSelfCause),
            Box::new(CalciumEntryCausesGeneExpression),
            Box::new(CalciumEntryCausesNOProduction),
            Box::new(CalciumIsSecondMessenger),
            Box::new(CaMKIIIsKinase),
            Box::new(PhosphorylationRequiresATP),
            Box::new(BiochemistryOppositionSymmetric),
            Box::new(BiochemistryOppositionIrreflexive),
        ]
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::validate::check_category_laws;
    use pr4xis::ontology::reasoning::causation::CausalCategory;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;

    // -- Axiom tests --

    #[test]
    fn test_taxonomy_is_dag() {
        assert!(
            BiochemistryTaxonomyIsDAG.holds(),
            "{}",
            BiochemistryTaxonomyIsDAG.description()
        );
    }

    #[test]
    fn test_causal_asymmetric() {
        assert!(
            BiochemistryCausalAsymmetric.holds(),
            "{}",
            BiochemistryCausalAsymmetric.description()
        );
    }

    #[test]
    fn test_causal_no_self_cause() {
        assert!(
            BiochemistryCausalNoSelfCause.holds(),
            "{}",
            BiochemistryCausalNoSelfCause.description()
        );
    }

    #[test]
    fn test_calcium_entry_causes_gene_expression() {
        assert!(
            CalciumEntryCausesGeneExpression.holds(),
            "{}",
            CalciumEntryCausesGeneExpression.description()
        );
    }

    #[test]
    fn test_calcium_entry_causes_no_production() {
        assert!(
            CalciumEntryCausesNOProduction.holds(),
            "{}",
            CalciumEntryCausesNOProduction.description()
        );
    }

    #[test]
    fn test_calcium_is_second_messenger() {
        assert!(
            CalciumIsSecondMessenger.holds(),
            "{}",
            CalciumIsSecondMessenger.description()
        );
    }

    #[test]
    fn test_camkii_is_kinase() {
        assert!(CaMKIIIsKinase.holds(), "{}", CaMKIIIsKinase.description());
    }

    #[test]
    fn test_phosphorylation_requires_atp() {
        assert!(
            PhosphorylationRequiresATP.holds(),
            "{}",
            PhosphorylationRequiresATP.description()
        );
    }

    #[test]
    fn test_opposition_symmetric() {
        assert!(
            BiochemistryOppositionSymmetric.holds(),
            "{}",
            BiochemistryOppositionSymmetric.description()
        );
    }

    #[test]
    fn test_opposition_irreflexive() {
        assert!(
            BiochemistryOppositionIrreflexive.holds(),
            "{}",
            BiochemistryOppositionIrreflexive.description()
        );
    }

    // -- Category law tests --

    #[test]
    fn test_biochemistry_category_laws() {
        check_category_laws::<BiochemistryCategory>().unwrap();
    }

    #[test]
    fn test_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<BiochemistryTaxonomy>>().unwrap();
    }

    #[test]
    fn test_causal_category_laws() {
        check_category_laws::<CausalCategory<BiochemistryCauses>>().unwrap();
    }

    // -- Opposition tests --

    #[test]
    fn test_atp_opposes_adp() {
        use BiochemistryEntity::*;
        assert!(opposition::are_opposed::<BiochemistryOpposition>(
            &ATP, &ADP
        ));
        assert!(opposition::are_opposed::<BiochemistryOpposition>(
            &ADP, &ATP
        ));
    }

    #[test]
    fn test_glycolysis_opposes_oxidative_phosphorylation() {
        use BiochemistryEntity::*;
        assert!(opposition::are_opposed::<BiochemistryOpposition>(
            &Glycolysis,
            &OxidativePhosphorylation
        ));
    }

    #[test]
    fn test_phosphorylation_opposes_gene_transcription() {
        use BiochemistryEntity::*;
        assert!(opposition::are_opposed::<BiochemistryOpposition>(
            &PhosphorylationCascade,
            &GeneTranscription
        ));
    }

    #[test]
    fn test_calcium_does_not_oppose_atp() {
        use BiochemistryEntity::*;
        assert!(!opposition::are_opposed::<BiochemistryOpposition>(
            &CalciumIon,
            &ATP
        ));
    }

    #[test]
    fn test_atp_opposites_query() {
        use BiochemistryEntity::*;
        let opps = opposition::opposites::<BiochemistryOpposition>(&ATP);
        assert_eq!(opps, vec![ADP]);
    }

    // -- Taxonomy tests --

    #[test]
    fn test_calcium_is_signaling_molecule() {
        use BiochemistryEntity::*;
        assert!(taxonomy::is_a::<BiochemistryTaxonomy>(
            &CalciumIon,
            &SignalingMolecule
        ));
    }

    #[test]
    fn test_camkii_is_signaling_molecule() {
        use BiochemistryEntity::*;
        assert!(taxonomy::is_a::<BiochemistryTaxonomy>(
            &CaMKII,
            &SignalingMolecule
        ));
    }

    #[test]
    fn test_atp_is_energy_metabolite() {
        use BiochemistryEntity::*;
        assert!(taxonomy::is_a::<BiochemistryTaxonomy>(
            &ATP,
            &EnergyMetabolite
        ));
    }

    #[test]
    fn test_phosphorylation_is_biochemical_process() {
        use BiochemistryEntity::*;
        assert!(taxonomy::is_a::<BiochemistryTaxonomy>(
            &PhosphorylationCascade,
            &BiochemicalProcess
        ));
    }

    #[test]
    fn test_signaling_molecule_descendants() {
        use BiochemistryEntity::*;
        let descendants = taxonomy::descendants::<BiochemistryTaxonomy>(&SignalingMolecule);
        assert!(descendants.contains(&CalciumIon));
        assert!(descendants.contains(&Calmodulin));
        assert!(descendants.contains(&CaMKII));
        assert!(descendants.contains(&CREB));
        assert!(descendants.contains(&NitricOxide));
        assert!(descendants.contains(&CAMP));
        assert!(descendants.contains(&IP3));
        assert!(descendants.contains(&ProteinKinaseC));
        assert_eq!(descendants.len(), 8);
    }

    #[test]
    fn test_biochemical_process_descendants() {
        use BiochemistryEntity::*;
        let descendants = taxonomy::descendants::<BiochemistryTaxonomy>(&BiochemicalProcess);
        assert!(descendants.contains(&SignalTransduction));
        assert!(descendants.contains(&PhosphorylationCascade));
        assert!(descendants.contains(&GeneTranscription));
        assert!(descendants.contains(&ProteinSynthesis));
        assert!(descendants.contains(&SecondMessenger));
        assert!(descendants.contains(&Glycolysis));
        assert!(descendants.contains(&OxidativePhosphorylation));
        assert_eq!(descendants.len(), 7);
    }

    // -- Causal chain tests --

    #[test]
    fn test_calcium_entry_full_cascade() {
        use BiochemistryCausalEvent::*;
        let effects = causation::effects_of::<BiochemistryCauses>(&CalciumEntry);
        // Main pathway: CalmodulinActivation, CaMKIIPhosphorylation, CREBActivation,
        //               GeneExpressionChange, ProteinSynthesisChange
        // PKC branch: PKCActivation, DownstreamSignaling
        // NO branch: NOSynthaseActivation, NOProduction
        assert!(effects.contains(&CalmodulinActivation));
        assert!(effects.contains(&CaMKIIPhosphorylation));
        assert!(effects.contains(&CREBActivation));
        assert!(effects.contains(&GeneExpressionChange));
        assert!(effects.contains(&ProteinSynthesisChange));
        assert!(effects.contains(&PKCActivation));
        assert!(effects.contains(&DownstreamSignaling));
        assert!(effects.contains(&NOSynthaseActivation));
        assert!(effects.contains(&NOProduction));
        assert_eq!(effects.len(), 9);
    }

    #[test]
    fn test_atp_hydrolysis_causes_energy_release() {
        use BiochemistryCausalEvent::*;
        let effects = causation::effects_of::<BiochemistryCauses>(&ATPHydrolysis);
        assert!(effects.contains(&EnergyRelease));
        assert_eq!(effects.len(), 1);
    }

    #[test]
    fn test_calmodulin_activation_has_multiple_causes() {
        use BiochemistryCausalEvent::*;
        let causes = causation::causes_of::<BiochemistryCauses>(&CalmodulinActivation);
        assert!(causes.contains(&CalciumEntry));
    }

    // -- Quality tests --

    #[test]
    fn test_second_messengers() {
        use BiochemistryEntity::*;
        assert_eq!(IsSecondMessenger.get(&CalciumIon), Some(true));
        assert_eq!(IsSecondMessenger.get(&CAMP), Some(true));
        assert_eq!(IsSecondMessenger.get(&IP3), Some(true));
        assert_eq!(IsSecondMessenger.get(&NitricOxide), Some(true));
        assert_eq!(IsSecondMessenger.get(&CaMKII), Some(false));
    }

    #[test]
    fn test_kinases() {
        use BiochemistryEntity::*;
        assert_eq!(IsKinase.get(&CaMKII), Some(true));
        assert_eq!(IsKinase.get(&ProteinKinaseC), Some(true));
        assert_eq!(IsKinase.get(&CalciumIon), Some(false));
    }

    #[test]
    fn test_requires_atp() {
        use BiochemistryEntity::*;
        assert_eq!(RequiresATP.get(&PhosphorylationCascade), Some(true));
        assert_eq!(RequiresATP.get(&ProteinSynthesis), Some(true));
        assert_eq!(RequiresATP.get(&SignalTransduction), Some(false));
    }

    #[test]
    fn test_time_scales() {
        use BiochemistryEntity::*;
        assert_eq!(
            ProcessTimeScale.get(&SignalTransduction),
            Some(TimeScale::Milliseconds)
        );
        assert_eq!(
            ProcessTimeScale.get(&PhosphorylationCascade),
            Some(TimeScale::Seconds)
        );
        assert_eq!(
            ProcessTimeScale.get(&GeneTranscription),
            Some(TimeScale::Hours)
        );
        assert_eq!(ProcessTimeScale.get(&Glycolysis), Some(TimeScale::Minutes));
    }

    #[test]
    fn test_reversibility() {
        use BiochemistryEntity::*;
        assert_eq!(IsReversible.get(&PhosphorylationCascade), Some(true));
        assert_eq!(IsReversible.get(&GeneTranscription), Some(false));
        assert_eq!(IsReversible.get(&ProteinSynthesis), Some(false));
    }

    // -- Count tests --

    #[test]
    fn test_entity_count() {
        assert_eq!(BiochemistryEntity::variants().len(), 20);
    }

    #[test]
    fn test_causal_event_count() {
        assert_eq!(BiochemistryCausalEvent::variants().len(), 12);
    }

    #[test]
    fn test_ontology_validates() {
        BiochemistryOntology::validate().unwrap();
    }

    // -- Proptest --

    fn arb_biochemistry_entity() -> impl Strategy<Value = BiochemistryEntity> {
        (0..BiochemistryEntity::variants().len()).prop_map(|i| BiochemistryEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_second_messenger_is_signaling_molecule(entity in arb_biochemistry_entity()) {
            // Every second messenger should be a signaling molecule
            if IsSecondMessenger.get(&entity) == Some(true) {
                prop_assert!(
                    taxonomy::is_a::<BiochemistryTaxonomy>(&entity, &BiochemistryEntity::SignalingMolecule),
                    "{:?} is a second messenger but not a signaling molecule", entity
                );
            }
        }

        #[test]
        fn prop_kinase_is_signaling_molecule(entity in arb_biochemistry_entity()) {
            // Every kinase should be a signaling molecule
            if IsKinase.get(&entity) == Some(true) {
                prop_assert!(
                    taxonomy::is_a::<BiochemistryTaxonomy>(&entity, &BiochemistryEntity::SignalingMolecule),
                    "{:?} is a kinase but not a signaling molecule", entity
                );
            }
        }

        #[test]
        fn prop_atp_requiring_is_process(entity in arb_biochemistry_entity()) {
            // Every entity requiring ATP should be a biochemical process
            if RequiresATP.get(&entity) == Some(true) {
                prop_assert!(
                    taxonomy::is_a::<BiochemistryTaxonomy>(&entity, &BiochemistryEntity::BiochemicalProcess),
                    "{:?} requires ATP but is not a biochemical process", entity
                );
            }
        }
    }
}
