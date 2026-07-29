#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

// Lambek types — the objects in the syntax category.
//
// In Lambek grammar, every word has a type that describes how it
// combines with other words. A transitive verb like "sees" has type
// (NP\S)/NP — it takes an NP on the right and an NP on the left
// to produce a sentence S.
//
// Reference: Lambek, The Mathematics of Sentence Structure (1958)

/// Sentence features — CCGbank's mechanism for distinguishing clause types.
/// From Hockenmaier & Steedman (2007), CCGbank.
///
/// Rather than introducing new atomic types (AP, QP, etc.), CCG adds
/// features to the sentence type S. This keeps the type system small
/// while capturing syntactic distinctions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SentenceFeature {
    /// `S[dcl]` — declarative finite clause: "the dog runs"
    Dcl,
    /// `S[adj]` — adjective-headed predicate: "big", "happy" (predicative)
    Adj,
    /// `S[q]` — yes/no question: "is it a dog?"
    Q,
    /// `S[wq]` — wh-question: "what is a dog?"
    Wq,
    /// `S[b]` — bare stem/infinitive: "run" in "can run"
    Bare,
    /// `S[ng]` — present participle: "running"
    Ng,
    /// `S[pss]` — passive participle: "seen" in "was seen"
    Pss,
    /// `S[pt]` — past participle: "gone" in "has gone"
    Pt,
    /// `S[to]` — to-infinitive: "to run"
    To,
}

/// An atomic syntactic type — the base types from which complex types are built.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AtomicType {
    /// S — a sentence, optionally with a feature (`S[dcl]`, `S[adj]`, `S[q]`, etc.).
    /// None = unspecified S (matches any feature in reduction).
    S(Option<SentenceFeature>),
    /// NP — a noun phrase.
    NP,
    /// N — a common noun.
    N,
    /// PP — a prepositional phrase.
    PP,
}

/// A Lambek type — atomic or complex (function types).
///
/// Complex types describe how words combine:
/// - `A/B` (right division): takes a B on the right, produces A
/// - `A\B` (left division): takes an A on the left, produces B
///
/// Examples:
/// - Determiner "the": NP/N (takes noun on right, produces NP)
/// - Intransitive verb "runs": NP\S (takes NP on left, produces S)
/// - Transitive verb "sees": (NP\S)/NP (takes NP right, then NP left, produces S)
/// - Adjective "big": N/N (takes noun on right, produces noun)
/// - Adverb "quickly": (NP\S)\(NP\S) (modifies a verb phrase)
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LambekType {
    /// An atomic type (S, NP, N, PP).
    Atom(AtomicType),
    /// Right division: A/B — takes B on the right, produces A.
    /// "the" : NP/N — give me a noun on my right, I'll give you an NP.
    RightDiv(Box<LambekType>, Box<LambekType>),
    /// Left division: A\B — takes A on the left, produces B.
    /// "runs" : NP\S — give me an NP on my left, I'll give you a sentence.
    LeftDiv(Box<LambekType>, Box<LambekType>),
}

impl LambekType {
    pub fn atom(a: AtomicType) -> Self {
        Self::Atom(a)
    }

    /// S — unspecified sentence (matches any feature in reduction).
    pub fn s() -> Self {
        Self::Atom(AtomicType::S(None))
    }

    /// `S[dcl]` — declarative sentence.
    pub fn s_dcl() -> Self {
        Self::Atom(AtomicType::S(Some(SentenceFeature::Dcl)))
    }

    /// `S[adj]` — adjective-headed predicate.
    pub fn s_adj() -> Self {
        Self::Atom(AtomicType::S(Some(SentenceFeature::Adj)))
    }

    /// `S[q]` — yes/no question (replaces old Q atomic type).
    pub fn q() -> Self {
        Self::Atom(AtomicType::S(Some(SentenceFeature::Q)))
    }

    /// `S[b]` — bare stem/infinitive: "run" in "can run", "mean" in "does X mean".
    pub fn s_bare() -> Self {
        Self::Atom(AtomicType::S(Some(SentenceFeature::Bare)))
    }

    /// `S[ng]` — present participle: "implementing" in "is implementing EVV".
    /// Hockenmaier & Steedman (2005), *CCGbank User's Manual*, MS-CIS-05-09,
    /// §3.4 item (28)e, p.35: "`S[ng]\NP`: for present participles."
    pub fn s_ng() -> Self {
        Self::Atom(AtomicType::S(Some(SentenceFeature::Ng)))
    }

    /// `S[pss]` — passive/past participle: "exposed" in "workers exposed to
    /// it", "used" in "a form of asbestos once used to make cigarette
    /// filters". Hockenmaier & Steedman (2005), *CCGbank User's Manual*,
    /// MS-CIS-05-09, Figure 3.1 p. 55 (the worked asbestos derivation, where
    /// `used` heads an `S[pss]\NP` that type-changes to the `NP\NP`
    /// postmodifier) and §3.8's (53)/(54) type-changing schema, whose `S$`
    /// "has the appropriate verbal features".
    pub fn s_pss() -> Self {
        Self::Atom(AtomicType::S(Some(SentenceFeature::Pss)))
    }

    /// `S[wq]` — wh-question.
    pub fn wq() -> Self {
        Self::Atom(AtomicType::S(Some(SentenceFeature::Wq)))
    }

    /// `S[to]` — to-infinitive clause: "to run" in "wants to run". Hockenmaier
    /// & Steedman (2005), *CCGbank User's Manual*, MS-CIS-05-09, §3.5.2 (the
    /// feature inventory: `S[to]` names a to-infinitival clause, alongside
    /// `S[b]` for the bare infinitive it is built from).
    pub fn s_to() -> Self {
        Self::Atom(AtomicType::S(Some(SentenceFeature::To)))
    }

    pub fn np() -> Self {
        Self::Atom(AtomicType::NP)
    }

    pub fn n() -> Self {
        Self::Atom(AtomicType::N)
    }

    pub fn pp() -> Self {
        Self::Atom(AtomicType::PP)
    }

    /// A/B — right division.
    pub fn right_div(result: Self, argument: Self) -> Self {
        Self::RightDiv(Box::new(result), Box::new(argument))
    }

    /// A\B — left division.
    pub fn left_div(argument: Self, result: Self) -> Self {
        Self::LeftDiv(Box::new(argument), Box::new(result))
    }

    /// Is this an atomic type?
    pub fn is_atomic(&self) -> bool {
        matches!(self, Self::Atom(_))
    }

    /// Is this a sentence type (any feature)?
    pub fn is_sentence(&self) -> bool {
        matches!(self, Self::Atom(AtomicType::S(_)))
    }

    /// Is this a noun type (N)?
    pub fn is_noun(&self) -> bool {
        matches!(self, Self::Atom(AtomicType::N))
    }

    /// Is this a noun phrase type (NP)?
    pub fn is_noun_phrase(&self) -> bool {
        matches!(self, Self::Atom(AtomicType::NP))
    }

    pub fn is_complex(&self) -> bool {
        !self.is_atomic()
    }

    /// The atomic type this category yields once fully saturated with its
    /// arguments — e.g. `(NP\S)/NP` (a transitive verb) ultimately yields
    /// `S`, the same way `NP/N` (a determiner) ultimately yields `NP`.
    fn ultimate_result(&self) -> &AtomicType {
        match self {
            Self::Atom(a) => a,
            Self::RightDiv(result, _) => result.ultimate_result(),
            Self::LeftDiv(_, result) => result.ultimate_result(),
        }
    }

    /// Is this a predicate category — a functor that, once saturated, yields
    /// a sentence (Steedman 2000 CCG)? Covers verbs of any valency
    /// (`NP\S`, `(NP\S)/NP`, `(NP\S)/NP/NP`), copulas, and auxiliaries alike,
    /// generalizing over open-class content verbs the same way `is_noun`
    /// generalizes over open-class nouns — every token's category comes from
    /// the same OLiA-class → Lambek-category projection regardless of
    /// whether the word is closed-class or WordNet-sourced.
    pub fn is_predicate(&self) -> bool {
        self.is_complex() && matches!(self.ultimate_result(), AtomicType::S(_))
    }

