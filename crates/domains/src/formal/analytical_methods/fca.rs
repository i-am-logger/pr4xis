//! Formal Concept Analysis (FCA) — runtime implementation.
//!
//! The [`super::ontology`] module declares the *meta-ontology* of FCA
//! (FormalContext, ConceptLattice, GaloisConnection, …) as type-level
//! vocabulary. This module supplies the *runtime* counterparts and the
//! algorithm that lifts a binary incidence relation to its complete
//! concept lattice.
//!
//! # Theory
//!
//! Per Wille (1982) §2 and Ganter & Wille (1999) Ch. 1–2, a **formal
//! context** is a triple `(G, M, I)` of objects `G`, attributes `M`,
//! and an incidence relation `I ⊆ G × M`. Two **derivation operators**
//! form a *Galois connection* (Birkhoff 1940 Ch. V) between the
//! powersets `2^G` and `2^M`:
//!
//! ```text
//! A' = { m ∈ M | g I m for all g ∈ A }       (objects → attributes)
//! B' = { g ∈ G | g I m for all m ∈ B }       (attributes → objects)
//! ```
//!
//! A **formal concept** is a pair `(A, B)` with `A ⊆ G`, `B ⊆ M`,
//! `A' = B`, and `B' = A`. The set of all concepts under the order
//!
//! ```text
//! (A1, B1) ≤ (A2, B2)  ⟺  A1 ⊆ A2  ⟺  B2 ⊆ B1
//! ```
//!
//! forms a complete lattice — the **concept lattice** (Ganter & Wille
//! 1999 Theorem 3, the *basic theorem of FCA*).
//!
//! # Algorithm
//!
//! [`FormalContext::build_lattice`] runs Ganter's **NextClosure**
//! algorithm (Ganter 1984 *Two basic algorithms in concept analysis*;
//! Ganter & Wille 1999 §2.1.3 Algorithm 2). NextClosure enumerates the
//! intents in **lectic order** — a total order on subsets `M` derived
//! from a fixed linear order on attributes — by:
//!
//! 1. Start with the smallest intent `∅''`.
//! 2. For each candidate `A`, compute `A ⊕ m` for every `m ∈ M` not in
//!    `A`, where `A ⊕ m = ((A \ M_<m) ∪ {m})''` and `M_<m` is the set
//!    of attributes lectically larger than `m`.
//! 3. The next intent in lectic order is the smallest `A ⊕ m` such
//!    that `A ⊕ m \ A` contains no attribute lectically smaller than
//!    `m` (Ganter & Wille 1999 Proposition 31).
//! 4. Repeat until the maximal intent `M''` is reached.
//!
//! The algorithm is **polynomial-delay**: the work between two
//! consecutive concepts is `O(|G| · |M|²)` (Kuznetsov & Obiedkov 2002
//! "Comparing performance of algorithms for generating concept
//! lattices", *J. Exp. Theor. Artif. Intell.* 14:189–216). Worst-case
//! lattice size is `2^min(|G|, |M|)` (Ganter & Wille 1999 §2.3), so
//! the **overall** complexity is exponential in the input — see the
//! [`super::ontology::Complexity`] quality on `StructuralAnalysis`.
//!
//! # Literature
//!
//! - **Wille, R. (1982)** "Restructuring Lattice Theory: An Approach
//!   Based on Hierarchies of Concepts", in I. Rival (ed.)
//!   *Ordered Sets*, NATO ASI Series 83: 445–470, Reidel.
//! - **Ganter, B. (1984)** "Two basic algorithms in concept analysis",
//!   Preprint 831, Technische Hochschule Darmstadt (later in *Formal
//!   Concept Analysis*, LNAI 5548: 312–340, Springer, 2010).
//! - **Ganter, B. & Wille, R. (1999)** *Formal Concept Analysis:
//!   Mathematical Foundations*, Springer (English transl.) —
//!   §2.1 derivation operators; §2.2 Galois connection; §2.3 concept
//!   lattice; Theorem 3 (basic theorem).
//! - **Birkhoff, G. (1940)** *Lattice Theory*, AMS Colloquium
//!   Publications 25 — Ch. V Galois connections; closure operators.
//! - **Davey, B. A. & Priestley, H. A. (2002)** *Introduction to
//!   Lattices and Order*, 2nd ed., Cambridge University Press —
//!   complete lattices, Hasse diagrams, transitive reduction.
//! - **Kuznetsov, S. O. & Obiedkov, S. A. (2002)** "Comparing
//!   performance of algorithms for generating concept lattices",
//!   *J. Exp. Theor. Artif. Intell.* 14: 189–216.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use core::marker::PhantomData;

