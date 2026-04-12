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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SentenceFeature {
    /// S[dcl] — declarative finite clause: "the dog runs"
    Dcl,
    /// S[adj] — adjective-headed predicate: "big", "happy" (predicative)
    Adj,
    /// S[q] — yes/no question: "is it a dog?"
    Q,
    /// S[wq] — wh-question: "what is a dog?"
    Wq,
    /// S[b] — bare stem/infinitive: "run" in "can run"
    Bare,
    /// S[ng] — present participle: "running"
    Ng,
    /// S[pss] — passive participle: "seen" in "was seen"
    Pss,
    /// S[pt] — past participle: "gone" in "has gone"
    Pt,
    /// S[to] — to-infinitive: "to run"
    To,
}

/// An atomic syntactic type — the base types from which complex types are built.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AtomicType {
    /// S — a sentence, optionally with a feature (S[dcl], S[adj], S[q], etc.).
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

    /// S[dcl] — declarative sentence.
    pub fn s_dcl() -> Self {
        Self::Atom(AtomicType::S(Some(SentenceFeature::Dcl)))
    }

    /// S[adj] — adjective-headed predicate.
    pub fn s_adj() -> Self {
        Self::Atom(AtomicType::S(Some(SentenceFeature::Adj)))
    }

    /// S[q] — yes/no question (replaces old Q atomic type).
    pub fn q() -> Self {
        Self::Atom(AtomicType::S(Some(SentenceFeature::Q)))
    }

    /// S[wq] — wh-question.
    pub fn wq() -> Self {
        Self::Atom(AtomicType::S(Some(SentenceFeature::Wq)))
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
pub fn reduce(left: &LambekType, right: &LambekType) -> Option<LambekType> {
    // Forward application: (A/B) + B → A
    if let LambekType::RightDiv(a, b) = left
        && types_match(b, right)
    {
        return Some(*a.clone());
    }

    // Backward application: A + (A\B) → B
    if let LambekType::LeftDiv(a, b) = right
        && types_match(a, left)
    {
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

    /// Adjective: N/N — "big", "red"
    pub fn adjective() -> LambekType {
        LambekType::right_div(LambekType::n(), LambekType::n())
    }

    /// Preposition: (NP\NP)/NP — "in", "on", "with"
    pub fn preposition() -> LambekType {
        LambekType::right_div(
            LambekType::left_div(LambekType::np(), LambekType::np()),
            LambekType::np(),
        )
    }

    /// Adverb (verb modifier): (NP\S)\(NP\S) — "quickly", "slowly"
    pub fn adverb() -> LambekType {
        let vp = LambekType::left_div(LambekType::np(), LambekType::s());
        LambekType::left_div(vp.clone(), vp)
    }

    // ---- Predicate adjective (CCGbank: S[adj]\NP) ----

    /// Predicate adjective: S[adj]\NP — "big" in "a dog is big"
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

    /// Copula with adjective complement: (S[dcl]\NP)/(S[adj]\NP)
    /// "is" in "a dog is big" — takes predicate adjective, produces declarative VP
    pub fn copula_adj() -> LambekType {
        LambekType::right_div(
            LambekType::left_div(LambekType::np(), LambekType::s_dcl()),
            predicate_adjective(),
        )
    }

    // ---- Question types ----

    /// Question copula (sentence-initial "is"): (S[q]/NP)/NP
    /// "is" in "is a dog a mammal?" — takes two NPs, produces question.
    pub fn question_copula() -> LambekType {
        LambekType::right_div(
            LambekType::right_div(LambekType::q(), LambekType::np()),
            LambekType::np(),
        )
    }

    /// "what" as question word: S[wq]/(S/NP) — "what is a dog?"
    /// Takes a sentence-missing-NP on right, produces wh-question.
    pub fn wh_what() -> LambekType {
        // CCGbank: S[wq]/(S[dcl]\NP) — takes a sentence-missing-subject on the right.
        // "what is a dog" → what + [is a dog : NP\S] → S[wq]
        LambekType::right_div(
            LambekType::wq(),
            LambekType::left_div(LambekType::np(), LambekType::s()),
        )
    }
}