    pub fn is_right_div(&self) -> bool {
        matches!(self, Self::RightDiv(_, _))
    }

    pub fn is_left_div(&self) -> bool {
        matches!(self, Self::LeftDiv(_, _))
    }

    /// Display the type in standard notation.
    pub fn notation(&self) -> String {
        match self {
            Self::Atom(a) => match a {
                AtomicType::S(None) => "S".into(),
                AtomicType::S(Some(f)) => match f {
                    SentenceFeature::Dcl => "S[dcl]".into(),
                    SentenceFeature::Adj => "S[adj]".into(),
                    SentenceFeature::Q => "S[q]".into(),
                    SentenceFeature::Wq => "S[wq]".into(),
                    SentenceFeature::Bare => "S[b]".into(),
                    SentenceFeature::Ng => "S[ng]".into(),
                    SentenceFeature::Pss => "S[pss]".into(),
                    SentenceFeature::Pt => "S[pt]".into(),
                    SentenceFeature::To => "S[to]".into(),
                },
                AtomicType::NP => "NP".into(),
                AtomicType::N => "N".into(),
                AtomicType::PP => "PP".into(),
            },
            Self::RightDiv(a, b) => {
                let a_str = if a.is_left_div() {
                    format!("({})", a.notation())
                } else {
                    a.notation()
                };
                let b_str = if b.is_complex() {
                    format!("({})", b.notation())
                } else {
                    b.notation()
                };
                format!("{a_str}/{b_str}")
            }
            Self::LeftDiv(a, b) => {
                let a_str = if a.is_complex() {
                    format!("({})", a.notation())
                } else {
                    a.notation()
                };
                let b_str = if b.is_right_div() {
                    format!("({})", b.notation())
                } else {
                    b.notation()
                };
                format!("{a_str}\\{b_str}")
            }
        }
    }
}

/// Check if two Lambek types match, with feature unification for S.
/// S(None) matches any S(Some(_)) — unspecified S is a wildcard.
pub fn types_match(a: &LambekType, b: &LambekType) -> bool {
    match (a, b) {
        (LambekType::Atom(AtomicType::S(f1)), LambekType::Atom(AtomicType::S(f2))) => {
            // S(None) matches anything; S(Some(x)) matches S(Some(x)) or S(None)
            f1 == f2 || f1.is_none() || f2.is_none()
        }
        (LambekType::Atom(a), LambekType::Atom(b)) => a == b,
        (LambekType::RightDiv(a1, b1), LambekType::RightDiv(a2, b2)) => {
            types_match(a1, a2) && types_match(b1, b2)
        }
        (LambekType::LeftDiv(a1, b1), LambekType::LeftDiv(a2, b2)) => {
            types_match(a1, a2) && types_match(b1, b2)
        }
        _ => false,
    }
}

/// Try to reduce two adjacent types via function application.
///
/// Forward application (>): A/B + B → A
/// Backward application (<): B + A\B → A  (note: A\B means "A on left gives B")
///
/// Uses feature unification: S(None) matches any S(Some(_)).
///
/// Returns the result type if reduction succeeds, None if types don't combine.
///
/// A CCG MODIFIER — a functor whose declared result and argument categories
/// are the SAME type, `A/A` or `A\A` (Steedman 2000, *The Syntactic
/// Process*, Ch. 2's adjunct category account: a modifier's category is
/// "X for an X", so applying it to an argument carries that ARGUMENT's own
/// category forward, not a fresh copy of the functor's declared one) —
/// returns the ACTUAL matched argument's type, not the functor's OWN stored
/// `A`. For every atom except `S`, the two are indistinguishable (`NP` has
/// no internal feature to differ), so this is a no-op change for `NP\NP`
/// modifiers (`close_apposition`, `medial_supplement_np`). It is load-
/// bearing for `S`, whose optional [`SentenceFeature`] makes `S(None)` a
/// WILDCARD (`types_match`, just above): `fronted_scope_adjunct_np`/`_pp`'s
/// `(S/S)` pass-through (`svo_types::fronted_scope_adjunct_np`'s own doc) is
/// exactly this modifier shape, and returning the functor's fixed, feature-
/// erased `A = S(None)` — the PRE-EXISTING behavior this replaces — silently
/// collapsed a passed-through QUESTION's `[wq]`/`[q]` feature down to a bare
/// declarative `S` at the TYPE level (even though `montague::apply`'s own
/// semantic-level pass-through correctly keeps the argument's real `Sem`).
/// Confirmed real defect this closes: the weighted chart
/// ([`super::reduce::chart_reduce`]) keys each cell by TYPE, so a
/// declarative and a question reading of the SAME inner span, once BOTH
/// collapse to the identical outer `S(None)` key, directly compete on cost
/// alone — and the loaded CCGbank frequencies price a declarative `S` far
/// more common than the rare `S[wq]` (that chart's own doc: "count 8"), so
/// the declarative reading always won regardless of which one the sentence
/// actually needed ("In self-direction, what is X?" parsed "what" as a bare
/// NP subject of a degenerate declarative instead of the interrogative
/// operator). Carrying the argument's own feature forward keeps the two
/// readings on SEPARATE type keys (`S(Some(Q))` vs `S(None)`) at the outer
/// span, so the chart's own goal search — already tiered to prefer an
/// interrogative-featured `S` over a bare one, this module's own doc says so
/// — can find and prefer the question reading as designed.
///
/// Application only — NO composition / type-raising. SUBJECT relative clauses
/// reduce here by plain application (`the dog that runs` → NP). OBJECT relatives
/// need forward composition (`>B`) AND subject type-raising to build the `S/NP`
/// gap; adding `>B` alone both over-generates (regressed a taxonomy answer) and
/// is insufficient without type-raising, so the combinator extension is a
/// tracked grammar slice, not bundled here (the object-relative category loads
/// but does not yet reduce).
pub fn reduce(left: &LambekType, right: &LambekType) -> Option<LambekType> {
    // Forward application: (A/B) + B → A
    if let LambekType::RightDiv(a, b) = left
        && types_match(b, right)
    {
        if **a == **b {
            return Some(right.clone());
        }
        return Some(*a.clone());
    }

    // Backward application: A + (A\B) → B
    if let LambekType::LeftDiv(a, b) = right
        && types_match(a, left)
    {
        if **a == **b {
            return Some(left.clone());
        }
        return Some(*b.clone());
    }

    None
}

// ---- Standard Lambek type assignments for SVO languages ----
//
// These are the canonical type assignments from the Lambek calculus
// literature for Subject-Verb-Object languages (English, French, etc.).
// They follow Lambek (1958) and Moortgat (1997).
// Language-agnostic: any SVO language uses these assignments.

/// Lambek type assignments for SVO word order.
pub mod svo {
    use super::*;

    /// Determiner: NP/N — "the", "a", "every"
    pub fn determiner() -> LambekType {
        LambekType::right_div(LambekType::np(), LambekType::n())
    }

    /// Genitive clitic: (NP/N)\NP — "'s" in "the consumer's family home".
    /// Takes a possessor NP on the left, yields a determiner-shaped NP/N
    /// that then takes the possessed common noun on the right.
    /// Steedman (2000), *The Syntactic Process*, MIT Press, §2.3
    /// (specifying-genitive category); Huddleston & Pullum (2002),
    /// *The Cambridge Grammar of the English Language*, Ch. 5 §16.
    pub fn genitive_clitic() -> LambekType {
        LambekType::left_div(
            LambekType::np(),
            LambekType::right_div(LambekType::np(), LambekType::n()),
        )
    }

    /// Common noun: N — "dog", "cat", "idea"
    pub fn noun() -> LambekType {
        LambekType::n()
    }

    /// Proper noun / pronoun: NP — "John", "she", "it"
    pub fn proper_noun() -> LambekType {
        LambekType::np()
    }

    /// Intransitive verb: NP\S — "runs", "sleeps"
    pub fn intransitive_verb() -> LambekType {
        LambekType::left_div(LambekType::np(), LambekType::s())
    }

