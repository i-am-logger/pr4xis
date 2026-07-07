//! The English hypernym taxonomy edges as an immutable, zero-copy CSR — and the
//! reflexive-transitive is-a reachability queries computed *per query* over them.
//!
//! `English` answers four reachability questions against WordNet's hypernym
//! (Subsumption) DAG — [`is_a`](TaxonomyStore::is_a),
//! [`ancestors`](TaxonomyStore::ancestors),
//! [`common_ancestor`](TaxonomyStore::common_ancestor) and
//! [`ancestor_chain`](TaxonomyStore::ancestor_chain). Historically these read a
//! pre-folded reflexive-transitive closure (`HashMap<Cid, HashMap<Cid, u32>>`,
//! ~697k `(child, ancestor)` pairs) that was materialized eagerly at load and
//! retained for the process lifetime, next to the two owned adjacency maps
//! (`taxonomy_parents` / `taxonomy_children`). Together those three structures
//! were tens of MiB of resident heap.
//!
//! # Why a per-query BFS replaces the eager closure
//!
//! Open English WordNet's hypernym relation is a **shallow** DAG: the maximum
//! is-a depth is 16 and the largest reflexive ancestor set is 33 nodes. A single
//! reachability query therefore visits only a few tens of nodes. A bounded
//! breadth-first ascent over the direct parent edges reproduces the closure's
//! answer *exactly* — BFS on unit-weight edges grades every node at its minimal
//! hop count (Moore 1959), which is the same shortest-path length the eager fold
//! carries (Floyd 1962) — at a per-query cost (~0.3 µs, ~7 node visits) that is
//! indistinguishable from the O(1) closure lookup on the bounded query counts
//! English's consumers issue, while dropping the ~697k-pair closure entirely.
//!
//! Crucially, a per-query BFS over immutable edges needs **no interior
//! mutability** — no `RefCell` memo — so `TaxonomyStore` (hence `English`) stays
//! `Sync` and remains valid inside its process-wide `OnceLock<English>` static.
//! (`pr4xis-runtime`'s lazily-memoized `MaterializedClosure` is `!Sync` precisely
//! because of its `RefCell` memo; the shallow taxonomy lets us drop the memo and
//! keep both the reclaim and `Sync`.)
//!
//! # The representation
//!
//! Because a [`ConceptId`] is the **dense** synset index `0..N` (assigned as
//! `ConceptId::new(idx)` at
//! [`from_wordnet`](super::ontology::English::from_wordnet) time), the edges are
//! held as a plain **CSR** (compressed sparse row) indexed *directly* by the
//! id — no hashing. Under `prx` on a little-endian target both directions are
//! transcoded ONCE, at load, into a single [`AlignedVec<16>`]:
//!
//! ```text
//! [ 0..16)                 header:  n:u32, p_edges:u32, c_edges:u32, _pad:u32
//! [16 ..)                  p_targets:  p_edges × 8   packed little-endian ConceptId
//! then c_edges × 8         c_targets:  the children edges, same packing
//! then (n+1) × 4           p_offsets:  CSR offsets into p_targets (ConceptId units)
//! then (n+1) × 4           c_offsets:  CSR offsets into c_targets (ConceptId units)
//! ```
//!
//! The two `*_targets` arrays lead the body so they inherit the buffer's
//! 16-alignment (hence 8-alignment), which lets [`parents`](TaxonomyStore::parents)
//! / [`children`](TaxonomyStore::children) hand back a `&[ConceptId]` slice with a
//! zero-copy cast — exactly the [`word_index`](super::word_index) `ids_at`
//! discipline. The owned `HashMap` adjacency (the two maps English used to hold)
//! is consumed by [`TaxonomyStore::build`] and dropped; only the buffer survives —
//! a REPLACEMENT of the owned build, not an addition on top of it. Each concept's
//! parent (and child) run is copied in its owned-map order, so `parents(id)` /
//! `children(id)` return byte-identical slices to the owned fallback.
//!
//! # Endianness invariant (why the archived variant is `little`-only)
//!
//! The zero-copy cast reinterprets packed id bytes as `&[ConceptId]`, where
//! [`ConceptId`] = `Ref<4> { value: u64 }` is a single-`u64` POD. The cast is
//! sound only where the machine's native integer byte order equals the
//! little-endian order the ids were written with. wasm32 and x86-64 — the two
//! targets praxis ships — are both little-endian. The whole zero-copy variant is
//! therefore compiled only under `cfg(target_endian = "little")`; a (hypothetical)
//! big-endian target falls back to the owned `HashMap`s, exactly as a non-`prx`
//! build does.
//!
//! Reference: Hill, D. *rkyv: zero-copy deserialization framework for Rust* (v0.8)
//! — `AlignedVec` is rkyv's own little-endian aligned buffer; this module reuses
//! its aligned-buffer discipline for the taxonomy CSR. See
//! <https://github.com/rkyv/rkyv>.

