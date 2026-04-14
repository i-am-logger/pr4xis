# 02 — First Query

The second step in the [Get started](get-started.md) tutorial sequence. After this page you will have made your first interaction with the pr4xis engine, in your browser, with the same WASM build that runs in production.

This page assumes you have already completed [01 — Install](01-install.md).

## Option A (recommended): Local WASM via `dev-web`

`dev-web` is a workspace-provided dev script that builds the WASM bundle, watches `crates/` for changes, and serves the chat surface locally with live reload. The interaction runs entirely in your browser tab — sandboxed, no local process to manage, exactly the same code path as the production [pr4xis.dev](https://pr4xis.dev) demo.

```bash
dev-web
```

You will see something like:

```
Building WASM...
Starting pr4xis-web with live reload...
  /                — WASM chatbot
  /decks/technical — presentation
Watching crates/ for changes — WASM rebuilds automatically.
```

Open the URL it serves (typically `http://localhost:8080` or similar). The page loads the English ontology (~107K WordNet concepts compiled into the WASM binary at build time) and gives you a chat input.

If you don't have `dev-web` on your path, you are not in the dev shell. Run `devenv shell` first if you use Nix, or fall back to Option B below.

## Try a few queries

```
is a cat a mammal
define telescope
is a guitar a string instrument
```

The system tokenizes each input, parses it through the [Lambek pregroup grammar](https://en.wikipedia.org/wiki/Pregroup_grammar), looks up the entities in the WordNet taxonomy, and answers from the loaded category. The trace pane shows every step the engine took.

## What you should expect

The chat surface is **a working surface, not a polished product**. Three classes of behavior you will see:

1. **Clean derivation.** The query parses, the entities resolve, the taxonomy traversal answers. The trace shows every step.
2. **Honest "no" with a reason.** The query parses, but the answer doesn't follow from the loaded ontology. The system says so. *This is correct behavior* — pr4xis cannot make up answers that aren't in its axioms.
3. **Grammar gap.** The query doesn't fit the current pregroup parser coverage. The system says so. *This is a bug report* — file an issue with the exact input.

The third case is the one we care most about closing. Pr4xis will get better the more grammar gaps users surface.

## Option B: The CLI chatbot (no browser)

If you cannot use `dev-web` for any reason, the same engine ships as a CLI:

```bash
cargo run -p pr4xis-cli
```

This starts a chat loop in your terminal. Same engine, same ontology, same answers — minus the browser sandbox and the live trace pane. Useful for headless servers, CI debugging, or anywhere a browser isn't available.

## Option C: The hosted demo

If you don't want to run anything locally at all, open **[pr4xis.dev](https://pr4xis.dev)**. Same WASM build as Option A, hosted as a static site. Loads the English ontology at startup (a few seconds), then accepts the same queries.

## Inspecting the trace

All three options produce the same structured trace for each query. The trace is a sequence of `PipelineTraceEntry` records, one per pipeline stage:

1. **Tokenize** — input → `TypedToken[]`
2. **Parse** — tokens → Lambek pregroup reduction
3. **Interpret** — parse tree → Montague semantic form
4. **Speech act classification** — what kind of thing the user said
5. **Metacognition** — which response strategy fits
6. **Entity lookup / taxonomy traversal / common ancestor / etc.** — the actual reasoning
7. **Realization** — semantic answer → human-readable text

When something goes wrong, the trace is where you look first. Every entry tells you which ontology produced it, which operation was performed, and whether it succeeded.

## Inspecting the trace

Both surfaces produce a structured trace for each query. The trace is a sequence of `PipelineTraceEntry` records, one per pipeline stage:

1. **Tokenize** — input → `TypedToken[]`
2. **Parse** — tokens → Lambek pregroup reduction
3. **Interpret** — parse tree → Montague semantic form
4. **Speech act classification** — what kind of thing the user said
5. **Metacognition** — which response strategy fits
6. **Entity lookup / taxonomy traversal / common ancestor / etc.** — the actual reasoning
7. **Realization** — semantic answer → human-readable text

When something goes wrong, the trace is where you look first. Every entry tells you which ontology produced it, which operation was performed, and whether it succeeded.

## What you have now

- A working interaction with the pr4xis engine
- A sense of which queries land cleanly and which don't
- A way to see the full reasoning trace for any query

## Next

Continue with **[03 — First Ontology](03-first-ontology.md)** to write your own minimal `define_ontology!` block.

---

- **Document date:** 2026-04-14
