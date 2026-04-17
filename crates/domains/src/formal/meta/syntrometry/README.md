# Syntrometry — Heim's syntrometric logic (Phase 1)

Encodes the core of Burkhard Heim's *Syntrometrische Maximentelezentrik* — the logical/philosophical foundation underneath Heim theory — as a pr4xis ontology, and verifies the long-standing claim that "pr4xis instantiates Heim's syntrometric structure" by a Functor whose laws are checked at test time.

Per `feedback_docs_need_proof.md`: the lineage claim was until now asserted in prose. This module turns it into a verified theorem.

## Verification — one command

```
cargo test -p pr4xis-domains -- formal::meta::syntrometry
```

Runs **25 tests**: category laws for both ontologies, all three domain axioms (single-point + proptest sweeps), forward functor laws (`lineage_functor_laws_pass`), adjunction unit/counit round-trips, gap analysis with measured collapse percentages, plus 12 proptest-based randomised sweeps over concepts, morphisms, and round-trips.

The headline — "pr4xis instantiates Heim's syntrometric structure" — is verified by `lineage_functor_laws_pass`.

## Measured loss profile (gap analysis)

`cargo test -p pr4xis-domains -- test_syntrometry_substrate_gaps_surface_missing_distinctions --nocapture`

| Direction | Loss | What collapses |
|---|---|---|
| **Unit** (Syntrometry → Substrate → Syntrometry) | **40%** (4/10) | `Dialektik → Syntrix`, `Aspekt → Syntrix`, `SyntrixLevel → Predicate`, `Part → Koordination` |
| **Counit** (Substrate → Syntrometry → Substrate) | **0%** (0/6) | none — substrate is closed under the round-trip |

The four unit collapses are the specific distinctions Heim carries that pr4xis's core substrate does not. They are actionable future work: add `OppositionCategory`, `ProductCategory`, `LeveledEntity`, and `MereologicalMorphism` sub-kinds to the substrate if the round-trip loss should drop.

## Phase 1 entities (16)

### Syntrometry (10)

| Family | Entities |
|---|---|
| Distinction primitives (5) | `Predicate`, `Predikatrix`, `Dialektik`, `Koordination`, `Aspekt` |
| Syntrometric structures (4) | `Syntrix`, `SyntrixLevel`, `Synkolator`, `Korporator` |
| Mereology (1) | `Part` |

### Pr4xis-substrate (6)

| Family | Entities |
|---|---|
| Core primitives (6) | `SubEntity`, `SubMorphism`, `SubCategory`, `SubFunctor`, `SubEndofunctor`, `SubOntology` |

## The lineage mapping (§1-§2 of the modernized paper)

| Syntrometry | Pr4xis substrate |
|---|---|
| `Predicate`     | `SubEntity` |
| `Predikatrix`   | `SubOntology` |
| `Dialektik`     | `SubCategory` |
| `Koordination`  | `SubMorphism` |
| `Aspekt`        | `SubCategory` (product [D × K × P]) |
| `Syntrix`       | `SubCategory` (C_SL, §2.2) |
| `SyntrixLevel`  | `SubEntity` |
| `Synkolator`    | `SubEndofunctor` |
| `Korporator`    | `SubFunctor` |
| `Part`          | `SubMorphism` |

Many-to-one collapses (`Dialektik/Aspekt/Syntrix → SubCategory`, `Predicate/SyntrixLevel → SubEntity`) are honest: pr4xis's substrate doesn't distinguish the subjective/objective/hierarchical flavours of leveled structures — they're all Categories from its vantage point. The composition law still holds because the substrate target is dense.

## Domain axioms (3)

| Axiom | Source | Claim |
|---|---|---|
| `AspektIsTripleProduct` | Heim §1 | Aspekt has `{Dialektik, Koordination, Predikatrix}` as direct parents |
| `SynkolatorIsKorporator` | Mac Lane Ch. II §1 | Endofunctor specialises functor (recorded structurally) |
| `SyntrixIsLeveled` | Heim §2.2 | Syntrix carries `LevelOf` and `InhabitsLevelOf` edges |

## Phase 2 (deferred)

The "metaphysical" concepts from Heim that `project_heim_transport.md` identifies as actually being **architectural**:

- **Telecenter** — goal-attractor / Eigenform / `CommunicativeGoal` / Colimit
- **Maxime** — BDI Intention / C1 Attention / Optimization
- **Transzendenzstufe** — Staging level / C1/C2 split / Metroplex grade

These require the C1/C2 consciousness ontology + BDI planning + Eigenform work that the cognitive tree already provides; Phase 2 will encode them and lift the lineage functor accordingly. Not blocking.

## Files

- `ontology.rs` — `SyntrometryOntology` + 3 domain axioms + qualities
- `substrate.rs` — `Pr4xisSubstrateOntology` (functor target)
- `lineage_functor.rs` — `SyntrometryToPr4xisSubstrate` + verification test
- `mod.rs` — module wiring
- `README.md` — this file
- `citings.md` — bibliography
