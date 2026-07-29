#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::category_projection;
use super::operators::{self, OperatorVocabulary};
use super::quote_glyphs::{self, QuoteGlyphVocabulary, QuoteRole};
use super::reduce::{ExpressionUse, TypedToken};
use super::types::LambekType;
use super::types::svo as svo_types;
use crate::cognitive::linguistics::language::Language;
use crate::cognitive::linguistics::lemon::lexicon::ConceptRef;
use crate::cognitive::linguistics::lexicon::pos::{PosTag, WhAdverbRole};
use crate::cognitive::linguistics::orthography::distance;
use crate::cognitive::linguistics::symbols::character;
use crate::cognitive::linguistics::symbols::dash_punctuation::{self, DashPunctuationVocabulary};
use crate::cognitive::linguistics::symbols::punctuation;
use crate::cognitive::linguistics::text::Token;
use crate::formal::math::quantity::value::Quantity;
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
    #[cfg(feature = "std")]
    let quotes = quote_glyphs::vocabulary();
    #[cfg(not(feature = "std"))]
    let owned_quotes = quote_glyphs::load();
    #[cfg(not(feature = "std"))]
    let quotes = &owned_quotes;
    #[cfg(feature = "std")]
    let dashes = dash_punctuation::vocabulary();
    #[cfg(not(feature = "std"))]
    let owned_dashes = dash_punctuation::load();
    #[cfg(not(feature = "std"))]
    let dashes = &owned_dashes;
    let (words0, sentence_initial0, comma_after0, semicolon_after0) =
        surface_tokens_with_sentence_bounds(text, vocab, dashes, language);
    let (words0, sentence_initial0) = collapse_medial_comma_adjuncts(
        words0,
        sentence_initial0,
        comma_after0,
        semicolon_after0,
        language,
    );
    let (words, quoted, sentence_initial) =
        collapse_quoted_spans(words0, sentence_initial0, quotes);
    let (words, quoted, sentence_initial) =
        split_possessive_clitics(words, quoted, sentence_initial, language);
    let words = correct_unknown_word_surfaces(words, &quoted, vocab, language, &|_| false, 1);
    let (words, forcing, sentence_initial) =
        collapse_capitalized_runs(words, quoted, sentence_initial, &|_| false);
    words
        .iter()
        .enumerate()
        .map(|(i, word)| {
            let lower = word.to_lowercase();
            let lambek_type = if forcing[i].forces_np() {
                svo_types::proper_noun()
            } else {
                assign_type(&lower, sentence_initial[i], language, vocab)
            };
            TypedToken {
                word: lower,
                lambek_type,
                expression_use: forcing[i].expression_use(),
            }
        })
        .collect()
}

/// Split surface text into tokens, KEEPING loaded math-operator glyphs as their
/// own tokens instead of stripping them as punctuation (#169).
///
/// An operator glyph is recognized from the loaded `math_operators` vocabulary
/// — never a hardcoded `"+-*/=<>"` set — so `"10+10"` splits into
/// `["10", "+", "10"]` and a standalone `"+"` survives, while non-operator
/// trailing punctuation (`"dog?"` → `"dog"`, `"10."` → `"10"`) still trims, so
/// the question path is preserved.
fn surface_tokens(
    text: &str,
    vocab: &OperatorVocabulary,
    dashes: &DashPunctuationVocabulary,
    language: &dyn Language,
) -> Vec<String> {
    surface_tokens_with_sentence_bounds(text, vocab, dashes, language).0
}

/// Is `c` the loaded comma glyph — [`punctuation::comma`]'s own character —
/// never a bare `','` literal? The same "query the loaded concept, don't
/// hardcode the character" idiom [`is_sentence_terminal_char`] already
/// applies to sentence-ending punctuation.
fn is_comma_char(c: char) -> bool {
    c == punctuation::comma().character
}

/// Is `c` the loaded semicolon glyph — [`punctuation::semicolon`]'s own
/// character, `PunctuationFunction::Connector` (a between-INDEPENDENT-
/// clauses role, `punctuation.rs`) — never a bare `';'` literal? Mirrors
/// [`is_comma_char`]'s own precedent exactly. [`split_into_sentences`]
/// already treats this glyph as a clause boundary for ITS OWN caller
/// (`defines_pointers`); [`SurfaceTokenBounds::semicolon_after`] gives the
/// GENERAL tokenizer (the one live chat uses) the same per-token signal,
/// deliberately WITHOUT making the semicolon a general sentence boundary —
/// `sentence_initial`/`is_sentence_terminal_char` stay period/question/
/// exclamation-only, exactly as `split_into_sentences`'s own doc comment
/// requires for ordinary chat parsing (a semicolon joining two related
/// clauses is NOT a sentence break there). Only a caller that specifically
/// needs "never reach across an independent-clause boundary" — e.g.
/// [`collapse_medial_comma_adjuncts`]'s trailing-whether-adjunct span —
/// reads this flag.
fn is_semicolon_char(c: char) -> bool {
    c == punctuation::semicolon().character
}

/// Is `c` a loaded sentence-ending punctuation mark — [`PunctuationFunction::
/// is_sentence_ending`](crate::cognitive::linguistics::symbols::punctuation::
/// PunctuationFunction::is_sentence_ending) (period/question-mark/exclamation)?
/// Read from the loaded [`punctuation::standard_punctuation`] table, never a
/// bare `matches!(c, '.' | '!' | '?')` — the same "query the loaded concept,
/// don't hardcode the character" idiom [`split_possessive_clitics`] already
/// uses for the apostrophe.
fn is_sentence_terminal_char(c: char) -> bool {
    punctuation::standard_punctuation()
        .into_iter()
        .any(|m| m.character == c && m.is_sentence_ending())
}

/// [`surface_tokens`], additionally reporting which output words are
/// SENTENCE-INITIAL — the very first word of `text`, or any word
/// immediately following one that ended in loaded sentence-ending
/// punctuation (before that punctuation is trimmed off by [`flush_word`],
/// which is why this must be computed HERE rather than re-derived from the
/// already-trimmed output). Needed by [`collapse_capitalized_runs`] to
/// avoid the false-positive risk ordinary sentence-initial capitalization
/// (and a second sentence's own opening word, in a multi-sentence input)
/// would otherwise cause a symbolic proper-noun detector — confirmed a real
/// risk on this corpus (11% of questions are Title-Case STYLED, not
/// entity-marked) before this function was built, not assumed.
/// Per-token boundary metadata [`surface_tokens_with_sentence_bounds`]
/// accumulates ALONGSIDE the raw surface words — one entry per emitted word,
/// aligned by index with `words`. A rich type bundling what would otherwise
/// be FOUR separately-threaded `Vec` out-parameters (`words`,
/// `sentence_initial`, `comma_after`, `semicolon_after`), so [`flush_word_tracked`]
/// stays within a sane arity as [`collapse_medial_comma_adjuncts`] (defines-lens
/// gap G2) adds tracked boundaries alongside the pre-existing sentence-
/// initial one.
#[derive(Debug, Default)]
struct SurfaceTokenBounds {
    words: Vec<String>,
    /// Was this word the first word of its sentence (or of `text` itself)?
    sentence_initial: Vec<bool>,
    /// Was this word immediately followed by the loaded comma glyph in the
    /// RAW source text — the only point in the pipeline where that fact is
    /// still observable, since `flush_word` trims the comma away as
    /// ordinary trailing punctuation before any later stage runs.
    comma_after: Vec<bool>,
    /// Was this word immediately followed by the loaded semicolon glyph
    /// ([`is_semicolon_char`]) in the RAW source text — the SAME "still
    /// observable only here, before `flush_word` trims it" rationale
    /// `comma_after` already establishes, now for the OTHER punctuation
    /// mark [`collapse_medial_comma_adjuncts`]'s trailing-whether-adjunct
    /// branch needs to never scan across (a semicolon is
    /// `PunctuationFunction::Connector`, a between-INDEPENDENT-clauses
    /// role — see [`is_semicolon_char`]'s own doc).
    semicolon_after: Vec<bool>,
}

fn surface_tokens_with_sentence_bounds(
    text: &str,
    vocab: &OperatorVocabulary,
    dashes: &DashPunctuationVocabulary,
    language: &dyn Language,
) -> (Vec<String>, Vec<bool>, Vec<bool>, Vec<bool>) {
    let mut bounds = SurfaceTokenBounds::default();
    let mut next_is_sentence_initial = true;
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
                flush_word_tracked(
                    &mut buf,
                    vocab,
                    dashes,
                    language,
                    &mut bounds,
                    &mut next_is_sentence_initial,
                );
                bounds.words.push(c.to_string());
                bounds.sentence_initial.push(next_is_sentence_initial);
                bounds.comma_after.push(false);
                bounds.semicolon_after.push(false);
                next_is_sentence_initial = false;
            } else {
                buf.push(c);
            }
        }
        flush_word_tracked(
            &mut buf,
            vocab,
            dashes,
            language,
            &mut bounds,
            &mut next_is_sentence_initial,
        );
    }
    (
        bounds.words,
        bounds.sentence_initial,
        bounds.comma_after,
        bounds.semicolon_after,
    )
}

/// Split `text` into its individual sentences, using the SAME sentence-
/// boundary computation `surface_tokens_with_sentence_bounds` already
/// does internally (now abbreviation-aware — a mid-clause citation like
/// "42 U.S.C. § 1395x(r)" never falsely splits) — grouping the already-
/// tokenized words into runs starting at each `sentence_initial[i] ==
/// true`, and rejoining each run with spaces.
///
/// Exists for callers whose input can be a large multi-sentence blob where
/// only ONE sentence is the actual thing they need to parse — `reduce()`
/// has no notion of "the next sentence" (it combines only within the token
/// array it is handed), so a caller that hands a whole multi-definition
/// statutory blob to a whole-span parser pays for the ENTIRE blob's
/// combinatorial chart cost even though a real "the term X means Y"
/// definition is, by grammatical necessity, one declarative sentence
/// bounded by its own sentence-final punctuation. Confirmed directly
/// against real USC provisions during this investigation: candidates like
/// `/us/usc/t15/s80a-2/a` (a 23,342-character "when used in this
/// subchapter... 'Advisory board' means..." preamble bundling dozens of
/// separate definitions) took ~88 CPU-MINUTES as one whole-blob parse
/// attempt and extracted ZERO definitions — not a performance-only issue,
/// a recall issue: the real definitions inside are never reached because
/// the parser is stuck failing to find one derivation spanning the entire
/// blob. Splitting first lets each sentence be judged independently, cheap
/// and fast, recovering exactly the definitions the whole-blob attempt
/// was silently losing.
///
/// A rejoined sentence is a NORMALIZED reconstruction from tokens (space-
/// joined), not a byte-identical substring of `text` — never assumed to
/// preserve original whitespace/formatting, only well-formed enough for a
/// caller (like
/// [`defines_pointers`](crate::social::judicial::statute_structure::grounding::defines_pointers))
/// that re-tokenizes its input from scratch regardless.
///
/// ALSO splits on the loaded semicolon glyph ([`punctuation::semicolon`]),
/// in addition to period/question/exclamation — confirmed necessary by
/// direct measurement, not assumed: a statutory "definitions" preamble
/// ("when used in this subchapter, unless the context otherwise
/// requires— 'X' means A; 'Y' means B; …") is conventionally ONE
/// grammatical run-on sentence terminating in a single final period, with
/// each individual definition separated by semicolons rather than sentence-
/// final punctuation — splitting on period/question/exclamation alone left
/// the worst real Title 15 outliers (`/us/usc/t15/s80a-2/a`,
/// `/us/usc/t15/s78c/a`) as single, still-enormous spans that continued to
/// time out. The semicolon is `PunctuationFunction::Connector`
/// (`punctuation.rs`), a between-independent-clauses role in formal/legal
/// prose that makes each side a plausible standalone declarative — exactly
/// the boundary this extraction needs, deliberately SCOPED to this
/// function alone (the general tokenizer's own `is_sentence_terminal_char`
/// stays period/question/exclamation-only for ordinary chat parsing, where
/// a semicolon joining two related clauses is NOT a sentence break).
/// Splitting the RAW text on semicolons first, then running the existing
/// abbreviation-aware boundary logic on each resulting clause, avoids
/// making the semicolon a SECOND general sentence-boundary character
/// through the shared `flush_word_tracked`/`SurfaceTokenBounds` machinery
/// every OTHER caller (live chat included) also depends on —
/// `sentence_initial` stays exactly as narrow as documented above.
/// `SurfaceTokenBounds::semicolon_after` (added alongside `comma_after`)
/// is a much lighter per-token FLAG, not a boundary: it lets a caller that
/// needs "never scan across an independent-clause connector" —
/// `collapse_medial_comma_adjuncts`'s trailing-whether-adjunct branch —
/// bound its OWN local span without perturbing `sentence_initial`/
/// `is_clause_initial`/question-forming category assignment for anyone
/// else.
pub fn split_into_sentences(
    text: &str,
    vocab: &OperatorVocabulary,
    dashes: &DashPunctuationVocabulary,
    language: &dyn Language,
) -> Vec<String> {
    text.split(punctuation::semicolon().character)
        .flat_map(|clause| {
            // `_semicolon_after` is unused here: this function has already
            // split `text` on the raw semicolon character above, so no
            // per-clause `semicolon_after` entry can ever be `true` within
            // an already-split clause.
            let (words, sentence_initial, comma_after, _semicolon_after) =
                surface_tokens_with_sentence_bounds(clause, vocab, dashes, language);
            let mut sentences: Vec<String> = Vec::new();
            for (i, word) in words.iter().enumerate() {
                if sentence_initial[i] || sentences.is_empty() {
                    sentences.push(word.clone());
                } else {
                    let last = sentences.last_mut().expect("just ensured non-empty");
                    last.push(' ');
                    last.push_str(word);
                }
                // Reinsert the loaded comma glyph immediately after this
                // word when the RAW source had one here — dropping it
                // would silently erase the medial-adjunct comma structure
                // `collapse_medial_comma_adjuncts` (defines-lens gap G2)
                // depends on to recognize "as used in X,"/"with respect
                // to Y," set-off clauses; confirmed a real regression via
                // 3 existing `grounding.rs` tests that lost their
                // extracted pointer entirely once commas were dropped here.
                if comma_after[i] {
                    sentences
                        .last_mut()
                        .expect("just pushed to sentences above")
                        .push(punctuation::comma().character);
                }
            }
            sentences
        })
        .collect()
}

/// [`flush_word`], additionally keeping `bounds.sentence_initial`/
/// `bounds.comma_after` aligned with `bounds.words` (one entry per emitted
/// word, none for an empty flush) and updating `next_is_sentence_initial`
/// from whether `buf`'s LAST non-whitespace character — before trimming
/// removes it — was loaded sentence-ending punctuation.
fn flush_word_tracked(
    buf: &mut String,
    vocab: &OperatorVocabulary,
    dashes: &DashPunctuationVocabulary,
    language: &dyn Language,
    bounds: &mut SurfaceTokenBounds,
    next_is_sentence_initial: &mut bool,
) {
    if buf.is_empty() {
        return;
    }
    // A lexicon-known abbreviation ("U.S.C.", "Pub. L.", "Stat.", "No.", …)
    // ending in a period is never a genuine sentence boundary, even though
    // its last character IS loaded sentence-terminal punctuation — the SAME
    // check `flush_word` already applies to decide whether the trailing
    // period survives in the token TEXT, now also gating this token-
    // boundary signal. Without this, a mid-clause citation like "42 U.S.C.
    // § 1395x(r)" — common in exactly the USC prose `defines_pointers`
    // parses — reads as a genuine sentence break at "U.S.C.": harmless while
    // `sentence_initial` only fed a soft proper-noun disambiguation heuristic
    // (`collapse_capitalized_runs`), but not once it drives `is_clause_initial`
    // into `assign_type`'s question-copula branch (`is_question_forming` =
    // Copula/Auxiliary) for the word immediately after — exactly the
    // "shall"/"is"/"does" that commonly follows a citation in statutory
    // prose, minting a spurious extra interrogative-category chart
    // alternative at that position on every such occurrence.
    let ends_sentence = buf
        .chars()
        .next_back()
        .is_some_and(is_sentence_terminal_char)
        && !trailing_period_is_abbreviation(buf, vocab, dashes, language);
    // Whether the ORIGINAL buffer (before `flush_word` trims the comma off
    // as ordinary trailing punctuation) ended in the loaded comma glyph —
    // see `SurfaceTokenBounds::comma_after`'s own doc.
    let ends_with_comma = buf.chars().next_back().is_some_and(is_comma_char);
    // Same, for the loaded semicolon glyph — see `SurfaceTokenBounds::
    // semicolon_after`'s own doc.
    let ends_with_semicolon = buf.chars().next_back().is_some_and(is_semicolon_char);
    let was_sentence_initial = *next_is_sentence_initial;
    let before = bounds.words.len();
    flush_word(buf, vocab, dashes, language, &mut bounds.words);
    // `flush_word` can now emit MORE THAN ONE word per flush
    // ([`split_unresolved_slash_compound`] — an unresolved letter-flanked
    // slash compound splits into its halves), so `bounds.sentence_initial`/
    // `bounds.comma_after` must gain one entry per emitted word, not just
    // one per flush, to stay aligned 1:1 with `bounds.words`. Only the FIRST
    // piece can genuinely be sentence-initial (it occupies the original
    // token's own position); every later piece is a mid-token continuation.
    // `comma_after`/`semicolon_after` (whether the ORIGINAL buffer ended in
    // that glyph) apply only to the LAST piece — the glyph trailed the
    // whole buffer, not an earlier split-off half.
    for (offset, _) in (before..bounds.words.len()).enumerate() {
        let is_first = offset == 0;
        let is_last = before + offset + 1 == bounds.words.len();
        bounds
            .sentence_initial
            .push(is_first && was_sentence_initial);
        bounds.comma_after.push(is_last && ends_with_comma);
        bounds.semicolon_after.push(is_last && ends_with_semicolon);
    }
    *next_is_sentence_initial = ends_sentence;
}

/// Trim non-operator ASCII punctuation AND non-operator loaded Unicode
/// dash-punctuation glyphs off an accumulated word and push it if non-empty.
/// A loaded operator glyph is never trimmed here — it is emitted as its own
/// token by [`surface_tokens`]; other punctuation (`"dog?"` → `"dog"`) trims
/// exactly as the tokenizer did before #169.
///
/// The Unicode dash coverage (`dashes`, [`DashPunctuationVocabulary`] —
/// every General_Category=Pd code point, 25 members) is ADDITIVE to the
/// pre-existing ASCII check, never a replacement: `char::is_ascii_punctuation`
/// is TRUE only for the ASCII subset (including the ASCII hyphen-minus,
/// already trimmed before this fix), and FALSE for every non-ASCII dash — so
/// real U.S. Code legal prose carrying an EM DASH after a defining verb
/// (`the term "developmental disability" means—`, a common USC drafting
/// convention introducing an enumerated list) left the malformed token
/// `means—` untrimmed, which then missed lexicon lookup for the bare word
/// `means`. Querying the loaded [`DashPunctuationVocabulary`] instead of a
/// second hardcoded char check closes that gap for the whole Pd set, not
/// just the one glyph the bug report named.
///
/// A single ABBREVIATION-DEFINING trailing period is the one exception: if
/// restoring exactly one trailing period onto the maximally-trimmed core
/// reconstructs a PREFIX of the original buffer (so any further trailing
/// punctuation — a sentence-final `?`, a closing quote artifact — is still
/// stripped) AND the lexicon actually holds that period-bearing form
/// (`language.is_known_surface`; WordNet spells `"O.K."` / `"Ph.D."` with the
/// period as part of the lemma, not as sentence punctuation), the period
/// survives. Every other case (`"dog?"`, `"10."`, an unresolvable
/// `"X.Y.Z."`) is unaffected — the lexicon gate means this can only ever
/// WIDEN, never narrow, what already resolves.
/// Is `c` specifically the loaded DIVISION operator glyph (`/`) — identified
/// by its own OpenMath symbol identity (`arith1#divide`), never a bare `'/'`
/// literal — as opposed to the OTHER letter-flanked operator glyph this
/// grammar also loads, subtraction/hyphen-minus (`arith1#minus`/
/// `arith1#unary_minus`, `-`). The two need to stay distinguishable: see
/// [`split_unresolved_slash_compound`]'s own doc for why only the division
/// glyph gets the unknown-compound split, never the hyphen.
fn is_division_glyph(c: char, vocab: &OperatorVocabulary) -> bool {
    vocab
        .operators_for(c)
        .iter()
        .any(|op| op.openmath_symbol.ends_with("divide"))
}

/// A letter-flanked DIVISION glyph inside `word` ("and/or", "Support/
/// Navigation") is kept glued into ONE surface upstream
/// ([`surface_tokens_with_sentence_bounds`]'s own `between_letters` guard —
/// `-` and `/` are loaded operators, but "well-being"/"and/or"/"x-ray" are
/// single words) so a genuinely lexicalized slash compound ("and/or", a real
/// WordNet closed-class lemma) tokenizes as the one lexical unit it is. Not
/// every letter-flanked slash is lexicalized, though: "Support/Navigation"
/// (a program's own alternate-name notation, "Support" OR "Navigation") is
/// TWO separate proper nouns joined by an "or"-slash — gluing it into one
/// token makes it permanently unresolvable (a confirmed real regression:
/// "What is Community Direct Support/Navigation?" fused into the single
/// unresolvable pseudo-token "support/navigation", never reaching either
/// half's own lexicon entry). The lexicon is the discriminator — mirroring
/// this SAME function's own trailing-abbreviation-period exception just
/// above (`with_period`): if the WHOLE glued surface (case-folded) is a
/// known surface, it survives fused, unchanged from today; otherwise it
/// splits on every division glyph it contains, each half re-emitted as its
/// own word. Scoped to division ONLY (never the hyphen/subtraction glyph
/// `between_letters` also protects) — the corpus regression this closes is
/// specifically slash-joined ALTERNATE NAMES, a distinct English
/// orthographic convention (Chicago Manual of Style §6.106, the "shilling
/// slash" marking alternatives) from hyphenated compounding (Huddleston &
/// Pullum 2002 Ch.19 §3, a genuine word-formation process where an
/// unresolvable hyphenated OOV compound is already handled honestly
/// elsewhere in this pipeline, e.g. Gap A's N/N compounding). Widens
/// resolution only — a genuinely lexicalized slash compound is untouched,
/// and a word with no internal division glyph is untouched.
fn split_unresolved_slash_compound(
    word: &str,
    vocab: &OperatorVocabulary,
    language: &dyn Language,
) -> Option<Vec<String>> {
    if !has_letter_flanked_division(word, vocab) || language.is_known_surface(&word.to_lowercase())
    {
        return None;
    }
    let pieces = division_glyph_pieces(word, vocab);
    if pieces.len() < 2 {
        return None;
    }
    // Never split when EITHER side is a CLOSED-CLASS function word —
    // "and/or", "his/her", "he/she" are established closed grammatical
    // formations (Huddleston & Pullum 2002 Ch.19 §3's abbreviatory/
    // coordinative word-formation account), fused REGARDLESS of whether the
    // whole glued form happens to be its own WordNet headword — a confirmed
    // real regression this guard fixes: `is_known_surface("and/or")` is
    // FALSE (it is not itself a WordNet lemma), so the lexicon check ALONE
    // still split it, breaking the pre-existing `hyphenated_and_slashed_
    // words_are_not_split_by_operator_glyphs` invariant. `Support/
    // Navigation`'s two content proper nouns are neither closed-class, so
    // this guard does not protect them — they still split.
    if pieces.iter().any(|p| {
        language
            .lexical_lookup(&p.to_lowercase())
            .is_some_and(|entry| !entry.pos_tag().is_content())
    }) {
        return None;
    }
    Some(pieces)
}

/// Does `word` contain a DIVISION glyph sitting directly between two letters
/// — the shape [`split_unresolved_slash_compound`]/
/// [`split_letter_flanked_division`] both split on?
fn has_letter_flanked_division(word: &str, vocab: &OperatorVocabulary) -> bool {
    let chars: Vec<char> = word.chars().collect();
    chars.iter().enumerate().any(|(i, &c)| {
        i > 0
            && i + 1 < chars.len()
            && chars[i - 1].is_alphabetic()
            && chars[i + 1].is_alphabetic()
            && is_division_glyph(c, vocab)
    })
}

