# Driver ontology — bibliography

## Primary sources

- **Corbet, J., Rubini, A. & Kroah-Hartman, G. (2005).** *Linux Device Drivers*, 3rd ed. O'Reilly Media. ISBN 978-0-596-00590-0. Freely available at [lwn.net/Kernel/LDD3](https://lwn.net/Kernel/LDD3/). Grounds `Driver`, `Device`, and the three device classes `CharacterDevice`/`BlockDevice`/`NetworkDevice` (Ch. 1), `HardwareRegister` (Ch. 9), `Interrupt` and `InterruptHandler` (Ch. 10), `Dma` (Ch. 15), `Hal`, the `Drives`/`Handles`/`Accesses`/`Abstracts` edges, and the `ThreeDeviceClasses` and `DriverBridgesSoftwareHardware` axioms (with the engine's probe/read/write/interrupt actions).
- **Swift, M. M., Bershad, B. N. & Levy, H. M. (2003).** *"Improving the Reliability of Commodity Operating Systems"* (Nooks). SOSP '03, 207–222. DOI: [10.1145/945445.945466](https://doi.org/10.1145/945445.945466). Grounds `IsolationDomain` (the lightweight protection domain, sec. 3), `DriverFault` (driver code as the dominant crash cause, sec. 1), `Recovery` (restart without kernel crash), the `Isolates` and `Recovers` edges, the `IsIsolated` quality, and the `IsolatedDriverContainsFault` axiom (the engine's fault-containment experiment reproduces sec. 3's design: identical faults injected with and without isolation).
- **Liedtke, J. (1995).** *"On µ-Kernel Construction"*. SOSP '95, 237–250. DOI: [10.1145/224056.224075](https://doi.org/10.1145/224056.224075). Grounds `DriverAsServer` (sec. 4: drivers as isolated user-space processes) and, with Swift et al. (2003), the isolated end of the `IsIsolated` axis.
- **Ganapathy, V., Renzelmann, M. J., Balakrishnan, A., Swift, M. M. & Jha, S. (2008).** *"The Design and Implementation of Microdrivers"*. ASPLOS XIII, 168–178. DOI: [10.1145/1346281.1346303](https://doi.org/10.1145/1346281.1346303). Grounds `Microdriver` (critical path in the kernel, bulk in user space) and its `Isolates` edge (the user-mode portion runs in its own protection domain).
- **Ryzhyk, L., Chubb, P., Kuz, I., Le Sueur, E. & Heiser, G. (2009).** *"Automatic Device Driver Synthesis with Termite"*. SOSP '09, 73–86. DOI: [10.1145/1629575.1629583](https://doi.org/10.1145/1629575.1629583). Grounds `DeviceModel`, the `SynthesizedFrom` edge, and the `SynthesizableFromModel` axiom.

## Functor sources

- **Avizienis, A., Laprie, J.-C., Randell, B. & Landwehr, C. (2004).** *"Basic Concepts and Taxonomy of Dependable and Secure Computing"*. IEEE TDSC 1(1), 11–33. DOI: [10.1109/TDSC.2004.2](https://doi.org/10.1109/TDSC.2004.2). Grounds the `DriverToDependability` object map: `Fault` (sec. 2.2), `FaultHandling` (sec. 5.2: diagnosis, isolation, reconfiguration), `ErrorRecovery` and `FaultTolerance` (sec. 5.2), `FaultPrevention` (sec. 5.1), `Service` (sec. 2.1).
- **Groves, P. D. (2013).** *Principles of GNSS, Inertial, and Multisensor Integrated Navigation Systems*, 2nd ed. Artech House. Grounds (with Corbet et al. 2005, Ch. 1) the `SensorToDriver` constant functor: every sensor of the Groves modality taxonomy is a device a driver drives.

## Honest-tier notes

- **`Interrupt` → `Activation`** in `DriverToDependability` is an *analogy*, not an equation: Avizienis' `Activation` is the transition of a dormant fault to an active one, while an interrupt is an asynchronous service request. The mapping records "the taxonomy's activation-related event concept" as the closest image, per the family plan; the mismatch is documented at the map site.
- **`DeviceModel` → `FaultPrevention`** is a reads-as: Ryzhyk et al. (2009) motivate synthesis as eliminating driver bugs by construction, which classifies under Avizienis sec. 5.1's development-methodology fault prevention — but Termite itself does not use the Avizienis vocabulary.
- **Kind collapses in `DriverToDependability`**: `Drives`/`Accesses`/`Abstracts`/`Parthood` map to the identity on the collapsed `Service` object (below the taxonomy's resolution); `Handles` and `Recovers` read as `Opposition` (response counters event); `Isolates` reads as `Causation` (containment brings about continued service); `SynthesizedFrom` reads as `Subsumption` (classification under the preventive means). The dependability category offers only Subsumption/Causation/Opposition, so these are documented reads-as choices, not literature equations.
- **`SensorToDriver`** is a total collapse (constant functor onto `Device`); the sensor/driver adjunction that would make the lost structure explicit is deferred follow-up.
- DOIs above are cited from the papers' ACM/IEEE records as known to the author of this module and were not re-fetched from the network in this environment (LLM-checked tier, not machine-verified).

## Related workspace ontologies

- `applied::dependability` — target of `DriverToDependability`.
- `applied::sensor_fusion::sensor` — source of `SensorToDriver` (the functor file lives with its source module).
- `formal::systems::concurrency` — the formal ground of the operating-system family (interrupt handling is a concurrency phenomenon).
