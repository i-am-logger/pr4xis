use pr4xis::category::entity::Entity;
use pr4xis::logic::Axiom;

// Pregroup Grammar Ontology — parsing as group algebra.
//
// A pregroup is a partially ordered monoid where every element has
// left and right adjoints. This is the algebraic structure of grammar.
//
// Word types are products of basic types with adjoints.
// Parsing = "does the product of all word types contract to the sentence type?"
//
// The contraction laws:
//   a^l · a ≤ 1   (left adjoint cancels on the left)
//   a · a^r ≤ 1   (right adjoint cancels on the right)
//
// The expansion laws:
//   1 ≤ a · a^l   (type raising, left)
//   1 ≤ a^r · a   (type raising, right)
//
// References:
// - Lambek, Type Grammar Revisited (1999) — pregroups replace slashes
// - Casadio & Lambek, A Tale of Four Grammars (2002)
// - Goodman, Semiring Parsing (1999) — chart as semiring
// - Preller & Lambek, Free Compact 2-Categories (2007)

/// A basic grammatical type — the atoms of the pregroup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BasicType {
    /// S — sentence.
    S,
    /// NP — noun phrase.
    NP,
    /// N — common noun.
    N,
    /// PP — prepositional phrase.
    PP,
}

impl Entity for BasicType {
    fn variants() -> Vec<Self> {
        vec![Self::S, Self::NP, Self::N, Self::PP]
    }
}

/// A pregroup element — a basic type with an adjoint exponent.
///
/// Exponent 0 = the type itself: `np`
/// Exponent 1 = right adjoint: `np^r`
/// Exponent -1 = left adjoint: `np^l`
/// Exponent 2 = double right adjoint: `np^rr`
///
/// In group terms: the exponent is like a power of the "inverse."
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PregroupElement {
    pub base: BasicType,
    pub exponent: i32,
}

impl PregroupElement {
    pub fn new(base: BasicType, exponent: i32) -> Self {
        Self { base, exponent }
    }

    /// The basic type (exponent 0).
    pub fn basic(base: BasicType) -> Self {
        Self { base, exponent: 0 }
    }

    /// Right adjoint (exponent +1).
    pub fn right_adj(base: BasicType) -> Self {
        Self { base, exponent: 1 }
    }

    /// Left adjoint (exponent -1).
    pub fn left_adj(base: BasicType) -> Self {
        Self { base, exponent: -1 }
    }

    /// Can this element contract with the next one?
    /// a^l · a → 1 when exponents are (-1, 0) or more generally (k-1, k) for same base.
    /// a · a^r → 1 when exponents are (0, 1) or more generally (k, k+1) for same base.
    pub fn contracts_with(&self, next: &PregroupElement) -> bool {
        self.base == next.base && self.exponent + 1 == next.exponent
    }
}

