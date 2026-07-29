//! A string interner for lexical SURFACES — the classic compiler symbol-table
//! device: map each distinct surface to a small integer handle so the many maps
//! that key on a surface hold a 4-byte handle instead of re-owning the bytes.
//!
//! Every distinct surface's bytes live exactly ONCE, contiguously, in the
//! [`Interner`]'s arena; a [`Symbol`] is a dense `u32` index into that arena.
//! The membership set is a `hashbrown::HashTable<Symbol>` hashed and compared
//! through each handle's RESOLVED surface, so the set stores only handles — the
//! surface bytes are never copied a second time into the map.
//!
//! Why this earns its place here: the [`ComposedReasoner`](super::composed) used
//! to key its union lookup on `HashMap<String, Vec<ConceptId>>`, minting a fresh
//! owned `String` for every one of English's ~131k surfaces (plus every loaded
//! ontology's surfaces) — strings the underlying graph already owns. Interning
//! collapses that per-surface duplication to one shared arena keyed by `Symbol`,
//! a representation change with IDENTICAL resolution: a surface interns to the
//! same handle it did before, so it resolves to the same `ConceptId`s.
//!
//! The interner is a MECHANISM (a data structure), not domain knowledge: it
//! case-folds nothing and normalizes nothing. Callers intern exactly the bytes
//! they would otherwise have used as a `String` key, and query with exactly the
//! bytes they would otherwise have looked up — so the handle-keyed map is a
//! bijective re-encoding of the string-keyed one.
//!
//! Literature:
//! - Aho, Lam, Sethi & Ullman (2006) *Compilers: Principles, Techniques, and
//!   Tools* (2nd ed.), §2.7 "Symbol Tables" — the canonical account of interning
//!   identifiers into a single stored copy addressed by an integer handle. The
//!   general technique is "string interning": one arena, integer handles,
//!   equality and storage by handle.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::hash::BuildHasher;

use hashbrown::{DefaultHashBuilder, HashTable};

use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

/// An interned lexical surface: a small `Copy` handle standing for exactly one
/// unique surface string held once in an [`Interner`]'s arena.
///
/// A `Symbol` is minted SOLELY by [`Interner::intern`]; there is deliberately no
/// public constructor from a raw `u32`, so a handle can never be forged for a
/// surface the interner never saw. Two surfaces share a `Symbol` iff their bytes
/// are equal. As a `u32` handle it is cheap to `Copy`, hash, and order — a rich
/// interned-surface identity, not a bare integer leaking through the API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Symbol(u32);

/// A surface's byte span into the [`Interner`] arena — its `[start, start+len)`
/// bytes. Packed as two `u32`s; the arena holds at most `u32::MAX` bytes and each
/// surface is at most `u32::MAX` bytes, both enforced by [`Interner::intern`].
#[derive(Clone, Copy)]
struct Span {
    /// Byte offset of the surface's first byte in the arena.
    start: u32,
    /// Length of the surface in bytes.
    len: u32,
}

/// The hash of a surface under `build`, computed the SAME way whether the bytes
/// come from a query string or from a handle resolved against the arena — so a
/// query hashes to the same bucket its interning did.
fn hash_surface(build: &DefaultHashBuilder, surface: &str) -> u64 {
    build.hash_one(surface)
}

/// A string interner over lexical surfaces: one contiguous byte arena of unique
/// surfaces, addressed by dense [`Symbol`] handles.
pub struct Interner {
    /// Every interned surface's bytes, back to back — each distinct surface once.
    arena: String,
    /// Per-`Symbol` byte span into `arena`, indexed by the handle's `u32` value.
    spans: Vec<Span>,
    /// The interned handles, hashed and compared by their resolved surface, so
    /// the set stores only 4-byte handles (the bytes live once, in `arena`).
    table: HashTable<Symbol>,
    /// The hasher placing a surface in `table`; reused verbatim on lookup and on
    /// resize so a surface always hashes to the same bucket.
    hasher: DefaultHashBuilder,
}

impl Interner {
    /// An empty interner.
    pub fn new() -> Self {
        Interner {
            arena: String::new(),
            spans: Vec::new(),
            table: HashTable::new(),
            hasher: DefaultHashBuilder::default(),
        }
    }

    /// The surface a handle stands for — its bytes, borrowed from the arena.
    ///
    /// Total for every `Symbol` this interner minted (they index `spans` densely
    /// in interning order); a handle from a *different* interner is not valid
    /// here, which is why handles are only ever minted, never constructed.
    pub fn resolve(&self, symbol: Symbol) -> &str {
        let span = self.spans[symbol.0 as usize];
        &self.arena[span.start as usize..span.start as usize + span.len as usize]
    }

    /// The handle for `surface` if it has already been interned, without minting
    /// one — the non-mutating query path. `None` means this interner never saw
    /// `surface`, so (by construction) no map keyed on its handles holds it.
    pub fn get(&self, surface: &str) -> Option<Symbol> {
        let hash = hash_surface(&self.hasher, surface);
        self.table
            .find(hash, |&symbol| self.resolve(symbol) == surface)
            .copied()
    }

