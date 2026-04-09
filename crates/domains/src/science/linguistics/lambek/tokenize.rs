use super::reduce::TypedToken;
use super::types::LambekType;
use super::types::english as en_types;
use crate::science::linguistics::language::Language;
use crate::science::linguistics::orthography::distance;

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
    // When a copula is followed by an adjective, the adjective gets S[adj]\NP
    // and the copula gets (S[dcl]\NP)/(S[adj]\NP).
    // From Hockenmaier & Steedman (2007), CCGbank.
    assign_predicate_adjectives(&mut tokens);

    tokens
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
            return en_types::question_copula();
        }

        // Interrogative pronouns at sentence start → wh-question type
        if first.is_some_and(|e| e.is_interrogative()) {
            return en_types::wh_what();
        }
    }

    // Copula in non-initial position → copula type (NP complement default)
    if pos.is_some_and(|p| p.is_copula()) && position > 0 {
        return en_types::copula();
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
    en_types::noun()
}

/// Select the best lexical entry when multiple are available.
/// Uses the first entry as default — the language orders entries by priority.
/// For verbs with ambiguous transitivity, the reducer handles both
/// by retrying with alternative types if the first attempt fails.
fn select_best_entry(
    entries: &[crate::science::linguistics::lexicon::pos::LexicalEntry],
) -> Option<&crate::science::linguistics::lexicon::pos::LexicalEntry> {
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
        let is_copula = tokens[i].lambek_type == en_types::copula();
        let is_adj = tokens[i + 1].lambek_type == en_types::adjective();
        if is_copula && is_adj {
            tokens[i].lambek_type = en_types::copula_adj();
            tokens[i + 1].lambek_type = en_types::predicate_adjective();
        }
    }
}

/// Map a lexical entry's POS to its Lambek type.
fn pos_to_lambek(entry: &crate::science::linguistics::lexicon::pos::LexicalEntry) -> LambekType {
    use crate::science::linguistics::lexicon::pos::{LexicalEntry, Transitivity};
    match entry {
        LexicalEntry::Noun(_) => en_types::noun(),
        LexicalEntry::Verb(v) => match v.transitivity {
            Transitivity::Intransitive => en_types::intransitive_verb(),
            Transitivity::Transitive => en_types::transitive_verb(),
            Transitivity::Ditransitive => en_types::ditransitive_verb(),
        },
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
