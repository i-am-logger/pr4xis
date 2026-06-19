#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};
use hashbrown::{HashMap, HashSet};

/// Keybinding ontology — formal model of keyboard shortcuts and presets.
///
/// A keybinding maps (Key, Modifiers, Mode) → Action.
/// The ontology defines the structure; presets (vim, emacs, macOS, windows)
/// are instances — different morphism sets over the same key space.
///
/// Sources:
/// - Card, Mackinlay & Robertson, "Morphological Analysis of Input Devices" (1991)
/// - Beaudouin-Lafon, "Instrumental Interaction" (2000): modes activate instruments
/// - Harel, "Statecharts" (1987): mode-scoped keybindings
/// - XKB specification: modifier model (Shift, Ctrl, Alt, Super, Hyper)
use super::modes::ModeId;
use super::wm_action::{
    ActionWord, Cycle, Direction, Follow, HyprlandRealization, Orientation, SubmapTarget, WmAction,
    WorkspaceTarget,
};
use pr4xis::category::Functor;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

/// A physical key identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    Letter(char),       // a-z
    Number(u8),         // 0-9
    Function(u8),       // F1-F24
    Named(NamedKey),    // Enter, Escape, Space, Tab, etc.
    Mouse(MouseButton), // mouse buttons
}

/// Named (non-character) keys.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NamedKey {
    Enter,
    Escape,
    Space,
    Tab,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Print,
    ScrollLock,
    CapsLock,
    /// Media/hardware keys
    VolumeUp,
    VolumeDown,
    VolumeMute,
    BrightnessUp,
    BrightnessDown,
    MediaPlay,
    MediaNext,
    MediaPrev,
}

/// Mouse buttons.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    ScrollUp,
    ScrollDown,
}

/// Modifier keys — can be combined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Modifier {
    Shift,
    Ctrl,
    Alt,
    Super,
    Hyper,
}

/// A key combination: modifiers + key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyCombo {
    pub modifiers: Vec<Modifier>,
    pub key: Key,
}

impl KeyCombo {
    pub fn new(key: Key) -> Self {
        Self {
            modifiers: Vec::new(),
            key,
        }
    }

    pub fn with_mod(mut self, modifier: Modifier) -> Self {
        if !self.modifiers.contains(&modifier) {
            self.modifiers.push(modifier);
            self.modifiers.sort(); // canonical order
        }
        self
    }

    /// Human-readable representation.
    pub fn display(&self) -> String {
        let mut parts: Vec<String> = self
            .modifiers
            .iter()
            .map(|m| match m {
                Modifier::Shift => "Shift",
                Modifier::Ctrl => "Ctrl",
                Modifier::Alt => "Alt",
                Modifier::Super => "Super",
                Modifier::Hyper => "Hyper",
            })
            .map(String::from)
            .collect();
        parts.push(match &self.key {
            Key::Letter(c) => c.to_uppercase().to_string(),
            Key::Number(n) => n.to_string(),
            Key::Function(n) => format!("F{}", n),
            Key::Named(k) => format!("{:?}", k),
            Key::Mouse(b) => format!("Mouse{:?}", b),
        });
        parts.join(" + ")
    }
}

/// An action that a keybinding triggers — its name, a human description, and the
/// typed [`WmAction`] sequence it performs (the abstract intent, **not** a raw
/// dispatcher string). The concrete compositor command is *derived* on demand by
/// [`Action::command`] via the [`HyprlandRealization`] functor.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Action {
    pub name: String,
    pub description: String,
    /// The typed action sequence this binding performs.
    pub actions: ActionWord,
}

impl Action {
    pub fn new(
        name: impl Into<String>,
        desc: impl Into<String>,
        actions: impl Into<ActionWord>,
    ) -> Self {
        Self {
            name: name.into(),
            description: desc.into(),
            actions: actions.into(),
        }
    }

    /// The concrete Hyprland command this binding emits — the realization of its
    /// [`ActionWord`] under [`HyprlandRealization`], rendered at the single wire
    /// boundary. A composite (multi-action) binding batches into one `exec`.
    pub fn command(&self) -> String {
        HyprlandRealization::map_morphism(&self.actions).command()
    }
}

/// A keybinding: key combo + mode context → action.
#[derive(Debug, Clone)]
pub struct Binding {
    pub combo: KeyCombo,
    pub mode: ModeId,
    pub action: Action,
    /// Does this binding repeat when held?
    pub repeat: bool,
}

/// A keybinding set — all bindings for a configuration.
#[derive(Debug, Clone)]
pub struct BindingSet {
    pub name: String,
    pub bindings: Vec<Binding>,
}

impl BindingSet {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bindings: Vec::new(),
        }
    }

    pub fn add(&mut self, combo: KeyCombo, mode: ModeId, action: Action, repeat: bool) {
        self.bindings.push(Binding {
            combo,
            mode,
            action,
            repeat,
        });
    }

    /// Get all bindings for a specific mode.
    pub fn for_mode(&self, mode: &ModeId) -> Vec<&Binding> {
        self.bindings.iter().filter(|b| b.mode == *mode).collect()
    }

    /// Detect conflicts: same key combo in the same mode.
    pub fn conflicts(&self) -> Vec<(&Binding, &Binding)> {
        let mut found = Vec::new();
        for (i, a) in self.bindings.iter().enumerate() {
            for b in self.bindings.iter().skip(i + 1) {
                if a.combo == b.combo && a.mode == b.mode {
                    found.push((a, b));
                }
            }
        }
        found
    }

    /// Count of unique key combos per mode.
    pub fn combos_per_mode(&self) -> HashMap<ModeId, usize> {
        let mut counts: HashMap<ModeId, HashSet<&KeyCombo>> = HashMap::new();
        for b in &self.bindings {
            counts.entry(b.mode.clone()).or_default().insert(&b.combo);
        }
        counts.into_iter().map(|(k, v)| (k, v.len())).collect()
    }
}

/// A keybinding remap: transforms one key combo to another.
/// Used for Super→Ctrl remapping (macOS-style).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remap {
    pub from: KeyCombo,
    pub to: KeyCombo,
}

/// A remap set — a collection of key remappings.
/// This is a functor: maps the key space to itself.
#[derive(Debug, Clone)]
pub struct RemapSet {
    pub name: String,
    pub remaps: Vec<Remap>,
}

impl RemapSet {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            remaps: Vec::new(),
        }
    }

    pub fn add(&mut self, from: KeyCombo, to: KeyCombo) {
        self.remaps.push(Remap { from, to });
    }

    /// Apply the remap: if the combo matches a `from`, return the `to`.
    pub fn apply(&self, combo: &KeyCombo) -> Option<&KeyCombo> {
        self.remaps.iter().find(|r| r.from == *combo).map(|r| &r.to)
    }
}

// ── Presets ──

/// macOS-style Super→Ctrl remap for common shortcuts.
///
/// Source: macOS uses Command (≈ Super) for copy/paste/save/etc.
/// On Linux, these are Ctrl+C/V/S. The remap makes Super behave like Command.
pub fn macos_remap() -> RemapSet {
    let mut rs = RemapSet::new("macos");
    let letters = "cvxzsafwtnprobluigdyq";
    for c in letters.chars() {
        rs.add(
            KeyCombo::new(Key::Letter(c)).with_mod(Modifier::Super),
            KeyCombo::new(Key::Letter(c)).with_mod(Modifier::Ctrl),
        );
    }
    rs
}

