/// Generic logical composition: Proposition, AllOf, AnyOf, Not, Implies, Compare, Threshold.
///
/// This is the boolean composition layer for ANY ontology or enforcement engine.
/// Compose rules with AND/OR/NOT to build complex enforcement.
use std::fmt::Debug;

/// A logical proposition that can be evaluated to true/false with a reason.
pub trait Proposition: Debug {
    /// The context needed to evaluate this proposition.
    type Context;

    /// Evaluate the proposition. Returns (satisfied, reason).
    fn evaluate(&self, context: &Self::Context) -> Evaluation;

    /// Human-readable description of what this proposition checks.
    fn describe(&self) -> String;
}

/// Result of evaluating a proposition.
#[derive(Debug, Clone, PartialEq)]
pub enum Evaluation {
    Satisfied { reason: String },
    Violated { reason: String },
}

impl Evaluation {
    pub fn is_satisfied(&self) -> bool {
        matches!(self, Evaluation::Satisfied { .. })
    }

    pub fn reason(&self) -> &str {
        match self {
            Evaluation::Satisfied { reason } => reason,
            Evaluation::Violated { reason } => reason,
        }
    }
}

/// Logical AND: all propositions must be satisfied.
/// Uses trait objects so you can mix different proposition types.
pub struct AllOf<Ctx> {
    pub propositions: Vec<Box<dyn Proposition<Context = Ctx>>>,
}

impl<Ctx: Debug> AllOf<Ctx> {
    pub fn new(propositions: Vec<Box<dyn Proposition<Context = Ctx>>>) -> Self {
        Self { propositions }
    }
}

impl<Ctx: Debug> Debug for AllOf<Ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AllOf({} props)", self.propositions.len())
    }
}

impl<Ctx: Debug> Proposition for AllOf<Ctx> {
    type Context = Ctx;

    fn evaluate(&self, context: &Self::Context) -> Evaluation {
        for prop in &self.propositions {
            let result = prop.evaluate(context);
            if let Evaluation::Violated { reason } = result {
                return Evaluation::Violated {
                    reason: format!("AllOf failed: {} — {}", prop.describe(), reason),
                };
            }
        }
        Evaluation::Satisfied {
            reason: format!("all {} conditions met", self.propositions.len()),
        }
    }

    fn describe(&self) -> String {
        let descs: Vec<String> = self.propositions.iter().map(|p| p.describe()).collect();
        format!("ALL({})", descs.join(", "))
    }
}

/// Logical OR: at least one proposition must be satisfied.
pub struct AnyOf<Ctx> {
    pub propositions: Vec<Box<dyn Proposition<Context = Ctx>>>,
}

impl<Ctx: Debug> AnyOf<Ctx> {
    pub fn new(propositions: Vec<Box<dyn Proposition<Context = Ctx>>>) -> Self {
        Self { propositions }
    }
}

impl<Ctx: Debug> Debug for AnyOf<Ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AnyOf({} props)", self.propositions.len())
    }
}

impl<Ctx: Debug> Proposition for AnyOf<Ctx> {
    type Context = Ctx;

    fn evaluate(&self, context: &Self::Context) -> Evaluation {
        let mut violations = Vec::new();
        for prop in &self.propositions {
            let result = prop.evaluate(context);
            if result.is_satisfied() {
                return Evaluation::Satisfied {
                    reason: format!("AnyOf satisfied: {}", prop.describe()),
                };
            }
            violations.push(format!("{}: {}", prop.describe(), result.reason()));
        }
        Evaluation::Violated {
            reason: format!(
                "AnyOf failed: none of [{}] satisfied",
                violations.join("; ")
            ),
        }
    }

    fn describe(&self) -> String {
        let descs: Vec<String> = self.propositions.iter().map(|p| p.describe()).collect();
        format!("ANY({})", descs.join(", "))
    }
}

/// Logical NOT: the proposition must NOT be satisfied.
#[derive(Debug)]
pub struct Not<P: Proposition> {
    pub proposition: P,
}

impl<P: Proposition> Not<P> {
    pub fn new(proposition: P) -> Self {
        Self { proposition }
    }
}

impl<P: Proposition> Proposition for Not<P> {
    type Context = P::Context;

