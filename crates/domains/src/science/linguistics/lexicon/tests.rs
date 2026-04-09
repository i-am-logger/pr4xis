use praxis::category::validate::check_category_laws;
use praxis::ontology::Quality;

use super::function_words;
use super::ontology::*;
use super::pos::*;
use super::vocabulary;

// =============================================================================
// Function word tests (OLiA-classified closed class)
// =============================================================================

#[test]
fn function_words_has_entries() {
    let fw = function_words::english_function_words();
    assert!(fw.len() > 50);
}

#[test]
fn function_word_lookup_determiner() {
    let the = function_words::lookup("the").unwrap();
    assert_eq!(the.pos_tag(), PosTag::Determiner);
    match &the {
        LexicalEntry::Determiner(d) => {
            assert_eq!(d.definiteness, Definiteness::Definite);
            assert_eq!(d.number, None);
        }
        _ => panic!("expected Determiner"),
    }

    let a = function_words::lookup("a").unwrap();
    match &a {
        LexicalEntry::Determiner(d) => {
            assert_eq!(d.definiteness, Definiteness::Indefinite);
            assert_eq!(d.number, Some(Number::Singular));
        }
        _ => panic!("expected Determiner"),
    }
}

#[test]
fn function_word_lookup_copula() {
    let is = function_words::lookup("is").unwrap();
    assert_eq!(is.pos_tag(), PosTag::Copula);
    match &is {
        LexicalEntry::Copula(c) => {
            assert_eq!(c.number, Number::Singular);
            assert_eq!(c.person, Person::Third);
            assert_eq!(c.tense, Tense::Present);
        }
        _ => panic!("expected Copula"),
    }
}

#[test]
fn function_word_lookup_auxiliary() {
    let can = function_words::lookup("can").unwrap();
    assert_eq!(can.pos_tag(), PosTag::Auxiliary);
}

#[test]
fn function_word_lookup_pronoun() {
    let she = function_words::lookup("she").unwrap();
    assert_eq!(she.pos_tag(), PosTag::Pronoun);
    match &she {
        LexicalEntry::Pronoun(p) => {
            assert_eq!(p.number, Number::Singular);
            assert_eq!(p.person, Person::Third);
        }
        _ => panic!("expected Pronoun"),
    }
}

#[test]
fn function_word_lookup_preposition() {
    let in_ = function_words::lookup("in").unwrap();
    assert_eq!(in_.pos_tag(), PosTag::Preposition);
}

#[test]
fn function_word_lookup_conjunction() {
    let and = function_words::lookup("and").unwrap();
    assert_eq!(and.pos_tag(), PosTag::Conjunction);
}

#[test]
fn function_word_lookup_particle() {
    let not = function_words::lookup("not").unwrap();
    assert_eq!(not.pos_tag(), PosTag::Particle);
}

#[test]
fn function_word_lookup_interjection() {
    let hello = function_words::lookup("hello").unwrap();
    assert_eq!(hello.pos_tag(), PosTag::Interjection);
}

#[test]
fn function_word_unknown() {
    assert!(function_words::lookup("dog").is_none());
    assert!(function_words::lookup("xyzzy").is_none());
}

// =============================================================================
// Content word tests (vocabulary.rs — to be replaced by WordNet runtime)
// =============================================================================

#[test]
fn vocabulary_has_content_words() {
    let vocab = vocabulary::english();
    assert!(vocab.len() > 100);
}

#[test]
fn vocabulary_noun_lookup() {
    let dog = vocabulary::lookup("dog").unwrap();
    assert_eq!(dog.pos_tag(), PosTag::Noun);
    match &dog {
        LexicalEntry::Noun(n) => {
            assert_eq!(n.number, Number::Singular);
            assert_eq!(n.person, Person::Third);
            assert_eq!(n.countability, Countability::Countable);
            assert_eq!(n.kind, NounKind::Common);
        }
        _ => panic!("expected Noun"),
    }
}

#[test]
fn vocabulary_verb_with_rich_data() {
    let runs = vocabulary::lookup("runs").unwrap();
    match &runs {
        LexicalEntry::Verb(v) => {
            assert_eq!(v.lemma, "run");
            assert_eq!(v.number, Number::Singular);
            assert_eq!(v.person, Person::Third);
            assert_eq!(v.tense, Tense::Present);
            assert_eq!(v.transitivity, Transitivity::Intransitive);
        }
        _ => panic!("expected Verb"),
    }
}

