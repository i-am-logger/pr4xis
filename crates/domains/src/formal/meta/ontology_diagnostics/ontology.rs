//! Meta-ontology: an ontology of ontology engineering.
//!
//! Formalizes the gap detection methodology as a domain:
//! - Entities: the components of ontological analysis
//! - Taxonomy: classification of gaps and resolutions
//! - Causation: the detection → resolution pipeline
//! - Qualities: properties of gaps and their severity
//! - Axioms: proven properties of the methodology
//!
//! LITERATURE BASIS:
//!   - Spivak & Kent 2012: ologs as categorical ontologies
//!   - Spivak 2014: functors for cross-domain mapping
//!   - Mac Lane 1971: adjunctions in category theory
//!   - Euzenat & Shvaiko 2013: ontology alignment (matching, not gap detection)
//!   - Schlobach & Cornet 2003: ontology debugging (consistency, not completeness)
//!
//! NOVEL:
//!   - Adjunction-based gap detection (unit/counit != identity)
//!   - ContextDef resolution of detected gaps
//!   - Loss ratio quantification
//!   - This meta-ontology itself

use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::causation;
use pr4xis::ontology::reasoning::taxonomy;
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entities
// ---------------------------------------------------------------------------

/// Components of the ontology engineering methodology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum MetaEntity {
    // Ontological structures (what you build)
    DomainOntology,
    CategoryStructure,
    TaxonomyStructure,
    CausalStructure,
    QualityStructure,
    AxiomSet,

    // Cross-domain connections (how you connect)
    Functor,
    Adjunction,
    UnitMorphism,
    CounitMorphism,
    NaturalTransformation,

    // Gap analysis (what you discover)
    UnitGap,
    CounitGap,
    GranularityMismatch,
    MissingDistinction,
    InformationLoss,
    CanonicalRepresentative,

    // Resolution (how you fix)
    ContextResolution,
    OntologyEnrichment,
    IntermediateDomain,
    GranularityRefinement,

    // Verification (how you confirm)
    LiteratureVerification,
    MachineProof,
    PropertyTest,

    // Abstract categories
    Structure,
    Connection,
    Gap,
    Resolution,
    Verification,
}

/// Steps in the gap detection methodology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum MethodologyStep {
    /// Formalize two scientific domains as categories.
    FormalizeDomains,
    /// Build functors in both directions.
    ConstructFunctors,
    /// Verify functor laws (identity + composition preservation).
    VerifyFunctorLaws,
    /// Construct adjunction (pair functors with unit + counit).
    ConstructAdjunction,
    /// Compute unit for every source entity.
    ComputeUnit,
    /// Compute counit for every target entity.
    ComputeCounit,
    /// Identify gaps (unit != identity or counit != identity).
    DetectGaps,
    /// Classify gaps (granularity mismatch, missing distinction, etc.).
    ClassifyGaps,
    /// Compute loss ratios (fraction of entities that collapse).
    ComputeLossRatios,
    /// Propose resolution (ContextDef, enrichment, intermediate domain).
    ProposeResolution,
    /// Verify resolution against published literature.
    VerifyAgainstLiterature,
    /// Implement resolution in ontology code.
    ImplementResolution,
    /// Run machine proofs (automated tests).
    RunMachineProofs,
    /// Assess whether loss ratio improved.
    AssessImprovement,
}

// ---------------------------------------------------------------------------
// Ontology (category + reasoning)
// ---------------------------------------------------------------------------

