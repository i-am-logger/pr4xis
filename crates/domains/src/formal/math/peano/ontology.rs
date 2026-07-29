//! Peano Arithmetic — the recursive definitions of addition and
//! multiplication over the naturals, made first-class axioms rather than
//! trusted implicitly by `formal::calculator::op::BinaryOp`.
//!
//! `formal::math::ontology` (the sibling `Number` ontology) already
//! establishes N as the base of the N \u{2282} Z \u{2282} Q \u{2282} R \u{2282} C inclusion
//! chain but only as a taxonomic node -- it carries no successor function
//! and no arithmetic laws. This ontology supplies exactly that: a 0-based
//! `Zero`/`Successor` system and the two operations defined by primitive
//! recursion over the successor function.
//!
//! CITATION CORRECTION (2026-07-12, after adversarial audit): this file
//! originally cited "Peano (1889) axiom 1: 0 is a natural number" and
//! "Landau (1930) \u{00a7}1 Satz 1/4" for the 0-based content below. Both are
//! wrong -- verified against the primary sources (Landau's own 1930 text,
//! machine-extracted): Landau's actual Axiom 1 is "1 ist eine natürliche
//! Zahl" (1, not 0, is a natural number), his addition definition is
//! "\u{00a7}2 Satz 4" (not \u{00a7}1 Satz 1) with base case x+1=x' (not x+0=x), and
//! his multiplication definition is "\u{00a7}4 Satz 28" (not \u{00a7}1 Satz 4) with
//! base case x\u{00b7}1=x. Landau numbers Satz sequentially through the whole
//! 301-theorem book, so "per-\u{00a7} Satz numbering" was never even how the
//! source is organized. Peano's original 1889 *Arithmetices Principia* is
//! ALSO 1-based (corroborated by multiple secondary sources tracing to
//! Peano 1889 p.1; not independently primary-verified here). Neither
//! source can honestly ground a 0-based system -- they structurally have
//! no zero. The correct citation for exactly this 0-based Zero, Successor,
//! and recursive addition/multiplication shape is Herbert B. Enderton,
//! *Elements of Set Theory* (Academic Press, 1977), ch. 4 "Natural
//! Numbers": Theorem 4D (p.71, the Peano system \u{27e8}\u{03c9}, \u{03c3}, 0\u{27e9}), Theorem 4I
//! (p.79, addition: m+0=m, m+n⁺=(m+n)⁺), Theorem 4J (p.80, multiplication:
//! m·0=0, m·n⁺=m·n+m) -- machine-verified against the primary text.
//! Peano (1889)/Landau (1930) are kept below ONLY as historical framing
//! for the naming "Peano Arithmetic" (the modern term for this theory,
//! standardly presented 0-based in logic/set-theory texts even though
//! Peano's own original formulation was 1-based) -- never as the source of
//! the specific 0-based formulas, which are Enderton's.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "PeanoArithmetic",
    source: "Enderton (1977) Elements of Set Theory, ch. 4 \"Natural Numbers\" (the 0-based axiomatization); Peano (1889)/Landau (1930) as historical framing only (both are 1-based, not the source of these formulas)",

    concepts: [
        Zero,
        Successor,
        Addition,
        Multiplication,
    ],

    labels: {
        Zero: ("en", "Zero",
            "Enderton (1977) Theorem 4D, p.71: the Peano system \u{27e8}\u{03c9}, \u{03c3}, 0\u{27e9} -- 0 is the distinguished base element every recursive definition over N grounds out in."),
        Successor: ("en", "Successor",
            "Enderton (1977) Theorem 4D, p.71: \u{03c3} (here S), the successor function N -> N -- 'the next natural number', written n⁺."),
        Addition: ("en", "Addition",
            "Enderton (1977) Theorem 4I, p.79, defined by primitive recursion over Successor: a+0=a (base case), a+S(b)=S(a+b) (recursive case)."),
        Multiplication: ("en", "Multiplication",
            "Enderton (1977) Theorem 4J, p.80, defined by primitive recursion over Successor: a\u{00d7}0=0 (base case), a\u{00d7}S(b)=(a\u{00d7}b)+a (recursive case)."),
    },

    edges: [
        (Zero, Addition, IsIdentityFor),
        (Successor, Addition, RecursesThrough),
        (Zero, Multiplication, Annihilates),
        (Successor, Multiplication, RecursesThrough),
    ],
}

