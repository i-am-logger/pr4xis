#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::category_projection;
use super::operators::{self, OperatorVocabulary};
use super::reduce::TypedToken;
use super::types::LambekType;
use super::types::svo as svo_types;
use crate::cognitive::linguistics::language::Language;
use crate::cognitive::linguistics::lemon::lexicon::ConceptRef;
use crate::cognitive::linguistics::orthography::distance;
use crate::cognitive::linguistics::text::Token;
use pr4xis::category::entity::Concept;

/// Tokenize text into typed tokens using a language's lexicon.
///
/// This is a functor: Text → TypedTokens, parameterized by Language.
/// The tokenizer is language-agnostic — it calls language.lexical_lookup(),
/// not hardcoded word lists.
///
/// Unknown words go through the noisy channel adjunction:
/// Observation → closest_matches → corrected word's type.
pub fn tokenize(text: &str, language: &dyn Language) -> Vec<TypedToken> {
    let vocab = operators::load();
    let mut tokens: Vec<TypedToken> = surface_tokens(text, &vocab)
        .iter()
        .enumerate()
        .map(|(i, word)| {
            let lower = word.to_lowercase();
            let lambek_type = assign_type(&lower, i, language, &vocab);
            TypedToken {
                word: lower,
                lambek_type,
            }
        })
        .collect();

    // Post-processing: assign predicate adjective types based on context.
    assign_predicate_adjectives(&mut tokens);

    tokens
}

/// Split surface text into tokens, KEEPING loaded math-operator glyphs as their
/// own tokens instead of stripping them as punctuation (#169).
///
/// An operator glyph is recognized from the loaded `math_operators` vocabulary
/// — never a hardcoded `"+-*/=<>"` set — so `"10+10"` splits into
/// `["10", "+", "10"]` and a standalone `"+"` survives, while non-operator
/// trailing punctuation (`"dog?"` → `"dog"`, `"10."` → `"10"`) still trims, so
/// the question path is preserved.
fn surface_tokens(text: &str, vocab: &OperatorVocabulary) -> Vec<String> {
    let mut out = Vec::new();
    for word in text.split_whitespace() {
        let mut buf = String::new();
        for c in word.chars() {
            if vocab.is_operator_glyph(c) {
                flush_word(&mut buf, vocab, &mut out);
                out.push(c.to_string());
            } else {
                buf.push(c);
            }
        }
        flush_word(&mut buf, vocab, &mut out);
    }
    out
}

/// Trim non-operator ASCII punctuation off an accumulated word and push it if
/// non-empty. A loaded operator glyph is never trimmed here — it is emitted as
/// its own token by [`surface_tokens`]; other punctuation (`"dog?"` → `"dog"`)
/// trims exactly as the tokenizer did before #169.
fn flush_word(buf: &mut String, vocab: &OperatorVocabulary, out: &mut Vec<String>) {
    let trimmed =
        buf.trim_matches(|c: char| c.is_ascii_punctuation() && !vocab.is_operator_glyph(c));
    if !trimmed.is_empty() {
        out.push(trimmed.to_string());
    }
    buf.clear();
}

/// Tokenize with alternatives — returns tokens AND all possible types for each.
/// Used by the ambiguity-aware reducer to try multiple type assignments.
pub fn tokenize_with_alternatives(
    text: &str,
    language: &dyn Language,
) -> (Vec<TypedToken>, Vec<Vec<LambekType>>) {
    let vocab = operators::load();
    let words = surface_tokens(text, &vocab);

    // A fronted interrogative ADVERB (where/when/why/how) licenses subject-aux
    // inversion, so the copula can derive S[q]/PP (question_copula_pp) and the
    // wh-adverb category reduces (Huddleston & Pullum 2002 Ch.11).
    let leads_with_wh_adverb = words.first().is_some_and(|w| {
        language
            .lexical_lookup_all(&w.to_lowercase())
            .iter()
            .any(|e| e.olia_class() == Some("InterrogativeAdverb"))
    });

    let mut tokens = Vec::new();
    let mut alternatives = Vec::new();

    for (i, word) in words.iter().enumerate() {
        let lower = word.to_lowercase();

        // Get ALL entries from the language
        let all_entries = language.lexical_lookup_all(&lower);

        // Primary type assignment
        let primary_type = assign_type(&lower, i, language, &vocab);

        // Alternative types from all entries
        let mut alt_types: Vec<LambekType> = all_entries
            .iter()
            .map(pos_to_lambek)
            .filter(|t| *t != primary_type)
            .collect();

        // An operator glyph carries its full set of derived readings (e.g. `-`
        // is both binary subtraction and unary negation); the chart explores
        // all so the one that reduces wins (#169).
        for t in vocab.lambek_types_for(&lower) {
            if t != primary_type && !alt_types.contains(&t) {
                alt_types.push(t);
            }
        }

        // Interrogative words carry their CCG category via the loaded OLiA→CCG
        // functor; the chart explores every reading (e.g. "which" as both
        // pronoun and determiner).
        for entry in &all_entries {
            if let Some(olia) = entry.olia_class() {
                for t in category_projection::categories_for_class(olia) {
                    if t != primary_type && !alt_types.contains(&t) {
                        alt_types.push(t);
                    }
                }
            }
        }

        // Under wh-adverb fronting, a copula/auxiliary also offers the inverted
        // PP-gap reading (S[q]/PP)/NP, so "where is the dog" reduces.
        if leads_with_wh_adverb
            && all_entries
                .iter()
                .any(|e| e.pos_tag().is_copula() || e.pos_tag().is_question_forming())
        {
            let qpp = svo_types::question_copula_pp();
            if qpp != primary_type && !alt_types.contains(&qpp) {
                alt_types.push(qpp);
            }
        }

        tokens.push(TypedToken {
            word: lower,
            lambek_type: primary_type,
        });
        alternatives.push(alt_types);
    }

    assign_predicate_adjectives(&mut tokens);

    (tokens, alternatives)
}

