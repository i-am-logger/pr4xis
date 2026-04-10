use super::response::ResponseFrame;

// Response realization — the Levelt production pipeline.
//
// This is the RIGHT ADJOINT of parsing (the generation functor).
// Parse: Text → Syntax → Semantics (left adjoint, F)
// Generate: Semantics → Syntax → Text (right adjoint, G)
//
// Levelt (1989): Conceptualizer → PreverbalMessage → Formulator → SurfaceForm
//
// The ResponseFrame is the CommunicativeGoal (Appelt 1985).
// The ResponseContent is the PreverbalMessage.
// The SentencePlan composes words in SVO order.
// The SurfaceForm is the realized text.
//
// de Groote (2001): generation in categorial grammar = beta-reduction.
// We compose NP + (NP\S)/NP + NP → S for assertions,
// (S/NP)/NP + NP + NP → S[q] for questions.
//
// References:
// - Levelt, "Speaking" (1989) — the production model
// - Reiter & Dale, "Building NLG Systems" (2000) — pipeline
// - de Groote, "Towards Abstract Categorial Grammars" (2001)
// - Appelt, "Planning English Sentences" (1985)

/// A structured response before surface realization.
/// This is the PreverbalMessage in Levelt's model.
#[derive(Debug, Clone)]
pub struct ResponseContent {
    pub frame: ResponseFrame,
    pub predicate: Option<String>,
    pub entities: Vec<String>,
    pub definitions: Vec<(String, String)>,
}

impl ResponseContent {
    pub fn new(frame: ResponseFrame) -> Self {
        Self {
            frame,
            predicate: None,
            entities: Vec::new(),
            definitions: Vec::new(),
        }
    }

    pub fn with_predicate(mut self, pred: &str) -> Self {
        self.predicate = Some(pred.into());
        self
    }

    pub fn with_entity(mut self, entity: &str) -> Self {
        self.entities.push(entity.into());
        self
    }

    pub fn with_definition(mut self, entity: &str, definition: &str) -> Self {
        self.definitions.push((entity.into(), definition.into()));
        self
    }
}

/// Realize a ResponseContent into surface text.
///
/// Levelt pipeline: Frame (goal) → Content (message) → Plan (syntax) → Surface (text).
pub fn realize(content: &ResponseContent) -> String {
    match content.frame {
        ResponseFrame::AssertKnowledge => realize_assertion(content),
        ResponseFrame::AcknowledgeGap => realize_gap(content),
        ResponseFrame::SuggestInterpretation => realize_suggestion(content),
        ResponseFrame::AdmitLimitation => realize_limitation(content),
    }
}

// ---- Sentence planning: compose from semantic roles ----
//
// SVO grammar: Subject + Verb + Object
// Copula sentences: NP + cop + NP → S[dcl]
// Existential: NP + cop + Det + N → S[dcl]
// Negation: NP + cop + "not" + Det + N → S[dcl]

/// Build a copula sentence: "{subject} is {complement}"
/// SVO: NP + (NP\S)/NP + NP → S
fn sentence_copula(subject: &str, complement: &str) -> String {
    let det = select_determiner(subject);
    let det2 = select_determiner(complement);
    let cop = select_copula(subject);
    format!("{det}{subject} {cop} {det2}{complement}")
}

/// Build a negative copula sentence: "{subject} is not {complement}"
fn sentence_copula_neg(subject: &str, complement: &str) -> String {
    let det = select_determiner(subject);
    let det2 = select_determiner(complement);
    let cop = select_copula(subject);
    format!("{det}{subject} {cop} not {det2}{complement}")
}

/// Build a copula question: "is {subject} {complement}?"
/// Inversion: (S[q]/NP)/NP + NP + NP → S[q]
fn sentence_question(subject: &str, complement: &str) -> String {
    let det = select_determiner(subject);
    let det2 = select_determiner(complement);
    let cop = select_copula(subject);
    format!("{cop} {det}{subject} {det2}{complement}?")
}

/// Select determiner: "a" for common nouns, empty for proper/mass nouns.
/// Morphology: "a" before consonant, "an" before vowel.
fn select_determiner(word: &str) -> &'static str {
    if word.is_empty() {
        return "";
    }
    let first = word.chars().next().unwrap_or('x');
    if first.is_uppercase() {
        // Proper noun — no determiner
        ""
    } else if "aeiou".contains(first) {
        "an "
    } else {
        "a "
    }
}

/// Select copula form based on subject.
fn select_copula(_subject: &str) -> &'static str {
    "is" // Default 3rd person singular present
}

// ---- Realization functions per frame ----

fn realize_assertion(content: &ResponseContent) -> String {
    if content.entities.len() >= 2 {
        let subject = &content.entities[0];
        let object = &content.entities[1];

        let is_taxonomy = content
            .predicate
            .as_ref()
            .is_some_and(|p| p == "is_a" || p == "is-a" || p == "isa");

        if is_taxonomy {
            // Generate: "Yes. {subject} is a {object}."
            let sentence = sentence_copula(subject, object);
            let mut result = format!("Yes. {sentence}.");
            for (entity, def) in &content.definitions {
                result.push_str(&format!("\n  {entity} -- {def}"));
            }
            return result;
        }

        let pred = content.predicate.as_deref().unwrap_or("relates to");
        let sentence = format!("{subject} {pred} {object}");
        return format!("Yes. {sentence}.");
    }

    if content.entities.len() == 1 {
        return realize_definition(&content.entities[0], &content.definitions);
    }

    content
        .predicate
        .as_deref()
        .unwrap_or("Understood.")
        .to_string()
}

