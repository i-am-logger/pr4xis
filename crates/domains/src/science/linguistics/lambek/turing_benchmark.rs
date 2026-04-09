// Turing Test Benchmark — questions from real Turing test competitions.
//
// Each test represents a question that a human would ask.
// Tests FAIL until the proper ontology is built to answer them.
// A failing test = a missing ontology = a research task.
//
// Sources:
// - Loebner Prize transcripts (Shieber 1994, Shah & Pell 2003)
// - Winograd Schema Challenge (Levesque, Davis, Morgenstern 2012)
// - 2014 Royal Society event (Warwick & Shah 2016)
// - Turing, Computing Machinery and Intelligence (1950)

#[cfg(test)]
mod tests {
    use crate::science::linguistics::english::English;
    use crate::science::linguistics::lambek::{montague, tokenize};
    use crate::technology::software::markup::xml::lmf;

    /// Load sample English for benchmarks.
    /// TODO: use full WordNet when embedded in binary.
    fn english() -> English {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/wordnet/english-wordnet-2025.xml"
        );
        if !std::path::Path::new(path).exists() {
            panic!("WordNet XML not found — needed for Turing benchmark");
        }
        let xml = std::fs::read_to_string(path).unwrap();
        let wn = lmf::reader::read_wordnet(&xml).unwrap();
        English::from_wordnet(&wn)
    }

    /// Process input through the full pipeline and return the semantic result.
    fn understand(en: &English, input: &str) -> montague::Sem {
        let tokens = tokenize::tokenize(input, en);
        montague::interpret(&tokens, en)
    }

    /// Check if the pipeline produces a question.
    #[allow(dead_code)]
    fn is_question(en: &English, input: &str) -> bool {
        understand(en, input).is_question()
    }

    // =========================================================================
    // LEVEL 1: Taxonomy questions — SHOULD PASS NOW
    // Ontology: WordNet taxonomy (107K concepts)
    // =========================================================================

    #[test]
    fn taxonomy_is_a_dog_a_mammal() {
        let en = english();
        let dog = en.lookup("dog");
        let mammal = en.lookup("mammal");
        assert!(!dog.is_empty() && !mammal.is_empty());
        let mut found = false;
        for &d in dog {
            for &m in mammal {
                if en.is_a(d, m) {
                    found = true;
                }
            }
        }
        assert!(found, "dog should be a mammal");
    }

    #[test]
    fn taxonomy_is_a_dog_an_animal() {
        let en = english();
        let dog = en.lookup("dog");
        let animal = en.lookup("animal");
        assert!(!dog.is_empty() && !animal.is_empty());
        let mut found = false;
        for &d in dog {
            for &a in animal {
                if en.is_a(d, a) {
                    found = true;
                }
            }
        }
        assert!(found, "dog should be an animal");
    }

    #[test]
    fn taxonomy_what_is_a_dog() {
        let en = english();
        let ids = en.lookup("dog");
        assert!(!ids.is_empty());
        let concept = en.concept(ids[0]).unwrap();
        assert!(
            !concept.definitions.is_empty(),
            "dog should have a definition"
        );
    }

    // =========================================================================
    // LEVEL 2: Grammar — parsing questions correctly
    // Ontology: Lambek/Pregroup grammar
    // =========================================================================

    #[test]
    #[ignore = "needs pregroup pipeline wired into CLI (#71)"]
    fn grammar_parses_is_question() {
        // "is a dog a mammal" should produce a Question semantic
        // Currently fails with full WordNet because verb frame data
        // changes type assignments. Needs pregroup parser (#71).
        let en = english();
        let sem = understand(&en, "is a dog a mammal");
        assert!(sem.is_question(), "got: {}", sem.describe());
    }

    #[test]
    #[ignore = "needs pregroup pipeline wired into CLI (#71)"]
    fn grammar_parses_what_question() {
        let en = english();
        let sem = understand(&en, "what is a dog");
        assert!(sem.is_question(), "got: {}", sem.describe());
    }

    // =========================================================================
    // LEVEL 3: Factual knowledge — FAILS UNTIL ONTOLOGY BUILT
    // Each test documents which ontology is needed.
    // =========================================================================

    #[test]
    #[ignore = "needs geography ontology"]
    fn factual_capital_of_france() {
        // "What is the capital of France?"
        // Needs: geography ontology with capital-of relation
        // Source: Loebner Prize (Shah & Pell 2003)
        let _en = english();
        todo!("geography ontology: capital_of(France) = Paris");
    }

    #[test]
    #[ignore = "needs literature ontology"]
    fn factual_who_wrote_hamlet() {
        // "Who wrote Hamlet?"
        // Needs: literature/authorship ontology
        // Source: Loebner Prize 2003
        todo!("literature ontology: author_of(Hamlet) = Shakespeare");
    }

    #[test]
    #[ignore = "needs mereology for animals"]
    fn factual_how_many_legs_cat() {
        // "How many legs does a cat have?"
        // Needs: mereology loaded for animal body parts + counting
        // Source: Loebner Prize 1991 (Shieber 1994)
        todo!("mereology: cat has-part leg, count = 4");
    }

    // =========================================================================
    // LEVEL 4: Reasoning — FAILS UNTIL REASONING ONTOLOGY EXTENDED
    // =========================================================================

    #[test]
    #[ignore = "needs arithmetic reasoning"]
    fn reasoning_brick_weight() {
        // "If a brick weighs one pound plus half a brick, how much does a brick weigh?"
        // Needs: arithmetic + equation solving
        // Source: Loebner Prize (Moor 2001)
        // Answer: 2 pounds (b = 1 + b/2 → b = 2)
        todo!("arithmetic reasoning ontology");
    }

    #[test]
    #[ignore = "needs arithmetic reasoning"]
    fn reasoning_apple_subtraction() {
        // "If you have three apples and I take away two, how many do you have?"
        // Needs: arithmetic + possession ontology
        // Source: Loebner Prize (Shah & Pell 2003)
        todo!("arithmetic: 3 - 2 = 1");
    }

    // =========================================================================
    // LEVEL 5: Winograd Schema — pronoun resolution via world knowledge
    // Ontology: DRT + common sense knowledge
    // =========================================================================

    #[test]
    #[ignore = "needs physical world ontology + DRT"]
    fn winograd_trophy_big() {
        // "The trophy doesn't fit in the brown suitcase because it is too big.
        //  What is too big?"
        // Answer: the trophy
        // Needs: physical object ontology (size), DRT pronoun resolution
        // Source: Winograd Schema Challenge (Levesque et al. 2012)
        todo!("physical world: big things don't fit in small things → 'it' = trophy");
    }

    #[test]
    #[ignore = "needs physical world ontology + DRT"]
    fn winograd_trophy_small() {
        // "The trophy doesn't fit in the brown suitcase because it is too small.
        //  What is too small?"
        // Answer: the suitcase
        // Source: WSC (Levesque et al. 2012)
        todo!("physical world: 'too small' = container → 'it' = suitcase");
    }

    #[test]
    #[ignore = "needs social reasoning ontology + DRT"]
    fn winograd_councilmen_feared() {
        // "The city councilmen refused the demonstrators a permit because they
        //  feared violence. Who feared violence?"
        // Answer: the councilmen
        // Source: Original Winograd (1972), WSC
        todo!("social reasoning: authority fears → authority refuses");
    }

    // =========================================================================
    // LEVEL 6: Common sense — FAILS UNTIL COMMON SENSE ONTOLOGY
    // =========================================================================

    #[test]
    #[ignore = "needs physics/material ontology"]
    fn common_sense_drop_egg() {
        // "What happens if you drop an egg on a concrete floor?"
        // Needs: material properties ontology (fragile + hard surface → break)
        // Source: Loebner Prize (French 2000)
        todo!("material ontology: egg(fragile) + concrete(hard) + drop → break");
    }

    #[test]
    #[ignore = "needs biological/sensation ontology"]
    fn common_sense_knife_painful() {
        // "Is it painful to be stabbed with a knife?"
        // Needs: sensation ontology + causation (sharp object + flesh → pain)
        // Source: Loebner Prize (Shieber 1994)
        todo!("sensation ontology: knife(sharp) + stab → pain");
    }

    // =========================================================================
    // LEVEL 7: Social / phatic — FAILS UNTIL DIALOGUE ONTOLOGY EXTENDED
    // =========================================================================

    #[test]
    #[ignore = "needs social dialogue ontology"]
    fn social_how_are_you() {
        // "How are you today?"
        // Needs: social dialogue ontology (phatic communion, Jakobson)
        // The response should be through the dialogue ontology, not hardcoded
        // Source: 2014 Royal Society event
        todo!("social dialogue: phatic greeting → appropriate phatic response");
    }

    #[test]
    #[ignore = "needs self-model ontology"]
    fn meta_are_you_a_computer() {
        // "Are you a computer?"
        // Needs: self-model ontology (praxis knowing what it IS)
        // Source: Every Loebner Prize (Shah & Pell 2003)
        todo!("self-model ontology: I am praxis, an ontological reasoning system");
    }

    #[test]
    #[ignore = "needs metacognition + self-model"]
    fn meta_what_are_you_thinking() {
        // "What are you thinking right now?"
        // Needs: metacognition (introspection) + self-model
        // Source: Loebner Prize (French 2000)
        todo!("metacognition: report current processing state ontologically");
    }
}
