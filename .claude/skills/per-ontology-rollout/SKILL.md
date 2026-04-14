---
name: per-ontology-rollout
description: Wrapper that runs the four per-ontology generation skills in sequence on a single ontology directory — README, citings, internal mermaid, external mermaid. Use this for the #57/#59 rollout when you want one invocation per ontology instead of four.
---

# per-ontology-rollout

Wrapper skill that runs the full per-ontology generation pack on one ontology directory. Composes four sibling skills.

## When to invoke

When you want to bring a single ontology directory to the #57/#59 standard (README + citings + two mermaid diagrams) in one shot. For batch processing all 106 ontologies, invoke this wrapper repeatedly with different paths.

## Inputs

- **Required**: absolute path to an ontology directory under `crates/domains/src/`
- **Optional**: `--overwrite` flag to overwrite existing files (default: refuse and ask per skill)

## Steps

In order, with each step's failure halting the rest:

1. **Validate the input**: confirm the directory exists, contains `ontology.rs`, and the file has a `define_ontology!` block. If not, stop with an error.

2. **Invoke [`per-ontology-readme`](../per-ontology-readme/SKILL.md)** to generate the README skeleton. This produces the file with the structural sections; the mermaid diagrams will be added by steps 4 and 5.

3. **Invoke [`per-ontology-citings`](../per-ontology-citings/SKILL.md)** to extract citation comments from the code and produce `citings.md` next to the README.

4. **Invoke [`per-ontology-mermaid-internal`](../per-ontology-mermaid-internal/SKILL.md)** to generate the internal-structure diagram and insert it into the README between auto-generated markers.

5. **Invoke [`per-ontology-mermaid-external`](../per-ontology-mermaid-external/SKILL.md)** to generate the cross-ontology connections diagram and insert it into the same README.

6. **Relocate cited PDFs into `<ontology-dir>/papers/`**. For each primary source in the newly generated `citings.md` that resolves to a PDF in `docs/papers/`, `git mv` the PDF into `<ontology-dir>/papers/` (create the directory if needed) and update the citings.md link to `papers/<filename>.pdf`. If the same PDF is cited by multiple ontologies, leave it in `docs/papers/` and link across — do not duplicate.

7. **Run `cargo test -p pr4xis-domains <ontology-name>`** to confirm the new files (which are markdown, not Rust) haven't somehow disturbed any build script or test runner.

## Output

A consolidated summary from all four sub-skills:

- Path of the new README
- Path of the new citings.md
- Word count of the README
- Number of citations extracted (primary + supporting + pending)
- Number of internal nodes/edges in the diagram
- Number of external connections in the diagram
- Test command that was verified
- Anything across all sub-skills that the human should review (collected from each sub-skill's report)

## Failure modes

- **Any sub-skill fails**: stop, report which one failed and why, do not proceed to the next. Files written by earlier successful sub-skills remain on disk for the human to inspect.
- **A file already exists**: defer to the sub-skill's behavior (refuse + ask, unless `--overwrite` is set)
- **The ontology directory has subdirectories that are themselves ontologies** (e.g., `crates/domains/src/natural/biomedical/` contains many child ontologies): only operate on the directory passed in. Recursing is the caller's job, not this skill's.

## Notes

This wrapper exists for ergonomics — running four skills with one command. It does not add logic beyond sequencing. If you need finer control (e.g., regenerate only the diagrams), invoke the individual sibling skills directly.