/// A pregroup type — a product (sequence) of pregroup elements.
/// This IS the word's grammatical type.
///
/// Examples:
///   "dog"  → [n]
///   "the"  → [n^r, np]          (takes noun on right, produces NP)
///   "runs" → [np^l, s]          (takes NP on left, produces S)
///   "sees" → [np^l, s, np^r]    (takes NP on left, produces S, takes NP on right... wait)
///
/// Actually for transitive verb in pregroups:
///   "sees" → [np^l, s, np^l]... no.
///
/// Correct pregroup types (Lambek 1999):
///   "the"     → n^r · np           ≈ NP/N
///   "dog"     → n                   ≈ N
///   "runs"    → np^l · s            ≈ NP\S
///   "sees"    → np^l · s · np^r     ≈ (NP\S)/NP  -- wait, let me check
///
/// For "she sees the dog":
///   np · (np^l · s · np^r) · (n^r · np) · n
///   Contract np · np^l → 1: s · np^r · n^r · np · n
///   Hmm, that doesn't work. Let me re-check.
///
/// Actually: "sees" as transitive = π₂^l · s · π₁^r  where π are NPs.
/// But in the simple pregroup: sees = np^l · s · np^r  is wrong because
/// np^r needs np on its RIGHT, but the object is on the right.
///
/// Correct: "sees" = (np^l) · (s · (np)^r) -- no.
/// From Lambek's papers: transitive verb = p₁^l · s · p₂^l
/// where p₁ = subject NP (left), p₂ = object NP (right... but ^l?)
///
/// Let me just use the standard: in pregroups the transitive verb
/// "likes" gets type  np^l · s · np^l  (NO, this is wrong for English word order)
///
/// Actually from the research: For SVO languages:
///   Transitive verb: π₁^l · σ · π₂^l  -- this contracts as:
///   π₁ · (π₁^l · σ · π₂^l) · π₂  →  σ
///   Because π₁ · π₁^l ≤ 1 and π₂^l · π₂ ≤ 1... wait, π₂^l · π₂ ≤ 1?
///   No! The contraction law is a^l · a ≤ 1, not a · a^l ≤ 1.
///   So we need the OBJECT on the LEFT of the verb's right part.
///
/// For English SVO: subject VERB object
///   np · (np^l · s · np^l) · np
///   Contract: np · np^l → 1 (left): s · np^l · np
///   Contract: np^l · np → 1 (left adj cancels on left): s
///   YES! This works because np^l · np ≤ 1 (the left contraction law).
///
/// So transitive verb in pregroup = np^l · s · np^l (both arguments use left adjoint
/// because in SVO order, both subject and object are on the OUTSIDE).
/// Wait no — "np^l · s · np^l" means: need NP on left AND NP on right?
/// np · (np^l · s · np^l) · np
///   = (np · np^l) · s · (np^l · np)
///   ≤ 1 · s · 1 = s  ✓
///
/// But this means both contractions use the SAME law: a^l · a ≤ 1.
/// The subject contracts on the left of np^l, and the object contracts
/// on the right of the second np^l. Both work because adjacency.
///
/// Let me just implement and test.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PregroupType {
    pub elements: Vec<PregroupElement>,
}

impl PregroupType {
    pub fn new(elements: Vec<PregroupElement>) -> Self {
        Self { elements }
    }

    pub fn single(base: BasicType) -> Self {
        Self {
            elements: vec![PregroupElement::basic(base)],
        }
    }

    /// The product of two pregroup types (concatenation).
    pub fn product(&self, other: &PregroupType) -> PregroupType {
        let mut elements = self.elements.clone();
        elements.extend_from_slice(&other.elements);
        PregroupType { elements }
    }

    /// Attempt to contract this type by applying the pregroup laws.
    /// Returns the contracted type (with adjacent adjoint pairs removed).
    ///
    /// Contraction: if element[i] contracts_with element[i+1],
    /// remove both (they reduce to the identity 1).
    pub fn contract(&self) -> PregroupType {
        let mut elements = self.elements.clone();
        let mut changed = true;

        while changed {
            changed = false;
            for i in 0..elements.len().saturating_sub(1) {
                if elements[i].contracts_with(&elements[i + 1]) {
                    elements.remove(i + 1);
                    elements.remove(i);
                    changed = true;
                    break;
                }
            }
        }

        PregroupType { elements }
    }

    /// Does this type contract to a single basic type?
    pub fn contracts_to(&self, target: BasicType) -> bool {
        let contracted = self.contract();
        contracted.elements.len() == 1
            && contracted.elements[0].base == target
            && contracted.elements[0].exponent == 0
    }

    /// Is this the identity (empty product)?
    pub fn is_identity(&self) -> bool {
        self.elements.is_empty()
    }

    /// Display in standard notation.
    pub fn notation(&self) -> String {
        if self.elements.is_empty() {
            return "1".to_string();
        }
        self.elements
            .iter()
            .map(|e| {
                let base = match e.base {
                    BasicType::S => "s",
                    BasicType::NP => "np",
                    BasicType::N => "n",
                    BasicType::PP => "pp",
                };
                match e.exponent {
                    0 => base.to_string(),
                    1 => format!("{}^r", base),
                    -1 => format!("{}^l", base),
                    k => format!("{}^({})", base, k),
                }
            })
            .collect::<Vec<_>>()
            .join(" · ")
    }
}

// =============================================================================
// Standard English pregroup type assignments
// =============================================================================

/// Pregroup types for English words, from Lambek (1999).
pub mod svo {
    use super::*;

    /// Noun: n
    pub fn noun() -> PregroupType {
        PregroupType::single(BasicType::N)
    }

    /// Proper noun / pronoun: np
    pub fn proper_noun() -> PregroupType {
        PregroupType::single(BasicType::NP)
    }

