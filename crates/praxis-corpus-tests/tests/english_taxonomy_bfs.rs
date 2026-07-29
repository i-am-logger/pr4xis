//! Full-corpus equivalence gate for the "English IS a RuntimeOntology" taxonomy
//! convergence: English's per-query **bounded breadth-first ascent** over the
//! archived hypernym edges (`TaxonomyStore`) must reproduce the eager
//! `ReachabilityClosure` answer EXACTLY over the real Open English WordNet corpus
//! — for all four methods (`is_a`, `ancestors`, `common_ancestor`,
//! `ancestor_chain`), and critically over the **multi-parent (DAG) nodes** where a
//! tree-shaped sample can never witness a tie-break divergence.
//!
//! The eager `ReachabilityClosure` (a Floyd fixpoint fold, the shipping semantics
//! before this convergence) is the independent oracle: it is folded here from the
//! very same parent edges the BFS ascends, so the two disagree only if the BFS is
//! wrong. This is the hard gate the task requires before the eager closure could
//! be deleted from `English`.

use pr4xis::category::quiver::ReachabilityClosure;
use pr4xis_domains::cognitive::linguistics::english::{ConceptId, english_loaded};

/// The eager `ancestor_chain` reference: the reflexive ancestors of `child` that
/// still reach `ancestor`, ordered by `(distance from child, ConceptId.value())`.
/// Verbatim `English::ancestor_chain`'s pre-convergence body, over the oracle
/// closure.
fn eager_chain(
    closure: &ReachabilityClosure<ConceptId>,
    child: ConceptId,
    ancestor: ConceptId,
) -> Vec<ConceptId> {
    let mut chain: Vec<(ConceptId, u32)> = closure
        .reflexive_image(&child)
        .into_iter()
        .filter(|(x, _)| closure.reaches(x, &ancestor))
        .collect();
    chain.sort_unstable_by(|(a, da), (b, db)| da.cmp(db).then_with(|| a.value().cmp(&b.value())));
    chain.into_iter().map(|(v, _)| v).collect()
}

#[test]
fn bfs_taxonomy_matches_eager_closure_over_full_corpus() {
    // The real loaded English (compact `.prx` fast path; HARD-FAILs if the corpus
    // is not provisioned — no skip).
    let en = english_loaded();
    let n = en.concept_count().value as usize;
    assert!(
        n > 100_000,
        "the real corpus must be loaded; got {n} concepts"
    );

    // Independent oracle: fold the eager reflexive-transitive closure from the SAME
    // parent edges the per-query BFS ascends. Two different algorithms (Floyd
    // fixpoint vs breadth-first ascent) over one edge set — they agree iff the BFS
    // reproduces the closure.
    let mut edges: Vec<(ConceptId, ConceptId)> = Vec::new();
    for i in 0..n {
        let c = ConceptId::new(i as u64);
        for &p in en.parents(c) {
            edges.push((c, p));
        }
    }
    let closure = ReachabilityClosure::fold(edges.iter().copied());

    // The multi-parent (DAG) nodes — where a tie-break CAN occur and a tree-shaped
    // sample would silently pass.
    let dag_nodes: Vec<ConceptId> = (0..n)
        .map(|i| ConceptId::new(i as u64))
        .filter(|&c| en.parents(c).len() >= 2)
        .collect();
    assert!(
        dag_nodes.len() > 1_000,
        "the corpus is a real DAG; expected thousands of multi-parent nodes, got {}",
        dag_nodes.len()
    );

    // (1) is_a + ancestors, over EVERY concept: the BFS reflexive image, ordered by
    // (distance, ConceptId.value()), must equal the eager image; is_a must hold for
    // every ancestor in that image.
    for i in 0..n {
        let c = ConceptId::new(i as u64);
        let mut want = closure.reflexive_image(&c);
        want.sort_unstable_by(|(a, da), (b, db)| {
            da.cmp(db).then_with(|| a.value().cmp(&b.value()))
        });
        let want_ids: Vec<ConceptId> = want.iter().map(|(v, _)| *v).collect();
        assert_eq!(
            en.ancestors(c),
            want_ids,
            "ancestors mismatch at concept {i}"
        );
        for (a, _) in &want {
            assert!(en.is_a(c, *a), "is_a({i}, {}) must hold", a.value());
        }
    }

    // (2) common_ancestor + ancestor_chain over the DAG nodes — the tie surface.
    // `common_ancestor(m, m)` exercises m's OWN equidistant parents (a guaranteed
    // per-node tie); `ancestor_chain(m, a)` over each strict ancestor a exercises
    // the chain's equidistant-mid ordering. Both must match the oracle's
    // ConceptId.value() tie-break exactly.
    for &m in &dag_nodes {
        assert_eq!(
            en.common_ancestor(m, m),
            closure.meet_by(&m, &m, |id| id.value()),
            "common_ancestor tie mismatch at DAG node {}",
            m.value()
        );
        for (a, _) in closure.strict_image(&m) {
            assert_eq!(
                en.ancestor_chain(m, a),
                Some(eager_chain(&closure, m, a)),
                "ancestor_chain mismatch {} -> {}",
                m.value(),
                a.value()
            );
        }
    }

    // (3) a broad, deterministic ordered-pair sweep across the whole corpus for
    // common_ancestor + ancestor_chain — catches any non-DAG regression too.
    let step = 7_919usize; // prime stride
    let mut a_idx = 0usize;
    for k in 0..50_000usize {
        a_idx = (a_idx + step) % n;
        let b_idx = (a_idx.wrapping_mul(3).wrapping_add(k)) % n;
        let a = ConceptId::new(a_idx as u64);
        let b = ConceptId::new(b_idx as u64);
        assert_eq!(
            en.common_ancestor(a, b),
            closure.meet_by(&a, &b, |id| id.value()),
            "common_ancestor sweep mismatch at ({a_idx}, {b_idx})"
        );
        if en.is_a(a, b) {
            assert_eq!(
                en.ancestor_chain(a, b),
                Some(eager_chain(&closure, a, b)),
                "ancestor_chain sweep mismatch at ({a_idx}, {b_idx})"
            );
        } else {
            assert_eq!(
                en.ancestor_chain(a, b),
                None,
                "ancestor_chain must be None when !is_a at ({a_idx}, {b_idx})"
            );
        }
    }

    eprintln!(
        "BFS≡EAGER PASS: {n} concepts, {} DAG nodes, {} taxonomy edges — all four \
         methods match the eager closure",
        dag_nodes.len(),
        edges.len()
    );
}