/// `word` split on every DIVISION glyph it contains, empty pieces dropped —
/// the raw splitting kernel [`split_unresolved_slash_compound`]/
/// [`split_letter_flanked_division`] both build on.
fn division_glyph_pieces(word: &str, vocab: &OperatorVocabulary) -> Vec<String> {
    word.split(|c: char| is_division_glyph(c, vocab))
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// Split `word` on every letter-flanked DIVISION glyph it contains — the
/// SAME word-boundary discipline `split_unresolved_slash_compound`
/// applies during tokenization (see its own doc for the full "and/or" vs.
/// "Support/Navigation" rationale), exposed here UNCONDITIONALLY (no
/// `language.is_known_surface` lexicon gate) for a caller that scans RAW,
/// never-tokenized input text — [`crate`]'s own chat pipeline's probable-
/// acronym decline gate, `pr4xis_chat::decline_if_an_unresolved_acronym_
/// was_ignored`. That gate's [`is_probable_acronym`] counts uppercase
/// letters over whatever string it is given, with no notion of word
/// boundaries inside it: "Support/Navigation" (two ordinary Title-Case
/// words, each with exactly ONE capital letter) has TWO capitals when
/// counted as one glued whitespace-chunk, a false-positive "probable
/// acronym" — a confirmed real regression ("What is Community Direct
/// Support/Navigation?" declined as if "Support/Navigation" were an
/// unresolved acronym like "RN"/"IHSS"). Splitting first and checking each
/// piece independently is a strict refinement: a genuine acronym pair
/// joined by a slash ("TANF/SNAP") still has each half individually
/// qualify, and an ordinary word with no internal division glyph returns as
/// its own single-element vector, unchanged.
pub fn split_letter_flanked_division(word: &str) -> Vec<String> {
    #[cfg(feature = "std")]
    let vocab = operators::vocabulary();
    #[cfg(not(feature = "std"))]
    let owned_vocab = operators::load();
    #[cfg(not(feature = "std"))]
    let vocab = &owned_vocab;
    if !has_letter_flanked_division(word, vocab) {
        return alloc::vec![word.to_string()];
    }
    division_glyph_pieces(word, vocab)
}

/// Whether `buf`'s trimmed content, immediately followed by a period, is
/// itself a lexicon-known surface ("U.S.C.", "Pub. L.", "Stat.", "No.", …) —
/// the shared abbreviation check [`flush_word`] uses to decide whether a
/// trailing period survives in the emitted token text, and
/// [`flush_word_tracked`] additionally uses to keep a genuine mid-clause
/// abbreviation from being misread as a sentence boundary.
fn trailing_period_is_abbreviation(
    buf: &str,
    vocab: &OperatorVocabulary,
    dashes: &DashPunctuationVocabulary,
    language: &dyn Language,
) -> bool {
    let trimmed = buf.trim_matches(|c: char| {
        (c.is_ascii_punctuation() || dashes.is_dash_glyph(c)) && !vocab.is_operator_glyph(c)
    });
    let with_period = format!("{trimmed}.");
    buf.starts_with(with_period.as_str()) && language.is_known_surface(&with_period)
}

fn flush_word(
    buf: &mut String,
    vocab: &OperatorVocabulary,
    dashes: &DashPunctuationVocabulary,
    language: &dyn Language,
    out: &mut Vec<String>,
) {
    let trimmed = buf.trim_matches(|c: char| {
        (c.is_ascii_punctuation() || dashes.is_dash_glyph(c)) && !vocab.is_operator_glyph(c)
    });
    let word = if trailing_period_is_abbreviation(buf, vocab, dashes, language) {
        format!("{trimmed}.")
    } else {
        trimmed.to_string()
    };
    if word.is_empty() {
        buf.clear();
        return;
    }
    match split_unresolved_slash_compound(&word, vocab, language) {
        Some(pieces) => out.extend(pieces),
        None => out.push(word),
    }
    buf.clear();
}

/// Collapse a quoted span — a loaded Unicode Pi glyph through its matching Pf
/// closer — into ONE surface token, quote glyphs stripped, so the caller can
/// force the NP reading a MENTIONED (not used) expression takes instead of
/// running the word through its normal lexicon-driven typing (Slice B's
/// quoted-mention NP, `.notes/chat-fix-c-build-state.md`: "what does
/// {adverb} mean"; Quine 1940:26 via SEP "Quotation" §3.1, cited in
/// [`quote_glyphs`]).
///
/// Scoped to the DIRECTIONAL glyphs (`QuoteRole::Initial`/`Final`) only — the
/// two ASCII marks are `QuoteRole::Ambiguous` (the UCD itself does not encode
/// their directionality) and are left untouched here, to `flush_word`'s
/// existing punctuation trim. Greedy nearest-close: the first legitimately
/// closing glyph found scanning forward from an opener wins (no nested-quote
/// handling). An opener with no legitimate closer among the remaining tokens
/// is left as-is — malformed/unclosed input falls through to ordinary
/// tokenization rather than collapsing incorrectly.
///
/// Returns the (possibly shorter) token list, a parallel `is_quoted` flag per
/// output position (so the caller can gate typing without the merged surface
/// itself carrying any marker), and `sentence_initial` carried through the
/// merge (a collapsed span inherits its FIRST word's flag; an untouched word
/// keeps its own) — needed downstream by [`collapse_capitalized_runs`].
pub(crate) fn collapse_quoted_spans(
    words: Vec<String>,
    sentence_initial: Vec<bool>,
    quotes: &QuoteGlyphVocabulary,
) -> (Vec<String>, Vec<bool>, Vec<bool>) {
    let mut out_words = Vec::with_capacity(words.len());
    let mut out_quoted = Vec::with_capacity(words.len());
    let mut out_sentence_initial = Vec::with_capacity(words.len());
    let mut i = 0;
    while i < words.len() {
        let opener = words[i]
            .chars()
            .next()
            .filter(|&c| quotes.get(c).is_some_and(|g| g.role == QuoteRole::Initial));
        if let Some(open_char) = opener {
            let closer = (i..words.len()).find(|&j| {
                words[j]
                    .chars()
                    .last()
                    .is_some_and(|c| quotes.closes(open_char, c))
            });
            if let Some(end) = closer {
                let joined = words[i..=end].join(" ");
                let mut chars: Vec<char> = joined.chars().collect();
                chars.remove(0);
                chars.pop();
                let bare: String = chars.into_iter().collect::<String>().trim().to_string();
                if !bare.is_empty() {
                    out_words.push(bare);
                    out_quoted.push(true);
                    out_sentence_initial.push(sentence_initial[i]);
                    i = end + 1;
                    continue;
                }
            }
        }
        out_words.push(words[i].clone());
        out_quoted.push(false);
        out_sentence_initial.push(sentence_initial[i]);
        i += 1;
    }
    (out_words, out_quoted, out_sentence_initial)
}

/// The reserved synthetic surface minted for the merged interior of a
/// SUBJECT-VERB-position medial comma supplement (`svo_types::
/// medial_supplement_np`, defines-lens gap G2) — "the term 'X', used with
/// respect to Y, means ..." / "the term 'X', as used in this title, means
/// ...". U+2063 INVISIBLE SEPARATOR (Unicode Standard, "Format Characters",
/// General_Category=Cf) brackets it so it can never collide with a real
/// written surface a user or a statute could type — the same sentinel-value
/// idiom [`montague::Sem::unresolved`](super::montague::Sem::unresolved)
/// already establishes for its own `"?"` marker.
pub(crate) const MEDIAL_SUPPLEMENT_NP_MARKER: &str = "\u{2063}medial-supplement-np\u{2063}";

/// The reserved synthetic surface minted for the merged interior of a
/// POST-VERB-position medial comma supplement (`svo_types::
/// medial_supplement_verb`, defines-lens gap G2) — "means, with respect to
/// Y, Z" (the EVV headline shape). Same rationale as
/// [`MEDIAL_SUPPLEMENT_NP_MARKER`].
pub(crate) const MEDIAL_SUPPLEMENT_VERB_MARKER: &str = "\u{2063}medial-supplement-verb\u{2063}";

/// Is `word` one of the two reserved medial-supplement markers
/// [`collapse_medial_comma_adjuncts`] mints? `montague::apply`'s NP-result
/// and function-result branches import this (rather than re-deriving their
/// own check) so the tokenizer's minting and the semantics' dispatch can
/// never drift apart — the exact precedent
/// [`is_fronted_scope_adjunct_head`] already establishes for G1.
pub(crate) fn is_medial_supplement_marker(word: &str) -> bool {
    word == MEDIAL_SUPPLEMENT_NP_MARKER || word == MEDIAL_SUPPLEMENT_VERB_MARKER
}

// ---- N-ary list-comma coordination (defines-lens gap G4(a)) ----
//
// English coordinates 3+ items with a comma standing in for the repeated
// conjunction at every link but the last ("bodily injury, impairment, or
// disease" — 42 U.S.C. § 3002(42); "an unpaid family member, a foster
// parent, or another unpaid individual" — 42 U.S.C. § 300ii(5)):
// Huddleston & Pullum (2002), *The Cambridge Grammar of the English
// Language*, Cambridge University Press, Ch. 15 "Coordination and
// Supplementation" §3 "Coordination of three or more elements" — the comma
// realizes the SAME coordinating relation the final overt "and"/"or" names,
// not a separate construction. This is the categorial-grammar analogue of
// [`svo_types::nominal_coordinator_np`]/[`nominal_coordinator_n`]: rather
// than a hardcoded n-ary rule, EACH qualifying comma is minted as its own
// instance of the identical binary coordinator category, so a 3-, 4-, or
// N-item list reduces by iterated binary application — the same "iterated
// binary combinator" treatment Steedman (2000), *The Syntactic Process*,
// MIT Press, Ch. 4 gives repeated coordination generally.

/// The reserved synthetic surface minted for a comma that stands in for
/// "and" in an n-ary list ([`find_list_coordinator_commas`]) — U+2063
/// INVISIBLE SEPARATOR-bracketed, the same sentinel idiom
/// [`MEDIAL_SUPPLEMENT_NP_MARKER`] already establishes.
pub(crate) const LIST_COORDINATOR_MARKER_AND: &str = "\u{2063}list-coordinator-and\u{2063}";

/// The "or" counterpart of [`LIST_COORDINATOR_MARKER_AND`].
pub(crate) const LIST_COORDINATOR_MARKER_OR: &str = "\u{2063}list-coordinator-or\u{2063}";

/// Is `word` one of the two reserved list-coordinator markers
/// [`collapse_medial_comma_adjuncts`] mints? Mirrors
/// [`is_medial_supplement_marker`]'s own precedent exactly.
pub(crate) fn is_list_coordinator_marker(word: &str) -> bool {
    word == LIST_COORDINATOR_MARKER_AND || word == LIST_COORDINATOR_MARKER_OR
}

/// The CANONICAL coordinating conjunction ("and"/"or") `word` realizes —
/// whether `word` is the literal surface itself (see [`is_nominal_coordinator`])
/// or one of [`LIST_COORDINATOR_MARKER_AND`]/[`LIST_COORDINATOR_MARKER_OR`].
/// `montague::apply`'s coordination-flattening branches import this single
/// check (rather than re-deriving their own) so the tokenizer's minting and
/// the semantics' dispatch can never drift apart — the exact precedent
/// [`is_fronted_scope_adjunct_head`] already establishes for G1.
pub(crate) fn nominal_coordinator_canonical(word: &str) -> Option<&'static str> {
    match word {
        "and" => Some("and"),
        "or" => Some("or"),
        w if w == LIST_COORDINATOR_MARKER_AND => Some("and"),
        w if w == LIST_COORDINATOR_MARKER_OR => Some("or"),
        _ => None,
    }
}

// ---- Coordinated close-apposition definienda (defines-lens gap G5) ----
//
// "The terms 'exploitation' and 'financial exploitation' mean ..." (42
// U.S.C. § 3002(18)(A)) / "The term 'fiber' or 'textile fiber' means ..."
// (15 U.S.C. § 70(b)): a coordinated PAIR (or list) of quoted definienda
// sharing one definiens. Both [`nominal_coordinator_apposition`]
// (`svo_types`) and ORDINARY `and`/`or` reduce to the IDENTICAL derived
// `NP\NP` shape once saturated (see that category's own doc), so a
// dedicated reserved marker — never the literal surface — is what lets
// `montague::apply` tell "and"/"or" coordinating two ALREADY-CLOSE-
// APPOSITION-TYPED quoted spans apart from ordinary NP/N-level
// (definiens-side) coordination, the SAME "type shape AND lexical
// surface/marker" double-guard discipline every other `NP\NP`-colliding
// construction in this module already follows.

/// The reserved synthetic surface minted for "and" coordinating two
/// close-apposition-typed quoted spans directly (defines-lens gap G5) —
/// U+2063 INVISIBLE SEPARATOR-bracketed, the same sentinel idiom
/// [`MEDIAL_SUPPLEMENT_NP_MARKER`] already establishes.
pub(crate) const APPOSITION_COORDINATOR_MARKER_AND: &str =
    "\u{2063}apposition-coordinator-and\u{2063}";

/// The "or" counterpart of [`APPOSITION_COORDINATOR_MARKER_AND`].
pub(crate) const APPOSITION_COORDINATOR_MARKER_OR: &str =
    "\u{2063}apposition-coordinator-or\u{2063}";

/// Is `word` one of the two reserved apposition-coordinator markers
/// [`mark_apposition_coordinators`] mints? Mirrors
/// [`is_list_coordinator_marker`]'s own precedent exactly.
pub(crate) fn is_apposition_coordinator_marker(word: &str) -> bool {
    word == APPOSITION_COORDINATOR_MARKER_AND || word == APPOSITION_COORDINATOR_MARKER_OR
}

/// The CANONICAL coordinating conjunction ("and"/"or") an apposition-
/// coordinator marker realizes — mirrors [`nominal_coordinator_canonical`]'s
/// own precedent exactly, but for THIS module's dedicated marker set only
/// (deliberately DISJOINT from [`nominal_coordinator_canonical`]'s own set,
/// which never recognizes these two markers — the disambiguation this
/// whole mechanism exists for).
pub(crate) fn apposition_coordinator_canonical(word: &str) -> Option<&'static str> {
    match word {
        w if w == APPOSITION_COORDINATOR_MARKER_AND => Some("and"),
        w if w == APPOSITION_COORDINATOR_MARKER_OR => Some("or"),
        _ => None,
    }
}

/// If `words[i]` opens a "the term"/"the terms" repetition (case-
/// insensitive), return the index just past it; otherwise return `i`
/// unchanged. The closed 2-word Dictionary-Act boilerplate — the SAME
/// REGISTER-SPECIFIC closed-vocabulary precedent
/// `grounding::is_partial_definition_verb` (its own doc has the full
/// citation) and [`is_medial_supplement_interior_head`] already
/// establish, applied here to a determiner+noun pair instead of a verb —
/// used ONLY by [`mark_apposition_coordinators`] to bridge a coordinated
/// FULL-NP definiendum repetition ("the term 'X' and THE TERM 'Y' mean
/// ...", 42 U.S.C. § 1395x(aa)(5)(A)).
fn skip_the_term_prefix(words: &[String], i: usize) -> usize {
    if words.get(i).map(|w| w.to_lowercase().as_str() == "the") != Some(true) {
        return i;
    }
    match words.get(i + 1).map(|w| w.to_lowercase()).as_deref() {
        Some("term") | Some("terms") => i + 2,
        _ => i,
    }
}

/// Replace a literal "and"/"or" with its reserved apposition-coordinator
/// marker wherever it coordinates two quoted definienda — EITHER
/// DIRECTLY (`quoted[i-1] && quoted[i+1]`, "the terms 'X' and 'Y' mean
/// ..." / "the term 'X' or 'Y' means ...") OR bridging a repeated "the
/// term(s)" prefix on the right (`quoted[i-1]` and
/// [`skip_the_term_prefix`] reaches a quoted span, "the term 'X' and THE
/// TERM 'Y' mean ...", 42 U.S.C. § 1395x(aa)(5)(A)) — in which case the
/// bridged "the"/"term(s)" tokens are ELIDED so the marker and the second
/// quoted span sit directly adjacent, the shape
/// `svo_types::nominal_coordinator_apposition` expects on its right.
///
/// The direct case is a 1:1 surface substitution; the bridging case
/// removes two tokens, so `quoted`/`sentence_initial` are rebuilt in
/// lockstep here rather than left for the caller to realign.
///
/// Precise by construction: a REAL statutory coordinated-definiens list
/// ("a grant, contract, or cooperative agreement") never has "and"/"or"
/// with a quoted span (or a "the term(s)"-prefixed one) on BOTH sides —
/// quoting in USLM prose is reserved for the definiendum itself (Bluebook
/// §8 quotation convention), never a bare definiens conjunct — so this
/// gate does not need the verb-adjacency precision guards
/// [`is_transitive_verb_leaf`]/[`is_determiner`] earn for
/// [`collapse_medial_comma_adjuncts`]'s own comma-bracket gates. Scoped to
/// exactly ONE coordinated pair (not an n-ary list): no real report
/// example needs more, and `grounding::defines_pointers`'s own
/// `definiendum_words` doc documents this as the deliberate G5 scope
/// boundary.
fn mark_apposition_coordinators(
    words: Vec<String>,
    quoted: Vec<bool>,
    sentence_initial: Vec<bool>,
) -> (Vec<String>, Vec<bool>, Vec<bool>) {
    let mut out_words = Vec::with_capacity(words.len());
    let mut out_quoted = Vec::with_capacity(words.len());
    let mut out_sentence_initial = Vec::with_capacity(words.len());
    let mut i = 0;
    while i < words.len() {
        let is_and_or = matches!(words[i].to_lowercase().as_str(), "and" | "or");
        let left_quoted = i.checked_sub(1).and_then(|j| quoted.get(j)).copied() == Some(true);
        if is_and_or && left_quoted {
            let marker = if words[i].eq_ignore_ascii_case("and") {
                APPOSITION_COORDINATOR_MARKER_AND
            } else {
                APPOSITION_COORDINATOR_MARKER_OR
            };
            if quoted.get(i + 1).copied() == Some(true) {
                // Direct: "... 'X' and/or 'Y' ..." — 1:1 substitution.
                out_words.push(marker.to_string());
                out_quoted.push(quoted[i]);
                out_sentence_initial.push(sentence_initial[i]);
                i += 1;
                continue;
            }
            let after_prefix = skip_the_term_prefix(&words, i + 1);
            if after_prefix > i + 1 && quoted.get(after_prefix).copied() == Some(true) {
                // Bridging: "... 'X' and/or the term(s) 'Y' ..." — elide
                // the "the"/"term(s)" tokens between the marker and 'Y'.
                out_words.push(marker.to_string());
                out_quoted.push(quoted[i]);
                out_sentence_initial.push(sentence_initial[i]);
                i = after_prefix;
                continue;
            }
        }
        out_words.push(words[i].clone());
        out_quoted.push(quoted[i]);
        out_sentence_initial.push(sentence_initial[i]);
        i += 1;
    }
    (out_words, out_quoted, out_sentence_initial)
}

/// Find every comma that functions as an implicit list coordinator —
/// [`nominal_coordinator_canonical`]'s own doc block above has the citation.
///
/// For each occurrence of a literal "and"/"or" at position `p`, walk
/// BACKWARD from `p - 1` collecting every comma boundary of a CONTIGUOUS
/// chain of comma-delimited conjuncts leading up to it: if `p - 1` itself is
/// NOT comma-marked, this is a plain two-item "X and Y" and nothing further
/// is done (the existing binary category already covers it). Otherwise —
/// the Oxford-comma-before-the-coordinator case — repeatedly search
/// backward for the NEAREST earlier comma boundary; each one found is a
/// real list-coordinator comma (it separates two conjuncts of the SAME
/// list), marked with the pivot's own conjunction; the walk continues from
/// there until no earlier comma boundary remains. This naturally stops at
/// the list's true start and never crosses into an unrelated clause: a
/// comma that opens a DIFFERENT construction (a relative clause, another
/// nested list with its own, closer coordinator) is only ever reached by
/// walking back from ITS OWN pivot, never this one's — verified by hand
/// against both real report-cited examples this module's own tests carry
/// (the outer 3-item family-caregiver list and the doubly-nested "in-home
/// monitoring, management, supervision, or treatment" list inside its own
/// relative clause do not bleed into each other) — with the help of
/// [`is_clause_boundary_word`]'s guard, below.
///
/// The Oxford comma immediately preceding the pivot itself (`p - 1`) is
/// deliberately NOT marked — the literal "and"/"or" already provides that
/// boundary's coordination, so marking it too would only ever be a dead
/// (never-reducing) chart alternative.
///
/// The backward search for the nearest earlier comma boundary is NOT
/// unbounded: it stops (ending the whole chain right there, marking nothing
/// further) the moment it would have to cross a genuine CLAUSE-boundary word
/// ([`is_clause_boundary_word`]) before reaching a comma. This is the
/// load-bearing precision guard: without it, a distant, unrelated comma
/// (one that ends a DIFFERENT, earlier list entirely, on the far side of an
/// intervening relative clause) can be the nearest earlier `true` in
/// `comma_after` by pure coincidence — exactly the false positive a real
/// byte-verified sentence surfaced during development: 42 U.S.C. §
/// 300ii(5)'s "...another unpaid individual, who provides in-home
/// monitoring, management, supervision, or treatment ...". Walking backward
/// from the INNER list's own "or" pivot (before "treatment") must stop at
/// "monitoring" (no comma before "in-home") — without this guard it instead
/// crossed straight through "who provides" and wrongly reached the OUTER
/// list's "individual," comma.
pub(crate) fn find_list_coordinator_commas(
    words: &[String],
    comma_after: &[bool],
    language: &dyn Language,
) -> alloc::collections::BTreeMap<usize, &'static str> {
    let mut marks = alloc::collections::BTreeMap::new();
    for (p, word) in words.iter().enumerate() {
        let Some(canonical) = nominal_coordinator_canonical(word) else {
            continue;
        };
        let Some(mut search_from) = p.checked_sub(1) else {
            continue;
        };
        if !comma_after.get(search_from).copied().unwrap_or(false) {
            continue; // "X and Y" -- no comma chain, the plain binary category suffices
        }
        'chain: while search_from > 0 {
            for k in (0..search_from).rev() {
                if comma_after[k] {
                    marks.insert(k, canonical);
                    search_from = k;
                    continue 'chain;
                }
                if is_clause_boundary_word(&words[k], language) {
                    break 'chain;
                }
            }
            break;
        }
    }
    marks
}

/// Is `word` a genuine CLAUSE-boundary token — a RELATIVE PRONOUN (opening a
/// postnominal relative clause, the SAME closed
/// `data/function-words/english.xml` `RelativePronoun` class
/// [`RelativePronoun`](super::category_projection)'s own loaded category
/// already keys on)? [`find_list_coordinator_commas`]'s own precision guard,
/// above.
///
/// Deliberately scoped to JUST this one closed function-word class, NOT
/// "any word with a verb reading among its lexical entries": a first
/// attempt checking `PosTag::Verb`/`Copula`/`Auxiliary` too was a REAL,
/// measured false positive against 42 U.S.C. § 300ii(5)'s own OUTER list —
/// "foster" (as in "a foster parent") ALSO carries a genuine transitive-verb
/// WordNet sense ("to foster a relationship"), so a same-entries-any check
/// wrongly treated it as a clause boundary and blocked the walk from ever
/// reaching "member,"'s comma. English is full of exactly this
/// noun/adjective/verb homonymy (open-class content words), whereas the
/// RELATIVE-PRONOUN class is closed and unambiguous (a function word with no
/// competing open-class reading) — the SAME "closed function-word class over
/// an ambiguity-prone open-class POS check" precision preference
/// [`is_medial_supplement_interior_head`]'s own doc establishes.
fn is_clause_boundary_word(word: &str, language: &dyn Language) -> bool {
    language
        .lexical_lookup_all(&word.to_lowercase())
        .iter()
        .any(|entry| entry.olia_class() == Some("RelativePronoun"))
}

/// Does `word` carry a [`svo_types::transitive_verb`] reading among ALL of
/// its loaded lexical entries (never just the primary one — an entry
/// ORDERING dependency [`is_medial_supplement_interior_head`]'s own
/// verb-adjacency check must not have, since a homonym's transitive-verb
/// sense need not be listed first)? The generic "is this word a transitive
/// verb" query [`collapse_medial_comma_adjuncts`] anchors BOTH medial-
/// supplement gates on, reusing [`entry_categories`] rather than a
/// hardcoded verb list — so the mechanism generalizes to ANY transitive
/// verb a comma-bracketed supplement sits beside (not just "means"/
/// "includes", the two this task's real examples happen to use).
fn is_transitive_verb_leaf(word: &str, language: &dyn Language) -> bool {
    language
        .lexical_lookup_all(&word.to_lowercase())
        .iter()
        .any(|entry| entry_categories(entry).contains(&svo_types::transitive_verb()))
}

/// Is `word` a DETERMINER — checked against its PRIMARY loaded reading only
/// (mirroring [`select_best_entry`]'s own "the language orders entries by
/// priority" convention), never ANY of its entries the way
/// [`is_transitive_verb_leaf`] deliberately does? [`collapse_medial_comma_adjuncts`]'s
/// post-verb gate uses this as a NOUN-USAGE precision guard: English never
/// puts a finite verb directly after a determiner (a determiner selects a
/// NOUN), so a word IMMEDIATELY preceded by one is never a genuine "means"-
/// like verb trigger, however many OTHER homonym senses it also carries — a
/// REAL, measured false positive against 42 U.S.C. § 289b–1(f)(2)
/// "assistance": "grant" is a legitimate transitive verb in one WordNet
/// sense ("to grant approval"), which — before this guard — wrongly fired
/// the post-verb gate on "a grant, contract, or cooperative agreement." (a
/// three-item coordinated list, "grant" used as a plain NOUN) and swallowed
/// "contract" into a bogus supplement bracket.
fn is_determiner(word: &str, language: &dyn Language) -> bool {
    language
        .lexical_lookup_all(&word.to_lowercase())
        .first()
        .is_some_and(|entry| matches!(entry.pos_tag(), PosTag::Determiner | PosTag::Article))
}

/// Is `word` a CLOSED medial-supplement INTERIOR-HEAD surface — "used" /
/// "as" / "when" — the word that OPENS a subject-verb-position comma
/// supplement in every real construction this task's report names or this
/// module's own test suite already carries: "used with respect to
/// individuals with developmental disabilities" (42 U.S.C. § 15002-area,
/// "inclusion"), "when used in connection with ..." (42 U.S.C. § 1395x(r),
/// "physician"), "as used in this title" (title 18's "vessel of the United
/// States", already a committed `grounding` test fixture before this task).
///
/// A hand-authored closed-class check, the SAME rationale
/// [`is_do_support`]/[`is_modal_auxiliary`]/[`is_nominal_coordinator`]/
/// [`is_fronted_scope_adjunct_np_head`] already establish: these three are a
/// REGISTER-SPECIFIC legislative-drafting formula (the "as used in this
/// Act/section/title" boilerplate — U.S. House Office of the Legislative
/// Counsel, "Quick Guide to Legislative Drafting", <https://legcounsel.house.gov/holc-guide-legislative-drafting>,
/// the SAME publisher-class citation `is_partial_definition_verb`
/// (`social::judicial::statute_structure::grounding`) already uses for its
/// own closed "includes" class), instantiating the general reduced-
/// participial / "as"-headed supplement Huddleston & Pullum (2002), *The
/// Cambridge Grammar of the English Language*, Ch. 15 "Supplements" §7
/// describes — not a distinction the loaded lexicon's POS classes make (a
/// past participle and a subordinator carry no shared loaded feature this
/// grammar could key a broader row on).
///
/// This gate is the load-bearing PRECISION guard against a coordinated
/// SUBJECT list wrongly read as a supplement bracket ("The agency, county,
/// and state, participate." — comma-adjacent-to-a-later-verb alone is NOT
/// enough, since "county"/"state" are never members of this closed set):
/// verified no real coordination-list continuation in this codebase's own
/// corpus opens with "as"/"when"/"used".
fn is_medial_supplement_interior_head(word: &str) -> bool {
    matches!(word.to_lowercase().as_str(), "as" | "when" | "used")
}

/// Is `word` the head of a TRAILING, comma-set-off "whether X or Y"
/// adjunct — an EXHAUSTIVE CONDITIONAL in Huddleston & Pullum's own
/// terminology (2002, *The Cambridge Grammar of the English Language*,
/// Cambridge University Press, Ch. 8 "Adjuncts" §14.6 "Exhaustive
/// conditionals", pp. 761-5, with the "whether ... or" interrogative-clause
/// structure it embeds covered in Ch. 11 "Content clauses and reported
/// speech", pp. 985-91: the main clause holds regardless of which
/// alternative obtains, e.g. "we'll go ahead whether you turn up or not".
/// Page range confirmed against a SECOND, independent secondary source —
/// Arnold, D. & Borsley, R. D. (2014), "On the Analysis of English
/// Exhaustive Conditionals", *Proceedings of the 21st International
/// Conference on Head-Driven Phrase Structure Grammar*, CSLI Publications,
/// pp. 27-47 at p.28 — which cites the identical "H&P 2002: 761-5, 985-91"
/// range for this construction and its own examples "(no matter) whether
/// he goes to Wales or to Scotland" / "(no matter) whether it's essential
/// or not", the same syntactic shape this corpus's statutory idiom
/// instantiates. Quirk, Greenbaum, Leech & Svartvik (1985), *A
/// Comprehensive Grammar of the English Language*, Longman, §15.40
/// "Alternative conditional-concessive clauses", pp. 1070-1, cover the same
/// "whether ... or" construction under the older "conditional-concessive"
/// label — a viable alternative citation, but H&P's "exhaustive
/// conditional" is the more precise, more widely cited modern term and the
/// one this file's own naming already follows). Confirmed a recurring
/// statutory idiom in this corpus's business-entity definitions ("any
/// organized group of persons, whether incorporated or unincorporated",
/// "any other banking institution..., whether incorporated or not") — and
/// confirmed, via direct bisection on the real "Company" definition, that
/// its presence breaks the WHOLE derivation: even a trivial single-item
/// definiens ("Company" means a corporation.) fails to extract once this
/// trailing adjunct is appended, and succeeds again the moment it's
/// removed. "Whether" carries no loaded POS feature distinguishing this
/// exhaustive-conditional use from its ordinary embedded-interrogative use
/// ("I wonder whether X") — the SAME register-specific hand-authored-
/// closed-class rationale [`is_medial_supplement_interior_head`] already
/// establishes for "as"/"when"/"used" applies here. Scoped further by
/// [`collapse_medial_comma_adjuncts`]'s own caller: the construction is
/// DEFINITIONALLY "whether P or Q"/"whether or not P" (H&P, same section),
/// so the caller additionally requires an "or" ([`nominal_coordinator_
/// canonical`]) inside the swallowed span before this head triggers a drop
/// — a bare comma-adjacent "whether" with no "or" anywhere before the next
/// clause/sentence boundary is NOT this construction and is left untouched.
/// A THIRD guard — [`is_predicate_leaf`], on the word immediately BEFORE the
/// comma — additionally excludes exactly the "ordinary embedded-
/// interrogative use" case named above: see that function's own doc for the
/// real, corpus-attested sentence that use requires.
fn is_trailing_alternative_adjunct_head(word: &str) -> bool {
    word.to_lowercase() == "whether"
}

