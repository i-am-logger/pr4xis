/// Propositional logic:
/// - Connectives: AND, OR, NOT, IMPLIES, IFF, XOR, NAND, NOR
/// - Laws: De Morgan's, double negation, modus ponens, etc.
/// - Proven via exhaustive truth table evaluation

/// Logical connectives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Connective {
    And,
    Or,
    Not,
    Implies,
    Iff,
    Xor,
    Nand,
    Nor,
}

impl Connective {
    /// All connective variants.
    pub fn variants() -> Vec<Self> {
        vec![
            Connective::And,
            Connective::Or,
            Connective::Not,
            Connective::Implies,
            Connective::Iff,
            Connective::Xor,
            Connective::Nand,
            Connective::Nor,
        ]
    }

    /// Evaluate the connective for given inputs.
    pub fn eval(&self, a: bool, b: bool) -> bool {
        match self {
            Connective::And => a && b,
            Connective::Or => a || b,
            Connective::Not => !a, // b ignored
            Connective::Implies => !a || b,
            Connective::Iff => a == b,
            Connective::Xor => a ^ b,
            Connective::Nand => !(a && b),
            Connective::Nor => !(a || b),
        }
    }

    /// Is this connective commutative? (a OP b = b OP a)
    pub fn is_commutative(&self) -> bool {
        matches!(
            self,
            Connective::And
                | Connective::Or
                | Connective::Iff
                | Connective::Xor
                | Connective::Nand
                | Connective::Nor
        )
    }

    /// Arity: 1 for NOT, 2 for everything else.
    pub fn arity(&self) -> u8 {
        match self {
            Connective::Not => 1,
            _ => 2,
        }
    }
}

// =============================================================================
// Proof functions: verify logical laws by exhaustive truth table
//
// Clippy thinks these boolean expressions are "logic bugs" or can be simplified.
// That's the point — we're PROVING these tautologies hold by evaluation, not simplifying them.
// =============================================================================

#[allow(clippy::nonminimal_bool, clippy::overly_complex_bool_expr)]
/// Verify a tautology: f(a, b) is true for all (a, b).
pub fn is_tautology(f: impl Fn(bool, bool) -> bool) -> bool {
    for &a in &[true, false] {
        for &b in &[true, false] {
            if !f(a, b) {
                return false;
            }
        }
    }
    true
}

#[allow(clippy::nonminimal_bool, clippy::overly_complex_bool_expr)]
/// De Morgan's law: !(A && B) == (!A || !B)
pub fn de_morgan_and() -> bool {
    is_tautology(|a, b| !(a && b) == (!a || !b))
}

#[allow(clippy::nonminimal_bool, clippy::overly_complex_bool_expr)]
/// De Morgan's law: !(A || B) == (!A && !B)
pub fn de_morgan_or() -> bool {
    is_tautology(|a, b| !(a || b) == (!a && !b))
}

#[allow(clippy::nonminimal_bool, clippy::overly_complex_bool_expr)]
/// Double negation: !!A == A
pub fn double_negation() -> bool {
    [true, false].iter().all(|&a| !!a == a)
}

#[allow(clippy::nonminimal_bool, clippy::overly_complex_bool_expr)]
/// Modus ponens: (A && (A → B)) → B
pub fn modus_ponens() -> bool {
    is_tautology(|a, b| {
        let premise = a && (!a || b); // A && (A → B)
        !premise || b // premise → B
    })
}

#[allow(clippy::nonminimal_bool, clippy::overly_complex_bool_expr)]
/// Contrapositive: (A → B) == (!B → !A)
pub fn contrapositive() -> bool {
    is_tautology(|a, b| {
        let forward = !a || b; // A → B
        let contra = b || !a; // !B → !A = B || !A
        forward == contra
    })
}

#[allow(clippy::nonminimal_bool, clippy::overly_complex_bool_expr)]
/// Excluded middle: A || !A
pub fn excluded_middle() -> bool {
    [true, false].iter().all(|&a| a || !a)
}

#[allow(clippy::nonminimal_bool, clippy::overly_complex_bool_expr)]
/// Non-contradiction: !(A && !A)
pub fn non_contradiction() -> bool {
    [true, false].iter().all(|&a| !(a && !a))
}

