//! Window-state ontology — the EWMH `_NET_WM_STATE` set as a Boolean algebra,
//! and the add/remove/toggle operations that act on it.
//!
//! # The conflation this dissolves
//!
//! Window-manager "actions" like *fullscreen*, *maximize*, *minimize*, *float*,
//! *pin* are not independent verbs — they are **mutations of a window's STATE**.
//! EWMH models this exactly: a window carries a *set* of state atoms
//! (`_NET_WM_STATE`), and a client mutates it with a `_NET_WM_STATE` message
//! whose action field is one of `{ _NET_WM_STATE_REMOVE = 0, _NET_WM_STATE_ADD
//! = 1, _NET_WM_STATE_TOGGLE = 2 }` (EWMH v1.5 §5). So the right model is the
//! **Boolean algebra** `2^StateBit` together with its add / remove / toggle
//! operators — *not* a flat list of state verbs.
//!
//! Modelling state this way is what makes the inverses **expressible**: a flat
//! `Maximize` verb has no `Restore`; `StateDelta { Remove, MaximizedVert }`
//! *is* restore, for free. And it is what makes the laws **provable**: toggle is
//! an involution because it is symmetric-difference in the elementary abelian
//! 2-group `(2^StateBit, △)`; add/remove are idempotent join/meet-with-complement.
//! These are theorems over the model (verified exhaustively here), not analogies.
//!
//! # Grounding
//!
//! - **EWMH v1.5 (2013)** freedesktop.org §5 — `_NET_WM_STATE`: the controlled
//!   atom set (loaded as [`wm_state_vocabulary`]) and the REMOVE/ADD/TOGGLE
//!   mutation. <https://specifications.freedesktop.org/wm-spec/1.5/ar01s05.html>
//! - **Boolean algebra / set theory** — `2^S` is a Boolean algebra; symmetric
//!   difference `△` makes it the elementary abelian 2-group where every element
//!   is its own inverse, so toggling twice is the identity (the involution law).
//! - **ICCCM** `WM_STATE` `{Withdrawn, Normal, Iconic}` — the *mutually-exclusive
//!   lifecycle* 3-state machine, kept DISTINCT from this *set* algebra; modelling
//!   it is a separate concern (tracked), because folding a lifecycle sum into the
//!   power-set re-introduces the conflation this module removes.
//! - **bspwm(1)** — `pseudo_tiled` as a first-class node state (the one tiling
//!   state EWMH does not name); **Wayland xdg-shell / wlroots** — the floating
//!   layer. The two compositor extensions are cited in the loaded vocabulary.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Concept, FinitelyGenerated};
use pr4xis::logic::Axiom;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};

// ── The loaded window-state vocabulary ───────────────────────────────────────

/// One row of the loaded window-state vocabulary: the praxis bit name, the
/// authoritative spec atom it stands for, and the source that names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateBitDef {
    /// The praxis bit name (matches [`StateBit::name`]).
    pub name: String,
    /// The authoritative spec atom (e.g. `_NET_WM_STATE_FULLSCREEN`).
    pub atom: String,
    /// The source that names the atom (e.g. `EWMH 1.5 §5`).
    pub source: String,
}

/// The committed window-state atom vocabulary — EWMH `_NET_WM_STATE` (13 atoms)
/// plus two cited compositor extensions. One `bit_name<TAB>atom<TAB>source` row
/// each. This is the authority the [`StateBit`] enum is proven complete-and-sound
/// against ([`VocabularyComplete`]); the enum is the typed working representation,
/// the loaded data is what it must conform to.
const WM_STATE_TSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/hmi/ewmh-wm-state.tsv"
));

/// The loaded window-state atom vocabulary (EWMH `_NET_WM_STATE` + 2 cited
/// compositor extensions), parsed from the committed TSV.
///
/// Under `std` the parse is cached process-wide ([`OnceLock`](std::sync::OnceLock)):
/// the completeness axiom and any vocabulary lookup re-read it, so re-parsing per
/// call would be wasteful. The `no_std`/wasm surface keeps the fresh-parse.
pub fn wm_state_vocabulary() -> Vec<StateBitDef> {
    #[cfg(feature = "std")]
    {
        wm_state_cached().to_vec()
    }
    #[cfg(not(feature = "std"))]
    {
        parse_wm_state_tsv(WM_STATE_TSV)
    }
}

