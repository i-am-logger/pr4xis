#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::supertag_costs::{DerivationCost, SupertagCostTable};
use super::types::{LambekType, reduce};

/// Whether a token's surface is USED — contributing its ordinary denotation,
/// the default for every word in running prose — or MENTIONED, denoting the
/// EXPRESSION ITSELF rather than what that expression ordinarily names.
///
/// Quine's use/mention distinction (W. V. O. Quine (1940), *Mathematical
/// Logic*, Harvard University Press, §4 "Use versus mention", p. 23–26;
/// Cappelen & Lepore, "Quotation", *Stanford Encyclopedia of Philosophy*,
/// §3.1, <https://plato.stanford.edu/entries/quotation/> — the SAME pair of
/// sources [`super::montague::PredProvenance`]'s own doc already cites for
/// the sibling "a mentioned/queried expression is a singular term"
/// distinction it draws at the predicate level). In `“radiation” means
/// ionizing … radiation`, the subject MENTIONS the word while the definiens
/// USES it: the two occurrences of the same surface denote different things,
/// and no property of the surface itself can tell them apart.
///
/// First-class on the token (never re-derived downstream from punctuation or
/// from the word's spelling) because the tokenizer is the ONLY stage that can
/// still see the evidence: `tokenize::collapse_quoted_spans` folds a
/// quoted span into ONE token and DISCARDS its quote glyphs, so by the time a
/// parser, an interpreter, or a grounding lens sees the token, "was this
/// quoted?" is unanswerable — the exact information loss that let
/// `statute_structure::grounding::defines_pointers` read a definiendum off an
/// ordinary used subject ("Any benefit **provided** under subsection (c) …")
/// and mint a `defines` edge for a word the provision never defines.
///
/// A rich two-valued enum, never a bare `quoted: bool` — the same reason
/// [`super::tokenize`]'s own `NpForcing` replaced the lossy `force_np: bool`
/// it documents at its definition: a boolean at this boundary names the
/// EVIDENCE (a quote glyph was seen) rather than the CLAIM (this expression
/// is mentioned, not used), and only the claim is what any downstream stage
/// can reason with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExpressionUse {
    /// Ordinary occurrence: the surface contributes its own denotation.
    #[default]
    Used,
    /// Metalinguistic occurrence: the surface denotes ITSELF (the
    /// expression), not what it ordinarily names.
    Mentioned,
}

/// A typed token — a word, its Lambek type assignment, and whether that word
/// is used or mentioned ([`ExpressionUse`]).
#[derive(Debug, Clone, PartialEq)]
pub struct TypedToken {
    pub word: String,
    pub lambek_type: LambekType,
    /// Used (ordinary) or mentioned (metalinguistic) — see [`ExpressionUse`].
    pub expression_use: ExpressionUse,
}

/// Result of attempting to reduce a sequence of typed tokens.
#[derive(Debug, Clone)]
pub struct ReductionResult {
    pub success: bool,
    pub final_type: Option<LambekType>,
    pub remaining: Vec<TypedToken>,
    /// How many loaded unary (type-changing) rule applications the winning
    /// derivation used — 0 whenever a pure-application derivation won (the
    /// un-raised reading), and always 0 while the loaded cost table carries no
    /// unary rows. First-class so honesty tests can assert a marked rule did
    /// NOT carry a reading it should not have.
    pub unary_steps: usize,
}

/// CYK chart parser for Lambek grammars with lexical ambiguity — a WEIGHTED
/// chart with a deterministic derivation-preference order, not a boolean
/// recognizer.
///
/// Standard algorithm from the literature:
/// - Goodman, "Semiring Parsing" (1999) — the SAME CYK, evaluated with an
///   ordered weight per (span, type) instead of bare membership. The weight
///   is lexicographic (`CellEntry::key`): fewest loaded type-changing
///   steps (type-changing is the marked, non-combinatory option —
///   Hockenmaier & Steedman 2007 §4.6), then the committed chart's
///   leftmost-split preference (derivation-choice compatibility), then
///   lowest total cost under the LOADED negative-log CCGbank frequencies
///   ([`supertag_costs`](super::supertag_costs)). The key compares each
///   step's TOP split locally (greedy, matching the boolean chart's
///   committed choice), so it selects a preferred derivation, not a global
///   cost optimum.
/// - Hepple, "Chart Parsing Lambek Grammars" (1992)
/// - Moroz, "A Savateev-Style Parsing Algorithm for Pregroup Grammars" (2009)
///
/// Each word has a SET of possible types (from the lexicon). The chart tries
/// ALL combinations simultaneously via dynamic programming; loaded unary
/// (type-changing) rule rows close each cell as a fixpoint that can never
/// displace a rewrite-free derivation. A sentence is grammatical iff
/// S ∈ chart[0, n]; the goal is tiered — interrogative-featured (`S[q]`/`S[wq]`),
/// then other featured, then bare S(None) — because the newswire unigram
/// costs price wh categories by their rarity (`S[wq]` subject wh: count 8) and
/// would otherwise demote question readings; within a tier the preference
/// key decides, then the types' total order. Fully deterministic, unlike the
/// first-writer backpointers this replaces (whose hash-seeded iteration
/// order made same-tier tie choices flicker run-to-run).
///
/// Complexity: O(n³ × K²) where K = max types per word.
/// For natural language (K ≤ 10, n ≤ 20): trivially real-time.
///
/// `type_sets` provides all possible types for each token position.
/// `type_sets[i]` = all Lambek types that word_i could have.
pub fn chart_reduce(words: &[String], type_sets: &[Vec<LambekType>]) -> ReductionResult {
    #[cfg(feature = "std")]
    {
        chart_reduce_with_costs(
            words,
            type_sets,
            super::supertag_costs::supertag_cost_table(),
        )
    }
    #[cfg(not(feature = "std"))]
    {
        // No process cache without `std`: rebuild the loaded table by value —
        // the same tracked degradation the OLiA→CCG functor path has.
        chart_reduce_with_costs(words, type_sets, &super::supertag_costs::build_table())
    }
}

/// How a chart entry was derived — the backpointer of the min-cost chart.
///
/// The derived total order participates in the deterministic tie-break: a
/// lexical leaf, then a unary rewrite, then binary splits by structure.
/// Every tie resolves identically on every run — replacing the seed-dependent
/// first-writer choice.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Back {
    /// A lexical leaf assignment.
    Lex,
    /// A loaded unary (type-changing) rule applied over the same span.
    Unary { from: LambekType },
    /// Binary application: left over `[i..split]`, right over `[split..j]`.
    Binary {
        split: usize,
        left: LambekType,
        right: LambekType,
    },
}

