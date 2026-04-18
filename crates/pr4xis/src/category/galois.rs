#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};
use core::fmt::Debug;

// Galois connection — an adjunction between partially ordered sets.
//
// A Galois connection between posets (A, ≤) and (B, ≤) is a pair of
// monotone functions (f, g) where:
//   f: A → B (lower adjoint / left adjoint)
//   g: B → A (upper adjoint / right adjoint)
//   f(a) ≤ b ⟺ a ≤ g(b)
//
// Equivalently:
//   a ≤ g(f(a))     (unit / inflation)
//   f(g(b)) ≤ b     (counit / deflation)
//
// In pr4xis, Galois connections formalize:
//   - Taxonomy: ancestors(x) ⊣ descendants(x)
//   - Abstraction/Concretization: abstract ⊣ concretize (Cousot & Cousot 1977)
//   - Functor adjunctions restricted to posets
//
// References:
// - Ore, "Galois Connexions" (1944, Trans. AMS)
//   https://doi.org/10.2307/1990165
// - Davey & Priestley, "Introduction to Lattices and Order" (2002, Cambridge)
// - Cousot & Cousot, "Abstract Interpretation" (1977, POPL)
//   https://doi.org/10.1145/512950.512973

/// A Galois connection between two types with a partial order.
///
/// The pair (lower, upper) where:
///   lower(a) ≤ b ⟺ a ≤ upper(b)
pub struct GaloisConnection<A, B> {
    /// Lower adjoint: A → B (more abstract direction).
    lower: Box<dyn Fn(&A) -> B>,
    /// Upper adjoint: B → A (more concrete direction).
    upper: Box<dyn Fn(&B) -> A>,
}

impl<A: 'static + Clone + Debug + PartialOrd, B: 'static + Clone + Debug + PartialOrd>
    GaloisConnection<A, B>
{
    /// Create a Galois connection from two monotone functions.
    pub fn new(lower: impl Fn(&A) -> B + 'static, upper: impl Fn(&B) -> A + 'static) -> Self {
        Self {
            lower: Box::new(lower),
            upper: Box::new(upper),
        }
    }

    /// Apply the lower adjoint: A → B.
    pub fn lower(&self, a: &A) -> B {
        (self.lower)(a)
    }

    /// Apply the upper adjoint: B → A.
    pub fn upper(&self, b: &B) -> A {
        (self.upper)(b)
    }

    /// Check the unit/inflation law: a ≤ upper(lower(a)).
    pub fn check_unit(&self, a: &A) -> bool {
        *a <= (self.upper)(&(self.lower)(a))
    }

    /// Check the counit/deflation law: lower(upper(b)) ≤ b.
    pub fn check_counit(&self, b: &B) -> bool {
        (self.lower)(&(self.upper)(b)) <= *b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn galois_floor_ceil() {
        // Classic example: floor ⊣ ceil between f64 and i64
        // floor: f64 → i64 (round down)
        // ceil: i64 → f64 (embed as float)
        let gc = GaloisConnection::new(
            |x: &f64| *x as i64, // floor (lower)
            |n: &i64| *n as f64, // embed (upper)
        );

        assert_eq!(gc.lower(&3.7), 3);
        assert_eq!(gc.upper(&3), 3.0);

        // Unit: x ≤ embed(floor(x)) — 3.7 ≤ 3.0? No! But 3.0 ≤ 3.0.
        // Actually floor is left adjoint: floor(x) ≤ n ⟺ x ≤ embed(n)
        // The unit is: x ≤ embed(floor(x)) is NOT always true for floor.
        // The correct Galois connection for floor is:
        // embed ⊣ floor (embed is left, floor is right)
        // So: embed(n) ≤ x ⟺ n ≤ floor(x)
        // Unit: n ≤ floor(embed(n)) = floor(n.0) = n ✓
        let gc2 = GaloisConnection::new(
            |n: &i64| *n as f64, // embed (lower)
            |x: &f64| *x as i64, // floor (upper)
        );
        assert!(gc2.check_unit(&3)); // 3 ≤ floor(3.0) = 3 ✓
        assert!(gc2.check_counit(&3.7)); // embed(floor(3.7)) = 3.0 ≤ 3.7 ✓
    }

    #[test]
    fn galois_taxonomy_abstraction() {
        // Taxonomy levels: specific ≤ general
        // abstraction: "dog" → "mammal" (go up)
        // concretization: "mammal" → "dog" (go down to most specific)
        //
        // Simplified as numeric levels: 0=species, 1=genus, 2=family
        let gc = GaloisConnection::new(
            |level: &i32| level + 1, // abstract (lower: go up one level)
            |level: &i32| level - 1, // concretize (upper: go down one level)
        );

        // Unit: level ≤ concretize(abstract(level)) = (level+1)-1 = level ✓
        assert!(gc.check_unit(&0));
        assert!(gc.check_unit(&5));

        // Counit: abstract(concretize(level)) = (level-1)+1 = level ≤ level ✓
        assert!(gc.check_counit(&1));
        assert!(gc.check_counit(&5));
    }

    mod prop {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            /// For embed ⊣ floor: unit law holds for all integers
            #[test]
            fn prop_galois_unit(n in -1000..1000i64) {
                let gc = GaloisConnection::new(
                    |n: &i64| *n as f64,
                    |x: &f64| *x as i64,
                );
                prop_assert!(gc.check_unit(&n));
            }

            /// For embed ⊣ floor: counit law holds for all positive floats
            #[test]
            fn prop_galois_counit(x in 0.0..1000.0f64) {
                let gc = GaloisConnection::new(
                    |n: &i64| *n as f64,
                    |x: &f64| *x as i64,
                );
                prop_assert!(gc.check_counit(&x));
            }
        }
    }
}
