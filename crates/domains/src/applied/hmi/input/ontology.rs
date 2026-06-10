//! The modal interaction ontology — modes, triggers, and gestures as category
//! theory.
//!
//! This is the categorical model behind a modal keyboard UX (e.g. a tiling-WM
//! "desktop mode" entered by holding CapsLock). It is built entirely from
//! praxis primitives so the design's guarantees are *proven*, not asserted:
//!
//! - **Objects = modes** ([`Mode`]): `app` (root), `desktop`, `move`, `resize`,
//!   `console`.
//! - **Generators = triggers** ([`Trigger`]): the declared single-step
//!   transitions (CapsLock enters desktop, `m`/`r` enter the sub-modes, Esc
//!   exits, …). They form a [`Quiver`] ([`InteractionQuiver`]).
//! - **The interaction ontology = the free category on the trigger quiver**
//!   ([`Interaction`]): its morphisms are *gestures* — sequences of triggers
//!   (Mac Lane 1971 CWM II.7).
//! - **The runtime = a functor** ([`RuntimeSemantics`]) from the interaction
//!   category into [`ModeReachability`], assigning each trigger its effect. By
//!   the free–forgetful universal property this functor is *uniquely determined*
//!   by the per-trigger assignment — the runtime adds no policy of its own.
//! - **No-stuck = `app` is a terminal object** of [`ModeReachability`]
//!   ([`TerminalObject`]): from every mode there is exactly one way back to root,
//!   so no input can strand the user. The "stuck in a mode" bug is precisely the
//!   failure of this universal property.
//! - **Quasimode = a categorical identity** ([`QuasimodeRoundTripIsIdentity`]):
//!   entering a momentary mode and leaving it nets to `id` — Raskin's quasimode,
//!   the cure for the mode error (Norman 1981).
//!
//! Literature:
//! - Harel (1987) *Statecharts: A Visual Formalism*, Sci. Comput. Program. 8(3)
//!   — modes as a hierarchical statechart; the root is always reachable.
//! - Raskin (2000) *The Humane Interface* §3-2 — the quasimode (hold = active,
//!   release = exit), a construction with no mode-lock-in.
//! - Norman (1981) *Categorization of Action Slips*, Psychological Review 88(1)
//!   — the mode error this design prevents.
//! - Mac Lane (1971) *Categories for the Working Mathematician* II.7 (free
//!   categories), III.4 (terminal objects).

use std::collections::VecDeque;

use pr4xis::category::laws::functor_law_axioms;
use pr4xis::category::{
    Arrow, Category, Concept, FinitelyGenerated, FreeCategory, FreeExtension, FullyConnected,
    Functor, Path, Quiver, QuiverInterpretation, TerminalObject, TerminalTarget,
};
use pr4xis::logic::Axiom;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Ontology, Quality};

// ── Objects: modes ───────────────────────────────────────────────────────────

/// An interaction mode. `App` is the root (keys reach the focused application).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    App,
    Desktop,
    Move,
    Resize,
    Console,
}

impl Concept for Mode {
    fn name(&self) -> &'static str {
        match self {
            Mode::App => "app",
            Mode::Desktop => "desktop",
            Mode::Move => "move",
            Mode::Resize => "resize",
            Mode::Console => "console",
        }
    }
}
impl FinitelyGenerated for Mode {
    fn variants() -> Vec<Self> {
        vec![
            Mode::App,
            Mode::Desktop,
            Mode::Move,
            Mode::Resize,
            Mode::Console,
        ]
    }
}

// ── Generators: triggers (the quiver edges) ──────────────────────────────────

/// The kind of a trigger — the user action that fires the transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TriggerKey {
    /// Hold/click CapsLock — enter the desktop mode.
    Caps,
    /// `m` — enter/switch to the move sub-mode.
    ToMove,
    /// `r` — enter/switch to the resize sub-mode.
    ToResize,
    /// Esc / release — leave to the parent (root).
    Exit,
    /// The console key — open the console overlay.
    ToConsole,
    /// Leave the console overlay back to root.
    ConsoleExit,
}