impl Back {
    /// The split this derivation step committed to, for the leftmost-split
    /// preference: leaf/unary steps carry none (rank 0), a binary step ranks
    /// by its split point.
    fn split_rank(&self) -> usize {
        match self {
            Back::Lex | Back::Unary { .. } => 0,
            Back::Binary { split, .. } => *split,
        }
    }
}

/// A chart cell entry: the preferred derivation of a type over a span.
struct CellEntry {
    /// Loaded unary (type-changing) applications along the whole derivation.
    unary: usize,
    cost: DerivationCost,
    back: Back,
}

impl CellEntry {
    /// The derivation-preference key, most-significant first:
    ///
    /// 1. FEWEST type-changing steps — a marked, non-combinatory rule
    ///    (Hockenmaier & Steedman 2007 §4.6) may never displace a derivation
    ///    that does without it; this is the structural guarantee that makes
    ///    loading `N → NP` safe.
    /// 2. LEFTMOST split (of the TOP step, compared locally — a greedy
    ///    preference, not a global optimum) — the committed boolean chart's
    ///    first-writer fired at the smallest split of the k-ascending fill
    ///    loop, and the corpus behavior grew around that systematic choice
    ///    (e.g. the copula reading of "what is able", split 1, over the
    ///    degenerate wh-determiner + noun-homonym reading of "is", split 2,
    ///    which the newswire unigram costs would prefer). Kept explicit so
    ///    the weighted chart preserves the recognizer's derivation choice.
    /// 3. LOWEST cost — the loaded negative-log CCGbank frequencies order
    ///    what structure leaves open (the residue the boolean chart resolved
    ///    by hash-seeded iteration order: the observed ±2 corpus flake).
    /// 4. The backpointer's structural order — a total order, so every tie
    ///    resolves identically on every run.
    fn key(&self) -> (usize, usize, DerivationCost, &Back) {
        (self.unary, self.back.split_rank(), self.cost, &self.back)
    }
}

/// A chart cell: type → preferred derivation. The weighted analogue of the
/// boolean chart's `HashSet<LambekType>`.
type Cell = hashbrown::HashMap<LambekType, CellEntry>;

/// Chart ⊕: keep the preferred derivation of `t` under `CellEntry::key`
/// (deterministic, iteration-order-independent). Returns whether the cell
/// improved.
fn relax(cell: &mut Cell, t: LambekType, candidate: CellEntry) -> bool {
    match cell.entry(t) {
        hashbrown::hash_map::Entry::Vacant(v) => {
            v.insert(candidate);
            true
        }
        hashbrown::hash_map::Entry::Occupied(mut o) => {
            if candidate.key() < o.get().key() {
                o.insert(candidate);
                true
            } else {
                false
            }
        }
    }
}

/// Close a cell under the LOADED unary (type-changing) rule rows — a fixpoint,
/// terminating because each application strictly increases the unary count
/// (the most-significant component of `CellEntry::key`), so relaxation is
/// monotone decreasing in the preference order.
fn close_unary(cell: &mut Cell, table: &SupertagCostTable) {
    if table.unary_rules().is_empty() {
        return;
    }
    loop {
        let mut candidates: Vec<(LambekType, CellEntry)> = Vec::new();
        for (t, e) in cell.iter() {
            for rule in table.unary_rules() {
                if rule.from == *t {
                    candidates.push((
                        rule.to.clone(),
                        CellEntry {
                            unary: e.unary + 1,
                            cost: e.cost.plus(rule.cost),
                            back: Back::Unary { from: t.clone() },
                        },
                    ));
                }
            }
        }
        let mut changed = false;
        for (t, candidate) in candidates {
            changed |= relax(cell, t, candidate);
        }
        if !changed {
            break;
        }
    }
}

/// The Viterbi min-cost chart over an explicit loaded cost table — the engine
/// behind [`chart_reduce`], parameterized so tests can drive it with a table
/// parsed from a fixture through the SAME generic loader.
/// The chart-width DoS-avoidance bound [`chart_reduce_with_costs`] enforces
/// by default: the CYK chart is O(n²) space and O(n³) time, so an unbounded
/// token count (a pathologically long utterance) is a resource-exhaustion
/// DoS on the user-facing chat path. Real chat questions are far under this
/// bound; past it, refuse gracefully (abstain) rather than allocate (n+1)²
/// cells and hang.
///
/// [`chart_reduce_with_costs_bounded`] is the one other entry point allowed
/// a DIFFERENT bound — an explicit, caller-supplied one, for a caller whose
/// own call path does NOT run on the live per-turn chat path (see
/// [`crate::social::judicial::statute_structure::grounding::defines_pointers`]'s
/// own doc, which needs a wider bound for real, long, corpus-build-time-only
/// statutory sentences and explains why widening THIS shared constant
/// instead would not be safe).
const MAX_CHART_WIDTH: usize = 256;

pub fn chart_reduce_with_costs(
    words: &[String],
    type_sets: &[Vec<LambekType>],
    table: &SupertagCostTable,
) -> ReductionResult {
    chart_reduce_with_costs_bounded(words, type_sets, table, MAX_CHART_WIDTH)
}

/// [`chart_reduce_with_costs`], but over an EXPLICIT caller-supplied
/// chart-width bound instead of the shared `MAX_CHART_WIDTH` — see that
/// constant's own doc for which callers may use this and why. Every other
/// caller keeps using [`chart_reduce_with_costs`] (the shared bound)
/// unchanged.
pub fn chart_reduce_with_costs_bounded(
    words: &[String],
    type_sets: &[Vec<LambekType>],
    table: &SupertagCostTable,
    max_width: usize,
) -> ReductionResult {
    let Some(chart) = build_chart(words, type_sets, table, max_width) else {
        return ReductionResult {
            success: false,
            final_type: None,
            remaining: Vec::new(),
            unary_steps: 0,
        };
    };
    let n = words.len();

    // Step 3: the goal, at the WHOLE-string cell. The only thing separating
    // this from `clause_fragments_with_costs_bounded` is which cell is read.
    let goal = select_sentence_goal(&chart[0][n]);

    let success = goal.is_some();
    let (final_type, unary_steps) = match goal {
        Some((t, unary)) => (Some(t), unary),
        None => (None, 0),
    };

    // Step 4: Backtrack the WINNING derivation for the lexical assignment
    // Montague interprets.
    let remaining = if let Some(st) = &final_type {
        let mut winning_types = vec![None; n];
        extract_winning_types(0, n, st, &chart, &mut winning_types);

        words
            .iter()
            .enumerate()
            .map(|(i, w)| TypedToken {
                expression_use: ExpressionUse::Used,
                word: w.clone(),
                lambek_type: winning_types[i]
                    .clone()
                    .unwrap_or_else(|| type_sets[i][0].clone()),
            })
            .collect()
    } else {
        words
            .iter()
            .zip(type_sets.iter())
            .map(|(w, types)| TypedToken {
                expression_use: ExpressionUse::Used,
                word: w.clone(),
                lambek_type: types
                    .first()
                    .cloned()
                    .unwrap_or(LambekType::Atom(super::types::AtomicType::N)),
            })
            .collect()
    };

    ReductionResult {
        success,
        final_type,
        remaining,
        unary_steps,
    }
}

