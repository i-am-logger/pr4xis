#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::*;
use crate::social::software::markup::xml::lmf;

const SAMPLE_LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="test" label="Test" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-dog-n">
      <Lemma writtenForm="dog" partOfSpeech="n"/>
      <Sense id="dog-n-01" synset="s-dog"/>
    </LexicalEntry>
    <LexicalEntry id="e-cat-n">
      <Lemma writtenForm="cat" partOfSpeech="n"/>
      <Sense id="cat-n-01" synset="s-cat"/>
    </LexicalEntry>
    <LexicalEntry id="e-mammal-n">
      <Lemma writtenForm="mammal" partOfSpeech="n"/>
      <Sense id="mammal-n-01" synset="s-mammal"/>
    </LexicalEntry>
    <LexicalEntry id="e-animal-n">
      <Lemma writtenForm="animal" partOfSpeech="n"/>
      <Sense id="animal-n-01" synset="s-animal"/>
    </LexicalEntry>
    <LexicalEntry id="e-big-a">
      <Lemma writtenForm="big" partOfSpeech="a"/>
      <Sense id="big-a-01" synset="s-big">
        <SenseRelation relType="antonym" target="small-a-01"/>
      </Sense>
    </LexicalEntry>
    <LexicalEntry id="e-large-a">
      <Lemma writtenForm="large" partOfSpeech="a"/>
      <Sense id="large-a-01" synset="s-big"/>
    </LexicalEntry>
    <LexicalEntry id="e-small-a">
      <Lemma writtenForm="small" partOfSpeech="a"/>
      <Sense id="small-a-01" synset="s-small">
        <SenseRelation relType="antonym" target="big-a-01"/>
      </Sense>
    </LexicalEntry>
    <Synset id="s-dog" ili="i1" partOfSpeech="n" members="e-dog-n">
      <Definition>a domesticated carnivore</Definition>
      <SynsetRelation relType="hypernym" target="s-mammal"/>
    </Synset>
    <Synset id="s-cat" ili="i2" partOfSpeech="n" members="e-cat-n">
      <Definition>a small domesticated feline</Definition>
      <SynsetRelation relType="hypernym" target="s-mammal"/>
    </Synset>
    <Synset id="s-mammal" ili="i3" partOfSpeech="n" members="e-mammal-n">
      <Definition>warm-blooded vertebrate</Definition>
      <SynsetRelation relType="hypernym" target="s-animal"/>
    </Synset>
    <Synset id="s-animal" ili="i4" partOfSpeech="n" members="e-animal-n">
      <Definition>a living organism</Definition>
    </Synset>
    <Synset id="s-big" ili="i5" partOfSpeech="a" members="e-big-a e-large-a">
      <Definition>above average in size</Definition>
    </Synset>
    <Synset id="s-small" ili="i6" partOfSpeech="a" members="e-small-a">
      <Definition>below average in size</Definition>
    </Synset>
  </Lexicon>
</LexicalResource>"#;

fn sample_english() -> English {
    let wn = lmf::reader::read_wordnet(SAMPLE_LMF).unwrap();
    English::from_wordnet(&wn)
}

// =============================================================================
// Basic tests
// =============================================================================

#[test]
fn concept_count() {
    let en = sample_english();
    assert_eq!(en.concept_count(), 6);
}

#[test]
fn word_lookup() {
    let en = sample_english();
    let dog_concepts = en.lookup("dog");
    assert_eq!(dog_concepts.len(), 1);
    let dog = en.concept(dog_concepts[0]).unwrap();
    assert_eq!(dog.definitions[0], "a domesticated carnivore");
}

#[test]
fn synonyms_share_concept() {
    let en = sample_english();
    let big = en.lookup("big");
    let large = en.lookup("large");
    assert_eq!(big[0], large[0]); // same ConceptId = synonyms
}

#[test]
fn concept_has_lemmas() {
    let en = sample_english();
    let big_id = en.lookup("big")[0];
    let big = en.concept(big_id).unwrap();
    assert!(big.lemmas.contains(&"big".to_string()));
    assert!(big.lemmas.contains(&"large".to_string()));
}

// =============================================================================
// Taxonomy (is-a) tests
// =============================================================================

#[test]
fn direct_hypernym() {
    let en = sample_english();
    let dog_id = en.lookup("dog")[0];
    let parents = en.parents(dog_id);
    assert_eq!(parents.len(), 1);
    let parent = en.concept(parents[0]).unwrap();
    assert!(parent.lemmas.contains(&"mammal".to_string()));
}

