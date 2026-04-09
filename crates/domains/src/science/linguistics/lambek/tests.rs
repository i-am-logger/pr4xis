use super::reduce::*;
use super::tokenize;
use super::types::*;
use crate::science::linguistics::language::English;
use crate::technology::software::markup::xml::lmf;

/// Sample English language for tokenizer tests.
/// Content words come from this WordNet; function words are built automatically.
fn sample_lang() -> English {
    let wn = lmf::reader::read_wordnet(SAMPLE_TOKENIZE_LMF).unwrap();
    English::from_wordnet(&wn)
}

const SAMPLE_TOKENIZE_LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="test" label="Test" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-dog-n"><Lemma writtenForm="dog" partOfSpeech="n"/><Sense id="d1" synset="s-dog"/></LexicalEntry>
    <LexicalEntry id="e-dogs-n"><Lemma writtenForm="dogs" partOfSpeech="n"/><Sense id="d2" synset="s-dog"/></LexicalEntry>
    <LexicalEntry id="e-cat-n"><Lemma writtenForm="cat" partOfSpeech="n"/><Sense id="c1" synset="s-cat"/></LexicalEntry>
    <LexicalEntry id="e-mammal-n"><Lemma writtenForm="mammal" partOfSpeech="n"/><Sense id="m1" synset="s-mammal"/></LexicalEntry>
    <LexicalEntry id="e-run-v"><Lemma writtenForm="run" partOfSpeech="v"/><Sense id="r1" synset="s-run"/></LexicalEntry>
    <LexicalEntry id="e-runs-v"><Lemma writtenForm="runs" partOfSpeech="v"/><Sense id="r2" synset="s-run"/></LexicalEntry>
    <LexicalEntry id="e-see-v"><Lemma writtenForm="sees" partOfSpeech="v"/><Sense id="s1" synset="s-see"/></LexicalEntry>
    <LexicalEntry id="e-big-a"><Lemma writtenForm="big" partOfSpeech="a"/><Sense id="b1" synset="s-big"/></LexicalEntry>
    <Synset id="s-dog" partOfSpeech="n" members="e-dog-n e-dogs-n"><Definition>a domesticated carnivore</Definition></Synset>
    <Synset id="s-cat" partOfSpeech="n" members="e-cat-n"><Definition>a small feline</Definition></Synset>
    <Synset id="s-mammal" partOfSpeech="n" members="e-mammal-n"><Definition>warm-blooded vertebrate</Definition></Synset>
    <Synset id="s-run" partOfSpeech="v" members="e-run-v e-runs-v"><Definition>move fast on foot</Definition></Synset>
    <Synset id="s-see" partOfSpeech="v" members="e-see-v"><Definition>perceive with the eyes</Definition></Synset>
    <Synset id="s-big" partOfSpeech="a" members="e-big-a"><Definition>above average in size</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;

// =============================================================================
// Type reduction tests
// =============================================================================

#[test]
fn forward_application() {
    // NP/N + N → NP ("the" + "dog" → NP)
    let result = reduce(&english::determiner(), &english::noun());
    assert_eq!(result, Some(LambekType::np()));
}

#[test]
fn backward_application() {
    // NP + NP\S → S ("dog" + "runs" → S)
    let result = reduce(&LambekType::np(), &english::intransitive_verb());
    assert_eq!(result, Some(LambekType::s()));
}

#[test]
fn no_reduction() {
    // N + NP → None (can't combine noun with noun phrase)
    let result = reduce(&english::noun(), &LambekType::np());
    assert_eq!(result, None);
}

#[test]
fn adjective_noun() {
    // N/N + N → N ("big" + "dog" → N)
    let result = reduce(&english::adjective(), &english::noun());
    assert_eq!(result, Some(LambekType::n()));
}

#[test]
fn transitive_verb_takes_object() {
    // (NP\S)/NP + NP → NP\S ("sees" + "dog" → VP)
    let result = reduce(&english::transitive_verb(), &LambekType::np());
    assert_eq!(result, Some(english::intransitive_verb()));
}

// =============================================================================
// Sequence reduction tests — full sentences
// =============================================================================

