use super::ontology::*;
use crate::technology::software::markup::xml::lmf;

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

// =============================================================================
// Opposition (antonym) tests
// =============================================================================

#[test]
fn big_opposes_small() {
    let en = sample_english();
    assert!(en.opposition_count() > 0);
}

// =============================================================================
// Full WordNet load + performance
// =============================================================================

#[test]
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
}
