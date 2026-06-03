//! Recommendation — formalises the science of suggesting actions:
//! evaluating alternatives against criteria, assessing confidence, and
//! producing ranked recommendations.
//!
//! # Literature
//!
//! - **Von Neumann & Morgenstern (1944)** *Theory of Games and Economic
//!   Behavior*, Princeton UP — expected utility theory.
//! - **Keeney & Raiffa (1976)** *Decisions with Multiple Objectives:
//!   Preferences and Value Tradeoffs*, Wiley — multi-attribute utility
//!   theory (MAUT).
//! - **Roy (1968)** "Classement et choix en présence de points de vue
//!   multiples (la méthode ELECTRE)", *Revue Française d'Informatique
//!   et de Recherche Opérationnelle* 8 — multi-criteria decision
//!   analysis (MCDA).

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Recommendation",
    source: "Von Neumann & Morgenstern (1944) Theory of Games and Economic Behavior, Princeton UP; Keeney & Raiffa (1976) Decisions with Multiple Objectives, Wiley; Roy (1968) ELECTRE method, RAIRO 8",

    concepts: [
        // === Types ===
        Suggestion,
        Ranking,
        Classification,
        Warning,
        Prescription,
        // === Components ===
        Alternative,
        Criterion,
        Weight,
        Threshold,
        Evidence,
        Confidence,
        // === Outcomes ===
        Accept,
        Reject,
        Defer,
        Escalate,
        // === Abstract categories ===
        RecommendationType,
        DecisionComponent,
        DecisionOutcome,
        // === Pipeline stages ===
        EvidenceGathering,
        CriteriaEvaluation,
        AlternativeScoring,
        ThresholdComparison,
        OutcomeSelection,
        ConfidenceAssessment,
        RecommendationFormulation,
        ActionProposal,
    ],

    labels: {
        Suggestion: ("en", "Suggestion",
            "An advisory recommendation - acceptance by the user is optional."),
        Ranking: ("en", "Ranking",
            "Keeney & Raiffa (1976): ordered list of alternatives by utility."),
        Classification: ("en", "Classification",
            "Assignment of an alternative to a predefined category."),
        Warning: ("en", "Warning",
            "A cautionary recommendation - issued at low confidence to alert the user."),
        Prescription: ("en", "Prescription",
            "A strongly-recommended action - usually requires expert sign-off."),
        Alternative: ("en", "Alternative",
            "Keeney & Raiffa (1976): one option in the decision set."),
        Criterion: ("en", "Criterion",
            "Keeney & Raiffa (1976) MAUT: an attribute against which alternatives are evaluated."),
        Weight: ("en", "Weight",
            "Keeney & Raiffa (1976): scalar importance of a criterion."),
        Threshold: ("en", "Threshold",
            "Roy (1968) ELECTRE: a cut-off level below/above which a decision changes."),
        Evidence: ("en", "Evidence",
            "Information bearing on the decision."),
        Confidence: ("en", "Confidence",
            "Von Neumann & Morgenstern (1944): probability that the recommendation is correct."),
        Accept: ("en", "Accept", "Decision outcome: proceed with the recommendation."),
        Reject: ("en", "Reject", "Decision outcome: do not proceed."),
        Defer: ("en", "Defer", "Decision outcome: postpone the decision."),
        Escalate: ("en", "Escalate", "Decision outcome: send to a higher authority."),
        RecommendationType: ("en", "Recommendation type", "Abstract category for recommendation kinds."),
        DecisionComponent: ("en", "Decision component", "Abstract category for decision-process components."),
        DecisionOutcome: ("en", "Decision outcome", "Abstract category for decision results."),

        EvidenceGathering: ("en", "Evidence gathering", "Pipeline stage 1: collect evidence relevant to the decision."),
        CriteriaEvaluation: ("en", "Criteria evaluation", "Pipeline stage 2: evaluate alternatives against criteria."),
        AlternativeScoring: ("en", "Alternative scoring", "Pipeline stage 3: score each alternative on weighted criteria."),
        ThresholdComparison: ("en", "Threshold comparison", "Pipeline stage 4: compare scores against thresholds."),
        OutcomeSelection: ("en", "Outcome selection", "Pipeline stage 5: select an outcome (accept/reject/defer/escalate)."),
        ConfidenceAssessment: ("en", "Confidence assessment", "Pipeline stage 6: assess confidence in the recommendation."),
        RecommendationFormulation: ("en", "Recommendation formulation", "Pipeline stage 7: formulate the recommendation with justification."),
        ActionProposal: ("en", "Action proposal", "Pipeline stage 8: propose a concrete action to the user."),
    },

    is_a: [
        (Suggestion, RecommendationType),
        (Ranking, RecommendationType),
        (Classification, RecommendationType),
        (Warning, RecommendationType),
        (Prescription, RecommendationType),
        (Alternative, DecisionComponent),
        (Criterion, DecisionComponent),
        (Weight, DecisionComponent),
        (Threshold, DecisionComponent),
        (Evidence, DecisionComponent),
        (Confidence, DecisionComponent),
        (Accept, DecisionOutcome),
        (Reject, DecisionOutcome),
        (Defer, DecisionOutcome),
        (Escalate, DecisionOutcome),
    ],

    causes: [
        (EvidenceGathering, CriteriaEvaluation),
        (CriteriaEvaluation, AlternativeScoring),
        (AlternativeScoring, ThresholdComparison),
        (ThresholdComparison, OutcomeSelection),
        (OutcomeSelection, ConfidenceAssessment),
        (ConfidenceAssessment, RecommendationFormulation),
        (RecommendationFormulation, ActionProposal),
    ],

    opposes: [
        (Accept, Reject),
        (Reject, Accept),
        (Suggestion, Warning),
        (Warning, Suggestion),
    ],
}

