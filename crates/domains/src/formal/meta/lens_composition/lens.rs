//! The asymmetric well-behaved [`Lens`] and its sequential
//! composition (Foster, Greenwald, Moore, Pierce & Schmitt 2007).
//!
//! A lens between a source type `S` and a view type `V` is a pair of
//! functions
//!
//! - `get : S → V` — extract the view, and
//! - `put : V × S → S` — write an updated view back into a source,
//!
//! satisfying the three *well-behaved-lens laws* (Foster et al. 2007
//! §2.2):
//!
//! - **GetPut** `put(get(s), s) = s` — putting back an unchanged view
//!   leaves the source untouched.
//! - **PutGet** `get(put(v, s)) = v` — getting a just-put view returns
//!   that view.
//! - **PutPut** `put(v', put(v, s)) = put(v', s)` — a later put
//!   overwrites an earlier one (this is the *very*-well-behaved law).
//!
//! Lenses **compose sequentially** (Foster et al. 2007 §3): given
//! `l : S ⇆ V` and `k : V ⇆ W`, the composite `l ; k : S ⇆ W` has
//! `get = k.get ∘ l.get` and `put(w, s) = l.put(k.put(w, l.get(s)), s)`.
//! Composition of well-behaved lenses is well-behaved, and lens
//! composition is associative with [`IdentityLens`] as the unit — the
//! `S ⇆ V` lenses form a category (Foster et al. 2007 §3; Pierce 2006
//! lecture notes on lenses).
//!
//! This is the building block for the `file ⇄ xml ⇄ xsd ⇄ statute`
//! chain: each hop is a lens, and the whole pipeline is their
//! composite.
//!
//! ## Citation
//!
//! - **Foster, J. N., Greenwald, M. B., Moore, J. T., Pierce, B. C. &
//!   Schmitt, A.** "Combinators for Bidirectional Tree
//!   Transformations: A Linguistic Approach to the View-Update
//!   Problem", *ACM TOPLAS* 29(3), 2007. §2.2 (lens laws), §3
//!   (composition).
//! - **Bancilhon, F. & Spyratos, N.** "Update Semantics of Relational
//!   Views", *ACM TODS* 6(4), 1981 (constant complement).

#[allow(unused_imports)]
use alloc::vec::Vec;
use core::fmt;
use core::marker::PhantomData;

use crate::formal::meta::well_behaved_lens::WellBehavedLens;

/// An asymmetric well-behaved lens `Source ⇆ View` (Foster et al.
/// 2007 §2.2). Implementors should satisfy GetPut, PutGet, and PutPut
/// (verifiable with [`get_put_holds`] / [`put_get_holds`] /
/// [`put_put_holds`]).
pub trait Lens {
    /// The source type (the larger structure).
    type Source;
    /// The view type (the focused / projected structure).
    type View;
    /// Error type for `get` / `put`.
    type Error: fmt::Display;

    /// `get : S → V` — extract the view from a source.
    fn get(&self, source: &Self::Source) -> Result<Self::View, Self::Error>;

    /// `put : V × S → S` — write an updated view back into a source.
    fn put(&self, view: &Self::View, source: &Self::Source) -> Result<Self::Source, Self::Error>;
}

// =============================================================================
// Law checkers (Foster et al. 2007 §2.2).
// =============================================================================

/// GetPut: `put(get(s), s) == s`. Returns `false` if either operation
/// errors or the round-trip differs from `s`.
pub fn get_put_holds<L>(lens: &L, source: &L::Source) -> bool
where
    L: Lens,
    L::Source: PartialEq,
{
    match lens.get(source) {
        Ok(v) => matches!(lens.put(&v, source), Ok(s2) if &s2 == source),
        Err(_) => false,
    }
}

/// PutGet: `get(put(v, s)) == v`. Returns `false` if either operation
/// errors or the recovered view differs from `v`.
pub fn put_get_holds<L>(lens: &L, view: &L::View, source: &L::Source) -> bool
where
    L: Lens,
    L::View: PartialEq,
{
    match lens.put(view, source) {
        Ok(s2) => matches!(lens.get(&s2), Ok(v2) if &v2 == view),
        Err(_) => false,
    }
}

/// PutPut: `put(v2, put(v1, s)) == put(v2, s)`. Returns `false` if any
/// operation errors or the two sources differ.
pub fn put_put_holds<L>(lens: &L, v1: &L::View, v2: &L::View, source: &L::Source) -> bool
where
    L: Lens,
    L::Source: PartialEq,
{
    match lens.put(v1, source) {
        Ok(s1) => match (lens.put(v2, &s1), lens.put(v2, source)) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        },
        Err(_) => false,
    }
}

// =============================================================================
// Identity lens — the unit of composition.
// =============================================================================

