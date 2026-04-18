# Syntrometry — Heim's syntrometric logic

Encodes the core of Burkhard Heim's *Syntrometrische Maximentelezentrik* — the logical/philosophical foundation underneath Heim theory — as a pr4xis ontology, and verifies the long-standing claim that "pr4xis instantiates Heim's syntrometric structure" by a family of Functors whose laws are checked at test time.

Per `feedback_docs_need_proof.md`: the lineage claim was until now asserted in prose. This module turns it into a verified theorem — both structurally (all functor laws pass) and quantitatively (gap analysis measures exactly which distinctions collapse where).

## Verification — one command

```
cargo test -p pr4xis-domains -- formal::meta::syntrometry
```

40+ tests run: category laws for both ontologies, six domain axioms as single-point + proptest sweeps, six cross-functor law checks, adjunction round-trips, gap analysis with measured collapse percentages, and randomised proptest sweeps over concepts, morphisms, and round-trips.

## Entities

### Syntrometry (18)

| Family | Entities |
|---|---|
| Distinction primitives (5) | `Predicate`, `PredicateMatrix`, `Dialectic`, `Coordination`, `Aspect` |
| Syntrometric structures (4) | `Syntrix`, `SyntrixLevel`, `Syncolator`, `Composer` |
| Mereology (1) | `Part` |
| Teleological / hierarchical (4) | `Telecenter`, `Maxim`, `TranscendenceLevel`, `Metroplex` |
| Permutation operators (2) | `SequencePermutation` (C), `OrientationPermutation` (c) |
| Multi-aspect (1) | `AspectivalSystem` |
| Self-observation (1) | `Reflexivity` (ρ) |

> Heim's original German terms: `Predikatrix`, `Dialektik`, `Koordination`, `Aspekt`, `Synkolator`, `Korporator`, `Maxime`, `Transzendenzstufe`, `Aspektivsystem` — preserved in each concept's `labels:` entry and in the axiom citations.

### Pr4xis-substrate (14)

| Family | Entities |
|---|---|
| Core categorical primitives (6) | `SubEntity`, `SubMorphism`, `SubCategory`, `SubFunctor`, `SubEndofunctor`, `SubOntology` |
| Architectural primitives (4) | `SubEigenform`, `SubIntention`, `SubStagingLevel`, `SubSystemOfSystems` |
| Refined sub-kinds (3) | `SubProductCategory`, `SubGradedObject`, `SubObject` |
| Natural transformation (1) | `SubNaturalTransformation` |

## Lineage mappings (eight verified functors)

### Primary: `Syntrometry → Pr4xisSubstrate`

14 of 18 concepts round-trip cleanly. Four concepts — `Dialectic`, `SequencePermutation`, `OrientationPermutation`, `AspectivalSystem` — intentionally collapse to a substrate parent because their richer semantics lives in dedicated cross-functor targets (Dialectics, Kripke).

| Syntrometry | Pr4xis substrate |
|---|---|
| `Predicate`     | `SubEntity` |
| `PredicateMatrix`   | `SubOntology` |
| `Dialectic`     | `SubCategory` (collapses — handled by Dialectics cross-functor) |
| `Coordination`  | `SubMorphism` |
| `Aspect`        | `SubProductCategory` |
| `Syntrix`       | `SubCategory` |
| `SyntrixLevel`  | `SubGradedObject` |
| `Syncolator`    | `SubEndofunctor` |
| `Composer`    | `SubFunctor` |
| `Part`          | `SubObject` |
| `Telecenter`    | `SubEigenform` |
| `Maxim`        | `SubIntention` |
| `TranscendenceLevel` | `SubStagingLevel` |
| `Metroplex`     | `SubSystemOfSystems` |
| `SequencePermutation` | `SubEndofunctor` (collapses with Syncolator) |
| `OrientationPermutation` | `SubEndofunctor` (collapses) |
| `AspectivalSystem` | `SubOntology` (collapses — handled by Kripke cross-functor) |
| `Reflexivity`   | `SubNaturalTransformation` |

Gap analysis: **4 intentional unit collapses out of 18, 0% counit loss** — verified by `test_syntrometry_substrate_intentional_collapses`. Each collapsed concept is preserved at full resolution by a dedicated cross-functor.

