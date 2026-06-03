//! Optimization — formalises search methods, objective evaluation,
//! constraint satisfaction, and characterisation of optimal points.
//!
//! This is a PURE-SCIENCE ontology of optimisation — not an
//! implementation of an optimiser.
//!
//! # Literature
//!
//! - **Boyd & Vandenberghe (2004)** *Convex Optimization*, Cambridge
//!   UP — objective functions, constraints, feasibility; convex /
//!   non-convex distinctions.
//! - **Pareto (1906)** *Manuale di Economia Politica*, Società
//!   Editrice Libraria — Pareto optimality; multi-objective tradeoffs.
//! - **Holland (1975)** *Adaptation in Natural and Artificial
//!   Systems*, University of Michigan Press — genetic algorithms.
//! - **Kirkpatrick, Gelatt & Vecchi (1983)** "Optimization by
//!   Simulated Annealing", *Science* 220(4598):671-680.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Optimization",
    source: "Boyd & Vandenberghe (2004) Convex Optimization, Cambridge UP; Pareto (1906) Manuale di Economia Politica; Holland (1975) Adaptation in Natural and Artificial Systems, University of Michigan Press; Kirkpatrick, Gelatt & Vecchi (1983) Optimization by Simulated Annealing, Science 220(4598):671-680",

    concepts: [
        // === Methods ===
        ExhaustiveSearch,
        GradientDescent,
        GeneticAlgorithm,
        SimulatedAnnealing,
        ParetoOptimization,
        GridSearch,
        // === Components ===
        ObjectiveFunction,
        Constraint,
        SearchSpace,
        FeasibleRegion,
        OptimalPoint,
        ParetoFront,
        // === Properties ===
        Convergence,
        LocalOptimum,
        GlobalOptimum,
        Tradeoff,
        // === Abstract categories ===
        OptimizationMethod,
        OptimizationComponent,
        OptimalityProperty,
        // === Pipeline stages ===
        ProblemFormulation,
        SearchSpaceDefinition,
        ConstraintSpecification,
        ObjectiveEvaluation,
        CandidateGeneration,
        FeasibilityCheck,
        OptimalityAssessment,
        SolutionSelection,
    ],

    labels: {
        ExhaustiveSearch: ("en", "Exhaustive search",
            "Boyd & Vandenberghe (2004): enumerate the entire search space - guarantees the global optimum but is exponential."),
        GradientDescent: ("en", "Gradient descent",
            "Boyd & Vandenberghe (2004): local-gradient method - polynomial-time but can stick in local optima."),
        GeneticAlgorithm: ("en", "Genetic algorithm",
            "Holland (1975): population-based stochastic search."),
        SimulatedAnnealing: ("en", "Simulated annealing",
            "Kirkpatrick, Gelatt & Vecchi (1983): probabilistic acceptance with cooling schedule."),
        ParetoOptimization: ("en", "Pareto optimization",
            "Pareto (1906): finds the Pareto-optimal front in multi-objective space."),
        GridSearch: ("en", "Grid search",
            "Enumerate solutions on a discretized grid."),
        ObjectiveFunction: ("en", "Objective function",
            "Boyd & Vandenberghe (2004): the scalar (or vector) function being minimised / maximised."),
        Constraint: ("en", "Constraint",
            "Boyd & Vandenberghe (2004): a condition the solution must satisfy."),
        SearchSpace: ("en", "Search space",
            "The set of candidate solutions."),
        FeasibleRegion: ("en", "Feasible region",
            "Boyd & Vandenberghe (2004): the subset of the search space satisfying all constraints."),
        OptimalPoint: ("en", "Optimal point",
            "A solution at which the objective attains its optimum value."),
        ParetoFront: ("en", "Pareto front",
            "Pareto (1906): the set of non-dominated solutions in multi-objective space."),
        Convergence: ("en", "Convergence",
            "Boyd & Vandenberghe (2004): the property that the iteration approaches an optimum as time grows."),
        LocalOptimum: ("en", "Local optimum",
            "A solution that is optimal within a neighbourhood."),
        GlobalOptimum: ("en", "Global optimum",
            "A solution that is optimal over the entire search space."),
        Tradeoff: ("en", "Tradeoff",
            "Pareto (1906): a multi-objective compromise - improving one objective worsens another."),
        OptimizationMethod: ("en", "Optimization method", "Abstract category for optimisation methods."),
        OptimizationComponent: ("en", "Optimization component", "Abstract category for optimisation problem components."),
        OptimalityProperty: ("en", "Optimality property", "Abstract category for properties characterising optimal solutions."),

        ProblemFormulation: ("en", "Problem formulation", "Pipeline stage 1: state the optimisation problem."),
        SearchSpaceDefinition: ("en", "Search space definition", "Pipeline stage 2: define the candidate solution space."),
        ConstraintSpecification: ("en", "Constraint specification", "Pipeline stage 3: specify what solutions must satisfy."),
        ObjectiveEvaluation: ("en", "Objective evaluation", "Pipeline stage 4: evaluate objective on candidates."),
        CandidateGeneration: ("en", "Candidate generation", "Pipeline stage 5: generate candidate solutions."),
        FeasibilityCheck: ("en", "Feasibility check", "Pipeline stage 6: check feasibility against constraints."),
        OptimalityAssessment: ("en", "Optimality assessment", "Pipeline stage 7: assess optimality of feasible candidates."),
        SolutionSelection: ("en", "Solution selection", "Pipeline stage 8: select the best solution."),
    },

    is_a: [
        (ExhaustiveSearch, OptimizationMethod),
        (GradientDescent, OptimizationMethod),
        (GeneticAlgorithm, OptimizationMethod),
        (SimulatedAnnealing, OptimizationMethod),
        (ParetoOptimization, OptimizationMethod),
        (GridSearch, OptimizationMethod),
        (ObjectiveFunction, OptimizationComponent),
        (Constraint, OptimizationComponent),
        (SearchSpace, OptimizationComponent),
        (FeasibleRegion, OptimizationComponent),
        (OptimalPoint, OptimizationComponent),
        (ParetoFront, OptimizationComponent),
        (Convergence, OptimalityProperty),
        (LocalOptimum, OptimalityProperty),
        (GlobalOptimum, OptimalityProperty),
        (Tradeoff, OptimalityProperty),
    ],

    causes: [
        (ProblemFormulation, SearchSpaceDefinition),
        (SearchSpaceDefinition, ConstraintSpecification),
        (ConstraintSpecification, ObjectiveEvaluation),
        (ObjectiveEvaluation, CandidateGeneration),
        (CandidateGeneration, FeasibilityCheck),
        (FeasibilityCheck, OptimalityAssessment),
        (OptimalityAssessment, SolutionSelection),
    ],

    opposes: [
        // Boyd & Vandenberghe (2004): local vs global - the central tension.
        (LocalOptimum, GlobalOptimum),
        (GlobalOptimum, LocalOptimum),
        // Exact vs heuristic.
        (ExhaustiveSearch, GeneticAlgorithm),
        (GeneticAlgorithm, ExhaustiveSearch),
    ],
}

