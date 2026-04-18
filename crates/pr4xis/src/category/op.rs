use core::marker::PhantomData;

use super::category::Category;
use super::relationship::Relationship;

/// The opposite (dual) category of `C`.
///
/// Mac Lane (1971), *Categories for the Working Mathematician*, Ch. II §2.
///
/// `Op<C>` has the same objects as `C` but every morphism is reversed:
///
/// - An arrow `f: A → B` in `C` becomes an arrow `f^op: B → A` in `Op<C>`.
/// - Composition order is reversed: `compose_op(f, g) = compose_C(g, f)`.
///
/// This lets a contravariant construction (e.g., abductive inference from
/// observation back to cause) be expressed as a covariant `Functor` impl —
/// the Rust side of what category theorists write as `F: C^op → D`.
///
/// # Laws
///
/// When `C` satisfies the category laws, `Op<C>` does automatically. Verified
/// by [`crate::category::validate::check_category_laws`] applied to `Op<C>`.
pub struct Op<C>(PhantomData<C>);

/// A morphism in `Op<C>`: the same underlying `C::Morphism`, but [`source`](Relationship::source)
/// and [`target`](Relationship::target) are swapped.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OpMorphism<M>(pub M);

impl<M: Relationship> Relationship for OpMorphism<M> {
    type Object = M::Object;
    type Kind = M::Kind;

    fn source(&self) -> Self::Object {
        self.0.target()
    }

    fn target(&self) -> Self::Object {
        self.0.source()
    }

    fn kind(&self) -> Self::Kind {
        self.0.kind()
    }
}

impl<C: Category> Category for Op<C> {
    type Object = C::Object;
    type Morphism = OpMorphism<C::Morphism>;

    fn identity(obj: &Self::Object) -> Self::Morphism {
        // id_A^op == (id_A)^op — identity is its own opposite.
        OpMorphism(C::identity(obj))
    }

    fn compose(f: &Self::Morphism, g: &Self::Morphism) -> Option<Self::Morphism> {
        // In Op<C>:
        //   f: A → B  means  f.0: B → A  in C
        //   g: B → C  means  g.0: C → B  in C
        // We want Op::compose(f, g): A → C in Op<C>, i.e. an underlying
        // morphism C → A in C. That's the composition g.0 (C→B) then f.0 (B→A)
        // — in diagrammatic order: C::compose(&g.0, &f.0).
        C::compose(&g.0, &f.0).map(OpMorphism)
    }

    fn morphisms() -> Vec<Self::Morphism> {
        C::morphisms().into_iter().map(OpMorphism).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::validate::check_category_laws;
    use crate::category::{Entity, Relationship};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum Light {
        Red,
        Green,
    }

    impl Entity for Light {
        fn variants() -> Vec<Self> {
            vec![Light::Red, Light::Green]
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct LightEdge {
        from: Light,
        to: Light,
    }

    impl Relationship for LightEdge {
        type Object = Light;
        type Kind = ();
        fn source(&self) -> Light {
            self.from
        }
        fn target(&self) -> Light {
            self.to
        }
        fn kind(&self) {}
    }

    struct LightCat;
    impl Category for LightCat {
        type Object = Light;
        type Morphism = LightEdge;
        fn identity(obj: &Light) -> LightEdge {
            LightEdge {
                from: *obj,
                to: *obj,
            }
        }
        fn compose(f: &LightEdge, g: &LightEdge) -> Option<LightEdge> {
            if f.to != g.from {
                return None;
            }
            Some(LightEdge {
                from: f.from,
                to: g.to,
            })
        }
        fn morphisms() -> Vec<LightEdge> {
            vec![
                LightEdge {
                    from: Light::Red,
                    to: Light::Red,
                },
                LightEdge {
                    from: Light::Green,
                    to: Light::Green,
                },
                LightEdge {
                    from: Light::Red,
                    to: Light::Green,
                },
                LightEdge {
                    from: Light::Green,
                    to: Light::Red,
                },
            ]
        }
    }

    #[test]
    fn op_preserves_category_laws() {
        check_category_laws::<LightCat>().unwrap();
        check_category_laws::<Op<LightCat>>().unwrap();
    }

    #[test]
    fn op_flips_source_and_target() {
        let m = LightEdge {
            from: Light::Red,
            to: Light::Green,
        };
        let m_op = OpMorphism(m);
        assert_eq!(m_op.source(), Light::Green);
        assert_eq!(m_op.target(), Light::Red);
    }

    #[test]
    fn op_composition_reverses_order() {
        // In C: Red → Green composed with Green → Red gives Red → Red.
        let r_to_g = LightEdge {
            from: Light::Red,
            to: Light::Green,
        };
        let g_to_r = LightEdge {
            from: Light::Green,
            to: Light::Red,
        };

        // In Op<C>: OpMorphism(r_to_g) has source Green, target Red.
        //          OpMorphism(g_to_r) has source Red, target Green.
        // Composing OpMorphism(g_to_r): Red → Green with OpMorphism(r_to_g): Green → Red
        // should produce a morphism Red → Red in Op<C>.
        let composed = <Op<LightCat>>::compose(&OpMorphism(g_to_r), &OpMorphism(r_to_g)).unwrap();
        assert_eq!(composed.source(), Light::Red);
        assert_eq!(composed.target(), Light::Red);
    }
}