### Cross-functors into existing pr4xis ontologies

Each of the seven cross-functors below passes `check_functor_laws` and carries its own collapse profile, demonstrating that Heim's vocabulary aligns with pr4xis's meta-ontology, composition, staging, cognitive, logical, and modal layers. The last row — `Distinction → Syntrometry` — runs in the historical direction (Spencer-Brown 1969 → Heim, embedding the earlier distinction calculus into the richer syntrometric vocabulary):

| Functor | Target | Headline mapping |
|---|---|---|
| `Syntrometry → Dialectics` | `formal::logic::dialectics` (Hegel / Aristotle / Priest) | `Dialectic` ↦ `DialecticalMoment`, `Syncolator` ↦ `Sublation`, `Telecenter` ↦ `Synthesis` |
| `Syntrometry → Kripke` | `formal::logic::kripke` (Kripke 1959/63) | `Aspect` ↦ `KripkeFrame`, `AspectivalSystem` ↦ `AccessibilityRelation`, `Reflexivity` ↦ `Reflexive` |
| `Syntrometry → MetaOntology` | `formal::meta::ontology_diagnostics` | `Syncolator` / `Composer` ↦ `Functor`, `Maxim` ↦ `PropertyTest` |
| `Syntrometry → Staging` | `formal::meta::staging` (Futamura 1971) | `TranscendenceLevel` ↦ `Interpreter`, `Metroplex` ↦ `CompilerGenerator` |
| `Syntrometry → Algebra` | `formal::meta::algebra` (Goguen / Zimmermann) | `Composer` ↦ `Mapping`, `Aspect` ↦ `Product`, `Telecenter` ↦ `Pushout` |
| `Syntrometry → C1` | `cognitive::cognition::consciousness` (Dehaene GWT 2014) | `Maxim` ↦ `Attention`, `Metroplex` ↦ `GlobalWorkspace` |
| `Distinction → Syntrometry` | — (historical direction, Spencer-Brown 1969 → Heim) | `ReEntry` ↦ `Syncolator` |

The `Syntrometry → C1` mapping is the most load-bearing interpretive claim: Heim's `Maxim` (Heim's *Maxime*, an extremal-selection operator) and Dehaene's GWT `Attention` (spotlight selection) are structurally the same morphism, landing on C1's declared `(Attention, ConsciousAccess, Selects)` edge. Heim anticipated the attention/workspace split Dehaene's GWT formalises 34 years later.

## Domain axioms (6)

| Axiom | Source | Claim |
|---|---|---|
| `AspectIsTripleProduct` | Heim §1 | Aspect mereologically contains `{Dialectic, Coordination, PredicateMatrix}` |
| `SyncolatorIsComposer` | Mac Lane Ch. II §1 | Endofunctor specialises functor |
| `SyntrixIsLeveled` | Heim §2.2 | Syntrix carries `LevelOf` and `InhabitsLevelOf` edges |
| `MetroplexContainsSyntrixAndLevels` | Heim Metroplextheorie | Metroplex mereologically contains `{Syntrix, TranscendenceLevel}` |
| `MaximConvergesTowardTelecenter` | Heim Telezentrik | Maxim carries `ConvergesToward` edge into Telecenter |
| `TelecenterIsSyncolatorFixedPoint` | Heim × Mac Lane | Syncolator carries `FixedPointOf` edge into Telecenter (eigenform X = F(X)) |

## Files

- `ontology.rs` — `SyntrometryOntology` + the six domain axioms + qualities
- `substrate.rs` — `Pr4xisSubstrateOntology` (functor target, mirrors pr4xis core + architectural primitives)
- `lineage_functor.rs` — the primary `Syntrometry → Pr4xisSubstrate` functor + verification test
- `substrate_functor.rs` — reverse object map (feeds gap analysis)
- `adjunction.rs` — `unit_pair` / `counit_pair` helpers + round-trip tests
- `meta_ontology_functor.rs`, `staging_functor.rs`, `algebra_functor.rs`, `consciousness_functor.rs`, `distinction_functor.rs` — cross-functors into existing pr4xis ontologies
- `proptests.rs` — proptest sweeps for every functor + axiom
- `README.md`, `citings.md` — documentation + bibliography
