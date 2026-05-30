//! Classification fibration encoding — view a concept lattice as a
//! fibration over the Classification ontology's `Taxon` hierarchy.
//!
//! # Categorical setting
//!
//! Per Grothendieck (1971) *SGA 1* §VI.2 and Jacobs (1999) Ch. 1, a
//! **fibration** `p: E → B` is a functor whose fibers `E_b = p^{-1}(b)`
//! are categories themselves and whose morphisms admit *cartesian
//! lifts* along the base. The construction here treats:
//!
//! - **Base category** `B` = the `ClassificationCategory` (Taxa, Ranks,
//!   Differentiae from Linnaeus (1735), Aristotle, Porphyry, Guarino
//!   (2009)).
//! - **Total category** `E` = the `ConceptLattice` (from the sibling
//!   `fca` module) of a
//!   formal context, viewed as a thin category whose objects are
//!   formal concepts and whose only morphisms are the extent-
//!   inclusion order.
//! - **Projection** `p` = [`ConceptLatticeFibration::rank_of`]: assigns
//!   every concept a `ClassificationConcept` rank based on its
//!   extent's size relative to the lattice's total height.
//!
//! Two concepts in the same fiber share the same Linnaean rank —
//! e.g. the seven concepts at "Species" rank are the leaves of the
//! lattice's Hasse diagram. The cartesian lift is the natural one:
//! moving from a concept `c` along an upper-cover edge in the lattice
//! corresponds to moving from one Taxon to its SubordinateTo-related
//! parent in the Classification ontology.
//!
//! # Why this encoding
//!
//! FCA gives you a lattice; downstream praxis ontologies that reason
//! about *kinds* expect a taxonomy (Taxon / Genus / Species / …). The
//! fibration is the *categorical bridge* between the two views:
//!
//! - The concept lattice's bottom-to-top structure (extent ⊆) maps to
//!   the Linnaean Species-to-Kingdom subordination chain.
//! - The number of concepts at each Hasse-diagram level determines
//!   which existing rank concept (Species through Kingdom) is the
//!   most natural classifier — finer-grained levels get
//!   Species/Genus, coarser get Family/Order/Class, and so on.
//!
//! # Literature
//!
//! - **Grothendieck, A. (1971)** "Catégories fibrées et descente",
//!   *Séminaire de Géométrie Algébrique du Bois-Marie* (SGA 1), §VI.2.
//! - **Jacobs, B. (1999)** *Categorical Logic and Type Theory*,
//!   Elsevier — Ch. 1 fibrations and indexed categories; Ch. 5
//!   indexed categories ↔ Grothendieck construction.
//! - **Linnaeus, C. (1735)** *Systema Naturae*, 1st ed., Leiden —
//!   seven ranks (Species/Genus/Family/Order/Class/Phylum/Kingdom).
//! - **Wille, R. (1982)** "Restructuring Lattice Theory" — the
//!   concept lattice that's the total category here.
//! - **Ganter, B. & Wille, R. (1999)** *Formal Concept Analysis:
//!   Mathematical Foundations*, Springer — Theorem 3 and §3
//!   sub-lattice / fibration discussion.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::Axiom;

use super::fca::{ConceptLattice, FormalContext};
use crate::formal::classification::ontology::ClassificationConcept;

/// A concept lattice viewed as a fibration over the Classification
/// ontology. Each concept is assigned a Linnaean rank derived from
/// its position in the Hasse diagram — concepts on the longest
/// extent-inclusion chain receive ranks from Species (bottom) to
/// Kingdom (top).
#[derive(Debug, Clone)]
pub struct ConceptLatticeFibration<O, A> {
    /// The underlying lattice. Stays as a peer rather than being
    /// consumed so callers can still query the original concepts.
    pub lattice: ConceptLattice<O, A>,
    /// Per-concept rank assignment. `ranks[i]` is the
    /// `ClassificationConcept::*` (Species / Genus / Family / Order /
    /// Class / Phylum / Kingdom) for `lattice.concepts[i]`.
    pub ranks: Vec<ClassificationConcept>,
}