/// The identity lens `S ⇆ S` (`get = id`, `put = fst`), the unit for
/// lens composition (Foster et al. 2007 §3).
#[derive(Debug, Clone, Copy, Default)]
pub struct IdentityLens<S>(PhantomData<S>);

impl<S> IdentityLens<S> {
    /// Construct the identity lens on `S`.
    pub fn new() -> Self {
        IdentityLens(PhantomData)
    }
}

impl<S: Clone> Lens for IdentityLens<S> {
    type Source = S;
    type View = S;
    type Error = core::convert::Infallible;

    fn get(&self, source: &S) -> Result<S, Self::Error> {
        Ok(source.clone())
    }

    fn put(&self, view: &S, _source: &S) -> Result<S, Self::Error> {
        Ok(view.clone())
    }
}

// =============================================================================
// WellBehavedLens adapter — bridge the byte-anchored well-behaved lens
// (file ⇄ ontology) to the general `Lens` so the byte hop composes with
// the typed-layer chain above it.
// =============================================================================

/// Adapt a [`WellBehavedLens`] `L : bytes ⇄ Target` (Foster et al. 2007
/// §2.2, byte-anchored) into the general [`Lens`] `Vec<u8> ⇄ L::Target`,
/// so it composes with the typed-layer lenses above the byte boundary.
///
/// The well-behaved-lens contract guarantees that
/// [`WellBehavedLens::put`] reconstructs the source bytes from the
/// target value alone (the target carries the constant complement per
/// Bancilhon & Spyratos 1981 — for the praxis byte lenses this is
/// literally the original `Vec<u8>` source carried as the target's
/// `complement` field). So the general-lens [`Lens::put`] can ignore
/// its `source` argument and delegate to `L::put(target)`; GetPut /
/// PutGet then follow from the well-behaved-lens laws of `L`.
#[derive(Debug, Clone, Copy)]
pub struct WellBehavedLensAdapter<L>(PhantomData<L>);

impl<L> WellBehavedLensAdapter<L> {
    /// Construct the adapter for `L`.
    pub fn new() -> Self {
        WellBehavedLensAdapter(PhantomData)
    }
}

impl<L> Default for WellBehavedLensAdapter<L> {
    fn default() -> Self {
        Self::new()
    }
}

impl<L> Lens for WellBehavedLensAdapter<L>
where
    L: WellBehavedLens,
{
    type Source = Vec<u8>;
    type View = L::Target;
    type Error = L::Error;

    fn get(&self, bytes: &Vec<u8>) -> Result<L::Target, L::Error> {
        L::get(bytes)
    }

    fn put(&self, target: &L::Target, _bytes: &Vec<u8>) -> Result<Vec<u8>, L::Error> {
        L::put(target)
    }
}

// =============================================================================
// Sequential composition (Foster et al. 2007 §3).
// =============================================================================

/// The sequential composite `first ; second` of two lenses
/// `first : S ⇆ V` and `second : V ⇆ W`, giving a lens `S ⇆ W`
/// (Foster et al. 2007 §3). Composition of well-behaved lenses is
/// well-behaved.
#[derive(Debug, Clone, Copy)]
pub struct Compose<L1, L2> {
    /// The first lens `S ⇆ V`.
    pub first: L1,
    /// The second lens `V ⇆ W`.
    pub second: L2,
}

impl<L1, L2> Compose<L1, L2> {
    /// Compose `first ; second`.
    pub fn new(first: L1, second: L2) -> Self {
        Compose { first, second }
    }
}

/// Error of a composite lens — the failure of whichever stage faulted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComposeError<E1, E2> {
    /// The first (outer, `S ⇆ V`) lens errored.
    First(E1),
    /// The second (inner, `V ⇆ W`) lens errored.
    Second(E2),
}

impl<E1: fmt::Display, E2: fmt::Display> fmt::Display for ComposeError<E1, E2> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComposeError::First(e) => write!(f, "first lens: {e}"),
            ComposeError::Second(e) => write!(f, "second lens: {e}"),
        }
    }
}

