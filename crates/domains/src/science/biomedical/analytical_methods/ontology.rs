//! Ontology of structural analysis methods.
//!
//! Formalizes the science of analyzing structure in data: methods for
//! extracting patterns, detecting anomalies, and building concept lattices.
//!
//! LITERATURE BASIS:
//!   - Wille 1982: Formal Concept Analysis (restructuring lattice theory)
//!   - Ganter & Wille 1999: Formal Concept Analysis (mathematical foundations)
//!   - Birkhoff 1940: Lattice Theory (algebraic structure of partial orders)
//!
//! This is a PURE SCIENCE ontology of analysis methods — not an implementation
//! of analysis. It formalizes the reasoning that ontology_diagnostics uses
//! when performing structural analysis.

use praxis::category::{Category, Entity, Relationship};
use praxis::ontology::reasoning::causation::{self, CausalDef};
use praxis::ontology::reasoning::opposition::{self, OppositionDef};
use praxis::ontology::reasoning::taxonomy::{self, TaxonomyDef};
use praxis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entities
// ---------------------------------------------------------------------------

/// Components of structural analysis methodology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnalyticalEntity {
    // Methods (how you analyze)
    StructuralAnalysis,
    PatternAnalysis,
    StatisticalAnalysis,
    ComparativeAnalysis,
    AbsorptionAnalysis,
    ClusterAnalysis,

    // Components (what you work with)
    FormalContext,
    ConceptLattice,
    GaloisConnection,
    ObjectSet,
    AttributeSet,
    BinaryRelation,

    // Outputs (what you produce)
    Pattern,
    Cluster,
    Anomaly,
    Invariant,

    // Abstract categories
    AnalysisMethod,
    AnalysisComponent,
    AnalysisOutput,
}

impl Entity for AnalyticalEntity {
    fn variants() -> Vec<Self> {
        use AnalyticalEntity::*;
        vec![
            StructuralAnalysis,
            PatternAnalysis,
            StatisticalAnalysis,
            ComparativeAnalysis,
            AbsorptionAnalysis,
            ClusterAnalysis,
            FormalContext,
            ConceptLattice,
            GaloisConnection,
            ObjectSet,
            AttributeSet,
            BinaryRelation,
            Pattern,
            Cluster,
            Anomaly,
            Invariant,
            AnalysisMethod,
            AnalysisComponent,
            AnalysisOutput,
        ]
    }
}

// ---------------------------------------------------------------------------
// Taxonomy
// ---------------------------------------------------------------------------

/// Classification of analytical entities.
pub struct AnalyticalTaxonomy;

impl TaxonomyDef for AnalyticalTaxonomy {
    type Entity = AnalyticalEntity;

    fn relations() -> Vec<(AnalyticalEntity, AnalyticalEntity)> {
        use AnalyticalEntity::*;
        vec![
            // Methods → AnalysisMethod
            (StructuralAnalysis, AnalysisMethod),
            (PatternAnalysis, AnalysisMethod),
            (StatisticalAnalysis, AnalysisMethod),
            (ComparativeAnalysis, AnalysisMethod),
            (AbsorptionAnalysis, AnalysisMethod),
            (ClusterAnalysis, AnalysisMethod),
            // Components → AnalysisComponent
            (FormalContext, AnalysisComponent),
            (ConceptLattice, AnalysisComponent),
            (GaloisConnection, AnalysisComponent),
            (ObjectSet, AnalysisComponent),
            (AttributeSet, AnalysisComponent),
            (BinaryRelation, AnalysisComponent),
            // Outputs → AnalysisOutput
            (Pattern, AnalysisOutput),
            (Cluster, AnalysisOutput),
            (Anomaly, AnalysisOutput),
            (Invariant, AnalysisOutput),
        ]
    }
}

// ---------------------------------------------------------------------------
// Causal graph: the analysis pipeline
// ---------------------------------------------------------------------------