/// Build the CYK chart — Steps 1 and 2 of
/// [`chart_reduce_with_costs_bounded`], factored out UNCHANGED so a second
/// goal criterion ([`clause_fragments_with_costs_bounded`]) can read the SAME
/// completed chart rather than re-parsing.
///
/// A CYK chart is a *well-formed substring table* (Sheil, "Observations on
/// Context-Free Parsing", Statistical Methods in Linguistics 1976): by
/// construction `chart[i][j]` already records the preferred derivation of
/// EVERY derivable sub-span, not only the whole string. Reading a sub-span
/// goal out of it therefore costs nothing beyond the parse the caller already
/// paid for — the sub-span analyses are not extra work, they are work the
/// whole-string goal throws away.
///
/// `None` when the input is empty or exceeds `max_width` (the caller's own
/// DoS-avoidance bound).
fn build_chart(
    words: &[String],
    type_sets: &[Vec<LambekType>],
    table: &SupertagCostTable,
    max_width: usize,
) -> Option<Vec<Vec<Cell>>> {
    let n = words.len();
    if n == 0 || n > max_width {
        return None;
    }

    // chart[i][j] = cheapest derivation per type for span words[i..j]
    let mut chart: Vec<Vec<Cell>> = vec![];
    for _ in 0..=n {
        let mut row = Vec::with_capacity(n + 1);
        for _ in 0..=n {
            row.push(Cell::new());
        }
        chart.push(row);
    }

    // Step 1: Initialize — every lexical type at its LOADED unigram cost,
    // then close the leaf cell under the loaded unary rules.
    for i in 0..n {
        for t in &type_sets[i] {
            let candidate = CellEntry {
                unary: 0,
                cost: table.lexical_cost(t),
                back: Back::Lex,
            };
            relax(&mut chart[i][i + 1], t.clone(), candidate);
        }
        close_unary(&mut chart[i][i + 1], table);
    }

    // Step 2: Fill chart bottom-up (CYK), relaxation per result type under the
    // derivation-preference key; binary application itself costs nothing in
    // the unigram model (the leaves already paid), so a combined derivation
    // sums its parts' costs and unary counts.
    for span in 2..=n {
        for i in 0..=(n - span) {
            let j = i + span;
            for k in (i + 1)..j {
                let left: Vec<(LambekType, DerivationCost, usize)> = chart[i][k]
                    .iter()
                    .map(|(t, e)| (t.clone(), e.cost, e.unary))
                    .collect();
                let right: Vec<(LambekType, DerivationCost, usize)> = chart[k][j]
                    .iter()
                    .map(|(t, e)| (t.clone(), e.cost, e.unary))
                    .collect();

                for (t_left, c_left, u_left) in &left {
                    for (t_right, c_right, u_right) in &right {
                        if let Some(t_result) = reduce(t_left, t_right) {
                            let candidate = CellEntry {
                                unary: u_left + u_right,
                                cost: c_left.plus(*c_right),
                                back: Back::Binary {
                                    split: k,
                                    left: t_left.clone(),
                                    right: t_right.clone(),
                                },
                            };
                            relax(&mut chart[i][j], t_result, candidate);
                        }
                    }
                }
            }
            close_unary(&mut chart[i][j], table);
        }
    }

    Some(chart)
}

/// Step 3: Goal — the preferred S-family type in ONE chart cell, tiered:
///   1. featured beats bare S(None) (the committed baseline criterion —
///      featured types carry more information);
///   2. among featured, INTERROGATIVE (S[q]/S[wq]) beats non-interrogative:
///      cost must not pick across clause types, because the unigram prior
///      prices wh categories by their newswire rarity (S[wq] subject
///      wh: count 8) far above the wh-word's bare-NP pronoun homograph
///      (NP: 975), which would flip every "what is X" into a declarative
///      mis-parse — CCGbank annotates matrix wh-initial strings as S[wq],
///      and its free-relative readings carry NP/(S[dcl]\NP)-family
///      categories (Hockenmaier 2003 §5.12 p. 186), never a bare-NP
///      matrix subject "what";
///   3. within a tier, the preferred derivation under `CellEntry::key`
///      wins (fewest raises, then leftmost top split, then lowest cost);
///   4. exact ties fall to the types' total order (deterministic).
///
/// Factored out of [`chart_reduce_with_costs_bounded`]'s body UNCHANGED, so
/// that the whole-string goal and every sub-span goal
/// ([`clause_fragments_with_costs_bounded`]) apply the IDENTICAL criterion —
/// a partial parse must not be allowed to prefer a reading the whole-string
/// parse would have rejected.
fn select_sentence_goal(cell: &Cell) -> Option<(LambekType, usize)> {
    cell.iter()
        .filter(|(t, _)| matches!(t, LambekType::Atom(super::types::AtomicType::S(_))))
        .min_by(|(t1, e1), (t2, e2)| {
            use super::types::{AtomicType, SentenceFeature};
            let tier = |t: &LambekType| -> u8 {
                match t {
                    LambekType::Atom(AtomicType::S(Some(
                        SentenceFeature::Q | SentenceFeature::Wq,
                    ))) => 0,
                    LambekType::Atom(AtomicType::S(Some(_))) => 1,
                    _ => 2, // bare S(None)
                }
            };
            tier(t1)
                .cmp(&tier(t2))
                .then_with(|| e1.key().cmp(&e2.key()))
                .then_with(|| t1.cmp(t2))
        })
        .map(|(t, e)| (t.clone(), e.unary))
}

