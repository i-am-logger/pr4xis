# DistributedFusion ontology — bibliography

## Primary sources

- **Olfati-Saber, R. (2005).** *"Distributed Kalman Filter with Embedded Consensus Filters"*. Proc. 44th IEEE Conference on Decision and Control (CDC), 8179–8184. DOI: [10.1109/CDC.2005.1583486](https://doi.org/10.1109/CDC.2005.1583486). Grounds `DistributedKalmanFilter` (cited in its gloss together with the 2007 paper — note the two distinct CDC titles and years).
- **Olfati-Saber, R. (2007).** *"Distributed Kalman Filtering for Sensor Networks"*. Proc. 46th IEEE Conference on Decision and Control (CDC), 5492–5498. DOI: [10.1109/CDC.2007.4434303](https://doi.org/10.1109/CDC.2007.4434303). Grounds `DistributedKalmanFilter`, `ConsensusEstimate`, `InnovationExchange`, the `Produces` edge, and the `DistributedFilterAgreesWithCentralInTheLimit` axiom (consensus on information contributions recovers the centralized information-filter fuse).
- **Julier, S. J. & Uhlmann, J. K. (1997).** *"A Non-divergent Estimation Algorithm in the Presence of Unknown Correlations"*. Proc. American Control Conference, 2369–2373. DOI: [10.1109/ACC.1997.609105](https://doi.org/10.1109/ACC.1997.609105). Grounds `CiOverNetwork`, the `Prevents` edge, the omega-weighted form `P_CI^{-1} = omega*P1^{-1} + (1-omega)*P2^{-1}` and its `OMEGA_GRID`/`CORRELATION_GRID` fixture constants, the `ConsistentUnderInterPeerCorrelation` quality, and the `CiFusionConsistentAcrossPeers` axiom.
- **Bar-Shalom, Y. (1981).** *"On the Track-to-Track Correlation Problem"*. IEEE Transactions on Automatic Control 26(2), 571–572. DOI: [10.1109/TAC.1981.1102635](https://doi.org/10.1109/TAC.1981.1102635). Grounds `DataIncest`, the `Corrupts` edge, the 3-peer ring fixture (double counting requires a cycle), and the `NaiveInformationFusionOverconfidentUnderCycles` axiom.
- **Mutambara, A. G. O. (1998).** *Decentralized Estimation and Control for Multisensor Systems*. CRC Press. ISBN 978-0849318658. Grounds `NetworkFusionArchitecture` (Ch. 1), the additive information form the engine reuses (Ch. 3), the false-value side of the consistency quality, and (with Bar-Shalom) the honest-negative axiom.

## Secondary sources

- **Bar-Shalom, Y., Li, X. R. & Kirubarajan, T. (2001).** *Estimation with Applications to Tracking and Navigation*. Wiley. DOI: [10.1002/0471221279](https://doi.org/10.1002/0471221279). Grounds the `FusedCovarianceRemainsPsd` axiom and the planar-fixture dimensioning.
- **Olfati-Saber, R., Fax, J. A. & Murray, R. M. (2007).** *"Consensus and Cooperation in Networked Multi-Agent Systems"*. Proceedings of the IEEE 95(1), 215–233. DOI: [10.1109/JPROC.2006.887293](https://doi.org/10.1109/JPROC.2006.887293). Grounds the `DistributedFusionToConsensus` functor's kind readings (with Olfati-Saber 2007 CDC for the object map).
- **Maybeck, P. S. (1979).** *Stochastic Models, Estimation, and Control*, Vol. 1. Academic Press. Grounds (with Mutambara 1998) the reused `InformationEstimate` type — see `applied/sensor_fusion/state/information.rs`.

## Functor-target sources (reused from the targets' own headers)

- **Liggins, M. E., Hall, D. L. & Llinas, J. (eds.) (2008).** *Handbook of Multisensor Data Fusion: Theory and Practice*, 2nd ed., Ch. 2. CRC Press. Grounds the `DistributedFusionToFusionArchitecture` object map (the architecture enum's own source).
- **Castanedo, F. (2013).** *"A Review of Data Fusion Techniques"*. The Scientific World Journal 2013, 704504. DOI: [10.1155/2013/704504](https://doi.org/10.1155/2013/704504). Ditto.
- **Carlson, N. A. (1990).** *"Federated square root filter for decentralized parallel processes"*. IEEE Transactions on Aerospace and Electronic Systems 26(3), 517–525. DOI: [10.1109/7.106130](https://doi.org/10.1109/7.106130). Grounds the `CompositionStrategy` target enum (its own source) for the `DistributedFusionToCompositionStrategy` functor.
- **Mac Lane, S. (1971).** *Categories for the Working Mathematician*. Springer GTM 5. Grounds the discrete/indiscrete wrapper-category constructions the two enum functors ride on.

## Related workspace ontologies

- `applied::sensor_fusion::state` — the reused `InformationEstimate` / covariance validity substrate.
- `applied::sensor_fusion::fusion` — the two bare-enum functor targets (`architecture.rs`, `composition.rs`).
- `applied::swarm::consensus` — the sibling ontology whose engine drives the consensus-on-information run and whose category is the third functor target.