    /// Bare-stem intransitive verb: `S[b]\NP` — "opt-out" in "can an agency
    /// opt-out?". The `S[b]` counterpart of [`intransitive_verb`], mirroring
    /// [`bare_transitive_verb`]'s own relationship to [`transitive_verb`]:
    /// same (zero) argument structure, but the result is the UNINFLECTED
    /// clause a do-support/modal auxiliary selects for (Huddleston & Pullum
    /// 2002, *The Cambridge Grammar of the English Language*, Ch. 3 — the
    /// bare infinitival complement of an auxiliary), not a full finite `S`.
    pub fn bare_intransitive_verb() -> LambekType {
        LambekType::left_div(LambekType::np(), LambekType::s_bare())
    }

    /// Progressive-participle intransitive verb: `NP\S[ng]`. Hockenmaier &
    /// Steedman (2005), *CCGbank User's Manual*, MS-CIS-05-09, §3.4 item
    /// (28)e p.35 ("`S[ng]\NP`: for present participles"); §3.5.2 p.36,
    /// worked example (29c) `(VP (VBP are) (VP (VBG taking) (NP action)))`
    /// — "Both sides are taking action" — the `S[ng]\NP` complement a
    /// progressive-copula selects, mirroring [`bare_intransitive_verb`]'s
    /// own relationship to [`intransitive_verb`] one feature over.
    pub fn progressive_intransitive_verb() -> LambekType {
        LambekType::left_div(LambekType::np(), LambekType::s_ng())
    }

    /// Passive-participle verb phrase: `NP\S[pss]` (CCGbank's `S[pss]\NP`) —
    /// "exposed" in "workers exposed to it", "provided" in "services
    /// provided under a State plan", "used" in "a form of asbestos once used
    /// to make cigarette filters".
    ///
    /// Hockenmaier & Steedman (2005), *CCGbank User's Manual*,
    /// MS-CIS-05-09, University of Pennsylvania, Figure 3.1 p. 55: the
    /// worked derivation assigns `used` the category `S[pss]\NP` in the
    /// reduced-relative reading (and `(S[pss]\NP)/(S[to]\NP)` once it takes
    /// its own infinitival complement), which §3.8's type-changing rule
    /// (54)a then turns into the `NP\NP` postmodifier — see
    /// [`crate::cognitive::linguistics::lambek::supertag_costs::reduced_passive_relative_unary_rule`].
    ///
    /// The participle carries NO object slot: the passive has absorbed it
    /// (that is what makes the reduced relative a modifier rather than a
    /// clause), so this is the `S[pss]` sibling of [`intransitive_verb`] —
    /// exactly as [`progressive_intransitive_verb`] is its `S[ng]` sibling —
    /// and NOT of [`transitive_verb`].
    pub fn passive_participle_verb() -> LambekType {
        LambekType::left_div(LambekType::np(), LambekType::s_pss())
    }

    /// Reduced-relative postmodifier: `NP\NP` — the category CCGbank's
    /// type-changing rule (54)a PRODUCES from a passive participle, as in
    /// "workers [exposed to it]" / "services [provided under a State plan]".
    /// Hockenmaier & Steedman (2005), *CCGbank User's Manual*,
    /// MS-CIS-05-09, §3.8 p. 55 and Figure 3.1 (same page).
    ///
    /// STRUCTURALLY IDENTICAL to [`close_apposition`] and
    /// [`medial_supplement_np`] (all three are `NP\NP`, the one postnominal
    /// NP-modifier slot this grammar has) — named separately because the
    /// CONSTRUCTION and its citation differ, the same convention those two
    /// already follow with respect to each other. A derived PP
    /// ([`preposition`] applied to its object) lands in the SAME slot, which
    /// is exactly what lets "services provided under a State plan" stack a
    /// participial modifier and a PP without any further mechanism.
    pub fn reduced_relative_postmodifier() -> LambekType {
        LambekType::left_div(LambekType::np(), LambekType::np())
    }

    /// Transitive verb: (NP\S)/NP — "sees", "likes"
    pub fn transitive_verb() -> LambekType {
        LambekType::right_div(
            LambekType::left_div(LambekType::np(), LambekType::s()),
            LambekType::np(),
        )
    }

    /// Ditransitive verb: ((NP\S)/NP)/NP — "gives"
    pub fn ditransitive_verb() -> LambekType {
        LambekType::right_div(transitive_verb(), LambekType::np())
    }

    /// Bare-stem transitive verb: `(S[b]\NP)/NP` — "mean" in "does X mean",
    /// "run" in "can X run Y". The `S[b]` counterpart of
    /// [`transitive_verb`]: same argument structure, but the result is the
    /// UNINFLECTED clause a do-support/modal auxiliary selects for
    /// (Huddleston & Pullum 2002 Ch.3 — the bare infinitival complement of
    /// an auxiliary), not a full finite `S`.
    pub fn bare_transitive_verb() -> LambekType {
        LambekType::right_div(
            LambekType::left_div(LambekType::np(), LambekType::s_bare()),
            LambekType::np(),
        )
    }

    /// Progressive-participle transitive verb: `(NP\S[ng])/NP` —
    /// "implementing" in "Illinois is implementing EVV". Corpus-attested:
    /// Hockenmaier & Steedman (2007), *CCGbank: A Corpus of CCG Derivations
    /// and Dependency Structures*, Computational Linguistics 33(3):355-396,
    /// §6.3.1 "Right Node Raising", p.379, dependency (43) — from a real
    /// Treebank sentence ("Who is / who should be making the criminal law
    /// here"): `⟨is,(S[dcl]\NP)/(S[ng]\NP),2,making⟩`. Page-checked directly
    /// against the published PDF.
    pub fn progressive_transitive_verb() -> LambekType {
        LambekType::right_div(progressive_intransitive_verb(), LambekType::np())
    }

    /// Adjective: N/N — "big", "red"
    pub fn adjective() -> LambekType {
        LambekType::right_div(LambekType::n(), LambekType::n())
    }

    /// Nominal premodifier (bare noun functioning attributively): N/N —
    /// "consultation" in "consultation services", "program" is the head (not
    /// the modifier) in "PCA program". Levi (1978), *The Syntax and Semantics
    /// of Complex Nominals*, Academic Press (the closed set of recoverable
    /// predicates linking the two nouns of a complex nominal — cited here as
    /// the reason plain concatenation, not predicate recovery, is the correct
    /// scope: this codebase has no loaded ontology of Levi's predicates, and
    /// building one would be an unjustified asymmetry against how
    /// [`adjective`] already composes); Selkirk (1982), *The Syntax of
    /// Words*, MIT Press (right-headed `[N N]_N`); Huddleston & Pullum
    /// (2002), *The Cambridge Grammar of the English Language*, Ch. 19
    /// "Lexical word-formation" (Bauer & Huddleston), pp. 1621-1722.
    /// STRUCTURALLY IDENTICAL to [`adjective`] — CCGbank's own treatment
    /// (Hockenmaier & Steedman 2005, *CCGbank User's Manual* MS-CIS-05-09
    /// §3.6.1: "Other prenominal modifiers are functions from nouns to
    /// nouns, e.g. Dutch ⊢ N/N"; §3.6.2 "Compound nouns": compound nouns
    /// have no internal structure in the Treebank — every non-head noun in a
    /// compound chain is assigned `N/N` directly, combining by ordinary
    /// forward application) assigns a premodifying noun the identical
    /// category an attributive adjective gets — no separate category exists
    /// in the literature for this construction, and CCGbank's genuinely
    /// type-changing unary rules (§3.8) are a disjoint mechanism scoped to
    /// `S`-level clausal adjuncts, not nominal premodification. The
    /// type-equality with [`adjective`] is intentional (matching the
    /// genuine linguistic fact), not a collision needing disambiguation —
    /// nothing downstream distinguishes "adjective N/N" from "noun-as-
    /// modifier N/N" since the composition (`montague::apply`'s generic
    /// `N/N + N → N` concatenation) is identical either way.
    pub fn nominal_modifier_noun() -> LambekType {
        adjective()
    }

    /// Preposition: (NP\NP)/NP — "in", "on", "with"
    pub fn preposition() -> LambekType {
        LambekType::right_div(
            LambekType::left_div(LambekType::np(), LambekType::np()),
            LambekType::np(),
        )
    }