#[test]
fn vocabulary_homograph() {
    let reads = vocabulary::lookup_all("read");
    assert!(reads.len() >= 2);
    let tenses: Vec<_> = reads
        .iter()
        .filter_map(|e| match e {
            LexicalEntry::Verb(v) => Some(v.tense),
            _ => None,
        })
        .collect();
    assert!(tenses.contains(&Tense::Present));
    assert!(tenses.contains(&Tense::Past));
}

#[test]
fn vocabulary_noun_pairs() {
    let dog_sg = vocabulary::lookup("dog").unwrap();
    let dog_pl = vocabulary::lookup("dogs").unwrap();
    assert_eq!(dog_sg.number(), Some(Number::Singular));
    assert_eq!(dog_pl.number(), Some(Number::Plural));
}

#[test]
fn vocabulary_verb_transitivity() {
    let see = vocabulary::lookup("sees").unwrap();
    match see {
        LexicalEntry::Verb(v) => assert_eq!(v.transitivity, Transitivity::Transitive),
        _ => panic!("expected Verb"),
    }

    let run = vocabulary::lookup("runs").unwrap();
    match run {
        LexicalEntry::Verb(v) => assert_eq!(v.transitivity, Transitivity::Intransitive),
        _ => panic!("expected Verb"),
    }

    let give = vocabulary::lookup("gives").unwrap();
    match give {
        LexicalEntry::Verb(v) => assert_eq!(v.transitivity, Transitivity::Ditransitive),
        _ => panic!("expected Verb"),
    }
}

// =============================================================================
// Ontology tests
// =============================================================================

#[test]
fn lexical_category_laws() {
    check_category_laws::<LexicalCategory>().unwrap();
}

#[test]
fn adjective_modifies_noun() {
    let morphisms = LexicalCategory::morphisms();
    assert!(morphisms.contains(&Modifies {
        modifier: PosTag::Adjective,
        head: PosTag::Noun,
    }));
}

#[test]
fn adverb_modifies_verb() {
    let morphisms = LexicalCategory::morphisms();
    assert!(morphisms.contains(&Modifies {
        modifier: PosTag::Adverb,
        head: PosTag::Verb,
    }));
}

#[test]
fn auxiliary_modifies_verb() {
    let morphisms = LexicalCategory::morphisms();
    assert!(morphisms.contains(&Modifies {
        modifier: PosTag::Auxiliary,
        head: PosTag::Verb,
    }));
}

#[test]
fn content_word_quality() {
    let q = IsContentWord;
    assert_eq!(q.get(&PosTag::Noun), Some(true));
    assert_eq!(q.get(&PosTag::Verb), Some(true));
    assert_eq!(q.get(&PosTag::Determiner), Some(false));
    assert_eq!(q.get(&PosTag::Preposition), Some(false));
    assert_eq!(q.get(&PosTag::Copula), Some(false));
    assert_eq!(q.get(&PosTag::Auxiliary), Some(false));
}

use praxis::category::Category;

// =============================================================================
// Property-based tests
// =============================================================================

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_pos() -> impl Strategy<Value = PosTag> {
        prop_oneof![
            Just(PosTag::Noun),
            Just(PosTag::Verb),
            Just(PosTag::Determiner),
            Just(PosTag::Adjective),
            Just(PosTag::Adverb),
            Just(PosTag::Preposition),
            Just(PosTag::Conjunction),
            Just(PosTag::Pronoun),
            Just(PosTag::Copula),
            Just(PosTag::Auxiliary),
            Just(PosTag::Article),
            Just(PosTag::Interjection),
            Just(PosTag::Particle),
            Just(PosTag::Numeral),
        ]
    }

    proptest! {
        #[test]
        fn prop_identity_exists(pos in arb_pos()) {
            let id = LexicalCategory::identity(&pos);
            prop_assert_eq!(id.modifier, pos);
            prop_assert_eq!(id.head, pos);
        }

        #[test]
        fn prop_all_function_words_have_pos(idx in 0..100usize) {
            let fw = super::function_words::english_function_words();
            if let Some(entry) = fw.get(idx) {
                let _tag = entry.pos_tag();
                let _text = entry.text();
                prop_assert!(!_text.is_empty());
            }
        }

        #[test]
        fn prop_content_or_function(pos in arb_pos()) {
            prop_assert!(pos.is_content() || pos.is_function());
            prop_assert!(pos.is_content() != pos.is_function());
        }
    }
}
