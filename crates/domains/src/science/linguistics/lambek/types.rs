// Lambek types — the objects in the syntax category.
//
// In Lambek grammar, every word has a type that describes how it
// combines with other words. A transitive verb like "sees" has type
// (NP\S)/NP — it takes an NP on the right and an NP on the left
// to produce a sentence S.
//
// Reference: Lambek, The Mathematics of Sentence Structure (1958)

/// An atomic syntactic type — the base types from which complex types are built.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AtomicType {
    /// S — a complete declarative sentence.
    S,
    /// Q — a question (interrogative sentence).
    Q,
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

    pub fn s() -> Self {
        Self::Atom(AtomicType::S)
    }

    pub fn q() -> Self {
        Self::Atom(AtomicType::Q)
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

    /// Display the type in standard notation.
    pub fn notation(&self) -> String {
        match self {
            Self::Atom(a) => match a {
                AtomicType::S => "S".into(),
                AtomicType::Q => "Q".into(),
                AtomicType::NP => "NP".into(),
                AtomicType::N => "N".into(),
                AtomicType::PP => "PP".into(),
            },
            Self::RightDiv(a, b) => {
                let a_str = a.notation();
                let b_str = b.notation();
                if a.is_atomic() && b.is_atomic() {
                    format!("{}/{}", a_str, b_str)
                } else {
                    format!("({})/({})​", a_str, b_str)
                }
            }
            Self::LeftDiv(a, b) => {
                let a_str = a.notation();
                let b_str = b.notation();
                if a.is_atomic() && b.is_atomic() {
                    format!("{}\\{}", a_str, b_str)
                } else {
                    format!("({})\\({})", a_str, b_str)
                }
            }
        }
    }
}

/// Try to reduce two adjacent types via function application.
///
/// Forward application (>): A/B + B → A
/// Backward application (<): B + A\B → A  (note: A\B means "A on left gives B")
///
/// Returns the result type if reduction succeeds, None if types don't combine.
pub fn reduce(left: &LambekType, right: &LambekType) -> Option<LambekType> {
    // Forward application: (A/B) + B → A
    if let LambekType::RightDiv(a, b) = left
        && b.as_ref() == right
    {
        return Some(*a.clone());
    }

    // Backward application: A + (A\B) → B
    if let LambekType::LeftDiv(a, b) = right
        && a.as_ref() == left
    {
        return Some(*b.clone());
    }

    None
}

// ---- Standard type assignments for English ----

/// Type assignments for common English word categories.
pub mod english {
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

    // ---- Question types ----

    /// Question copula (sentence-initial "is"): ((Q/NP)/NP)
    /// "is" in "is a dog a mammal?"
    /// Takes two NPs: is + NP(subject) + NP(predicate) → Q
    ///
    /// Derivation: "is a dog a mammal"
    ///   is:((Q/NP)/NP)  a:NP/N  dog:N  a:NP/N  mammal:N
    ///   is:((Q/NP)/NP)  [a dog]:NP     [a mammal]:NP
    ///   [is [a mammal]]:Q/NP            [a dog]:NP        ← forward: is takes predicate NP
    ///   wait — need backward for subject. Let's use (Q\NP)/NP instead:
    ///
    /// Actually per CCG: sentence-initial "is" for yes/no questions
    /// has type (S[q]/(S[b]\NP))/NP — but simplified for Lambek:
    /// Use Q/NP/NP — takes predicate NP right, then subject NP right.
    ///
    /// Simpler approach: treat "is X a Y?" as the copula building
    /// a declarative that then becomes a question.
    /// Type: (Q/NP)/NP — takes predicate NP, then subject NP, produces Q.
    pub fn question_copula() -> LambekType {
        LambekType::right_div(
            LambekType::right_div(LambekType::q(), LambekType::np()),
            LambekType::np(),
        )
    }

    /// Copula in declarative: (NP\S)/NP — "is" in "a dog is a mammal"
    pub fn copula() -> LambekType {
        LambekType::right_div(
            LambekType::left_div(LambekType::np(), LambekType::s()),
            LambekType::np(),
        )
    }

    /// "what" as question word: Q/(S/NP) — "what is a dog?"
    /// Takes a sentence-missing-NP on right, produces Q
    pub fn wh_what() -> LambekType {
        LambekType::right_div(
            LambekType::q(),
            LambekType::right_div(LambekType::s(), LambekType::np()),
        )
    }
}
