/// Truth tables as an ontology-driven proof mechanism.
use super::propositional::Connective;
#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// A truth table row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub inputs: Vec<bool>,
    pub output: bool,
}

/// Generate full truth table for a binary connective.
pub fn binary_truth_table(connective: Connective) -> Vec<Row> {
    vec![
        Row {
            inputs: vec![true, true],
            output: connective.eval(true, true),
        },
        Row {
            inputs: vec![true, false],
            output: connective.eval(true, false),
        },
        Row {
            inputs: vec![false, true],
            output: connective.eval(false, true),
        },
        Row {
            inputs: vec![false, false],
            output: connective.eval(false, false),
        },
    ]
}

/// Check if two connectives are logically equivalent (same truth table).
pub fn equivalent(a: Connective, b: Connective) -> bool {
    binary_truth_table(a) == binary_truth_table(b)
}

/// Check if a connective is a tautology (all outputs true).
pub fn is_tautology(connective: Connective) -> bool {
    binary_truth_table(connective).iter().all(|r| r.output)
}

/// Check if a connective is a contradiction (all outputs false).
pub fn is_contradiction(connective: Connective) -> bool {
    binary_truth_table(connective).iter().all(|r| !r.output)
}

/// Check if a connective is satisfiable (at least one true output).
pub fn is_satisfiable(connective: Connective) -> bool {
    binary_truth_table(connective).iter().any(|r| r.output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_and_truth_table() {
        let table = binary_truth_table(Connective::And);
        assert!(table[0].output); // T && T
        assert!(!table[1].output); // T && F
        assert!(!table[2].output); // F && T
        assert!(!table[3].output); // F && F
    }

    #[test]
    fn test_or_truth_table() {
        let table = binary_truth_table(Connective::Or);
        assert!(table[0].output);
        assert!(table[1].output);
        assert!(table[2].output);
        assert!(!table[3].output);
    }

    #[test]
    fn test_implies_truth_table() {
        let table = binary_truth_table(Connective::Implies);
        assert!(table[0].output); // T -> T
        assert!(!table[1].output); // T -> F
        assert!(table[2].output); // F -> T
        assert!(table[3].output); // F -> F
    }

    #[test]
    fn test_xor_iff_are_complements() {
        let xor = binary_truth_table(Connective::Xor);
        let iff = binary_truth_table(Connective::Iff);
        for (x, i) in xor.iter().zip(iff.iter()) {
            assert_ne!(x.output, i.output, "XOR and IFF should be complements");
        }
    }

    #[test]
    fn test_no_connective_is_tautology() {
        for c in Connective::variants() {
            if c == Connective::Not {
                continue;
            } // unary
            assert!(!is_tautology(c), "{:?} should not be a tautology", c);
        }
    }

    #[test]
    fn test_all_connectives_satisfiable() {
        for c in Connective::variants() {
            if c == Connective::Not {
                continue;
            }
            assert!(is_satisfiable(c), "{:?} should be satisfiable", c);
        }
    }

    #[test]
    fn test_no_connective_is_contradiction() {
        for c in Connective::variants() {
            if c == Connective::Not {
                continue;
            }
            assert!(
                !is_contradiction(c),
                "{:?} should not be a contradiction",
                c
            );
        }
    }

    #[test]
    fn test_nand_nor_not_equivalent() {
        assert!(!equivalent(Connective::Nand, Connective::Nor));
    }
}
