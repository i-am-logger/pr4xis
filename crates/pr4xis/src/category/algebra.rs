use std::fmt::Debug;

// F-algebra and F-coalgebra �� recursive structure patterns.
//
// An F-algebra is a pair (A, α: F(A) → A) — a way to "fold" a functor
// into a carrier type. Dually, an F-coalgebra is (A, α: A → F(A)) —
// a way to "unfold" a carrier into a functor.
//
// These are THE fundamental patterns for recursive data:
//   Catamorphism (fold):  collapse a recursive structure bottom-up
//   Anamorphism (unfold): build a recursive structure top-down
//   Hylomorphism:         unfold then fold (build then collapse)
//
// In pr4xis:
//   - Cofree.fold() IS a catamorphism
//   - Taxonomy traversal (ancestors) IS an anamorphism (unfold parent chain)
//   - Pipeline processing IS a hylomorphism (parse→fold into meaning)
//
// References:
// - Meijer, Fokkinga & Paterson, "Functional Programming with Bananas,
//   Lenses, Envelopes and Barbed Wire" (1991, FPCA)
//   https://doi.org/10.1007/3540543961_7
// - Bird & de Moor, "Algebra of Programming" (1997, Prentice Hall)
// - Hinze, Wu & Gibbons, "Unifying Structured Recursion Schemes" (2013, ICFP)
//   https://doi.org/10.1145/2500365.2500578

/// An F-algebra: a carrier type A with a folding function F(A) → A.
///
/// For a tree-like structure, the algebra says how to combine children
/// into a parent value. This is the generalization of `fold`.
///
/// ```
/// use pr4xis::category::algebra::Algebra;
///
/// // Sum algebra: fold a tree of i32 by summing
/// let sum_alg = Algebra::new(|node: &i32, children: &[i32]| {
///     node + children.iter().sum::<i32>()
/// });
///
/// assert_eq!(sum_alg.apply(&10, &[1, 2, 3]), 16);
/// ```
#[allow(clippy::type_complexity)]
pub struct Algebra<A, F> {
    f: Box<dyn Fn(&A, &[F]) -> F>,
}

impl<A: 'static, F: 'static> Algebra<A, F> {
    pub fn new(f: impl Fn(&A, &[F]) -> F + 'static) -> Self {
        Self { f: Box::new(f) }
    }

    /// Apply the algebra: combine a node with its folded children.
    pub fn apply(&self, node: &A, children: &[F]) -> F {
        (self.f)(node, children)
    }
}

/// An F-coalgebra: a seed type S with an unfolding function S → (A, [S]).
///
/// Produces a node value and zero or more seeds for children.
/// This is the generalization of `unfold`.
///
/// ```
/// use pr4xis::category::algebra::Coalgebra;
///
/// // Count-down coalgebra: unfold from n to 0
/// let countdown = Coalgebra::new(|n: &i32| {
///     if *n <= 0 {
///         (*n, vec![])
///     } else {
///         (*n, vec![n - 1])
///     }
/// });
///
/// let (val, children) = countdown.apply(&3);
/// assert_eq!(val, 3);
/// assert_eq!(children, vec![2]);
/// ```
#[allow(clippy::type_complexity)]
pub struct Coalgebra<S, A> {
    f: Box<dyn Fn(&S) -> (A, Vec<S>)>,
}

impl<S: 'static, A: 'static> Coalgebra<S, A> {
    pub fn new(f: impl Fn(&S) -> (A, Vec<S>) + 'static) -> Self {
        Self { f: Box::new(f) }
    }

    /// Apply the coalgebra: produce a value and child seeds.
    pub fn apply(&self, seed: &S) -> (A, Vec<S>) {
        (self.f)(seed)
    }
}

/// Catamorphism: fold a recursive structure using an algebra.
///
/// cata(alg, tree) = alg.apply(tree.head, [cata(alg, child) for child in tree.children])
pub fn cata<A: Clone + Debug + 'static, F: 'static>(
    alg: &Algebra<A, F>,
    tree: &super::comonad::Cofree<A>,
) -> F {
    let children: Vec<F> = tree.tail.iter().map(|c| cata(alg, c)).collect();
    alg.apply(&tree.head, &children)
}

/// Anamorphism: unfold a recursive structure from a seed using a coalgebra.
///
/// ana(coalg, seed) = Cofree(value, [ana(coalg, child) for child in children])
pub fn ana<S: Clone + 'static, A: Clone + Debug + 'static>(
    coalg: &Coalgebra<S, A>,
    seed: &S,
) -> super::comonad::Cofree<A> {
    let (value, children) = coalg.apply(seed);
    super::comonad::Cofree {
        head: value,
        tail: children.iter().map(|s| ana(coalg, s)).collect(),
    }
}

