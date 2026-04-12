use std::fmt::Debug;

// Applicative functor — independent computations that can be combined.
//
// Sits between Functor and Monad in the hierarchy:
//   Functor: fmap f over a context
//   Applicative: apply a function IN a context to a value IN a context
//   Monad: sequence dependent computations
//
// Applicative allows parallel/independent combination. Monad requires
// sequential dependency. Many computations that seem monadic are
// actually applicative (e.g., querying multiple ontologies independently).
//
// The applicative laws:
//   1. Identity:     ap(pure(id), v) = v
//   2. Composition:  ap(ap(ap(pure(compose), u), v), w) = ap(u, ap(v, w))
//   3. Homomorphism: ap(pure(f), pure(x)) = pure(f(x))
//   4. Interchange:  ap(u, pure(y)) = ap(pure(|f| f(y)), u)
//
// References:
// - McBride & Paterson, "Applicative Programming with Effects" (2008, JFP)
//   https://doi.org/10.1017/S0956796807006326
// - Marlow et al., "There is No Fork" (2014, Haskell Symposium)
//   — applicative vs monadic concurrency
// - Capriotti & Kaposi, "Free Applicative Functors" (2014, MSFP)

/// An applicative value: a value in a computational context
/// that supports independent combination.
///
/// `Ap<F, A>` wraps a value `A` in context `F` where independent
/// computations can be combined without sequencing.
///
/// ```
/// use pr4xis::category::applicative::Ap;
///
/// // Combine independent results
/// let x = Ap::pure(3);
/// let y = Ap::pure(4);
/// let sum = x.map2(y, |a, b| a + b);
/// assert_eq!(sum.value, 7);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Ap<A> {
    pub value: A,
}

impl<A: Clone + Debug> Ap<A> {
    /// Lift a pure value into the applicative context.
    pub fn pure(a: A) -> Self {
        Self { value: a }
    }

    /// Functor map: apply a function to the value.
    pub fn map<B: Clone + Debug>(self, f: impl FnOnce(A) -> B) -> Ap<B> {
        Ap {
            value: f(self.value),
        }
    }

    /// Applicative combine: combine two independent values with a function.
    /// Unlike monadic bind, neither computation depends on the other's result.
    pub fn map2<B: Clone + Debug, C: Clone + Debug>(
        self,
        other: Ap<B>,
        f: impl FnOnce(A, B) -> C,
    ) -> Ap<C> {
        Ap {
            value: f(self.value, other.value),
        }
    }

    /// Apply a function in context to a value in context.
    /// In Rust, map2 is preferred over ap (avoids Box<dyn FnOnce> issues).
    pub fn ap<B: Clone + Debug>(self, f: impl FnOnce(A) -> B) -> Ap<B> {
        Ap {
            value: f(self.value),
        }
    }
}

/// Combine a vec of independent applicative computations.
pub fn sequence<A: Clone + Debug>(items: Vec<Ap<A>>) -> Ap<Vec<A>> {
    Ap {
        value: items.into_iter().map(|a| a.value).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_wraps_value() {
        let a = Ap::pure(42);
        assert_eq!(a.value, 42);
    }

    #[test]
    fn map_applies_function() {
        let result = Ap::pure(21).map(|x| x * 2);
        assert_eq!(result.value, 42);
    }

    #[test]
    fn map2_combines_independent() {
        let x = Ap::pure(3);
        let y = Ap::pure(4);
        let result = x.map2(y, |a, b| a + b);
        assert_eq!(result.value, 7);
    }

    #[test]
    fn sequence_collects_all() {
        let items = vec![Ap::pure(1), Ap::pure(2), Ap::pure(3)];
        let result = sequence(items);
        assert_eq!(result.value, vec![1, 2, 3]);
    }

    // --- Applicative laws ---

    #[test]
    fn identity_law() {
        // ap(id, v) = v
        let result = Ap::pure(42).ap(|x| x);
        assert_eq!(result.value, 42);
    }

    #[test]
    fn homomorphism_law() {
        // ap(f, pure(x)) = pure(f(x))
        let f = |x: i32| x * 2;
        let x = 21;
        let left = Ap::pure(x).ap(f);
        let right = Ap::pure(f(x));
        assert_eq!(left.value, right.value);
    }

    // --- Practical: parallel ontology queries ---

    #[test]
    fn ontology_query_combination() {
        // Simulate querying taxonomy and mereology independently
        let taxonomy_result = Ap::pure(vec!["Dog", "Mammal", "Animal"]);
        let mereology_result = Ap::pure(vec!["Tail", "Fur"]);

        let combined = taxonomy_result.map2(mereology_result, |ancestors, parts| {
            (ancestors.len(), parts.len())
        });

        assert_eq!(combined.value, (3, 2));
    }

    mod prop {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            /// pure then map = pure of applied function
            #[test]
            fn prop_map_pure(x in any::<i32>()) {
                let result = Ap::pure(x).map(|a| a + 1);
                prop_assert_eq!(result.value, x + 1);
            }

            /// map2 is commutative for commutative operations
            #[test]
            fn prop_map2_commutative(a in any::<i32>(), b in any::<i32>()) {
                let r1 = Ap::pure(a).map2(Ap::pure(b), |x, y| x.wrapping_add(y));
                let r2 = Ap::pure(b).map2(Ap::pure(a), |x, y| x.wrapping_add(y));
                prop_assert_eq!(r1.value, r2.value);
            }

            /// sequence preserves order and length
            #[test]
            fn prop_sequence_length(n in 0..20usize) {
                let items: Vec<Ap<usize>> = (0..n).map(Ap::pure).collect();
                let result = sequence(items);
                prop_assert_eq!(result.value.len(), n);
            }
        }
    }
}
