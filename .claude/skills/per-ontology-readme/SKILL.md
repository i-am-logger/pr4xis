---
name: per-ontology-readme
description: Generate a README.md for one pr4xis ontology directory by reading its ontology.rs (the define_ontology! block), surrounding code, and source-code citation comments. Produces a short, accurate README that orients a new reader. Does not modify ontology.rs itself; only writes the README.
---

# per-ontology-readme

Generate a `README.md` for a single pr4xis ontology directory.

## When to invoke

When a directory under `crates/domains/src/` (e.g., `crates/domains/src/natural/biomedical/biology/`) contains an `ontology.rs` with a `define_ontology!` block but no `README.md`. Or when the existing `README.md` is stale and needs to be regenerated from the current code.

This is one of four sibling skills for the per-ontology rollout. The wrapper [`per-ontology-rollout`](../per-ontology-rollout/SKILL.md) invokes this one along with `per-ontology-citings`, `per-ontology-mermaid-internal`, and `per-ontology-mermaid-external`.

## Inputs

- **Required**: the absolute path to an ontology directory, e.g., `crates/domains/src/natural/biomedical/biology/`.
- **Optional**: a flag to overwrite an existing README (default: refuse and ask)

## What to read

1. `<ontology-dir>/ontology.rs` — the canonical source. Find the `define_ontology!` block, extract:
   - The ontology struct name (e.g., `BiologyOntology`)
   - The category type name
   - The entity enum name and its variants (these are the concepts)
   - The relation type name
   - Which reasoning systems are populated (`taxonomy:`, `mereology:`, `causation:`, `opposition:`, `context:`)
   - Any structural axioms beyond the auto-generated ones
   - The doc comment on the ontology declaration (often contains the citation)

2. `<ontology-dir>/mod.rs` — to find the public exports and any sibling files (functors, qualities, etc.)

3. `<ontology-dir>/tests.rs` (if present) — to see what's already tested. Useful for the "Status" section.

4. **Functors that reference this ontology** — `grep -rn "<EntityName>" crates/domains/src/ | grep -v "<ontology-dir>"` to find places that import or map this ontology's entities. These reveal cross-domain composition.

5. **The relevant `crates/domains/src/<branch>/mod.rs`** (e.g., `natural/biomedical/mod.rs`) to see how the ontology fits into the broader branch.

## What to write

Follow the **established pattern** already in use across the biomedical ontologies (`crates/domains/src/natural/biomedical/biology/README.md` is the reference). The pattern is more concrete and useful than a generic template — it includes per-reasoning-system mermaid diagrams, entity tables, and a functors index.

The pattern, in order:

```markdown
# <Name> -- <One-line description>

<One short paragraph: what this ontology models. Domain-specific, not abstract.>

Key references:
- <Author Year>: <one-line>
- <Author Year>: <one-line>

## Entities (N)

| Category | Entities |
|---|---|
| <Subcategory> (N) | <comma-separated entity variant names> |
| ... | ... |
| Abstract (N) | <abstract entities like Cell, Tissue, Organism> |

## Taxonomy (is-a)

```mermaid
graph TD
    <Child> --> <Parent>
    ...
```

(One line per row in the `taxonomy:` section of `define_ontology!`.)

## Mereology (has-a)

```mermaid
graph TD
    <Whole> -->|has-a| <Part>
    ...
```

(One line per row in the `mereology:` section. Skip this section if the ontology has no mereology.)

## Causal Graph

<One short sentence naming the events>

```mermaid
graph LR
    <Cause> --> <Effect>
    ...
