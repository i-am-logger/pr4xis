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
    let descriptors = super::descriptor::describe_knowledge_base();
    assert!(
        descriptors.len() > 100,
        "describe_knowledge_base() returned only {} ontologies — likely missing registrations",
        descriptors.len()
    );
}

#[test]
fn describe_knowledge_base_names_are_unique() {
    let descriptors = super::descriptor::describe_knowledge_base();
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
    let descriptors = super::descriptor::describe_knowledge_base();
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
    let descriptors = super::descriptor::describe_knowledge_base();
    for d in &descriptors {
        assert!(
            d.concept_count > 0,
            "{} ({}) has 0 concepts",
            d.name(),
            d.domain()
        );
    }
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
        assert_eq!(biochem.concept_count(), 5);
        assert!(biochem.name().contains("&"));
        assert_eq!(biochem.level(), 1);
        assert!(biochem.validate().is_ok());

        let vocab = biochem.vocabulary();
        assert_eq!(vocab.concept_count, 5);
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
        assert_eq!(coupled.concept_count(), 4);
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
        assert_eq!(sensors.concept_count(), 4);
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
        assert!(partial.concept_names().contains("A"));
        assert!(partial.concept_names().contains("B"));
        assert!(partial.concept_names().contains("X"));
        assert!(!partial.concept_names().contains("C"));
        assert!(!partial.concept_names().contains("D"));

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
        let descriptors = super::super::descriptor::describe_knowledge_base();
        let first = &descriptors[0];

        let syntrix = pr4xis::ontology::compose::from_vocabulary(first);
        assert_eq!(syntrix.name(), first.ontology_name);
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
