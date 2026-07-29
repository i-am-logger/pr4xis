#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;

/// Grammatical number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum Number {
    Singular,
    Plural,
}

/// Grammatical person.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum Person {
    First,
    Second,
    Third,
}

/// Verb tense.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum Tense {
    Present,
    Past,
    Future,
}

/// Noun countability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum Countability {
    Countable,
    Uncountable,
}

/// Noun type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum NounKind {
    Common,
    Proper,
}

/// Verb transitivity.
///
/// `#[repr(u8)]` pins each variant to a 1-byte discriminant (`Transitive` = 0,
/// `Intransitive` = 1, `Ditransitive` = 2) so a packed byte-run of discriminants
/// casts zero-copy to `&[Transitivity]` — the layout the
/// [`VerbTransitivityIndex`](crate::cognitive::linguistics::english::verb_transitivity_index)
/// archive relies on (the `word_index` id-run pattern with 1-byte elements). The
/// `rkyv` derives let `Transitivity` also ride the `FunctionWordStore`
/// `LexicalEntry` mirror (as a `Verb` field) under `prx`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[repr(u8)]
pub enum Transitivity {
    Transitive,
    Intransitive,
    Ditransitive,
}

/// Determiner subclass on the definiteness / quantification axis.
///
/// Definite vs Indefinite is the core *definiteness* contrast — the property
/// expressed by the definite article (Lyons 1999, *Definiteness*, Cambridge,
/// DOI 10.1017/CBO9780511605789). Demonstrative and Quantifier are the
/// recognized determiner subclasses (Huddleston & Pullum 2002, CGEL ch.5), NOT
/// further definiteness *values*: on the orthodox semantics a demonstrative NP
/// is *itself* definite, and quantifiers cross-cut the ±definite axis
/// (Abbott 2010, *Reference*, OUP).
///
/// PRAXIS-HONESTY FLAG (audit 2026-06-12 B-1): the four labels are each citable,
/// but they live on two axes — this is a flat *determiner subclass* feature, not
/// the binary ±definite contrast the legacy name "Definiteness" implied. Renamed
/// `Definiteness` → `DeterminerKind` to name what it is. `Indefinite` is the
/// unmarked default (Lyons 1999 §1; H&P ch.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum DeterminerKind {
    /// The definite article `the` (Lyons 1999 — the definiteness contrast).
    Definite,
    /// The indefinite articles `a`/`an` and bare indefinites `some`/`any` — the
    /// unmarked member (Lyons 1999 §1).
    Indefinite,
    /// Demonstratives `this`/`that`/`these`/`those` — a subtype of *definite*
    /// (Abbott 2010), kept distinct as a determiner subclass (H&P ch.5).
    Demonstrative,
    /// Quantificational determiners `every`/`each`/`all`/`no` — the
    /// quantificational axis that cross-cuts ±definite (Barwise & Cooper 1981;
    /// Abbott 2010).
    Quantifier,
}

/// The "expected answer type" of an interrogative PRONOUN or DETERMINER —
/// "who"/"whom"/"whose" ask about a PERSON, "what" asks about a THING, and
/// "which" asks the hearer to SELECT from a contextually given set (neither
/// strictly personal nor strictly nonpersonal — "which of you" and "which
/// book" are both well-formed). Cross-linguistic typology of interrogative
/// categories (Cysouw 2004, "Interrogative words: an exercise in lexical
/// typology", handout, session on question formation in Bantu, ZAS Berlin,
/// 13 Feb 2004, §3.2 table (9): "who/whom/whose → PERSON, what → THING,
/// which → SELECTION"; building on Ultan 1978, "Some general characteristics
/// of interrogative systems", in J. Greenberg (ed.) *Universals of Human
/// Language* vol.4, Stanford UP, pp.211-248, the cross-linguistic source
/// Cysouw's own table cites). This is the loaded feature the closed
/// `wh_adverb`/`wh_manner_adverb`-style hand-authored checks this project
/// otherwise avoids would have to fall back on; carried on both
/// [`Pronoun::referent_role`] and [`Determiner::referent_role`] since English
/// realizes the SAME lexical item ("what"/"which"/"whose") as either POS
/// depending on whether it heads its own NP or modifies one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum WhReferentRole {
    /// "who", "whom", "whose" — Cysouw (2004) §3.2 table (9) PERSON.
    Person,
    /// "what" — Cysouw (2004) §3.2 table (9) THING.
    Thing,
    /// "which" — Cysouw (2004) §3.2 table (9) SELECTION (choice from a
    /// contextually restricted set, orthogonal to person-hood).
    Selection,
}