/// Steps in the structural analysis pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnalysisStep {
    /// Gather raw data from the domain.
    DataCollection,
    /// Form a formal context (objects x attributes x incidence).
    ContextFormation,
    /// Compute derivation operators (Galois connection).
    DerivationComputation,
    /// Build the concept lattice from the formal context.
    LatticeConstruction,
    /// Extract patterns from the lattice structure.
    PatternExtraction,
    /// Detect anomalies (deviations from expected patterns).
    AnomalyDetection,
    /// Interpret results in domain terms.
    ResultInterpretation,
    /// Update knowledge base with findings.
    KnowledgeUpdate,
}

impl Entity for AnalysisStep {
    fn variants() -> Vec<Self> {
        use AnalysisStep::*;
        vec![
            DataCollection,
            ContextFormation,
            DerivationComputation,
            LatticeConstruction,
            PatternExtraction,
            AnomalyDetection,
            ResultInterpretation,
            KnowledgeUpdate,
        ]
    }
}

/// The analysis pipeline as a causal graph.
pub struct AnalysisCausalGraph;

impl CausalDef for AnalysisCausalGraph {
    type Entity = AnalysisStep;

    fn relations() -> Vec<(AnalysisStep, AnalysisStep)> {
        use AnalysisStep::*;
        vec![
            (DataCollection, ContextFormation),
            (ContextFormation, DerivationComputation),
            (DerivationComputation, LatticeConstruction),
            (LatticeConstruction, PatternExtraction),
            (PatternExtraction, AnomalyDetection),
            (AnomalyDetection, ResultInterpretation),
            (ResultInterpretation, KnowledgeUpdate),
        ]
    }
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyticalRelation {
    pub from: AnalyticalEntity,
    pub to: AnalyticalEntity,
}

impl Relationship for AnalyticalRelation {
    type Object = AnalyticalEntity;
    fn source(&self) -> AnalyticalEntity {
        self.from
    }
    fn target(&self) -> AnalyticalEntity {
        self.to
    }
}

pub struct AnalyticalCategory;

impl Category for AnalyticalCategory {
    type Object = AnalyticalEntity;
    type Morphism = AnalyticalRelation;

