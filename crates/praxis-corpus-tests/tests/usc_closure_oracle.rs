//! Full-corpus equivalence gate for the runtime's `MaterializedClosure` — the
//! u32-CSR lazy engine — against the INDEPENDENT eager
//! [`ReachabilityClosure`] oracle, over a real loaded USC title at full scale.
//!
//! This is the runtime sibling of `english_taxonomy_bfs.rs`: the riskiest
//! engine (the closure every loaded `.prx` is reasoned over) gets the same
//! teeth English's `TaxonomyStore` has. The oracle is the eager Floyd-fixpoint
//! fold in `pr4xis::category::quiver` — deliberately NOT delegated to the
//! shared `reach` kernel (see the note in `pr4xis::category::reach`), so the
//! two engines share only the edge set, never the algorithm. The edge set is
//! read off the ARCHIVED buffer via `morphisms_from` (the archive read path),
//! not off the closure's own adjacency, so a fold bug cannot certify itself.
//!
//! Checked over EVERY node of the title: the graded Parthood image (set AND
//! minimal hop count AND the canonical `(hops, ConceptRef::Ord)` order), the
//! reachable set, the membership probe, the evidence chain to every ancestor
//! (plus the reflexive chain), the per-node self-meet, and a deterministic
//! ordered-pair sweep for meet + chain across the whole title. The u32 vertex
//! ids are an internal representation; every mismatch here would be a broken
//! id↔ConceptRef order isomorphism or a broken CSR row.

use std::collections::BTreeSet;

use pr4xis::category::quiver::ReachabilityClosure;
use pr4xis::ontology::meta::OntologyName;
use pr4xis_domains::social::software::markup::xml::uslm::UsCode;
use pr4xis_domains::social::software::markup::xml::uslm::corpus::bridge::usc_runtime_ontology;
use pr4xis_runtime::ontology::{
    ConceptRef, MaterializedClosure, RuntimeEdge, relations_kind, subsumption_kind,
};
use praxis_corpus_tests::{load_uslm_corpus, require};

/// The eager `chain` reference — the reflexive ancestors of `child` that still
/// (reflexively) reach `ancestor`, in `(hops from child, ConceptRef::Ord)`
/// order. Verbatim the shared kernel's chain CONTRACT, evaluated over the
/// oracle closure only.
fn eager_chain(
    closure: &ReachabilityClosure<ConceptRef>,
    child: &ConceptRef,
    ancestor: &ConceptRef,
) -> Option<Vec<ConceptRef>> {
    if child != ancestor && !closure.reaches(child, ancestor) {
        return None;
    }
    let mut chain: Vec<(ConceptRef, u32)> = closure
        .reflexive_image(child)
        .into_iter()
        .filter(|(x, _)| closure.reaches(x, ancestor))
        .collect();
    chain.sort_unstable_by(|(a, da), (b, db)| da.cmp(db).then_with(|| a.cmp(b)));
    Some(chain.into_iter().map(|(v, _)| v).collect())
}

