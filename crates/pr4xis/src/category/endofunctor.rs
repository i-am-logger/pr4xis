use super::category::Category;
use super::functor::Functor;

/// An endofunctor is a functor whose source and target categories coincide: F: C → C.
///
/// Mac Lane (1971), *Categories for the Working Mathematician*, Ch. II §1.
///
/// # Why it matters
///
/// Endofunctors are the foundation of monads, comonads, fixed points, and every
/// self-referential construction in category theory. A monad is "just" an endofunctor
/// with η (unit) and μ (multiplication) natural transformations satisfying coherence
/// laws (Mac Lane Ch. VI §1; Wadler, *Monads for Functional Programming*, 1992).
///
/// Declaring a functor as `Endofunctor` makes the C → C identity first-class, rather
/// than implicit in `type Source = type Target`. Downstream code can require an
/// endofunctor at the type level (e.g., monad definitions, involutions, fixed-point
/// operators) without inspecting associated types.
///
/// # Laws
///
/// Endofunctors inherit the [`Functor`] laws (identity and composition preservation).
/// There are no additional laws — but
/// [`crate::category::validate::check_endofunctor_laws`] specialises those checks to a
/// single carrier category, which makes involution and fixed-point mistakes easier to
/// localise.
///
/// # Implementing
///
/// An `Endofunctor` impl is an explicit claim: "this functor's source and target are
/// the same category." The compiler enforces it via the `Source = Self::Category` and
/// `Target = Self::Category` constraints on the associated type.
///
/// ```ignore
/// impl Endofunctor for MyInvolution {
///     type Category = MyCategory;
/// }
/// ```
pub trait Endofunctor:
    Functor<Source = <Self as Endofunctor>::Category, Target = <Self as Endofunctor>::Category>
{
    /// The single category this endofunctor operates within.
    type Category: Category;
}