use alloc::collections::VecDeque;
use alloc::vec::Vec;

use hashbrown::{HashMap, HashSet};

use super::ontology::ConceptId;

/// The zero-copy, `prx`-gated taxonomy store (little-endian targets).
#[cfg(all(feature = "prx", target_endian = "little"))]
pub use archived::TaxonomyStore;

/// The owned fallback taxonomy store (no `prx`, or a big-endian target).
#[cfg(not(all(feature = "prx", target_endian = "little")))]
pub use owned::TaxonomyStore;

// ── shared reachability surface ──────────────────────────────────────────────
//
// The four is-a reachability queries, written ONCE against the common
// `parents` / `children` API so they are identical for both representations. The
// bounded breadth-first ascent reproduces the eager `ReachabilityClosure`'s
// answers exactly: unit-weight BFS grades each node at its minimal hop count,
// which is the shortest-path length the closure's Floyd fold carries, and the
// `(distance, ConceptId.value())` orderings are applied verbatim — including the
// `common_ancestor` "distance from `b`" asymmetry and the DAG tie-break over
// multi-parent nodes.

impl TaxonomyStore {
    /// Does `child` is-a `ancestor` (reflexive-transitively)? — a bounded
    /// breadth-first ascent over the parent edges. Reflexive (`child == ancestor`
    /// holds) and cycle-safe (`seen` set, keyed by the dense id). Verbatim the
    /// eager `ReachabilityClosure::reaches` semantics
    /// (`source == target || target ∈ strict_image(source)`).
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
    /// `(minimal is-a distance, ConceptId.value())`. Verbatim the eager
    /// `ancestors`: `reflexive_image` sorted by `(distance, value)`.
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
    /// Verbatim the eager `ReachabilityClosure::meet_by(a, b, |id| id.value())`:
    /// `b`'s image is strict (a common ancestor sits strictly above `b`), `a`'s is
    /// reflexive (so `a` counts when it is itself an ancestor of `b`). The
    /// `(distance, value)` key is a total order (ids are unique), so the argmin is
    /// deterministic regardless of visitation order.
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
    /// `child` is-a `ancestor`, else `None`. The is-a evidence path: exactly those
    /// reflexive ancestors `x` of `child` that themselves reach `ancestor` (so `x`
    /// lies on a `child ⇝ ancestor` path), ordered by `(distance from child,
    /// ConceptId.value())`. Verbatim the eager `ancestor_chain`
    /// (`reflexive_image(child)` filtered by `reaches(x, ancestor)`, sorted by
    /// `(distance, value)`).
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
    /// breadth-first ascent (`seen` set → first enqueue is the minimal hop for
    /// unit-weight edges), verbatim `pr4xis-runtime`'s `compute_image` and the
    /// eager `ReachabilityClosure::strict_image`, minus the memo.
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
    /// its [`strict_ancestors`](Self::strict_ancestors). Verbatim the eager
    /// `ReachabilityClosure::reflexive_image`.
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

// ── owned fallback ───────────────────────────────────────────────────────────

/// The owned fallback: the two plain adjacency `HashMap`s English used to hold
/// directly. Kept as the mandatory non-`prx` (and big-endian) path, mirroring the
/// [`concept_store`](super::concept_store) / [`word_index`](super::word_index)
/// splits.
#[cfg(not(all(feature = "prx", target_endian = "little")))]
mod owned {
    use super::*;

    /// Child → parents and parent → children adjacency, held as owned maps.
    pub struct TaxonomyStore {
        parents: HashMap<ConceptId, Vec<ConceptId>>,
        children: HashMap<ConceptId, Vec<ConceptId>>,
        concepts: usize,
    }

    impl TaxonomyStore {
        /// Retain the owned adjacency maps as-is — the fallback keeps the
        /// `HashMap`s. `concepts` is the dense id count (`0..concepts`).
        pub fn build(
            parents: HashMap<ConceptId, Vec<ConceptId>>,
            children: HashMap<ConceptId, Vec<ConceptId>>,
            concepts: usize,
        ) -> Self {
            Self {
                parents,
                children,
                concepts,
            }
        }

