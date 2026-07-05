# SmartElement — the autonomic edge element

A **smart element** is the synthesis of three established ideas into one edge component:

- an **autonomic element** — a managed element governed by an autonomic manager that closes a MAPE-K loop over a knowledge base, exhibiting the four self-* properties (Kephart & Chess 2003, *The Vision of Autonomic Computing*, IEEE Computer 36(1));
- a **smart transducer** — a physical Transducer plus a Network Capable Application Processor (NCAP), carrying a self-describing Transducer Electronic Data Sheet (TEDS) (IEEE Std 1451.0-2007; Lee 2000);
- a **fusion peer** — a node that signs and gossips a local state estimate and takes part in consensus-on-information, excluding equivocators (Olfati-Saber, Fax & Murray 2007; Lamport, Shostak & Pease 1982; Li et al. 2004 SUNDR).

The autonomic loop and the estimation math are established literature — this module encodes them and claims no novelty for them. **The only novelty claimed is the ontological synthesis**: an autonomic driver that is *simultaneously* a MAPE-K element and a signed-estimate fusion peer, with fusion consistency expressed as cited axioms, `no_std` at the edge. `SmartElement`, `SmartSensor`, and `SmartDriver` are this codebase's synthesis concepts; their glosses say so and ground the parts.

## Verification

```
cargo test -p pr4xis-domains smart_element
```

Category laws, ontology validation, five domain axioms (single-point + proptest sweeps), engine tests (loop closure, exclusion-before-aggregation), and six functor-law suites.

## Concepts (14)

| Family | Concepts |
|---|---|
| Smart transducer (IEEE 1451.0-2007; Lee 2000) | `Transducer`, `Teds`, `Ncap` |
| Autonomic element (Kephart & Chess 2003 §3) | `ManagedElement`, `AutonomicManager`, `LocalOntology` |
| The synthesis | `SmartElement`, `SmartSensor`, `SmartDriver` |
| Self-* properties (Kephart & Chess 2003 §2, Table 1) | `SelfStarProperty`, `SelfConfiguration`, `SelfHealing`, `SelfOptimization`, `SelfProtection` |

Taxonomy: `SmartSensor` / `SmartDriver` is-a `SmartElement`; each self-* property is-a `SelfStarProperty`. Mereology: `SmartElement` has-a `AutonomicManager` and a `LocalOntology`; `SmartSensor` and `Ncap` each has-a `Transducer`.

Custom edge kinds: `Carries` (`SmartElement → LocalOntology`), `Exhibits` (`SmartElement → SelfStarProperty`), `Operates` (`SmartDriver → Transducer`), `DescribedBy` (`Transducer → Teds`), `Manages` (`AutonomicManager → ManagedElement`).

## Qualities

- `SelfStarKind` → `AutonomicProperty {Configuration, Healing, Optimization, Protection}` — defined on exactly the four self-* concepts (K&C Table 1).
- `MapeKPhaseFocus` → `MapeKConcept` (reused, not redefined) — which MAPE phase each self-* property most exercises: healing → `Analyze`, optimization → `Plan`, configuration → `Execute`, protection → `Monitor` (each grounded in K&C Table 1).
- `IsFusionPeer`, `HasClosedLoop`, `HasQueryableOntology` → `bool` — the three smartness predicates, each `Some(true)` for the Smart* concepts, `None` elsewhere.

## Domain axioms

| Axiom | Source | Discharged against |
|---|---|---|
| `SmartClosesMapeKLoop` | Kephart & Chess (2003) §3 | the `SmartElement → MapeK` image covers `{Monitor, Analyze, Plan, Execute, Knowledge}` and MAPE-K's own `LoopIsClosed` axiom verifies |
| `SmartIsFusionPeer` | Olfati-Saber et al. (2007); K&C (2003) | the three Smart* concepts map to `Peer` and `IsFusionPeer` is `Some(true)` for all three |
| `SmartCarriesQueryableOntology` | IEEE 1451.0-2007 §5; K&C (2003) | the `Carries` edge exists, `HasQueryableOntology` is `Some(true)`, and `Transducer` is `DescribedBy` `Teds` |
| `SelfStarComplete` | Kephart & Chess (2003) §2 | the Subsumption children of `SelfStarProperty` are exactly the four self-* properties |
| `SelfProtectionExcludesEquivocators` | Avizienis et al. (2004); LSP (1982) | `SelfProtection → FaultHandling`, and on the engine an observed equivocator is excluded before the next aggregation (changing the fused posterior) |

## Engine

[`engine.rs`](engine.rs) — the autonomic loop, pure `no_std`. The `SmartElementSituation` wraps the existing sensor-fusion `InformationEstimate` (every aggregation is its additive `fuse()`, never reimplemented); its actions are the four MAPE steps plus `GossipEstimate` and `ObserveEquivocation`. A full cycle visits Monitor → Analyze → Plan → Execute in order and wraps back to Monitor (the loop-closure witness). `ObserveEquivocation` distrusts a peer so the next `aggregate_trusted` excludes it — exclusion before aggregation, reusing the consensus engine's `PeerId` / `PeerTrust`. Every constant is a documented, cited fixture parameter.

## Cross-functors (6)

| Functor | Faithful anchors |
|---|---|
| [`mape_k_functor.rs`](mape_k_functor.rs) `→ MapeK` | `LocalOntology`/`Teds → Knowledge`; the four self-* → the four phases |
| [`sensor_functor.rs`](sensor_functor.rs) `→ Sensor` | forgetful: every concept → the `Sensor` umbrella |
| [`consensus_functor.rs`](consensus_functor.rs) `→ Consensus` | `Smart* → Peer`; `SelfProtection → DistrustedPeer` |
| [`driver_functor.rs`](driver_functor.rs) `→ Driver` | `SmartDriver → Driver`; `Teds → DeviceModel`; `SelfHealing → Recovery` |
| [`dependability_functor.rs`](dependability_functor.rs) `→ Dependability` | `SelfHealing → ErrorRecovery`; `SelfProtection → FaultHandling` |
| [`constitutive_functor.rs`](constitutive_functor.rs) `→ ConstitutiveProtocol` | `Smart* → Identity`; `SelfProtection → Slashing`; `LocalOntology`/`Teds → DeviceId` (an honest collapse — never `Constitution`/`ChannelManifest`) |

## Files

- `ontology.rs` — `SmartElementOntology`, three smartness predicates + two typed qualities, five domain axioms
- `engine.rs` — the MAPE loop, the estimate/neighbourhood situation, exclusion-before-aggregation
- `mape_k_functor.rs`, `sensor_functor.rs`, `consensus_functor.rs`, `driver_functor.rs`, `dependability_functor.rs`, `constitutive_functor.rs`
- `tests.rs` — proptest sweeps + engine loop/exclusion tests
- `mod.rs`, `README.md`, `citings.md`

An end-to-end demonstration — a ring of smart sensors converging by consensus-on-information, with one peer equivocating and being excluded by self-protection — lives in the examples crate at `crates/examples/src/swarm/smart_swarm.rs`.
