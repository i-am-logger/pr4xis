use pr4xis::engine::{Action, Engine, Precondition, Situation};
use pr4xis::logic::proof::{Counterexample, SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

use super::super::lexicon::pos::*;
use super::phrase::{PhraseType, SyntaxNode};

fn grammar_meta(name: &'static str, description: &'static str) -> Provenance {
    Provenance {
        name: OntologyName::new_static(name),
        description: Label::new_static(description),
        citation: Citation::parse_static(
            "Chomsky (1957) Syntactic Structures; Chomsky (1965) Aspects of the Theory of Syntax — phrase-structure grammar",
        ),
        module_path: ModulePath::new_static(module_path!()),
    }
}

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

impl Situation for ParseState {}

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
    fn check(&self, state: &ParseState, action: &ParseAction) -> Verdict {
        let meta = grammar_meta(
            "PhraseStructureRule",
            "children must be valid within their parent phrase",
        );
        let ok = match action {
            ParseAction::AddWord { entry } => state
                .stack
                .last()
                .map(|top| Self::is_valid_child(top.phrase, entry.pos_tag()))
                .unwrap_or(false),
            ParseAction::OpenPhrase { phrase } => state
                .stack
                .last()
                .map(|top| Self::is_valid_subphrase(top.phrase, *phrase))
                .unwrap_or(true),
            ParseAction::ClosePhrase => true,
        };
        if ok {
            Ok(Box::new(SimpleProof::new(meta)))
        } else {
            Err(Box::new(SimpleCounterexample::new(meta)))
        }
    }
}

/// Precondition: stack must not be empty for ClosePhrase.
pub struct StackNotEmpty;

impl Precondition<ParseAction> for StackNotEmpty {
    fn check(&self, state: &ParseState, action: &ParseAction) -> Verdict {
        let meta = grammar_meta(
            "StackNotEmpty",
            "cannot close a phrase when the stack is empty",
        );
        if matches!(action, ParseAction::ClosePhrase) && state.stack.is_empty() {
            return Err(Box::new(SimpleCounterexample::new(meta)));
        }
        Ok(Box::new(SimpleProof::new(meta)))
    }
}

/// Precondition: parse must not already be complete.
pub struct NotComplete;

impl Precondition<ParseAction> for NotComplete {
    fn check(&self, state: &ParseState, _action: &ParseAction) -> Verdict {
        let meta = grammar_meta("NotComplete", "cannot modify a completed parse");
        if state.completed.is_some() {
            Err(Box::new(SimpleCounterexample::new(meta)))
        } else {
            Ok(Box::new(SimpleProof::new(meta)))
        }
    }
}

/// Precondition: subject-verb agreement in number.
pub struct SubjectVerbAgreement;

impl Precondition<ParseAction> for SubjectVerbAgreement {
    fn check(&self, state: &ParseState, action: &ParseAction) -> Verdict {
        let meta = grammar_meta(
            "SubjectVerbAgreement",
            "subject and verb must agree in number",
        );
        if matches!(action, ParseAction::ClosePhrase)
            && let Some(top) = state.stack.last()
            && top.phrase == PhraseType::Sentence
            && !self.check_agreement(top)
        {
            return Err(Box::new(SimpleCounterexample::new(meta)));
        }
        Ok(Box::new(SimpleProof::new(meta)))
    }
}

impl SubjectVerbAgreement {
    fn check_agreement(&self, sentence: &OpenPhrase) -> bool {
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
            (Some(s), Some(v)) => s == v,
            _ => true,
        }
    }
}

// ---- Apply function ----

fn apply_parse(
    state: &ParseState,
    action: &ParseAction,
) -> Result<ParseState, Box<dyn Counterexample>> {
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
