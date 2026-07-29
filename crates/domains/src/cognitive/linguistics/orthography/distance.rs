use pr4xis::category::{Arrow, Category, Concept, FinitelyGenerated};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

// Spelling Error Ontology — the science of misspelling.
//
// Spelling errors are classified on three orthogonal axes:
//
// 1. ETIOLOGY (WHY): competence vs performance errors
//    - Corder (1967), "The significance of learner errors"
//    - Coltheart (1981), Dual-route model of spelling
//    - Caramazza & Miceli (1990), Graphemic buffer model
//
// 2. LINGUISTIC LEVEL (WHAT knowledge is violated):
//    - POMAS framework (Silliman, Brimo 2013)
//    - Phonological, Orthographic, Morphological, Visual
//
// 3. OPERATION (HOW it manifests as string edits):
//    - Damerau (1964), Four basic edit operations
//    - Pollock & Zamora (1983), 90-95% are single-edit
//    - Brill & Moore (2000), String-to-string partition model
//
// The noisy channel model (Shannon 1948, applied by Kernighan, Church & Gale 1990):
//   w_hat = argmax_w P(x|w) * P(w)
//   where x = observed misspelling, w = intended word
//   This IS a functor: Word → Channel → Observation
//
// Orthographic Depth Hypothesis (Katz & Frost 1992):
//   Shallow orthographies (Finnish, Spanish) → mostly performance errors
//   Deep orthographies (English, French) → mostly competence errors

// =============================================================================
// Axis 1: Etiology — WHY the error happened
// =============================================================================

/// Why a spelling error occurred — the causal classification.
/// Corder (1967): competence vs performance distinction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorEtiology {
    /// The writer does not know the correct spelling.
    /// Systematic, reproducible. Caused by incomplete orthographic knowledge.
    /// Example: "definately" for "definitely"
    Competence,
    /// The writer knows the spelling but produced wrong output.
    /// Random, variable. Caused by motor/attention/speed issues.
    /// Example: "teh" for "the"
    Performance,
}

impl Concept for ErrorEtiology {}
impl FinitelyGenerated for ErrorEtiology {
    fn variants() -> Vec<Self> {
        vec![Self::Competence, Self::Performance]
    }
}

/// Competence error sources — what's missing from the writer's knowledge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompetenceSource {
    /// L1 transfer — spelling influenced by first language patterns.
    L1Transfer,
    /// Overgeneralization of a rule: "goed" for "went" (overgeneralizing -ed).
    Overgeneralization,
    /// Ignorance of an orthographic rule: "recieve" (i before e rule).
    RuleIgnorance,
    /// Analogy with another word: "definately" by analogy with "unfortunately".
    Analogy,
}

impl Concept for CompetenceSource {}
impl FinitelyGenerated for CompetenceSource {
    fn variants() -> Vec<Self> {
        vec![
            Self::L1Transfer,
            Self::Overgeneralization,
            Self::RuleIgnorance,
            Self::Analogy,
        ]
    }
}

/// Performance error mechanisms — what went wrong in production.
/// Maps to the dual-route model (Coltheart 1981, Caramazza & Miceli 1990).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PerformanceMechanism {
    /// Motor execution error (finger slip on keyboard).
    Motor,
    /// Attention lapse — knew the spelling but wasn't paying attention.
    Attention,
    /// Graphemic buffer failure — working memory lost part of the letter sequence.
    /// Wing & Baddeley (1980): errors peak in word-medial positions.
    GraphemicBuffer,
    /// Speed-accuracy tradeoff — typing too fast.
    Speed,
}

impl Concept for PerformanceMechanism {}
impl FinitelyGenerated for PerformanceMechanism {
    fn variants() -> Vec<Self> {
        vec![
            Self::Motor,
            Self::Attention,
            Self::GraphemicBuffer,
            Self::Speed,
        ]
    }
}

// =============================================================================
// Axis 2: Linguistic Level — WHAT knowledge is violated
// =============================================================================

