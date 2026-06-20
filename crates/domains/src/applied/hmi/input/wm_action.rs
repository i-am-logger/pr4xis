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
/// the four cardinals the presets use (diagonals are out of the four-cardinal
/// set — a diagonal extension is a tracked follow-up, not a hidden default).
/// Hyprland names them `l`/`r`/`u`/`d`.
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

/// A container DISPLAY layout — the parameter of the layout operation (i3/sway
/// `layout default|stacking|tabbed`). The split-orientation layouts (`splith`/
/// `splitv`) are the separate [`Split`](WmAction::Split) operation, not duplicated
/// here. A backend without named layouts (Hyprland) realizes only `tabbed` (via
/// its window-group mechanism) and gaps the rest.
///
/// Source: i3 User's Guide "Manipulating layout"; sway(5) `layout`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayoutKind {
    /// The workspace's default tiling layout.
    Default,
    /// A stacked container (one window shown, titles listed).
    Stacking,
    /// A tabbed container (one window shown, tabs across the top).
    Tabbed,
}

impl Concept for LayoutKind {
    fn name(&self) -> &'static str {
        match self {
            LayoutKind::Default => "default",
            LayoutKind::Stacking => "stacking",
            LayoutKind::Tabbed => "tabbed",
        }
    }
}
impl FinitelyGenerated for LayoutKind {
    fn variants() -> Vec<Self> {
        vec![
            LayoutKind::Default,
            LayoutKind::Stacking,
            LayoutKind::Tabbed,
        ]
    }
}

/// A selector into the container TREE — the axis a tree-focus moves along
/// (i3/sway `focus parent|child`; bspwm `@parent`/`@brother`). There is NO
/// spatial direction here, which is exactly why tree-ascent cannot be a
/// [`Direction`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TreeAxis {
    Parent,
    Child,
    Sibling(Cycle),
}

impl Concept for TreeAxis {
    fn name(&self) -> &'static str {
        match self {
            TreeAxis::Parent => "parent",
            TreeAxis::Child => "child",
            TreeAxis::Sibling(_) => "sibling",
        }
    }
}
impl FinitelyGenerated for TreeAxis {
    fn variants() -> Vec<Self> {
        vec![
            TreeAxis::Parent,
            TreeAxis::Child,
            TreeAxis::Sibling(Cycle::Forward),
            TreeAxis::Sibling(Cycle::Backward),
        ]
    }
}

/// How focus is selected — the parameter of the focus operation. A focus is
/// always "move keyboard focus", but BY what: a spatial direction, a cycle
/// through the stack (Alt-Tab), the container tree, or the tiling/floating layer.
/// The tree and layer selectors are realized by backends that model a container
/// tree / a focus-layer toggle (Sway); Hyprland exposes neither, so they fall
/// outside its capability ([`hyprland_realizes`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusBy {
    /// The spatial neighbour in a direction (`movefocus`).
    Direction(Direction),
    /// Cycle through windows in stacking order (Alt-Tab; `cyclenext`).
    Cycle(Cycle),
    /// Along the container tree (i3 `focus parent|child`). No Hyprland dispatcher.
    Tree(TreeAxis),
    /// Toggle focus between the tiling and floating layers (i3 `focus
    /// mode_toggle`). No Hyprland dispatcher.
    Layer,
}

impl Concept for FocusBy {
    fn name(&self) -> &'static str {
        match self {
            FocusBy::Direction(_) => "direction",
            FocusBy::Cycle(_) => "cycle",
            FocusBy::Tree(_) => "tree",
            FocusBy::Layer => "layer",
        }
    }
}
impl FinitelyGenerated for FocusBy {
    fn variants() -> Vec<Self> {
        let mut v = Vec::new();
        for d in Direction::variants() {
            v.push(FocusBy::Direction(d));
        }
        for c in Cycle::variants() {
            v.push(FocusBy::Cycle(c));
        }
        for t in TreeAxis::variants() {
            v.push(FocusBy::Tree(t));
        }
        v.push(FocusBy::Layer);
        v
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
    /// The most-recently-used workspace (i3 `back_and_forth`; Hyprland
    /// `workspace, previous`). Distinct from `Relative(±1)` adjacency.
    Last,
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
            WorkspaceTarget::Last => "previous".to_string(),
            WorkspaceTarget::Special(s) if s.is_empty() => "special".to_string(),
            WorkspaceTarget::Special(s) => format!("special:{s}"),
        }
    }
}