impl<O, A> ConceptLatticeFibration<O, A> {
    /// Build the fibration. Computes each concept's depth in the
    /// Hasse diagram (its height above the bottom), then bins depths
    /// linearly onto the seven Linnaean ranks. Ereshefsky (2001)
    /// criticism noted: the binning is *pragmatic*, not biological —
    /// the ranks here are FCA-derived classificatory levels, not
    /// natural kinds.
    #[must_use]
    pub fn from_lattice(lattice: ConceptLattice<O, A>) -> Self {
        let depths = compute_depths(&lattice);
        let max_depth = depths.iter().copied().max().unwrap_or(0);
        let ranks: Vec<ClassificationConcept> = depths
            .iter()
            .map(|&d| rank_for_depth(d, max_depth))
            .collect();
        Self { lattice, ranks }
    }

    /// Build directly from a formal context.
    #[must_use]
    pub fn from_context(ctx: &FormalContext<O, A>) -> Self
    where
        O: Clone,
        A: Clone,
    {
        Self::from_lattice(ctx.build_lattice())
    }

    /// Project a concept index to its Linnaean rank — the fibration
    /// projection `p: E → B`. Returns `None` only when the index is
    /// out of range.
    #[must_use]
    pub fn rank_of(&self, concept_idx: usize) -> Option<ClassificationConcept> {
        self.ranks.get(concept_idx).copied()
    }

    /// All concept indices in the fiber over a given rank.
    #[must_use]
    pub fn fiber(&self, rank: ClassificationConcept) -> Vec<usize> {
        self.ranks
            .iter()
            .enumerate()
            .filter_map(|(i, &r)| (r == rank).then_some(i))
            .collect()
    }

    /// All ranks present in the fibration. Useful for asking "what
    /// taxonomic levels did this corpus generate?".
    #[must_use]
    pub fn populated_ranks(&self) -> Vec<ClassificationConcept> {
        let mut out: Vec<_> = self.ranks.to_vec();
        out.sort_by_key(|c| linnaean_order(*c));
        out.dedup();
        out
    }
}

/// Topological depth of each concept: 0 for the bottom (minimum
/// extent), increasing along the Hasse diagram toward the top
/// (maximum extent). Computed by Kahn-style longest-path BFS in
/// `O(n + |edges|)`.
fn compute_depths<O, A>(lattice: &ConceptLattice<O, A>) -> Vec<usize> {
    let n = lattice.concepts.len();
    let mut depths = vec![0usize; n];
    if n == 0 {
        return depths;
    }
    // Sort by extent size ascending — the bottom concept has fewest
    // objects, the top has all. Iterate in that order, and for each
    // concept set its depth to 1 + max depth of its lower covers.
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by_key(|&i| lattice.concepts[i].extent.len());
    for i in order {
        let max_lc = lattice
            .lower_covers(i)
            .into_iter()
            .map(|j| depths[j])
            .max()
            .unwrap_or(0);
        depths[i] = if lattice.lower_covers(i).is_empty() {
            0
        } else {
            max_lc + 1
        };
    }
    depths
}

/// Linnaean-rank ordering: Species (lowest) through Kingdom
/// (highest). Other classification concepts get rank 0 / max+1.
fn linnaean_order(c: ClassificationConcept) -> u8 {
    use ClassificationConcept as C;
    match c {
        C::Species => 1,
        C::Genus => 2,
        C::Family => 3,
        C::Order => 4,
        C::Class => 5,
        C::Phylum => 6,
        C::Kingdom => 7,
        _ => 0,
    }
}

/// Map a Hasse-diagram depth `d` (relative to a lattice of maximum
/// depth `max_depth`) onto a Linnaean rank. Uses a linear binning:
/// the seven ranks divide `[0, max_depth]` into equal-width slices.
/// At `max_depth < 7` we use the lowest `max_depth + 1` ranks;
/// at `max_depth ≥ 7` we spread depths across all seven.
fn rank_for_depth(depth: usize, max_depth: usize) -> ClassificationConcept {
    use ClassificationConcept as C;
    const RANKS: [ClassificationConcept; 7] = [
        C::Species,
        C::Genus,
        C::Family,
        C::Order,
        C::Class,
        C::Phylum,
        C::Kingdom,
    ];
    if max_depth == 0 {
        // Trivial lattice: only the top concept exists.
        return C::Kingdom;
    }
    // Map depth ∈ [0, max_depth] to an index in [0, 7).
    // For max_depth < 7 we use one rank per depth (Species first);
    // for larger max_depth we squash via integer division.
    let usable_ranks = (max_depth + 1).min(7);
    if max_depth < 7 {
        return RANKS[depth.min(usable_ranks - 1)];
    }
    // depth / (max_depth / 7) maps [0, max_depth] to [0, 7] inclusive.
    // Clamp to [0, 6].
    let bucket = (depth * 7) / (max_depth + 1);
    RANKS[bucket.min(6)]
}