    /// Determiner: np · n^l (produces NP, takes N on right via left adjoint)
    /// "the dog" = np · n^l · n → np (left contraction: n^l · n ≤ 1)
    pub fn determiner() -> PregroupType {
        PregroupType::new(vec![
            PregroupElement::basic(BasicType::NP),
            PregroupElement::left_adj(BasicType::N),
        ])
    }

    /// Intransitive verb: np^r · s (takes NP on left via right adjoint)
    /// "she runs" = np · np^r · s → s (right contraction: np · np^r ≤ 1)
    pub fn intransitive_verb() -> PregroupType {
        PregroupType::new(vec![
            PregroupElement::right_adj(BasicType::NP),
            PregroupElement::basic(BasicType::S),
        ])
    }

    /// Transitive verb: np^r · s · np^l (subject right adj, object left adj)
    /// "she sees the dog" = np · np^r · s · np^l · np · n^l · n → s
    pub fn transitive_verb() -> PregroupType {
        PregroupType::new(vec![
            PregroupElement::right_adj(BasicType::NP),
            PregroupElement::basic(BasicType::S),
            PregroupElement::left_adj(BasicType::NP),
        ])
    }

    /// Adjective: n · n^l (produces N, takes N on right — modifier)
    /// "big dog" = n · n^l · n → n (left contraction: n^l · n ≤ 1)
    pub fn adjective() -> PregroupType {
        PregroupType::new(vec![
            PregroupElement::basic(BasicType::N),
            PregroupElement::left_adj(BasicType::N),
        ])
    }
}

// =============================================================================
// Parsing — contract the product of word types
// =============================================================================

/// Parse a sentence by contracting the product of word types.
/// Returns true if the product contracts to the sentence type.
pub fn parse(word_types: &[PregroupType]) -> bool {
    if word_types.is_empty() {
        return false;
    }

    // Build the product of all word types
    let mut product = word_types[0].clone();
    for wt in &word_types[1..] {
        product = product.product(wt);
    }

    // Contract and check if it reduces to S
    product.contracts_to(BasicType::S)
}

// =============================================================================
// Axioms — the pregroup laws
// =============================================================================

/// Left contraction axiom: a^l · a ≤ 1
pub struct LeftContraction;

impl Axiom for LeftContraction {
    fn description(&self) -> &str {
        "pregroup left contraction: a^l · a ≤ 1 (Lambek 1999)"
    }

    fn holds(&self) -> bool {
        // Verify for all basic types
        for base in BasicType::variants() {
            let product = PregroupType::new(vec![
                PregroupElement::left_adj(base),
                PregroupElement::basic(base),
            ]);
            let contracted = product.contract();
            if !contracted.is_identity() {
                return false;
            }
        }
        true
    }
}
pr4xis::register_axiom!(LeftContraction);

/// Right contraction axiom: a · a^r ≤ 1
pub struct RightContraction;

impl Axiom for RightContraction {
    fn description(&self) -> &str {
        "pregroup right contraction: a · a^r ≤ 1 (Lambek 1999)"
    }

    fn holds(&self) -> bool {
        for base in BasicType::variants() {
            let product = PregroupType::new(vec![
                PregroupElement::basic(base),
                PregroupElement::right_adj(base),
            ]);
            let contracted = product.contract();
            if !contracted.is_identity() {
                return false;
            }
        }
        true
    }
}
pr4xis::register_axiom!(RightContraction);

// =============================================================================
// The pregroup as a Category
// =============================================================================

/// A morphism in the pregroup: a product that contracts from source to target.
/// A/B = "takes B on right, produces A" = morphism from B to A.
/// The product np · n^l is a morphism from N to NP.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PregroupMorphism {
    pub source: BasicType,
    pub target: BasicType,
    pub product: PregroupType,
}

impl pr4xis::category::relationship::Relationship for PregroupMorphism {
    type Object = BasicType;
    type Kind = ();
    fn source(&self) -> BasicType {
        self.source
    }
    fn target(&self) -> BasicType {
        self.target
    }
    fn kind(&self) {}
}

/// The pregroup as a category.
/// Objects: BasicType (S, NP, N, PP).
/// Morphisms: PregroupType products that contract from source to target.
/// Composition: product concatenation + contraction.
pub struct PregroupCategory;

impl pr4xis::category::Category for PregroupCategory {
    type Object = BasicType;
    type Morphism = PregroupMorphism;