fn kinded_edge_exists(
    from: PeanoArithmeticConcept,
    to: PeanoArithmeticConcept,
    kind: PeanoArithmeticRelationKind,
) -> bool {
    use pr4xis::category::{Arrow, Category};
    PeanoArithmeticCategory::morphisms()
        .iter()
        .any(|m| m.source() == from && m.target() == to && m.kind() == kind)
}

/// Which of the two recursive definitions each edge witnesses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecursionRole {
    /// The base case (a op Zero = ...).
    BaseCase,
    /// The recursive case (a op Successor(b) = ... in terms of a op b).
    RecursiveCase,
}

/// Quality: is this edge the base case or the recursive case of its operation's
/// primitive-recursive definition?
#[derive(Debug, Clone)]
pub struct RecursionStep;

impl Quality for RecursionStep {
    type Individual = PeanoArithmeticConcept;
    type Value = RecursionRole;

    fn get(&self, c: &PeanoArithmeticConcept) -> Option<RecursionRole> {
        use PeanoArithmeticConcept as P;
        match c {
            P::Zero => Some(RecursionRole::BaseCase),
            P::Successor => Some(RecursionRole::RecursiveCase),
            P::Addition | P::Multiplication => None,
        }
    }
}

/// Structural: Zero is the additive identity (a+0=a) -- the edge exists.
pub struct ZeroIsIdentityForAddition;

impl Axiom for ZeroIsIdentityForAddition {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if kinded_edge_exists(
            PeanoArithmeticConcept::Zero,
            PeanoArithmeticConcept::Addition,
            PeanoArithmeticRelationKind::IsIdentityFor,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ZeroIsIdentityForAddition",
        "(Zero, Addition, IsIdentityFor): a+0=a, the base case of the recursive definition of addition",
        "Enderton (1977) Elements of Set Theory, Theorem 4I, p.79"
    );
}

pr4xis::register_axiom!(
    ZeroIsIdentityForAddition,
    "Enderton (1977) Elements of Set Theory, Theorem 4I"
);

/// Structural: Zero annihilates multiplication (a\u{00d7}0=0) -- the edge exists.
pub struct ZeroAnnihilatesMultiplication;

impl Axiom for ZeroAnnihilatesMultiplication {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if kinded_edge_exists(
            PeanoArithmeticConcept::Zero,
            PeanoArithmeticConcept::Multiplication,
            PeanoArithmeticRelationKind::Annihilates,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ZeroAnnihilatesMultiplication",
        "(Zero, Multiplication, Annihilates): a\u{00d7}0=0, the base case of the recursive definition of multiplication",
        "Enderton (1977) Elements of Set Theory, Theorem 4J, p.80"
    );
}

pr4xis::register_axiom!(
    ZeroAnnihilatesMultiplication,
    "Enderton (1977) Elements of Set Theory, Theorem 4J"
);

/// COMPUTATIONAL: the calculator's actual `BinaryOp::Add.apply` satisfies
/// Peano's recursive definition of addition -- a+0=a and a+S(b)=S(a+b),
/// checked against the REAL evaluator, not assumed to match it.
pub struct CalculatorAdditionSatisfiesPeanoRecursion;