/// Is `word` a VERB, COPULA, or AUXILIARY under ANY of its loaded lexical
/// entries — checking every entry, never just the primary one, the SAME
/// safety-biased "any entry" convention [`is_transitive_verb_leaf`] already
/// establishes (as opposed to [`is_determiner`]'s deliberately PRIMARY-only
/// check): this guard's failure mode if it under-fires is silent deletion of
/// real sentence content, so it must not miss a homonym's verbal reading.
///
/// [`collapse_medial_comma_adjuncts`]'s trailing-whether-adjunct branch
/// requires this to be FALSE for the word immediately before the triggering
/// comma. Huddleston & Pullum's exhaustive-conditional ADJUNCT (2002, *The
/// Cambridge Grammar of the English Language*, Ch. 8 §14.6) modifies a
/// clause that is already syntactically COMPLETE without it — but "whether
/// ... or" is ALSO the standard introducer of an INTERROGATIVE CONTENT
/// CLAUSE functioning as a verb's or copula's own COMPLEMENT (H&P Ch. 11
/// "Content clauses and reported speech", pp. 971-1030: "know"/"wonder"/
/// "ask"/"doubt"/"determine"/"question"/"matter" all subcategorize for an
/// interrogative complement, and a copula's own SUBJECT-COMPLEMENT content
/// clause — "the question is whether P or Q" — is the identical
/// construction with `be` in place of the matrix verb) — and dropping THAT
/// reading destroys the sentence's entire propositional content rather than
/// removing an optional adjunct.
///
/// Confirmed via a REAL, professionally-edited sentence already present in
/// this repo's own PropBank corroboration data (`crates/domains/data/
/// propbank/propbank-3.4.0.propbank`, Wall Street Journal-sourced, Palmer,
/// Gildea & Kingsbury 2005): "That is , whether there should be a
/// separation of politics and economics or not ." — comma directly before
/// "whether", an "or" present, no closing comma before the final period,
/// EXACTLY the trailing-adjunct trigger shape — yet "whether ... or not"
/// here is the copula "is"'s own subject-complement clause, not an adjunct;
/// without this guard the branch collapsed the whole sentence to `["that",
/// "is"]`, confirmed by direct execution
/// (`trailing_whether_after_a_complement_taking_predicate_survives`,
/// below). A direct sweep of every real "means"/"includes" definitional
/// sentence containing a trailing ", whether" across all 9 loaded USC
/// titles (`crates/domains/data/legal/uscode/**`, `rg -a --no-ignore -o
/// ".{0,60}\bmeans\b.{0,400}?, ?[Ww]hether.{0,120}"` and the `includes`
/// counterpart) found this guard costs NO measured recall on all but ONE
/// real definitional trigger: every other real "means"/"includes" sentence
/// with this shape has a bare NOUN immediately before the comma (e.g. the
/// "Company" definition's "persons, whether incorporated or
/// unincorporated" — [`is_trailing_alternative_adjunct_head`]'s own
/// doc-cited "banking institution..., whether incorporated or not";
/// confirmed directly: `english_loaded().lexical_lookup_all("persons")`
/// returns `[Noun]` only). The ONE measured exception — 15 U.S.C. §§
/// 80a–2(a)(10), 80b–2(a)(6) ("Convicted" includes ... "has not been
/// reversed, set aside, or withdrawn, whether or not sentence has been
/// imposed.") — has the coordination-FINAL past participle "withdrawn"
/// (`[Adjective, Verb, Verb]`) immediately before the comma; this guard now
/// blocks that sentence's trailing-whether drop too, so `defines_pointers`
/// no longer extracts "Convicted" from this specific phrasing (a RECALL
/// loss, not a wrong extraction — the sentence simply fails to derive a
/// chart parse, same as before either whether-fix existed). A DELIBERATE,
/// disclosed trade: distinguishing "withdrawn" (coordination-final,
/// non-complement-taking) from "know"/"is" (complement-taking) precisely
/// would need a real subcategorization-frame check (e.g. against the
/// already-loaded VerbNet classes) rather than a bare POS tag — a
/// well-scoped follow-up, not required to close the safety gap this guard
/// targets, and the caregiver-chat safety cost of the OPPOSITE failure mode
/// (silently deleting unrelated live-chat content, confirmed above) is
/// categorically worse than this narrow, honest recall miss.
fn is_predicate_leaf(word: &str, language: &dyn Language) -> bool {
    language
        .lexical_lookup_all(&word.to_lowercase())
        .iter()
        .any(|entry| {
            matches!(
                entry.pos_tag(),
                PosTag::Verb | PosTag::Copula | PosTag::Auxiliary
            )
        })
}

/// Collapse a MEDIAL, comma-delimited SUPPLEMENT — a parenthetical breaking
/// either a definiendum's adjacency to its own verb ("the term 'X', used
/// with respect to Y, means ...") or a "means"/"includes"-class verb's
/// adjacency to its own object ("means, with respect to Y, Z", the EVV
/// headline shape, 42 U.S.C. § 1396b(l)(5)) — into ONE opaque synthetic
/// token (mirroring [`collapse_quoted_spans`]'s own bracket-merge shape,
/// generalized from a quote-delimited span to a comma-delimited one),
/// carrying a reserved marker word ([`MEDIAL_SUPPLEMENT_NP_MARKER`] /
/// [`MEDIAL_SUPPLEMENT_VERB_MARKER`]) so `assign_type` can give it the
/// dedicated transparent-modifier category (`svo_types::
/// medial_supplement_np`/`medial_supplement_verb`) without ever running its
/// (deliberately unparsed) interior through ordinary lexicon-driven typing —
/// the defines-lens gap backlog's G2, Huddleston & Pullum (2002), *The
/// Cambridge Grammar of the English Language*, Ch. 15 "Supplements" §1: a
/// supplement is "not integrated into the syntactic structure" it
/// interrupts, so its INTERNAL shape (participial, PP-headed, whatever) is
/// irrelevant to how the HOST sentence composes — exactly why this grammar
/// never needs to parse it (that would be G4 territory: participial
/// postmodifiers, PP chains, out of scope here).
///
/// Runs BEFORE [`collapse_quoted_spans`] (this pipeline's OTHER bracket
/// merge) — the comma is the ONLY signal this function reads, still visible
/// via `comma_after` (computed in [`flush_word_tracked`], the sole point in
/// the pipeline where the raw comma character survives `flush_word`'s
/// trim), so it never needs the definiendum's own quote span to have
/// resolved first; a definiendum quote span sitting OUTSIDE whatever range
/// this function merges is untouched, and `collapse_quoted_spans` still
/// finds and merges it normally afterward.
///
/// THREE gates, checked at each candidate comma boundary `i` (a position
/// with `comma_after[i]`), in this exact priority order — HIGHEST priority
/// first, unlike an earlier version of this function, which checked the two
/// supplement gates BEFORE consulting `list_coordinator_commas` and, as a
/// result, let a lexical false positive silently corrupt a real coordinated
/// list (see the list-coordination gate's own inline comment, below, for the
/// confirmed real-statute example and the direct chart/Montague
/// instrumentation that isolated it):
/// - **List coordination** ([`find_list_coordinator_commas`], defines-lens
///   gap G4(a)), checked FIRST: is `i` already a member of
///   `list_coordinator_commas` — a comma this function has ALREADY,
///   STRUCTURALLY determined stands in for a list's own "and"/"or" (reached
///   by an unbroken, clause-boundary-free comma chain from a literal
///   coordinator later in the sentence)? If so, mint the coordinator marker
///   token RIGHT AFTER it, without skipping or merging anything (unlike the
///   two supplement gates below, which swallow their whole bracketed
///   interior) — every other word in the sentence is untouched. This MUST
///   run before either supplement gate: a comma this map contains can never
///   simultaneously be a genuine supplement-bracket opener, and the two
///   supplement gates key on a word's incidental LEXICAL polysemy
///   (`is_transitive_verb_leaf`), a strictly weaker signal than this map's
///   own list-structural one.
/// - **Post-verb** (`svo_types::medial_supplement_verb`), checked SECOND,
///   only once the list-coordination gate has passed: `words[i]` itself
///   is a transitive verb ([`is_transitive_verb_leaf`]), NOT immediately
///   preceded by a determiner ([`is_determiner`] — see its own doc for the
///   real "a grant, contract, or cooperative agreement" false positive this
///   guard fixes) — the comma sits DIRECTLY after it — and a later closing
///   comma exists. The verb-ADJACENCY requirement is the PRIMARY precision
///   guard here: an ordinary coordinated object list ("pays for doctors,
///   nurses, and equipment") never has its FIRST comma directly after the
///   verb; the determiner guard closes the remaining gap where the
///   adjacent word is itself a coordinated list's own determiner-headed
///   FIRST conjunct that merely happens to carry an unrelated verb sense —
///   though NOT the gap where a MULTI-WORD premodified compound's own HEAD
///   noun carries the unrelated verb sense and is NOT itself adjacent to
///   the determiner (the list-coordination gate above closes that one).
/// - **Subject-verb** (`svo_types::medial_supplement_np`), checked THIRD: a
///   later closing comma exists, its interior OPENS with a closed
///   supplement head ([`is_medial_supplement_interior_head`]), AND the word
///   immediately after the closing comma is ALSO a transitive verb — the
///   "the true definiens on either side... still gets extracted correctly"
///   shape the report's G2 entry and this module's own pre-existing
///   `a_real_parenthetical_interrupted_sample_yields_no_pointer` fixture
///   ("as used in this title") both name.
///
/// Nearest-close (the FIRST comma found after the opener), never nested-
/// bracket-aware — the SAME documented, accepted limitation
/// [`collapse_quoted_spans`]'s own doc comment carries for quote spans. A
/// comma satisfying none of the three gates is left completely alone: the
/// comma information is simply dropped, exactly as it already was before
/// this function existed.
///
/// Widen `sentence_initial` to also mark the token that begins the MAIN
/// clause after a fronted scope-setting adjunct ("In self-direction, WHAT
/// is an authorized representative?") — a real, confirmed defect this
/// closes. `assign_type`'s `position == 0` branch is the ONLY place a
/// sentence-initial interrogative pronoun/adverb or modal auxiliary gets
/// its question-forming CCG category (via the loaded OLiA→CCG functor, or
/// `svo_types::modal_question`/`question_copula_pp`) — correct for an
/// ordinary sentence, where token 0 IS the clause's own first word, but
/// `svo_types::fronted_scope_adjunct_np`/`_pp` (G1) lets a DIFFERENT word
/// occupy token 0 while the actual interrogative clause starts several
/// tokens later, right after the adjunct's own comma. Left unwidened, "what"
/// in "In personal care, what is X?" falls through `assign_type`'s
/// `position == 0` gate entirely and receives the ordinary (non-question)
/// pronoun category instead — `chart_reduce`'s exhaustive search then finds
/// SOME OTHER complete bracketing (misreading "what" as the clause's plain
/// NP subject instead of its wh-operator), which `montague::interpret`
/// faithfully reports as a garbled `Sem::Prop` rather than the intended
/// `Sem::Question`, and the turn falls through to a generic decline instead
/// of answering. This is exactly the "a new capability exposes an adjacent
/// pre-existing weakness" pattern this file's own `nominal_coordinator_np`
/// entry (defines-lens gap G4) already names: the `position == 0` gate was
/// always this narrow, but no sentence could ever place a real interrogative
/// clause after token 0 until G1 gave a fronted adjunct its own S/S reading.
///
/// A fronted adjunct's own complement is, by construction (Huddleston &
/// Pullum 2002 Ch. 15 §2's supplementary/parenthetical-comma account, the
/// SAME citation this file's medial-supplement machinery above already
/// relies on), set off by the FIRST comma boundary after the adjunct head —
/// so the token right after that comma is where the main clause begins.
/// Reusing `sentence_initial` (rather than threading a brand-new parallel
/// array through every downstream merge/split stage this module's pipeline
/// already carries `sentence_initial` through) is deliberate: every
/// consumer downstream of this function (`collapse_capitalized_runs`'s own
/// sentence-initial exclusion, and — via `assign_type`/the `i == 0`-gated
/// alternative blocks in `tokenize_with_alternatives_registry_aware` —
/// every question-forming category assignment) wants the IDENTICAL
/// "starts a new clause" signal a real sentence boundary already carries;
/// a fronted comma-set-off adjunct genuinely starts one too, so folding
/// this into the same flag is not an overload, it is the same concept.
fn mark_clause_initial_after_fronted_adjunct(
    words: &[String],
    mut sentence_initial: Vec<bool>,
    comma_after: &[bool],
) -> Vec<bool> {
    let mut start = 0;
    while start < words.len() {
        let end = ((start + 1)..words.len())
            .find(|&k| sentence_initial[k])
            .unwrap_or(words.len());
        if is_fronted_scope_adjunct_head(&words[start].to_lowercase())
            && let Some(comma_at) = (start..end).find(|&k| comma_after[k])
            && comma_at + 1 < end
        {
            sentence_initial[comma_at + 1] = true;
        }
        start = end;
    }
    sentence_initial
}

fn collapse_medial_comma_adjuncts(
    words: Vec<String>,
    sentence_initial: Vec<bool>,
    comma_after: Vec<bool>,
    semicolon_after: Vec<bool>,
    language: &dyn Language,
) -> (Vec<String>, Vec<bool>) {
    let sentence_initial =
        mark_clause_initial_after_fronted_adjunct(&words, sentence_initial, &comma_after);
    let list_coordinator_commas = find_list_coordinator_commas(&words, &comma_after, language);
    let mut out_words = Vec::with_capacity(words.len());
    let mut out_sentence_initial = Vec::with_capacity(words.len());
    let mut i = 0;
    while i < words.len() {
        if comma_after[i] {
            if let Some(&conjunction) = list_coordinator_commas.get(&i) {
                // List coordination: "X, Y, or Z" -- the comma stands in for
                // the list's own conjunction, minted as its own coordinator
                // token, everything else untouched. Checked FIRST, ahead of
                // BOTH medial-supplement gates below -- `list_coordinator_commas`
                // (computed once, above, by `find_list_coordinator_commas`)
                // is a STRUCTURAL fact about this exact comma: it is reachable
                // by an unbroken, clause-boundary-free chain from a literal
                // "and"/"or" pivot later in the sentence, so it can never
                // simultaneously be a genuine supplement-bracket opener. That
                // is a strictly more precise signal than either medial-
                // supplement gate's own check (`is_transitive_verb_leaf` --
                // ANY lexical entry, not just the word's PRIMARY sense, paired
                // with `is_determiner` looking only at the IMMEDIATELY
                // preceding word). A REAL, confirmed false positive without
                // this priority ordering: 15 U.S.C. § 80a-2(a)(8)/77b(a)(2)'s
                // "a corporation, a partnership, an association, a joint-stock
                // company, a trust, ..., or ..." -- "company", the HEAD noun
                // of the multi-word premodified compound "joint-stock
                // company", carries an archaic transitive-verb WordNet sense
                // ("to company with" = accompany) among its entries, and is
                // NOT itself immediately preceded by a determiner (the
                // premodifier "joint-stock" sits between "a" and "company"),
                // so the post-verb gate's own guard does not catch it. With
                // 2+ conjuncts still remaining in the list after "company"
                // (giving the gate a plausible-looking closing comma), it
                // used to swallow the ENTIRE next conjunct into an opaque,
                // verb-typed marker that can never combine with the coordinated-
                // NP structure actually to its left -- breaking the whole
                // derivation, confirmed via direct chart/Montague instrumentation
                // (`probe_company_chart_divergence_point`,
                // crates/praxis-corpus-tests/tests/scratch_probe.rs) showing
                // `chart_reduce_with_costs_bounded` correctly finds NO S-level
                // derivation once the token stream is corrupted this way --
                // never a chart-level coordination-composition gap. With <2
                // items remaining (so no valid closing comma exists), the old
                // ordering happened to fall through safely by luck, not by
                // design -- exactly why this is a priority-inversion bug, not
                // an arity limit: a genuinely N-ary-robust fix must not depend
                // on how many conjuncts happen to remain.
                out_words.push(words[i].clone());
                out_sentence_initial.push(sentence_initial[i]);
                out_words.push(
                    match conjunction {
                        "and" => LIST_COORDINATOR_MARKER_AND,
                        _ => LIST_COORDINATOR_MARKER_OR,
                    }
                    .to_string(),
                );
                out_sentence_initial.push(false);
                i += 1;
                continue;
            }
            let preceded_by_determiner = i > 0 && is_determiner(&words[i - 1], language);
            if is_transitive_verb_leaf(&words[i], language) && !preceded_by_determiner {
                // Post-verb: "means, with respect to Y, Z".
                if let Some(close) = ((i + 1)..words.len()).find(|&k| comma_after[k]) {
                    out_words.push(words[i].clone());
                    out_sentence_initial.push(sentence_initial[i]);
                    out_words.push(MEDIAL_SUPPLEMENT_VERB_MARKER.to_string());
                    out_sentence_initial.push(false);
                    i = close + 1;
                    continue;
                }
            } else if let Some(close) = ((i + 1)..words.len()).find(|&k| comma_after[k]) {
                // Subject-verb: "X, used with respect to Y, means Z".
                let interior_head_ok = words
                    .get(i + 1)
                    .is_some_and(|w| is_medial_supplement_interior_head(w));
                let followed_by_verb = words
                    .get(close + 1)
                    .is_some_and(|w| is_transitive_verb_leaf(w, language));
                if interior_head_ok && followed_by_verb {
                    out_words.push(words[i].clone());
                    out_sentence_initial.push(sentence_initial[i]);
                    out_words.push(MEDIAL_SUPPLEMENT_NP_MARKER.to_string());
                    out_sentence_initial.push(false);
                    i = close + 1;
                    continue;
                }
            }
            let sentence_bound = ((i + 1)..words.len())
                .find(|&k| sentence_initial[k])
                .unwrap_or(words.len());
            // NEVER scan across a semicolon — `PunctuationFunction::
            // Connector`, a between-INDEPENDENT-clauses role
            // (`is_semicolon_char`'s own doc) — even though the general
            // tokenizer's `sentence_initial` does not treat one as a
            // sentence boundary (deliberately, `split_into_sentences`'s own
            // doc). Confirmed a REAL regression without this bound, not
            // hypothetical: on the live-chat path (`tokenize_with_
            // alternatives_registry_aware`/`tokenize_ontological_registry_
            // aware`, both called directly on raw user input with no
            // semicolon pre-split), "The agent may act for the principal,
            // whether or not authorized in writing; the principal remains
            // liable for any debts the agent incurs." tokenized to just 7
            // words — the ENTIRE independent clause after the semicolon
            // silently vanished (`probe_trailing_whether_drop_crosses_
            // semicolon_on_chat_path`, `crates/praxis-corpus-tests/tests/
            // scratch_probe.rs`). Bounding at the semicolon instead of the
            // next sentence boundary keeps the fix scoped to exactly the
            // "whether"-adjunct's own span.
            let semicolon_bound = ((i + 1)..words.len())
                .find(|&k| semicolon_after[k])
                .map(|k| k + 1)
                .unwrap_or(words.len());
            let clause_end = sentence_bound.min(semicolon_bound);
            // Huddleston & Pullum's exhaustive conditional is definitionally
            // "whether P or Q" / "whether or not P" (2002, *The Cambridge
            // Grammar of the English Language*, Ch. 8 "Adjuncts" §14.6
            // "Exhaustive conditionals", pp. 761-5, cross-referencing the
            // interrogative-clause-internal "whether ... or" structure
            // covered in Ch. 11 "Content clauses and reported speech",
            // pp. 985-91 — page range confirmed against Arnold & Borsley
            // (2014), "On the Analysis of English Exhaustive Conditionals",
            // Proc. HPSG21, CSLI, pp. 27-47 at p.28, which cites the SAME
            // "H&P (2002: 761-5, 985-91)" range for this construction) — the
            // "or" is a REQUIRED part of the construction, not incidental.
            // Reusing [`nominal_coordinator_canonical`] (already loaded,
            // already used by this same module's list-coordination
            // machinery) rather than re-deriving a second "or" recognizer
            // keeps this gate consistent with every OTHER coordinator check
            // in this file, and closes off the one shape neither the
            // semicolon bound above nor the closing-comma guard below would
            // catch: an ordinary comma-adjacent, no-further-comma, no-"or"
            // "whether"-headed clause that is NOT this construction at all.
            let contains_alternative_marker = ((i + 1)..clause_end)
                .any(|k| nominal_coordinator_canonical(&words[k]) == Some("or"));
            // `words[i]` — the word DIRECTLY BEFORE the triggering comma —
            // must not itself be a verb/copula/auxiliary reading
            // ([`is_predicate_leaf`]'s own doc has the citation and the real
            // PropBank counter-example this guard was built to close): a
            // "whether ... or" run immediately after "know"/"is"/"wonder"/
            // "matter" is that predicate's own interrogative COMPLEMENT, not
            // an adjunct on an already-complete clause, and dropping it
            // destroys the sentence rather than trimming an optional
            // adjunct. Every currently-verified real trigger for this branch
            // has a bare noun here, so this costs no measured precision.
            let host_clause_is_complete = !is_predicate_leaf(&words[i], language);
            if words
                .get(i + 1)
                .is_some_and(|w| is_trailing_alternative_adjunct_head(w))
                && contains_alternative_marker
                && !((i + 1)..clause_end).any(|k| comma_after[k])
                && host_clause_is_complete
            {
                // TRAILING comma-set-off "whether X or Y" adjunct running to
                // the end of the clause (or, now, to the nearest semicolon —
                // see above) — the counterpart to the MEDIAL supplements
                // above, which all require a CLOSING comma to bound the span
                // ("X, used with respect to Y, means Z"). "a corporation,
                // whether incorporated or unincorporated." has no closing
                // comma (the clause simply ends), so none of the medial
                // branches above ever fire for it, and the raw "whether"/
                // "or" tokens reach the chart parser with no valid category,
                // breaking the WHOLE derivation — confirmed via direct
                // bisection: even a single-item definiens ("Company" means a
                // corporation.) fails once this trailing adjunct is
                // appended, succeeding again the moment it's removed. Unlike
                // the medial markers (which reconnect a gap that has content
                // on both sides), a trailing adjunct has nothing after it to
                // reconnect to, so the whole span is dropped rather than
                // replaced with a placeholder token.
                out_words.push(words[i].clone());
                out_sentence_initial.push(sentence_initial[i]);
                i = clause_end;
                continue;
            }
        }
        out_words.push(words[i].clone());
        out_sentence_initial.push(sentence_initial[i]);
        i += 1;
    }
    (out_words, out_sentence_initial)
}

/// Split a genuine possessive/genitive clitic ("consumer's") off its stem
/// into two surface tokens, so the stem resolves through its OWN normal
/// lexicon entry (instead of the whole compound falling out-of-vocabulary)
/// and the clitic carries the dedicated `svo_types::genitive_clitic`
/// category.
///
/// Scoped to stems whose PRIMARY lexical reading ([`select_best_entry`]'s
/// same first-entry priority convention `assign_type` already uses) is a
/// common noun, or that are unknown to the lexicon entirely (the same
/// open-class default an out-of-vocabulary word already gets). A stem whose
/// primary reading is a pronoun/determiner/adverb/verb ("what's", "there's",
/// "that's", "he's", "let's", ...) is a CONTRACTION of "is"/"has"/"us", not
/// a genitive — Huddleston & Pullum (2002), *The Cambridge Grammar of the
/// English Language*, Ch. 5 §16: the genitive attaches productively to full
/// noun phrases, not to the closed pronoun/wh class, which instead has its
/// own dedicated contracted forms — so those stems are left untouched here
/// for the existing pipeline (unaffected: verified against
/// `probe_genitive_split_safety_closed_class_check`, which confirmed every
/// contraction-prone closed-class word's PRIMARY entry is non-Noun even
/// where a secondary homonym entry is a noun, e.g. "why"/"there"/"here").
///
/// A quote-collapsed span (`quoted[i]`) is a mentioned expression and is
/// left intact — splitting inside it would break the Slice B quoted-NP
/// reading. Never touches leading punctuation; the possessive marker is
/// read from the loaded [`punctuation::apostrophe`] concept, not a bare
/// char literal.
fn split_possessive_clitics(
    words: Vec<String>,
    quoted: Vec<bool>,
    sentence_initial: Vec<bool>,
    language: &dyn Language,
) -> (Vec<String>, Vec<bool>, Vec<bool>) {
    let apostrophe_char = punctuation::apostrophe().character;
    let mut out_words = Vec::with_capacity(words.len());
    let mut out_quoted = Vec::with_capacity(words.len());
    let mut out_sentence_initial = Vec::with_capacity(words.len());
    for ((word, is_quoted), is_initial) in words.into_iter().zip(quoted).zip(sentence_initial) {
        if is_quoted {
            out_words.push(word);
            out_quoted.push(is_quoted);
            out_sentence_initial.push(is_initial);
            continue;
        }
        let chars: Vec<char> = word.chars().collect();
        let split_at = if chars.len() >= 3
            && chars[chars.len() - 2] == apostrophe_char
            && chars[chars.len() - 1].eq_ignore_ascii_case(&'s')
        {
            Some(chars.len() - 2)
        } else {
            None
        };
        let stem_is_noun_or_unknown = split_at.is_some_and(|at| {
            let stem: String = chars[..at].iter().collect();
            let primary_pos = language
                .lexical_lookup_all(&stem.to_lowercase())
                .first()
                .map(|e| e.pos_tag());
            matches!(primary_pos, None | Some(PosTag::Noun))
        });
        if let Some(at) = split_at
            && stem_is_noun_or_unknown
        {
            out_words.push(chars[..at].iter().collect());
            out_quoted.push(false);
            out_sentence_initial.push(is_initial);
            out_words.push(chars[at..].iter().collect());
            out_quoted.push(false);
            out_sentence_initial.push(false);
        } else {
            out_words.push(word);
            out_quoted.push(is_quoted);
            out_sentence_initial.push(is_initial);
        }
    }
    (out_words, out_quoted, out_sentence_initial)
}

/// Is `c` an uppercase LATIN letter — per the loaded [`character::latin`]
/// script's own [`character::UnicodeCategory::UppercaseLetter`]
/// classification, never a bare `char::is_ascii_uppercase`? The same
/// "query the loaded concept, don't hardcode the range" idiom
/// [`is_sentence_terminal_char`] already applies to punctuation.
fn is_uppercase_latin(c: char) -> bool {
    character::latin()
        .characters
        .into_iter()
        .any(|ch| ch.codepoint == c && ch.category == character::UnicodeCategory::UppercaseLetter)
}

/// Collapse a maximal run of 2+ consecutive Title-Case words into ONE
/// surface token typed as a determinerless proper-noun NP — the symbolic
/// capitalization-run precedent for named-entity recognition (Grishman &
/// Sundheim 1996, MUC-6 task definition §2; Huddleston & Pullum 2002 Ch. 5
/// §20 for the determinerless-NP proper-name analysis itself, the same
/// citation the OOV proper-noun alternative in
/// [`tokenize_with_alternatives`] already uses).
///
/// A word can START or CONTINUE a run only if it is NOT already quoted (a
/// quote-collapsed span already carries its own NP reading — [Slice B]),
/// NOT sentence-initial, and its first character is genuinely uppercase
/// Latin per [`is_uppercase_latin`]. The sentence-initial exclusion is load-
/// bearing, not cosmetic: an audit of this corpus found ~11% of questions
/// are written in Title-Case STYLE throughout (a formatting convention, not
/// entity marking), so ordinary sentence-initial capitalization — and a
/// second sentence's own opening word, in multi-sentence input — carries no
/// entity signal on its own. A run of exactly one capitalized word is left
/// alone here: it already reaches a proper-noun reading through the
/// existing OOV-alternative path in [`tokenize_with_alternatives`] when
/// unknown to the lexicon, so collapsing singletons here would be
/// redundant, not a new capability.
///
/// Is `c` a Latin-script letter (upper OR lower) per the loaded
/// [`character::latin`] script — the same loaded-concept idiom
/// [`is_uppercase_latin`] uses for the narrower uppercase check?
fn is_latin_letter(c: char) -> bool {
    character::latin().contains(c)
}

/// The maximum fraction of an input's significant (Latin-letter) words that
/// may be capitalized before the WHOLE input is treated as TITLE-CASE
/// STYLED — a document formatting convention, not entity marking — rather
/// than judged word-by-word. Mikheev (1999), "A Knowledge-free Method for
/// Capitalized Word Disambiguation" (ACL), establishes the general method
/// this follows: use DOCUMENT-WIDE capitalization density as the
/// disambiguating signal for a word's capitalization, rather than judging
/// each word in isolation. The specific ratio is this corpus's own
/// empirically-set operating point, not itself from Mikheev: an audit of
/// the caregiver corpus found ~11% of its questions are written in
/// Title-Case style throughout (most/all significant words capitalized,
/// e.g. "Mom Is on Medicaid. What Happens If We Sell Her House?"); 0.6
/// cleanly separates that population from ordinary prose carrying an
/// occasional genuine capitalized-entity run. A dimensionless [`Quantity`]
/// (never a bare `f64` literal at the comparison site), compared via
/// [`Quantity`]'s dimension-safe `PartialOrd`.
fn title_case_style_ratio_ceiling() -> Quantity {
    Quantity::dimensionless(0.6)
}