    fn identity(obj: &BasicType) -> PregroupMorphism {
        // Identity is the empty product (1 in the pregroup monoid)
        PregroupMorphism {
            source: *obj,
            target: *obj,
            product: PregroupType::new(vec![]),
        }
    }

    fn compose(f: &PregroupMorphism, g: &PregroupMorphism) -> Option<PregroupMorphism> {
        if f.target != g.source {
            return None;
        }
        // Identity handling
        if f.product.is_identity() {
            return Some(g.clone());
        }
        if g.product.is_identity() {
            return Some(f.clone());
        }
        // Compose = concatenate products, then contract
        let combined = f.product.product(&g.product);
        let contracted = combined.contract();
        Some(PregroupMorphism {
            source: f.source,
            target: g.target,
            product: contracted,
        })
    }

    fn morphisms() -> Vec<PregroupMorphism> {
        let mut m = Vec::new();

        // Identities
        for b in BasicType::variants() {
            m.push(PregroupMorphism {
                source: b,
                target: b,
                product: PregroupType::single(b),
            });
        }

        // Standard English type assignments as morphisms:
        // Structural morphisms: for each pair (A, B) where A ≠ B,
        // the product A^r · B is a morphism from A to B.
        // These are the fundamental arrows the pregroup algebra provides.
        let types = BasicType::variants();
        for &src in &types {
            for &tgt in &types {
                if src != tgt {
                    m.push(PregroupMorphism {
                        source: src,
                        target: tgt,
                        product: PregroupType::new(vec![
                            PregroupElement::right_adj(src),
                            PregroupElement::basic(tgt),
                        ]),
                    });
                }
            }
        }

        m
    }
}

// =============================================================================
// Lambek → Pregroup functor (ontology evolution)
// =============================================================================

