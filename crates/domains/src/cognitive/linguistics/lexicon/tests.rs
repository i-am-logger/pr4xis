#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::cognitive::linguistics::language::Language;
use pr4xis::category::Category;
use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::Quality;

use super::ontology::*;
use super::pos::*;
use crate::cognitive::linguistics::english::English;
use crate::social::software::markup::xml::lmf;

fn sample_lang() -> English {
    let wn = lmf::reader::read_wordnet(SAMPLE_LMF).unwrap();
    English::from_wordnet(&wn)
}

const SAMPLE_LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="test" label="Test" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-dog-n"><Lemma writtenForm="dog" partOfSpeech="n"/><Sense id="d1" synset="s-dog"/></LexicalEntry>
    <LexicalEntry id="e-dogs-n"><Lemma writtenForm="dogs" partOfSpeech="n"/><Sense id="d2" synset="s-dog"/></LexicalEntry>
    <LexicalEntry id="e-run-v"><Lemma writtenForm="runs" partOfSpeech="v"/><Sense id="r1" synset="s-run"/></LexicalEntry>
    <LexicalEntry id="e-see-v"><Lemma writtenForm="sees" partOfSpeech="v"/><Sense id="s1" synset="s-see"/></LexicalEntry>
    <LexicalEntry id="e-give-v"><Lemma writtenForm="gives" partOfSpeech="v"/><Sense id="g1" synset="s-give"/></LexicalEntry>
    <LexicalEntry id="e-read-v1"><Lemma writtenForm="read" partOfSpeech="v"/><Sense id="rd1" synset="s-read"/></LexicalEntry>
    <LexicalEntry id="e-read-n"><Lemma writtenForm="read" partOfSpeech="n"/><Sense id="rd2" synset="s-read-n"/></LexicalEntry>
    <Synset id="s-dog" partOfSpeech="n" members="e-dog-n e-dogs-n"><Definition>a domesticated carnivore</Definition></Synset>
    <Synset id="s-run" partOfSpeech="v" members="e-run-v"><Definition>move fast</Definition></Synset>
    <Synset id="s-see" partOfSpeech="v" members="e-see-v"><Definition>perceive with eyes</Definition></Synset>
    <Synset id="s-give" partOfSpeech="v" members="e-give-v"><Definition>transfer possession</Definition></Synset>
    <Synset id="s-read" partOfSpeech="v" members="e-read-v1"><Definition>interpret written text</Definition></Synset>
    <Synset id="s-read-n" partOfSpeech="n" members="e-read-n"><Definition>an act of reading</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;

// =============================================================================
// Language lexical lookup tests — ALL through the Language trait
// =============================================================================

#[test]
fn lookup_function_word_determiner() {
    let lang = sample_lang();
    let the = lang.lexical_lookup("the").unwrap();
    assert_eq!(the.pos_tag(), PosTag::Determiner);
}

#[test]
fn lookup_function_word_copula() {
    let lang = sample_lang();
    let is = lang.lexical_lookup("is").unwrap();
    assert_eq!(is.pos_tag(), PosTag::Copula);
}

#[test]
fn lookup_function_word_pronoun() {
    let lang = sample_lang();
    let she = lang.lexical_lookup("she").unwrap();
    assert_eq!(she.pos_tag(), PosTag::Pronoun);
}

#[test]
fn lookup_interrogative_pronoun() {
    let lang = sample_lang();
    let what = lang.lexical_lookup("what").unwrap();
    assert!(what.is_interrogative());
    assert!(!what.is_anaphoric());
}

#[test]
fn lookup_personal_pronoun_is_anaphoric() {
    let lang = sample_lang();
    let it = lang.lexical_lookup("it").unwrap();
    assert!(it.is_anaphoric());
    assert!(!it.is_interrogative());
}

#[test]
fn lookup_content_word_noun() {
    let lang = sample_lang();
    let dog = lang.lexical_lookup("dog").unwrap();
    assert_eq!(dog.pos_tag(), PosTag::Noun);
}

#[test]
fn lookup_content_word_verb() {
    let lang = sample_lang();
    let runs = lang.lexical_lookup("runs").unwrap();
    assert_eq!(runs.pos_tag(), PosTag::Verb);
}

#[test]
fn lookup_homograph_has_multiple() {
    let lang = sample_lang();
    let all = lang.lexical_lookup_all("read");
    assert!(all.len() >= 2, "read should have verb + noun entries");
}

#[test]
fn lookup_unknown_returns_none() {
    let lang = sample_lang();
    assert!(lang.lexical_lookup("xyzzy").is_none());
}

#[test]
fn function_word_not_in_content() {
    let lang = sample_lang();
    // "the" should be found as function word (determiner), not as content word
    let the = lang.lexical_lookup("the").unwrap();
    assert_eq!(the.pos_tag(), PosTag::Determiner);
}

#[test]
fn verb_has_both_transitive_and_intransitive() {
    let lang = sample_lang();
    let entries = lang.lexical_lookup_all("runs");
    let transitivities: Vec<_> = entries
        .iter()
        .filter_map(|e| match e {
            LexicalEntry::Verb(v) => Some(v.transitivity),
            _ => None,
        })
        .collect();
    assert!(transitivities.contains(&Transitivity::Intransitive));
    assert!(transitivities.contains(&Transitivity::Transitive));
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
    assert_eq!(q.get(&PosTag::Copula), Some(false));
}

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
        fn prop_content_or_function(pos in arb_pos()) {
            prop_assert!(pos.is_content() || pos.is_function());
            prop_assert!(pos.is_content() != pos.is_function());
        }
    }
}