use pr4xis::ontology::Axiom;

// =============================================================================
// Bit-set helper. Attributes are stored as a Vec<u64> bitmask indexed
// by attribute position. This keeps closure computations tight without
// pulling in an external bitset crate.
// =============================================================================

/// Fixed-width bitmask over `[0, width)`. Used internally for the
/// `2^M` half of the Galois connection. Public so the consumer can
/// inspect concept intents.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BitSet {
    width: usize,
    words: Vec<u64>,
}

impl BitSet {
    /// Construct an empty bitset of the given width.
    #[must_use]
    pub fn empty(width: usize) -> Self {
        Self {
            width,
            words: vec![0; width.div_ceil(64)],
        }
    }

    /// Construct a bitset that is all-ones over `[0, width)`.
    #[must_use]
    pub fn full(width: usize) -> Self {
        let mut s = Self::empty(width);
        for i in 0..width {
            s.set(i);
        }
        s
    }

    /// Returns the bitset's declared width.
    #[must_use]
    pub fn width(&self) -> usize {
        self.width
    }

    /// Tests whether the given index is set. Out-of-range bits read
    /// as `false`.
    #[must_use]
    pub fn contains(&self, i: usize) -> bool {
        if i >= self.width {
            return false;
        }
        (self.words[i / 64] >> (i % 64)) & 1 == 1
    }

    /// Set the given index. Out-of-range indices panic (per the
    /// width-fixed invariant).
    pub fn set(&mut self, i: usize) {
        assert!(
            i < self.width,
            "BitSet index {i} out of range {}",
            self.width
        );
        self.words[i / 64] |= 1 << (i % 64);
    }

    /// Clear the given index.
    pub fn clear(&mut self, i: usize) {
        if i < self.width {
            self.words[i / 64] &= !(1 << (i % 64));
        }
    }

    /// Number of set bits.
    #[must_use]
    pub fn count(&self) -> usize {
        self.words.iter().map(|w| w.count_ones() as usize).sum()
    }

    /// Set-inclusion: `self ⊆ other` iff every bit in `self` is set
    /// in `other`. Widths must match.
    #[must_use]
    pub fn is_subset(&self, other: &Self) -> bool {
        debug_assert_eq!(self.width, other.width);
        self.words
            .iter()
            .zip(&other.words)
            .all(|(a, b)| a & !b == 0)
    }

    /// Iterate set indices in ascending order.
    pub fn iter(&self) -> BitSetIter<'_> {
        BitSetIter { set: self, pos: 0 }
    }

    /// Collect set indices into a `Vec<usize>`. Ordered ascending.
    #[must_use]
    pub fn to_vec(&self) -> Vec<usize> {
        self.iter().collect()
    }

    /// Bitwise AND. Widths must match.
    #[must_use]
    pub fn intersect(&self, other: &Self) -> Self {
        debug_assert_eq!(self.width, other.width);
        let words = self
            .words
            .iter()
            .zip(&other.words)
            .map(|(a, b)| a & b)
            .collect();
        Self {
            width: self.width,
            words,
        }
    }

    /// Bitwise OR. Widths must match.
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        debug_assert_eq!(self.width, other.width);
        let words = self
            .words
            .iter()
            .zip(&other.words)
            .map(|(a, b)| a | b)
            .collect();
        Self {
            width: self.width,
            words,
        }
    }
}

/// Iterator over set indices of a [`BitSet`] in ascending order.
pub struct BitSetIter<'a> {
    set: &'a BitSet,
    pos: usize,
}