impl Axiom for CalculatorAdditionSatisfiesPeanoRecursion {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use crate::formal::calculator::op::BinaryOp;
        use crate::formal::calculator::value::Value;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};

        for a in 0..10i64 {
            // Base case: a + 0 = a.
            let base = BinaryOp::Add.apply(&Value::int(a), &Value::int(0));
            if base != Ok(Value::int(a)) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            for b in 0..10i64 {
                // Recursive case: a + S(b) = S(a + b), i.e. a+(b+1) == (a+b)+1.
                let lhs = BinaryOp::Add.apply(&Value::int(a), &Value::int(b + 1));
                let inner = BinaryOp::Add.apply(&Value::int(a), &Value::int(b));
                let Ok(inner_val) = inner else {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                };
                let rhs = BinaryOp::Add.apply(&inner_val, &Value::int(1));
                if lhs != rhs {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "CalculatorAdditionSatisfiesPeanoRecursion",
        "BinaryOp::Add.apply satisfies the recursive definition of addition (a+0=a; a+S(b)=S(a+b)) for the WAIS-IV single-digit operand band (0-9)",
        "Enderton (1977) Elements of Set Theory, Theorem 4I, p.79"
    );
}

pr4xis::register_axiom!(
    CalculatorAdditionSatisfiesPeanoRecursion,
    "Enderton (1977) Elements of Set Theory, Theorem 4I"
);

/// COMPUTATIONAL: the calculator's actual `BinaryOp::Multiply.apply`
/// satisfies Peano's recursive definition of multiplication -- a\u{00d7}0=0 and
/// a\u{00d7}S(b)=(a\u{00d7}b)+a.
pub struct CalculatorMultiplicationSatisfiesPeanoRecursion;

impl Axiom for CalculatorMultiplicationSatisfiesPeanoRecursion {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use crate::formal::calculator::op::BinaryOp;
        use crate::formal::calculator::value::Value;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};

        for a in 0..10i64 {
            // Base case: a * 0 = 0.
            let base = BinaryOp::Multiply.apply(&Value::int(a), &Value::int(0));
            if base != Ok(Value::int(0)) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            for b in 0..10i64 {
                // Recursive case: a * S(b) = (a * b) + a.
                let lhs = BinaryOp::Multiply.apply(&Value::int(a), &Value::int(b + 1));
                let Ok(ab) = BinaryOp::Multiply.apply(&Value::int(a), &Value::int(b)) else {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                };
                let rhs = BinaryOp::Add.apply(&ab, &Value::int(a));
                if lhs != rhs {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "CalculatorMultiplicationSatisfiesPeanoRecursion",
        "BinaryOp::Multiply.apply satisfies the recursive definition of multiplication (a\u{00d7}0=0; a\u{00d7}S(b)=(a\u{00d7}b)+a) for the WAIS-IV single-digit operand band (0-9)",
        "Enderton (1977) Elements of Set Theory, Theorem 4J, p.80"
    );
}

pr4xis::register_axiom!(
    CalculatorMultiplicationSatisfiesPeanoRecursion,
    "Enderton (1977) Elements of Set Theory, Theorem 4J"
);

impl Ontology for PeanoArithmeticOntology {
    type Cat = PeanoArithmeticCategory;
    type Qual = RecursionStep;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(ZeroIsIdentityForAddition));
        axioms.push(Box::new(ZeroAnnihilatesMultiplication));
        axioms.push(Box::new(CalculatorAdditionSatisfiesPeanoRecursion));
        axioms.push(Box::new(CalculatorMultiplicationSatisfiesPeanoRecursion));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<PeanoArithmeticCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        PeanoArithmeticOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn zero_is_identity_for_addition_holds() {
        assert!(ZeroIsIdentityForAddition.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn zero_annihilates_multiplication_holds() {
        assert!(ZeroAnnihilatesMultiplication.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn calculator_addition_satisfies_peano_recursion_holds() {
        assert!(CalculatorAdditionSatisfiesPeanoRecursion.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn calculator_multiplication_satisfies_peano_recursion_holds() {
        assert!(
            CalculatorMultiplicationSatisfiesPeanoRecursion
                .verify()
                .is_ok()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn zero_is_base_case_successor_is_recursive_case() {
        assert_eq!(
            RecursionStep.get(&PeanoArithmeticConcept::Zero),
            Some(RecursionRole::BaseCase)
        );
        assert_eq!(
            RecursionStep.get(&PeanoArithmeticConcept::Successor),
            Some(RecursionRole::RecursiveCase)
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn addition_and_multiplication_are_not_recursion_steps() {
        // Addition/Multiplication are the OPERATIONS being defined by
        // recursion, not a base/recursive CASE of that recursion -- honestly
        // None, not defaulted onto one of the two roles.
        assert_eq!(RecursionStep.get(&PeanoArithmeticConcept::Addition), None);
        assert_eq!(
            RecursionStep.get(&PeanoArithmeticConcept::Multiplication),
            None
        );
    }
}