    fn identity(obj: &AnalyticalEntity) -> AnalyticalRelation {
        AnalyticalRelation {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &AnalyticalRelation, g: &AnalyticalRelation) -> Option<AnalyticalRelation> {
        if f.to != g.from {
            return None;
        }
        Some(AnalyticalRelation {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<AnalyticalRelation> {
        let entities = AnalyticalEntity::variants();
        entities
            .iter()
            .flat_map(|&a| {
                entities
                    .iter()
                    .map(move |&b| AnalyticalRelation { from: a, to: b })
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Whether an analysis method can be fully automated.
#[derive(Debug, Clone)]
pub struct IsAutomatable;

impl Quality for IsAutomatable {
    type Individual = AnalyticalEntity;
    type Value = bool;

    fn get(&self, entity: &AnalyticalEntity) -> Option<bool> {
        use AnalyticalEntity::*;
        match entity {
            StructuralAnalysis => Some(true),
            PatternAnalysis => Some(true),
            StatisticalAnalysis => Some(true),
            ClusterAnalysis => Some(true),
            ComparativeAnalysis => Some(false), // requires human judgment on what to compare
            AbsorptionAnalysis => Some(false),  // requires domain expertise
            _ => None,
        }
    }
}

/// Whether an analysis method requires human judgment for interpretation.
#[derive(Debug, Clone)]
pub struct RequiresHumanJudgment;

impl Quality for RequiresHumanJudgment {
    type Individual = AnalyticalEntity;
    type Value = bool;

    fn get(&self, entity: &AnalyticalEntity) -> Option<bool> {
        use AnalyticalEntity::*;
        match entity {
            StructuralAnalysis => Some(false),
            PatternAnalysis => Some(false),
            StatisticalAnalysis => Some(false),
            ClusterAnalysis => Some(false),
            ComparativeAnalysis => Some(true),
            AbsorptionAnalysis => Some(true),
            _ => None,
        }
    }
}

/// Computational complexity class of an analysis method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComplexityClass {
    Linear,
    Quadratic,
    Exponential,
}

#[derive(Debug, Clone)]
pub struct Complexity;

impl Quality for Complexity {
    type Individual = AnalyticalEntity;
    type Value = ComplexityClass;

    fn get(&self, entity: &AnalyticalEntity) -> Option<ComplexityClass> {
        use AnalyticalEntity::*;
        match entity {
            StatisticalAnalysis => Some(ComplexityClass::Linear),
            ClusterAnalysis => Some(ComplexityClass::Quadratic),
            PatternAnalysis => Some(ComplexityClass::Quadratic),
            StructuralAnalysis => Some(ComplexityClass::Exponential), // concept lattice can be exponential
            ComparativeAnalysis => Some(ComplexityClass::Quadratic),
            AbsorptionAnalysis => Some(ComplexityClass::Quadratic),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Opposition
// ---------------------------------------------------------------------------

/// Semantic contrasts in analytical methods.
pub struct AnalyticalOpposition;

impl OppositionDef for AnalyticalOpposition {
    type Entity = AnalyticalEntity;

    fn pairs() -> Vec<(AnalyticalEntity, AnalyticalEntity)> {
        use AnalyticalEntity::*;
        vec![
            // Structure vs distribution (different lenses on data)
            (StructuralAnalysis, StatisticalAnalysis),
            // Regular vs irregular (what you find)
            (Pattern, Anomaly),
        ]
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Axiom: taxonomy is a DAG.
pub struct AnalyticalTaxonomyIsDAG;

impl Axiom for AnalyticalTaxonomyIsDAG {
    fn description(&self) -> &str {
        "analytical methods taxonomy has no cycles"
    }
    fn holds(&self) -> bool {
        taxonomy::NoCycles::<AnalyticalTaxonomy>::default().holds()
    }
}

/// Axiom: causal graph is asymmetric (no circular causation).
pub struct AnalysisCausalAsymmetric;

impl Axiom for AnalysisCausalAsymmetric {
    fn description(&self) -> &str {
        "analysis pipeline has no circular causation"
    }
    fn holds(&self) -> bool {
        causation::Asymmetric::<AnalysisCausalGraph>::default().holds()
    }
}

/// Axiom: data collection transitively causes knowledge update.
pub struct DataCollectionCausesKnowledgeUpdate;

impl Axiom for DataCollectionCausesKnowledgeUpdate {
    fn description(&self) -> &str {
        "data collection transitively causes knowledge update (full pipeline)"
    }
    fn holds(&self) -> bool {
        use AnalysisStep::*;
        let effects = causation::effects_of::<AnalysisCausalGraph>(&DataCollection);
        effects.contains(&KnowledgeUpdate)
    }
}

/// Axiom: Galois connection is an analysis component.
pub struct GaloisConnectionIsComponent;

impl Axiom for GaloisConnectionIsComponent {
    fn description(&self) -> &str {
        "Galois connection is classified as an analysis component"
    }
    fn holds(&self) -> bool {
        use AnalyticalEntity::*;
        taxonomy::is_a::<AnalyticalTaxonomy>(&GaloisConnection, &AnalysisComponent)
    }
}

/// Axiom: pattern and anomaly are both analysis outputs.
pub struct PatternAndAnomalyAreOutputs;

impl Axiom for PatternAndAnomalyAreOutputs {
    fn description(&self) -> &str {
        "pattern and anomaly are both classified as analysis outputs"
    }
    fn holds(&self) -> bool {
        use AnalyticalEntity::*;
        taxonomy::is_a::<AnalyticalTaxonomy>(&Pattern, &AnalysisOutput)
            && taxonomy::is_a::<AnalyticalTaxonomy>(&Anomaly, &AnalysisOutput)
    }
}

/// Axiom: some methods are automatable, some are not.
pub struct SomeMethodsAutomatableSomeNot;

impl Axiom for SomeMethodsAutomatableSomeNot {
    fn description(&self) -> &str {
        "some analysis methods are automatable and some require human judgment"
    }
    fn holds(&self) -> bool {
        use AnalyticalEntity::*;
        let methods = [
            StructuralAnalysis,
            PatternAnalysis,
            StatisticalAnalysis,
            ComparativeAnalysis,
            AbsorptionAnalysis,
            ClusterAnalysis,
        ];
        let auto_count = methods
            .iter()
            .filter(|m| IsAutomatable.get(m) == Some(true))
            .count();
        auto_count > 0 && auto_count < methods.len()
    }
}

/// Axiom: opposition is symmetric.
pub struct AnalyticalOppositionSymmetric;

impl Axiom for AnalyticalOppositionSymmetric {
    fn description(&self) -> &str {
        "analytical opposition is symmetric"
    }
    fn holds(&self) -> bool {
        opposition::Symmetric::<AnalyticalOpposition>::new().holds()
    }
}

/// Axiom: opposition is irreflexive.
pub struct AnalyticalOppositionIrreflexive;

impl Axiom for AnalyticalOppositionIrreflexive {
    fn description(&self) -> &str {
        "analytical opposition is irreflexive"
    }
    fn holds(&self) -> bool {
        opposition::Irreflexive::<AnalyticalOpposition>::new().holds()
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

pub struct AnalyticalMethodsOntology;

impl Ontology for AnalyticalMethodsOntology {
    type Cat = AnalyticalCategory;
    type Qual = IsAutomatable;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(AnalyticalTaxonomyIsDAG),
            Box::new(AnalysisCausalAsymmetric),
            Box::new(DataCollectionCausesKnowledgeUpdate),
            Box::new(GaloisConnectionIsComponent),
            Box::new(PatternAndAnomalyAreOutputs),
            Box::new(SomeMethodsAutomatableSomeNot),
            Box::new(AnalyticalOppositionSymmetric),
            Box::new(AnalyticalOppositionIrreflexive),
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
    use praxis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;

    #[test]
    fn test_entity_count() {
        assert_eq!(AnalyticalEntity::variants().len(), 19);
    }

    #[test]
    fn test_step_count() {
        assert_eq!(AnalysisStep::variants().len(), 8);
    }

    #[test]
    fn test_category_laws() {
        check_category_laws::<AnalyticalCategory>().unwrap();
    }

    #[test]
    fn test_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<AnalyticalTaxonomy>>().unwrap();
    }

    #[test]
    fn test_causal_category_laws() {
        check_category_laws::<CausalCategory<AnalysisCausalGraph>>().unwrap();
    }

    #[test]
    fn test_ontology_validates() {
        AnalyticalMethodsOntology::validate().unwrap();
    }

    // -- Individual axiom tests --

    #[test]
    fn test_taxonomy_dag() {
        assert!(AnalyticalTaxonomyIsDAG.holds());
    }

    #[test]
    fn test_causal_asymmetric() {
        assert!(AnalysisCausalAsymmetric.holds());
    }

    #[test]
    fn test_data_collection_causes_knowledge_update() {
        assert!(DataCollectionCausesKnowledgeUpdate.holds());
    }

    #[test]
    fn test_galois_connection_is_component() {
        assert!(GaloisConnectionIsComponent.holds());
    }

    #[test]
    fn test_pattern_and_anomaly_are_outputs() {
        assert!(PatternAndAnomalyAreOutputs.holds());
    }

    #[test]
    fn test_some_methods_automatable_some_not() {
        assert!(SomeMethodsAutomatableSomeNot.holds());
    }

    #[test]
    fn test_opposition_symmetric() {
        assert!(AnalyticalOppositionSymmetric.holds());
    }

    #[test]
    fn test_opposition_irreflexive() {
        assert!(AnalyticalOppositionIrreflexive.holds());
    }

    // -- Taxonomy tests --

    #[test]
    fn test_methods_are_analysis_methods() {
        use AnalyticalEntity::*;
        assert!(taxonomy::is_a::<AnalyticalTaxonomy>(
            &StructuralAnalysis,
            &AnalysisMethod
        ));
        assert!(taxonomy::is_a::<AnalyticalTaxonomy>(
            &PatternAnalysis,
            &AnalysisMethod
        ));
        assert!(taxonomy::is_a::<AnalyticalTaxonomy>(
            &StatisticalAnalysis,
            &AnalysisMethod
        ));
        assert!(taxonomy::is_a::<AnalyticalTaxonomy>(
            &ClusterAnalysis,
            &AnalysisMethod
        ));
    }

    #[test]
    fn test_components_are_analysis_components() {
        use AnalyticalEntity::*;
        assert!(taxonomy::is_a::<AnalyticalTaxonomy>(
            &FormalContext,
            &AnalysisComponent
        ));
        assert!(taxonomy::is_a::<AnalyticalTaxonomy>(
            &ConceptLattice,
            &AnalysisComponent
        ));
        assert!(taxonomy::is_a::<AnalyticalTaxonomy>(
            &GaloisConnection,
            &AnalysisComponent
        ));
    }

    #[test]
    fn test_outputs_are_analysis_outputs() {
        use AnalyticalEntity::*;
        assert!(taxonomy::is_a::<AnalyticalTaxonomy>(
            &Pattern,
            &AnalysisOutput
        ));
        assert!(taxonomy::is_a::<AnalyticalTaxonomy>(
            &Cluster,
            &AnalysisOutput
        ));
        assert!(taxonomy::is_a::<AnalyticalTaxonomy>(
            &Anomaly,
            &AnalysisOutput
        ));
        assert!(taxonomy::is_a::<AnalyticalTaxonomy>(
            &Invariant,
            &AnalysisOutput
        ));
    }

    // -- Causal chain tests --

    #[test]
    fn test_full_pipeline_connected() {
        use AnalysisStep::*;
        let effects = causation::effects_of::<AnalysisCausalGraph>(&DataCollection);
        assert!(effects.contains(&ContextFormation));
        assert!(effects.contains(&DerivationComputation));
        assert!(effects.contains(&LatticeConstruction));
        assert!(effects.contains(&PatternExtraction));
        assert!(effects.contains(&AnomalyDetection));
        assert!(effects.contains(&ResultInterpretation));
        assert!(effects.contains(&KnowledgeUpdate));
    }

    // -- Opposition tests --

    #[test]
    fn test_structural_opposes_statistical() {
        use AnalyticalEntity::*;
        assert!(opposition::are_opposed::<AnalyticalOpposition>(
            &StructuralAnalysis,
            &StatisticalAnalysis
        ));
    }

    #[test]
    fn test_pattern_opposes_anomaly() {
        use AnalyticalEntity::*;
        assert!(opposition::are_opposed::<AnalyticalOpposition>(
            &Pattern, &Anomaly
        ));
    }

    // -- Quality tests --

    #[test]
    fn test_automatability() {
        use AnalyticalEntity::*;
        assert_eq!(IsAutomatable.get(&StructuralAnalysis), Some(true));
        assert_eq!(IsAutomatable.get(&StatisticalAnalysis), Some(true));
        assert_eq!(IsAutomatable.get(&ComparativeAnalysis), Some(false));
        assert_eq!(IsAutomatable.get(&AbsorptionAnalysis), Some(false));
    }

    #[test]
    fn test_human_judgment() {
        use AnalyticalEntity::*;
        assert_eq!(RequiresHumanJudgment.get(&ComparativeAnalysis), Some(true));
        assert_eq!(RequiresHumanJudgment.get(&StructuralAnalysis), Some(false));
    }

    #[test]
    fn test_complexity() {
        use AnalyticalEntity::*;
        assert_eq!(
            Complexity.get(&StatisticalAnalysis),
            Some(ComplexityClass::Linear)
        );
        assert_eq!(
            Complexity.get(&ClusterAnalysis),
            Some(ComplexityClass::Quadratic)
        );
        assert_eq!(
            Complexity.get(&StructuralAnalysis),
            Some(ComplexityClass::Exponential)
        );
    }

    // -- Proptest --

    fn arb_entity() -> impl Strategy<Value = AnalyticalEntity> {
        (0..AnalyticalEntity::variants().len()).prop_map(|i| AnalyticalEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_taxonomy_reflexive(entity in arb_entity()) {
            prop_assert!(taxonomy::is_a::<AnalyticalTaxonomy>(&entity, &entity));
        }

        #[test]
        fn prop_every_entity_has_category(entity in arb_entity()) {
            use AnalyticalEntity::*;
            let categories = [AnalysisMethod, AnalysisComponent, AnalysisOutput];
            let belongs = categories.iter().any(|cat| taxonomy::is_a::<AnalyticalTaxonomy>(&entity, cat));
            let is_abstract = categories.contains(&entity);
            prop_assert!(belongs || is_abstract,
                "{:?} should belong to at least one category", entity);
        }
    }
}
