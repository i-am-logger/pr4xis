use super::pos::*;

// Function word lexicon — the closed class of English function words.
//
// In linguistics, words divide into two classes:
// - Open class (content words): nouns, verbs, adjectives, adverbs
//   → infinite, productive, new words created constantly
//   → these come from WordNet at runtime
//
// - Closed class (function words): determiners, pronouns, prepositions,
//   conjunctions, copulas, auxiliaries, particles
//   → finite, fixed set, rarely changes
//   → these are declared here, classified by OLiA categories
//
// This is not a hack — closed-class function words ARE a finite enumeration
// in every natural language. OLiA classifies them; we declare the English instances.
//
// References:
// - Chiarcos & Sukhareva, OLiA (Semantic Web journal, 2015)
// - Jurafsky & Martin, Speech and Language Processing (2024) — §8.1 closed class

/// All English function words, classified by OLiA POS categories.
pub fn english_function_words() -> Vec<LexicalEntry> {
    let mut entries = Vec::new();

    // ---- Determiners (OLiA: Determiner) ----
    entries.extend([
        LexicalEntry::Determiner(Determiner {
            text: "the".into(),
            definiteness: Definiteness::Definite,
            number: None,
        }),
        LexicalEntry::Determiner(Determiner {
            text: "a".into(),
            definiteness: Definiteness::Indefinite,
            number: Some(Number::Singular),
        }),
        LexicalEntry::Determiner(Determiner {
            text: "an".into(),
            definiteness: Definiteness::Indefinite,
            number: Some(Number::Singular),
        }),
        LexicalEntry::Determiner(Determiner {
            text: "this".into(),
            definiteness: Definiteness::Demonstrative,
            number: Some(Number::Singular),
        }),
        LexicalEntry::Determiner(Determiner {
            text: "that".into(),
            definiteness: Definiteness::Demonstrative,
            number: Some(Number::Singular),
        }),
        LexicalEntry::Determiner(Determiner {
            text: "these".into(),
            definiteness: Definiteness::Demonstrative,
            number: Some(Number::Plural),
        }),
        LexicalEntry::Determiner(Determiner {
            text: "those".into(),
            definiteness: Definiteness::Demonstrative,
            number: Some(Number::Plural),
        }),
        LexicalEntry::Determiner(Determiner {
            text: "every".into(),
            definiteness: Definiteness::Quantifier,
            number: Some(Number::Singular),
        }),
        LexicalEntry::Determiner(Determiner {
            text: "some".into(),
            definiteness: Definiteness::Quantifier,
            number: None,
        }),
        LexicalEntry::Determiner(Determiner {
            text: "no".into(),
            definiteness: Definiteness::Quantifier,
            number: None,
        }),
        LexicalEntry::Determiner(Determiner {
            text: "all".into(),
            definiteness: Definiteness::Quantifier,
            number: Some(Number::Plural),
        }),
        LexicalEntry::Determiner(Determiner {
            text: "any".into(),
            definiteness: Definiteness::Quantifier,
            number: None,
        }),
        LexicalEntry::Determiner(Determiner {
            text: "each".into(),
            definiteness: Definiteness::Quantifier,
            number: Some(Number::Singular),
        }),
    ]);

    // ---- Copulas (OLiA: Copula) ----
    entries.extend([
        LexicalEntry::Copula(Copula {
            text: "is".into(),
            number: Number::Singular,
            person: Person::Third,
            tense: Tense::Present,
        }),
        LexicalEntry::Copula(Copula {
            text: "are".into(),
            number: Number::Plural,
            person: Person::Third,
            tense: Tense::Present,
        }),
        LexicalEntry::Copula(Copula {
            text: "am".into(),
            number: Number::Singular,
            person: Person::First,
            tense: Tense::Present,
        }),
        LexicalEntry::Copula(Copula {
            text: "was".into(),
            number: Number::Singular,
            person: Person::Third,
            tense: Tense::Past,
        }),
        LexicalEntry::Copula(Copula {
            text: "were".into(),
            number: Number::Plural,
            person: Person::Third,
            tense: Tense::Past,
        }),
    ]);

    // ---- Auxiliaries (OLiA: AuxiliaryVerb) ----
    for text in [
        "has", "have", "had", "do", "does", "did", "will", "would", "can", "could", "shall",
        "should", "may", "might", "must",
    ] {
        entries.push(LexicalEntry::Auxiliary(Auxiliary {
            text: text.into(),
            number: None,
            tense: None,
        }));
    }

    // ---- Pronouns (OLiA: Pronoun) ----
    entries.extend([
        LexicalEntry::Pronoun(Pronoun {
            text: "i".into(),
            number: Number::Singular,
            person: Person::First,
        }),
        LexicalEntry::Pronoun(Pronoun {
            text: "you".into(),
            number: Number::Singular,
            person: Person::Second,
        }),
        LexicalEntry::Pronoun(Pronoun {
            text: "he".into(),
            number: Number::Singular,
            person: Person::Third,
        }),
        LexicalEntry::Pronoun(Pronoun {
            text: "she".into(),
            number: Number::Singular,
            person: Person::Third,
        }),
        LexicalEntry::Pronoun(Pronoun {
            text: "it".into(),
            number: Number::Singular,
            person: Person::Third,
        }),
        LexicalEntry::Pronoun(Pronoun {
            text: "we".into(),
            number: Number::Plural,
            person: Person::First,
        }),
        LexicalEntry::Pronoun(Pronoun {
            text: "they".into(),
            number: Number::Plural,
            person: Person::Third,
        }),
        LexicalEntry::Pronoun(Pronoun {
            text: "me".into(),
            number: Number::Singular,
            person: Person::First,
        }),
        LexicalEntry::Pronoun(Pronoun {
            text: "him".into(),
            number: Number::Singular,
            person: Person::Third,
        }),
        LexicalEntry::Pronoun(Pronoun {
            text: "her".into(),
            number: Number::Singular,
            person: Person::Third,
        }),
        LexicalEntry::Pronoun(Pronoun {
            text: "us".into(),
            number: Number::Plural,
            person: Person::First,
        }),
        LexicalEntry::Pronoun(Pronoun {
            text: "them".into(),
            number: Number::Plural,
            person: Person::Third,
        }),
        LexicalEntry::Pronoun(Pronoun {
            text: "what".into(),
            number: Number::Singular,
            person: Person::Third,
        }),
        LexicalEntry::Pronoun(Pronoun {
            text: "who".into(),
            number: Number::Singular,
            person: Person::Third,
        }),
        LexicalEntry::Pronoun(Pronoun {
            text: "which".into(),
            number: Number::Singular,
            person: Person::Third,
        }),
    ]);

    // ---- Prepositions (OLiA: Preposition) ----
    for text in [
        "in", "on", "at", "with", "to", "from", "by", "for", "of", "about", "into", "through",
        "during", "before", "after", "above", "below", "between", "under", "over",
    ] {
        entries.push(LexicalEntry::Preposition(Preposition { text: text.into() }));
    }

    // ---- Conjunctions (OLiA: Conjunction) ----
    for text in [
        "and", "but", "or", "so", "yet", "nor", "because", "although", "if", "when",
    ] {
        entries.push(LexicalEntry::Conjunction(Conjunction { text: text.into() }));
    }

    // ---- Particles (OLiA: Particle) ----
    entries.extend([
        LexicalEntry::Particle(Particle { text: "not".into() }),
        LexicalEntry::Particle(Particle { text: "to".into() }),
    ]);

    // ---- Interjections (OLiA: Interjection) ----
    for text in [
        "hello", "hi", "hey", "oh", "wow", "yes", "no", "please", "thanks", "goodbye", "bye",
    ] {
        entries.push(LexicalEntry::Interjection(Interjection {
            text: text.into(),
        }));
    }

    entries
}

/// Look up a function word by text. Returns the first match.
pub fn lookup(text: &str) -> Option<LexicalEntry> {
    english_function_words()
        .into_iter()
        .find(|w| w.text() == text)
}

/// Look up all function word entries matching a text.
pub fn lookup_all(text: &str) -> Vec<LexicalEntry> {
    english_function_words()
        .into_iter()
        .filter(|w| w.text() == text)
        .collect()
}
