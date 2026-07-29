#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

// Integration tests — full WordNet, real pipeline, no sample data.
//
// These tests load the complete English WordNet (107K concepts) and test
// the ACTUAL pipeline. If these fail, the chatbot fails. No lies.

#[cfg(test)]
mod tests {
    use crate::cognitive::linguistics::english::{English, english_loaded};
    use crate::cognitive::linguistics::lambek::reduce::chart_reduce;
    use crate::cognitive::linguistics::lambek::tokenize;

    /// Full English — loaded ONCE per process via the shared `english_loaded()`
    /// fast path: the content-addressed compact `.prx` archive when present
    /// (~ms), else the WN-LMF XML parse. Under nextest's process-per-test model
    /// each test re-enters this, so the compact archive is what keeps it cheap.
    fn english() -> &'static English {
        english_loaded()
    }

    fn tokens_debug(en: &English, input: &str) -> String {
        let tokens = tokenize::tokenize(input, en);
        tokens
            .iter()
            .map(|t| format!("{}:{}", t.word, t.lambek_type.notation()))
            .collect::<Vec<_>>()
            .join("  ")
    }

    fn parses(en: &English, input: &str) -> bool {
        // Use chart parser with ALL types per word (Goodman 1999, Moroz 2009).
        // The grammar tries all type combinations simultaneously.
        let (tokens, alternatives) = tokenize::tokenize_with_alternatives(input, en);
        let words: Vec<String> = tokens.iter().map(|t| t.word.clone()).collect();
        let type_sets: Vec<Vec<crate::cognitive::linguistics::lambek::types::LambekType>> = tokens
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
        chart_reduce(&words, &type_sets).success
    }

    fn parses_as_question(en: &English, input: &str) -> bool {
        // Chart parser with ALL types per word.
        let (tokens, alternatives) = tokenize::tokenize_with_alternatives(input, en);
        let words: Vec<String> = tokens.iter().map(|t| t.word.clone()).collect();
        let type_sets: Vec<Vec<crate::cognitive::linguistics::lambek::types::LambekType>> = tokens
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
        // Check if the chart derives a question type (S[q] or S[wq])
        result.success
            && result.final_type.as_ref().is_some_and(|t| {
                matches!(
                    t,
                    crate::cognitive::linguistics::lambek::types::LambekType::Atom(
                        crate::cognitive::linguistics::lambek::types::AtomicType::S(Some(
                            crate::cognitive::linguistics::lambek::types::SentenceFeature::Q
                                | crate::cognitive::linguistics::lambek::types::SentenceFeature::Wq
                        ))
                    )
                )
            })
    }

    // =========================================================================
    // These MUST pass — they're what the chatbot needs to work
    // =========================================================================

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_dog_runs() {
        let en = english();
        assert!(
            parses(en, "the dog runs"),
            "FAILED: {}",
            tokens_debug(en, "the dog runs")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_big_dog_runs() {
        let en = english();
        assert!(
            parses(en, "the big dog runs"),
            "FAILED: {}",
            tokens_debug(en, "the big dog runs")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn chart_parses_question() {
        let en = english();
        let (tokens, alts) = tokenize::tokenize_with_alternatives("is a dog a mammal", en);
        let words: Vec<String> = tokens.iter().map(|t| t.word.clone()).collect();
        let type_sets: Vec<Vec<crate::cognitive::linguistics::lambek::types::LambekType>> = tokens
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let mut types = vec![t.lambek_type.clone()];
                if let Some(a) = alts.get(i) {
                    for alt in a {
                        if !types.contains(alt) {
                            types.push(alt.clone());
                        }
                    }
                }
                types
            })
            .collect();
        eprintln!("Chart input:");
        for (w, ts) in words.iter().zip(type_sets.iter()) {
            let notations: Vec<_> = ts.iter().map(|t| t.notation()).collect();
            eprintln!("  {}: {:?}", w, notations);
        }
        let result = chart_reduce(&words, &type_sets);
        eprintln!(
            "Chart result: success={}, type={:?}",
            result.success,
            result.final_type.as_ref().map(|t| t.notation())
        );
        assert!(result.success, "chart should parse 'is a dog a mammal'");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn is_a_dog_a_mammal() {
        let en = english();
        assert!(
            parses_as_question(en, "is a dog a mammal"),
            "FAILED: {}",
            tokens_debug(en, "is a dog a mammal")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn is_a_dog_an_animal() {
        let en = english();
        assert!(
            parses_as_question(en, "is a dog an animal"),
            "FAILED: {}",
            tokens_debug(en, "is a dog an animal")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn what_is_a_dog() {
        let en = english();
        assert!(
            parses_as_question(en, "what is a dog"),
            "FAILED: {}",
            tokens_debug(en, "what is a dog")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn what_is_able_backtracks_the_predicative_reading() {
        // The FIX-B adjective-define frame end-to-end at the CHART level: the
        // whole-span goal is S[wq], and the winning derivation's lexical
        // assignment is the one the Montague extractor can re-reduce and
        // extract the definiendum from — what:wh, is:copula_adj,
        // able:predicate_adjective. The min-cost backtrack must not trade it
        // for a same-goal derivation whose leaves strand the extractor
        // (the gate-1 regression: define 932→1043, "what is able" abstained).
        let en = english();
        let (tokens, alternatives) = tokenize::tokenize_with_alternatives("what is able", en);
        let words: Vec<String> = tokens.iter().map(|t| t.word.clone()).collect();
        let type_sets: Vec<Vec<crate::cognitive::linguistics::lambek::types::LambekType>> = tokens
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
        let sets_debug: Vec<String> = words
            .iter()
            .zip(type_sets.iter())
            .map(|(w, ts)| {
                format!(
                    "{w}:{{{}}}",
                    ts.iter()
                        .map(|t| t.notation())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
            .collect();
        let result = chart_reduce(&words, &type_sets);
        let remaining_debug: Vec<String> = result
            .remaining
            .iter()
            .map(|t| format!("{}:{}", t.word, t.lambek_type.notation()))
            .collect();
        assert!(
            result.success
                && result.final_type
                    == Some(crate::cognitive::linguistics::lambek::types::LambekType::wq())
                && result.unary_steps == 0
                && result.remaining[1].lambek_type
                    == crate::cognitive::linguistics::lambek::types::svo::copula_adj()
                && result.remaining[2].lambek_type
                    == crate::cognitive::linguistics::lambek::types::svo::predicate_adjective(),
            "sets: {sets_debug:?}\nfinal: {:?}\nremaining: {remaining_debug:?}",
            result.final_type.map(|t| t.notation()),
        );
    }

    /// SLICE-C (R-2): the predicative reading reaches a VERB-FIRST adjective
    /// homonym through the CHART ALTERNATIVES, not a token rewrite. "shy" is
    /// verb-first in the loaded lexicon, so the deleted destructive
    /// `assign_predicate_adjectives` (which fired only when the NEXT token's
    /// PRIMARY type was the attributive adjective) never reached it — "what
    /// is shy" parsed degenerately and the chat answered with the gloss of
    /// "is". Now: every lexicon entry contributes ALL its loaded category
    /// rows as alternatives (the loader's documented multi-row contract) and
    /// a medial copula offers `copula_adj` additively, so the chart derives
    /// the S[wq] predicative reading and backtracks the extractable leaves.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn what_is_shy_backtracks_the_predicative_reading() {
        let en = english();
        let (tokens, alternatives) = tokenize::tokenize_with_alternatives("what is shy", en);
        let words: Vec<String> = tokens.iter().map(|t| t.word.clone()).collect();
        let type_sets: Vec<Vec<crate::cognitive::linguistics::lambek::types::LambekType>> = tokens
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
        // The alternatives half: "shy" (verb-first) carries BOTH loaded
        // adjective rows; the medial copula carries `copula_adj`.
        assert!(
            type_sets[2].contains(&crate::cognitive::linguistics::lambek::types::svo::adjective())
                && type_sets[2].contains(
                    &crate::cognitive::linguistics::lambek::types::svo::predicate_adjective()
                ),
            "the verb-first adjective homonym carries all loaded rows; got {:?}",
            type_sets[2]
        );
        assert!(
            type_sets[1].contains(&crate::cognitive::linguistics::lambek::types::svo::copula_adj()),
            "the medial copula offers the predicative-complement reading; got {:?}",
            type_sets[1]
        );
        // The chart half: the predicative S[wq] derivation wins with the
        // extractable leaves.
        let result = chart_reduce(&words, &type_sets);
        assert!(
            result.success
                && result.final_type
                    == Some(crate::cognitive::linguistics::lambek::types::LambekType::wq())
                && result.remaining[1].lambek_type
                    == crate::cognitive::linguistics::lambek::types::svo::copula_adj()
                && result.remaining[2].lambek_type
                    == crate::cognitive::linguistics::lambek::types::svo::predicate_adjective(),
            "final: {:?}, remaining: {:?}",
            result.final_type.as_ref().map(|t| t.notation()),
            result
                .remaining
                .iter()
                .map(|t| format!("{}:{}", t.word, t.lambek_type.notation()))
                .collect::<Vec<_>>(),
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_gerund_carries_the_nominal_reading() {
        // The analysis half of the verb-question slice: "coughing" resolves
        // through the dual-route lemmatizer to the verb lemma "cough", and
        // the form-level OLiA class (`ing`, the EAGLES gerund-participle
        // merger) projects the gerundial-nominal NP reading through the
        // loaded OLiA→CCG functor (CCGbank Manual App. B.4.1: gerund
        // subjects are treated like NPs).
        let en = english();
        let (tokens, alternatives) = tokenize::tokenize_with_alternatives("coughing", en);
        let mut types = vec![tokens[0].lambek_type.clone()];
        for alt in &alternatives[0] {
            if !types.contains(alt) {
                types.push(alt.clone());
            }
        }
        assert!(
            types.contains(&crate::cognitive::linguistics::lambek::types::LambekType::np()),
            "no NP reading for the gerund; got {:?}",
            types.iter().map(|t| t.notation()).collect::<Vec<_>>()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn is_coughing_exhaling_parses_as_a_question() {
        // The verb-hypernymy polar frame in gerundial nominalization
        // (Fellbaum 1998 troponymy; CGEL bare gerunds as subject and
        // predicative complement): pure application over the position-0
        // question copula — (S[q]/NP)/NP + NP + NP → S[q].
        let en = english();
        assert!(
            parses_as_question(en, "is coughing exhaling"),
            "FAILED: {}",
            tokens_debug(en, "is coughing exhaling")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn what_is_exhaling_parses_as_a_question() {
        // The verb-define frame in gerundial nominalization: what + [is
        // exhaling] — the copula takes the gerundial NP complement.
        let en = english();
        assert!(
            parses_as_question(en, "what is exhaling"),
            "FAILED: {}",
            tokens_debug(en, "what is exhaling")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_dog_is_big() {
        let en = english();
        assert!(
            parses(en, "a dog is big"),
            "FAILED: {}",
            tokens_debug(en, "a dog is big")
        );
    }

    // =========================================================================
    // Extended sentence suite — the grammar must handle these
    // =========================================================================

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn she_sees_the_dog() {
        let en = english();
        assert!(
            parses(en, "she sees the dog"),
            "FAILED: {}",
            tokens_debug(en, "she sees the dog")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_cat_runs() {
        let en = english();
        assert!(
            parses(en, "the cat runs"),
            "FAILED: {}",
            tokens_debug(en, "the cat runs")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_big_cat_runs() {
        let en = english();
        assert!(
            parses(en, "a big cat runs"),
            "FAILED: {}",
            tokens_debug(en, "a big cat runs")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_dog_sees_the_cat() {
        let en = english();
        assert!(
            parses(en, "the dog sees the cat"),
            "FAILED: {}",
            tokens_debug(en, "the dog sees the cat")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn is_a_cat_an_animal() {
        let en = english();
        assert!(
            parses_as_question(en, "is a cat an animal"),
            "FAILED: {}",
            tokens_debug(en, "is a cat an animal")
        );
    }

    // `what_is_a_cat` (#71 chart-alternatives gap): "cat" gets a rare verb
    // sense from WordNet alongside its noun sense; the chart parser used to
    // try only one type per token, so the verb sense could starve out the
    // noun reading the wh-question needs. `tokenize_with_alternatives`
    // (tokenize.rs) now gathers every lexicon entry and every loaded
    // category row per token as chart alternatives, so this should parse.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn what_is_a_cat() {
        let en = english();
        assert!(
            parses_as_question(en, "what is a cat"),
            "FAILED: {}",
            tokens_debug(en, "what is a cat")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_big_dog_sees_the_small_cat() {
        let en = english();
        assert!(
            parses(en, "the big dog sees the small cat"),
            "FAILED: {}",
            tokens_debug(en, "the big dog sees the small cat")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_dog_is_an_animal() {
        let en = english();
        assert!(
            parses(en, "a dog is an animal"),
            "FAILED: {}",
            tokens_debug(en, "a dog is an animal")
        );
    }

    // `is_a_dog_big` (#71 copula_adj-vs-question-copula conflict):
    // predicate-adjective questions used to lose out to a destructive
    // copula_adj rewrite that clobbered the question-copula type.
    // `copula_adj` is now offered additively as a chart alternative
    // (tokenize.rs) whenever the primary copula type is medial, so both
    // readings should coexist and the question form should parse.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn is_a_dog_big() {
        let en = english();
        assert!(
            parses_as_question(en, "is a dog big"),
            "FAILED: {}",
            tokens_debug(en, "is a dog big")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn she_runs() {
        let en = english();
        assert!(
            parses(en, "she runs"),
            "FAILED: {}",
            tokens_debug(en, "she runs")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn he_sees_her() {
        let en = english();
        assert!(
            parses(en, "he sees her"),
            "FAILED: {}",
            tokens_debug(en, "he sees her")
        );
    }

    // =========================================================================
    // Math operators (#169) — the tokenizer must KEEP loaded operator glyphs,
    // and the loaded type must reduce the whole sentence. Co-authored-by:
    // awfmilton (the #169 bug report + research framed this).
    // =========================================================================

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn hyphenated_and_slashed_words_are_not_split_by_operator_glyphs() {
        // `-` and `/` are operator glyphs, but a glyph BETWEEN two letters is
        // internal punctuation, not a math operator: `well-being` / `and/or` /
        // `x-ray` stay single tokens, while `10+10` still splits (Copilot #2).
        let en = english();
        for (input, kept) in [
            ("the well-being of dogs", "well-being"),
            ("cats and/or dogs", "and/or"),
            ("an x-ray", "x-ray"),
        ] {
            let words: Vec<String> = tokenize::tokenize(input, en)
                .iter()
                .map(|t| t.word.clone())
                .collect();
            assert!(
                words.iter().any(|w| w == kept),
                "{kept:?} must survive as one token in {input:?}; got {words:?}"
            );
        }
        // ...but a math glyph between digits still splits.
        let math: Vec<String> = tokenize::tokenize("10+10", en)
            .iter()
            .map(|t| t.word.clone())
            .collect();
        assert_eq!(math, vec!["10", "+", "10"]);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn operator_glyph_survives_tokenization() {
        // The bug: `+` was stripped as punctuation, leaving "what is 10 10".
        // Now `+` is a token carrying its loaded type `(NP\NP)/NP`.
        let en = english();
        let words: Vec<String> = tokenize::tokenize("what is 10 + 10", en)
            .iter()
            .map(|t| t.word.clone())
            .collect();
        assert!(
            words.iter().any(|w| w == "+"),
            "the operator glyph must survive tokenization; got {words:?}"
        );
        assert_eq!(
            words,
            vec!["what", "is", "10", "+", "10"],
            "operator and numbers are all preserved"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn what_is_10_plus_10() {
        // The headline #169 case: a wh-question whose complement is an
        // arithmetic expression reduces to S[wq].
        let en = english();
        assert!(
            parses_as_question(en, "what is 10 + 10"),
            "FAILED: {}",
            tokens_debug(en, "what is 10 + 10")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn what_is_10_plus_10_glued() {
        // Glued form: the tokenizer splits "10+10" at the loaded glyph.
        let en = english();
        assert!(
            parses_as_question(en, "what is 10+10"),
            "FAILED: {}",
            tokens_debug(en, "what is 10+10")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_relation_reduces_to_a_sentence() {
        // A relational operator's result sort is `truth` → `(NP\S)/NP`, so
        // "10 < 20" reduces to a sentence (a proposition), not a noun phrase.
        // Spaced and glued both split at the loaded glyph.
        let en = english();
        for s in ["10 < 20", "10<20"] {
            assert!(parses(en, s), "FAILED: {}", tokens_debug(en, s));
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn iso_unicode_operators_parse() {
        // The full vocabulary the bundle loads includes the ISO 80000-2 glyphs
        // (× ÷ − ≤ ≥ ≠), recognized exactly like the ASCII forms — end-to-end,
        // not merely loaded.
        let en = english();
        for s in ["what is 10 × 10", "what is 10 ÷ 2"] {
            assert!(parses_as_question(en, s), "FAILED: {}", tokens_debug(en, s));
        }
        for s in ["10 ≤ 20", "10 ≥ 5", "10 ≠ 20"] {
            assert!(parses(en, s), "FAILED: {}", tokens_debug(en, s));
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_trailing_question_mark_still_trims() {
        // The tokenizer gate is "is this a LOADED operator glyph?", not
        // is_ascii_punctuation — so a trailing "?" is NOT mistaken for an
        // operator: it still trims, and the arithmetic question still parses.
        let en = english();
        assert!(
            parses_as_question(en, "what is 10 + 10?"),
            "FAILED: {}",
            tokens_debug(en, "what is 10 + 10?")
        );
    }

    // =========================================================================
    // Unicode dash-punctuation trimming — a confirmed tokenizer bug: `flush_
    // word`'s trim predicate was a raw `char::is_ascii_punctuation()` check,
    // which is FALSE for every non-ASCII Unicode dash, so real U.S. Code
    // legal prose carrying an EM DASH after a defining verb (`the term X
    // means—`, a common USC drafting convention introducing an enumerated
    // list) left the malformed token "means—" in the stream, missing lexicon
    // lookup for the bare word "means". Fixed by querying the loaded
    // `DashPunctuationVocabulary` (Unicode General_Category=Pd, 25 members)
    // instead of the ASCII-only check.
    // =========================================================================

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn an_em_dash_trailing_a_defining_verb_still_trims() {
        // Modeled on the real USC drafting pattern: `the term "X" means—`
        // introducing an enumerated list.
        let en = english();
        let input = "The term \"developmental disability\" means\u{2014}";
        let words: Vec<String> = tokenize::tokenize(input, en)
            .iter()
            .map(|t| t.word.clone())
            .collect();
        assert!(
            words.iter().any(|w| w == "means"),
            "the bare word \"means\" must survive tokenization; got {words:?}"
        );
        assert!(
            !words.iter().any(|w| w.contains('\u{2014}')),
            "no token may carry a leftover em dash; got {words:?}"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn other_unicode_dashes_also_trim() {
        // Not overfit to the one glyph the bug report named — the fix loads
        // the WHOLE Pd set. En dash (U+2013) and horizontal bar (U+2015),
        // both real General_Category=Pd members distinct from the em dash.
        let en = english();
        for dash in ['\u{2013}', '\u{2015}'] {
            let input = format!("the dog runs{dash}");
            let words: Vec<String> = tokenize::tokenize(&input, en)
                .iter()
                .map(|t| t.word.clone())
                .collect();
            assert!(
                words.iter().any(|w| w == "runs"),
                "{dash:?}: the bare word \"runs\" must survive; got {words:?}"
            );
        }
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_dash_glued_word_still_tokenizes_cleanly_mid_sentence() {
        // End-to-end sanity check alongside the dedicated unit test for
        // `is_sentence_terminal_char` in `tokenize`'s own test module (a
        // dash never carries `is_sentence_ending()`): a dash-trailing word
        // followed by more text still yields a clean token stream, with no
        // leftover dash glyph on any token.
        let en = english();
        let input = "the dog runs\u{2014} fast";
        let words: Vec<String> = tokenize::tokenize(input, en)
            .iter()
            .map(|t| t.word.clone())
            .collect();
        assert_eq!(words, vec!["the", "dog", "runs", "fast"]);
    }

    // =========================================================================
    // Wh-questions via the loaded OLiA→CCG functor (no Rust wh-word list, no
    // wh_what() constant): the word→OLiA-class binding is loaded data, the
    // class→category projection is the loaded Connection functor, and the
    // category reduces the sentence. Pronoun / determiner / adverb each.
    // =========================================================================

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn what_is_a_dog_via_the_functor() {
        // Pronoun: "what" → InterrogativePronoun → S[wq]/(NP\S), through the
        // loaded functor (this is the same surface as what_is_a_dog, asserted
        // here as the functor path's pronoun case).
        let en = english();
        assert!(
            parses_as_question(en, "what is a dog"),
            "FAILED: {}",
            tokens_debug(en, "what is a dog")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn which_dog_is_a_mammal() {
        // Determiner: "which" → InterrogativeDeterminer → (S[wq]/(NP\S))/N.
        let en = english();
        assert!(
            parses_as_question(en, "which dog is a mammal"),
            "FAILED: {}",
            tokens_debug(en, "which dog is a mammal")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn where_is_the_dog() {
        // Adverb: "where" → InterrogativeAdverb → S[wq]/(S[q]/PP); the fronted
        // adverb licenses the inverted PP-gap copula (S[q]/PP)/NP so it reduces.
        let en = english();
        assert!(
            parses_as_question(en, "where is the dog"),
            "FAILED: {}",
            tokens_debug(en, "where is the dog")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn when_why_how_questions_reduce() {
        // The rest of the interrogative adverbs, same mechanism (#169 follow-on).
        let en = english();
        for s in ["when is the game", "why is the dog", "how is the cat"] {
            assert!(parses_as_question(en, s), "FAILED: {}", tokens_debug(en, s));
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn what_does_it_mean() {
        // Slice B's do-support object-question frame
        // (.notes/chat-fix-c-build-state.md): "what" offers the OBJECT-gap
        // reading S[wq]/(S[q]/NP) alongside its subject-question primary;
        // "does" (a closed do-support form) offers the bare-stem-VP
        // object-question category; "mean" (an ordinary transitive verb)
        // offers its bare-stem alternative (S[b]\NP)/NP under the same
        // wh-pronoun fronting. "it" is a pronoun (NP) -- proving the
        // category-level mechanics reduce, independent of the (separately
        // built, not-yet-wired-in) quoted-mention NP collapse that will
        // eventually put a bare content word in this slot.
        let en = english();
        assert!(
            parses_as_question(en, "what does it mean"),
            "FAILED: {}",
            tokens_debug(en, "what does it mean")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn what_does_a_curly_quoted_word_mean() {
        // The full chain: the quote-glyph collapse (tokenize.rs) mints ONE
        // NP-typed token for the mentioned word "deadly" from the loaded
        // Unicode Pi/Pf glyph vocabulary, filling the exact slot "it" fills
        // in `what_does_it_mean` above — so a real bare content word, not a
        // pronoun placeholder, now proves the grammar end to end.
        let en = english();
        let input = "what does \u{201C}deadly\u{201D} mean";
        assert!(
            parses_as_question(en, input),
            "FAILED: {}",
            tokens_debug(en, input)
        );
        let tokens = tokenize::tokenize(input, en);
        assert_eq!(
            tokens.iter().map(|t| t.word.as_str()).collect::<Vec<_>>(),
            vec!["what", "does", "deadly", "mean"],
            "the quote glyphs are stripped, the span collapses to one token"
        );
        assert_eq!(
            tokens[2].lambek_type,
            crate::cognitive::linguistics::lambek::types::svo::proper_noun(),
            "the quoted mention types NP, not deadly's ordinary adverb/adjective reading"
        );
    }

    // =========================================================================
    // Relative clauses (loaded RelativePronoun categories + forward composition).
    // Subject relatives reduce by application; the relative pronoun's category
    // comes from the loaded OLiA→CCG functor, same as everything else.
    // =========================================================================

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_subject_relative_clause_reduces() {
        // "the dog that runs" : the(NP/N) dog(N) that((NP\NP)/(NP\S)) runs(NP\S)
        //   → "the dog":NP, "that runs":NP\NP, "the dog that runs":NP, then the
        //   object of a transitive verb → a full sentence S.
        let en = english();
        assert!(
            parses(en, "she sees the dog that runs"),
            "FAILED: {}",
            tokens_debug(en, "she sees the dog that runs")
        );
    }

    // =========================================================================
    // Fronted scope-setting sentential adjuncts (defines-lens gap G1): "For
    // purposes of X," / "In this subsection," / "Except for the purposes of
    // X," / "Subject to Y," must not block the parse of the REST of the
    // sentence. Real report-cited constructions, recombined with the SAME
    // already-grammar-proven "the term 'consumer' means a natural person."
    // declarative (15 U.S.C. § 6603(h)(6)(A), which
    // `social::judicial::statute_structure::grounding`'s own test suite
    // already grounds) — isolating the adjunct-attachment mechanism from
    // SEPARATE, still-open NP-formation gaps (a bare plural noun without a
    // determiner, a proper-noun run as a determiner's complement,
    // coordination inside an NP) that keep the REAL, unmodified report
    // sentences from a full green today — see
    // `grounding::tests::a_real_unmodified_for_purposes_of_this_subsection_sentence_still_yields_no_pointer`
    // for the honest baseline. The SAME isolation precedent the S1
    // heading-shadowing fix's own test suite already establishes.
    // =========================================================================

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn for_this_purpose_the_term_x_means_y_parses() {
        // "For this purpose, " — real, byte-verified against
        // `usc_title_42-pl-119-90.xml` ("For this purpose, any evaluation of
        // such assets shall be made...").
        let en = english();
        let input = "For this purpose, the term \u{201C}consumer\u{201D} means a natural person.";
        assert!(parses(en, input), "FAILED: {}", tokens_debug(en, input));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn in_this_section_the_term_x_means_y_parses() {
        // "In this section, " — real, byte-verified against
        // `usc_title_42-pl-119-90.xml` ("In this section, the term 'fiscal
        // agent' means a carrier described in...").
        let en = english();
        let input = "In this section, the term \u{201C}consumer\u{201D} means a natural person.";
        assert!(parses(en, input), "FAILED: {}", tokens_debug(en, input));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn except_for_a_capital_offense_the_term_x_means_y_parses() {
        // "Except for a capital offense, " — real, byte-verified against
        // `usc_title_42-pl-119-90.xml` ("Except for a capital offense, no
        // individual or person shall be prosecuted...").
        let en = english();
        let input = "Except for a capital offense, the term \u{201C}consumer\u{201D} means a natural person.";
        assert!(parses(en, input), "FAILED: {}", tokens_debug(en, input));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn subject_to_this_part_the_term_x_means_y_parses() {
        // "Subject to this part, " — real, byte-verified against
        // `usc_title_42-pl-119-90.xml` ("Subject to this part, a State to
        // which a grant is made under...").
        let en = english();
        let input =
            "Subject to this part, the term \u{201C}consumer\u{201D} means a natural person.";
        assert!(parses(en, input), "FAILED: {}", tokens_debug(en, input));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn sentence_initial_for_in_except_subject_offer_the_scope_adjunct_categories() {
        // The category-level half of the fix, independent of whether the
        // REST of a given sentence happens to parse: each closed head offers
        // its new category as a chart ALTERNATIVE, additive alongside its
        // ordinary reading.
        let en = english();
        let np_variant =
            crate::cognitive::linguistics::lambek::types::svo::fronted_scope_adjunct_np();
        let pp_variant =
            crate::cognitive::linguistics::lambek::types::svo::fronted_scope_adjunct_pp();
        for (input, expected) in [
            ("For this purpose, the dog runs.", &np_variant),
            ("In this section, the dog runs.", &np_variant),
            ("Except for this part, the dog runs.", &pp_variant),
            ("Subject to this part, the dog runs.", &pp_variant),
        ] {
            let (_tokens, alternatives) = tokenize::tokenize_with_alternatives(input, en);
            assert!(
                alternatives[0].contains(expected),
                "{input:?}: expected {:?} among {:?}",
                expected.notation(),
                alternatives[0]
                    .iter()
                    .map(|t| t.notation())
                    .collect::<Vec<_>>()
            );
        }
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_medial_for_does_not_offer_the_scope_adjunct_alternative() {
        // Position-gated: only a SENTENCE-INITIAL "for"/"in"/"except"/
        // "subject" offers the new category; medially, each keeps its
        // ordinary preposition reading only, unaffected.
        let en = english();
        let np_variant =
            crate::cognitive::linguistics::lambek::types::svo::fronted_scope_adjunct_np();
        let (tokens, alternatives) =
            tokenize::tokenize_with_alternatives("she cares for the dog", en);
        let for_idx = tokens
            .iter()
            .position(|t| t.word == "for")
            .expect("'for' token present");
        assert!(
            for_idx > 0,
            "'for' must not be sentence-initial in this fixture"
        );
        assert!(
            !alternatives[for_idx].contains(&np_variant),
            "a medial 'for' must not offer the scope-adjunct alternative; got {:?}",
            alternatives[for_idx]
                .iter()
                .map(|t| t.notation())
                .collect::<Vec<_>>()
        );
    }

    // =========================================================================
    // Medial comma-delimited supplements (defines-lens gap G2): a
    // parenthetical breaking either subject-verb adjacency ("the term 'X',
    // used with respect to Y, means ...") or verb-object adjacency ("means,
    // with respect to Y, Z", the EVV headline shape) must not block the
    // parse. Real report-cited fragments, recombined with the SAME
    // already-grammar-proven "the term 'consumer' means a natural person."
    // declarative G1's own tests already isolate on — the same isolation
    // precedent.
    // =========================================================================

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn means_with_respect_to_the_evv_adjunct_the_term_x_means_y_parses() {
        // "means, with respect to personal care services or home health care
        // services," — the EVV headline shape, verbatim from 42 U.S.C.
        // § 1396b(l)(5): 'The term "electronic visit verification system"
        // means, with respect to personal care services or home health care
        // services, a system under which ...'.
        let en = english();
        let input = "The term \u{201C}consumer\u{201D} means, with respect to personal \
                      care services or home health care services, a natural person.";
        assert!(parses(en, input), "FAILED: {}", tokens_debug(en, input));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_term_x_used_with_respect_to_y_means_z_parses() {
        // ", used with respect to individuals with developmental
        // disabilities," — verbatim from the report's "inclusion" example:
        // 'The term "inclusion", used with respect to individuals with
        // developmental disabilities, means ...'.
        let en = english();
        let input = "The term \u{201C}consumer\u{201D}, used with respect to \
                      individuals with developmental disabilities, means a natural person.";
        assert!(parses(en, input), "FAILED: {}", tokens_debug(en, input));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_term_x_as_used_in_this_title_means_z_parses() {
        // ", as used in this title," — verbatim from title 18's "vessel of
        // the United States" definition (the SAME real fragment
        // `grounding::tests::a_real_parenthetical_interrupted_sample_yields_no_pointer`
        // already carries as a committed fixture): 'The term "vessel of the
        // United States", as used in this title, means ...'.
        let en = english();
        let input = "The term \u{201C}consumer\u{201D}, as used in this title, \
                      means a natural person.";
        assert!(parses(en, input), "FAILED: {}", tokens_debug(en, input));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_merged_medial_supplements_carry_the_dedicated_categories() {
        // The category-level half of the fix: the interior of each bracket
        // collapses into ONE synthetic token carrying the dedicated
        // transparent-modifier category — never left as ordinary,
        // separately-typed words that would strand the derivation.
        use crate::cognitive::linguistics::lambek::types::svo;
        let en = english();

        let post_verb = "The term \u{201C}consumer\u{201D} means, with respect to \
                          personal care services or home health care services, a natural person.";
        let tokens = tokenize::tokenize(post_verb, en);
        assert!(
            tokens
                .iter()
                .any(|t| t.lambek_type == svo::medial_supplement_verb()),
            "FAILED: {}",
            tokens_debug(en, post_verb)
        );

        let subject_verb = "The term \u{201C}consumer\u{201D}, as used in this title, \
                             means a natural person.";
        let tokens = tokenize::tokenize(subject_verb, en);
        assert!(
            tokens
                .iter()
                .any(|t| t.lambek_type == svo::medial_supplement_np()),
            "FAILED: {}",
            tokens_debug(en, subject_verb)
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_coordinated_subject_list_is_not_swept_into_a_medial_supplement() {
        // Precision guard: "The agency, the county, and the state cover
        // this." has TWO commas around a verb-followed span, but its
        // interior does not open with a closed supplement head ("the", not
        // "as"/"when"/"used") — must NOT collapse into a bogus supplement
        // (which would silently drop "the county and the state" from the
        // subject).
        use crate::cognitive::linguistics::lambek::types::svo;
        let en = english();
        let input = "The agency, the county, and the state cover this.";
        let tokens = tokenize::tokenize(input, en);
        assert!(
            !tokens.iter().any(|t| {
                t.lambek_type == svo::medial_supplement_np()
                    || t.lambek_type == svo::medial_supplement_verb()
            }),
            "a coordinated subject list must not be swept into a medial \
             supplement; got {}",
            tokens_debug(en, input)
        );
    }

    // =========================================================================
    // N-ary list-comma coordination (defines-lens gap G4(a)): real report-
    // cited sentences, full WordNet, full pipeline.
    // =========================================================================

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_real_three_item_np_coordination_parses() {
        // 42 U.S.C. § 289b–1(f)(2) "assistance" (see
        // `grounding::tests::recognizes_the_term_x_means_y_behind_a_real_three_item_np_coordination`
        // for why this real sentence, not § 300ii(5)'s own "family
        // caregiver": "family member" is a bare noun-noun compound WordNet
        // gives no adjective sense to combine on, a SEPARATE, unbuilt gap
        // that would block this parse-level test for a reason unrelated to
        // coordination).
        let en = english();
        let input = "The term \u{201C}assistance\u{201D} means a grant, contract, or \
                      cooperative agreement.";
        assert!(parses(en, input), "FAILED: {}", tokens_debug(en, input));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_real_physical_harm_bare_noun_coordination_parses() {
        // 42 U.S.C. § 3002(42) — the report's own headline coordination
        // example; needs the SCOPED bare-noun-phrase unary rule as well as
        // coordination, so this test alone (plain `tokenize::tokenize_with_alternatives`
        // + `reduce_with_alternatives`, the SHARED, unscoped table) is
        // expected NOT to parse -- `grounding::tests::
        // recognizes_the_term_x_means_y_behind_the_real_physical_harm_bare_noun_coordination`
        // proves the scoped, full end-to-end success.
        let en = english();
        let input =
            "The term \u{201C}physical harm\u{201D} means bodily injury, impairment, or disease.";
        assert!(
            !parses(en, input),
            "the SHARED, unscoped table must not carry the bare-noun-phrase \
             unary rule (a corpus-gate rejected it globally); got a parse for {}",
            tokens_debug(en, input)
        );
    }

    // =========================================================================
    // Debug: show what types the tokenizer assigns with full WordNet
    // =========================================================================

    #[pr4xis::praxis_value(Explainable)]
    #[test]
    fn debug_token_types() {
        let en = english();
        let sentences = [
            "the dog runs",
            "is a dog a mammal",
            "is a dog an animal",
            "what is a dog",
            "a dog is big",
            "she sees the dog",
        ];
        for s in sentences {
            eprintln!("  {}: {}", s, tokens_debug(en, s));
        }

        // Debug: show ALL types per word (chart input)
        for s in ["the dog runs", "is a dog a mammal", "what is a dog"] {
            eprintln!("\n  === Chart type sets: \"{}\" ===", s);
            let (tokens, alts) = tokenize::tokenize_with_alternatives(s, en);
            for (i, t) in tokens.iter().enumerate() {
                let mut all = vec![t.lambek_type.notation()];
                if let Some(a) = alts.get(i) {
                    for alt in a {
                        all.push(alt.notation());
                    }
                }
                eprintln!("    {}: [{}]", t.word, all.join(", "));
            }
        }
    }
}