impl<L1, L2> Lens for Compose<L1, L2>
where
    L1: Lens,
    L2: Lens<Source = L1::View>,
{
    type Source = L1::Source;
    type View = L2::View;
    type Error = ComposeError<L1::Error, L2::Error>;

    fn get(&self, source: &Self::Source) -> Result<Self::View, Self::Error> {
        let v = self.first.get(source).map_err(ComposeError::First)?;
        self.second.get(&v).map_err(ComposeError::Second)
    }

    fn put(&self, view: &Self::View, source: &Self::Source) -> Result<Self::Source, Self::Error> {
        // put(w, s) = l.put(k.put(w, l.get(s)), s)  (Foster et al. §3).
        let v = self.first.get(source).map_err(ComposeError::First)?;
        let v2 = self.second.put(view, &v).map_err(ComposeError::Second)?;
        self.first.put(&v2, source).map_err(ComposeError::First)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::{String, ToString};

    /// A two-field record used as a concrete source for the example
    /// lenses below.
    #[derive(Debug, Clone, PartialEq)]
    struct Pair {
        a: i32,
        b: i32,
    }

    /// `fst : Pair ⇆ i32` — focuses the `a` field.
    struct Fst;
    impl Lens for Fst {
        type Source = Pair;
        type View = i32;
        type Error = core::convert::Infallible;
        fn get(&self, s: &Pair) -> Result<i32, Self::Error> {
            Ok(s.a)
        }
        fn put(&self, v: &i32, s: &Pair) -> Result<Pair, Self::Error> {
            Ok(Pair { a: *v, b: s.b })
        }
    }

    /// `affine : i32 ⇆ i32` — view is `x + 1`; put recovers `v - 1`.
    struct PlusOne;
    impl Lens for PlusOne {
        type Source = i32;
        type View = i32;
        type Error = core::convert::Infallible;
        fn get(&self, s: &i32) -> Result<i32, Self::Error> {
            Ok(s + 1)
        }
        fn put(&self, v: &i32, _s: &i32) -> Result<i32, Self::Error> {
            Ok(v - 1)
        }
    }

    #[test]
    fn fst_is_well_behaved() {
        let s = Pair { a: 3, b: 7 };
        assert!(get_put_holds(&Fst, &s));
        assert!(put_get_holds(&Fst, &10, &s));
        assert!(put_put_holds(&Fst, &10, &20, &s));
    }

    #[test]
    fn plus_one_is_well_behaved() {
        assert!(get_put_holds(&PlusOne, &5));
        assert!(put_get_holds(&PlusOne, &9, &5));
        assert!(put_put_holds(&PlusOne, &9, &12, &5));
    }

    #[test]
    fn identity_is_well_behaved() {
        let id = IdentityLens::<Pair>::new();
        let s = Pair { a: 1, b: 2 };
        assert!(get_put_holds(&id, &s));
        assert!(put_get_holds(&id, &Pair { a: 9, b: 9 }, &s));
        assert!(put_put_holds(
            &id,
            &Pair { a: 9, b: 9 },
            &Pair { a: 8, b: 8 },
            &s
        ));
    }

    #[test]
    fn composition_preserves_well_behavedness() {
        // Fst ; PlusOne : Pair ⇆ i32  (view = a + 1).
        let lens = Compose::new(Fst, PlusOne);
        let s = Pair { a: 3, b: 7 };
        assert_eq!(lens.get(&s).unwrap(), 4);
        // The composite is well-behaved (Foster et al. §3).
        assert!(get_put_holds(&lens, &s));
        assert!(put_get_holds(&lens, &11, &s)); // put view 11 → a = 10
        assert!(put_put_holds(&lens, &11, &21, &s));
        // put writes through both stages: view 11 ⇒ a = 10, b kept.
        assert_eq!(lens.put(&11, &s).unwrap(), Pair { a: 10, b: 7 });
    }

    #[test]
    fn identity_is_the_composition_unit() {
        // id ; Fst  and  Fst ; id  both behave as Fst.
        let s = Pair { a: 3, b: 7 };
        let left = Compose::new(IdentityLens::<Pair>::new(), Fst);
        let right = Compose::new(Fst, IdentityLens::<i32>::new());
        assert_eq!(left.get(&s).unwrap(), Fst.get(&s).unwrap());
        assert_eq!(right.get(&s).unwrap(), Fst.get(&s).unwrap());
        assert_eq!(left.put(&10, &s).unwrap(), Fst.put(&10, &s).unwrap());
        assert_eq!(right.put(&10, &s).unwrap(), Fst.put(&10, &s).unwrap());
    }

    #[test]
    fn composition_is_associative() {
        // (Fst ; PlusOne) ; PlusOne  ==  Fst ; (PlusOne ; PlusOne).
        let s = Pair { a: 3, b: 7 };
        let l = Compose::new(Compose::new(Fst, PlusOne), PlusOne);
        let r = Compose::new(Fst, Compose::new(PlusOne, PlusOne));
        assert_eq!(l.get(&s).unwrap(), r.get(&s).unwrap()); // a + 2 = 5
        assert_eq!(l.put(&12, &s).unwrap(), r.put(&12, &s).unwrap());
    }

    #[test]
    fn compose_error_displays_the_faulting_stage() {
        struct Boom;
        impl Lens for Boom {
            type Source = i32;
            type View = i32;
            type Error = String;
            fn get(&self, _s: &i32) -> Result<i32, String> {
                Err("kaboom".to_string())
            }
            fn put(&self, _v: &i32, _s: &i32) -> Result<i32, String> {
                Err("kaboom".to_string())
            }
        }
        let lens = Compose::new(IdentityLens::<i32>::new(), Boom);
        let err = lens.get(&1).unwrap_err();
        assert!(err.to_string().contains("second lens: kaboom"));
    }
}
