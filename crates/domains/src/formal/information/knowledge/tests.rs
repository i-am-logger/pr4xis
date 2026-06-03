#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::*;
use pr4xis::category::Category;
use pr4xis::category::entity::FinitelyGenerated;
use pr4xis::category::laws::assert_category_laws;

#[test]
fn category_laws() {
    assert_category_laws::<KnowledgeCategory>();
}

#[test]
fn six_concepts() {
    assert_eq!(KnowledgeConcept::variants().len(), 6);
}

#[test]
fn knowledge_base_catalogs_vocabulary() {
    let m = KnowledgeCategory::morphisms();
    assert!(m.iter().any(|r| r.from == KnowledgeConcept::KnowledgeBase
        && r.to == KnowledgeConcept::Vocabulary
        && r.kind == KnowledgeRelationKind::Catalogs));
}

#[test]
fn vocabulary_conforms_to_schema() {
    let m = KnowledgeCategory::morphisms();
    assert!(m.iter().any(|r| r.from == KnowledgeConcept::Vocabulary
        && r.to == KnowledgeConcept::Schema
        && r.kind == KnowledgeRelationKind::ConformsTo));
}

#[test]
fn vocabulary_contains_entries() {
    let m = KnowledgeCategory::morphisms();
    assert!(m.iter().any(|r| r.from == KnowledgeConcept::Vocabulary
        && r.to == KnowledgeConcept::Entry
        && r.kind == KnowledgeRelationKind::Contains));
}

#[test]
fn vocabulary_derived_from_datasource() {
    let m = KnowledgeCategory::morphisms();
    assert!(m.iter().any(|r| r.from == KnowledgeConcept::Vocabulary
        && r.to == KnowledgeConcept::DataSource
        && r.kind == KnowledgeRelationKind::DerivedFrom));
}

#[test]
fn schema_defines_entry() {
    let m = KnowledgeCategory::morphisms();
    assert!(m.iter().any(|r| r.from == KnowledgeConcept::Schema
        && r.to == KnowledgeConcept::Entry
        && r.kind == KnowledgeRelationKind::Defines));
}

// =============================================================================
// Knowledge-base descriptor registry tests
// =============================================================================

#[test]
fn describe_knowledge_base_is_nonempty() {
    let descriptors = super::describe_knowledge_base();
    assert!(
        descriptors.len() > 100,
        "describe_knowledge_base() returned only {} ontologies — likely missing registrations",
        descriptors.len()
    );
}

#[test]
fn describe_knowledge_base_names_are_unique() {
    let descriptors = super::describe_knowledge_base();
    let mut seen = hashbrown::HashSet::new();
    for d in &descriptors {
        assert!(
            seen.insert((d.name(), d.domain())),
            "duplicate (name, domain): ({}, {})",
            d.name(),
            d.domain()
        );
    }
}

#[test]
fn describe_knowledge_base_no_stale_science_prefix() {
    let descriptors = super::describe_knowledge_base();
    for d in &descriptors {
        assert!(
            !d.domain().starts_with("science."),
            "stale domain prefix: {} has domain '{}' — should use cognitive/formal/natural/social/applied",
            d.name(),
            d.domain()
        );
    }
}

#[test]
fn every_descriptor_has_nonzero_concepts() {
    let descriptors = super::describe_knowledge_base();
    for d in &descriptors {
        assert!(
            !d.concepts().is_empty(),
            "{} ({}) has 0 concepts",
            d.name(),
            d.domain()
        );
    }
}