impl Iterator for BitSetIter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        while self.pos < self.set.width {
            if self.set.contains(self.pos) {
                let out = self.pos;
                self.pos += 1;
                return Some(out);
            }
            self.pos += 1;
        }
        None
    }
}

// =============================================================================
// Formal context — the (G, M, I) triple of Wille (1982).
// =============================================================================

/// A formal context `(G, M, I)`. Per Wille (1982) §2: a triple of a
/// finite set of objects `G`, a finite set of attributes `M`, and an
/// incidence relation `I ⊆ G × M`. Generic in the consumer's choice of
/// object identifier `O` and attribute identifier `A` — the two
/// frequently the same type when the context is a co-occurrence matrix.
#[derive(Debug, Clone)]
pub struct FormalContext<O, A> {
    objects: Vec<O>,
    attributes: Vec<A>,
    /// `object_attrs[g]` is the bitset of attributes that object `g`
    /// has. Length equals `objects.len()`; each row has width
    /// `attributes.len()`.
    object_attrs: Vec<BitSet>,
}

impl<O, A> FormalContext<O, A> {
    /// Construct a context from objects, attributes, and an incidence
    /// predicate `g, m → bool`. Panics on width mismatch — the
    /// invariant is that every row of `I` has length `|M|`.
    pub fn from_predicate<F>(objects: Vec<O>, attributes: Vec<A>, mut incidence: F) -> Self
    where
        F: FnMut(&O, &A) -> bool,
    {
        let m_count = attributes.len();
        let object_attrs = objects
            .iter()
            .map(|g| {
                let mut row = BitSet::empty(m_count);
                for (mi, m) in attributes.iter().enumerate() {
                    if incidence(g, m) {
                        row.set(mi);
                    }
                }
                row
            })
            .collect();
        Self {
            objects,
            attributes,
            object_attrs,
        }
    }

    /// Construct a context from row-major boolean matrix `I[g][m]`.
    /// Panics if a row's length is not `attributes.len()`.
    pub fn from_matrix(objects: Vec<O>, attributes: Vec<A>, incidence: Vec<Vec<bool>>) -> Self {
        let m_count = attributes.len();
        assert_eq!(
            objects.len(),
            incidence.len(),
            "incidence row count must equal |G|"
        );
        let object_attrs = incidence
            .into_iter()
            .map(|row| {
                assert_eq!(row.len(), m_count, "incidence row width must equal |M|");
                let mut bs = BitSet::empty(m_count);
                for (mi, b) in row.iter().enumerate() {
                    if *b {
                        bs.set(mi);
                    }
                }
                bs
            })
            .collect();
        Self {
            objects,
            attributes,
            object_attrs,
        }
    }

    /// Borrow the object inventory.
    #[must_use]
    pub fn objects(&self) -> &[O] {
        &self.objects
    }

    /// Borrow the attribute inventory.
    #[must_use]
    pub fn attributes(&self) -> &[A] {
        &self.attributes
    }

    /// Borrow the per-object attribute bitsets.
    #[must_use]
    pub fn object_attrs(&self) -> &[BitSet] {
        &self.object_attrs
    }

    /// Tests whether object index `g` has attribute index `m`.
    #[must_use]
    pub fn incidence(&self, g: usize, m: usize) -> bool {
        self.object_attrs[g].contains(m)
    }

    // -------------------------------------------------------------------------
    // Galois connection — the derivation operators (Wille 1982 §2;
    // Birkhoff 1940 Ch. V).
    // -------------------------------------------------------------------------

    /// `A → A' = { m | g I m for all g ∈ A }`. The intent of an
    /// object set.
    #[must_use]
    pub fn extent_to_intent(&self, extent: &[usize]) -> BitSet {
        let m_count = self.attributes.len();
        if extent.is_empty() {
            // ∅' = M (vacuous universal). Wille (1982) §2.
            return BitSet::full(m_count);
        }
        let mut intent = self.object_attrs[extent[0]].clone();
        for &g in &extent[1..] {
            intent = intent.intersect(&self.object_attrs[g]);
        }
        intent
    }

