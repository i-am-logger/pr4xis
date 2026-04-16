use pr4xis::category::Entity;

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

/// Pronoun kind — from OLiA classification.
/// OLiA: PersonalPronoun, InterrogativePronoun, DemonstrativePronoun, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PronounKind {
    /// "he", "she", "it", "they" — refers to previously mentioned entities.
    Personal,
    /// "what", "who", "which" — asks for information.
    Interrogative,
    /// "this", "that" — points to entities.
    Demonstrative,
    /// "who", "which", "that" — introduces relative clauses.
    Relative,
    /// "myself", "themselves" — refers back to the subject.
    Reflexive,
    /// "someone", "anything" — refers to unspecified entities.
    Indefinite,
}

/// A pronoun: "he", "she", "they", "what", "who".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pronoun {
    pub text: String,
    pub number: Number,
    pub person: Person,
    pub kind: PronounKind,
}

/// A copula: "is", "are", "was", "were".
/// Links subject to predicate. OLiA: Copula.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Copula {
    pub text: String,
    pub number: Number,
    pub person: Person,
    pub tense: Tense,
}

/// An auxiliary verb: "has", "will", "can", "does".
/// Modifies tense, aspect, mood. OLiA: AuxiliaryVerb.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Auxiliary {
    pub text: String,
    pub number: Option<Number>,
    pub tense: Option<Tense>,
}

/// Interjection kind — classified by communicative function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InterjectionKind {
    /// "hello", "hi", "hey" — greeting/opening.
    Greeting,
    /// "goodbye", "bye" — farewell/closing.
    Farewell,
    /// "oh", "wow" — expressive.
    Expressive,
    /// "yes", "no" — response.
    Response,
    /// "please", "thanks" — politeness.
    Politeness,
}

/// An interjection: "oh", "wow", "hello", "goodbye".
/// OLiA: Interjection.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Interjection {
    pub text: String,
    pub kind: InterjectionKind,
}

/// A particle: "not", "to" (infinitive marker).
/// OLiA: Particle.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Particle {
    pub text: String,
}

/// A numeral: "one", "two", "first".
/// OLiA: Numeral.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Numeral {
    pub text: String,
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
    Copula(Copula),
    Auxiliary(Auxiliary),
    Interjection(Interjection),
    Particle(Particle),
    Numeral(Numeral),
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
            Self::Copula(c) => &c.text,
            Self::Auxiliary(a) => &a.text,
            Self::Interjection(i) => &i.text,
            Self::Particle(p) => &p.text,
            Self::Numeral(n) => &n.text,
        }
    }

    pub fn number(&self) -> Option<Number> {
        match self {
            Self::Noun(n) => Some(n.number),
            Self::Verb(v) => Some(v.number),
            Self::Determiner(d) => d.number,
            Self::Pronoun(p) => Some(p.number),
            Self::Copula(c) => Some(c.number),
            Self::Auxiliary(a) => a.number,
            _ => None,
        }
    }

    pub fn person(&self) -> Option<Person> {
        match self {
            Self::Noun(n) => Some(n.person),
            Self::Verb(v) => Some(v.person),
            Self::Pronoun(p) => Some(p.person),
            Self::Copula(c) => Some(c.person),
            _ => None,
        }
    }

    /// Is this an anaphoric expression that needs resolution?
    /// Personal pronouns are anaphoric — they refer to previously mentioned entities.
    /// Interrogative pronouns are NOT anaphoric — they ask for new information.
    pub fn is_anaphoric(&self) -> bool {
        match self {
            Self::Pronoun(p) => p.kind == PronounKind::Personal,
            _ => false,
        }
    }

    /// Is this a farewell interjection? ("goodbye", "bye")
    pub fn is_farewell(&self) -> bool {
        match self {
            Self::Interjection(i) => i.kind == InterjectionKind::Farewell,
            _ => false,
        }
    }

    /// Is this an interrogative pronoun?
    /// Determined by the OLiA classification, not by the word itself.
    pub fn is_interrogative(&self) -> bool {
        match self {
            Self::Pronoun(p) => p.kind == PronounKind::Interrogative,
            _ => false,
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
            Self::Copula(_) => PosTag::Copula,
            Self::Auxiliary(_) => PosTag::Auxiliary,
            Self::Interjection(_) => PosTag::Interjection,
            Self::Particle(_) => PosTag::Particle,
            Self::Numeral(_) => PosTag::Numeral,
        }
    }
}

/// Part-of-speech tag — the category identifier (used by grammar layer).
/// This is the Entity for category-theoretic operations.
///
/// Categories are aligned with OLiA (Ontologies of Linguistic Annotation).
/// Reference: Chiarcos & Sukhareva, OLiA (Semantic Web journal, 2015)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum PosTag {
    Noun,
    Verb,
    Determiner,
    Adjective,
    Adverb,
    Preposition,
    Conjunction,
    Pronoun,
    /// OLiA: Copula — a verb linking subject to predicate ("is", "are", "was").
    Copula,
    /// OLiA: AuxiliaryVerb — verb modifying tense/aspect/mood ("has", "will", "can").
    Auxiliary,
    /// OLiA: Article — a subclass of Determiner ("a", "an", "the").
    Article,
    /// OLiA: Interjection — standalone exclamation ("oh", "wow", "hello").
    Interjection,
    /// OLiA: Particle — function word with grammatical role ("not", "to").
    Particle,
    /// OLiA: Numeral — number words ("one", "two", "first").
    Numeral,
}

impl PosTag {
    pub fn is_content(&self) -> bool {
        matches!(
            self,
            Self::Noun | Self::Verb | Self::Adjective | Self::Adverb | Self::Interjection
        )
    }

    pub fn is_function(&self) -> bool {
        !self.is_content()
    }

    /// Is this a copula? (OLiA: Copula)
    pub fn is_copula(&self) -> bool {
        matches!(self, Self::Copula)
    }

    /// Is this an auxiliary verb? (OLiA: AuxiliaryVerb)
    pub fn is_auxiliary(&self) -> bool {
        matches!(self, Self::Auxiliary)
    }

    /// Is this a pronoun? (OLiA: Pronoun)
    pub fn is_pronoun(&self) -> bool {
        matches!(self, Self::Pronoun)
    }

    /// Is this a noun? (OLiA: Noun)
    pub fn is_noun(&self) -> bool {
        matches!(self, Self::Noun)
    }

    /// Is this an adjective? (OLiA: Adjective)
    pub fn is_adjective(&self) -> bool {
        matches!(self, Self::Adjective)
    }

    /// Does this POS form questions when sentence-initial?
    /// Copulas and auxiliaries trigger question formation.
    pub fn is_question_forming(&self) -> bool {
        matches!(self, Self::Copula | Self::Auxiliary)
    }
}