/// Is this WHOLE input written in Title-Case STYLE — so pervasively
/// capitalized that no individual word's capitalization carries entity
/// signal? Computed once per input (never per-sentence, since a harvested
/// question routinely mixes a Title-Case headline sentence with a plain
/// second sentence — e.g. "Mom's in a Nursing Home... What Do I Do?" — and
/// EITHER sentence being styled this way makes capitalization untrustworthy
/// for the WHOLE input) over every non-quoted word with a Latin-letter first
/// character, against [`title_case_style_ratio_ceiling`].
fn is_title_case_styled(words: &[String], quoted: &[bool]) -> bool {
    let significant: Vec<&String> = words
        .iter()
        .zip(quoted)
        .filter(|&(w, q)| !*q && w.chars().next().is_some_and(is_latin_letter))
        .map(|(w, _)| w)
        .collect();
    if significant.len() < 2 {
        return false;
    }
    let capitalized = significant
        .iter()
        .filter(|w| w.chars().next().is_some_and(is_uppercase_latin))
        .count();
    let ratio = Quantity::dimensionless(capitalized as f64 / significant.len() as f64);
    ratio > title_case_style_ratio_ceiling()
}

/// Why a token bypasses lexicon-driven typing and takes a saturated
/// proper-noun NP outright — the TYPED provenance
/// [`collapse_capitalized_runs`] reports per output token, replacing the
/// earlier lossy `force_np: bool`. The two forcing provenances need DIFFERENT
/// alternative-type inventories at the typing site, a distinction a bare bool
/// erased ([`tokenize_with_alternatives`] reads it; the two single-primary
/// callers only ever ask [`forces_np`](NpForcing::forces_np)):
///
/// - [`QuotedMention`](NpForcing::QuotedMention): a quoted metalinguistic
///   mention span ([`collapse_quoted_spans`]) — NP outright (a mentioned
///   expression's ordinary category is not the reading a quoted argument
///   needs), PLUS the close-apposition `NP\NP` the postnominal-definiendum
///   reading "the term 'X'" needs. A mention is NOT a common noun: it must
///   never offer a bare `N`, or "the term 'State'" would admit a spurious
///   "the [term state]" `N/N`-compound bracketing that erases the apposition
///   `grounding::definiendum_words` keys on.
/// - [`ProperNounRun`](NpForcing::ProperNounRun): a collapsed multi-word
///   capitalized proper-noun run ("United States", "New York") — NP primary
///   (a determinerless proper name: Huddleston & Pullum 2002, *The Cambridge
///   Grammar of the English Language*, Ch. 5 §20), the same close-apposition
///   `NP\NP` alternative, AND a common-noun `N` alternative, because a proper
///   noun DOES combine with a preceding determiner ("the United States", "the
///   Netherlands", "the Commonwealth"). CCGbank types proper nouns `N` and
///   promotes them to `NP` (Hockenmaier & Steedman 2007, "CCGbank: A Corpus
///   of CCG Derivations…", *Computational Linguistics* 33(3), §4.2 — the same
///   `N → NP` promotion [`super::supertag_costs::bare_noun_phrase_unary_rule`]
///   already loads); the `N` reading is what lets `the:NP/N` consume the run
///   via `NP/N + N → NP`, instead of the derivation dying on `NP/N + NP` (no
///   Lambek reduction), which was the confirmed root cause of "…of the United
///   States" (1 U.S.C. § 7(b)) failing to parse — a proper-noun-run/determiner
///   collision, NOT a coordination-arity limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NpForcing {
    /// Not forced — typed by the lexicon via [`assign_type`].
    Lexical,
    /// A quoted metalinguistic mention span.
    QuotedMention,
    /// A collapsed multi-word capitalized proper-noun run.
    ProperNounRun,
}

impl NpForcing {
    /// Does this token bypass lexicon typing and take a saturated NP outright?
    /// True for BOTH forcing provenances — the exact predicate the earlier
    /// `force_np: bool` expressed, so the single-primary-type callers read
    /// identically.
    fn forces_np(self) -> bool {
        matches!(self, NpForcing::QuotedMention | NpForcing::ProperNounRun)
    }

    /// Is the token this provenance describes USED or MENTIONED
    /// ([`ExpressionUse`], whose own doc carries the Quine/SEP citation)?
    ///
    /// Exactly ONE of the three provenances is metalinguistic. A collapsed
    /// proper-noun RUN is a USED name — "the United States" refers to the
    /// country, it does not talk about the phrase — so it must NOT come out
    /// mentioned merely because it shares [`forces_np`](Self::forces_np) with
    /// a quoted span; that shared NP-forcing is a SYNTACTIC fact (both take a
    /// saturated NP outright) and this is a SEMANTIC one.
    ///
    /// This is the ONE place the evidence is still available:
    /// [`collapse_quoted_spans`] has already folded the span into a single
    /// token and dropped its quote glyphs, and the typing loop below is the
    /// last stage that still holds the parallel `forcing` row.
    fn expression_use(self) -> ExpressionUse {
        match self {
            NpForcing::QuotedMention => ExpressionUse::Mentioned,
            NpForcing::Lexical | NpForcing::ProperNounRun => ExpressionUse::Used,
        }
    }
}

/// Returns the (possibly shorter) token list and a parallel [`NpForcing`]
/// provenance per token — the union of the incoming `quoted` flag (passed
/// through as [`NpForcing::QuotedMention`]) and newly-detected capitalized
/// proper-noun spans ([`NpForcing::ProperNounRun`]) — so callers can gate
/// typing exactly as they already did for `quoted` alone (via
/// [`NpForcing::forces_np`]), while the alternatives caller additionally reads
/// WHICH provenance to pick the right extra-type inventory.
///
/// [`is_title_case_styled`] gates the WHOLE function: when the input itself
/// is written in Title-Case style, no run is collapsed at all — every
/// capitalized word passes through untouched, its forcing provenance equal to
/// `quoted` alone (a quoted word is [`NpForcing::QuotedMention`], every other
/// [`NpForcing::Lexical`]), exactly as if this detector did not run. This is the
/// piece that closes the false-positive risk [`is_title_case_styled`]'s own
/// doc comment names: the sentence-initial exclusion alone protects only
/// the FIRST word of each sentence, not the many OTHER capitalized-by-style
/// words a Title-Case-styled sentence carries throughout.
///
/// `is_registry_known` is this function's precedence guard against a
/// SEPARATE, richer multi-word-surface recognizer downstream
/// (`collapse_multiword_surfaces` in the chat pipeline, which resolves a
/// span against the FULL composed reasoner — WordNet ⊕ every registered
/// domain lexicon, e.g. a statutory EVV definition or a program name like
/// "Residential Habilitation"). This function runs FIRST, before typing,
/// over raw case alone, so without this guard it would greedily fuse a
/// registry-known span into a bare, gloss-less `proper_noun()` — pre-empting
/// the richer downstream lookup, which never gets a second, already-merged
/// token to re-classify (`collapse_multiword_surfaces` only ever WIDENS
/// adjacent tokens, it never re-splits one). A confirmed real regression on
/// this corpus before this guard existed: "Residential Habilitation",
/// "Shared Living", "Overlap Declaration", and "Health Care Quality" are ALL
/// registered program/agency names whose statutory or program-specific
/// gloss was silently lost once fused into an anonymous NP. Consulted at TWO
/// grains — the run's own maximal joined phrase (so a registry entry keyed
/// on the exact multi-word surface, e.g. "residential habilitation", is
/// respected) AND each individual candidate word (so a registry entry keyed
/// on just one word inside an otherwise-unknown run, e.g. "EVV" inside
/// "Time4Care EVV", is not swallowed by its unknown neighbor). Callers with
/// no richer downstream recognizer to defer to (plain [`tokenize`] and its
/// test callers) pass a closure that always returns `false`, reproducing
/// this function's behavior exactly as it was before this parameter existed.
fn collapse_capitalized_runs(
    words: Vec<String>,
    quoted: Vec<bool>,
    sentence_initial: Vec<bool>,
    is_registry_known: &dyn Fn(&str) -> bool,
) -> (Vec<String>, Vec<NpForcing>, Vec<bool>) {
    if is_title_case_styled(&words, &quoted) {
        // Nothing collapses, but the incoming `quoted` flags still carry
        // their forcing provenance through untouched (an ordinary word is
        // `Lexical`, a quoted mention is `QuotedMention`).
        let forcing = quoted
            .iter()
            .map(|&q| {
                if q {
                    NpForcing::QuotedMention
                } else {
                    NpForcing::Lexical
                }
            })
            .collect();
        return (words, forcing, sentence_initial);
    }
    let can_join_run = |i: usize| {
        !quoted[i]
            && !sentence_initial[i]
            && words[i].chars().next().is_some_and(is_uppercase_latin)
            && !is_registry_known(&words[i].to_lowercase())
    };
    let mut out_words = Vec::with_capacity(words.len());
    let mut out_forcing = Vec::with_capacity(words.len());
    let mut out_sentence_initial = Vec::with_capacity(words.len());
    let mut i = 0;
    while i < words.len() {
        if can_join_run(i) {
            let mut end = i + 1;
            while end < words.len() && can_join_run(end) {
                end += 1;
            }
            let joined = words[i..end].join(" ");
            if end - i >= 2 && !is_registry_known(&joined.to_lowercase()) {
                out_words.push(joined);
                out_forcing.push(NpForcing::ProperNounRun);
                out_sentence_initial.push(sentence_initial[i]);
                i = end;
                continue;
            }
        }
        out_words.push(words[i].clone());
        out_forcing.push(if quoted[i] {
            NpForcing::QuotedMention
        } else {
            NpForcing::Lexical
        });
        out_sentence_initial.push(sentence_initial[i]);
        i += 1;
    }
    (out_words, out_forcing, out_sentence_initial)
}

/// The TOKENIZER-NORMAL FORM of a written surface — the exact word sequence
/// `surface_tokens` emits for it, joined by single spaces. This is the key a
/// multi-word surface OCCURS under once user input has been tokenized: the
/// collapse step ([`collapse_multiword_surfaces`]) matches candidate spans by
/// joining token words with `" "`, so a lexicon surface whose ORTHOGRAPHY the
/// tokenizer alters (an operator glyph splits out of `"80/20 rule"` →
/// `80 / 20 rule`; a trailing paren trims off `"1915(c) waiver"` →
/// `1915(c waiver`) is unreachable under its authored spelling. Indexing the
/// surface ALSO under this normal form (see `ComposedReasoner::new`) makes the
/// match hold BY CONSTRUCTION — both sides pass through the SAME tokenizer —
/// with zero change to how any input tokenizes. The OntoLex-Lemon reading: the
/// normal form is one more *written representation* of the same Form
/// (McCrae et al. 2017, the `ontolex:writtenRep` variant channel), minted
/// mechanically rather than authored.
///
/// ALSO runs `split_possessive_clitics` — the one OTHER token-COUNT-
/// changing step on the real query path
/// ([`tokenize_with_alternatives_registry_aware`]) a bare authored label can
/// actually trigger (a label is plain prose, never a quoted mention or a
/// comma-supplemented/coordinated span, so those steps are moot for a
/// standalone surface; a genitive-marked head noun IS ordinary English and
/// does appear in authored labels). A confirmed real gap without this:
/// "child's insurance benefits" (`hc-e-childs-insurance-benefits`) indexed
/// under its own unsplit spelling while every REAL query joins the
/// post-split tokens with a space before "'s" ("child 's insurance
/// benefits") — a permanent mismatch the lexicon zero-defect gate caught
/// ("what is a child's insurance benefits" abstaining on a loaded term).
/// `quoted`/`sentence_initial` are synthesized all-`false` — a standalone
/// surface has no quoted span to preserve, and `split_possessive_clitics`
/// never reads `sentence_initial` to decide whether to split (only threads
/// it through for the caller's own alignment), so the synthesized values
/// cannot change the split decision.
pub fn tokenizer_normal_form(surface: &str, language: &dyn Language) -> String {
    #[cfg(feature = "std")]
    let vocab = operators::vocabulary();
    #[cfg(not(feature = "std"))]
    let owned_vocab = operators::load();
    #[cfg(not(feature = "std"))]
    let vocab = &owned_vocab;
    #[cfg(feature = "std")]
    let dashes = dash_punctuation::vocabulary();
    #[cfg(not(feature = "std"))]
    let owned_dashes = dash_punctuation::load();
    #[cfg(not(feature = "std"))]
    let dashes = &owned_dashes;
    let words = surface_tokens(surface, vocab, dashes, language);
    let quoted = vec![false; words.len()];
    let sentence_initial = vec![false; words.len()];
    let (words, _, _) = split_possessive_clitics(words, quoted, sentence_initial, language);
    words.join(" ")
}

/// Collapse maximal multi-word SURFACE spans into single proper-noun tokens — the
/// multi-token (phrase / citation) recognition the chat needs so a loaded
/// ontology's multi-word surface (a USC citation "section 1514a", an OWL label, a
/// WordNet collocation "ice cream") resolves as ONE lookup unit instead of
/// splitting into tokens that each miss.
///
/// Greedy longest-match over the LOADED surface set (`classify`, widest window
/// first, up to `max_surface_words`): a matched span becomes one [`TypedToken`]
/// carrying whatever READINGS `classify` returns for it — an ENTITY surface (a
/// citation/label/collocation) → [`proper_noun`](super::types::svo::proper_noun)
/// (the NP slot the copula's `/NP` consumes) plus
/// [`noun`](super::types::svo::noun) (so a determiner attaches: "a cough out"
/// reduces NP/N + N → NP, the same dual reading the loaded-surface push gives
/// single tokens), a RELATIONAL surface ("part of") →
/// [`relational_predicate`](super::types::svo::relational_predicate) (so "is X
/// part of Y" parses). Recognition is DATA-DRIVEN (the surface set + its category
/// are the reasoner's loaded indices), NEVER a baked citation pattern. A
/// `max_surface_words` of 1 (embedded English) makes the window degenerate → a
/// pure no-op, so single-token chat is byte-identical.
///
/// Returns the collapsed tokens AND their per-token Lambek type sets, ready for
/// `chart_reduce` / `interpret`: a collapsed span → its classified readings
/// (primary = the first; the chart explores them all); an uncollapsed token →
/// its own type plus that position's `alternatives` (exactly the set the
/// pipeline built before, so an uncollapsed stream is unchanged).
/// Reconstruct a token window's ORIGINAL written surface — the join
/// `collapse_multiword_surfaces` uses both to probe `classify` and (in the
/// maximal-munch look-ahead) to detect a longer span starting inside the
/// current one. Plain `' '.join` is correct for ordinary word-by-word
/// spacing, but a [`svo_types::genitive_clitic`] token is not a word of its
/// own — `split_possessive_clitics` split it OFF its stem precisely so the
/// stem could resolve through its own lexicon entry, and a clitic attaches
/// to its host with NO intervening space in written English (Huddleston &
/// Pullum 2002, Ch. 5 §16; Zwicky 1977, *On Clitics*). Reintroducing that
/// space is exactly what makes a lexicalized possessive compound ("guard's
/// van", "welder's mask" — real WordNet multi-word lemmas spelled with the
/// clitic attached) unreachable once its clitic has been split off: the
/// reconstructed join would read "guard 's van", which byte-matches nothing
/// in any surface index, so the compound never re-collapses into the single
/// lexical unit its own lemma denotes, and the genitive grammar composes a
/// LIVE (and here spurious) possessive reading instead. Checking the
/// token's own typed [`LambekType`] (not its text) keeps this a structural,
/// loaded-category test — the same idiom every other clitic-aware branch in
/// this module already uses for the synthetic tokens `split_possessive_clitics`
/// and `collapse_medial_comma_adjuncts` mint.
fn join_token_window(tokens: &[TypedToken]) -> String {
    let clitic = svo_types::genitive_clitic();
    let mut joined = String::new();
    for t in tokens {
        if !joined.is_empty() && t.lambek_type != clitic {
            joined.push(' ');
        }
        joined.push_str(&t.word);
    }
    joined
}

/// Is `word` one of the CLOSED English coordinating-conjunction class
/// relevant to a CLAUSE boundary — "and"/"or"/"but" (Huddleston & Pullum
/// 2002, *The Cambridge Grammar of the English Language*, Ch. 15 §2, the
/// three central coordinators)? Deliberately a DIFFERENT (wider) membership
/// than [`is_nominal_coordinator`]'s own "and"/"or"-only scope: that
/// function excludes "but" because a contrastive coordinator does not
/// behave like plain NOMINAL conjunction, but a clause-initial "but" is
/// exactly as capable of opening a second independent clause as "and"/"or"
/// are — the boundary [`crosses_coordinator_wh_boundary`] guards against.
/// Hand-authored, the SAME closed-class rationale as [`is_do_support`]/
/// [`is_modal_auxiliary`]/[`is_nominal_coordinator`] already establish in
/// this file.
fn is_clause_coordinator(word: &str) -> bool {
    matches!(word, "and" | "or" | "but")
}

/// Is `word` one of the CLOSED English interrogative ("wh-") word class —
/// what/who/whom/whose/which/when/where/why/how (Huddleston & Pullum 2002,
/// Ch. 11 — the same wh-adverb/wh-pronoun paradigm `leads_with_wh_adverb`/
/// `leads_with_wh_pronoun` above already consult through the LOADED
/// `InterrogativeAdverb`/`InterrogativePronoun` OLiA classes)? Hand-authored
/// here rather than a `language.lexical_lookup_all` query for the same
/// reason [`is_do_support`]/[`is_modal_auxiliary`] are: this check runs
/// inside [`collapse_multiword_surfaces`], which — unlike `assign_type` —
/// has no `Language` handle in scope (its signature is deliberately
/// `Language`-free so both a WordNet-only and a registry-composed caller
/// share one implementation), so the loaded-class route is not reachable
/// here. English's interrogative-word paradigm is a genuinely closed,
/// enumerable set regardless of which route checks it.
fn is_wh_word(word: &str) -> bool {
    matches!(
        word,
        "what" | "who" | "whom" | "whose" | "which" | "when" | "where" | "why" | "how"
    )
}

/// Does `window` contain a CLAUSE-COORDINATOR immediately followed by a
/// WH-WORD anywhere inside it? [`collapse_multiword_surfaces`]'s maximal-
/// munch matcher has NO clause/coordination-boundary awareness of its own —
/// it only ever asks "does this joined span match a loaded surface" — so a
/// coincidental loaded ADVERB lemma spelled exactly "and how" (a real
/// WordNet `<Lemma writtenForm="and how" partOfSpeech="r"/>`) swallows the
/// coordinator that closes the FIRST clause plus the wh-word that OPENS the
/// second ("What is long-term care insurance and how can it help me?" — two
/// independent interrogative clauses joined by "and", not one adverbial
/// phrase). A coordinator+wh-word bigram is never a genuine constituent of
/// either flanking clause (Huddleston & Pullum 2002 Ch. 15 §2: a
/// coordinator conjoins two LIKE constituents, never begins one), so a
/// window crossing this boundary is never a real lexical span worth
/// probing, independent of what any particular lexicon happens to contain
/// under that spelling. Checked anywhere inside `window`, not just at its
/// own start, so a wider window that happens to CONTAIN the boundary
/// mid-span is excluded too — the SAME "never crosses" discipline, not
/// merely "never starts there".
fn crosses_coordinator_wh_boundary(window: &[TypedToken]) -> bool {
    window
        .windows(2)
        .any(|pair| is_clause_coordinator(&pair[0].word) && is_wh_word(&pair[1].word))
}

/// Probe `classify` for `window`'s known-multiword-surface reading,
/// trying — in order — the plain space-joined surface, then a family of
/// ORTHOGRAPHIC/INFLECTIONAL variants of it, returning the FIRST match
/// (its matched surface text, alongside its readings). Closes two
/// confirmed, unrelated corpus gaps behind the exact same "the span would
/// resolve, but not under the literal spelling the user typed" symptom:
///
/// - HYPHEN-insensitive (`parenthetical_abbreviation` bucket, ~14 corpus
///   rows — e.g. "Home and Community Based Services (HCBS)"): for each
///   internal single space in the joined surface, independently, the same
///   string with just THAT space turned into a hyphen. Compound
///   hyphenation is an ORTHOGRAPHIC choice, not a distinct lexical item
///   (Quirk, Greenbaum, Leech & Svartvik 1985, *A Comprehensive Grammar of
///   the English Language*, §17.106: solid/hyphenated/open spellings of one
///   compound vary freely) — "community based" and "community-based" name
///   the SAME lexeme, but the loaded WordNet lemma is spelled with the
///   hyphen while ordinary prose (and this corpus's questions) often is
///   not.
/// - Head-noun SINGULARIZED (`why_does_x_occur` bucket, ~3 corpus rows —
///   e.g. "GPS exceptions" vs. loaded "GPS exception"): the window's
///   RIGHTMOST word (English compounds are head-final — Huddleston & Pullum
///   2002 Ch. 19 §2 — so the head noun carries the number the loaded
///   surface is spelled under) replaced by each of its
///   [`lemmatize`](crate::cognitive::linguistics::morphology::lemmatizer::lemmatize)
///   candidates, via the loaded AGID-plus-rule-inversion pipeline
///   (Plisson, Lavrac & Mladenic 2004) — never a bare `s`-strip, so
///   irregulars ("children") and `-ies`/`-es` allomorphy resolve exactly
///   like every OTHER lemmatize call site in this codebase.
///
/// Neither transformation invents a new WORD: both recover an
/// orthographic/inflectional VARIANT of the exact span the user wrote, so a
/// caller that finds no match under any variant is exactly as conservative
/// as `classify(&join_token_window(window))` alone was before this existed.
/// Identity is tried FIRST and returned immediately on a hit, so every
/// span that already matched literally (the overwhelming common case)
/// pays no extra `lemmatize`/hyphen-variant cost at all.
fn probe_multiword_surface<F: Fn(&str) -> Option<Vec<LambekType>>>(
    window: &[TypedToken],
    classify: &F,
) -> Option<(String, Vec<LambekType>)> {
    let joined = join_token_window(window);
    if let Some(readings) = classify(&joined) {
        return Some((joined, readings));
    }

    // Hyphen-insensitive: exactly one internal space swapped for a hyphen,
    // tried at each position independently — bounded at `joined`'s own
    // space count, never exponential in window width.
    let chars: Vec<char> = joined.chars().collect();
    for (idx, &c) in chars.iter().enumerate() {
        if c != ' ' {
            continue;
        }
        let variant: String = chars
            .iter()
            .enumerate()
            .map(|(j, &c2)| if j == idx { '-' } else { c2 })
            .collect();
        if let Some(readings) = classify(&variant) {
            return Some((variant, readings));
        }
    }

    // Head-noun singularized: rebuild the window with its last token
    // replaced by each non-identity lemmatize() candidate, rejoined the
    // SAME clitic-aware way `join_token_window` always joins.
    if let Some((head, rest)) = window.split_last() {
        use crate::cognitive::linguistics::morphology::lemmatizer::{
            Language as MorphLanguage, lemmatize,
        };
        let head_lower = head.word.to_lowercase();
        for form in lemmatize(&head_lower, MorphLanguage::English) {
            if form.written_rep == head_lower {
                continue;
            }
            let mut rebuilt: Vec<TypedToken> = rest.to_vec();
            rebuilt.push(TypedToken {
                word: form.written_rep,
                lambek_type: head.lambek_type.clone(),
                // Singularizing the head noun rewrites its SURFACE, never
                // whether that surface was quoted.
                expression_use: head.expression_use,
            });
            let variant = join_token_window(&rebuilt);
            if let Some(readings) = classify(&variant) {
                return Some((variant, readings));
            }
        }
    }

    None
}