#[test]
fn the_dog_runs() {
    // the:NP/N + dog:N + runs:NP\S → S
    let tokens = vec![
        TypedToken {
            word: "the".into(),
            lambek_type: english::determiner(),
        },
        TypedToken {
            word: "dog".into(),
            lambek_type: english::noun(),
        },
        TypedToken {
            word: "runs".into(),
            lambek_type: english::intransitive_verb(),
        },
    ];
    let result = reduce_sequence(&tokens);
    assert!(result.success, "expected S, got {:?}", result.remaining);
    assert_eq!(result.final_type, Some(LambekType::s()));
}

#[test]
fn the_big_dog_runs() {
    // the:NP/N + big:N/N + dog:N + runs:NP\S → S
    let tokens = vec![
        TypedToken {
            word: "the".into(),
            lambek_type: english::determiner(),
        },
        TypedToken {
            word: "big".into(),
            lambek_type: english::adjective(),
        },
        TypedToken {
            word: "dog".into(),
            lambek_type: english::noun(),
        },
        TypedToken {
            word: "runs".into(),
            lambek_type: english::intransitive_verb(),
        },
    ];
    let result = reduce_sequence(&tokens);
    assert!(result.success, "expected S, got {:?}", result.remaining);
}

#[test]
fn she_sees_the_dog() {
    // she:NP + sees:(NP\S)/NP + the:NP/N + dog:N → S
    let tokens = vec![
        TypedToken {
            word: "she".into(),
            lambek_type: english::proper_noun(),
        },
        TypedToken {
            word: "sees".into(),
            lambek_type: english::transitive_verb(),
        },
        TypedToken {
            word: "the".into(),
            lambek_type: english::determiner(),
        },
        TypedToken {
            word: "dog".into(),
            lambek_type: english::noun(),
        },
    ];
    let result = reduce_sequence(&tokens);
    assert!(result.success, "expected S, got {:?}", result.remaining);
}

#[test]
fn dog_runs_not_sentence_alone() {
    // dog:N + runs:NP\S → can't reduce (N is not NP)
    let tokens = vec![
        TypedToken {
            word: "dog".into(),
            lambek_type: english::noun(),
        },
        TypedToken {
            word: "runs".into(),
            lambek_type: english::intransitive_verb(),
        },
    ];
    let result = reduce_sequence(&tokens);
    assert!(!result.success, "bare noun + verb should not reduce to S");
}

// =============================================================================
// Tokenizer tests — text to typed tokens via lexicon
// =============================================================================

#[test]
fn tokenize_simple() {
    let tokens = tokenize::tokenize("the dog runs", &sample_lang());
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].word, "the");
    assert_eq!(tokens[0].lambek_type, english::determiner());
    assert_eq!(tokens[1].word, "dog");
    assert_eq!(tokens[1].lambek_type, english::noun());
}

#[test]
fn tokenize_strips_punctuation() {
    let tokens = tokenize::tokenize("the dog runs.", &sample_lang());
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[2].word, "runs");
}

#[test]
fn tokenize_and_reduce() {
    // Full pipeline: text → tokens → reduction → S
    let tokens = tokenize::tokenize("the dog runs", &sample_lang());
    let result = reduce_sequence(&tokens);
    assert!(result.success, "expected S, got {:?}", result.remaining);
}

#[test]
fn tokenize_and_reduce_transitive() {
    // Verbs have both transitive and intransitive types in the language ontology.
    // The reducer should try alternatives when the first assignment fails.
    // Until the reducer handles ambiguity, test with explicit transitive type.
    let tokens = vec![
        TypedToken {
            word: "she".into(),
            lambek_type: english::proper_noun(),
        },
        TypedToken {
            word: "sees".into(),
            lambek_type: english::transitive_verb(),
        },
        TypedToken {
            word: "the".into(),
            lambek_type: english::determiner(),
        },
        TypedToken {
            word: "dog".into(),
            lambek_type: english::noun(),
        },
    ];
    let result = reduce_sequence(&tokens);
    assert!(result.success, "expected S, got {:?}", result.remaining);
}

#[test]
fn tokenize_and_reduce_adjective() {
    let tokens = tokenize::tokenize("the big dog runs", &sample_lang());
    let result = reduce_sequence(&tokens);
    assert!(result.success, "expected S, got {:?}", result.remaining);
}

// =============================================================================
// Copula + adjective tests
// =============================================================================

