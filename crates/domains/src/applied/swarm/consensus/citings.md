# Consensus ontology — bibliography

## Primary sources

- **Olfati-Saber, R., Fax, J. A. & Murray, R. M. (2007).** *"Consensus and Cooperation in Networked Multi-Agent Systems"*. Proceedings of the IEEE 95(1), 215–233. DOI: [10.1109/JPROC.2006.887293](https://doi.org/10.1109/JPROC.2006.887293). Grounds `Peer`, `Neighborhood`, `Topology`, `ConsensusProtocol`, `AverageConsensus`, `Disagreement`, `Convergence`, the `GossipsWith`/`MemberOf`/`Reduces` edges, the step-size stability bound `eps < 1/Delta_max` (sec II), and the `ConsensusValueInConvexHull` and `DisagreementMonotoneNonIncreasing` axioms (sec II–III).
- **Olfati-Saber, R. & Murray, R. M. (2004).** *"Consensus Problems in Networks of Agents With Switching Topology and Time-Delays"*. IEEE Transactions on Automatic Control 49(9), 1520–1533. DOI: [10.1109/TAC.2004.834113](https://doi.org/10.1109/TAC.2004.834113). Grounds the `AverageConsensus` update rule cited in its gloss.
- **Xiao, L. & Boyd, S. (2004).** *"Fast linear iterations for distributed averaging"*. Systems & Control Letters 53(1), 65–78. DOI: [10.1016/j.sysconle.2004.02.022](https://doi.org/10.1016/j.sysconle.2004.02.022). Grounds `GossipAveraging`, the spectral view behind `SpectralGap`, the `RequiresConnectivity` quality, and (with Fiedler) the `ConnectedTopologyConverges` axiom.
- **Kempe, D., Dobra, A. & Gehrke, J. (2003).** *"Gossip-Based Computation of Aggregate Information"*. Proc. 44th IEEE FOCS, 482–491. DOI: [10.1109/SFCS.2003.1238221](https://doi.org/10.1109/SFCS.2003.1238221). Grounds `GossipRound`, `GossipAveraging` (push-sum), the engine's `(s, w)` pairs and `PUSH_SUM_SHARE`, and the `GossipMassConservation` axiom.
- **Fiedler, M. (1973).** *"Algebraic connectivity of graphs"*. Czechoslovak Mathematical Journal 23(2), 298–305. DOI: [10.21136/CMJ.1973.101168](https://doi.org/10.21136/CMJ.1973.101168). Grounds `SpectralGap`, the `Governs` edge, the fixtures' cited constants `lambda_2(P3) = 1.0` / disconnected `= 0.0`, and (with Xiao & Boyd) the `ConnectedTopologyConverges` axiom.

## Trust wiring sources

- **Lamport, L., Shostak, R. & Pease, M. (1982).** *"The Byzantine Generals Problem"*. ACM TOPLAS 4(3), 382–401. DOI: [10.1145/357172.357176](https://doi.org/10.1145/357172.357176). Grounds `Equivocation`, `TrustedNeighbor`/`DistrustedPeer`, the `Triggers` and `DistrustsAfterEquivocation` edges, the `TrustState` quality, and the `EquivocatorExcludedBeforeAggregation` axiom.
- **Li, J., Krohn, M., Mazieres, D. & Shasha, D. (2004).** *"Secure Untrusted Data Repository (SUNDR)"*. Proc. 6th USENIX OSDI, 121–136. Grounds fork-consistency as the operational definition of equivocation (the pair of conflicting same-round reports IS the proof) and the exclusion discipline of the `EquivocatorExcludedBeforeAggregation` axiom.
- **Lamport, L. (1979).** *"Constructing Digital Signatures from a One-Way Function"*. SRI International CSL-98. Grounds `PeerIdentity` and the `IdentifiedBy` edge (to be able to sign is what an identity is) — the bridge concept the constitutive functor maps to `Identity`.

## Functor-target sources

- **Kephart, J. O. & Chess, D. M. (2003).** *"The Vision of Autonomic Computing"*. IEEE Computer 36(1), 41–50. DOI: [10.1109/MC.2003.1160055](https://doi.org/10.1109/MC.2003.1160055). Grounds the `ConsensusToMapeK` phase assignment.
- **Avizienis, A., Laprie, J.-C., Randell, B. & Landwehr, C. (2004).** *"Basic Concepts and Taxonomy of Dependable and Secure Computing"*. IEEE TDSC 1(1), 11–33. DOI: [10.1109/TDSC.2004.2](https://doi.org/10.1109/TDSC.2004.2). Grounds the `ConsensusToDependability` object map (service / threat / means facets).
- **Buterin, V. & Griffith, V. (2017).** *"Casper the Friendly Finality Gadget"*. arXiv:1710.09437. Grounds the `DistrustedPeer → Slashing` reading in `ConsensusToConstitutiveProtocol`.

## Related workspace ontologies

- `social::protocols::constitutive` — the trust-bridge functor target.
- `applied::dependability` — the fault-taxonomy functor target.
- `formal::systems::mape_k` — the autonomic-loop functor target.
- `applied::swarm::fusion` — consumes this engine's consensus step for consensus-on-information.
