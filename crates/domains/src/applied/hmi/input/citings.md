# Citings — Input -- Interaction modes and keybindings

Every published source this ontology stands on. Entries below are drawn from the ontology's [README.md](README.md) and the doc comments on its axioms. Where a full bibliographic entry exists in the workspace-wide [`docs/papers/references.md`](../../../../../../docs/papers/references.md), the short form here is a pointer.

## Primary sources

- Harel 1987: *Statecharts: a visual formalism for complex systems* (Science of Computer Programming 8:3) — mode graphs, parallel regions, hierarchical states
- Thimbleby 2004: *User Interface Design with Matrix Algebra* (ACM TOCHI 11:2) — interaction as algebra over state and input
- Raskin 2000: *The Humane Interface* — monotony, modelessness discipline
- Beaudouin-Lafon 2000: *Instrumental Interaction* (CHI 2000) — the interaction algebra framing
- ECMA-48 5th Ed 1991 — terminal input conventions
- VT520/xterm escape sequences — the de facto terminal input grammar

## Window-action ontology sources (`wm_action.rs`)

The spine grounding the abstract action vocabulary, its typed parameters, and the realization functor — established by a literature sweep, not a single product's config schema.

- Myers, B. A. 1988: *A Taxonomy of Window Manager User Interfaces* (IEEE Computer Graphics and Applications 8(5):65–84, doi:10.1109/38.7762) — the WM intent verb set; the paper's own split of operation **functionality** (which operations exist) from operation **user interface** (how invoked) = intent-vs-mechanism in WM literature
- Card, Moran & Newell 1983: *The Psychology of Human-Computer Interaction* (Erlbaum) — GOMS: Goal / Operator / **Method** / Selection-rule; a goal is independent of the method that realizes it
- Goguen, Thatcher & Wagner 1978: *An Initial Algebra Approach to the Specification, Correctness, and Implementation of Abstract Data Types* (Current Trends in Programming Methodology IV, Prentice-Hall, pp. 80–149) — the initial algebra's unique homomorphism into any other algebra of the signature = the realization projection
- Payne & Green 1986: *Task-Action Grammars* (Human-Computer Interaction 2(2):93–133, doi:10.1207/s15327051hci0202_1) — features as strongly-typed variables over small closed value-sets; one rule-schema per intent family
- EWMH v1.5 2013 (freedesktop.org) — `_NET_WM_ACTION_*`/`_NET_WM_STATE_*` named atoms, `_NET_WM_MOVERESIZE` direction enum, add/remove/toggle, the desktop CARDINAL; the *maximize ≠ fullscreen* state distinction
- Foley & van Dam 1982: *Fundamentals of Interactive Computer Graphics* (Addison-Wesley) — conceptual/semantic/syntactic/lexical stack: mechanism emission as lexical lowering
- Mac Lane 1971: *Categories for the Working Mathematician* II.1 — functor laws (identity + composition preservation)
- Reiter 1978: *On Closed World Data Bases* — the closed/open-world split: `WmAction` is open-world because `Exec` carries arbitrary external data
- Plotkin & Power 2003: *Algebraic Operations and Generic Effects* — sequenced effects compose in the free monoid (the composite-action realization)
- Hyprland dispatchers — <https://wiki.hyprland.org/Configuring/Dispatchers/> — the realization vocabulary the `Dispatch` enum names

## Cross-references

- Workspace bibliography: [`docs/papers/references.md`](../../../../../../docs/papers/references.md)
- Source attributions per axiom: see the `Source` column in the `## Axioms` table in [`README.md`](README.md)
- Code-level citations: `grep -n 'Source:\|Reference:' *.rs` in this directory

## Pending verification

Every entry under **Primary sources** is a short pointer. For each one, confirm that a full citation (Author, Year, Title, DOI/URL) exists in `docs/papers/references.md`. Where no entry exists, add it (or a local PDF under a `papers/` subdirectory) before declaring the ontology citation-complete.

Open items for human review:

- [ ] Cross-check every primary source against `docs/papers/references.md`
- [ ] Add code-comment-level citations (`// Source: ...`) to axioms that currently lack attribution
- [ ] If this ontology depends on a paper not yet in the workspace bibliography, move/copy the PDF into a local `papers/` subdirectory and link it from the primary source line above

---

- **Document date:** 2026-04-15
- **How this file is maintained:** initialized by the per-ontology rollout (issue #57 / #173) from `README.md`'s *Key references* section. Update by hand as code-comment citations, local PDFs, and `docs/papers/references.md` entries are added.