/// Time complexity class. Boyd & Vandenberghe (2004): polynomial =
/// tractable; exponential = intractable in general.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeComplexityClass {
    Polynomial,
    Exponential,
}

/// Quality: does the method guarantee finding the global optimum?
#[derive(Debug, Clone)]
pub struct GuaranteesGlobal;

impl Quality for GuaranteesGlobal {
    type Individual = OptimizationConcept;
    type Value = bool;

    fn get(&self, c: &OptimizationConcept) -> Option<bool> {
        use OptimizationConcept as O;
        match c {
            O::ExhaustiveSearch | O::GridSearch => Some(true),
            O::GradientDescent
            | O::GeneticAlgorithm
            | O::SimulatedAnnealing
            | O::ParetoOptimization => Some(false),
            _ => None,
        }
    }
}

/// Quality: time complexity class of the method.
#[derive(Debug, Clone)]
pub struct TimeComplexity;

impl Quality for TimeComplexity {
    type Individual = OptimizationConcept;
    type Value = TimeComplexityClass;

    fn get(&self, c: &OptimizationConcept) -> Option<TimeComplexityClass> {
        use OptimizationConcept as O;
        match c {
            O::GradientDescent
            | O::GeneticAlgorithm
            | O::SimulatedAnnealing
            | O::ParetoOptimization => Some(TimeComplexityClass::Polynomial),
            O::ExhaustiveSearch | O::GridSearch => Some(TimeComplexityClass::Exponential),
            _ => None,
        }
    }
}

/// Quality: can the method handle multi-objective optimisation?
#[derive(Debug, Clone)]
pub struct HandlesMultiObjective;

impl Quality for HandlesMultiObjective {
    type Individual = OptimizationConcept;
    type Value = bool;

    fn get(&self, c: &OptimizationConcept) -> Option<bool> {
        use OptimizationConcept as O;
        match c {
            O::ParetoOptimization | O::GeneticAlgorithm | O::ExhaustiveSearch | O::GridSearch => {
                Some(true)
            }
            O::GradientDescent | O::SimulatedAnnealing => Some(false),
            _ => None,
        }
    }
}

// Legacy alias for the old enum name.
pub type OptimizationEntity = OptimizationConcept;

