#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Concept, FinitelyGenerated};

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
///
/// Note: there is no `Mathematics` domain here. The grammar of mathematical
/// OPERATORS (`+`, `<`, …) — their OpenMath symbol, arity, and Lambek type —
/// is a LOADED vocabulary (`cognitive::linguistics::lambek::operators`,
/// `data/operators/math-operators.xml`), not a Rust tag. This catalog names
/// characters by typographic identity; an operator glyph's mathematical role
/// lives in the loaded vocabulary, so `+`/`=`/`%` are `General` here (their
/// meaning, like `<`, depends on context).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolDomain {
    /// Programming and markup.
    Computing,
    /// Currency and commerce.
    Currency,
    /// General purpose / multiple domains.
    General,
}

impl Concept for SymbolDomain {}
impl FinitelyGenerated for SymbolDomain {
    fn variants() -> Vec<Self> {
        vec![Self::Computing, Self::Currency, Self::General]
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
        // Arithmetic / relational glyphs — typographic identity only; their
        // operator grammar (arity, OpenMath symbol, Lambek type) is loaded from
        // data/operators/math-operators.xml, not tagged here (#169).
        SpecialSymbol::new('+', "plus", SymbolDomain::General),
        SpecialSymbol::new('-', "minus/hyphen", SymbolDomain::General),
        SpecialSymbol::new('*', "asterisk", SymbolDomain::General),
        SpecialSymbol::new('/', "slash", SymbolDomain::General),
        SpecialSymbol::new('=', "equals", SymbolDomain::General),
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
        SpecialSymbol::new('%', "percent", SymbolDomain::General),
    ]
}