/// The semantic role of an interrogative ADVERB — "how" asks about MANNER,
/// "why" about REASON, "where" about PLACE, "when" about TIME. Same source
/// as [`WhReferentRole`]: Cysouw (2004) §3.2 table (9); the loaded OLiA
/// `InterrogativeAdverb` class (carried in [`Adverb::olia_class`]) covers all
/// four uniformly, so this is the finer feature that distinguishes them —
/// the SAME kind of codec-lowering [`DeterminerKind`]/[`PronounKind`] already
/// perform on a different axis than their shared OLiA fragment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum WhAdverbRole {
    /// "how" — Cysouw (2004) §3.2 table (9) MANNER.
    Manner,
    /// "why" — Cysouw (2004) §3.2 table (9) REASON.
    Reason,
    /// "where" — Cysouw (2004) §3.2 table (9) PLACE.
    Place,
    /// "when" — Cysouw (2004) §3.2 table (9) TIME.
    Time,
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
    /// The loaded OLiA class fragment for a morphologically MARKED form of
    /// this verb, if any — e.g. the EAGLES gerund-participle merger class
    /// (`ing`) when the entry's `text` is the verb's -ing form (CGEL pp.
    /// 1220–1222: the gerund/participle distinction "can't be sustained" —
    /// OLiA's `ing` class IS that merger). Populated only by morphological
    /// derivation ([`form_level_class`](super::olia)); `None` for a base
    /// form. The OLiA→CCG functor projects it to the form's categories,
    /// exactly as the Pronoun/Adverb/Determiner carriers below.
    pub olia_class: Option<String>,
}

/// A determiner: "the", "a", "this", "every".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Determiner {
    pub text: String,
    /// The determiner's subclass on the definiteness/quantification axis
    /// ([`DeterminerKind`]).
    pub kind: DeterminerKind,
    pub number: Option<Number>,
    /// The loaded OLiA class fragment (e.g. `InterrogativeDeterminer`), if this
    /// determiner carries one — the universal grammatical-class identity the
    /// OLiA→CCG functor projects to a category. `None` for an ordinary
    /// determiner. Decoded once from the LMF `Sense.subcat`.
    pub olia_class: Option<String>,
    /// This determiner's [`WhReferentRole`] ("what"/"which"/"whose" ask
    /// about a thing/selection/person respectively), if it is interrogative.
    /// `None` for an ordinary (non-interrogative) determiner. Decoded once
    /// from the LMF `Sense.synset`.
    pub referent_role: Option<WhReferentRole>,
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
    /// The loaded OLiA class fragment (e.g. `InterrogativeAdverb`), if this
    /// adverb carries one — the slot a plain adverb lacked, so an interrogative
    /// adverb (`where`/`when`/`why`/`how`) is no longer dropped or mistyped.
    /// `None` for an ordinary adverb. Decoded once from the LMF `Sense.subcat`.
    pub olia_class: Option<String>,
    /// This adverb's [`WhAdverbRole`] (manner/reason/place/time), if it is
    /// interrogative. `None` for an ordinary (non-interrogative) adverb.
    /// Decoded once from the LMF `Sense.synset`.
    pub role: Option<WhAdverbRole>,
}

