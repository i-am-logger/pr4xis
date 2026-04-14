---
name: functor-author
description: Given two pr4xis ontologies, scaffold a Functor impl that maps the source ontology's entities to the target's, with map_morphism and the check_functor_laws test. Produces a draft for human review — the user fills in the cases the skill can't infer with confidence.
---

# functor-author

Scaffold a `Functor` impl between two pr4xis ontologies and the test that verifies its laws hold.

## When to invoke

When you have two ontologies that share concepts (e.g., the new domain you just authored has a `Cell` concept that exists in the biology ontology) and you want to prove the structural correspondence with a verified functor. Or as part of [`ontology-from-paper`](../ontology-from-paper/SKILL.md), which calls this skill for every plausible cross-domain link in a newly created ontology.

## Inputs

- **Required**: source ontology path (e.g., `crates/domains/src/natural/biomedical/molecular/`)
- **Required**: target ontology path (e.g., `crates/domains/src/natural/biomedical/bioelectricity/`)
- **Optional**: a name for the functor (default: `<Source>To<Target>`, e.g., `MolecularToBioelectric`)
- **Optional**: a hint mapping for entities the human already knows the answer for

## What to read

1. Both ontologies' `Entity` enums and `Relation` types
2. Both ontologies' `mod.rs` to confirm the public exports
3. Any existing functor impls between these or related ontologies — to find naming conventions and patterns
4. Any source paper notes / doc comments that hint at the conceptual mapping

## What to generate

A new file at `<source-ontology-dir>/<target_short_name>_functor.rs` with:

```rust
//! Functor: <SourceCategory> → <TargetCategory>
//!
//! <One sentence describing what this functor expresses about the
//! relationship between the two domains.>

use pr4xis::category::{Functor, Relationship};

use crate::<source-path>::ontology::{<SourceCategory>, <SourceEntity>, <SourceRelation>};
use crate::<target-path>::ontology::{<TargetCategory>, <TargetEntity>, <TargetRelation>};

pub struct <FunctorName>;

impl Functor for <FunctorName> {
    type Source = <SourceCategory>;
    type Target = <TargetCategory>;

    fn map_object(obj: &<SourceEntity>) -> <TargetEntity> {
        match obj {
            // Inferred mappings (high confidence — same concept name in both ontologies):
            <SourceEntity>::<X> => <TargetEntity>::<X>,
            // ...

            // Inferred mappings (medium confidence — name similarity):
            <SourceEntity>::<Y> => <TargetEntity>::<RelatedY>,  // TODO verify
            // ...

            // Cases the skill could not infer — human must fill in:
            <SourceEntity>::<Z> => todo!("map Z to a target entity"),
            // ...
        }
    }

    fn map_morphism(m: &<SourceRelation>) -> <TargetRelation> {
        <TargetRelation> {
            from: Self::map_object(&m.source()),
            to: Self::map_object(&m.target()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn test_functor_laws() {
        check_functor_laws::<<FunctorName>>().unwrap();
    }
}
```

Plus an entry in `<source-ontology-dir>/mod.rs` that exposes the new module.

## Inference rules for `map_object`

For each `SourceEntity` variant, try in order:

1. **Exact name match**: if `TargetEntity` has a variant with the identical name, use it (high confidence)
2. **Substring match**: if a `TargetEntity` variant contains or is contained by the source variant name (e.g., `Calcium` → `CalciumIon`), use it with a `// TODO verify` comment (medium confidence)
3. **Domain-knowledge match**: if the source paper or doc comments hint at the mapping, use it with the source cited in a comment (medium confidence)
4. **No match**: emit `todo!("map <variant> to a target entity")` and surface in the report (zero confidence — human required)

## Verification

1. The new file compiles — `cargo check -p pr4xis-domains`
2. The `check_functor_laws` test passes — `cargo test -p pr4xis-domains <functor-name>`
3. If it fails: report which entity or morphism breaks the laws and stop. Do not auto-fix; the structural mismatch may be real.

## Output

Report:
- Path of the new functor file
- Number of entities mapped at each confidence level (high / medium / TODO)
- Whether the functor laws check passed or failed
- For each `todo!`, the source entity name and a one-line guess (if any) at what it might map to

## Failure modes

- **The `Entity` enums use generic types or attributes the skill can't parse**: stop and ask
- **Source entity has many variants and target has very few**: map_object will be lossy by construction. This is fine — many functors are non-injective. Just emit the mappings, the laws check will validate
- **The functor laws check fails after mapping**: report the failing case (which morphism breaks composition or identity) and stop. The mapping needs human revision

## Notes

This skill produces a *draft*. The `todo!` cases must be filled in by a human with domain knowledge before the file ships. Even the "high confidence" inferred mappings should be reviewed — name similarity is not always semantic similarity.
