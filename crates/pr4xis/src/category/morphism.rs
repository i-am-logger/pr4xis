use super::category::Category;
use super::relationship::Relationship;
#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// A morphism bound to its category — enables functional chaining.
///
/// Wraps a raw relationship with its category context so you can write:
/// ```ignore
/// Morphism::of::<MyCat>(f)
///     .then(&g)
///     .then(&h)
/// ```
#[derive(Debug, Clone)]
pub struct Morphism<C: Category> {
    inner: C::Morphism,
}

impl<C: Category> Morphism<C> {
    /// Wrap a relationship in its category context.
    pub fn of(m: C::Morphism) -> Self {
        Self { inner: m }
    }

    /// The identity morphism for an object.
    pub fn id(obj: &C::Object) -> Self {
        Self {
            inner: C::identity(obj),
        }
    }

    /// Compose with another morphism: self then other (self: A→B, other: B→C → A→C).
    pub fn then(&self, other: &C::Morphism) -> Option<Self>
    where
        C::Morphism: Clone,
    {
        C::compose(&self.inner, other).map(|m| Self { inner: m })
    }

    /// Compose with another wrapped morphism.
    pub fn and_then(&self, other: &Self) -> Option<Self>
    where
        C::Morphism: Clone,
    {
        self.then(&other.inner)
    }

    /// The source entity.
    pub fn source(&self) -> C::Object {
        self.inner.source()
    }

    /// The target entity.
    pub fn target(&self) -> C::Object {
        self.inner.target()
    }

    /// Unwrap to the raw relationship.
    pub fn into_inner(self) -> C::Morphism {
        self.inner
    }

    /// Reference to the raw relationship.
    pub fn inner(&self) -> &C::Morphism {
        &self.inner
    }
}

impl<C: Category> PartialEq for Morphism<C>
where
    C::Morphism: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<C: Category> Eq for Morphism<C> where C::Morphism: Eq {}

/// Compose a sequence of morphisms left to right.
///
/// ```ignore
/// let path = compose_all::<MyCat>(&[f, g, h]);
/// // equivalent to: f.then(g).then(h)
/// ```
pub fn compose_all<C: Category>(morphisms: &[C::Morphism]) -> Option<Morphism<C>>
where
    C::Morphism: Clone,
{
    let mut iter = morphisms.iter();
    let first = Morphism::<C>::of(iter.next()?.clone());
    iter.try_fold(first, |acc, m| acc.then(m))
}

/// All direct morphisms from `start` to `end` (single-step, not multi-hop).
pub fn direct_morphisms<C: Category>(start: &C::Object, end: &C::Object) -> Vec<C::Morphism>
where
    C::Object: PartialEq,
{
    C::morphisms()
        .into_iter()
        .filter(|m| m.source() == *start && m.target() == *end)
        .collect()
}
