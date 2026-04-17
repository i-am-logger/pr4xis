# Syntrometry — Heim's syntrometric logic

Encodes the core of Burkhard Heim's *Syntrometrische Maximentelezentrik* — the logical/philosophical foundation underneath Heim theory — as a pr4xis ontology, and verifies the long-standing claim that "pr4xis instantiates Heim's syntrometric structure" by a family of Functors whose laws are checked at test time.

Per `feedback_docs_need_proof.md`: the lineage claim was until now asserted in prose. This module turns it into a verified theorem — both structurally (all functor laws pass) and quantitatively (gap analysis measures exactly which distinctions collapse where).

## Verification — one command

```
cargo test -p pr4xis-domains -- formal::meta::syntrometry
```

40+ tests run: category laws for both ontologies, six domain axioms as single-point + proptest sweeps, six cross-functor law checks, adjunction round-trips, gap analysis with measured collapse percentages, and randomised proptest sweeps over concepts, morphisms, and round-trips.

## Entities

### Syntrometry (14)

| Family | Entities |
|---|---|
| Distinction primitives (5) | `Predicate`, `Predikatrix`, `Dialektik`, `Koordination`, `Aspekt` |
| Syntrometric structures (4) | `Syntrix`, `SyntrixLevel`, `Synkolator`, `Korporator` |
| Mereology (1) | `Part` |
| Teleological / hierarchical (4) | `Telecenter`, `Maxime`, `Transzendenzstufe`, `Metroplex` |

### Pr4xis-substrate (14)

| Family | Entities |
|---|---|
| Core categorical primitives (6) | `SubEntity`, `SubMorphism`, `SubCategory`, `SubFunctor`, `SubEndofunctor`, `SubOntology` |
| Architectural primitives (4) | `SubEigenform`, `SubIntention`, `SubStagingLevel`, `SubSystemOfSystems` |
| Refined sub-kinds (3) | `SubProductCategory`, `SubGradedObject`, `SubObject` |
| Natural transformation (1) | `SubNaturalTransformation` |

## Lineage mappings (six verified functors)

### Primary: `Syntrometry → Pr4xisSubstrate`

13 of 14 concepts round-trip cleanly. `Dialektik` intentionally collapses to `SubCategory` because opposition-structure is carried by the dedicated [Dialectics](../../logic/dialectics/) ontology (Aristotle / Hegel / Marx / Adorno / Priest), not by the core substrate. Reach it via the `SyntrometryToDialectics` cross-functor.

| Syntrometry | Pr4xis substrate |
|---|---|
| `Predicate`     | `SubEntity` |
| `Predikatrix`   | `SubOntology` |
| `Dialektik`     | `SubCategory` (collapses — handled by Dialectics cross-functor) |
| `Koordination`  | `SubMorphism` |
| `Aspekt`        | `SubProductCategory` |
| `Syntrix`       | `SubCategory` |
| `SyntrixLevel`  | `SubLeveledEntity` |
| `Synkolator`    | `SubEndofunctor` |
| `Korporator`    | `SubFunctor` |
| `Part`          | `SubMereologicalMorphism` |
| `Telecenter`    | `SubEigenform` |
| `Maxime`        | `SubIntention` |
| `Transzendenzstufe` | `SubStagingLevel` |
| `Metroplex`     | `SubSystemOfSystems` |

Gap analysis: **~7% unit loss (1/14 — Dialektik), 0% counit loss** — verified by `test_syntrometry_substrate_dialektik_collapses`. The one collapse is intentional and preserved at full resolution by the Dialectics cross-functor.

### Cross-functors into existing pr4xis ontologies

Each of the five cross-functors below passes `check_functor_laws` and carries its own collapse profile, demonstrating that Heim's vocabulary aligns with pr4xis's meta-ontology, composition, staging, and cognitive layers:

| Functor | Target | Headline mapping |
|---|---|---|
| `Syntrometry → Dialectics` | `formal::logic::dialectics` (Hegel / Aristotle / Priest) | `Dialektik` ↦ `DialecticalMoment`, `Synkolator` ↦ `Sublation`, `Telecenter` ↦ `Synthesis` |
| `Syntrometry → MetaOntology` | `formal::meta::ontology_diagnostics` | `Synkolator` / `Korporator` ↦ `Functor`, `Maxime` ↦ `PropertyTest` |
| `Syntrometry → Staging` | `formal::meta::staging` (Futamura 1971) | `Transzendenzstufe` ↦ `Interpreter`, `Metroplex` ↦ `CompilerGenerator` |
| `Syntrometry → Algebra` | `formal::meta::algebra` (Goguen / Zimmermann) | `Korporator` ↦ `Mapping`, `Aspekt` ↦ `Product`, `Telecenter` ↦ `Pushout` |
| `Syntrometry → C1` | `cognitive::cognition::consciousness` (Dehaene GWT 2014) | `Maxime` ↦ `Attention`, `Metroplex` ↦ `GlobalWorkspace` |
| `Distinction → Syntrometry` | — (historical direction, Spencer-Brown 1969 → Heim) | `ReEntry` ↦ `Synkolator` |

The `Syntrometry → C1` mapping is the most load-bearing interpretive claim: Heim's `Maxime` (extremal-selection operator) and Dehaene's GWT `Attention` (spotlight selection) are structurally the same morphism, landing on C1's declared `(Attention, ConsciousAccess, Selects)` edge. Heim anticipated the attention/workspace split Dehaene's GWT formalises 34 years later.

## Domain axioms (6)

| Axiom | Source | Claim |
|---|---|---|
| `AspektIsTripleProduct` | Heim §1 | Aspekt mereologically contains `{Dialektik, Koordination, Predikatrix}` |
| `SynkolatorIsKorporator` | Mac Lane Ch. II §1 | Endofunctor specialises functor |
| `SyntrixIsLeveled` | Heim §2.2 | Syntrix carries `LevelOf` and `InhabitsLevelOf` edges |
| `MetroplexContainsSyntrixAndLevels` | Heim Metroplextheorie | Metroplex mereologically contains `{Syntrix, Transzendenzstufe}` |
| `MaximeConvergesTowardTelecenter` | Heim Telezentrik | Maxime carries `ConvergesToward` edge into Telecenter |
| `TelecenterIsSynkolatorFixedPoint` | Heim × Mac Lane | Synkolator carries `FixedPointOf` edge into Telecenter (eigenform X = F(X)) |

## Files

- `ontology.rs` — `SyntrometryOntology` + the six domain axioms + qualities
- `substrate.rs` — `Pr4xisSubstrateOntology` (functor target, mirrors pr4xis core + architectural primitives)
- `lineage_functor.rs` — the primary `Syntrometry → Pr4xisSubstrate` functor + verification test
- `substrate_functor.rs` — reverse object map (feeds gap analysis)
- `adjunction.rs` — `unit_pair` / `counit_pair` helpers + round-trip tests
- `meta_ontology_functor.rs`, `staging_functor.rs`, `algebra_functor.rs`, `consciousness_functor.rs`, `distinction_functor.rs` — cross-functors into existing pr4xis ontologies
- `proptests.rs` — proptest sweeps for every functor + axiom
- `README.md`, `citings.md` — documentation + bibliography