/// Confidence level (Von Neumann & Morgenstern 1944): qualitative bands
/// for recommendation confidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfidenceLevelValue {
    High,
    Medium,
    Low,
}

/// Quality: typical confidence level for each recommendation type.
#[derive(Debug, Clone)]
pub struct ConfidenceLevel;

impl Quality for ConfidenceLevel {
    type Individual = RecommendationConcept;
    type Value = ConfidenceLevelValue;

    fn get(&self, c: &RecommendationConcept) -> Option<ConfidenceLevelValue> {
        use RecommendationConcept as R;
        match c {
            R::Prescription | R::Classification => Some(ConfidenceLevelValue::High),
            R::Ranking | R::Suggestion => Some(ConfidenceLevelValue::Medium),
            R::Warning => Some(ConfidenceLevelValue::Low),
            _ => None,
        }
    }
}

/// Quality: is the outcome reversible?
#[derive(Debug, Clone)]
pub struct IsReversible;

impl Quality for IsReversible {
    type Individual = RecommendationConcept;
    type Value = bool;

    fn get(&self, c: &RecommendationConcept) -> Option<bool> {
        use RecommendationConcept as R;
        match c {
            R::Accept => Some(false),
            R::Reject | R::Defer | R::Escalate => Some(true),
            _ => None,
        }
    }
}

/// Quality: does the recommendation type require expert validation?
#[derive(Debug, Clone)]
pub struct RequiresExpertValidation;

impl Quality for RequiresExpertValidation {
    type Individual = RecommendationConcept;
    type Value = bool;

    fn get(&self, c: &RecommendationConcept) -> Option<bool> {
        use RecommendationConcept as R;
        match c {
            R::Prescription | R::Warning => Some(true),
            R::Suggestion | R::Ranking | R::Classification => Some(false),
            _ => None,
        }
    }
}

// Legacy alias.
pub type RecommendationEntity = RecommendationConcept;

impl Ontology for RecommendationOntology {
    type Cat = RecommendationCategory;
    type Qual = ConfidenceLevel;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(PrescriptionsNeedExperts));
        axioms.push(Box::new(RejectReversibleAcceptNot));
        axioms
    }
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

pub struct PrescriptionsNeedExperts;

impl Axiom for PrescriptionsNeedExperts {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use RecommendationConcept as R;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if RequiresExpertValidation.get(&R::Prescription) == Some(true)
            && RequiresExpertValidation.get(&R::Suggestion) == Some(false)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PrescriptionsNeedExperts",
        "Prescription requires expert validation; Suggestion does not",
        "Keeney & Raiffa (1976) Decisions with Multiple Objectives, Wiley"
    );
}

pr4xis::register_axiom!(
    PrescriptionsNeedExperts,
    "Keeney & Raiffa (1976) Decisions with Multiple Objectives, Wiley"
);

pub struct RejectReversibleAcceptNot;

impl Axiom for RejectReversibleAcceptNot {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use RecommendationConcept as R;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if IsReversible.get(&R::Reject) == Some(true) && IsReversible.get(&R::Accept) == Some(false)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "RejectReversibleAcceptNot",
        "Rejection is reversible; acceptance is not (asymmetric commitment)",
        "Von Neumann & Morgenstern (1944) Theory of Games and Economic Behavior, Princeton UP"
    );
}

pr4xis::register_axiom!(
    RejectReversibleAcceptNot,
    "Von Neumann & Morgenstern (1944) Theory of Games and Economic Behavior, Princeton UP"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<RecommendationCategory>();
    }

    #[test]
    fn ontology_validates() {
        RecommendationOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn pipeline_reaches_action_proposal() {
        let caus: Vec<_> = RecommendationCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == RecommendationRelationKind::Causation)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(caus.contains(&(
            RecommendationConcept::EvidenceGathering,
            RecommendationConcept::ActionProposal
        )));
    }

    #[test]
    fn accept_opposes_reject() {
        let opp: Vec<_> = RecommendationCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == RecommendationRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opp.contains(&(RecommendationConcept::Accept, RecommendationConcept::Reject)));
    }

    #[test]
    fn outcomes_classify_as_decision_outcome() {
        use RecommendationConcept as R;
        let sub: Vec<_> = RecommendationCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == RecommendationRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for outcome in [R::Accept, R::Reject, R::Defer, R::Escalate] {
            assert!(sub.contains(&(outcome, R::DecisionOutcome)));
        }
    }

    #[test]
    fn confidence_levels() {
        use RecommendationConcept as R;
        assert_eq!(
            ConfidenceLevel.get(&R::Prescription),
            Some(ConfidenceLevelValue::High)
        );
        assert_eq!(
            ConfidenceLevel.get(&R::Warning),
            Some(ConfidenceLevelValue::Low)
        );
    }

    #[test]
    fn all_axioms_hold() {
        for axiom in RecommendationOntology::axioms() {
            if let Err(c) = axiom.verify() {
                panic!(
                    "axiom failed: {} - {}",
                    c.meta().name.as_str(),
                    c.meta().description.as_str()
                );
            }
        }
    }

    fn arb_concept() -> impl Strategy<Value = RecommendationConcept> {
        proptest::sample::select(RecommendationConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in RecommendationCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in RecommendationOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }

        #[test]
        fn prop_confidence_total_on_types(c in arb_concept()) {
            use RecommendationConcept as R;
            let v = ConfidenceLevel.get(&c);
            let is_type = matches!(c,
                R::Suggestion | R::Ranking | R::Classification | R::Warning | R::Prescription
            );
            prop_assert_eq!(v.is_some(), is_type);
        }
    }
}
