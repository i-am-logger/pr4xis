#[cfg(test)]
use super::composition::Evaluation;
/// Logical inference: deduction, induction, and abduction.
///
/// These operate on propositions and axioms to derive new knowledge.
///
/// - **Deduction**: if A and A→B, then B (guaranteed truth via modus ponens)
/// - **Induction**: observed pattern across instances → general rule (probable truth)
/// - **Abduction**: observation + rule → best explanation (hypothesis)
use super::composition::Proposition;
use std::fmt::Debug;

/// A deductive inference: given premises, a rule (A→B), and a conclusion, derive truth.
///
/// Modus ponens: if all premises hold and the rule holds, the conclusion is guaranteed.
/// The rule must encode the logical connection between premises and conclusion
/// (typically an `Implies` proposition). Without the rule, checking premises and
/// conclusion independently would be a coincidence check, not entailment.
#[derive(Debug)]
pub struct Deduction<Ctx: Debug> {
    pub premises: Vec<Box<dyn Proposition<Context = Ctx>>>,
    pub rule: Box<dyn Proposition<Context = Ctx>>,
    pub conclusion: Box<dyn Proposition<Context = Ctx>>,
    pub name: String,
}

/// Result of a deductive inference.
#[derive(Debug, Clone, PartialEq)]
pub enum DeductionResult {
    /// All premises hold, rule holds, conclusion holds — valid deduction.
    Valid { conclusion: String },
    /// All premises hold but the rule doesn't — the implication is unsound.
    Unsound { conclusion: String, reason: String },
    /// Premises don't hold — deduction is vacuously valid but not applicable.
    Inapplicable {
        failed_premise: String,
        reason: String,
    },
}

impl<Ctx: Debug> Deduction<Ctx> {
    pub fn new(
        name: impl Into<String>,
        premises: Vec<Box<dyn Proposition<Context = Ctx>>>,
        rule: Box<dyn Proposition<Context = Ctx>>,
        conclusion: Box<dyn Proposition<Context = Ctx>>,
    ) -> Self {
        Self {
            premises,
            rule,
            conclusion,
            name: name.into(),
        }
    }

    /// Apply the deduction to a context.
    pub fn apply(&self, context: &Ctx) -> DeductionResult {
        // Check all premises
        for premise in &self.premises {
            let result = premise.evaluate(context);
            if !result.is_satisfied() {
                return DeductionResult::Inapplicable {
                    failed_premise: premise.describe(),
                    reason: result.reason().to_string(),
                };
            }
        }

        // Premises hold — check the rule (the logical connection A→B)
        let rule_result = self.rule.evaluate(context);
        if !rule_result.is_satisfied() {
            return DeductionResult::Unsound {
                conclusion: self.conclusion.describe(),
                reason: rule_result.reason().to_string(),
            };
        }

        // Rule holds — conclusion must follow (verify as safety check)
        let conclusion_result = self.conclusion.evaluate(context);
        if conclusion_result.is_satisfied() {
            DeductionResult::Valid {
                conclusion: self.conclusion.describe(),
            }
        } else {
            // Rule passed but conclusion failed — mismatch between rule and conclusion
            DeductionResult::Unsound {
                conclusion: self.conclusion.describe(),
                reason: format!(
                    "rule '{}' passed but conclusion failed: {}",
                    self.rule.describe(),
                    conclusion_result.reason()
                ),
            }
        }
    }
}

/// An inductive inference: observe a pattern across many instances, generalize to a rule.
///
/// Given a proposition and a set of test cases, induction checks whether the proposition
/// holds for all of them. If it does, the generalization is supported (but not proven —
/// induction gives confidence, not certainty).
#[derive(Debug)]
pub struct Induction<Ctx: Debug> {
    pub hypothesis: Box<dyn Proposition<Context = Ctx>>,
    pub name: String,
}

/// Result of an inductive inference.
#[derive(Debug, Clone, PartialEq)]
pub struct InductionResult {
    pub supported: bool,
    pub instances_checked: usize,
    pub counterexamples: Vec<String>,
}

impl<Ctx: Debug> Induction<Ctx> {
    pub fn new(name: impl Into<String>, hypothesis: Box<dyn Proposition<Context = Ctx>>) -> Self {
        Self {
            hypothesis,
            name: name.into(),
        }
    }

    /// Test the hypothesis against a set of instances.
    /// Returns how many held and any counterexamples found.
    /// Empty input returns supported=true (vacuous truth — no counterexamples exist).
    pub fn test(&self, instances: &[Ctx]) -> InductionResult {
        let mut counterexamples = Vec::new();

        for instance in instances {
            let result = self.hypothesis.evaluate(instance);
            if !result.is_satisfied() {
                counterexamples.push(format!("{:?}: {}", instance, result.reason()));
            }
        }

        InductionResult {
            supported: counterexamples.is_empty(),
            instances_checked: instances.len(),
            counterexamples,
        }
    }
}