pub fn collapse_multiword_surfaces(
    tokens: &[TypedToken],
    alternatives: &[Vec<LambekType>],
    max_surface_words: usize,
    classify: impl Fn(&str) -> Option<Vec<LambekType>>,
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
            let window = &tokens[i..i + w];
            // Never let a collapse span cross a coordinator+wh-word clause
            // boundary — see `crosses_coordinator_wh_boundary`'s own doc for
            // the "and how" corpus regression this closes.
            if crosses_coordinator_wh_boundary(window) {
                continue;
            }
            if let Some((joined, readings)) = probe_multiword_surface(window, &classify) {
                // MAXIMAL MUNCH, longest-match-wins GLOBALLY (Reps 1998,
                // "Maximal-Munch" Tokenization in Linear Time, ACM TOPLAS
                // 20(2) — the longest-lexeme discipline): a span collapses
                // only if no STRICTLY LONGER known span starts INSIDE it.
                // Plain left-greedy matching would let an earlier, shorter
                // surface swallow the first words of a longer term whose
                // match starts one token later ("a level" — the WordNet
                // A-level collocation — consuming the "level" of "level of
                // care determination"). Deferring emits the current token
                // uncollapsed; the longer span then collapses when the scan
                // reaches its own start. The inner probe applies the exact
                // SAME coordinator/wh-boundary exclusion and hyphen/lemma
                // variant tolerance as the outer one, so a longer inner
                // span is only ever counted as blocking when it is itself a
                // span this function would actually collapse.
                let blocked = (i + 1..i + w).any(|j| {
                    let upper_j = (tokens.len() - j).min(max_window);
                    (w + 1..=upper_j).any(|wj| {
                        let inner = &tokens[j..j + wj];
                        !crosses_coordinator_wh_boundary(inner)
                            && probe_multiword_surface(inner, &classify).is_some()
                    })
                });
                if blocked {
                    continue;
                }
                let primary = readings
                    .first()
                    .expect("a classified surface carries at least one reading")
                    .clone();
                out_tokens.push(TypedToken {
                    word: joined,
                    lambek_type: primary,
                    // A collapsed multi-word surface is mentioned iff any
                    // token it swallowed was: quotation scopes over a SPAN,
                    // so a mention cannot be half-used (Cappelen & Lepore,
                    // "Quotation", SEP §3.1 — the quoted expression is the
                    // whole enclosed string).
                    expression_use: if window
                        .iter()
                        .any(|t| t.expression_use == ExpressionUse::Mentioned)
                    {
                        ExpressionUse::Mentioned
                    } else {
                        ExpressionUse::Used
                    },
                });
                out_types.push(readings);
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
///
/// No richer downstream multi-word recognizer to defer to here — the
/// capitalized-run detector runs with an always-false registry-known oracle
/// (see [`tokenize_with_alternatives_registry_aware`] for the chat pipeline's
/// registry-aware caller).
pub fn tokenize_with_alternatives(
    text: &str,
    language: &dyn Language,
) -> (Vec<TypedToken>, Vec<Vec<LambekType>>) {
    // `1`: the always-false oracle below knows no registry surfaces at all,
    // matching `LexicalReasoner::max_surface_words`'s own default-`1`
    // convention for "no composed registry" exactly — see
    // `tokenize_with_alternatives_registry_aware`'s own `max_registry_
    // surface_words` parameter doc.
    tokenize_with_alternatives_registry_aware(text, language, &|_| false, 1)
}

/// [`tokenize_with_alternatives`], additionally deferring capitalized-run
/// collapsing to `is_registry_known` — see `collapse_capitalized_runs`'s
/// doc comment for why the chat pipeline needs this: without it, a
/// registered domain term ("Residential Habilitation", "EVV") that happens
/// to be capitalized gets fused into a gloss-less NP before
/// `collapse_multiword_surfaces` (the reasoner-aware recognizer downstream)
/// ever gets a chance to resolve it.
pub fn tokenize_with_alternatives_registry_aware(
    text: &str,
    language: &dyn Language,
    is_registry_known: &dyn Fn(&str) -> bool,
    // The composed reasoner's own longest registered surface (in
    // whitespace-separated words) — `reasoner.max_surface_words()`
    // (`LexicalReasoner::max_surface_words`) at every REAL registry-backed
    // call site (the chat pipeline), `1` when `is_registry_known` is the
    // trivial always-false oracle. Bounds `multiword_surface_spans`'s
    // window search (via `correct_unknown_word_surfaces`) so it never scans
    // past the longest surface EITHER `language` or the registry can
    // actually recognize — see that function's own doc for the real,
    // measured O(n²)-to-O(n³) cost this closes.
    max_registry_surface_words: usize,
) -> (Vec<TypedToken>, Vec<Vec<LambekType>>) {
    #[cfg(feature = "std")]
    let vocab = operators::vocabulary();
    #[cfg(not(feature = "std"))]
    let owned_vocab = operators::load();
    #[cfg(not(feature = "std"))]
    let vocab = &owned_vocab;
    #[cfg(feature = "std")]
    let quotes = quote_glyphs::vocabulary();
    #[cfg(not(feature = "std"))]
    let owned_quotes = quote_glyphs::load();
    #[cfg(not(feature = "std"))]
    let quotes = &owned_quotes;
    #[cfg(feature = "std")]
    let dashes = dash_punctuation::vocabulary();
    #[cfg(not(feature = "std"))]
    let owned_dashes = dash_punctuation::load();
    #[cfg(not(feature = "std"))]
    let dashes = &owned_dashes;
    let (words0, sentence_initial0, comma_after0, semicolon_after0) =
        surface_tokens_with_sentence_bounds(text, vocab, dashes, language);
    let (words0, sentence_initial0) = collapse_medial_comma_adjuncts(
        words0,
        sentence_initial0,
        comma_after0,
        semicolon_after0,
        language,
    );
    let (words, quoted, sentence_initial) =
        collapse_quoted_spans(words0, sentence_initial0, quotes);
    // Defines-lens gap G5: "and"/"or" coordinating two quoted definienda
    // (directly, or bridging a repeated "the term(s)" prefix) is a
    // coordinated-definienda marker, not an ordinary coordinator — see
    // `mark_apposition_coordinators`'s own doc.
    let (words, quoted, sentence_initial) =
        mark_apposition_coordinators(words, quoted, sentence_initial);
    let (words, quoted, sentence_initial) =
        split_possessive_clitics(words, quoted, sentence_initial, language);
    let words = correct_unknown_word_surfaces(
        words,
        &quoted,
        vocab,
        language,
        is_registry_known,
        max_registry_surface_words,
    );
    let (words, forcing, sentence_initial) =
        collapse_capitalized_runs(words, quoted, sentence_initial, is_registry_known);

    // The token that heads the INTERROGATIVE clause — ordinarily token 0,
    // but when the sentence LEADS with a fronted scope-setting adjunct
    // (`is_fronted_scope_adjunct_head`, G1: "In self-direction, WHAT is an
    // authorized representative?"), the adjunct itself occupies token 0 and
    // the real clause head is the next widened-`sentence_initial` position
    // `mark_clause_initial_after_fronted_adjunct` already marked (the token
    // right after the adjunct's own comma boundary — see that function's
    // doc for the citation and the confirmed defect this closes: every
    // `words.first()`/`i == 0` check below previously looked at the
    // ADJUNCT, never the clause it introduces).
    let clause_head_idx = if words
        .first()
        .is_some_and(|w| is_fronted_scope_adjunct_head(&w.to_lowercase()))
    {
        sentence_initial
            .iter()
            .enumerate()
            .skip(1)
            .find(|&(_, &flag)| flag)
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    } else {
        0
    };
    let clause_head_word = words.get(clause_head_idx);

    // A fronted interrogative ADVERB (where/when/why/how) licenses subject-aux
    // inversion, so the copula can derive S[q]/PP (question_copula_pp) and the
    // wh-adverb category reduces (Huddleston & Pullum 2002 Ch.11).
    let leads_with_wh_adverb = clause_head_word.is_some_and(|w| {
        language
            .lexical_lookup_all(&w.to_lowercase())
            .iter()
            .any(|e| e.olia_class() == Some("InterrogativeAdverb"))
    });

    // A fronted interrogative PRONOUN ("what") licenses the same subject-aux
    // inversion for an OBJECT gap, so do-support can derive the bare-stem-VP
    // object-question category (Slice B: "what does X mean").
    let leads_with_wh_pronoun = clause_head_word.is_some_and(|w| {
        language
            .lexical_lookup_all(&w.to_lowercase())
            .iter()
            .any(|e| e.olia_class() == Some("InterrogativePronoun"))
    });

    // A fronted MODAL AUXILIARY ("can"/"should"/"must"/…) licenses the same
    // subject-aux inversion for a bare-stem-VP complement, so the modal can
    // derive `modal_question` ("can an agency opt-out?") and the following
    // verb can offer its bare-stem reading. Gated on the closed 9-item
    // Huddleston & Pullum (2002) Ch. 3 §9 modal class via `is_modal_auxiliary`
    // (a hand-authored membership check, not loaded data — the SAME
    // rationale as `is_do_support`: the loaded OLiA subclasses do not
    // isolate modals from other auxiliaries that select a DIFFERENT
    // complement shape).
    let leads_with_modal = clause_head_word.is_some_and(|w| is_modal_auxiliary(&w.to_lowercase()));

    // A CLAUSE-INITIAL do-support auxiliary ("do"/"does"/"did") licenses the
    // SAME subject-aux inversion for a bare-stem-VP complement as a modal
    // does — Huddleston & Pullum (2002) Ch. 3 §7-9: do-support and modal
    // auxiliaries share the identical subject-aux-inversion-selects-bare-VP-
    // complement structure. Previously this reading was only offered under
    // `leads_with_wh_pronoun` (the object-question "what does X mean" case,
    // Slice B) — leaving a genuine, general polar (yes/no) do-support
    // question ("Does Medicaid cover hospice?", "Does EVV training count
    // for continuing education?") without any bare-stem-VP-selecting
    // reading for its clause-initial "does" at all, even though
    // `modal_question`'s own type shape is already generic over which
    // token carries the auxiliary. `leads_with_do_support` closes that half
    // by mirroring `leads_with_modal` exactly, one clause below and in both
    // bare-stem-offering blocks further down.
    let leads_with_do_support = clause_head_word.is_some_and(|w| is_do_support(&w.to_lowercase()));

    // Does this CLAUSE contain a copula token ("is"/"are"/"was"/"were")
    // anywhere — licensing an adjacent -ing-form verb's PROGRESSIVE-
    // participle reading (`progressive_transitive_verb`/
    // `progressive_intransitive_verb`, `NP\S[ng]`) the SAME way
    // `leads_with_modal`/`leads_with_do_support` license a bare-stem
    // reading? Unlike modal/do-support (clause-HEAD-scoped, since a bare
    // auxiliary elsewhere would over-broadcast into an unrelated embedded
    // clause), a copula's own progressive-selecting behavior does not
    // depend on clause position — the SAME position-independence
    // `copula_adj`'s own unconditional offering below already relies on
    // (`primary_type == svo_types::copula()`, checked per-token, no clause
    // scoping needed since `is_copula()` is a LOADED per-word class, not an
    // ambiguous auxiliary). Hockenmaier & Steedman (2007), *Computational
    // Linguistics* 33(3), §6.3.1 p.379, dependency (43): the corpus-attested
    // `is,(S[dcl]\NP)/(S[ng]\NP)` category this construction generalizes
    // (declarative, direct-polar-question, and wh-fronted-question) all
    // select the identical `S[ng]`-participle complement.
    let sentence_has_copula = words.iter().any(|w| {
        language
            .lexical_lookup_all(&w.to_lowercase())
            .iter()
            .any(|e| e.pos_tag().is_copula())
    });

    let mut tokens = Vec::new();
    let mut alternatives = Vec::new();

    for (i, word) in words.iter().enumerate() {
        let lower = word.to_lowercase();

        // A reserved apposition-coordinator marker (defines-lens gap G5) has
        // no lexicon entry of its own — the SAME synthetic-token rationale
        // as the medial-supplement/list-coordinator markers: its category
        // comes directly from which marker minted it.
        if is_apposition_coordinator_marker(&lower) {
            tokens.push(TypedToken {
                word: lower,
                lambek_type: svo_types::nominal_coordinator_apposition(),
                // The marker is a SYNTHETIC coordinator this tokenizer minted,
                // never a surface anyone quoted.
                expression_use: ExpressionUse::Used,
            });
            alternatives.push(Vec::new());
            continue;
        }

        // A quote-collapsed mention: the whole span types NP outright (Slice
        // B) — it never goes through lexicon-driven typing, since a MENTIONED
        // expression's normal category (adverb, adjective, whatever "deadly"
        // would otherwise be) is not the reading a quoted argument needs. It
        // ALSO offers close apposition (NP\NP, `svo_types::close_apposition`)
        // as an ADDITIVE alternative — the postnominal-definiendum reading
        // "the term 'X'" needs — so the chart keeps whichever reading
        // derives a complete parse; the bare-NP mention reading stays
        // primary and unaffected.
        if forcing[i].forces_np() {
            tokens.push(TypedToken {
                word: lower,
                lambek_type: svo_types::proper_noun(),
                // The ONE surviving carrier of the quote glyphs this stage
                // has already discarded — see `NpForcing::expression_use`.
                expression_use: forcing[i].expression_use(),
            });
            // Both provenances take NP primary + the close-apposition NP\NP
            // the "the term 'X'"/postnominal reading needs. A collapsed
            // proper-noun RUN additionally offers the common-noun N reading a
            // preceding determiner consumes ("the United States"): CCGbank's
            // N-typing of proper nouns (see `NpForcing::ProperNounRun`'s own
            // doc for the citation and the exact `NP/N + NP`-has-no-reduction
            // defect this closes). A quoted MENTION deliberately does NOT get
            // it — an N reading there would admit a spurious N/N-compound
            // bracketing of "the term 'X'" that erases the apposition
            // `grounding::definiendum_words` keys on.
            let mut mention_alts = vec![svo_types::close_apposition()];
            if forcing[i] == NpForcing::ProperNounRun {
                mention_alts.push(svo_types::noun());
            }
            alternatives.push(mention_alts);
            continue;
        }

        // Get ALL entries from the language
        let all_entries = language.lexical_lookup_all(&lower);

        // Primary type assignment
        let primary_type = assign_type(&lower, sentence_initial[i], language, vocab);

        // Alternative types from all entries — EVERY loaded category row of
        // every entry (the loader's multi-row contract; e.g. the Adjective
        // class carries both the attributive N/N and the predicative
        // NP\S[adj] rows), not just each entry's first row. This is how the
        // predicative reading reaches a verb-first adjective homonym ("shy",
        // "backed") whose adjective entry is not the primary.
        let mut alt_types: Vec<LambekType> = Vec::new();
        for entry in &all_entries {
            for t in entry_categories(entry) {
                if t != primary_type && !alt_types.contains(&t) {
                    alt_types.push(t);
                }
            }
        }

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

            // The SAME fronted-wh-adverb inversion ALSO licenses the
            // PROGRESSIVE-complement question reading `progressive_question_copula`
            // (`(S[q]/(NP\S[ng]))/NP`) — "IS" in "why IS Illinois implementing
            // EVV?" (the copula sits medially, right after the fronted "why",
            // never at `clause_head_idx` itself since "why" occupies that
            // slot) reduces the inverted clause the following `-ing` verb's
            // `progressive_transitive_verb`/`progressive_intransitive_verb`
            // then saturates. Hockenmaier & Steedman (2007), *Computational
            // Linguistics* 33(3), §6.3.1 p.379, dependency (43).
            let pqc = svo_types::progressive_question_copula();
            if pqc != primary_type && !alt_types.contains(&pqc) {
                alt_types.push(pqc);
            }
        }

        // The manner wh-adverb "how" (used INTRANSITIVELY, "how does X
        // work?" — no PP anywhere in the clause for `wh_adverb`'s `S[q]/PP`
        // gap to satisfy) ALSO offers the complete-clause-adjunct reading
        // `wh_manner_adverb` (`S[wq]/S[q]`), alongside its OLiA-derived
        // `wh_adverb` (`S[wq]/(S[q]/PP)`) primary. The loaded OLiA
        // `InterrogativeAdverb` class covers "how"/"why"/"where"/"when"
        // uniformly, but the LOADED [`WhAdverbRole`] feature (Cysouw 2004
        // §3.2 table (9), decoded from each word's own `english.xml` synset)
        // distinguishes them — a query over loaded data, not a hand-authored
        // membership check. Position-independent (unlike the clause-head-
        // scoped `leads_with_wh_adverb` block just above): "how" can occur
        // mid-sentence in a compound question ("What is X, and how does it
        // work?"), the same rationale the "and"/"or" coordination blocks
        // below use.
        let wh_role = |w: &str| {
            language
                .lexical_lookup_all(w)
                .iter()
                .find_map(|e| e.wh_adverb_role())
        };
        if wh_role(&lower) == Some(WhAdverbRole::Manner) {
            let wma = svo_types::wh_manner_adverb();
            if wma != primary_type && !alt_types.contains(&wma) {
                alt_types.push(wma);
            }
        }

        // The reason wh-adverb "why" (used INTRANSITIVELY over a COMPLETE
        // inverted clause, "why is Illinois implementing EVV?" — no PP gap
        // anywhere in the clause for `wh_adverb`'s `S[q]/PP` gap to satisfy)
        // ALSO offers the complete-clause-adjunct reading `wh_reason_adverb`
        // (`S[wq]/S[q]`), alongside its OLiA-derived `wh_adverb`
        // (`S[wq]/(S[q]/PP)`) primary — the SAME loaded [`WhAdverbRole`]
        // pattern as the manner block immediately above. Position-
        // independent for the SAME reason the manner block is.
        if wh_role(&lower) == Some(WhAdverbRole::Reason) {
            let wra = svo_types::wh_reason_adverb();
            if wra != primary_type && !alt_types.contains(&wra) {
                alt_types.push(wra);
            }
        }

        // Do-support ("do"/"does"/"did") IMMEDIATELY preceded by manner
        // "how" ALSO offers `modal_question` (`(S[q]/(NP\S[b]))/NP`) — the
        // SAME auxiliary-selects-bare-VP-under-inversion pattern
        // `modal_question`'s own doc already documents as general, not
        // modal-specific (Steedman 2000, *The Syntactic Process*;
        // Huddleston & Pullum 2002 Ch.3 §7-9), reused rather than
        // duplicated so "does" in "how does it work?" combines with the
        // subject NP and a bare-stem VP (`intransitive_verb`'s `S(None)`
        // wildcard unifies with the `S[b]` slot via `types_match`) to
        // derive `S[q]`, which `wh_manner_adverb` then adjoins. Gated on
        // LOCAL adjacency to the immediately preceding token — narrower
        // than `leads_with_wh_adverb`'s clause-head scoping — so this never
        // over-broadcasts into an unrelated declarative "does" elsewhere in
        // the sentence.
        if is_do_support(&lower)
            && i > 0
            && wh_role(&words[i - 1].to_lowercase()) == Some(WhAdverbRole::Manner)
        {
            let mq = svo_types::modal_question();
            if mq != primary_type && !alt_types.contains(&mq) {
                alt_types.push(mq);
            }
        }

        // A sentence-initial question copula (`(S[q]/NP)/NP`) ALSO offers the
        // predicative-complement reading `(S[q]/(S[adj]\NP))/NP`, so "is X part of
        // Y" reduces (the relational predicate "part of Y" is the predicative
        // complement). Additive — "is X a Y" still reduces via the NP-complement
        // `question_copula`; the chart keeps whichever reading derives `S[q]`.
        if primary_type == svo_types::question_copula() {
            let qcp = svo_types::question_copula_pred();
            if !alt_types.contains(&qcp) {
                alt_types.push(qcp);
            }

            // Same additive pattern for the PROGRESSIVE-complement question
            // reading — "Is Illinois implementing EVV?" (a genuine
            // clause-initial polar question, no wh-adverb fronting). Additive:
            // "is X a Y" still reduces via the NP-complement `question_copula`.
            let pqc = svo_types::progressive_question_copula();
            if !alt_types.contains(&pqc) {
                alt_types.push(pqc);
            }
        }

        // A MEDIAL copula ALSO offers the predicative-complement reading
        // `copula_adj` (CCGbank (S[dcl]\NP)/(S[adj]\NP), Hockenmaier &
        // Steedman 2007 §3.4) as a chart alternative — the same additive
        // pattern as the question copula above. This replaces the deleted
        // destructive copula-adjacency rewrite, which retyped the token pair
        // only when the NEXT token's PRIMARY type was the attributive
        // adjective — so "what is able" worked but the verb-first homonym
        // "what is shy" never received the reading and parsed degenerately.
        if primary_type == svo_types::copula() {
            let ca = svo_types::copula_adj();
            if !alt_types.contains(&ca) {
                alt_types.push(ca);
            }

            // Same additive pattern for the PROGRESSIVE-complement declarative
            // reading `progressive_copula` (`(NP\S[dcl])/(NP\S[ng])`) — "is" in
            // "Illinois is implementing EVV" (declarative, not a question).
            // Hockenmaier & Steedman (2007) §6.3.1 p.379, dependency (43).
            let pc = svo_types::progressive_copula();
            if !alt_types.contains(&pc) {
                alt_types.push(pc);
            }
        }

        // The object-question "what" (subject-question is the PRIMARY
        // reading via the loaded OLiA→CCG functor) ALSO offers the
        // OBJECT-gap category `S[wq]/(S[q]/NP)`, so "what does X mean"
        // reduces alongside "what is a dog" — additive, never overwriting
        // the subject-question primary (Slice B,
        // `.notes/chat-fix-c-build-state.md`).
        if primary_type == svo_types::wh_what() {
            let wo = svo_types::wh_what_object();
            if !alt_types.contains(&wo) {
                alt_types.push(wo);
            }
        }

        // Under wh-pronoun fronting (object-question "what"), do-support
        // ALSO offers the bare-stem-VP-selecting object-question category,
        // so "what does X mean" reduces. Do-support — "do"/"does"/"did" —
        // is a closed, complete grammatical class (Huddleston & Pullum 2002
        // *The Cambridge Grammar of the English Language* Ch.3 §7-8), not a
        // loaded lexicon distinction: the loaded OLiA subclasses
        // (StrictAuxiliaryVerb, ModalVerb) do not isolate it from
        // have/has/had, which select a participle, not a bare stem — see
        // `svo::does_support`'s own doc comment for why this is a code
        // constructor, matching `question_copula_pp`'s precedent, rather
        // than a shared-key TSV row.
        if leads_with_wh_pronoun && is_do_support(&lower) {
            let ds = svo_types::does_support();
            if ds != primary_type && !alt_types.contains(&ds) {
                alt_types.push(ds);
            }
        }

        // A clause-initial modal auxiliary ALSO offers `modal_question`
        // (`(S[q]/(NP\S[b]))/NP`) — the clause-initial branch of `assign_type`
        // unconditionally types every `Copula | Auxiliary` word
        // `question_copula` (the two-NP shape), which never reduces for a
        // modal's bare-stem-VP complement ("can an agency opt-out?"), so
        // this is additive alongside that primary reading, never replacing
        // it — the chart keeps whichever derives `S[q]`. `clause_head_idx`,
        // not raw `i == 0`, so a modal after a fronted scope adjunct ("In
        // self-direction, CAN an agency opt-out?") is covered too.
        if i == clause_head_idx && is_modal_auxiliary(&lower) {
            let mq = svo_types::modal_question();
            if mq != primary_type && !alt_types.contains(&mq) {
                alt_types.push(mq);
            }
        }

        // A clause-initial do-support auxiliary ALSO offers `modal_question`
        // — the SAME additive reading the modal block immediately above
        // offers, for the SAME reason (`leads_with_do_support`'s own doc
        // comment has the citation): `assign_type`'s clause-initial branch
        // types every `Copula | Auxiliary` word (including "do"/"does"/
        // "did") `question_copula` (the two-NP shape), which never reduces
        // for a bare-stem-VP complement. This is what lets a plain polar
        // do-support question ("Does Medicaid cover hospice?") derive a
        // genuine `S[q]` instead of falling through to a spurious
        // `SuggestInterpretation` fallback that only LOOKED like a parse.
        if i == clause_head_idx && is_do_support(&lower) {
            let mq = svo_types::modal_question();
            if mq != primary_type && !alt_types.contains(&mq) {
                alt_types.push(mq);
            }
        }

        // "and"/"or" ALSO offer the two evidenced nominal-coordination
        // readings — `(NP\NP)/NP` and `(N\N)/N` — alongside their loaded
        // Conjunction→N primary (unaffected: this is purely additive, and
        // the primary reading still lets a non-nominal "and" pass through
        // untouched exactly as before). Position-independent (unlike
        // modal/do-support): coordination can occur anywhere in a
        // sentence, not just sentence-initially. A list-coordinator marker
        // (defines-lens gap G4(a)) offers the SAME additive N-level
        // alternative alongside its own NP-level primary.
        if is_nominal_coordinator(&lower) || is_list_coordinator_marker(&lower) {
            let ncnp = svo_types::nominal_coordinator_np();
            if ncnp != primary_type && !alt_types.contains(&ncnp) {
                alt_types.push(ncnp);
            }
            let ncn = svo_types::nominal_coordinator_n();
            if ncn != primary_type && !alt_types.contains(&ncn) {
                alt_types.push(ncn);
            }
        }

        // A literal "and"/"or" surface ALSO offers the sentential
        // wh-question coordinator `(S[wq]\S[wq])/S[wq]` — "What Is
        // Medicaid, and Who Is Eligible?" — alongside its NP/N-level
        // coordination readings above. Additive, and scoped to the literal
        // surface only (never the synthetic list-coordinator marker, which
        // carries no real "and"/"or" word for `montague::apply`'s
        // surface-level dispatch to key on): `svo_types::
        // sentential_coordinator_wq`'s own doc has the citation and the
        // shape-collision check. Position-independent, matching the
        // NP/N-level readings just above: the chart's own interrogative-
        // preferring tiered goal selection is what keeps this reading from
        // spuriously winning when neither conjunct actually derives
        // `S[wq]`.
        if is_nominal_coordinator(&lower) {
            let scwq = svo_types::sentential_coordinator_wq();
            if scwq != primary_type && !alt_types.contains(&scwq) {
                alt_types.push(scwq);
            }
        }

        // A literal "and"/"or" surface, OR a synthetic list-coordinator
        // marker (`is_list_coordinator_marker` —
        // `find_list_coordinator_commas`'s own N-ary list machinery), ALSO
        // offers the transitive-verb coordinator
        // `((NP\S)/NP\(NP\S)/NP)/((NP\S)/NP)` — "negotiates and enters into
        // [a qualifying non-binding instrument]" (1 U.S.C. § 112b(k)(2),
        // inside a subject relative clause), the object-sharing
        // right-node-raising reading — alongside its NP/N/wh-level readings
        // above. Additive, mirroring the NP/N-level coordinator block's OWN
        // gate just above (`is_nominal_coordinator(&lower) ||
        // is_list_coordinator_marker(&lower)`) rather than
        // `sentential_coordinator_wq`'s literal-only scoping: unlike a
        // wh-question coordination (never evidenced with a 3+-item list in
        // this corpus), an N-ary COORDINATED-VERB list is a real, attested
        // statutory-drafting shape ("drafts, negotiates, and enters into
        // ..."), so restricting to the literal final "and"/"or" alone would
        // leave every 3+-way verb coordination unreachable — confirmed a
        // real, measured gap this session
        // (`probe_the_transitive_verb_coordinator_fix_generalizes`,
        // `crates/praxis-corpus-tests/tests/scratch_probe.rs`: a 3-way
        // "drafts, negotiates, and enters into" produced ZERO pointers with
        // only the literal-surface gate, and a correct one once this marker
        // gate was added). A coordinator (literal or marker) flanked by
        // anything other than two transitive verbs never reduces via this
        // category, so the chart falls through to whichever reading does.
        // `svo_types::transitive_verb_coordinator`'s own doc has the
        // citation and the shape-collision check (distinct from every other
        // coordinator here — it reduces to `TV`, not an NP/N/S[wq] family).
        if is_nominal_coordinator(&lower) || is_list_coordinator_marker(&lower) {
            let tvc = svo_types::transitive_verb_coordinator();
            if tvc != primary_type && !alt_types.contains(&tvc) {
                alt_types.push(tvc);
            }
        }

        // An ORDINARY preposition ALSO offers the prepositional-verb
        // particle reading `((NP\S)/NP)\((NP\S)/NP)` — "into" in "enters
        // INTO a qualifying non-binding instrument" (1 U.S.C. § 112b(k)(2)),
        // alongside its ordinary adjunct `(NP\NP)/NP` primary
        // ([`svo_types::preposition`]). Additive and gated on the derived
        // TYPE shape (`primary_type == svo_types::preposition()`, the SAME
        // "gate on the type this word's assignment already produced" idiom
        // `copula_adj`/`progressive_copula` use just below for the copula),
        // not the literal surface: `svo_types::transitive_verb_particle`'s
        // own doc has the full citation and explains why this is offered to
        // every preposition rather than a hand-authored closed list (no
        // loaded per-verb subcategorization lexicon exists to restrict it
        // further). A preposition that never sits directly after a
        // transitive-verb-shaped functor simply never reduces via this
        // category, so the chart falls through to the ordinary adjunct
        // reading — no destructive removal needed.
        if primary_type == svo_types::preposition() {
            let tvp = svo_types::transitive_verb_particle();
            if tvp != primary_type && !alt_types.contains(&tvp) {
                alt_types.push(tvp);
            }
        }

        // A SENTENCE-INITIAL fronted scope-setting preposition ("for"/"in")
        // ALSO offers the scope-adjunct category `(S/S)/NP` -- "For purposes
        // of this subsection," / "In this subsection," -- alongside its
        // ordinary preposition() primary reading. Additive, and gated on
        // position: a MEDIAL "for"/"in" (e.g. "for the purposes of X" inside
        // "except for the purposes of X") keeps its plain preposition
        // reading only, unaffected.
        if i == 0 && is_fronted_scope_adjunct_np_head(&lower) {
            let fsa = svo_types::fronted_scope_adjunct_np();
            if fsa != primary_type && !alt_types.contains(&fsa) {
                alt_types.push(fsa);
            }
        }

        // A SENTENCE-INITIAL fronted scope-setting head that selects an
        // ALREADY-derived NP\NP ("PP") complement ("except"/"subject") ALSO
        // offers `(S/S)/(NP\NP)` -- "Except for the purposes of X," /
        // "Subject to Y,". Additive and position-gated, mirroring the
        // NP-complement block above exactly.
        if i == 0 && is_fronted_scope_adjunct_pp_head(&lower) {
            let fsa = svo_types::fronted_scope_adjunct_pp();
            if fsa != primary_type && !alt_types.contains(&fsa) {
                alt_types.push(fsa);
            }
        }

        // A DEVERBAL PARTICIPIAL ADJECTIVE — a word independently attested in
        // WordNet as BOTH an Adjective AND a Verb ("required", "expected",
        // "supposed", "needed", "ordered", "authorized" — confirmed live;
        // NOT "asked"/"allowed"/"instructed"/"told", which carry no WordNet
        // Adjective sense and so never reach [`svo_types::predicate_adjective`]
        // in the first place) ALSO offers the catenative-infinitival-
        // predicate reading `(NP\S[adj])/(NP\S[to])` — "required" in
        // "services are required to use EVV" — alongside its ordinary
        // Adjective-class primary/alternative rows. Data-driven gate (no
        // hand-listed word set): reuses the SAME `all_entries` this loop
        // already fetched from `language.lexical_lookup_all`, never a
        // literal surface match. Additive and position-independent, the same
        // pattern [`is_nominal_coordinator`]'s block above uses — a plain
        // bare-adjective reading ("Is a report required?", no infinitival
        // complement) still reduces via the existing
        // [`svo_types::predicate_adjective`] row, untouched.
        if all_entries.iter().any(|e| e.pos_tag() == PosTag::Adjective)
            && all_entries.iter().any(|e| e.pos_tag() == PosTag::Verb)
        {
            let cip = svo_types::catenative_infinitival_predicate();
            if cip != primary_type && !alt_types.contains(&cip) {
                alt_types.push(cip);
            }
        }

        // An OUT-OF-VOCABULARY word (no lexicon entry; assign_type's
        // open-class default typed it a bare noun N) ALSO offers the
        // saturated proper-noun NP reading: an OOV nominal filling an
        // argument slot is most plausibly a proper name or unregistered
        // term mention ("what is flurbogast", "what is Kubernetes") — the
        // proper-name NP analysis of Huddleston & Pullum (2002) Ch. 5 §20
        // (proper names are determinerless NPs), and the standard OOV
        // treatment. Additive: the copula's /NP complement can consume the
        // saturated reading where bare N cannot reduce, the chart keeps
        // whichever derives S, and every in-vocabulary word is untouched —
        // so an unresolvable-but-parseable question reaches the honest
        // vocabulary-gap abstention instead of a degenerate parse failure.
        if all_entries.is_empty() && primary_type == svo_types::noun() {
            let np = svo_types::proper_noun();
            if !alt_types.contains(&np) {
                alt_types.push(np);
            }

            // An OOV word already assumed to be a Noun (this same gate) is ALSO
            // offered the nominal-premodifier reading N/N, via the SAME loaded
            // "Noun" class projection `category_projection::categories_for_class`
            // uses for a real lexicon Noun entry — not a separate hardcoded
            // `svo_types::nominal_modifier_noun()` call, so the OOV path can never
            // drift from whatever rows the Noun class TSV carries (single source
            // of truth with task #29's second Noun row). CCGbank's own convention
            // (Hockenmaier & Steedman 2005, CCGbank User's Manual §3.6.1/§3.6.2)
            // draws no lexicon-membership distinction — Levi (1978)/Selkirk (1982)/
            // Huddleston & Pullum (2002) Ch.19's account of English N-N compounds
            // (already `nominal_modifier_noun`'s own citation set) does not
            // restrict the modifier slot to dictionary-attested common nouns
            // either. Additive alongside `noun()` (primary) and `proper_noun()`
            // (above) — the chart decides which reading actually derives a
            // complete parse, the same arbitration every other alternative in
            // this loop already relies on.
            //
            // MEASURED NET-NEGATIVE against the STALE (pre-Gap-A,
            // pre-G1-G7-grammar) compact_defines_signatures overlay pins
            // (Green -2, OverAnswered +1, UnparsedKnownTerm +1, 2026-07-20) —
            // restored uncommitted pending a fresh compute_defines_overlay
            // regeneration + re-measurement, since defines_lens is wired into
            // the live chat pipeline and a stale overlay directly confounds
            // this measurement. Do not re-run the monotonic-or-nothing
            // acceptance check against caregiver_capability_ratchet until the
            // overlay has been regenerated against the current grammar.
            for t in category_projection::categories_for_class("Noun") {
                if t != primary_type && !alt_types.contains(&t) {
                    alt_types.push(t);
                }
            }
        }

        // Under the SAME wh-pronoun fronting, an ordinary transitive verb
        // ALSO offers its bare-stem-VP reading `(S[b]\NP)/NP` — the
        // complement do-support selects for ("what does X mean"). Additive:
        // the verb's normal finite reading `transitive_verb()` (used
        // everywhere else, e.g. "she means well") stays primary; only the
        // do-support-selected bare-stem alternative is offered, and only
        // when a do-support/object-wh context exists for it to combine
        // with.
        if leads_with_wh_pronoun && primary_type == svo_types::transitive_verb() {
            let btv = svo_types::bare_transitive_verb();
            if !alt_types.contains(&btv) {
                alt_types.push(btv);
            }
        }

        // Under a SENTENCE-INITIAL MODAL OR do-support auxiliary, a
        // transitive verb ALSO offers its bare-stem-VP reading
        // `(NP\S[b])/NP` (mirroring the wh-pronoun block above exactly,
        // gated on `leads_with_modal`/`leads_with_do_support` instead of
        // `leads_with_wh_pronoun`) — "take" in "can Medicaid take a
        // house?", "cover" in "does Medicaid cover hospice?" — and an
        // intransitive verb offers `bare_intransitive_verb` (`NP\S[b]`)
        // directly — "opt-out" in "can an agency opt-out?",
        // `modal_question`'s complement slot. Additive: the verb's normal
        // finite reading stays primary. `leads_with_do_support` mirrors
        // `leads_with_modal` per Huddleston & Pullum (2002) Ch.3 §7-9's
        // shared subject-aux-inversion structure (see its own doc comment).
        if (leads_with_modal || leads_with_do_support)
            && primary_type == svo_types::transitive_verb()
        {
            let btv = svo_types::bare_transitive_verb();
            if !alt_types.contains(&btv) {
                alt_types.push(btv);
            }
        }
        if (leads_with_modal || leads_with_do_support)
            && primary_type == svo_types::intransitive_verb()
        {
            let biv = svo_types::bare_intransitive_verb();
            if !alt_types.contains(&biv) {
                alt_types.push(biv);
            }
        }

        // Under a clause carrying a copula ANYWHERE (`sentence_has_copula`,
        // see its own doc for why this is position-independent unlike
        // `leads_with_modal`/`leads_with_do_support`), a transitive verb in
        // its -ing SURFACE FORM ALSO offers the progressive-participle
        // reading `(NP\S[ng])/NP` — "implementing" in "Illinois is
        // implementing EVV" / "why is Illinois implementing EVV?" — and an
        // intransitive verb offers `progressive_intransitive_verb`
        // (`NP\S[ng]`) directly. Gated on the LOADED morphological signal
        // `entry.olia_class() == Some("ing")` (the dual-route
        // generating-direction lemmatizer check in
        // `english/ontology.rs`, `ing_form(&stem) == word`, AGID-exception
        // aware) — NEVER a suffix strip or word list — so this never
        // over-broadcasts to a verb's ordinary finite surface ("means" does
        // not carry the `ing` class). Additive: the verb's normal finite
        // reading stays primary. Hockenmaier & Steedman (2007),
        // *Computational Linguistics* 33(3), §6.3.1 p.379, dependency (43).
        if sentence_has_copula
            && all_entries.iter().any(|e| e.olia_class() == Some("ing"))
            && primary_type == svo_types::transitive_verb()
        {
            let ptv = svo_types::progressive_transitive_verb();
            if !alt_types.contains(&ptv) {
                alt_types.push(ptv);
            }
        }
        if sentence_has_copula
            && all_entries.iter().any(|e| e.olia_class() == Some("ing"))
            && primary_type == svo_types::intransitive_verb()
        {
            let piv = svo_types::progressive_intransitive_verb();
            if !alt_types.contains(&piv) {
                alt_types.push(piv);
            }
        }

        // A bare (determinerless) PLURAL common noun promoting directly to
        // NP (Carlson 1977's kind reading — "Dogs eat meat") was tried
        // here, gated on the SAME loaded `LexicalEntry::number()` the
        // AGID-verified plural detection below now populates. MEASURED
        // NET-NEGATIVE against the full caregiver corpus gate (2026-07-20,
        // task #38 Construction D): Green -1 net, OverAnswered +2 (breach),
        // PossibleMisroute +1 (breach) — an UNCONDITIONAL bare-noun-to-NP
        // promotion over-generates confident answers for exactly the
        // pattern this codebase's OWN prior history already flagged
        // (`supertag_costs.rs`'s `bare_noun_phrase_unary_rule` doc: a
        // GLOBAL `N → NP` unary rule was previously rejected for the same
        // reason, `define -6`, 2026-07-10) — "What are the benefits to
        // members of an EVV system?" / "What tax credits in 2026?" / "What
        // is the cost to providers?" all flipped from a correct honest
        // abstention to a confidently WRONG answer once their plural object
        // ("benefits"/"credits"/"providers") could stand alone as an NP.
        // Reverted per this task's own monotonic-or-nothing discipline;
        // the underlying morphological infrastructure below (a real,
        // AGID-oracle-verified fix to `Number` being hardcoded `Singular`
        // for every noun) is KEPT — it is correct and complete on its own
        // terms, independent of this specific (rejected) grammar rule. A
        // future attempt at this construction should scope the promotion
        // more narrowly (e.g. only when no other reading derives a
        // complete parse, or restricted to specific argument positions)
        // rather than reintroduce it unconditionally.

        tokens.push(TypedToken {
            word: lower,
            lambek_type: primary_type,
            // Reached only when `forcing[i].forces_np()` was false above, so
            // this token is lexicon-typed running prose — used, not mentioned.
            expression_use: forcing[i].expression_use(),
        });
        alternatives.push(alt_types);
    }

    (tokens, alternatives)
}

/// Tokenize into ontological Tokens — Word occurrences connected through
/// Lemon (sense), Lambek (grammar type), and OLiA (POS annotation).
///
/// This is the Parse functor's first stage: Surface → typed tokens.
/// Each token carries its lexical sense (which ontology concept the word
/// references) and its POS tag, in addition to the Lambek type.
///
/// No richer downstream multi-word recognizer to defer to here — see
/// [`tokenize_ontological_registry_aware`] for the chat pipeline's
/// registry-aware caller.
pub fn tokenize_ontological(text: &str, language: &dyn Language) -> Vec<Token> {
    // `1`: see `tokenize_with_alternatives`'s identical call for why.
    tokenize_ontological_registry_aware(text, language, &|_| false, 1)
}

/// [`tokenize_ontological`], additionally deferring capitalized-run
/// collapsing to `is_registry_known` — see `collapse_capitalized_runs`'s
/// doc comment for the precedence rationale.
///
/// `max_registry_surface_words`: see `tokenize_with_alternatives_registry_
/// aware`'s identical parameter doc.
pub fn tokenize_ontological_registry_aware(
    text: &str,
    language: &dyn Language,
    is_registry_known: &dyn Fn(&str) -> bool,
    max_registry_surface_words: usize,
) -> Vec<Token> {
    #[cfg(feature = "std")]
    let vocab = operators::vocabulary();
    #[cfg(not(feature = "std"))]
    let owned_vocab = operators::load();
    #[cfg(not(feature = "std"))]
    let vocab = &owned_vocab;
    #[cfg(feature = "std")]
    let quotes = quote_glyphs::vocabulary();
    #[cfg(not(feature = "std"))]
    let owned_quotes = quote_glyphs::load();
    #[cfg(not(feature = "std"))]
    let quotes = &owned_quotes;
    #[cfg(feature = "std")]
    let dashes = dash_punctuation::vocabulary();
    #[cfg(not(feature = "std"))]
    let owned_dashes = dash_punctuation::load();
    #[cfg(not(feature = "std"))]
    let dashes = &owned_dashes;
    let (words0, sentence_initial0, comma_after0, semicolon_after0) =
        surface_tokens_with_sentence_bounds(text, vocab, dashes, language);
    let (words0, sentence_initial0) = collapse_medial_comma_adjuncts(
        words0,
        sentence_initial0,
        comma_after0,
        semicolon_after0,
        language,
    );
    let (words, quoted, sentence_initial) =
        collapse_quoted_spans(words0, sentence_initial0, quotes);
    let (words, quoted, sentence_initial) =
        split_possessive_clitics(words, quoted, sentence_initial, language);
    let words = correct_unknown_word_surfaces(
        words,
        &quoted,
        vocab,
        language,
        is_registry_known,
        max_registry_surface_words,
    );
    let (words, forcing, sentence_initial) =
        collapse_capitalized_runs(words, quoted, sentence_initial, is_registry_known);
    words
        .iter()
        .enumerate()
        .map(|(i, word)| {
            let lower = word.to_lowercase();
            let lambek_type = if forcing[i].forces_np() {
                svo_types::proper_noun()
            } else {
                assign_type(&lower, sentence_initial[i], language, vocab)
            };

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
                tense: None,
                expression_use: forcing[i].expression_use(),
            }
        })
        .collect()
}

