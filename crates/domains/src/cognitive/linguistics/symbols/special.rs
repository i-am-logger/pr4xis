#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;

/// A special symbol — a character whose meaning is defined
/// by the language that uses it, not by universal convention.
///
/// The same character can have completely different meanings:
/// - '<' in English = "less than" (comparison)
/// - '<' in XML = "element open" (markup)
/// - '<' in math = "less than" (ordering)
/// - '<' in shell = "input redirect" (I/O)
///
/// This is context disambiguation at the symbol level.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpecialSymbol {
    pub character: char,
    pub name: String,
    pub domain: SymbolDomain,
}

/// The domain in which a special symbol has meaning.
/// A symbol can belong to multiple domains with different meanings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolDomain {
    /// Mathematical operators and relations.
    Mathematics,
    /// Programming and markup.
    Computing,
    /// Currency and commerce.
    Currency,
    /// General purpose / multiple domains.
    General,
}

impl Concept for SymbolDomain {
    fn variants() -> Vec<Self> {
        vec![
            Self::Mathematics,
            Self::Computing,
            Self::Currency,
            Self::General,
        ]
    }
}

impl SpecialSymbol {
    pub fn new(character: char, name: &str, domain: SymbolDomain) -> Self {
        Self {
            character,
            name: name.into(),
            domain,
        }
    }
}

/// Special symbols commonly used across languages.
pub fn common_symbols() -> Vec<SpecialSymbol> {
    vec![
        // Mathematical
        SpecialSymbol::new('+', "plus", SymbolDomain::Mathematics),
        SpecialSymbol::new('-', "minus/hyphen", SymbolDomain::General),
        SpecialSymbol::new('*', "asterisk", SymbolDomain::General),
        SpecialSymbol::new('/', "slash", SymbolDomain::General),
        SpecialSymbol::new('=', "equals", SymbolDomain::Mathematics),
        SpecialSymbol::new('<', "less-than/angle-open", SymbolDomain::General),
        SpecialSymbol::new('>', "greater-than/angle-close", SymbolDomain::General),
        // Computing
        SpecialSymbol::new('&', "ampersand", SymbolDomain::General),
        SpecialSymbol::new('|', "pipe", SymbolDomain::Computing),
        SpecialSymbol::new('#', "hash", SymbolDomain::General),
        SpecialSymbol::new('@', "at", SymbolDomain::General),
        SpecialSymbol::new('\\', "backslash", SymbolDomain::Computing),
        SpecialSymbol::new('~', "tilde", SymbolDomain::General),
        SpecialSymbol::new('^', "caret", SymbolDomain::General),
        SpecialSymbol::new('_', "underscore", SymbolDomain::General),
        SpecialSymbol::new('{', "open brace", SymbolDomain::Computing),
        SpecialSymbol::new('}', "close brace", SymbolDomain::Computing),
        SpecialSymbol::new('[', "open bracket", SymbolDomain::General),
        SpecialSymbol::new(']', "close bracket", SymbolDomain::General),
        // Currency
        SpecialSymbol::new('$', "dollar", SymbolDomain::Currency),
        SpecialSymbol::new('%', "percent", SymbolDomain::Mathematics),
    ]
}