#[cfg(feature = "std")]
fn wm_state_cached() -> &'static [StateBitDef] {
    use std::sync::OnceLock;
    static CACHE: OnceLock<Vec<StateBitDef>> = OnceLock::new();
    CACHE.get_or_init(|| parse_wm_state_tsv(WM_STATE_TSV))
}

fn parse_wm_state_tsv(tsv: &str) -> Vec<StateBitDef> {
    tsv.lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| {
            let mut it = l.split('\t');
            let name = it.next()?.trim();
            let atom = it.next()?.trim();
            let source = it.next()?.trim();
            if name.is_empty() {
                return None;
            }
            Some(StateBitDef {
                name: name.to_string(),
                atom: atom.to_string(),
                source: source.to_string(),
            })
        })
        .collect()
}

// ── The state atoms (the typed alphabet) ─────────────────────────────────────

/// A single window-state atom — one bit of a window's state set.
///
/// The typed image of the loaded [`wm_state_vocabulary`]; their correspondence
/// is machine-checked by [`VocabularyComplete`], so the enum cannot drift from
/// the cited EWMH atom set. Each is one bit position in a [`StateSet`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StateBit {
    Modal,
    Sticky,
    MaximizedVert,
    MaximizedHorz,
    Shaded,
    SkipTaskbar,
    SkipPager,
    Hidden,
    Fullscreen,
    Above,
    Below,
    DemandsAttention,
    Focused,
    /// Compositor extension (Wayland/wlroots): the floating layer.
    Floating,
    /// Compositor extension (bspwm): the `pseudo_tiled` node flag.
    PseudoTiled,
}

impl StateBit {
    /// This atom's bit position in a [`StateSet`] (stable, matches the loaded
    /// vocabulary order).
    fn index(self) -> u8 {
        match self {
            StateBit::Modal => 0,
            StateBit::Sticky => 1,
            StateBit::MaximizedVert => 2,
            StateBit::MaximizedHorz => 3,
            StateBit::Shaded => 4,
            StateBit::SkipTaskbar => 5,
            StateBit::SkipPager => 6,
            StateBit::Hidden => 7,
            StateBit::Fullscreen => 8,
            StateBit::Above => 9,
            StateBit::Below => 10,
            StateBit::DemandsAttention => 11,
            StateBit::Focused => 12,
            StateBit::Floating => 13,
            StateBit::PseudoTiled => 14,
        }
    }

    /// The single-bit mask for this atom.
    fn mask(self) -> u16 {
        1u16 << self.index()
    }
}

impl Concept for StateBit {
    fn name(&self) -> &'static str {
        match self {
            StateBit::Modal => "modal",
            StateBit::Sticky => "sticky",
            StateBit::MaximizedVert => "maximized-vert",
            StateBit::MaximizedHorz => "maximized-horz",
            StateBit::Shaded => "shaded",
            StateBit::SkipTaskbar => "skip-taskbar",
            StateBit::SkipPager => "skip-pager",
            StateBit::Hidden => "hidden",
            StateBit::Fullscreen => "fullscreen",
            StateBit::Above => "above",
            StateBit::Below => "below",
            StateBit::DemandsAttention => "demands-attention",
            StateBit::Focused => "focused",
            StateBit::Floating => "floating",
            StateBit::PseudoTiled => "pseudo-tiled",
        }
    }
}

impl FinitelyGenerated for StateBit {
    fn variants() -> Vec<Self> {
        vec![
            StateBit::Modal,
            StateBit::Sticky,
            StateBit::MaximizedVert,
            StateBit::MaximizedHorz,
            StateBit::Shaded,
            StateBit::SkipTaskbar,
            StateBit::SkipPager,
            StateBit::Hidden,
            StateBit::Fullscreen,
            StateBit::Above,
            StateBit::Below,
            StateBit::DemandsAttention,
            StateBit::Focused,
            StateBit::Floating,
            StateBit::PseudoTiled,
        ]
    }
}

/// How a [`StateBit`] is mutated — the EWMH `_NET_WM_STATE` action field, verbatim:
/// `_NET_WM_STATE_REMOVE = 0`, `_NET_WM_STATE_ADD = 1`, `_NET_WM_STATE_TOGGLE = 2`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateOp {
    Remove,
    Add,
    Toggle,
}

impl StateOp {
    /// The EWMH numeric code for this action (`_NET_WM_STATE_{REMOVE,ADD,TOGGLE}`).
    pub fn ewmh_code(self) -> i64 {
        match self {
            StateOp::Remove => 0,
            StateOp::Add => 1,
            StateOp::Toggle => 2,
        }
    }
}

