//! Window-manager action ontology — abstract intents, typed parameters, and the
//! functor that realizes them as concrete compositor dispatchers.
//!
//! # The conflation this dissolves
//!
//! A keybinding used to carry a single opaque string — a raw Hyprland dispatcher
//! like `"fullscreen, 1"` or `"movetoworkspacesilent, special:hidden"`. That one
//! string conflates three things an ontology must keep apart:
//!
//! 1. **Intent** — the user's goal ([`WmAction`]): *focus the window to the
//!    left*, *make this window fullscreen*.
//! 2. **Parameters** — the typed arguments ([`Direction`], [`WorkspaceTarget`],
//!    [`Follow`], [`Cycle`], …): *which direction, which workspace*.
//! 3. **Mechanism** — the concrete realization ([`Dispatch`]): the compositor
//!    dispatcher string. This is produced by a *projection*, never authored.
//!
//! The projection is a **functor** [`HyprlandRealization`] from the free monoid
//! of abstract actions ([`ActionAlgebra`]) to the free monoid of dispatcher
//! sequences ([`DispatchAlgebra`]). "The runtime is a functor of the ontology":
//! the realization is *forced* by the per-action lowering rule, with no policy
//! of its own, and the functor laws (identity + composition preservation) are
//! machine-proven. A single dispatcher string is emitted only at one blessed
//! wire boundary, [`Dispatch::render`].
//!
//! # Grounding (the literature spine)
//!
//! - **Myers (1988)** *A Taxonomy of Window Manager User Interfaces*, IEEE CG&A
//!   8(5):65–84 — the intent verb set and, crucially, the paper's own split of
//!   operation **functionality** (which operations exist) from operation **user
//!   interface** (how they are invoked): intent-vs-mechanism, in WM literature.
//! - **Card, Moran & Newell (1983)** *The Psychology of Human-Computer
//!   Interaction* (GOMS) — Goal / Operator / **Method** / Selection-rule. A Goal
//!   is independent of the Method that realizes it; the Method is the projection.
//! - **Goguen, Thatcher & Wagner (1978)** *An Initial Algebra Approach to … ADTs*
//!   — a many-sorted signature's terms form the initial algebra, so there is a
//!   **unique homomorphism** into any other algebra of the signature. Model the
//!   actions as the signature and the dispatchers as a second algebra: the
//!   projection IS that forced homomorphism.
//! - **Payne & Green (1986)** *Task-Action Grammars* — features as **strongly
//!   typed variables** over small closed value-sets; one rule-schema per intent
//!   family, not a per-action string table.
//! - **EWMH v1.5 (2013)** freedesktop.org — the cross-implementation controlled
//!   vocabulary: `_NET_WM_ACTION_*`/`_NET_WM_STATE_*`, the `_NET_WM_MOVERESIZE`
//!   direction enum, add/remove/**toggle**, and the desktop CARDINAL. Supplies
//!   the named atoms and typed parameter domains (and the *maximize ≠ fullscreen*
//!   state distinction this module preserves).
//! - **Foley & van Dam (1982)** *Fundamentals of Interactive Computer Graphics*
//!   — the conceptual/semantic/syntactic/lexical stack: mechanism emission is
//!   lexical lowering (code generation), licensing the functor.
//! - **Mac Lane (1971)** *Categories for the Working Mathematician* II.1 — the
//!   functor laws the projection must satisfy.
//! - **Reiter (1978)** *On Closed World Data Bases* — [`WmAction`] is **open
//!   world**: [`WmAction::Exec`] carries an arbitrary external command line, so
//!   the vocabulary is not finitely enumerable and the type implements
//!   [`Concept`] but **not** [`FinitelyGenerated`].
//! - **Hyprland dispatchers** <https://wiki.hyprland.org/Configuring/Dispatchers/>
//!   — the realization vocabulary [`Dispatch`] names.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::laws::functor_law_axioms;
use pr4xis::category::{Arrow, Category, Concept, FinitelyGenerated, Functor};
use pr4xis::logic::Axiom;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict, combine_verdicts};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

use super::modes::ModeId;
use super::window_state::{StateBit, StateDelta, StateOp};

// ── Layer 2: typed parameters ────────────────────────────────────────────────

/// A spatial direction — the parameter of focus / move / swap / resize.
///
/// A strongly-typed variable over a small closed value-set (Payne & Green 1986);
/// the value domain is EWMH's `_NET_WM_MOVERESIZE` direction family restricted to
/// the four cardinals the presets use (diagonals are deferred — see the pack
/// README). Hyprland names them `l`/`r`/`u`/`d`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    /// The Hyprland direction token (`l`/`r`/`u`/`d`).
    fn hypr(self) -> char {
        match self {
            Direction::Left => 'l',
            Direction::Right => 'r',
            Direction::Up => 'u',
            Direction::Down => 'd',
        }
    }
}

impl Concept for Direction {
    fn name(&self) -> &'static str {
        match self {
            Direction::Left => "left",
            Direction::Right => "right",
            Direction::Up => "up",
            Direction::Down => "down",
        }
    }
}
impl FinitelyGenerated for Direction {
    fn variants() -> Vec<Self> {
        vec![
            Direction::Left,
            Direction::Right,
            Direction::Up,
            Direction::Down,
        ]
    }
}

/// Cycle direction for window / group traversal (Alt-Tab style).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cycle {
    Forward,
    Backward,
}

impl Concept for Cycle {
    fn name(&self) -> &'static str {
        match self {
            Cycle::Forward => "forward",
            Cycle::Backward => "backward",
        }
    }
}
impl FinitelyGenerated for Cycle {
    fn variants() -> Vec<Self> {
        vec![Cycle::Forward, Cycle::Backward]
    }
}