/// What linguistic level the error occurs at.
/// POMAS framework (Silliman, Brimo 2013).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LinguisticLevel {
    /// Violates the phonological skeleton — the misspelling sounds different.
    /// Example: "bocket" for "bucket" (phoneme substitution).
    Phonological,
    /// Preserves phonology but violates orthographic convention.
    /// The misspelling would sound the same if pronounced.
    /// Example: "elefant" for "elephant", "nite" for "night".
    Orthographic,
    /// Violates morphological rules at morpheme boundaries.
    /// Example: "runing" for "running" (doubling rule at boundary).
    Morphological,
    /// Visual/graphemic confusion between similar letter shapes.
    /// Example: b/d confusion, p/q confusion.
    Visual,
}

impl Concept for LinguisticLevel {}
impl FinitelyGenerated for LinguisticLevel {
    fn variants() -> Vec<Self> {
        vec![
            Self::Phonological,
            Self::Orthographic,
            Self::Morphological,
            Self::Visual,
        ]
    }
}

// =============================================================================
// Axis 3: Operation — HOW the error manifests as string edits
// =============================================================================

/// Edit operations — how the error manifests at the string level.
/// Damerau (1964): >80% of misspellings involve a single operation.
/// Extended with run-on and split (Kukich 1992).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditOperation {
    /// Replace one character with another: "definate" → "definite"
    Substitution,
    /// A character is missing: "occuring" → "occurring"
    Deletion,
    /// An extra character added: "off" → "of"
    Insertion,
    /// Two adjacent characters swapped: "lable" → "label"
    Transposition,
    /// Two words merged: "alot" → "a lot"
    RunOn,
    /// One word split: "to gether" → "together"
    Split,
}

impl Concept for EditOperation {}
impl FinitelyGenerated for EditOperation {
    fn variants() -> Vec<Self> {
        vec![
            Self::Substitution,
            Self::Deletion,
            Self::Insertion,
            Self::Transposition,
            Self::RunOn,
            Self::Split,
        ]
    }
}

// =============================================================================
// Orthographic depth — writing system determines error patterns
// =============================================================================

/// Orthographic depth — how transparent the grapheme-phoneme mapping is.
/// Katz & Frost (1992): depth predicts dominant error type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrthographicDepth {
    /// Nearly 1:1 grapheme-phoneme mapping (Finnish, Spanish, Turkish).
    /// Dominant errors: performance (typos). Competence errors rare.
    Shallow,
    /// Moderate complexity (German, Dutch).
    Intermediate,
    /// Many-to-many grapheme-phoneme mapping (English, French, Danish).
    /// Dominant errors: competence. Phonologically plausible errors common.
    Deep,
}

impl Concept for OrthographicDepth {}
impl FinitelyGenerated for OrthographicDepth {
    fn variants() -> Vec<Self> {
        vec![Self::Shallow, Self::Intermediate, Self::Deep]
    }
}

// =============================================================================
// The spelling error category
// =============================================================================

/// A spelling error entity — combining all three axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpellingErrorConcept {
    Etiology,
    LinguisticLevel,
    Operation,
    OrthographicDepth,
    /// The observed misspelling (input).
    Observation,
    /// The intended word (target).
    Intention,
    /// The correction process (noisy channel inverse).
    Correction,
}

impl Concept for SpellingErrorConcept {}
impl FinitelyGenerated for SpellingErrorConcept {
    fn variants() -> Vec<Self> {
        vec![
            Self::Etiology,
            Self::LinguisticLevel,
            Self::Operation,
            Self::OrthographicDepth,
            Self::Observation,
            Self::Intention,
            Self::Correction,
        ]
    }
}

/// Relation-kind tag for the spelling-error category.
///
/// Per OBO-RO (Smith 2005), every arrow carries a canonical kind.
/// The spelling-error category has a single relation type: the
/// directed flow from etiology → linguistic level → operation →
/// observation → correction → intention (Damerau 1964; Brill & Moore 2000).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellingErrorRelationKind {
    Flow,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpellingRelation {
    pub from: SpellingErrorConcept,
    pub to: SpellingErrorConcept,
}

impl Arrow for SpellingRelation {
    type Object = SpellingErrorConcept;
    type Kind = SpellingErrorRelationKind;