    fn evaluate(&self, context: &Self::Context) -> Evaluation {
        match self.proposition.evaluate(context) {
            Evaluation::Satisfied { reason } => Evaluation::Violated {
                reason: format!(
                    "NOT failed: {} was satisfied — {}",
                    self.proposition.describe(),
                    reason
                ),
            },
            Evaluation::Violated { reason } => Evaluation::Satisfied {
                reason: format!(
                    "NOT satisfied: {} was violated — {}",
                    self.proposition.describe(),
                    reason
                ),
            },
        }
    }

    fn describe(&self) -> String {
        format!("NOT({})", self.proposition.describe())
    }
}

/// Logical implication: if A then B. A and B can be different proposition types.
#[derive(Debug)]
pub struct Implies<A: Proposition, B: Proposition<Context = A::Context>> {
    pub antecedent: A,
    pub consequent: B,
}

impl<A: Proposition, B: Proposition<Context = A::Context>> Implies<A, B> {
    pub fn new(antecedent: A, consequent: B) -> Self {
        Self {
            antecedent,
            consequent,
        }
    }
}

impl<A: Proposition, B: Proposition<Context = A::Context>> Proposition for Implies<A, B> {
    type Context = A::Context;

    fn evaluate(&self, context: &Self::Context) -> Evaluation {
        match self.antecedent.evaluate(context) {
            Evaluation::Violated { .. } => Evaluation::Satisfied {
                reason: format!(
                    "implication vacuously true: {} not met",
                    self.antecedent.describe()
                ),
            },
            Evaluation::Satisfied { .. } => match self.consequent.evaluate(context) {
                Evaluation::Satisfied { reason } => Evaluation::Satisfied {
                    reason: format!(
                        "{} → {} holds: {}",
                        self.antecedent.describe(),
                        self.consequent.describe(),
                        reason
                    ),
                },
                Evaluation::Violated { reason } => Evaluation::Violated {
                    reason: format!(
                        "{} is true but {} failed: {}",
                        self.antecedent.describe(),
                        self.consequent.describe(),
                        reason
                    ),
                },
            },
        }
    }

    fn describe(&self) -> String {
        format!(
            "IF {} THEN {}",
            self.antecedent.describe(),
            self.consequent.describe()
        )
    }
}

// =============================================================================
// Comparison propositions — ordering relationships
// =============================================================================

/// A value that can be extracted from a context and compared.
pub trait Measurable<Ctx>: Debug {
    type Value: PartialOrd + Debug;

    fn measure(&self, context: &Ctx) -> Self::Value;
    fn name(&self) -> &str;
}

/// Comparison operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
    Equal,
    NotEqual,
}

impl CompareOp {
    pub fn symbol(&self) -> &'static str {
        match self {
            CompareOp::LessThan => "<",
            CompareOp::LessOrEqual => "<=",
            CompareOp::GreaterThan => ">",
            CompareOp::GreaterOrEqual => ">=",
            CompareOp::Equal => "==",
            CompareOp::NotEqual => "!=",
        }
    }

    pub fn check<T: PartialOrd>(&self, left: &T, right: &T) -> bool {
        match self {
            CompareOp::LessThan => left < right,
            CompareOp::LessOrEqual => left <= right,
            CompareOp::GreaterThan => left > right,
            CompareOp::GreaterOrEqual => left >= right,
            CompareOp::Equal => left == right,
            CompareOp::NotEqual => left != right,
        }
    }
}

/// Compare two measurable values from the same context.
#[derive(Debug)]
pub struct Compare<Ctx, L: Measurable<Ctx>, R: Measurable<Ctx>> {
    pub left: L,
    pub op: CompareOp,
    pub right: R,
    _ctx: std::marker::PhantomData<Ctx>,
}

impl<Ctx, L, R> Compare<Ctx, L, R>
where
    L: Measurable<Ctx>,
    R: Measurable<Ctx, Value = L::Value>,
{
    pub fn new(left: L, op: CompareOp, right: R) -> Self {
        Self {
            left,
            op,
            right,
            _ctx: std::marker::PhantomData,
        }
    }
}

impl<Ctx, L, R> Proposition for Compare<Ctx, L, R>
where
    Ctx: Debug,
    L: Measurable<Ctx>,
    R: Measurable<Ctx, Value = L::Value>,
{
    type Context = Ctx;

    fn evaluate(&self, context: &Self::Context) -> Evaluation {
        let lv = self.left.measure(context);
        let rv = self.right.measure(context);
        if self.op.check(&lv, &rv) {
            Evaluation::Satisfied {
                reason: format!(
                    "{} ({:?}) {} {} ({:?})",
                    self.left.name(),
                    lv,
                    self.op.symbol(),
                    self.right.name(),
                    rv
                ),
            }
        } else {
            Evaluation::Violated {
                reason: format!(
                    "{} ({:?}) is NOT {} {} ({:?})",
                    self.left.name(),
                    lv,
                    self.op.symbol(),
                    self.right.name(),
                    rv
                ),
            }
        }
    }

    fn describe(&self) -> String {
        format!(
            "{} {} {}",
            self.left.name(),
            self.op.symbol(),
            self.right.name()
        )
    }
}

