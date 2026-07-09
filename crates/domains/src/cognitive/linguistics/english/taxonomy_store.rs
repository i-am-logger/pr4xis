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
//! (Moore 1959 / Floyd 1962) — at a per-query cost indistinguishable from the
//! O(1) closure lookup, while dropping the ~697k-pair closure entirely. A
//! per-query BFS over immutable edges needs **no interior mutability**, so
//! `TaxonomyStore` stays `Sync` inside `English`'s `OnceLock`.
//!
//! # The representation
//!
//! Both hypernym directions (child→parents, parent→children) are ONE instance of
//! the shared
//! [`PackedCsrFamily`](crate::formal::meta::packed_csr::PackedCsrFamily): a
//! [`DenseId`](crate::formal::meta::packed_csr::DenseId)-indexed family of
//! [`PodRun`](crate::formal::meta::packed_csr::PodRun)`<ConceptId>` columns,
//! labelled by [`Direction`]. Because a [`ConceptId`] is the *dense* synset index
//! `0..N`, each column is indexed directly by the id — no hashing. The zero-copy
//! `&[ConceptId]` cast, the CSR reader, the little-endian invariant, and the
//! owned adjacency fallback all live in that one hand-audited generic;
//! [`parents`](TaxonomyStore::parents) / [`children`](TaxonomyStore::children)
//! are the two labelled columns, and the four reachability queries read them.

use alloc::collections::VecDeque;
use alloc::vec::Vec;

use hashbrown::{HashMap, HashSet};

use super::ontology::ConceptId;
use crate::formal::meta::packed_csr::{LabelKind, PackedCsrFamily, PodRun};

/// The two directions of the hypernym (Subsumption) relation — the family label.
///
/// Literature: SKOS `broader` / `narrower` (Miles & Bechhofer, *SKOS Reference*,
/// W3C REC-skos-reference-20090818, §8.6.1–8.6.2) — the hierarchical link and its
/// inverse. `Parent` (a concept's hypernyms) is the SKOS-`broader` direction;
/// `Child` (its hyponyms) is SKOS-`narrower`. Equivalently OBO Relation Ontology
/// `is_a` and its inverse (Smith et al. 2005).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
/// them. All representation is the shared [`PackedCsrFamily`]; the reachability
/// BFS is the store's own domain logic.
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
    pub fn parent_edge_count(&self) -> usize {
        self.0.edge_count(Direction::Parent)
    }

    /// The dense concept count (`0..concept_count` are valid ids).
    pub fn concept_count(&self) -> usize {
        self.0.row_count(Direction::Parent)
    }

    // ── reachability surface (the store's own domain logic) ──────────────────
    //
    // The bounded breadth-first ascent reproduces the eager `ReachabilityClosure`'s
    // answers exactly: unit-weight BFS grades each node at its minimal hop count,
    // and the `(distance, ConceptId.value())` orderings are applied verbatim —
    // including the `common_ancestor` "distance from `b`" asymmetry and the DAG
    // tie-break over multi-parent nodes.

    /// Does `child` is-a `ancestor` (reflexive-transitively)? — a bounded
    /// breadth-first ascent over the parent edges. Reflexive and cycle-safe.
    /// Verbatim the eager `ReachabilityClosure::reaches` semantics.
    pub fn is_a(&self, child: ConceptId, ancestor: ConceptId) -> bool {
        if child.value() == ancestor.value() {
            return true;
        }
        let mut seen: HashSet<u64> = HashSet::new();
        seen.insert(child.value());
        let mut queue: VecDeque<ConceptId> = VecDeque::new();
        queue.push_back(child);
        while let Some(vertex) = queue.pop_front() {
            for &parent in self.parents(vertex) {
                if parent.value() == ancestor.value() {
                    return true;
                }
                if seen.insert(parent.value()) {
                    queue.push_back(parent);
                }
            }
        }
        false
    }

    /// The reflexive-transitive hypernym image of `id` — `id` itself (distance 0)
    /// plus every ancestor reachable up the taxonomy, ordered nearest-first by
    /// `(minimal is-a distance, ConceptId.value())`.
    pub fn ancestors(&self, id: ConceptId) -> Vec<ConceptId> {
        let mut image = self.reflexive_ancestors(id);
        image.sort_unstable_by(|(a, da), (b, db)| {
            da.cmp(db).then_with(|| a.value().cmp(&b.value()))
        });
        image.into_iter().map(|(v, _)| v).collect()
    }

    /// The lowest common ancestor of `a` and `b` — the lattice meet over the
    /// hypernym relation: the nearest vertex in
    /// `strict_ancestors(b) ∩ reflexive_ancestors(a)`, ranked by distance **from
    /// `b`** (nearest first), ties broken by the smaller `ConceptId.value()`.
    /// Verbatim the eager `ReachabilityClosure::meet_by(a, b, |id| id.value())`.
    pub fn common_ancestor(&self, a: ConceptId, b: ConceptId) -> Option<ConceptId> {
        let anc_a: HashSet<u64> = self
            .reflexive_ancestors(a)
            .into_iter()
            .map(|(v, _)| v.value())
            .collect();
        self.strict_ancestors(b)
            .into_iter()
            .filter(|(v, _)| anc_a.contains(&v.value()))
            .min_by(|(v1, d1), (v2, d2)| d1.cmp(d2).then_with(|| v1.value().cmp(&v2.value())))
            .map(|(v, _)| v)
    }

    /// The ordered hypernym chain `[child, …, ancestor]` (nearest-first) when
    /// `child` is-a `ancestor`, else `None`. Verbatim the eager `ancestor_chain`.
    pub fn ancestor_chain(&self, child: ConceptId, ancestor: ConceptId) -> Option<Vec<ConceptId>> {
        if !self.is_a(child, ancestor) {
            return None;
        }
        let mut chain: Vec<(ConceptId, u32)> = self
            .reflexive_ancestors(child)
            .into_iter()
            .filter(|(x, _)| self.is_a(*x, ancestor))
            .collect();
        chain.sort_unstable_by(|(a, da), (b, db)| {
            da.cmp(db).then_with(|| a.value().cmp(&b.value()))
        });
        Some(chain.into_iter().map(|(v, _)| v).collect())
    }

    /// The STRICT reachable ancestor image of `source` — every strict ancestor
    /// (excluding `source`), each paired with its minimal hop count. A cycle-safe
    /// breadth-first ascent, verbatim the eager `strict_image`, minus the memo.
    fn strict_ancestors(&self, source: ConceptId) -> Vec<(ConceptId, u32)> {
        let mut image: Vec<(ConceptId, u32)> = Vec::new();
        let mut seen: HashSet<u64> = HashSet::new();
        seen.insert(source.value());
        let mut queue: VecDeque<(ConceptId, u32)> = VecDeque::new();
        queue.push_back((source, 0));
        while let Some((vertex, hops)) = queue.pop_front() {
            for &parent in self.parents(vertex) {
                if seen.insert(parent.value()) {
                    image.push((parent, hops + 1));
                    queue.push_back((parent, hops + 1));
                }
            }
        }
        image
    }

    /// The REFLEXIVE reachable ancestor image of `source` — `source` at hop 0 plus
    /// its [`strict_ancestors`](Self::strict_ancestors).
    fn reflexive_ancestors(&self, source: ConceptId) -> Vec<(ConceptId, u32)> {
        let mut out = alloc::vec![(source, 0u32)];
        out.extend(self.strict_ancestors(source));
        out
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
        assert_eq!(store.concept_count(), 4);
        assert_eq!(store.parent_edge_count(), 4); // 0 + 1 + 2 + 1
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
