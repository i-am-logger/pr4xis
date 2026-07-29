#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::reduce::*;
use super::tokenize;
use super::types::*;
use crate::cognitive::linguistics::english::English;
use crate::social::software::markup::xml::lmf;

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
    <LexicalEntry id="e-bug-n"><Lemma writtenForm="bug" partOfSpeech="n"/><Sense id="bu1" synset="s-bug"/></LexicalEntry>
    <Synset id="s-dog" partOfSpeech="n" members="e-dog-n e-dogs-n"><Definition>a domesticated carnivore</Definition></Synset>
    <Synset id="s-cat" partOfSpeech="n" members="e-cat-n"><Definition>a small feline</Definition></Synset>
    <Synset id="s-mammal" partOfSpeech="n" members="e-mammal-n"><Definition>warm-blooded vertebrate</Definition></Synset>
    <Synset id="s-run" partOfSpeech="v" members="e-run-v e-runs-v"><Definition>move fast on foot</Definition></Synset>
    <Synset id="s-see" partOfSpeech="v" members="e-see-v"><Definition>perceive with the eyes</Definition></Synset>
    <Synset id="s-big" partOfSpeech="a" members="e-big-a"><Definition>above average in size</Definition></Synset>
    <Synset id="s-bug" partOfSpeech="n" members="e-bug-n"><Definition>an insect</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;