/// Convenience: compare a measurable against a constant.
#[derive(Debug)]
pub struct Threshold<Ctx, M: Measurable<Ctx>> {
    pub measurable: M,
    pub op: CompareOp,
    pub threshold: M::Value,
    _ctx: std::marker::PhantomData<Ctx>,
}

impl<Ctx, M> Threshold<Ctx, M>
where
    M: Measurable<Ctx>,
    M::Value: Clone,
{
    pub fn new(measurable: M, op: CompareOp, threshold: M::Value) -> Self {
        Self {
            measurable,
            op,
            threshold,
            _ctx: std::marker::PhantomData,
        }
    }

    pub fn less_than(measurable: M, threshold: M::Value) -> Self {
        Self::new(measurable, CompareOp::LessThan, threshold)
    }

    pub fn greater_than(measurable: M, threshold: M::Value) -> Self {
        Self::new(measurable, CompareOp::GreaterThan, threshold)
    }

    pub fn at_least(measurable: M, threshold: M::Value) -> Self {
        Self::new(measurable, CompareOp::GreaterOrEqual, threshold)
    }

    pub fn at_most(measurable: M, threshold: M::Value) -> Self {
        Self::new(measurable, CompareOp::LessOrEqual, threshold)
    }
}

impl<Ctx, M> Proposition for Threshold<Ctx, M>
where
    Ctx: Debug,
    M: Measurable<Ctx>,
    M::Value: Clone,
{
    type Context = Ctx;

    fn evaluate(&self, context: &Self::Context) -> Evaluation {
        let value = self.measurable.measure(context);
        if self.op.check(&value, &self.threshold) {
            Evaluation::Satisfied {
                reason: format!(
                    "{} ({:?}) {} {:?}",
                    self.measurable.name(),
                    value,
                    self.op.symbol(),
                    self.threshold
                ),
            }
        } else {
            Evaluation::Violated {
                reason: format!(
                    "{} ({:?}) is NOT {} {:?}",
                    self.measurable.name(),
                    value,
                    self.op.symbol(),
                    self.threshold
                ),
            }
        }
    }

    fn describe(&self) -> String {
        format!(
            "{} {} {:?}",
            self.measurable.name(),
            self.op.symbol(),
            self.threshold
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

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

    #[test]
    fn test_proposition_satisfied() {
        assert!(IsPositive.evaluate(&5).is_satisfied());
        assert!(!IsPositive.evaluate(&-3).is_satisfied());
    }

    #[test]
    fn test_allof() {
        let both = AllOf::new(vec![Box::new(IsPositive), Box::new(IsPositive)]);
        assert!(both.evaluate(&5).is_satisfied());
    }

    #[test]
    fn test_anyof() {
        let either = AnyOf::new(vec![Box::new(IsPositive), Box::new(IsEven)]);
        assert!(either.evaluate(&-4).is_satisfied());
        assert!(either.evaluate(&3).is_satisfied());
        assert!(
            !AnyOf::new(vec![Box::new(IsPositive), Box::new(IsEven)])
                .evaluate(&-3)
                .is_satisfied()
        );
    }

    #[test]
    fn test_not() {
        let not_positive = Not::new(IsPositive);
        assert!(not_positive.evaluate(&-5).is_satisfied());
        assert!(!not_positive.evaluate(&5).is_satisfied());
    }

    #[test]
    fn test_implies() {
        let if_pos_then_even = Implies::new(IsPositive, IsEven);
        assert!(if_pos_then_even.evaluate(&4).is_satisfied());
        assert!(!if_pos_then_even.evaluate(&3).is_satisfied());
        assert!(if_pos_then_even.evaluate(&-3).is_satisfied());
    }

    // --- Measurable / Threshold tests ---

    #[derive(Debug)]
    struct ValueOf;

    impl Measurable<i32> for ValueOf {
        type Value = i32;
        fn measure(&self, n: &i32) -> i32 {
            *n
        }
        fn name(&self) -> &str {
            "value"
        }
    }

    #[test]
    fn test_threshold_greater_than() {
        let check = Threshold::greater_than(ValueOf, 10);
        assert!(check.evaluate(&15).is_satisfied());
        assert!(!check.evaluate(&5).is_satisfied());
    }

    #[test]
    fn test_threshold_less_than() {
        let check = Threshold::less_than(ValueOf, 10);
        assert!(check.evaluate(&5).is_satisfied());
        assert!(!check.evaluate(&15).is_satisfied());
    }

    #[test]
    fn test_threshold_at_least() {
        let check = Threshold::at_least(ValueOf, 10);
        assert!(check.evaluate(&10).is_satisfied());
        assert!(check.evaluate(&11).is_satisfied());
        assert!(!check.evaluate(&9).is_satisfied());
    }

    #[test]
    fn test_compare_op() {
        assert!(CompareOp::LessThan.check(&3, &5));
        assert!(!CompareOp::LessThan.check(&5, &3));
        assert!(CompareOp::Equal.check(&5, &5));
        assert!(CompareOp::NotEqual.check(&3, &5));
        assert!(CompareOp::GreaterOrEqual.check(&5, &5));
    }

    proptest! {
        /// NOT NOT x = x
        #[test]
        fn prop_double_negation(n in -100..100i32) {
            let p = IsPositive;
            let nn = Not::new(Not::new(IsPositive));
            prop_assert_eq!(p.evaluate(&n).is_satisfied(), nn.evaluate(&n).is_satisfied());
        }

        /// AllOf with single element = the element
        #[test]
        fn prop_allof_single(n in -100..100i32) {
            let single = AllOf::new(vec![Box::new(IsPositive)]);
            prop_assert_eq!(single.evaluate(&n).is_satisfied(), IsPositive.evaluate(&n).is_satisfied());
        }

        /// AnyOf with single element = the element
        #[test]
        fn prop_anyof_single(n in -100..100i32) {
            let single = AnyOf::new(vec![Box::new(IsPositive)]);
            prop_assert_eq!(single.evaluate(&n).is_satisfied(), IsPositive.evaluate(&n).is_satisfied());
        }

        /// AllOf(A, B) implies AnyOf(A, B)
        #[test]
        fn prop_allof_implies_anyof(n in -100..100i32) {
            let all = AllOf::new(vec![Box::new(IsPositive), Box::new(IsEven)]);
            let any = AnyOf::new(vec![Box::new(IsPositive), Box::new(IsEven)]);
            if all.evaluate(&n).is_satisfied() {
                prop_assert!(any.evaluate(&n).is_satisfied());
            }
        }

        /// Implication: A → B is equivalent to NOT(A) OR B
        #[test]
        fn prop_implication_equivalence(n in -100..100i32) {
            let implies = Implies::new(IsPositive, IsEven);
            let a = IsPositive.evaluate(&n).is_satisfied();
            let b = IsEven.evaluate(&n).is_satisfied();
            let expected = !a || b;
            prop_assert_eq!(implies.evaluate(&n).is_satisfied(), expected);
        }

        /// Threshold: x > t iff x >= t+1 for integers
        #[test]
        fn prop_threshold_gt_vs_gte(value in -100..100i32, threshold in -50..50i32) {
            let gt = Threshold::greater_than(ValueOf, threshold);
            let gte = Threshold::at_least(ValueOf, threshold + 1);
            prop_assert_eq!(gt.evaluate(&value).is_satisfied(), gte.evaluate(&value).is_satisfied());
        }

        /// CompareOp is consistent with PartialOrd
        #[test]
        fn prop_compare_consistent(a in -100..100i32, b in -100..100i32) {
            prop_assert_eq!(CompareOp::LessThan.check(&a, &b), a < b);
            prop_assert_eq!(CompareOp::GreaterThan.check(&a, &b), a > b);
            prop_assert_eq!(CompareOp::Equal.check(&a, &b), a == b);
            prop_assert_eq!(CompareOp::LessOrEqual.check(&a, &b), a <= b);
            prop_assert_eq!(CompareOp::GreaterOrEqual.check(&a, &b), a >= b);
            prop_assert_eq!(CompareOp::NotEqual.check(&a, &b), a != b);
        }

        /// Evaluation reason is never empty
        #[test]
        fn prop_evaluation_has_reason(n in -100..100i32) {
            let result = IsPositive.evaluate(&n);
            prop_assert!(!result.reason().is_empty());
        }
    }
}
