//! Ontology of recommendation and decision-making.
//!
//! Formalizes the science of suggesting actions: evaluating alternatives
//! against criteria, assessing confidence, and producing ranked recommendations.
//!
//! LITERATURE BASIS:
//!   - Von Neumann & Morgenstern 1944: expected utility theory
//!   - Keeney & Raiffa 1976: multi-attribute utility theory
//!   - Multi-Criteria Decision Analysis (MCDA): weighted scoring, threshold comparison
//!
//! This is a PURE SCIENCE ontology of recommendation — not an implementation
//! of a recommender system. It formalizes the reasoning that ontology_diagnostics
//! uses when suggesting resolutions.

use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::causation;
use pr4xis::ontology::reasoning::taxonomy;
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entities
// ---------------------------------------------------------------------------

/// Components of the recommendation methodology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum RecommendationEntity {
    // Types of recommendation (what you produce)
    Suggestion,
    Ranking,
    Classification,
    Warning,
    Prescription,

    // Decision components (what you work with)
    Alternative,
    Criterion,
    Weight,
    Threshold,
    Evidence,
    Confidence,

    // Outcomes (what happens next)
    Accept,
    Reject,
    Defer,
    Escalate,

    // Abstract categories
    RecommendationType,
    DecisionComponent,
    DecisionOutcome,
}

/// Steps in the recommendation pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum RecommendationStep {
    /// Collect evidence relevant to the decision.
    EvidenceGathering,
    /// Evaluate alternatives against criteria.
    CriteriaEvaluation,
    /// Score each alternative on weighted criteria.
    AlternativeScoring,
    /// Compare scores against thresholds.
    ThresholdComparison,
    /// Select outcome (accept, reject, defer, escalate).
    OutcomeSelection,
    /// Assess confidence in the recommendation.
    ConfidenceAssessment,
    /// Formulate the recommendation with justification.
    RecommendationFormulation,
    /// Propose concrete action to the user.
    ActionProposal,
}

// ---------------------------------------------------------------------------
// Ontology (category + reasoning)
// ---------------------------------------------------------------------------