// =============================================================================
// Type reduction tests
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn forward_application() {
    // NP/N + N → NP ("the" + "dog" → NP)
    let result = reduce(&svo::determiner(), &svo::noun());
    assert_eq!(result, Some(LambekType::np()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn backward_application() {
    // NP + NP\S → S ("dog" + "runs" → S)
    let result = reduce(&LambekType::np(), &svo::intransitive_verb());
    assert_eq!(result, Some(LambekType::s()));
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn no_reduction() {
    // N + NP → None (can't combine noun with noun phrase)
    let result = reduce(&svo::noun(), &LambekType::np());
    assert_eq!(result, None);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn adjective_noun() {
    // N/N + N → N ("big" + "dog" → N)
    let result = reduce(&svo::adjective(), &svo::noun());
    assert_eq!(result, Some(LambekType::n()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn nominal_modifier_noun_composes_with_a_head_noun() {
    // N/N + N → N ("consultation" + "services" → N) -- the bare-nominal
    // (noun-noun) compounding rule: Hockenmaier & Steedman (2005), CCGbank
    // User's Manual, MS-CIS-05-09, §3.6.1/§3.6.2 (prenominal nouns are
    // functions from nouns to nouns, combining by ordinary forward
    // application, exactly like an attributive adjective).
    let result = reduce(&svo::nominal_modifier_noun(), &svo::noun());
    assert_eq!(result, Some(LambekType::n()));
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn nominal_modifier_noun_is_structurally_identical_to_adjective() {
    // The design's central claim, made checkable: no separate category
    // exists in the literature for a premodifying noun vs. an attributive
    // adjective, so `nominal_modifier_noun` is DELIBERATELY the same
    // LambekType as `adjective` (both N/N) -- not a collision to
    // disambiguate (unlike the `NP\NP` trio, which needed a marker), since
    // `montague::apply`'s generic `N/N + N → N` concatenation composes them
    // identically either way.
    assert_eq!(svo::nominal_modifier_noun(), svo::adjective());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn transitive_verb_takes_object() {
    // (NP\S)/NP + NP → NP\S ("sees" + "dog" → VP)
    let result = reduce(&svo::transitive_verb(), &LambekType::np());
    assert_eq!(result, Some(svo::intransitive_verb()));
}

// =============================================================================
// Tokenizer tests — text to typed tokens via lexicon
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn tokenize_simple() {
    let tokens = tokenize::tokenize("the dog runs", &sample_lang());
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].word, "the");
    assert_eq!(tokens[0].lambek_type, svo::determiner());
    assert_eq!(tokens[1].word, "dog");
    assert_eq!(tokens[1].lambek_type, svo::noun());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn tokenize_strips_punctuation() {
    let tokens = tokenize::tokenize("the dog runs.", &sample_lang());
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[2].word, "runs");
}

// =============================================================================
// Copula + adjective tests
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn a_dog_is_big() {
    // Lexical primaries stay LEXICAL — no destructive rewrite: is → the
    // medial copula, big → the attributive N/N (first loaded Adjective row).
    // The PREDICATIVE reading (CCGbank §3.4: copula_adj (S[dcl]\NP)/(S[adj]\NP)
    // + predicate adjective S[adj]\NP) rides the ALTERNATIVES — every entry
    // contributes all its loaded category rows, and a medial copula offers
    // copula_adj additively — so the chart, not token order, decides.
    let (tokens, alternatives) =
        tokenize::tokenize_with_alternatives("a dog is big", &sample_lang());
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0].lambek_type, svo::determiner()); // a
    assert_eq!(tokens[1].lambek_type, svo::noun()); // dog
    assert_eq!(tokens[2].lambek_type, svo::copula()); // is (lexical primary)
    assert_eq!(tokens[3].lambek_type, svo::adjective()); // big (attributive primary)
    assert!(
        alternatives[2].contains(&svo::copula_adj()),
        "the medial copula offers the predicative-complement reading; got {:?}",
        alternatives[2]
    );
    assert!(
        alternatives[3].contains(&svo::predicate_adjective()),
        "the adjective carries its loaded predicative row; got {:?}",
        alternatives[3]
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn a_dog_is_big_reduces() {
    // The predicative statement derives S[dcl] through the CHART over the
    // merged primary+alternative type sets (the same merge the chat applies).
    let (tokens, alternatives) =
        tokenize::tokenize_with_alternatives("a dog is big", &sample_lang());
    let words: Vec<String> = tokens.iter().map(|t| t.word.clone()).collect();
    let type_sets: Vec<Vec<LambekType>> = tokens
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let mut types = vec![t.lambek_type.clone()];
            if let Some(alts) = alternatives.get(i) {
                for alt in alts {
                    if !types.contains(alt) {
                        types.push(alt.clone());
                    }
                }
            }
            types
        })
        .collect();
    let result = chart_reduce(&words, &type_sets);
    assert!(
        result.success,
        "expected S via the predicative alternatives, got {:?}",
        result.remaining
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn spelling_correction_teh() {
    // "teh" is distance 1 from "the" — performance error (transposition)
    let tokens = tokenize::tokenize("teh dog runs", &sample_lang());
    assert_eq!(tokens[0].lambek_type, svo::determiner());
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn an_equidistant_ambiguous_misspelling_stays_unresolved_rather_than_guess() {
    // "byg" is distance 1 from BOTH "big" (i->y) and "bug" (u->y) -- a
    // confirmed real failure mode without this guard: "medicad" (distance 1
    // from both "medicaid" and "medical") got silently, confidently
    // "corrected" to the WRONG word. With no language-model prior to break
    // the tie, staying unresolved is the honest behavior, not a guess.
    let tokens = tokenize::tokenize("byg", &sample_lang());
    assert_eq!(
        tokens[0].word, "byg",
        "an ambiguous equidistant misspelling must NOT be silently corrected \
         to either candidate: {:?}",
        tokens[0]
    );
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn spelling_correction_propagates_the_corrected_surface_not_just_the_type() {
    // A confirmed real gap: a misspelling used to reach a correct TYPE (via
    // assign_type's own noisy-channel fallback) while the misspelled SURFACE
    // itself survived into the token, so downstream entity/definition
    // resolution never benefited — only the parse did. "teh" must now
    // surface as "the" itself, not merely type as a determiner.
    let tokens = tokenize::tokenize("teh dog runs", &sample_lang());
    assert_eq!(
        tokens[0].word, "the",
        "the corrected SURFACE must reach the token, not just its type"
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn what_is_a_dog() {
    let tokens = tokenize::tokenize("what is a dog", &sample_lang());
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0].lambek_type, svo::wh_what()); // what
}

// =============================================================================
// Type notation tests
// =============================================================================

#[pr4xis::praxis_value(Explainable)]
#[test]
fn type_notation() {
    assert_eq!(LambekType::s().notation(), "S");
    assert_eq!(LambekType::np().notation(), "NP");
    assert_eq!(svo::determiner().notation(), "NP/N");
    assert_eq!(svo::intransitive_verb().notation(), "NP\\S");
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
            let result = reduce(&svo::determiner(), &svo::noun());
            prop_assert_eq!(result, Some(LambekType::np()));
        }

        /// NP + intransitive verb always gives S.
        #[test]
        fn prop_np_iv_gives_s(_dummy in 0..1i32) {
            let result = reduce(&LambekType::np(), &svo::intransitive_verb());
            prop_assert_eq!(result, Some(LambekType::s()));
        }
    }

    pr4xis::register_praxis_value!(prop_forward_application, Verifiable);
    pr4xis::register_praxis_value!(prop_backward_application, Verifiable);
    pr4xis::register_praxis_value!(prop_atoms_dont_reduce, Honest);
    pr4xis::register_praxis_value!(prop_det_noun_gives_np, Verifiable);
    pr4xis::register_praxis_value!(prop_np_iv_gives_s, Verifiable);
}

// =============================================================================
// Montague functor tests — type-driven interpretation
// =============================================================================

use super::montague;

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

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn montague_the_dog_runs() {
    // the:NP/N + dog:N + runs:NP\S → S
    // Semantics: the(dog) = entity, runs(entity) = proposition
    let en = sample_english();
    let tokens = vec![
        TypedToken {
            expression_use: ExpressionUse::Used,
            word: "the".into(),
            lambek_type: svo::determiner(),
        },
        TypedToken {
            expression_use: ExpressionUse::Used,
            word: "dog".into(),
            lambek_type: svo::noun(),
        },
        TypedToken {
            expression_use: ExpressionUse::Used,
            word: "runs".into(),
            lambek_type: svo::intransitive_verb(),
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

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn montague_she_sees_the_dog() {
    let en = sample_english();
    let tokens = vec![
        TypedToken {
            expression_use: ExpressionUse::Used,
            word: "she".into(),
            lambek_type: svo::proper_noun(),
        },
        TypedToken {
            expression_use: ExpressionUse::Used,
            word: "sees".into(),
            lambek_type: svo::transitive_verb(),
        },
        TypedToken {
            expression_use: ExpressionUse::Used,
            word: "the".into(),
            lambek_type: svo::determiner(),
        },
        TypedToken {
            expression_use: ExpressionUse::Used,
            word: "dog".into(),
            lambek_type: svo::noun(),
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

/// Regression: a ditransitive verb (`((NP\S)/NP)/NP`) absorbs TWO arguments
/// via the function-result branch of `apply` before reaching an atomic `S`
/// result -- "she gives Mary cake" first absorbs "Mary" (leaving
/// `Func{"gives", body:[Mary]}` at type `(NP\S)/NP`), THEN absorbs "cake" on
/// the SECOND pass through the same branch. A `Box<Sem>`-bodied `Func`
/// silently overwrote the first absorbed argument ("Mary") with the second
/// ("cake") on that second pass -- confirmed by hand-tracing before the
/// `Vec<Sem>`-bodied fix. All three arguments (both objects plus the
/// subject, absorbed last via backward application) must survive, in
/// absorption order.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn montague_ditransitive_verb_keeps_every_absorbed_argument() {
    let en = sample_english();
    let tokens = vec![
        TypedToken {
            expression_use: ExpressionUse::Used,
            word: "she".into(),
            lambek_type: svo::proper_noun(),
        },
        TypedToken {
            expression_use: ExpressionUse::Used,
            word: "gives".into(),
            lambek_type: svo::ditransitive_verb(),
        },
        TypedToken {
            expression_use: ExpressionUse::Used,
            word: "mary".into(),
            lambek_type: svo::proper_noun(),
        },
        TypedToken {
            expression_use: ExpressionUse::Used,
            word: "cake".into(),
            // Ditransitive_verb's two argument slots are both NP (Lambek
            // types::svo::ditransitive_verb = ((NP\S)/NP)/NP); "cake" is
            // typed NP here (rather than the bare N a real "the cake" would
            // reduce from) purely so this stays a minimal plain-application
            // fixture -- the point under test is argument survival, not
            // determiner grammar.
            lambek_type: svo::proper_noun(),
        },
    ];
    let sem = montague::interpret(&tokens, &en);
    match &sem {
        montague::Sem::Prop {
            predicate,
            arguments,
        } => {
            assert_eq!(predicate, "gives");
            let names: Vec<String> = arguments.iter().map(|a| a.describe()).collect();
            assert_eq!(
                names,
                vec!["mary".to_string(), "cake".to_string(), "she".to_string()],
                "the first-absorbed argument (mary) must survive the second \
                 function-result reduction (absorbing cake), not be silently \
                 overwritten by it -- got {names:?}"
            );
        }
        other => panic!("expected Prop, got {:?}", other),
    }
}

/// The object-question `what` category ([`svo::wh_what_object`]) and the
/// do-support category ([`svo::does_support`]) compose correctly through
/// `montague::interpret` -- "what does deadly mean" reduces to a
/// `Question{predicate:"what", illocution:Content}` naming exactly one
/// queried entity (the definiendum "deadly"), buried two Func-absorption
/// levels deep (does absorbs the subject, THEN absorbs mean) -- reachable
/// only because of the `montague_ditransitive_verb_keeps_every_absorbed_
/// argument` fix above; before it, the subject NP would have been silently
/// dropped and `argument_leaf` would have found nothing to define. This is
/// the type-system half of Slice B's "what does {adverb} mean" frame
/// (`.notes/chat-fix-c-build-state.md`, Slice B) -- the tokenizer/lexicon
/// wiring (do-support surface gating, the quoted-mention NP glyph
/// inventory, `Frames::wh_mean`) is separate, larger follow-up work; this
/// proves the categories themselves reduce and compose the right semantics.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn montague_what_does_x_mean_is_a_content_question_naming_one_entity() {
    let en = sample_english();
    let tokens = vec![
        TypedToken {
            expression_use: ExpressionUse::Used,
            word: "what".into(),
            lambek_type: svo::wh_what_object(),
        },
        TypedToken {
            expression_use: ExpressionUse::Used,
            word: "does".into(),
            lambek_type: svo::does_support(),
        },
        TypedToken {
            expression_use: ExpressionUse::Used,
            word: "deadly".into(),
            lambek_type: svo::proper_noun(),
        },
        TypedToken {
            expression_use: ExpressionUse::Used,
            word: "mean".into(),
            lambek_type: svo::bare_transitive_verb(),
        },
    ];
    let sem = montague::interpret(&tokens, &en);
    match &sem {
        montague::Sem::Question {
            predicate,
            arguments,
            illocution,
        } => {
            assert_eq!(predicate, "what");
            assert_eq!(*illocution, montague::QuestionIllocution::Content);
            let entities: Vec<String> = arguments
                .iter()
                .filter_map(montague::Sem::argument_name)
                .collect();
            assert_eq!(
                entities,
                vec!["deadly".to_string()],
                "exactly one queried entity (the definiendum), found through \
                 two levels of Func absorption; got {arguments:?}"
            );
        }
        other => panic!("expected Question, got {:?}", other),
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn montague_the_big_dog_runs() {
    let en = sample_english();
    let tokens = vec![
        TypedToken {
            expression_use: ExpressionUse::Used,
            word: "the".into(),
            lambek_type: svo::determiner(),
        },
        TypedToken {
            expression_use: ExpressionUse::Used,
            word: "big".into(),
            lambek_type: svo::adjective(),
        },
        TypedToken {
            expression_use: ExpressionUse::Used,
            word: "dog".into(),
            lambek_type: svo::noun(),
        },
        TypedToken {
            expression_use: ExpressionUse::Used,
            word: "runs".into(),
            lambek_type: svo::intransitive_verb(),
        },
    ];
    let sem = montague::interpret(&tokens, &en);
    match &sem {
        montague::Sem::Prop { .. } => {} // should produce a proposition
        other => panic!("expected Prop, got {:?}", other),
    }
}

#[pr4xis::praxis_value(Explainable)]
#[test]
fn montague_describe() {
    let en = sample_english();
    let tokens = vec![
        TypedToken {
            expression_use: ExpressionUse::Used,
            word: "the".into(),
            lambek_type: svo::determiner(),
        },
        TypedToken {
            expression_use: ExpressionUse::Used,
            word: "dog".into(),
            lambek_type: svo::noun(),
        },
        TypedToken {
            expression_use: ExpressionUse::Used,
            word: "runs".into(),
            lambek_type: svo::intransitive_verb(),
        },
    ];
    let sem = montague::interpret(&tokens, &en);
    let desc = sem.describe();
    // Should be something like "runs(dog)" or "runs(dog, ...)"
    assert!(!desc.is_empty());
}