    fn source(&self) -> SpellingErrorConcept {
        self.from
    }
    fn target(&self) -> SpellingErrorConcept {
        self.to
    }
    fn kind(&self) -> SpellingErrorRelationKind {
        SpellingErrorRelationKind::Flow
    }
    fn meta(&self) -> Provenance {
        Provenance {
            name: OntologyName::new_static("SpellingRelation"),
            description: Label::new_static(
                "directed flow between spelling-error category concepts",
            ),
            citation: Citation::parse_static("Damerau (1964); Brill & Moore (2000)"),
            module_path: ModulePath::new_static(module_path!()),
        }
    }
}

pub struct SpellingErrorCategory;

impl Category for SpellingErrorCategory {
    type Object = SpellingErrorConcept;
    type Morphism = SpellingRelation;

    fn identity(obj: &SpellingErrorConcept) -> SpellingRelation {
        SpellingRelation {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &SpellingRelation, g: &SpellingRelation) -> Option<SpellingRelation> {
        if f.to != g.from {
            return None;
        }
        if f.from == f.to {
            return Some(g.clone());
        }
        if g.from == g.to {
            return Some(f.clone());
        }
        let candidate = SpellingRelation {
            from: f.from,
            to: g.to,
        };
        // Partial category (#166): composition is defined only when the
        // composite is itself a declared morphism. The morphism set
        // computes the full reachability closure below, so any composable
        // pair lands inside it.
        if Self::morphisms().contains(&candidate) {
            Some(candidate)
        } else {
            None
        }
    }

    fn morphisms() -> Vec<SpellingRelation> {
        use SpellingErrorConcept::*;
        use std::collections::HashSet;

        // Direct relations from the spelling-error ontology.
        let direct: Vec<(SpellingErrorConcept, SpellingErrorConcept)> = vec![
            (Etiology, LinguisticLevel),
            (LinguisticLevel, Operation),
            (OrthographicDepth, Etiology),
            (Intention, Operation),
            (Operation, Observation),
            (Observation, Correction),
            (Correction, Intention),
        ];

        // Warshall (1962) transitive closure — required for AssociativityLaw
        // (Mac Lane CWM Ch. I §1).
        let mut closure: HashSet<(SpellingErrorConcept, SpellingErrorConcept)> =
            direct.iter().cloned().collect();
        loop {
            let mut added = false;
            let snap: Vec<_> = closure.iter().cloned().collect();
            for &(a, b) in &snap {
                for &(b2, c) in &snap {
                    if b == b2 && !closure.contains(&(a, c)) {
                        closure.insert((a, c));
                        added = true;
                    }
                }
            }
            if !added {
                break;
            }
        }

        let mut m: Vec<SpellingRelation> = Vec::new();
        for c in SpellingErrorConcept::variants() {
            m.push(SpellingRelation { from: c, to: c });
        }
        for (f, t) in closure {
            if f != t {
                m.push(SpellingRelation { from: f, to: t });
            }
        }
        m
    }
}

// =============================================================================
// Distance computation — the operational level
// =============================================================================

/// Compute the TRUE Damerau-Levenshtein distance between two strings — a
/// genuine metric (satisfies the triangle inequality), via the Lowrance &
/// Wagner (1975) dynamic-programming algorithm.
///
/// This is distinct from — and NOT the same quantity as — the commonly
/// (mis)implemented "Optimal String Alignment" (OSA) distance, which forbids
/// editing any substring more than once. OSA is NOT a metric: it can and
/// does violate the triangle inequality, because it cannot express a
/// transposition that interacts with a following insertion/deletion on the
/// SAME substring — exactly the case this fix addresses. A real,
/// proptest-discovered counterexample this function used to fail on:
/// `OSA("ob","bco") = 3`, but `OSA("ob","bo") + OSA("bo","bco") = 1 + 1 = 2`
/// — OSA cannot find the genuine 2-operation path (transpose `"ob"→"bo"`,
/// then insert `'c'`), because that transposition and that insertion both
/// touch the same two-character region. The true Lowrance-Wagner algorithm
/// below allows exactly this, and correctly returns 2.
///
/// Damerau, F. J. (1964). "A technique for computer detection and correction
/// of spelling errors." *Communications of the ACM*, 7(3), 171-176 — the
/// original four edit operations (insertion, deletion, substitution, and
/// transposition of two adjacent characters) this distance counts.
/// Lowrance, R. & Wagner, R. A. (1975). "An extension of the string-to-string
/// correction problem." *Journal of the ACM*, 22(2), 177-183 — the O(nm)
/// dynamic-programming algorithm implemented here, which (unlike OSA) yields
/// a genuine metric over the four Damerau edit operations.
///
/// Returns a dimensionless [`Quantity`] (`unit::UNITLESS`), never a bare
/// integer — an edit count is exactly the kind of value the rest of this
/// codebase already treats this way (e.g. the RMS scheduler's `utilization`/
/// `rm_utilization_bound`, `applied::operating_system::scheduler::engine`):
/// the DP table below is the numeric kernel (raw `usize`, for the
/// integer-indexed algorithm itself), wrapped as a typed quantity only at
/// the function's boundary, so a bare count never leaks out to code that
/// reasons about it.
pub fn damerau_levenshtein(a: &str, b: &str) -> Quantity {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let n = a.len();
    let m = b.len();

    if n == 0 {
        return Quantity::from_unit(m as f64, &unit::UNITLESS);
    }
    if m == 0 {
        return Quantity::from_unit(n as f64, &unit::UNITLESS);
    }

    let max_dist = n + m;

    // d[r][c] here holds the Lowrance-Wagner pseudocode's d[r-1, c-1] (their
    // indices run from -1; Rust arrays are 0-based, so every index is
    // shifted by +1 in both dimensions throughout this function).
    let mut d = vec![vec![0usize; m + 2]; n + 2];
    d[0][0] = max_dist;
    for i in 0..=n {
        d[i + 1][0] = max_dist;
        d[i + 1][1] = i;
    }
    for j in 0..=m {
        d[0][j + 1] = max_dist;
        d[1][j + 1] = j;
    }

    // da[c] = the last row index (1-based) at which character c occurred in
    // `a`, among rows already completed; absent = never seen yet.
    let mut da: alloc::collections::BTreeMap<char, usize> = alloc::collections::BTreeMap::new();

    for i in 1..=n {
        let mut db = 0usize; // last column (1-based) in THIS row where a[i-1] matched b; 0 = none yet.
        for j in 1..=m {
            let k = da.get(&b[j - 1]).copied().unwrap_or(0);
            let l = db;
            let cost = if a[i - 1] == b[j - 1] {
                db = j;
                0
            } else {
                1
            };
            // k < i and l < j always hold here (da only carries completed
            // earlier rows; db only carries earlier columns of this same
            // row), so these subtractions never underflow.
            let transposition = d[k][l] + (i - k - 1) + 1 + (j - l - 1);
            d[i + 1][j + 1] = (d[i][j] + cost) // substitution
                .min(d[i + 1][j] + 1) // insertion
                .min(d[i][j + 1] + 1) // deletion
                .min(transposition);
        }
        da.insert(a[i - 1], i);
    }

    Quantity::from_unit(d[n + 1][m + 1] as f64, &unit::UNITLESS)
}

/// Find the closest matches for a word from a list of candidates.
/// Returns candidates within the given maximum distance, sorted by distance.
/// `max_distance` is an ordinary count parameter (a caller-supplied cutoff,
/// not itself a reasoned-about result — the same shape as
/// `scheduler::engine::rm_utilization_bound(n: usize)`'s own `n`), so it
/// stays `usize`; the returned distances are typed [`Quantity`]s.
pub fn closest_matches<'a>(
    word: &str,
    candidates: &[&'a str],
    max_distance: usize,
) -> Vec<(&'a str, Quantity)> {
    // Length-difference prune: |len(word) - len(candidate)| is a lower bound on
    // edit distance, so a candidate whose length differs by more than
    // max_distance cannot match — skip its O(n·m) DP. Without this, a long
    // untrusted `word` runs the full Damerau–Levenshtein table against every
    // candidate (a resource-exhaustion DoS).
    let word_len = word.chars().count();
    let max_distance_f = max_distance as f64;
    let mut matches: Vec<(&str, Quantity)> = candidates
        .iter()
        .filter_map(|&candidate| {
            if word_len.abs_diff(candidate.chars().count()) > max_distance {
                return None;
            }
            let dist = damerau_levenshtein(word, candidate);
            if dist.value <= max_distance_f && dist.value > 0.0 {
                Some((candidate, dist))
            } else {
                None
            }
        })
        .collect();
    matches.sort_by(|a, b| {
        a.1.partial_cmp(&b.1)
            .expect("both sides are dimensionless UNITLESS quantities, always comparable")
    });
    matches
}

/// Classify an edit as likely performance or competence error.
/// Heuristic based on Damerau (1964) and Pollock & Zamora (1983):
/// - Single transposition → almost certainly performance (finger slip)
/// - Single adjacent-key substitution → performance (motor)
/// - Phonologically plausible substitution → competence (orthographic)
pub fn classify_etiology(original: &str, misspelled: &str) -> ErrorEtiology {
    let dist = damerau_levenshtein(original, misspelled);
    if dist.value == 1.0 {
        // Single-edit errors are overwhelmingly performance errors
        // (Pollock & Zamora: 90-95% of errors are single-edit)
        ErrorEtiology::Performance
    } else {
        // Multi-edit errors are more likely competence errors
        ErrorEtiology::Competence
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    /// A dimensionless UNITLESS edit-distance quantity, for comparing against
    /// [`damerau_levenshtein`]'s typed return value in these tests.
    fn q(n: u32) -> Quantity {
        Quantity::from_unit(f64::from(n), &unit::UNITLESS)
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<SpellingErrorCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn identical_strings() {
        assert_eq!(damerau_levenshtein("dog", "dog"), q(0));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn single_substitution() {
        assert_eq!(damerau_levenshtein("dog", "doo"), q(1));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn single_insertion() {
        assert_eq!(damerau_levenshtein("dg", "dog"), q(1));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn single_deletion() {
        assert_eq!(damerau_levenshtein("dogg", "dog"), q(1));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn single_transposition() {
        assert_eq!(damerau_levenshtein("dgo", "dog"), q(1));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn multiple_edits() {
        assert_eq!(damerau_levenshtein("kitten", "sitting"), q(3));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn empty_strings() {
        assert_eq!(damerau_levenshtein("", ""), q(0));
        assert_eq!(damerau_levenshtein("abc", ""), q(3));
        assert_eq!(damerau_levenshtein("", "abc"), q(3));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn closest_matches_finds_dog() {
        let candidates = vec!["dog", "dig", "log", "cat"];
        let matches = closest_matches("dgo", &candidates, 2);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].0, "dog");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn single_edit_is_performance() {
        assert_eq!(classify_etiology("dog", "dgo"), ErrorEtiology::Performance);
        assert_eq!(classify_etiology("the", "teh"), ErrorEtiology::Performance);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn multi_edit_is_competence() {
        // "elefant" for "elephant" — distance 2 (ph→f, drop h)
        assert_eq!(
            classify_etiology("elephant", "elefant"),
            ErrorEtiology::Competence
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn etiology_has_two_variants() {
        assert_eq!(ErrorEtiology::variants().len(), 2);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn linguistic_level_has_four_variants() {
        assert_eq!(LinguisticLevel::variants().len(), 4);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn edit_operation_has_six_variants() {
        assert_eq!(EditOperation::variants().len(), 6);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn orthographic_depth_determines_errors() {
        // Shallow orthography → performance errors dominate
        // Deep orthography → competence errors dominate
        // This is the Orthographic Depth Hypothesis (Katz & Frost 1992)
        let morphisms = SpellingErrorCategory::morphisms();
        assert!(morphisms.contains(&SpellingRelation {
            from: SpellingErrorConcept::OrthographicDepth,
            to: SpellingErrorConcept::Etiology,
        }));
    }

    // =========================================================================
    // Property-based tests — mathematical properties of edit distance
    // =========================================================================

    mod prop {
        use super::*;
        use proptest::prelude::*;

        fn arb_word() -> impl Strategy<Value = String> {
            "[a-z]{1,8}"
        }

        proptest! {
            /// Metric axiom: d(x, x) = 0 (identity of indiscernibles).
            #[test]
            fn prop_distance_identity(word in arb_word()) {
                prop_assert_eq!(damerau_levenshtein(&word, &word), q(0));
            }

            /// Metric axiom: d(x, y) = d(y, x) (symmetry).
            #[test]
            fn prop_distance_symmetry(a in arb_word(), b in arb_word()) {
                prop_assert_eq!(
                    damerau_levenshtein(&a, &b),
                    damerau_levenshtein(&b, &a)
                );
            }

            /// Metric axiom: d(x, y) >= 0 (non-negativity).
            /// Trivially true for a value built from a usize DP count, but
            /// documents the property over the typed Quantity this function
            /// actually returns.
            #[test]
            fn prop_distance_non_negative(a in arb_word(), b in arb_word()) {
                let dist = damerau_levenshtein(&a, &b);
                prop_assert!(dist.value >= 0.0);
            }

            /// Triangle inequality: d(x, z) <= d(x, y) + d(y, z).
            /// This proves Damerau-Levenshtein is a proper metric.
            #[test]
            fn prop_triangle_inequality(
                a in arb_word(),
                b in arb_word(),
                c in arb_word()
            ) {
                let ab = damerau_levenshtein(&a, &b);
                let bc = damerau_levenshtein(&b, &c);
                let ac = damerau_levenshtein(&a, &c);
                let ab_plus_bc = ab.add(&bc)
                    .expect("both UNITLESS quantities, always compatible");
                prop_assert!(ac <= ab_plus_bc,
                    "triangle inequality violated: d({},{})={} > d({},{})={} + d({},{})={}",
                    a, c, ac.value, a, b, ab.value, b, c, bc.value
                );
            }

            /// Upper bound: d(x, y) <= max(|x|, |y|).
            /// You can always transform by deleting everything and inserting.
            #[test]
            fn prop_distance_upper_bound(a in arb_word(), b in arb_word()) {
                let dist = damerau_levenshtein(&a, &b);
                let upper = a.len().max(b.len());
                prop_assert!(dist.value <= upper as f64,
                    "distance {} exceeds upper bound {} for '{}' and '{}'",
                    dist.value, upper, a, b
                );
            }

            /// Single character difference → distance exactly 1.
            #[test]
            fn prop_single_substitution_is_one(
                prefix in "[a-z]{0,4}",
                a in "[a-z]",
                b in "[a-z]",
                suffix in "[a-z]{0,4}"
            ) {
                prop_assume!(a != b);
                let word_a = format!("{}{}{}", prefix, a, suffix);
                let word_b = format!("{}{}{}", prefix, b, suffix);
                prop_assert_eq!(damerau_levenshtein(&word_a, &word_b), q(1));
            }

            /// Etiology classification is total — every error gets classified.
            #[test]
            fn prop_etiology_always_classifies(a in arb_word(), b in arb_word()) {
                let etiology = classify_etiology(&a, &b);
                prop_assert!(
                    etiology == ErrorEtiology::Competence
                    || etiology == ErrorEtiology::Performance
                );
            }
        }

        pr4xis::register_praxis_value!(prop_distance_identity, Verifiable);
        pr4xis::register_praxis_value!(prop_distance_symmetry, Verifiable);
        pr4xis::register_praxis_value!(prop_distance_non_negative, Verifiable);
        pr4xis::register_praxis_value!(prop_triangle_inequality, Verifiable);
        pr4xis::register_praxis_value!(prop_distance_upper_bound, Verifiable);
        pr4xis::register_praxis_value!(prop_single_substitution_is_one, Verifiable);
        pr4xis::register_praxis_value!(prop_etiology_always_classifies, Verifiable);
    }
}
