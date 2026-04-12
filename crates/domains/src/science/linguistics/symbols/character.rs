use pr4xis::category::Entity;

/// A character — the atomic unit of written communication.
///
/// Characters are abstract symbols that carry no meaning on their own.
/// Meaning is assigned by the language that uses them (context disambiguation).
/// The same character 'a' means different things in English vs. math vs. XML.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Character {
    pub codepoint: char,
    pub name: String,
    pub category: UnicodeCategory,
}

impl Character {
    pub fn new(codepoint: char, name: &str, category: UnicodeCategory) -> Self {
        Self {
            codepoint,
            name: name.into(),
            category,
        }
    }

    pub fn is_letter(&self) -> bool {
        matches!(
            self.category,
            UnicodeCategory::UppercaseLetter | UnicodeCategory::LowercaseLetter
        )
    }

    pub fn is_digit(&self) -> bool {
        matches!(self.category, UnicodeCategory::DecimalDigit)
    }

    pub fn is_punctuation(&self) -> bool {
        matches!(self.category, UnicodeCategory::Punctuation)
    }

    pub fn is_symbol(&self) -> bool {
        matches!(self.category, UnicodeCategory::Symbol)
    }

    pub fn is_whitespace(&self) -> bool {
        matches!(self.category, UnicodeCategory::Whitespace)
    }
}

/// Unicode general category (simplified for our needs).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnicodeCategory {
    UppercaseLetter,
    LowercaseLetter,
    DecimalDigit,
    Punctuation,
    Symbol,
    Whitespace,
    Other,
}

impl Entity for UnicodeCategory {
    fn variants() -> Vec<Self> {
        vec![
            Self::UppercaseLetter,
            Self::LowercaseLetter,
            Self::DecimalDigit,
            Self::Punctuation,
            Self::Symbol,
            Self::Whitespace,
            Self::Other,
        ]
    }
}

/// A script — an organized set of characters used by one or more languages.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Script {
    pub name: String,
    pub characters: Vec<Character>,
    pub direction: Direction,
}

/// Writing direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    LeftToRight,
    RightToLeft,
    TopToBottom,
}

impl Entity for Direction {
    fn variants() -> Vec<Self> {
        vec![Self::LeftToRight, Self::RightToLeft, Self::TopToBottom]
    }
}

impl Script {
    pub fn new(name: &str, direction: Direction) -> Self {
        Self {
            name: name.into(),
            characters: Vec::new(),
            direction,
        }
    }

    pub fn with_range(mut self, from: char, to: char, category: UnicodeCategory) -> Self {
        for c in from..=to {
            self.characters
                .push(Character::new(c, &format!("{}", c), category));
        }
        self
    }

    pub fn with_char(mut self, c: char, name: &str, category: UnicodeCategory) -> Self {
        self.characters.push(Character::new(c, name, category));
        self
    }

    pub fn contains(&self, c: char) -> bool {
        self.characters.iter().any(|ch| ch.codepoint == c)
    }

    pub fn letter_count(&self) -> usize {
        self.characters.iter().filter(|c| c.is_letter()).count()
    }
}

// ---- Well-known scripts ----

/// Latin script (a-z, A-Z).
pub fn latin() -> Script {
    Script::new("Latin", Direction::LeftToRight)
        .with_range('A', 'Z', UnicodeCategory::UppercaseLetter)
        .with_range('a', 'z', UnicodeCategory::LowercaseLetter)
}

/// Hebrew script (aleph-tav).
pub fn hebrew() -> Script {
    Script::new("Hebrew", Direction::RightToLeft).with_range(
        '\u{05D0}',
        '\u{05EA}',
        UnicodeCategory::LowercaseLetter,
    )
}

/// Arabic numerals (0-9).
pub fn arabic_numerals() -> Script {
    Script::new("Arabic Numerals", Direction::LeftToRight).with_range(
        '0',
        '9',
        UnicodeCategory::DecimalDigit,
    )
}
