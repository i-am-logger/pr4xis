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
    #[cfg(feature = "std")]
    let vocab = operators::vocabulary();
    #[cfg(not(feature = "std"))]
    let owned_vocab = operators::load();
    #[cfg(not(feature = "std"))]
    let vocab = &owned_vocab;
    let mut tokens: Vec<TypedToken> = surface_tokens(text, vocab)
        .iter()
        .enumerate()
        .map(|(i, word)| {
            let lower = word.to_lowercase();
            let lambek_type = assign_type(&lower, i, language, vocab);
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
        let chars: Vec<char> = word.chars().collect();
        let mut buf = String::new();
        for (i, &c) in chars.iter().enumerate() {
            // A loaded operator glyph splits out ONLY in a math/standalone
            // context, NEVER when it sits BETWEEN two letters — `-` and `/` are
            // operators, but `well-being` / `and/or` / `x-ray` are single words.
            // So `10+10`, `10-10`, and a standalone `+` split; internal
            // letter-flanked punctuation stays in the word.
            let between_letters = i > 0
                && chars[i - 1].is_alphabetic()
                && i + 1 < chars.len()
                && chars[i + 1].is_alphabetic();
            if vocab.is_operator_glyph(c) && !between_letters {
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

/// Collapse maximal multi-word SURFACE spans into single proper-noun tokens — the
/// multi-token (phrase / citation) recognition the chat needs so a loaded
/// ontology's multi-word surface (a USC citation "section 1514a", an OWL label, a
/// WordNet collocation "ice cream") resolves as ONE lookup unit instead of
/// splitting into tokens that each miss.
///
/// Greedy longest-match over the LOADED surface set (`is_surface`, widest window
/// first, up to `max_surface_words`): a matched span becomes one [`TypedToken`]
/// typed [`proper_noun`](super::types::svo::proper_noun) — the NP (named-entity)
/// category the copula's `/NP` slot consumes, so it flows through the Lambek/CYK
/// parse exactly like a single proper noun ("John"). Recognition is DATA-DRIVEN
/// (the surface set is the reasoner's loaded index), NEVER a baked citation
/// pattern. A `max_surface_words` of 1 (embedded English) makes the window
/// degenerate → a pure no-op, so single-token chat is byte-identical.
///
/// Returns the collapsed tokens AND their per-token Lambek type sets, ready for
/// `chart_reduce` / `interpret`: a collapsed span → `[proper_noun()]`; an
/// uncollapsed token → its own type plus that position's `alternatives` (exactly
/// the set the pipeline built before, so an uncollapsed stream is unchanged).
pub fn collapse_multiword_surfaces(
    tokens: &[TypedToken],
    alternatives: &[Vec<LambekType>],
    max_surface_words: usize,
    is_surface: impl Fn(&str) -> bool,
) -> (Vec<TypedToken>, Vec<Vec<LambekType>>) {
    let max_window = max_surface_words.max(1);
    let mut out_tokens: Vec<TypedToken> = Vec::with_capacity(tokens.len());
    let mut out_types: Vec<Vec<LambekType>> = Vec::with_capacity(tokens.len());
    let mut i = 0;
    while i < tokens.len() {
        // Longest-match first: the widest window (>= 2) whose joined surface is a
        // known loaded surface wins; a single token never collapses.
        let upper = (tokens.len() - i).min(max_window);
        let mut collapsed = false;
        for w in (2..=upper).rev() {
            let joined = tokens[i..i + w]
                .iter()
                .map(|t| t.word.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            if is_surface(&joined) {
                out_tokens.push(TypedToken {
                    word: joined,
                    lambek_type: svo_types::proper_noun(),
                });
                out_types.push(vec![svo_types::proper_noun()]);
                i += w;
                collapsed = true;
                break;
            }
        }
        if !collapsed {
            let token = &tokens[i];
            let mut ts = vec![token.lambek_type.clone()];
            if let Some(alts) = alternatives.get(i) {
                for alt in alts {
                    if !ts.contains(alt) {
                        ts.push(alt.clone());
                    }
                }
            }
            out_tokens.push(token.clone());
            out_types.push(ts);
            i += 1;
        }
    }
    (out_tokens, out_types)
}

/// Tokenize with alternatives — returns tokens AND all possible types for each.
/// Used by the ambiguity-aware reducer to try multiple type assignments.
pub fn tokenize_with_alternatives(
    text: &str,
    language: &dyn Language,
) -> (Vec<TypedToken>, Vec<Vec<LambekType>>) {
    #[cfg(feature = "std")]
    let vocab = operators::vocabulary();
    #[cfg(not(feature = "std"))]
    let owned_vocab = operators::load();
    #[cfg(not(feature = "std"))]
    let vocab = &owned_vocab;
    let words = surface_tokens(text, vocab);

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
        let primary_type = assign_type(&lower, i, language, vocab);

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
    #[cfg(feature = "std")]
    let vocab = operators::vocabulary();
    #[cfg(not(feature = "std"))]
    let owned_vocab = operators::load();
    #[cfg(not(feature = "std"))]
    let vocab = &owned_vocab;
    let mut tokens: Vec<Token> = surface_tokens(text, vocab)
        .iter()
        .enumerate()
        .map(|(i, word)| {
            let lower = word.to_lowercase();
            let lambek_type = assign_type(&lower, i, language, vocab);

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

/// The loaded OLiA key for a lexical entry — its OLiA class fragment plus an
/// optional valency coordinate (verbs). This yields a functor KEY, never a
/// category.
///
/// Content words (from WordNet) carry no OLiA class, so they canonicalize
/// through [`olia::canonical_olia_fragment`] — the irreducible PosTag → OLiA-class
/// bridge (a closed coarse enum mapping to canonical OLiA fragment names). Verbs
/// add their valency (a loaded OLiA `ValencyFeature` class) as the second
/// coordinate — the `operators::derive_lambek(arity, …)` parameter pattern.
fn olia_key(
    entry: &crate::cognitive::linguistics::lexicon::pos::LexicalEntry,
) -> (&'static str, Option<&'static str>) {
    use crate::cognitive::linguistics::lexicon::olia;
    use crate::cognitive::linguistics::lexicon::pos::{LexicalEntry, Transitivity};
    let fragment = olia::canonical_olia_fragment(entry.pos_tag());
    let valency = match entry {
        LexicalEntry::Verb(v) => Some(match v.transitivity {
            // Transitivity (a closed enum) → its OLiA ValencyFeature class.
            Transitivity::Transitive => "Transitive",
            Transitivity::Intransitive => "Intransitive",
            Transitivity::Ditransitive => "Ditransitive",
        }),
        _ => None,
    };
    (fragment, valency)
}

/// A lexical entry's Lambek category — from the loaded OLiA→CCG functor, NOT a
/// Rust `match`. The entry's OLiA key ([`olia_key`]) selects the cited category
/// row; the notation parser lowers it to a [`LambekType`].
///
/// Copula is the one irreducible exception: its category is position-dependent
/// (sentence-initial question vs medial copula vs pre-adjective), resolved by
/// [`assign_type`]; its default medial reading `copula()` is grammar logic, not
/// a class→category map, so it stays here rather than as a functor row.
fn pos_to_lambek(entry: &crate::cognitive::linguistics::lexicon::pos::LexicalEntry) -> LambekType {
    use crate::cognitive::linguistics::lexicon::pos::LexicalEntry;
    if let LexicalEntry::Copula(_) = entry {
        return svo_types::copula();
    }
    let (fragment, valency) = olia_key(entry);
    category_projection::categories_for_class_valency(fragment, valency)
        .into_iter()
        .next()
        .unwrap_or_else(svo_types::noun)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(word: &str) -> TypedToken {
        TypedToken {
            word: word.to_string(),
            lambek_type: svo_types::noun(),
        }
    }

    #[test]
    fn collapses_a_known_multiword_surface_into_one_proper_noun() {
        // "ice cream" is a known surface → its two tokens collapse into ONE
        // proper-noun (NP) token; the surrounding tokens pass through unchanged.
        let tokens = vec![tok("what"), tok("is"), tok("ice"), tok("cream")];
        let alts = vec![vec![], vec![], vec![], vec![]];
        let (out, types) = collapse_multiword_surfaces(&tokens, &alts, 2, |s| s == "ice cream");
        assert_eq!(out.len(), 3, "ice + cream collapse into one token");
        assert_eq!(out[2].word, "ice cream");
        assert_eq!(out[2].lambek_type, svo_types::proper_noun());
        assert_eq!(types[2], vec![svo_types::proper_noun()]);
        assert_eq!(out[0].word, "what", "earlier tokens are untouched");
        assert_eq!(out[1].word, "is");
    }

    #[test]
    fn never_collapses_with_a_degenerate_window_or_no_match() {
        let tokens = vec![tok("ice"), tok("cream")];
        let alts = vec![vec![], vec![]];
        // max_surface_words == 1 → the window is degenerate → a pure no-op, even
        // if EVERY span would match (the embedded single-word-lexicon path).
        let (out, _) = collapse_multiword_surfaces(&tokens, &alts, 1, |_| true);
        assert_eq!(out.len(), 2, "max=1 never collapses");
        // No surface matches → nothing collapses.
        let (out2, _) = collapse_multiword_surfaces(&tokens, &alts, 3, |_| false);
        assert_eq!(out2.len(), 2, "no matching surface → no collapse");
    }

    #[test]
    fn longest_match_wins() {
        // Both "new york" and "new york city" are surfaces — the LONGEST one wins
        // (greedy maximal munch), so a citation is not under-recognized.
        let tokens = vec![tok("new"), tok("york"), tok("city")];
        let alts = vec![vec![], vec![], vec![]];
        let (out, _) = collapse_multiword_surfaces(&tokens, &alts, 3, |s| {
            s == "new york" || s == "new york city"
        });
        assert_eq!(out.len(), 1, "the longest match collapses all three");
        assert_eq!(out[0].word, "new york city");
    }

    #[test]
    fn an_uncollapsed_token_keeps_its_type_and_alternatives() {
        // The no-collapse path must reproduce the pipeline's prior type_sets
        // exactly: the token's own type plus that position's alternatives.
        let tokens = vec![tok("dog")];
        let alts = vec![vec![svo_types::proper_noun()]];
        let (_, types) = collapse_multiword_surfaces(&tokens, &alts, 1, |_| false);
        assert_eq!(types[0], vec![svo_types::noun(), svo_types::proper_noun()]);
    }
}