    /// Nominal coordinator (NP level): `(NP\NP)/NP` — "and"/"or" in
    /// "Medicare and Medicaid", "NILP or Stavros". Steedman (2000), *The
    /// Syntactic Process*, MIT Press, Ch. 4 "Combinators for Coordination" —
    /// scoped here to the concrete NP level rather than Steedman's fully
    /// general schematic `(X\X)/X`-for-any-X category (Dowty 1988, "Type
    /// Raising, Functional Composition, and Non-Constituent Coordination,"
    /// in Oehrle, Bach & Wheeler (eds.), *Categorial Grammars and Natural
    /// Language Structures*, Reidel, pp. 153-197, on why a fully general
    /// treatment needs composition/type-raising machinery beyond plain
    /// application). STRUCTURALLY IDENTICAL to [`preposition`] — the same
    /// `(NP\NP)/NP` shape already denotes an unrelated construction, so the
    /// semantic composition rule for this category (`montague::apply`) MUST
    /// dispatch on the lexical SURFACE ("and"/"or"), never on this shape
    /// alone, or the two constructions silently alias each other exactly
    /// the way `copula`/`transitive_verb` already do (documented, accepted
    /// there; NOT acceptable here, since the two meanings are unrelated).
    pub fn nominal_coordinator_np() -> LambekType {
        LambekType::right_div(
            LambekType::left_div(LambekType::np(), LambekType::np()),
            LambekType::np(),
        )
    }

    /// Nominal coordinator (N level): `(N\N)/N` — "and"/"or" in "a
    /// certificate or license", "check-in and check-out times". The common-
    /// noun-level sibling of [`nominal_coordinator_np`] — same citation, no
    /// collision found against any other `svo::*` constructor (verified: no
    /// other category in this module has this shape).
    pub fn nominal_coordinator_n() -> LambekType {
        LambekType::right_div(
            LambekType::left_div(LambekType::n(), LambekType::n()),
            LambekType::n(),
        )
    }

    /// Nominal coordinator, CLOSE-APPOSITION level: `((NP\NP)\(NP\NP))/(NP\NP)`
    /// — "and"/"or" coordinating TWO close-apposition-typed quoted
    /// definienda into ONE combined apposition modifier ("the terms
    /// 'exploitation' and 'financial exploitation' mean ...", 42 U.S.C. §
    /// 3002(18)(A); the defines-lens gap backlog's G5). Steedman's fully
    /// general coordination schema `(X\X)/X` (Steedman (2000), *The
    /// Syntactic Process*, MIT Press, Ch. 4 "Combinators for Coordination")
    /// instantiated at X = [`close_apposition`] rather than at the NP level
    /// [`nominal_coordinator_np`] already scopes it to — the SAME "scope
    /// the fully general schema to one concrete level rather than Dowty's
    /// fully schematic treatment" rationale [`nominal_coordinator_np`]'s
    /// own doc establishes (Dowty 1988 on why the fully general
    /// X-for-any-X treatment needs composition/type-raising this grammar's
    /// plain-application chart deliberately does not use).
    ///
    /// STRUCTURALLY DISTINCT from [`nominal_coordinator_np`] (`(NP\NP)/NP`,
    /// which coordinates BARE NP conjuncts — a definiens-side list like "a
    /// grant, contract, or cooperative agreement"): here EACH conjunct is
    /// ITSELF already close-apposition-typed (`NP\NP`), not a bare NP — the
    /// shape a quoted definiendum leaf carries BEFORE it ever meets its
    /// head noun ("term"/"terms"). Only ever offered as the PRIMARY type of
    /// the tokenizer's OWN reserved apposition-coordinator marker
    /// (`tokenize::is_apposition_coordinator_marker`) — never the literal
    /// "and"/"or" surface: both this category and [`nominal_coordinator_np`]
    /// converge to the IDENTICAL derived `NP\NP` shape once each has
    /// absorbed its own conjuncts (the SAME type collision
    /// [`close_apposition`]'s own doc already documents for two other
    /// constructions, here a third and fourth), so a dedicated marker —
    /// not surface+type alone — disambiguates which coordination this is
    /// at `montague::apply`'s dispatch.
    pub fn nominal_coordinator_apposition() -> LambekType {
        let apposition = close_apposition();
        LambekType::right_div(
            LambekType::left_div(apposition.clone(), apposition.clone()),
            apposition,
        )
    }

    /// Sentential coordinator (wh-question level): `(S[wq]\S[wq])/S[wq]` —
    /// "and"/"or" coordinating TWO complete wh-question clauses into one,
    /// as in "What Is Medicaid, and Who Is Eligible?". Steedman (2000), *The
    /// Syntactic Process*, MIT Press, Ch. 4 "Combinators for Coordination" —
    /// the SAME `(X\X)/X` schema [`nominal_coordinator_np`]/
    /// [`nominal_coordinator_n`]/[`nominal_coordinator_apposition`] already
    /// instantiate at X = NP / N / close-apposition, here instantiated at
    /// X = [`LambekType::wq`] instead, per [`nominal_coordinator_np`]'s own
    /// doc for why this codebase scopes Steedman's fully general schema to
    /// concrete evidenced levels rather than Dowty's fully schematic
    /// treatment (Dowty 1988, "Type Raising, Functional Composition, and
    /// Non-Constituent Coordination," in Oehrle, Bach & Wheeler (eds.),
    /// *Categorial Grammars and Natural Language Structures*, Reidel,
    /// pp. 153-197). Unlike Dowty's non-constituent-coordination problem
    /// (coordinating sub-clausal FRAGMENTS that are not themselves
    /// constituents), both conjuncts here are already complete,
    /// independently well-formed `S[wq]` constituents — the textbook easy
    /// case Steedman's own basic schema handles by plain application, no
    /// composition/type-raising needed.
    ///
    /// STRUCTURALLY DISTINCT from [`nominal_coordinator_np`]/`_n`/
    /// `_apposition` (all three reduce to an `NP`-family shape once
    /// saturated; this reduces to `S[wq]`), so no shape collision with any
    /// of them — verified: no other category in this module reduces to a
    /// bare `S[wq]\S[wq]` shape. Offered ADDITIVELY alongside the NP/N-level
    /// coordinator readings for the SAME "and"/"or" surface
    /// (`tokenize::is_nominal_coordinator`) — the chart's own tiered goal
    /// selection (interrogative-featured `S[wq]`/`S[q]` beats any other `S`
    /// reading, `reduce::chart_reduce`'s own doc) picks this reading over a
    /// spurious NP/N-level one whenever both conjuncts genuinely reduce to
    /// `S[wq]` on their own, so no destructive removal of the other
    /// readings is needed.
    pub fn sentential_coordinator_wq() -> LambekType {
        LambekType::right_div(
            LambekType::left_div(LambekType::wq(), LambekType::wq()),
            LambekType::wq(),
        )
    }

