#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};

use super::super::lexicon::pos::*;
use super::phrase::{PhraseType, SyntaxNode};

/// The state of an in-progress parse — a stack of open phrases.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseState {
    pub stack: Vec<OpenPhrase>,
    pub completed: Option<SyntaxNode>,
}

/// An open (in-progress) phrase on the parse stack.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenPhrase {
    pub phrase: PhraseType,
    pub children: Vec<SyntaxNode>,
}

impl Situation for ParseState {
    fn describe(&self) -> String {
        if let Some(tree) = &self.completed {
            format!("COMPLETE: \"{}\"", tree.text())
        } else if let Some(top) = self.stack.last() {
            let words: usize = self.stack.iter().map(|p| p.children.len()).sum();
            format!(
                "building {:?} | depth {} | {} nodes",
                top.phrase,
                self.stack.len(),
                words
            )
        } else {
            "empty".into()
        }
    }

    fn is_terminal(&self) -> bool {
        self.completed.is_some()
    }
}

/// Actions for building a parse tree.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseAction {
    /// Open a new phrase on the stack.
    OpenPhrase { phrase: PhraseType },
    /// Add a word as a leaf to the current phrase.
    AddWord { entry: LexicalEntry },
    /// Close the top phrase, adding it as a child of the phrase below.
    ClosePhrase,
}

impl Action for ParseAction {
    type Sit = ParseState;

    fn describe(&self) -> String {
        match self {
            Self::OpenPhrase { phrase } => format!("open {:?}", phrase),
            Self::AddWord { entry } => {
                format!("add \"{}\" ({:?})", entry.text(), entry.pos_tag())
            }
            Self::ClosePhrase => "close phrase".into(),
        }
    }
}

// ---- Preconditions ----

/// Precondition: phrase structure rules — a child must be valid within its parent.
pub struct PhraseStructureRule;

impl PhraseStructureRule {
    fn is_valid_child(parent: PhraseType, child_pos: PosTag) -> bool {
        matches!(
            (parent, child_pos),
            // Sentence: NP + VP (handled via sub-phrases, not direct POS)
            // NounPhrase children
            (PhraseType::NounPhrase, PosTag::Noun)
                | (PhraseType::NounPhrase, PosTag::Determiner)
                | (PhraseType::NounPhrase, PosTag::Adjective)
                | (PhraseType::NounPhrase, PosTag::Pronoun)
                // VerbPhrase children
                | (PhraseType::VerbPhrase, PosTag::Verb)
                | (PhraseType::VerbPhrase, PosTag::Adverb)
                // PrepPhrase children
                | (PhraseType::PrepPhrase, PosTag::Preposition)
                // AdjPhrase children
                | (PhraseType::AdjPhrase, PosTag::Adjective)
                | (PhraseType::AdjPhrase, PosTag::Adverb)
                // AdvPhrase children
                | (PhraseType::AdvPhrase, PosTag::Adverb)
        )
    }

    fn is_valid_subphrase(parent: PhraseType, child: PhraseType) -> bool {
        matches!(
            (parent, child),
            // Sentence contains NP + VP
            (PhraseType::Sentence, PhraseType::NounPhrase)
                | (PhraseType::Sentence, PhraseType::VerbPhrase)
                // NP can contain PP ("the dog in the park") or AdjP
                | (PhraseType::NounPhrase, PhraseType::PrepPhrase)
                | (PhraseType::NounPhrase, PhraseType::AdjPhrase)
                // VP can contain NP, PP, AdvP
                | (PhraseType::VerbPhrase, PhraseType::NounPhrase)
                | (PhraseType::VerbPhrase, PhraseType::PrepPhrase)
                | (PhraseType::VerbPhrase, PhraseType::AdvPhrase)
                // PP contains NP ("in the park")
                | (PhraseType::PrepPhrase, PhraseType::NounPhrase)
                // AdjP can contain AdvP ("very big")
                | (PhraseType::AdjPhrase, PhraseType::AdvPhrase)
        )
    }
}

impl Precondition<ParseAction> for PhraseStructureRule {
    fn check(&self, state: &ParseState, action: &ParseAction) -> PreconditionResult {
        match action {
            ParseAction::AddWord { entry } => {
                if let Some(top) = state.stack.last() {
                    if Self::is_valid_child(top.phrase, entry.pos_tag()) {
                        PreconditionResult::satisfied(
                            "phrase_structure",
                            &format!("{:?} is valid in {:?}", entry.pos_tag(), top.phrase),
                        )
                    } else {
                        PreconditionResult::violated(
                            "phrase_structure",
                            &format!("{:?} is not valid in {:?}", entry.pos_tag(), top.phrase),
                            &state.describe(),
                            &action.describe(),
                        )
                    }
                } else {
                    PreconditionResult::violated(
                        "phrase_structure",
                        "no open phrase to add word to",
                        &state.describe(),
                        &action.describe(),
                    )
                }
            }
            ParseAction::OpenPhrase { phrase } => {
                if let Some(top) = state.stack.last() {
                    if Self::is_valid_subphrase(top.phrase, *phrase) {
                        PreconditionResult::satisfied(
                            "phrase_structure",
                            &format!("{:?} is valid in {:?}", phrase, top.phrase),
                        )
                    } else {
                        PreconditionResult::violated(
                            "phrase_structure",
                            &format!("{:?} is not valid in {:?}", phrase, top.phrase),
                            &state.describe(),
                            &action.describe(),
                        )
                    }
                } else {
                    // Opening the root phrase (Sentence) — always valid
                    PreconditionResult::satisfied("phrase_structure", "opening root phrase")
                }
            }
            ParseAction::ClosePhrase => {
                PreconditionResult::satisfied("phrase_structure", "close is structural")
            }
        }
    }

