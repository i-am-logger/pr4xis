---
name: adjunction-author
description: Given two existing functors going in opposite directions between two ontologies, scaffold an Adjunction impl with unit/counit and the gap-analysis test suite. Used after functor-author for both directions, when the goal is to detect missing distinctions between scales.
---

# adjunction-author

Scaffold an `Adjunction` impl from two existing functors and the test suite that verifies its unit/counit and runs the gap-analysis.

## When to invoke

After [`functor-author`](../functor-author/SKILL.md) has produced two functors going in opposite directions between the same pair of ontologies (e.g., `MolecularToBioelectric` and `BioelectricToMolecular`). Whenever you want to detect missing distinctions between scales via round-trip collapse — the methodology from `docs/research/papers/02-adjunction-information-loss.md`.

## Inputs

- **Required**: name of the left functor (e.g., `MolecularToBioelectric`)
- **Required**: name of the right functor (e.g., `BioelectricToMolecular`)
- **Required**: ontology path where the adjunction module should live (typically the parent of the two ontologies, e.g., `crates/domains/src/natural/biomedical/`)
- **Optional**: name for the adjunction (default: `<LeftDomain><RightDomain>Adjunction`)

## What to read

1. Both functor source files — confirm they exist, exposed types, and that the source/target categories pair up correctly (left functor's target is the right functor's source, and vice versa)
2. Existing adjunctions in `crates/domains/src/natural/biomedical/adjunctions.rs` for the canonical pattern
3. The `Adjunction` trait at `crates/pr4xis/src/category/adjunction.rs` for the required signatures

## What to generate

In `<adjunction-dir>/adjunctions.rs` (creating the file or appending if it exists), add:

```rust
/// Adjunction between <left-domain> and <right-domain>.
///
/// Left adjoint F = <LeftFunctor>: maps <source> to <target>.
/// Right adjoint G = <RightFunctor>: maps <target> back to <source>.
///
/// Unit η_A: A → G(F(A)) — embeds an entity into its round-trip canonical form.
/// Counit ε_B: F(G(B)) → B — projects the round-trip back to its target.
pub struct <AdjunctionName>;

impl Adjunction for <AdjunctionName> {
    type Left = <LeftFunctor>;
    type Right = <RightFunctor>;

    fn unit(obj: &<SourceEntity>) -> <SourceRelation> {
        let round_trip = <RightFunctor>::map_object(&<LeftFunctor>::map_object(obj));
        <SourceRelation> { from: *obj, to: round_trip }
    }

    fn counit(obj: &<TargetEntity>) -> <TargetRelation> {
        let round_trip = <LeftFunctor>::map_object(&<RightFunctor>::map_object(obj));
        <TargetRelation> { from: round_trip, to: *obj }
    }
}
```

Plus tests in the same file:

```rust
#[cfg(test)]
mod <adjunction_name>_tests {
    use super::*;

    #[test]
    fn test_unit_is_valid() {
        for obj in <SourceEntity>::variants() {
            let unit = <AdjunctionName>::unit(&obj);
            assert!(<SourceEntity>::variants().contains(&unit.from));
            assert!(<SourceEntity>::variants().contains(&unit.to));
        }
    }

    #[test]
    fn test_counit_is_valid() {
        for obj in <TargetEntity>::variants() {
            let counit = <AdjunctionName>::counit(&obj);
            assert!(<TargetEntity>::variants().contains(&counit.from));
            assert!(<TargetEntity>::variants().contains(&counit.to));
        }
    }

    #[test]
    fn test_gaps_exist_or_are_empty() {
        // Either there are gaps (round-trip collapse) or the adjunction is an
        // equivalence (no collapse). Both are valid outcomes; this test just
        // checks that the analysis runs without panicking.
        let _gaps: Vec<_> = <SourceEntity>::variants().iter()
            .map(|obj| <AdjunctionName>::unit(obj))
            .collect();
    }
}
```

If a `gap_analysis.rs` exists at `crates/domains/src/formal/meta/`, also add an `analyze_<adjunction_name>()` function to it that mirrors the existing `analyze_molecular_bioelectric()` pattern.

## Verification

1. The new code compiles — `cargo check -p pr4xis-domains`
2. The unit/counit tests pass — `cargo test -p pr4xis-domains <adjunction_name>`
3. Optionally: the gap-analysis function (if added to gap_analysis.rs) runs without panicking and produces sensible numbers via `cargo test -p pr4xis-domains test_full_chain_collapse_measurement -- --nocapture`

## Output

Report:
- Path of the file the adjunction was added to (created or appended)
- Whether the unit/counit tests passed
- If gap-analysis was integrated, the live unit-loss and counit-loss percentages from running the test
- Any `todo!` left over from the underlying functors (the adjunction inherits whatever the functors couldn't infer)

## Failure modes

- **The two functors don't pair up** (left's target ≠ right's source, or vice versa): refuse with an error explaining the mismatch
- **One of the functors has unfilled `todo!` cases**: emit a warning and proceed anyway — the adjunction will compile but the unit/counit may panic at runtime when the `todo!` is hit
- **The unit or counit test panics**: report which entity caused it. This usually means a `todo!` from the underlying functor; the human should fill it in, then re-run

## Notes

Adjunctions are pr4xis's gap-detection mechanism. After generating the adjunction, consider running [the gap analysis](../../../crates/domains/src/formal/meta/gap_analysis.rs) to surface missing distinctions in the source ontology. Each round-trip collapse is a candidate for a `ContextDef` resolution.