/// Backtrack the winning derivation to the per-word lexical assignment.
///
/// A unary rewrite over a LEAF span reports the rewritten (target) type as the
/// word's contribution — the type the winning spine actually consumed, which
/// is what the Montague extractor must re-reduce with binary application. A
/// unary rewrite over a longer span leaves the leaves' own types intact (the
/// rewrite happened above the phrase).
fn extract_winning_types(
    i: usize,
    j: usize,
    target: &LambekType,
    chart: &[Vec<Cell>],
    result: &mut [Option<LambekType>],
) {
    let Some(entry) = chart[i][j].get(target) else {
        return;
    };
    match &entry.back {
        Back::Lex => {
            if j == i + 1 {
                result[i] = Some(target.clone());
            }
        }
        Back::Unary { from } => {
            if j == i + 1 {
                result[i] = Some(target.clone());
            } else {
                extract_winning_types(i, j, from, chart, result);
            }
        }
        Back::Binary { split, left, right } => {
            extract_winning_types(i, *split, left, chart, result);
            extract_winning_types(*split, j, right, chart, result);
        }
    }
}

/// Build the per-token candidate type SETS `chart_reduce` needs: the primary
/// type plus every alternative, deduplicated — the shared plumbing
/// [`reduce_with_alternatives`] and [`reduce_with_alternatives_and_table`]
/// both build identically, factored out so the two entry points cannot drift.
fn build_type_sets(
    tokens: &[TypedToken],
    alternatives: &[Vec<LambekType>],
) -> (Vec<String>, Vec<Vec<LambekType>>) {
    let words: Vec<String> = tokens.iter().map(|t| t.word.clone()).collect();
    let type_sets: Vec<Vec<LambekType>> = tokens
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let mut types = vec![t.lambek_type.clone()];
            if let Some(alts) = alternatives.get(i) {
                for alt in alts {
                    if !types.contains(alt) {
                        types.push(alt.clone());
                    }
                }
            }
            types
        })
        .collect();
    (words, type_sets)
}

/// Reduce with ambiguity using the CYK chart parser.
///
/// Combines the primary type and all alternatives into type sets,
/// then runs the chart parser over all combinations simultaneously.
pub fn reduce_with_alternatives(
    tokens: &[TypedToken],
    alternatives: &[Vec<LambekType>],
) -> ReductionResult {
    let (words, type_sets) = build_type_sets(tokens, alternatives);
    carry_expression_use(chart_reduce(&words, &type_sets), tokens)
}

/// Re-attach each input token's [`ExpressionUse`] to the corresponding
/// re-typed output token.
///
/// The chart itself works over bare `(word, type-set)` pairs
/// ([`chart_reduce_with_costs`]'s own signature) — it re-types tokens, it
/// never re-orders or re-segments them: BOTH `remaining` constructions in
/// that function map over the SAME `words` slice positionally
/// ([`build_type_sets`] is likewise 1:1 with `tokens`), so position `i` out
/// is position `i` in. Without this step every derived token would come back
/// [`ExpressionUse::Used`] and the use/mention distinction would survive
/// tokenization only to be erased by the parser — the same silent loss
/// [`ExpressionUse`]'s own doc describes for quote glyphs.
///
/// The length guard is not defensive dressing: it is the one condition under
/// which positional carry-over is sound, and it is exactly the condition
/// `defines_pointers`'s own `reduction.remaining.len() == tokens.len()` check
/// already relies on before handing `remaining` to the interpreter.
fn carry_expression_use(mut result: ReductionResult, tokens: &[TypedToken]) -> ReductionResult {
    if result.remaining.len() == tokens.len() {
        for (out, src) in result.remaining.iter_mut().zip(tokens.iter()) {
            out.expression_use = src.expression_use;
        }
    }
    result
}

/// [`reduce_with_alternatives`], but over an EXPLICIT loaded cost table
/// rather than the shared production one ([`chart_reduce`]'s own
/// [`super::supertag_costs::supertag_cost_table`]) — needed by
/// [`crate::social::judicial::statute_structure::grounding::defines_pointers`],
/// which supplies a definiens-scoped table carrying a unary rule the SHARED
/// table deliberately does not (see
/// [`super::supertag_costs::SupertagCostTable::with_extra_unary`]'s own doc
/// for why). Every other caller keeps using [`reduce_with_alternatives`]
/// unchanged.
pub fn reduce_with_alternatives_and_table(
    tokens: &[TypedToken],
    alternatives: &[Vec<LambekType>],
    table: &super::supertag_costs::SupertagCostTable,
) -> ReductionResult {
    let (words, type_sets) = build_type_sets(tokens, alternatives);
    carry_expression_use(chart_reduce_with_costs(&words, &type_sets, table), tokens)
}

/// [`reduce_with_alternatives_and_table`], but over an EXPLICIT chart-width
/// bound too ([`chart_reduce_with_costs_bounded`]) — needed by
/// [`crate::social::judicial::statute_structure::grounding::defines_pointers`]'s
/// own scoped, corpus-build-time-only chart derivation (that function's own
/// doc has the full rationale). Every other caller keeps using
/// [`reduce_with_alternatives`]/[`reduce_with_alternatives_and_table`]
/// (the shared bound) unchanged.
pub fn reduce_with_alternatives_and_table_and_width(
    tokens: &[TypedToken],
    alternatives: &[Vec<LambekType>],
    table: &super::supertag_costs::SupertagCostTable,
    max_width: usize,
) -> ReductionResult {
    let (words, type_sets) = build_type_sets(tokens, alternatives);
    carry_expression_use(
        chart_reduce_with_costs_bounded(&words, &type_sets, table, max_width),
        tokens,
    )
}

/// A contiguous, half-open run of token positions `[start, end)`.
///
/// A rich type rather than a bare `(usize, usize)` pair for the same reason
/// [`ExpressionUse`] replaced `quoted: bool`: at the boundary between the
/// chart and its consumers a naked index pair names neither which array it
/// indexes nor which end is exclusive, so nothing downstream can reason with
/// it — and a partial parse's whole content is *which* run of the input it
/// covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TokenSpan {
    start: usize,
    end: usize,
}

