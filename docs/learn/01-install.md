# 01 — Install

The first step in the [Get started](get-started.md) tutorial sequence. After this page you will have pr4xis built locally and the test suite green.

## Prerequisites

- A Rust toolchain — version 1.85 or later, edition 2024. The simplest install is via [rustup](https://rustup.rs/).
- `git`.
- About 1 GB of free disk space for the build artifacts (the `target/` directory grows during compilation).

If you use Nix, the project ships with `devenv` configured. After cloning, `devenv shell` drops you into a fully-configured environment with the right Rust version, `cargo-nextest`, `treefmt`, and other dev tools already on the path. You do not need to install Rust separately.

## Clone and build

```bash
git clone https://github.com/i-am-logger/pr4xis
cd pr4xis
cargo build --workspace
```

The first build pulls dependencies and compiles the seven workspace crates. Expect 2-5 minutes on a modern machine, longer on first run.

## Verify with the test suite

```bash
cargo test --workspace
```

This runs the entire test suite — more than 7,000 tests across the workspace, including category laws, functor laws, axiom checks, and property-based tests via [proptest](https://github.com/proptest-rs/proptest). A release run on a multi-core machine takes a few minutes. You should see something like:

```text
test result: ok. <N> passed; 0 failed; ...
```

If anything fails, **that is the bug report**. File an issue with the failing test name and your toolchain version. Pre-existing test failures are not normal — the workspace is meant to be green at every commit on master.

## Optional: run the property-based tests with more iterations

The default proptest budget is 256 cases per property, which is enough for routine CI but may miss subtle counterexamples. To run with a larger budget:

```bash
PROPTEST_CASES=10000 cargo test --workspace
```

This is slower (5-10 minutes) but exercises a much wider input space. Use it before submitting a PR that touches reasoning-system code or category-law-checking infrastructure.

## Fetch the WordNet data

The English ontology is built from a WordNet XML file that is not checked into the repository, so a fresh clone does not have it yet. The CLI in [02 — First Query](02-first-query.md) needs it and will refuse to start without it. Fetch it once:

```bash
pr4xis update wordnet
```

If you use Nix, the `dev-data` dev script wraps the same `pr4xis update` step. The file lands at `crates/domains/data/wordnet/english-wordnet-2025.xml`; set `WORDNET_XML` if you keep it elsewhere.

## What you have now

- A working `pr4xis` workspace built from source
- The full test suite passing
- A `target/` directory with the `pr4xis-cli` and `pr4xis-web` binaries
- The full source for the domain ontologies under `crates/domains/src/`
- The WordNet data fetched and ready for the CLI

## Next

Continue with **[02 — First Query](02-first-query.md)** to interact with the engine, either through the CLI chatbot or the WASM browser demo.

---

- **Document date:** 2026-04-14