/// Assign a Lambek type to a word using the language's lexicon.
/// Clause-position-sensitive: a copula/auxiliary/interrogative at the head
/// of its clause gets a question-forming type. `is_clause_initial` is
/// TRUE at raw sentence position 0 AND at the token that begins the MAIN
/// clause after a fronted scope-setting adjunct (`mark_clause_initial_
/// after_fronted_adjunct`'s widened `sentence_initial` — "In self-
/// direction, WHAT is X?"'s "what" is clause-initial even though it is not
/// token 0) — NOT a raw token index, so a caller must thread the SAME
/// widened array this module's own pipeline already carries through every
/// merge/split stage, not recompute position from scratch.
/// For ambiguous words (e.g. verbs with unknown transitivity), all entries
/// are considered and the best fit for the position is selected.
fn assign_type(
    word: &str,
    is_clause_initial: bool,
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

    // The genitive clitic split off its stem by `split_possessive_clitics`
    // has no lexicon entry of its own (it is a synthetic, structural token,
    // like an operator glyph or a number literal) — its category comes
    // directly from the loaded `svo_types::genitive_clitic` assignment.
    if word == "'s" {
        return svo_types::genitive_clitic();
    }

    // A merged medial-comma-supplement token (`collapse_medial_comma_adjuncts`,
    // defines-lens gap G2) has no lexicon entry of its own either — the SAME
    // synthetic-token rationale as the genitive clitic immediately above —
    // its category comes directly from WHICH of the two reserved markers
    // minted it.
    if word == MEDIAL_SUPPLEMENT_NP_MARKER {
        return svo_types::medial_supplement_np();
    }
    if word == MEDIAL_SUPPLEMENT_VERB_MARKER {
        return svo_types::medial_supplement_verb();
    }

    // A list-coordinator marker (`collapse_medial_comma_adjuncts`, defines-lens
    // gap G4(a)) has no lexicon entry either -- the SAME synthetic-token
    // rationale as the two medial-supplement markers immediately above. Its
    // primary category is the NP-level coordinator; the N-level alternative
    // is offered additively below, mirroring "and"/"or"'s own dual reading.
    if is_list_coordinator_marker(word) {
        return svo_types::nominal_coordinator_np();
    }

    // Look up ALL entries — a word can have multiple types
    let entries = language.lexical_lookup_all(word);
    let first = entries.first();
    let pos = first.map(|e| e.pos_tag());

    // Question-forming: clause-initial copulas/auxiliaries
    if is_clause_initial {
        if pos.is_some_and(|p| p.is_question_forming()) {
            return svo_types::question_copula();
        }

        // Clause-initial interrogative → its CCG category from the loaded
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

    // Copula in non-clause-initial position → copula type (NP complement default)
    if pos.is_some_and(|p| p.is_copula()) && !is_clause_initial {
        return svo_types::copula();
    }

    // For verbs with multiple transitivity options, prefer transitive
    // (it can still reduce with intransitive sentences via partial application).
    // The grammar resolves ambiguity through successful derivation.
    if let Some(best) = select_best_entry(&entries) {
        return pos_to_lambek(best);
    }

    // Noisy channel: by this point [`correct_unknown_word_surfaces`] has
    // ALREADY run (earlier in the pipeline, over the raw-case word, with
    // the ambiguity/etiology/acronym/contraction/registry-known guards
    // [`try_spelling_correction`]'s own doc comment describes) and already
    // replaced `word` if a safe correction existed. If `word` reaches this
    // point still unresolved, that was ALREADY the considered decision, not
    // an oversight — re-attempting the SAME correction here, on the
    // already-lowercased word (which has lost the raw-case signal
    // [`is_probable_acronym`] needs), would be both redundant and unsafe: a
    // confirmed real regression when this fallback still called
    // `try_spelling_correction` directly — "RN"/"PPL"/"LO" (real corpus
    // acronyms `correct_unknown_word_surfaces` correctly left alone) were
    // still getting a spuriously "corrected" TYPE here, changing the
    // sentence's parse shape even though the SURFACE stayed correct.

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

/// Noisy channel adjunction: Observation → Correction → Intention (Shannon
/// 1948; Kernighan, Church & Gale 1990). Given an unknown word, find the
/// closest known word within edit-distance 1 and return its CORRECTED
/// SURFACE and type — surface, not just type, so a correction lets
/// downstream entity/definition resolution succeed too, not only the
/// syntactic parse. A confirmed real gap this closes: "medicad" (distance 1
/// from "medicaid") and "resptie" (a transposition of "respite", also
/// distance 1) previously parsed via a correctly-typed but still-misspelled
/// token, then reported as an UNKNOWN surface at entity resolution — the
/// correction helped the sentence parse but never actually let the chat
/// answer the question.
///
/// Gated on [`distance::classify_etiology`] (Corder 1967; Pollock & Zamora
/// 1983) — a genuinely consulted safety invariant, not a decorative call:
/// only a `Performance`-classified correction (the writer knew the
/// spelling but mis-executed it — Pollock & Zamora's own cited statistic is
/// that 90-95% of real single-edit errors ARE performance errors) is
/// trusted. At this function's distance-1 bound the classification is
/// near-certain to read `Performance` — but the check still matters: it
/// documents the invariant this bound relies on, so a future extension to
/// admit distance-2+ candidates (more real typo coverage, but also far more
/// false-positive risk of silently substituting a different real word)
/// cannot accidentally inherit unconditional trust without this gate also
/// being revisited. Building the full Bayesian channel (`Correction =
/// argmax_w P(x|w) * P(w)`, `crate::cognitive::linguistics::orthography::
/// channel`) needs real confusion-matrix and word-frequency corpora this
/// codebase does not yet load — out of scope here, tracked separately.
///
/// AMBIGUITY GUARD: requires exactly one DISTINCT candidate at distance 1.
/// A confirmed real failure mode without this guard: "medicad" is
/// EQUIDISTANT (distance 1) from both "medicaid" (insert 'i') and "medical"
/// (substitute 'd'→'l') — with no `LanguageModel`/P(w) prior to break the
/// tie (the exact missing piece [`crate::cognitive::linguistics::
/// orthography::channel`]'s own `LanguageModel` concept names but has no
/// data behind), an arbitrary pick is a CONFIDENTLY WRONG answer, worse
/// than the honest abstention this word previously got. Ambiguous cases
/// stay unresolved rather than guess — matching this codebase's own
/// standing "honest abstention over confident wrong answer" discipline.
fn try_spelling_correction(word: &str, language: &dyn Language) -> Option<(String, LambekType)> {
    let known = language.known_words();
    let matches = distance::closest_matches(word, &known, 1);
    let mut distinct: Vec<&str> = matches.iter().map(|(c, _)| *c).collect();
    distinct.sort_unstable();
    distinct.dedup();
    let [corrected] = distinct[..] else {
        return None;
    };
    if distance::classify_etiology(word, corrected) != distance::ErrorEtiology::Performance {
        return None;
    }
    let entry = language.lexical_lookup(corrected)?;
    Some((corrected.to_string(), pos_to_lambek(&entry)))
}

/// Is `word` PROBABLY an acronym/initialism rather than a misspelling —
/// 2 or more uppercase-Latin letters (`is_uppercase_latin`, the loaded
/// `character::latin` script query, never a bare `char::is_uppercase`)?
/// English acronym/initialism formation conventionally mints a NEW word
/// from multiple capitalized letters (RN, LO, PPL, IHSS, CDPAP, SDP) — a
/// closed, deliberate word-formation process, orthographically distinct
/// from an ordinary lowercase misspelling like "teh". A confirmed real
/// regression without this guard, found via full corpus regen: short
/// informal acronyms/initialisms real caregivers actually write ("RN",
/// "LO", "PPL", "PAs") sit within edit-distance 1 of an unrelated common
/// WordNet word purely by coincidence of their short length, and were
/// being silently "corrected" away to that unrelated word.
pub fn is_probable_acronym(word: &str) -> bool {
    word.chars().filter(|&c| is_uppercase_latin(c)).count() >= 2
}

/// Is `word` already COVERED by an already-resolved multi-word concept
/// span — one of `resolved_entities`' own WHITESPACE-separated component
/// WORDS (case-insensitive), never a raw character-substring match?
///
/// [`is_probable_acronym`] has no notion of word BOUNDARIES beyond its own
/// input string — it just counts uppercase Latin letters — so a caller that
/// scans raw whitespace-split input tokens for "a probable acronym whose
/// OWN isolated lookup is empty" (`decline_if_an_unresolved_acronym_was_
/// ignored`, `crates/chat/src/lib.rs`) can flag a token that is really just
/// ONE WORD of an already-resolved compound, never a standalone unresolved
/// acronym at all. Three confirmed real corpus false positives, all the
/// SAME mechanism: "What is MyCare Ohio?" declines on "MyCare" even though
/// "mycare ohio" already resolved (a camelCase brand name, not an acronym);
/// "What is a fixed VoIP phone?" declines on "VoIP" even though "voip
/// phone" already resolved; "What is the DHS Aggregator?" declines on "DHS"
/// even though "dhs aggregator" already resolved. In every case the flagged
/// substring is the FIRST word of a multi-word entity the pipeline already
/// named as its answer (`ResponseResult::entities_found`) — checked here at
/// WORD granularity (`str::split_whitespace`), not raw substring
/// containment, so an UNRELATED short coincidental character run inside a
/// longer resolved word (which naive substring matching would wrongly
/// excuse) is never mistaken for coverage.
pub fn is_covered_by_resolved_compound(word: &str, resolved_entities: &[String]) -> bool {
    let lower = word.to_lowercase();
    resolved_entities.iter().any(|entity| {
        entity
            .to_lowercase()
            .split_whitespace()
            .any(|part| part == lower)
    })
}

/// Is `word` an ALPHANUMERIC MIX — at least one digit AND at least one
/// letter, in either order, anywhere in the token (a genitive-clitic split,
/// possessive apostrophe, and quoted-span guards upstream in
/// `correct_unknown_word_surfaces` already run before this one, so this
/// only ever sees the raw mixed token itself)? English orthography never
/// embeds a digit inside an ordinary word stem — no genuine misspelling of
/// an English word ever introduces a digit — so this is a closed,
/// zero-false-positive signal that `word` is NOT a spelling-correction
/// candidate at all, the SAME "closed set of non-English-word token
/// shapes" rationale `operators::is_number_literal` (pure digits) and
/// [`is_probable_acronym`] (2+ uppercase letters) already establish, for a
/// THIRD shape neither of those two covers: a statutory cross-reference
/// fragment like "1395k" or "1395m(h)(4" (a U.S. Code section number
/// immediately followed by subsection letters, with internal parentheses
/// that survive `flush_word`'s LEADING/TRAILING-only punctuation trim).
///
/// A confirmed real regression this guard closes, found via direct
/// instrumentation against real Title 42 Medicare provisions
/// (`crates/praxis-corpus-tests/tests/scratch_probe.rs`,
/// `probe_title42_stage_isolation_for_the_1395l_exception_clause`): 42
/// U.S.C. §1395l(a)(2)'s "except that (A) ... (B) ..." exception clause is
/// saturated with cross-references like this — `try_spelling_correction`
/// ran the FULL noisy-channel edit-distance search (`distance::
/// closest_matches` against `language.known_words()`, ~160k WordNet
/// entries) against each one, at ~674ms average per call — measured
/// directly: 81 calls, 54.586s total, for a single 14,115-character span.
/// None of these fragments are misspellings of anything; they were never
/// spelling-correction candidates to begin with, so this guard is a pure
/// precision fix (no different final tokenization, only skipped dead
/// search), the identical "changes nothing recognized, only how much dead
/// search space is paid for" rationale `multiword_surface_spans`'s own
/// `max_window` bound already establishes for the sibling cost in this
/// same function.
pub fn is_alphanumeric_mix(word: &str) -> bool {
    word.chars().any(|c| c.is_ascii_digit()) && word.chars().any(|c| c.is_alphabetic())
}

/// Correct an unknown word's SURFACE before typing runs, so a single-edit
/// typo benefits entity/definition resolution too — see
/// [`try_spelling_correction`]'s doc comment for the real gap this closes.
/// Skips a quote-collapsed span (a mentioned expression is never
/// "corrected"), a probable acronym/initialism ([`is_probable_acronym`]),
/// an alphanumeric mix like a statutory cross-reference fragment
/// ([`is_alphanumeric_mix`] — see its own doc for the ~674ms/call
/// noisy-channel cost this closes on real Title 42 Medicare provisions),
/// any word already known to the lexicon (case-folded), any word covered by
/// a known MULTI-WORD surface span ([`multiword_surface_spans`] — see its
/// own doc for why a per-word check alone is unsafe), and — per
/// `is_registry_known` — any word already known to the FULL composed
/// reasoner (WordNet ⊕ every registered domain lexicon), the SAME
/// precedence guard [`collapse_capitalized_runs`] needs and for the
/// identical reason: this function sees only `language: &dyn Language`
/// (base English/WordNet), never the registered domain lexicons a chat
/// pipeline composes in separately. A confirmed real regression without
/// THAT guard, found via full corpus regen: "EVV" (a registered acronym
/// with its own statutory gloss, not a WordNet word) sits within
/// edit-distance 1 of an unrelated WordNet word from THIS function's
/// narrow view, so it was being silently "corrected" away before
/// `collapse_multiword_surfaces` (the reasoner-aware recognizer
/// downstream) ever got to resolve it — 15 real Green corpus regressions,
/// nearly all "...EVV?" questions. Leaves an unresolvable word untouched
/// (the OOV-alternative path in [`tokenize_with_alternatives`] still
/// handles it honestly as unknown).
fn correct_unknown_word_surfaces(
    words: Vec<String>,
    quoted: &[bool],
    vocab: &OperatorVocabulary,
    language: &dyn Language,
    is_registry_known: &dyn Fn(&str) -> bool,
    // The composed reasoner's own longest registered surface (in words) —
    // `LexicalReasoner::max_surface_words()` at the real chat call site,
    // `1` (no registry surfaces at all) at every non-registry-aware call
    // site. Combined with `language.max_known_surface_words()` inside
    // `multiword_surface_spans` so NEITHER knowledge source's protection
    // window is ever narrower than its own real maximum.
    registry_max_surface_words: usize,
) -> Vec<String> {
    let max_window = language
        .max_known_surface_words()
        .max(registry_max_surface_words);
    let protected_span = multiword_surface_spans(&words, language, is_registry_known, max_window);
    // Statutory prose repeats the same unrecognized token many times in one
    // sentence (e.g. "subparagraph" 7x in a single Title 42 enumeration).
    // try_spelling_correction runs an uncached brute-force edit-distance
    // search over the full known-word vocabulary, so without this cache
    // every repeat re-pays that full search from scratch — confirmed via
    // direct instrumentation against the real worst Title 42 candidate
    // (crates/praxis-corpus-tests/tests/scratch_probe.rs,
    // probe_title42_stage_isolation_for_the_1395l_exception_clause): roughly
    // half of the 22 remaining slow-path calls were exact repeats of an
    // already-corrected word within the same call to this function.
    let mut correction_cache: alloc::collections::BTreeMap<String, Option<String>> =
        alloc::collections::BTreeMap::new();
    let result: Vec<String> = words
        .into_iter()
        .enumerate()
        .map(|(i, word)| {
            // Mirrors assign_type's own early exits (operator glyph, number
            // literal, genitive clitic) — none of these go through lexicon
            // lookup, so none should be run past the noisy channel either.
            // A contraction ("don't", "can't") is its own closed
            // morphological class, the same reasoning
            // split_possessive_clitics already applies to the genitive
            // clitic — a confirmed real regression without this exclusion:
            // "don't" isn't indexed under language.lexical_lookup_all the
            // way an ordinary word is, so it fell through to the noisy
            // channel and was silently "corrected" to the unrelated word
            // "donut".
            if quoted[i]
                || word == "'s"
                || is_medial_supplement_marker(&word)
                || is_list_coordinator_marker(&word)
                || is_apposition_coordinator_marker(&word)
                || word.contains(punctuation::apostrophe().character)
                || vocab.primary_type(&word).is_some()
                || operators::is_number_literal(&word)
                || is_probable_acronym(&word)
                || is_alphanumeric_mix(&word)
                || protected_span[i]
            {
                return word;
            }
            let lower = word.to_lowercase();
            if !language.lexical_lookup_all(&lower).is_empty() || is_registry_known(&lower) {
                return word;
            }
            if let Some(cached) = correction_cache.get(&lower) {
                return cached.clone().unwrap_or(word);
            }
            let corrected = try_spelling_correction(&lower, language).map(|(c, _)| c);
            correction_cache.insert(lower, corrected.clone());
            corrected.unwrap_or(word)
        })
        .collect();
    result
}

/// Positions in `words` COVERED by a known multi-word surface span (>= 2
/// words) starting at or before that position — the guard
/// [`correct_unknown_word_surfaces`] needs so its per-word noisy-channel
/// correction never fires INSIDE a span [`collapse_multiword_surfaces`]
/// (the reasoner-aware recognizer downstream) would otherwise have
/// recognized as one unit. Window length runs from 2 up to `max_window`
/// (inclusive of both endpoints) — NOT the full remaining sentence: a
/// loaded/registered surface's word count is itself data, so `max_window`
/// is the REAL measured maximum over both knowledge sources this function
/// checks (see its parameter doc), never a guessed constant, and never
/// unbounded either.
///
/// Bounding this window is a REAL, measured fix, not a cosmetic one: before
/// `max_window` existed, this loop tried every window length up to
/// `words.len()` for every start position — O(n²) window checks, each
/// paying its own `O(window length)` string allocation
/// (`words[start..end].join(" ")`), so O(n³) in the worst case. Direct
/// instrumentation against real USC Title 42 candidates
/// (`crates/praxis-corpus-tests/tests/scratch_probe.rs`,
/// `probe_bisect_title42_pathological_candidates`) isolated this as the
/// dominant cost behind `defines_pointers` timing out on long statutory
/// prose: `find_list_coordinator_commas`/`collapse_medial_comma_adjuncts`
/// (the OTHER O(n)-ish preprocessing stages) together cost under 700ms even
/// on a 1524-word candidate, while the unbounded window scan inside THIS
/// function (reached via `correct_unknown_word_surfaces`) dominated the
/// remaining 30+ seconds. A REAL statutory or ordinary-English sentence
/// never contains a multi-word LEXICAL surface anywhere near sentence
/// length — "the term X means Y" definitional prose and Medicare
/// payment-formula enumerations alike are built from short (2-6 word)
/// collocations at most — so bounding by the true measured maximum changes
/// nothing recognized, only how much dead search space is paid for to reach
/// the SAME answer.
///
/// A confirmed real regression the SPAN-GUARD ITSELF (independent of this
/// window bound) closes — ONE mechanism behind both a WordNet-scale and a
/// registered-lexicon corpus-gate break, found via full corpus regen: with
/// no span guard at all, `correct_unknown_word_surfaces`
/// 1-edit-distance-corrects each word of a multi-word surface IN ISOLATION
/// before the span is ever joined. "ursus"→"urus" and "arctos"→"arccos"
/// (both real, unrelated WordNet words) for the WordNet collocation "Ursus
/// arctos"; "look-back"→"look back" (an unrelated WordNet phrasal-verb
/// sense) for the registered term "look-back period"; "old-age"→"old age"
/// for "old-age insurance benefits"; "authenticare"→"authenticate" for
/// "authenticare alabama"; "gfe"→"gee" for "gfe exemption";
/// "timesheet"→"time sheet" for "paper timesheet"/"manual
/// timesheet"/"written timesheet". Every one of these is a genuinely
/// resolvable multi-word surface the correction pass silently destroyed one
/// word of before the reasoner-aware recognizer ever ran.
///
/// Checked against BOTH knowledge sources [`correct_unknown_word_surfaces`]
/// itself defers to: the base [`Language::is_known_surface`] (WordNet's own
/// multi-word lemmas — "Ursus arctos", "old-age insurance benefits") and
/// `is_registry_known` (the FULL composed reasoner a chat pipeline unions
/// in — "look-back period", "gfe exemption" — the SAME registry the
/// single-word gate consults two lines below, for the identical precedence
/// reason [`correct_unknown_word_surfaces`]'s own doc explains). Joined
/// windows are case-folded before either lookup, mirroring the single-word
/// gate's own `to_lowercase` — the case-FOLDED fallback tier both knowledge
/// sources already carry (`lookup_case_folded` / `is_registry_known`'s own
/// composed check) recovers a differently-cased original surface
/// ("Ursus arctos") from the lowercased join.
fn multiword_surface_spans(
    words: &[String],
    language: &dyn Language,
    is_registry_known: &dyn Fn(&str) -> bool,
    // The REAL measured maximum window (in words) worth trying — the caller
    // (`correct_unknown_word_surfaces`) computes this as `max(language.
    // max_known_surface_words(), registry_max_surface_words)`, so it is
    // ALWAYS at least as wide as the longest surface EITHER knowledge
    // source can actually recognize: never a source of a missed known
    // surface, only of skipped windows that could never have matched
    // anything in the first place.
    max_window: usize,
) -> Vec<bool> {
    let mut protected = vec![false; words.len()];
    let max_window = max_window.max(1);
    for start in 0..words.len() {
        // An empty range (`max_end < start + 2`, i.e. fewer than 2 words
        // remain within the window) simply iterates zero times — no
        // separate short-input guard needed.
        let max_end = (start + max_window).min(words.len());
        for end in (start + 2)..=max_end {
            let joined = words[start..end].join(" ").to_lowercase();
            if language.is_known_surface(&joined) || is_registry_known(&joined) {
                for slot in &mut protected[start..end] {
                    *slot = true;
                }
            }
        }
    }
    protected
}

/// Is `word` a do-support form? Do-support ("do"/"does"/"did", the dummy
/// auxiliary that realizes subject-aux inversion when no other auxiliary is
/// present, selecting a bare/non-finite VP complement — Huddleston & Pullum
/// 2002 Ch.3 §7-8) is a GENUINELY CLOSED grammatical category: exactly these
/// three surface forms, with no fourth ever coming. Represented the same way
/// [`super::types::SentenceFeature`]/[`super::types::AtomicType`] themselves
/// are — a small hand-authored membership check, not loaded data — because
/// (unlike a lexicon or a category-projection table) English's do-support
/// paradigm is not an open or extensible set. See [`svo_types::does_support`]
/// for why this is a code constructor rather than a loaded TSV row keyed on
/// a shared OLiA auxiliary class.
fn is_do_support(word: &str) -> bool {
    matches!(word, "do" | "does" | "did")
}

/// Is `word` one of the CLOSED 9-item modal auxiliary class (Huddleston &
/// Pullum 2002, *The Cambridge Grammar of the English Language*, Ch. 3 §9)?
/// Represented as a hand-authored membership check, not loaded data —
/// mirroring [`is_do_support`]'s own rationale exactly: the loaded OLiA
/// subclasses do not distinguish modals from do/have/will-periphrastic
/// auxiliaries, which select a DIFFERENT complement shape (a participle, not
/// a bare stem), so a blind `PosTag::Auxiliary` row would broadcast
/// `modal_question` to every auxiliary — the same over-broadcast
/// `is_do_support`'s own doc comment warns against for do-support.
fn is_modal_auxiliary(word: &str) -> bool {
    matches!(
        word,
        "can" | "could" | "may" | "might" | "shall" | "should" | "will" | "would" | "must"
    )
}

/// Is `word` the infinitive-marker "to" — the closed, single-member surface
/// [`svo_types::infinitive_to`] applies to? English has exactly one
/// infinitival particle (Huddleston & Pullum 2002, *The Cambridge Grammar of
/// the English Language*, Ch. 14 §2), so this is a one-literal membership
/// check — the SAME closed-surface rationale [`is_do_support`]/
/// [`is_modal_auxiliary`] already use, minimal for a class of exactly one.
/// `montague::apply`'s syncategorematic pass-through rule for
/// [`svo_types::infinitive_to`] imports this single check (rather than
/// re-deriving its own literal) so the tokenizer's typing and the
/// semantics' dispatch can never drift apart — the same discipline
/// [`is_fronted_scope_adjunct_head`] already establishes.
pub(crate) fn is_infinitive_marker(word: &str) -> bool {
    word == "to"
}

/// Is `word` a NOMINAL coordinator surface — "and"/"or" specifically, never
/// "but"/"nor"/"yet" (contrastive/negative coordinators a corpus regression
/// check found do NOT behave like plain nominal conjunction) and never a
/// subordinator (`SubordinatingConjunction` already has its own concrete
/// category, `S/S/S`)? A hand-authored surface check, not a loaded
/// `Conjunction`-class row — the SAME rationale as [`is_do_support`]/
/// [`is_modal_auxiliary`]: the loaded `Conjunction` OLiA class covers ALL
/// coordinators uniformly (its own TSV row is an explicitly-flagged
/// PROVISIONAL placeholder, `N`), so a class-keyed row would broadcast
/// nominal-coordination to "but" too. Scoped to exactly the two surfaces
/// the corpus evidences (task #8's coordination slice).
fn is_nominal_coordinator(word: &str) -> bool {
    matches!(word, "and" | "or")
}

/// Is `word` a CLOSED fronted-scope-adjunct head that selects its complement
/// DIRECTLY as an NP — "for" (`"For purposes of this subsection, ..."`) /
/// "in" (`"In this subsection, ..."`)? A hand-authored closed-class check,
/// the SAME rationale as [`is_do_support`]/[`is_modal_auxiliary`]/
/// [`is_nominal_coordinator`]: this is a SYNTACTIC-POSITION-SPECIFIC reading
/// (sentence-initial, scope-setting) of an ordinary closed-class
/// preposition, not a distinction the loaded `Preposition` OLiA class itself
/// makes — every preposition in `data/function-words/english.xml` shares one
/// undifferentiated `adp` class, so a class-keyed row would broadcast the
/// scope-adjunct reading to every preposition regardless of position or
/// headword. See [`svo_types::fronted_scope_adjunct_np`]'s own doc for the
/// citation.
pub(crate) fn is_fronted_scope_adjunct_np_head(word: &str) -> bool {
    matches!(word, "for" | "in")
}

/// Is `word` a CLOSED fronted-scope-adjunct head that selects an
/// ALREADY-derived `NP\NP` ("PP") complement — "except" (`"Except for the
/// purposes of X, ..."`) / "subject" (`"Subject to subparagraphs (B) and
/// (C), ..."`)? See [`svo_types::fronted_scope_adjunct_pp`]'s own doc for the
/// citation. Mirrors [`is_fronted_scope_adjunct_np_head`]'s rationale
/// exactly.
pub(crate) fn is_fronted_scope_adjunct_pp_head(word: &str) -> bool {
    matches!(word, "except" | "subject")
}

/// Is `word` ANY of the CLOSED fronted-scope-adjunct heads this grammar
/// recognizes, regardless of which complement shape it selects — the union
/// [`is_fronted_scope_adjunct_np_head`] ∪ [`is_fronted_scope_adjunct_pp_head`].
/// `montague::apply`'s transparent-pass-through composition rule imports this
/// single check (rather than re-deriving its own closed list) so the
/// tokenizer's gating and the semantics' dispatch can never drift apart.
pub(crate) fn is_fronted_scope_adjunct_head(word: &str) -> bool {
    is_fronted_scope_adjunct_np_head(word) || is_fronted_scope_adjunct_pp_head(word)
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

/// ALL of a lexical entry's Lambek categories — every loaded row of its OLiA
/// key ([`olia_key`]) from the OLiA→CCG functor, in file order (the loader's
/// documented multi-row contract: "a class with several rows yields several
/// readings — the chart explores them"). The first row is the entry's PRIMARY
/// reading ([`pos_to_lambek`]); the rest ride the chart alternatives.
///
/// Copula is the one irreducible exception: its category is position-dependent
/// (sentence-initial question vs medial copula vs pre-adjective), resolved by
/// [`assign_type`]; its default medial reading `copula()` is grammar logic, not
/// a class→category map, so it stays here rather than as a functor row.
fn entry_categories(
    entry: &crate::cognitive::linguistics::lexicon::pos::LexicalEntry,
) -> Vec<LambekType> {
    use crate::cognitive::linguistics::lexicon::pos::LexicalEntry;
    if let LexicalEntry::Copula(_) = entry {
        return vec![svo_types::copula()];
    }
    let (fragment, valency) = olia_key(entry);
    category_projection::categories_for_class_valency(fragment, valency)
}

/// A lexical entry's PRIMARY Lambek category — the FIRST loaded row of
/// [`entry_categories`]; the remaining rows are chart alternatives, so the
/// chart (never entry order) decides the winning reading.
fn pos_to_lambek(entry: &crate::cognitive::linguistics::lexicon::pos::LexicalEntry) -> LambekType {
    entry_categories(entry)
        .into_iter()
        .next()
        .unwrap_or_else(svo_types::noun)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(word: &str) -> TypedToken {
        TypedToken {
            expression_use: ExpressionUse::Used,
            word: word.to_string(),
            lambek_type: svo_types::noun(),
        }
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_dash_is_never_sentence_terminal() {
        // Trimmable (the `flush_word` fix), never sentence-ending: a dash
        // carries `PunctuationFunction::Dash`, which
        // `PunctuationMark::is_sentence_ending()` never matches (only
        // StatementTerminator/QuestionMarker/EmphasisMarker do) — confirming
        // this fix did not accidentally widen `is_sentence_terminal_char`'s
        // loaded-query gate. The genuine sentence terminators still trip it.
        for dash in ['-', '\u{2013}', '\u{2014}', '\u{2015}'] {
            assert!(
                !is_sentence_terminal_char(dash),
                "{dash:?} must not be treated as sentence-terminal"
            );
        }
        for terminator in ['.', '?', '!'] {
            assert!(
                is_sentence_terminal_char(terminator),
                "{terminator:?} must still be treated as sentence-terminal"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn fronted_scope_adjunct_heads_are_the_closed_reported_set() {
        // The defines-lens gap report's own four G1 constructions verbatim:
        // "For purposes of X," / "In this subsection," / "Except for the
        // purposes of X," / "Subject to Y,". "for"/"in" select an NP
        // complement directly; "except"/"subject" select an already-derived
        // NP\NP ("PP") complement — the two categories stay disjoint.
        for w in ["for", "in"] {
            assert!(
                is_fronted_scope_adjunct_np_head(w),
                "{w:?} must be the NP-complement variant"
            );
            assert!(
                !is_fronted_scope_adjunct_pp_head(w),
                "{w:?} must not ALSO be the PP-complement variant"
            );
        }
        for w in ["except", "subject"] {
            assert!(
                is_fronted_scope_adjunct_pp_head(w),
                "{w:?} must be the PP-complement variant"
            );
            assert!(
                !is_fronted_scope_adjunct_np_head(w),
                "{w:?} must not ALSO be the NP-complement variant"
            );
        }
        for w in ["for", "in", "except", "subject"] {
            assert!(
                is_fronted_scope_adjunct_head(w),
                "{w:?} must be in the union check `montague::apply` dispatches on"
            );
        }
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn an_ordinary_preposition_is_not_a_fronted_scope_adjunct_head() {
        // "to" heads the COMPLEMENT of "subject to Y" but is never itself
        // the fronted-adjunct HEAD; every other ordinary preposition stays
        // untouched too.
        for w in ["of", "with", "to", "by", "on", "under", "from"] {
            assert!(
                !is_fronted_scope_adjunct_head(w),
                "{w:?} must not be treated as a scope-adjunct head"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn medial_supplement_interior_heads_are_the_closed_reported_set() {
        // The defines-lens gap report's own G2 constructions, plus this
        // module's own pre-existing "as used in this title" fixture
        // (`grounding::tests::a_real_parenthetical_interrupted_sample_yields_no_pointer`,
        // title 18): "used with respect to Y" (inclusion), "when used in
        // connection with Y" (physician), "as used in this title" (vessel
        // of the United States).
        for w in ["as", "when", "used", "As", "WHEN", "Used"] {
            assert!(
                is_medial_supplement_interior_head(w),
                "{w:?} must be a recognized medial-supplement interior head"
            );
        }
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn an_ordinary_word_is_not_a_medial_supplement_interior_head() {
        // Real coordination-list continuations ("county", "state") and
        // ordinary prepositions/pronouns must NOT open a supposed
        // subject-verb supplement — this is the precision guard against
        // sweeping a coordinated subject list into a bogus supplement (see
        // `collapse_medial_comma_adjuncts`'s own doc comment).
        for w in ["county", "state", "which", "with", "the", "and", "or"] {
            assert!(
                !is_medial_supplement_interior_head(w),
                "{w:?} must not be treated as a medial-supplement interior head"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn medial_supplement_markers_are_recognized_and_nothing_else_is() {
        assert!(is_medial_supplement_marker(MEDIAL_SUPPLEMENT_NP_MARKER));
        assert!(is_medial_supplement_marker(MEDIAL_SUPPLEMENT_VERB_MARKER));
        for w in ["means", "used", "consumer", "", "medial-supplement-np"] {
            assert!(
                !is_medial_supplement_marker(w),
                "{w:?} must not be mistaken for a reserved medial-supplement marker"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn recognizes_the_trailing_alternative_adjunct_head_case_insensitively() {
        for w in ["whether", "Whether", "WHETHER"] {
            assert!(
                is_trailing_alternative_adjunct_head(w),
                "{w:?} must be a recognized trailing-alternative-adjunct head"
            );
        }
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn an_ordinary_word_is_not_a_trailing_alternative_adjunct_head() {
        for w in ["if", "when", "as", "used", "unless", "that"] {
            assert!(
                !is_trailing_alternative_adjunct_head(w),
                "{w:?} must not be treated as a trailing-alternative-adjunct head"
            );
        }
    }

    /// The load-bearing REGRESSION coverage for the trailing-"whether"
    /// adjunct drop on the path that ACTUALLY matters most: the live-chat
    /// tokenizer (`tokenize_with_alternatives_registry_aware`/
    /// `tokenize_ontological_registry_aware`, called directly on raw user
    /// input with no semicolon pre-split — unlike `defines_pointers`, whose
    /// own `grounding.rs` tests cover only ITS path via `split_into_
    /// sentences`). Confirmed via direct execution (`probe_trailing_
    /// whether_drop_crosses_semicolon_on_chat_path`,
    /// `crates/praxis-corpus-tests/tests/scratch_probe.rs`) that WITHOUT the
    /// semicolon bound this test locks in, the fix silently deleted an
    /// entire unrelated independent clause following a semicolon.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn trailing_whether_adjunct_drop_never_crosses_a_semicolon_on_the_chat_path() {
        let en = crate::cognitive::linguistics::english::english_loaded();
        let (tokens, _alts) = tokenize_with_alternatives_registry_aware(
            "The agent may act for the principal, whether or not authorized in \
             writing; the principal remains liable for any debts the agent incurs.",
            en,
            &|_| false,
            1,
        );
        let surface: Vec<&str> = tokens.iter().map(|t| t.word.as_str()).collect();
        assert!(
            surface.contains(&"liable"),
            "the independent clause after the semicolon must survive tokenization, got {surface:?}"
        );
        assert!(
            surface.contains(&"incurs"),
            "the independent clause after the semicolon must survive tokenization, got {surface:?}"
        );
        assert!(
            !surface.contains(&"writing"),
            "the whether-adjunct itself (up to the semicolon) must still be dropped, got {surface:?}"
        );
    }

    /// PRECISION guard: the trailing-adjunct drop requires the construction's
    /// own definitional "or" ([`is_trailing_alternative_adjunct_head`]'s own
    /// doc, H&P 2002 Ch. 8 §14.6) — a bare comma-adjacent "whether" clause
    /// with no "or" anywhere before the clause boundary is NOT this
    /// construction and must be left untouched, not silently swallowed.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn trailing_whether_without_an_or_marker_is_not_swallowed() {
        let en = crate::cognitive::linguistics::english::english_loaded();
        let (tokens, _alts) = tokenize_with_alternatives_registry_aware(
            "The board approved the request, whether it was ready.",
            en,
            &|_| false,
            1,
        );
        let surface: Vec<&str> = tokens.iter().map(|t| t.word.as_str()).collect();
        assert!(
            surface.contains(&"ready"),
            "an \"or\"-less trailing \"whether\" clause must not be dropped, got {surface:?}"
        );
    }

    /// PRECISION guard: a "whether ... or" run immediately after a verb/
    /// copula/auxiliary ([`is_predicate_leaf`]'s own doc, H&P Ch. 11
    /// "Content clauses and reported speech") is that predicate's own
    /// interrogative COMPLEMENT, not an exhaustive-conditional ADJUNCT, and
    /// must not be swallowed. Real, professionally-edited counter-example
    /// already present in this repo's own PropBank corroboration data
    /// (`crates/domains/data/propbank/propbank-3.4.0.propbank`, Wall Street
    /// Journal-sourced): "That is , whether there should be a separation of
    /// politics and economics or not ." — confirmed by direct execution
    /// that WITHOUT this guard, the trailing-whether branch collapsed this
    /// entire sentence to just `["that", "is"]`.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn trailing_whether_after_a_complement_taking_predicate_survives() {
        let en = crate::cognitive::linguistics::english::english_loaded();
        let cases = [
            (
                "copula subject-complement (real PropBank/WSJ sentence)",
                "That is, whether there should be a separation of politics and \
                 economics or not.",
                "separation",
            ),
            (
                "matrix verb taking an interrogative complement",
                "I don't know, whether we should call the doctor or not.",
                "doctor",
            ),
        ];
        for (label, text, must_survive) in cases {
            let (tokens, _alts) =
                tokenize_with_alternatives_registry_aware(text, en, &|_| false, 1);
            let surface: Vec<&str> = tokens.iter().map(|t| t.word.as_str()).collect();
            assert!(
                surface.contains(&must_survive),
                "[{label}] a complement-taking predicate's own \"whether ... or\" \
                 clause must not be dropped, got {surface:?}"
            );
        }
    }

    // ---- defines-lens gap G4(a): n-ary list-comma coordination ----

    /// The REAL, measured false positive [`is_determiner`]'s own doc names:
    /// "grant" carries a genuine transitive-verb WordNet sense, but
    /// immediately after a determiner ("a grant,") it can only be a NOUN —
    /// English never places a finite verb directly after a determiner.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_determiner_headed_verb_homonym_noun_is_not_mistaken_for_a_verb_trigger() {
        let en = crate::cognitive::linguistics::english::english_loaded();
        assert!(
            is_transitive_verb_leaf("grant", en),
            "\"grant\" really does carry a transitive-verb WordNet sense"
        );
        assert!(is_determiner("a", en));
        assert!(
            !is_determiner("grant", en),
            "\"grant\"'s PRIMARY reading must not itself be a determiner"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn list_coordinator_markers_are_recognized_and_nothing_else_is() {
        assert!(is_list_coordinator_marker(LIST_COORDINATOR_MARKER_AND));
        assert!(is_list_coordinator_marker(LIST_COORDINATOR_MARKER_OR));
        for w in ["and", "or", "means", "", "list-coordinator-or"] {
            assert!(
                !is_list_coordinator_marker(w),
                "{w:?} must not be mistaken for a reserved list-coordinator marker"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn nominal_coordinator_canonical_covers_literal_words_and_both_markers() {
        assert_eq!(nominal_coordinator_canonical("and"), Some("and"));
        assert_eq!(nominal_coordinator_canonical("or"), Some("or"));
        assert_eq!(
            nominal_coordinator_canonical(LIST_COORDINATOR_MARKER_AND),
            Some("and")
        );
        assert_eq!(
            nominal_coordinator_canonical(LIST_COORDINATOR_MARKER_OR),
            Some("or")
        );
        for w in ["but", "nor", "consumer", ""] {
            assert_eq!(
                nominal_coordinator_canonical(w),
                None,
                "{w:?} must not be treated as a coordinator surface"
            );
        }
    }

    /// The REAL report-cited example, 42 U.S.C. § 3002(42): "The term
    /// "physical harm" means bodily injury, impairment, or disease." Only
    /// the comma between "injury" and "impairment" stands in for the list's
    /// own "or" — the Oxford comma directly before "or" itself is NOT
    /// marked (the literal "or" already provides that boundary).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn finds_the_real_physical_harm_list_coordinator_comma() {
        let words: Vec<String> = ["bodily", "injury", "impairment", "or", "disease"]
            .iter()
            .map(|w| w.to_string())
            .collect();
        // comma_after: "injury," "impairment," -- true at both; nothing else.
        let comma_after = alloc::vec![false, true, true, false, false];
        let en = crate::cognitive::linguistics::english::english_loaded();
        let marks = find_list_coordinator_commas(&words, &comma_after, en);
        assert_eq!(
            marks.get(&1).copied(),
            Some("or"),
            "the comma after \"injury\" (index 1) must be marked as the list's \"or\""
        );
        assert_eq!(
            marks.len(),
            1,
            "the Oxford comma directly before \"or\" (index 2) must NOT be marked; got {marks:?}"
        );
    }

    /// The REAL report-cited example, 42 U.S.C. § 300ii(5): "...means an
    /// unpaid family member, a foster parent, or another unpaid
    /// individual, who provides in-home monitoring, management,
    /// supervision, or treatment of a child or adult with a special
    /// need." Proves the chain-walk finds ONLY the two real list commas in
    /// the outer 3-item list, does NOT bleed into the comma that opens the
    /// relative clause ("individual,"), and independently finds the two
    /// real list commas in the doubly-nested inner list -- verified by
    /// hand against the byte-real sentence, not a simplified stand-in.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn finds_the_real_family_caregiver_list_coordinator_commas_without_bleeding_into_the_relative_clause()
     {
        let surface = [
            "an",
            "unpaid",
            "family",
            "member",
            "a",
            "foster",
            "parent",
            "or",
            "another",
            "unpaid",
            "individual",
            "who",
            "provides",
            "in-home",
            "monitoring",
            "management",
            "supervision",
            "or",
            "treatment",
            "of",
            "a",
            "child",
            "or",
            "adult",
            "with",
            "a",
            "special",
            "need",
        ];
        let words: Vec<String> = surface.iter().map(|w| w.to_string()).collect();
        let comma_after: Vec<bool> = surface
            .iter()
            .map(|w| {
                matches!(
                    *w,
                    "member"
                        | "parent"
                        | "individual"
                        | "monitoring"
                        | "management"
                        | "supervision"
                )
            })
            .collect();
        let en = crate::cognitive::linguistics::english::english_loaded();
        let marks = find_list_coordinator_commas(&words, &comma_after, en);
        let idx = |w: &str| surface.iter().position(|&x| x == w).unwrap();
        assert_eq!(marks.get(&idx("member")).copied(), Some("or"));
        assert_eq!(marks.get(&idx("monitoring")).copied(), Some("or"));
        assert_eq!(marks.get(&idx("management")).copied(), Some("or"));
        assert_eq!(
            marks.get(&idx("individual")),
            None,
            "the comma before the relative pronoun \"who\" must NOT be treated as a list coordinator"
        );
        assert_eq!(
            marks.len(),
            3,
            "exactly the three real list commas; got {marks:?}"
        );
    }

    /// A plain two-item "X and Y" (no comma at all) must not mark anything
    /// -- the existing binary category already covers it.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_plain_two_item_list_marks_nothing() {
        let words: Vec<String> = ["a", "child", "or", "adult"]
            .iter()
            .map(|w| w.to_string())
            .collect();
        let comma_after = alloc::vec![false, false, false, false];
        let en = crate::cognitive::linguistics::english::english_loaded();
        let marks = find_list_coordinator_commas(&words, &comma_after, en);
        assert!(marks.is_empty(), "got {marks:?}");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_multi_capital_token_is_a_probable_acronym() {
        // Real corpus evidence this guard exists for: RN, LO, PPL were all
        // being miscorrected to an unrelated WordNet word before this check.
        assert!(is_probable_acronym("RN"));
        assert!(is_probable_acronym("LO"));
        assert!(is_probable_acronym("PPL"));
        assert!(is_probable_acronym("PAs"));
        assert!(is_probable_acronym("IHSS"));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn an_ordinary_lowercase_misspelling_is_not_a_probable_acronym() {
        assert!(!is_probable_acronym("teh"));
        assert!(!is_probable_acronym("dog"));
        // A single sentence-initial capital alone is not enough to read as
        // an acronym -- ordinary capitalization needs 2+ uppercase letters
        // to trip this guard.
        assert!(!is_probable_acronym("The"));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_probable_acronym_substring_of_an_already_resolved_compound_is_covered() {
        // The exact three confirmed corpus false positives
        // `decline_if_an_unresolved_acronym_was_ignored`
        // (`crates/chat/src/lib.rs`) produced before this guard existed: in
        // each case the flagged substring is really just the FIRST word of
        // an already-resolved multi-word compound, not a standalone
        // unresolved acronym.
        assert!(is_covered_by_resolved_compound(
            "MyCare",
            &["mycare ohio".to_string()]
        ));
        assert!(is_covered_by_resolved_compound(
            "VoIP",
            &["voip phone".to_string()]
        ));
        assert!(is_covered_by_resolved_compound(
            "DHS",
            &["dhs aggregator".to_string()]
        ));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn an_unrelated_acronym_is_not_covered_by_an_unrelated_resolved_entity() {
        // A genuinely unresolved acronym next to an UNRELATED resolved
        // entity must still decline -- this guard only excuses a substring
        // that is itself one of the resolved entity's own words, never a
        // blanket "some entity resolved, so stop checking" bypass.
        assert!(!is_covered_by_resolved_compound(
            "IHSS",
            &["mycare ohio".to_string()]
        ));
        // Nor does a coincidental raw-character substring count -- "DH"
        // occurs inside "dhs aggregator" as characters, but is not one of
        // its whitespace-separated WORDS.
        assert!(!is_covered_by_resolved_compound(
            "DH",
            &["dhs aggregator".to_string()]
        ));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_digit_letter_mixed_token_is_alphanumeric_mix() {
        // Real corpus evidence: statutory cross-reference fragments like
        // these are what `correct_unknown_word_surfaces` must never run the
        // brute-force edit-distance search against — direct causal testing
        // this session (`crates/praxis-corpus-tests/tests/scratch_probe.rs`,
        // `probe_is_alphanumeric_mix_direct_defines_pointers_impact`) found
        // real occurrences where "correcting" one of these into its nearest
        // WordNet neighbor (e.g. "77j" -> "77", "256h" -> "25th") DESTROYED
        // an extraction that succeeds with the fragment left alone — never
        // the reverse direction, across 556 real (candidate, token) pairs.
        assert!(is_alphanumeric_mix("1395m(h)(4"));
        assert!(is_alphanumeric_mix("77j"));
        assert!(is_alphanumeric_mix("256h"));
        assert!(is_alphanumeric_mix("42a"));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_pure_word_or_pure_number_is_not_alphanumeric_mix() {
        // Pure-alphabetic and pure-digit tokens are guarded by
        // `is_probable_acronym`/`operators::is_number_literal` respectively
        // -- `is_alphanumeric_mix` exists for the THIRD case, digit+letter
        // both present, and must not fire on either of the other two.
        assert!(!is_alphanumeric_mix("subparagraph"));
        assert!(!is_alphanumeric_mix("1395"));
        assert!(!is_alphanumeric_mix(""));
    }

    /// End-to-end proof (not just the pure-predicate unit test above) that
    /// the guard actually reaches `correct_unknown_word_surfaces` and
    /// prevents a statutory cross-reference fragment from being silently
    /// noisy-channel-"corrected" into an unrelated WordNet word during real
    /// tokenization -- the exact mechanism
    /// `probe_is_alphanumeric_mix_direct_defines_pointers_impact` confirmed
    /// against 556 real (candidate, token) occurrences this session.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_citation_fragment_survives_tokenization_unchanged() {
        let en = crate::cognitive::linguistics::english::english_loaded();
        let (tokens, _alts) = tokenize_with_alternatives_registry_aware(
            "The term \u{201C}State\u{201D} means each jurisdiction described \
             in section 77j of this title.",
            en,
            &|_| false,
            1,
        );
        let surface: Vec<&str> = tokens.iter().map(|t| t.word.as_str()).collect();
        assert!(
            surface.contains(&"77j"),
            "the citation fragment must survive tokenization verbatim, not be \
             noisy-channel-corrected into an unrelated WordNet word (e.g. \
             \u{201C}77\u{201D}); got {surface:?}"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn collapses_a_known_multiword_surface_into_one_proper_noun() {
        // "ice cream" is a known surface → its two tokens collapse into ONE
        // token carrying the classifier's readings (primary = the first); the
        // surrounding tokens pass through unchanged.
        let tokens = vec![tok("what"), tok("is"), tok("ice"), tok("cream")];
        let alts = vec![vec![], vec![], vec![], vec![]];
        let (out, types) = collapse_multiword_surfaces(&tokens, &alts, 2, |s| {
            (s == "ice cream").then(|| vec![svo_types::proper_noun()])
        });
        assert_eq!(out.len(), 3, "ice + cream collapse into one token");
        assert_eq!(out[2].word, "ice cream");
        assert_eq!(out[2].lambek_type, svo_types::proper_noun());
        assert_eq!(types[2], vec![svo_types::proper_noun()]);
        assert_eq!(out[0].word, "what", "earlier tokens are untouched");
        assert_eq!(out[1].word, "is");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_collapsed_span_carries_every_classified_reading() {
        // The determiner case behind the corpus's multi-word is-a/define reds:
        // a collapsed nominal span must be able to carry BOTH the stand-alone
        // NP reading and the common-noun N reading, so "a cough out" reduces
        // (determiner NP/N + N → NP) exactly like the loaded-surface dual
        // push already does for single tokens. The classifier decides the
        // reading set from its loaded indices; this stage only carries it.
        let tokens = vec![tok("a"), tok("cough"), tok("out")];
        let alts = vec![vec![], vec![], vec![]];
        let (out, types) = collapse_multiword_surfaces(&tokens, &alts, 2, |s| {
            (s == "cough out").then(|| vec![svo_types::proper_noun(), svo_types::noun()])
        });
        assert_eq!(out.len(), 2, "cough + out collapse into one token");
        assert_eq!(out[1].word, "cough out");
        assert_eq!(
            out[1].lambek_type,
            svo_types::proper_noun(),
            "the primary reading is the classifier's first"
        );
        assert_eq!(
            types[1],
            vec![svo_types::proper_noun(), svo_types::noun()],
            "the chart sees every classified reading"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn never_collapses_with_a_degenerate_window_or_no_match() {
        let tokens = vec![tok("ice"), tok("cream")];
        let alts = vec![vec![], vec![]];
        // max_surface_words == 1 → the window is degenerate → a pure no-op, even
        // if EVERY span would match (the embedded single-word-lexicon path).
        let (out, _) = collapse_multiword_surfaces(&tokens, &alts, 1, |_| {
            Some(vec![svo_types::proper_noun()])
        });
        assert_eq!(out.len(), 2, "max=1 never collapses");
        // No surface matches → nothing collapses.
        let (out2, _) = collapse_multiword_surfaces(&tokens, &alts, 3, |_| None);
        assert_eq!(out2.len(), 2, "no matching surface → no collapse");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn longest_match_wins() {
        // Both "new york" and "new york city" are surfaces — the LONGEST one wins
        // (greedy maximal munch), so a citation is not under-recognized.
        let tokens = vec![tok("new"), tok("york"), tok("city")];
        let alts = vec![vec![], vec![], vec![]];
        let (out, _) = collapse_multiword_surfaces(&tokens, &alts, 3, |s| {
            (s == "new york" || s == "new york city").then(|| vec![svo_types::proper_noun()])
        });
        assert_eq!(out.len(), 1, "the longest match collapses all three");
        assert_eq!(out[0].word, "new york city");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn an_uncollapsed_token_keeps_its_type_and_alternatives() {
        // The no-collapse path must reproduce the pipeline's prior type_sets
        // exactly: the token's own type plus that position's alternatives.
        let tokens = vec![tok("dog")];
        let alts = vec![vec![svo_types::proper_noun()]];
        let (_, types) = collapse_multiword_surfaces(&tokens, &alts, 1, |_| None);
        assert_eq!(types[0], vec![svo_types::noun(), svo_types::proper_noun()]);
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn never_collapses_across_a_coordinator_wh_word_boundary() {
        // The exact corpus regression shape: WordNet has a real lemma
        // `<Lemma writtenForm="and how" partOfSpeech="r"/>`, so a fake
        // classifier that recognizes "and how" (mirroring that real entry)
        // must NOT be allowed to swallow the "and" that closes "What is
        // long-term care insurance and how can it help me?"'s first clause
        // plus the "how" that opens its second one.
        let tokens = vec![tok("insurance"), tok("and"), tok("how"), tok("it")];
        let alts = vec![vec![], vec![], vec![], vec![]];
        let (out, _) = collapse_multiword_surfaces(&tokens, &alts, 2, |s| {
            (s == "and how").then(|| vec![svo_types::adverb()])
        });
        assert_eq!(
            out.len(),
            4,
            "no collapse: \"and\"/\"how\" must stay separate tokens, got {out:?}"
        );
        assert_eq!(out[1].word, "and");
        assert_eq!(out[2].word, "how");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_coordinator_still_collapses_with_a_non_wh_neighbor() {
        // Precision control for the boundary gate above: it must exclude
        // ONLY a coordinator directly followed by a wh-word, not every
        // "and"-initiated span -- an ordinary loaded surface spelled with a
        // leading "and" and a non-wh second word still collapses exactly as
        // before.
        let tokens = vec![tok("rules"), tok("and"), tok("regulations")];
        let alts = vec![vec![], vec![], vec![]];
        let (out, _) = collapse_multiword_surfaces(&tokens, &alts, 2, |s| {
            (s == "and regulations").then(|| vec![svo_types::noun()])
        });
        assert_eq!(out.len(), 2, "\"and regulations\" collapses, got {out:?}");
        assert_eq!(out[1].word, "and regulations");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn hyphen_insensitive_multiword_collapse() {
        // The `parenthetical_abbreviation` corpus bucket: the question
        // spells the compound open ("Community Based", two words), the
        // loaded WordNet lemma is hyphenated ("community-based"). Only the
        // hyphenated spelling is a known surface here -- the collapse must
        // still succeed and must carry the MATCHED (hyphenated) spelling
        // forward as the token's own word, since downstream code
        // (`process_with_reasoner`) re-checks `reasoner.is_loaded_surface`
        // against exactly this field.
        let tokens = vec![tok("home"), tok("community"), tok("based")];
        let alts = vec![vec![], vec![], vec![]];
        let (out, types) = collapse_multiword_surfaces(&tokens, &alts, 3, |s| {
            (s == "community-based").then(|| vec![svo_types::proper_noun(), svo_types::noun()])
        });
        assert_eq!(
            out.len(),
            2,
            "\"community\"+\"based\" collapse under the hyphenated variant, got {out:?}"
        );
        assert_eq!(out[0].word, "home", "earlier token is untouched");
        assert_eq!(out[1].word, "community-based");
        assert_eq!(types[1], vec![svo_types::proper_noun(), svo_types::noun()]);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn plural_head_noun_lemmatizes_before_multiword_match() {
        // The `why_does_x_occur` corpus bucket ("Why do GPS exceptions
        // occur?"): the question pluralizes the compound's head noun, the
        // loaded surface is singular ("GPS exception"). Only the singular
        // spelling is a known surface here -- the collapse must still
        // succeed by singularizing the RIGHTMOST (head) word before
        // probing, via the loaded lemmatizer (not a bare `s`-strip).
        let tokens = vec![tok("gps"), tok("exceptions")];
        let alts = vec![vec![], vec![]];
        let (out, _) = collapse_multiword_surfaces(&tokens, &alts, 2, |s| {
            (s == "gps exception").then(|| vec![svo_types::proper_noun()])
        });
        assert_eq!(
            out.len(),
            1,
            "\"gps\"+\"exceptions\" collapse under the singularized head noun, got {out:?}"
        );
        assert_eq!(out[0].word, "gps exception");
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn plural_lemmatization_does_not_fabricate_a_match_that_is_not_loaded() {
        // Conservativeness control: when NO variant (literal, hyphenated,
        // or singularized) is actually loaded, nothing collapses -- the
        // variant probe only ever recovers a genuinely loaded surface,
        // never invents one.
        let tokens = vec![tok("gps"), tok("exceptions")];
        let alts = vec![vec![], vec![]];
        let (out, _) = collapse_multiword_surfaces(&tokens, &alts, 2, |_| None);
        assert_eq!(
            out.len(),
            2,
            "no loaded surface under any variant -> no collapse"
        );
    }

    fn words(ws: &[&str]) -> Vec<String> {
        ws.iter().map(|w| w.to_string()).collect()
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_curly_quoted_single_word_collapses_to_its_bare_np_content() {
        let quotes = quote_glyphs::load();
        let sentence_initial = vec![true, false, false, false];
        let (out, quoted, _sentence_initial) = collapse_quoted_spans(
            words(&["what", "does", "\u{201C}deadly\u{201D}", "mean"]),
            sentence_initial,
            &quotes,
        );
        assert_eq!(out, words(&["what", "does", "deadly", "mean"]));
        assert_eq!(quoted, vec![false, false, true, false]);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_multi_word_curly_quoted_span_collapses_across_tokens() {
        // The opener glued to the front of the FIRST content word, the closer
        // glued to the back of the LAST — the common case when a quoted
        // phrase sits inside a larger surface string with no internal spaces
        // around the quote marks.
        let quotes = quote_glyphs::load();
        let sentence_initial = vec![true, false, false, false, false];
        let (out, quoted, _sentence_initial) = collapse_quoted_spans(
            words(&["what", "does", "\u{201C}turkish", "bath\u{201D}", "mean"]),
            sentence_initial,
            &quotes,
        );
        assert_eq!(out, words(&["what", "does", "turkish bath", "mean"]));
        assert_eq!(quoted, vec![false, false, true, false]);
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn an_unclosed_quote_falls_through_untouched() {
        // No legitimate closer anywhere in the remaining tokens — the opener
        // stays glued to its word rather than collapsing incorrectly.
        let quotes = quote_glyphs::load();
        let (out, quoted, _sentence_initial) = collapse_quoted_spans(
            words(&["\u{201C}deadly", "mean"]),
            vec![true, false],
            &quotes,
        );
        assert_eq!(out, words(&["\u{201C}deadly", "mean"]));
        assert_eq!(quoted, vec![false, false]);
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn ascii_quote_marks_are_out_of_scope_for_this_pass() {
        // The two ASCII marks are QuoteRole::Ambiguous, not Initial — the UCD
        // itself does not encode their directionality, so this pass leaves
        // them for `flush_word`'s existing punctuation trim rather than
        // guessing a pairing.
        let quotes = quote_glyphs::load();
        let (out, quoted, _sentence_initial) =
            collapse_quoted_spans(words(&["\"deadly\"", "mean"]), vec![true, false], &quotes);
        assert_eq!(out, words(&["\"deadly\"", "mean"]));
        assert_eq!(quoted, vec![false, false]);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn collapses_a_capitalized_run_into_one_proper_noun_and_skips_a_singleton() {
        // "New York" (a contiguous 2-word run) collapses; the later,
        // non-adjacent singleton "Care" does not — singletons already reach
        // a proper-noun ALTERNATIVE through the OOV path in
        // `tokenize_with_alternatives`, so collapsing here would be
        // redundant, not a new capability.
        let (out, forcing, _) = collapse_capitalized_runs(
            words(&["the", "New", "York", "program", "for", "Care"]),
            vec![false; 6],
            vec![false; 6],
            &|_| false,
        );
        assert_eq!(out, words(&["the", "New York", "program", "for", "Care"]));
        assert_eq!(
            forcing,
            vec![
                NpForcing::Lexical,
                NpForcing::ProperNounRun,
                NpForcing::Lexical,
                NpForcing::Lexical,
                NpForcing::Lexical,
            ]
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_sentence_initial_capitalized_run_does_not_collapse() {
        // Ordinary sentence-initial capitalization carries no entity
        // signal — this is the exact false-positive risk the ~11%
        // Title-Case-styled slice of this corpus creates for a naive
        // capitalization-run detector.
        let (out, forcing, _) = collapse_capitalized_runs(
            words(&["Home", "Health", "services", "explained"]),
            vec![false, false, false, false],
            vec![true, false, false, false],
            &|_| false,
        );
        assert_eq!(out, words(&["Home", "Health", "services", "explained"]));
        assert_eq!(forcing, vec![NpForcing::Lexical; 4]);
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_title_case_styled_sentence_never_collapses_even_mid_sentence() {
        // A real regression this exact input caused before this guard
        // existed: "Happens If We Sell Her House" is Title-Case STYLE, not
        // six words of entity evidence — the sentence-initial exclusion
        // alone only protects "What", not the rest of the headline-style
        // capitalization that follows it.
        let (out, forcing, _) = collapse_capitalized_runs(
            words(&["What", "Happens", "If", "We", "Sell", "Her", "House"]),
            vec![false; 7],
            vec![true, false, false, false, false, false, false],
            &|_| false,
        );
        assert_eq!(
            out,
            words(&["What", "Happens", "If", "We", "Sell", "Her", "House"]),
            "nothing collapses once the whole input is judged Title-Case styled"
        );
        assert_eq!(forcing, vec![NpForcing::Lexical; 7]);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ordinary_prose_with_one_capitalized_run_still_collapses() {
        // The positive control for the title-case guard: a low overall
        // capitalization density (most words lowercase) must not suppress a
        // genuine mid-sentence run.
        let (out, forcing, _) = collapse_capitalized_runs(
            words(&[
                "does", "the", "New", "York", "program", "cover", "respite", "care",
            ]),
            vec![false; 8],
            vec![true, false, false, false, false, false, false, false],
            &|_| false,
        );
        assert_eq!(
            out,
            words(&[
                "does", "the", "New York", "program", "cover", "respite", "care"
            ])
        );
        assert_eq!(
            forcing,
            vec![
                NpForcing::Lexical,
                NpForcing::Lexical,
                NpForcing::ProperNounRun,
                NpForcing::Lexical,
                NpForcing::Lexical,
                NpForcing::Lexical,
                NpForcing::Lexical,
            ]
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_quoted_span_breaks_an_otherwise_adjacent_capitalized_run() {
        // "Home" / "Health" flank a quoted "About" — without the quoted
        // exclusion this would collapse into one 3-word span; instead each
        // singleton is too short to collapse on its own, so nothing merges.
        let (out, forcing, _) = collapse_capitalized_runs(
            words(&["Home", "About", "Health"]),
            vec![false, true, false],
            vec![false, false, false],
            &|_| false,
        );
        assert_eq!(out, words(&["Home", "About", "Health"]), "no merge occurs");
        assert_eq!(
            forcing,
            vec![
                NpForcing::Lexical,
                NpForcing::QuotedMention,
                NpForcing::Lexical
            ],
            "the quoted flag still passes through as a QuotedMention (never a \
             ProperNounRun — a quoted mention gets no bare-N reading)"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_registry_known_full_run_defers_to_the_richer_downstream_recognizer() {
        // A confirmed real regression before this guard existed: "Residential
        // Habilitation" is a registered program name whose statutory gloss
        // was silently lost once fused into an anonymous NP here, since
        // `collapse_multiword_surfaces` (the reasoner-aware recognizer
        // downstream) never gets a second, already-merged token to
        // re-classify.
        let (out, forcing, _) = collapse_capitalized_runs(
            words(&["Residential", "Habilitation", "services"]),
            vec![false; 3],
            vec![true, false, false],
            &|s| s == "residential habilitation",
        );
        assert_eq!(out, words(&["Residential", "Habilitation", "services"]));
        assert_eq!(forcing, vec![NpForcing::Lexical; 3]);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_registry_known_word_is_excluded_from_an_otherwise_eligible_run() {
        // "EVV" alone has its own registered statutory gloss ("Time4Care
        // EVV" is NOT itself a registered surface) — bundling it into an
        // unrelated neighbor's span would swallow that gloss, so "EVV" is
        // excluded from the run and "Time4Care" is left a singleton (too
        // short to collapse on its own).
        let (out, forcing, _) = collapse_capitalized_runs(
            words(&["using", "Time4Care", "EVV"]),
            vec![false; 3],
            vec![false, false, false],
            &|s| s == "evv",
        );
        assert_eq!(out, words(&["using", "Time4Care", "EVV"]));
        assert_eq!(forcing, vec![NpForcing::Lexical; 3]);
    }

    // ---- defines-lens gap G5: coordinated close-apposition definienda ----

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn apposition_coordinator_markers_are_recognized_and_nothing_else_is() {
        assert!(is_apposition_coordinator_marker(
            APPOSITION_COORDINATOR_MARKER_AND
        ));
        assert!(is_apposition_coordinator_marker(
            APPOSITION_COORDINATOR_MARKER_OR
        ));
        for w in ["and", "or", "consumer", "", "apposition-coordinator-and"] {
            assert!(
                !is_apposition_coordinator_marker(w),
                "{w:?} must not be mistaken for a reserved apposition-coordinator marker"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn apposition_coordinator_canonical_covers_only_its_own_markers() {
        assert_eq!(
            apposition_coordinator_canonical(APPOSITION_COORDINATOR_MARKER_AND),
            Some("and")
        );
        assert_eq!(
            apposition_coordinator_canonical(APPOSITION_COORDINATOR_MARKER_OR),
            Some("or")
        );
        // DISJOINT from `nominal_coordinator_canonical`'s own set — the
        // literal surface, and the list-coordinator markers, must NOT
        // ALSO be apposition-coordinator markers (the disambiguation this
        // whole mechanism exists for).
        for w in [
            "and",
            "or",
            LIST_COORDINATOR_MARKER_AND,
            LIST_COORDINATOR_MARKER_OR,
            "",
        ] {
            assert_eq!(
                apposition_coordinator_canonical(w),
                None,
                "{w:?} must not resolve as an apposition-coordinator marker"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn mark_apposition_coordinators_replaces_and_or_directly_between_two_quoted_spans() {
        // "the terms 'consumer' and 'individual' mean ..." — "and" sits
        // directly between two quoted spans.
        let ws = words(&["the", "terms", "consumer", "and", "individual", "mean"]);
        let quoted = vec![false, false, true, false, true, false];
        let sentence_initial = vec![true, false, false, false, false, false];
        let (out, out_quoted, out_sentence_initial) =
            mark_apposition_coordinators(ws, quoted.clone(), sentence_initial.clone());
        assert_eq!(
            out,
            vec![
                "the",
                "terms",
                "consumer",
                APPOSITION_COORDINATOR_MARKER_AND,
                "individual",
                "mean",
            ]
        );
        assert_eq!(out_quoted, quoted, "no token was added or removed");
        assert_eq!(out_sentence_initial, sentence_initial);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn mark_apposition_coordinators_bridges_a_repeated_the_term_prefix() {
        // "the term 'consumer' and the term 'individual' mean ..." — "and"
        // is followed by a REPEATED "the term" before the second quoted
        // span; the bridged tokens are elided.
        let ws = words(&[
            "the",
            "term",
            "consumer",
            "and",
            "the",
            "term",
            "individual",
            "mean",
        ]);
        let quoted = vec![false, false, true, false, false, false, true, false];
        let sentence_initial = vec![true, false, false, false, false, false, false, false];
        let (out, out_quoted, _) = mark_apposition_coordinators(ws, quoted, sentence_initial);
        assert_eq!(
            out,
            vec![
                "the",
                "term",
                "consumer",
                APPOSITION_COORDINATOR_MARKER_AND,
                "individual",
                "mean",
            ],
            "the bridged \"the\"/\"term\" tokens are elided"
        );
        assert_eq!(
            out_quoted,
            vec![false, false, true, false, true, false],
            "quoted stays aligned with the shortened token stream"
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn mark_apposition_coordinators_leaves_an_ordinary_coordinated_subject_untouched() {
        // "the term 'consumer', the county, and the state cover this
        // benefit." — only ONE conjunct is quoted; "and" is preceded by
        // "county" (not quoted), so nothing is marked. The REAL, measured
        // regression `grounding::definiendum_words`'s own doc documents.
        let ws = words(&[
            "the", "term", "consumer", "the", "county", "and", "the", "state", "cover",
        ]);
        let quoted = vec![false, false, true, false, false, false, false, false, false];
        let sentence_initial = vec![true, false, false, false, false, false, false, false, false];
        let (out, out_quoted, _) =
            mark_apposition_coordinators(ws.clone(), quoted.clone(), sentence_initial.clone());
        assert_eq!(out, ws, "nothing is marked; the token stream is unchanged");
        assert_eq!(out_quoted, quoted);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn skip_the_term_prefix_recognizes_the_closed_singular_and_plural_pair() {
        let ws = words(&["the", "term", "consumer"]);
        assert_eq!(skip_the_term_prefix(&ws, 0), 2);
        let ws_plural = words(&["the", "terms", "consumer"]);
        assert_eq!(skip_the_term_prefix(&ws_plural, 0), 2);
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn skip_the_term_prefix_leaves_an_ordinary_np_untouched() {
        let ws = words(&["the", "county", "cover"]);
        assert_eq!(
            skip_the_term_prefix(&ws, 0),
            0,
            "\"the county\" is not the Dictionary-Act boilerplate"
        );
    }

    // =========================================================================
    // Gap A: OOV/acronym compound-modifier N/N — an OOV singleton (no lexicon
    // entry) already offers the proper-noun NP reading via the "OUT-OF-
    // VOCABULARY word" block above; it now ALSO offers the nominal-premodifier
    // N/N reading, sourced from the SAME loaded "Noun" class projection a real
    // lexicon Noun entry gets (`category_projection::categories_for_class`),
    // so "PCA program"/"evv administrator" derive as N/N + N -> N (Hockenmaier
    // & Steedman 2005, CCGbank User's Manual §3.6.1/§3.6.2; Levi 1978; Selkirk
    // 1982; Huddleston & Pullum 2002 Ch.19).
    // =========================================================================

    /// A tiny WordNet with just enough content words ("program",
    /// "administrator") to test OOV compound-modifier behavior against a
    /// following IN-vocabulary noun, without pulling in the full loaded
    /// English corpus — "pca"/"evv" are deliberately absent so they stay
    /// genuinely out-of-vocabulary. Function words ("the", "an") are built
    /// automatically by `English::from_wordnet`, mirroring `tests.rs`'s own
    /// `sample_lang()`.
    fn oov_compound_test_lang() -> crate::cognitive::linguistics::english::English {
        const LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="test" label="Test" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-program-n"><Lemma writtenForm="program" partOfSpeech="n"/><Sense id="p1" synset="s-program"/></LexicalEntry>
    <LexicalEntry id="e-administrator-n"><Lemma writtenForm="administrator" partOfSpeech="n"/><Sense id="a1" synset="s-administrator"/></LexicalEntry>
    <Synset id="s-program" partOfSpeech="n" members="e-program-n"><Definition>a planned series of activities</Definition></Synset>
    <Synset id="s-administrator" partOfSpeech="n" members="e-administrator-n"><Definition>one who administers</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let wn = crate::social::software::markup::xml::lmf::reader::read_wordnet(LMF).expect("LMF");
        crate::cognitive::linguistics::english::English::from_wordnet(&wn)
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn an_oov_singleton_also_offers_nominal_premodifier_alongside_proper_noun() {
        let lang = oov_compound_test_lang();
        let (tokens, alts) = tokenize_with_alternatives("the PCA program", &lang);
        assert_eq!(tokens[1].word, "pca", "the OOV token is lowercased");
        assert!(
            alts[1].contains(&svo_types::proper_noun()),
            "the pre-existing OOV proper-noun alternative must still be offered: {:?}",
            alts[1]
        );
        assert!(
            alts[1].contains(&svo_types::nominal_modifier_noun()),
            "the new OOV nominal-premodifier N/N alternative must be offered: {:?}",
            alts[1]
        );
    }

    /// A lexicon-known abbreviation ending in a period ("U.S.C.", the exact
    /// citation shape `defines_pointers` parses constantly in real USC
    /// prose — 42 U.S.C. § 1395x(r) etc.) must not be misread as ending the
    /// sentence: the word immediately after it needs `sentence_initial ==
    /// false`, or `assign_type`'s `is_clause_initial` branch mints it a
    /// spurious extra question-copula chart alternative whenever that next
    /// word is a Copula/Auxiliary ("shall", "is", "does", …) — exactly the
    /// mechanism identified as a likely driver of the catastrophic
    /// (~30,000x baseline) chart-parse blowup on specific USC provisions
    /// during the `pr4xis compile --defines --lock` regen.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn trailing_period_abbreviation_does_not_falsely_end_a_sentence() {
        const LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="test" label="Test" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-usc"><Lemma writtenForm="U.S.C." partOfSpeech="n"/><Sense id="u1" synset="s-usc"/></LexicalEntry>
    <Synset id="s-usc" partOfSpeech="n" members="e-usc"><Definition>United States Code</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let wn = crate::social::software::markup::xml::lmf::reader::read_wordnet(LMF).expect("LMF");
        let lang = crate::cognitive::linguistics::english::English::from_wordnet(&wn);
        let vocab = operators::vocabulary();
        let dashes = dash_punctuation::vocabulary();
        let (words, sentence_initial, _comma_after, _semicolon_after) =
            surface_tokens_with_sentence_bounds("see 42 U.S.C. shall apply", vocab, dashes, &lang);
        let shall_idx = words
            .iter()
            .position(|w| w == "shall")
            .expect("\"shall\" survives tokenization");
        assert!(
            !sentence_initial[shall_idx],
            "a mid-clause abbreviation must not read as a sentence boundary: {words:?} / {sentence_initial:?}"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn an_oov_lowercase_modifier_also_offers_nominal_premodifier_not_capitalization_gated() {
        // A lowercase, non-capitalized OOV modifier ("evv" in "the evv
        // administrator") must ALSO get N/N — proving the fix reuses the
        // existing lexicon-membership OOV gate rather than a new, narrower
        // capitalization/ALL-CAPS heuristic.
        let lang = oov_compound_test_lang();
        let (tokens, alts) = tokenize_with_alternatives("the evv administrator", &lang);
        assert_eq!(tokens[1].word, "evv");
        assert!(
            alts[1].contains(&svo_types::nominal_modifier_noun()),
            "a lowercase OOV modifier must offer N/N too: {:?}",
            alts[1]
        );
    }
}