impl TokenSpan {
    /// The run `[start, end)`. `end` is clamped to be at least `start`, so a
    /// `TokenSpan` can never denote a negative-length run.
    #[must_use]
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end: end.max(start),
        }
    }

    /// First covered position.
    #[must_use]
    pub const fn start(&self) -> usize {
        self.start
    }

    /// One past the last covered position.
    #[must_use]
    pub const fn end(&self) -> usize {
        self.end
    }

    /// How many token positions the run covers.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.end - self.start
    }

    /// Does the run cover no position at all?
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Does the run cover the WHOLE of an `n`-token input — i.e. is this
    /// "partial" parse in fact the total one?
    #[must_use]
    pub const fn is_whole(&self, n: usize) -> bool {
        self.start == 0 && self.end == n
    }

    /// The covered slice of `tokens`.
    #[must_use]
    pub fn slice<'a>(&self, tokens: &'a [TypedToken]) -> &'a [TypedToken] {
        &tokens[self.start..self.end.min(tokens.len())]
    }
}

/// One CLAUSE of a partial (chunk) parse: a maximal run of the input over
/// which the chart derives a complete S-family predication, together with the
/// per-token type assignment THAT derivation committed to.
///
/// See [`clause_fragments_with_costs_bounded`] for why a partial parse is the
/// right output shape and for the literature it comes from.
#[derive(Debug, Clone)]
pub struct ClauseFragment {
    /// Which run of the input the clause covers.
    pub span: TokenSpan,
    /// The S-family type derived over `span`.
    pub final_type: LambekType,
    /// `span`'s own tokens, re-typed by the winning derivation — exactly what
    /// [`ReductionResult::remaining`] carries for a whole-string parse, sliced
    /// to the clause.
    pub tokens: Vec<TypedToken>,
    /// Loaded unary (type-changing) applications the winning derivation used.
    pub unary_steps: usize,
}

/// Every maximal CLAUSE the chart derives over `words`, left to right — the
/// PARTIAL-PARSE goal, as opposed to
/// [`chart_reduce_with_costs_bounded`]'s all-or-nothing whole-string goal.
///
/// # Why a partial goal is a goal at all
///
/// A whole-string goal answers "does this entire string form one sentence?".
/// That is the wrong question for any consumer that needs a specific
/// PREDICATION out of running text: a single unattachable adjunct anywhere in
/// the string — a fronted participial preamble, a trailing infinitival
/// purpose clause, an appositive the grammar has no category for — makes
/// `chart[0][n]` empty and destroys an analysis the chart in fact completed
/// over the clause itself.
///
/// Abney ("Parsing By Chunks", in Berwick, Abney & Tenny (eds.),
/// *Principle-Based Parsing*, Kluwer 1991, §1–§3) is the standing argument
/// that these are two separable problems: a parser can identify constituents
/// reliably in exactly the places where it cannot resolve their ATTACHMENT,
/// and forcing one spanning analysis discards the constituents it got right
/// in order to report the attachment it did not. His chunker therefore emits
/// a SEQUENCE of non-overlapping chunks covering the input and leaves
/// unattached material unattached, rather than failing. Abney ("Partial
/// Parsing via Finite-State Cascades", *Natural Language Engineering* 2(4),
/// 1996, §1) states the selection rule this function uses: at each position,
/// take the LONGEST constituent that starts there.
///
/// This costs nothing extra. A CYK chart is a well-formed substring table
/// (Sheil 1976 — see `build_chart`): the sub-span analyses are already in
/// the chart the whole-string goal built and then ignored.
///
/// # The cover
///
/// Greedy, leftmost-longest, deterministic: from position `i`, take the
/// LONGEST `j` for which `select_sentence_goal` finds an S-family type over
/// `[i, j)`, emit that clause, and continue from `j`; when no clause starts at
/// `i`, advance one position (that token is residue this parse does not
/// attach). The clauses are therefore non-overlapping and in input order.
///
/// # Relationship to the whole-string goal
///
/// When the whole string DOES derive an S, the longest clause starting at 0
/// is `[0, n)` itself, so the cover is the single fragment
/// [`chart_reduce_with_costs_bounded`] would have returned, carrying the
/// SAME type (both call `select_sentence_goal`) and the SAME per-token
/// assignment (both call `extract_winning_types`). A consumer that switches
/// from the whole-string goal to this one therefore cannot lose a reading it
/// already had.
pub fn clause_fragments_with_costs_bounded(
    words: &[String],
    type_sets: &[Vec<LambekType>],
    table: &SupertagCostTable,
    max_width: usize,
) -> Vec<ClauseFragment> {
    let Some(chart) = build_chart(words, type_sets, table, max_width) else {
        return Vec::new();
    };
    let n = words.len();
    let mut fragments = Vec::new();
    let mut i = 0;
    while i < n {
        // Longest-match (Abney 1996 §1): the widest clause starting here.
        let clause = (i + 1..=n)
            .rev()
            .find_map(|j| select_sentence_goal(&chart[i][j]).map(|(t, u)| (j, t, u)));
        let Some((j, final_type, unary_steps)) = clause else {
            i += 1;
            continue;
        };
        let mut winning_types = vec![None; n];
        extract_winning_types(i, j, &final_type, &chart, &mut winning_types);
        let tokens = (i..j)
            .map(|p| TypedToken {
                expression_use: ExpressionUse::Used,
                word: words[p].clone(),
                lambek_type: winning_types[p]
                    .clone()
                    .unwrap_or_else(|| type_sets[p][0].clone()),
            })
            .collect();
        fragments.push(ClauseFragment {
            span: TokenSpan::new(i, j),
            final_type,
            tokens,
            unary_steps,
        });
        i = j;
    }
    fragments
}

/// [`clause_fragments_with_costs_bounded`] over the SAME `(tokens,
/// alternatives)` input shape [`reduce_with_alternatives_and_table_and_width`]
/// takes, re-attaching each input token's [`ExpressionUse`] to the
/// corresponding output token (the clause-sliced analogue of
/// `carry_expression_use` — same positional-carry soundness argument, since
/// a fragment's tokens are `words[span.start()..span.end()]` in order).
pub fn clause_fragments_with_alternatives_and_table_and_width(
    tokens: &[TypedToken],
    alternatives: &[Vec<LambekType>],
    table: &super::supertag_costs::SupertagCostTable,
    max_width: usize,
) -> Vec<ClauseFragment> {
    let (words, type_sets) = build_type_sets(tokens, alternatives);
    let mut fragments = clause_fragments_with_costs_bounded(&words, &type_sets, table, max_width);
    for fragment in &mut fragments {
        let start = fragment.span.start();
        for (offset, out) in fragment.tokens.iter_mut().enumerate() {
            if let Some(src) = tokens.get(start + offset) {
                out.expression_use = src.expression_use;
            }
        }
    }
    fragments
}