        /// Direct parents (hypernyms) of a concept (empty slice if none).
        pub fn parents(&self, id: ConceptId) -> &[ConceptId] {
            self.parents.get(&id).map(|v| v.as_slice()).unwrap_or(&[])
        }

        /// Direct children (hyponyms) of a concept (empty slice if none).
        pub fn children(&self, id: ConceptId) -> &[ConceptId] {
            self.children.get(&id).map(|v| v.as_slice()).unwrap_or(&[])
        }

        /// Total number of parent (hypernym) edges.
        pub fn parent_edge_count(&self) -> usize {
            self.parents.values().map(|v| v.len()).sum()
        }

        /// The dense concept count (`0..concept_count` are valid ids).
        pub fn concept_count(&self) -> usize {
            self.concepts
        }
    }
}

// ── archived variant ─────────────────────────────────────────────────────────

/// The zero-copy archived store: both taxonomy directions in one CSR
/// [`AlignedVec<16>`], indexed directly by the dense [`ConceptId`], read back
/// through a zero-copy id-slice cast.
#[cfg(all(feature = "prx", target_endian = "little"))]
mod archived {
    use rkyv::util::AlignedVec;

    use super::*;

    /// Byte length of the fixed header (`n`, `p_edges`, `c_edges`, pad — four
    /// `u32`s), and the offset at which `p_targets` begins. Sized to keep the
    /// buffer's 16-alignment (⇒ 8-alignment) on the `*_targets` arrays.
    const HEADER: usize = 16;

    /// The soundness precondition of the zero-copy id cast: native integer byte
    /// order must equal the little-endian order ids are stored in. Enforced by the
    /// `cfg(target_endian = "little")` gate on this module; asserted here so a
    /// mis-configuration is a compile error, not a silent miscast.
    const _: () = assert!(cfg!(target_endian = "little"));

    /// `TaxonomyStore` is `Sync`: its only fields are an [`AlignedVec`] (which rkyv
    /// declares `Send + Sync`) plus `Copy` scalars — no interior mutability (the
    /// per-query BFS carries its own stack-local `VecDeque` + `HashSet`). This is
    /// what keeps `English`'s process-wide `OnceLock<English>` static valid without
    /// a memo.
    const _: fn() = || {
        fn assert_sync<T: Sync>() {}
        assert_sync::<TaxonomyStore>();
    };

    /// Both taxonomy directions, as one packed, dense-indexed, zero-copy CSR.
    ///
    /// See the [module docs](super) for the buffer layout and the endianness
    /// invariant.
    pub struct TaxonomyStore {
        /// The whole CSR: header + p_targets + c_targets + p_offsets + c_offsets.
        buf: AlignedVec<16>,
        /// Dense concept count (`= header.n`); the number of CSR rows.
        n: usize,
        /// Number of parent (hypernym) edges (`= header.p_edges`).
        p_edges: usize,
        /// Byte offset of the `c_targets` id array.
        c_targets_at: usize,
        /// Byte offset of the `p_offsets` CSR array.
        p_offsets_at: usize,
        /// Byte offset of the `c_offsets` CSR array.
        c_offsets_at: usize,
    }

