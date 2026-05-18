//! Typed const-constructable handles into an ontology corpus.
//!
//! [`EntityRef`] is the build-time→runtime handle emitted by codegen and
//! consumed by ontology `from_codegen` functors. The phantom marker `P`
//! tags each handle with the ontology it belongs to, so handles from
//! different ontologies are distinct types at compile time but compile
//! to the same machine code (a single `u64`).
//!
//! Literature: typed-handle pattern is the standard way to express
//! ontology-distinct identity at the type level — see Spivak (2014)
//! *Category Theory for the Sciences* §3.1 on typed morphisms, and the
//! `Reference<N>` pattern already in pr4xis-domains.

use core::marker::PhantomData;

/// Typed const-constructable handle into an ontology corpus.
///
/// Phantom marker `P` distinguishes handles per ontology — `EntityRef<English>`
/// and `EntityRef<UsCode>` are distinct types at compile time but compile to
/// the same machine code (a single u64). Const-constructable so it can appear
/// directly inside `&'static [(EntityRef<P>, EntityRef<P>)]` static arrays
/// emitted by codegen.
///
/// Literature: typed-handle pattern is the standard way to express
/// ontology-distinct identity at the type level — see Spivak (2014)
/// *Category Theory for the Sciences* §3.1 on typed morphisms, and the
/// `Reference<N>` pattern already in pr4xis-domains.
#[derive(Debug)]
pub struct EntityRef<P> {
    value: u64,
    _phantom: PhantomData<P>,
}

// Manual impls so the trait bounds don't transitively require `P: Trait`.
// `PhantomData<P>` itself implements Copy/Clone/PartialEq/Eq/Hash
// unconditionally, but deriving on the outer struct would force `P` to
// satisfy each trait too. The marker `P` is purely a tag; never inspected.
impl<P> Clone for EntityRef<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P> Copy for EntityRef<P> {}

impl<P> PartialEq for EntityRef<P> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<P> Eq for EntityRef<P> {}

impl<P> core::hash::Hash for EntityRef<P> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<P> EntityRef<P> {
    /// Construct a new typed handle. `const` so it can appear in
    /// `static` arrays emitted by codegen.
    pub const fn new(value: u64) -> Self {
        Self {
            value,
            _phantom: PhantomData,
        }
    }

    /// The raw numeric value backing this handle.
    pub const fn value(&self) -> u64 {
        self.value
    }
}