/// An unattachable adjunct never hides the clause it sits beside.
///
/// A whole-string parse goal answers "is this entire string one sentence?".
/// Asked of running text, that question conflates two independent ones —
/// which constituents are there, and how they attach — and answers BOTH
/// "no" whenever only the second fails. Abney's argument ("Parsing By
/// Chunks", in Berwick, Abney & Tenny (eds.), *Principle-Based Parsing*,
/// Kluwer 1991, §1–§3) is that a parser is at its most reliable about
/// constituency exactly where it is least reliable about attachment, so the
/// two must be reported separately; his chunker therefore emits a sequence of
/// chunks and leaves unattached material unattached rather than failing.
///
/// This axiom states the resulting closure over
/// [`clause_fragments_with_costs_bounded`] as a machine-checkable claim, with
/// no lexicon, VerbNet or corpus load — only the shipped cost table:
///
/// 1. A string the whole-string goal ([`chart_reduce_with_costs`]) accepts
///    yields EXACTLY ONE fragment, spanning the whole string, carrying the
///    same final type and the same per-token assignment. A partial parse can
///    therefore never lose a reading the total parse already had.
/// 2. Prepending, appending, or doing both with a token that combines with
///    NOTHING (a bare `N` beside an `S`, which no application rule reduces,
///    and which the shipped table carries no type-changing row for) destroys
///    the whole-string parse — and leaves the clause fragment itself exactly
///    as it was, at its shifted position.
///
/// That second half is the whole content of the fix it grounds: the fronted
/// participial preamble ("As used in this section, …") and the trailing
/// infinitival purpose clause ("… authorized by law to perform the duties
/// thereof") of a statutory definition are unattachable adjuncts of precisely
/// this shape, and under a whole-string goal each one silently discarded a
/// definiendum the chart had already fully analysed.
pub struct AnUnattachableAdjunctNeverHidesItsClause;

impl pr4xis::ontology::Axiom for AnUnattachableAdjunctNeverHidesItsClause {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};

        use super::types::svo;

        #[cfg(feature = "std")]
        let table = super::supertag_costs::supertag_cost_table();
        #[cfg(not(feature = "std"))]
        let table = &super::supertag_costs::build_table();

        let fail = || -> pr4xis::logic::proof::Verdict {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        };

        // The clause: `w1:NP w2:NP\S` → S, a complete predication.
        let clause_words = ["w1".to_string(), "w2".to_string()];
        let clause_types = vec![vec![LambekType::np()], vec![svo::intransitive_verb()]];

        // (1) The total parse and the partial parse agree exactly.
        let total = chart_reduce_with_costs(&clause_words, &clause_types, table);
        let fragments = clause_fragments_with_costs_bounded(&clause_words, &clause_types, table, 8);
        if !total.success || fragments.len() != 1 {
            return fail();
        }
        let only = &fragments[0];
        if !only.span.is_whole(clause_words.len())
            || Some(only.final_type.clone()) != total.final_type
            || only.tokens != total.remaining
        {
            return fail();
        }

        // (2) A bare `N` neighbour reduces with nothing (no application rule
        // applies to two atoms, and the shipped table carries no unary rows),
        // so it kills the whole-string goal and must not touch the clause.
        let adjunct = ("w0".to_string(), vec![LambekType::n()]);
        for (before, after) in [(1, 0), (0, 1), (1, 1)] {
            let mut words = Vec::new();
            let mut types = Vec::new();
            for _ in 0..before {
                words.push(adjunct.0.clone());
                types.push(adjunct.1.clone());
            }
            words.extend(clause_words.iter().cloned());
            types.extend(clause_types.iter().cloned());
            for _ in 0..after {
                words.push(adjunct.0.clone());
                types.push(adjunct.1.clone());
            }
            if chart_reduce_with_costs(&words, &types, table).success {
                // The premise of the claim — that this really is an
                // unattachable adjunct — does not hold; the check below would
                // prove nothing.
                return fail();
            }
            let shifted = clause_fragments_with_costs_bounded(&words, &types, table, 8);
            if shifted.len() != 1 {
                return fail();
            }
            let clause = &shifted[0];
            if clause.span != TokenSpan::new(before, before + clause_words.len())
                || clause.final_type != only.final_type
                || clause.tokens != only.tokens
            {
                return fail();
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "AnUnattachableAdjunctNeverHidesItsClause",
        "the partial-parse (chunk) goal returns exactly the whole-string parse when one exists, and returns the clause unchanged — same span content, same type, same per-token assignment — when an adjunct the grammar cannot attach is prepended or appended; constituency and attachment are reported separately, so a failure of the second never destroys the first",
        "Abney (1991) Parsing By Chunks, in Berwick, Abney & Tenny (eds.) Principle-Based Parsing, Kluwer, §1-§3; Abney (1996) Partial Parsing via Finite-State Cascades, Natural Language Engineering 2(4) §1 (longest-match chunk selection); Sheil (1976) Observations on Context-Free Parsing, Statistical Methods in Linguistics (the well-formed substring table: a CYK chart already records every derivable sub-span)"
    );
}
pr4xis::register_axiom!(AnUnattachableAdjunctNeverHidesItsClause, constructor);

#[cfg(all(test, feature = "std"))]
mod chart_tests {
    use super::*;
    use crate::cognitive::linguistics::lambek::supertag_costs::{parse_table, supertag_cost_table};
    use crate::cognitive::linguistics::lambek::types::svo;
    use alloc::string::ToString;

    fn words(ws: &[&str]) -> Vec<String> {
        ws.iter().map(|w| w.to_string()).collect()
    }