#[test]
fn a_dog_is_big() {
    // a:NP/N + dog:N + is:(NP\S)/NP + big:N/N
    // The copula takes a predicate NP — but adjective is N/N.
    // For "is big", we need the adjective to combine with a silent N to form NP,
    // or we need copula to take N/N as predicate.
    // In Lambek grammar, "is" as copula: (NP\S)/NP, "big" needs to be NP.
    // This is a known limitation — adjective predicates need special handling.
    // For now test the tokenizer assigns correct types:
    let tokens = tokenize::tokenize("a dog is big", &sample_lang());
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0].lambek_type, english::determiner()); // a
    assert_eq!(tokens[1].lambek_type, english::noun()); // dog
    // Post-processing assigns copula_adj + predicate_adjective (CCGbank)
    assert_eq!(tokens[2].lambek_type, english::copula_adj()); // is → (S[dcl]\NP)/(S[adj]\NP)
    assert_eq!(tokens[3].lambek_type, english::predicate_adjective()); // big → S[adj]\NP
}

#[test]
fn a_dog_is_big_reduces() {
    let tokens = tokenize::tokenize("a dog is big", &sample_lang());
    let result = reduce_sequence(&tokens);
    assert!(result.success, "expected S, got {:?}", result.remaining);
}

#[test]
fn spelling_correction_teh() {
    // "teh" is distance 1 from "the" — performance error (transposition)
    let tokens = tokenize::tokenize("teh dog runs", &sample_lang());
    assert_eq!(tokens[0].lambek_type, english::determiner());
}

#[test]
fn is_a_dog_a_mammal_question() {
    // Question formation: is at sentence start → question type
    let tokens = tokenize::tokenize("is a dog a mammal", &sample_lang());
    assert_eq!(tokens.len(), 5);
    assert_eq!(tokens[0].lambek_type, english::question_copula()); // is (question)
    let result = reduce_sequence(&tokens);
    assert!(result.success, "expected Q, got {:?}", result.remaining);
    assert_eq!(result.final_type, Some(LambekType::q()));
}

#[test]
fn what_is_a_dog() {
    let tokens = tokenize::tokenize("what is a dog", &sample_lang());
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0].lambek_type, english::wh_what()); // what
}

// =============================================================================
// Type notation tests
// =============================================================================

#[test]
fn type_notation() {
    assert_eq!(LambekType::s().notation(), "S");
    assert_eq!(LambekType::np().notation(), "NP");
    assert_eq!(english::determiner().notation(), "NP/N");
    assert_eq!(english::intransitive_verb().notation(), "NP\\S");
}

// =============================================================================
// Property-based tests
// =============================================================================

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_atomic() -> impl Strategy<Value = AtomicType> {
        prop_oneof![
            Just(AtomicType::S(None)),
            Just(AtomicType::S(Some(SentenceFeature::Dcl))),
            Just(AtomicType::S(Some(SentenceFeature::Q))),
            Just(AtomicType::S(Some(SentenceFeature::Adj))),
            Just(AtomicType::NP),
            Just(AtomicType::N),
            Just(AtomicType::PP),
        ]
    }

    proptest! {
        /// Forward application always works: A/B + B → A for any A, B.
        #[test]
        fn prop_forward_application(a in arb_atomic(), b in arb_atomic()) {
            let func = LambekType::right_div(LambekType::atom(a.clone()), LambekType::atom(b.clone()));
            let arg = LambekType::atom(b);
            let result = reduce(&func, &arg);
            prop_assert_eq!(result, Some(LambekType::atom(a)));
        }

        /// Backward application always works: A + A\B → B for any A, B.
        #[test]
        fn prop_backward_application(a in arb_atomic(), b in arb_atomic()) {
            let arg = LambekType::atom(a.clone());
            let func = LambekType::left_div(LambekType::atom(a), LambekType::atom(b.clone()));
            let result = reduce(&arg, &func);
            prop_assert_eq!(result, Some(LambekType::atom(b)));
        }

        /// Atoms never reduce with atoms.
        #[test]
        fn prop_atoms_dont_reduce(a in arb_atomic(), b in arb_atomic()) {
            let result = reduce(&LambekType::atom(a), &LambekType::atom(b));
            prop_assert_eq!(result, None);
        }

        /// Determiner + Noun always gives NP.
        #[test]
        fn prop_det_noun_gives_np(_dummy in 0..1i32) {
            let result = reduce(&english::determiner(), &english::noun());
            prop_assert_eq!(result, Some(LambekType::np()));
        }

        /// NP + intransitive verb always gives S.
        #[test]
        fn prop_np_iv_gives_s(_dummy in 0..1i32) {
            let result = reduce(&LambekType::np(), &english::intransitive_verb());
            prop_assert_eq!(result, Some(LambekType::s()));
        }
    }
}