/// A trigger: a single declared transition `from → to`, fired by `key`. These
/// are the generating arrows of the interaction quiver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Trigger {
    pub from: Mode,
    pub to: Mode,
    pub key: TriggerKey,
}

impl Arrow for Trigger {
    type Object = Mode;
    type Kind = TriggerKey;

    fn source(&self) -> Mode {
        self.from
    }

    fn target(&self) -> Mode {
        self.to
    }

    fn kind(&self) -> TriggerKey {
        self.key
    }
}

/// The interaction quiver: modes as vertices, triggers as generating edges.
///
/// `move`/`resize` exit to `app` (their parent), matching the deployed scheme.
pub struct InteractionQuiver;

impl Quiver for InteractionQuiver {
    type Vertex = Mode;
    type Edge = Trigger;

    fn edges() -> Vec<Trigger> {
        use Mode::*;
        use TriggerKey::*;
        vec![
            Trigger {
                from: App,
                to: Desktop,
                key: Caps,
            },
            Trigger {
                from: Desktop,
                to: Move,
                key: ToMove,
            },
            Trigger {
                from: Desktop,
                to: Resize,
                key: ToResize,
            },
            Trigger {
                from: Move,
                to: Resize,
                key: ToResize,
            },
            Trigger {
                from: Resize,
                to: Move,
                key: ToMove,
            },
            Trigger {
                from: Desktop,
                to: App,
                key: Exit,
            },
            Trigger {
                from: Move,
                to: App,
                key: Exit,
            },
            Trigger {
                from: Resize,
                to: App,
                key: Exit,
            },
            Trigger {
                from: App,
                to: Console,
                key: ToConsole,
            },
            Trigger {
                from: Desktop,
                to: Console,
                key: ToConsole,
            },
            Trigger {
                from: Move,
                to: Console,
                key: ToConsole,
            },
            Trigger {
                from: Resize,
                to: Console,
                key: ToConsole,
            },
            Trigger {
                from: Console,
                to: App,
                key: ConsoleExit,
            },
        ]
    }
}

/// The interaction ontology: the **free category** on the trigger quiver. Its
/// morphisms are gestures (sequences of triggers).
pub type Interaction = FreeCategory<InteractionQuiver>;

// ── The semantic target: reachability ────────────────────────────────────────

/// Modes reachable from `start` (BFS over the trigger quiver; includes `start`).
fn reachable_set(start: Mode) -> Vec<Mode> {
    let edges = InteractionQuiver::edges();
    let mut seen = vec![start];
    let mut queue = VecDeque::from([start]);
    while let Some(cur) = queue.pop_front() {
        for e in &edges {
            if e.from == cur && !seen.contains(&e.to) {
                seen.push(e.to);
                queue.push_back(e.to);
            }
        }
    }
    seen
}

/// A reachability arrow `from → to` (exists iff `to` is reachable from `from`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reach {
    pub from: Mode,
    pub to: Mode,
}

impl Arrow for Reach {
    type Object = Mode;
    type Kind = ();

    fn source(&self) -> Mode {
        self.from
    }

    fn target(&self) -> Mode {
        self.to
    }

    fn kind(&self) {}
}

/// The reachability (thin) category on modes — a preorder: one morphism per
/// reachable pair, composition by transitivity. This is the semantic target the
/// runtime functor lands in.
pub struct ModeReachability;

impl Category for ModeReachability {
    type Object = Mode;
    type Morphism = Reach;

    fn identity(o: &Mode) -> Reach {
        Reach { from: *o, to: *o }
    }

