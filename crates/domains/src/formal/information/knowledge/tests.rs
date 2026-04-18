use super::ontology::*;
use pr4xis::category::Category;
use pr4xis::category::entity::Entity;
use pr4xis::category::validate::check_category_laws;

#[test]
fn category_laws() {
    check_category_laws::<KnowledgeBaseCategory>().unwrap();
}

#[test]
fn six_concepts() {
    assert_eq!(KnowledgeConcept::variants().len(), 6);
}

#[test]
fn knowledge_base_catalogs_vocabulary() {
    let m = KnowledgeBaseCategory::morphisms();
    assert!(m.iter().any(|r| r.from == KnowledgeConcept::KnowledgeBase
        && r.to == KnowledgeConcept::Vocabulary
        && r.kind == KnowledgeRelationKind::Catalogs));
}

#[test]
fn vocabulary_conforms_to_schema() {
    let m = KnowledgeBaseCategory::morphisms();
    assert!(m.iter().any(|r| r.from == KnowledgeConcept::Vocabulary
        && r.to == KnowledgeConcept::Schema
        && r.kind == KnowledgeRelationKind::ConformsTo));
}

#[test]
fn vocabulary_contains_entries() {
    let m = KnowledgeBaseCategory::morphisms();
    assert!(m.iter().any(|r| r.from == KnowledgeConcept::Vocabulary
        && r.to == KnowledgeConcept::Entry
        && r.kind == KnowledgeRelationKind::Contains));
}

#[test]
fn vocabulary_derived_from_datasource() {
    let m = KnowledgeBaseCategory::morphisms();
    assert!(m.iter().any(|r| r.from == KnowledgeConcept::Vocabulary
        && r.to == KnowledgeConcept::DataSource
        && r.kind == KnowledgeRelationKind::DerivedFrom));
}

