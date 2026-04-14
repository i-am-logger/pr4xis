# Novel Contributions — Draft Papers

Original research on ontological structure, gap detection, and machine-checkable scientific knowledge representation. All three papers are **drafts**, intended for publication. Every numerical claim in every paper is paired with a `cargo test` command that re-derives it from the live codebase — see the "Code & Verification" section at the end of each paper.

| # | Paper | Focus |
|---|-------|---|
| 1 | [Categorical Bioelectricity](01-categorical-bioelectricity.md) | First category-theoretic formalization of Levin's bioelectric framework — 14 biomedical domains as formal ontologies, 21 cross-domain functors, 3 adjunctions |
| 2 | [Adjunction Information Loss](02-adjunction-information-loss.md) | A general methodology for using categorical adjunctions to automatically detect missing distinctions in scientific ontologies; applied to the biomedical stack with measured loss percentages |
| 3 | [Ontology Diagnostics](03-ontology-diagnostics.md) | A formal meta-ontology — an ontology about ontology engineering — that captures the gap-detection methodology as 29 entities, 14 pipeline steps, and 13 axioms |

## Reproducibility

Every paper's numerical claims can be verified from the codebase by running:

```bash
git clone https://github.com/i-am-logger/pr4xis
cd pr4xis
cargo test --workspace
cargo test -p pr4xis-domains test_full_chain_collapse_measurement -- --nocapture
```

The first command runs the full test suite (4,855 tests as of the document date; re-counted on every run). The second prints the live per-adjunction collapse percentages cited in all three papers — including the 85.2% molecular-bioelectric round-trip loss and the Kv channel discovery.

For the per-claim test index, see the "Code & Verification" section at the bottom of each paper.

## Status

- **Paper 1** — first complete draft. Awaits literature-review pass and one final scan for stale subset counts. Target: submit Q3 2026.
- **Paper 2** — first complete draft. Awaits the methodology generalization section to be fleshed out with at least one non-biomedical adjunction example.
- **Paper 3** — first complete draft. Awaits the threshold-validation section to be expanded once more adjunctions are measured (currently 3; ideally 5+).

A fourth paper on the syntrometric logic lineage (the structural alignment with Heim's syntrometry, formalized as a pr4xis ontology with a verified functor) is tracked in [#62](https://github.com/i-am-logger/pr4xis/issues/62) — not yet drafted, will become Paper 4 in this directory once #62 lands.

## Related

- [Novelty argument](../novelty.md) — the careful prose about what is new vs prior art, with the same intellectual honesty applied to the lineage claims
- [Gap detection](../gap-detection.md) — the same biomedical result, written for the README audience instead of the journal audience
- [Foundations](../../understand/foundations.md) — academic lineage that grounds the broader project
- [Concepts](../../understand/concepts.md) — the categorical machinery the papers build on, in plain English
