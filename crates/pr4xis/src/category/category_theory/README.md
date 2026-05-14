# Category Theory

Meta-ontology grounding pr4xis's categorical substrate.

## Why this exists

pr4xis's Rust core defines trait and struct names (`Category`, `Arrow`, `Morphism`, `Functor`, `NaturalTransformation`, `Adjunction`, …) for the substrate that every domain ontology sits on. By pr4xis's own principle — "ALL code uses typed ontological concepts, never primitives" — those substrate types must themselves be grounded in an ontology. This module is that ontology.

Every name in `pr4xis::category::*` is an instance of a concept declared here, cited to primary literature.

## Synonymy — Morphism and Arrow

Mac Lane (1971) uses `morphism` and `arrow` interchangeably. Awodey (2010) uses `arrow` as primary. In pr4xis:

- **One concept**: `CategoryTheoryConcept::Morphism`
- **Two labels**: "Morphism" (Mac Lane primary) and "Arrow" (Awodey primary) — both point at the same concept. The Rust trait is `Arrow`; the Rust struct is `Morphism`. Both are instances of this concept.

The surface names differ so the trait and struct don't collide in a Rust module; ontologically they name the same thing.

## Concepts

| Concept | Source |
|---|---|
| Object | Mac Lane (1971) Ch. I §1 — 0-cell |
| Morphism | Mac Lane (1971) Ch. I §1 — 1-cell; Awodey (2010) primary "arrow" |
| Composition | Mac Lane (1971) Ch. I §1 |
| Identity | Mac Lane (1971) Ch. I §1 |
| Source, Target, Kind | OBO-RO (Smith et al. 2005); Mac Lane (1971) |
| Endomorphism, Isomorphism, Automorphism, Monomorphism, Epimorphism | Mac Lane (1971) Ch. I §5 |
| CategoryStructure | Mac Lane (1971) Ch. I §1 |
| Functor | Mac Lane (1971) Ch. II §1 |
| NaturalTransformation | Mac Lane (1971) Ch. II §4 |
| Adjunction, Unit, Counit | Mac Lane (1971) Ch. IV §1 |
| Bicategory | Bénabou (1967) |
| TwoCategory | Mac Lane (1971) XII.3 |
| HigherCategory | Leinster (2004) |

## Is-a structure

Specialised morphisms are morphisms; higher-dimensional cells are morphisms at their dimension:

- `Endomorphism`, `Isomorphism`, `Monomorphism`, `Epimorphism` → `Morphism`
- `Automorphism` → `Endomorphism`, `Isomorphism`
- `Functor` → `Morphism` (1-cell in Cat)
- `NaturalTransformation` → `Morphism` (2-cell in Cat)
- `Unit`, `Counit` → `NaturalTransformation`
- `TwoCategory`, `Bicategory` → `HigherCategory`

## Part-of structure

- `Morphism` has Source, Target, Kind
- `CategoryStructure` has Object, Morphism, Composition, Identity
- `Adjunction` has Unit, Counit
- `TwoCategory` has CategoryStructure, Functor, NaturalTransformation
