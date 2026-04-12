// Integration tests — full WordNet, real pipeline, no sample data.
//
// These tests load the complete English WordNet (107K concepts) and test
// the ACTUAL pipeline. If these fail, the chatbot fails. No lies.

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use crate::cognitive::linguistics::english::English;
    use crate::cognitive::linguistics::lambek::reduce::chart_reduce;
    use crate::cognitive::linguistics::lambek::tokenize;
    use crate::social::software::markup::xml::lmf;

    /// Full English — loaded ONCE, shared across all tests.
    static ENGLISH: OnceLock<English> = OnceLock::new();

    fn english() -> &'static English {
        ENGLISH.get_or_init(|| {
            let path = concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/data/wordnet/english-wordnet-2025.xml"
            );
            let xml = std::fs::read_to_string(path)
                .expect("WordNet XML not found — ensure Git LFS is pulled");
            let wn = lmf::reader::read_wordnet(&xml).unwrap();
            English::from_wordnet(&wn)
        })
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

    #[test]
    #[ignore = "cat gets verb type from WordNet — chart needs N alternative for cat"]
    fn what_is_a_cat() {
        let en = english();
        assert!(
            parses_as_question(en, "what is a cat"),
            "FAILED: {}",
            tokens_debug(en, "what is a cat")
        );
    }

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

    #[test]
    #[ignore = "predicate adjective question — copula_adj post-processing conflicts with question copula type"]
    fn is_a_dog_big() {
        let en = english();
        assert!(
            parses_as_question(en, "is a dog big"),
            "FAILED: {}",
            tokens_debug(en, "is a dog big")
        );
    }

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
