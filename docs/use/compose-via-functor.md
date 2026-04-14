# Compose Two Ontologies via Functor

This page is the practical guide for writing a cross-domain functor in pr4xis — the mechanism that lets two ontologies *compose* with mathematical proof that the composition is sound.

## When you need a functor

If you have an ontology that uses concepts from another ontology — explicitly or implicitly — you should write a functor between them. Examples from the existing workspace:

- Pharmacology talks about molecular targets → `PharmacologyToMolecular`
- Biology talks about bioelectric phenomena → `BiologyToBioelectric`
- Concurrency talks about events → `ConcurrencyToEvents`
- Chess talks about state machines → `ChessToConcurrency`, `ChessToEvents`

The functor makes the implicit dependency explicit, and proves at test time that the dependency preserves structure.

## What a functor must do

A functor `F: Source → Target` is a Rust impl of `pr4xis::category::Functor`. It must:

1. **Map every object** in the source category to an object in the target category.
2. **Map every morphism** in the source category to a morphism in the target category, with matching source and target.
3. **Preserve identities**: `F(id_x) = id_{F(x)}`.
4. **Preserve composition**: `F(g ∘ f) = F(g) ∘ F(f)`.

If the laws hold, the functor is a categorical theorem that the source domain's structure faithfully embeds in the target. If they don't hold, your encoding has a bug or the composition you proposed isn't actually structural — either way, the failing test surfaces it before the functor ships.

## The pattern

Skeleton for a functor between two existing ontologies:

```rust
use pr4xis::category::{Functor, Relationship};

use crate::domain_a::{ACategory, AEntity, ARelation};
use crate::domain_b::{BCategory, BEntity, BRelation};

pub struct AToB;

impl Functor for AToB {
    type Source = ACategory;
    type Target = BCategory;

    fn map_object(obj: &AEntity) -> BEntity {
        match obj {
            AEntity::Foo => BEntity::CorrespondingFoo,
            AEntity::Bar => BEntity::CorrespondingBar,
            // ... one arm per source entity
        }
    }

    fn map_morphism(m: &ARelation) -> BRelation {
        BRelation {
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
    fn test_a_to_b_functor_laws() {
        check_functor_laws::<AToB>().unwrap();
    }
}
```

That's it. The validator iterates through every source entity, maps it, maps the identity morphism, and checks that `F(id_x) == id_{F(x)}`. Then it iterates through every morphism, maps it, and checks that the source and target of the mapped morphism match the mapped source and mapped target.

If the test passes, the functor is verified. If it fails, the error message names the specific entity or morphism where the laws break, and you fix the `map_object` arm that's wrong.

## Three things to know

### 1. `map_object` must be total

Every variant of `AEntity` must have a corresponding case in the match. The Rust compiler enforces this — exhaustiveness checking is your friend. If the source ontology adds a new entity, the compiler tells you to add a new case to every functor that uses it. This is one of the safety guarantees of writing functors as Rust types instead of as runtime mappings.

### 2. `map_object` does not need to be injective

Two distinct source entities can map to the same target entity. This is how the molecular-bioelectric functor collapses 27 molecular entities onto 4 unique bioelectric entities (the 85.2% collapse from [Gap detection](../research/gap-detection.md)). The collapse is not a bug — it's a *measurement* of how much information the target ontology cannot represent.

If you want to detect collapses, pair your forward functor with a reverse functor and check whether they form an [adjunction](../reference/glossary.md#adjunction). The round-trip `G(F(x))` will collapse onto a different entity for every concept the source ontology distinguishes that the target cannot.

### 3. The functor is a *theorem*, not a translation

A functor is not just "convert objects from A to B". It is the claim that the conversion preserves structure. If you can write `map_object` correctly but `check_functor_laws` fails, you have not proven the functor — the conversion does not actually preserve composition or identity, which means the source domain's structure is *not* faithfully present in the target. The failing test is telling you the proposed embedding doesn't hold.

When that happens, two paths forward:

- **Find a different target.** Maybe the source belongs in a different ontology, with different morphism structure.
- **Restrict the source.** Maybe only a subset of the source's morphisms map cleanly. Write the functor over the restricted source category; document what's left out and why.

Either is fine. The wrong move is to fudge the `map_object` to make the test pass — that hides the structural mismatch instead of surfacing it.

## When you also need an adjunction

If you have two functors `F: A → B` and `G: B → A` going in opposite directions, you may have an adjunction. Adjunctions enable [gap detection](../research/gap-detection.md) — the round-trip `G(F(x))` surfaces concepts that one side cannot represent.

To check whether your functor pair is an adjunction, implement the `Adjunction` trait:

```rust
use pr4xis::category::Adjunction;

pub struct ABAdjunction;

impl Adjunction for ABAdjunction {
    type Left = AToB;
    type Right = BToA;

    fn unit(obj: &AEntity) -> ARelation {
        // η_A: A → G(F(A))
        let round_trip = BToA::map_object(&AToB::map_object(obj));
        ARelation { from: *obj, to: round_trip }
    }

    fn counit(obj: &BEntity) -> BRelation {
        // ε_B: F(G(B)) → B
        let round_trip = AToB::map_object(&BToA::map_object(obj));
        BRelation { from: round_trip, to: *obj }
    }
}
```

Then run the gap-analysis pattern from `crates/domains/src/formal/meta/gap_analysis.rs` against your adjunction. The collapses the analysis surfaces are missing distinctions in your source ontology — fix them with `ContextDef::resolve` or by splitting the entity, and the gap closes.

## Where to look in the codebase

- `crates/domains/src/natural/biomedical/adjunctions.rs` — the three biomedical adjunctions, with `unit` and `counit` implementations and the full law-checking test suite
- `crates/domains/src/natural/biomedical/biochemistry/bioelectricity_functor.rs` — a clean single-functor example, no adjunction
- `crates/pr4xis/src/category/validate/check_functor_laws.rs` — the validator
- `crates/domains/src/formal/meta/gap_analysis.rs` — the gap-analysis pattern that uses the adjunctions

## Related

- [Build an ontology from a paper](build-ontology-from-paper.md) — the upstream tutorial; if you are writing a functor, you have probably already authored both source and target ontologies
- [Write axioms](write-axioms.md) — domain-specific axioms that go beyond the structural laws functors enforce
- [Concepts](../understand/concepts.md) — the categorical machinery, with examples
- [Gap detection](../research/gap-detection.md) — the bioelectricity result, the canonical example of an adjunction surfacing a real ontological gap
- [Glossary](../reference/glossary.md#functor) — formal definition of a functor

---

- **Document date:** 2026-04-14