    /// Intern `surface`, returning its handle — the same handle on every call
    /// with equal bytes. A first sighting appends the bytes to the arena and
    /// mints the next dense handle; a repeat returns the existing one.
    pub fn intern(&mut self, surface: &str) -> Symbol {
        let hash = hash_surface(&self.hasher, surface);
        if let Some(&symbol) = self
            .table
            .find(hash, |&symbol| self.resolve(symbol) == surface)
        {
            return symbol;
        }

        // A fresh surface: append its bytes and mint the next handle. The `u32`
        // span/index bounds are the `Symbol` type's domain — assert they hold so
        // an over-capacity corpus fails closed rather than truncating silently.
        let start = self.arena.len();
        assert!(
            start + surface.len() <= u32::MAX as usize && self.spans.len() < u32::MAX as usize,
            "interner capacity exceeded: a Symbol addresses at most u32::MAX \
             surfaces and u32::MAX arena bytes",
        );
        self.arena.push_str(surface);
        self.spans.push(Span {
            start: start as u32,
            len: surface.len() as u32,
        });
        let symbol = Symbol((self.spans.len() - 1) as u32);

        // Insert under the SAME hash. On a resize hashbrown re-hashes existing
        // handles through this closure, which resolves each against the arena —
        // so the table stores only handles, never a second copy of the bytes.
        // Fields are destructured to borrow `table` mutably while
        // `arena`/`spans`/`hasher` are borrowed immutably (disjoint borrows).
        let Interner {
            arena,
            spans,
            table,
            hasher,
        } = self;
        table.insert_unique(hash, symbol, |&symbol| {
            let span = spans[symbol.0 as usize];
            hash_surface(
                hasher,
                &arena[span.start as usize..span.start as usize + span.len as usize],
            )
        });
        symbol
    }

    /// The number of distinct surfaces interned.
    pub fn len(&self) -> Quantity {
        Quantity::from_unit(self.spans.len() as f64, &unit::UNITLESS)
    }

    /// Whether nothing has been interned yet.
    pub fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }
}

impl Default for Interner {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Interner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The arena bytes are not interesting to a reader; the shape is.
        f.debug_struct("Interner")
            .field("symbols", &self.spans.len())
            .field("arena_bytes", &self.arena.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Interning is idempotent by bytes: equal surfaces share one handle,
    /// distinct surfaces get distinct handles, and the arena holds each once.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn equal_surfaces_share_one_handle() {
        let mut interner = Interner::new();
        let a1 = interner.intern("section");
        let a2 = interner.intern("section");
        let b = interner.intern("statute");
        assert_eq!(a1, a2, "equal surfaces intern to the same handle");
        assert_ne!(a1, b, "distinct surfaces intern to distinct handles");
        // Two distinct surfaces → two symbols, and the arena grew by the bytes
        // of each once (no third copy of "section").
        assert_eq!(interner.len().value, 2.0);
        assert_eq!(interner.resolve(a1), "section");
        assert_eq!(interner.resolve(b), "statute");
    }

    /// `get` is the non-mutating query path: `Some` for an interned surface (the
    /// same handle `intern` minted), `None` for one never seen — and a `None`
    /// leaves the interner untouched.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn get_resolves_only_interned_surfaces() {
        let mut interner = Interner::new();
        let seen = interner.intern("title");
        assert_eq!(interner.get("title"), Some(seen));
        assert_eq!(interner.get("subsection"), None);
        // The failed lookup did not intern anything.
        assert_eq!(interner.len().value, 1.0);
    }

    /// Case is significant — the interner normalizes nothing, so "Statute" and
    /// "statute" are distinct surfaces (any case-folding is the CALLER's, done
    /// identically on both the intern and query sides, exactly as before).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn interning_is_byte_exact_never_case_folding() {
        let mut interner = Interner::new();
        let upper = interner.intern("Statute");
        let lower = interner.intern("statute");
        assert_ne!(upper, lower, "the interner does not case-fold");
        assert_eq!(interner.get("Statute"), Some(upper));
        assert_eq!(interner.get("statute"), Some(lower));
    }

    /// A resize (many insertions forcing the table to grow and re-hash existing
    /// handles through the arena-resolving closure) preserves every handle's
    /// identity and resolution — the round-trip the reasoner relies on at scale.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn survives_resize_with_stable_handles() {
        let mut interner = Interner::new();
        let mut handles = alloc::vec::Vec::new();
        for i in 0..1_000u32 {
            let surface = alloc::format!("surface-{i}");
            handles.push((interner.intern(&surface), surface));
        }
        // Every handle still resolves to its own surface, and re-interning
        // yields the identical handle after all the resizes.
        for (symbol, surface) in &handles {
            assert_eq!(interner.resolve(*symbol), surface.as_str());
            assert_eq!(interner.intern(surface), *symbol);
        }
        assert_eq!(interner.len().value, 1_000.0);
    }
}
