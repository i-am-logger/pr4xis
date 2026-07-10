# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.28.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-runtime-v0.27.0...pr4xis-runtime-v0.28.0) - 2026-07-10

### Chore

- move the remaining internal working docs out of the published docs/ tree

### Docs

- fix 109 feature-gated intra-doc links — the dev-ci docs stage was red

### Feat

- *(english)* [**breaking**] audit-5 wave 5 — ship the 9 store buffers as the load payload; the +348 MiB wasm transient is dead
- *(hardening)* audit-5 wave 4 — validated PackedCsr entry, load ∀-properties, self-describing load envelope
- *(domains)* [**breaking**] generalize grounding -- any .prx carries its grounding functor as data (W2.1)
- *(domains)* statutes compose -- a loaded USC section reaches the statute/law taxonomy (Step 4)
- *(runtime)* Lever A -- RuntimeOntology reasons over the archived rkyv buffer, zero-copy (Step 1c)
- *(runtime)* lazy memoized reachability -- drop the pre-folded owned closure (Step 1a)
- *(runtime)* ArchiveLens -- a law-checked rkyv lens over the runtime Archive (Step 0)

### Fix

- *(audit)* audit-4 — 21 confirmed findings on the post-review commits, all fixed or justified
- *(review)* third-review polish + hardening -- doc accuracy, test teeth, one primitive-leak
- *(grounding)* make into-English reachable via the public load path + fail-close two silent drops

### Perf

- *(closure)* [**breaking**] audit-5 wave 3 — u32-CSR MaterializedClosure, strings resident once (−42 MiB on title 42)
- *(lens)* owned-consuming put leg — the rich stores MOVE into their mirrors at build
- *(runtime,wasm)* index by-name node lookup + compile-time embedded-demo guard (follow-ups)

### Refactor

- *(reach)* [**breaking**] ReachSubstrate engine — LazyKindReach and the 216 MiB English bridge deleted
- *(reach)* [**breaking**] one graded-reach kernel for both engines — the de-privilege-English core
- *(english)* all 9 archived stores → 2 cited WellBehavedLens families; English owns nothing

## [0.27.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-runtime-v0.26.0...pr4xis-runtime-v0.27.0) - 2026-07-06

### Feat

- *(runtime)* [**breaking**] mint lexical surfaces on every emitted ontology by default
- *(chat)* answer conceptual legal questions from a loaded ontology

## [0.26.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-runtime-v0.25.5...pr4xis-runtime-v0.26.0) - 2026-07-04

### Feat

- constitution coverage — every test declares its guarantee, gate-enforced

## [0.25.4](https://github.com/i-am-logger/pr4xis/compare/pr4xis-runtime-v0.25.3...pr4xis-runtime-v0.25.4) - 2026-06-19

### Chore

- crate metadata for discoverability — crates.io categories/keywords, homepage, CITATION.cff

## [0.25.3](https://github.com/i-am-logger/pr4xis/compare/pr4xis-runtime-v0.25.1...pr4xis-runtime-v0.25.3) - 2026-06-17

### Feat

- *(runtime)* one generic loader for every envelope (A5)
- *(runtime)* teach-a-peer agrees on a kind's meaning (A3, slice c)
- *(runtime)* load the relation-kind vocabulary from the Relations ontology (A3, slice b)
- *(runtime)* a morphism carries the kind's address, not its name (A3, slice a)
- *(runtime)* the functor rides with the concept — teach-a-peer interpretation (A2, slice c)
- *(runtime)* teach-a-peer extraction — a concept's minimal payload round-trips its recursive address (A2, slice b)
- *(runtime)* recursive content-addressing — a concept address that transitively fixes its definition (A2, slice a)

## [0.25.2](https://github.com/i-am-logger/pr4xis/compare/pr4xis-runtime-v0.25.1...pr4xis-runtime-v0.25.2) - 2026-06-17

### Feat

- *(runtime)* one generic loader for every envelope (A5)
- *(runtime)* teach-a-peer agrees on a kind's meaning (A3, slice c)
- *(runtime)* load the relation-kind vocabulary from the Relations ontology (A3, slice b)
- *(runtime)* a morphism carries the kind's address, not its name (A3, slice a)
- *(runtime)* the functor rides with the concept — teach-a-peer interpretation (A2, slice c)
- *(runtime)* teach-a-peer extraction — a concept's minimal payload round-trips its recursive address (A2, slice b)
- *(runtime)* recursive content-addressing — a concept address that transitively fixes its definition (A2, slice a)

## [0.25.1](https://github.com/i-am-logger/pr4xis/compare/pr4xis-runtime-v0.24.0...pr4xis-runtime-v0.25.1) - 2026-06-16

### Feat

- *(runtime,chat)* relation-parametric image/meet/chain — Parthood evidence chains
- *(self-aware)* per-ontology capabilities — what each loaded ontology can answer (§4.7)
- *(derive,runtime)* transitive kinds are loaded from data — Phase A Step 4
- *(runtime)* relation kind is a ConceptRef, not a closed enum — Phase A Step 3
- *(runtime)* transitive_kinds — relation transitivity as loaded data, not a constant

## [0.25.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-runtime-v0.24.0...pr4xis-runtime-v0.25.0) - 2026-06-16

### Feat

- *(runtime,chat)* relation-parametric image/meet/chain — Parthood evidence chains
- *(self-aware)* per-ontology capabilities — what each loaded ontology can answer (§4.7)
- *(derive,runtime)* transitive kinds are loaded from data — Phase A Step 4
- *(runtime)* relation kind is a ConceptRef, not a closed enum — Phase A Step 3
- *(runtime)* transitive_kinds — relation transitivity as loaded data, not a constant

## [0.24.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-runtime-v0.23.1...pr4xis-runtime-v0.24.0) - 2026-06-14

### Build

- *(release)* re-baseline the workspace to 0.24.0
- *(release)* single workspace version via inheritance — fix the release-plz drift

## [0.23.1](https://github.com/i-am-logger/pr4xis/compare/pr4xis-runtime-v0.23.0...pr4xis-runtime-v0.23.1) - 2026-06-12

### Feat

- generic grounding — ground(lens) over the substrate, denotes is one lens
- *(runtime)* ContainsAtom — resolve a grounded edge's foreign atom, fail-closed (G3a)
- *(runtime)* EdgeTarget — the foreign-atom slot, byte-exact (grounding G2)
- *(runtime)* Grounding — a Connection refined to the atom level, in the meta-.prx
- *(runtime)* apply — the data-driven FreeExtension, projection-as-data interpreter

### Fix

- *(docs,grounding)* broken intra-doc links + Copilot review

## [0.22.0](https://github.com/i-am-logger/pr4xis/releases/tag/pr4xis-runtime-v0.22.0) - 2026-06-10

### Added

- `.prx` — praxis' knowledge, in a file ([#186](https://github.com/i-am-logger/pr4xis/pull/186))
