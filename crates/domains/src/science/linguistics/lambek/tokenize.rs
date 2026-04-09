use super::reduce::TypedToken;
use super::types::LambekType;
use super::types::english as en_types;
use crate::science::linguistics::lexicon::pos::LexicalEntry;
use crate::science::linguistics::lexicon::vocabulary;

/// Tokenize text into typed tokens using the lexicon ontology.
///
/// This is a functor: Text → TypedTokens.
/// Characters become words (via whitespace/punctuation boundaries).
/// Words become typed tokens (via lexicon lookup + Lambek type assignment).
/// Position-sensitive: "is" at sentence start gets question type.
pub fn tokenize(text: &str) -> Vec<TypedToken> {
    let cleaned = text
        .trim()
        .trim_end_matches(|c: char| c.is_ascii_punctuation());

    let words: Vec<&str> = cleaned.split_whitespace().collect();

    words
        .iter()
        .enumerate()
        .filter_map(|(i, word)| {
            let word_clean = word.trim_matches(|c: char| c.is_ascii_punctuation());
            if word_clean.is_empty() {
                return None;
            }
            let lower = word_clean.to_lowercase();
            let lambek_type = assign_type(&lower, i, &words);
            Some(TypedToken {
                word: lower,
                lambek_type,
            })
        })
        .collect()
}

/// Assign a Lambek type to a word using the lexicon ontology.
/// Position-sensitive: auxiliaries at sentence start get question types.
fn assign_type(word: &str, position: usize, _words: &[&str]) -> LambekType {
    // Question-forming auxiliaries at sentence start
    if position == 0 && is_auxiliary(word) {
        return en_types::question_copula();
    }

    // "what" at sentence start → wh-question
    if position == 0 && word == "what" {
        return en_types::wh_what();
    }

    // Copula/auxiliary in non-initial position → copula (transitive verb-like)
    if is_copula(word) && position > 0 {
        return en_types::copula();
    }

    // Look up in lexicon ontology
    if let Some(entry) = vocabulary::lookup(word) {
        return pos_to_lambek(&entry);
    }

    let entries = vocabulary::lookup_all(word);
    if let Some(entry) = entries.first() {
        return pos_to_lambek(entry);
    }

    // Unknown word — assume noun
    en_types::noun()
}

fn is_auxiliary(word: &str) -> bool {
    matches!(
        word,
        "is" | "are"
            | "was"
            | "were"
            | "does"
            | "do"
            | "did"
            | "has"
            | "have"
            | "had"
            | "can"
            | "could"
            | "will"
            | "would"
            | "shall"
            | "should"
            | "may"
            | "might"
    )
}

fn is_copula(word: &str) -> bool {
    matches!(word, "is" | "are" | "was" | "were")
}

/// Map a lexical entry's POS to its Lambek type.
fn pos_to_lambek(entry: &LexicalEntry) -> LambekType {
    match entry {
        LexicalEntry::Noun(_) => en_types::noun(),
        LexicalEntry::Verb(v) => {
            use crate::science::linguistics::lexicon::pos::Transitivity;
            match v.transitivity {
                Transitivity::Intransitive => en_types::intransitive_verb(),
                Transitivity::Transitive => en_types::transitive_verb(),
                Transitivity::Ditransitive => en_types::ditransitive_verb(),
            }
        }
        LexicalEntry::Determiner(_) => en_types::determiner(),
        LexicalEntry::Adjective(_) => en_types::adjective(),
        LexicalEntry::Adverb(_) => en_types::adverb(),
        LexicalEntry::Preposition(_) => en_types::preposition(),
        LexicalEntry::Pronoun(_) => en_types::proper_noun(),
        LexicalEntry::Conjunction(_) => en_types::noun(),
    }
}