// =============================================================================
// Lemon-uniform registry tests (issue #148) — axioms / functors / adjunctions
// auto-registered alongside ontologies.
// =============================================================================

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn declared_axioms_are_registered() {
    let axioms = pr4xis::ontology::describe_axioms();
    // MAPE-K declared three axioms via the `axioms:` clause; they must
    // appear in the global registry with structured citations.
    let names: Vec<String> = axioms.iter().map(|m| m.name.as_str().to_string()).collect();
    assert!(
        names.iter().any(|n| n == "FourPhaseCycle"),
        "FourPhaseCycle not auto-registered: got {:?}",
        names
    );
    assert!(
        names.iter().any(|n| n == "LoopIsClosed"),
        "LoopIsClosed not auto-registered"
    );
    assert!(
        names.iter().any(|n| n == "EveryPhaseConsultsKnowledge"),
        "EveryPhaseConsultsKnowledge not auto-registered"
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn declared_functors_are_registered() {
    let functors = pr4xis::ontology::describe_functors();
    // PipelineStepToMapeK migrated to pr4xis::functor! — should be in the slice.
    let names: Vec<String> = functors
        .iter()
        .map(|m| m.name.as_str().to_string())
        .collect();
    assert!(
        names.iter().any(|n| n == "PipelineStepToMapeK"),
        "PipelineStepToMapeK not auto-registered: got {:?}",
        names
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn declared_adjunctions_are_registered() {
    let adjunctions = pr4xis::ontology::describe_adjunctions();
    // KnowledgeLemonAdjunction migrated to pr4xis::adjunction!.
    let names: Vec<String> = adjunctions
        .iter()
        .map(|m| m.name.as_str().to_string())
        .collect();
    assert!(
        names.iter().any(|n| n == "KnowledgeLemonAdjunction"),
        "KnowledgeLemonAdjunction not auto-registered: got {:?}",
        names
    );
}

// -----------------------------------------------------------------------------
// Relations-ontology refactor (issue #152) — parity baseline.
//
// Captures the registry counts post-#150 as the baseline for the four-PR
// Relations-ontology refactor. Later PRs in that series assert the counts
// stay ≥ these numbers (they should grow as new ontologies land and shrink
// only by the intentional deletion of the four primitive reasoning modules).
// -----------------------------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn refactor_parity_baseline_counts() {
    let vocabs = pr4xis::ontology::describe_knowledge_base().len();
    let axioms = pr4xis::ontology::describe_axioms().len();
    let functors = pr4xis::ontology::describe_functors().len();
    let adjunctions = pr4xis::ontology::describe_adjunctions().len();
    let nats = pr4xis::ontology::describe_natural_transformations().len();

    // Baseline thresholds (post-#150). PRs B–D must not regress below.
    // New ontologies in PR B push the vocab count up; PR D removes the four
    // primitive reasoning modules (which aren't registered vocabularies
    // anyway, so the count isn't affected by their removal).
    assert!(
        vocabs >= 130,
        "vocabularies below baseline: {}, expected ≥130",
        vocabs
    );
    // Baseline rebased after #166 / C.21 / C.24 — the per-def reasoning
    // traits (TaxonomyDef/MereologyDef/CausalDef/OppositionDef) were
    // collapsed into kinded morphisms; their per-ontology axiom emission
    // went away, lowering the workspace axiom count. Bar is the
    // post-refactor floor.
    assert!(
        axioms >= 450,
        "axioms below baseline: {}, expected ≥450",
        axioms
    );
    assert!(
        functors >= 80,
        "functors below baseline: {}, expected ≥80",
        functors
    );
    assert!(
        adjunctions >= 5,
        "adjunctions below baseline: {}, expected ≥5",
        adjunctions
    );

    // Print counts on --nocapture for manual inspection during the refactor.
    eprintln!(
        "parity baseline: vocabs={}, axioms={}, functors={}, adjunctions={}, nats={}",
        vocabs, axioms, functors, adjunctions, nats
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn registry_sees_workspace_scale() {
    // After full migration (issue #148), the three secondary registries
    // should hold substantial numbers — hundreds of axioms, dozens of
    // functors, a handful of adjunctions — matching the workspace scale.
    let axioms = pr4xis::ontology::describe_axioms().len();
    let functors = pr4xis::ontology::describe_functors().len();
    let adjunctions = pr4xis::ontology::describe_adjunctions().len();

    assert!(
        axioms > 100,
        "expected >100 registered axioms after full migration; got {}",
        axioms
    );
    assert!(
        functors > 30,
        "expected >30 registered functors after full migration; got {}",
        functors
    );
    assert!(
        adjunctions > 2,
        "expected >2 registered adjunctions after full migration; got {}",
        adjunctions
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn workspace_axioms_mostly_carry_citations() {
    // After issue #148 citation migration, the majority of registered
    // axioms should carry non-empty citations (from their file's
    // literature doc block). Some will still be empty (files with no
    // clear Source: marker); that's acceptable as long as the ratio is high.
    let axioms = pr4xis::ontology::describe_axioms();
    let total = axioms.len();
    let with_citation = axioms
        .iter()
        .filter(|m| !m.citation.as_str().is_empty())
        .count();
    let ratio = with_citation as f64 / total as f64;
    assert!(
        ratio > 0.20,
        "expected >20% of {} axioms to carry citations; got {} ({:.1}%)",
        total,
        with_citation,
        ratio * 100.0
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn registered_axioms_carry_nonempty_citations() {
    // Sample check: axioms declared via the `axioms:` clause must carry
    // the literature citation given at declaration, not the empty placeholder.
    let axioms = pr4xis::ontology::describe_axioms();
    let four_phase = axioms
        .iter()
        .find(|m| m.name.as_str() == "FourPhaseCycle")
        .expect("FourPhaseCycle should be registered");
    assert!(
        !four_phase.citation.as_str().is_empty(),
        "FourPhaseCycle citation is empty — declaration site didn't propagate"
    );
    assert!(
        four_phase.citation.as_str().contains("Kephart"),
        "FourPhaseCycle citation should reference Kephart & Chess, got: {}",
        four_phase.citation.as_str()
    );
}

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_knowledge() -> impl Strategy<Value = KnowledgeConcept> {
        prop_oneof![
            Just(KnowledgeConcept::KnowledgeBase),
            Just(KnowledgeConcept::Vocabulary),
            Just(KnowledgeConcept::Schema),
            Just(KnowledgeConcept::Entry),
            Just(KnowledgeConcept::Descriptor),
            Just(KnowledgeConcept::DataSource),
        ]
    }

    proptest! {
        #[test]
        fn prop_identity_idempotent(c in arb_knowledge()) {
            let id = KnowledgeCategory::identity(&c);
            prop_assert_eq!(KnowledgeCategory::compose(&id, &id), Some(id));
        }

        /// Every concept has an Identity self-morphism. Per #166 the
        /// auto-generated kind no longer emits `Composed` self-loops;
        /// composition of typed morphisms is partial.
        #[test]
        fn prop_self_morphisms(c in arb_knowledge()) {
            let m = KnowledgeCategory::morphisms();
            let has_identity = m.iter().any(|r| r.from == c && r.to == c && r.kind == KnowledgeRelationKind::Identity);
            prop_assert!(has_identity);
        }

        /// VoID: KnowledgeBase reaches every concept transitively. Per #166
        /// the heterogeneous-kind closure isn't a single morphism — walk
        /// the graph. DataSource is leaf-only (terminal w.r.t. the
        /// outgoing-edge graph from KB) and is excluded.
        #[test]
        fn prop_knowledge_base_reaches_all(c in arb_knowledge()) {
            use std::collections::{HashSet, VecDeque};
            use pr4xis::category::Arrow;
            let ms = KnowledgeCategory::morphisms();
            let mut visited: HashSet<KnowledgeConcept> = HashSet::new();
            let mut queue: VecDeque<KnowledgeConcept> = VecDeque::new();
            queue.push_back(KnowledgeConcept::KnowledgeBase);
            let mut reachable = c == KnowledgeConcept::KnowledgeBase;
            while let Some(n) = queue.pop_front() {
                if n == c {
                    reachable = true;
                    break;
                }
                if !visited.insert(n) {
                    continue;
                }
                for m in ms.iter().filter(|m| m.source() == n) {
                    queue.push_back(m.target());
                }
            }
            prop_assert!(reachable, "KnowledgeBase should reach {:?}", c);
        }

        /// Composition with identity preserves any morphism.
        #[test]
        fn prop_left_identity(c in arb_knowledge()) {
            let m = KnowledgeCategory::morphisms();
            let id = KnowledgeCategory::identity(&c);
            for morph in m.iter().filter(|r| r.from == c) {
                let composed = KnowledgeCategory::compose(&id, morph);
                prop_assert_eq!(composed.as_ref().map(|r| (r.from, r.to)), Some((morph.from, morph.to)));
            }
        }
    }
}