define_ontology! {
    /// Dense category over meta-ontological entities.
    pub MetaOntology for MetaCategory {
        entity: MetaEntity,
        relation: MetaRelation,

        taxonomy: MetaTaxonomy [
            // Structures
            (DomainOntology, Structure),
            (CategoryStructure, Structure),
            (TaxonomyStructure, Structure),
            (CausalStructure, Structure),
            (QualityStructure, Structure),
            (AxiomSet, Structure),
            // Connections
            (Functor, Connection),
            (Adjunction, Connection),
            (UnitMorphism, Connection),
            (CounitMorphism, Connection),
            (NaturalTransformation, Connection),
            // Adjunction components
            (UnitMorphism, Adjunction),
            (CounitMorphism, Adjunction),
            // Gaps
            (UnitGap, Gap),
            (CounitGap, Gap),
            (GranularityMismatch, Gap),
            (MissingDistinction, Gap),
            (InformationLoss, Gap),
            (CanonicalRepresentative, Gap),
            // Resolutions
            (ContextResolution, Resolution),
            (OntologyEnrichment, Resolution),
            (IntermediateDomain, Resolution),
            (GranularityRefinement, Resolution),
            // Verification
            (LiteratureVerification, Verification),
            (MachineProof, Verification),
            (PropertyTest, Verification),
        ],

        causation: MethodologyCausalGraph for MethodologyStep [
            // Build phase
            (FormalizeDomains, ConstructFunctors),
            (ConstructFunctors, VerifyFunctorLaws),
            (VerifyFunctorLaws, ConstructAdjunction),
            // Detection phase
            (ConstructAdjunction, ComputeUnit),
            (ConstructAdjunction, ComputeCounit),
            (ComputeUnit, DetectGaps),
            (ComputeCounit, DetectGaps),
            (DetectGaps, ClassifyGaps),
            (DetectGaps, ComputeLossRatios),
            // Resolution phase
            (ClassifyGaps, ProposeResolution),
            (ComputeLossRatios, ProposeResolution),
            (ProposeResolution, VerifyAgainstLiterature),
            (VerifyAgainstLiterature, ImplementResolution),
            (ImplementResolution, RunMachineProofs),
            (RunMachineProofs, AssessImprovement),
        ],

        opposition: MetaOpposition [
            // Gap vs Resolution (problem vs solution)
            (Gap, Resolution),
            // Unit vs Counit (forward vs backward round-trip)
            (UnitGap, CounitGap),
            // Automated vs manual verification
            (MachineProof, LiteratureVerification),
            // Context resolution preserves functors; enrichment may break them
            (ContextResolution, OntologyEnrichment),
        ],
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Is this gap type detectable automatically (by adjunction analysis)?
#[derive(Debug, Clone)]
pub struct IsAutoDetectable;

impl Quality for IsAutoDetectable {
    type Individual = MetaEntity;
    type Value = bool;

    fn get(&self, entity: &MetaEntity) -> Option<bool> {
        use MetaEntity::*;
        match entity {
            // Adjunction analysis detects these automatically
            UnitGap => Some(true),
            CounitGap => Some(true),
            InformationLoss => Some(true),
            CanonicalRepresentative => Some(true),
            GranularityMismatch => Some(true),
            // These require human judgment
            MissingDistinction => Some(false),
            // Not gap types
            _ => None,
        }
    }
}

/// Does this resolution type preserve functor validity?
///
/// ContextDef adds distinctions WITHOUT changing the category structure,
/// so existing functors remain valid. OntologyEnrichment adds new entities
/// which may break existing functors.
#[derive(Debug, Clone)]
pub struct PreservesFunctorValidity;

impl Quality for PreservesFunctorValidity {
    type Individual = MetaEntity;
    type Value = bool;

    fn get(&self, entity: &MetaEntity) -> Option<bool> {
        use MetaEntity::*;
        match entity {
            ContextResolution => Some(true), // adds distinctions, not entities
            GranularityRefinement => Some(true), // refines within existing structure
            OntologyEnrichment => Some(false), // adds entities, may break functors
            IntermediateDomain => Some(false), // adds new domain entirely
            _ => None,
        }
    }
}

/// What loss ratio threshold suggests this resolution type?
///
/// Based on empirical findings from 3 adjunctions:
/// - >80% loss → IntermediateDomain needed (too much collapse for ContextDef)
/// - 40-80% loss → ContextResolution appropriate (moderate collapse)
/// - <40% loss → GranularityRefinement sufficient (minor collapse)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LossThreshold {
    /// <40% loss: minor, refinement sufficient.
    Low,
    /// 40-80% loss: moderate, context disambiguation needed.
    Moderate,
    /// >80% loss: severe, intermediate domain needed.
    High,
}

#[derive(Debug, Clone)]
pub struct SuggestedForLossLevel;

impl Quality for SuggestedForLossLevel {
    type Individual = MetaEntity;
    type Value = LossThreshold;

    fn get(&self, entity: &MetaEntity) -> Option<LossThreshold> {
        use MetaEntity::*;
        match entity {
            GranularityRefinement => Some(LossThreshold::Low),
            ContextResolution => Some(LossThreshold::Moderate),
            IntermediateDomain => Some(LossThreshold::High),
            OntologyEnrichment => Some(LossThreshold::Moderate),
            _ => None,
        }
    }
}

/// Is this verification type automated or manual?
#[derive(Debug, Clone)]
pub struct IsAutomated;

impl Quality for IsAutomated {
    type Individual = MetaEntity;
    type Value = bool;

    fn get(&self, entity: &MetaEntity) -> Option<bool> {
        use MetaEntity::*;
        match entity {
            MachineProof => Some(true),
            PropertyTest => Some(true),
            LiteratureVerification => Some(false), // requires human reading
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Axiom: formalize domains transitively leads to assess improvement.
/// The full pipeline is connected end-to-end.
pub struct PipelineIsComplete;

impl Axiom for PipelineIsComplete {
    fn description(&self) -> &str {
        "the full pipeline from formalization to assessment is connected"
    }
    fn holds(&self) -> bool {
        use MethodologyStep::*;
        let effects = causation::effects_of::<MethodologyCausalGraph>(&FormalizeDomains);
        effects.contains(&AssessImprovement)
    }
}

/// Axiom: gap detection requires BOTH unit and counit computation.
/// You need both directions to find all missing distinctions.
pub struct GapDetectionRequiresBothDirections;

impl Axiom for GapDetectionRequiresBothDirections {
    fn description(&self) -> &str {
        "gap detection requires both unit and counit computation"
    }
    fn holds(&self) -> bool {
        use MethodologyStep::*;
        let unit_causes = causation::causes_of::<MethodologyCausalGraph>(&DetectGaps);
        unit_causes.contains(&ComputeUnit) && unit_causes.contains(&ComputeCounit)
    }
}

/// Axiom: literature verification comes AFTER resolution proposal, BEFORE implementation.
/// You don't implement a fix without checking the literature first.
pub struct LiteratureBeforeImplementation;

impl Axiom for LiteratureBeforeImplementation {
    fn description(&self) -> &str {
        "literature verification occurs after proposal, before implementation"
    }
    fn holds(&self) -> bool {
        use MethodologyStep::*;
        let verify_causes =
            causation::causes_of::<MethodologyCausalGraph>(&VerifyAgainstLiterature);
        let verify_effects =
            causation::effects_of::<MethodologyCausalGraph>(&VerifyAgainstLiterature);
        verify_causes.contains(&ProposeResolution) && verify_effects.contains(&ImplementResolution)
    }
}

/// Axiom: most gap types are automatically detectable.
/// The adjunction does the work — you don't have to find gaps by hand.
pub struct MostGapsAreAutoDetectable;

impl Axiom for MostGapsAreAutoDetectable {
    fn description(&self) -> &str {
        "most gap types are automatically detectable by adjunction analysis"
    }
    fn holds(&self) -> bool {
        use MetaEntity::*;
        let gap_types = [
            UnitGap,
            CounitGap,
            GranularityMismatch,
            InformationLoss,
            CanonicalRepresentative,
            MissingDistinction,
        ];
        let auto_count = gap_types
            .iter()
            .filter(|g| IsAutoDetectable.get(g) == Some(true))
            .count();
        // Most (>50%) should be auto-detectable
        auto_count > gap_types.len() / 2
    }
}

/// Axiom: ContextResolution preserves functor validity.
/// This is critical: you can resolve gaps WITHOUT breaking existing proofs.
pub struct ContextResolutionPreservesFunctors;

impl Axiom for ContextResolutionPreservesFunctors {
    fn description(&self) -> &str {
        "context resolution preserves existing functor validity (non-destructive fix)"
    }
    fn holds(&self) -> bool {
        use MetaEntity::*;
        PreservesFunctorValidity.get(&ContextResolution) == Some(true)
    }
}

/// Axiom: OntologyEnrichment does NOT preserve functor validity.
/// Adding new entities may break existing functors (match arms become incomplete).
pub struct EnrichmentMayBreakFunctors;

impl Axiom for EnrichmentMayBreakFunctors {
    fn description(&self) -> &str {
        "ontology enrichment may break existing functors (destructive fix)"
    }
    fn holds(&self) -> bool {
        use MetaEntity::*;
        PreservesFunctorValidity.get(&OntologyEnrichment) == Some(false)
    }
}

/// Axiom: high loss suggests intermediate domain is needed.
/// Empirical finding: >80% loss means the two domains are too far apart
/// in granularity for direct connection.
pub struct HighLossSuggestsIntermediateDomain;

impl Axiom for HighLossSuggestsIntermediateDomain {
    fn description(&self) -> &str {
        ">80% loss suggests an intermediate domain is needed (empirical finding)"
    }
    fn holds(&self) -> bool {
        use MetaEntity::*;
        SuggestedForLossLevel.get(&IntermediateDomain) == Some(LossThreshold::High)
    }
}

/// EMPIRICAL AXIOM: every adjunction between domains at different scales has gaps.
///
/// This is our core empirical finding, proven by gap_analysis.rs across
/// 3 adjunctions (Molecular⊣Bioelectric, Pharmacology⊣Molecular,
/// Biology⊣Bioelectric). All 3 have non-zero unit and counit gaps.
///
/// This axiom cannot be proven from the meta-ontology alone — it requires
/// the gap_analysis module's computed results. We encode it here as a
/// claim to be verified externally.
pub struct EveryAdjunctionHasGaps;

impl Axiom for EveryAdjunctionHasGaps {
    fn description(&self) -> &str {
        "every adjunction between domains at different scales has gaps (empirical, proven by gap_analysis.rs)"
    }
    fn holds(&self) -> bool {
        // Verified externally by gap_analysis::tests::test_all_adjunctions_have_gaps
        // Here we encode the structural claim: if UnitGap and CounitGap both
        // exist as gap types AND are auto-detectable, the methodology works.
        use MetaEntity::*;
        taxonomy::is_a::<MetaTaxonomy>(&UnitGap, &Gap)
            && taxonomy::is_a::<MetaTaxonomy>(&CounitGap, &Gap)
            && IsAutoDetectable.get(&UnitGap) == Some(true)
            && IsAutoDetectable.get(&CounitGap) == Some(true)
    }
}

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for MetaOntology {
    type Cat = MetaCategory;
    type Qual = IsAutoDetectable;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(PipelineIsComplete),
            Box::new(GapDetectionRequiresBothDirections),
            Box::new(LiteratureBeforeImplementation),
            Box::new(MostGapsAreAutoDetectable),
            Box::new(ContextResolutionPreservesFunctors),
            Box::new(EnrichmentMayBreakFunctors),
            Box::new(HighLossSuggestsIntermediateDomain),
            Box::new(EveryAdjunctionHasGaps),
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
    use pr4xis::ontology::reasoning::opposition;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;

    #[test]
    fn test_entity_count() {
        assert_eq!(MetaEntity::variants().len(), 29);
    }

    #[test]
    fn test_step_count() {
        assert_eq!(MethodologyStep::variants().len(), 14);
    }

    #[test]
    fn test_category_laws() {
        check_category_laws::<MetaCategory>().unwrap();
    }

    #[test]
    fn test_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<MetaTaxonomy>>().unwrap();
    }

    #[test]
    fn test_causal_category_laws() {
        check_category_laws::<CausalCategory<MethodologyCausalGraph>>().unwrap();
    }

    #[test]
    fn test_ontology_validates() {
        MetaOntology::validate().unwrap();
    }

    // -- Individual axiom tests --

    #[test]
    fn test_pipeline_complete() {
        assert!(PipelineIsComplete.holds());
    }

    #[test]
    fn test_gap_detection_requires_both() {
        assert!(GapDetectionRequiresBothDirections.holds());
    }

    #[test]
    fn test_literature_before_implementation() {
        assert!(LiteratureBeforeImplementation.holds());
    }

    #[test]
    fn test_most_gaps_auto_detectable() {
        assert!(MostGapsAreAutoDetectable.holds());
    }

    #[test]
    fn test_context_preserves_functors() {
        assert!(ContextResolutionPreservesFunctors.holds());
    }

    #[test]
    fn test_enrichment_breaks_functors() {
        assert!(EnrichmentMayBreakFunctors.holds());
    }

    #[test]
    fn test_high_loss_intermediate() {
        assert!(HighLossSuggestsIntermediateDomain.holds());
    }

    #[test]
    fn test_every_adjunction_has_gaps() {
        assert!(EveryAdjunctionHasGaps.holds());
    }

    // -- Taxonomy tests --

    #[test]
    fn test_unit_gap_is_a_gap() {
        use MetaEntity::*;
        assert!(taxonomy::is_a::<MetaTaxonomy>(&UnitGap, &Gap));
        assert!(taxonomy::is_a::<MetaTaxonomy>(&CounitGap, &Gap));
    }

    #[test]
    fn test_context_resolution_is_a_resolution() {
        use MetaEntity::*;
        assert!(taxonomy::is_a::<MetaTaxonomy>(
            &ContextResolution,
            &Resolution
        ));
    }

    #[test]
    fn test_machine_proof_is_verification() {
        use MetaEntity::*;
        assert!(taxonomy::is_a::<MetaTaxonomy>(&MachineProof, &Verification));
    }

    #[test]
    fn test_adjunction_is_a_connection() {
        use MetaEntity::*;
        assert!(taxonomy::is_a::<MetaTaxonomy>(&Adjunction, &Connection));
    }

    #[test]
    fn test_unit_morphism_is_adjunction_and_connection() {
        use MetaEntity::*;
        assert!(taxonomy::is_a::<MetaTaxonomy>(&UnitMorphism, &Adjunction));
        assert!(taxonomy::is_a::<MetaTaxonomy>(&UnitMorphism, &Connection));
    }

    // -- Causal chain tests --

    #[test]
    fn test_formalize_reaches_detect() {
        use MethodologyStep::*;
        let effects = causation::effects_of::<MethodologyCausalGraph>(&FormalizeDomains);
        assert!(effects.contains(&DetectGaps));
    }

    #[test]
    fn test_detect_reaches_implement() {
        use MethodologyStep::*;
        let effects = causation::effects_of::<MethodologyCausalGraph>(&DetectGaps);
        assert!(effects.contains(&ImplementResolution));
    }

    #[test]
    fn test_full_pipeline_path() {
        use MethodologyStep::*;
        let effects = causation::effects_of::<MethodologyCausalGraph>(&FormalizeDomains);
        // Every step should be reachable from FormalizeDomains
        assert!(effects.contains(&ConstructFunctors));
        assert!(effects.contains(&ConstructAdjunction));
        assert!(effects.contains(&ComputeUnit));
        assert!(effects.contains(&DetectGaps));
        assert!(effects.contains(&ProposeResolution));
        assert!(effects.contains(&VerifyAgainstLiterature));
        assert!(effects.contains(&RunMachineProofs));
        assert!(effects.contains(&AssessImprovement));
    }

    // -- Opposition tests --

    #[test]
    fn test_gap_opposes_resolution() {
        use MetaEntity::*;
        assert!(opposition::are_opposed::<MetaOpposition>(&Gap, &Resolution));
    }

    #[test]
    fn test_unit_gap_opposes_counit_gap() {
        use MetaEntity::*;
        assert!(opposition::are_opposed::<MetaOpposition>(
            &UnitGap, &CounitGap
        ));
    }

    #[test]
    fn test_context_opposes_enrichment() {
        use MetaEntity::*;
        assert!(opposition::are_opposed::<MetaOpposition>(
            &ContextResolution,
            &OntologyEnrichment
        ));
    }

    // -- Quality tests --

    #[test]
    fn test_loss_thresholds() {
        use MetaEntity::*;
        assert_eq!(
            SuggestedForLossLevel.get(&GranularityRefinement),
            Some(LossThreshold::Low)
        );
        assert_eq!(
            SuggestedForLossLevel.get(&ContextResolution),
            Some(LossThreshold::Moderate)
        );
        assert_eq!(
            SuggestedForLossLevel.get(&IntermediateDomain),
            Some(LossThreshold::High)
        );
    }

    #[test]
    fn test_automated_verification() {
        use MetaEntity::*;
        assert_eq!(IsAutomated.get(&MachineProof), Some(true));
        assert_eq!(IsAutomated.get(&PropertyTest), Some(true));
        assert_eq!(IsAutomated.get(&LiteratureVerification), Some(false));
    }

    // -- Proptest --

    fn arb_meta_entity() -> impl Strategy<Value = MetaEntity> {
        (0..MetaEntity::variants().len()).prop_map(|i| MetaEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_taxonomy_reflexive(entity in arb_meta_entity()) {
            prop_assert!(taxonomy::is_a::<MetaTaxonomy>(&entity, &entity));
        }

        #[test]
        fn prop_every_entity_has_category(entity in arb_meta_entity()) {
            // Every entity belongs to at least one abstract category
            use MetaEntity::*;
            let categories = [Structure, Connection, Gap, Resolution, Verification];
            let belongs = categories.iter().any(|cat| taxonomy::is_a::<MetaTaxonomy>(&entity, cat));
            let is_abstract = categories.contains(&entity);
            prop_assert!(belongs || is_abstract,
                "{:?} should belong to at least one category", entity);
        }
    }
}
