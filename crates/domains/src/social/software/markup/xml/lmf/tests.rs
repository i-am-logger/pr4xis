#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::*;
use super::reader;
use super::reader::LmfReadError;

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

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn read_sample_lmf() {
    let wn = reader::read_wordnet(SAMPLE_LMF).unwrap();
    assert_eq!(wn.synset_count(), 7);
    assert_eq!(wn.entry_count(), 6);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn synset_has_definition() {
    let wn = reader::read_wordnet(SAMPLE_LMF).unwrap();
    let dog = wn.find_synset("synset-dog-n-01").unwrap();
    assert_eq!(dog.definitions[0], "a domesticated carnivore");
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn synset_has_example() {
    let wn = reader::read_wordnet(SAMPLE_LMF).unwrap();
    let run = wn.find_synset("synset-run-v-01").unwrap();
    assert_eq!(run.examples[0], "she ran to the store");
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn synset_pos() {
    let wn = reader::read_wordnet(SAMPLE_LMF).unwrap();
    let dog = wn.find_synset("synset-dog-n-01").unwrap();
    assert_eq!(dog.pos, LmfPos::Noun);
    let big = wn.find_synset("synset-big-a-01").unwrap();
    assert_eq!(big.pos, LmfPos::Adjective);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn lookup_word() {
    let wn = reader::read_wordnet(SAMPLE_LMF).unwrap();
    let dog_synsets = wn.lookup_word("dog");
    assert_eq!(dog_synsets.len(), 1);
    assert_eq!(dog_synsets[0].id, "synset-dog-n-01");
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn big_and_large_share_synset() {
    let wn = reader::read_wordnet(SAMPLE_LMF).unwrap();
    let big_synsets = wn.lookup_word("big");
    let large_synsets = wn.lookup_word("large");
    assert_eq!(big_synsets[0].id, large_synsets[0].id); // same synset = synonyms
}

#[pr4xis::praxis_value(Verifiable)]
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

#[pr4xis::praxis_value(Verifiable)]
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

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn opposition_relations() {
    let wn = reader::read_wordnet(SAMPLE_LMF).unwrap();
    let opposition = wn.opposition_relations();
    // big ↔ small (both directions)
    assert_eq!(opposition.len(), 2);
    assert!(opposition.contains(&("big-a-01", "small-a-01")));
    assert!(opposition.contains(&("small-a-01", "big-a-01")));
}

#[pr4xis::praxis_value(Verifiable)]
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

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn sense_relation_type_classification() {
    assert!(SenseRelationType::Antonym.is_opposition());
    assert!(!SenseRelationType::Pertainym.is_opposition());
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn lmf_pos_roundtrip() {
    for code in ["n", "v", "a", "r"] {
        let pos = LmfPos::parse(code);
        assert_eq!(LmfPos::parse(pos.to_tag()), pos);
    }
}

mod prop {
    use super::*;
    use pr4xis::category::entity::FinitelyGenerated;
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

    pr4xis::register_praxis_value!(prop_pos_roundtrip, Deterministic);
    pr4xis::register_praxis_value!(prop_open_class_is_content, Verifiable);
    pr4xis::register_praxis_value!(prop_all_variants_exist, Verifiable);
}

// =============================================================================
// Each LmfReadError variant exercised (uniform-depth uplift vs USLM)
// =============================================================================

#[pr4xis::praxis_value(Honest)]
#[test]
fn error_xml_on_malformed_input() {
    let err = reader::read_wordnet("<not<<>valid").expect_err("malformed XML must fail");
    match err {
        LmfReadError::Xml(_) => {}
        other => panic!("expected Xml error, got {other:?}"),
    }
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn error_structure_on_no_lexicon_element() {
    // Well-formed XML with no <Lexicon> child of root.
    let xml = r##"<LexicalResource><NotALexicon/></LexicalResource>"##;
    let err = reader::read_wordnet(xml).expect_err("missing <Lexicon> must fail");
    match err {
        LmfReadError::Structure(s) => {
            assert!(s.contains("Lexicon"), "got: {s}");
        }
        other => panic!("expected Structure error, got {other:?}"),
    }
}

// =============================================================================
// Edge cases — empty containers (uniform-depth uplift vs USLM)
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn lexicon_with_zero_synsets_and_entries_parses() {
    let xml = r##"<LexicalResource><Lexicon id="empty" language="en"/></LexicalResource>"##;
    let wn = reader::read_wordnet(xml).expect("empty lexicon must parse");
    assert_eq!(wn.synset_count(), 0);
    assert_eq!(wn.entry_count(), 0);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn synset_with_no_relations_parses() {
    let xml = r##"<LexicalResource><Lexicon id="t" language="en"><Synset id="s1" partOfSpeech="n"><Definition>x</Definition></Synset></Lexicon></LexicalResource>"##;
    let wn = reader::read_wordnet(xml).unwrap();
    assert_eq!(wn.synset_count(), 1);
    let s = &wn.synsets[0];
    assert!(s.relations.is_empty());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn entry_with_no_senses_parses() {
    // LMF allows orphan entries (lemma without a sense). Verify
    // the parser doesn't choke.
    let xml = r##"<LexicalResource><Lexicon id="t" language="en"><LexicalEntry id="e1"><Lemma writtenForm="orphan" partOfSpeech="n"/></LexicalEntry></Lexicon></LexicalResource>"##;
    let wn = reader::read_wordnet(xml).unwrap();
    assert_eq!(wn.entry_count(), 1);
    assert!(wn.entries[0].senses.is_empty());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn sense_with_no_relations_parses() {
    let xml = r##"<LexicalResource><Lexicon id="t" language="en"><LexicalEntry id="e1"><Lemma writtenForm="x" partOfSpeech="n"/><Sense id="s1" synset="syn1"/></LexicalEntry></Lexicon></LexicalResource>"##;
    let wn = reader::read_wordnet(xml).unwrap();
    let entry = &wn.entries[0];
    assert_eq!(entry.senses.len(), 1);
    assert!(entry.senses[0].relations.is_empty());
}

// =============================================================================
// Each synset relation kind tested individually (uniform-depth uplift)
//
// Existing `taxonomy_relations` / `opposition_relations` tests verify
// classification family-wise. These exercise each relation kind one at
// a time so a regression in a single kind's parsing surfaces directly.
// =============================================================================

fn lmf_with_synset_relation(rel_type: &str) -> String {
    format!(
        r##"<LexicalResource><Lexicon id="t" language="en"><Synset id="src" partOfSpeech="n"><Definition>x</Definition><SynsetRelation relType="{rel_type}" target="dst"/></Synset><Synset id="dst" partOfSpeech="n"><Definition>y</Definition></Synset></Lexicon></LexicalResource>"##
    )
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn synset_relation_hypernym_round_trips() {
    let wn = reader::read_wordnet(&lmf_with_synset_relation("hypernym")).unwrap();
    let src = wn.synsets.iter().find(|s| s.id == "src").unwrap();
    assert_eq!(src.relations.len(), 1);
    assert_eq!(src.relations[0].rel_type, SynsetRelationType::Hypernym);
    assert_eq!(src.relations[0].target, "dst");
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn synset_relation_hyponym_round_trips() {
    let wn = reader::read_wordnet(&lmf_with_synset_relation("hyponym")).unwrap();
    let src = wn.synsets.iter().find(|s| s.id == "src").unwrap();
    assert_eq!(src.relations[0].rel_type, SynsetRelationType::Hyponym);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn synset_relation_holo_member_round_trips() {
    let wn = reader::read_wordnet(&lmf_with_synset_relation("holo_member")).unwrap();
    let src = wn.synsets.iter().find(|s| s.id == "src").unwrap();
    assert_eq!(src.relations[0].rel_type, SynsetRelationType::HoloMember);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn synset_relation_mero_part_round_trips() {
    let wn = reader::read_wordnet(&lmf_with_synset_relation("mero_part")).unwrap();
    let src = wn.synsets.iter().find(|s| s.id == "src").unwrap();
    assert_eq!(src.relations[0].rel_type, SynsetRelationType::MeroPart);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn synset_relation_causes_round_trips() {
    let wn = reader::read_wordnet(&lmf_with_synset_relation("causes")).unwrap();
    let src = wn.synsets.iter().find(|s| s.id == "src").unwrap();
    assert_eq!(src.relations[0].rel_type, SynsetRelationType::Causes);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn synset_relation_entails_round_trips() {
    let wn = reader::read_wordnet(&lmf_with_synset_relation("entails")).unwrap();
    let src = wn.synsets.iter().find(|s| s.id == "src").unwrap();
    assert_eq!(src.relations[0].rel_type, SynsetRelationType::Entails);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn synset_relation_similar_round_trips() {
    let wn = reader::read_wordnet(&lmf_with_synset_relation("similar")).unwrap();
    let src = wn.synsets.iter().find(|s| s.id == "src").unwrap();
    assert_eq!(src.relations[0].rel_type, SynsetRelationType::Similar);
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn synset_relation_unknown_collapses_to_other() {
    let wn =
        reader::read_wordnet(&lmf_with_synset_relation("definitely_not_a_real_reltype")).unwrap();
    let src = wn.synsets.iter().find(|s| s.id == "src").unwrap();
    assert!(matches!(
        src.relations[0].rel_type,
        SynsetRelationType::Other(_)
    ));
}

// =============================================================================
// Non-ASCII text round-trip (uniform-depth uplift vs USLM)
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn unicode_in_definition_preserved() {
    let xml = r##"<LexicalResource><Lexicon id="t" language="en"><Synset id="s1" partOfSpeech="n"><Definition>déjà vu — the feeling of having “been here before”</Definition></Synset></Lexicon></LexicalResource>"##;
    let wn = reader::read_wordnet(xml).unwrap();
    let d = &wn.synsets[0].definitions[0];
    assert!(d.contains("déjà"), "accented chars lost: {d:?}");
    assert!(d.contains('—'), "em-dash lost: {d:?}");
    assert!(d.contains('“'), "curly quote lost: {d:?}");
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn unicode_in_lemma_preserved() {
    let xml = r##"<LexicalResource><Lexicon id="t" language="en"><LexicalEntry id="e"><Lemma writtenForm="café" partOfSpeech="n"/><Sense id="s" synset="syn"/></LexicalEntry></Lexicon></LexicalResource>"##;
    let wn = reader::read_wordnet(xml).unwrap();
    assert_eq!(wn.entries[0].lemma.written_form, "café");
}

// =============================================================================
// Sense relations — each kind individually (uniform-depth uplift)
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn sense_relation_antonym_round_trips() {
    let xml = r##"<LexicalResource><Lexicon id="t" language="en"><LexicalEntry id="e"><Lemma writtenForm="big" partOfSpeech="a"/><Sense id="big-a-01" synset="s"><SenseRelation relType="antonym" target="small-a-01"/></Sense></LexicalEntry></Lexicon></LexicalResource>"##;
    let wn = reader::read_wordnet(xml).unwrap();
    let sense = &wn.entries[0].senses[0];
    assert_eq!(sense.relations.len(), 1);
    assert_eq!(sense.relations[0].rel_type, SenseRelationType::Antonym);
    assert_eq!(sense.relations[0].target, "small-a-01");
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn sense_relation_unknown_collapses_to_other() {
    let xml = r##"<LexicalResource><Lexicon id="t" language="en"><LexicalEntry id="e"><Lemma writtenForm="x" partOfSpeech="n"/><Sense id="x-01" synset="s"><SenseRelation relType="nonsense_reltype" target="y"/></Sense></LexicalEntry></Lexicon></LexicalResource>"##;
    let wn = reader::read_wordnet(xml).unwrap();
    let sense = &wn.entries[0].senses[0];
    assert!(matches!(
        sense.relations[0].rel_type,
        SenseRelationType::Other(_)
    ));
}

// =============================================================================
// Generated arbitrary LMF proptest (uniform-depth uplift)
// =============================================================================

use proptest::prelude::*;

#[derive(Debug, Clone)]
struct ArbSynset {
    id: String,
    pos: String,
    relations: Vec<(String, String)>, // (reltype, target_id)
}

#[derive(Debug, Clone)]
struct ArbEntry {
    id: String,
    lemma: String,
    pos: String,
}

fn arb_pos_str() -> impl Strategy<Value = &'static str> {
    proptest::sample::select(vec!["n", "v", "a", "r", "s"])
}

fn arb_synset_strategy() -> impl Strategy<Value = ArbSynset> {
    (
        "syn-[a-z]{1,8}-[0-9]{1,3}",
        arb_pos_str(),
        proptest::collection::vec(
            (
                proptest::sample::select(vec![
                    "hypernym",
                    "hyponym",
                    "holo_member",
                    "mero_part",
                    "causes",
                    "entails",
                    "similar",
                ]),
                "syn-[a-z]{1,8}-[0-9]{1,3}",
            )
                .prop_map(|(r, t)| (r.to_string(), t)),
            0..4,
        ),
    )
        .prop_map(|(id, pos, relations)| ArbSynset {
            id,
            pos: pos.to_string(),
            relations,
        })
}

fn arb_entry_strategy() -> impl Strategy<Value = ArbEntry> {
    ("e-[a-z]{1,8}-[a-z]", "[a-z]{1,10}", arb_pos_str()).prop_map(|(id, lemma, pos)| ArbEntry {
        id,
        lemma,
        pos: pos.to_string(),
    })
}

fn render_arb_lmf(synsets: &[ArbSynset], entries: &[ArbEntry]) -> String {
    let mut buf = String::from(r##"<LexicalResource><Lexicon id="t" language="en">"##);
    for e in entries {
        buf.push_str(&format!(
            r##"<LexicalEntry id="{}"><Lemma writtenForm="{}" partOfSpeech="{}"/><Sense id="{}-sense" synset="{}-syn"/></LexicalEntry>"##,
            e.id, e.lemma, e.pos, e.id, e.id
        ));
    }
    for s in synsets {
        buf.push_str(&format!(
            r##"<Synset id="{}" partOfSpeech="{}"><Definition>def</Definition>"##,
            s.id, s.pos
        ));
        for (rel, target) in &s.relations {
            buf.push_str(&format!(
                r##"<SynsetRelation relType="{rel}" target="{target}"/>"##
            ));
        }
        buf.push_str("</Synset>");
    }
    buf.push_str("</Lexicon></LexicalResource>");
    buf
}

proptest! {
    /// Property — for arbitrary synset and entry vectors, render →
    /// parse round-trip preserves the counts.
    #[test]
    fn prop_arbitrary_synsets_entries_count_preserved(
        synsets in proptest::collection::vec(arb_synset_strategy(), 0..10),
        entries in proptest::collection::vec(arb_entry_strategy(), 0..10),
    ) {
        let xml = render_arb_lmf(&synsets, &entries);
        let wn = reader::read_wordnet(&xml).unwrap();
        prop_assert_eq!(wn.synset_count(), synsets.len());
        prop_assert_eq!(wn.entry_count(), entries.len());
    }

    /// Property — every emitted relType round-trips into the
    /// matching SynsetRelationType (or Other for unknowns).
    #[test]
    fn prop_arbitrary_relations_round_trip(
        synsets in proptest::collection::vec(arb_synset_strategy(), 1..5),
    ) {
        let xml = render_arb_lmf(&synsets, &[]);
        let wn = reader::read_wordnet(&xml).unwrap();
        for (orig, parsed) in synsets.iter().zip(wn.synsets.iter()) {
            prop_assert_eq!(orig.relations.len(), parsed.relations.len());
            for ((orig_rel, _), parsed_rel) in
                orig.relations.iter().zip(parsed.relations.iter())
            {
                let expected = SynsetRelationType::parse(orig_rel);
                // `rel_type` is no longer `Copy` (it carries the source
                // string in `Other(String)`), so clone for the by-value
                // assertion.
                prop_assert_eq!(parsed_rel.rel_type.clone(), expected);
            }
        }
    }

    /// Property — parser is deterministic across arbitrary inputs.
    #[test]
    fn prop_arbitrary_lmf_parse_is_deterministic(
        synsets in proptest::collection::vec(arb_synset_strategy(), 0..5),
        entries in proptest::collection::vec(arb_entry_strategy(), 0..5),
    ) {
        let xml = render_arb_lmf(&synsets, &entries);
        let a = reader::read_wordnet(&xml).unwrap();
        let b = reader::read_wordnet(&xml).unwrap();
        prop_assert_eq!(a.synset_count(), b.synset_count());
        prop_assert_eq!(a.entry_count(), b.entry_count());
    }

    /// Property — POS values round-trip through Lemma.
    #[test]
    fn prop_lemma_pos_round_trips(
        entries in proptest::collection::vec(arb_entry_strategy(), 1..8),
    ) {
        let xml = render_arb_lmf(&[], &entries);
        let wn = reader::read_wordnet(&xml).unwrap();
        for (orig, parsed) in entries.iter().zip(wn.entries.iter()) {
            let expected = LmfPos::parse(&orig.pos);
            prop_assert_eq!(parsed.lemma.pos, expected);
        }
    }
}

pr4xis::register_praxis_value!(
    prop_arbitrary_synsets_entries_count_preserved,
    Deterministic
);
pr4xis::register_praxis_value!(prop_arbitrary_relations_round_trip, Deterministic);
pr4xis::register_praxis_value!(prop_arbitrary_lmf_parse_is_deterministic, Deterministic);
pr4xis::register_praxis_value!(prop_lemma_pos_round_trips, Deterministic);
