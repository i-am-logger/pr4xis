# Syntrometry — Heim's syntrometric logic (Phases 1 + 2)

Encodes the core of Burkhard Heim's *Syntrometrische Maximentelezentrik* — the logical/philosophical foundation underneath Heim theory — as a pr4xis ontology, and verifies the long-standing claim that "pr4xis instantiates Heim's syntrometric structure" by a Functor whose laws are checked at test time.

Per `feedback_docs_need_proof.md`: the lineage claim was until now asserted in prose. This module turns it into a verified theorem — both structurally (the functor laws pass) and quantitatively (gap analysis measures exactly which distinctions Heim carries that pr4xis does not).

## Verification — one command

```
cargo test -p pr4xis-domains -- formal::meta::syntrometry
```

Runs **28+ tests**: category laws for both ontologies, six domain axioms (Phase 1 + Phase 2) as single-point + proptest sweeps, forward functor laws (`lineage_functor_laws_pass`), adjunction unit/counit round-trips, gap analysis with measured collapse percentages, plus 12 proptest-based randomised sweeps over concepts, morphisms, and round-trips.

The headline — "pr4xis instantiates Heim's syntrometric structure" — is verified by `lineage_functor_laws_pass`.

## Measured loss profile (gap analysis)

`cargo test -p pr4xis-domains -- test_syntrometry_substrate_is_object_equivalence --nocapture`

| Direction | Loss | What collapses |
|---|---|---|
| **Unit** (Syntrometry → Substrate → Syntrometry) | **0%** (0/14) | none — object-level equivalence |
| **Counit** (Substrate → Syntrometry → Substrate) | **0%** (0/14) | none — substrate is closed under the round-trip |

Phase 3 closed the remaining 28.6% loss by adding four sub-kinds to `Pr4xisSubstrate` — `SubOppositionCategory`, `SubProductCategory`, `SubLeveledEntity`, `SubMereologicalMorphism` — so `Dialektik`, `Aspekt`, `SyntrixLevel`, and `Part` each land at a distinct substrate target. The lineage is now an **object-level equivalence**: every Heim concept has a unique pr4xis-substrate counterpart and every substrate primitive has a unique Heim representative.

Each phase's incremental progress, for the record:

| Phase | Unit loss | Counit loss |
|---|---|---|
| 1 | 40% (4/10) | 0% (0/6) |
| 2 | 28.6% (4/14) | 0% (0/10) |
| 3 | **0% (0/14)** | **0% (0/14)** |

## Phase 1 entities (Phase 1 + Phase 2 = 14 Syntrometry + 10 Substrate)

### Syntrometry (14)

| Family | Entities |
|---|---|
| Distinction primitives (5) | `Predicate`, `Predikatrix`, `Dialektik`, `Koordination`, `Aspekt` |
| Syntrometric structures (4) | `Syntrix`, `SyntrixLevel`, `Synkolator`, `Korporator` |
| Mereology (1) | `Part` |
| **Teleological / hierarchical (Phase 2) (4)** | `Telecenter`, `Maxime`, `Transzendenzstufe`, `Metroplex` |

### Pr4xis-substrate (14)

| Family | Entities |
|---|---|
| Core categorical primitives (6) | `SubEntity`, `SubMorphism`, `SubCategory`, `SubFunctor`, `SubEndofunctor`, `SubOntology` |
| Architectural primitives (Phase 2) (4) | `SubEigenform`, `SubIntention`, `SubStagingLevel`, `SubSystemOfSystems` |
| **Refined sub-kinds (Phase 3) (4)** | `SubOppositionCategory`, `SubProductCategory`, `SubLeveledEntity`, `SubMereologicalMorphism` |

## The lineage mapping (object-level equivalence after Phase 3)

| Syntrometry | Pr4xis substrate | Interpretation |
|---|---|---|
| `Predicate`     | `SubEntity` | atomic distinction = Entity variant |
| `Predikatrix`   | `SubOntology` | predicate-system = small ontology |
| `Dialektik`     | `SubOppositionCategory` | binary-opposition structure (Phase 3) |
| `Koordination`  | `SubMorphism` | ordering between predicates = morphism |
| `Aspekt`        | `SubProductCategory` | product [D × K × P] = product category (Phase 3) |
| `Syntrix`       | `SubCategory` | C_SL (§2.2 — Category of Leveled Structures) |
| `SyntrixLevel`  | `SubLeveledEntity` | grade-indexed entity (Phase 3) |
| `Synkolator`    | `SubEndofunctor` | endofunctor on the Syntrix |
| `Korporator`    | `SubFunctor` | structure-mapping functor |
| `Part`          | `SubMereologicalMorphism` | CEM-satisfying morphism (Phase 3) |
| `Telecenter`    | `SubEigenform` | goal-attractor = X=F(X) (Phase 2) |
| `Maxime`        | `SubIntention` | extremal selection = BDI Intention (Phase 2) |
| `Transzendenzstufe` | `SubStagingLevel` | transcendence-level (Phase 2) |
| `Metroplex`     | `SubSystemOfSystems` | hierarchical container (Phase 2) |

Bijection: every Heim concept has a unique substrate target; every substrate primitive has a unique Heim representative. Verified by `test_syntrometry_substrate_is_object_equivalence`.

## Domain axioms (6)

| Axiom | Source | Claim |
|---|---|---|
| `AspektIsTripleProduct` | Heim §1 | Aspekt mereologically contains `{Dialektik, Koordination, Predikatrix}` |
| `SynkolatorIsKorporator` | Mac Lane Ch. II §1 | Endofunctor specialises functor (structural) |
| `SyntrixIsLeveled` | Heim §2.2 | Syntrix carries `LevelOf` and `InhabitsLevelOf` edges |
| `MetroplexContainsSyntrixAndLevels` | Heim Metroplextheorie | Metroplex mereologically contains `{Syntrix, Transzendenzstufe}` |
| `MaximeConvergesTowardTelecenter` | Heim Telezentrik | Maxime carries `ConvergesToward` edge into Telecenter |
| `TelecenterIsSynkolatorFixedPoint` | Heim Telezentrik × Mac Lane | Synkolator carries `FixedPointOf` edge into Telecenter (eigenform) |

## Phase 3 (future)

Reducing the 28.6% unit loss by enriching the substrate with the four missing distinctions:

- `SubOppositionCategory` to receive `Dialektik` without collapse
- `SubProductCategory` to receive `Aspekt` without collapse
- `SubLeveledEntity` / `SubGradedObject` to receive `SyntrixLevel` without collapse
- `SubMereologicalMorphism` to receive `Part` without collapse

Each would be a judgement call — the collapse is honest information compression, and adding sub-kinds only pays off if downstream ontologies need them.

## Files

- `ontology.rs` — `SyntrometryOntology` (14 concepts) + 6 domain axioms + qualities
- `substrate.rs` — `Pr4xisSubstrateOntology` (10 concepts, functor target)
- `lineage_functor.rs` — `SyntrometryToPr4xisSubstrate` + verification test
- `substrate_functor.rs` — `map_substrate` reverse object map (for gap analysis)
- `adjunction.rs` — `unit_pair` / `counit_pair` helpers + round-trip tests
- `proptests.rs` — 12 proptest sweeps over concepts, morphisms, axioms, round-trips
- `mod.rs` — module wiring
- `README.md` — this file
- `citings.md` — bibliography
