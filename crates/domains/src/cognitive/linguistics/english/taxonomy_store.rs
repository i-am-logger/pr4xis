//! The English hypernym taxonomy edges as an immutable, zero-copy CSR family —
//! and the reflexive-transitive is-a reachability queries computed *per query*
//! over them.
//!
//! `English` answers four reachability questions against WordNet's hypernym
//! (Subsumption) DAG — [`is_a`](TaxonomyStore::is_a),
//! [`ancestors`](TaxonomyStore::ancestors),
//! [`common_ancestor`](TaxonomyStore::common_ancestor) and
//! [`ancestor_chain`](TaxonomyStore::ancestor_chain). Open English WordNet's
//! hypernym relation is a **shallow** DAG (max is-a depth 16, largest reflexive
//! ancestor set 33 nodes), so a bounded breadth-first ascent over the direct
//! parent edges reproduces the eager reflexive-transitive closure's answer
//! *exactly* — unit-weight BFS grades every node at its minimal hop count
//! (Moore 1959) — at a per-query cost indistinguishable from the
//! O(1) closure lookup, while dropping the ~697k-pair closure entirely. The
//! ascent itself is the ONE shared graded-reach engine
//! ([`pr4xis::category::reach`]): `TaxonomyStore` implements
//! [`ReachSubstrate`] (kind = [`Direction`], vertex = [`ConceptId`], whose
//! derived `Ord` is its `value()` order, so the pinned
//! `(distance, ConceptId.value())` tie-break is the kernel's own
//! `(hops, V::Ord)` contract) and every query mints a per-call [`ReachView`]
//! over the [`Uncached`] memo policy — stateless, **no interior mutability**,
//! so `TaxonomyStore` stays `Sync` inside `English`'s `OnceLock`.
//!
//! # The representation
//!
//! Both hypernym directions (child→parents, parent→children) are ONE instance of
//! the shared
//! [`PackedCsrFamily`]: a
//! [`DenseId`](crate::formal::meta::packed_csr::DenseId)-indexed family of
//! [`PodRun`]`<ConceptId>` columns,
//! labelled by [`Direction`]. Because a [`ConceptId`] is the *dense* synset index
//! `0..N`, each column is indexed directly by the id — no hashing. The zero-copy
//! `&[ConceptId]` cast, the CSR reader, the little-endian invariant, and the
//! owned adjacency fallback all live in that one hand-audited generic;
//! [`parents`](TaxonomyStore::parents) / [`children`](TaxonomyStore::children)
//! are the two labelled columns, and the four reachability queries read them.

use alloc::vec::Vec;

use hashbrown::HashMap;

use pr4xis::category::reach::{ReachSubstrate, ReachView, Uncached};

use super::ontology::ConceptId;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;
use crate::formal::meta::packed_csr::{LabelKind, PackedCsrFamily, PodRun};

/// The two directions of the hypernym (Subsumption) relation — the family label,
/// and the [`ReachSubstrate::Kind`] axis of the shared graded-reach engine
/// (`Ord` because a substrate kind is a map key / total order there).
///
/// Literature: SKOS `broader` / `narrower` (Miles & Bechhofer, *SKOS Reference*,
/// W3C REC-skos-reference-20090818, §8.1, §8.3 — the hierarchical link, with its
/// inverse pair stated by axiom S25 `skos:narrower owl:inverseOf skos:broader`). `Parent` (a concept's hypernyms) is the SKOS-`broader` direction;
/// `Child` (its hyponyms) is SKOS-`narrower`. Equivalently OBO Relation Ontology
/// `is_a` and its inverse (Smith et al. 2005).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction {
    /// Hypernyms — a concept's direct parents (SKOS `broader` / OBO-RO `is_a`).
    Parent,
    /// Hyponyms — a concept's direct children (SKOS `narrower`).
    Child,
}

impl LabelKind for Direction {
    const COUNT: usize = 2;
    fn index(self) -> usize {
        self as usize
    }
    fn all() -> &'static [Self] {
        &[Direction::Parent, Direction::Child]
    }
}