/// The orientation of a tiling split — the parameter of the split operation
/// (i3/sway `split horizontal|vertical|toggle`).
///
/// A strongly-typed variable over i3's split value-set (Payne & Green 1986).
/// `Toggle` flips the current orientation — the third value, not a separate verb.
/// Hyprland exposes only the toggle, so the absolute orientations collapse to it
/// there; a directed backend (Sway `split horizontal`) realizes them distinctly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Orientation {
    Horizontal,
    Vertical,
    Toggle,
}

impl Concept for Orientation {
    fn name(&self) -> &'static str {
        match self {
            Orientation::Horizontal => "horizontal",
            Orientation::Vertical => "vertical",
            Orientation::Toggle => "toggle",
        }
    }
}
impl FinitelyGenerated for Orientation {
    fn variants() -> Vec<Self> {
        vec![
            Orientation::Horizontal,
            Orientation::Vertical,
            Orientation::Toggle,
        ]
    }
}

/// Whether moving a window to a workspace **follows** it (focus travels) or is
/// **silent** (window leaves, focus stays).
///
/// This is the `movetoworkspace` vs `movetoworkspacesilent` distinction; it is a
/// real parameter of the move intent (EWMH source-indication governs whether a
/// request changes the active desktop), not two separate verbs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Follow {
    Follow,
    Silent,
}

impl Concept for Follow {
    fn name(&self) -> &'static str {
        match self {
            Follow::Follow => "follow",
            Follow::Silent => "silent",
        }
    }
}
impl FinitelyGenerated for Follow {
    fn variants() -> Vec<Self> {
        vec![Follow::Follow, Follow::Silent]
    }
}

/// A workspace selector — the parameter of workspace and move-to-workspace.
///
/// The value domain is EWMH `_NET_WM_DESKTOP` (a bounded CARDINAL index) plus the
/// relative and named extensions every real compositor adds. The virtual-desktop
/// dimension itself is Henderson & Card (1986) *Rooms*.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WorkspaceTarget {
    /// Absolute index (`workspace, 3`).
    Index(u8),
    /// Relative step (`workspace, +1` / `workspace, -1`).
    Relative(i32),
    /// Named workspace (`workspace, Music`).
    Named(String),
    /// A special / scratchpad workspace. Empty name = the default `special`
    /// scratchpad; a name = `special:hidden`, `special:minimized`, …
    Special(String),
}

impl WorkspaceTarget {
    /// The Hyprland workspace argument.
    fn render(&self) -> String {
        match self {
            WorkspaceTarget::Index(n) => n.to_string(),
            WorkspaceTarget::Relative(k) => {
                if *k >= 0 {
                    format!("+{k}")
                } else {
                    k.to_string()
                }
            }
            WorkspaceTarget::Named(s) => s.clone(),
            WorkspaceTarget::Special(s) if s.is_empty() => "special".to_string(),
            WorkspaceTarget::Special(s) => format!("special:{s}"),
        }
    }
}

/// What a submap keybinding does — enter a named submap, or reset to root.
///
/// Submaps are the keybinding-layer realization of the modal interaction
/// ontology; a submap *is* a [`ModeId`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SubmapTarget {
    Enter(ModeId),
    Reset,
}

impl SubmapTarget {
    fn render(&self) -> String {
        match self {
            SubmapTarget::Enter(m) => m.0.clone(),
            SubmapTarget::Reset => "reset".to_string(),
        }
    }
}

// ── Layer 1: the abstract intent vocabulary ──────────────────────────────────

/// An abstract window-manager **action** — the user's intent, with typed
/// parameters, independent of the compositor that realizes it.
///
/// This is the open-world ([`Concept`], not [`FinitelyGenerated`]) vocabulary:
/// [`WmAction::Exec`] carries an arbitrary external command, so the set of
/// actions is not finitely enumerable (Reiter 1978).
/// [`WmAction::representative_actions`] supplies the finite generating set used to
/// machine-check the functor and to seed property tests.
///
/// Each variant cites the source that names it; see the module header for the
/// full spine.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WmAction {
    /// Move keyboard focus to the neighbour in a direction (Myers
    /// "change-listener"; EWMH input focus).
    Focus(Direction),
    /// Move the focused window one slot in a direction (Myers "move").
    MoveWindow(Direction),
    /// Swap the focused window with its neighbour in a direction.
    SwapWindow(Direction),
    /// Resize the focused window by a step along a direction's axis (Myers
    /// "change-size"/grow; the amount is a pixel magnitude).
    Resize(Direction, u16),
    /// Close / kill the focused window (Myers "delete"; EWMH `CLOSE`).
    Close,
    /// Mutate one window-state atom — the EWMH `_NET_WM_STATE` add/remove/toggle
    /// on a [`StateBit`] (see [`window_state`](super::window_state)). This single
    /// constructor subsumes the former flat fullscreen / maximize / minimize /
    /// float / pin / pseudotile verbs (the convenience constructors
    /// [`WmAction::fullscreen`] … preserve their names) **and** makes the inverse
    /// direction — restore / unmaximize — expressible as `State { Remove, … }`,
    /// which a flat verb set could not name.
    State(StateDelta),
    /// Set or toggle the split orientation of the tiling layout — i3/sway
    /// `split h|v|toggle`. Hyprland exposes only the toggle (`layoutmsg
    /// togglesplit`), so all orientations collapse to it there; a backend with
    /// absolute split (Sway `split horizontal`) realizes them distinctly.
    Split(Orientation),
    /// Toggle grouping (tabbed group) for the focused window.
    ToggleGroup,
    /// Cycle the active window *within* the current group.
    CycleGroup(Cycle),
    /// Cycle focus across windows (Alt-Tab; EWMH stacking order traversal).
    CycleWindow(Cycle),
    /// Switch to a workspace (Henderson & Card 1986 *Rooms*; EWMH
    /// `_NET_CURRENT_DESKTOP`).
    Workspace(WorkspaceTarget),
    /// Send the focused window to a workspace, optionally following it (EWMH
    /// `_NET_WM_DESKTOP`).
    MoveToWorkspace(WorkspaceTarget, Follow),
    /// Toggle a special / scratchpad workspace overlay (e.g. an overview).
    ToggleSpecialWorkspace(String),
    /// Run an external command. The **one blessed wire boundary**: the command
    /// line is genuine external data, not a vocabulary praxis controls — the
    /// reason [`WmAction`] is open-world (Reiter 1978).
    Exec(String),
    /// Enter or reset a keybinding submap (the modal-ontology mode layer).
    Submap(SubmapTarget),
}