/// Tokenize into ontological Tokens — Word occurrences connected through
/// Lemon (sense), Lambek (grammar type), and OLiA (POS annotation).
///
/// This is the Parse functor's first stage: Surface → typed tokens.
/// Each token carries its lexical sense (which ontology concept the word
/// references) and its POS tag, in addition to the Lambek type.
pub fn tokenize_ontological(text: &str, language: &dyn Language) -> Vec<Token> {
    let vocab = operators::load();
    let mut tokens: Vec<Token> = surface_tokens(text, &vocab)
        .iter()
        .enumerate()
        .map(|(i, word)| {
            let lower = word.to_lowercase();
            let lambek_type = assign_type(&lower, i, language, &vocab);

            let entry = language.lexical_lookup(&lower);
            let pos = entry.as_ref().map(|e| e.pos_tag());
            let sense = pos.map(|p| ConceptRef {
                ontology: "cognitive.linguistics.lexicon".to_string(),
                concept: p.name().to_string(),
            });

            Token {
                word: lower,
                lambek_type,
                sense,
                pos,
            }
        })
        .collect();

    assign_predicate_adjectives_typed(&mut tokens);
    tokens
}

/// Post-processing for ontological tokens (same logic as assign_predicate_adjectives).
fn assign_predicate_adjectives_typed(tokens: &mut [Token]) {
    for i in 0..tokens.len().saturating_sub(1) {
        let is_copula = tokens[i].lambek_type == svo_types::copula();
        let is_adj = tokens[i + 1].lambek_type == svo_types::adjective();
        if is_copula && is_adj {
            tokens[i].lambek_type = svo_types::copula_adj();
            tokens[i + 1].lambek_type = svo_types::predicate_adjective();
        }
    }
}

/// Assign a Lambek type to a word using the language's lexicon.
/// Position-sensitive: copulas/auxiliaries at sentence start get question types.
/// For ambiguous words (e.g. verbs with unknown transitivity), all entries
/// are considered and the best fit for the position is selected.
fn assign_type(
    word: &str,
    position: usize,
    language: &dyn Language,
    vocab: &OperatorVocabulary,
) -> LambekType {
    // A loaded mathematical operator glyph → its DERIVED categorial type
    // (math_operators vocabulary; the type comes from the loaded arity +
    // result-sort, never a per-symbol constant). Checked before the lexicon /
    // spelling-correction path so an operator is never mis-corrected to a
    // content word (#169).
    if let Some(op_type) = vocab.primary_type(word) {
        return op_type;
    }

    // A decimal number literal → a saturated noun phrase (NP), so it composes
    // with an operator's NP argument and a copula's NP complement.
    if operators::is_number_literal(word) {
        return operators::operand_atom();
    }

    // Look up ALL entries — a word can have multiple types
    let entries = language.lexical_lookup_all(word);
    let first = entries.first();
    let pos = first.map(|e| e.pos_tag());

    // Question-forming: sentence-initial copulas/auxiliaries
    if position == 0 {
        if pos.is_some_and(|p| p.is_question_forming()) {
            return svo_types::question_copula();
        }

        // Sentence-initial interrogative → its CCG category from the loaded
        // OLiA→CCG functor (the word's OLiA class projects to a category),
        // never a `wh_what()` constant. The chart explores the other readings.
        if let Some(category) = entries
            .iter()
            .filter_map(|e| e.olia_class())
            .flat_map(category_projection::categories_for_class)
            .next()
        {
            return category;
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