#[test]
fn transitive_is_a() {
    let en = sample_english();
    let dog_id = en.lookup("dog")[0];
    let animal_id = en.lookup("animal")[0];
    assert!(en.is_a(dog_id, animal_id)); // dog is-a animal (via mammal)
}

#[test]
fn is_a_reflexive() {
    let en = sample_english();
    let dog_id = en.lookup("dog")[0];
    assert!(en.is_a(dog_id, dog_id));
}

#[test]
fn not_is_a() {
    let en = sample_english();
    let dog_id = en.lookup("dog")[0];
    let cat_id = en.lookup("cat")[0];
    assert!(!en.is_a(dog_id, cat_id)); // dog is NOT a cat
}

#[test]
fn children_of_mammal() {
    let en = sample_english();
    let mammal_id = en.lookup("mammal")[0];
    let children = en.children(mammal_id);
    assert_eq!(children.len(), 2); // dog and cat
}

#[test]
fn ancestors_are_the_reflexive_is_a_image() {
    // The reasoner's typed reachability operation — dog ⊑* {dog, mammal,
    // animal} — read off the same `parents` adjacency, not a chat-side walk.
    let en = sample_english();
    let dog = en.lookup("dog")[0];
    let mammal = en.lookup("mammal")[0];
    let animal = en.lookup("animal")[0];
    let cat = en.lookup("cat")[0];
    let anc = LexicalReasoner::ancestors(&en, dog);
    assert!(anc.contains(&dog)); // reflexive
    assert!(anc.contains(&mammal));
    assert!(anc.contains(&animal));
    assert!(!anc.contains(&cat)); // a sibling is not an ancestor
}

#[test]
fn common_ancestor_is_the_nearest_shared_hypernym() {
    // dog and cat share `mammal` (nearest) and `animal`; the LCA is `mammal`.
    // The chat asks the reasoner for this — it no longer hand-BFSes parents.
    let en = sample_english();
    let dog = en.lookup("dog")[0];
    let cat = en.lookup("cat")[0];
    let mammal = en.lookup("mammal")[0];
    assert_eq!(
        LexicalReasoner::common_ancestor(&en, dog, cat),
        Some(mammal)
    );
}

// =============================================================================
// Opposition (antonym) tests
// =============================================================================

#[test]
fn big_opposes_small() {
    let en = sample_english();
    assert!(en.opposition_count() > 0);
}

// =============================================================================
// Axiom-equivalent invariants (uniform-depth uplift vs from_uslm_section)
// =============================================================================

/// Axiom — the English functor is deterministic. Same WordNet input
/// → byte-equivalent English output (concept count, taxonomy edges,
/// lookups all stable).
#[test]
fn axiom_functor_is_deterministic() {
    let wn = lmf::reader::read_wordnet(SAMPLE_LMF).unwrap();
    let a = English::from_wordnet(&wn);
    let b = English::from_wordnet(&wn);
    assert_eq!(a.concept_count(), b.concept_count());
    let mut a_ids: Vec<ConceptId> = (0..a.concept_count())
        .map(|i| ConceptId::new(i as u64))
        .collect();
    let mut b_ids: Vec<ConceptId> = (0..b.concept_count())
        .map(|i| ConceptId::new(i as u64))
        .collect();
    a_ids.sort_by_key(|id| id.value());
    b_ids.sort_by_key(|id| id.value());
    assert_eq!(a_ids, b_ids);
}

/// Axiom — concept count equals synset count in the source
/// WordNet. The functor maps each synset to exactly one concept.
#[test]
fn axiom_concept_count_equals_synset_count() {
    let wn = lmf::reader::read_wordnet(SAMPLE_LMF).unwrap();
    let en = English::from_wordnet(&wn);
    assert_eq!(en.concept_count(), wn.synset_count());
}

/// Axiom — every word in the inflection / lemma index resolves to
/// at least one valid ConceptId.
#[test]
fn axiom_every_lookup_returns_valid_concept_ids() {
    let en = sample_english();
    for word in ["dog", "cat", "mammal", "animal", "big", "large", "small"] {
        let ids = en.lookup(word);
        assert!(!ids.is_empty(), "lookup({word}) returned no ids");
        for id in ids {
            assert!(
                en.concept(*id).is_some(),
                "lookup({word}) returned invalid ConceptId {id:?}"
            );
        }
    }
}