/// vim-style mode keybindings (insert returns to normal on Escape).
pub fn vim_preset() -> BindingSet {
    let mut bs = BindingSet::new("vim");
    let normal = ModeId::new("normal");
    let insert = ModeId::new("insert");

    // Normal mode: hjkl navigation, i=insert, :=command
    bs.add(
        KeyCombo::new(Key::Letter('h')),
        normal.clone(),
        Action::new(
            "move_left",
            "Move cursor left",
            WmAction::Focus(Direction::Left),
        ),
        false,
    );
    bs.add(
        KeyCombo::new(Key::Letter('j')),
        normal.clone(),
        Action::new(
            "move_down",
            "Move cursor down",
            WmAction::Focus(Direction::Down),
        ),
        false,
    );
    bs.add(
        KeyCombo::new(Key::Letter('k')),
        normal.clone(),
        Action::new("move_up", "Move cursor up", WmAction::Focus(Direction::Up)),
        false,
    );
    bs.add(
        KeyCombo::new(Key::Letter('l')),
        normal.clone(),
        Action::new(
            "move_right",
            "Move cursor right",
            WmAction::Focus(Direction::Right),
        ),
        false,
    );
    bs.add(
        KeyCombo::new(Key::Letter('i')),
        normal.clone(),
        Action::new(
            "enter_insert",
            "Enter insert mode",
            WmAction::Submap(SubmapTarget::Enter(ModeId::new("insert"))),
        ),
        false,
    );

    // Insert mode: Escape returns to normal
    bs.add(
        KeyCombo::new(Key::Named(NamedKey::Escape)),
        insert,
        Action::new(
            "exit_insert",
            "Return to normal mode",
            WmAction::Submap(SubmapTarget::Reset),
        ),
        false,
    );

    bs
}

/// CUA/Windows-style keybindings — standard PC shortcuts.
///
/// Source: IBM CUA specification (1987), Microsoft Windows UX Guidelines
pub fn cua_preset() -> BindingSet {
    let mut bs = BindingSet::new("cua");
    let app = ModeId::new("app");

    let binds = [
        ('c', "copy", "Copy", WmAction::Exec("wl-copy".to_string())),
        (
            'v',
            "paste",
            "Paste",
            WmAction::Exec("wl-paste".to_string()),
        ),
        ('x', "cut", "Cut", WmAction::Exec("wl-copy".to_string())),
        ('z', "undo", "Undo", WmAction::Exec("undo".to_string())),
        ('s', "save", "Save", WmAction::Exec("save".to_string())),
        (
            'a',
            "select_all",
            "Select all",
            WmAction::Exec("select-all".to_string()),
        ),
        ('f', "find", "Find", WmAction::Exec("find".to_string())),
        (
            'n',
            "new_window",
            "New window",
            WmAction::Exec("new-window".to_string()),
        ),
        ('o', "open", "Open file", WmAction::Exec("open".to_string())),
        ('p', "print", "Print", WmAction::Exec("print".to_string())),
        ('w', "close_tab", "Close tab", WmAction::Close),
        (
            't',
            "new_tab",
            "New tab",
            WmAction::Exec("new-tab".to_string()),
        ),
    ];
    for (c, name, desc, action) in binds {
        bs.add(
            KeyCombo::new(Key::Letter(c)).with_mod(Modifier::Ctrl),
            app.clone(),
            Action::new(name, desc, action),
            false,
        );
    }

    // Alt+F4 = quit
    bs.add(
        KeyCombo::new(Key::Function(4)).with_mod(Modifier::Alt),
        app.clone(),
        Action::new("quit", "Quit application", WmAction::Close),
        false,
    );
    // Alt+Tab = switch window
    bs.add(
        KeyCombo::new(Key::Named(NamedKey::Tab)).with_mod(Modifier::Alt),
        app,
        Action::new(
            "switch_window",
            "Switch window",
            WmAction::CycleWindow(Cycle::Forward),
        ),
        false,
    );

    bs
}

/// emacs-style keybindings — Ctrl/Meta prefix navigation.
///
/// Source: GNU Emacs Manual, readline conventions
pub fn emacs_preset() -> BindingSet {
    let mut bs = BindingSet::new("emacs");
    let app = ModeId::new("app");

    // C-a/e/k/y — line editing (readline)
    bs.add(
        KeyCombo::new(Key::Letter('a')).with_mod(Modifier::Ctrl),
        app.clone(),
        Action::new(
            "line_start",
            "Beginning of line",
            WmAction::Exec("line-start".to_string()),
        ),
        false,
    );
    bs.add(
        KeyCombo::new(Key::Letter('e')).with_mod(Modifier::Ctrl),
        app.clone(),
        Action::new(
            "line_end",
            "End of line",
            WmAction::Exec("line-end".to_string()),
        ),
        false,
    );
    bs.add(
        KeyCombo::new(Key::Letter('k')).with_mod(Modifier::Ctrl),
        app.clone(),
        Action::new(
            "kill_line",
            "Kill to end of line",
            WmAction::Exec("kill-line".to_string()),
        ),
        false,
    );
    bs.add(
        KeyCombo::new(Key::Letter('y')).with_mod(Modifier::Ctrl),
        app.clone(),
        Action::new("yank", "Yank (paste)", WmAction::Exec("yank".to_string())),
        false,
    );

    // C-f/b/n/p — character/line movement
    bs.add(
        KeyCombo::new(Key::Letter('f')).with_mod(Modifier::Ctrl),
        app.clone(),
        Action::new(
            "forward_char",
            "Forward one character",
            WmAction::Focus(Direction::Right),
        ),
        false,
    );
    bs.add(
        KeyCombo::new(Key::Letter('b')).with_mod(Modifier::Ctrl),
        app.clone(),
        Action::new(
            "backward_char",
            "Backward one character",
            WmAction::Focus(Direction::Left),
        ),
        false,
    );
    bs.add(
        KeyCombo::new(Key::Letter('n')).with_mod(Modifier::Ctrl),
        app.clone(),
        Action::new("next_line", "Next line", WmAction::Focus(Direction::Down)),
        false,
    );
    bs.add(
        KeyCombo::new(Key::Letter('p')).with_mod(Modifier::Ctrl),
        app.clone(),
        Action::new("prev_line", "Previous line", WmAction::Focus(Direction::Up)),
        false,
    );

    // M-f/b — word movement
    bs.add(
        KeyCombo::new(Key::Letter('f')).with_mod(Modifier::Alt),
        app.clone(),
        Action::new(
            "forward_word",
            "Forward one word",
            WmAction::Exec("forward-word".to_string()),
        ),
        false,
    );
    bs.add(
        KeyCombo::new(Key::Letter('b')).with_mod(Modifier::Alt),
        app.clone(),
        Action::new(
            "backward_word",
            "Backward one word",
            WmAction::Exec("backward-word".to_string()),
        ),
        false,
    );

    // C-g — cancel
    bs.add(
        KeyCombo::new(Key::Letter('g')).with_mod(Modifier::Ctrl),
        app.clone(),
        Action::new(
            "cancel",
            "Cancel / keyboard quit",
            WmAction::Submap(SubmapTarget::Reset),
        ),
        false,
    );

    // C-s/r — search
    bs.add(
        KeyCombo::new(Key::Letter('s')).with_mod(Modifier::Ctrl),
        app.clone(),
        Action::new(
            "search_forward",
            "Incremental search forward",
            WmAction::Exec("search-forward".to_string()),
        ),
        false,
    );
    bs.add(
        KeyCombo::new(Key::Letter('r')).with_mod(Modifier::Ctrl),
        app,
        Action::new(
            "search_backward",
            "Incremental search backward",
            WmAction::Exec("search-backward".to_string()),
        ),
        false,
    );

    bs
}

