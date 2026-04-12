//! Ontology of logical derivation and proof construction.
//!
//! Formalizes the science of building logical chains: types of inference,
//! proof components, and properties of sound derivations.
//!
//! LITERATURE BASIS:
//!   - Gentzen 1935: Untersuchungen uber das logische Schliessen (natural deduction, sequent calculus)
//!   - Prawitz 1965: Natural Deduction (normalization of proofs)
//!   - Martin-Löf 1984: Intuitionistic Type Theory (constructive proofs)
//!   - Peirce 1903: abductive inference (inference to the best explanation)
//!
//! This is a PURE SCIENCE ontology of derivation — not an implementation
//! of a theorem prover. It formalizes the reasoning that ontology_diagnostics
//! uses when constructing proof chains for axiom verification.

use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::causation;
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entities
// ---------------------------------------------------------------------------

/// Components of logical derivation methodology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum DerivationEntity {
    // Types of derivation (how you reason)
    Deduction,
    Induction,
    Abduction,
    Analogy,
    Composition,

    // Components (what you build with)
    Premise,
    Conclusion,
    InferenceRule,
    Evidence,
    Justification,
    ProofStep,

    // Properties (what characterizes proofs)
    Soundness,
    Completeness,
    Validity,
    Decidability,

    // Abstract categories
    DerivationType,
    DerivationComponent,
    LogicalProperty,
}

/// Steps in the derivation pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum DerivationStep {
    /// Establish premises (axioms, assumptions, given facts).
    PremiseEstablishment,
    /// Apply inference rules to premises.
    RuleApplication,
    /// Derive intermediate conclusions.
    IntermediateConclusion,
    /// Extend the proof chain with further steps.
    ChainExtension,
    /// Check validity of each step.
    ValidityCheck,
    /// Verify soundness of the overall argument.
    SoundnessVerification,
    /// Complete the proof (all steps verified).
    ProofCompletion,
    /// Extract new knowledge from the completed proof.
    KnowledgeExtension,
}

// ---------------------------------------------------------------------------
// Ontology (category + reasoning)
// ---------------------------------------------------------------------------

