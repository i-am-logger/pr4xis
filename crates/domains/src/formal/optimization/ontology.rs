//! Ontology of optimization methods.
//!
//! Formalizes the science of finding best configurations: methods for
//! searching solution spaces, evaluating objectives, and characterizing
//! optimal points.
//!
//! LITERATURE BASIS:
//!   - Boyd & Vandenberghe 2004: Convex Optimization (objective functions, constraints, feasibility)
//!   - Pareto 1906: Manual of Political Economy (Pareto optimality, multi-objective tradeoffs)
//!   - Holland 1975: Adaptation in Natural and Artificial Systems (genetic algorithms)
//!   - Kirkpatrick et al. 1983: Optimization by Simulated Annealing
//!
//! This is a PURE SCIENCE ontology of optimization — not an implementation
//! of an optimizer. It formalizes the reasoning that ontology_diagnostics uses
//! when searching for optimal ontological configurations.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::causation;
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entities
// ---------------------------------------------------------------------------

/// Components of the optimization methodology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
pub enum OptimizationEntity {
    // Methods (how you optimize)
    ExhaustiveSearch,
    GradientDescent,
    GeneticAlgorithm,
    SimulatedAnnealing,
    ParetoOptimization,
    GridSearch,

    // Components (what you work with)
    ObjectiveFunction,
    Constraint,
    SearchSpace,
    FeasibleRegion,
    OptimalPoint,
    ParetoFront,

    // Properties (what characterizes solutions)
    Convergence,
    LocalOptimum,
    GlobalOptimum,
    Tradeoff,

    // Abstract categories
    OptimizationMethod,
    OptimizationComponent,
    OptimalityProperty,
}

/// Steps in the optimization pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
pub enum OptimizationStep {
    /// Formulate the optimization problem.
    ProblemFormulation,
    /// Define the search space of possible solutions.
    SearchSpaceDefinition,
    /// Specify constraints that solutions must satisfy.
    ConstraintSpecification,
    /// Evaluate objective function on candidates.
    ObjectiveEvaluation,
    /// Generate candidate solutions.
    CandidateGeneration,
    /// Check candidate feasibility against constraints.
    FeasibilityCheck,
    /// Assess optimality of feasible candidates.
    OptimalityAssessment,
    /// Select the best solution.
    SolutionSelection,
}

// ---------------------------------------------------------------------------
// Ontology (category + reasoning)
// ---------------------------------------------------------------------------