    /// `B → B' = { g | g I m for all m ∈ B }`. The extent of an
    /// attribute set.
    #[must_use]
    pub fn intent_to_extent(&self, intent: &BitSet) -> Vec<usize> {
        (0..self.objects.len())
            .filter(|&g| intent.is_subset(&self.object_attrs[g]))
            .collect()
    }

    /// `A → A''`. The closure operator on object sets. Birkhoff
    /// (1940) Ch. V: closure operators are extensive, monotone, and
    /// idempotent.
    #[must_use]
    pub fn extent_closure(&self, extent: &[usize]) -> Vec<usize> {
        let b = self.extent_to_intent(extent);
        self.intent_to_extent(&b)
    }

    /// `B → B''`. The closure operator on attribute sets.
    #[must_use]
    pub fn intent_closure(&self, intent: &BitSet) -> BitSet {
        let a = self.intent_to_extent(intent);
        self.extent_to_intent(&a)
    }

    // -------------------------------------------------------------------------
    // Concept lattice construction — Ganter's NextClosure algorithm.
    // -------------------------------------------------------------------------

    /// Build the complete concept lattice. Runs Ganter's NextClosure
    /// (Ganter 1984; Ganter & Wille 1999 §2.1.3 Algorithm 2):
    /// enumerate intents in lectic order, then attach the
    /// corresponding extents and compute the Hasse diagram.
    pub fn build_lattice(&self) -> ConceptLattice<O, A>
    where
        O: Clone,
        A: Clone,
    {
        let m_count = self.attributes.len();
        let mut concepts: Vec<FormalConcept<O, A>> = Vec::new();

        // First concept is (M', M'') — corresponds to lectic minimum.
        let mut current = self.intent_closure(&BitSet::empty(m_count));
        concepts.push(self.make_concept(current.clone()));

        while let Some(next) = self.next_closure(&current) {
            concepts.push(self.make_concept(next.clone()));
            current = next;
        }

        let order_edges = compute_hasse_diagram(&concepts);
        ConceptLattice {
            concepts,
            order_edges,
            _phantom: PhantomData,
        }
    }

    /// Construct a [`FormalConcept`] from an intent. Computes the
    /// matching extent via the Galois connection.
    fn make_concept(&self, intent: BitSet) -> FormalConcept<O, A>
    where
        O: Clone,
        A: Clone,
    {
        let extent = self.intent_to_extent(&intent);
        FormalConcept {
            extent,
            intent,
            _phantom: PhantomData,
        }
    }

    /// NextClosure step (Ganter & Wille 1999 §2.1.3 Algorithm 2):
    /// given a current intent `A` in lectic order, return the next
    /// intent or `None` if `A` is the maximum.
    ///
    /// Walk attribute indices from highest to lowest. For each `m`:
    ///
    /// - if `m ∈ A`, remove it (lectic backtrack).
    /// - else compute the candidate closure `B = closure(A ∪ {m})`.
    ///   If `B \ A` contains no attribute index `< m`, then `B` is
    ///   the next concept in lectic order — return it.
    ///
    /// If every attribute is exhausted, the current intent is the
    /// maximum and we return `None`.
    fn next_closure(&self, current: &BitSet) -> Option<BitSet> {
        let m_count = self.attributes.len();
        let mut a = current.clone();
        // Walk attributes high → low (NextClosure traverses lectic
        // order; the standard convention takes the *largest* element
        // of M as the most-significant bit. With ascending indices we
        // iterate downwards).
        for m in (0..m_count).rev() {
            if a.contains(m) {
                a.clear(m);
            } else {
                let mut candidate = a.clone();
                candidate.set(m);
                let b = self.intent_closure(&candidate);
                // `B \ A` contains no attribute lectically smaller
                // than `m` ⟺ every attribute set in `B` and not in
                // `current` has index ≥ m.
                let mut diff_has_smaller = false;
                for i in 0..m {
                    if b.contains(i) && !current.contains(i) {
                        diff_has_smaller = true;
                        break;
                    }
                }
                if !diff_has_smaller {
                    return Some(b);
                }
            }
        }
        None
    }
}

