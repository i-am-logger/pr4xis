#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;

/// A numeral system — how numbers are represented in writing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NumeralSystem {
    pub name: String,
    pub base: u32,
    pub digits: Vec<Digit>,
}

/// A single digit in a numeral system.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Digit {
    pub character: char,
    pub value: u32,
}

/// Well-known numeral system types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NumeralSystemKind {
    /// Positional: 0-9, value depends on position (Arabic/Hindu-Arabic).
    Positional,
    /// Additive: I, V, X, L, C, D, M — values add (Roman).
    Additive,
}

impl Concept for NumeralSystemKind {
    fn variants() -> Vec<Self> {
        vec![Self::Positional, Self::Additive]
    }
}

impl NumeralSystem {
    pub fn new(name: &str, base: u32) -> Self {
        Self {
            name: name.into(),
            base,
            digits: Vec::new(),
        }
    }

    pub fn with_digit(mut self, character: char, value: u32) -> Self {
        self.digits.push(Digit { character, value });
        self
    }

    pub fn contains(&self, c: char) -> bool {
        self.digits.iter().any(|d| d.character == c)
    }

    pub fn value_of(&self, c: char) -> Option<u32> {
        self.digits
            .iter()
            .find(|d| d.character == c)
            .map(|d| d.value)
    }
}

/// Arabic (Hindu-Arabic) numeral system: 0-9, base 10, positional.
pub fn arabic() -> NumeralSystem {
    NumeralSystem::new("Arabic", 10)
        .with_digit('0', 0)
        .with_digit('1', 1)
        .with_digit('2', 2)
        .with_digit('3', 3)
        .with_digit('4', 4)
        .with_digit('5', 5)
        .with_digit('6', 6)
        .with_digit('7', 7)
        .with_digit('8', 8)
        .with_digit('9', 9)
}

/// Roman numeral system: I, V, X, L, C, D, M — additive/subtractive.
pub fn roman() -> NumeralSystem {
    NumeralSystem::new("Roman", 10)
        .with_digit('I', 1)
        .with_digit('V', 5)
        .with_digit('X', 10)
        .with_digit('L', 50)
        .with_digit('C', 100)
        .with_digit('D', 500)
        .with_digit('M', 1000)
}