/// i3/sway tiling WM keybindings — Super + key for WM actions.
///
/// Source: i3 User's Guide, sway(5) man page
pub fn i3_preset() -> BindingSet {
    let mut bs = BindingSet::new("i3");
    let app = ModeId::new("app");
    let resize = ModeId::new("resize");

    // Window management
    bs.add(
        KeyCombo::new(Key::Named(NamedKey::Enter)).with_mod(Modifier::Super),
        app.clone(),
        Action::new(
            "terminal",
            "Launch terminal",
            WmAction::Exec("$TERMINAL".to_string()),
        ),
        false,
    );
    bs.add(
        KeyCombo::new(Key::Letter('d')).with_mod(Modifier::Super),
        app.clone(),
        Action::new(
            "launcher",
            "Application launcher",
            WmAction::Exec("walker".to_string()),
        ),
        false,
    );
    bs.add(
        KeyCombo::new(Key::Letter('q'))
            .with_mod(Modifier::Super)
            .with_mod(Modifier::Shift),
        app.clone(),
        Action::new("kill", "Kill focused window", WmAction::Close),
        false,
    );
    bs.add(
        KeyCombo::new(Key::Letter('f')).with_mod(Modifier::Super),
        app.clone(),
        Action::new("fullscreen", "Toggle fullscreen", WmAction::fullscreen()),
        false,
    );

    // Focus: Super+hjkl
    for (c, direction, desc) in [
        ('h', Direction::Left, "left"),
        ('j', Direction::Down, "down"),
        ('k', Direction::Up, "up"),
        ('l', Direction::Right, "right"),
    ] {
        bs.add(
            KeyCombo::new(Key::Letter(c)).with_mod(Modifier::Super),
            app.clone(),
            Action::new(
                format!("focus_{desc}"),
                format!("Focus {desc}"),
                WmAction::Focus(direction),
            ),
            false,
        );
    }

    // Move: Super+Shift+hjkl
    for (c, direction, desc) in [
        ('h', Direction::Left, "left"),
        ('j', Direction::Down, "down"),
        ('k', Direction::Up, "up"),
        ('l', Direction::Right, "right"),
    ] {
        bs.add(
            KeyCombo::new(Key::Letter(c))
                .with_mod(Modifier::Super)
                .with_mod(Modifier::Shift),
            app.clone(),
            Action::new(
                format!("move_{desc}"),
                format!("Move window {desc}"),
                WmAction::MoveWindow(direction),
            ),
            false,
        );
    }

    // Workspaces: Super+1-9
    for i in 1u8..=9 {
        bs.add(
            KeyCombo::new(Key::Number(i)).with_mod(Modifier::Super),
            app.clone(),
            Action::new(
                format!("workspace_{i}"),
                format!("Workspace {i}"),
                WmAction::Workspace(WorkspaceTarget::Index(i)),
            ),
            false,
        );
    }

    // Move to workspace: Super+Shift+1-9
    for i in 1u8..=9 {
        bs.add(
            KeyCombo::new(Key::Number(i))
                .with_mod(Modifier::Super)
                .with_mod(Modifier::Shift),
            app.clone(),
            Action::new(
                format!("move_to_{i}"),
                format!("Move to workspace {i}"),
                WmAction::MoveToWorkspace(WorkspaceTarget::Index(i), Follow::Follow),
            ),
            false,
        );
    }

    // Layout
    bs.add(
        KeyCombo::new(Key::Letter('v')).with_mod(Modifier::Super),
        app.clone(),
        Action::new(
            "split_v",
            "Split vertical",
            WmAction::Split(Orientation::Vertical),
        ),
        false,
    );
    bs.add(
        KeyCombo::new(Key::Named(NamedKey::Space))
            .with_mod(Modifier::Super)
            .with_mod(Modifier::Shift),
        app.clone(),
        Action::new("float", "Toggle floating", WmAction::toggle_float()),
        false,
    );

    // Resize mode
    bs.add(
        KeyCombo::new(Key::Letter('r')).with_mod(Modifier::Super),
        app,
        Action::new(
            "enter_resize",
            "Enter resize mode",
            WmAction::Submap(SubmapTarget::Enter(ModeId::new("resize"))),
        ),
        false,
    );

    // Resize mode bindings
    for (c, direction, desc) in [
        ('h', Direction::Left, "Shrink width"),
        ('j', Direction::Down, "Grow height"),
        ('k', Direction::Up, "Shrink height"),
        ('l', Direction::Right, "Grow width"),
    ] {
        bs.add(
            KeyCombo::new(Key::Letter(c)),
            resize.clone(),
            Action::new(format!("resize_{c}"), desc, WmAction::Resize(direction, 30)),
            true,
        );
    }
    bs.add(
        KeyCombo::new(Key::Named(NamedKey::Escape)),
        resize,
        Action::new(
            "exit_resize",
            "Exit resize mode",
            WmAction::Submap(SubmapTarget::Reset),
        ),
        false,
    );

    bs
}

/// tmux-style keybindings — prefix key (Ctrl+B) then action key.
///
/// Source: tmux(1) man page, vi copy mode conventions
pub fn tmux_preset() -> BindingSet {
    let mut bs = BindingSet::new("tmux");
    let prefix = ModeId::new("tmux-prefix");

    // Window management
    bs.add(
        KeyCombo::new(Key::Letter('c')),
        prefix.clone(),
        Action::new(
            "new_window",
            "Create new window",
            WmAction::Exec("tmux new-window".to_string()),
        ),
        false,
    );
    bs.add(
        KeyCombo::new(Key::Letter('n')),
        prefix.clone(),
        Action::new(
            "next_window",
            "Next window",
            WmAction::Exec("tmux next-window".to_string()),
        ),
        false,
    );
    bs.add(
        KeyCombo::new(Key::Letter('p')),
        prefix.clone(),
        Action::new(
            "prev_window",
            "Previous window",
            WmAction::Exec("tmux previous-window".to_string()),
        ),
        false,
    );
    bs.add(
        KeyCombo::new(Key::Letter('l')),
        prefix.clone(),
        Action::new(
            "last_window",
            "Last window",
            WmAction::Exec("tmux last-window".to_string()),
        ),
        false,
    );

    // Window numbers
    for i in 0u8..=9 {
        bs.add(
            KeyCombo::new(Key::Number(i)),
            prefix.clone(),
            Action::new(
                format!("window_{i}"),
                format!("Switch to window {i}"),
                WmAction::Exec(format!("tmux select-window -t {i}")),
            ),
            false,
        );
    }

    // Pane splitting
    bs.add(
        KeyCombo::new(Key::Letter('%')),
        prefix.clone(),
        Action::new(
            "split_v",
            "Split pane vertical",
            WmAction::Exec("tmux split-window -h".to_string()),
        ),
        false,
    );
    bs.add(
        KeyCombo::new(Key::Letter('"')),
        prefix.clone(),
        Action::new(
            "split_h",
            "Split pane horizontal",
            WmAction::Exec("tmux split-window -v".to_string()),
        ),
        false,
    );

    // Pane navigation (arrows — hjkl conflicts with window commands in default tmux)
    for (key, dir, name) in [
        (NamedKey::Left, "L", "left"),
        (NamedKey::Down, "D", "down"),
        (NamedKey::Up, "U", "up"),
        (NamedKey::Right, "R", "right"),
    ] {
        bs.add(
            KeyCombo::new(Key::Named(key)),
            prefix.clone(),
            Action::new(
                format!("pane_{name}"),
                format!("Focus pane {name}"),
                WmAction::Exec(format!("tmux select-pane -{dir}")),
            ),
            false,
        );
    }

    // Session/pane management
    bs.add(
        KeyCombo::new(Key::Letter('d')),
        prefix.clone(),
        Action::new(
            "detach",
            "Detach session",
            WmAction::Exec("tmux detach".to_string()),
        ),
        false,
    );
    bs.add(
        KeyCombo::new(Key::Letter('z')),
        prefix.clone(),
        Action::new(
            "zoom",
            "Toggle pane zoom",
            WmAction::Exec("tmux resize-pane -Z".to_string()),
        ),
        false,
    );
    bs.add(
        KeyCombo::new(Key::Letter('x')),
        prefix.clone(),
        Action::new(
            "kill_pane",
            "Kill pane",
            WmAction::Exec("tmux kill-pane".to_string()),
        ),
        false,
    );
    bs.add(
        KeyCombo::new(Key::Letter('?')),
        prefix,
        Action::new(
            "help",
            "List keybindings",
            WmAction::Exec("tmux list-keys".to_string()),
        ),
        false,
    );

    bs
}

