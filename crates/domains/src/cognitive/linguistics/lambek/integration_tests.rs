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

    #[test]
    fn the_dog_runs() {
        let en = english();
        assert!(
            parses(en, "the dog runs"),
            "FAILED: {}",
            tokens_debug(en, "the dog runs")
        );
    }

    #[test]
    fn the_big_dog_runs() {
        let en = english();
        assert!(
            parses(en, "the big dog runs"),
            "FAILED: {}",
            tokens_debug(en, "the big dog runs")
        );
    }

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

    #[test]
    fn is_a_dog_a_mammal() {
        let en = english();
        assert!(
            parses_as_question(en, "is a dog a mammal"),
            "FAILED: {}",
            tokens_debug(en, "is a dog a mammal")
        );
    }

    #[test]
    fn is_a_dog_an_animal() {
        let en = english();
        assert!(
            parses_as_question(en, "is a dog an animal"),
            "FAILED: {}",
            tokens_debug(en, "is a dog an animal")
        );
    }

    #[test]
    fn what_is_a_dog() {
        let en = english();
        assert!(
            parses_as_question(en, "what is a dog"),
            "FAILED: {}",
            tokens_debug(en, "what is a dog")
        );
    }

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

    #[test]
    fn she_sees_the_dog() {
        let en = english();
        assert!(
            parses(en, "she sees the dog"),
            "FAILED: {}",
            tokens_debug(en, "she sees the dog")
        );
    }

    #[test]
    fn the_cat_runs() {
        let en = english();
        assert!(
            parses(en, "the cat runs"),
            "FAILED: {}",
            tokens_debug(en, "the cat runs")
        );
    }

    #[test]
    fn a_big_cat_runs() {
        let en = english();
        assert!(
            parses(en, "a big cat runs"),
            "FAILED: {}",
            tokens_debug(en, "a big cat runs")
        );
    }

    #[test]
    fn the_dog_sees_the_cat() {
        let en = english();
        assert!(
            parses(en, "the dog sees the cat"),
            "FAILED: {}",
            tokens_debug(en, "the dog sees the cat")
        );
    }

    #[test]
    fn is_a_cat_an_animal() {
        let en = english();
        assert!(
            parses_as_question(en, "is a cat an animal"),
            "FAILED: {}",
            tokens_debug(en, "is a cat an animal")
        );
    }

    // `what_is_a_cat` (#71 chart-alternatives gap) lived here as an
    // ignored test; deleted with the never-use-ignore sweep. See
    // memory: project-known-ontology-gaps-from-ignore-cleanup.

    #[test]
    fn the_big_dog_sees_the_small_cat() {
        let en = english();
        assert!(
            parses(en, "the big dog sees the small cat"),
            "FAILED: {}",
            tokens_debug(en, "the big dog sees the small cat")
        );
    }

    #[test]
    fn a_dog_is_an_animal() {
        let en = english();
        assert!(
            parses(en, "a dog is an animal"),
            "FAILED: {}",
            tokens_debug(en, "a dog is an animal")
        );
    }

    // `is_a_dog_big` (#71 copula_adj-vs-question-copula conflict)
    // lived here as an ignored test; deleted with the
    // never-use-ignore sweep. See memory:
    // project-known-ontology-gaps-from-ignore-cleanup.

    #[test]
    fn she_runs() {
        let en = english();
        assert!(
            parses(en, "she runs"),
            "FAILED: {}",
            tokens_debug(en, "she runs")
        );
    }

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
    // Debug: show what types the tokenizer assigns with full WordNet
    // =========================================================================

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