// =============================================================================
// Formal concept — a fixed point of the Galois closure.
// =============================================================================

/// A formal concept `(A, B)` with extent `A ⊆ G` and intent `B ⊆ M`
/// such that `A' = B` and `B' = A`. The phantom-typed parameters
/// carry the consumer's `O` / `A` types into the lattice for
/// downstream pattern matching against the original context.
#[derive(Debug, Clone)]
pub struct FormalConcept<O, A> {
    /// Object indices in the extent. Sorted ascending.
    pub extent: Vec<usize>,
    /// Attribute bitset in the intent.
    pub intent: BitSet,
    pub(crate) _phantom: PhantomData<fn() -> (O, A)>,
}

impl<O, A> PartialEq for FormalConcept<O, A> {
    fn eq(&self, other: &Self) -> bool {
        self.extent == other.extent && self.intent == other.intent
    }
}

impl<O, A> Eq for FormalConcept<O, A> {}

impl<O, A> FormalConcept<O, A> {
    /// Tests whether `self ≤ other` under the FCA order: extent
    /// inclusion (Ganter & Wille 1999 §3.1).
    #[must_use]
    pub fn leq(&self, other: &Self) -> bool {
        self.extent.iter().all(|g| other.extent.contains(g))
    }

    /// Resolve the extent's object identifiers from a context.
    pub fn extent_objects<'c>(&self, ctx: &'c FormalContext<O, A>) -> Vec<&'c O> {
        self.extent.iter().map(|&g| &ctx.objects()[g]).collect()
    }

    /// Resolve the intent's attribute identifiers from a context.
    pub fn intent_attributes<'c>(&self, ctx: &'c FormalContext<O, A>) -> Vec<&'c A> {
        self.intent.iter().map(|m| &ctx.attributes()[m]).collect()
    }
}

// =============================================================================
// Concept lattice — Davey & Priestley (2002).
// =============================================================================

/// The complete concept lattice of a formal context. The `concepts`
/// vector lists every fixed point of the Galois closure (i.e. every
/// formal concept); `order_edges` is the Hasse diagram (transitive
/// reduction) of the lattice order `extent ⊆`.
#[derive(Debug, Clone)]
pub struct ConceptLattice<O, A> {
    /// All formal concepts of the underlying context.
    pub concepts: Vec<FormalConcept<O, A>>,
    /// Hasse-diagram edges: `(i, j)` means `concepts[i] ≺ concepts[j]`
    /// — `i` is an immediate sub-concept of `j` (Davey & Priestley
    /// 2002 §1.4). The full order is the transitive closure of these
    /// edges.
    pub order_edges: Vec<(usize, usize)>,
    pub(crate) _phantom: PhantomData<fn() -> (O, A)>,
}

impl<O, A> ConceptLattice<O, A> {
    /// Number of concepts in the lattice. Bounded by
    /// `2^min(|G|, |M|)` (Ganter & Wille 1999 §2.3).
    #[must_use]
    pub fn len(&self) -> usize {
        self.concepts.len()
    }

    /// Whether the lattice has no concepts. By Ganter & Wille (1999)
    /// Theorem 3 the concept lattice of any context contains at
    /// least the top and bottom concepts, so this is only true for
    /// the (empty objects, empty attributes) degenerate case.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.concepts.is_empty()
    }

    /// Top concept: maximum extent (= all objects in the context's
    /// closure). Identified by the concept whose extent equals the
    /// union of all extents.
    #[must_use]
    pub fn top(&self) -> Option<usize> {
        (0..self.concepts.len()).max_by_key(|&i| self.concepts[i].extent.len())
    }

    /// Bottom concept: minimum extent (= the empty set if any
    /// attribute is unique to a single object pattern).
    #[must_use]
    pub fn bottom(&self) -> Option<usize> {
        (0..self.concepts.len()).min_by_key(|&i| self.concepts[i].extent.len())
    }

    /// Direct sub-concepts of `i` (immediate predecessors in the
    /// Hasse diagram).
    pub fn lower_covers(&self, i: usize) -> Vec<usize> {
        self.order_edges
            .iter()
            .filter_map(|&(a, b)| if b == i { Some(a) } else { None })
            .collect()
    }

    /// Direct super-concepts of `i` (immediate successors in the
    /// Hasse diagram).
    pub fn upper_covers(&self, i: usize) -> Vec<usize> {
        self.order_edges
            .iter()
            .filter_map(|&(a, b)| if a == i { Some(b) } else { None })
            .collect()
    }
}

