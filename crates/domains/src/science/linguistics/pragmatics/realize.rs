use super::response::ResponseFrame;

// Response realization — maps semantic frames to surface text.
//
// This is the RIGHT ADJOINT of parsing (the generation functor).
// Parsing: Text → Syntax → Semantics
// Generation: Semantics → Syntax → Text
//
// The ResponseFrame determines HOW to say something.
// The content determines WHAT to say.
// Together they compose into surface text.
//
// This is NOT hardcoded strings — it's the composition of:
// 1. ResponseFrame (from epistemics)
// 2. Content (from knowledge/query)
// 3. Language (writing system, grammar rules)
//
// References:
// - Reiter & Dale, "Building Natural Language Generation Systems" (2000)
//   Content determination → Document planning → Microplanning → Realization
// - de Groote, "Towards Abstract Categorial Grammars" (2001)
//   Generation = beta-reduction of lexicon homomorphism

/// A structured response before surface realization.
/// Content is organized by semantic role, not by string position.
#[derive(Debug, Clone)]
pub struct ResponseContent {
    /// The epistemic frame — how the system relates to this knowledge.
    pub frame: ResponseFrame,
    /// The predicate — the relationship being expressed.
    pub predicate: Option<String>,
    /// The entities involved (subject, object, etc.).
    pub entities: Vec<String>,
    /// Definitions or elaborations for entities.
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
/// This is the generation functor: ResponseContent → String.
/// The frame determines the structure, the content fills the slots.
///
/// Reiter & Dale (2000): the four-stage pipeline collapses here because
/// our content is already determined by the query result and the frame
/// is determined by the epistemic state. We only need microplanning
/// and surface realization.
pub fn realize(content: &ResponseContent) -> String {
    match content.frame {
        ResponseFrame::AssertKnowledge => realize_assertion(content),
        ResponseFrame::AcknowledgeGap => realize_gap(content),
        ResponseFrame::SuggestInterpretation => realize_suggestion(content),
        ResponseFrame::AdmitLimitation => realize_limitation(content),
    }
}

/// Realize a confident assertion.
/// Epistemic state: KnownKnown — the system knows and can prove it.
fn realize_assertion(content: &ResponseContent) -> String {
    if content.entities.len() >= 2 {
        let child = &content.entities[0];
        let parent = &content.entities[1];

        if let Some(pred) = &content.predicate
            && (pred == "is_a" || pred == "is-a" || pred == "isa")
        {
            let mut result = format!("Yes. {child} is a {parent}.");
            for (entity, def) in &content.definitions {
                result.push_str(&format!("\n  {entity} -- {def}"));
            }
            return result;
        }

        // General two-entity assertion
        let pred = content.predicate.as_deref().unwrap_or("relates to");
        return format!("Yes. {child} {pred} {parent}.");
    }

    if content.entities.len() == 1 {
        return realize_definition(&content.entities[0], &content.definitions);
    }

    // Bare assertion
    content
        .predicate
        .as_deref()
        .unwrap_or("Understood.")
        .to_string()
}

/// Realize a negative assertion.
pub fn realize_negation(child: &str, parent: &str) -> String {
    format!("No, {child} is not a {parent}.")
}

/// Realize a definition (word lookup result).
fn realize_definition(word: &str, definitions: &[(String, String)]) -> String {
    if definitions.is_empty() {
        return format!("I know '{word}' but have no definition for it.");
    }

    let mut lines = Vec::new();
    for (i, (_entity, def)) in definitions.iter().enumerate() {
        lines.push(format!("  {}. {def}", i + 1));
    }
    format!("{word}:\n{}", lines.join("\n"))
}

/// Realize acknowledgment of a knowledge gap.
/// Epistemic state: KnownUnknown — the system knows what it doesn't know.
fn realize_gap(content: &ResponseContent) -> String {
    if !content.entities.is_empty() {
        let unknown: Vec<&str> = content.entities.iter().map(|s| s.as_str()).collect();
        return format!("I don't know the word(s): {:?}", unknown,);
    }
    "I don't have enough information to answer.".to_string()
}

/// Realize a suggestion based on partial understanding.
/// Epistemic state: UnknownKnown — the system knows things but can't parse.
fn realize_suggestion(content: &ResponseContent) -> String {
    if content.entities.len() >= 2 {
        return format!(
            "I couldn't parse the full sentence, but I found two concepts.\nDid you mean: is {} a {}?",
            content.entities[0], content.entities[1]
        );
    }
    if !content.entities.is_empty() {
        return format!(
            "I know the words {:?} but couldn't understand the sentence structure.\nCould you rephrase as 'is X a Y' or 'what is X'?",
            content.entities,
        );
    }
    "I understood some of what you said but couldn't form a complete interpretation.".to_string()
}

/// Realize admission of complete limitation.
/// Epistemic state: UnknownUnknown — the system has no relevant knowledge.
fn realize_limitation(_content: &ResponseContent) -> String {
    "I don't understand. Could you try a simpler question like 'is a dog a mammal'?".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assert_taxonomy_with_definitions() {
        let content = ResponseContent::new(ResponseFrame::AssertKnowledge)
            .with_predicate("is_a")
            .with_entity("dog")
            .with_entity("mammal")
            .with_definition("dog", "a domesticated canine")
            .with_definition("mammal", "warm-blooded vertebrate");

        let text = realize(&content);
        assert!(text.starts_with("Yes. dog is a mammal."));
        assert!(text.contains("dog -- a domesticated canine"));
        assert!(text.contains("mammal -- warm-blooded vertebrate"));
    }

    #[test]
    fn assert_negation() {
        let text = realize_negation("dog", "fish");
        assert_eq!(text, "No, dog is not a fish.");
    }

    #[test]
    fn assert_definition() {
        let content = ResponseContent::new(ResponseFrame::AssertKnowledge)
            .with_entity("dog")
            .with_definition("dog", "a domesticated canine");

        let text = realize(&content);
        assert!(text.contains("dog:"));
        assert!(text.contains("a domesticated canine"));
    }

    #[test]
    fn gap_with_unknown_words() {
        let content = ResponseContent::new(ResponseFrame::AcknowledgeGap).with_entity("xyzzy");

        let text = realize(&content);
        assert!(text.contains("don't know"));
        assert!(text.contains("xyzzy"));
    }

    #[test]
    fn suggest_two_concepts() {
        let content = ResponseContent::new(ResponseFrame::SuggestInterpretation)
            .with_entity("dog")
            .with_entity("mammal");

        let text = realize(&content);
        assert!(text.contains("Did you mean: is dog a mammal?"));
    }

    #[test]
    fn limitation_provides_guidance() {
        let content = ResponseContent::new(ResponseFrame::AdmitLimitation);
        let text = realize(&content);
        assert!(text.contains("is a dog a mammal"));
    }

    #[test]
    fn frame_determines_structure() {
        let base = ResponseContent::new(ResponseFrame::AssertKnowledge)
            .with_entity("dog")
            .with_entity("animal")
            .with_predicate("is_a");

        let assert_text = realize(&base);
        assert!(assert_text.starts_with("Yes."));

        // Same content, different frame → different structure
        let mut suggest = base.clone();
        suggest.frame = ResponseFrame::SuggestInterpretation;
        let suggest_text = realize(&suggest);
        assert!(suggest_text.contains("Did you mean"));
    }
}