impl WmAction {
    /// Toggle true fullscreen — `State{Toggle, Fullscreen}` (EWMH
    /// `_NET_WM_STATE_FULLSCREEN`). Convenience for the common window-state verb.
    pub fn fullscreen() -> Self {
        WmAction::State(StateDelta::new(StateOp::Toggle, StateBit::Fullscreen))
    }

    /// Maximize — fill the work area keeping reserved space. The canonical
    /// maximize toggles the (representative) `MaximizedVert` bit; the Hyprland
    /// realization lowers both `MaximizedVert` and `MaximizedHorz` to the single
    /// `fullscreen, 1` dispatcher (Hyprland has no per-axis maximize). A backend
    /// with independent axes realizes them distinctly, and "maximize both axes"
    /// is the `[Vert, Horz]` composite there.
    pub fn maximize() -> Self {
        WmAction::State(StateDelta::new(StateOp::Toggle, StateBit::MaximizedVert))
    }

    /// Minimize / iconify — `State{Add, Hidden}` (EWMH `_NET_WM_STATE_HIDDEN`,
    /// the canonical minimize). Hyprland has no native iconify, so the functor
    /// emulates it with a silent move to a dedicated special workspace.
    pub fn minimize() -> Self {
        WmAction::State(StateDelta::new(StateOp::Add, StateBit::Hidden))
    }

    /// Toggle floating — `State{Toggle, Floating}` (the tiled ⇄ floating layer).
    pub fn toggle_float() -> Self {
        WmAction::State(StateDelta::new(StateOp::Toggle, StateBit::Floating))
    }

    /// Pin above / across workspaces — `State{Toggle, Above}` (EWMH `ABOVE`;
    /// Hyprland's `pin` realizes above+sticky together, so `Sticky` lowers
    /// identically below the functor).
    pub fn pin() -> Self {
        WmAction::State(StateDelta::new(StateOp::Toggle, StateBit::Above))
    }

    /// Toggle pseudo-tiling — `State{Toggle, PseudoTiled}` (bspwm's node state;
    /// Hyprland dwindle's `pseudo`).
    pub fn pseudotile() -> Self {
        WmAction::State(StateDelta::new(StateOp::Toggle, StateBit::PseudoTiled))
    }

    /// Toggle the tiling split orientation — `Split(Orientation::Toggle)` (i3
    /// `split toggle`; Hyprland `layoutmsg togglesplit`).
    pub fn toggle_split() -> Self {
        WmAction::Split(Orientation::Toggle)
    }

    /// The finite generating set — one representative of every variant. Used to
    /// machine-check the realization functor (totality, the functor laws) and to
    /// seed property tests. This is a *generating* set, not the (open-world) whole
    /// of [`WmAction`].
    pub fn representative_actions() -> Vec<WmAction> {
        use Direction::*;
        vec![
            WmAction::Focus(Left),
            WmAction::Focus(Right),
            WmAction::Focus(Up),
            WmAction::Focus(Down),
            WmAction::MoveWindow(Left),
            WmAction::SwapWindow(Up),
            WmAction::Resize(Left, 30),
            WmAction::Resize(Down, 30),
            WmAction::Close,
            WmAction::fullscreen(),
            WmAction::maximize(),
            // The inverse direction a flat verb set could not name — restore.
            WmAction::State(StateDelta::new(StateOp::Remove, StateBit::MaximizedVert)),
            WmAction::minimize(),
            WmAction::toggle_float(),
            WmAction::pin(),
            WmAction::pseudotile(),
            WmAction::Split(Orientation::Toggle),
            WmAction::Split(Orientation::Horizontal),
            WmAction::Split(Orientation::Vertical),
            WmAction::ToggleGroup,
            WmAction::CycleGroup(Cycle::Forward),
            WmAction::CycleWindow(Cycle::Forward),
            WmAction::CycleWindow(Cycle::Backward),
            WmAction::Workspace(WorkspaceTarget::Index(1)),
            WmAction::Workspace(WorkspaceTarget::Relative(1)),
            WmAction::Workspace(WorkspaceTarget::Relative(-1)),
            WmAction::Workspace(WorkspaceTarget::Named("Music".to_string())),
            WmAction::MoveToWorkspace(WorkspaceTarget::Index(2), Follow::Follow),
            WmAction::MoveToWorkspace(WorkspaceTarget::Relative(-1), Follow::Follow),
            WmAction::MoveToWorkspace(WorkspaceTarget::Index(3), Follow::Silent),
            WmAction::MoveToWorkspace(
                WorkspaceTarget::Special("hidden".to_string()),
                Follow::Silent,
            ),
            WmAction::MoveToWorkspace(WorkspaceTarget::Special(String::new()), Follow::Silent),
            WmAction::ToggleSpecialWorkspace("overview".to_string()),
            WmAction::Exec("$TERMINAL".to_string()),
            WmAction::Submap(SubmapTarget::Enter(ModeId::new("resize"))),
            WmAction::Submap(SubmapTarget::Reset),
        ]
    }
}

