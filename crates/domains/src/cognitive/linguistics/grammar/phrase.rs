use pr4xis::category::Entity;

use super::super::lexicon::pos::LexicalEntry;

/// Phrase type — non-terminal categories in the syntax tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhraseType {
    Sentence,
    NounPhrase,
    VerbPhrase,
    PrepPhrase,
    AdjPhrase,
    AdvPhrase,
}

impl Entity for PhraseType {
    fn variants() -> Vec<Self> {
        vec![
            Self::Sentence,
            Self::NounPhrase,
            Self::VerbPhrase,
            Self::PrepPhrase,
            Self::AdjPhrase,
            Self::AdvPhrase,
        ]
    }
}

/// A node in a syntax tree — either a leaf (word) or a branch (phrase).
#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxNode {
    Leaf {
        entry: LexicalEntry,
    },
    Branch {
        phrase: PhraseType,
        children: Vec<SyntaxNode>,
    },
}

impl SyntaxNode {
    /// Get the text of this node (concatenation of all leaves).
    pub fn text(&self) -> String {
        match self {
            Self::Leaf { entry } => entry.text().to_string(),
            Self::Branch { children, .. } => children
                .iter()
                .map(|c| c.text())
                .collect::<Vec<_>>()
                .join(" "),
        }
    }

    /// Get all leaf entries in order.
    pub fn leaves(&self) -> Vec<&LexicalEntry> {
        match self {
            Self::Leaf { entry } => vec![entry],
            Self::Branch { children, .. } => children.iter().flat_map(|c| c.leaves()).collect(),
        }
    }

    /// Get the head word of a phrase (the noun in NP, verb in VP, etc.).
    pub fn head(&self) -> Option<&LexicalEntry> {
        match self {
            Self::Leaf { entry } => Some(entry),
            Self::Branch { phrase, children } => {
                let target_tag = match phrase {
                    PhraseType::NounPhrase => super::super::lexicon::pos::PosTag::Noun,
                    PhraseType::VerbPhrase => super::super::lexicon::pos::PosTag::Verb,
                    PhraseType::PrepPhrase => super::super::lexicon::pos::PosTag::Preposition,
                    PhraseType::AdjPhrase => super::super::lexicon::pos::PosTag::Adjective,
                    PhraseType::AdvPhrase => super::super::lexicon::pos::PosTag::Adverb,
                    PhraseType::Sentence => return None,
                };
                // Find first leaf or sub-branch head matching target POS
                for child in children {
                    match child {
                        Self::Leaf { entry } if entry.pos_tag() == target_tag => {
                            return Some(entry);
                        }
                        Self::Branch { phrase: p, .. }
                            if self.phrase_matches_pos(*p, target_tag) =>
                        {
                            return child.head();
                        }
                        _ => {}
                    }
                }
                // Fallback: first leaf
                children.iter().find_map(|c| match c {
                    Self::Leaf { entry } => Some(entry),
                    _ => None,
                })
            }
        }
    }

    fn phrase_matches_pos(
        &self,
        phrase: PhraseType,
        pos: super::super::lexicon::pos::PosTag,
    ) -> bool {
        use super::super::lexicon::pos::PosTag;
        matches!(
            (phrase, pos),
            (PhraseType::NounPhrase, PosTag::Noun)
                | (PhraseType::VerbPhrase, PosTag::Verb)
                | (PhraseType::AdjPhrase, PosTag::Adjective)
                | (PhraseType::AdvPhrase, PosTag::Adverb)
        )
    }

    /// Tree depth.
    pub fn depth(&self) -> usize {
        match self {
            Self::Leaf { .. } => 0,
            Self::Branch { children, .. } => {
                1 + children.iter().map(|c| c.depth()).max().unwrap_or(0)
            }
        }
    }

    /// Count of all leaves.
    pub fn word_count(&self) -> usize {
        self.leaves().len()
    }
}