/// Hylomorphism: unfold then fold. Build a structure and immediately collapse it.
///
/// hylo(alg, coalg, seed) = cata(alg, ana(coalg, seed))
/// But computed without building the intermediate structure.
pub fn hylo<S: Clone + 'static, A: Clone + Debug + 'static, F: 'static>(
    alg: &Algebra<A, F>,
    coalg: &Coalgebra<S, A>,
    seed: &S,
) -> F {
    let (value, children) = coalg.apply(seed);
    let child_results: Vec<F> = children.iter().map(|s| hylo(alg, coalg, s)).collect();
    alg.apply(&value, &child_results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::comonad::Cofree;

    #[test]
    fn algebra_sum_tree() {
        let sum = Algebra::new(|node: &i32, children: &[i32]| node + children.iter().sum::<i32>());

        let tree = Cofree::node(1, vec![Cofree::leaf(2), Cofree::leaf(3)]);
        assert_eq!(cata(&sum, &tree), 6);
    }

    #[test]
    fn algebra_count_nodes() {
        let count =
            Algebra::new(|_node: &&str, children: &[usize]| 1 + children.iter().sum::<usize>());

        let tree = Cofree::node(
            "root",
            vec![
                Cofree::node("left", vec![Cofree::leaf("ll")]),
                Cofree::leaf("right"),
            ],
        );
        assert_eq!(cata(&count, &tree), 4);
    }

    #[test]
    fn coalgebra_countdown() {
        let countdown = Coalgebra::new(|n: &i32| {
            if *n <= 0 {
                (*n, vec![])
            } else {
                (*n, vec![n - 1])
            }
        });

        let tree = ana(&countdown, &3);
        assert_eq!(*tree.extract(), 3);
        assert_eq!(tree.tail.len(), 1);
        assert_eq!(*tree.tail[0].extract(), 2);
        assert_eq!(*tree.tail[0].tail[0].extract(), 1);
        assert_eq!(*tree.tail[0].tail[0].tail[0].extract(), 0);
    }

    #[test]
    fn coalgebra_binary_tree() {
        // Unfold a complete binary tree of depth 2
        let binary = Coalgebra::new(|depth: &i32| {
            if *depth <= 0 {
                (*depth, vec![])
            } else {
                (*depth, vec![depth - 1, depth - 1])
            }
        });

        let tree = ana(&binary, &2);
        assert_eq!(*tree.extract(), 2);
        assert_eq!(tree.tail.len(), 2); // two children
        assert_eq!(tree.tail[0].tail.len(), 2); // each has two children
        assert_eq!(tree.tail[0].tail[0].tail.len(), 0); // leaves
    }

    #[test]
    fn hylomorphism_factorial() {
        // Unfold: n → (n, [n-1]) until 0
        let coalg = Coalgebra::new(|n: &i32| {
            if *n <= 1 {
                (*n, vec![])
            } else {
                (*n, vec![n - 1])
            }
        });

        // Fold: multiply node with child results
        let alg = Algebra::new(|n: &i32, children: &[i32]| {
            if children.is_empty() {
                1
            } else {
                n * children[0]
            }
        });

        assert_eq!(hylo(&alg, &coalg, &5), 120); // 5!
        assert_eq!(hylo(&alg, &coalg, &1), 1);
    }

    #[test]
    fn hylomorphism_fibonacci() {
        // Unfold: n → (n, [n-1, n-2]) for n > 1
        let coalg = Coalgebra::new(|n: &i32| {
            if *n <= 1 {
                (*n, vec![])
            } else {
                (*n, vec![n - 1, n - 2])
            }
        });

        // Fold: sum children (base cases: fib(0)=0, fib(1)=1)
        let alg = Algebra::new(|n: &i32, children: &[i32]| {
            if children.is_empty() {
                *n // base: fib(0)=0, fib(1)=1
            } else {
                children.iter().sum()
            }
        });

        assert_eq!(hylo(&alg, &coalg, &0), 0);
        assert_eq!(hylo(&alg, &coalg, &1), 1);
        assert_eq!(hylo(&alg, &coalg, &6), 8); // fib(6) = 8
        assert_eq!(hylo(&alg, &coalg, &10), 55); // fib(10) = 55
    }

    // --- Practical: taxonomy traversal as anamorphism ---

    #[test]
    fn taxonomy_as_anamorphism() {
        // Unfold: given an entity, produce it + its children in taxonomy
        let taxonomy: Vec<(&str, &str)> =
            vec![("Animal", "Mammal"), ("Mammal", "Dog"), ("Mammal", "Cat")];

        let unfold = Coalgebra::new(move |entity: &&str| {
            let children: Vec<&str> = taxonomy
                .iter()
                .filter(|(parent, _)| parent == entity)
                .map(|(_, child)| *child)
                .collect();
            (*entity, children)
        });

        let tree = ana(&unfold, &"Animal");
        assert_eq!(*tree.extract(), "Animal");
        assert_eq!(tree.tail.len(), 1); // Mammal
        assert_eq!(*tree.tail[0].extract(), "Mammal");
        assert_eq!(tree.tail[0].tail.len(), 2); // Dog, Cat
    }
}