impl Concept for WmAction {
    fn name(&self) -> &'static str {
        match self {
            WmAction::Focus(_) => "focus",
            WmAction::MoveWindow(_) => "move-window",
            WmAction::SwapWindow(_) => "swap-window",
            WmAction::Resize(_, _) => "resize",
            WmAction::Close => "close",
            WmAction::State(_) => "state",
            WmAction::Split(_) => "split",
            WmAction::ToggleGroup => "toggle-group",
            WmAction::CycleGroup(_) => "cycle-group",
            WmAction::CycleWindow(_) => "cycle-window",
            WmAction::Workspace(_) => "workspace",
            WmAction::MoveToWorkspace(_, _) => "move-to-workspace",
            WmAction::ToggleSpecialWorkspace(_) => "toggle-special-workspace",
            WmAction::Exec(_) => "exec",
            WmAction::Submap(_) => "submap",
        }
    }
}

// ── Layer 3: the realization alphabet (Hyprland dispatchers) ──────────────────

/// A single Hyprland dispatcher — the typed mechanism atom. The dispatcher name
/// is vocabulary (a typed variant); its arguments are typed where they are
/// vocabulary and carried verbatim only where they are genuine external data
/// ([`Dispatch::Exec`]). [`Dispatch::render`] is the single wire boundary that
/// turns this into the compositor command string.
///
/// Names follow the Hyprland dispatcher list
/// (<https://wiki.hyprland.org/Configuring/Dispatchers/>).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Dispatch {
    MoveFocus(Direction),
    MoveWindow(Direction),
    SwapWindow(Direction),
    /// `resizeactive, <dx> <dy>`.
    ResizeActive(i16, i16),
    KillActive,
    /// `fullscreen, <0|1>`.
    Fullscreen(u8),
    ToggleFloating,
    Pin,
    Pseudo,
    /// `layoutmsg, <message>`.
    LayoutMsg(&'static str),
    ToggleGroup,
    /// `changegroupactive, <f|b>`.
    ChangeGroupActive(char),
    Workspace(WorkspaceTarget),
    MoveToWorkspace(WorkspaceTarget),
    MoveToWorkspaceSilent(WorkspaceTarget),
    ToggleSpecialWorkspace(String),
    /// `cyclenext,` or `cyclenext, prev`.
    CycleNext(bool),
    Exec(String),
    Submap(String),
}

impl Dispatch {
    /// The single wire boundary: render this dispatcher as the Hyprland command
    /// string (the pre-`dispatch` form — `"<dispatcher>, <args>"`).
    pub fn render(&self) -> String {
        match self {
            Dispatch::MoveFocus(d) => format!("movefocus, {}", d.hypr()),
            Dispatch::MoveWindow(d) => format!("movewindow, {}", d.hypr()),
            Dispatch::SwapWindow(d) => format!("swapwindow, {}", d.hypr()),
            Dispatch::ResizeActive(dx, dy) => format!("resizeactive, {dx} {dy}"),
            Dispatch::KillActive => "killactive,".to_string(),
            // Default fullscreen is the bare `fullscreen` dispatcher (the Hyprland
            // idiom and what deployed configs carry); an explicit non-zero mode
            // (1 = maximize, keep the reserved area) carries its argument.
            Dispatch::Fullscreen(0) => "fullscreen".to_string(),
            Dispatch::Fullscreen(n) => format!("fullscreen, {n}"),
            Dispatch::ToggleFloating => "togglefloating,".to_string(),
            Dispatch::Pin => "pin,".to_string(),
            Dispatch::Pseudo => "pseudo,".to_string(),
            Dispatch::LayoutMsg(m) => format!("layoutmsg, {m}"),
            Dispatch::ToggleGroup => "togglegroup,".to_string(),
            Dispatch::ChangeGroupActive(c) => format!("changegroupactive, {c}"),
            Dispatch::Workspace(w) => format!("workspace, {}", w.render()),
            Dispatch::MoveToWorkspace(w) => format!("movetoworkspace, {}", w.render()),
            Dispatch::MoveToWorkspaceSilent(w) => {
                format!("movetoworkspacesilent, {}", w.render())
            }
            Dispatch::ToggleSpecialWorkspace(n) => format!("togglespecialworkspace, {n}"),
            Dispatch::CycleNext(prev) => {
                if *prev {
                    "cyclenext, prev".to_string()
                } else {
                    "cyclenext,".to_string()
                }
            }
            Dispatch::Exec(cmd) => format!("exec, {cmd}"),
            Dispatch::Submap(name) => format!("submap, {name}"),
        }
    }

    /// The `hyprctl dispatch …` argument form (dispatcher + space-separated args,
    /// no comma). Used to batch a multi-dispatcher action into one `exec` bind.
    fn hyprctl_args(&self) -> String {
        let r = self.render();
        match r.split_once(", ") {
            Some((head, tail)) => format!("{head} {tail}"),
            None => r.trim_end_matches(',').to_string(),
        }
    }
}

// ── The two monoids: one-object free categories ───────────────────────────────

/// The single object of both action and dispatch monoids — the compositor
/// surface that actions act on. (A monoid is a one-object category; Mac Lane
/// 1971 I.2.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WmSurface {
    Compositor,
}

impl Concept for WmSurface {
    fn name(&self) -> &'static str {
        "compositor"
    }
}
impl FinitelyGenerated for WmSurface {
    fn variants() -> Vec<Self> {
        vec![WmSurface::Compositor]
    }
}

/// A word in the free monoid of abstract actions — the action(s) a single
/// binding performs. Length 1 for almost every binding; length > 1 for genuine
/// composites (e.g. float-then-pin).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActionWord(pub Vec<WmAction>);

impl From<WmAction> for ActionWord {
    fn from(a: WmAction) -> Self {
        ActionWord(vec![a])
    }
}
impl From<Vec<WmAction>> for ActionWord {
    fn from(v: Vec<WmAction>) -> Self {
        ActionWord(v)
    }
}

impl Arrow for ActionWord {
    type Object = WmSurface;
    type Kind = ();

    fn source(&self) -> WmSurface {
        WmSurface::Compositor
    }
    fn target(&self) -> WmSurface {
        WmSurface::Compositor
    }
    fn kind(&self) {}
}

