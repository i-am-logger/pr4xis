# pr4xis skills

Project-specific Claude Code skills for working on the pr4xis repo. Each skill is a self-contained instruction set in its own directory; invoke with the `Skill` tool from inside a Claude Code session running in this workspace.

These skills depend on the workspace layout (`crates/pr4xis/src/`, `crates/domains/src/`), the `define_ontology!` macro, and the cite-the-test memory set. They are not portable to other projects without rewriting.

## Skill catalog

### Per-ontology rollout (#57, #59)

Building blocks for the per-ontology README + citings + diagrams pattern. Each skill operates on one ontology directory at a time.

| Skill | What it does |
|---|---|
| [`per-ontology-readme`](per-ontology-readme/SKILL.md) | Read `ontology.rs`, generate `README.md` |
| [`per-ontology-citings`](per-ontology-citings/SKILL.md) | Extract in-code citation comments, generate `citings.md` |
| [`per-ontology-mermaid-internal`](per-ontology-mermaid-internal/SKILL.md) | Generate the internal-structure mermaid diagram |
| [`per-ontology-mermaid-external`](per-ontology-mermaid-external/SKILL.md) | Generate the external-connections mermaid diagram (functors, adjunctions) |
| [`per-ontology-rollout`](per-ontology-rollout/SKILL.md) | Wrapper that invokes all four of the above for one ontology |

### Composable building blocks

Smaller skills that are useful on their own and also called by the bigger ones.

| Skill | What it does |
|---|---|
| [`functor-author`](functor-author/SKILL.md) | Given two ontologies, scaffold a `Functor` impl with `map_object`, `map_morphism`, and the `check_functor_laws` test |
| [`adjunction-author`](adjunction-author/SKILL.md) | Given two functors going opposite ways, scaffold an `Adjunction` impl with unit/counit and the gap-analysis test suite |

### The closing loop

The full ontology authoring workflow as a single skill, composing all of the above.

| Skill | What it does |
|---|---|
| [`ontology-from-paper`](ontology-from-paper/SKILL.md) | Take a topic or source paper, research it, extract concepts/relations/axioms, scaffold the ontology, scaffold candidate functors and adjunctions, run the laws check, run gap analysis, and report. Produces a draft for human review — does not auto-commit. |

## Conventions

All skills in this directory follow the same conventions:

- **Self-contained**: every skill has its own SKILL.md with the full instructions; nothing depends on the conversation history
- **Read-modify-verify-report**: every skill reads source files, makes changes, runs the relevant tests to verify, and reports a summary. None auto-commit.
- **Cite the test**: every numerical claim a skill makes about the codebase is paired with the command that re-derives it
- **Modest framing**: skills produce drafts, not finished work. The user reviews and commits.
- **Memory-aware**: skills check their output against the `feedback_*.md` memories before declaring success

## Adding a new skill

1. Create a directory under `.claude/skills/<skill-name>/`
2. Write a `SKILL.md` with frontmatter (`name`, `description`) and instructions
3. Add a row to this README's catalog
4. Test it on a small case before relying on it for batch work

## Related issues

- [#57](https://github.com/i-am-logger/pr4xis/issues/57) — per-ontology README + citings + co-located papers
- [#59](https://github.com/i-am-logger/pr4xis/issues/59) — per-ontology mermaid diagrams
- [#44](https://github.com/i-am-logger/pr4xis/issues/44) — build an ontology from a paper (the docs side)
- [#62](https://github.com/i-am-logger/pr4xis/issues/62) — encode Heim's syntrometric primitives (one specific use of `ontology-from-paper`)
