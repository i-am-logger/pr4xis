use super::ontology::*;
use super::reader;

const SAMPLE_LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="test" label="Test" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="entry-dog-n">
      <Lemma writtenForm="dog" partOfSpeech="n"/>
      <Sense id="dog-n-01" synset="synset-dog-n-01"/>
    </LexicalEntry>
    <LexicalEntry id="entry-cat-n">
      <Lemma writtenForm="cat" partOfSpeech="n"/>
      <Sense id="cat-n-01" synset="synset-cat-n-01"/>
    </LexicalEntry>
    <LexicalEntry id="entry-big-a">
      <Lemma writtenForm="big" partOfSpeech="a"/>
      <Sense id="big-a-01" synset="synset-big-a-01">
        <SenseRelation relType="antonym" target="small-a-01"/>
      </Sense>
    </LexicalEntry>
    <LexicalEntry id="entry-large-a">
      <Lemma writtenForm="large" partOfSpeech="a"/>
      <Sense id="large-a-01" synset="synset-big-a-01"/>
    </LexicalEntry>
    <LexicalEntry id="entry-small-a">
      <Lemma writtenForm="small" partOfSpeech="a"/>
      <Sense id="small-a-01" synset="synset-small-a-01">
        <SenseRelation relType="antonym" target="big-a-01"/>
      </Sense>
    </LexicalEntry>
    <LexicalEntry id="entry-run-v">
      <Lemma writtenForm="run" partOfSpeech="v"/>
      <Form writtenForm="runs"/>
      <Form writtenForm="ran"/>
      <Form writtenForm="running"/>
      <Sense id="run-v-01" synset="synset-run-v-01"/>
    </LexicalEntry>
    <Synset id="synset-dog-n-01" ili="i1" partOfSpeech="n" members="entry-dog-n">
      <Definition>a domesticated carnivore</Definition>
      <SynsetRelation relType="hypernym" target="synset-mammal-n-01"/>
    </Synset>
    <Synset id="synset-cat-n-01" ili="i2" partOfSpeech="n" members="entry-cat-n">
      <Definition>a small domesticated feline</Definition>
      <SynsetRelation relType="hypernym" target="synset-mammal-n-01"/>
    </Synset>
    <Synset id="synset-mammal-n-01" ili="i3" partOfSpeech="n" members="">
      <Definition>warm-blooded vertebrate with hair</Definition>
      <SynsetRelation relType="hypernym" target="synset-animal-n-01"/>
    </Synset>
    <Synset id="synset-animal-n-01" ili="i4" partOfSpeech="n" members="">
      <Definition>a living organism</Definition>
    </Synset>
    <Synset id="synset-big-a-01" ili="i5" partOfSpeech="a" members="entry-big-a entry-large-a">
      <Definition>above average in size</Definition>
    </Synset>
    <Synset id="synset-small-a-01" ili="i6" partOfSpeech="a" members="entry-small-a">
      <Definition>below average in size</Definition>
    </Synset>
    <Synset id="synset-run-v-01" ili="i7" partOfSpeech="v" members="entry-run-v">
      <Definition>move fast by using one's feet</Definition>
      <Example>she ran to the store</Example>
    </Synset>
  </Lexicon>
</LexicalResource>"#;

// =============================================================================
// LMF Reader tests
// =============================================================================

#[test]
fn read_sample_lmf() {
    let wn = reader::read_wordnet(SAMPLE_LMF).unwrap();
    assert_eq!(wn.synset_count(), 7);
    assert_eq!(wn.entry_count(), 6);
}

#[test]
fn synset_has_definition() {
    let wn = reader::read_wordnet(SAMPLE_LMF).unwrap();
    let dog = wn.find_synset("synset-dog-n-01").unwrap();
    assert_eq!(dog.definitions[0], "a domesticated carnivore");
}

#[test]
fn synset_has_example() {
    let wn = reader::read_wordnet(SAMPLE_LMF).unwrap();
    let run = wn.find_synset("synset-run-v-01").unwrap();
    assert_eq!(run.examples[0], "she ran to the store");
}

#[test]
fn synset_pos() {
    let wn = reader::read_wordnet(SAMPLE_LMF).unwrap();
    let dog = wn.find_synset("synset-dog-n-01").unwrap();
    assert_eq!(dog.pos, LmfPos::Noun);
    let big = wn.find_synset("synset-big-a-01").unwrap();
    assert_eq!(big.pos, LmfPos::Adjective);
}

#[test]
fn lookup_word() {
    let wn = reader::read_wordnet(SAMPLE_LMF).unwrap();
    let dog_synsets = wn.lookup_word("dog");
    assert_eq!(dog_synsets.len(), 1);
    assert_eq!(dog_synsets[0].id, "synset-dog-n-01");
}

#[test]
fn big_and_large_share_synset() {
    let wn = reader::read_wordnet(SAMPLE_LMF).unwrap();
    let big_synsets = wn.lookup_word("big");
    let large_synsets = wn.lookup_word("large");
    assert_eq!(big_synsets[0].id, large_synsets[0].id); // same synset = synonyms
}

#[test]
fn morphological_forms() {
    let wn = reader::read_wordnet(SAMPLE_LMF).unwrap();
    let run_entry = wn
        .entries
        .iter()
        .find(|e| e.lemma.written_form == "run")
        .unwrap();
    let forms: Vec<&str> = run_entry
        .forms
        .iter()
        .map(|f| f.written_form.as_str())
        .collect();
    assert!(forms.contains(&"runs"));
    assert!(forms.contains(&"ran"));
    assert!(forms.contains(&"running"));
}