/// The free monoid on [`WmAction`] as a one-object category: composition is
/// concatenation, identity is the empty word. Used as the functor source; its
/// full morphism set is infinite, so [`Category::morphisms`] returns the finite
/// generating set (empty word + single-action words), as the sibling modal
/// ontology's effect trace does.
pub struct ActionAlgebra;

impl Category for ActionAlgebra {
    type Object = WmSurface;
    type Morphism = ActionWord;

    fn identity(_: &WmSurface) -> ActionWord {
        ActionWord(Vec::new())
    }

    fn compose(f: &ActionWord, g: &ActionWord) -> Option<ActionWord> {
        let mut w = f.0.clone();
        w.extend(g.0.iter().cloned());
        Some(ActionWord(w))
    }

    fn morphisms() -> Vec<ActionWord> {
        let mut ms = vec![ActionWord(Vec::new())];
        ms.extend(
            WmAction::representative_actions()
                .into_iter()
                .map(ActionWord::from),
        );
        ms
    }
}

/// A word in the free monoid of dispatchers — the concrete realization of an
/// action word.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DispatchWord(pub Vec<Dispatch>);

impl DispatchWord {
    /// Render this dispatcher sequence as the command string a *single* keybind
    /// emits. A one-dispatcher word renders directly; a multi-dispatcher word is
    /// batched into one `exec, hyprctl dispatch … ; hyprctl dispatch …` —
    /// Hyprland binds one dispatcher each, so a composite is realized this way.
    pub fn command(&self) -> String {
        match self.0.as_slice() {
            [] => String::new(),
            [single] => single.render(),
            many => {
                let parts: Vec<String> = many
                    .iter()
                    .map(|d| format!("hyprctl dispatch {}", d.hyprctl_args()))
                    .collect();
                format!("exec, {}", parts.join(" ; "))
            }
        }
    }
}

impl Arrow for DispatchWord {
    type Object = WmSurface;
    type Kind = ();

    fn source(&self) -> WmSurface {
        WmSurface::Compositor
    }
    fn target(&self) -> WmSurface {
        WmSurface::Compositor
    }
    fn kind(&self) {}
}

/// The free monoid on [`Dispatch`] as a one-object category — the functor target.
pub struct DispatchAlgebra;

impl Category for DispatchAlgebra {
    type Object = WmSurface;
    type Morphism = DispatchWord;

    fn identity(_: &WmSurface) -> DispatchWord {
        DispatchWord(Vec::new())
    }

    fn compose(f: &DispatchWord, g: &DispatchWord) -> Option<DispatchWord> {
        let mut w = f.0.clone();
        w.extend(g.0.iter().cloned());
        Some(DispatchWord(w))
    }

    fn morphisms() -> Vec<DispatchWord> {
        let mut ms = vec![DispatchWord(Vec::new())];
        ms.extend(
            WmAction::representative_actions()
                .iter()
                .map(|a| DispatchWord(lower(a))),
        );
        ms
    }
}

// ── The realization functor ───────────────────────────────────────────────────

/// Lower one abstract action to its dispatcher sequence — the per-action rule
/// schema (Payne & Green 1986). One shared schema per intent family; the functor
/// is its free extension. Almost every action lowers to a single dispatcher; the
/// genuine composites lower to a short sequence.
fn lower(action: &WmAction) -> Vec<Dispatch> {
    match action {
        WmAction::Focus(d) => vec![Dispatch::MoveFocus(*d)],
        WmAction::MoveWindow(d) => vec![Dispatch::MoveWindow(*d)],
        WmAction::SwapWindow(d) => vec![Dispatch::SwapWindow(*d)],
        WmAction::Resize(d, amt) => {
            let a = *amt as i16;
            let (dx, dy) = match d {
                Direction::Left => (-a, 0),
                Direction::Right => (a, 0),
                Direction::Up => (0, -a),
                Direction::Down => (0, a),
            };
            vec![Dispatch::ResizeActive(dx, dy)]
        }
        WmAction::Close => vec![Dispatch::KillActive],
        WmAction::State(d) => lower_state(d),
        // Hyprland exposes only the split toggle; all orientations collapse to it
        // here (Sway realizes split horizontal/vertical distinctly — see lower_sway).
        WmAction::Split(_) => vec![Dispatch::LayoutMsg("togglesplit")],
        WmAction::ToggleGroup => vec![Dispatch::ToggleGroup],
        WmAction::CycleGroup(c) => vec![Dispatch::ChangeGroupActive(match c {
            Cycle::Forward => 'f',
            Cycle::Backward => 'b',
        })],
        WmAction::CycleWindow(c) => vec![Dispatch::CycleNext(matches!(c, Cycle::Backward))],
        WmAction::Workspace(w) => vec![Dispatch::Workspace(w.clone())],
        WmAction::MoveToWorkspace(w, Follow::Follow) => vec![Dispatch::MoveToWorkspace(w.clone())],
        WmAction::MoveToWorkspace(w, Follow::Silent) => {
            vec![Dispatch::MoveToWorkspaceSilent(w.clone())]
        }
        WmAction::ToggleSpecialWorkspace(n) => vec![Dispatch::ToggleSpecialWorkspace(n.clone())],
        WmAction::Exec(cmd) => vec![Dispatch::Exec(cmd.clone())],
        WmAction::Submap(t) => vec![Dispatch::Submap(t.render())],
    }
}

