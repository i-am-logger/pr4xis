# Test Architecture

Praxis loads ontologies from large external corpora — the 89 MB Open English
WordNet, the U.S. Code (Title 42 alone is 113 MB). How the test suite *handles*
those corpora is itself a praxis-aligned design, not an afterthought.

## The principle: parse-once is immutability

The `.prx` artifact exists because parsing a source once and addressing the
result by content is the right shape: a source is parsed, compiled to a
compact content-addressed `.prx`, and every consumer loads that immutable
artifact instead of re-parsing. The test suite mirrors the same discipline —
**a corpus is parsed once and shared, never re-parsed per test.**

This matters because of how the two test runners differ.

## Two runners, on purpose

| runner | model | a parsing `LazyLock`/`OnceLock`… |
|---|---|---|
| **nextest** | one OS **process per `#[test]`** | …is re-initialized for *every* test — no sharing |
| **`cargo test`** | threads in **one process per binary** | …is shared once across all tests in the binary |

Nextest's process-per-test isolation is exactly what you want for the bulk of
the suite (crash isolation, per-test timeouts, `cargo nextest archive`
build-once/run-many, cross-binary scheduling). But it means a test that parses a
89 MB corpus in a process-local static re-pays that parse for *every* test that
touches it. With ~30 USC structural tests, that is ~30 redundant 100 MB parses.

So praxis runs each tier under the runner that fits it.

## The tiers

1. **Fast unit / axiom / proptest** — small inline fixtures, in-crate
   `#[cfg(test)]`. Runner: **nextest**. The overwhelming majority of tests.

2. **Heavy-corpus producer / round-trip** — the few tests that *must* parse a
   raw multi-hundred-MB giant (full-title structural invariants, codec
   round-trips, byte-exact reconstruction). Runner: **`cargo test`**, in the
   workspace-excluded `crates/praxis-corpus-tests`, where a `LazyLock` parses
   each giant **once** for the whole binary.

3. **Product-metric gates** — assert the properties the `.prx` work delivers
   (compactness, load-speed, losslessness) over the on-disk corpora, so a
   regression fails CI. Also in `praxis-corpus-tests`.

4. **Consumer** — load the content-pinned `.prx` and assert on the materialized
   ontology, instead of re-parsing the source. The `.prx-cache` (below) is the
   shared cross-process fixture.

## `crates/praxis-corpus-tests`

A dedicated crate, **excluded from the default workspace** (alongside `wasm` and
`e2e`). Exclusion is deliberate:

- it depends on `pr4xis-domains` with `test-internals` (+ `codegen`, `prx`); as a
  non-member, that feature never unifies into the normal workspace build;
- the giants never re-parse under `cargo test --workspace` / nextest — they live
  only in this explicit, `cargo test`-run lane.

Each giant gets a `LazyLock` fixture; every test in the file borrows the one
shared parse:

```rust,ignore
static TITLE_18: LazyLock<Option<UslmCorpus>> =
    LazyLock::new(|| load_uslm_corpus("legal/uscode/usc_title_18/usc_title_18-pl-119-90.xml"));

macro_rules! corpus_or_skip {
    () => {
        match &*TITLE_18 {
            Some(c) => c,
            None => { eprintln!("SKIP: not on disk"); return; }
        }
    };
}

#[test]
fn every_section_satisfies_every_axiom() {
    let UslmCorpus { title, .. } = corpus_or_skip!();
    axiom_every_section_has_num(title).expect("…");
    // … all 37 Title-18 tests share this one parse.
}
```

A corpus absent on a fresh checkout is **skipped gracefully** — the giants are
fetched (`pr4xis update`), not committed.

## The `.prx-cache`: a content-addressed fixture

`pr4xis compile --compact` parses each giant once and writes a compact,
content-addressed `.prx` to the cache. The runtime's `loaded()` fast path reads
that cache through a fail-closed pin gate, with no XML re-parse. This is the
cross-*process* parse-once fixture — the same "compile once, load an immutable
artifact" the product ships, reused by the test suite. CI emits it
(`pr4xis compile --compact`) before the test run.

## Product-metric gates

The compactness and load-speed numbers the `.prx` work exists to deliver are
asserted, not just observed. `usc_compact_gate.rs` iterates every on-disk USC
title (source-agnostic, via the data-source registry) and gates:

- **compactness** — the compact `.prx.gz` is smaller than `gzip(source)`;
- **load-speed** — materializing from the compact `.prx` is far faster than
  parsing the raw USLM XML (asserted on the aggregate with a generous margin, so
  it gates the regression without flaking on CI jitter);
- **losslessness** — the compact-loaded section count equals the XML-parsed
  count, including the 113 MB Title 42.

## Running the lanes

```bash
# Fast bulk (nextest, all workspace crates):
cargo nextest run --workspace --profile ci --release

# Heavy-corpus lane + the product-metric gates (cargo test, parse-once):
cargo test --manifest-path crates/praxis-corpus-tests/Cargo.toml --release

# Doctests (nextest can't run them):
cargo test --doc --workspace --release
```

`dev-ci` runs all of these in sequence (see `devenv.nix`); CI mirrors it.