#[test]
fn schema_defines_entry() {
    let m = KnowledgeBaseCategory::morphisms();
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
    let mut seen = std::collections::HashSet::new();
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
            d.concepts().len() > 0,
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
    assert!(
        axioms >= 500,
        "axioms below baseline: {}, expected ≥500",
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

// =============================================================================
// Compose API integration tests
// =============================================================================

mod compose {
    use pr4xis::ontology::RuntimeOntology;
    use pr4xis::ontology::compose::{EdgeKind, Metroplex};
    use pr4xis::ontology::upper::being::Being;

    #[test]
    fn compose_biology_chemistry_via_korporator() {
        let bio = RuntimeOntology::create("Biology")
            .source("Mayr (1982)")
            .being(Being::AbstractObject)
            .concept("Cell")
            .concept("Organism")
            .concept("Tissue")
            .is_a("Cell", "Tissue")
            .is_a("Tissue", "Organism")
            .build();

        let chem = RuntimeOntology::create("Chemistry")
            .source("Pauling (1960)")
            .being(Being::AbstractObject)
            .concept("Atom")
            .concept("Molecule")
            .concept("Cell")
            .has_a("Molecule", "Atom")
            .has_a("Cell", "Molecule")
            .build();

        let shared = bio.shared_with(&chem);
        assert_eq!(shared.len(), 1);
        assert!(shared.contains("Cell"));

        let biochem = bio.compose(&chem);
        assert_eq!(biochem.concepts().len(), 5);
        assert!(biochem.name().contains("&"));
        assert_eq!(biochem.level(), 1);
        assert!(biochem.validate().is_ok());

        let vocab = biochem.vocabulary();
        assert_eq!(vocab.concepts().len(), 5);
    }

    #[test]
    fn korporator_coupling_preserves_both() {
        let physics = RuntimeOntology::create("Physics")
            .concept("Force")
            .concept("Mass")
            .causes("Force", "Mass")
            .build();

        let music = RuntimeOntology::create("Music")
            .concept("Pitch")
            .concept("Rhythm")
            .opposes("Pitch", "Rhythm")
            .build();

        let coupled = physics.couple(&music);
        assert_eq!(coupled.concepts().len(), 4);
        assert!(coupled.concept("Force").is_some());
        assert!(coupled.concept("Pitch").is_some());
        assert!(coupled.name().contains("||"));
    }

    #[test]
    fn metroplex_builds_hierarchy() {
        let algebra = RuntimeOntology::create("Algebra")
            .source("Goguen & Burstall (1992)")
            .being(Being::AbstractObject)
            .concept("Coproduct")
            .concept("Product")
            .concept("Colimit")
            .is_a("Coproduct", "Colimit")
            .opposes("Coproduct", "Product")
            .build();

        let staging = RuntimeOntology::create("Staging")
            .source("Futamura (1971)")
            .being(Being::AbstractObject)
            .concept("Program")
            .concept("Interpreter")
            .concept("Compiler")
            .is_a("Compiler", "Program")
            .is_a("Interpreter", "Program")
            .build();

        let meta = algebra.compose(&staging);

        let mut mplex = Metroplex::new("pr4xis");
        mplex.add(algebra);
        mplex.add(staging);
        mplex.add(meta);

        assert_eq!(mplex.grade_count(), 2);
        assert_eq!(mplex.grade(0).len(), 2);
        assert_eq!(mplex.grade(1).len(), 1);

        let vocabs = mplex.vocabularies();
        assert_eq!(vocabs.len(), 3);
    }

    #[test]
    fn specialize_extracts_subtaxonomy() {
        let nav = RuntimeOntology::create("Navigation")
            .source("Groves (2013)")
            .concept("Sensor")
            .concept("GNSS")
            .concept("IMU")
            .concept("Odometry")
            .concept("Fusion")
            .is_a("GNSS", "Sensor")
            .is_a("IMU", "Sensor")
            .is_a("Odometry", "Sensor")
            .build();

        let sensors = nav.specialize("Sensor").unwrap();
        assert_eq!(sensors.concepts().len(), 4);
        assert!(sensors.concept("GNSS").is_some());
        assert!(sensors.concept("Fusion").is_none());
    }

    #[test]
    fn partial_korporator_selective_composition() {
        let full = RuntimeOntology::create("FullDomain")
            .concept("A")
            .concept("B")
            .concept("C")
            .concept("D")
            .is_a("A", "B")
            .is_a("C", "D")
            .build();

        let base = RuntimeOntology::create("Base").concept("X").build();

        let partial = base.couple_partial(&full, &["A", "B"]);
        assert!(partial.concepts().contains_key("A"));
        assert!(partial.concepts().contains_key("B"));
        assert!(partial.concepts().contains_key("X"));
        assert!(!partial.concepts().contains_key("C"));
        assert!(!partial.concepts().contains_key("D"));

        let ab_edges: Vec<_> = partial
            .edges()
            .iter()
            .filter(|e| e.kind == EdgeKind::IsA)
            .collect();
        assert_eq!(ab_edges.len(), 1);
        assert_eq!(ab_edges[0].from, "A");
        assert_eq!(ab_edges[0].to, "B");
    }

    #[test]
    fn compose_from_vocabulary_bridge() {
        let descriptors = super::super::describe_knowledge_base();
        let first = &descriptors[0];

        let syntrix = pr4xis::ontology::compose::from_vocabulary(first);
        assert_eq!(syntrix.name(), first.ontology_name.as_str());
        assert_eq!(syntrix.level(), 0);
    }

    #[test]
    fn compose_level_tracks_depth() {
        let a = RuntimeOntology::create("L0a").concept("X").build();
        let b = RuntimeOntology::create("L0b").concept("Y").build();
        assert_eq!(a.level(), 0);
        assert_eq!(b.level(), 0);

        let ab = a.couple(&b);
        assert_eq!(ab.level(), 1);

        let c = RuntimeOntology::create("L0c").concept("Z").build();
        let abc = ab.couple(&c);
        assert_eq!(abc.level(), 2);
    }
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
            let id = KnowledgeBaseCategory::identity(&c);
            prop_assert_eq!(KnowledgeBaseCategory::compose(&id, &id), Some(id));
        }

        /// Every concept has both Identity and Composed self-morphisms.
        #[test]
        fn prop_self_morphisms(c in arb_knowledge()) {
            let m = KnowledgeBaseCategory::morphisms();
            let has_identity = m.iter().any(|r| r.from == c && r.to == c && r.kind == KnowledgeRelationKind::Identity);
            let has_composed = m.iter().any(|r| r.from == c && r.to == c && r.kind == KnowledgeRelationKind::Composed);
            prop_assert!(has_identity);
            prop_assert!(has_composed);
        }

        /// VoID: KnowledgeBase reaches every concept transitively.
        #[test]
        fn prop_knowledge_base_reaches_all(c in arb_knowledge()) {
            let m = KnowledgeBaseCategory::morphisms();
            let reachable = m.iter().any(|r| r.from == KnowledgeConcept::KnowledgeBase && r.to == c);
            prop_assert!(reachable, "KnowledgeBase should reach {:?}", c);
        }

        /// Composition with identity preserves any morphism.
        #[test]
        fn prop_left_identity(c in arb_knowledge()) {
            let m = KnowledgeBaseCategory::morphisms();
            let id = KnowledgeBaseCategory::identity(&c);
            for morph in m.iter().filter(|r| r.from == c) {
                let composed = KnowledgeBaseCategory::compose(&id, morph);
                prop_assert_eq!(composed.as_ref().map(|r| (r.from, r.to)), Some((morph.from, morph.to)));
            }
        }
    }
}