    /// `S/(NP\S)` — a deliberately unlisted category (hapax-priced).
    fn unseen_subject_slot() -> LambekType {
        LambekType::right_div(LambekType::s(), svo::intransitive_verb())
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_cheapest_derivation_wins_the_goal() {
        // Two whole-span derivations of the SAME bare-S goal compete:
        //   w1:S/(NP\S) (unlisted → hapax 10.72) + w2:NP\S → S   (≈16.00)
        //   w1:NP (loaded 3.84)                  + w2:NP\S → S   (≈ 9.12)
        // The boolean chart picked whichever its seed-dependent iteration
        // reached first; the Viterbi chart MUST report the cheap reading.
        let result = chart_reduce_with_costs(
            &words(&["w1", "w2"]),
            &[
                vec![unseen_subject_slot(), LambekType::np()],
                vec![svo::intransitive_verb()],
            ],
            supertag_cost_table(),
        );
        assert!(result.success);
        assert_eq!(result.remaining[0].lambek_type, LambekType::np());
        assert_eq!(result.unary_steps, 0);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn an_interrogative_goal_beats_a_cheaper_declarative_goal() {
        // "what is able" — the FIX-B adjective-define frame. TWO featured
        // whole-span goals compete: S[wq] via what:S[wq]/(NP\S) (count 8 →
        // 8.64 nats) and S[dcl] via the pronoun-row homograph what:NP (975 →
        // 3.84 nats) + copula_adj. Cost must NOT pick across clause types:
        // CCGbank annotates matrix wh-initial strings as S[wq] — its
        // free-relative readings carry NP/(S[dcl]\NP)-family categories
        // (Hockenmaier 2003 §5.12 p. 186), never bare NP — so the
        // interrogative tier outranks the cheaper declarative mis-parse.
        // (Gate-1 regression shape: define 932→1043 when cost crossed tiers.)
        let result = chart_reduce_with_costs(
            &words(&["what", "is", "able"]),
            &[
                vec![svo::wh_what(), LambekType::np()],
                vec![svo::copula_adj()],
                vec![svo::predicate_adjective()],
            ],
            supertag_cost_table(),
        );
        assert!(result.success);
        assert_eq!(result.final_type, Some(LambekType::wq()));
        assert_eq!(result.remaining[0].lambek_type, svo::wh_what());
        assert_eq!(result.remaining[2].lambek_type, svo::predicate_adjective());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_featured_goal_beats_a_cheaper_bare_goal() {
        // "what is a dog" in miniature: the wh-reading (count 8 → 8.64 nats)
        // is far DEARER than the bare-NP homograph (975 → 3.84 nats), so a
        // cost-primary goal would demote every wh-question to a statement.
        // Featured-over-bare stays PRIMARY; cost only ranks within a tier.
        let result = chart_reduce_with_costs(
            &words(&["what", "is-a-dog"]),
            &[
                vec![svo::wh_what(), LambekType::np()],
                vec![svo::intransitive_verb()],
            ],
            supertag_cost_table(),
        );
        assert!(result.success);
        assert_eq!(result.final_type, Some(LambekType::wq()));
        assert_eq!(result.remaining[0].lambek_type, svo::wh_what());
    }

    /// A fixture table parsed through the SAME generic loader, carrying the
    /// gate-2 unary row shape (CCGbank `NP → N`, 115,516 / 929,552 — the
    /// published numbers) so the unary machinery is proven BEFORE the loaded
    /// production table carries the row.
    fn fixture_with_bare_np_rule()
    -> crate::cognitive::linguistics::lambek::supertag_costs::SupertagCostTable {
        parse_table(
            "lex\tN\tN\t10498\t45422\t1.464812\n\
             lex\tNP\tNP\t975\t45422\t3.841314\n\
             lex\tNP/N\tNP[nb]/N\t4077\t45422\t2.410635\n\
             unary\tN\tNP\tNP -> N\t115516\t929552\t2.085294\n\
             default\t1\t45422\t10.723752\n",
        )
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_loaded_unary_row_parses_the_bare_plural_and_reports_the_raised_leaf() {
        // "dogs run": no derivation reaches S without the type-changing row —
        // the raise fires exactly where it is CORRECT (determiner-less
        // argument), and the leaf reports the raised NP so the Montague
        // extractor can re-reduce the spine with binary application alone.
        let table = fixture_with_bare_np_rule();
        let ts = [vec![LambekType::n()], vec![svo::intransitive_verb()]];
        let ws = words(&["dogs", "run"]);
        let without = chart_reduce_with_costs(
            &ws,
            &ts,
            &parse_table("lex\tN\tN\t10498\t45422\t1.464812\ndefault\t1\t45422\t10.723752\n"),
        );
        assert!(!without.success, "no raise, no parse — the red baseline");
        let with = chart_reduce_with_costs(&ws, &ts, &table);
        assert!(with.success, "the loaded row parses the bare plural");
        assert_eq!(with.final_type, Some(LambekType::s()));
        assert_eq!(with.unary_steps, 1);
        assert_eq!(with.remaining[0].lambek_type, LambekType::np());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_determiner_frame_never_pays_for_a_raise_it_does_not_need() {
        // "a dog runs" — the corpus's determiner-framed shape. The det path is
        // the only whole-span S, and the winning derivation uses ZERO unary
        // steps even with the raise on offer: the safety property whose
        // absence made the naive guarded N→NP regress the corpus (+273/+274).
        let result = chart_reduce_with_costs(
            &words(&["a", "dog", "runs"]),
            &[
                vec![svo::determiner()],
                vec![LambekType::n()],
                vec![svo::intransitive_verb()],
            ],
            &fixture_with_bare_np_rule(),
        );
        assert!(result.success);
        assert_eq!(result.unary_steps, 0, "the un-raised reading carries it");
        assert_eq!(result.remaining[1].lambek_type, LambekType::n());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_leftmost_split_beats_a_cheaper_derivation_at_a_larger_split() {
        // The gate-1 take-1 regression class in miniature: two whole-span
        // derivations of the SAME bare-S goal, the CHEAPER one at the larger
        // split. Cost-primary relaxation regressed the corpus (define
        // 932→1043) by preferring exactly such derivations; the preference
        // key must keep the committed chart's leftmost-split choice.
        //   k=1: w1:S/PP (hapax) + [w2 w3]:PP (N + N\PP hapax)   ≈ 22.91
        //   k=2: [w1 w2]:NP (NP/N + N) + w3:NP\S                 ≈  9.16
        let s_over_pp = LambekType::right_div(LambekType::s(), LambekType::pp());
        let n_under_pp = LambekType::left_div(LambekType::n(), LambekType::pp());
        let ts = [
            vec![s_over_pp.clone(), svo::determiner()],
            vec![LambekType::n()],
            vec![n_under_pp.clone(), svo::intransitive_verb()],
        ];
        let ws = words(&["w1", "w2", "w3"]);
        let table = supertag_cost_table();
        let result = chart_reduce_with_costs(&ws, &ts, table);
        assert!(result.success);
        assert_eq!(result.remaining[0].lambek_type, s_over_pp);
        assert_eq!(result.remaining[2].lambek_type, n_under_pp);
        // ...and the choice is input-order-invariant.
        let permuted = chart_reduce_with_costs(
            &ws,
            &[
                vec![svo::determiner(), s_over_pp.clone()],
                vec![LambekType::n()],
                vec![svo::intransitive_verb(), n_under_pp.clone()],
            ],
            table,
        );
        assert_eq!(result.final_type, permuted.final_type);
        assert_eq!(
            result
                .remaining
                .iter()
                .map(|t| &t.lambek_type)
                .collect::<Vec<_>>(),
            permuted
                .remaining
                .iter()
                .map(|t| &t.lambek_type)
                .collect::<Vec<_>>(),
        );
    }

    #[pr4xis::praxis_value(Deterministic, Verifiable)]
    #[test]
    fn equal_cost_ties_break_identically_regardless_of_input_order() {
        // Two whole-span bare-S derivations tie EXACTLY (both legs are
        // unlisted → 2 × hapax). The boolean chart's winner flickered with
        // hashbrown's per-process seeds (observed as ±2 corpus jitter between
        // identical runs); the Viterbi chart's Back-order tie-break must pick
        // the same winner however the alternatives are ordered.
        let a = LambekType::right_div(LambekType::s(), LambekType::pp()); // S/PP
        let b = LambekType::right_div(
            LambekType::s(),
            LambekType::left_div(LambekType::np(), LambekType::np()),
        ); // S/(NP\NP)
        let arg_a = LambekType::pp();
        let arg_b = LambekType::left_div(LambekType::np(), LambekType::np());
        let table = supertag_cost_table();
        let ws = words(&["w1", "w2"]);
        let one = chart_reduce_with_costs(
            &ws,
            &[
                vec![a.clone(), b.clone()],
                vec![arg_a.clone(), arg_b.clone()],
            ],
            table,
        );
        let two = chart_reduce_with_costs(&ws, &[vec![b, a], vec![arg_b, arg_a]], table);
        assert!(one.success && two.success);
        assert_eq!(one.final_type, two.final_type);
        assert_eq!(
            one.remaining
                .iter()
                .map(|t| &t.lambek_type)
                .collect::<Vec<_>>(),
            two.remaining
                .iter()
                .map(|t| &t.lambek_type)
                .collect::<Vec<_>>(),
        );
    }
}

#[cfg(test)]
mod dos_tests {
    use super::chart_reduce;
    use alloc::{format, string::String, vec::Vec};

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn unbounded_token_count_is_refused_not_a_resource_bomb() {
        // The CYK chart allocates (n+1)² type-sets + (n+1)² backpointers and
        // runs in O(n³); an unbounded token count is a memory/time DoS. A huge
        // count must abstain (success:false) immediately — without the
        // MAX_CHART_WIDTH bound this allocates gigabytes / hangs.
        let words: Vec<String> = (0..10_000).map(|i| format!("w{i}")).collect();
        let type_sets: Vec<Vec<_>> = words.iter().map(|_| Vec::new()).collect();
        let result = chart_reduce(&words, &type_sets);
        assert!(!result.success);
    }

    /// A CALLER-SUPPLIED wider bound
    /// ([`chart_reduce_with_costs_bounded`]) derives a genuinely
    /// well-formed derivation past the shared [`MAX_CHART_WIDTH`] the
    /// default [`chart_reduce_with_costs`] refuses — the exact scoped-bound
    /// need
    /// [`crate::social::judicial::statute_structure::grounding::defines_pointers`]'s
    /// own `DEFINES_MAX_CHART_WIDTH` doc describes. The derivation itself is
    /// a right-branching chain of Steedman's (2000) `S/S` sentence-modifier
    /// category — `w_i : S/S` for every position but the last, `w_last : S`
    /// — needing no real lexicon: each `S/S` forward-applies to the `S` its
    /// right neighbor eventually derives, a standard, unambiguous
    /// application chain.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_caller_supplied_wider_bound_accepts_what_the_shared_bound_refuses() {
        use super::{LambekType, chart_reduce_with_costs, chart_reduce_with_costs_bounded};
        use crate::cognitive::linguistics::lambek::supertag_costs::supertag_cost_table;

        let width = 300;
        let words: Vec<String> = (0..width).map(|i| format!("w{i}")).collect();
        let s_s = LambekType::right_div(LambekType::s(), LambekType::s());
        let mut type_sets: Vec<Vec<LambekType>> =
            (0..width - 1).map(|_| alloc::vec![s_s.clone()]).collect();
        type_sets.push(alloc::vec![LambekType::s()]);
        let table = supertag_cost_table();

        let shared = chart_reduce_with_costs(&words, &type_sets, table);
        assert!(
            !shared.success,
            "the shared 256-token bound refuses a {width}-token derivation, \
             even though the grammar itself could derive it"
        );

        let widened = chart_reduce_with_costs_bounded(&words, &type_sets, table, 512);
        assert!(
            widened.success,
            "a caller-supplied 512-token bound derives the SAME {width}-token \
             chain the shared bound refuses"
        );
        assert_eq!(widened.final_type, Some(LambekType::s()));
    }

    /// The partial-parse goal's own closure
    /// ([`AnUnattachableAdjunctNeverHidesItsClause`]) holds: it reproduces the
    /// whole-string parse exactly where one exists, and an adjunct the grammar
    /// cannot attach leaves the clause untouched.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn an_unattachable_adjunct_never_hides_its_clause() {
        use super::AnUnattachableAdjunctNeverHidesItsClause;
        use pr4xis::ontology::Axiom;
        assert!(AnUnattachableAdjunctNeverHidesItsClause.verify().is_ok());
    }

    /// The cover is a COVER: two clauses separated by material that attaches
    /// to neither are BOTH reported, in input order and disjoint — the shape
    /// a provision whose chapeau and definition never combine ("In this
    /// section: The term “X” means …") presents.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_clause_cover_reports_every_clause_not_just_the_first() {
        use super::{LambekType, clause_fragments_with_costs_bounded};
        use crate::cognitive::linguistics::lambek::supertag_costs::supertag_cost_table;
        use crate::cognitive::linguistics::lambek::types::svo;

        let words: Vec<String> = ["a1", "v1", "gap", "a2", "v2"]
            .iter()
            .map(|w| w.to_string())
            .collect();
        let type_sets = alloc::vec![
            alloc::vec![LambekType::np()],
            alloc::vec![svo::intransitive_verb()],
            alloc::vec![LambekType::n()],
            alloc::vec![LambekType::np()],
            alloc::vec![svo::intransitive_verb()],
        ];
        let fragments =
            clause_fragments_with_costs_bounded(&words, &type_sets, supertag_cost_table(), 16);
        let spans: Vec<(usize, usize)> = fragments
            .iter()
            .map(|f| (f.span.start(), f.span.end()))
            .collect();
        assert_eq!(spans, alloc::vec![(0, 2), (3, 5)]);
    }
}