/// A preposition: "in", "on", "with".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Preposition {
    pub text: String,
}

/// A conjunction: "and", "but", "or", "since", "because".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Conjunction {
    pub text: String,
    /// The loaded OLiA class fragment (e.g. `SubordinatingConjunction`), if
    /// this conjunction carries one — the universal grammatical-class
    /// identity the OLiA→CCG functor projects to a category, same pattern
    /// as the Pronoun/Adverb/Determiner carriers. `None` for a plain
    /// coordinating conjunction ("and", "or", "but"). Decoded once from the
    /// LMF `Sense.subcat`.
    pub olia_class: Option<String>,
}

/// Pronoun kind — from OLiA classification.
/// OLiA: PersonalPronoun, InterrogativePronoun, DemonstrativePronoun, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
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
    /// "mine", "yours", "his", "hers", "ours", "theirs" (independent) and
    /// "my", "your", "his", "its", "our", "their" (dependent/genitive
    /// determiner use) — Huddleston & Pullum 2002 Ch. 5 §10's genitive
    /// pronoun class, kept distinct from `Personal`: the independent forms
    /// ("mine" = "a gold mine", not just the possessive) are far more prone
    /// to colliding with an unrelated open-class common noun than the
    /// plain/oblique personal pronouns are, so callers gating entity-hood on
    /// "is this a pronoun" must be able to exclude this class specifically.
    Possessive,
}

/// A pronoun: "he", "she", "they", "what", "who".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pronoun {
    pub text: String,
    pub number: Number,
    pub person: Person,
    pub kind: PronounKind,
    /// The loaded OLiA class fragment (e.g. `InterrogativePronoun`), if this
    /// pronoun carries one — the universal grammatical-class identity the
    /// OLiA→CCG functor projects to a category. `None` for an ordinary pronoun.
    /// Decoded once from the LMF `Sense.subcat`.
    pub olia_class: Option<String>,
    /// This pronoun's [`WhReferentRole`] ("who"/"what"/"which" ask about a
    /// person/thing/selection respectively), if it is interrogative. `None`
    /// for an ordinary (non-interrogative) pronoun. Decoded once from the
    /// LMF `Sense.synset`.
    pub referent_role: Option<WhReferentRole>,
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

/// Interjection communicative function, after Ameka 1992 ("Interjections: the
/// universal yet neglected part of speech", J. Pragmatics 18(2/3):101-118,
/// a combined double issue -- DOI 10.1016/0378-2166(92)90048-G), whose three
/// top-level functions are
/// EXPRESSIVE (symptoms of the speaker's state), CONATIVE (directed at an
/// auditor — demanding attention/action), and PHATIC (establishing/maintaining
/// contact — greetings, farewells, back-channel feedback). OLiA and H&P 2002
/// ch.16 give a single `Interjection` class with no sub-functions, so this kind
/// is a praxis feature (Wierzbicka 1992; Wharton 2003 refine the semantic side).
///
/// `Expressive` is Ameka's expressive (top-level); `Greeting`/`Farewell`/
/// `Response` are subtypes of his PHATIC; `Politeness` is a phatic interactional
/// routine. `Conative` is Ameka's third function (audit 2026-06-12 B-2 fixed the
/// prior gap — conative items like `sh!`/`psst` used to silently default to
/// Expressive).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum InterjectionKind {
    /// "hello", "hi", "hey" — greeting (Ameka PHATIC).
    Greeting,
    /// "goodbye", "bye" — farewell (Ameka PHATIC).
    Farewell,
    /// "oh", "wow", "ouch" — expressive of the speaker's state (Ameka EXPRESSIVE,
    /// the prototypical/most-frequent interjection function — the unmarked default).
    Expressive,
    /// "yes", "no", "uh-huh" — response / back-channel feedback (Ameka PHATIC).
    Response,
    /// "please", "thanks" — politeness routine (Ameka phatic-interactional).
    Politeness,
    /// "sh!", "psst", summoning "hey!" — directed at an auditor, demanding
    /// attention or action (Ameka CONATIVE).
    Conative,
}