    /// Transitive-verb coordinator: `(TV\TV)/TV` with `TV = (NP\S)/NP`
    /// ([`transitive_verb`]) — "and"/"or" coordinating two OBJECT-sharing
    /// transitive verbs, "negotiates and enters into [a qualifying non-binding
    /// instrument]" (1 U.S.C. § 112b(k)(2), inside the subject relative clause
    /// "that negotiates and enters into ...") or "leases or sells \[property\]".
    /// Steedman's fully general coordination schema `(X\X)/X` (Steedman
    /// (2000), *The Syntactic Process*, MIT Press, Ch. 4 "Combinators for
    /// Coordination") instantiated at `X = `[`transitive_verb`] — the SAME
    /// "scope the general schema to one concrete evidenced level rather than
    /// Dowty's fully schematic treatment" rationale [`nominal_coordinator_np`]
    /// and [`sentential_coordinator_wq`] already establish (Dowty 1988, "Type
    /// Raising, Functional Composition, and Non-Constituent Coordination," in
    /// Oehrle, Bach & Wheeler (eds.), *Categorial Grammars and Natural
    /// Language Structures*, Reidel, pp. 153-197).
    ///
    /// The single SHARED object is absorbed AFTER the coordination
    /// ("negotiates and enters into" combines FIRST, into one `TV`, then that
    /// coordinated verb takes the object once) — the right-node-raising
    /// reading Steedman's schema handles here by PLAIN APPLICATION, no
    /// composition/type-raising (both conjuncts are complete, independently
    /// well-formed `TV` constituents still awaiting the same object; the
    /// textbook easy case, exactly as [`sentential_coordinator_wq`]'s two
    /// `S[wq]` conjuncts are — NOT Dowty's genuinely-non-constituent problem).
    /// This is why `TV` (the still-unsaturated verb), not the saturated VP
    /// `NP\S`, is the coordination level: coordinating two VPs would give each
    /// verb its OWN object, losing the shared-object reading the statute means.
    ///
    /// STRUCTURALLY DISTINCT from every other coordinator here —
    /// [`nominal_coordinator_np`]/`_n`/`_apposition` reduce to an `NP`-family
    /// shape and [`sentential_coordinator_wq`] to `S[wq]`, while this reduces
    /// to `TV = (NP\S)/NP` — so no shape collision (verified: no other
    /// category in this module reduces to a bare `((NP\S)/NP)\((NP\S)/NP)`
    /// shape). Offered ADDITIVELY for the SAME "and"/"or" surface
    /// (`tokenize::is_nominal_coordinator`) alongside the NP/N-level
    /// readings; a coordinator flanked by anything OTHER than two transitive
    /// verbs simply never reduces via this category, so the chart falls
    /// through to whichever reading does — no destructive removal needed.
    pub fn transitive_verb_coordinator() -> LambekType {
        LambekType::right_div(
            LambekType::left_div(transitive_verb(), transitive_verb()),
            transitive_verb(),
        )
    }

    /// Adverb (verb modifier): (NP\S)\(NP\S) — "quickly", "slowly"
    pub fn adverb() -> LambekType {
        let vp = LambekType::left_div(LambekType::np(), LambekType::s());
        LambekType::left_div(vp.clone(), vp)
    }

    /// Prepositional-verb particle, TRANSITIVE-VERB level:
    /// `((NP\S)/NP)\((NP\S)/NP)` — "into" in "enters INTO a qualifying
    /// non-binding instrument" (1 U.S.C. § 112b(k)(2)), "in" in "believes IN
    /// X", "on" in "relies ON X". Huddleston & Pullum (2002), *The Cambridge
    /// Grammar of the English Language*, Ch. 7 §3 "Prepositional verbs":
    /// a closed-per-verb but open-class-of-verbs construction where the
    /// preposition is LEXICALLY SELECTED by the verb and contributes no
    /// independent modifying content of its own (unlike an ordinary
    /// adjunct PP, "in the room"/"of Commerce", [`preposition`]'s existing
    /// `(NP\NP)/NP` reading) — the SAME "X for an X" modifier schema
    /// [`adverb`] instantiates one level up, at the saturated-clause/VP
    /// slot (`(NP\S)\(NP\S)`), instantiated here ONE level down, at the
    /// still-unsaturated transitive-verb slot, so the particle attaches
    /// BEFORE the verb consumes its object rather than after: "enters"
    /// ([`transitive_verb`]'s `(NP\S)/NP` alt reading) + "into" (this
    /// category) → `(NP\S)/NP` again, ready to take the object exactly like
    /// an ordinary transitive verb — closing the genuine grammar gap this
    /// codebase's application-only [`reduce`] (no composition/type-raising:
    /// see that function's own doc) cannot otherwise cross, since a plain
    /// preposition's `(NP\NP)/NP` argument type never matches a transitive
    /// verb's own category and the two can never combine by application
    /// alone.
    ///
    /// Corpus-attested support for treating a verb-selected PP as part of
    /// the verb's OWN subcategorization rather than an ordinary adjunct:
    /// Hockenmaier & Steedman (2005), *CCGbank User's Manual*, MS-CIS-05-09,
    /// Appendix A.2 "Complement-adjunct distinction" — the VP row lists a
    /// PP child as a recognized COMPLEMENT type (alongside NP/S/SBAR),
    /// distinct from an ADVP-tagged (`-DIR`/`-LOC`/`-MNR`/`-TMP`/`-PRP`)
    /// adjunct PP; §3.7.6 "Multi-word expressions" notes CCGbank does not
    /// generally re-derive a single fused category for such verb+preposition
    /// pairs either, so this additive alternative — not a special-cased
    /// "enter into" lexical entry — is the properly-scoped, non-hardcoded
    /// fix (no loaded per-verb subcategorization lexicon exists in this
    /// codebase to restrict it further; see [`preposition`]'s own doc for
    /// why this codebase already accepts an unrestricted-surface,
    /// cost-and-completeness-arbitrated additive offering as its standing
    /// disambiguation discipline rather than a hand-authored closed list
    /// here, where — unlike a genuinely closed function-word class such as
    /// the modal auxiliaries — which verb selects which preposition is an
    /// open per-lexeme fact this codebase has no loaded resource for).
    ///
    /// STRUCTURALLY DISTINCT from [`adverb`] (`(NP\S)\(NP\S)`, one level
    /// higher — a SATURATED clause modifier) and from [`preposition`]
    /// (`(NP\NP)/NP`, a different division direction and result atom
    /// entirely) — verified: no other category in this module has this
    /// shape.
    pub fn transitive_verb_particle() -> LambekType {
        let tv = transitive_verb();
        LambekType::left_div(tv.clone(), tv)
    }

    /// Close apposition (postnominal): NP\NP — a quoted definiendum
    /// following its head NP, as in "the term 'state'" (Dictionary Act
    /// shape) or "my friend Kim". The second NP restates/identifies the
    /// referent of the first rather than modifying it as a property, so it
    /// takes the SAME NP on its left and produces NP — the postnominal
    /// close-apposition category (Hockenmaier & Steedman 2007, *CCGbank: A
    /// Corpus of CCG Derivations and Dependency Structures*, Computational
    /// Linguistics 33(3):355-396, the NP-NP appositive convention; the
    /// underlying construction is Huddleston & Pullum (2002) *The Cambridge
    /// Grammar of the English Language*, Ch. 5 "Nouns and noun phrases", the
    /// apposition discussion — the exact section/page for "close apposition"
    /// specifically could not be independently re-verified against the
    /// primary text in this session; this citation names the chapter, not a
    /// pinpoint page).
    ///
    /// Offered as an ADDITIVE alternative reading for a quote-collapsed
    /// span (`tokenize::collapse_quoted_spans`), alongside its existing
    /// bare-NP mention reading — never replacing it, so
    /// every existing quoted-mention derivation ("what does 'X' mean") is
    /// unaffected.
    pub fn close_apposition() -> LambekType {
        LambekType::left_div(LambekType::np(), LambekType::np())
    }

    // ---- Predicate adjective (CCGbank: S[adj]\NP) ----

    /// Predicate adjective: `S[adj]\NP` — "big" in "a dog is big"
    /// From Hockenmaier & Steedman (2007): predicative adjectives are
    /// sentence-like, headed by the adjective feature.
    pub fn predicate_adjective() -> LambekType {
        LambekType::left_div(LambekType::np(), LambekType::s_adj())
    }

    // ---- Copula types (CCGbank: multiple entries per complement type) ----

    /// Copula with NP complement: (S\NP)/NP — "is" in "a dog is a mammal"
    pub fn copula() -> LambekType {
        LambekType::right_div(
            LambekType::left_div(LambekType::np(), LambekType::s()),
            LambekType::np(),
        )
    }

    /// Copula with adjective complement: `(S[dcl]\NP)/(S[adj]\NP)`
    /// "is" in "a dog is big" — takes predicate adjective, produces declarative VP
    pub fn copula_adj() -> LambekType {
        LambekType::right_div(
            LambekType::left_div(LambekType::np(), LambekType::s_dcl()),
            predicate_adjective(),
        )
    }

    /// Progressive copula, declarative: `(NP\S[dcl])/(NP\S[ng])` — "is" in
    /// "Illinois is implementing EVV". Mirrors [`copula_adj`] exactly, one
    /// complement-feature over. Hockenmaier & Steedman (2007),
    /// *Computational Linguistics* 33(3), §6.3.1 p.379, dependency (43):
    /// `⟨is,(S[dcl]\NP)/(S[ng]\NP),2,making⟩`.
    pub fn progressive_copula() -> LambekType {
        LambekType::right_div(
            LambekType::left_div(LambekType::np(), LambekType::s_dcl()),
            progressive_intransitive_verb(),
        )
    }

