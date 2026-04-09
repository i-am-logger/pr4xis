use super::reduce::*;
use super::tokenize;
use super::types::*;

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
    let tokens = tokenize::tokenize("the dog runs");
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].word, "the");
    assert_eq!(tokens[0].lambek_type, english::determiner());
    assert_eq!(tokens[1].word, "dog");
    assert_eq!(tokens[1].lambek_type, english::noun());
}

#[test]
fn tokenize_strips_punctuation() {
    let tokens = tokenize::tokenize("the dog runs.");
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[2].word, "runs");
}

#[test]
fn tokenize_and_reduce() {
    // Full pipeline: text → tokens → reduction → S
    let tokens = tokenize::tokenize("the dog runs");
    let result = reduce_sequence(&tokens);
    assert!(result.success, "expected S, got {:?}", result.remaining);
}

#[test]
fn tokenize_and_reduce_transitive() {
    let tokens = tokenize::tokenize("she sees the dog");
    let result = reduce_sequence(&tokens);
    assert!(result.success, "expected S, got {:?}", result.remaining);
}

#[test]
fn tokenize_and_reduce_adjective() {
    let tokens = tokenize::tokenize("the big dog runs");
    let result = reduce_sequence(&tokens);
    assert!(result.success, "expected S, got {:?}", result.remaining);
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
            Just(AtomicType::S),
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
use crate::technology::software::markup::xml::lmf;

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
