/-
Open Systems Have Lossy Interfaces
==================================

Author:  Ido Samuelson
Date:    July 2026
License: CC BY-SA 4.0

A Lean 4 formalization of the axiom: an open system cannot have a
complete lens or adjunction with its environment; the interface must
lose information.

This is the information-theoretic dual of the openability principle
in `ProvisionalClosure.lean`. Where openability says every closure can
be extended, this file says every open interface must be lossy. The
two together characterize the relationship between closure and
information:

  closed  ⟺  complete self-description possible, lossless self-lens
  open    ⟺  self-description must lose information across boundary

The claim is provable, not merely postulated: given a workable
definition of openness (the environment has distinct states that
the system's view identifies), losslessness of any lens with that
view is impossible. Openness *is* information collapse.

The file also states the categorical form: an adjunction F ⊣ G
between environment and system is an equivalence iff the coupling
is lossless iff the system is closed with respect to that
environment.
-/

namespace OpenSystemsLossy

/-! ## §1. Lenses -/

/-- A very-well-behaved lens between source `S` and view `V`. The
    three lens laws (get-put, put-get, put-put) express that `get`
    and `put` compose as a projection with a section. -/
structure Lens (S V : Type) where
  get     : S → V
  put     : S → V → S
  get_put : ∀ s v, get (put s v) = v
  put_get : ∀ s, put s (get s) = s
  put_put : ∀ s v₁ v₂, put (put s v₁) v₂ = put s v₂

/-- A lens is *lossless* when its `get` distinguishes all source
    states. Equivalently: no two distinct source states project
    to the same view. -/
def Lens.Lossless {S V : Type} (L : Lens S V) : Prop :=
  ∀ s₁ s₂ : S, L.get s₁ = L.get s₂ → s₁ = s₂

/-- A lens is *complete* when it is lossless and its source type
    is inhabited (so that `get` is also surjective — the lens laws
    give surjectivity of `get` via `get_put`, provided the source
    has at least one state). -/
def Lens.Complete {S V : Type} (L : Lens S V) : Prop :=
  L.Lossless ∧ Nonempty S

/-- Given nonempty source, the lens laws force `get` to be
    surjective: for any view `v`, `put s₀ v` is a preimage. -/
theorem lens_get_surjective_of_nonempty
    {S V : Type} (L : Lens S V) (h : Nonempty S) :
    Function.Surjective L.get := by
  intro v
  obtain ⟨s₀⟩ := h
  exact ⟨L.put s₀ v, L.get_put s₀ v⟩

/-- Completeness reduces to losslessness plus nonemptiness of the
    source; the surjectivity half is free from the lens laws. -/
theorem lens_complete_iff_lossless_and_nonempty
    {S V : Type} (L : Lens S V) :
    L.Complete ↔ L.Lossless ∧ Nonempty S := Iff.rfl

/-! ## §2. Open Systems -/

/-- A system is *open with respect to a view* when the view
    identifies two distinct source states. This is the operational
    definition of openness: from inside the system, information
    about the environment is being collapsed.

    This is not a cardinality statement — it says something specific:
    the view function is not injective. Distinct environmental
    configurations produce the same system-visible state. That's
    what it means for the boundary to be permeable in a way the
    system can't fully track. -/
def IsOpenWRT {Env Sys : Type} (view : Env → Sys) : Prop :=
  ∃ e₁ e₂ : Env, e₁ ≠ e₂ ∧ view e₁ = view e₂

/-- The dual: a system is *closed with respect to a view* when the
    view is injective — every distinguishable environmental state
    is preserved in the system's picture of it. -/
def IsClosedWRT {Env Sys : Type} (view : Env → Sys) : Prop :=
  ∀ e₁ e₂ : Env, view e₁ = view e₂ → e₁ = e₂

/-- Openness and closedness (with respect to a view) are exact
    negations of each other. The backward direction uses classical
    logic (existence from non-universality). -/
theorem open_iff_not_closed {Env Sys : Type} (view : Env → Sys) :
    IsOpenWRT view ↔ ¬ IsClosedWRT view := by
  constructor
  · intro ⟨e₁, e₂, hne, heq⟩ hC
    exact hne (hC e₁ e₂ heq)
  · intro hNC
    apply Classical.byContradiction
    intro hno
    apply hNC
    intro e₁ e₂ heq
    apply Classical.byContradiction
    intro hne
    exact hno ⟨e₁, e₂, hne, heq⟩

/-! ## §3. The Lossy Interface Theorem -/

/-- **Theorem (Lossy Interface).** If a system is open with respect
    to the `get` view of a lens, then that lens cannot be lossless.
    Openness is exactly the failure of lossless interface. -/
theorem open_system_has_lossy_lens
    {Env Sys : Type} (L : Lens Env Sys)
    (h : IsOpenWRT L.get) :
    ¬ L.Lossless := by
  intro hLL
  obtain ⟨e₁, e₂, hne, heq⟩ := h
  exact hne (hLL e₁ e₂ heq)

/-- **Contrapositive form.** A lossless lens forces the system to
    be closed with respect to its view. Complete boundary
    description is only possible for closed systems. -/
theorem lossless_lens_forces_closed
    {Env Sys : Type} (L : Lens Env Sys)
    (hL : L.Lossless) :
    IsClosedWRT L.get := by
  intro e₁ e₂ heq
  exact hL e₁ e₂ heq

/-- **The characterization.** Openness with respect to a lens's
    view is exactly equivalent to the lens being lossy. Not
    accidentally related — the two notions coincide. -/
theorem open_iff_lens_lossy
    {Env Sys : Type} (L : Lens Env Sys) :
    IsOpenWRT L.get ↔ ¬ L.Lossless := by
  constructor
  · exact open_system_has_lossy_lens L
  · intro hNL
    apply Classical.byContradiction
    intro hno
    apply hNL
    intro e₁ e₂ heq
    apply Classical.byContradiction
    intro hne
    exact hno ⟨e₁, e₂, hne, heq⟩

/-! ## §4. Categorical Form: No Equivalence for Open Systems

An adjunction F ⊣ G between environment and system is an
*equivalence* when unit and counit are natural isomorphisms.
Restricted to the setoid/type level, this becomes: F is a
bijection with G as inverse. We formalize this level.
-/

/-- An equivalence of types: bijection witnessed by an inverse pair. -/
structure TypeEquiv (Env Sys : Type) where
  toFun    : Env → Sys
  invFun   : Sys → Env
  left_inv : ∀ e, invFun (toFun e) = e
  right_inv: ∀ s, toFun (invFun s) = s

/-- The forward direction of an equivalence is always injective,
    hence lossless in the sense of §1. -/
theorem equiv_toFun_injective
    {Env Sys : Type} (E : TypeEquiv Env Sys) :
    ∀ e₁ e₂ : Env, E.toFun e₁ = E.toFun e₂ → e₁ = e₂ := by
  intro e₁ e₂ heq
  have : E.invFun (E.toFun e₁) = E.invFun (E.toFun e₂) := by rw [heq]
  rw [E.left_inv, E.left_inv] at this
  exact this

/-- **Theorem (No Equivalence for Open Systems).**
    If an environment and system are open with respect to some
    view, then there is no equivalence between them extending
    that view. Openness rules out categorical equivalence. -/
theorem no_equivalence_when_open
    {Env Sys : Type}
    (E : TypeEquiv Env Sys)
    (h : IsOpenWRT E.toFun) :
    False := by
  obtain ⟨e₁, e₂, hne, heq⟩ := h
  exact hne (equiv_toFun_injective E e₁ e₂ heq)

/-! ## §5. Composition with the Provisional-Closure Framework

Combined with the openability principle from `ProvisionalClosure.lean`,
this file establishes the following architecture:

  • Every closure is provisional (openability axiom).
  • Every open interface loses information (this file).
  • Therefore: extending a closure to accommodate a previously
    undecided proposition either (i) enlarges the closure while
    keeping the system self-describable, or (ii) opens the closure
    to an environment, at which point the interface is
    unavoidably lossy.

This is not a defect. It is what "open" means. A system whose
interfaces are lossless has no environment; it is a closed universe
unto itself. Any system embedded in a larger context — which is to
say, any real system — pays the information cost of being embedded,
and that cost is exactly the lens's lossiness.

This connects to physical intuitions in a direct way:

  • Thermodynamics: open systems produce entropy at the boundary.
  • Information theory: lossy channels have capacity < 1.
  • Category theory: non-equivalences have non-invertible
    unit or counit.
  • Systems theory: the boundary of an open system is where the
    system's model of the environment fails to be complete.

All four are the same fact, seen from four different frames. The
theorem `open_iff_lens_lossy` above is the formal statement common
to all of them.
-/

/-! ## §6. The Complete Axiomatic Statement

For emphasis: this is not a limitation imposed on open systems by
choice of formalism. It is what openness *is*. Any formalism that
represents openness as a real phenomenon — as distinct from
closure — must have this property, because the alternative
(lossless open interface) is definitionally equivalent to closure.

pr4xis handles this correctly. Each ontology is a closed system
with complete internal self-description. Analogy<F> is a functor
between closed systems, which may or may not be an equivalence —
when it isn't, it captures a lossy interface between the two
ontologies, honestly. Property verification samples the closed
ontology's state space, which is well-defined precisely because
the system is closed. The moment an ontology attempts to describe
its own environment (the world it models), the description becomes
lossy — and pr4xis's PipelineTrace makes the loss inspectable
rather than hidden.
-/

end OpenSystemsLossy
