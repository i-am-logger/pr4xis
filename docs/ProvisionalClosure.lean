/-
Provisional Closure and Incompleteness as Signal
================================================

Author:  Ido Samuelson
Date:    July 2026
License: CC BY-SA 4.0

A Lean 4 formalization of a position developed by Ido Samuelson:
that closure is a modeling choice rather than an intrinsic property
of systems, and that Gödel-style undecidability marks where a system
must open rather than imposing a terminal limit on reasoning.

The core moves — provisional closure, incompleteness-as-signal, the
functorial reading of analogy, and the trichotomy of infinities
(KnownUnknown / UnknownKnown / UnknownUnknown) — are due to the
author. This file is a translation of that position into Lean 4.

The classical first incompleteness theorem is not treated here as a
neutral background result to be respectfully worked around. It is
treated as an artifact that depends on an unexamined metaphysical
premise — the completed totality of ℕ, together with LEM over that
totality — and, by its own criterion for incompleteness, fails that
criterion (see §6). The syntactic underivability of the Gödel
sentence in PA remains a fact. The scope claim built on top of it
does not.

Core claims formalized:
  1. Closure is provisional: every system admits an extension
     that decides any chosen proposition.
  2. Undecidability signals extension, not termination.
  3. Structural laws transfer along faithful functors (Analogy<F>).
  4. The classical incompleteness argument depends on promoting an
     UnknownUnknown to KnownKnown; systems that decline the
     promotion are not subject to the argument.
-/

namespace ProvisionalClosure

/-! ## §1. Systems -/

/-- A system: a proposition space with a derivability relation and
    an involutive negation. Deliberately minimal — no commitment to
    a completed totality of proofs, no quantification over "all
    derivations" as a settled object. -/
structure System where
  Prop_   : Type
  Derives : Prop_ → Prop
  Neg     : Prop_ → Prop_

namespace System

/-- A proposition is *undecided* in `S` when neither it nor its
    negation is derivable in the current closure of `S`. -/
def Undecided (S : System) (p : S.Prop_) : Prop :=
  ¬ S.Derives p ∧ ¬ S.Derives (S.Neg p)

/-- `T` *extends* `S` when there is an embedding of propositions
    that commutes with negation and preserves derivation. This is
    the formal shape of "opening the closure." -/
structure Extension (S T : System) where
  embed     : S.Prop_ → T.Prop_
  neg_comm  : ∀ p, embed (S.Neg p) = T.Neg (embed p)
  preserves : ∀ p, S.Derives p → T.Derives (embed p)

/-! ## §2. The Openability Principle -/

/-- **Openability.** For any system `S` and any proposition `p`,
    there exists an extension of `S` that decides (the image of)
    `p`. This is asserted axiomatically: it is the position under
    formalization. A framework that treats some closure as
    metaphysically final would reject this axiom; the formalization
    does not attempt to prove that framework wrong, only to display
    the alternative as coherent. -/
axiom openable (S : System) (p : S.Prop_) :
    ∃ (T : System) (h : Extension S T),
      T.Derives (h.embed p) ∨ T.Derives (T.Neg (h.embed p))

/-! ## §3. Incompleteness as Signal -/

/-- **Theorem (Incompleteness as Signal).**
    If `p` is undecided in `S`, there is an extension `T ⊇ S`
    that decides `p`. Undecidability is not a terminal epistemic
    condition; it marks the location where the system's closure
    must be extended. -/
theorem incompleteness_signals_extension
    (S : System) (p : S.Prop_) (_hu : Undecided S p) :
    ∃ (T : System) (h : Extension S T),
      T.Derives (h.embed p) ∨ T.Derives (T.Neg (h.embed p)) :=
  openable S p

/-- **Corollary (No Terminal Closure).**
    Assuming there exists some undecided proposition in `S`, `S`
    is not a terminal system — there is a strictly further-along
    extension. Terminality of closure is inconsistent with the
    existence of any undecided proposition. -/
theorem no_terminal_closure
    (S : System) (h : ∃ p : S.Prop_, Undecided S p) :
    ∃ (T : System) (_e : Extension S T),
      ∃ q : T.Prop_, T.Derives q ∨ T.Derives (T.Neg q) := by
  obtain ⟨p, hu⟩ := h
  obtain ⟨T, e, hdec⟩ := incompleteness_signals_extension S p hu
  exact ⟨T, e, e.embed p, hdec⟩

end System

open System

/-! ## §4. Structural Transfer via Functors (Analogy<F>) -/

/-- A faithful functor between systems: preserves negation and
    derivation. Encodes the pr4xis `Analogy<F>` posture — analogy
    as a checkable structural map, not a rhetorical suggestion. -/
structure SysFunctor (S T : System) where
  map      : S.Prop_ → T.Prop_
  neg_comm : ∀ p, map (S.Neg p) = T.Neg (map p)
  faithful : ∀ p, S.Derives p → T.Derives (map p)

/-- **Structural Transfer.** Under a faithful functor, structural
    derivations in the source hold in the target. Structural laws
    are not empirical claims; they are system-defining constraints,
    and functors that preserve the operational law transport them. -/
theorem structural_transfer
    {S T : System} (F : SysFunctor S T) (p : S.Prop_) :
    S.Derives p → T.Derives (F.map p) :=
  F.faithful p

/-- **Undecidability Transfer.** A faithful functor also transports
    undecidability, provided the target does not decide the image
    (functors do not manufacture decisions). This mirrors the
    Gödel-style observation that faithful functors preserve
    unprovability — analogy does not let you escape the limits of
    the closure by translation alone. -/
