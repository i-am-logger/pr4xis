use praxis::category::Entity;

/// Grammatical number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Number {
    Singular,
    Plural,
}

/// Grammatical person.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Person {
    First,
    Second,
    Third,
}

/// Verb tense.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tense {
    Present,
    Past,
    Future,
}

/// Noun countability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Countability {
    Countable,
    Uncountable,
}

/// Noun type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NounKind {
    Common,
    Proper,
}

/// Verb transitivity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Transitivity {
    Transitive,
    Intransitive,
    Ditransitive,
}

/// Determiner definiteness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Definiteness {
    Definite,
    Indefinite,
    Demonstrative,
    Quantifier,
}

/// A noun: "dog", "city", "water".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Noun {
    pub text: String,
    pub number: Number,
    pub person: Person,
    pub countability: Countability,
    pub kind: NounKind,
}

/// A verb: "runs", "saw", "have".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Verb {
    pub text: String,
    pub lemma: String,
    pub number: Number,
    pub person: Person,
    pub tense: Tense,
    pub transitivity: Transitivity,
}

/// A determiner: "the", "a", "this", "every".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Determiner {
    pub text: String,
    pub definiteness: Definiteness,
    pub number: Option<Number>,
}

/// An adjective: "big", "red", "happy".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Adjective {
    pub text: String,
}

/// An adverb: "quickly", "very", "never".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Adverb {
    pub text: String,
}

/// A preposition: "in", "on", "with".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Preposition {
    pub text: String,
}

/// A conjunction: "and", "but", "or".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Conjunction {
    pub text: String,
}

/// A pronoun: "he", "she", "they".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pronoun {
    pub text: String,
    pub number: Number,
    pub person: Person,
}

/// A lexical entry — a word with its full part-of-speech structure.
/// Each variant carries the rich type for that part of speech.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LexicalEntry {
    Noun(Noun),
    Verb(Verb),
    Determiner(Determiner),
    Adjective(Adjective),
    Adverb(Adverb),
    Preposition(Preposition),
    Conjunction(Conjunction),
    Pronoun(Pronoun),
}

impl LexicalEntry {
    pub fn text(&self) -> &str {
        match self {
            Self::Noun(n) => &n.text,
            Self::Verb(v) => &v.text,
            Self::Determiner(d) => &d.text,
            Self::Adjective(a) => &a.text,
            Self::Adverb(a) => &a.text,
            Self::Preposition(p) => &p.text,
            Self::Conjunction(c) => &c.text,
            Self::Pronoun(p) => &p.text,
        }
    }

    pub fn number(&self) -> Option<Number> {
        match self {
            Self::Noun(n) => Some(n.number),
            Self::Verb(v) => Some(v.number),
            Self::Determiner(d) => d.number,
            Self::Pronoun(p) => Some(p.number),
            _ => None,
        }
    }

    pub fn person(&self) -> Option<Person> {
        match self {
            Self::Noun(n) => Some(n.person),
            Self::Verb(v) => Some(v.person),
            Self::Pronoun(p) => Some(p.person),
            _ => None,
        }
    }

    pub fn pos_tag(&self) -> PosTag {
        match self {
            Self::Noun(_) => PosTag::Noun,
            Self::Verb(_) => PosTag::Verb,
            Self::Determiner(_) => PosTag::Determiner,
            Self::Adjective(_) => PosTag::Adjective,
            Self::Adverb(_) => PosTag::Adverb,
            Self::Preposition(_) => PosTag::Preposition,
            Self::Conjunction(_) => PosTag::Conjunction,
            Self::Pronoun(_) => PosTag::Pronoun,
        }
    }
}

/// Part-of-speech tag — the category identifier (used by grammar layer).
/// This is the Entity for category-theoretic operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PosTag {
    Noun,
    Verb,
    Determiner,
    Adjective,
    Adverb,
    Preposition,
    Conjunction,
    Pronoun,
}

impl Entity for PosTag {
    fn variants() -> Vec<Self> {
        vec![
            Self::Noun,
            Self::Verb,
            Self::Determiner,
            Self::Adjective,
            Self::Adverb,
            Self::Preposition,
            Self::Conjunction,
            Self::Pronoun,
        ]
    }
}

impl PosTag {
    pub fn is_content(&self) -> bool {
        matches!(
            self,
            Self::Noun | Self::Verb | Self::Adjective | Self::Adverb
        )
    }

    pub fn is_function(&self) -> bool {
        !self.is_content()
    }
}
