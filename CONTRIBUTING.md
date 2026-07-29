# Contributing to pr4xis

pr4xis is an ontology-first reasoning engine: domain knowledge lives in typed,
axiom-backed ontologies under `crates/domains/src/`, not in ad-hoc code. Before
anything else, read the three ways to contribute in the [README](README.md#contributing) —
this file covers the practical mechanics once you know which one you're doing.

## Setup

The project uses [devenv](https://devenv.sh) to pin the exact Rust toolchain and
tool versions CI uses:

```bash
devenv shell
```

Everything below assumes you're inside that shell (or run each command as
`devenv shell -- <command>`).

## Before opening a PR

Run the full local CI pipeline — it mirrors `.github/workflows/ci.yml` exactly
(same `--release` flags, same lint levels), so a green `dev-ci` locally means a
green PR check:

```bash
dev-ci
```

This runs, in order: `treefmt` formatting, external-data fetch, clippy
(`-D warnings`, workspace + wasm target), rustdoc, doc tests, `mdbook test`,
the full nextest suite, the constitution completeness gate, the heavy-corpus
tests, and the wasm/e2e browser tests. `enterShell` lists the individual
scripts (`dev-test`, `dev-fmt`, `dev-lint`, `dev-check`, …) if you want to run
one piece in isolation while iterating.

## Hard requirements the CI enforces

These aren't style preferences — they're `deny`-level lints or explicit gates,
so a PR that violates them won't build or won't pass `dev-ci`:

- **No stubs.** `todo!()`, `unimplemented!()`, and `unreachable!()` are
  `clippy::deny` workspace-wide. Land a feature complete, or don't land it.
- **No `debug_assert!`.** Invariants must hold identically in debug and
  release; use `assert!` or a test instead.
- **Every test in `pr4xis-domains` declares which constitutional guarantee it
  witnesses** via `#[pr4xis::praxis_value(..)]` (see
  [The Constitution](docs/understand/constitution.md)). `scripts/constitution-gate.sh`
  fails the build if a test is untagged.
- **New domain knowledge goes through an ontology**, not inline string
  matching, magic numbers, or hardcoded lists — see
  [Write axioms](docs/use/write-axioms.md) and
  [Build an ontology from a paper](docs/use/build-ontology-from-paper.md) for
  the authoring workflow, and [Compose via functor](docs/use/compose-via-functor.md)
  for connecting a new ontology to an existing one.

## License

By contributing, you agree your contribution is licensed under the project's
[CC BY-NC-SA 4.0](LICENSE) license.

## Code of Conduct

This project follows the [Code of Conduct](CODE_OF_CONDUCT.md).