/// Both taxonomy directions as one dense-indexed, zero-copy CSR family, plus the
/// four reflexive-transitive is-a reachability queries computed per query over
/// them. All representation is the shared [`PackedCsrFamily`]; all reachability
/// is the shared graded-reach kernel ([`pr4xis::category::reach`]) over the
/// parent column.
pub struct TaxonomyStore(PackedCsrFamily<Direction, PodRun<ConceptId>>);

impl TaxonomyStore {
    /// Transcode the owned adjacency maps into the packed CSR family ONCE,
    /// consuming and freeing them. `concepts` is the dense id count (`0..concepts`
    /// are valid ids); both directions share it as their row count. Each
    /// concept's parent (and child) run is written in its owned-map order, so
    /// [`parents`](Self::parents) / [`children`](Self::children) return
    /// byte-identical slices to the owned fallback.
    pub fn build(
        parents: HashMap<ConceptId, Vec<ConceptId>>,
        children: HashMap<ConceptId, Vec<ConceptId>>,
        concepts: usize,
    ) -> Self {
        // Bundle in `Direction::ALL` order: [Parent, Child], both over `concepts`.
        let bundle = alloc::vec![(concepts, parents), (concepts, children)];
        Self(PackedCsrFamily::build(bundle))
    }

    /// Direct parents (hypernyms) of a concept, from the [`Direction::Parent`]
    /// column (empty slice if none or out of range).
    pub fn parents(&self, id: ConceptId) -> &[ConceptId] {
        self.0.column(Direction::Parent, id)
    }

    /// Direct children (hyponyms) of a concept, from the [`Direction::Child`]
    /// column (empty slice if none or out of range).
    pub fn children(&self, id: ConceptId) -> &[ConceptId] {
        self.0.column(Direction::Child, id)
    }

    /// Total number of parent (hypernym) edges.
    pub fn parent_edge_count(&self) -> Quantity {
        Quantity::from_unit(self.0.edge_count(Direction::Parent) as f64, &unit::UNITLESS)
    }

    /// The dense concept count (`0..concept_count` are valid ids).
    pub fn concept_count(&self) -> Quantity {
        Quantity::from_unit(self.0.row_count(Direction::Parent) as f64, &unit::UNITLESS)
    }

    // ── reachability surface (the ONE generic graded-reach engine) ────────────
    //
    // All four queries mint the shared engine's per-call `ReachView` over the
    // parent column (`pr4xis::category::reach`, Moore 1959) — the same engine
    // the runtime's `MaterializedClosure` instantiates, so the algorithm lives
    // ONCE. The memo POLICY is `Uncached`: nothing is stored, every query
    // re-walks the shallow DAG, and `TaxonomyStore` stays `Sync` (no interior
    // mutability — the runtime picks `Cached` instead). The kernel's
    // determinism contract is `(hops, V::Ord)`; `ConceptId`'s derived `Ord` IS
    // its `value()` order, so the pinned `(distance, ConceptId.value())`
    // orderings — including the `common_ancestor` "distance from `b`"
    // asymmetry and the DAG tie-break over multi-parent nodes — are UNCHANGED
    // (the full-corpus `english_taxonomy_bfs` oracle gate holds
    // byte-identically).