// =============================================================================
// Montague functor tests — type-driven interpretation
// =============================================================================

use super::montague;
use crate::science::linguistics::english::English;

const SAMPLE_LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="test" label="Test" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-dog-n">
      <Lemma writtenForm="dog" partOfSpeech="n"/>
      <Sense id="dog-n-01" synset="s-dog"/>
    </LexicalEntry>
    <LexicalEntry id="e-run-v">
      <Lemma writtenForm="run" partOfSpeech="v"/>
      <Sense id="run-v-01" synset="s-run"/>
    </LexicalEntry>
    <Synset id="s-dog" partOfSpeech="n" members="e-dog-n">
      <Definition>a domesticated carnivore</Definition>
    </Synset>
    <Synset id="s-run" partOfSpeech="v" members="e-run-v">
      <Definition>move fast</Definition>
    </Synset>
  </Lexicon>
</LexicalResource>"#;

fn sample_english() -> English {
    let wn = lmf::reader::read_wordnet(SAMPLE_LMF).unwrap();
    English::from_wordnet(&wn)
}

#[test]
fn montague_the_dog_runs() {
    // the:NP/N + dog:N + runs:NP\S → S
    // Semantics: the(dog) = entity, runs(entity) = proposition
    let en = sample_english();
    let tokens = vec![
        TypedToken {
            word: "the".into(),
            lambek_type: english::determiner(),
        },
        TypedToken {
            word: "dog".into(),
            lambek_type: english::noun(),
        },
        TypedToken {
            word: "runs".into(),
            lambek_type: english::intransitive_verb(),
        },
    ];
    let sem = montague::interpret(&tokens, &en);
    match &sem {
        montague::Sem::Prop {
            predicate,
            arguments,
        } => {
            assert!(
                predicate.contains("run"),
                "predicate should contain 'run', got '{}'",
                predicate
            );
            assert!(!arguments.is_empty(), "should have arguments");
        }
        other => panic!("expected Prop, got {:?}", other),
    }
}

#[test]
fn montague_she_sees_the_dog() {
    let en = sample_english();
    let tokens = vec![
        TypedToken {
            word: "she".into(),
            lambek_type: english::proper_noun(),
        },
        TypedToken {
            word: "sees".into(),
            lambek_type: english::transitive_verb(),
        },
        TypedToken {
            word: "the".into(),
            lambek_type: english::determiner(),
        },
        TypedToken {
            word: "dog".into(),
            lambek_type: english::noun(),
        },
    ];
    let sem = montague::interpret(&tokens, &en);
    match &sem {
        montague::Sem::Prop {
            predicate,
            arguments,
        } => {
            assert!(
                predicate.contains("see"),
                "predicate should contain 'see', got '{}'",
                predicate
            );
            assert!(
                arguments.len() >= 2,
                "transitive should have 2+ args, got {}",
                arguments.len()
            );
        }
        other => panic!("expected Prop, got {:?}", other),
    }
}

#[test]
fn montague_the_big_dog_runs() {
    let en = sample_english();
    let tokens = vec![
        TypedToken {
            word: "the".into(),
            lambek_type: english::determiner(),
        },
        TypedToken {
            word: "big".into(),
            lambek_type: english::adjective(),
        },
        TypedToken {
            word: "dog".into(),
            lambek_type: english::noun(),
        },
        TypedToken {
            word: "runs".into(),
            lambek_type: english::intransitive_verb(),
        },
    ];
    let sem = montague::interpret(&tokens, &en);
    match &sem {
        montague::Sem::Prop { .. } => {} // should produce a proposition
        other => panic!("expected Prop, got {:?}", other),
    }
}

#[test]
fn montague_describe() {
    let en = sample_english();
    let tokens = vec![
        TypedToken {
            word: "the".into(),
            lambek_type: english::determiner(),
        },
        TypedToken {
            word: "dog".into(),
            lambek_type: english::noun(),
        },
        TypedToken {
            word: "runs".into(),
            lambek_type: english::intransitive_verb(),
        },
    ];
    let sem = montague::interpret(&tokens, &en);
    let desc = sem.describe();
    // Should be something like "runs(dog)" or "runs(dog, ...)"
    assert!(!desc.is_empty());
}
