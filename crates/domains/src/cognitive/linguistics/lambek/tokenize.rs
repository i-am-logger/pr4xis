use super::reduce::TypedToken;
use super::types::LambekType;
use super::types::svo as svo_types;
use crate::cognitive::linguistics::language::Language;
use crate::cognitive::linguistics::orthography::distance;

/// Tokenize text into typed tokens using a language's lexicon.
///
/// This is a functor: Text → TypedTokens, parameterized by Language.
/// The tokenizer is language-agnostic — it calls language.lexical_lookup(),
/// not hardcoded word lists.
///
/// Unknown words go through the noisy channel adjunction:
/// Observation → closest_matches → corrected word's type.
pub fn tokenize(text: &str, language: &dyn Language) -> Vec<TypedToken> {
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
            let lambek_type = assign_type(&lower, i, language);
            Some(TypedToken {
                word: lower,
                lambek_type,
            })
        })
        .collect();

    // Post-processing: assign predicate adjective types based on context.
    assign_predicate_adjectives(&mut tokens);

    tokens
}

/// Tokenize with alternatives — returns tokens AND all possible types for each.
/// Used by the ambiguity-aware reducer to try multiple type assignments.
pub fn tokenize_with_alternatives(
    text: &str,
    language: &dyn Language,
) -> (Vec<TypedToken>, Vec<Vec<LambekType>>) {
    let cleaned = text
        .trim()
        .trim_end_matches(|c: char| c.is_ascii_punctuation());

    let words: Vec<&str> = cleaned.split_whitespace().collect();

    let mut tokens = Vec::new();
    let mut alternatives = Vec::new();

    for (i, word) in words.iter().enumerate() {
        let word_clean = word.trim_matches(|c: char| c.is_ascii_punctuation());
        if word_clean.is_empty() {
            continue;
        }
        let lower = word_clean.to_lowercase();

        // Get ALL entries from the language
        let all_entries = language.lexical_lookup_all(&lower);

        // Primary type assignment
        let primary_type = assign_type(&lower, i, language);

        // Alternative types from all entries
        let alt_types: Vec<LambekType> = all_entries
            .iter()
            .map(pos_to_lambek)
            .filter(|t| *t != primary_type)
            .collect();

        tokens.push(TypedToken {
            word: lower,
            lambek_type: primary_type,
        });
        alternatives.push(alt_types);
    }

    assign_predicate_adjectives(&mut tokens);

    (tokens, alternatives)
}

/// Assign a Lambek type to a word using the language's lexicon.
/// Position-sensitive: copulas/auxiliaries at sentence start get question types.
/// For ambiguous words (e.g. verbs with unknown transitivity), all entries
/// are considered and the best fit for the position is selected.
fn assign_type(word: &str, position: usize, language: &dyn Language) -> LambekType {
    // Look up ALL entries — a word can have multiple types
    let entries = language.lexical_lookup_all(word);
    let first = entries.first();
    let pos = first.map(|e| e.pos_tag());

    // Question-forming: sentence-initial copulas/auxiliaries
    if position == 0 {
        if pos.is_some_and(|p| p.is_question_forming()) {
            return svo_types::question_copula();
        }

        // Interrogative pronouns at sentence start → wh-question type
        if first.is_some_and(|e| e.is_interrogative()) {
            return svo_types::wh_what();
        }
    }

    // Copula in non-initial position → copula type (NP complement default)
    if pos.is_some_and(|p| p.is_copula()) && position > 0 {
        return svo_types::copula();
    }

    // For verbs with multiple transitivity options, prefer transitive
    // (it can still reduce with intransitive sentences via partial application).
    // The grammar resolves ambiguity through successful derivation.
    if let Some(best) = select_best_entry(&entries) {
        return pos_to_lambek(best);
    }

    // Noisy channel: unknown word → try spelling correction via the language
    if let Some(corrected_type) = try_spelling_correction(word, language) {
        return corrected_type;
    }

    // Unknown word — assume noun (open class default)
    svo_types::noun()
}

/// Select the best lexical entry when multiple are available.
/// Uses the first entry as default — the language orders entries by priority.
/// For verbs with ambiguous transitivity, the reducer handles both
/// by retrying with alternative types if the first attempt fails.
fn select_best_entry(
    entries: &[crate::cognitive::linguistics::lexicon::pos::LexicalEntry],
) -> Option<&crate::cognitive::linguistics::lexicon::pos::LexicalEntry> {
    entries.first()
}

/// Noisy channel adjunction: Observation → Correction → Intention.
/// Given an unknown word, find the closest known word and use its type.
fn try_spelling_correction(word: &str, language: &dyn Language) -> Option<LambekType> {
    let known = language.known_words();
    let matches = distance::closest_matches(word, &known, 1);
    if let Some((corrected, _)) = matches.first()
        && let Some(entry) = language.lexical_lookup(corrected)
    {
        return Some(pos_to_lambek(&entry));
    }
    None
}

/// Post-processing: when copula is followed by adjective, reassign types.
/// CCGbank: copula + adj → (S[dcl]\NP)/(S[adj]\NP) + S[adj]\NP
fn assign_predicate_adjectives(tokens: &mut [TypedToken]) {
    for i in 0..tokens.len().saturating_sub(1) {
        let is_copula = tokens[i].lambek_type == svo_types::copula();
        let is_adj = tokens[i + 1].lambek_type == svo_types::adjective();
        if is_copula && is_adj {
            tokens[i].lambek_type = svo_types::copula_adj();
            tokens[i + 1].lambek_type = svo_types::predicate_adjective();
        }
    }
}

/// Map a lexical entry's POS to its Lambek type.
/// Uses SVO type assignments — standard for Subject-Verb-Object languages.
fn pos_to_lambek(entry: &crate::cognitive::linguistics::lexicon::pos::LexicalEntry) -> LambekType {
    use crate::cognitive::linguistics::lexicon::pos::{LexicalEntry, Transitivity};
    match entry {
        LexicalEntry::Noun(_) => svo_types::noun(),
        LexicalEntry::Verb(v) => match v.transitivity {
            Transitivity::Intransitive => svo_types::intransitive_verb(),
            Transitivity::Transitive => svo_types::transitive_verb(),
            Transitivity::Ditransitive => svo_types::ditransitive_verb(),
        },
        LexicalEntry::Determiner(_) | LexicalEntry::Numeral(_) => svo_types::determiner(),
        LexicalEntry::Adjective(_) => svo_types::adjective(),
        LexicalEntry::Adverb(_) => svo_types::adverb(),
        LexicalEntry::Preposition(_) => svo_types::preposition(),
        LexicalEntry::Pronoun(_) => svo_types::proper_noun(),
        LexicalEntry::Conjunction(_) => svo_types::noun(),
        LexicalEntry::Copula(_) => svo_types::copula(),
        LexicalEntry::Auxiliary(_) => svo_types::intransitive_verb(),
        LexicalEntry::Interjection(_) => svo_types::noun(),
        LexicalEntry::Particle(_) => svo_types::adverb(),
    }
}