    /// The engine's view over the hypernym ascent — this substrate, the
    /// [`Uncached`] policy, the [`Direction::Parent`] column. MINTED PER CALL
    /// (a stored view would self-borrow); three references, free to build.
    fn ascent(&self) -> ReachView<'_, Self, Uncached> {
        ReachView::new(self, &Uncached, &Direction::Parent)
    }

    /// Does `child` is-a `ancestor` (reflexive-transitively)? — the engine's
    /// strict membership probe under the reflexive short-circuit. Cycle-safe.
    /// Verbatim the eager `ReachabilityClosure::reaches` semantics.
    pub fn is_a(&self, child: ConceptId, ancestor: ConceptId) -> bool {
        child == ancestor || self.ascent().reaches(&child, &ancestor)
    }

    /// The reflexive-transitive hypernym image of `id` — `id` itself (distance 0)
    /// plus every ancestor reachable up the taxonomy, ordered nearest-first by
    /// the kernel's canonical `(minimal is-a distance, ConceptId::Ord)` order
    /// (`ConceptId`'s derived `Ord` is its `value()` order).
    pub fn ancestors(&self, id: ConceptId) -> Vec<ConceptId> {
        self.ascent()
            .reflexive_image(&id)
            .into_iter()
            .map(|(v, _)| v)
            .collect()
    }

    /// The lowest common ancestor of `a` and `b` — the engine's lattice meet
    /// over the hypernym relation: the nearest vertex in
    /// `strict_ancestors(b) ∩ reflexive_ancestors(a)`, ranked by distance **from
    /// `b`** (nearest first), ties broken by the smaller `ConceptId` (its
    /// `value()` order). Verbatim the eager
    /// `ReachabilityClosure::meet_by(a, b, |id| id.value())`.
    pub fn common_ancestor(&self, a: ConceptId, b: ConceptId) -> Option<ConceptId> {
        self.ascent().meet(&a, &b)
    }

    /// The ordered hypernym chain `[child, …, ancestor]` (nearest-first, the
    /// kernel's `(distance, ConceptId::Ord)` order) when `child` is-a
    /// `ancestor`, else `None`. Verbatim the eager `ancestor_chain`.
    pub fn ancestor_chain(&self, child: ConceptId, ancestor: ConceptId) -> Option<Vec<ConceptId>> {
        self.ascent().chain(&child, &ancestor)
    }
}

/// The store-bundle serialization surface — the packed family bytes, the
/// per-column layout table the buffer does not carry, and the VALIDATING
/// re-entry for bytes this process did not pack. Archived (`prx` +
/// little-endian) only: the bundle serializes the packed representation.
#[cfg(all(feature = "prx", target_endian = "little"))]
impl TaxonomyStore {
    /// The packed family bytes (see
    /// [`PackedCsrFamily::as_bytes`](crate::formal::meta::packed_csr::ArchivedCsrFamily::as_bytes)).
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// The per-column `(row_count, edge_count)` table, in [`Direction`] layout
    /// order — carried in the bundle frame beside the bytes.
    pub fn col_layout(&self) -> Vec<(usize, usize)> {
        self.0.col_layout()
    }

    /// Recover a taxonomy store from UNTRUSTED packed bytes + the declared
    /// column table, through the fail-closed
    /// [`ArchivedCsrFamily::from_untrusted_buf`](crate::formal::meta::packed_csr::ArchivedCsrFamily::from_untrusted_buf)
    /// validation.
    pub fn from_untrusted_buf(
        buf: rkyv::util::AlignedVec<16>,
        cols: &[(usize, usize)],
    ) -> Result<Self, crate::formal::meta::packed_csr::PackedCsrError> {
        Ok(Self(PackedCsrFamily::from_untrusted_buf(buf, cols)?))
    }
}

impl ReachSubstrate for TaxonomyStore {
    type Kind = Direction;
    type Vertex = ConceptId;

    /// One labelled CSR column read, zero-copy: the [`Direction`] kind selects
    /// the column, the vertex indexes its run — empty for an out-of-range id.
    fn neighbors<'s>(
        &'s self,
        kind: &Direction,
        vertex: &ConceptId,
    ) -> impl Iterator<Item = ConceptId> + use<'s> {
        self.0.column(*kind, *vertex).iter().copied()
    }
}

impl core::fmt::Debug for TaxonomyStore {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TaxonomyStore")
            .field("concepts", &self.concept_count())
            .field("parent_edges", &self.parent_edge_count())
            .finish()
    }
}

