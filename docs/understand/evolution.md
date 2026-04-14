# Ontology Evolution

Knowledge changes. Scientific understanding deepens, terminology shifts, errors get corrected, fields merge. An ontology that cannot grow without breaking its existing users is a dead ontology. pr4xis treats evolution as a first-class concern, with a single rule:

> **Transform ontologies via functor. Never rewrite.**

This page explains what that means and why it works.

## The principle

When an ontology needs to change — concepts added, deprecated, split, merged, or refined — the change is expressed as a **functor** between the old version and the new one. The old ontology is not deleted; it is preserved alongside the new, and the functor is the mathematical proof that everything provable in the old version is still provable in the new one (or that specific things have been intentionally invalidated, with the ones that change made explicit).

Categorically, this is the same machinery pr4xis already uses for cross-domain composition. A cross-domain functor proves that domain A's structure embeds in domain B's. A cross-version functor proves that ontology version 1's structure embeds in version 2's. Both are first-class theorems verified at test time.

The alternative — silently rewriting an ontology in place — has two failure modes pr4xis avoids:
1. **Hidden breakage.** Existing tests that passed against version 1 silently start using version 2, and any change that broke a previously-true claim is invisible until something downstream fails.
2. **Lost provenance.** Once the old version is gone, there is no way to ask "what changed and why" — the audit trail is the diff in git, not a categorical proof object.

With the functor pattern, both problems are solved by construction: the old ontology stays compilable, the new one stays compilable, and the functor between them either passes the laws check or fails it explicitly.

## The patterns

Five common evolution operations, each expressed as a functor:

### Add a concept

The new ontology has all the old concepts plus the new one. The functor `Old → New` is the inclusion functor — every old concept maps to itself, every old morphism maps to itself. The functor laws hold trivially. Existing tests against the old ontology continue to pass; new tests against the new ontology can use the new concept.

### Refine a concept (add a quality, add an axiom)

The new ontology has the same concepts but with additional structure on one of them — a new `Quality` attached, a new structural axiom enforced. The functor `Old → New` is still inclusion, but downstream tests that depend on the new structure use the new ontology directly. Old tests are not affected unless they assumed the absence of the new structure.

### Deprecate a concept

Add a `#[deprecated]` annotation to the old concept and create a `New` version without it. The functor `Old → New` cannot be a total inclusion — the deprecated concept has nowhere to go in the new ontology. Two valid handlings:

- **Map to a placeholder.** The deprecated concept maps to a `Deprecated` entity in the new ontology that exists only to hold its identity. Useful when downstream code still references it by name.
- **Document the removal in the functor's domain restriction.** The functor is partial — it is defined only on the non-deprecated concepts. Tests that try to map the deprecated concept fail at compile or test time, surfacing the dependency.

### Split a concept (gap closure via context)

The pattern that the [gap detection result](../research/gap-detection.md) demonstrates concretely. A single concept in the old ontology turns out to encode two distinct things; the new ontology splits them. The mechanism is `ContextDef::resolve` — the old `Kv` becomes `Kv_in_Constitutive_Context` vs `Kv_in_Therapeutic_Context`, and a `ContextDef` resolves which one is meant in any given query. The functor from the old ontology to the new is no longer total — it requires a context to disambiguate. This is the only pattern in pr4xis where evolution requires changing the calling code, not just the ontology.

### Merge two concepts

The opposite of split. Two concepts in the old ontology turn out to be the same thing; the new ontology merges them. The functor `Old → New` collapses the two old concepts onto one new concept. Provable because category theory allows non-injective functors as long as the laws still hold.

## Why this matters

Most ontology systems treat a knowledge base as a single mutable artifact: edit in place, hope nothing downstream breaks, run tests, fix what surfaced. pr4xis cannot do that and stay true to its core promise (every claim is provable; nothing is guessed). If an ontology silently rewrote concepts in place, every downstream proof would be re-grounded against potentially-incompatible axioms with no record of what changed.

The functor pattern makes evolution **explicit, machine-checkable, and reversible**. Explicit because the functor must be written down. Machine-checkable because the functor laws must hold. Reversible because the old ontology is still there — you can always run a test against the previous version, or compose a chain of evolution functors and ask exactly what migrated where.

## Where evolution gets triggered

Three common triggers in practice:

1. **Gap detection surfaced a missing distinction.** The Molecular ⊣ Bioelectric adjunction collapsing 85.2% of entities revealed that `Kv` was conflating two roles. Evolution closed the gap with a `ContextDef` split. See [Gap detection in scientific ontologies](../research/gap-detection.md).
2. **A new source paper invalidates an old axiom.** Research updates the consensus; the ontology needs to follow. The new ontology uses the new axiom; the functor either preserves the old structure (if the new axiom is a refinement) or is partial (if the old axiom is now disproven and downstream code must update).
3. **Cross-domain composition reveals a name collision.** Two ontologies built independently use the same name for different things. The fix is a functor that renames one to disambiguate, then the composition can proceed.

## Where to look in the codebase

- `crates/domains/src/natural/biomedical/molecular/` — the `MolecularFunctionalContext` and `ContextDef::resolve` that closed the Kv gap
- `crates/pr4xis/src/category/validate/` — the `check_functor_laws` validator that every evolution functor must pass
- `crates/pr4xis/src/ontology/reasoning/context.rs` — the `ContextDef` trait that handles context-driven resolution

## Related

- [Concepts](concepts.md) — what an ontology is, why categories, how functors work
- [Architecture](architecture.md) — the five-layer Rust stack and the engine that runs these ontologies
- [Foundations](foundations.md) — academic lineage including Spivak's functorial data migration, which this pattern is modeled after
- [Gap detection](../research/gap-detection.md) — the bioelectricity result that demonstrates evolution closing a real gap
- [#46](https://github.com/i-am-logger/pr4xis/issues/46) — the issue that prompted this doc

---

- **Document date:** 2026-04-14
