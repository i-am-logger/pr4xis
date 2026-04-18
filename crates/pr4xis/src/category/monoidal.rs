use core::fmt::Debug;

// Monoidal category — a category with a tensor product and unit object.
//
// A monoidal category (C, ⊗, I) has:
//   Objects: objects of C
//   Tensor product ⊗: C × C → C (combines two objects)
//   Unit object I: the identity for ⊗
//   Associator: (A ⊗ B) ⊗ C ≅ A ⊗ (B ⊗ C)
//   Left unitor: I ⊗ A ≅ A
//   Right unitor: A ⊗ I ≅ A
//
// In pr4xis, monoidal categories formalize:
//   - Product ontologies: Vol × Con × Dur (storage tiers)
//   - Parallel composition: running two ontology queries simultaneously
//   - The traced monoidal structure (Joyal-Street-Verity 1996)
//
// References:
// - Mac Lane, "Categories for the Working Mathematician" (1971), Ch. VII
// - Mac Lane, "Natural Associativity and Commutativity" (1963, Rice Univ. Studies)
//   https://doi.org/10.1007/BFb0074297
// - Joyal & Street, "Braided Tensor Categories" (1993, Advances in Math.)

/// A product of two values — the tensor product in a monoidal category.
///
/// `Product<A, B>` = A ⊗ B. The simplest monoidal structure.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Product<A, B> {
    pub left: A,
    pub right: B,
}

impl<A: Clone + Debug, B: Clone + Debug> Product<A, B> {
    pub fn new(left: A, right: B) -> Self {
        Self { left, right }
    }

    /// Functor: map over both components.
    pub fn bimap<C: Clone + Debug, D: Clone + Debug>(
        self,
        f: impl FnOnce(A) -> C,
        g: impl FnOnce(B) -> D,
    ) -> Product<C, D> {
        Product {
            left: f(self.left),
            right: g(self.right),
        }
    }

    /// Map left component only.
    pub fn map_left<C: Clone + Debug>(self, f: impl FnOnce(A) -> C) -> Product<C, B> {
        Product {
            left: f(self.left),
            right: self.right,
        }
    }

    /// Map right component only.
    pub fn map_right<D: Clone + Debug>(self, g: impl FnOnce(B) -> D) -> Product<A, D> {
        Product {
            left: self.left,
            right: g(self.right),
        }
    }
}

/// The unit type for monoidal products.
/// I ⊗ A ≅ A ≅ A ⊗ I
pub type Unit = ();

/// Associator: (A ⊗ B) ⊗ C → A ⊗ (B ⊗ C)
pub fn assoc_right<A, B, C>(p: Product<Product<A, B>, C>) -> Product<A, Product<B, C>> {
    Product {
        left: p.left.left,
        right: Product {
            left: p.left.right,
            right: p.right,
        },
    }
}

/// Inverse associator: A ⊗ (B ⊗ C) → (A ⊗ B) ⊗ C
pub fn assoc_left<A, B, C>(p: Product<A, Product<B, C>>) -> Product<Product<A, B>, C> {
    Product {
        left: Product {
            left: p.left,
            right: p.right.left,
        },
        right: p.right.right,
    }
}

/// Left unitor: I ⊗ A → A
pub fn left_unitor<A>(p: Product<Unit, A>) -> A {
    p.right
}

/// Right unitor: A ⊗ I → A
pub fn right_unitor<A>(p: Product<A, Unit>) -> A {
    p.left
}

/// Swap: A ⊗ B → B ⊗ A (braiding for symmetric monoidal)
pub fn swap<A, B>(p: Product<A, B>) -> Product<B, A> {
    Product {
        left: p.right,
        right: p.left,
    }
}

/// The coproduct (sum type) — dual of product in a monoidal category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Coproduct<A, B> {
    Left(A),
    Right(B),
}

impl<A, B> Coproduct<A, B> {
    pub fn fold<C>(self, f: impl FnOnce(A) -> C, g: impl FnOnce(B) -> C) -> C {
        match self {
            Coproduct::Left(a) => f(a),
            Coproduct::Right(b) => g(b),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_new() {
        let p = Product::new(1, "hello");
        assert_eq!(p.left, 1);
        assert_eq!(p.right, "hello");
    }

    #[test]
    fn product_bimap() {
        let p = Product::new(1, 2).bimap(|x| x + 10, |y| y * 2);
        assert_eq!(p.left, 11);
        assert_eq!(p.right, 4);
    }

    // --- Monoidal laws ---

    #[test]
    fn associator_roundtrip() {
        let p = Product::new(Product::new(1, 2), 3);
        let reassoc = assoc_right(p.clone());
        let back = assoc_left(reassoc);
        assert_eq!(p, back);
    }

    #[test]
    fn left_unitor_law() {
        let p = Product::new((), 42);
        assert_eq!(left_unitor(p), 42);
    }

    #[test]
    fn right_unitor_law() {
        let p = Product::new(42, ());
        assert_eq!(right_unitor(p), 42);
    }

    #[test]
    fn swap_involution() {
        let p = Product::new(1, 2);
        let swapped = swap(swap(p.clone()));
        assert_eq!(p, swapped);
    }

    // --- Coproduct ---

    #[test]
    fn coproduct_fold() {
        let left: Coproduct<i32, &str> = Coproduct::Left(42);
        let right: Coproduct<i32, &str> = Coproduct::Right("hello");

        assert_eq!(left.fold(|x| x.to_string(), |s| s.to_string()), "42");
        assert_eq!(right.fold(|x| x.to_string(), |s| s.to_string()), "hello");
    }

    // --- Practical: product ontology ---

    #[test]
    fn product_ontology_storage() {
        // Vol × Con × Dur = storage tier classification
        #[derive(Debug, Clone, PartialEq)]
        enum Volatility {
            Register,
            SRAM,
            DRAM,
        }
        #[derive(Debug, Clone, PartialEq)]
        enum Consistency {
            Linearizable,
            Sequential,
            Eventual,
        }

        // Exercise all variants — product type works with any combination
        let dram_seq = Product::new(Volatility::DRAM, Consistency::Sequential);
        assert_eq!(dram_seq.left, Volatility::DRAM);

        let sram_lin = Product::new(Volatility::SRAM, Consistency::Linearizable);
        assert_eq!(sram_lin.right, Consistency::Linearizable);

        let reg_ev = Product::new(Volatility::Register, Consistency::Eventual);
        assert_eq!(reg_ev.left, Volatility::Register);
        assert_eq!(reg_ev.right, Consistency::Eventual);
    }

    mod prop {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            /// Associator roundtrip is identity
            #[test]
            fn prop_assoc_roundtrip(a in any::<i32>(), b in any::<i32>(), c in any::<i32>()) {
                let p = Product::new(Product::new(a, b), c);
                let back = assoc_left(assoc_right(p.clone()));
                prop_assert_eq!(p, back);
            }

            /// Swap is involution
            #[test]
            fn prop_swap_involution(a in any::<i32>(), b in any::<i32>()) {
                let p = Product::new(a, b);
                prop_assert_eq!(swap(swap(p.clone())), p);
            }

            /// Left unitor
            #[test]
            fn prop_left_unitor(a in any::<i32>()) {
                prop_assert_eq!(left_unitor(Product::new((), a)), a);
            }

            /// Right unitor
            #[test]
            fn prop_right_unitor(a in any::<i32>()) {
                prop_assert_eq!(right_unitor(Product::new(a, ())), a);
            }
        }
    }
}