    fn describe(&self) -> &str {
        "children must be valid within their parent phrase"
    }
}

/// Precondition: stack must not be empty for ClosePhrase.
pub struct StackNotEmpty;

impl Precondition<ParseAction> for StackNotEmpty {
    fn check(&self, state: &ParseState, action: &ParseAction) -> PreconditionResult {
        if matches!(action, ParseAction::ClosePhrase) && state.stack.is_empty() {
            return PreconditionResult::violated(
                "stack_not_empty",
                "cannot close phrase: stack is empty",
                &state.describe(),
                &action.describe(),
            );
        }
        PreconditionResult::satisfied("stack_not_empty", "stack has open phrases")
    }

    fn describe(&self) -> &str {
        "cannot close a phrase when the stack is empty"
    }
}

/// Precondition: parse must not already be complete.
pub struct NotComplete;

impl Precondition<ParseAction> for NotComplete {
    fn check(&self, state: &ParseState, action: &ParseAction) -> PreconditionResult {
        if state.completed.is_some() {
            PreconditionResult::violated(
                "not_complete",
                "parse is already complete",
                &state.describe(),
                &action.describe(),
            )
        } else {
            PreconditionResult::satisfied("not_complete", "parse in progress")
        }
    }

    fn describe(&self) -> &str {
        "cannot modify a completed parse"
    }
}

/// Precondition: subject-verb agreement in number.
pub struct SubjectVerbAgreement;

impl Precondition<ParseAction> for SubjectVerbAgreement {
    fn check(&self, state: &ParseState, action: &ParseAction) -> PreconditionResult {
        // Only check when closing a Sentence
        if matches!(action, ParseAction::ClosePhrase)
            && let Some(top) = state.stack.last()
            && top.phrase == PhraseType::Sentence
        {
            return self.check_agreement(top);
        }
        PreconditionResult::satisfied("subject_verb_agreement", "not closing a sentence")
    }

    fn describe(&self) -> &str {
        "subject and verb must agree in number"
    }
}

impl SubjectVerbAgreement {
    fn check_agreement(&self, sentence: &OpenPhrase) -> PreconditionResult {
        let mut subject_number = None;
        let mut verb_number = None;

        for child in &sentence.children {
            match child {
                SyntaxNode::Branch {
                    phrase: PhraseType::NounPhrase,
                    ..
                } => {
                    if let Some(head) = child.head() {
                        subject_number = head.number();
                    }
                }
                SyntaxNode::Branch {
                    phrase: PhraseType::VerbPhrase,
                    ..
                } => {
                    if let Some(head) = child.head() {
                        verb_number = head.number();
                    }
                }
                _ => {}
            }
        }

        match (subject_number, verb_number) {
            (Some(s), Some(v)) if s != v => PreconditionResult::violated(
                "subject_verb_agreement",
                &format!("subject is {:?} but verb is {:?}", s, v),
                &format!("sentence with {} children", sentence.children.len()),
                "close sentence",
            ),
            (Some(s), Some(v)) if s == v => {
                PreconditionResult::satisfied("subject_verb_agreement", &format!("both {:?}", s))
            }
            _ => PreconditionResult::satisfied(
                "subject_verb_agreement",
                "agreement not applicable (missing subject or verb)",
            ),
        }
    }
}

// ---- Apply function ----

fn apply_parse(state: &ParseState, action: &ParseAction) -> Result<ParseState, String> {
    let mut next = state.clone();
    match action {
        ParseAction::OpenPhrase { phrase } => {
            next.stack.push(OpenPhrase {
                phrase: *phrase,
                children: vec![],
            });
        }
        ParseAction::AddWord { entry } => {
            if let Some(top) = next.stack.last_mut() {
                top.children.push(SyntaxNode::Leaf {
                    entry: entry.clone(),
                });
            }
        }
        ParseAction::ClosePhrase => {
            if let Some(closed) = next.stack.pop() {
                let node = SyntaxNode::Branch {
                    phrase: closed.phrase,
                    children: closed.children,
                };
                if let Some(parent) = next.stack.last_mut() {
                    parent.children.push(node);
                } else {
                    next.completed = Some(node);
                }
            }
        }
    }
    Ok(next)
}

pub type GrammarEngine = Engine<ParseAction>;

/// Create a new parse engine.
pub fn new_parse() -> GrammarEngine {
    Engine::new(
        ParseState {
            stack: vec![],
            completed: None,
        },
        vec![
            Box::new(NotComplete),
            Box::new(StackNotEmpty),
            Box::new(PhraseStructureRule),
            Box::new(SubjectVerbAgreement),
        ],
        apply_parse,
    )
}
