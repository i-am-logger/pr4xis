# Microkernel ontology — bibliography

## Primary sources

- **Liedtke, J. (1995).** *"On µ-Kernel Construction"*. SOSP '95, 237–250. DOI: [10.1145/224056.224075](https://doi.org/10.1145/224056.224075). Grounds `Microkernel` (§2 minimality principle), `AddressSpace` (§2.1), `Thread` and `Ipc` (§2.2, Threads and IPC), `UserServer` and `Pager` (§3, Flexibility), the four `Privileges` edges, the `Isolates` edge, the `RunsInUserSpace` edges, the `MinimalPrivilegedSet` and (with Haertig et al. 1997) `ServersRunUnprivileged` axioms, and the engine's isolation guard.
- **Liedtke, J. (1996).** *"Toward Real Microkernels"*. CACM 39(9), 70–77. DOI: [10.1145/234215.234473](https://doi.org/10.1145/234215.234473). Grounds `MonolithicKernel` (the contrast concept), `Scheduler`, the `MonolithicKernel` ↔ `Microkernel` opposition, and (with Klein et al. 2009) the `MicrokernelMinimizesTcb` axiom.
- **Klein, G., Elphinstone, K., Heiser, G., Andronick, J., Cock, D., Derrin, P., Elkaduwe, D., Engelhardt, K., Kolanski, R., Norrish, M., Sewell, T., Tuch, H. & Winwood, S. (2009).** *"seL4: Formal Verification of an OS Kernel"*. SOSP '09, 207–220. DOI: [10.1145/1629575.1629596](https://doi.org/10.1145/1629575.1629596). Grounds `Capability`, `Endpoint`, `TrustedComputingBase`, the two `Grants` edges, and (with Liedtke 1996) the `MicrokernelMinimizesTcb` axiom.
- **Brinch Hansen, P. (1970).** *"The Nucleus of a Multiprogramming System"*. CACM 13(4), 238–241. DOI: [10.1145/362258.362278](https://doi.org/10.1145/362258.362278). Grounds `Nucleus`, `Message`, the `Mediates` edge, and the engine's endpoint-mediated delivery (message buffering, FIFO order, blocking receive).
- **Dijkstra, E. W. (1968).** *"The Structure of the 'THE'-Multiprogramming System"*. CACM 11(5), 341–346. DOI: [10.1145/363095.363143](https://doi.org/10.1145/363095.363143). Grounds `Kernel` (with Brinch Hansen 1970), `PrivilegedMode`, `UserMode`, and their opposition — the layered privilege structure. Note: this is a different Dijkstra (1968) from the concurrency ontology's EWD-123 ("Cooperating Sequential Processes").
- **Levin, R., Cohen, E., Corwin, W., Pollack, F. & Wulf, W. (1975).** *"Policy/Mechanism Separation in HYDRA"*. SOSP '75, 132–140. DOI: [10.1145/800213.806531](https://doi.org/10.1145/800213.806531). Grounds `Mechanism`, `Policy`, the `Separates` edge, the `IsMechanism` quality, the `MechanismPolicySeparation` axiom, and (with Klein et al. 2009) `Capability`.
- **Haertig, H., Hohmuth, M., Liedtke, J., Schoenberg, S. & Wolter, J. (1997).** *"The Performance of µ-Kernel-Based Systems"*. SOSP '97, 66–77. DOI: [10.1145/268998.266660](https://doi.org/10.1145/268998.266660). Grounds `UserServer` (with Liedtke 1995 §3) and the `ServersRunUnprivileged` axiom's user-level-service claim; the engine's client–server fixture is the configuration this paper measures.

## Functor sources

- **von Bertalanffy, L. (1968).** *General System Theory: Foundations, Development, Applications*. George Braziller. Grounds the `MicrokernelToSystem` functor's object map (with Liedtke 1995 for the architectural roles).
- **Hoare, C. A. R. (1978).** *"Communicating Sequential Processes"*. CACM 21(8), 666–677. DOI: [10.1145/359576.359585](https://doi.org/10.1145/359576.359585). Grounds the `MicrokernelToConcurrency` functor's faithful pairings: `Thread → Process`, `Ipc → Channel`.

## Related workspace ontologies

- `formal::systems` — the `MicrokernelToSystem` target (`SystemCategory`).
- `formal::systems::concurrency` — the `MicrokernelToConcurrency` target (`ConcurrencyCategory`); the formal process-composition theory beneath the kernel.