/// Lower a window-state mutation to its Hyprland dispatcher(s).
///
/// Hyprland exposes only TOGGLE dispatchers for window states, so the EWMH
/// add/remove/toggle distinction ([`StateOp`]) is not observable in the Hyprland
/// realization (a directed backend such as X11 honours it via `_NET_WM_STATE`
/// client messages). The states Hyprland gives a user dispatcher are its
/// window-state CAPABILITY ([`hyprland_state_capability`]); EWMH states outside
/// it (app-set hints, the below layer, shading) have no Hyprland user action and
/// lower to the empty word — they are never *generated* for a Hyprland binding
/// (`representative_actions` excludes them; [`StateLoweringMatchesCapability`]
/// witnesses the boundary, so the empty arm is intentional, not a silent drop).
fn lower_state(d: &StateDelta) -> Vec<Dispatch> {
    match d.bit {
        StateBit::Fullscreen => vec![Dispatch::Fullscreen(0)],
        StateBit::MaximizedVert | StateBit::MaximizedHorz => vec![Dispatch::Fullscreen(1)],
        StateBit::Hidden => vec![Dispatch::MoveToWorkspaceSilent(WorkspaceTarget::Special(
            "minimized".to_string(),
        ))],
        StateBit::Floating => vec![Dispatch::ToggleFloating],
        StateBit::Above | StateBit::Sticky => vec![Dispatch::Pin],
        StateBit::PseudoTiled => vec![Dispatch::Pseudo],
        StateBit::Shaded
        | StateBit::Below
        | StateBit::SkipTaskbar
        | StateBit::SkipPager
        | StateBit::Modal
        | StateBit::DemandsAttention
        | StateBit::Focused => Vec::new(),
    }
}

/// The window states Hyprland exposes a user dispatcher for — its window-state
/// CAPABILITY. The EWMH atoms outside this set have no Hyprland user action and
/// are excluded from Hyprland's generating set.
fn hyprland_state_capability() -> [StateBit; 8] {
    [
        StateBit::Fullscreen,
        StateBit::MaximizedVert,
        StateBit::MaximizedHorz,
        StateBit::Hidden,
        StateBit::Floating,
        StateBit::Above,
        StateBit::Sticky,
        StateBit::PseudoTiled,
    ]
}

/// The realization functor `HyprlandRealization : ActionAlgebra → DispatchAlgebra`.
///
/// On the single object it is the identity; on morphisms it lowers each action
/// and concatenates. Because lowering distributes over concatenation, the result
/// is a monoid homomorphism — a strict, total functor (Goguen-Thatcher-Wagner
/// 1978's unique homomorphism; Mac Lane 1971 II.1's functor laws), with no policy
/// of its own. "The runtime is a functor of the ontology."
pub struct HyprlandRealization;

impl Functor for HyprlandRealization {
    type Source = ActionAlgebra;
    type Target = DispatchAlgebra;

    fn map_object(_: &WmSurface) -> WmSurface {
        WmSurface::Compositor
    }

    fn map_morphism(word: &ActionWord) -> DispatchWord {
        DispatchWord(word.0.iter().flat_map(lower).collect())
    }

    fn meta() -> Provenance {
        Provenance {
            name: OntologyName::new_static("HyprlandRealization"),
            description: Label::new_static(
                "abstract WM actions → Hyprland dispatcher sequences (strict, total monoid functor)",
            ),
            citation: Citation::parse_static(
                "Goguen, Thatcher & Wagner (1978) An Initial Algebra Approach to ADTs (unique homomorphism); \
                 Foley & van Dam (1982) Fundamentals of Interactive Computer Graphics (lexical lowering); \
                 Mac Lane (1971) Categories for the Working Mathematician Ch. II §1 (functor laws)",
            ),
            module_path: ModulePath::new_static(module_path!()),
        }
    }
}

/// Convenience: lower a single action to the command string a keybind emits.
pub fn realize(action: &WmAction) -> String {
    HyprlandRealization::map_morphism(&ActionWord::from(action.clone())).command()
}

// ── Domain axioms ─────────────────────────────────────────────────────────────

/// Every abstract action realizes to at least one dispatcher — the projection is
/// **total**: no intent silently vanishes.
pub struct LoweringTotal;