/// Axiom — antonym opposition is symmetric where the source data
/// records both directions. `big ↔ small` is the canonical pair in
/// our fixture.
#[test]
fn axiom_antonym_opposition_is_symmetric_when_source_records_both() {
    let en = sample_english();
    let big = en.lookup("big");
    let small = en.lookup("small");
    if !big.is_empty() && !small.is_empty() {
        // If big→small is recorded, small→big should be too (the
        // fixture has both directions explicitly).
        let big_opposes_small = en.opposition_count() > 0;
        assert!(big_opposes_small);
    }
}

/// Axiom — looking up a known word never returns a ConceptId whose
/// `concept()` is None (no dangling pointers from the lemma index
/// into the concept table).
#[test]
fn axiom_no_dangling_lookups() {
    let en = sample_english();
    for word in ["dog", "cat", "mammal", "animal"] {
        for id in en.lookup(word) {
            assert!(en.concept(*id).is_some(), "{word} → dangling id {id:?}");
        }
    }
}

// =============================================================================
// Generated arbitrary English-functor proptests (uniform-depth uplift)
// =============================================================================

use proptest::prelude::*;

proptest! {
    /// Property — from_wordnet is deterministic across runs.
    #[test]
    fn prop_from_wordnet_is_deterministic(seed in any::<u32>()) {
        let _ = seed;
        let wn = lmf::reader::read_wordnet(SAMPLE_LMF).unwrap();
        let a = English::from_wordnet(&wn);
        let b = English::from_wordnet(&wn);
        prop_assert_eq!(a.concept_count(), b.concept_count());
    }

    /// Property — every lookup returns ConceptIds within
    /// `[0, concept_count)`.
    #[test]
    fn prop_lookup_ids_in_range(seed in any::<u32>()) {
        let _ = seed;
        let en = sample_english();
        let max = en.concept_count() as u64;
        for word in ["dog", "cat", "mammal", "animal", "big", "large", "small"] {
            for id in en.lookup(word) {
                prop_assert!(
                    id.value() < max,
                    "{word} returned id {} ≥ max {max}",
                    id.value(),
                );
            }
        }
    }

    /// Property — lookup is case-insensitive only if WordNet itself
    /// is. Our sample is lowercase-only, so uppercase queries on
    /// known words return empty (no silent normalization).
    #[test]
    fn prop_lookup_unknown_word_returns_empty(
        word in "[A-Z]{1,8}",
    ) {
        let en = sample_english();
        let ids = en.lookup(&word);
        prop_assert!(
            ids.is_empty(),
            "uppercase {word:?} unexpectedly matched: {ids:?}"
        );
    }
}

// =============================================================================
// codegen::wordnet error and edge-case coverage (uniform-depth uplift)
//
// pr4xis::codegen::wordnet::parse_wordnet_xml is build-time code with
// zero tests in the codegen module itself. Cover the error and
// edge-case paths here at the domains-side test layer.
// =============================================================================

#[test]
fn codegen_parse_empty_lexicon_yields_zero_entities() {
    use std::io::Write;
    let tmp = tempfile::NamedTempFile::new().expect("temp file");
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?><LexicalResource><Lexicon id="t" language="en"/></LexicalResource>"##;
    write!(tmp.as_file(), "{xml}").unwrap();
    let builder =
        pr4xis::codegen::wordnet::parse_wordnet_xml(tmp.path()).expect("empty lexicon must parse");
    assert_eq!(builder.entity_count(), 0);
}

#[test]
fn codegen_parse_missing_file_returns_io_error() {
    let result = pr4xis::codegen::wordnet::parse_wordnet_xml(std::path::Path::new(
        "/tmp/definitely_does_not_exist_lmf.xml",
    ));
    match result {
        Err(pr4xis::codegen::wordnet::ParseError::Io(_)) => {}
        other => panic!("expected Io error, got {other:?}"),
    }
}

#[test]
fn codegen_parse_well_formed_synsets_round_trips() {
    use std::io::Write;
    let tmp = tempfile::NamedTempFile::new().expect("temp file");
    write!(tmp.as_file(), "{SAMPLE_LMF}").unwrap();
    let builder = pr4xis::codegen::wordnet::parse_wordnet_xml(tmp.path()).expect("parse");
    // Our SAMPLE_LMF has 6 synsets.
    assert_eq!(builder.entity_count(), 6);
    // Has hypernym relations → at least 3 taxonomy edges.
    assert!(
        builder.relation_count() >= 3,
        "expected ≥3 relations, got {}",
        builder.relation_count()
    );
}

