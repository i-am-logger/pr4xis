use core::marker::PhantomData;

use super::category::Category;
use super::functor::Functor;

/// A type-level handle for "the single target object this terminal functor
/// maps to" — the ergonomic workaround for the fact that Rust const generics
/// can't yet carry enum variants.
///
/// Implement this on a zero-sized marker type to declare which object of which
/// category every source is being collapsed to.
///
/// ```ignore
/// struct FaultToleranceTarget;
/// impl TerminalTarget for FaultToleranceTarget {
///     type Category = DependabilityCategory;
///     fn target() -> DependabilityConcept { DependabilityConcept::FaultTolerance }
/// }
/// ```
pub trait TerminalTarget {
    type Category: Category;
    fn target() -> <Self::Category as Category>::Object;
}

/// The terminal functor `!_T: C → {T, id_T}` onto the one-object subcategory
/// of `T::Category` spanned by a single object `T::target()`.
///
/// Every object in the source category maps to `T::target()`; every morphism
/// maps to `id_{T::target()}`. The functor laws hold trivially:
///
/// - `F(id_A) = id_{T::target()} = id_{F(A)}`
/// - `F(g ∘ f) = id_{T::target()} = id_{T::target()} ∘ id_{T::target()} = F(g) ∘ F(f)`
///
/// This is the ontologically honest "S factors through a single aspect of T"
/// mapping — e.g., "every Resilience pattern is a FaultTolerance means."
pub struct TerminalFunctor<Src, T>(PhantomData<(Src, T)>);

impl<Src, T> Functor for TerminalFunctor<Src, T>
where
    Src: Category,
    T: TerminalTarget,
{
    type Source = Src;
    type Target = T::Category;

    fn map_object(_: &<Src as Category>::Object) -> <T::Category as Category>::Object {
        T::target()
    }

    fn map_morphism(_: &<Src as Category>::Morphism) -> <T::Category as Category>::Morphism {
        <T::Category>::identity(&T::target())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::validate::check_functor_laws;
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
        fn source(&self) -> Light {
            self.from
        }
        fn target(&self) -> Light {
            self.to
        }
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
            let vs = Light::variants();
            vs.iter()
                .flat_map(|&a| vs.iter().map(move |&b| LightEdge { from: a, to: b }))
                .collect()
        }
    }

    struct RedTarget;
    impl TerminalTarget for RedTarget {
        type Category = LightCat;
        fn target() -> Light {
            Light::Red
        }
    }

    type LightToRed = TerminalFunctor<LightCat, RedTarget>;

    #[test]
    fn terminal_functor_onto_red_satisfies_laws() {
        check_functor_laws::<LightToRed>().unwrap();
    }
}
