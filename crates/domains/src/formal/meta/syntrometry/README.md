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

`cargo test -p pr4xis-domains -- test_syntrometry_substrate_gaps_surface_missing_distinctions --nocapture`

| Direction | Loss | What collapses |
|---|---|---|
| **Unit** (Syntrometry → Substrate → Syntrometry) | **28.6%** (4/14) | `Dialektik → Syntrix`, `Aspekt → Syntrix`, `SyntrixLevel → Predicate`, `Part → Koordination` |
| **Counit** (Substrate → Syntrometry → Substrate) | **0%** (0/10) | none — substrate is closed under the round-trip |

Phase 2's four new concepts (Telecenter/Maxime/Transzendenzstufe/Metroplex) all round-trip cleanly because Phase 2 added matching substrate targets (`SubEigenform`/`SubIntention`/`SubStagingLevel`/`SubSystemOfSystems`).

The four remaining unit collapses are the specific distinctions Heim's vocabulary carries that pr4xis's core substrate still does not. Phase 3 follow-up: either add `OppositionCategory`/`ProductCategory` sub-kinds to reduce collapse, or accept that these four are compression the substrate makes deliberately.

## Phase 1 entities (Phase 1 + Phase 2 = 14 Syntrometry + 10 Substrate)

### Syntrometry (14)

| Family | Entities |
|---|---|
| Distinction primitives (5) | `Predicate`, `Predikatrix`, `Dialektik`, `Koordination`, `Aspekt` |
| Syntrometric structures (4) | `Syntrix`, `SyntrixLevel`, `Synkolator`, `Korporator` |
| Mereology (1) | `Part` |
| **Teleological / hierarchical (Phase 2) (4)** | `Telecenter`, `Maxime`, `Transzendenzstufe`, `Metroplex` |

### Pr4xis-substrate (10)

| Family | Entities |
|---|---|
| Core categorical primitives (6) | `SubEntity`, `SubMorphism`, `SubCategory`, `SubFunctor`, `SubEndofunctor`, `SubOntology` |
| **Architectural primitives (Phase 2) (4)** | `SubEigenform`, `SubIntention`, `SubStagingLevel`, `SubSystemOfSystems` |

## The lineage mapping

| Syntrometry | Pr4xis substrate | Interpretation |
|---|---|---|
| `Predicate`     | `SubEntity` | atomic distinction = Entity variant |
| `Predikatrix`   | `SubOntology` | predicate-system = small ontology |
| `Dialektik`     | `SubCategory` | binary-opposition structure |
| `Koordination`  | `SubMorphism` | ordering between predicates = morphism |
| `Aspekt`        | `SubCategory` | product [D × K × P] = product category |
| `Syntrix`       | `SubCategory` | C_SL (§2.2 — Category of Leveled Structures) |
| `SyntrixLevel`  | `SubEntity` | single level = object in the category |
| `Synkolator`    | `SubEndofunctor` | endofunctor on the Syntrix |
| `Korporator`    | `SubFunctor` | structure-mapping functor |
| `Part`          | `SubMorphism` | mereological relation = morphism |
| **`Telecenter`** | **`SubEigenform`** | **goal-attractor = X=F(X) / CommunicativeGoal / Colimit** |
| **`Maxime`**    | **`SubIntention`** | **extremal selection = BDI Intention / C1 Attention** |
| **`Transzendenzstufe`** | **`SubStagingLevel`** | **transcendence-level = Staging grade / C1-vs-C2 split** |
| **`Metroplex`** | **`SubSystemOfSystems`** | **hierarchical container = SoS composition (#94)** |

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