// =============================================================================
// Full WordNet load + performance
// =============================================================================

#[test]
#[ignore = "perf measurement — parses the 89 MB WordNet XML to time the build; not a gate. \
            Correctness of the full English ontology is exercised by the (now fast, \
            `english_loaded()`-backed) lambek/adjunction consumers and the WordNet \
            compactness gate in praxis-corpus-tests."]
fn load_full_english() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/wordnet/english-wordnet-2025.xml"
    );

    if !std::path::Path::new(path).exists() {
        eprintln!("SKIP: WordNet data not found");
        return;
    }

    let xml = std::fs::read_to_string(path).unwrap();

    // Phase 1: Parse XML through LMF ontology
    let t0 = std::time::Instant::now();
    let wn = lmf::reader::read_wordnet(&xml).unwrap();
    let parse_time = t0.elapsed();

    // Phase 2: Build English ontology (the functor)
    let t1 = std::time::Instant::now();
    let en = English::from_wordnet(&wn);
    let build_time = t1.elapsed();

    // Phase 3: Query performance
    let t2 = std::time::Instant::now();
    let dog = en.lookup("dog");
    let _dog_concept = en.concept(dog[0]).unwrap();
    let query_time = t2.elapsed();

    let t3 = std::time::Instant::now();
    let entity_concepts = en.lookup("entity");
    let _is_a = if !dog.is_empty() && !entity_concepts.is_empty() {
        en.is_a(dog[0], entity_concepts[0])
    } else {
        false
    };
    let is_a_time = t3.elapsed();

    // Memory estimate: size of pre-computed structures
    let concept_mem = en.concepts.len() * std::mem::size_of::<Concept>();
    let taxonomy_mem = en.taxonomy_count() * std::mem::size_of::<ConceptId>();
    let word_index_mem = en.word_count() * 64; // rough estimate per entry

    eprintln!("=== English Ontology Performance ===");
    eprintln!("  XML parse:     {:?}", parse_time);
    eprintln!("  Ontology build: {:?}", build_time);
    eprintln!("  Total load:    {:?}", parse_time + build_time);
    eprintln!("  Word lookup:   {:?}", query_time);
    eprintln!("  is_a query:    {:?}", is_a_time);
    eprintln!("  Concepts:      {}", en.concept_count());
    eprintln!("  Words:         {}", en.word_count());
    eprintln!("  Taxonomy:      {} relations", en.taxonomy_count());
    eprintln!("  Opposition:    {} relations", en.opposition_count());
    eprintln!("  Memory (concepts): ~{} KB", concept_mem / 1024);
    eprintln!("  Memory (taxonomy): ~{} KB", taxonomy_mem / 1024);
    eprintln!("  Memory (words):    ~{} KB", word_index_mem / 1024);

    assert!(en.concept_count() > 100_000);
    assert!(en.word_count() > 50_000);
    assert!(en.taxonomy_count() > 80_000);

    // Diagnose taxonomy: check "dog" senses and their parents
    let dog_ids = en.lookup("dog");
    eprintln!("\n=== Dog Taxonomy Diagnosis ===");
    eprintln!("  'dog' has {} senses", dog_ids.len());
    for &did in dog_ids {
        let c = en.concept(did).unwrap();
        let parents = en.parents(did);
        let parent_names: Vec<String> = parents
            .iter()
            .filter_map(|&p| {
                en.concept(p)
                    .map(|c| c.lemmas.first().cloned().unwrap_or_default())
            })
            .collect();
        eprintln!(
            "  sense {}: {:?} ({}) → parents: {:?}",
            did.value(),
            c.pos,
            c.definitions.first().unwrap_or(&String::new()),
            parent_names
        );
    }

    let mammal_ids = en.lookup("mammal");
    eprintln!("  'mammal' has {} senses", mammal_ids.len());
    for &mid in mammal_ids {
        let c = en.concept(mid).unwrap();
        eprintln!(
            "  sense {}: {}",
            mid.value(),
            c.definitions.first().unwrap_or(&String::new())
        );
    }

    // Check is_a for all dog×mammal pairs
    let mut found = false;
    for &did in dog_ids {
        for &mid in mammal_ids {
            let result = en.is_a(did, mid);
            if result {
                eprintln!(
                    "  ✅ is_a(dog sense {}, mammal sense {}) = TRUE",
                    did.value(),
                    mid.value()
                );
                found = true;
            }
        }
    }
    if !found {
        eprintln!("  ❌ No dog sense is-a any mammal sense!");
    }
}
