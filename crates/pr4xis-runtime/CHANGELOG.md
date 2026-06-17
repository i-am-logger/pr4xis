# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