    fn compose(f: &Reach, g: &Reach) -> Option<Reach> {
        if f.to != g.from {
            return None;
        }
        // Reachability is transitive: f.from ↠ f.to = g.from ↠ g.to.
        Some(Reach {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<Reach> {
        Mode::variants()
            .into_iter()
            .flat_map(|a| {
                reachable_set(a)
                    .into_iter()
                    .map(move |b| Reach { from: a, to: b })
            })
            .collect()
    }
}

// ── The runtime: a functor (interaction → reachability) ──────────────────────

/// Interpret each trigger as its reachability arrow. By the free–forgetful
/// universal property this extends to the unique [`RuntimeSemantics`] functor.
pub struct ModalSemantics;

impl QuiverInterpretation for ModalSemantics {
    type Quiver = InteractionQuiver;
    type Target = ModeReachability;

    fn on_vertex(v: &Mode) -> Mode {
        *v
    }

    fn on_edge(e: &Trigger) -> Reach {
        Reach {
            from: e.from,
            to: e.to,
        }
    }
}

/// The runtime semantics: the unique functor mapping a gesture to its net mode
/// change. "The runtime is a functor of the ontology" — and it is *forced* by
/// the per-trigger assignment, with no policy of its own.
pub type RuntimeSemantics = FreeExtension<ModalSemantics>;

/// Marker selecting `app` as the terminal object (the root every mode returns to).
pub struct AppTerminal;

impl TerminalTarget for AppTerminal {
    type Category = ModeReachability;

    fn target() -> Mode {
        Mode::App
    }
}

// ── The operational target: effects ──────────────────────────────────────────

/// An effect atom — the runtime action a trigger fires. These mirror the engine
/// actions, so a gesture's image under [`RuntimeEffects`] is the action sequence
/// the runtime performs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Effect {
    /// Enter a (momentary or sticky) mode.
    Enter(Mode),
    /// Switch sideways to another mode, keeping the held/locked kind.
    Switch(Mode),
    /// Return to the root mode.
    ExitToRoot,
    /// Open an overlay mode (e.g. the console).
    Open(Mode),
}

/// The effect a single trigger fires.
fn effect_of(t: &Trigger) -> Effect {
    match t.key {
        TriggerKey::Caps => Effect::Enter(t.to),
        TriggerKey::ToMove | TriggerKey::ToResize => Effect::Switch(t.to),
        TriggerKey::Exit | TriggerKey::ConsoleExit => Effect::ExitToRoot,
        TriggerKey::ToConsole => Effect::Open(t.to),
    }
}

/// The single object of the effect category — the running input daemon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Runtime {
    State,
}

impl Concept for Runtime {
    fn name(&self) -> &'static str {
        "runtime"
    }
}
impl FinitelyGenerated for Runtime {
    fn variants() -> Vec<Self> {
        vec![Runtime::State]
    }
}

/// A word in the free monoid of effects — the trace a gesture produces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectWord(pub Vec<Effect>);

impl Arrow for EffectWord {
    type Object = Runtime;
    type Kind = ();

    fn source(&self) -> Runtime {
        Runtime::State
    }

    fn target(&self) -> Runtime {
        Runtime::State
    }

    fn kind(&self) {}
}

/// The effect category: one object, morphisms are effect words, composition is
/// concatenation, identity is the empty word. This is the free monoid on
/// [`Effect`] viewed as a one-object category (Mac Lane 1971 CWM I.2 — a monoid
/// is a one-object category). Used only as a functor target; its full morphism
/// set (all words) is infinite, so [`Category::morphisms`] returns the finite
/// generating set (the empty word + the single-effect words actually produced).
pub struct EffectTrace;

impl Category for EffectTrace {
    type Object = Runtime;
    type Morphism = EffectWord;

    fn identity(_: &Runtime) -> EffectWord {
        EffectWord(Vec::new())
    }

    fn compose(f: &EffectWord, g: &EffectWord) -> Option<EffectWord> {
        let mut word = f.0.clone();
        word.extend(g.0.iter().copied());
        Some(EffectWord(word))
    }

