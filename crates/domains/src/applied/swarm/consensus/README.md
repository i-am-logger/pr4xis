# Consensus — distributed agreement over a peer graph

Average consensus and gossip averaging over a communication graph, the disagreement Lyapunov function and the Fiedler spectral gap that governs its decay, plus the trust wiring that detects equivocation and excludes the equivocator before aggregation. The mathematics is established literature (Olfati-Saber, Fax & Murray 2007; Xiao & Boyd 2004; Kempe, Dobra & Gehrke 2003; Fiedler 1973; Lamport, Shostak & Pease 1982; Li et al. 2004 SUNDR) — this ontology encodes it and claims no novelty for it.

## Verification

```
cargo test -p pr4xis-domains swarm::consensus
```

Category laws, ontology validation, five domain axioms (single-point + proptest sweeps), engine property tests, and three functor-law suites.

## Concepts (14)

| Family | Concepts |
|---|---|
| Networked multi-agent system (OSFM 2007) | `Peer`, `Neighborhood`, `Topology` |
| Protocols and steps (OSFM 2007; Kempe et al. 2003) | `GossipRound`, `ConsensusProtocol`, `AverageConsensus`, `GossipAveraging` |
| Convergence analysis (OSFM 2007; Fiedler 1973) | `Disagreement`, `Convergence`, `SpectralGap` |
| Trust wiring (Lamport 1979; LSP 1982; SUNDR) | `PeerIdentity`, `TrustedNeighbor`, `DistrustedPeer`, `Equivocation` |

Taxonomy: `AverageConsensus` / `GossipAveraging` is-a `ConsensusProtocol`; `TrustedNeighbor` / `DistrustedPeer` is-a `Peer`. Mereology: `Topology` has-a `Neighborhood`.

Custom edge kinds: `GossipsWith` (`Peer` → `Peer`), `MemberOf` (`Peer` → `Neighborhood`), `Reduces` (`GossipRound` → `Disagreement`), `Governs` (`SpectralGap` → `Convergence`), `IdentifiedBy` (`Peer` → `PeerIdentity`), `Triggers` (`Equivocation` → `DistrustedPeer`), `DistrustsAfterEquivocation` (`Peer` → `DistrustedPeer`).

## Qualities

- `RequiresConnectivity` → `bool` — both concrete protocols require a connected graph (Fiedler 1973; Xiao & Boyd 2004); `None` elsewhere.
- `TrustState` → `PeerTrust { Trusted, Distrusted }` — defined exactly on the two trust standings (LSP 1982).

## Domain axioms

| Axiom | Source | Discharged against |
|---|---|---|
| `ConsensusValueInConvexHull` | OSFM (2007) sec II | every `P3` iterate stays in `[min, max]` of the initial values; the limit is the initial average |
| `ConnectedTopologyConverges` | Fiedler (1973); Xiao & Boyd (2004) | disagreement < 1e-9 on `P3`, ≥ 1 on the `2+1` graph; cited `lambda_2` constants 1.0 and 0.0 |
| `GossipMassConservation` | Kempe, Dobra & Gehrke (2003) | `sum(s)/sum(w)` constant across every push-sum round |
| `DisagreementMonotoneNonIncreasing` | OSFM (2007) sec III | the disagreement function never increases along the stable-step run |
| `EquivocatorExcludedBeforeAggregation` | LSP (1982); Li et al. (2004) | the flagged peer is distrusted, leaves every neighbourhood, and influences no aggregation; the honest round does move values |

## Engine

[`engine.rs`](engine.rs) — every constant documented and cited: the `P3` path graph and disconnected `2+1` fixture with their known algebraic connectivities (`lambda_2(P3) = 1`, disconnected `= 0`, Fiedler 1973); the step size `eps = 1/(2 * Delta_max)`, the midpoint of OSFM (2007) sec II's stability interval `(0, 1/Delta_max)`; push-sum `(s, w)` pairs on a deterministic rotating schedule (mass conservation is schedule-independent); same-round inconsistent-report detection and exclusion-before-aggregation.

## Cross-functors

- [`mape_k_functor.rs`](mape_k_functor.rs) — `Consensus → MapeK`: sensing → `Monitor`, disagreement/spectral gap → `Analyze`, protocols → `Plan`, the round → `Execute`, identities/trust/convergence → `Knowledge`.
- [`dependability_functor.rs`](dependability_functor.rs) — `Consensus → Dependability`: `Equivocation → ByzantineFault` (LSP 1982 is Avizienis's own gloss), `DistrustedPeer → FaultHandling`, `TrustedNeighbor → CorrectService`.
- [`constitutive_functor.rs`](constitutive_functor.rs) — `Consensus → ConstitutiveProtocol`: `PeerIdentity → Identity`, `Equivocation → Equivocation`, `DistrustedPeer → Slashing`, `TrustedNeighbor → Membership`.

### Deferred: Consensus → PraxisKnowledgeGraph

A functor onto `formal/meta/praxis_knowledge_graph` was considered and deferred: that ontology's morphism vocabulary is exclusively structural (`Subsumption`/`Parthood` — it models the content-addressed persistence substrate, not agents or exchanges), so every consensus exchange relation (`GossipsWith`, `Reduces`, `Triggers`, …) would have to be tagged as taxonomy or mereology in the image. That is a false structural claim, not a documented collapse, so no lawful-and-honest total functor exists until the knowledge-graph ontology grows exchange/wire-event morphisms.

## Files

- `ontology.rs` — `ConsensusOntology`, two qualities, five domain axioms
- `engine.rs` — the graph fixtures, consensus/push-sum steps, equivocation handling
- `mape_k_functor.rs`, `dependability_functor.rs`, `constitutive_functor.rs`
- `tests.rs` — proptest sweeps + detection guard test
- `mod.rs`, `README.md`, `citings.md`