define_ontology! {
    /// Dense category over derivation entities.
    pub DerivationOntology for DerivationCategory {
        entity: DerivationEntity,
        relation: DerivationRelation,

        taxonomy: DerivationTaxonomy [
            // Types → DerivationType
            (Deduction, DerivationType),
            (Induction, DerivationType),
            (Abduction, DerivationType),
            (Analogy, DerivationType),
            (Composition, DerivationType),
            // Components → DerivationComponent
            (Premise, DerivationComponent),
            (Conclusion, DerivationComponent),
            (InferenceRule, DerivationComponent),
            (Evidence, DerivationComponent),
            (Justification, DerivationComponent),
            (ProofStep, DerivationComponent),
            // Properties → LogicalProperty
            (Soundness, LogicalProperty),
            (Completeness, LogicalProperty),
            (Validity, LogicalProperty),
            (Decidability, LogicalProperty),
        ],

        causation: DerivationCausalGraph for DerivationStep [
            (PremiseEstablishment, RuleApplication),
            (RuleApplication, IntermediateConclusion),
            (IntermediateConclusion, ChainExtension),
            (ChainExtension, ValidityCheck),
            (ValidityCheck, SoundnessVerification),
            (SoundnessVerification, ProofCompletion),
            (ProofCompletion, KnowledgeExtension),
        ],

        opposition: DerivationOpposition [
            // Deduction vs abduction (certain→certain vs uncertain→plausible)
            (Deduction, Abduction),
            // Soundness vs completeness (all proved are true vs all true are provable)
            (Soundness, Completeness),
        ],
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Whether a derivation type is monotonic (adding premises never invalidates conclusions).
///
/// Deduction is monotonic: if P entails Q, then P+R still entails Q.
/// Abduction is non-monotonic: new evidence can retract abduced conclusions.
/// Induction is non-monotonic: new observations can overturn generalizations.
#[derive(Debug, Clone)]
pub struct IsMonotonic;

impl Quality for IsMonotonic {
    type Individual = DerivationEntity;
    type Value = bool;

    fn get(&self, entity: &DerivationEntity) -> Option<bool> {
        use DerivationEntity::*;
        match entity {
            Deduction => Some(true),   // monotonic: more premises, same conclusions
            Composition => Some(true), // composing valid steps preserves validity
            Induction => Some(false),  // new observations can overturn
            Abduction => Some(false),  // new evidence can retract
            Analogy => Some(false),    // analogies break with new disanalogies
            _ => None,
        }
    }
}

/// Whether a derivation type preserves truth from premises to conclusion.
///
/// Deduction preserves truth (if premises true, conclusion true).
/// Induction does not (premises can be true, conclusion false).
#[derive(Debug, Clone)]
pub struct PreservesTruth;

impl Quality for PreservesTruth {
    type Individual = DerivationEntity;
    type Value = bool;

    fn get(&self, entity: &DerivationEntity) -> Option<bool> {
        use DerivationEntity::*;
        match entity {
            Deduction => Some(true),   // truth-preserving by definition
            Composition => Some(true), // composing truth-preserving steps is truth-preserving
            Induction => Some(false),  // ampliative: conclusion goes beyond premises
            Abduction => Some(false),  // plausible, not certain
            Analogy => Some(false),    // suggestive, not guaranteed
            _ => None,
        }
    }
}

/// Whether a derivation type requires all premises to be present.
///
/// Deduction requires all premises (missing one invalidates the argument).
/// Abduction works with incomplete evidence (that is its purpose).
#[derive(Debug, Clone)]
pub struct RequiresAllPremises;

impl Quality for RequiresAllPremises {
    type Individual = DerivationEntity;
    type Value = bool;

    fn get(&self, entity: &DerivationEntity) -> Option<bool> {
        use DerivationEntity::*;
        match entity {
            Deduction => Some(true),   // all premises needed
            Composition => Some(true), // all steps needed
            Induction => Some(false),  // works with sample of observations
            Abduction => Some(false),  // works with incomplete evidence
            Analogy => Some(false),    // partial similarity suffices
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Axiom: premise establishment transitively causes knowledge extension.
pub struct PremiseCausesKnowledge;

impl Axiom for PremiseCausesKnowledge {
    fn description(&self) -> &str {
        "premise establishment transitively causes knowledge extension (full pipeline)"
    }
    fn holds(&self) -> bool {
        use DerivationStep::*;
        let effects = causation::effects_of::<DerivationCausalGraph>(&PremiseEstablishment);
        effects.contains(&KnowledgeExtension)
    }
}

/// Axiom: deduction is monotonic, abduction is not.
pub struct DeductionMonotonicAbductionNot;

impl Axiom for DeductionMonotonicAbductionNot {
    fn description(&self) -> &str {
        "deduction is monotonic but abduction is not (Gentzen vs Peirce)"
    }
    fn holds(&self) -> bool {
        use DerivationEntity::*;
        IsMonotonic.get(&Deduction) == Some(true) && IsMonotonic.get(&Abduction) == Some(false)
    }
}

/// Axiom: deduction preserves truth, induction does not.
pub struct DeductionPreservesTruthInductionNot;

impl Axiom for DeductionPreservesTruthInductionNot {
    fn description(&self) -> &str {
        "deduction preserves truth but induction does not (deductive vs ampliative)"
    }
    fn holds(&self) -> bool {
        use DerivationEntity::*;
        PreservesTruth.get(&Deduction) == Some(true)
            && PreservesTruth.get(&Induction) == Some(false)
    }
}

/// Axiom: deduction requires all premises, abduction does not.
pub struct DeductionRequiresAllAbductionNot;

impl Axiom for DeductionRequiresAllAbductionNot {
    fn description(&self) -> &str {
        "deduction requires all premises but abduction works with incomplete evidence"
    }
    fn holds(&self) -> bool {
        use DerivationEntity::*;
        RequiresAllPremises.get(&Deduction) == Some(true)
            && RequiresAllPremises.get(&Abduction) == Some(false)
    }
}

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for DerivationOntology {
    type Cat = DerivationCategory;
    type Qual = IsMonotonic;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(PremiseCausesKnowledge),
            Box::new(DeductionMonotonicAbductionNot),
            Box::new(DeductionPreservesTruthInductionNot),
            Box::new(DeductionRequiresAllAbductionNot),
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
    use pr4xis::ontology::reasoning::taxonomy;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;

    #[test]
    fn test_entity_count() {
        assert_eq!(DerivationEntity::variants().len(), 18);
    }

    #[test]
    fn test_step_count() {
        assert_eq!(DerivationStep::variants().len(), 8);
    }

    #[test]
    fn test_category_laws() {
        check_category_laws::<DerivationCategory>().unwrap();
    }

    #[test]
    fn test_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<DerivationTaxonomy>>().unwrap();
    }

    #[test]
    fn test_causal_category_laws() {
        check_category_laws::<CausalCategory<DerivationCausalGraph>>().unwrap();
    }

    #[test]
    fn test_ontology_validates() {
        DerivationOntology::validate().unwrap();
    }

    // -- Individual axiom tests --

    #[test]
    fn test_premise_causes_knowledge() {
        assert!(PremiseCausesKnowledge.holds());
    }

    #[test]
    fn test_deduction_monotonic_abduction_not() {
        assert!(DeductionMonotonicAbductionNot.holds());
    }

    #[test]
    fn test_deduction_preserves_truth_induction_not() {
        assert!(DeductionPreservesTruthInductionNot.holds());
    }

    #[test]
    fn test_deduction_requires_all_abduction_not() {
        assert!(DeductionRequiresAllAbductionNot.holds());
    }

    // -- Taxonomy tests --

    #[test]
    fn test_types_are_derivation_types() {
        use DerivationEntity::*;
        assert!(taxonomy::is_a::<DerivationTaxonomy>(
            &Deduction,
            &DerivationType
        ));
        assert!(taxonomy::is_a::<DerivationTaxonomy>(
            &Induction,
            &DerivationType
        ));
        assert!(taxonomy::is_a::<DerivationTaxonomy>(
            &Abduction,
            &DerivationType
        ));
        assert!(taxonomy::is_a::<DerivationTaxonomy>(
            &Analogy,
            &DerivationType
        ));
        assert!(taxonomy::is_a::<DerivationTaxonomy>(
            &Composition,
            &DerivationType
        ));
    }

    #[test]
    fn test_components_are_derivation_components() {
        use DerivationEntity::*;
        assert!(taxonomy::is_a::<DerivationTaxonomy>(
            &Premise,
            &DerivationComponent
        ));
        assert!(taxonomy::is_a::<DerivationTaxonomy>(
            &Conclusion,
            &DerivationComponent
        ));
        assert!(taxonomy::is_a::<DerivationTaxonomy>(
            &InferenceRule,
            &DerivationComponent
        ));
        assert!(taxonomy::is_a::<DerivationTaxonomy>(
            &Evidence,
            &DerivationComponent
        ));
        assert!(taxonomy::is_a::<DerivationTaxonomy>(
            &Justification,
            &DerivationComponent
        ));
        assert!(taxonomy::is_a::<DerivationTaxonomy>(
            &ProofStep,
            &DerivationComponent
        ));
    }

    #[test]
    fn test_properties_are_logical_properties() {
        use DerivationEntity::*;
        assert!(taxonomy::is_a::<DerivationTaxonomy>(
            &Soundness,
            &LogicalProperty
        ));
        assert!(taxonomy::is_a::<DerivationTaxonomy>(
            &Completeness,
            &LogicalProperty
        ));
        assert!(taxonomy::is_a::<DerivationTaxonomy>(
            &Validity,
            &LogicalProperty
        ));
        assert!(taxonomy::is_a::<DerivationTaxonomy>(
            &Decidability,
            &LogicalProperty
        ));
    }

    // -- Causal chain tests --

    #[test]
    fn test_full_pipeline_connected() {
        use DerivationStep::*;
        let effects = causation::effects_of::<DerivationCausalGraph>(&PremiseEstablishment);
        assert!(effects.contains(&RuleApplication));
        assert!(effects.contains(&IntermediateConclusion));
        assert!(effects.contains(&ChainExtension));
        assert!(effects.contains(&ValidityCheck));
        assert!(effects.contains(&SoundnessVerification));
        assert!(effects.contains(&ProofCompletion));
        assert!(effects.contains(&KnowledgeExtension));
    }

    // -- Opposition tests --

    #[test]
    fn test_deduction_opposes_abduction() {
        use DerivationEntity::*;
        assert!(opposition::are_opposed::<DerivationOpposition>(
            &Deduction, &Abduction
        ));
    }

    #[test]
    fn test_soundness_opposes_completeness() {
        use DerivationEntity::*;
        assert!(opposition::are_opposed::<DerivationOpposition>(
            &Soundness,
            &Completeness
        ));
    }

    // -- Quality tests --

    #[test]
    fn test_monotonicity() {
        use DerivationEntity::*;
        assert_eq!(IsMonotonic.get(&Deduction), Some(true));
        assert_eq!(IsMonotonic.get(&Composition), Some(true));
        assert_eq!(IsMonotonic.get(&Induction), Some(false));
        assert_eq!(IsMonotonic.get(&Abduction), Some(false));
        assert_eq!(IsMonotonic.get(&Analogy), Some(false));
    }

    #[test]
    fn test_truth_preservation() {
        use DerivationEntity::*;
        assert_eq!(PreservesTruth.get(&Deduction), Some(true));
        assert_eq!(PreservesTruth.get(&Composition), Some(true));
        assert_eq!(PreservesTruth.get(&Induction), Some(false));
        assert_eq!(PreservesTruth.get(&Abduction), Some(false));
    }

    #[test]
    fn test_requires_all_premises() {
        use DerivationEntity::*;
        assert_eq!(RequiresAllPremises.get(&Deduction), Some(true));
        assert_eq!(RequiresAllPremises.get(&Abduction), Some(false));
        assert_eq!(RequiresAllPremises.get(&Induction), Some(false));
    }

    // -- Proptest --

    fn arb_entity() -> impl Strategy<Value = DerivationEntity> {
        (0..DerivationEntity::variants().len()).prop_map(|i| DerivationEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_taxonomy_reflexive(entity in arb_entity()) {
            prop_assert!(taxonomy::is_a::<DerivationTaxonomy>(&entity, &entity));
        }

        #[test]
        fn prop_every_entity_has_category(entity in arb_entity()) {
            use DerivationEntity::*;
            let categories = [DerivationType, DerivationComponent, LogicalProperty];
            let belongs = categories.iter().any(|cat| taxonomy::is_a::<DerivationTaxonomy>(&entity, cat));
            let is_abstract = categories.contains(&entity);
            prop_assert!(belongs || is_abstract,
                "{:?} should belong to at least one category", entity);
        }
    }
}