// =============================================================================
// Domain axioms — invariants that any classification fibration over a
// concept lattice must satisfy.
// =============================================================================

fn small_context_for_axioms() -> FormalContext<&'static str, &'static str> {
    // Reuse the canonical Ganter-Wille test context (small, hand-
    // verifiable lattice).
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

/// Axiom (Grothendieck 1971 §VI.2): the fibration's projection is
/// total — every concept in the lattice receives a rank.
pub struct ProjectionIsTotal;

impl Axiom for ProjectionIsTotal {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ctx = small_context_for_axioms();
        let fib = ConceptLatticeFibration::from_context(&ctx);
        if fib.ranks.len() == fib.lattice.concepts.len()
            && (0..fib.lattice.concepts.len()).all(|i| fib.rank_of(i).is_some())
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ProjectionIsTotal",
        "every concept in the lattice receives a Linnaean-rank assignment",
        "Grothendieck (1971) SGA 1 §VI.2; Jacobs (1999) Categorical Logic and Type Theory Ch. 1"
    );
}

/// Axiom (Linnaean monotonicity): the fibration is *monotone* — if
/// concept `i` is a sub-concept of `j` in the Hasse diagram, then
/// `rank_of(i) ≤ rank_of(j)` in the Linnaean order (Species ≤
/// Kingdom). Verified by walking the Hasse edges.
pub struct FibrationIsLinnaeanMonotone;

impl Axiom for FibrationIsLinnaeanMonotone {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ctx = small_context_for_axioms();
        let fib = ConceptLatticeFibration::from_context(&ctx);
        for &(i, j) in &fib.lattice.order_edges {
            let ri = fib.rank_of(i).expect("rank present");
            let rj = fib.rank_of(j).expect("rank present");
            if linnaean_order(ri) > linnaean_order(rj) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "FibrationIsLinnaeanMonotone",
        "ranks are monotone along Hasse edges: sub-concept rank <= super-concept rank",
        "Linnaeus (1735) Systema Naturae; Jacobs (1999) Categorical Logic and Type Theory Ch. 1 (cartesian lifts respect the base order)"
    );
}

/// Axiom: the populated ranks form a contiguous prefix of the
/// Linnaean order — gaps would mean the binning lost a level.
pub struct PopulatedRanksAreContiguous;

impl Axiom for PopulatedRanksAreContiguous {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ctx = small_context_for_axioms();
        let fib = ConceptLatticeFibration::from_context(&ctx);
        let mut orders: Vec<u8> = fib
            .populated_ranks()
            .into_iter()
            .map(linnaean_order)
            .collect();
        orders.sort_unstable();
        for w in orders.windows(2) {
            if w[1] != w[0] + 1 && w[1] != w[0] {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "PopulatedRanksAreContiguous",
        "the set of populated ranks forms a contiguous slice of the Linnaean order",
        "Linnaeus (1735) Systema Naturae; modern systematics (ICZN / ICNafp)"
    );
}

pr4xis::register_axiom!(
    ProjectionIsTotal,
    "Grothendieck (1971) SGA 1 §VI.2; Jacobs (1999) Categorical Logic and Type Theory Ch. 1"
);
pr4xis::register_axiom!(
    FibrationIsLinnaeanMonotone,
    "Linnaeus (1735) Systema Naturae; Jacobs (1999) Ch. 1 (cartesian lifts respect base order)"
);
pr4xis::register_axiom!(
    PopulatedRanksAreContiguous,
    "Linnaeus (1735) Systema Naturae; modern systematics (ICZN / ICNafp)"
);

#[cfg(test)]
#[path = "classification_fibration_tests.rs"]
mod tests;