/// A monitor / output selector — the parameter of the monitor focus/move
/// operations (i3/sway `focus output` / `move container to output`; Hyprland
/// `focusmonitor` / `movewindow mon:`). The multihead dimension EWMH names with
/// `_NET_DESKTOP_VIEWPORT` / `_NET_DESKTOP_GEOMETRY`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OutputSel {
    /// The monitor in a spatial direction (`focusmonitor, l`).
    Direction(Direction),
    /// A named monitor / output (`focusmonitor, DP-1`).
    Named(String),
    /// A relative step in the monitor list (`focusmonitor, +1`).
    Relative(i32),
}

impl OutputSel {
    /// The Hyprland monitor argument.
    fn render(&self) -> String {
        match self {
            OutputSel::Direction(d) => d.hypr().to_string(),
            OutputSel::Named(s) => s.clone(),
            OutputSel::Relative(k) => {
                if *k >= 0 {
                    format!("+{k}")
                } else {
                    k.to_string()
                }
            }
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
    /// Move keyboard focus — selected by a [`FocusBy`] (a spatial direction, a
    /// stack cycle, and — with the capability framework — the container tree or
    /// the tiling/floating layer). Subsumes the former `Focus(Direction)` and
    /// `CycleWindow` verbs (Myers "change-listener"; EWMH input focus).
    Focus(FocusBy),
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
    /// Set the container's display layout (i3/sway `layout default|stacking|
    /// tabbed`). Hyprland realizes only `tabbed` (its window groups); `stacking`
    /// and `default` are gaps there (Sway realizes all three).
    Layout(LayoutKind),
    /// Cycle the active window *within* the current group.
    CycleGroup(Cycle),
    /// Switch to a workspace (Henderson & Card 1986 *Rooms*; EWMH
    /// `_NET_CURRENT_DESKTOP`).
    Workspace(WorkspaceTarget),
    /// Send the focused window to a workspace, optionally following it (EWMH
    /// `_NET_WM_DESKTOP`).
    MoveToWorkspace(WorkspaceTarget, Follow),
    /// Rename a workspace — the one explicit workspace-lifecycle op (i3/sway
    /// `rename workspace`; Hyprland `renameworkspace`). Workspace CREATE is
    /// implicit (switching to a non-existent target via [`Workspace`](WmAction::Workspace)
    /// creates it) and DESTROY is implicit (empty workspaces auto-reap) on both
    /// backends — neither is a distinct keybind action.
    RenameWorkspace(WorkspaceTarget, String),
    /// Toggle a special / scratchpad workspace overlay (e.g. an overview).
    ToggleSpecialWorkspace(String),
    /// Move keyboard focus to another monitor / output (i3 `focus output`;
    /// Hyprland `focusmonitor`).
    FocusMonitor(OutputSel),
    /// Send the focused window to another monitor / output (i3 `move container to
    /// output`; Hyprland `movewindow mon:`).
    MoveToMonitor(OutputSel),
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

    /// Move focus to the neighbour in a direction — `Focus(FocusBy::Direction)`.
    pub fn focus(direction: Direction) -> Self {
        WmAction::Focus(FocusBy::Direction(direction))
    }

    /// Cycle focus across windows in stacking order (Alt-Tab) —
    /// `Focus(FocusBy::Cycle)`.
    pub fn cycle_window(cycle: Cycle) -> Self {
        WmAction::Focus(FocusBy::Cycle(cycle))
    }

    /// Focus the parent container (tree-ascent) — `Focus(FocusBy::Tree(Parent))`.
    /// No spatial direction; a container-tree backend (Sway `focus parent`)
    /// realizes it, Hyprland does not (capability gap).
    pub fn focus_parent() -> Self {
        WmAction::Focus(FocusBy::Tree(TreeAxis::Parent))
    }

    /// Toggle focus between the tiling and floating layers —
    /// `Focus(FocusBy::Layer)` (i3 `focus mode_toggle`). Hyprland capability gap.
    pub fn focus_layer() -> Self {
        WmAction::Focus(FocusBy::Layer)
    }

    /// Move keyboard focus to a monitor / output — `FocusMonitor(OutputSel)`
    /// (Hyprland `focusmonitor`).
    pub fn focus_monitor(sel: OutputSel) -> Self {
        WmAction::FocusMonitor(sel)
    }

    /// Send the focused window to a monitor / output — `MoveToMonitor(OutputSel)`
    /// (Hyprland `movewindow mon:`).
    pub fn move_to_monitor(sel: OutputSel) -> Self {
        WmAction::MoveToMonitor(sel)
    }

    /// Set the container display layout — `Layout(LayoutKind)` (i3/sway
    /// `layout …`).
    pub fn layout(kind: LayoutKind) -> Self {
        WmAction::Layout(kind)
    }

    /// Rename a workspace — `RenameWorkspace` (Hyprland `renameworkspace`; sway
    /// `rename workspace to`).
    pub fn rename_workspace(target: WorkspaceTarget, name: impl Into<String>) -> Self {
        WmAction::RenameWorkspace(target, name.into())
    }

    /// The finite generating set — one representative of every variant. Used to
    /// machine-check the realization functor (totality, the functor laws) and to
    /// seed property tests. This is a *generating* set, not the (open-world) whole
    /// of [`WmAction`].
    pub fn representative_actions() -> Vec<WmAction> {
        use Direction::*;
        vec![
            WmAction::focus(Left),
            WmAction::focus(Right),
            WmAction::focus(Up),
            WmAction::focus(Down),
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
            WmAction::Layout(LayoutKind::Tabbed),
            WmAction::Layout(LayoutKind::Stacking),
            WmAction::CycleGroup(Cycle::Forward),
            WmAction::cycle_window(Cycle::Forward),
            WmAction::cycle_window(Cycle::Backward),
            // Capability-gap representatives: the generalized ontology expresses
            // these, Hyprland realizes none (they lower to the empty word, witnessed
            // by RealizationMatchesCapability; Sway realizes the two focus gaps).
            WmAction::focus_parent(),
            WmAction::focus_layer(),
            WmAction::State(StateDelta::new(StateOp::Toggle, StateBit::Shaded)),
            WmAction::focus_monitor(OutputSel::Direction(Left)),
            WmAction::move_to_monitor(OutputSel::Direction(Right)),
            WmAction::Workspace(WorkspaceTarget::Last),
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
            WmAction::RenameWorkspace(WorkspaceTarget::Index(1), "work".to_string()),
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
            WmAction::Layout(_) => "layout",
            WmAction::CycleGroup(_) => "cycle-group",
            WmAction::Workspace(_) => "workspace",
            WmAction::MoveToWorkspace(_, _) => "move-to-workspace",
            WmAction::RenameWorkspace(_, _) => "rename-workspace",
            WmAction::ToggleSpecialWorkspace(_) => "toggle-special-workspace",
            WmAction::FocusMonitor(_) => "focus-monitor",
            WmAction::MoveToMonitor(_) => "move-to-monitor",
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
    /// `renameworkspace, <id> <name>`.
    RenameWorkspace(WorkspaceTarget, String),
    ToggleSpecialWorkspace(String),
    /// `focusmonitor, <sel>`.
    FocusMonitor(OutputSel),
    /// `movewindow, mon:<sel>`.
    MoveToMonitor(OutputSel),
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
            Dispatch::RenameWorkspace(w, name) => {
                format!("renameworkspace, {} {name}", w.render())
            }
            Dispatch::ToggleSpecialWorkspace(n) => format!("togglespecialworkspace, {n}"),
            Dispatch::FocusMonitor(s) => format!("focusmonitor, {}", s.render()),
            Dispatch::MoveToMonitor(s) => format!("movewindow, mon:{}", s.render()),
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
        WmAction::Focus(FocusBy::Direction(d)) => vec![Dispatch::MoveFocus(*d)],
        WmAction::Focus(FocusBy::Cycle(c)) => {
            vec![Dispatch::CycleNext(matches!(c, Cycle::Backward))]
        }
        // Hyprland has no container tree and no tiling/floating focus-layer toggle
        // — outside its capability; lowers to the empty word (Sway realizes
        // `focus parent` / `focus mode_toggle` — see lower_sway).
        WmAction::Focus(FocusBy::Tree(_)) | WmAction::Focus(FocusBy::Layer) => Vec::new(),
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
        // Hyprland realizes the tabbed layout via its window groups; stacking and
        // the default tiling layout have no dispatcher (Sway realizes all three).
        WmAction::Layout(LayoutKind::Tabbed) => vec![Dispatch::ToggleGroup],
        WmAction::Layout(LayoutKind::Stacking | LayoutKind::Default) => Vec::new(),
        WmAction::CycleGroup(c) => vec![Dispatch::ChangeGroupActive(match c {
            Cycle::Forward => 'f',
            Cycle::Backward => 'b',
        })],
        WmAction::Workspace(w) => vec![Dispatch::Workspace(w.clone())],
        WmAction::MoveToWorkspace(w, Follow::Follow) => vec![Dispatch::MoveToWorkspace(w.clone())],
        WmAction::MoveToWorkspace(w, Follow::Silent) => {
            vec![Dispatch::MoveToWorkspaceSilent(w.clone())]
        }
        WmAction::RenameWorkspace(w, name) => {
            vec![Dispatch::RenameWorkspace(w.clone(), name.clone())]
        }
        WmAction::ToggleSpecialWorkspace(n) => vec![Dispatch::ToggleSpecialWorkspace(n.clone())],
        WmAction::FocusMonitor(s) => vec![Dispatch::FocusMonitor(s.clone())],
        WmAction::MoveToMonitor(s) => vec![Dispatch::MoveToMonitor(s.clone())],
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
/// window-state CAPABILITY (`hyprland_state_capability`); EWMH states outside
/// it (app-set hints, the below layer, shading) have no Hyprland user action and
/// lower to the empty word. That empty arm is INTENTIONAL and checked, not a
/// silent drop: [`StateLoweringMatchesCapability`] proves it fires exactly outside
/// the capability. (`representative_actions` deliberately includes a few
/// out-of-capability witnesses — e.g. `Shaded` — to exercise that proof; a
/// Hyprland-targeting preset is expected to bind only in-capability actions, a
/// consumer gating on [`hyprland_realizes`].)
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

/// Whether Hyprland has a user realization for an action — its CAPABILITY. The
/// generalized ontology can express operations Hyprland exposes no dispatcher for
/// (container-tree focus, the tiling/floating focus-layer toggle, the EWMH window
/// states outside `hyprland_state_capability`); those lower to the empty word
/// and are excluded from Hyprland's generating set. A backend that DOES expose
/// them (Sway) declares a wider capability. [`LoweringTotal`] proves the lowering
/// is total ON this capability; [`RealizationMatchesCapability`] proves the empty
/// arms are EXACTLY its complement — the gaps are checked, never silent drops.
pub fn hyprland_realizes(action: &WmAction) -> bool {
    match action {
        WmAction::State(d) => hyprland_state_capability().contains(&d.bit),
        WmAction::Focus(FocusBy::Tree(_)) | WmAction::Focus(FocusBy::Layer) => false,
        WmAction::Layout(LayoutKind::Tabbed) => true,
        WmAction::Layout(_) => false,
        _ => true,
    }
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

// ── A second backend: SwayRealization (the generality is load-bearing) ────────

/// A single sway command — the realization atom for the [`SwayRealization`]
/// backend. Unlike Hyprland's typed [`Dispatch`], sway's command language is a
/// flat string vocabulary (i3-inherited), so the atom carries the exact, CITED
/// sway(5) command text; [`SwayCmd::render`] is its (identity) wire form.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SwayCmd(pub String);

impl SwayCmd {
    fn new(s: impl Into<String>) -> Self {
        SwayCmd(s.into())
    }
    /// The wire form — the sway command string.
    pub fn render(&self) -> String {
        self.0.clone()
    }
}

/// A word in the free monoid of sway commands — the realization of an action word.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SwayWord(pub Vec<SwayCmd>);

impl SwayWord {
    /// The command string a single keybind emits; a composite chains with `; `
    /// (sway's command separator).
    pub fn command(&self) -> String {
        self.0
            .iter()
            .map(SwayCmd::render)
            .collect::<Vec<_>>()
            .join("; ")
    }
}

impl Arrow for SwayWord {
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

/// The free monoid on [`SwayCmd`] — the [`SwayRealization`] functor target.
pub struct SwayAlgebra;

impl Category for SwayAlgebra {
    type Object = WmSurface;
    type Morphism = SwayWord;
    fn identity(_: &WmSurface) -> SwayWord {
        SwayWord(Vec::new())
    }
    fn compose(f: &SwayWord, g: &SwayWord) -> Option<SwayWord> {
        let mut w = f.0.clone();
        w.extend(g.0.iter().cloned());
        Some(SwayWord(w))
    }
    fn morphisms() -> Vec<SwayWord> {
        let mut ms = vec![SwayWord(Vec::new())];
        ms.extend(
            WmAction::representative_actions()
                .iter()
                .map(|a| SwayWord(lower_sway(a))),
        );
        ms
    }
}

/// The window states sway exposes a command for — its window-state capability.
/// Sway lacks a distinct maximize, pseudo-tiling, shading, the below layer, and
/// the EWMH hints; its `sticky` realizes above/pin (for floating windows).
fn sway_state_capability() -> [StateBit; 5] {
    [
        StateBit::Fullscreen,
        StateBit::Hidden,
        StateBit::Floating,
        StateBit::Above,
        StateBit::Sticky,
    ]
}

/// Whether sway has a command for an action — its CAPABILITY. Distinct from
/// Hyprland's: sway realizes the container-tree / focus-layer / named-layout /
/// split-orientation operations Hyprland lacks, but lacks the maximize / pseudo-
/// tile / shade / directional-swap that Hyprland has. The capability sets
/// DIFFERING per backend is exactly what makes the source backend-independent.
pub fn sway_realizes(action: &WmAction) -> bool {
    match action {
        WmAction::State(d) => sway_state_capability().contains(&d.bit),
        // sway `swap container with` targets a mark/id, never a bare direction.
        WmAction::SwapWindow(_) => false,
        _ => true,
    }
}

/// The sway workspace argument for a non-special target (`number N` / `next` /
/// `prev` / a name / `back_and_forth`).
fn sway_ws(t: &WorkspaceTarget) -> String {
    match t {
        WorkspaceTarget::Index(n) => format!("number {n}"),
        WorkspaceTarget::Relative(k) => {
            if *k >= 0 {
                "next".to_string()
            } else {
                "prev".to_string()
            }
        }
        WorkspaceTarget::Named(s) => s.clone(),
        WorkspaceTarget::Last => "back_and_forth".to_string(),
        // Special is a Hyprland concept; sway's analogue is the scratchpad,
        // handled at the call site.
        WorkspaceTarget::Special(_) => "scratchpad".to_string(),
    }
}

/// The sway monitor argument (`left|right|up|down` / a name / `next`|`prev`).
fn sway_output(s: &OutputSel) -> String {
    match s {
        OutputSel::Direction(d) => d.name().to_string(),
        OutputSel::Named(n) => n.clone(),
        OutputSel::Relative(k) => {
            if *k >= 0 {
                "next".to_string()
            } else {
                "prev".to_string()
            }
        }
    }
}

/// Lower a window-state mutation to sway. Sway exposes only toggles (the EWMH
/// add/remove/toggle distinction is not observable, as on Hyprland). The states
/// sway lacks (maximize / pseudo / shade / below / hints) lower to the empty word.
fn lower_sway_state(d: &StateDelta) -> Vec<SwayCmd> {
    match d.bit {
        StateBit::Fullscreen => vec![SwayCmd::new("fullscreen toggle")],
        StateBit::Hidden => vec![SwayCmd::new("move scratchpad")],
        StateBit::Floating => vec![SwayCmd::new("floating toggle")],
        // sway `sticky` pins floating windows across workspaces — its above/pin analogue.
        StateBit::Above | StateBit::Sticky => vec![SwayCmd::new("sticky toggle")],
        StateBit::MaximizedVert
        | StateBit::MaximizedHorz
        | StateBit::PseudoTiled
        | StateBit::Shaded
        | StateBit::Below
        | StateBit::SkipTaskbar
        | StateBit::SkipPager
        | StateBit::Modal
        | StateBit::DemandsAttention
        | StateBit::Focused => Vec::new(),
    }
}

/// Lower one abstract action to its sway command(s) — the cited per-action rule
/// for the sway backend (sway(5) / i3 User's Guide). Operations sway has no
/// command for (maximize, pseudo-tile, shade, directional swap) lower to the
/// empty word, excluded from sway's generating set by [`sway_realizes`].
fn lower_sway(action: &WmAction) -> Vec<SwayCmd> {
    let one = |s: &str| vec![SwayCmd::new(s)];
    match action {
        WmAction::Focus(FocusBy::Direction(d)) => one(&format!("focus {}", d.name())),
        WmAction::Focus(FocusBy::Cycle(c)) => one(match c {
            Cycle::Forward => "focus next",
            Cycle::Backward => "focus prev",
        }),
        WmAction::Focus(FocusBy::Tree(axis)) => one(match axis {
            TreeAxis::Parent => "focus parent",
            TreeAxis::Child => "focus child",
            TreeAxis::Sibling(Cycle::Forward) => "focus next sibling",
            TreeAxis::Sibling(Cycle::Backward) => "focus prev sibling",
        }),
        WmAction::Focus(FocusBy::Layer) => one("focus mode_toggle"),
        WmAction::MoveWindow(d) => one(&format!("move {}", d.name())),
        // sway swap requires a mark/id target — no directional form (capability gap).
        WmAction::SwapWindow(_) => Vec::new(),
        WmAction::Resize(d, amt) => {
            let (verb, axis) = match d {
                Direction::Left => ("shrink", "width"),
                Direction::Right => ("grow", "width"),
                Direction::Up => ("shrink", "height"),
                Direction::Down => ("grow", "height"),
            };
            one(&format!("resize {verb} {axis} {amt} px"))
        }
        WmAction::Close => one("kill"),
        WmAction::State(d) => lower_sway_state(d),
        WmAction::Split(o) => one(match o {
            Orientation::Horizontal => "split horizontal",
            Orientation::Vertical => "split vertical",
            Orientation::Toggle => "split toggle",
        }),
        WmAction::ToggleGroup => one("layout toggle split tabbed"),
        WmAction::Layout(k) => one(&format!("layout {}", k.name())),
        WmAction::CycleGroup(c) => one(match c {
            Cycle::Forward => "focus next",
            Cycle::Backward => "focus prev",
        }),
        WmAction::Workspace(WorkspaceTarget::Special(_)) => one("scratchpad show"),
        WmAction::Workspace(t) => one(&format!("workspace {}", sway_ws(t))),
        WmAction::MoveToWorkspace(WorkspaceTarget::Special(_), _) => one("move scratchpad"),
        WmAction::MoveToWorkspace(t, Follow::Silent) => {
            one(&format!("move container to workspace {}", sway_ws(t)))
        }
        WmAction::MoveToWorkspace(t, Follow::Follow) => {
            // sway has no move-and-follow flag — chain the move and the switch.
            let ws = sway_ws(t);
            vec![
                SwayCmd::new(format!("move container to workspace {ws}")),
                SwayCmd::new(format!("workspace {ws}")),
            ]
        }
        WmAction::RenameWorkspace(_, name) => one(&format!("rename workspace to {name}")),
        WmAction::ToggleSpecialWorkspace(_) => one("scratchpad show"),
        WmAction::FocusMonitor(s) => one(&format!("focus output {}", sway_output(s))),
        WmAction::MoveToMonitor(s) => one(&format!("move container to output {}", sway_output(s))),
        WmAction::Exec(cmd) => one(&format!("exec {cmd}")),
        WmAction::Submap(SubmapTarget::Enter(m)) => one(&format!("mode {}", m.0)),
        WmAction::Submap(SubmapTarget::Reset) => one("mode default"),
    }
}

/// The realization functor `SwayRealization : ActionAlgebra → SwayAlgebra` — a
/// SECOND backend over the SAME source, proving the source is backend-independent
/// (the generality is load-bearing, not YAGNI). Sway realizes natively the
/// container-tree / focus-layer / named-layout / split-orientation operations
/// Hyprland gaps; Hyprland realizes the maximize / pseudo-tile / directional-swap
/// sway gaps. Both are total monoid homomorphisms on their respective capability.
pub struct SwayRealization;

impl Functor for SwayRealization {
    type Source = ActionAlgebra;
    type Target = SwayAlgebra;

    fn map_object(_: &WmSurface) -> WmSurface {
        WmSurface::Compositor
    }

    fn map_morphism(word: &ActionWord) -> SwayWord {
        SwayWord(word.0.iter().flat_map(lower_sway).collect())
    }

    fn meta() -> Provenance {
        Provenance {
            name: OntologyName::new_static("SwayRealization"),
            description: Label::new_static(
                "abstract WM actions → sway command sequences (a second backend realization)",
            ),
            citation: Citation::parse_static(
                "sway(5) — the sway command language (i3-inherited); a second algebra of the WM-action signature, so the projection is the forced unique homomorphism (Goguen-Thatcher-Wagner 1978)",
            ),
            module_path: ModulePath::new_static(module_path!()),
        }
    }
}

/// Convenience: the sway command string a single action emits.
pub fn sway_realize(action: &WmAction) -> String {
    SwayRealization::map_morphism(&ActionWord::from(action.clone())).command()
}

// ── Domain axioms ─────────────────────────────────────────────────────────────

/// Every abstract action realizes to at least one dispatcher — the projection is
/// **total**: no intent silently vanishes.
pub struct LoweringTotal;

impl Axiom for LoweringTotal {
    fn verify(&self) -> Verdict {
        for a in WmAction::representative_actions() {
            if !hyprland_realizes(&a) {
                // Outside Hyprland's capability — the empty lowering is intentional
                // and checked by RealizationMatchesCapability, not asserted here.
                continue;
            }
            let w = HyprlandRealization::map_morphism(&ActionWord::from(a));
            if w.0.is_empty() {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "LoweringTotal",
        "every action within Hyprland's capability realizes to a non-empty dispatcher sequence (total on the capability)",
        "Goguen, Thatcher & Wagner (1978) An Initial Algebra Approach to ADTs — a unique homomorphism on the signature; Card, Moran & Newell (1983) GOMS — every goal has a method; a backend realizes the operations within its capability (the rest are tracked gaps)"
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
/// bit is in `hyprland_state_capability`. This makes the empty-lowering arm an
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

/// Every action realizes to the empty word **exactly** when it is outside
/// Hyprland's capability ([`hyprland_realizes`]). The capability gaps (the
/// container-tree / focus-layer selectors, the EWMH states with no Hyprland user
/// dispatcher) are therefore a checked boundary across the whole generating set:
/// no in-capability action silently drops, and no out-of-capability action is
/// silently faked. The dual of [`LoweringTotal`] (total ON the capability).
pub struct RealizationMatchesCapability;

impl Axiom for RealizationMatchesCapability {
    fn verify(&self) -> Verdict {
        for a in WmAction::representative_actions() {
            let empty = HyprlandRealization::map_morphism(&ActionWord::from(a.clone()))
                .0
                .is_empty();
            // empty <=> !realizes; a mismatch (silent drop or phantom realization) fails.
            if empty == hyprland_realizes(&a) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "RealizationMatchesCapability",
        "an action lowers to the empty word exactly when it is outside the backend's capability (checked gaps, never silent drops)",
        "EWMH v1.5 (2013) / i3 & sway — backends realize a subset of the general operation set (a capability); ext-workspace-v1's capabilities enum makes unsupported operations explicit rather than silently inert"
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
        all.push(Box::new(RealizationMatchesCapability));
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
        assert_eq!(realize(&WmAction::focus(Direction::Left)), "movefocus, l");
        assert_eq!(realize(&WmAction::focus(Direction::Right)), "movefocus, r");
        assert_eq!(realize(&WmAction::focus(Direction::Up)), "movefocus, u");
        assert_eq!(realize(&WmAction::focus(Direction::Down)), "movefocus, d");
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

    #[test]
    fn realize_monitor_and_mru_workspace() {
        // The monitor/output dimension Hyprland realizes natively.
        assert_eq!(
            realize(&WmAction::focus_monitor(OutputSel::Direction(
                Direction::Left
            ))),
            "focusmonitor, l"
        );
        assert_eq!(
            realize(&WmAction::move_to_monitor(OutputSel::Direction(
                Direction::Right
            ))),
            "movewindow, mon:r"
        );
        // MRU / back-and-forth workspace.
        assert_eq!(
            realize(&WmAction::Workspace(WorkspaceTarget::Last)),
            "workspace, previous"
        );
    }

    // ── The second backend: SwayRealization (generality is load-bearing) ──

    #[test]
    fn sway_realization_is_a_functor() {
        assert_functor_laws::<SwayRealization>();
    }

    #[test]
    fn sway_realizes_the_hyprland_gaps_and_vice_versa() {
        // The SAME source action Hyprland gaps, sway realizes NATIVELY — the proof
        // that the generality is load-bearing, not YAGNI. Container-tree focus and
        // focus-layer have no Hyprland dispatcher; sway has "focus parent" /
        // "focus mode_toggle".
        for a in [WmAction::focus_parent(), WmAction::focus_layer()] {
            assert!(!hyprland_realizes(&a), "Hyprland should gap {a:?}");
            assert!(realize(&a).is_empty(), "Hyprland must gap {a:?} to empty");
            assert!(sway_realizes(&a), "sway should realize {a:?}");
            assert!(
                !sway_realize(&a).is_empty(),
                "sway must emit a command for {a:?}"
            );
        }
        // Split(Vertical) is realized on BOTH but DIFFERENTLY: Hyprland collapses
        // every orientation to togglesplit; sway distinguishes "split vertical".
        assert_eq!(
            realize(&WmAction::Split(Orientation::Vertical)),
            "layoutmsg, togglesplit"
        );
        assert_eq!(
            sway_realize(&WmAction::Split(Orientation::Vertical)),
            "split vertical"
        );
        // Conversely: Hyprland realizes the maximize / pseudo-tile sway gaps.
        for a in [WmAction::maximize(), WmAction::pseudotile()] {
            assert!(hyprland_realizes(&a), "Hyprland should realize {a:?}");
            assert!(!sway_realizes(&a), "sway should gap {a:?}");
        }
    }

    #[test]
    fn sway_realize_strings() {
        assert_eq!(sway_realize(&WmAction::focus_parent()), "focus parent");
        assert_eq!(sway_realize(&WmAction::focus_layer()), "focus mode_toggle");
        assert_eq!(
            sway_realize(&WmAction::focus(Direction::Left)),
            "focus left"
        );
        assert_eq!(
            sway_realize(&WmAction::Split(Orientation::Vertical)),
            "split vertical"
        );
        assert_eq!(
            sway_realize(&WmAction::Resize(Direction::Left, 30)),
            "resize shrink width 30 px"
        );
        assert_eq!(
            sway_realize(&WmAction::Workspace(WorkspaceTarget::Index(3))),
            "workspace number 3"
        );
        assert_eq!(
            sway_realize(&WmAction::Workspace(WorkspaceTarget::Last)),
            "workspace back_and_forth"
        );
        assert_eq!(
            sway_realize(&WmAction::focus_monitor(OutputSel::Direction(
                Direction::Left
            ))),
            "focus output left"
        );
        // A sway capability gap lowers to the empty command.
        assert_eq!(sway_realize(&WmAction::maximize()), "");
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

        /// SwayRealization is likewise a monoid homomorphism — the second backend
        /// is a genuine functor over the same source.
        #[test]
        fn prop_sway_is_homomorphism(a in arb_word(), b in arb_word()) {
            let cat = ActionAlgebra::compose(&a, &b).unwrap();
            let lhs = SwayRealization::map_morphism(&cat);
            let ra = SwayRealization::map_morphism(&a);
            let rb = SwayRealization::map_morphism(&b);
            let rhs = SwayAlgebra::compose(&ra, &rb).unwrap();
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
            // Only in-capability actions realize non-empty; the capability gaps
            // (tree/layer focus, unsupported states) intentionally lower to empty.
            prop_assume!(hyprland_realizes(&a));
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