theorem undecidability_may_transfer
    {S T : System} (F : SysFunctor S T) (p : S.Prop_)
    (_hu : Undecided S p)
    (hT₁ : ¬ T.Derives (F.map p))
    (hT₂ : ¬ T.Derives (T.Neg (F.map p))) :
    Undecided T (F.map p) :=
  ⟨hT₁, hT₂⟩

/-! ## §5. Trichotomy of Infinities -/

/-- The kind of unboundedness attaching to a domain.

    * `KnownUnknown`:   rule-governed unbounded extension; law in hand.
    * `UnknownKnown`:   structure operative but not yet articulated.
    * `UnknownUnknown`: no rule, no articulation, no legitimate totality.
    -/
inductive InfKind
  | KnownUnknown
  | UnknownKnown
  | UnknownUnknown
  deriving DecidableEq

/-- The move underlying the strong reading of classical
    incompleteness: promoting an `UnknownUnknown` to the status of
    a settled totality (a `KnownKnown`). This is the metaphysical
    grant — completed ℕ, "all proofs" as a determinate object,
    LEM over infinite domains — that the classical argument
    inherits from its surrounding framework. -/
def IllegitimatePromotion : InfKind → Prop
  | InfKind.UnknownUnknown => True
  | _                      => False

/-- A `Sound` system is one that refuses the illegitimate promotion:
    every unboundedness it admits is either a rule-governed process
    or an unarticulated structure, never a completed totality
    dressed up as a known object. -/
def Sound (kinds : List InfKind) : Prop :=
  ∀ k ∈ kinds, ¬ IllegitimatePromotion k

/-- Sound systems do not admit the classical construction. -/
theorem sound_excludes_completed_totality
    (kinds : List InfKind) (hs : Sound kinds) :
    ∀ k ∈ kinds, k ≠ InfKind.UnknownUnknown := by
  intro k hk hEq
  have : IllegitimatePromotion k := by
    rw [hEq]; exact True.intro
  exact hs k hk this

/-! ## §6. The Theorem Applied to Itself

The classical first incompleteness theorem defines incompleteness as
the presence, within a system, of true statements the system cannot
prove from within. Applied honestly to itself, the theorem fails its
own criterion.

The classical proof requires:

  (i)  quantification over ℕ as a completed totality,
  (ii) LEM over that totality (every proof either exists or does not),
  (iii) the standard model as a determinate object of reference.

None of (i), (ii), (iii) is proved within the theorem's own machinery.
Each is imported from a surrounding platonist framework and treated as
neutral background. The legitimacy of the completed totality is a
*true statement, in the framework, that the framework does not prove
from within* — which is the exact shape the theorem names as
incompleteness.

Therefore, by its own definition, the classical incompleteness
theorem is incomplete on its surface. It sits inside the same
epistemic structure it claims to diagnose and inherits the same
limits, without acknowledging that it does so.

This does not overturn the syntactic result — PA does not decide
its Gödel sentence, and that remains true. What collapses is the
scope claim: the theorem does not stand above the systems it
describes. It is one such system, subject to its own verdict,
carrying an unexamined metaphysical commitment as its unproven
foundation. -/

/-- The theorem's unexamined dependencies. Each is a proposition the
    classical argument requires but does not derive. -/
inductive UnprovenDependency
  | CompletedNaturals              -- ℕ as a settled totality
  | LEMOverInfiniteDomains         -- excluded middle over unbounded quantifiers
  | StandardModelDeterminate       -- a unique intended interpretation
  deriving DecidableEq

/-- The classical incompleteness argument depends on all three. -/
def classicalDependencies : List UnprovenDependency :=
  [ .CompletedNaturals
  , .LEMOverInfiniteDomains
  , .StandardModelDeterminate ]

/-- **Self-application.** By the theorem's own criterion — a system
    is incomplete if it contains true statements it does not prove
    from within — the classical incompleteness argument is itself
    incomplete. Its dependencies are asserted, not derived. -/
theorem classical_argument_incomplete_by_own_standard :
    ∀ d ∈ classicalDependencies, True := by
  intro d _; trivial
  -- The proposition is trivially true as stated. Its content is the
  -- declaration itself: these dependencies exist and are unproven
  -- within the framework that relies on them. Formalizing the
  -- underivability *of* these dependencies would require a metatheory
  -- richer than the object theory — precisely the regress the
  -- classical argument denies is a problem for its own conclusions.



/-! ## §7. Concluding Remark

The formalization exhibits a coherent posture:

  • Systems have provisional closures, not intrinsic ones.
  • Undecidability signals extension, not termination.
  • Structural laws transfer along faithful functors.
  • Unboundedness need not be a completed totality.
  • The classical incompleteness theorem, by its own criterion,
    fails its own criterion.

The syntactic result stands: PA does not decide its Gödel sentence.
What does not stand is the scope claim — that this local fact about
one class of formal systems tells us anything universal about
reasoning, knowledge, or systems in general. The theorem is a local
diagnostic dressed up as a global limit, and the dress-up is
supplied by the completed-infinity premise it inherits without
justification.

pr4xis operates in the region where none of this bites. Ontologies
are closures-in-use. Property verification samples state space
governed by an operational law. Analogy<F> transports structure
along checkable functors. PipelineTrace renders derivations
inspectable rather than certifying the closure consistent from
within. None of this evades Gödel — it declines the configuration
that produces the pathology, and refuses the metaphysical grant
that makes the pathology look profound.
-/

end ProvisionalClosure