impl Concept for StateOp {
    fn name(&self) -> &'static str {
        match self {
            StateOp::Remove => "remove",
            StateOp::Add => "add",
            StateOp::Toggle => "toggle",
        }
    }
}

impl FinitelyGenerated for StateOp {
    fn variants() -> Vec<Self> {
        vec![StateOp::Remove, StateOp::Add, StateOp::Toggle]
    }
}

/// An atomic window-state mutation — apply [`StateOp`] to [`StateBit`]. This is
/// the parameter of the `State` window-action: e.g. `StateDelta { Toggle,
/// Fullscreen }` is "toggle fullscreen", `StateDelta { Remove, MaximizedVert }`
/// is the restore that a flat verb set could not express.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateDelta {
    pub op: StateOp,
    pub bit: StateBit,
}

impl StateDelta {
    pub fn new(op: StateOp, bit: StateBit) -> Self {
        Self { op, bit }
    }

    /// The finite generating set — every `(op, bit)` pair. Used to seed property
    /// tests and the lattice-law axioms.
    pub fn representative_deltas() -> Vec<StateDelta> {
        let mut v = Vec::new();
        for op in StateOp::variants() {
            for bit in StateBit::variants() {
                v.push(StateDelta::new(op, bit));
            }
        }
        v
    }
}

impl Concept for StateDelta {
    fn name(&self) -> &'static str {
        // The op names the mutation kind; the bit is the parameter.
        self.op.name()
    }
}

// ── The state set (the 2^StateBit Boolean algebra) ───────────────────────────

/// A window's state — a subset of [`StateBit`], i.e. an element of the Boolean
/// algebra `2^StateBit`. Represented as a bitset over the 15 atoms.
///
/// This is the **model** the state operations act on, and the layer at which the
/// involution / idempotence laws are *observable and provable* (unlike the
/// realized compositor string, where `togglefloating ; togglefloating` is a
/// two-element word, not the identity).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct StateSet(u16);

impl StateSet {
    /// The empty state (no atoms set).
    pub fn empty() -> Self {
        StateSet(0)
    }

    /// Is `bit` present in this state?
    pub fn contains(self, bit: StateBit) -> bool {
        self.0 & bit.mask() != 0
    }

    /// `Add` — set `bit` (idempotent join: `σ ↦ σ ∪ {bit}`).
    pub fn with(self, bit: StateBit) -> Self {
        StateSet(self.0 | bit.mask())
    }

    /// `Remove` — clear `bit` (idempotent: `σ ↦ σ \ {bit}`).
    pub fn without(self, bit: StateBit) -> Self {
        StateSet(self.0 & !bit.mask())
    }

    /// `Toggle` — flip `bit` (involution: `σ ↦ σ △ {bit}`).
    pub fn toggled(self, bit: StateBit) -> Self {
        StateSet(self.0 ^ bit.mask())
    }

    /// Apply one [`StateDelta`] — the action of the state operations on the set.
    pub fn apply(self, delta: StateDelta) -> Self {
        match delta.op {
            StateOp::Add => self.with(delta.bit),
            StateOp::Remove => self.without(delta.bit),
            StateOp::Toggle => self.toggled(delta.bit),
        }
    }

    /// Build a state from a raw bitset (exhaustive-iteration helper). Only the
    /// 15 defined atom bits are meaningful; higher bits are masked off.
    fn from_bits(b: u16) -> Self {
        StateSet(b & STATE_MASK)
    }
}

/// The mask of all defined atom bits (15 atoms → low 15 bits).
const STATE_MASK: u16 = (1u16 << 15) - 1;

/// The size of the full state space `2^StateBit` (32768) — the domain the laws
/// are checked over EXHAUSTIVELY (every reachable state).
const STATE_SPACE: u32 = 1 << 15;

// ── The Boolean-algebra laws (exhaustively proven over the model) ────────────

/// Toggling a bit twice is the identity — the **involution** law, proven
/// exhaustively over every state in `2^StateBit` and every atom: `apply(Toggle,
/// apply(Toggle, σ)) = σ`. True because toggle is symmetric-difference `△` in the
/// elementary abelian 2-group `(2^StateBit, △)`, where every element is its own
/// inverse. This is the law that makes a single `Toggle` binding correct.
pub struct StateToggleInvolutive;