    impl TaxonomyStore {
        /// Transcode the owned adjacency maps into the packed CSR ONCE, consuming
        /// and freeing them. Each concept's parent (and child) run is written in
        /// its owned-map order, so [`parents`](Self::parents) /
        /// [`children`](Self::children) return byte-identical slices to the owned
        /// fallback.
        pub fn build(
            parents: HashMap<ConceptId, Vec<ConceptId>>,
            children: HashMap<ConceptId, Vec<ConceptId>>,
            concepts: usize,
        ) -> Self {
            let n = concepts;

            let run = |map: &HashMap<ConceptId, Vec<ConceptId>>, i: usize| -> usize {
                map.get(&ConceptId::new(i as u64)).map_or(0, Vec::len)
            };
            let p_edges: usize = (0..n).map(|i| run(&parents, i)).sum();
            let c_edges: usize = (0..n).map(|i| run(&children, i)).sum();

            let p_targets_at = HEADER;
            let c_targets_at = p_targets_at + p_edges * 8;
            let p_offsets_at = c_targets_at + c_edges * 8;
            let c_offsets_at = p_offsets_at + (n + 1) * 4;
            let total = c_offsets_at + (n + 1) * 4;

            let mut buf = AlignedVec::<16>::with_capacity(total);

            // Header.
            buf.extend_from_slice(&(n as u32).to_le_bytes());
            buf.extend_from_slice(&(p_edges as u32).to_le_bytes());
            buf.extend_from_slice(&(c_edges as u32).to_le_bytes());
            buf.extend_from_slice(&0u32.to_le_bytes());

            // p_targets / c_targets: every concept's run, in dense-id order, each
            // id a little-endian u64 (`ConceptId`'s in-memory representation).
            let write_targets =
                |buf: &mut AlignedVec<16>, map: &HashMap<ConceptId, Vec<ConceptId>>| {
                    for i in 0..n {
                        if let Some(v) = map.get(&ConceptId::new(i as u64)) {
                            for id in v {
                                buf.extend_from_slice(&id.value().to_le_bytes());
                            }
                        }
                    }
                };
            write_targets(&mut buf, &parents);
            write_targets(&mut buf, &children);

            // p_offsets / c_offsets: CSR prefix sums over the per-id run lengths.
            let write_offsets =
                |buf: &mut AlignedVec<16>, map: &HashMap<ConceptId, Vec<ConceptId>>| {
                    let mut acc = 0u32;
                    buf.extend_from_slice(&acc.to_le_bytes());
                    for i in 0..n {
                        acc += run(map, i) as u32;
                        buf.extend_from_slice(&acc.to_le_bytes());
                    }
                };
            write_offsets(&mut buf, &parents);
            write_offsets(&mut buf, &children);

            assert_eq!(
                buf.len(),
                total,
                "taxonomy CSR buffer length must equal the computed layout size"
            );

            Self {
                buf,
                n,
                p_edges,
                c_targets_at,
                p_offsets_at,
                c_offsets_at,
            }
            // `parents` / `children` (and the ids they own) drop here — the owned
            // build is freed; only `buf` survives.
        }

        /// Read the `i`-th little-endian `u32` of the CSR array at byte offset
        /// `base`. A checked 4-byte read (no alignment requirement on the offset
        /// arrays; only the `*_targets` arrays are alignment-load-bearing).
        #[inline]
        fn csr(&self, base: usize, i: usize) -> usize {
            let at = base + i * 4;
            let b = &self.buf.as_slice()[at..at + 4];
            u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize
        }

        /// The run `targets[start..end]` of an id array beginning at byte
        /// `targets_at`, cast zero-copy to `&[ConceptId]`.
        #[inline]
        fn run(&self, targets_at: usize, offsets_at: usize, i: usize) -> &[ConceptId] {
            let start = self.csr(offsets_at, i);
            let end = self.csr(offsets_at, i + 1);
            let byte_start = targets_at + start * 8;
            let len = end - start;
            // SAFETY: the `*_targets` arrays begin at a multiple of 8 (`p_targets`
            // at HEADER = 16; `c_targets` at 16 + p_edges*8), inheriting the
            // buffer's 16-alignment (⇒ 8-aligned), so `byte_start = targets_at +
            // start*8` is 8-aligned — the alignment `ConceptId` (a single `u64`)
            // requires. The `len` ConceptIds at `byte_start` are in bounds by
            // construction (the CSR offsets partition exactly the edge count). Each
            // was written as a little-endian `u64`; on this little-endian-gated
            // build that IS `ConceptId`'s representation, so the reinterpretation is
            // a value-preserving zero-copy view. The returned slice borrows `self`.
            unsafe {
                let ptr = self.buf.as_ptr().add(byte_start) as *const ConceptId;
                core::slice::from_raw_parts(ptr, len)
            }
        }

        /// Direct parents (hypernyms) of a concept, cast zero-copy from the CSR
        /// (empty slice if `id` has no parents or is out of range).
        pub fn parents(&self, id: ConceptId) -> &[ConceptId] {
            let i = id.value() as usize;
            if i >= self.n {
                return &[];
            }
            self.run(HEADER, self.p_offsets_at, i)
        }

        /// Direct children (hyponyms) of a concept, cast zero-copy from the CSR
        /// (empty slice if `id` has no children or is out of range).
        pub fn children(&self, id: ConceptId) -> &[ConceptId] {
            let i = id.value() as usize;
            if i >= self.n {
                return &[];
            }
            self.run(self.c_targets_at, self.c_offsets_at, i)
        }

        /// Total number of parent (hypernym) edges.
        pub fn parent_edge_count(&self) -> usize {
            self.p_edges
        }

        /// The dense concept count (`0..concept_count` are valid ids).
        pub fn concept_count(&self) -> usize {
            self.n
        }
    }
}