```

(Skip this section if the ontology has no causation.)

## Opposition Pairs

| Pair | Meaning |
|---|---|
| A / B | <one-line description> |

(Skip if no opposition.)

## Qualities

| Quality | Type | Description |
|---|---|---|
| <QualityName> | <return type> | <one-line> |

(Skip if no qualities.)

## Axioms (N)

| Axiom | Description | Source |
|---|---|---|
| <AxiomName> | <description from the impl> | <author year, or "structural" for auto-generated, or "anatomy/biophysics/etc." for domain> |

## Functors

**Outgoing (N):**

| Functor | Target | File |
|---|---|---|
| <Name> | <target ontology name> | `<file.rs>` |

**Incoming (N):**

| Functor | Source | File |
|---|---|---|
| <Name> | <source ontology name> | `<relative path>` |

(One or both subtables may be empty. If both are, write "No cross-domain functors yet.")

## Files

- `ontology.rs` -- Entity, taxonomy, mereology, category, qualities, axioms, tests
- `<other-file>.rs` -- <what it contains>
- `mod.rs` -- Module declarations
```

This is the pattern. Match it as closely as possible for any new ontology README.

**Notes on the pattern**:

- The mermaid diagrams are simple `graph TD` or `graph LR` blocks with no styling. Don't add styling or `classDef` — the existing READMEs deliberately keep them minimal.
- The "Source" column in the Axioms table is one of: a real author-year citation, the word `structural` (for auto-generated structural axioms like NoCycles, Antisymmetric), or a domain category like `anatomy`, `biophysics`, `cross-domain`, `multi-scale`.
- The Files section lists every `.rs` file in the directory with a one-line description.
- Skip sections that don't apply (mereology, causation, opposition, qualities) rather than leaving them empty.

## Rules for what NOT to write

These rules come from the project's memory set. Violating any of them is a bug in the skill output:

- **No marketing language**: no "groundbreaking", "novel", "first", "publication-ready", "world-class". Keep the prose modest. The strength is in the tests, not the adjectives.
- **No specific test counts** unless they come from running `cargo test` and reading the output. If you mention a number, pair it with the command. Better: drop the number and reference the command alone.
- **No "zero domain knowledge"** framing. Use "domain knowledge in composable ontologies" if the topic comes up at all (it usually doesn't need to in a per-ontology README).
- **No target venue claims**, no "submitting to" or "publication-bound" language. Drafts are drafts.
- **No comparisons to LLMs** in a per-ontology README. That belongs in the README at the project root, not here.
- **No "we present" academic-paper voice**. This is a code documentation file, not a paper.

## Verification before declaring success

Before reporting "done":

1. The new `README.md` exists and is non-empty
2. Every section is present (six sections per the template)
3. The functor list under "Composition with other ontologies" matches what `grep` actually finds
4. The test command in the Status section actually exists and would run if invoked (don't make it up — verify against `tests.rs`)
5. The document date is today's ISO date
6. Run `cargo test -p pr4xis-domains <ontology-name>` to confirm nothing has broken

## Output

Report a summary with:

- Path of the new README
- Word count
- Number of functors listed under composition
- Test command that was verified
- Anything the skill noticed that the human should review (e.g., "the doc comment mentions Alberts 2015 but I couldn't find a corresponding entry in `references.md`; suggest adding it via `per-ontology-citings`")

Then stop. Do not commit. The user reviews and commits.

## Failure modes

- **`ontology.rs` doesn't have a `define_ontology!` block**: this is not a standard pr4xis ontology. Ask the user before proceeding — they may have a custom layout or this directory may not be an ontology.
- **The entity enum can't be found**: probably `#[derive(Entity)]` is on a struct elsewhere. Read more files until you find it. If you can't, surface the issue and stop.
- **The functor grep returns hundreds of results**: the entity name is probably too generic (e.g., `Cell`, `State`). Filter to results inside `impl Functor` blocks only. If still too noisy, list only the `impl Functor for <Name>` patterns.
- **Existing `README.md` already exists and matches the template**: report "no changes needed" and stop. Don't overwrite to a regenerated identical version.
- **Existing `README.md` is hand-written and doesn't match the template**: ask the user. Some ontologies may have been documented before this skill existed, and overwriting hand-written prose is destructive.

## Notes

This skill is intentionally conservative. It produces a short README that's easy to maintain, not a comprehensive one. Depth lives in the per-ontology paper(s) under `papers/`, in `docs/research/papers/`, and in the workspace-wide docs under `docs/`.
