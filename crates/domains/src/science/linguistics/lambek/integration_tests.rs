// Integration tests — full WordNet, real pipeline, no sample data.
//
// These tests load the complete English WordNet (107K concepts) and test
// the ACTUAL pipeline. If these fail, the chatbot fails. No lies.

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use crate::science::linguistics::english::English;
    use crate::science::linguistics::lambek::{montague, reduce_sequence, tokenize};
    use crate::science::linguistics::language::Language;
    use crate::technology::software::markup::xml::lmf;

    /// Full English — loaded ONCE, shared across all tests.
    static ENGLISH: OnceLock<English> = OnceLock::new();

    fn english() -> &'static English {
        ENGLISH.get_or_init(|| {
            let path = concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/data/wordnet/english-wordnet-2025.xml"
            );
            if !std::path::Path::new(path).exists() {
                panic!("Full WordNet required for integration tests");
            }
            let xml = std::fs::read_to_string(path).unwrap();
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
        let tokens = tokenize::tokenize(input, en);
        let result = reduce_sequence(&tokens);
        result.success
    }

    fn parses_as_question(en: &English, input: &str) -> bool {
        let tokens = tokenize::tokenize(input, en);
        let meaning = montague::interpret(&tokens, en);
        meaning.is_question()
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
    }
}