impl Ontology for OptimizationOntology {
    type Cat = OptimizationCategory;
    type Qual = GuaranteesGlobal;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(ExhaustiveGuaranteesGradientDoesNot));
        axioms.push(Box::new(ExactExponentialHeuristicPolynomial));
        axioms.push(Box::new(ParetoMultiObjectiveGradientNot));
        axioms
    }
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

pub struct ExhaustiveGuaranteesGradientDoesNot;

impl Axiom for ExhaustiveGuaranteesGradientDoesNot {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use OptimizationConcept as O;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if GuaranteesGlobal.get(&O::ExhaustiveSearch) == Some(true)
            && GuaranteesGlobal.get(&O::GradientDescent) == Some(false)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ExhaustiveGuaranteesGradientDoesNot",
        "ExhaustiveSearch guarantees the global optimum; GradientDescent does not",
        "Boyd & Vandenberghe (2004) Convex Optimization, Cambridge UP"
    );
}

pr4xis::register_axiom!(
    ExhaustiveGuaranteesGradientDoesNot,
    "Boyd & Vandenberghe (2004) Convex Optimization, Cambridge UP"
);

pub struct ExactExponentialHeuristicPolynomial;

impl Axiom for ExactExponentialHeuristicPolynomial {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use OptimizationConcept as O;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if TimeComplexity.get(&O::ExhaustiveSearch) == Some(TimeComplexityClass::Exponential)
            && TimeComplexity.get(&O::GeneticAlgorithm) == Some(TimeComplexityClass::Polynomial)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ExactExponentialHeuristicPolynomial",
        "Exact methods (ExhaustiveSearch) are exponential; heuristic methods (GeneticAlgorithm) are polynomial",
        "Holland (1975) Adaptation in Natural and Artificial Systems"
    );
}

pr4xis::register_axiom!(
    ExactExponentialHeuristicPolynomial,
    "Holland (1975) Adaptation in Natural and Artificial Systems"
);

pub struct ParetoMultiObjectiveGradientNot;

impl Axiom for ParetoMultiObjectiveGradientNot {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use OptimizationConcept as O;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if HandlesMultiObjective.get(&O::ParetoOptimization) == Some(true)
            && HandlesMultiObjective.get(&O::GradientDescent) == Some(false)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ParetoMultiObjectiveGradientNot",
        "ParetoOptimization handles multi-objective; GradientDescent does not",
        "Pareto (1906) Manuale di Economia Politica"
    );
}

pr4xis::register_axiom!(
    ParetoMultiObjectiveGradientNot,
    "Pareto (1906) Manuale di Economia Politica"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<OptimizationCategory>();
    }

    #[test]
    fn ontology_validates() {
        OptimizationOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn pipeline_reaches_solution_selection() {
        let caus: Vec<_> = OptimizationCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == OptimizationRelationKind::Causation)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(caus.contains(&(
            OptimizationConcept::ProblemFormulation,
            OptimizationConcept::SolutionSelection
        )));
    }

    #[test]
    fn local_opposes_global() {
        let opp: Vec<_> = OptimizationCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == OptimizationRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opp.contains(&(
            OptimizationConcept::LocalOptimum,
            OptimizationConcept::GlobalOptimum
        )));
    }

    #[test]
    fn methods_subsume_optimization_method() {
        use OptimizationConcept as O;
        let sub: Vec<_> = OptimizationCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == OptimizationRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for method in [
            O::ExhaustiveSearch,
            O::GradientDescent,
            O::GeneticAlgorithm,
            O::SimulatedAnnealing,
            O::ParetoOptimization,
            O::GridSearch,
        ] {
            assert!(sub.contains(&(method, O::OptimizationMethod)));
        }
    }

    #[test]
    fn all_axioms_hold() {
        for axiom in OptimizationOntology::axioms() {
            if let Err(c) = axiom.verify() {
                panic!(
                    "axiom failed: {} - {}",
                    c.meta().name.as_str(),
                    c.meta().description.as_str()
                );
            }
        }
    }

    fn arb_concept() -> impl Strategy<Value = OptimizationConcept> {
        proptest::sample::select(OptimizationConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in OptimizationCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_guarantees_global_total_on_methods(c in arb_concept()) {
            use OptimizationConcept as O;
            let v = GuaranteesGlobal.get(&c);
            let is_method = matches!(c,
                O::ExhaustiveSearch | O::GradientDescent | O::GeneticAlgorithm
                | O::SimulatedAnnealing | O::ParetoOptimization | O::GridSearch);
            prop_assert_eq!(v.is_some(), is_method);
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in OptimizationOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }
    }
}
