# Get Started

This page walks you through running pr4xis locally and writing your first interaction with the substrate. It assumes you have a recent Rust toolchain and `cargo`.

## 1. Install

```bash
git clone https://github.com/i-am-logger/pr4xis
cd pr4xis
cargo test --workspace
```

The full test suite runs in under a minute on a single core. If anything fails, [file an issue](https://github.com/i-am-logger/pr4xis/issues).

If you use Nix, the workspace is set up with `devenv` — `devenv shell` will drop you into a configured environment.

## 2. Try the CLI chatbot

```bash
cargo run -p pr4xis-cli
```

This loads the local English ontology (the same WordNet used by the WASM demo at [pr4xis.dev](https://pr4xis.dev)) and starts a chat loop. You can ask taxonomy questions, definitions, and simple compositional queries.

The CLI is a working surface, not a polished product. If a query parses but produces an unhelpful answer, that is a bug — file an issue with the exact input.

## 3. Use the engine in your own code

The substrate's central pattern is **runtime axiom enforcement**: an `Engine` carries a situation and a list of preconditions, and any action that violates a precondition is *blocked, named, and recoverable* — never silently approximated. The chess domain is a clean illustration:

```rust
use pr4xis_domains::social::games::chess::{new_game, ChessAction, Square};

fn main() {
    // A new game starts with the full rules of chess as preconditions.
    let game = new_game();

    // e2-e4: legal opening move, accepted.
    let game = game
        .next(ChessAction::new(Square::new(4, 1), Square::new(4, 3)))
        .unwrap();

    // An illegal move is BLOCKED. The failing precondition is named,
    // the engine is recoverable, nothing is approximated away.
    let illegal = ChessAction::new(Square::new(0, 0), Square::new(7, 7));
    assert!(game.next(illegal).is_err());  // axiom violation, not a wrong answer
}
```

This exact pattern is verified by `cargo test -p pr4xis-domains test_engine_illegal_move_blocked` (and many other engine tests in `crates/domains/src/social/games/chess/tests.rs`).

The same `Engine` pattern works for any domain in the workspace: traffic signals, elevator dispatch, HTTP state machines, judicial workflows, sensor fusion gates, orbital mechanics, and dozens more. A precondition that holds is a proof; a precondition that fails is a blocked action with a named cause.

## 4. Browse the 106 ontologies

Every ontology in the workspace lives at exactly one path under `crates/domains/src/`. To list them:

```bash
find crates/domains/src -name ontology.rs
```

The current organization (formal / applied / social / natural / cognitive) is described in the [Domain catalog](../reference/domain-catalog.md). Each ontology is a self-contained `define_ontology!` block — open one to see the pattern.

## 5. Where to go next

- **[Concepts](../understand/concepts.md)** — what an ontology is in pr4xis, why category theory, how functors and adjunctions work
- **[Architecture](../understand/architecture.md)** — the five-layer Rust stack and how the engine actually runs
- **[Foundations](../understand/foundations.md)** — academic lineage, source papers, the Spencer-Brown / Heim distinction-calculus tradition
- **[Gap detection](../research/gap-detection.md)** — a concrete result: pr4xis automatically detected a missing distinction in molecular biology that experts had collapsed into one entity

If you want to contribute an ontology of your own, the existing ones under `crates/domains/src/` are the working examples. A dedicated authoring guide is tracked in [#44](https://github.com/i-am-logger/pr4xis/issues/44).