define_ontology! {
    /// Dense category over optimization entities.
    pub OptimizationOntology for OptimizationCategory {
        entity: OptimizationEntity,
        relation: OptimizationRelation,
        being: AbstractObject,
        source: "Boyd & Vandenberghe (2004); Pareto (1906)",

        taxonomy: OptimizationTaxonomy [
            // Methods → OptimizationMethod
            (ExhaustiveSearch, OptimizationMethod),
            (GradientDescent, OptimizationMethod),
            (GeneticAlgorithm, OptimizationMethod),
            (SimulatedAnnealing, OptimizationMethod),
            (ParetoOptimization, OptimizationMethod),
            (GridSearch, OptimizationMethod),
            // Components → OptimizationComponent
            (ObjectiveFunction, OptimizationComponent),
            (Constraint, OptimizationComponent),
            (SearchSpace, OptimizationComponent),
            (FeasibleRegion, OptimizationComponent),
            (OptimalPoint, OptimizationComponent),
            (ParetoFront, OptimizationComponent),
            // Properties → OptimalityProperty
            (Convergence, OptimalityProperty),
            (LocalOptimum, OptimalityProperty),
            (GlobalOptimum, OptimalityProperty),
            (Tradeoff, OptimalityProperty),
        ],

        causation: OptimizationCausalGraph for OptimizationStep [
            (ProblemFormulation, SearchSpaceDefinition),
            (SearchSpaceDefinition, ConstraintSpecification),
            (ConstraintSpecification, ObjectiveEvaluation),
            (ObjectiveEvaluation, CandidateGeneration),
            (CandidateGeneration, FeasibilityCheck),
            (FeasibilityCheck, OptimalityAssessment),
            (OptimalityAssessment, SolutionSelection),
        ],

        opposition: OptimizationOpposition [
            // Local vs global (partial vs complete optimality)
            (LocalOptimum, GlobalOptimum),
            // Exact vs heuristic (guaranteed vs approximate)
            (ExhaustiveSearch, GeneticAlgorithm),
        ],
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Whether a method guarantees finding the global optimum.
#[derive(Debug, Clone)]
pub struct GuaranteesGlobal;

impl Quality for GuaranteesGlobal {
    type Individual = OptimizationEntity;
    type Value = bool;

    fn get(&self, entity: &OptimizationEntity) -> Option<bool> {
        use OptimizationEntity::*;
        match entity {
            ExhaustiveSearch => Some(true),    // checks everything
            GridSearch => Some(true),          // checks everything on grid
            GradientDescent => Some(false),    // can get stuck in local optima
            GeneticAlgorithm => Some(false),   // heuristic, no guarantee
            SimulatedAnnealing => Some(false), // probabilistic, no guarantee
            ParetoOptimization => Some(false), // finds Pareto front, not single global
            _ => None,
        }
    }
}

/// Time complexity class of an optimization method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeComplexityClass {
    Polynomial,
    Exponential,
}

#[derive(Debug, Clone)]
pub struct TimeComplexity;

impl Quality for TimeComplexity {
    type Individual = OptimizationEntity;
    type Value = TimeComplexityClass;

    fn get(&self, entity: &OptimizationEntity) -> Option<TimeComplexityClass> {
        use OptimizationEntity::*;
        match entity {
            GradientDescent => Some(TimeComplexityClass::Polynomial),
            GeneticAlgorithm => Some(TimeComplexityClass::Polynomial),
            SimulatedAnnealing => Some(TimeComplexityClass::Polynomial),
            ParetoOptimization => Some(TimeComplexityClass::Polynomial),
            ExhaustiveSearch => Some(TimeComplexityClass::Exponential),
            GridSearch => Some(TimeComplexityClass::Exponential),
            _ => None,
        }
    }
}

/// Whether a method handles multi-objective optimization.
#[derive(Debug, Clone)]
pub struct HandlesMultiObjective;

impl Quality for HandlesMultiObjective {
    type Individual = OptimizationEntity;
    type Value = bool;

    fn get(&self, entity: &OptimizationEntity) -> Option<bool> {
        use OptimizationEntity::*;
        match entity {
            ParetoOptimization => Some(true),  // designed for multi-objective
            GeneticAlgorithm => Some(true),    // NSGA-II and variants
            ExhaustiveSearch => Some(true),    // can evaluate multiple objectives
            GradientDescent => Some(false),    // single objective gradient
            SimulatedAnnealing => Some(false), // single objective energy
            GridSearch => Some(true),          // can evaluate multiple objectives
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Axiom: problem formulation transitively causes solution selection.
pub struct FormulationCausesSolution;

impl Axiom for FormulationCausesSolution {
    fn description(&self) -> &str {
        "problem formulation transitively causes solution selection (full pipeline)"
    }
    fn holds(&self) -> bool {
        use OptimizationStep::*;
        let effects = causation::effects_of::<OptimizationCausalGraph>(&ProblemFormulation);
        effects.contains(&SolutionSelection)
    }
}
pr4xis::register_axiom!(FormulationCausesSolution);

/// Axiom: exhaustive search guarantees global, gradient descent does not.
pub struct ExhaustiveGuaranteesGradientDoesNot;

impl Axiom for ExhaustiveGuaranteesGradientDoesNot {
    fn description(&self) -> &str {
        "exhaustive search guarantees global optimum but gradient descent does not"
    }
    fn holds(&self) -> bool {
        use OptimizationEntity::*;
        GuaranteesGlobal.get(&ExhaustiveSearch) == Some(true)
            && GuaranteesGlobal.get(&GradientDescent) == Some(false)
    }
}
pr4xis::register_axiom!(ExhaustiveGuaranteesGradientDoesNot);

/// Axiom: exact methods are exponential, heuristic methods are polynomial.
pub struct ExactExponentialHeuristicPolynomial;

impl Axiom for ExactExponentialHeuristicPolynomial {
    fn description(&self) -> &str {
        "exact methods (exhaustive) are exponential; heuristic methods (genetic) are polynomial"
    }
    fn holds(&self) -> bool {
        use OptimizationEntity::*;
        TimeComplexity.get(&ExhaustiveSearch) == Some(TimeComplexityClass::Exponential)
            && TimeComplexity.get(&GeneticAlgorithm) == Some(TimeComplexityClass::Polynomial)
    }
}
pr4xis::register_axiom!(ExactExponentialHeuristicPolynomial);

/// Axiom: Pareto optimization handles multi-objective; gradient descent does not.
pub struct ParetoMultiObjectiveGradientNot;

impl Axiom for ParetoMultiObjectiveGradientNot {
    fn description(&self) -> &str {
        "Pareto optimization handles multi-objective; gradient descent does not"
    }
    fn holds(&self) -> bool {
        use OptimizationEntity::*;
        HandlesMultiObjective.get(&ParetoOptimization) == Some(true)
            && HandlesMultiObjective.get(&GradientDescent) == Some(false)
    }
}
pr4xis::register_axiom!(ParetoMultiObjectiveGradientNot);

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for OptimizationOntology {
    type Cat = OptimizationCategory;
    type Qual = GuaranteesGlobal;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(FormulationCausesSolution),
            Box::new(ExhaustiveGuaranteesGradientDoesNot),
            Box::new(ExactExponentialHeuristicPolynomial),
            Box::new(ParetoMultiObjectiveGradientNot),
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
        assert_eq!(OptimizationEntity::variants().len(), 19);
    }

    #[test]
    fn test_step_count() {
        assert_eq!(OptimizationStep::variants().len(), 8);
    }

    #[test]
    fn test_category_laws() {
        check_category_laws::<OptimizationCategory>().unwrap();
    }

    #[test]
    fn test_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<OptimizationTaxonomy>>().unwrap();
    }

    #[test]
    fn test_causal_category_laws() {
        check_category_laws::<CausalCategory<OptimizationCausalGraph>>().unwrap();
    }

    #[test]
    fn test_ontology_validates() {
        OptimizationOntology::validate().unwrap();
    }

    // -- Individual axiom tests --

    #[test]
    fn test_formulation_causes_solution() {
        assert!(FormulationCausesSolution.holds());
    }

    #[test]
    fn test_exhaustive_guarantees_gradient_does_not() {
        assert!(ExhaustiveGuaranteesGradientDoesNot.holds());
    }

    #[test]
    fn test_exact_exponential_heuristic_polynomial() {
        assert!(ExactExponentialHeuristicPolynomial.holds());
    }

    #[test]
    fn test_pareto_multi_objective() {
        assert!(ParetoMultiObjectiveGradientNot.holds());
    }

    // -- Taxonomy tests --

    #[test]
    fn test_methods_are_optimization_methods() {
        use OptimizationEntity::*;
        assert!(taxonomy::is_a::<OptimizationTaxonomy>(
            &ExhaustiveSearch,
            &OptimizationMethod
        ));
        assert!(taxonomy::is_a::<OptimizationTaxonomy>(
            &GradientDescent,
            &OptimizationMethod
        ));
        assert!(taxonomy::is_a::<OptimizationTaxonomy>(
            &GeneticAlgorithm,
            &OptimizationMethod
        ));
        assert!(taxonomy::is_a::<OptimizationTaxonomy>(
            &SimulatedAnnealing,
            &OptimizationMethod
        ));
        assert!(taxonomy::is_a::<OptimizationTaxonomy>(
            &ParetoOptimization,
            &OptimizationMethod
        ));
        assert!(taxonomy::is_a::<OptimizationTaxonomy>(
            &GridSearch,
            &OptimizationMethod
        ));
    }

    #[test]
    fn test_components_are_optimization_components() {
        use OptimizationEntity::*;
        assert!(taxonomy::is_a::<OptimizationTaxonomy>(
            &ObjectiveFunction,
            &OptimizationComponent
        ));
        assert!(taxonomy::is_a::<OptimizationTaxonomy>(
            &Constraint,
            &OptimizationComponent
        ));
        assert!(taxonomy::is_a::<OptimizationTaxonomy>(
            &SearchSpace,
            &OptimizationComponent
        ));
        assert!(taxonomy::is_a::<OptimizationTaxonomy>(
            &FeasibleRegion,
            &OptimizationComponent
        ));
        assert!(taxonomy::is_a::<OptimizationTaxonomy>(
            &OptimalPoint,
            &OptimizationComponent
        ));
        assert!(taxonomy::is_a::<OptimizationTaxonomy>(
            &ParetoFront,
            &OptimizationComponent
        ));
    }

    #[test]
    fn test_properties_are_optimality_properties() {
        use OptimizationEntity::*;
        assert!(taxonomy::is_a::<OptimizationTaxonomy>(
            &Convergence,
            &OptimalityProperty
        ));
        assert!(taxonomy::is_a::<OptimizationTaxonomy>(
            &LocalOptimum,
            &OptimalityProperty
        ));
        assert!(taxonomy::is_a::<OptimizationTaxonomy>(
            &GlobalOptimum,
            &OptimalityProperty
        ));
        assert!(taxonomy::is_a::<OptimizationTaxonomy>(
            &Tradeoff,
            &OptimalityProperty
        ));
    }

    // -- Causal chain tests --

    #[test]
    fn test_full_pipeline_connected() {
        use OptimizationStep::*;
        let effects = causation::effects_of::<OptimizationCausalGraph>(&ProblemFormulation);
        assert!(effects.contains(&SearchSpaceDefinition));
        assert!(effects.contains(&ConstraintSpecification));
        assert!(effects.contains(&ObjectiveEvaluation));
        assert!(effects.contains(&CandidateGeneration));
        assert!(effects.contains(&FeasibilityCheck));
        assert!(effects.contains(&OptimalityAssessment));
        assert!(effects.contains(&SolutionSelection));
    }

    // -- Opposition tests --

    #[test]
    fn test_local_opposes_global() {
        use OptimizationEntity::*;
        assert!(opposition::are_opposed::<OptimizationOpposition>(
            &LocalOptimum,
            &GlobalOptimum
        ));
    }

    #[test]
    fn test_exhaustive_opposes_genetic() {
        use OptimizationEntity::*;
        assert!(opposition::are_opposed::<OptimizationOpposition>(
            &ExhaustiveSearch,
            &GeneticAlgorithm
        ));
    }

    // -- Quality tests --

    #[test]
    fn test_guarantees_global() {
        use OptimizationEntity::*;
        assert_eq!(GuaranteesGlobal.get(&ExhaustiveSearch), Some(true));
        assert_eq!(GuaranteesGlobal.get(&GridSearch), Some(true));
        assert_eq!(GuaranteesGlobal.get(&GradientDescent), Some(false));
        assert_eq!(GuaranteesGlobal.get(&GeneticAlgorithm), Some(false));
        assert_eq!(GuaranteesGlobal.get(&SimulatedAnnealing), Some(false));
    }

    #[test]
    fn test_time_complexity() {
        use OptimizationEntity::*;
        assert_eq!(
            TimeComplexity.get(&ExhaustiveSearch),
            Some(TimeComplexityClass::Exponential)
        );
        assert_eq!(
            TimeComplexity.get(&GradientDescent),
            Some(TimeComplexityClass::Polynomial)
        );
        assert_eq!(
            TimeComplexity.get(&GeneticAlgorithm),
            Some(TimeComplexityClass::Polynomial)
        );
    }

    #[test]
    fn test_handles_multi_objective() {
        use OptimizationEntity::*;
        assert_eq!(HandlesMultiObjective.get(&ParetoOptimization), Some(true));
        assert_eq!(HandlesMultiObjective.get(&GeneticAlgorithm), Some(true));
        assert_eq!(HandlesMultiObjective.get(&GradientDescent), Some(false));
    }

    // -- Proptest --

    fn arb_entity() -> impl Strategy<Value = OptimizationEntity> {
        (0..OptimizationEntity::variants().len()).prop_map(|i| OptimizationEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_taxonomy_reflexive(entity in arb_entity()) {
            prop_assert!(taxonomy::is_a::<OptimizationTaxonomy>(&entity, &entity));
        }

        #[test]
        fn prop_every_entity_has_category(entity in arb_entity()) {
            use OptimizationEntity::*;
            let categories = [OptimizationMethod, OptimizationComponent, OptimalityProperty];
            let belongs = categories.iter().any(|cat| taxonomy::is_a::<OptimizationTaxonomy>(&entity, cat));
            let is_abstract = categories.contains(&entity);
            prop_assert!(belongs || is_abstract,
                "{:?} should belong to at least one category", entity);
        }
    }
}
