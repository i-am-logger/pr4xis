use super::reduce::TypedToken;
use super::types::LambekType;
use super::types::english as en_types;
use crate::science::linguistics::lexicon::function_words;
use crate::science::linguistics::lexicon::pos::{LexicalEntry, PosTag};
use crate::science::linguistics::lexicon::vocabulary;
use crate::science::linguistics::orthography::distance;

/// Tokenize text into typed tokens using the lexicon ontology.
///
/// This is a functor: Text → TypedTokens.
/// Characters become words (via whitespace/punctuation boundaries).
/// Words become typed tokens (via lexicon lookup + Lambek type assignment).
/// Position-sensitive: copulas/auxiliaries at sentence start get question type.
///
/// Unknown words go through the noisy channel adjunction:
/// Observation → closest_matches → corrected word's type.
pub fn tokenize(text: &str) -> Vec<TypedToken> {
    let cleaned = text
        .trim()
        .trim_end_matches(|c: char| c.is_ascii_punctuation());

    let words: Vec<&str> = cleaned.split_whitespace().collect();

    let mut tokens: Vec<TypedToken> = words
        .iter()
        .enumerate()
        .filter_map(|(i, word)| {
            let word_clean = word.trim_matches(|c: char| c.is_ascii_punctuation());
            if word_clean.is_empty() {
                return None;
            }
            let lower = word_clean.to_lowercase();
            let lambek_type = assign_type(&lower, i);
            Some(TypedToken {
                word: lower,
                lambek_type,
            })
        })
        .collect();

    // Post-processing: assign predicate adjective types based on context.
    // When a copula is followed by an adjective, the adjective gets S[adj]\NP
    // and the copula gets (S[dcl]\NP)/(S[adj]\NP).
    // From Hockenmaier & Steedman (2007), CCGbank.
    assign_predicate_adjectives(&mut tokens);

    tokens
}

/// Assign a Lambek type to a word using the lexicon ontology.
/// Position-sensitive: copulas/auxiliaries at sentence start get question types.
fn assign_type(word: &str, position: usize) -> LambekType {
    // Look up in function word lexicon (closed class)
    let entry = function_words::lookup(word);
    let pos = entry.as_ref().map(|e| e.pos_tag());

    // Question-forming: copulas/auxiliaries at sentence start
    if position == 0 {
        match pos {
            Some(PosTag::Copula) | Some(PosTag::Auxiliary) => {
                return en_types::question_copula();
            }
            _ => {}
        }

        // "what" at sentence start → wh-question
        if word == "what" {
            return en_types::wh_what();
        }
    }

    // Copula in non-initial position → copula type (NP complement default)
    if pos == Some(PosTag::Copula) && position > 0 {
        return en_types::copula();
    }

    // Use function word entry if found
    if let Some(entry) = &entry {
        return pos_to_lambek(entry);
    }

    // Fall back to content word vocabulary
    if let Some(entry) = vocabulary::lookup(word) {
        return pos_to_lambek(&entry);
    }
    let entries = vocabulary::lookup_all(word);
    if let Some(entry) = entries.first() {
        return pos_to_lambek(entry);
    }

    // Noisy channel: unknown word → try spelling correction
    // Observation → closest_matches → corrected word's type
    if let Some(corrected_type) = try_spelling_correction(word) {
        return corrected_type;
    }

    // Unknown word — assume noun (open class default)
    en_types::noun()
}

/// Noisy channel adjunction: Observation → Correction → Intention.
/// Given an unknown word, find the closest known word and use its type.
fn try_spelling_correction(word: &str) -> Option<LambekType> {
    // Build candidate list from known words
    let fw = function_words::english_function_words();
    let fw_texts: Vec<&str> = fw.iter().map(|e| e.text()).collect();

    let vocab = vocabulary::english();
    let vocab_texts: Vec<&str> = vocab.iter().map(|e| e.text()).collect();

    // Try function words first (distance 1 only — performance errors)
    let fw_matches = distance::closest_matches(word, &fw_texts, 1);
    if let Some((corrected, _)) = fw_matches.first()
        && let Some(entry) = function_words::lookup(corrected)
    {
        return Some(pos_to_lambek(&entry));
    }

    // Try content words (distance 1)
    let vocab_matches = distance::closest_matches(word, &vocab_texts, 1);
    if let Some((corrected, _)) = vocab_matches.first()
        && let Some(entry) = vocabulary::lookup(corrected)
    {
        return Some(pos_to_lambek(&entry));
    }

    None
}

/// Post-processing: when copula is followed by adjective, reassign types.
/// CCGbank: copula + adj → (S[dcl]\NP)/(S[adj]\NP) + S[adj]\NP
fn assign_predicate_adjectives(tokens: &mut [TypedToken]) {
    for i in 0..tokens.len().saturating_sub(1) {
        let is_copula = tokens[i].lambek_type == en_types::copula();
        let is_adj = tokens[i + 1].lambek_type == en_types::adjective();
        if is_copula && is_adj {
            tokens[i].lambek_type = en_types::copula_adj();
            tokens[i + 1].lambek_type = en_types::predicate_adjective();
        }
    }
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
        LexicalEntry::Determiner(_) | LexicalEntry::Numeral(_) => en_types::determiner(),
        LexicalEntry::Adjective(_) => en_types::adjective(),
        LexicalEntry::Adverb(_) => en_types::adverb(),
        LexicalEntry::Preposition(_) => en_types::preposition(),
        LexicalEntry::Pronoun(_) => en_types::proper_noun(),
        LexicalEntry::Conjunction(_) => en_types::noun(),
        LexicalEntry::Copula(_) => en_types::copula(),
        LexicalEntry::Auxiliary(_) => en_types::intransitive_verb(),
        LexicalEntry::Interjection(_) => en_types::noun(),
        LexicalEntry::Particle(_) => en_types::adverb(),
    }
}