// ── CSR family fixture test ──────────────────────────────────────────────────
//
// The generic zero-copy CSR laws prove the build + the `&[ConceptId]` cast +
// archived-equals-owned + per-label faithfulness once, generically. This fixture
// pins the TAXONOMY instance's concrete `parents`/`children` results on a small
// known adjacency (a multi-parent node, a root, a leaf, out-of-range ids).
#[cfg(all(test, feature = "prx", target_endian = "little"))]
mod fixture_tests {
    use super::*;

    fn cid(i: u64) -> ConceptId {
        ConceptId::new(i)
    }

    /// A small KNOWN 4-concept taxonomy.
    /// parents (child → parents):  0:[] (root)  1:[0]  2:[0,1] (multi)  3:[2]
    /// children (parent → kids):   0:[1,2]  1:[2]  2:[3]  3:[] (leaf)
    fn fixture() -> TaxonomyStore {
        let mut parents: HashMap<ConceptId, Vec<ConceptId>> = HashMap::new();
        parents.insert(cid(1), alloc::vec![cid(0)]);
        parents.insert(cid(2), alloc::vec![cid(0), cid(1)]);
        parents.insert(cid(3), alloc::vec![cid(2)]);
        let mut children: HashMap<ConceptId, Vec<ConceptId>> = HashMap::new();
        children.insert(cid(0), alloc::vec![cid(1), cid(2)]);
        children.insert(cid(1), alloc::vec![cid(2)]);
        children.insert(cid(2), alloc::vec![cid(3)]);
        TaxonomyStore::build(parents, children, 4)
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn csr_slices_match_the_known_adjacency() {
        let store = fixture();
        let empty: &[ConceptId] = &[];
        assert_eq!(store.parents(cid(0)), empty);
        assert_eq!(store.parents(cid(1)), &[cid(0)]);
        // Multi-parent node — order preserved through the cast.
        assert_eq!(store.parents(cid(2)), &[cid(0), cid(1)]);
        assert_eq!(store.parents(cid(3)), &[cid(2)]);
        assert_eq!(store.children(cid(0)), &[cid(1), cid(2)]);
        assert_eq!(store.children(cid(1)), &[cid(2)]);
        assert_eq!(store.children(cid(2)), &[cid(3)]);
        assert_eq!(store.children(cid(3)), empty);
        // Out-of-range id → empty slice, both directions.
        assert_eq!(store.parents(cid(99)), empty);
        assert_eq!(store.children(cid(99)), empty);
        assert_eq!(store.concept_count().value, 4.0);
        assert_eq!(store.parent_edge_count().value, 4.0); // 0 + 1 + 2 + 1
    }

    /// The reachability BFS reads the generic CSR: is-a, ancestors, LCA, chain.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn reachability_reads_the_generic_csr() {
        let store = fixture();
        assert!(store.is_a(cid(3), cid(0)));
        assert!(store.is_a(cid(2), cid(0)));
        assert!(!store.is_a(cid(0), cid(3)));
        assert!(store.is_a(cid(1), cid(1))); // reflexive
        // 3's reflexive ancestors, nearest-first: [3, 2, 0, 1] (0 and 1 both at
        // distance 2 from 3 via 2; tie broken by smaller id ⇒ 0 before 1).
        assert_eq!(
            store.ancestors(cid(3)),
            alloc::vec![cid(3), cid(2), cid(0), cid(1)]
        );
        // strict_anc(3) ∩ refl_anc(1) = {0,1} at distance 2 from 3; tie → smaller id.
        assert_eq!(store.common_ancestor(cid(1), cid(3)), Some(cid(0)));
        // Every reflexive ancestor of 3 that itself reaches 0 lies on a 3⇝0 path:
        // 3→2→1→0 and 3→2→0, so the evidence set is {3,2,0,1}, ordered by
        // (distance from 3, id): [3, 2, 0, 1].
        assert_eq!(
            store.ancestor_chain(cid(3), cid(0)),
            Some(alloc::vec![cid(3), cid(2), cid(0), cid(1)])
        );
    }
}