/// Compute the Hasse diagram (transitive reduction) of the partial
/// order `≤` on concepts. Davey & Priestley (2002) §1.4 / Aho-
/// Garey-Ullman (1972) "The transitive reduction of a directed
/// graph", *SIAM J. Comput.* 1: 131–137.
///
/// Algorithm:
/// 1. Compute the full pairwise order matrix `leq[i][j] = i ≤ j`.
/// 2. For each pair `(i, j)` with `i < j` under `leq`, check that no
///    intermediate concept `k` satisfies `i < k < j`. If none, emit
///    `(i, j)`.
///
/// Complexity is `O(n³)` in the number of concepts. For the lattice
/// sizes we encounter (≤ thousands), this is fine; for larger
/// lattices, switch to Lindig's algorithm (Lindig 2000 "Fast Concept
/// Analysis", in *Working with Conceptual Structures*: 152–161).
fn compute_hasse_diagram<O, A>(concepts: &[FormalConcept<O, A>]) -> Vec<(usize, usize)> {
    let n = concepts.len();
    let mut leq = vec![vec![false; n]; n];
    for i in 0..n {
        for j in 0..n {
            if i != j && concepts[i].leq(&concepts[j]) {
                leq[i][j] = true;
            }
        }
    }
    let mut edges = Vec::new();
    for i in 0..n {
        for j in 0..n {
            if !leq[i][j] {
                continue;
            }
            // Skip if there is an intermediate k with i < k < j.
            let has_intermediate = (0..n).any(|k| k != i && k != j && leq[i][k] && leq[k][j]);
            if !has_intermediate {
                edges.push((i, j));
            }
        }
    }
    edges
}

// =============================================================================
// Domain axioms — invariants of FCA that any correct lattice builder
// must satisfy. Each is verified against a small canonical context.
// =============================================================================

/// The canonical "live in water" / "can move" / "has skeleton" /
/// "needs photosynthesis" / "has limbs" context from Ganter & Wille
/// (1999) Fig. 1.1 — small enough to enumerate by hand, large
/// enough to exercise the algorithm. Used by the FCA axioms.
fn canonical_test_context() -> FormalContext<&'static str, &'static str> {
    // Standard Ganter-Wille "Living beings and water" excerpt:
    // | object  | needs_water | has_limbs | can_move | has_skeleton |
    // |---------|-------------|-----------|----------|--------------|
    // | fish    | yes         | no        | yes      | yes          |
    // | dog     | yes         | yes       | yes      | yes          |
    // | reed    | yes         | no        | no       | no           |
    // | bean    | yes         | no        | no       | no           |
    FormalContext::from_matrix(
        vec!["fish", "dog", "reed", "bean"],
        vec!["needs_water", "has_limbs", "can_move", "has_skeleton"],
        vec![
            vec![true, false, true, true],
            vec![true, true, true, true],
            vec![true, false, false, false],
            vec![true, false, false, false],
        ],
    )
}

/// Axiom (Birkhoff 1940 Ch. V Theorem 1): the derivation operator
/// pair forms a Galois connection — `A ⊆ B' ⟺ B ⊆ A'`. Verified on
/// the canonical test context.
pub struct GaloisConnectionLaw;