/// A response interjection's affirmative/negative polarity (Holmberg 2016,
/// *The Syntax of Yes and No*, Oxford University Press — polar response
/// particles as their own grammatical category, cross-linguistically
/// distinct from ordinary negation) — orthogonal to [`InterjectionKind`]
/// (which classifies the pragmatic FUNCTION, Ameka 1992). Only ever
/// meaningful when [`Interjection::kind`] is [`InterjectionKind::Response`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum Polarity {
    Affirmative,
    Negative,
}

/// An interjection: "oh", "wow", "hello", "goodbye".
/// OLiA: Interjection.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Interjection {
    pub text: String,
    pub kind: InterjectionKind,
    /// `Some` iff `kind == Response` — see [`Polarity`].
    pub polarity: Option<Polarity>,
}

/// A particle: "not", "to" (infinitive marker).
/// OLiA: Particle.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Particle {
    pub text: String,
    /// The loaded OLiA class fragment (e.g. `InfinitiveParticle`, the
    /// subclass distinguishing "to" the infinitive marker from an ordinary
    /// particle), if this particle carries one — the universal grammatical-
    /// class identity the OLiA→CCG functor projects to a category, the same
    /// pattern as the Pronoun/Adverb/Determiner/Conjunction carriers.
    /// `None` for a particle with no finer subclass. Decoded once from the
    /// LMF `Sense.subcat`.
    pub olia_class: Option<String>,
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

    /// This entry's response polarity ("yes"/"ok" → `Affirmative`, "no" →
    /// `Negative`), or `None` for every non-Response entry. See [`Polarity`].
    pub fn response_polarity(&self) -> Option<Polarity> {
        match self {
            Self::Interjection(i) => i.polarity,
            _ => None,
        }
    }

    /// The loaded OLiA class fragment this entry carries, if any — the
    /// universal grammatical-class identity (e.g. `InterrogativeAdverb`) the
    /// OLiA→CCG functor projects to a category. Carried on the Pronoun /
    /// Adverb / Determiner carriers, decoded once from the LMF `Sense.subcat`.
    pub fn olia_class(&self) -> Option<&str> {
        match self {
            Self::Pronoun(p) => p.olia_class.as_deref(),
            Self::Adverb(a) => a.olia_class.as_deref(),
            Self::Determiner(d) => d.olia_class.as_deref(),
            Self::Verb(v) => v.olia_class.as_deref(),
            Self::Conjunction(c) => c.olia_class.as_deref(),
            Self::Particle(p) => p.olia_class.as_deref(),
            _ => None,
        }
    }

    /// Is this an interrogative word? A typed query over the loaded OLiA class
    /// (any `Interrogative*` fragment), across pronouns, adverbs, AND
    /// determiners — not a Pronoun-only enum-flag compare.
    pub fn is_interrogative(&self) -> bool {
        self.olia_class()
            .is_some_and(|c| c.starts_with("Interrogative"))
    }

    /// This entry's [`WhReferentRole`] (person/thing/selection), if it is an
    /// interrogative pronoun or determiner. `None` for every other entry —
    /// carried on the Pronoun/Determiner carriers, decoded once from the LMF
    /// `Sense.synset`.
    pub fn wh_referent_role(&self) -> Option<WhReferentRole> {
        match self {
            Self::Pronoun(p) => p.referent_role,
            Self::Determiner(d) => d.referent_role,
            _ => None,
        }
    }

    /// This entry's [`WhAdverbRole`] (manner/reason/place/time), if it is an
    /// interrogative adverb. `None` for every other entry — carried on the
    /// Adverb carrier, decoded once from the LMF `Sense.synset`.
    pub fn wh_adverb_role(&self) -> Option<WhAdverbRole> {
        match self {
            Self::Adverb(a) => a.role,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
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