// =============================================================================
// Reasoning ontology mapping tests
// =============================================================================

#[test]
fn taxonomy_relations() {
    let wn = reader::read_wordnet(SAMPLE_LMF).unwrap();
    let taxonomy = wn.taxonomy_relations();
    // dog → mammal, cat → mammal, mammal → animal
    assert_eq!(taxonomy.len(), 3);
    assert!(taxonomy.contains(&("synset-dog-n-01", "synset-mammal-n-01")));
    assert!(taxonomy.contains(&("synset-cat-n-01", "synset-mammal-n-01")));
    assert!(taxonomy.contains(&("synset-mammal-n-01", "synset-animal-n-01")));
}

#[test]
fn opposition_relations() {
    let wn = reader::read_wordnet(SAMPLE_LMF).unwrap();
    let opposition = wn.opposition_relations();
    // big ↔ small (both directions)
    assert_eq!(opposition.len(), 2);
    assert!(opposition.contains(&("big-a-01", "small-a-01")));
    assert!(opposition.contains(&("small-a-01", "big-a-01")));
}

#[test]
fn synset_relation_type_classification() {
    assert!(SynsetRelationType::Hypernym.is_taxonomy());
    assert!(SynsetRelationType::InstanceHypernym.is_taxonomy());
    assert!(!SynsetRelationType::Hypernym.is_mereology());

    assert!(SynsetRelationType::MeroPart.is_mereology());
    assert!(SynsetRelationType::HoloPart.is_mereology());
    assert!(!SynsetRelationType::MeroPart.is_taxonomy());

    assert!(SynsetRelationType::Causes.is_causal());
    assert!(!SynsetRelationType::Causes.is_taxonomy());
}

#[test]
fn sense_relation_type_classification() {
    assert!(SenseRelationType::Antonym.is_opposition());
    assert!(!SenseRelationType::Pertainym.is_opposition());
}

#[test]
fn lmf_pos_roundtrip() {
    for code in ["n", "v", "a", "r"] {
        let pos = LmfPos::parse(code);
        assert_eq!(LmfPos::parse(pos.to_tag()), pos);
    }
}

mod prop {
    use super::*;
    use pr4xis::category::entity::Entity;
    use proptest::prelude::*;

    fn arb_pos() -> impl Strategy<Value = LmfPos> {
        prop_oneof![
            Just(LmfPos::Noun),
            Just(LmfPos::Verb),
            Just(LmfPos::Adjective),
            Just(LmfPos::Adverb),
            Just(LmfPos::Determiner),
            Just(LmfPos::Pronoun),
            Just(LmfPos::Preposition),
            Just(LmfPos::Conjunction),
            Just(LmfPos::Particle),
            Just(LmfPos::Copula),
            Just(LmfPos::Auxiliary),
            Just(LmfPos::Interjection),
            Just(LmfPos::Numeral),
            Just(LmfPos::Other),
        ]
    }

    proptest! {
        /// Every POS tag round-trips through parse(to_tag()).
        #[test]
        fn prop_pos_roundtrip(pos in arb_pos()) {
            prop_assert_eq!(LmfPos::parse(pos.to_tag()), pos);
        }

        /// Open class POS: Noun, Verb, Adjective, Adverb.
        #[test]
        fn prop_open_class_is_content(pos in arb_pos()) {
            let is_open = matches!(pos, LmfPos::Noun | LmfPos::Verb | LmfPos::Adjective | LmfPos::Adverb);
            if is_open {
                prop_assert!(pos.is_open_class());
            }
        }

        /// Entity variants() includes every POS.
        #[test]
        fn prop_all_variants_exist(pos in arb_pos()) {
            prop_assert!(LmfPos::variants().contains(&pos));
        }
    }
}

// =============================================================================
// Full WordNet load test (only runs if data file exists)
// =============================================================================

#[test]
fn load_full_wordnet() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/wordnet/english-wordnet-2025.xml"
    );

    if !std::path::Path::new(path).exists() {
        eprintln!("SKIP: WordNet data file not found at {}", path);
        return;
    }

    let xml = std::fs::read_to_string(path).unwrap();

    let start = std::time::Instant::now();
    let wn = reader::read_wordnet(&xml).unwrap();
    let load_time = start.elapsed();

    let taxonomy = wn.taxonomy_relations();
    let opposition = wn.opposition_relations();
    let mereology = wn.mereology_relations();
    let causal = wn.causal_relations();

    eprintln!("=== WordNet Load Performance ===");
    eprintln!("  Load time:     {:?}", load_time);
    eprintln!("  Synsets:       {}", wn.synset_count());
    eprintln!("  Entries:       {}", wn.entry_count());
    eprintln!("  Taxonomy:      {} relations", taxonomy.len());
    eprintln!("  Opposition:    {} relations", opposition.len());
    eprintln!("  Mereology:     {} relations", mereology.len());
    eprintln!("  Causation:     {} relations", causal.len());
    eprintln!("  Memory (est):  ~{} MB", (xml.len() * 2) / (1024 * 1024));

    // Verify reasonable counts
    assert!(wn.synset_count() > 100_000, "expected 100k+ synsets");
    assert!(wn.entry_count() > 100_000, "expected 100k+ entries");
    assert!(taxonomy.len() > 80_000, "expected 80k+ taxonomy relations");
    assert!(
        opposition.len() > 5_000,
        "expected 5k+ opposition relations"
    );

    // Test specific lookups
    let dog = wn.lookup_word("dog");
    assert!(!dog.is_empty(), "should find 'dog'");

    let entity = wn.lookup_word("entity");
    assert!(!entity.is_empty(), "should find 'entity'");
}
