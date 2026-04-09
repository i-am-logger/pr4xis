use praxis::category::validate::check_category_laws;
use praxis::ontology::Quality;

use super::ontology::*;
use super::pos::*;
use super::vocabulary;

// =============================================================================
// Vocabulary tests
// =============================================================================

#[test]
fn vocabulary_has_entries() {
    let vocab = vocabulary::english();
    assert!(vocab.len() > 100);
}

#[test]
fn lookup_known_word() {
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
fn lookup_verb_with_rich_data() {
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
fn lookup_determiner_with_definiteness() {
    let the = vocabulary::lookup("the").unwrap();
    match &the {
        LexicalEntry::Determiner(d) => {
            assert_eq!(d.definiteness, Definiteness::Definite);
            assert_eq!(d.number, None); // "the" works with both singular and plural
        }
        _ => panic!("expected Determiner"),
    }

    let a = vocabulary::lookup("a").unwrap();
    match &a {
        LexicalEntry::Determiner(d) => {
            assert_eq!(d.definiteness, Definiteness::Indefinite);
            assert_eq!(d.number, Some(Number::Singular));
        }
        _ => panic!("expected Determiner"),
    }
}

#[test]
fn lookup_pronoun() {
    let she = vocabulary::lookup("she").unwrap();
    match &she {
        LexicalEntry::Pronoun(p) => {
            assert_eq!(p.number, Number::Singular);
            assert_eq!(p.person, Person::Third);
        }
        _ => panic!("expected Pronoun"),
    }
}

#[test]
fn lookup_homograph() {
    // "read" has both present and past forms
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
fn lookup_unknown_word() {
    assert!(vocabulary::lookup("xyzzy").is_none());
}

#[test]
fn noun_pairs_have_both_numbers() {
    let dog_sg = vocabulary::lookup("dog").unwrap();
    let dog_pl = vocabulary::lookup("dogs").unwrap();
    assert_eq!(dog_sg.number(), Some(Number::Singular));
    assert_eq!(dog_pl.number(), Some(Number::Plural));
}

#[test]
fn verb_transitivity() {
    // "see" is transitive, "run" is intransitive, "give" is ditransitive
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
fn content_word_quality() {
    let q = IsContentWord;
    assert_eq!(q.get(&PosTag::Noun), Some(true));
    assert_eq!(q.get(&PosTag::Verb), Some(true));
    assert_eq!(q.get(&PosTag::Determiner), Some(false));
    assert_eq!(q.get(&PosTag::Preposition), Some(false));
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
        ]
    }

    proptest! {
        /// Every POS has an identity morphism.
        #[test]
        fn prop_identity_exists(pos in arb_pos()) {
            let id = LexicalCategory::identity(&pos);
            prop_assert_eq!(id.modifier, pos);
            prop_assert_eq!(id.head, pos);
        }

        /// Every vocabulary entry has a valid POS tag.
        #[test]
        fn prop_all_entries_have_pos(idx in 0..150usize) {
            let vocab = super::vocabulary::english();
            if let Some(entry) = vocab.get(idx) {
                let _tag = entry.pos_tag(); // should not panic
                let _text = entry.text();
                prop_assert!(!_text.is_empty());
            }
        }

        /// Content/function classification is exhaustive (every POS is one or the other).
        #[test]
        fn prop_content_or_function(pos in arb_pos()) {
            prop_assert!(pos.is_content() || pos.is_function());
            prop_assert!(pos.is_content() != pos.is_function());
        }
    }
}
