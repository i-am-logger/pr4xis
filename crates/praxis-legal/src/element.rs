use crate::authority::Authority;
use crate::fact::Fact;

/// Answer to an element — NOT a bool. Each state carries its context.
#[derive(Debug, Clone, PartialEq)]
pub enum Answer {
    Yes { confidence: f64, basis: String },
    No { reason: String },
    Partial { met: String, unmet: String },
    Unknown { needs: String },
}

impl Answer {
    pub fn is_met(&self) -> bool {
        matches!(self, Answer::Yes { .. })
    }

    pub fn tag(&self) -> AnswerTag {
        match self {
            Answer::Yes { .. } => AnswerTag::Yes,
            Answer::No { .. } => AnswerTag::No,
            Answer::Partial { .. } => AnswerTag::Partial,
            Answer::Unknown { .. } => AnswerTag::Unknown,
        }
    }
}

/// Lightweight tag for matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnswerTag {
    Yes,
    No,
    Partial,
    Unknown,
}

/// A single element of a legal claim.
#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub name: String,
    pub description: String,
    pub answer: Answer,
    pub evidence: Vec<Fact>,
    pub analysis: Option<String>,
}

/// Prima facie element check for a claim under a specific authority.
#[derive(Debug, Clone, PartialEq)]
pub struct ElementCheck {
    pub statute: Authority,
    pub elements: Vec<Element>,
}

impl ElementCheck {
    /// Are all elements met?
    pub fn all_met(&self) -> bool {
        self.elements.iter().all(|e| e.answer.is_met())
    }

    /// Count by answer tag: (yes, no, partial, unknown).
    pub fn tally(&self) -> (usize, usize, usize, usize) {
        let mut yes = 0;
        let mut no = 0;
        let mut partial = 0;
        let mut unknown = 0;
        for e in &self.elements {
            match e.answer.tag() {
                AnswerTag::Yes => yes += 1,
                AnswerTag::No => no += 1,
                AnswerTag::Partial => partial += 1,
                AnswerTag::Unknown => unknown += 1,
            }
        }
        (yes, no, partial, unknown)
    }
}
