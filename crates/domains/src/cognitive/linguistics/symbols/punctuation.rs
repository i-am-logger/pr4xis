#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;

/// Punctuation — written marks that structure and modify meaning.
///
/// Each punctuation mark is a rich type carrying semantic meaning,
/// not just a character. The same visual mark can have different
/// meanings in different languages (context disambiguation).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PunctuationMark {
    pub character: char,
    pub name: String,
    pub function: PunctuationFunction,
    pub position: Position,
}

/// What a punctuation mark DOES semantically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PunctuationFunction {
    /// Terminates a declarative statement. Creates a complete thought.
    /// Commits the speaker to truth of the preceding statement.
    StatementTerminator,

    /// Transforms a statement into an inquiry. Demands a response.
    QuestionMarker,

    /// Adds emphasis or urgency. Intensifies the preceding content.
    EmphasisMarker,

    /// Separates items, clauses, or phrases. Creates a pause.
    /// One of the most context-dependent marks — meaning varies by position.
    Separator,

    /// Connects two related independent thoughts.
    /// Stronger than a comma, weaker than a period.
    Connector,

    /// Introduces elaboration, definition, or a list.
    /// Creates an expectation of what follows.
    Introducer,

    /// Marks direct speech, quotation, or technical terms.
    /// Creates a frame — content inside is attributed or special.
    Quotation,

    /// Marks possession or contraction. Modifies word meaning.
    Possessive,

    /// Groups content together. Creates a nested context.
    Grouping,

    /// Indicates a range, pause, or interruption.
    Dash,

    /// Indicates omission or trailing thought.
    Ellipsis,
}

impl Concept for PunctuationFunction {
    fn variants() -> Vec<Self> {
        vec![
            Self::StatementTerminator,
            Self::QuestionMarker,
            Self::EmphasisMarker,
            Self::Separator,
            Self::Connector,
            Self::Introducer,
            Self::Quotation,
            Self::Possessive,
            Self::Grouping,
            Self::Dash,
            Self::Ellipsis,
        ]
    }
}

/// Where the punctuation mark appears relative to content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Position {
    /// After content: period, question mark, exclamation.
    After,
    /// Before content: opening quote, opening parenthesis.
    Before,
    /// Between content: comma, semicolon, colon.
    Between,
    /// Wrapping content (paired): quotes, parentheses, brackets.
    Wrapping,
}

impl Concept for Position {
    fn variants() -> Vec<Self> {
        vec![Self::After, Self::Before, Self::Between, Self::Wrapping]
    }
}

impl PunctuationMark {
    pub fn new(
        character: char,
        name: &str,
        function: PunctuationFunction,
        position: Position,
    ) -> Self {
        Self {
            character,
            name: name.into(),
            function,
            position,
        }
    }

    /// Does this mark terminate a sentence?
    pub fn is_sentence_ending(&self) -> bool {
        matches!(
            self.function,
            PunctuationFunction::StatementTerminator
                | PunctuationFunction::QuestionMarker
                | PunctuationFunction::EmphasisMarker
        )
    }

    /// Does this mark expect a response from the listener?
    pub fn expects_response(&self) -> bool {
        matches!(self.function, PunctuationFunction::QuestionMarker)
    }
}

// ---- Standard punctuation marks ----

pub fn period() -> PunctuationMark {
    PunctuationMark::new(
        '.',
        "period",
        PunctuationFunction::StatementTerminator,
        Position::After,
    )
}

pub fn question_mark() -> PunctuationMark {
    PunctuationMark::new(
        '?',
        "question mark",
        PunctuationFunction::QuestionMarker,
        Position::After,
    )
}

pub fn exclamation_mark() -> PunctuationMark {
    PunctuationMark::new(
        '!',
        "exclamation mark",
        PunctuationFunction::EmphasisMarker,
        Position::After,
    )
}

pub fn comma() -> PunctuationMark {
    PunctuationMark::new(
        ',',
        "comma",
        PunctuationFunction::Separator,
        Position::Between,
    )
}

pub fn semicolon() -> PunctuationMark {
    PunctuationMark::new(
        ';',
        "semicolon",
        PunctuationFunction::Connector,
        Position::Between,
    )
}

pub fn colon() -> PunctuationMark {
    PunctuationMark::new(
        ':',
        "colon",
        PunctuationFunction::Introducer,
        Position::Between,
    )
}

pub fn apostrophe() -> PunctuationMark {
    PunctuationMark::new(
        '\'',
        "apostrophe",
        PunctuationFunction::Possessive,
        Position::Between,
    )
}

pub fn double_quote() -> PunctuationMark {
    PunctuationMark::new(
        '"',
        "double quote",
        PunctuationFunction::Quotation,
        Position::Wrapping,
    )
}

pub fn open_paren() -> PunctuationMark {
    PunctuationMark::new(
        '(',
        "opening parenthesis",
        PunctuationFunction::Grouping,
        Position::Before,
    )
}

pub fn close_paren() -> PunctuationMark {
    PunctuationMark::new(
        ')',
        "closing parenthesis",
        PunctuationFunction::Grouping,
        Position::After,
    )
}

pub fn hyphen() -> PunctuationMark {
    PunctuationMark::new('-', "hyphen", PunctuationFunction::Dash, Position::Between)
}

pub fn ellipsis() -> PunctuationMark {
    PunctuationMark::new(
        '\u{2026}',
        "ellipsis",
        PunctuationFunction::Ellipsis,
        Position::After,
    )
}

/// All standard punctuation marks.
pub fn standard_punctuation() -> Vec<PunctuationMark> {
    vec![
        period(),
        question_mark(),
        exclamation_mark(),
        comma(),
        semicolon(),
        colon(),
        apostrophe(),
        double_quote(),
        open_paren(),
        close_paren(),
        hyphen(),
        ellipsis(),
    ]
}
