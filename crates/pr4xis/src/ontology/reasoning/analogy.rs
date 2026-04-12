use std::marker::PhantomData;

use crate::category::validate::check_functor_laws;
use crate::category::{Category, Functor};

/// An analogy is a structure-preserving map between two domains (a named Functor).
///
/// If domain A is-like domain B, there exists an analogy (functor) F: A → B
/// that maps objects and morphisms while preserving identity and composition.
pub struct Analogy<F: Functor> {
    _marker: PhantomData<F>,
}

impl<F: Functor> Analogy<F> {
    /// Validate that the analogy preserves structure (functor laws).
    pub fn validate() -> Result<(), Vec<String>>
    where
        <F::Source as Category>::Morphism: PartialEq,
        <F::Target as Category>::Morphism: PartialEq,
    {
        check_functor_laws::<F>().map_err(|e| vec![e])
    }

    /// Map a concept from the source domain to the target domain.
    pub fn translate(obj: &<F::Source as Category>::Object) -> <F::Target as Category>::Object {
        F::map_object(obj)
    }

    /// Map a relationship from the source domain to the target domain.
    pub fn translate_morphism(
        m: &<F::Source as Category>::Morphism,
    ) -> <F::Target as Category>::Morphism {
        F::map_morphism(m)
    }
}