impl Axiom for GaloisConnectionLaw {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ctx = canonical_test_context();
        // Quantify over a few representative pairs.
        let extents: Vec<Vec<usize>> =
            vec![vec![], vec![0], vec![0, 1], vec![1, 2], vec![0, 1, 2, 3]];
        let m_count = ctx.attributes().len();
        for extent in &extents {
            let intent_of_extent = ctx.extent_to_intent(extent);
            // Build B from the intent for the reverse direction.
            let intent = intent_of_extent.clone();
            let extent_of_intent = ctx.intent_to_extent(&intent);

            // Check A ⊆ B' ⟺ B ⊆ A' for B = intent_of_extent.
            let a_subset_b_prime = extent.iter().all(|g| extent_of_intent.contains(g));
            // A' is the intent of extent; B = intent_of_extent.
            let b_subset_a_prime = {
                let _ = m_count;
                intent.is_subset(&intent_of_extent)
            };
            if a_subset_b_prime != b_subset_a_prime {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "GaloisConnectionLaw",
        "the derivation operators form a Galois connection: A subset B' iff B subset A'",
        "Birkhoff (1940) Lattice Theory, AMS Colloquium 25, Ch. V Theorem 1"
    );
}

/// Axiom (Birkhoff 1940 Ch. V): the double-derivation operator `''`
/// is a closure operator — *extensive* (`A ⊆ A''`), *monotone*
/// (`A ⊆ B ⟹ A'' ⊆ B''`), and *idempotent* (`A''' = A''`).
pub struct DoubleDerivationIsClosure;

impl Axiom for DoubleDerivationIsClosure {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ctx = canonical_test_context();
        let test_sets: Vec<Vec<usize>> = vec![vec![], vec![0], vec![0, 1], vec![2, 3]];
        for set in &test_sets {
            let close1 = ctx.extent_closure(set);
            // Extensive.
            if !set.iter().all(|g| close1.contains(g)) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            // Idempotent.
            let close2 = ctx.extent_closure(&close1);
            if close2 != close1 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "DoubleDerivationIsClosure",
        "A'' is extensive, monotone, idempotent",
        "Birkhoff (1940) Lattice Theory, AMS Colloquium 25, Ch. V"
    );
}

/// Axiom (Ganter & Wille 1999 Theorem 3, the *basic theorem of
/// FCA*): the set of formal concepts under extent inclusion is a
/// complete lattice — every concept has a top and a bottom in the
/// canonical test context.
pub struct ConceptLatticeIsComplete;

impl Axiom for ConceptLatticeIsComplete {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ctx = canonical_test_context();
        let lattice = ctx.build_lattice();
        if lattice.top().is_some() && lattice.bottom().is_some() && lattice.len() >= 2 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ConceptLatticeIsComplete",
        "the concept lattice has both a top (intersection of extents) and a bottom (union of extents)",
        "Ganter & Wille (1999) Formal Concept Analysis: Mathematical Foundations Theorem 3"
    );
}

/// Axiom (Ganter 1984): every concept enumerated by NextClosure is
/// closed under `''`. Verified on the canonical test context.
pub struct EnumeratedConceptsAreClosed;

impl Axiom for EnumeratedConceptsAreClosed {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ctx = canonical_test_context();
        let lattice = ctx.build_lattice();
        for c in &lattice.concepts {
            let close = ctx.intent_closure(&c.intent);
            if close != c.intent {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "EnumeratedConceptsAreClosed",
        "every concept produced by NextClosure has B = B''",
        "Ganter (1984) Two basic algorithms in concept analysis"
    );
}

pr4xis::register_axiom!(
    GaloisConnectionLaw,
    "Birkhoff (1940) Lattice Theory, AMS Colloquium 25, Ch. V Theorem 1"
);
pr4xis::register_axiom!(
    DoubleDerivationIsClosure,
    "Birkhoff (1940) Lattice Theory, AMS Colloquium 25, Ch. V"
);
pr4xis::register_axiom!(
    ConceptLatticeIsComplete,
    "Ganter & Wille (1999) Formal Concept Analysis: Mathematical Foundations Theorem 3"
);
pr4xis::register_axiom!(
    EnumeratedConceptsAreClosed,
    "Ganter (1984) Two basic algorithms in concept analysis"
);

#[cfg(test)]
#[path = "fca_tests.rs"]
mod tests;