/// Map a Lambek type to a pregroup type.
/// This IS the functor from the Lambek calculus to the free pregroup
/// (Casadio & Lambek 2002, "A Tale of Four Grammars").
///
/// The mapping:
///   A/B  →  A · B^l   (right division → product with left adjoint)
///   B\A  →  B^r · A   (left division → right adjoint then result)
///   Atom →  basic type
///
/// This preserves the reduction laws:
///   (A/B) · B  →  A · B^l · B  →  A · 1  →  A  ✓
///   B · (B\A)  →  B · B^r · A  →  1 · A  →  A  ✓
pub fn lambek_to_pregroup(lambek: &super::types::LambekType) -> PregroupType {
    use super::types::{AtomicType, LambekType};

    match lambek {
        LambekType::Atom(atom) => {
            let base = match atom {
                AtomicType::S(_) => BasicType::S,
                AtomicType::NP => BasicType::NP,
                AtomicType::N => BasicType::N,
                AtomicType::PP => BasicType::PP,
            };
            PregroupType::single(base)
        }
        LambekType::RightDiv(a, b) => {
            // A/B → A · B^l
            let a_pg = lambek_to_pregroup(a);
            let b_pg = lambek_to_pregroup(b);
            let b_adj: Vec<PregroupElement> = b_pg
                .elements
                .iter()
                .rev()
                .map(|e| PregroupElement::new(e.base, e.exponent - 1))
                .collect();
            let mut elements = a_pg.elements;
            elements.extend(b_adj);
            PregroupType::new(elements)
        }
        LambekType::LeftDiv(a, b) => {
            // A\B → A^r · B
            let a_pg = lambek_to_pregroup(a);
            let b_pg = lambek_to_pregroup(b);
            let a_adj: Vec<PregroupElement> = a_pg
                .elements
                .iter()
                .rev()
                .map(|e| PregroupElement::new(e.base, e.exponent + 1))
                .collect();
            let mut elements = a_adj;
            elements.extend(b_pg.elements);
            PregroupType::new(elements)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;
    use pr4xis::logic::Axiom;

    #[test]
    fn pregroup_category_laws() {
        check_category_laws::<PregroupCategory>().unwrap();
    }

    #[test]
    fn left_contraction_holds() {
        assert!(LeftContraction.holds());
    }

    #[test]
    fn right_contraction_holds() {
        assert!(RightContraction.holds());
    }

    #[test]
    fn the_dog_runs() {
        // the:n^r·np  dog:n  runs:np^l·s
        // Product: n^r · np · n · np^l · s
        // Contract n^r·n → 1: np · np^l · s
        // Contract np·np^l... wait, that's not a^l·a, it's a·a^l
        // np · np^l should use EXPANSION (1 ≤ a · a^l), not contraction.
        // Hmm. Let me re-check.
        //
        // Actually: "the" = n^r · np, "dog" = n, "runs" = np^l · s
        // Product: (n^r) · (np) · (n) · (np^l) · (s)
        //
        // n^r has exponent 1 (right adj of n)
        // np has exponent 0
        // n has exponent 0
        // np^l has exponent -1 (left adj of np)
        // s has exponent 0
        //
        // contracts_with checks: same base AND exponent + 1 == next exponent
        // n^r (n, exp=1) · np (np, exp=0) — different base, no
        // np (np, exp=0) · n (n, exp=0) — different base, no
        // n (n, exp=0) · np^l (np, exp=-1) — different base, no
        //
        // Hmm, n^r · n should contract but:
        // n^r is (N, exp=1), n is (N, exp=0)
        // contracts_with: exp + 1 == next_exp → 1 + 1 == 0? No!
        //
        // The issue: my contracts_with is wrong.
        // The contraction a^l · a ≤ 1 means (base, exp-1) · (base, exp) → 1
        // So for n^l · n: (N, -1) · (N, 0) → -1 + 1 = 0 ✓
        // For n · n^r: (N, 0) · (N, 1) → 0 + 1 = 1 ✓
        //
        // But n^r · n: (N, 1) · (N, 0) → 1 + 1 = 0? 1 ≠ 0+1=1, so 2≠0, no.
        //
        // Wait: n^r · n should NOT contract! The contraction laws are:
        //   a^l · a ≤ 1   (left adj on the LEFT, basic on the RIGHT)
        //   a · a^r ≤ 1   (basic on the LEFT, right adj on the RIGHT)
        //
        // So n^r · n is NOT a valid contraction. Instead:
        //   n · n^r ≤ 1 — this IS valid.
        //
        // So for "the dog": n^r · np · n
        //   We need to contract... hmm, there's no valid contraction here.
        //
        // I think the issue is that "the" should be np · n^l, not n^r · np.
        // Let me re-check Lambek's type assignments.
        //
        // From the research: "the" = n^r · np
        // But the contraction should be:
        //   n^r · np · n · np^l · s
        //   Can we contract n^r with the later n? No — they're not adjacent.
        //
        // Ah wait — pregroup contraction only works on ADJACENT elements.
        // So we need: ...(n^r) · (n)... to be adjacent.
        // In n^r · np · n, n^r and n are NOT adjacent (np is between them).
        //
        // Hmm. Let me reconsider the type assignment.
        // Maybe "the" should just be np · n^l?
        // Then: (np · n^l) · n · (np^l · s)
        //   = np · (n^l · n) · np^l · s
        //   = np · 1 · np^l · s          (n^l · n ≤ 1, left contraction)
        //   = np · np^l · s
        // Now np · np^l... this is a · a^l which is NOT a contraction.
        // Contraction is a^l · a, not a · a^l.
        //
        // Hmm. The expansion law says 1 ≤ a · a^l but parsing uses contraction not expansion.
        //
        // I think the issue is: determiner in English pregroup should be
        //   "the" = np · n^l
        //   "the dog" = np · n^l · n → np · 1 = np  (left contraction: n^l · n ≤ 1)
        //   "the dog runs" = np · np^l · s → 1 · s... wait, np · np^l is not contraction.
        //
        // OK I see the issue. Let me look at this more carefully.
        // For "the dog runs":
        //   np · n^l · n · np^l · s
        //   Step 1: n^l · n → 1 (left contraction): np · np^l · s
        //   Step 2: np · np^l → ??? This is NOT a contraction law.
        //
        // Hmm. np · np^l ≤ 1? That's the wrong direction.
        // The expansion law says 1 ≤ np · np^l, but that doesn't help.
        //
        // Wait — I had it backward. Let me re-read:
        //   Left contraction: a^l · a ≤ 1
        //   Right contraction: a · a^r ≤ 1
        //   Left expansion: 1 ≤ a · a^l
        //   Right expansion: 1 ≤ a^r · a
        //
        // So a · a^l is an EXPANSION, not a contraction. But np · np^l
        // is what we need to reduce. That's a · a^l — not contractible!
        //
        // The solution: "runs" should be s · np^r, not np^l · s.
        // Then: "the dog runs" = np · n^l · n · s · np^r
        //   Step 1: n^l · n → 1: np · s · np^r
        //   Step 2: np^r... at the end. This doesn't help.
        //
        // Actually the correct pregroup types for English are:
        //   "the"  = np · n^l     (determiner)
        //   "dog"  = n             (noun)
        //   "runs" = s · np^r      (intransitive verb for SUBJECT ON THE LEFT)
        //
        //   Wait, no. For SVO order with subject on LEFT:
        //   We need: np · VERB → s
        //   So VERB should be np^l · s (left adjoint of np, then s)
        //   And np · np^l · s = (np · np^l) · s
        //   But np · np^l is expansion, not contraction!
        //
        //   UNLESS I have the adjoint direction wrong.
        //   Let me re-read: a^l · a ≤ 1 means LEFT ADJOINT on the LEFT.
        //   So np^l · np ≤ 1 (left adj of np, then np itself).
        //
        //   For "she runs": np · (np^l · s)
        //   We need np and np^l to be adjacent.
        //   np · np^l · s — here np (exp 0) is followed by np^l (exp -1).
        //   contracts_with: 0 + 1 == -1? No! 1 ≠ -1.
        //
        //   But the LAW says: a^l · a ≤ 1, which is (exp -1) · (exp 0).
        //   contracts_with: -1 + 1 == 0? Yes!
        //   So np^l · np contracts, but np · np^l does NOT.
        //
        //   This means "she runs" = np · np^l · s does NOT contract!
        //   The np is BEFORE np^l, but the law requires np^l BEFORE np.
        //
        //   So the verb type must put the contraction in the right order.
        //   For subject on the LEFT: subject np is to the LEFT of the verb.
        //   We need: np · VERB → s, so VERB must "eat" np from the left.
        //   But left contraction is np^l · np, not np · np^l.
        //
        //   Hmm. Re-reading Lambek (1999):
        //   For English "John likes Mary":
        //     John: p (= np)
        //     likes: p^r · q · p^l  where q = s
        //     Mary: p (= np)
        //   Product: p · p^r · q · p^l · p
        //     p · p^r ≤ 1 (RIGHT contraction): q · p^l · p
        //     p^l · p ≤ 1 (LEFT contraction): q = s
        //
        //   AH HA! The transitive verb is p^r · s · p^l, not p^l · s · p^l!
        //   The SUBJECT uses RIGHT adjoint (p^r), the OBJECT uses LEFT adjoint (p^l).
        //   Because the subject is to the LEFT, it contracts via a · a^r ≤ 1.
        //   The object is to the RIGHT, it contracts via a^l · a ≤ 1.
        //
        //   So for intransitive verb "runs":
        //     runs: np^r · s  (NOT np^l · s!)
        //   "she runs" = np · np^r · s
        //     np · np^r ≤ 1 (right contraction): s ✓
        //
        //   And "the" must be:
        //     the: np · n^l  (NOT n^r · np!)
        //   "the dog" = np · n^l · n
        //     n^l · n ≤ 1 (left contraction): np ✓
        //
        //   Let me fix the type assignments and test.

        let types = vec![
            // the: np · n^l
            PregroupType::new(vec![
                PregroupElement::basic(BasicType::NP),
                PregroupElement::left_adj(BasicType::N),
            ]),
            // dog: n
            svo::noun(),
            // runs: np^r · s
            PregroupType::new(vec![
                PregroupElement::right_adj(BasicType::NP),
                PregroupElement::basic(BasicType::S),
            ]),
        ];
        assert!(parse(&types), "the dog runs should parse");
    }

    #[test]
    fn she_sees_the_dog() {
        // she: np
        // sees: np^r · s · np^l  (subject RIGHT adj, object LEFT adj)
        // the: np · n^l
        // dog: n
        //
        // Product: np · np^r · s · np^l · np · n^l · n
        //   np · np^r ≤ 1 (right contraction): s · np^l · np · n^l · n
        //   n^l · n ≤ 1 (left contraction): s · np^l · np
        //   np^l · np ≤ 1 (left contraction): s ✓
        let types = vec![
            svo::proper_noun(),
            PregroupType::new(vec![
                PregroupElement::right_adj(BasicType::NP),
                PregroupElement::basic(BasicType::S),
                PregroupElement::left_adj(BasicType::NP),
            ]),
            PregroupType::new(vec![
                PregroupElement::basic(BasicType::NP),
                PregroupElement::left_adj(BasicType::N),
            ]),
            svo::noun(),
        ];
        assert!(parse(&types), "she sees the dog should parse");
    }

    #[test]
    fn the_big_dog_runs() {
        // the: np · n^l
        // big: n · n^l  (adjective: modifier)
        // dog: n
        // runs: np^r · s
        //
        // Product: np · n^l · n · n^l · n · np^r · s
        //   n^l · n ≤ 1 (twice): np · np^r · s
        //   np · np^r ≤ 1: s ✓
        let types = vec![
            PregroupType::new(vec![
                PregroupElement::basic(BasicType::NP),
                PregroupElement::left_adj(BasicType::N),
            ]),
            PregroupType::new(vec![
                PregroupElement::basic(BasicType::N),
                PregroupElement::left_adj(BasicType::N),
            ]),
            svo::noun(),
            PregroupType::new(vec![
                PregroupElement::right_adj(BasicType::NP),
                PregroupElement::basic(BasicType::S),
            ]),
        ];
        assert!(parse(&types), "the big dog runs should parse");
    }

    #[test]
    fn bare_noun_doesnt_parse() {
        let types = vec![svo::noun()];
        assert!(!parse(&types), "bare noun should not parse as sentence");
    }

    #[test]
    fn pregroup_notation() {
        let det = PregroupType::new(vec![
            PregroupElement::basic(BasicType::NP),
            PregroupElement::left_adj(BasicType::N),
        ]);
        assert_eq!(det.notation(), "np · n^l");
    }

    // =========================================================================
    // Lambek → Pregroup functor tests
    // =========================================================================

    use super::super::types::svo as lambek_svo;

    #[test]
    fn functor_noun() {
        let pg = lambek_to_pregroup(&lambek_svo::noun());
        assert_eq!(pg, svo::noun());
    }

    #[test]
    fn functor_proper_noun() {
        let pg = lambek_to_pregroup(&lambek_svo::proper_noun());
        assert_eq!(pg, svo::proper_noun());
    }

    #[test]
    fn functor_determiner() {
        // Lambek: NP/N → pregroup: np · n^l
        let pg = lambek_to_pregroup(&lambek_svo::determiner());
        assert_eq!(pg, svo::determiner());
    }

    #[test]
    fn functor_intransitive_verb() {
        // Lambek: NP\S → pregroup: np^r · s
        let pg = lambek_to_pregroup(&lambek_svo::intransitive_verb());
        assert_eq!(pg, svo::intransitive_verb());
    }

    #[test]
    fn functor_preserves_parsing() {
        // If Lambek types parse, their pregroup images should also parse.
        // "the dog runs": det + noun + iv
        let lambek_types = vec![
            lambek_svo::determiner(),
            lambek_svo::noun(),
            lambek_svo::intransitive_verb(),
        ];
        let pregroup_types: Vec<PregroupType> =
            lambek_types.iter().map(lambek_to_pregroup).collect();
        assert!(
            parse(&pregroup_types),
            "functor should preserve parsing: {:?}",
            pregroup_types
                .iter()
                .map(|t| t.notation())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn functor_transitive_parses() {
        // "she sees the dog": np + tv + det + noun
        let lambek_types = vec![
            lambek_svo::proper_noun(),
            lambek_svo::transitive_verb(),
            lambek_svo::determiner(),
            lambek_svo::noun(),
        ];
        let pregroup_types: Vec<PregroupType> =
            lambek_types.iter().map(lambek_to_pregroup).collect();
        assert!(
            parse(&pregroup_types),
            "transitive should parse: {:?}",
            pregroup_types
                .iter()
                .map(|t| t.notation())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn functor_adjective_parses() {
        // "the big dog runs": det + adj + noun + iv
        let lambek_types = vec![
            lambek_svo::determiner(),
            lambek_svo::adjective(),
            lambek_svo::noun(),
            lambek_svo::intransitive_verb(),
        ];
        let pregroup_types: Vec<PregroupType> =
            lambek_types.iter().map(lambek_to_pregroup).collect();
        assert!(
            parse(&pregroup_types),
            "adjective should parse: {:?}",
            pregroup_types
                .iter()
                .map(|t| t.notation())
                .collect::<Vec<_>>()
        );
    }
}