/// Realize a negative assertion through the grammar.
pub fn realize_negation(child: &str, parent: &str) -> String {
    let sentence = sentence_copula_neg(child, parent);
    format!("No, {sentence}.")
}

fn realize_definition(word: &str, definitions: &[(String, String)]) -> String {
    if definitions.is_empty() {
        // Generate: "I know {word} but have no definition."
        // SVO: NP("I") + V("know") + NP(word) + conj("but") + V("have") + NP("no definition")
        return format!("{word}:\n  (no definitions available)",);
    }

    let mut lines = Vec::new();
    for (i, (_entity, def)) in definitions.iter().enumerate() {
        lines.push(format!("  {}. {def}", i + 1));
    }
    format!("{word}:\n{}", lines.join("\n"))
}

fn realize_gap(content: &ResponseContent) -> String {
    if content.entities.len() == 1 {
        let word = &content.entities[0];
        // "I do not know the word {word}."
        return format!("I do not know the word \"{word}\".");
    }
    if content.entities.len() > 1 {
        let words: Vec<String> = content
            .entities
            .iter()
            .map(|w| format!("\"{w}\""))
            .collect();
        return format!("I do not know the words {}.", words.join(", "));
    }
    // No entities — general gap
    "I do not have enough information to answer.".to_string()
}

fn realize_suggestion(content: &ResponseContent) -> String {
    if content.entities.len() >= 2 {
        // Generate a question from the found concepts
        let question = sentence_question(&content.entities[0], &content.entities[1]);
        return format!(
            "I found {} concepts but could not parse the sentence.\nDid you mean: {question}",
            content.entities.len()
        );
    }
    if !content.entities.is_empty() {
        let words: Vec<String> = content
            .entities
            .iter()
            .map(|w| format!("\"{w}\""))
            .collect();
        return format!(
            "I know the words {} but could not understand the sentence structure.",
            words.join(", ")
        );
    }
    "I understood some of the input but could not form a complete interpretation.".to_string()
}

fn realize_limitation(_content: &ResponseContent) -> String {
    // Generate an example question through the grammar
    let example = sentence_question("dog", "mammal");
    format!("I do not understand. Try a question like: {example}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copula_sentence_generates() {
        assert_eq!(sentence_copula("dog", "mammal"), "a dog is a mammal");
        assert_eq!(sentence_copula("cat", "animal"), "a cat is an animal");
    }

    #[test]
    fn copula_negation_generates() {
        assert_eq!(sentence_copula_neg("dog", "fish"), "a dog is not a fish");
    }

    #[test]
    fn copula_question_generates() {
        assert_eq!(sentence_question("dog", "mammal"), "is a dog a mammal?");
    }

    #[test]
    fn determiner_selection() {
        assert_eq!(select_determiner("dog"), "a ");
        assert_eq!(select_determiner("animal"), "an ");
        assert_eq!(select_determiner("Alice"), ""); // proper noun
    }

    #[test]
    fn assert_taxonomy_uses_grammar() {
        let content = ResponseContent::new(ResponseFrame::AssertKnowledge)
            .with_predicate("is_a")
            .with_entity("dog")
            .with_entity("mammal")
            .with_definition("dog", "a domesticated canine")
            .with_definition("mammal", "warm-blooded vertebrate");

        let text = realize(&content);
        assert!(text.starts_with("Yes. a dog is a mammal."));
        assert!(text.contains("dog -- a domesticated canine"));
    }

    #[test]
    fn negation_uses_grammar() {
        let text = realize_negation("dog", "fish");
        assert_eq!(text, "No, a dog is not a fish.");
    }

    #[test]
    fn definition_format() {
        let content = ResponseContent::new(ResponseFrame::AssertKnowledge)
            .with_entity("dog")
            .with_definition("dog", "a domesticated canine");

        let text = realize(&content);
        assert!(text.contains("dog:"));
        assert!(text.contains("a domesticated canine"));
    }

    #[test]
    fn gap_single_word() {
        let content = ResponseContent::new(ResponseFrame::AcknowledgeGap).with_entity("xyzzy");
        let text = realize(&content);
        assert_eq!(text, "I do not know the word \"xyzzy\".");
    }

    #[test]
    fn gap_multiple_words() {
        let content = ResponseContent::new(ResponseFrame::AcknowledgeGap)
            .with_entity("foo")
            .with_entity("bar");
        let text = realize(&content);
        assert!(text.contains("\"foo\""));
        assert!(text.contains("\"bar\""));
    }

    #[test]
    fn suggestion_generates_question() {
        let content = ResponseContent::new(ResponseFrame::SuggestInterpretation)
            .with_entity("dog")
            .with_entity("mammal");
        let text = realize(&content);
        assert!(text.contains("is a dog a mammal?"));
    }

    #[test]
    fn limitation_generates_example() {
        let content = ResponseContent::new(ResponseFrame::AdmitLimitation);
        let text = realize(&content);
        assert!(text.contains("is a dog a mammal?"));
    }

    #[test]
    fn frame_determines_structure() {
        let base = ResponseContent::new(ResponseFrame::AssertKnowledge)
            .with_entity("dog")
            .with_entity("animal")
            .with_predicate("is_a");

        let assert_text = realize(&base);
        assert!(assert_text.starts_with("Yes."));

        let mut suggest = base.clone();
        suggest.frame = ResponseFrame::SuggestInterpretation;
        let suggest_text = realize(&suggest);
        assert!(suggest_text.contains("Did you mean"));
    }
}