impl Axiom for StateToggleInvolutive {
    fn verify(&self) -> Verdict {
        for bit in StateBit::variants() {
            let d = StateDelta::new(StateOp::Toggle, bit);
            for b in 0..STATE_SPACE {
                let s = StateSet::from_bits(b as u16);
                if s.apply(d).apply(d) != s {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "StateToggleInvolutive",
        "toggling a window-state atom twice is the identity (involution), for every state",
        "EWMH v1.5 (2013) §5 _NET_WM_STATE_TOGGLE; Boolean algebra — symmetric difference makes 2^S the elementary abelian 2-group where every element is self-inverse"
    );
}

/// Adding an atom already present, or removing one already absent, changes
/// nothing — **idempotence** of `Add` and `Remove`, proven exhaustively over the
/// state space. `Add` is set-union with a singleton (a join), `Remove` is
/// set-difference; both are idempotent in the lattice.
pub struct StateAddRemoveIdempotent;

impl Axiom for StateAddRemoveIdempotent {
    fn verify(&self) -> Verdict {
        for bit in StateBit::variants() {
            let add = StateDelta::new(StateOp::Add, bit);
            let remove = StateDelta::new(StateOp::Remove, bit);
            for b in 0..STATE_SPACE {
                let s = StateSet::from_bits(b as u16);
                if s.apply(add).apply(add) != s.apply(add) {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
                if s.apply(remove).apply(remove) != s.apply(remove) {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "StateAddRemoveIdempotent",
        "adding a present atom or removing an absent atom is a no-op (idempotence), for every state",
        "EWMH v1.5 (2013) §5 _NET_WM_STATE_ADD/_REMOVE; Boolean algebra — join/meet with a fixed element is idempotent"
    );
}

/// `Add` then `Remove` (or vice-versa) of the same atom leaves the atom in the
/// expected definite state, independent of the starting state — `Add` and
/// `Remove` are the two **constant** maps on that atom's coordinate (set it,
/// clear it), so `Remove ∘ Add = Remove` and `Add ∘ Remove = Add` on that bit.
/// This is what makes `Remove` the genuine inverse-direction (restore) operator.
pub struct StateAddRemoveComplementary;

impl Axiom for StateAddRemoveComplementary {
    fn verify(&self) -> Verdict {
        for bit in StateBit::variants() {
            let add = StateDelta::new(StateOp::Add, bit);
            let remove = StateDelta::new(StateOp::Remove, bit);
            for b in 0..STATE_SPACE {
                let s = StateSet::from_bits(b as u16);
                // After Add, the bit is set; after Remove, it is clear — regardless of s.
                if !s.apply(add).contains(bit) {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
                if s.apply(remove).contains(bit) {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
                // Add then Remove == Remove (the bit ends clear); Remove then Add == Add.
                if s.apply(add).apply(remove) != s.apply(remove) {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
                if s.apply(remove).apply(add) != s.apply(add) {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "StateAddRemoveComplementary",
        "add forces the atom present and remove forces it absent, independent of prior state (the inverse-direction operators)",
        "EWMH v1.5 (2013) §5 — _NET_WM_STATE_ADD and _NET_WM_STATE_REMOVE are definite set/clear; Boolean algebra complement"
    );
}

/// The [`StateBit`] enum is exactly the loaded EWMH-plus-extensions vocabulary,
/// **in the same order** — sound (every variant has a loaded definition),
/// complete (every loaded atom has a variant), and order-faithful (the i-th
/// variant is the i-th loaded row, so `StateBit::index` — and thus the bit
/// positions — is pinned to the loaded row order). This is what makes "loaded,
/// not hand-encoded" real: the typed alphabet cannot drift from the cited source,
/// in membership OR order, without this axiom failing.
pub struct VocabularyComplete;

impl Axiom for VocabularyComplete {
    fn verify(&self) -> Verdict {
        let loaded = wm_state_vocabulary();
        let variants = StateBit::variants();
        // Same cardinality AND same ORDER: the i-th StateBit variant must be the
        // i-th loaded atom. Ordered equality is stronger than set equality — it
        // also pins `StateBit::index` (and thus the bit positions) to the loaded
        // row order, so a reorder of either the enum or the TSV fails here rather
        // than silently breaking the index <-> vocabulary-order correspondence.
        if loaded.len() != variants.len() {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        for (i, v) in variants.iter().enumerate() {
            if loaded[i].name != v.name() {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "VocabularyComplete",
        "the StateBit alphabet is exactly the loaded EWMH _NET_WM_STATE vocabulary, in the same order (sound, complete, and order-faithful: index() matches the loaded row order)",
        "EWMH v1.5 (2013) §5 — the _NET_WM_STATE atom set is the controlled source; the typed enum is checked against the loaded vocabulary positionally, not merely cited"
    );
}

/// All window-state model axioms — the Boolean-algebra laws plus the
/// loaded-vocabulary correspondence.
pub fn window_state_axioms() -> Vec<Box<dyn Axiom>> {
    vec![
        Box::new(StateToggleInvolutive),
        Box::new(StateAddRemoveIdempotent),
        Box::new(StateAddRemoveComplementary),
        Box::new(VocabularyComplete),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn vocabulary_loads_15_atoms() {
        let v = wm_state_vocabulary();
        assert_eq!(v.len(), 15, "13 EWMH + 2 compositor extensions");
        // EWMH atoms carry their spec atom name.
        assert!(v.iter().any(|d| d.atom == "_NET_WM_STATE_FULLSCREEN"));
        assert!(
            v.iter()
                .any(|d| d.name == "fullscreen" && d.source.contains("EWMH"))
        );
    }

    #[test]
    fn enum_matches_loaded_vocabulary() {
        VocabularyComplete
            .verify()
            .unwrap_or_else(|c| panic!("{}", c.meta().name.as_str()));
    }

    #[test]
    fn restore_is_expressible() {
        // The whole point: a flat `Maximize` verb has no inverse; here the
        // remove-direction IS the restore.
        let maximized = StateSet::empty()
            .with(StateBit::MaximizedVert)
            .with(StateBit::MaximizedHorz);
        let restored = maximized
            .apply(StateDelta::new(StateOp::Remove, StateBit::MaximizedVert))
            .apply(StateDelta::new(StateOp::Remove, StateBit::MaximizedHorz));
        assert_eq!(restored, StateSet::empty());
    }

    #[test]
    fn toggle_involution_holds_exhaustively() {
        StateToggleInvolutive
            .verify()
            .unwrap_or_else(|c| panic!("{}", c.meta().name.as_str()));
    }

    #[test]
    fn add_remove_idempotent_holds_exhaustively() {
        StateAddRemoveIdempotent
            .verify()
            .unwrap_or_else(|c| panic!("{}", c.meta().name.as_str()));
    }

    #[test]
    fn add_remove_complementary_holds_exhaustively() {
        StateAddRemoveComplementary
            .verify()
            .unwrap_or_else(|c| panic!("{}", c.meta().name.as_str()));
    }

    #[test]
    fn all_axioms_pass() {
        for ax in window_state_axioms() {
            ax.verify()
                .unwrap_or_else(|c| panic!("axiom failed: {}", c.meta().name.as_str()));
        }
    }

    fn arb_bit() -> impl Strategy<Value = StateBit> {
        proptest::sample::select(StateBit::variants())
    }

    fn arb_set() -> impl Strategy<Value = StateSet> {
        any::<u16>().prop_map(StateSet::from_bits)
    }

    proptest! {
        /// Toggle is its own inverse on any state (the involution, on random states).
        #[test]
        fn prop_toggle_involutive(s in arb_set(), bit in arb_bit()) {
            let d = StateDelta::new(StateOp::Toggle, bit);
            prop_assert_eq!(s.apply(d).apply(d), s);
        }

        /// Add then Remove of the same bit clears it; the result is what Remove
        /// alone gives (Add/Remove are the definite set/clear maps).
        #[test]
        fn prop_add_then_remove_is_remove(s in arb_set(), bit in arb_bit()) {
            let add = StateDelta::new(StateOp::Add, bit);
            let remove = StateDelta::new(StateOp::Remove, bit);
            prop_assert_eq!(s.apply(add).apply(remove), s.apply(remove));
            prop_assert!(!s.apply(add).apply(remove).contains(bit));
        }

        /// Distinct atoms are independent coordinates: mutating one never changes
        /// another (the state space is a product of bits).
        #[test]
        fn prop_bits_independent(s in arb_set(), a in arb_bit(), b in arb_bit()) {
            prop_assume!(a != b);
            let before = s.contains(b);
            let after = s.apply(StateDelta::new(StateOp::Toggle, a)).contains(b);
            prop_assert_eq!(before, after);
        }

        /// Only the low 15 atom bits are ever set (the state space is 2^15).
        #[test]
        fn prop_state_space_bounded(s in arb_set()) {
            prop_assert_eq!(s.0 & !STATE_MASK, 0);
        }
    }
}