#[allow(clippy::nonminimal_bool, clippy::overly_complex_bool_expr)]
/// NAND is functionally complete: can express AND, OR, NOT.
pub fn nand_is_universal() -> bool {
    let nand = |a: bool, b: bool| !(a && b);
    // NOT A = A NAND A
    let not_via_nand = |a: bool| nand(a, a);
    // AND = NOT(A NAND B) = (A NAND B) NAND (A NAND B)
    let and_via_nand = |a: bool, b: bool| nand(nand(a, b), nand(a, b));
    // OR = (A NAND A) NAND (B NAND B) = NOT A NAND NOT B
    let or_via_nand = |a: bool, b: bool| nand(nand(a, a), nand(b, b));

    is_tautology(|a, _b| not_via_nand(a) == !a)
        && is_tautology(|a, b| and_via_nand(a, b) == (a && b))
        && is_tautology(|a, b| or_via_nand(a, b) == (a || b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_8_connectives() {
        assert_eq!(Connective::variants().len(), 8);
    }

    #[test]
    fn test_commutative_connectives() {
        let count = Connective::variants()
            .iter()
            .filter(|c| c.is_commutative())
            .count();
        assert_eq!(count, 6); // AND, OR, IFF, XOR, NAND, NOR
    }

    #[test]
    fn test_de_morgan_and() {
        assert!(de_morgan_and());
    }

    #[test]
    fn test_de_morgan_or() {
        assert!(de_morgan_or());
    }

    #[test]
    fn test_double_negation() {
        assert!(double_negation());
    }

    #[test]
    fn test_modus_ponens() {
        assert!(modus_ponens());
    }

    #[test]
    fn test_contrapositive() {
        assert!(contrapositive());
    }

    #[test]
    fn test_excluded_middle() {
        assert!(excluded_middle());
    }

    #[test]
    fn test_non_contradiction() {
        assert!(non_contradiction());
    }

    #[test]
    fn test_nand_universal() {
        assert!(nand_is_universal());
    }

    proptest! {
        /// Commutativity: verified by evaluation for commutative connectives
        #[test]
        fn prop_commutative(a in proptest::bool::ANY, b in proptest::bool::ANY) {
            for c in Connective::variants() {
                if c.is_commutative() {
                    prop_assert_eq!(c.eval(a, b), c.eval(b, a), "{:?} not commutative", c);
                }
            }
        }

        /// Implies is NOT commutative (except when a==b)
        #[test]
        fn prop_implies_not_commutative(a in proptest::bool::ANY, b in proptest::bool::ANY) {
            if a != b {
                prop_assert_ne!(Connective::Implies.eval(a, b), Connective::Implies.eval(b, a));
            }
        }

        /// AND identity: A && true == A
        #[test]
        fn prop_and_identity(a in proptest::bool::ANY) {
            prop_assert_eq!(Connective::And.eval(a, true), a);
        }

        /// OR identity: A || false == A
        #[test]
        fn prop_or_identity(a in proptest::bool::ANY) {
            prop_assert_eq!(Connective::Or.eval(a, false), a);
        }

        /// XOR is self-inverse: A XOR A == false
        #[test]
        fn prop_xor_self_false(a in proptest::bool::ANY) {
            prop_assert_eq!(Connective::Xor.eval(a, a), false);
        }

        /// IFF is reflexive: A IFF A == true
        #[test]
        fn prop_iff_reflexive(a in proptest::bool::ANY) {
            prop_assert_eq!(Connective::Iff.eval(a, a), true);
        }

        /// NAND == NOT AND
        #[test]
        fn prop_nand_is_not_and(a in proptest::bool::ANY, b in proptest::bool::ANY) {
            prop_assert_eq!(Connective::Nand.eval(a, b), !Connective::And.eval(a, b));
        }

        /// NOR == NOT OR
        #[test]
        fn prop_nor_is_not_or(a in proptest::bool::ANY, b in proptest::bool::ANY) {
            prop_assert_eq!(Connective::Nor.eval(a, b), !Connective::Or.eval(a, b));
        }
    }
}