    fn morphisms() -> Vec<EffectWord> {
        let mut ms = vec![EffectWord(Vec::new())];
        ms.extend(
            InteractionQuiver::edges()
                .iter()
                .map(|e| EffectWord(vec![effect_of(e)])),
        );
        ms
    }
}

/// Interpret each trigger as the single effect it fires. By the free–forgetful
/// universal property this extends to the unique [`RuntimeEffects`] functor.
pub struct EffectSemantics;

impl QuiverInterpretation for EffectSemantics {
    type Quiver = InteractionQuiver;
    type Target = EffectTrace;

    fn on_vertex(_: &Mode) -> Runtime {
        Runtime::State
    }

    fn on_edge(e: &Trigger) -> EffectWord {
        EffectWord(vec![effect_of(e)])
    }
}

/// The operational semantics: the unique functor mapping a gesture to the
/// sequence of runtime effects it fires. This is "runtime = functor (ontology →
/// effects)" made literal; like [`RuntimeSemantics`] it is *forced* by the
/// per-trigger effect assignment, with no policy of its own.
pub type RuntimeEffects = FreeExtension<EffectSemantics>;

// ── A quality on modes ───────────────────────────────────────────────────────

/// Whether a mode is *catchall* (swallows unbound keys) vs passthrough.
#[derive(Debug, Clone)]
pub struct Catchall;

impl Quality for Catchall {
    type Individual = Mode;
    type Value = bool;

    fn get(&self, m: &Mode) -> Option<bool> {
        Some(matches!(m, Mode::Desktop | Mode::Move | Mode::Resize))
    }
}

// ── Domain axiom: the quasimode ──────────────────────────────────────────────

/// Entering a momentary mode and then leaving it nets to the identity.
///
/// `caps` (App → Desktop) followed by `exit` (Desktop → App) is a gesture whose
/// runtime semantics is `id_app`. This is Raskin's quasimode expressed
/// categorically — the structural cure for the mode error (Norman 1981): a
/// held-then-released mode cannot leave you stranded.
pub struct QuasimodeRoundTripIsIdentity;

impl Axiom for QuasimodeRoundTripIsIdentity {
    fn verify(&self) -> Verdict {
        use Mode::*;
        use TriggerKey::*;
        let enter = Path::<InteractionQuiver>::edge(Trigger {
            from: App,
            to: Desktop,
            key: Caps,
        });
        let leave = Path::<InteractionQuiver>::edge(Trigger {
            from: Desktop,
            to: App,
            key: Exit,
        });
        let Some(gesture) = Interaction::compose(&enter, &leave) else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        if RuntimeSemantics::map_morphism(&gesture) == ModeReachability::identity(&App) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "QuasimodeRoundTripIsIdentity",
        "entering a momentary mode then leaving it nets to the identity — a quasimode reverts, so there is no mode lock-in",
        "Raskin (2000) The Humane Interface §3-2; Norman (1981) Categorization of Action Slips, Psychological Review 88(1) pp. 1-15"
    );
}

// ── The ontology ─────────────────────────────────────────────────────────────

/// The modal interaction ontology. Validating it discharges: the reachability
/// category laws, the runtime functor laws (which certify the free interaction
/// category through its unique extension), no-stuck (`app` terminal), full
/// connectivity, and the quasimode identity.
pub struct ModalInteractionOntology;

impl Ontology for ModalInteractionOntology {
    type Cat = ModeReachability;
    type Qual = Catchall;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut all: Vec<Box<dyn Axiom>> = functor_law_axioms::<RuntimeSemantics>();
        all.extend(functor_law_axioms::<RuntimeEffects>());
        all.push(Box::new(
            TerminalObject::<ModeReachability, AppTerminal>::new(),
        ));
        all.push(Box::new(FullyConnected::<ModeReachability>::new()));
        all.push(Box::new(QuasimodeRoundTripIsIdentity));
        all
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::{assert_category_laws, assert_functor_laws};
    use proptest::prelude::*;

    #[test]
    fn reachability_is_a_category() {
        assert_category_laws::<ModeReachability>();
    }