#[test]
fn materialized_closure_matches_eager_oracle_over_a_full_usc_title() {
    // Title 42 — the largest provisioned title (~150k archive nodes), so the
    // u32-CSR engine is oracle-checked at the scale it exists for.
    let corpus = require(
        load_uslm_corpus("legal/uscode/usc_title_42/usc_title_42-pl-119-90.xml"),
        "usc",
    );
    let usc = UsCode::from_uslm_titles_owned(vec![corpus.title]);
    let onto = usc_runtime_ontology(&usc, OntologyName::new_static("usc")).expect("materializes");

    let node_count = onto.archive().nodes.len();
    assert!(
        node_count > 100_000,
        "the full title must be loaded; got {node_count} nodes"
    );

    // The concept universe + the generating edges, read off the ARCHIVED
    // buffer (morphisms_from), independent of the closure's own adjacency.
    let concepts: Vec<ConceptRef> = onto
        .archive()
        .nodes
        .iter()
        .map(|n| onto.concept(n.name.to_string()))
        .collect();
    let parthood = relations_kind("Parthood");
    let mut parthood_edges: Vec<(ConceptRef, ConceptRef)> = Vec::new();
    for c in &concepts {
        for edge in onto.morphisms_from(c) {
            if edge.kind == parthood {
                parthood_edges.push((edge.source, edge.target));
            }
        }
    }
    assert!(
        parthood_edges.len() > 100_000,
        "a real title is a deep mereology; got {} Parthood edges",
        parthood_edges.len()
    );

    // Parthood is the ONE populated transitive kind of a USC projection — so
    // the Parthood sweep below covers the closure's whole populated surface,
    // and the Subsumption surface must be honestly empty.
    assert_eq!(
        onto.closure().populated_kinds(),
        vec![parthood.clone()],
        "a USC title populates exactly Parthood"
    );

    // The independent eager oracle over the same edge set.
    let oracle = ReachabilityClosure::fold(parthood_edges.iter().cloned());

    // (1) EVERY node: graded image (set + hops + canonical order), reachable
    // set, membership probe, reflexive chain, and the chain to every ancestor.
    let closure = onto.closure();
    for c in &concepts {
        let mut want = oracle.strict_image(c);
        want.sort_unstable_by(|(a, da), (b, db)| da.cmp(db).then_with(|| a.cmp(b)));
        let got = closure.image(c, &parthood);
        assert_eq!(got, want, "graded Parthood image mismatch at {c:?}");

        let want_set: BTreeSet<ConceptRef> = want.iter().map(|(v, _)| v.clone()).collect();
        assert_eq!(
            closure.reachable_from(c, parthood.clone()),
            want_set,
            "reachable set mismatch at {c:?}"
        );
        for (a, _) in &want {
            assert!(
                closure.reaches(c, a, parthood.clone()),
                "reaches({c:?}, {a:?}) must hold"
            );
            assert_eq!(
                closure.chain(c, a, &parthood),
                eager_chain(&oracle, c, a),
                "chain mismatch {c:?} -> {a:?}"
            );
        }
        // The reflexive chain is the honest singleton-or-cycle case.
        assert_eq!(
            closure.chain(c, c, &parthood),
            eager_chain(&oracle, c, c),
            "reflexive chain mismatch at {c:?}"
        );
        // Self-meet: the nearest common ancestor of a node with itself —
        // strict_image(c) ∩ reflexive_image(c), i.e. its nearest parent.
        assert_eq!(
            closure.meet(c, c, &parthood),
            oracle.meet_by(c, c, |v| v.clone()),
            "self-meet mismatch at {c:?}"
        );
        // The UNPOPULATED kind answers honestly empty — the oracle over the
        // title's (empty) Subsumption edge set.
        assert!(
            closure.subsumption_image(c).is_empty(),
            "a USC title has no Subsumption edges; got a non-empty image at {c:?}"
        );
    }

    // (2) A deterministic ordered-pair sweep across the whole title for meet +
    // chain — including the None/unreachable cases the per-ancestor loop never
    // exercises.
    let n = concepts.len();
    let step = 7_919usize; // prime stride
    let mut a_idx = 0usize;
    for k in 0..50_000usize {
        a_idx = (a_idx + step) % n;
        let b_idx = (a_idx.wrapping_mul(3).wrapping_add(k)) % n;
        let (a, b) = (&concepts[a_idx], &concepts[b_idx]);
        assert_eq!(
            closure.meet(a, b, &parthood),
            oracle.meet_by(a, b, |v| v.clone()),
            "meet sweep mismatch at ({a_idx}, {b_idx})"
        );
        assert_eq!(
            closure.chain(a, b, &parthood),
            eager_chain(&oracle, a, b),
            "chain sweep mismatch at ({a_idx}, {b_idx})"
        );
        assert_eq!(
            closure.reaches(a, b, parthood.clone()),
            a != b && oracle.reaches(a, b),
            "reaches sweep mismatch at ({a_idx}, {b_idx})"
        );
    }

    // (3) The unpopulated-kind query surface at scale: no meet, no chain,
    // no reachability along Subsumption anywhere in the sweep.
    let sub = subsumption_kind();
    for k in 0..1_000usize {
        let a = &concepts[(k * step) % n];
        let b = &concepts[(k * step * 3 + 1) % n];
        assert!(!closure.reaches(a, b, sub.clone()));
        assert_eq!(closure.meet(a, b, &sub), None);
        assert_eq!(
            closure.chain(a, b, &sub),
            (a == b).then(|| vec![a.clone()]),
            "an unpopulated kind's chain is reflexive-singleton or None"
        );
    }

    // (4) The INVERSE mereology — whole → parts, folded through the PUBLIC
    // `fold` over the same title. The upward Parthood relation is a forest
    // (every subdivision composes into exactly one parent), so its images are
    // chains with DISTINCT hop counts and the canonical `(hops,
    // ConceptRef::Ord)` order is never discriminating there. The downward
    // fan-out is the real tie surface: a section's parts at equal depth are
    // thousands of equal-hop image members whose order IS the id ↔ ConceptRef
    // order isomorphism — a closure that assigned u32 ids in any order other
    // than `ConceptRef::Ord` fails here, not just in the unit-scale tie pins.
    drop(oracle);
    let inverse_edges: Vec<RuntimeEdge> = parthood_edges
        .iter()
        .map(|(source, target)| RuntimeEdge {
            source: target.clone(),
            kind: parthood.clone(),
            target: source.clone(),
        })
        .collect();
    let transitive: BTreeSet<ConceptRef> = [parthood.clone()].into_iter().collect();
    let downward = MaterializedClosure::fold(&inverse_edges, &transitive);
    let inverse_oracle =
        ReachabilityClosure::fold(parthood_edges.iter().map(|(s, t)| (t.clone(), s.clone())));
    let mut tied_images = 0usize;
    for c in &concepts {
        let mut want = inverse_oracle.strict_image(c);
        want.sort_unstable_by(|(a, da), (b, db)| da.cmp(db).then_with(|| a.cmp(b)));
        if want.windows(2).any(|w| w[0].1 == w[1].1) {
            tied_images += 1;
        }
        assert_eq!(
            downward.image(c, &parthood),
            want,
            "downward graded image mismatch at {c:?}"
        );
    }
    assert!(
        tied_images > 10_000,
        "the downward fan-out must be a real tie surface; got {tied_images} tied images"
    );
    // Meet + chain over the deterministic sweep, downward.
    let mut a_idx = 0usize;
    for k in 0..50_000usize {
        a_idx = (a_idx + step) % n;
        let b_idx = (a_idx.wrapping_mul(3).wrapping_add(k)) % n;
        let (a, b) = (&concepts[a_idx], &concepts[b_idx]);
        assert_eq!(
            downward.meet(a, b, &parthood),
            inverse_oracle.meet_by(a, b, |v| v.clone()),
            "downward meet sweep mismatch at ({a_idx}, {b_idx})"
        );
        assert_eq!(
            downward.chain(a, b, &parthood),
            eager_chain(&inverse_oracle, a, b),
            "downward chain sweep mismatch at ({a_idx}, {b_idx})"
        );
    }

    eprintln!(
        "CSR≡EAGER PASS: {node_count} nodes, {} Parthood edges ({tied_images} tied downward \
         images) — image/reachable/reaches/chain/meet match the eager oracle both directions",
        parthood_edges.len()
    );
}