    // ---- Question types ----

    /// Question copula (sentence-initial "is"): `(S[q]/NP)/NP`
    /// "is" in "is a dog a mammal?" — takes two NPs, produces question.
    pub fn question_copula() -> LambekType {
        LambekType::right_div(
            LambekType::right_div(LambekType::q(), LambekType::np()),
            LambekType::np(),
        )
    }

    /// Modal question (sentence-initial modal auxiliary): `(S[q]/(NP\S[b]))/NP`
    /// "can" in "can an agency opt-out?", "can Medicaid take a house?" —
    /// takes the subject NP (rightmost slot), then a bare-stem VP
    /// complement (`NP\S[b]`, itself already saturated with its own object
    /// via [`bare_transitive_verb`]/[`bare_intransitive_verb`] where
    /// applicable), yielding a yes/no question `S[q]`. English auxiliaries
    /// (including modals) select a bare-infinitival VP complement under
    /// subject-auxiliary inversion — the SAME combinatory pattern
    /// [`does_support`] already uses for object-questions, specialized here
    /// for the POLAR-question case (no trailing object-gap slot, since a
    /// polar question has no wh-gap to fill elsewhere). Steedman (2000),
    /// *The Syntactic Process*, MIT Press; Huddleston & Pullum (2002), *The
    /// Cambridge Grammar of the English Language*, Ch. 3 §9 "Modal
    /// auxiliaries" (the closed 9-item class this category is gated on:
    /// can/could/may/might/shall/should/will/would/must), Ch. 11 (subject-
    /// auxiliary inversion licensing the `S[q]` result — the same
    /// inversion phenomenon [`question_copula_pp`]/[`question_copula_pred`]
    /// already cite).
    pub fn modal_question() -> LambekType {
        LambekType::right_div(
            LambekType::right_div(
                LambekType::q(),
                LambekType::left_div(LambekType::np(), LambekType::s_bare()),
            ),
            LambekType::np(),
        )
    }

    /// Progressive question copula (inverted): `(S[q]/(NP\S[ng]))/NP` —
    /// "is" in "Is Illinois implementing EVV?" / "Why IS Illinois
    /// implementing EVV?". The `S[ng]` counterpart of [`modal_question`] by
    /// the identical `S[b]` -> `S[ng]` substitution, on the same flat
    /// inverted-SQ schema (Hockenmaier & Steedman 2005, *CCGbank User's
    /// Manual* MS-CIS-05-09, §3.5.6 p.40 "Yes-no questions": the inverted
    /// auxiliary/copula is a sister of both the subject NP and the VP —
    /// Carpenter 1992's categorial analysis of yes-no questions), licensed
    /// for `[ng]` specifically by the SAME corpus-attested dependency
    /// [`progressive_copula`] cites (Hockenmaier & Steedman 2007, §6.3.1
    /// p.379, dependency (43)).
    pub fn progressive_question_copula() -> LambekType {
        LambekType::right_div(
            LambekType::right_div(LambekType::q(), progressive_intransitive_verb()),
            LambekType::np(),
        )
    }

    /// "what" as question word: `S[wq]/(S/NP)` — "what is a dog?"
    /// Takes a sentence-missing-NP on right, produces wh-question.
    pub fn wh_what() -> LambekType {
        // CCGbank: S[wq]/(S[dcl]\NP) — takes a sentence-missing-subject on the right.
        // "what is a dog" → what + [is a dog : NP\S] → S[wq]
        LambekType::right_div(
            LambekType::wq(),
            LambekType::left_div(LambekType::np(), LambekType::s()),
        )
    }

    // ---- Interrogative wh-word categories (derived per OLiA class) ----
    //
    // Each is the category an OLiA interrogative class derives
    // ([`interrogatives::derive_wh_type`](crate::cognitive::linguistics::lambek::interrogatives::derive_wh_type)).
    // They are STRUCTURALLY DISTINCT (different slashes/atoms), not relabels of
    // one constant — the anti-fudge the wh redesign exists for.

    /// Interrogative determiner: `(S[wq]/(NP\S))/N` — "which"/"what" + N.
    /// Selects a noun, then questions the resulting NP like an interrogative
    /// pronoun. CCGbank (Hockenmaier & Steedman 2007).
    pub fn wh_determiner() -> LambekType {
        LambekType::right_div(wh_what(), LambekType::n())
    }

    /// Interrogative adverb: `S[wq]/(S[q]/PP)` — "where"/"when"/"why"/"how".
    /// Questions an ADJUNCT (a PP-typed gap) of a complete inverted clause —
    /// genuinely distinct from the pronoun's NP gap. CCGbank wh-adverb. Reduces
    /// only against an `S[q]/PP` clause, which [`question_copula_pp`] supplies.
    pub fn wh_adverb() -> LambekType {
        LambekType::right_div(
            LambekType::wq(),
            LambekType::right_div(LambekType::q(), LambekType::pp()),
        )
    }

    /// Manner interrogative adverb: `S[wq]/S[q]` — "how" in "how does it
    /// work?" used INTRANSITIVELY (no PP anywhere in the clause for
    /// [`wh_adverb`]'s `S[q]/PP` gap to satisfy). Adjoins a COMPLETE
    /// inverted clause rather than gapping a PP argument — the general
    /// CCGbank adjunct rule "the category of an adjunct child is a unary
    /// functor C/C … if \[it\] is to the left of the head" (Hockenmaier &
    /// Steedman 2007, *CCGbank: A Corpus of CCG Derivations and Dependency
    /// Structures*, Computational Linguistics 33(3):355-396, §4.3.3
    /// "Head and Adjunct", p.363), and directly attested for an EXTRACTED
    /// wh-adjunct over a complete `S[dcl]` (never a PP-gapped one) at
    /// §6.2.4 "Extraction of Adjuncts", pp.378-379 (the "when" example
    /// tree, deriving "When" as `(S/S)/S[dcl]`, is on p.378; the quoted
    /// sentence "the dependency between when and dropped is directly
    /// established by the fact that dropped is the head of the
    /// complement `S[dcl]`" opens p.379, continuing the same section —
    /// both page-checked directly against the published PDF, not
    /// recalled).
    /// Structurally distinct from [`wh_adverb`] — different argument
    /// shape (`S[q]` vs `S[q]/PP`) — because manner "how" questions the
    /// clause as a whole, not a locative/temporal/instrumental PP
    /// adjunct within it.
    pub fn wh_manner_adverb() -> LambekType {
        LambekType::right_div(LambekType::wq(), LambekType::q())
    }

    /// Reason interrogative adverb: `S[wq]/S[q]` — "why" adjoining a
    /// COMPLETE inverted clause (no PP gap), e.g. "why is Illinois
    /// implementing EVV?". STRUCTURALLY IDENTICAL to [`wh_manner_adverb`]
    /// (same shape, same general CCGbank adjunct-extraction rule) but a
    /// DISTINCT semantic kind (Reason, not Manner) — the SAME
    /// structurally-identical-but-separately-named precedent
    /// [`nominal_modifier_noun`]'s own doc already establishes against
    /// [`adjective`] in this file (both `N/N`, kept as two named functions
    /// because they answer two different linguistic questions). Grounded in
    /// the identical STRUCTURAL citation [`wh_manner_adverb`] already uses:
    /// Hockenmaier & Steedman (2007), *CCGbank: A Corpus of CCG Derivations
    /// and Dependency Structures*, Computational Linguistics 33(3):355-396,
    /// §6.2.4 "Extraction of Adjuncts" pp.378-379 (the "when" example tree,
    /// deriving a fronted wh-adjunct as `(S/S)/S[dcl]` over a COMPLETE
    /// clause) — page-checked directly against the published PDF, and
    /// general over any single-word clausal wh-adjunct, not just "when".
    /// A dedicated CGEL/Quirk page pinning "why = reason" as its OWN
    /// semantic-role label (as opposed to "how" = manner) was sought this
    /// session via direct source fetch (Cambridge Core, Google Books,
    /// archive.org, and several independent PDF hosts) but could not be
    /// retrieved in full text — flagged honestly per this codebase's
    /// citation discipline rather than guessing a page number.
    pub fn wh_reason_adverb() -> LambekType {
        LambekType::right_div(LambekType::wq(), LambekType::q())
    }