/// An abductive inference: given an observation and a set of candidate explanations,
/// find the best explanation.
///
/// Each explanation is a proposition that, if true, would account for the observation.
/// Abduction selects the explanation that holds in the current context — the most
/// consistent hypothesis.
#[derive(Debug)]
pub struct Abduction<Ctx: Debug> {
    pub observation: Box<dyn Proposition<Context = Ctx>>,
    pub explanations: Vec<Box<dyn Proposition<Context = Ctx>>>,
    pub name: String,
}

/// Result of an abductive inference.
#[derive(Debug, Clone, PartialEq)]
pub enum AbductionResult {
    /// One or more explanations are consistent with the observation.
    Explained {
        first_consistent: String,
        all_consistent: Vec<String>,
    },
    /// The observation holds but no explanation accounts for it.
    Unexplained { observation: String },
    /// The observation doesn't hold — nothing to explain.
    NoObservation { reason: String },
}

impl<Ctx: Debug> Abduction<Ctx> {
    pub fn new(
        name: impl Into<String>,
        observation: Box<dyn Proposition<Context = Ctx>>,
        explanations: Vec<Box<dyn Proposition<Context = Ctx>>>,
    ) -> Self {
        Self {
            observation,
            explanations,
            name: name.into(),
        }
    }

    /// Given a context, check the observation and find consistent explanations.
    pub fn infer(&self, context: &Ctx) -> AbductionResult {
        // First check if the observation holds
        let obs_result = self.observation.evaluate(context);
        if !obs_result.is_satisfied() {
            return AbductionResult::NoObservation {
                reason: obs_result.reason().to_string(),
            };
        }

        // Find all explanations consistent with the context
        let consistent: Vec<String> = self
            .explanations
            .iter()
            .filter(|e| e.evaluate(context).is_satisfied())
            .map(|e| e.describe())
            .collect();

        if consistent.is_empty() {
            AbductionResult::Unexplained {
                observation: self.observation.describe(),
            }
        } else {
            AbductionResult::Explained {
                first_consistent: consistent[0].clone(),
                all_consistent: consistent,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::composition::Implies;
    use proptest::prelude::*;

    // --- Test propositions ---

    #[derive(Debug)]
    struct IsPositive;
    impl Proposition for IsPositive {
        type Context = i32;
        fn evaluate(&self, n: &i32) -> Evaluation {
            if *n > 0 {
                Evaluation::Satisfied {
                    reason: format!("{} > 0", n),
                }
            } else {
                Evaluation::Violated {
                    reason: format!("{} <= 0", n),
                }
            }
        }
        fn describe(&self) -> String {
            "is positive".into()
        }
    }

    #[derive(Debug)]
    struct IsEven;
    impl Proposition for IsEven {
        type Context = i32;
        fn evaluate(&self, n: &i32) -> Evaluation {
            if n % 2 == 0 {
                Evaluation::Satisfied {
                    reason: format!("{} is even", n),
                }
            } else {
                Evaluation::Violated {
                    reason: format!("{} is odd", n),
                }
            }
        }
        fn describe(&self) -> String {
            "is even".into()
        }
    }

    #[derive(Debug)]
    struct LessThan(i32);
    impl Proposition for LessThan {
        type Context = i32;
        fn evaluate(&self, n: &i32) -> Evaluation {
            if *n < self.0 {
                Evaluation::Satisfied {
                    reason: format!("{} < {}", n, self.0),
                }
            } else {
                Evaluation::Violated {
                    reason: format!("{} >= {}", n, self.0),
                }
            }
        }
        fn describe(&self) -> String {
            format!("less than {}", self.0)
        }
    }

    // --- Deduction tests ---

    #[test]
    fn test_deduction_valid() {
        // Modus ponens: if positive, and positive→positive, then positive
        let d = Deduction::new(
            "positive implies positive",
            vec![Box::new(IsPositive)],
            Box::new(Implies::new(IsPositive, IsPositive)),
            Box::new(IsPositive),
        );
        let result = d.apply(&5);
        assert!(matches!(result, DeductionResult::Valid { .. }));
    }

    #[test]
    fn test_deduction_inapplicable() {
        let d = Deduction::new(
            "positive implies even",
            vec![Box::new(IsPositive)],
            Box::new(Implies::new(IsPositive, IsEven)),
            Box::new(IsEven),
        );
        // -4 is not positive, so premise doesn't hold
        let result = d.apply(&-4);
        assert!(matches!(result, DeductionResult::Inapplicable { .. }));
    }

    #[test]
    fn test_deduction_unsound() {
        // "If positive then even" — unsound for 3 (positive but odd)
        let d = Deduction::new(
            "positive implies even",
            vec![Box::new(IsPositive)],
            Box::new(Implies::new(IsPositive, IsEven)),
            Box::new(IsEven),
        );
        let result = d.apply(&3);
        assert!(matches!(result, DeductionResult::Unsound { .. }));
    }

    #[test]
    fn test_deduction_multiple_premises() {
        // If positive AND even, then less than 100
        let d = Deduction::new(
            "small positive even",
            vec![Box::new(IsPositive), Box::new(IsEven)],
            Box::new(LessThan(100)),
            Box::new(LessThan(100)),
        );
        assert!(matches!(d.apply(&4), DeductionResult::Valid { .. }));
        assert!(matches!(d.apply(&3), DeductionResult::Inapplicable { .. }));
    }

    // --- Induction tests ---

    #[test]
    fn test_induction_supported() {
        let ind = Induction::new("all positive", Box::new(IsPositive));
        let instances = vec![1, 2, 3, 4, 5];
        let result = ind.test(&instances);
        assert!(result.supported);
        assert_eq!(result.instances_checked, 5);
        assert!(result.counterexamples.is_empty());
    }

    #[test]
    fn test_induction_refuted() {
        let ind = Induction::new("all even", Box::new(IsEven));
        let instances = vec![2, 4, 5, 6];
        let result = ind.test(&instances);
        assert!(!result.supported);
        assert_eq!(result.counterexamples.len(), 1);
    }

    #[test]
    fn test_induction_empty() {
        let ind = Induction::new("no evidence", Box::new(IsPositive));
        let result = ind.test(&[]);
        assert!(result.supported); // vacuous truth
        assert_eq!(result.instances_checked, 0);
        assert!(result.counterexamples.is_empty());
    }

    // --- Abduction tests ---

    #[test]
    fn test_abduction_explained() {
        // Observation: n is positive
        // Explanations: n is even, n < 100
        let abd = Abduction::new(
            "why positive?",
            Box::new(IsPositive),
            vec![Box::new(IsEven), Box::new(LessThan(100))],
        );
        // 4 is positive, even, and < 100 — both explanations consistent
        let result = abd.infer(&4);
        match result {
            AbductionResult::Explained { all_consistent, .. } => {
                assert_eq!(all_consistent.len(), 2);
            }
            _ => panic!("expected Explained"),
        }
    }

    #[test]
    fn test_abduction_partial() {
        let abd = Abduction::new(
            "why positive?",
            Box::new(IsPositive),
            vec![Box::new(IsEven), Box::new(LessThan(100))],
        );
        // 3 is positive and < 100, but not even — one explanation consistent
        let result = abd.infer(&3);
        match result {
            AbductionResult::Explained { all_consistent, .. } => {
                assert_eq!(all_consistent.len(), 1);
            }
            _ => panic!("expected Explained"),
        }
    }

    #[test]
    fn test_abduction_unexplained() {
        let abd = Abduction::new(
            "why positive?",
            Box::new(IsPositive),
            vec![Box::new(IsEven), Box::new(LessThan(5))],
        );
        // 7 is positive but odd and >= 5 — no explanation works
        let result = abd.infer(&7);
        assert!(matches!(result, AbductionResult::Unexplained { .. }));
    }

    #[test]
    fn test_abduction_no_observation() {
        let abd = Abduction::new(
            "why positive?",
            Box::new(IsPositive),
            vec![Box::new(IsEven)],
        );
        // -4 is not positive — nothing to explain
        let result = abd.infer(&-4);
        assert!(matches!(result, AbductionResult::NoObservation { .. }));
    }

    proptest! {
        /// Deduction with identical premise, tautological rule, and conclusion is always valid or inapplicable
        #[test]
        fn prop_deduction_tautology(n in -100..100i32) {
            let d = Deduction::new(
                "tautology",
                vec![Box::new(IsPositive)],
                Box::new(Implies::new(IsPositive, IsPositive)),
                Box::new(IsPositive),
            );
            let result = d.apply(&n);
            let is_unsound = matches!(result, DeductionResult::Unsound { .. });
            prop_assert!(!is_unsound);
        }

        /// Induction with all instances satisfying → supported
        #[test]
        fn prop_induction_positive_instances(n in 1..100i32) {
            let instances: Vec<i32> = (1..=n).collect();
            let ind = Induction::new("all positive", Box::new(IsPositive));
            let result = ind.test(&instances);
            prop_assert!(result.supported);
        }

        /// Induction counterexample count is correct
        #[test]
        fn prop_induction_counterexample_count(n in 1..50i32) {
            let instances: Vec<i32> = (-n..=n).collect();
            let ind = Induction::new("all positive", Box::new(IsPositive));
            let result = ind.test(&instances);
            let expected_failures = (n + 1) as usize; // -n..0 inclusive
            prop_assert_eq!(result.counterexamples.len(), expected_failures);
        }

        /// Abduction: if observation doesn't hold, always NoObservation
        #[test]
        fn prop_abduction_no_observation(n in -100..-1i32) {
            let abd = Abduction::new(
                "why positive?",
                Box::new(IsPositive),
                vec![Box::new(IsEven)],
            );
            let result = abd.infer(&n);
            let is_no_obs = matches!(result, AbductionResult::NoObservation { .. });
            prop_assert!(is_no_obs);
        }

        /// Abduction: consistent explanations count <= total explanations
        #[test]
        fn prop_abduction_consistent_bounded(n in 1..100i32) {
            let abd = Abduction::new(
                "why positive?",
                Box::new(IsPositive),
                vec![
                    Box::new(IsEven),
                    Box::new(LessThan(50)),
                    Box::new(LessThan(100)),
                ],
            );
            if let AbductionResult::Explained { all_consistent, .. } = abd.infer(&n) {
                prop_assert!(all_consistent.len() <= 3);
            }
        }
    }
}
