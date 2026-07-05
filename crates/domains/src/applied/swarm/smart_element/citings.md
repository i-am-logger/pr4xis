# SmartElement ontology — bibliography

## Primary sources — the autonomic element

- **Kephart, J. O. & Chess, D. M. (2003).** *"The Vision of Autonomic Computing"*. IEEE Computer 36(1), 41–50. DOI: [10.1109/MC.2003.1160055](https://doi.org/10.1109/MC.2003.1160055). Grounds `ManagedElement`, `AutonomicManager`, `LocalOntology`, the four self-* concepts and `SelfStarProperty` (§2, Table 1), the `Exhibits` / `Manages` edges, the `MapeKPhaseFocus` quality assignments (§2 / Table 1 descriptions), and the `SmartClosesMapeKLoop`, `SmartIsFusionPeer`, `SmartCarriesQueryableOntology`, and `SelfStarComplete` axioms (§2–§3). The autonomic loop is established literature; this ontology encodes it.

## Primary sources — the smart transducer

- **IEEE Std 1451.0-2007.** *IEEE Standard for a Smart Transducer Interface for Sensors and Actuators — Common Functions, Communication Protocols, and Transducer Electronic Data Sheet (TEDS) Formats.* DOI: [10.1109/IEEESTD.2007.4338161](https://doi.org/10.1109/IEEESTD.2007.4338161). Grounds `Transducer`, `Teds` (§5), `Ncap`, the `Operates` / `DescribedBy` edges, and the `SmartCarriesQueryableOntology` axiom (the TEDS is the standardized self-description anchor).
- **Lee, K. (2000).** *"IEEE 1451: A Standard in Support of Smart Transducer Networking"*. Proc. 17th IEEE Instrumentation and Measurement Technology Conference (IMTC), 525–528. DOI: [10.1109/IMTC.2000.846923](https://doi.org/10.1109/IMTC.2000.846923). Grounds `Ncap` and `SmartSensor` (the "smart transducer" of the standard).

## Primary sources — the fusion peer

- **Olfati-Saber, R., Fax, J. A. & Murray, R. M. (2007).** *"Consensus and Cooperation in Networked Multi-Agent Systems"*. Proceedings of the IEEE 95(1), 215–233. DOI: [10.1109/JPROC.2006.887293](https://doi.org/10.1109/JPROC.2006.887293). Grounds `IsFusionPeer` and the `SmartIsFusionPeer` axiom (the element is a consensus `Peer`).
- **Maybeck, P. S. (1979).** *Stochastic Models, Estimation, and Control, Vol. 1.* Academic Press. Grounds the information form the engine reuses (`Y = P⁻¹`, additive fusion).
- **Mutambara, A. G. O. (1998).** *Decentralized Estimation and Control for Multisensor Systems.* CRC Press. Grounds the additive information-fusion step (`aggregate_trusted` folds `InformationEstimate::fuse`).
- **Bar-Shalom, Y., Li, X.-R. & Kirubarajan, T. (2001).** *Estimation with Applications to Tracking and Navigation.* Wiley, Ch. 1. Grounds the planar-state fixture dimension used by the engine.

## Trust and dependability sources

- **Lamport, L., Shostak, R. & Pease, M. (1982).** *"The Byzantine Generals Problem"*. ACM TOPLAS 4(3), 382–401. DOI: [10.1145/357172.357176](https://doi.org/10.1145/357172.357176). Grounds equivocation as inconsistent claims and the `SelfProtectionExcludesEquivocators` axiom's exclusion discipline.
- **Li, J., Krohn, M., Mazieres, D. & Shasha, D. (2004).** *"Secure Untrusted Data Repository (SUNDR)"*. Proc. 6th USENIX OSDI, 121–136. Grounds fork-consistency as the operational definition of the detected equivocation the engine excludes.
- **Lamport, L. (1979).** *"Constructing Digital Signatures from a One-Way Function"*. SRI International CSL-98. Grounds the constitutive functor's `Smart* → Identity` reading (to be able to sign is what an identity is).
- **Avizienis, A., Laprie, J.-C., Randell, B. & Landwehr, C. (2004).** *"Basic Concepts and Taxonomy of Dependable and Secure Computing"*. IEEE TDSC 1(1), 11–33. DOI: [10.1109/TDSC.2004.2](https://doi.org/10.1109/TDSC.2004.2). Grounds the dependability functor (`SelfHealing → ErrorRecovery`, `SelfProtection → FaultHandling`) and the structural half of `SelfProtectionExcludesEquivocators` (§5.2 fault handling: diagnosis, isolation, reconfiguration).

## Functor-target sources

- **Corbet, J., Rubini, A. & Kroah-Hartman, G. (2005).** *Linux Device Drivers*, 3rd ed., O'Reilly, Ch. 1. Grounds the driver functor's `SmartDriver → Driver` synthesis anchor.
- **Swift, M. M., Bershad, B. N. & Levy, H. M. (2003).** *"Improving the Reliability of Commodity Operating Systems"*. Proc. 19th SOSP. Grounds `SelfHealing → Recovery` and `SelfProtection → IsolationDomain` in the driver functor (shadow/restartable drivers; Nooks isolation).
- **Ryzhyk, L., Chubb, P., Kuz, I., Le Sueur, E. & Heiser, G. (2009).** *"Automatic Device Driver Synthesis with Termite"*. Proc. 22nd SOSP. Grounds `Teds → DeviceModel` (a driver is synthesized from a formal device model — a TEDS is exactly such a self-description).
- **Buterin, V. & Griffith, V. (2017).** *"Casper the Friendly Finality Gadget"*. arXiv:1710.09437. Grounds `SelfProtection → Slashing` in the constitutive functor.
- **Smith, B. et al. (2005).** *"Relations in biomedical ontologies"*. Genome Biology 6:R46. Grounds the canonical morphism-kind mapping (`Subsumption`/`Parthood`/`Causation`/`Opposition` → namesakes) used by every functor.

## Related workspace ontologies

- `formal::systems::mape_k` — the autonomic-loop functor target (`LoopIsClosed` invoked by `SmartClosesMapeKLoop`).
- `applied::sensor_fusion::sensor` — the smart-transducer functor target; `applied::sensor_fusion::state::information` — the reused `InformationEstimate`.
- `applied::swarm::consensus` — the fusion-peer functor target; reuses its `PeerId` / `PeerTrust` and exclusion discipline.
- `applied::swarm::fusion` — consensus-on-information, reused by the end-to-end swarm demo.
- `applied::operating_system::driver` — the synthesis-anchor functor target.
- `applied::dependability` — the fault-taxonomy functor target.
- `social::protocols::constitutive` — the signing-identity / slashing functor target.