    /// PP-gap question copula: `(S[q]/PP)/NP` — sentence-medial "is"/"are" in a
    /// wh-adverb question ("where IS the dog"): takes the subject NP, yielding a
    /// yes/no-question clause still missing its locative PP. Pairs with
    /// [`wh_adverb`] so "where is the dog" → `S[wq]`. The PP-complement analogue of
    /// [`question_copula`] (`(S[q]/NP)/NP`); subject-aux inversion is licensed by
    /// a fronted wh-adverb (Huddleston & Pullum 2002 Ch.11).
    pub fn question_copula_pp() -> LambekType {
        LambekType::right_div(
            LambekType::right_div(LambekType::q(), LambekType::pp()),
            LambekType::np(),
        )
    }

    /// Relational predicate (transitive predicative): `(S[adj]\NP)/NP` — "part of"
    /// in "X is part of Y". Takes the OBJECT NP (right) to form a predicative
    /// `S[adj]\NP` ("is part of Y"), which the subject NP then saturates — the
    /// categorial type of a transitive relational expression (Moortgat,
    /// type-logical grammar; the predicative analogue of a transitive verb
    /// `(S\NP)/NP`). WHICH relation it introduces is LOADED data (the relation
    /// lexicon's surface→kind map), never this category — the grammar is generic.
    pub fn relational_predicate() -> LambekType {
        LambekType::right_div(predicate_adjective(), LambekType::np())
    }

    /// Question copula with a predicative complement: `(S[q]/(S[adj]\NP))/NP` —
    /// sentence-initial "is" in "is X part of Y". Takes the subject NP (right),
    /// then a predicative complement `S[adj]\NP` (the relational predicate
    /// "part of Y"), yielding a yes/no question `S[q]`. The predicative-complement
    /// analogue of [`question_copula`] (`(S[q]/NP)/NP`): its second slot is for a
    /// predicative, not a second NP — which is why "is X part of Y" needs its own
    /// category (Hockenmaier & Steedman 2007; subject-aux inversion, Huddleston &
    /// Pullum 2002 Ch.11).
    pub fn question_copula_pred() -> LambekType {
        LambekType::right_div(
            LambekType::right_div(LambekType::q(), predicate_adjective()),
            LambekType::np(),
        )
    }

    /// Object-question "what": `S[wq]/(S[q]/NP)` — "what does X mean?".
    /// Structurally distinct from [`wh_what`] (the SUBJECT-question category,
    /// `S[wq]/(NP\S)`): this "what" questions the OBJECT of an inverted
    /// clause still missing its NP (`S[q]/NP`), not the subject of a
    /// declarative one. Attested CCG object-wh category (Steedman 2000;
    /// Clark & Steedman-derived category families, CS&C 2004's
    /// wide-coverage grammar).
    pub fn wh_what_object() -> LambekType {
        LambekType::right_div(
            LambekType::wq(),
            LambekType::right_div(LambekType::q(), LambekType::np()),
        )
    }

    /// Do-support (sentence-medial "does"/"do"/"did" under subject-aux
    /// inversion): `((S[q]/NP)/((S[b]\NP)/NP))/NP` — "does" in "does X
    /// mean?". Takes the subject NP (rightmost slot, absorbed first), then a
    /// bare-stem transitive verb ([`bare_transitive_verb`]), yielding a
    /// yes/no-question clause still missing its object NP — the object-wh
    /// gap [`wh_what_object`] questions. Huddleston & Pullum (2002) Ch.3
    /// §7-8: do-support realizes subject-aux inversion when no other
    /// auxiliary is present, selecting a bare (non-finite) VP complement.
    ///
    /// Deliberately a CODE constructor, not a loaded CCGbank-attested TSV
    /// row keyed on the shared `AuxiliaryVerb` OLiA class: that class covers
    /// all 15 closed-class auxiliaries undifferentiated (`has`, `will`,
    /// `can`, … — see `olia-ccg-categories.tsv`'s own `AuxiliaryVerb` row
    /// documentation), and neither `StrictAuxiliaryVerb` nor `ModalVerb`
    /// (the finer OLiA subclasses that ARE loaded) isolates do/does/did from
    /// have/has/had — both fall under `StrictAuxiliaryVerb`. A shared-key
    /// row would broadcast do-support to every auxiliary. This mirrors
    /// [`question_copula_pp`]'s precedent: gated in the tokenizer on the
    /// literal do-support surfaces, never a blind lexicon row.
    pub fn does_support() -> LambekType {
        LambekType::right_div(
            LambekType::right_div(
                LambekType::right_div(LambekType::q(), LambekType::np()),
                bare_transitive_verb(),
            ),
            LambekType::np(),
        )
    }

    // ---- Fronted scope-setting sentential adjunct (defines-lens gap G1) ----

    /// Fronted scope-setting adjunct, direct-NP-complement form: `(S/S)/NP` —
    /// "for"/"in" heading a preposed scene-setting adjunct that selects its
    /// complement directly as an NP: "For purposes of this subsection, the
    /// term ... means ..." / "In this subsection, the term ... means ...".
    ///
    /// `S/S` is CCGbank's own category for a sentence-initial adjunct or
    /// parenthetical (Hockenmaier & Steedman 2007, *CCGbank: A Corpus of CCG
    /// Derivations and Dependency Structures*, Computational Linguistics
    /// 33(3):355-396, §3.6 "Adjuncts" — the fronted, scene-setting reading of
    /// a preposition is distinguished from its POST-verbal VP-modifier
    /// reading `((S\NP)\(S\NP))/NP`, which this grammar does not need here):
    /// Steedman's own worked example, "In 1968, John was born", assigns "In"
    /// exactly `(S/S)/NP` — the preposition selects the NP object directly, no
    /// intervening genuine PP category (this grammar's operational analogue
    /// of a PP is `NP\NP`, the shape [`preposition`] itself already produces —
    /// see [`fronted_scope_adjunct_pp`] for the variant that consumes one of
    /// those). Applying `(S/S)/NP` to its NP object yields `S/S`, a modifier
    /// that then combines with the REST of the sentence (`S`, on its right,
    /// via forward application) to its own left — this is the ADJUNCT/
    /// ARGUMENT distinction Steedman (2000), *The Syntactic Process*, MIT
    /// Press, Ch. 2 draws structurally: the fronted PP scopes the clause but
    /// contributes no argument of its own (`montague::apply`'s S-result
    /// branch drops it transparently, the semantic mirror of this syntactic
    /// fact).
    pub fn fronted_scope_adjunct_np() -> LambekType {
        LambekType::right_div(
            LambekType::right_div(LambekType::s(), LambekType::s()),
            LambekType::np(),
        )
    }

    /// Fronted scope-setting adjunct, PP-complement form: `(S/S)/(NP\NP)` —
    /// "except"/"subject" heading a preposed scope-setting adjunct that
    /// selects an ALREADY-derived `NP\NP` ("PP", this grammar's operational
    /// shape — see [`preposition`]) rather than a bare NP directly: "Except
    /// for the purposes of subchapter X of this chapter, the term ... means
    /// ..." / "Subject to subparagraphs (B) and (C), the term ... means
    /// ...".
    ///
    /// Two distinct real constructions share this one category:
    /// - "except for X": Quirk, Greenbaum, Leech & Svartvik (1985), *A
    ///   Comprehensive Grammar of the English Language*, Longman, §9.10
    ///   "complex prepositions" — "except" combines with a following
    ///   preposition phrase ("for X") rather than an NP directly.
    /// - "subject to X": the predicative-adjective absolute/supplementive
    ///   clause of Huddleston & Pullum (2002), *The Cambridge Grammar of the
    ///   English Language*, Ch. 15 §2 "Supplements" — "subject" heads its own
    ///   small (verbless) clause taking a PP complement introduced by "to",
    ///   the whole thing functioning as a preposed, scope-setting
    ///   conditional adjunct ("subject to Y" ≈ "provided Y holds").
    ///
    /// Both "for X" and "to X" already reduce to `NP\NP` under
    /// [`preposition`] once they absorb their own NP object, so this category
    /// reuses that EXISTING derivation rather than introducing a second,
    /// genuine PP-forming combinator.
    pub fn fronted_scope_adjunct_pp() -> LambekType {
        LambekType::right_div(
            LambekType::right_div(LambekType::s(), LambekType::s()),
            LambekType::left_div(LambekType::np(), LambekType::np()),
        )
    }

