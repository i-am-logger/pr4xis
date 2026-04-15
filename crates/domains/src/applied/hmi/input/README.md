# Input -- Interaction modes and keybindings

Models user input as a structured two-layer ontology: **modes** are states in a statechart (Harel 1987), and **keybindings** are the morphisms that trigger transitions between them. Together they form the interaction algebra — the formal substrate for "what does this key do right now?" and "what keys are available in this context?".

Every rich-interaction surface (terminal, window manager, editor, shell, chat UI, WASM site) is a consumer: a surface loads a `ModeGraph` and a `KeybindingTable`, then routes every input event through them deterministically.

Key references:
- Harel 1987: *Statecharts: a visual formalism for complex systems* (Science of Computer Programming 8:3) — mode graphs, parallel regions, hierarchical states
- Thimbleby 2004: *User Interface Design with Matrix Algebra* (ACM TOCHI 11:2) — interaction as algebra over state and input
- Raskin 2000: *The Humane Interface* — monotony, modelessness discipline
- Beaudouin-Lafon 2000: *Instrumental Interaction* (CHI 2000) — the interaction algebra framing
- ECMA-48 5th Ed 1991 — terminal input conventions
- VT520/xterm escape sequences — the de facto terminal input grammar

## Entities

| Category | Entities |
|---|---|
| Modes (Harel) | `ModeId(String)`, `ModeProperties { catchall, parent, ... }`, `Transition { from, to }`, `ModeGraph { root, modes, transitions }` |
| Keys | `Key` — `Letter(char)`, `Number(u8)`, `Function(u8)`, `Named(NamedKey)`, `Mouse(MouseButton)` |
| Named keys | `Enter`, `Escape`, `Space`, `Tab`, `Backspace`, `Delete`, `Arrow{Up,Down,Left,Right}`, `Home`, `End`, `PageUp`, `PageDown` |
| Modifiers | `Shift`, `Ctrl`, `Alt`, `Meta` |
| Mouse buttons | `Left`, `Right`, `Middle`, `Scroll{Up,Down}` |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| (structural) | Identity and composition laws over the mode graph | auto-generated |
| RootModeIsUnique | A `ModeGraph` has exactly one root mode | Harel 1987 |
| TransitionsReferExistingModes | Every `Transition { from, to }` names modes that exist in the graph | well-formedness |
| EscapeReturnsToParent | If a mode has a parent, an Escape keybinding returns to the parent | Harel 1987 / Raskin 2000 |
| NoOrphanModes | Every non-root mode is reachable from the root via some transition | graph connectivity |

See `modes.rs` and `keybindings.rs` for the `impl Axiom for …` blocks.

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one. The input ontology is a substrate for every surface that needs modal input — the future `ChatSurface` will consume a `ModeGraph` for its chat-vs-command-vs-search modes, and the future `BrowserSurface` will expose a `KeybindingTable` for the site's navigation.

## Files

- `modes.rs` -- `ModeId`, `ModeProperties`, `Transition`, `ModeGraph`, mode-graph axioms, tests
- `keybindings.rs` -- `Key`, `NamedKey`, `Modifier`, `MouseButton`, keybinding tables, tests
- `README.md` -- this file
- `citings.md` -- per-ontology bibliography
- `mod.rs` -- module declarations

Previous home: `applied/theming/modes.rs` + `applied/theming/keybindings.rs`, moved here by [#66](https://github.com/i-am-logger/pr4xis/pull/66).
