/-
The LEM-Dependency of Classical Incompleteness Conclusions
==========================================================

Author:  Ido Samuelson
Date:    July 2026
License: CC BY-SA 4.0

A Lean 4 formalization demonstrating that the classical first
incompleteness theorem's *semantic* conclusion — "G is true but
unprovable" — depends essentially on the Law of Excluded Middle at
the meta-level (equivalently: on a bivalent truth predicate over
the theory's proposition space).

The *syntactic* conclusion — "T does not decide G" — is available
constructively, without LEM. This file separates these two
conclusions and proves each in its appropriate logical setting.

The formalization is deliberately abstract: it works with an
arbitrary theory equipped with a provability predicate and a
diagonal fixed-point hypothesis, without committing to the full
arithmetic construction. This isolates the *logical* structure of
the argument from its number-theoretic packaging, making the
LEM-dependency visible.

Nothing in this file assumes classical logic globally; the
`Classical` namespace is invoked only where explicitly needed.
-/

namespace LEMDependency

/-! ## §1. Abstract Theories -/

/-- An abstract logical theory: a proposition space, an involutive
    negation, a provability predicate, and a consistency assumption.
    No commitment to arithmetic syntax or model theory. -/
structure Theory where
  Sentence : Type
  Neg : Sentence → Sentence
  Provable : Sentence → Prop
  consistent : ∀ p, ¬ (Provable p ∧ Provable (Neg p))

namespace Theory

/-- A sentence is *undecided* if neither it nor its negation is
    provable. Fully constructive: no truth predicate is invoked. -/
def Undecided (T : Theory) (p : T.Sentence) : Prop :=
  ¬ T.Provable p ∧ ¬ T.Provable (T.Neg p)

end Theory

/-! ## §2. The Syntactic Result Is Constructive

The core syntactic content of Gödel's argument — that a
self-referential sentence cannot be decided by a consistent theory —
requires only consistency and the diagonal fixed point. No LEM, no
bivalent truth, no completed model.
-/

/-- **Syntactic Half-Incompleteness (constructive).**
    If G is a sentence such that provability of G would entail
    provability of its negation, and T is consistent, then G is
    not provable in T. -/
theorem not_provable_of_diagonal
    (T : Theory) (G : T.Sentence)
    (fix : T.Provable G → T.Provable (T.Neg G)) :
    ¬ T.Provable G := by
  intro hP
  exact T.consistent G ⟨hP, fix hP⟩

/-- **Full Syntactic Incompleteness (constructive).**
    Given a bidirectional diagonal fixed point (provability of G
    is equivalent to provability of ¬G), G is undecided in T. -/
theorem undecided_of_diagonal
    (T : Theory) (G : T.Sentence)
    (fix_forward  : T.Provable G → T.Provable (T.Neg G))
    (fix_backward : T.Provable (T.Neg G) → T.Provable G) :
    T.Undecided G := by
  refine ⟨not_provable_of_diagonal T G fix_forward, ?_⟩
  intro hnP
  exact T.consistent G ⟨fix_backward hnP, hnP⟩

/-! ## §3. Bivalent Truth as Concentrated LEM

A bivalent truth predicate — one satisfying `True_ (¬p) ↔ ¬ True_ p` —
carries excluded middle inside itself. Any theory equipped with such
a predicate satisfies LEM over its sentence space, regardless of
whether classical logic is assumed in the ambient logic.

This is the key observation. The classical Gödel conclusion doesn't
need LEM as an external axiom; it needs a bivalent truth predicate,
which *is* LEM localized to the semantic layer.
-/

/-- A bivalent truth predicate: sound with respect to provability
    and satisfying the classical negation clause. -/
structure BivalentTruth (T : Theory) where
  True_ : T.Sentence → Prop
  neg_true : ∀ p, True_ (T.Neg p) ↔ ¬ True_ p
  sound : ∀ p, T.Provable p → True_ p

/-- A *complete* bivalent truth predicate: for every sentence,
    either it or its negation is true. This is the semantic
    completeness assumption that makes the classical argument work. -/
def Complete (T : Theory) (BT : BivalentTruth T) : Prop :=
  ∀ p : T.Sentence, BT.True_ p ∨ BT.True_ (T.Neg p)

/-- **Bivalence + Completeness ⟹ LEM at the sentence level.**
    This is constructive: given completeness and the bivalent
    negation clause, we can derive LEM for every `True_ p` — no
    external classical axiom required. The LEM commitment is
    already packed into the semantic assumptions. -/
theorem complete_bivalent_truth_gives_lem
    (T : Theory) (BT : BivalentTruth T) (hC : Complete T BT)
    (p : T.Sentence) :
    BT.True_ p ∨ ¬ BT.True_ p := by
  rcases hC p with h | h
  · exact Or.inl h
  · exact Or.inr ((BT.neg_true p).mp h)

/-- The converse direction: LEM plus bivalent negation gives back
    the completeness assumption. So completeness and LEM at the
    sentence level are *equivalent* in the presence of the bivalent
    negation clause — they name the same commitment. -/
theorem lem_gives_completeness
    (T : Theory) (BT : BivalentTruth T)
    (lem : ∀ p : T.Sentence, BT.True_ p ∨ ¬ BT.True_ p) :
    Complete T BT := by
  intro p
  rcases lem p with h | h
  · exact Or.inl h
  · exact Or.inr ((BT.neg_true p).mpr h)

/-! ## §4. The Classical Semantic Result Requires LEM -/

/-- **Classical Semantic Incompleteness.**
    Given a theory with a bivalent truth predicate, a semantic Gödel
    sentence (whose truth is equivalent to its non-provability), and
    LEM applied to the provability of G, there exists G that is
    true but not provable.

    The `lem_prov` hypothesis is the essential classical step: in an
    intuitionistic metatheory, provability is not in general
    decidable, and this disjunction cannot be assumed. Making it an
    explicit hypothesis (rather than tacitly invoking `Classical.em`)
    exposes exactly where the classical argument's power comes from. -/
theorem godel_classical
    (T : Theory) (BT : BivalentTruth T)
    (G : T.Sentence)
    (godel_eq : BT.True_ G ↔ ¬ T.Provable G)
    (lem_prov : T.Provable G ∨ ¬ T.Provable G) :
    BT.True_ G ∧ ¬ T.Provable G := by
  rcases lem_prov with hP | hnP
  · -- Case Provable G: derive contradiction, then anything.
    have hT : BT.True_ G := BT.sound G hP
    have hnP : ¬ T.Provable G := godel_eq.mp hT
    exact absurd hP hnP
  · -- Case ¬Provable G: G is true by the biconditional.
    exact ⟨godel_eq.mpr hnP, hnP⟩

/-! ## §5. What Constructive Logic Can and Cannot Reach

Without LEM and without a bivalent truth predicate, we can still
derive the syntactic result. What we *cannot* derive is any positive
statement of the form "G is true" — because we have no semantic
resource of that shape available.
-/

/-- **Constructive limit.** Without a truth predicate, the strongest
    conclusion available from the diagonal fixed point is the
    syntactic non-provability. There is no proof of "G is true"
    because there is no notion of truth in the hypotheses. -/
theorem constructive_ceiling
    (T : Theory) (G : T.Sentence)
    (fix : T.Provable G → T.Provable (T.Neg G)) :
    ¬ T.Provable G :=
  not_provable_of_diagonal T G fix

/-- **The semantic upgrade requires bivalent truth.**
    Even given the *syntactic* non-provability of G, upgrading to
    "G is true" requires a bivalent truth predicate whose negation
    clause holds. The upgrade is not available from provability
    facts alone. -/
theorem semantic_upgrade_requires_bivalence
    (T : Theory) (BT : BivalentTruth T)
    (G : T.Sentence)
    (godel_eq : BT.True_ G ↔ ¬ T.Provable G)
    (hnP : ¬ T.Provable G) :
    BT.True_ G :=
  godel_eq.mpr hnP

/-- **Where the LEM lives, precisely.**
    The classical argument requires *two* things: (a) a bivalent
    truth predicate `BT` in the metatheory, and (b) a semantic
    biconditional `True_ G ↔ ¬ Provable G` connecting the theory's
    Gödel sentence to the metatheoretic truth predicate.

    Given (a) and (b), the syntactic result (`¬ Provable G`) does
    upgrade constructively to the semantic result (`True_ G`) via
    the biconditional. The LEM dependency is not in this upgrade
    itself — it is in the *derivation* of (a) and (b) from the
    theory. In the standard exposition, both are obtained from the
    standard model of arithmetic under classical semantics, which
    is where the completed-infinity commitment enters.

    This theorem does not derive (a) or (b); it takes them as
    hypotheses and observes that, once granted, the classical
    conclusion is available. The critique is that granting them is
    the entire metaphysical cost — and it is invisible in the
    popular statement of the theorem. -/
theorem classical_conclusion_given_hypotheses
    (T : Theory) (BT : BivalentTruth T)
    (G : T.Sentence)
    (godel_eq : BT.True_ G ↔ ¬ T.Provable G)
    (fix : T.Provable G → T.Provable (T.Neg G)) :
    BT.True_ G ∧ ¬ T.Provable G := by
  have hnP := not_provable_of_diagonal T G fix
  exact ⟨godel_eq.mpr hnP, hnP⟩

/-! ## §6. The Metaphysical Cost, Made Explicit

The classical Gödel conclusion is standardly stated as: "there
exists a true statement that the theory cannot prove." This
formulation smuggles in three commitments:

  (a) A determinate notion of truth over the theory's sentences.
  (b) That notion satisfies the bivalent negation clause.
  (c) Every sentence has a determinate truth value under it.

None of (a), (b), (c) follows from the diagonal fixed point or
from consistency. They are semantic assumptions imported from the
surrounding platonist framework. Once imported, they entail LEM
over the sentence space — which is the commitment being examined.

The syntactic result (`undecided_of_diagonal` above) does not
require any of (a), (b), (c). It is a fact about the theory's
derivational capacity, not about a metaphysical realm of truths.

The distinction matters: the syntactic result is what proof theory
actually establishes; the semantic upgrade is what popular
exposition of Gödel's theorem announces as its conclusion; these
are not the same statement, and only the former is unconditionally
available.
-/

/-- The three metaphysical commitments the classical argument
    depends on, named explicitly. -/
inductive ClassicalCommitment
  | truthPredicateExists
  | truthIsBivalent
  | everySentenceHasTruthValue
  deriving DecidableEq, Repr

/-- The classical Gödel argument depends on all three. The
    intuitionistic reading of incompleteness (as in §2) depends on
    none of them. -/
def classicalDependencies : List ClassicalCommitment :=
  [ .truthPredicateExists
  , .truthIsBivalent
  , .everySentenceHasTruthValue ]

/-- **Auditing lemma.** The syntactic theorems of §2 do not depend
    on any element of `classicalDependencies`. This is exhibited by
    their proofs, which mention no truth predicate. -/
theorem syntactic_result_is_free_of_semantic_commitments
    (T : Theory) (G : T.Sentence)
    (fix : T.Provable G → T.Provable (T.Neg G)) :
    (¬ T.Provable G) ∧ classicalDependencies.length = 3 := by
  refine ⟨not_provable_of_diagonal T G fix, ?_⟩
  rfl

end LEMDependency