    // ---- Medial comma-delimited supplement (defines-lens gap G2) ----

    /// Medial NP-level supplement, SUBJECT-VERB position: `NP\NP` — the
    /// merged interior of a comma-delimited parenthetical breaking
    /// definiendum-verb adjacency: "the term 'X', used with respect to Y,
    /// means ..." / "the term 'X', as used in this title, means ...".
    ///
    /// STRUCTURALLY IDENTICAL to [`close_apposition`] (both are `NP\NP`) —
    /// a genuine, expected collision: Huddleston & Pullum (2002), *The
    /// Cambridge Grammar of the English Language*, Cambridge University
    /// Press, Ch. 15 "Supplements", treat a comma-set-off NP-modifying
    /// supplement (participial, "as"-headed, or otherwise) as occupying the
    /// SAME postnominal `NP\NP` slot a close-apposition NP does — the
    /// difference is not syntactic SHAPE but semantic CONTRIBUTION: a
    /// close-apposition NP RESTATES/IDENTIFIES the head NP's referent (and
    /// so takes over reference — `montague::apply`'s existing close-
    /// apposition guard), whereas a supplement merely SCOPES it and
    /// contributes nothing (Ch. 15 §1: a supplement is "not integrated into
    /// the syntactic structure of the sentence" it attaches to). Only the
    /// TOKENIZER ever mints this exact category (`tokenize::
    /// collapse_medial_comma_adjuncts`, over the merged, opaque interior of
    /// a recognized comma bracket) — never a lexicon row — so
    /// `montague::apply`'s NP-result branch disambiguates the two
    /// `NP\NP` readings by the SAME "type shape AND lexical surface" double
    /// guard [`nominal_coordinator_np`]'s own doc comment establishes as
    /// this codebase's standing discipline: here, the surface is the
    /// reserved synthetic marker `tokenize::MEDIAL_SUPPLEMENT_NP_MARKER`
    /// (crate-private — never a real lexical word).
    pub fn medial_supplement_np() -> LambekType {
        LambekType::left_div(LambekType::np(), LambekType::np())
    }

    /// Medial TRANSITIVE-VERB-level supplement, POST-VERB position:
    /// `((NP\S)/NP)\((NP\S)/NP)` — the merged interior of a comma-delimited
    /// parenthetical breaking a "means"/"includes"-class verb's adjacency
    /// to its own object: "the term 'X' means, with respect to Y, Z" (the
    /// EVV headline shape, 42 U.S.C. § 1396b(l)(5)) / "the term 'X' means,
    /// with respect to an individual ..., Z" (42 U.S.C. § 3002).
    ///
    /// Steedman's general VP-modifier schema, `(X\X)/X`-for-any-X (Steedman
    /// (2000), *The Syntactic Process*, MIT Press, Ch. 2 the adjunct
    /// combinator; Ch. 4 on adjunction more generally) applied at the
    /// unsaturated TRANSITIVE-VERB level rather than the SATURATED verb
    /// phrase [`adverb`] already applies it to (`(NP\S)\(NP\S)`) — needed
    /// because the supplement here interrupts the verb BEFORE it has
    /// absorbed its object, not after. Scoped to the concrete
    /// [`transitive_verb`] shape rather than Steedman's fully schematic
    /// category, mirroring [`nominal_coordinator_np`]'s own documented
    /// rationale (Dowty 1988 on why the fully general treatment needs
    /// composition/type-raising beyond plain application, which this
    /// grammar's chart deliberately does not use).
    ///
    /// Minted ONLY by the tokenizer's comma-bracket collapse
    /// (`tokenize::collapse_medial_comma_adjuncts`), never a lexicon row —
    /// `montague::apply`'s function-result branch recognizes it by the SAME
    /// double guard (`result_type == func_type` at the [`transitive_verb`]
    /// shape, AND the reserved synthetic marker
    /// `tokenize::MEDIAL_SUPPLEMENT_VERB_MARKER`, crate-private), so the
    /// verb passes through unabsorbed and ready to take its REAL object
    /// next.
    pub fn medial_supplement_verb() -> LambekType {
        LambekType::left_div(transitive_verb(), transitive_verb())
    }

    // ---- Passive-infinitival ECM/control ("be required to VP") ----

    /// Infinitive-marker "to": `(NP\S[to])/(NP\S[b])` — a VP-to-VP raising
    /// functor, taking a bare-infinitival VP on the right and yielding a
    /// to-infinitival VP. Hockenmaier & Steedman (2005), *CCGbank User's
    /// Manual*, MS-CIS-05-09, §3.5.2 (the feature inventory) and the worked
    /// derivations at pp. 34/37: the infinitival particle "to" has the
    /// category `(S[to]\NP)/(S[b]\NP)` — it selects a bare infinitive
    /// (`S[b]\NP`) as its argument and yields a to-infinitival verb phrase
    /// (`S[to]\NP`), for both control ("to give") and raising ("to be
    /// given") uses alike. The FIRST use of `S[to]` anywhere in this
    /// grammar — no existing `svo::*` constructor shares this shape.
    pub fn infinitive_to() -> LambekType {
        LambekType::right_div(
            LambekType::left_div(LambekType::np(), LambekType::s_to()),
            LambekType::left_div(LambekType::np(), LambekType::s_bare()),
        )
    }

    /// Catenative-passive predicate complement: `(NP\S[adj])/(NP\S[to])` —
    /// "required"/"expected"/"supposed" etc. in their predicative-adjective
    /// reading, taking a to-infinitival VP complement instead of a bare
    /// predicate ("services ARE REQUIRED TO USE EVV"). The `S[to]`-
    /// complement sibling of [`relational_predicate`] (`(S[adj]\NP)/NP`,
    /// which takes a plain NP argument) — same rationale, a different
    /// complement shape. The raising/control PHENOMENON (a predicate whose
    /// surface subject corresponds to the notional object of the
    /// non-passive paraphrase — "X requires Y to VP" → "Y is required to
    /// VP") is Huddleston & Pullum (2002), *The Cambridge Grammar of the
    /// English Language*, Cambridge University Press, Ch. 14 "Non-finite
    /// and verbless clauses"; CGEL is a descriptive reference grammar and
    /// assigns no categorial-grammar category of its own, so this SHAPE is
    /// this codebase's own categorial rendering of that phenomenon (the
    /// CCGbank engineering route instead runs catenative passives through a
    /// dedicated passive-auxiliary "be" category this codebase does not
    /// have — see this construction's own doc for why that is a separate,
    /// larger, later-scoped piece of work, not needed for the corpus rows
    /// this category targets).
    ///
    /// Collision check: the result (`NP\S[adj]`) is shared only with
    /// [`predicate_adjective`]/[`relational_predicate`]'s own derived
    /// result — the SAME kind of disclosed, non-wildcard, argument-
    /// disambiguated sharing [`nominal_coordinator_np`]'s own doc comment
    /// already accepts elsewhere, never the `S(None)` WILDCARD collision a
    /// prior same-session attempt at a bare-copula passive category
    /// (`predicate_passive`, `NP\S(None)` — structurally IDENTICAL to
    /// [`intransitive_verb`]) produced: that attempt measured zero rows
    /// fixed and one new regression ("Power of attorney abuse.") from the
    /// unification wildcard, and was reverted. Every category in THIS
    /// construction produces or consumes `S[adj]`/`S[to]`/`S[b]` — never
    /// bare `S(None)` — so none of them can ever be the type CCGbank
    /// assigns [`intransitive_verb`]/[`transitive_verb`], structurally
    /// (checked by `PartialEq`/`types_match`), not just empirically.
    pub fn catenative_infinitival_predicate() -> LambekType {
        LambekType::right_div(
            predicate_adjective(),
            LambekType::left_div(LambekType::np(), LambekType::s_to()),
        )
    }
}