    #[test]
    fn runtime_semantics_is_a_functor() {
        // The runtime functor satisfies the functor laws on the generating set —
        // sound by the free–forgetful universal property.
        assert_functor_laws::<RuntimeSemantics>();
    }

    #[test]
    fn runtime_effects_is_a_functor() {
        // The operational (effects) functor — runtime = functor (ontology → effects).
        assert_functor_laws::<RuntimeEffects>();
    }

    #[test]
    fn quasimode_effect_trace() {
        use Mode::*;
        use TriggerKey::*;
        let enter = Path::<InteractionQuiver>::edge(Trigger {
            from: App,
            to: Desktop,
            key: Caps,
        });
        let leave = Path::<InteractionQuiver>::edge(Trigger {
            from: Desktop,
            to: App,
            key: Exit,
        });
        let gesture = Interaction::compose(&enter, &leave).unwrap();
        assert_eq!(
            RuntimeEffects::map_morphism(&gesture),
            EffectWord(vec![Effect::Enter(Desktop), Effect::ExitToRoot])
        );
    }

    #[test]
    fn app_is_terminal_no_stuck() {
        TerminalObject::<ModeReachability, AppTerminal>::new()
            .verify()
            .unwrap_or_else(|c| panic!("no-stuck failed: {}", c.meta().name.as_str()));
    }

    #[test]
    fn quasimode_round_trip_is_identity() {
        QuasimodeRoundTripIsIdentity
            .verify()
            .unwrap_or_else(|c| panic!("quasimode failed: {}", c.meta().name.as_str()));
    }

    #[test]
    fn whole_ontology_validates() {
        ModalInteractionOntology::validate()
            .unwrap_or_else(|c| panic!("ontology invalid: {}", c.meta().name.as_str()));
    }

    #[test]
    fn app_reachable_from_every_mode() {
        for m in Mode::variants() {
            assert!(
                reachable_set(m).contains(&Mode::App),
                "{m:?} cannot reach app"
            );
        }
    }

    proptest! {
        /// The net semantics of ANY gesture (a random walk of triggers from app)
        /// is the reachability arrow from start to end — the runtime functor is
        /// determined by endpoints, the hallmark of the free construction.
        #[test]
        fn prop_gesture_net_effect_is_endpoint_reachability(
            steps in proptest::collection::vec(any::<u8>(), 0..40)
        ) {
            let edges = InteractionQuiver::edges();
            let mut path = Interaction::identity(&Mode::App);
            for s in steps {
                let cur = path.target();
                let outs: Vec<Trigger> = edges.iter().copied().filter(|e| e.from == cur).collect();
                if outs.is_empty() {
                    break;
                }
                let step = Path::<InteractionQuiver>::edge(outs[(s as usize) % outs.len()]);
                path = Interaction::compose(&path, &step).expect("random walk stays composable");
            }
            let net = RuntimeSemantics::map_morphism(&path);
            prop_assert_eq!(net, Reach { from: Mode::App, to: path.target() });
        }

        /// A gesture's effect trace has exactly one effect per trigger — the
        /// operational functor is a homomorphism from gestures to effect words.
        #[test]
        fn prop_effect_trace_length_matches_gesture_length(
            steps in proptest::collection::vec(any::<u8>(), 0..40)
        ) {
            let edges = InteractionQuiver::edges();
            let mut path = Interaction::identity(&Mode::App);
            for s in steps {
                let cur = path.target();
                let outs: Vec<Trigger> = edges.iter().copied().filter(|e| e.from == cur).collect();
                if outs.is_empty() {
                    break;
                }
                let step = Path::<InteractionQuiver>::edge(outs[(s as usize) % outs.len()]);
                path = Interaction::compose(&path, &step).expect("random walk stays composable");
            }
            let trace = RuntimeEffects::map_morphism(&path);
            prop_assert_eq!(trace.0.len(), path.len());
        }
    }
}
