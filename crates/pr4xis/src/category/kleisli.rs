use super::category::Category;
use super::relationship::Relationship;

// Kleisli category — the category of effectful morphisms.
//
// For a monad M, the Kleisli category has:
//   Objects: same as the base category
//   Morphisms: A → M(B) (effectful arrows)
//   Composition: f >=> g = |a| f(a) >>= g (monadic composition)
//   Identity: pure/return
//
// In pr4xis, Category::compose returns Option<Morphism>.
// This IS the Kleisli category for the Maybe monad:
//   Objects: Entity variants
//   Morphisms: the morphisms that may or may not compose
//   Composition: compose(f, g) = if f.target == g.source then Some(...) else None
//
// The Kleisli structure makes partiality explicit and compositional.
//
// References:
// - Kleisli, "Every standard construction is induced by a pair of
//   adjoint functors" (1965, Proc. AMS)
//   https://doi.org/10.1090/S0002-9939-1965-0177024-4
// - Mac Lane, "Categories for the Working Mathematician" (1971), Ch. VI §5
// - Moggi, "Notions of Computation and Monads" (1991) — Kleisli as
//   the canonical model of effectful computation

/// A Kleisli morphism: an effectful arrow A → M(B).
///
/// Wraps a base morphism with explicit partiality.
/// `KleisliMorphism<C>` = the morphisms of the Kleisli category over C.
#[derive(Debug, Clone)]
pub struct KleisliMorphism<C: Category> {
    /// The underlying morphism (if it exists).
    pub arrow: Option<C::Morphism>,
    /// Source object (always known).
    pub source: C::Object,
    /// Target object (always known).
    pub target: C::Object,
}

impl<C: Category> KleisliMorphism<C>
where
    C::Object: Clone,
    C::Morphism: Clone,
{
    /// Lift a total morphism into the Kleisli category.
    pub fn total(morphism: C::Morphism) -> Self
    where
        C::Morphism: Relationship<Object = C::Object>,
    {
        let source = morphism.source();
        let target = morphism.target();
        Self {
            arrow: Some(morphism),
            source,
            target,
        }
    }

    /// The zero morphism (no connection).
    pub fn zero(source: C::Object, target: C::Object) -> Self {
        Self {
            arrow: None,
            source,
            target,
        }
    }

    /// Is this a total (non-partial) morphism?
    pub fn is_total(&self) -> bool {
        self.arrow.is_some()
    }

    /// Kleisli composition: f >=> g = if composable, compose; else None.
    pub fn compose(f: &Self, g: &Self) -> Self
    where
        C::Object: PartialEq,
    {
        if f.target != g.source {
            return Self::zero(f.source.clone(), g.target.clone());
        }
        match (&f.arrow, &g.arrow) {
            (Some(fm), Some(gm)) => Self {
                arrow: C::compose(fm, gm),
                source: f.source.clone(),
                target: g.target.clone(),
            },
            _ => Self::zero(f.source.clone(), g.target.clone()),
        }
    }

    /// Kleisli identity: pure/return.
    pub fn identity(obj: &C::Object) -> Self {
        Self {
            arrow: Some(C::identity(obj)),
            source: obj.clone(),
            target: obj.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::entity::Entity as EntityTrait;

    // Reuse a simple test category
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum Obj {
        A,
        B,
        C,
    }
    impl EntityTrait for Obj {
        fn variants() -> Vec<Self> {
            vec![Self::A, Self::B, Self::C]
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Morph {
        from: Obj,
        to: Obj,
    }
    impl Relationship for Morph {
        type Object = Obj;
        fn source(&self) -> Obj {
            self.from
        }
        fn target(&self) -> Obj {
            self.to
        }
    }

    struct Cat;
    impl Category for Cat {
        type Object = Obj;
        type Morphism = Morph;
        fn identity(obj: &Obj) -> Morph {
            Morph {
                from: *obj,
                to: *obj,
            }
        }
        fn compose(f: &Morph, g: &Morph) -> Option<Morph> {
            if f.to == g.from {
                Some(Morph {
                    from: f.from,
                    to: g.to,
                })
            } else {
                None
            }
        }
        fn morphisms() -> Vec<Morph> {
            vec![]
        }
    }

    #[test]
    fn kleisli_total_lifts_morphism() {
        let m = Morph {
            from: Obj::A,
            to: Obj::B,
        };
        let k = KleisliMorphism::<Cat>::total(m);
        assert!(k.is_total());
        assert_eq!(k.source, Obj::A);
        assert_eq!(k.target, Obj::B);
    }

    #[test]
    fn kleisli_zero_is_partial() {
        let k = KleisliMorphism::<Cat>::zero(Obj::A, Obj::C);
        assert!(!k.is_total());
    }

    #[test]
    fn kleisli_identity() {
        let id = KleisliMorphism::<Cat>::identity(&Obj::A);
        assert!(id.is_total());
        assert_eq!(id.source, Obj::A);
        assert_eq!(id.target, Obj::A);
    }

    #[test]
    fn kleisli_compose_total() {
        let f = KleisliMorphism::<Cat>::total(Morph {
            from: Obj::A,
            to: Obj::B,
        });
        let g = KleisliMorphism::<Cat>::total(Morph {
            from: Obj::B,
            to: Obj::C,
        });
        let h = KleisliMorphism::<Cat>::compose(&f, &g);
        assert!(h.is_total());
        assert_eq!(h.source, Obj::A);
        assert_eq!(h.target, Obj::C);
    }

    #[test]
    fn kleisli_compose_incompatible() {
        let f = KleisliMorphism::<Cat>::total(Morph {
            from: Obj::A,
            to: Obj::B,
        });
        let g = KleisliMorphism::<Cat>::total(Morph {
            from: Obj::C,
            to: Obj::A,
        });
        let h = KleisliMorphism::<Cat>::compose(&f, &g);
        assert!(!h.is_total()); // B ≠ C
    }

    #[test]
    fn kleisli_left_identity() {
        let f = KleisliMorphism::<Cat>::total(Morph {
            from: Obj::A,
            to: Obj::B,
        });
        let id = KleisliMorphism::<Cat>::identity(&Obj::A);
        let h = KleisliMorphism::<Cat>::compose(&id, &f);
        assert!(h.is_total());
        assert_eq!(h.arrow, f.arrow);
    }

    #[test]
    fn kleisli_right_identity() {
        let f = KleisliMorphism::<Cat>::total(Morph {
            from: Obj::A,
            to: Obj::B,
        });
        let id = KleisliMorphism::<Cat>::identity(&Obj::B);
        let h = KleisliMorphism::<Cat>::compose(&f, &id);
        assert!(h.is_total());
        assert_eq!(h.arrow, f.arrow);
    }
}