define_ontology! {
    /// Dense category over recommendation entities.
    pub RecommendationOntology for RecommendationCategory {
        entity: RecommendationEntity,
        relation: RecommendationRelation,

        taxonomy: RecommendationTaxonomy [
            // Types → RecommendationType
            (Suggestion, RecommendationType),
            (Ranking, RecommendationType),
            (Classification, RecommendationType),
            (Warning, RecommendationType),
            (Prescription, RecommendationType),
            // Components → DecisionComponent
            (Alternative, DecisionComponent),
            (Criterion, DecisionComponent),
            (Weight, DecisionComponent),
            (Threshold, DecisionComponent),
            (Evidence, DecisionComponent),
            (Confidence, DecisionComponent),
            // Outcomes → DecisionOutcome
            (Accept, DecisionOutcome),
            (Reject, DecisionOutcome),
            (Defer, DecisionOutcome),
            (Escalate, DecisionOutcome),
        ],

        causation: RecommendationCausalGraph for RecommendationStep [
            (EvidenceGathering, CriteriaEvaluation),
            (CriteriaEvaluation, AlternativeScoring),
            (AlternativeScoring, ThresholdComparison),
            (ThresholdComparison, OutcomeSelection),
            (OutcomeSelection, ConfidenceAssessment),
            (ConfidenceAssessment, RecommendationFormulation),
            (RecommendationFormulation, ActionProposal),
        ],

        opposition: RecommendationOpposition [
            // Accept vs reject (proceed vs stop)
            (Accept, Reject),
            // Suggestion vs warning (positive vs cautionary)
            (Suggestion, Warning),
        ],
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Confidence level of a recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfidenceLevelValue {
    High,
    Medium,
    Low,
}

/// Maps recommendation types to their typical confidence level.
#[derive(Debug, Clone)]
pub struct ConfidenceLevel;

impl Quality for ConfidenceLevel {
    type Individual = RecommendationEntity;
    type Value = ConfidenceLevelValue;

    fn get(&self, entity: &RecommendationEntity) -> Option<ConfidenceLevelValue> {
        use RecommendationEntity::*;
        match entity {
            Prescription => Some(ConfidenceLevelValue::High), // only prescribed when confident
            Classification => Some(ConfidenceLevelValue::High),
            Ranking => Some(ConfidenceLevelValue::Medium),
            Suggestion => Some(ConfidenceLevelValue::Medium),
            Warning => Some(ConfidenceLevelValue::Low), // warnings issued even at low confidence
            _ => None,
        }
    }
}

/// Whether a decision outcome is reversible.
#[derive(Debug, Clone)]
pub struct IsReversible;

impl Quality for IsReversible {
    type Individual = RecommendationEntity;
    type Value = bool;

    fn get(&self, entity: &RecommendationEntity) -> Option<bool> {
        use RecommendationEntity::*;
        match entity {
            Accept => Some(false),  // once accepted, action taken
            Reject => Some(true),   // can reconsider later
            Defer => Some(true),    // explicitly postponed
            Escalate => Some(true), // can be de-escalated
            _ => None,
        }
    }
}

/// Whether a recommendation type requires expert validation.
#[derive(Debug, Clone)]
pub struct RequiresExpertValidation;

impl Quality for RequiresExpertValidation {
    type Individual = RecommendationEntity;
    type Value = bool;

    fn get(&self, entity: &RecommendationEntity) -> Option<bool> {
        use RecommendationEntity::*;
        match entity {
            Prescription => Some(true),    // prescriptions need expert sign-off
            Warning => Some(true),         // warnings need expert assessment
            Suggestion => Some(false),     // suggestions are advisory
            Ranking => Some(false),        // rankings are informational
            Classification => Some(false), // classifications are descriptive
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Axiom: evidence gathering transitively causes action proposal.
pub struct EvidenceCausesAction;

impl Axiom for EvidenceCausesAction {
    fn description(&self) -> &str {
        "evidence gathering transitively causes action proposal (full pipeline)"
    }
    fn holds(&self) -> bool {
        use RecommendationStep::*;
        let effects = causation::effects_of::<RecommendationCausalGraph>(&EvidenceGathering);
        effects.contains(&ActionProposal)
    }
}

/// Axiom: accept and reject are both decision outcomes.
pub struct AcceptAndRejectAreOutcomes;

impl Axiom for AcceptAndRejectAreOutcomes {
    fn description(&self) -> &str {
        "accept and reject are both classified as decision outcomes"
    }
    fn holds(&self) -> bool {
        use RecommendationEntity::*;
        taxonomy::is_a::<RecommendationTaxonomy>(&Accept, &DecisionOutcome)
            && taxonomy::is_a::<RecommendationTaxonomy>(&Reject, &DecisionOutcome)
    }
}

/// Axiom: prescriptions require expert validation, suggestions do not.
pub struct PrescriptionsNeedExperts;

impl Axiom for PrescriptionsNeedExperts {
    fn description(&self) -> &str {
        "prescriptions require expert validation but suggestions do not"
    }
    fn holds(&self) -> bool {
        use RecommendationEntity::*;
        RequiresExpertValidation.get(&Prescription) == Some(true)
            && RequiresExpertValidation.get(&Suggestion) == Some(false)
    }
}

/// Axiom: reject is reversible, accept is not.
pub struct RejectReversibleAcceptNot;

impl Axiom for RejectReversibleAcceptNot {
    fn description(&self) -> &str {
        "rejection is reversible but acceptance is not (asymmetric commitment)"
    }
    fn holds(&self) -> bool {
        use RecommendationEntity::*;
        IsReversible.get(&Reject) == Some(true) && IsReversible.get(&Accept) == Some(false)
    }
}

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for RecommendationOntology {
    type Cat = RecommendationCategory;
    type Qual = ConfidenceLevel;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(EvidenceCausesAction),
            Box::new(AcceptAndRejectAreOutcomes),
            Box::new(PrescriptionsNeedExperts),
            Box::new(RejectReversibleAcceptNot),
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
        assert_eq!(RecommendationEntity::variants().len(), 18);
    }

    #[test]
    fn test_step_count() {
        assert_eq!(RecommendationStep::variants().len(), 8);
    }

    #[test]
    fn test_category_laws() {
        check_category_laws::<RecommendationCategory>().unwrap();
    }

    #[test]
    fn test_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<RecommendationTaxonomy>>().unwrap();
    }

    #[test]
    fn test_causal_category_laws() {
        check_category_laws::<CausalCategory<RecommendationCausalGraph>>().unwrap();
    }

    #[test]
    fn test_ontology_validates() {
        RecommendationOntology::validate().unwrap();
    }

    // -- Individual axiom tests --

    #[test]
    fn test_evidence_causes_action() {
        assert!(EvidenceCausesAction.holds());
    }

    #[test]
    fn test_accept_and_reject_are_outcomes() {
        assert!(AcceptAndRejectAreOutcomes.holds());
    }

    #[test]
    fn test_prescriptions_need_experts() {
        assert!(PrescriptionsNeedExperts.holds());
    }

    #[test]
    fn test_reject_reversible_accept_not() {
        assert!(RejectReversibleAcceptNot.holds());
    }

    // -- Taxonomy tests --

    #[test]
    fn test_types_are_recommendation_types() {
        use RecommendationEntity::*;
        assert!(taxonomy::is_a::<RecommendationTaxonomy>(
            &Suggestion,
            &RecommendationType
        ));
        assert!(taxonomy::is_a::<RecommendationTaxonomy>(
            &Ranking,
            &RecommendationType
        ));
        assert!(taxonomy::is_a::<RecommendationTaxonomy>(
            &Warning,
            &RecommendationType
        ));
        assert!(taxonomy::is_a::<RecommendationTaxonomy>(
            &Prescription,
            &RecommendationType
        ));
    }

    #[test]
    fn test_components_are_decision_components() {
        use RecommendationEntity::*;
        assert!(taxonomy::is_a::<RecommendationTaxonomy>(
            &Alternative,
            &DecisionComponent
        ));
        assert!(taxonomy::is_a::<RecommendationTaxonomy>(
            &Criterion,
            &DecisionComponent
        ));
        assert!(taxonomy::is_a::<RecommendationTaxonomy>(
            &Weight,
            &DecisionComponent
        ));
        assert!(taxonomy::is_a::<RecommendationTaxonomy>(
            &Threshold,
            &DecisionComponent
        ));
        assert!(taxonomy::is_a::<RecommendationTaxonomy>(
            &Evidence,
            &DecisionComponent
        ));
        assert!(taxonomy::is_a::<RecommendationTaxonomy>(
            &Confidence,
            &DecisionComponent
        ));
    }

    #[test]
    fn test_outcomes_are_decision_outcomes() {
        use RecommendationEntity::*;
        assert!(taxonomy::is_a::<RecommendationTaxonomy>(
            &Accept,
            &DecisionOutcome
        ));
        assert!(taxonomy::is_a::<RecommendationTaxonomy>(
            &Reject,
            &DecisionOutcome
        ));
        assert!(taxonomy::is_a::<RecommendationTaxonomy>(
            &Defer,
            &DecisionOutcome
        ));
        assert!(taxonomy::is_a::<RecommendationTaxonomy>(
            &Escalate,
            &DecisionOutcome
        ));
    }

    // -- Causal chain tests --

    #[test]
    fn test_full_pipeline_connected() {
        use RecommendationStep::*;
        let effects = causation::effects_of::<RecommendationCausalGraph>(&EvidenceGathering);
        assert!(effects.contains(&CriteriaEvaluation));
        assert!(effects.contains(&AlternativeScoring));
        assert!(effects.contains(&ThresholdComparison));
        assert!(effects.contains(&OutcomeSelection));
        assert!(effects.contains(&ConfidenceAssessment));
        assert!(effects.contains(&RecommendationFormulation));
        assert!(effects.contains(&ActionProposal));
    }

    // -- Opposition tests --

    #[test]
    fn test_accept_opposes_reject() {
        use RecommendationEntity::*;
        assert!(opposition::are_opposed::<RecommendationOpposition>(
            &Accept, &Reject
        ));
    }

    #[test]
    fn test_suggestion_opposes_warning() {
        use RecommendationEntity::*;
        assert!(opposition::are_opposed::<RecommendationOpposition>(
            &Suggestion,
            &Warning
        ));
    }

    // -- Quality tests --

    #[test]
    fn test_confidence_levels() {
        use RecommendationEntity::*;
        assert_eq!(
            ConfidenceLevel.get(&Prescription),
            Some(ConfidenceLevelValue::High)
        );
        assert_eq!(
            ConfidenceLevel.get(&Warning),
            Some(ConfidenceLevelValue::Low)
        );
        assert_eq!(
            ConfidenceLevel.get(&Suggestion),
            Some(ConfidenceLevelValue::Medium)
        );
    }

    #[test]
    fn test_reversibility() {
        use RecommendationEntity::*;
        assert_eq!(IsReversible.get(&Accept), Some(false));
        assert_eq!(IsReversible.get(&Reject), Some(true));
        assert_eq!(IsReversible.get(&Defer), Some(true));
        assert_eq!(IsReversible.get(&Escalate), Some(true));
    }

    #[test]
    fn test_expert_validation() {
        use RecommendationEntity::*;
        assert_eq!(RequiresExpertValidation.get(&Prescription), Some(true));
        assert_eq!(RequiresExpertValidation.get(&Warning), Some(true));
        assert_eq!(RequiresExpertValidation.get(&Suggestion), Some(false));
    }

    // -- Proptest --

    fn arb_entity() -> impl Strategy<Value = RecommendationEntity> {
        (0..RecommendationEntity::variants().len())
            .prop_map(|i| RecommendationEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_taxonomy_reflexive(entity in arb_entity()) {
            prop_assert!(taxonomy::is_a::<RecommendationTaxonomy>(&entity, &entity));
        }

        #[test]
        fn prop_every_entity_has_category(entity in arb_entity()) {
            use RecommendationEntity::*;
            let categories = [RecommendationType, DecisionComponent, DecisionOutcome];
            let belongs = categories.iter().any(|cat| taxonomy::is_a::<RecommendationTaxonomy>(&entity, cat));
            let is_abstract = categories.contains(&entity);
            prop_assert!(belongs || is_abstract,
                "{:?} should belong to at least one category", entity);
        }
    }
}
