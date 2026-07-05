# Driver — device drivers and the HAL

The operating-system half of the driver/device seam: the driver as software translating OS requests into device operations, the three Linux device classes, registers/interrupts/DMA (Corbet, Rubini & Kroah-Hartman 2005), the isolation design space from in-kernel driver to user-space server (Liedtke 1995) with Nooks fault containment and recovery (Swift, Bershad & Levy 2003), the microdriver split (Ganapathy et al. 2008), and driver synthesis from a formal device model (Ryzhyk et al. 2009).

## Verification

```
cargo test -p pr4xis-domains -- operating_system::driver
cargo test -p pr4xis-domains -- sensor::driver_functor
```

Category laws, ontology validation, four domain axioms (single-point + proptest sweeps), engine property tests, and the two cross-functor law suites.

## Concepts (16)

| Family | Concepts |
|---|---|
| Software/hardware seam (Corbet et al. 2005, Ch. 1) | `Driver`, `Device`, `CharacterDevice`, `BlockDevice`, `NetworkDevice` |
| Hardware side (Corbet et al. 2005, Ch. 9/10/15) | `HardwareRegister`, `Interrupt`, `InterruptHandler`, `Dma`, `Hal` |
| Isolation architectures (Liedtke 1995; Swift et al. 2003; Ganapathy et al. 2008) | `DriverAsServer`, `IsolationDomain`, `Microdriver` |
| Synthesis (Ryzhyk et al. 2009) | `DeviceModel` |
| Faults and recovery (Swift et al. 2003) | `DriverFault`, `Recovery` |

Taxonomy: the three device classes is-a `Device`; `DriverAsServer` / `Microdriver` is-a `Driver`. Parthood: `InterruptHandler` part-of `Driver`.

Custom edge kinds: `Drives` (`Driver` → `Device`), `Handles` (`InterruptHandler` → `Interrupt`), `Accesses` (`Driver` → `HardwareRegister`/`Dma`), `Isolates` (`IsolationDomain` → `Driver`/`DriverAsServer`/`Microdriver`), `SynthesizedFrom` (`Driver` → `DeviceModel`), `Recovers` (`Recovery` → `DriverFault`), `Abstracts` (`Hal` → `Device`).

## Qualities

- `DeviceClass` → `DevClass { Character, Block, Network }` — defined exactly on the three device-class concepts (Corbet et al. 2005, Ch. 1); `None` on the abstract `Device` parent.
- `IsIsolated` → `bool` — `Driver` false (the in-kernel commodity default), `DriverAsServer`/`Microdriver` true (Swift et al. 2003; Liedtke 1995); `None` elsewhere.

## Domain axioms

| Axiom | Source | Discharged against |
|---|---|---|
| `ThreeDeviceClasses` | Corbet et al. (2005) Ch. 1 | Subsumption children of `Device` equal the three classes, in bijection with `DevClass` |
| `IsolatedDriverContainsFault` | Swift et al. (2003) sec. 3 | every isolated architecture has an incoming `Isolates` edge; on the engine the same injected fault leaves kernel state intact (and the driver recoverable) inside an isolation domain and corrupts it (defeating recovery) in the kernel address space |
| `DriverBridgesSoftwareHardware` | Corbet et al. (2005) Ch. 1 | `Drives` + `Accesses` edges; on the engine a bound driver services read/write and an unbound one cannot |
| `SynthesizableFromModel` | Ryzhyk et al. (2009) | the `SynthesizedFrom` edge `Driver` → `DeviceModel` |

## Engine

[`engine.rs`](engine.rs) — a typed driver/device state machine: `DriverSituation` (device state × driver state × kernel integrity × isolation-domain membership) with `Probe` / `Read` / `Write` / `RaiseInterrupt` / `HandleInterrupt` / `InjectFault` / `Recover` actions. The `FAULT_CONTAINMENT_SCRIPT` fixture reproduces the Nooks experiment: the identical fault is injected into an isolated and a non-isolated driver and the kernel's fate compared (Swift et al. 2003 sec. 3). Every state component is a closed typed enum; no magic numbers.

## Cross-functors

- **`Driver → Dependability`** ([`dependability_functor.rs`](dependability_functor.rs)): `DriverFault` → `Fault`, `IsolationDomain` → `FaultHandling` (Avizienis sec. 5.2), `Recovery` → `ErrorRecovery`, `DriverAsServer`/`Microdriver` → `FaultTolerance`, `Interrupt` → `Activation`, `DeviceModel` → `FaultPrevention`; all nine remaining concepts collapse to `Service` (each collapse documented in the file). Total map.
- **`Sensor → Driver`** (at [`../../sensor_fusion/sensor/driver_functor.rs`](../../sensor_fusion/sensor/driver_functor.rs), with its source ontology): the constant functor — every sensor concept, composites included, is a `Device` a driver drives. The sensor/driver **adjunction** that would expose this collapse gap is deferred follow-up.

## Files

- `ontology.rs` — `DriverOntology`, two qualities, four domain axioms
- `engine.rs` — the driver/device state machine and containment fixture
- `dependability_functor.rs` — `DriverToDependability` + functor laws
- `tests.rs` — proptest sweeps + engine guard tests
- `mod.rs`, `README.md`, `citings.md`