impl Axiom for LoweringTotal {
    fn verify(&self) -> Verdict {
        for a in WmAction::representative_actions() {
            let w = HyprlandRealization::map_morphism(&ActionWord::from(a));
            if w.0.is_empty() {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "LoweringTotal",
        "every abstract action realizes to a non-empty dispatcher sequence (total projection)",
        "Goguen, Thatcher & Wagner (1978) An Initial Algebra Approach to ADTs — the initial algebra has a unique (total) homomorphism into any algebra of the signature; Card, Moran & Newell (1983) GOMS — every goal has a method"
    );
}

/// Fullscreen, Maximize, and Minimize are **distinct** window-state actions and
/// realize to pairwise-distinct dispatchers. EWMH keeps them as three separate
/// actions (`_NET_WM_ACTION_FULLSCREEN` / `_NET_WM_ACTION_MAXIMIZE_*` /
/// `_NET_WM_ACTION_MINIMIZE`); the bug where "maximize" collapsed onto true
/// fullscreen was exactly the loss of this distinction.
pub struct WindowStateActionsDistinct;

impl Axiom for WindowStateActionsDistinct {
    fn verify(&self) -> Verdict {
        let fs = realize(&WmAction::fullscreen());
        let mx = realize(&WmAction::maximize());
        let mn = realize(&WmAction::minimize());
        if fs != mx && mx != mn && fs != mn {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "WindowStateActionsDistinct",
        "fullscreen, maximize, and minimize realize to pairwise-distinct dispatchers",
        "EWMH v1.5 (2013) — _NET_WM_ACTION_FULLSCREEN, _NET_WM_ACTION_MAXIMIZE_*, _NET_WM_ACTION_MINIMIZE are distinct actions; Myers (1988) make-full-screen vs shrink-to-icon"
    );
}

/// A composite intent realizes to a **dispatcher sequence**, not a hand-written
/// shell hack: float-then-pin lowers to exactly `[togglefloating, pin]`. This is
/// the monoid (sequenced-effect) structure made concrete (Plotkin & Power 2003).
pub struct CompositeSequencePreserved;

impl Axiom for CompositeSequencePreserved {
    fn verify(&self) -> Verdict {
        let w = HyprlandRealization::map_morphism(&ActionWord(vec![
            WmAction::toggle_float(),
            WmAction::pin(),
        ]));
        if w.0 == vec![Dispatch::ToggleFloating, Dispatch::Pin] {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CompositeSequencePreserved",
        "a composite action (float+pin) realizes to a two-dispatcher sequence, not a single opaque command",
        "Plotkin & Power (2003) Algebraic Operations and Generic Effects — sequenced effects compose in the free monoid"
    );
}

/// The window-state mutations Hyprland realizes are **exactly** its declared
/// window-state capability: `lower_state` is non-empty for a [`StateBit`] iff the
/// bit is in [`hyprland_state_capability`]. This makes the empty-lowering arm an
/// intentional, checked capability boundary (the EWMH hints / layers Hyprland has
/// no user action for), never an accidental silent drop.
pub struct StateLoweringMatchesCapability;

impl Axiom for StateLoweringMatchesCapability {
    fn verify(&self) -> Verdict {
        let cap = hyprland_state_capability();
        for bit in <StateBit as FinitelyGenerated>::variants() {
            let realizable = !lower_state(&StateDelta::new(StateOp::Toggle, bit)).is_empty();
            if realizable != cap.contains(&bit) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "StateLoweringMatchesCapability",
        "a window-state atom lowers to a non-empty Hyprland dispatcher exactly when it is in Hyprland's declared window-state capability",
        "EWMH v1.5 (2013) §5 _NET_WM_STATE — the full state set; a backend realizes the subset its dispatchers expose (a capability), the rest are tracked gaps not silent drops"
    );
}

/// The restructure into the window-state lattice is **byte-faithful**: each
/// pre-restructure window-state verb still realizes to the exact Hyprland string
/// it emitted before. The frozen golden corpus is the v1 contract; if any
/// re-expression drifts a single byte, this axiom fails loudly (the migration
/// gate the design demanded).
pub struct MigrationFaithful;

impl Axiom for MigrationFaithful {
    fn verify(&self) -> Verdict {
        // (the frozen v1 Hyprland string, the restructured term that must still emit it)
        let golden: [(&str, WmAction); 7] = [
            ("fullscreen", WmAction::fullscreen()),
            ("fullscreen, 1", WmAction::maximize()),
            (
                "movetoworkspacesilent, special:minimized",
                WmAction::minimize(),
            ),
            ("togglefloating,", WmAction::toggle_float()),
            ("pin,", WmAction::pin()),
            ("pseudo,", WmAction::pseudotile()),
            ("layoutmsg, togglesplit", WmAction::toggle_split()),
        ];
        for (expected, action) in &golden {
            if realize(action) != *expected {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "MigrationFaithful",
        "every pre-restructure window-state verb realizes byte-for-byte to its frozen v1 Hyprland string",
        "the v1 realized-string corpus is the migration contract; the state-lattice restructure is a behaviour-preserving re-expression of the source signature"
    );
}

// ── The ontology ──────────────────────────────────────────────────────────────

/// The window-action ontology. Validating it discharges the realization functor
/// laws (identity + composition preservation — the proof that realization is a
/// strict, total functor) together with the domain axioms (totality, the
/// maximize/fullscreen distinction, composite sequencing).
///
/// Like the surface ontology and unlike the modal one, the underlying categories
/// are free monoids used as functor source/target — verified through
/// [`functor_law_axioms`], not category-closure laws (a free monoid's generating
/// set is not closed under composition).
pub struct WindowActionOntology;

impl WindowActionOntology {
    /// All axioms this ontology asserts.
    pub fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut all: Vec<Box<dyn Axiom>> = functor_law_axioms::<HyprlandRealization>();
        all.push(Box::new(LoweringTotal));
        all.push(Box::new(WindowStateActionsDistinct));
        all.push(Box::new(CompositeSequencePreserved));
        all.push(Box::new(StateLoweringMatchesCapability));
        all.push(Box::new(MigrationFaithful));
        all
    }

    /// Validate the whole ontology: functor laws + domain axioms, aggregated into
    /// one typed [`Verdict`].
    pub fn validate() -> Verdict {
        let meta = Provenance {
            name: OntologyName::new_static("WindowActionOntologyValidation"),
            description: Label::new_static(
                "aggregate validation: realization functor laws + window-action domain axioms",
            ),
            citation: Citation::parse_static(
                "Mac Lane (1971) Categories for the Working Mathematician Ch. II §1",
            ),
            module_path: ModulePath::new_static(module_path!()),
        };
        let subverdicts: Vec<Verdict> = Self::axioms().into_iter().map(|a| a.verify()).collect();
        combine_verdicts(meta, subverdicts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;
    use proptest::prelude::*;

    // ── Realization snapshots (the single wire boundary) ──

    #[test]
    fn realize_focus_directions() {
        assert_eq!(realize(&WmAction::Focus(Direction::Left)), "movefocus, l");
        assert_eq!(realize(&WmAction::Focus(Direction::Right)), "movefocus, r");
        assert_eq!(realize(&WmAction::Focus(Direction::Up)), "movefocus, u");
        assert_eq!(realize(&WmAction::Focus(Direction::Down)), "movefocus, d");
    }

    #[test]
    fn realize_resize_axes() {
        assert_eq!(
            realize(&WmAction::Resize(Direction::Left, 30)),
            "resizeactive, -30 0"
        );
        assert_eq!(
            realize(&WmAction::Resize(Direction::Down, 30)),
            "resizeactive, 0 30"
        );
    }

    #[test]
    fn realize_workspace_targets() {
        assert_eq!(
            realize(&WmAction::Workspace(WorkspaceTarget::Index(1))),
            "workspace, 1"
        );
        assert_eq!(
            realize(&WmAction::Workspace(WorkspaceTarget::Relative(1))),
            "workspace, +1"
        );
        assert_eq!(
            realize(&WmAction::Workspace(WorkspaceTarget::Relative(-1))),
            "workspace, -1"
        );
        assert_eq!(
            realize(&WmAction::Workspace(WorkspaceTarget::Named(
                "Music".to_string()
            ))),
            "workspace, Music"
        );
    }

    #[test]
    fn realize_special_workspace_hide() {
        // macOS-style hide: a silent move to a named special workspace.
        assert_eq!(
            realize(&WmAction::MoveToWorkspace(
                WorkspaceTarget::Special("hidden".to_string()),
                Follow::Silent
            )),
            "movetoworkspacesilent, special:hidden"
        );
        // GNOME-style hide to the default scratchpad.
        assert_eq!(
            realize(&WmAction::MoveToWorkspace(
                WorkspaceTarget::Special(String::new()),
                Follow::Silent
            )),
            "movetoworkspacesilent, special"
        );
    }

    // ── The three fixes ──

    #[test]
    fn fullscreen_maximize_minimize_are_distinct_verbs() {
        // Three distinct intents → three distinct dispatchers. Maximize is
        // fullscreen mode 1; minimize is the special-workspace emulation.
        assert_eq!(realize(&WmAction::fullscreen()), "fullscreen");
        assert_eq!(realize(&WmAction::maximize()), "fullscreen, 1");
        assert_eq!(
            realize(&WmAction::minimize()),
            "movetoworkspacesilent, special:minimized"
        );
    }

    #[test]
    fn fix_float_pin_is_two_dispatch_exec_batch() {
        let cmd = HyprlandRealization::map_morphism(&ActionWord(vec![
            WmAction::toggle_float(),
            WmAction::pin(),
        ]))
        .command();
        assert_eq!(
            cmd,
            "exec, hyprctl dispatch togglefloating ; hyprctl dispatch pin"
        );
    }

    // ── Functor + ontology validation ──

    #[test]
    fn realization_is_a_functor() {
        assert_functor_laws::<HyprlandRealization>();
    }

    #[test]
    fn lowering_is_total() {
        LoweringTotal
            .verify()
            .unwrap_or_else(|c| panic!("not total: {}", c.meta().name.as_str()));
    }

    #[test]
    fn window_state_actions_distinct() {
        WindowStateActionsDistinct
            .verify()
            .unwrap_or_else(|c| panic!("{}", c.meta().name.as_str()));
    }

    #[test]
    fn composite_sequence_preserved() {
        CompositeSequencePreserved
            .verify()
            .unwrap_or_else(|c| panic!("{}", c.meta().name.as_str()));
    }

    #[test]
    fn whole_ontology_validates() {
        WindowActionOntology::validate()
            .unwrap_or_else(|c| panic!("ontology invalid: {}", c.meta().name.as_str()));
    }

    // ── Property-based tests ──

    /// An arbitrary action drawn from the generating set.
    fn arb_action() -> impl Strategy<Value = WmAction> {
        prop_oneof![
            proptest::sample::select(WmAction::representative_actions()),
            any::<u8>().prop_map(|n| WmAction::Workspace(WorkspaceTarget::Index(n))),
            any::<i16>().prop_map(|k| WmAction::Workspace(WorkspaceTarget::Relative(k as i32))),
            "[a-z ]{0,12}".prop_map(WmAction::Exec),
        ]
    }

    fn arb_word() -> impl Strategy<Value = ActionWord> {
        proptest::collection::vec(arb_action(), 0..6).prop_map(ActionWord)
    }

    proptest! {
        /// The realization functor is a **monoid homomorphism**: lowering a
        /// concatenation equals concatenating the lowerings. This is composition
        /// preservation, the hallmark of a functor, on random action words.
        #[test]
        fn prop_realization_is_homomorphism(a in arb_word(), b in arb_word()) {
            let cat = ActionAlgebra::compose(&a, &b).unwrap();
            let lhs = HyprlandRealization::map_morphism(&cat);
            let ra = HyprlandRealization::map_morphism(&a);
            let rb = HyprlandRealization::map_morphism(&b);
            let rhs = DispatchAlgebra::compose(&ra, &rb).unwrap();
            prop_assert_eq!(lhs, rhs);
        }

        /// The empty action word maps to the empty dispatch word — identity
        /// preservation (F(id) = id).
        #[test]
        fn prop_realization_preserves_identity(_n in 0u8..1) {
            let id_src = ActionAlgebra::identity(&WmSurface::Compositor);
            let mapped = HyprlandRealization::map_morphism(&id_src);
            prop_assert_eq!(mapped, DispatchAlgebra::identity(&WmSurface::Compositor));
        }

        /// Every single action realizes to a non-empty dispatcher sequence
        /// (totality), and the rendered command's head token is a known
        /// dispatcher — there are no unbound placeholders left in the wire form.
        #[test]
        fn prop_single_action_realizes_nonempty(a in arb_action()) {
            let w = HyprlandRealization::map_morphism(&ActionWord::from(a));
            prop_assert!(!w.0.is_empty());
            let cmd = w.command();
            prop_assert!(!cmd.is_empty());
            // head token (before the first comma/space) is non-empty and lowercase.
            let head: String = cmd.chars().take_while(|c| *c != ',' && *c != ' ').collect();
            prop_assert!(!head.is_empty());
            prop_assert_eq!(&head, &head.to_lowercase());
        }

        /// Realization is deterministic: same action, same command string.
        #[test]
        fn prop_realization_deterministic(a in arb_action()) {
            prop_assert_eq!(realize(&a), realize(&a));
        }

        /// Concatenation in the action monoid is associative (the free-monoid law
        /// the functor's source category rests on).
        #[test]
        fn prop_action_concat_associative(a in arb_word(), b in arb_word(), c in arb_word()) {
            let ab = ActionAlgebra::compose(&a, &b).unwrap();
            let bc = ActionAlgebra::compose(&b, &c).unwrap();
            let left = ActionAlgebra::compose(&ab, &c).unwrap();
            let right = ActionAlgebra::compose(&a, &bc).unwrap();
            prop_assert_eq!(left, right);
        }
    }
}
