use super::pos::*;

/// Built-in English vocabulary with rich lexical entries.
pub fn english() -> Vec<LexicalEntry> {
    let mut entries = Vec::new();

    // ---- Determiners ----
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
    ]);

    // ---- Nouns (singular + plural pairs) ----
    let noun_pairs: Vec<(&str, &str, Countability)> = vec![
        ("dog", "dogs", Countability::Countable),
        ("cat", "cats", Countability::Countable),
        ("man", "men", Countability::Countable),
        ("woman", "women", Countability::Countable),
        ("child", "children", Countability::Countable),
        ("book", "books", Countability::Countable),
        ("city", "cities", Countability::Countable),
        ("park", "parks", Countability::Countable),
        ("car", "cars", Countability::Countable),
        ("idea", "ideas", Countability::Countable),
        ("house", "houses", Countability::Countable),
        ("tree", "trees", Countability::Countable),
        ("bird", "birds", Countability::Countable),
        ("river", "rivers", Countability::Countable),
        ("bank", "banks", Countability::Countable),
        ("day", "days", Countability::Countable),
    ];
    for (sg, pl, count) in noun_pairs {
        entries.push(LexicalEntry::Noun(Noun {
            text: sg.into(),
            number: Number::Singular,
            person: Person::Third,
            countability: count,
            kind: NounKind::Common,
        }));
        entries.push(LexicalEntry::Noun(Noun {
            text: pl.into(),
            number: Number::Plural,
            person: Person::Third,
            countability: count,
            kind: NounKind::Common,
        }));
    }

    // Uncountable nouns
    for text in ["water", "time", "music", "air", "light"] {
        entries.push(LexicalEntry::Noun(Noun {
            text: text.into(),
            number: Number::Singular,
            person: Person::Third,
            countability: Countability::Uncountable,
            kind: NounKind::Common,
        }));
    }

    // ---- Verbs (lemma + conjugated forms) ----
    let verb_defs: Vec<(&str, &str, &str, Transitivity)> = vec![
        ("run", "runs", "ran", Transitivity::Intransitive),
        ("see", "sees", "saw", Transitivity::Transitive),
        ("give", "gives", "gave", Transitivity::Ditransitive),
        ("take", "takes", "took", Transitivity::Transitive),
        ("make", "makes", "made", Transitivity::Transitive),
        ("go", "goes", "went", Transitivity::Intransitive),
        ("say", "says", "said", Transitivity::Transitive),
        ("know", "knows", "knew", Transitivity::Transitive),
        ("think", "thinks", "thought", Transitivity::Intransitive),
        ("want", "wants", "wanted", Transitivity::Transitive),
        ("like", "likes", "liked", Transitivity::Transitive),
        ("read", "reads", "read", Transitivity::Transitive),
    ];
    for (lemma, s3, past, trans) in verb_defs {
        // Base/plural present: "they run"
        entries.push(LexicalEntry::Verb(Verb {
            text: lemma.into(),
            lemma: lemma.into(),
            number: Number::Plural,
            person: Person::Third,
            tense: Tense::Present,
            transitivity: trans,
        }));
        // 3rd singular present: "she runs"
        entries.push(LexicalEntry::Verb(Verb {
            text: s3.into(),
            lemma: lemma.into(),
            number: Number::Singular,
            person: Person::Third,
            tense: Tense::Present,
            transitivity: trans,
        }));
        // Past: "she ran"
        entries.push(LexicalEntry::Verb(Verb {
            text: past.into(),
            lemma: lemma.into(),
            number: Number::Singular,
            person: Person::Third,
            tense: Tense::Past,
            transitivity: trans,
        }));
    }

    // Copulas (OLiA: Copula — links subject to predicate)
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

    // Auxiliaries (OLiA: AuxiliaryVerb — modifies tense/aspect/mood)
    entries.extend([
        LexicalEntry::Auxiliary(Auxiliary {
            text: "has".into(),
            number: Some(Number::Singular),
            tense: Some(Tense::Present),
        }),
        LexicalEntry::Auxiliary(Auxiliary {
            text: "have".into(),
            number: Some(Number::Plural),
            tense: Some(Tense::Present),
        }),
        LexicalEntry::Auxiliary(Auxiliary {
            text: "had".into(),
            number: None,
            tense: Some(Tense::Past),
        }),
        LexicalEntry::Auxiliary(Auxiliary {
            text: "do".into(),
            number: Some(Number::Plural),
            tense: Some(Tense::Present),
        }),
        LexicalEntry::Auxiliary(Auxiliary {
            text: "does".into(),
            number: Some(Number::Singular),
            tense: Some(Tense::Present),
        }),
        LexicalEntry::Auxiliary(Auxiliary {
            text: "did".into(),
            number: None,
            tense: Some(Tense::Past),
        }),
        LexicalEntry::Auxiliary(Auxiliary {
            text: "will".into(),
            number: None,
            tense: Some(Tense::Future),
        }),
        LexicalEntry::Auxiliary(Auxiliary {
            text: "would".into(),
            number: None,
            tense: None,
        }),
        LexicalEntry::Auxiliary(Auxiliary {
            text: "can".into(),
            number: None,
            tense: None,
        }),
        LexicalEntry::Auxiliary(Auxiliary {
            text: "could".into(),
            number: None,
            tense: None,
        }),
        LexicalEntry::Auxiliary(Auxiliary {
            text: "shall".into(),
            number: None,
            tense: None,
        }),
        LexicalEntry::Auxiliary(Auxiliary {
            text: "should".into(),
            number: None,
            tense: None,
        }),
        LexicalEntry::Auxiliary(Auxiliary {
            text: "may".into(),
            number: None,
            tense: None,
        }),
        LexicalEntry::Auxiliary(Auxiliary {
            text: "might".into(),
            number: None,
            tense: None,
        }),
    ]);

    // ---- Adjectives ----
    for text in [
        "big", "small", "red", "blue", "green", "old", "new", "good", "bad", "happy", "sad",
        "fast", "slow", "hot", "cold", "tall", "short", "long", "young", "dark",
    ] {
        entries.push(LexicalEntry::Adjective(Adjective { text: text.into() }));
    }

    // ---- Adverbs ----
    for text in [
        "quickly", "slowly", "very", "well", "often", "never", "always", "here", "there", "now",
    ] {
        entries.push(LexicalEntry::Adverb(Adverb { text: text.into() }));
    }

    // ---- Prepositions ----
    for text in [
        "in", "on", "at", "with", "to", "from", "by", "for", "of", "about",
    ] {
        entries.push(LexicalEntry::Preposition(Preposition { text: text.into() }));
    }

    // ---- Conjunctions ----
    for text in ["and", "but", "or", "so", "yet"] {
        entries.push(LexicalEntry::Conjunction(Conjunction { text: text.into() }));
    }

    // ---- Pronouns ----
    for (text, num, per) in [
        ("I", Number::Singular, Person::First),
        ("you", Number::Singular, Person::Second),
        ("he", Number::Singular, Person::Third),
        ("she", Number::Singular, Person::Third),
        ("it", Number::Singular, Person::Third),
        ("we", Number::Plural, Person::First),
        ("they", Number::Plural, Person::Third),
    ] {
        entries.push(LexicalEntry::Pronoun(Pronoun {
            text: text.into(),
            number: num,
            person: per,
            kind: PronounKind::Personal,
        }));
    }

    entries
}

/// Look up a word by text. Returns the first match.
pub fn lookup(text: &str) -> Option<LexicalEntry> {
    english().into_iter().find(|w| w.text() == text)
}

/// Look up all entries matching a text (handles homographs like "read").
pub fn lookup_all(text: &str) -> Vec<LexicalEntry> {
    english().into_iter().filter(|w| w.text() == text).collect()
}
