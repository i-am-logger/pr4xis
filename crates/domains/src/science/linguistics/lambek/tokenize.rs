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
pub fn tokenize(text: &str) -> Vec<TypedToken> {
    let cleaned = text
        .trim()
        .trim_end_matches(|c: char| c.is_ascii_punctuation());

    cleaned
        .split_whitespace()
        .filter_map(|word| {
            let word_clean = word.trim_matches(|c: char| c.is_ascii_punctuation());
            if word_clean.is_empty() {
                return None;
            }
            let lower = word_clean.to_lowercase();
            let lambek_type = assign_type(&lower);
            Some(TypedToken {
                word: lower,
                lambek_type,
            })
        })
        .collect()
}

/// Assign a Lambek type to a word using the lexicon ontology.
///
/// Looks up the word in the vocabulary, gets its POS tag,
/// and maps POS to the corresponding Lambek type.
fn assign_type(word: &str) -> LambekType {
    // Look up in lexicon ontology
    if let Some(entry) = vocabulary::lookup(word) {
        return pos_to_lambek(&entry);
    }

    // Try all entries for the word (first match)
    let entries = vocabulary::lookup_all(word);
    if let Some(entry) = entries.first() {
        return pos_to_lambek(entry);
    }

    // Unknown word — assume noun (most common for unknown words)
    en_types::noun()
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
        LexicalEntry::Pronoun(_) => en_types::proper_noun(), // pronouns act as NP
        LexicalEntry::Conjunction(_) => {
            // Conjunction: simplified as S\(S/S) — but for now treat as unknown
            en_types::noun() // TODO: proper conjunction type
        }
    }
}