// ── Axioms ──

/// No binding conflicts: same key combo in the same mode must not have two actions.
pub struct NoConflicts {
    pub bindings: BindingSet,
}

impl Axiom for NoConflicts {
    fn verify(&self) -> Verdict {
        if self.bindings.conflicts().is_empty() {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "NoConflicts",
        "no duplicate key combos in the same mode",
        "Harel (1987) Statecharts: A Visual Formalism, Science of Computer Programming 8"
    );
}

/// Remap is injective: each `from` maps to exactly one `to`.
pub struct RemapInjective {
    pub remaps: RemapSet,
}

impl Axiom for RemapInjective {
    fn verify(&self) -> Verdict {
        let froms: Vec<&KeyCombo> = self.remaps.remaps.iter().map(|r| &r.from).collect();
        let unique: HashSet<&KeyCombo> = froms.iter().copied().collect();
        if froms.len() == unique.len() {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "RemapInjective",
        "remap is injective (each source maps to one target)",
        "Beaudouin-Lafon (2000) Instrumental Interaction, CHI"
    );
}

/// Every mode in the binding set has at least one binding.
pub struct AllModesHaveBindings {
    pub bindings: BindingSet,
    pub modes: Vec<ModeId>,
}

impl Axiom for AllModesHaveBindings {
    fn verify(&self) -> Verdict {
        let ok = self
            .modes
            .iter()
            .all(|m| !self.bindings.for_mode(m).is_empty());
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AllModesHaveBindings",
        "every mode has at least one keybinding",
        "Harel (1987) Statecharts: A Visual Formalism — mode-scoped behaviour requires bindings"
    );
}

/// Super→Ctrl remap covers all standard shortcuts (copy, paste, save, etc).
pub struct MacosRemapComplete {
    pub remaps: RemapSet,
}

impl Axiom for MacosRemapComplete {
    fn verify(&self) -> Verdict {
        let essential = ['c', 'v', 'x', 'z', 's', 'a'];
        let ok = essential.iter().all(|&c| {
            let combo = KeyCombo::new(Key::Letter(c)).with_mod(Modifier::Super);
            self.remaps.apply(&combo).is_some()
        });
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MacosRemapComplete",
        "macOS remap covers essential shortcuts (C, V, X, Z, S, A)",
        "IBM (1987) Common User Access Specification; macOS Human Interface Guidelines"
    );
}

// ── Desktop / WM-navigation presets ───────────────────────────────────────
// The window-management navigation flavour of each desktop paradigm as a flat
// `BindingSet` (the consumer supplies the per-paradigm mode topology). Commands
// are Hyprland dispatchers. Lifted from vogix's input catalog so praxis is the
// single source of every paradigm's bindings.
use Modifier::{Alt, Ctrl, Shift, Super};

/// Build a [`KeyCombo`] from modifiers + a key.
fn combo(mods: &[Modifier], key: Key) -> KeyCombo {
    let mut c = KeyCombo::new(key);
    for m in mods {
        c = c.with_mod(*m);
    }
    c
}

/// The four cardinal directions as `(letter, arrow, hypr-suffix)`. The `vogix`
/// layout binds BOTH `hjkl` and the arrows to the same action (h=left, l=right,
/// j=up, k=down — a non-vim mapping), so every nav verb is generated for both.
const DIRS: &[(char, NamedKey, &str, Direction)] = &[
    ('h', NamedKey::Left, "l", Direction::Left),
    ('l', NamedKey::Right, "r", Direction::Right),
    ('j', NamedKey::Up, "u", Direction::Up),
    ('k', NamedKey::Down, "d", Direction::Down),
];

/// `vogix` — the house default: a flat `Super`-combos WM-navigation layout in one
/// `app` mode (focus/move/resize, workspaces 1–10, send-to-workspace). Both `hjkl`
/// and the arrow keys drive focus/move/resize.
pub fn vogix_preset() -> BindingSet {
    let mut bs = BindingSet::new("vogix");
    let app = ModeId::new("app");
    let mut add =
        |mods: &[Modifier], key: Key, name: &str, desc: &str, actions: ActionWord, repeat: bool| {
            bs.add(
                combo(mods, key),
                app.clone(),
                Action::new(name, desc, actions),
                repeat,
            );
        };

    // Focus (Super + dir -> movefocus); hjkl AND arrows.
    for (letter, arrow, suf, direction) in DIRS {
        add(
            &[Super],
            Key::Letter(*letter),
            &format!("focus_{suf}"),
            "Focus",
            WmAction::Focus(*direction).into(),
            false,
        );
        add(
            &[Super],
            Key::Named(arrow.clone()),
            &format!("focus_{suf}_arrow"),
            "Focus",
            WmAction::Focus(*direction).into(),
            false,
        );
    }
    // Move window (Super + Shift + dir -> swapwindow).
    for (letter, arrow, suf, direction) in DIRS {
        add(
            &[Super, Shift],
            Key::Letter(*letter),
            &format!("move_{suf}"),
            "Move window",
            WmAction::SwapWindow(*direction).into(),
            false,
        );
        add(
            &[Super, Shift],
            Key::Named(arrow.clone()),
            &format!("move_{suf}_arrow"),
            "Move window",
            WmAction::SwapWindow(*direction).into(),
            false,
        );
    }
    // Resize window (Ctrl + Shift + dir -> resizeactive; repeats). The pixel
    // deltas are derived from the direction by the realization functor.
    for (letter, arrow, suf, direction) in DIRS {
        add(
            &[Ctrl, Shift],
            Key::Letter(*letter),
            &format!("resize_{suf}"),
            "Resize",
            WmAction::Resize(*direction, 30).into(),
            true,
        );
        add(
            &[Ctrl, Shift],
            Key::Named(arrow.clone()),
            &format!("resize_{suf}_arrow"),
            "Resize",
            WmAction::Resize(*direction, 30).into(),
            true,
        );
    }

    // Window state.
    add(
        &[Super],
        Key::Letter('q'),
        "close",
        "Close window",
        WmAction::Close.into(),
        false,
    );
    // Float + pin is a genuine TWO-action composite — not a shell hack. The
    // realization functor batches [ToggleFloat, Pin] into one exec.
    add(
        &[Super],
        Key::Letter('y'),
        "float_pin",
        "Float + pin",
        vec![WmAction::toggle_float(), WmAction::pin()].into(),
        false,
    );
    add(
        &[Super],
        Key::Letter('f'),
        "fullscreen",
        "Fullscreen",
        WmAction::fullscreen().into(),
        false,
    );
    add(
        &[Super],
        Key::Letter('p'),
        "pseudo",
        "Pseudotile",
        WmAction::pseudotile().into(),
        false,
    );
    add(
        &[Super],
        Key::Letter('o'),
        "toggle_split",
        "Toggle split",
        WmAction::toggle_split().into(),
        false,
    );
    add(
        &[Super],
        Key::Letter('u'),
        "toggle_group",
        "Toggle group",
        WmAction::ToggleGroup.into(),
        false,
    );
    add(
        &[Super],
        Key::Named(NamedKey::Tab),
        "group_cycle",
        "Cycle window in group",
        WmAction::CycleGroup(Cycle::Forward).into(),
        false,
    );

    // Workspaces (Super + number; 0 = ws 10).
    for n in 1u8..=10 {
        let key = Key::Number(if n == 10 { 0 } else { n });
        add(
            &[Super],
            key,
            &format!("workspace_{n}"),
            "Workspace",
            WmAction::Workspace(WorkspaceTarget::Index(n)).into(),
            false,
        );
    }
    add(
        &[Super],
        Key::Letter('m'),
        "workspace_music",
        "Music workspace",
        WmAction::Workspace(WorkspaceTarget::Named("Music".to_string())).into(),
        false,
    );

    // Adjacent workspace (Super + Ctrl + arrows or j/l).
    add(
        &[Super, Ctrl],
        Key::Named(NamedKey::Left),
        "ws_prev",
        "Previous workspace",
        WmAction::Workspace(WorkspaceTarget::Relative(-1)).into(),
        false,
    );
    add(
        &[Super, Ctrl],
        Key::Named(NamedKey::Right),
        "ws_next",
        "Next workspace",
        WmAction::Workspace(WorkspaceTarget::Relative(1)).into(),
        false,
    );
    add(
        &[Super, Ctrl],
        Key::Letter('j'),
        "ws_prev_j",
        "Previous workspace",
        WmAction::Workspace(WorkspaceTarget::Relative(-1)).into(),
        false,
    );
    add(
        &[Super, Ctrl],
        Key::Letter('l'),
        "ws_next_l",
        "Next workspace",
        WmAction::Workspace(WorkspaceTarget::Relative(1)).into(),
        false,
    );

    // Send window to workspace (Super + Ctrl + number).
    for n in 1u8..=10 {
        let key = Key::Number(if n == 10 { 0 } else { n });
        add(
            &[Super, Ctrl],
            key,
            &format!("move_to_ws_{n}"),
            "Send window to workspace",
            WmAction::MoveToWorkspace(WorkspaceTarget::Index(n), Follow::Follow).into(),
            false,
        );
    }
    // Send window to adjacent workspace (Super + Ctrl + Shift + arrows or j/l).
    add(
        &[Super, Ctrl, Shift],
        Key::Named(NamedKey::Left),
        "send_ws_prev",
        "Send window \u{2190} workspace",
        WmAction::MoveToWorkspace(WorkspaceTarget::Relative(-1), Follow::Follow).into(),
        false,
    );
    add(
        &[Super, Ctrl, Shift],
        Key::Named(NamedKey::Right),
        "send_ws_next",
        "Send window \u{2192} workspace",
        WmAction::MoveToWorkspace(WorkspaceTarget::Relative(1), Follow::Follow).into(),
        false,
    );
    add(
        &[Super, Ctrl, Shift],
        Key::Letter('j'),
        "send_ws_prev_j",
        "Send window \u{2190} workspace",
        WmAction::MoveToWorkspace(WorkspaceTarget::Relative(-1), Follow::Follow).into(),
        false,
    );
    add(
        &[Super, Ctrl, Shift],
        Key::Letter('l'),
        "send_ws_next_l",
        "Send window \u{2192} workspace",
        WmAction::MoveToWorkspace(WorkspaceTarget::Relative(1), Follow::Follow).into(),
        false,
    );

    // Send window to workspace silently (Super + Shift + number).
    for n in 1u8..=10 {
        let key = Key::Number(if n == 10 { 0 } else { n });
        add(
            &[Super, Shift],
            key,
            &format!("move_silent_{n}"),
            "Send window to workspace (silent)",
            WmAction::MoveToWorkspace(WorkspaceTarget::Index(n), Follow::Silent).into(),
            false,
        );
    }

    bs
}

/// `windows` — Microsoft Windows 11 global window & virtual-desktop conventions,
/// projected to Hyprland. Single passthrough `app` mode, no remap (Windows uses
/// Ctrl natively). Snap/maximize are adapted to the nearest real dispatcher.
///
/// Source: Microsoft Support, "Keyboard shortcuts in Windows".
pub fn windows_preset() -> BindingSet {
    let mut bs = BindingSet::new("windows");
    let app = ModeId::new("app");
    let mut add = |mods: &[Modifier], key: Key, name: &str, desc: &str, action: WmAction| {
        bs.add(
            combo(mods, key),
            app.clone(),
            Action::new(name, desc, action),
            false,
        );
    };

    // Window switch (Alt+Tab) + close (Alt+F4) — faithful.
    add(
        &[Alt],
        Key::Named(NamedKey::Tab),
        "switch_window",
        "Switch window",
        WmAction::CycleWindow(Cycle::Forward),
    );
    add(
        &[Alt, Shift],
        Key::Named(NamedKey::Tab),
        "switch_window_prev",
        "Switch window (reverse)",
        WmAction::CycleWindow(Cycle::Backward),
    );
    add(
        &[Alt],
        Key::Function(4),
        "close",
        "Close window",
        WmAction::Close,
    );

    // Snap (Win+arrows) + maximize (Win+Up) — adapted. Win+Up maximizes (keeps
    // the bar): fullscreen mode 1, NOT true fullscreen.
    add(
        &[Super],
        Key::Named(NamedKey::Left),
        "snap_left",
        "Snap window left",
        WmAction::MoveWindow(Direction::Left),
    );
    add(
        &[Super],
        Key::Named(NamedKey::Right),
        "snap_right",
        "Snap window right",
        WmAction::MoveWindow(Direction::Right),
    );
    add(
        &[Super],
        Key::Named(NamedKey::Up),
        "maximize",
        "Maximize window",
        WmAction::maximize(),
    );

    // Move window (Win+Shift+arrows) — adapted to a directional move.
    add(
        &[Super, Shift],
        Key::Named(NamedKey::Left),
        "move_left",
        "Move window left",
        WmAction::MoveWindow(Direction::Left),
    );
    add(
        &[Super, Shift],
        Key::Named(NamedKey::Right),
        "move_right",
        "Move window right",
        WmAction::MoveWindow(Direction::Right),
    );
    add(
        &[Super, Shift],
        Key::Named(NamedKey::Up),
        "move_up",
        "Move window up",
        WmAction::MoveWindow(Direction::Up),
    );
    add(
        &[Super, Shift],
        Key::Named(NamedKey::Down),
        "move_down",
        "Move window down",
        WmAction::MoveWindow(Direction::Down),
    );

    // Virtual desktops: Ctrl+Win+arrows switch; Win+1..9 -> desktop N.
    add(
        &[Super, Ctrl],
        Key::Named(NamedKey::Left),
        "desktop_prev",
        "Previous virtual desktop",
        WmAction::Workspace(WorkspaceTarget::Relative(-1)),
    );
    add(
        &[Super, Ctrl],
        Key::Named(NamedKey::Right),
        "desktop_next",
        "Next virtual desktop",
        WmAction::Workspace(WorkspaceTarget::Relative(1)),
    );
    for n in 1u8..=9 {
        add(
            &[Super],
            Key::Number(n),
            &format!("workspace_{n}"),
            "Virtual desktop",
            WmAction::Workspace(WorkspaceTarget::Index(n)),
        );
    }
    bs
}

/// `macos` — Apple macOS Mission Control / Spaces / window conventions, projected
/// to Hyprland. Pairs with the `macos` remap (Cmd-feel); the window verbs
/// (Cmd+W/Q/H/M) are bound so they win over the remap.
///
/// Source: Apple Support, "Mac keyboard shortcuts" & "Use Mission Control".
pub fn macos_preset() -> BindingSet {
    let mut bs = BindingSet::new("macos");
    let app = ModeId::new("app");
    let mut add = |mods: &[Modifier], key: Key, name: &str, desc: &str, action: WmAction| {
        bs.add(
            combo(mods, key),
            app.clone(),
            Action::new(name, desc, action),
            false,
        );
    };

    // Spaces: Ctrl+arrows + Ctrl+1..9 (native Ctrl).
    add(
        &[Ctrl],
        Key::Named(NamedKey::Left),
        "workspace_prev",
        "Previous Space",
        WmAction::Workspace(WorkspaceTarget::Relative(-1)),
    );
    add(
        &[Ctrl],
        Key::Named(NamedKey::Right),
        "workspace_next",
        "Next Space",
        WmAction::Workspace(WorkspaceTarget::Relative(1)),
    );
    for n in 1u8..=9 {
        add(
            &[Ctrl],
            Key::Number(n),
            &format!("workspace_{n}"),
            "Space",
            WmAction::Workspace(WorkspaceTarget::Index(n)),
        );
    }
    // Mission Control (Ctrl+Up) — adapted to an `overview` special workspace.
    add(
        &[Ctrl],
        Key::Named(NamedKey::Up),
        "mission_control",
        "Mission Control",
        WmAction::ToggleSpecialWorkspace("overview".to_string()),
    );

    // Window switch (Cmd+Tab).
    add(
        &[Super],
        Key::Named(NamedKey::Tab),
        "switch_window",
        "Switch window",
        WmAction::CycleWindow(Cycle::Forward),
    );
    add(
        &[Super, Shift],
        Key::Named(NamedKey::Tab),
        "switch_window_prev",
        "Switch window (reverse)",
        WmAction::CycleWindow(Cycle::Backward),
    );

    // Window verbs (Cmd+W/Q/H/M) — bound, so they win over the remap. Hide and
    // "minimize" are silent moves to special workspaces, not a minimize state.
    add(
        &[Super],
        Key::Letter('w'),
        "close_window",
        "Close window",
        WmAction::Close,
    );
    add(
        &[Super],
        Key::Letter('q'),
        "quit",
        "Quit app",
        WmAction::Close,
    );
    add(
        &[Super],
        Key::Letter('h'),
        "hide",
        "Hide window",
        WmAction::MoveToWorkspace(
            WorkspaceTarget::Special("hidden".to_string()),
            Follow::Silent,
        ),
    );
    add(
        &[Super],
        Key::Letter('m'),
        "minimize",
        "Minimize window",
        WmAction::minimize(),
    );
    // Fullscreen (Ctrl+Cmd+F).
    add(
        &[Ctrl, Super],
        Key::Letter('f'),
        "fullscreen",
        "Toggle fullscreen",
        WmAction::fullscreen(),
    );
    bs
}

/// `linux` — mainstream GNOME Shell global window conventions, projected to
/// Hyprland. Single passthrough `app` mode, no remap. Tile/maximize/hide are
/// adapted to the nearest real dispatcher.
///
/// Source: GNOME Shell defaults (`org.gnome.desktop.wm.keybindings`).
pub fn linux_preset() -> BindingSet {
    let mut bs = BindingSet::new("linux");
    let app = ModeId::new("app");
    let mut add = |mods: &[Modifier], key: Key, name: &str, desc: &str, action: WmAction| {
        bs.add(
            combo(mods, key),
            app.clone(),
            Action::new(name, desc, action),
            false,
        );
    };

    // Workspaces: Super+PageUp/PageDown switch; Super+Shift+PageUp/Down move window.
    add(
        &[Super],
        Key::Named(NamedKey::PageUp),
        "workspace_prev",
        "Previous workspace",
        WmAction::Workspace(WorkspaceTarget::Relative(-1)),
    );
    add(
        &[Super],
        Key::Named(NamedKey::PageDown),
        "workspace_next",
        "Next workspace",
        WmAction::Workspace(WorkspaceTarget::Relative(1)),
    );
    add(
        &[Super, Shift],
        Key::Named(NamedKey::PageUp),
        "move_to_prev",
        "Move window \u{2190} workspace",
        WmAction::MoveToWorkspace(WorkspaceTarget::Relative(-1), Follow::Follow),
    );
    add(
        &[Super, Shift],
        Key::Named(NamedKey::PageDown),
        "move_to_next",
        "Move window \u{2192} workspace",
        WmAction::MoveToWorkspace(WorkspaceTarget::Relative(1), Follow::Follow),
    );

    // Window switch (Alt+Tab) + close (Alt+F4) — faithful.
    add(
        &[Alt],
        Key::Named(NamedKey::Tab),
        "switch_window",
        "Switch window",
        WmAction::CycleWindow(Cycle::Forward),
    );
    add(
        &[Alt],
        Key::Function(4),
        "kill",
        "Close window",
        WmAction::Close,
    );

    // Maximize (Super+Up) + tile (Super+arrows) + hide (Super+H) — adapted.
    // Super+Up maximizes (keeps the bar): fullscreen mode 1, not true fullscreen.
    add(
        &[Super],
        Key::Named(NamedKey::Up),
        "maximize",
        "Maximize window",
        WmAction::maximize(),
    );
    add(
        &[Super],
        Key::Named(NamedKey::Left),
        "tile_left",
        "Tile window left",
        WmAction::MoveWindow(Direction::Left),
    );
    add(
        &[Super],
        Key::Named(NamedKey::Right),
        "tile_right",
        "Tile window right",
        WmAction::MoveWindow(Direction::Right),
    );
    add(
        &[Super],
        Key::Letter('h'),
        "hide",
        "Hide window",
        WmAction::MoveToWorkspace(WorkspaceTarget::Special(String::new()), Follow::Silent),
    );
    bs
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── KeyCombo tests ──

    #[test]
    fn test_key_combo_display() {
        let combo = KeyCombo::new(Key::Letter('c')).with_mod(Modifier::Ctrl);
        assert_eq!(combo.display(), "Ctrl + C");
    }

    #[test]
    fn test_key_combo_multi_mod() {
        let combo = KeyCombo::new(Key::Letter('s'))
            .with_mod(Modifier::Ctrl)
            .with_mod(Modifier::Shift);
        assert_eq!(combo.display(), "Shift + Ctrl + S");
    }

    #[test]
    fn test_key_combo_no_duplicate_mods() {
        let combo = KeyCombo::new(Key::Letter('a'))
            .with_mod(Modifier::Ctrl)
            .with_mod(Modifier::Ctrl);
        assert_eq!(combo.modifiers.len(), 1);
    }

    #[test]
    fn test_key_combo_sorted_mods() {
        let combo = KeyCombo::new(Key::Letter('a'))
            .with_mod(Modifier::Super)
            .with_mod(Modifier::Alt)
            .with_mod(Modifier::Ctrl);
        // Should be sorted: Ctrl < Alt < Super (by Ord derive)
        assert_eq!(
            combo.modifiers,
            vec![Modifier::Ctrl, Modifier::Alt, Modifier::Super]
        );
    }

    // ── Preset tests ──

    #[test]
    fn test_macos_remap() {
        let rs = macos_remap();
        let copy = KeyCombo::new(Key::Letter('c')).with_mod(Modifier::Super);
        let result = rs.apply(&copy).unwrap();
        assert_eq!(result.modifiers, vec![Modifier::Ctrl]);
        assert_eq!(result.key, Key::Letter('c'));
    }

    #[test]
    fn test_macos_remap_complete() {
        let rs = macos_remap();
        assert!(MacosRemapComplete { remaps: rs }.verify().is_ok());
    }

    #[test]
    fn test_macos_remap_injective() {
        let rs = macos_remap();
        assert!(RemapInjective { remaps: rs }.verify().is_ok());
    }

    #[test]
    fn test_vim_preset_no_conflicts() {
        let bs = vim_preset();
        assert!(NoConflicts { bindings: bs }.verify().is_ok());
    }

    #[test]
    fn test_vim_preset_has_hjkl() {
        let bs = vim_preset();
        let normal = ModeId::new("normal");
        let normal_bindings = bs.for_mode(&normal);
        let keys: Vec<_> = normal_bindings.iter().map(|b| &b.combo.key).collect();
        assert!(keys.contains(&&Key::Letter('h')));
        assert!(keys.contains(&&Key::Letter('j')));
        assert!(keys.contains(&&Key::Letter('k')));
        assert!(keys.contains(&&Key::Letter('l')));
    }

    #[test]
    fn test_vim_preset_modes_have_bindings() {
        let bs = vim_preset();
        let modes = vec![ModeId::new("normal"), ModeId::new("insert")];
        assert!(
            AllModesHaveBindings {
                bindings: bs,
                modes
            }
            .verify()
            .is_ok()
        );
    }

    // ── Conflict detection ──

    #[test]
    fn test_conflict_detected() {
        let mut bs = BindingSet::new("conflicting");
        let mode = ModeId::new("test");
        let combo = KeyCombo::new(Key::Letter('a')).with_mod(Modifier::Ctrl);
        bs.add(
            combo.clone(),
            mode.clone(),
            Action::new("a1", "Action 1", WmAction::Close),
            false,
        );
        bs.add(
            combo,
            mode,
            Action::new("a2", "Action 2", WmAction::Close),
            false,
        );
        assert!(
            NoConflicts {
                bindings: bs.clone()
            }
            .verify()
            .is_err()
        );
        assert_eq!(bs.conflicts().len(), 1);
    }

    #[test]
    fn test_same_key_different_mode_no_conflict() {
        let mut bs = BindingSet::new("multi-mode");
        let combo = KeyCombo::new(Key::Letter('a'));
        bs.add(
            combo.clone(),
            ModeId::new("mode1"),
            Action::new("a1", "Action 1", WmAction::Close),
            false,
        );
        bs.add(
            combo,
            ModeId::new("mode2"),
            Action::new("a2", "Action 2", WmAction::Close),
            false,
        );
        assert!(NoConflicts { bindings: bs }.verify().is_ok());
    }

    // ── CUA preset ──

    #[test]
    fn test_cua_preset_no_conflicts() {
        assert!(
            NoConflicts {
                bindings: cua_preset()
            }
            .verify()
            .is_ok()
        );
    }

    #[test]
    fn test_cua_has_copy_paste() {
        let bs = cua_preset();
        let app = ModeId::new("app");
        let bindings = bs.for_mode(&app);
        let names: Vec<_> = bindings.iter().map(|b| b.action.name.as_str()).collect();
        assert!(names.contains(&"copy"));
        assert!(names.contains(&"paste"));
        assert!(names.contains(&"cut"));
    }

    // ── emacs preset ──

    #[test]
    fn test_emacs_preset_no_conflicts() {
        assert!(
            NoConflicts {
                bindings: emacs_preset()
            }
            .verify()
            .is_ok()
        );
    }

    #[test]
    fn test_emacs_has_readline() {
        let bs = emacs_preset();
        let app = ModeId::new("app");
        let bindings = bs.for_mode(&app);
        let names: Vec<_> = bindings.iter().map(|b| b.action.name.as_str()).collect();
        assert!(names.contains(&"line_start")); // C-a
        assert!(names.contains(&"line_end")); // C-e
        assert!(names.contains(&"kill_line")); // C-k
        assert!(names.contains(&"yank")); // C-y
    }

    // ── i3 preset ──

    #[test]
    fn test_i3_preset_no_conflicts() {
        assert!(
            NoConflicts {
                bindings: i3_preset()
            }
            .verify()
            .is_ok()
        );
    }

    #[test]
    fn test_i3_has_workspaces() {
        let bs = i3_preset();
        let app = ModeId::new("app");
        let bindings = bs.for_mode(&app);
        let names: Vec<_> = bindings.iter().map(|b| b.action.name.as_str()).collect();
        for i in 1..=9 {
            assert!(
                names.contains(&format!("workspace_{i}").as_str()),
                "missing workspace {i}"
            );
        }
    }

    #[test]
    fn test_i3_has_hjkl_focus() {
        let bs = i3_preset();
        let app = ModeId::new("app");
        let bindings = bs.for_mode(&app);
        let names: Vec<_> = bindings.iter().map(|b| b.action.name.as_str()).collect();
        assert!(names.contains(&"focus_left"));
        assert!(names.contains(&"focus_right"));
        assert!(names.contains(&"focus_up"));
        assert!(names.contains(&"focus_down"));
    }

    #[test]
    fn test_i3_resize_mode() {
        let bs = i3_preset();
        let resize = ModeId::new("resize");
        let bindings = bs.for_mode(&resize);
        assert!(bindings.len() >= 5, "resize mode should have hjkl + escape");
    }

    // ── tmux preset ──

    #[test]
    fn test_tmux_preset_no_conflicts() {
        assert!(
            NoConflicts {
                bindings: tmux_preset()
            }
            .verify()
            .is_ok()
        );
    }

    #[test]
    fn test_tmux_has_window_management() {
        let bs = tmux_preset();
        let prefix = ModeId::new("tmux-prefix");
        let bindings = bs.for_mode(&prefix);
        let names: Vec<_> = bindings.iter().map(|b| b.action.name.as_str()).collect();
        assert!(names.contains(&"new_window"));
        assert!(names.contains(&"next_window"));
        assert!(names.contains(&"prev_window"));
        assert!(names.contains(&"detach"));
    }

    #[test]
    fn test_tmux_has_pane_navigation() {
        let bs = tmux_preset();
        let prefix = ModeId::new("tmux-prefix");
        let bindings = bs.for_mode(&prefix);
        let names: Vec<_> = bindings.iter().map(|b| b.action.name.as_str()).collect();
        assert!(names.contains(&"pane_left"));
        assert!(names.contains(&"pane_down"));
        assert!(names.contains(&"pane_up"));
        assert!(names.contains(&"pane_right"));
    }

    // ── Cross-preset tests ──

    #[test]
    fn test_all_presets_no_conflicts() {
        for (name, bs) in [
            ("vim", vim_preset()),
            ("cua", cua_preset()),
            ("emacs", emacs_preset()),
            ("i3", i3_preset()),
            ("tmux", tmux_preset()),
            ("vogix", vogix_preset()),
            ("windows", windows_preset()),
            ("macos", macos_preset()),
            ("linux", linux_preset()),
        ] {
            let axiom = NoConflicts { bindings: bs };
            assert!(axiom.verify().is_ok(), "{name} preset has conflicts");
        }
    }

    #[test]
    fn test_vogix_preset_has_focus_and_workspaces() {
        let bs = vogix_preset();
        let app = ModeId::new("app");
        let names: Vec<_> = bs
            .for_mode(&app)
            .iter()
            .map(|b| b.action.name.clone())
            .collect();
        // Focus for both hjkl and arrows, plus all 10 workspaces.
        assert!(names.iter().any(|n| n == "focus_l"));
        assert!(names.iter().any(|n| n == "focus_l_arrow"));
        for i in 1..=10 {
            assert!(
                names.iter().any(|n| n == &format!("workspace_{i}")),
                "vogix missing workspace_{i}"
            );
        }
    }

    #[test]
    fn vogix_commands_realize_to_expected_dispatchers() {
        let bs = vogix_preset();
        let app = ModeId::new("app");
        let cmd = |n: &str| {
            bs.for_mode(&app)
                .into_iter()
                .find(|b| b.action.name == n)
                .map(|b| b.action.command())
        };
        assert_eq!(cmd("focus_l").as_deref(), Some("movefocus, l"));
        assert_eq!(cmd("move_u").as_deref(), Some("swapwindow, u"));
        assert_eq!(cmd("resize_l").as_deref(), Some("resizeactive, -30 0"));
        assert_eq!(cmd("fullscreen").as_deref(), Some("fullscreen"));
        assert_eq!(cmd("workspace_10").as_deref(), Some("workspace, 10"));
        assert_eq!(cmd("workspace_music").as_deref(), Some("workspace, Music"));
        assert_eq!(cmd("ws_next").as_deref(), Some("workspace, +1"));
        assert_eq!(
            cmd("move_silent_1").as_deref(),
            Some("movetoworkspacesilent, 1")
        );
        // The float+pin composite batches into one exec — derived, not hand-written.
        assert_eq!(
            cmd("float_pin").as_deref(),
            Some("exec, hyprctl dispatch togglefloating ; hyprctl dispatch pin")
        );
    }

    #[test]
    fn desktop_maximize_realizes_to_fullscreen_mode_one() {
        // The maximize-vs-fullscreen fix at the binding level: windows/linux
        // "maximize" => fullscreen mode 1 (keep the bar), not true fullscreen.
        for (label, bs, name) in [
            ("windows", windows_preset(), "maximize"),
            ("linux", linux_preset(), "maximize"),
        ] {
            let app = ModeId::new("app");
            let max = bs
                .for_mode(&app)
                .into_iter()
                .find(|b| b.action.name == name)
                .map(|b| b.action.command());
            assert_eq!(
                max.as_deref(),
                Some("fullscreen, 1"),
                "{label} maximize should realize to fullscreen, 1"
            );
        }
    }

    #[test]
    fn test_desktop_presets_have_window_switch() {
        // Each desktop paradigm binds a window-switch verb.
        for (name, bs) in [
            ("windows", windows_preset()),
            ("macos", macos_preset()),
            ("linux", linux_preset()),
        ] {
            let app = ModeId::new("app");
            let has_switch = bs
                .for_mode(&app)
                .iter()
                .any(|b| b.action.name == "switch_window");
            assert!(has_switch, "{name} preset missing switch_window");
        }
    }

    // ── Property-based tests ──
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_macos_remap_always_ctrl(idx in 0u8..26) {
            let c = (b'a' + idx) as char;
            let rs = macos_remap();
            let combo = KeyCombo::new(Key::Letter(c)).with_mod(Modifier::Super);
            if let Some(result) = rs.apply(&combo) {
                prop_assert!(result.modifiers == vec![Modifier::Ctrl]);
                prop_assert!(result.key == Key::Letter(c));
            }
        }

        #[test]
        fn prop_remap_preserves_key(idx in 0u8..26) {
            let c = (b'a' + idx) as char;
            let rs = macos_remap();
            let combo = KeyCombo::new(Key::Letter(c)).with_mod(Modifier::Super);
            if let Some(result) = rs.apply(&combo) {
                prop_assert!(result.key == combo.key);
            }
        }

        #[test]
        fn prop_display_contains_key(idx in 0u8..26) {
            let c = (b'a' + idx) as char;
            let combo = KeyCombo::new(Key::Letter(c));
            let display = combo.display();
            prop_assert!(display.contains(c.to_uppercase().next().unwrap()));
        }

        #[test]
        fn prop_no_duplicate_mods_after_double_add(idx in 0u8..26) {
            let c = (b'a' + idx) as char;
            let combo = KeyCombo::new(Key::Letter(c))
                .with_mod(Modifier::Ctrl)
                .with_mod(Modifier::Ctrl)
                .with_mod(Modifier::Ctrl);
            prop_assert_eq!(combo.modifiers.len(), 1);
        }

        #[test]
        fn prop_mods_always_sorted(idx in 0u8..26) {
            let c = (b'a' + idx) as char;
            let combo = KeyCombo::new(Key::Letter(c))
                .with_mod(Modifier::Hyper)
                .with_mod(Modifier::Shift)
                .with_mod(Modifier::Alt)
                .with_mod(Modifier::Ctrl)
                .with_mod(Modifier::Super);
            let mods = &combo.modifiers;
            for w in mods.windows(2) {
                prop_assert!(w[0] <= w[1], "modifiers not sorted");
            }
        }

        #[test]
        fn prop_binding_set_no_conflicts_when_unique_keys(n in 1usize..10) {
            let mut bs = BindingSet::new("unique");
            let mode = ModeId::new("test");
            for i in 0..n.min(26) {
                let c = (b'a' + i as u8) as char;
                bs.add(
                    KeyCombo::new(Key::Letter(c)),
                    mode.clone(),
                    Action::new(format!("a{}", i), "test", WmAction::Close),
                    false,
                );
            }
            let axiom = NoConflicts { bindings: bs };
            prop_assert!(axiom.verify().is_ok());
        }
    }
}
