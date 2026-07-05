# Microkernel — the minimality principle of kernel construction

Liedtke (1995) SOSP "On µ-Kernel Construction": a kernel provides ONLY address spaces, threads, and IPC; drivers, file systems, pagers, and every policy run as unprivileged user-space servers (Liedtke 1996 CACM 39(9); Haertig et al. 1997 SOSP). The lineage runs from Dijkstra's (1968) layered privilege and Brinch Hansen's (1970) nucleus through Levin et al.'s (1975) policy/mechanism separation to Klein et al.'s (2009) seL4, where the minimised trusted computing base is formally verified.

## Verification

```
cargo test -p pr4xis-domains -- microkernel
```

Category laws, ontology validation, four domain axioms (single-point + proptest sweeps), engine property tests, and the `MicrokernelToSystem` / `MicrokernelToConcurrency` functor laws.

## Concepts (18)

| Family | Concepts |
|---|---|
| Kernels (Dijkstra 1968; Brinch Hansen 1970; Liedtke 1995, 1996) | `Kernel`, `Microkernel`, `MonolithicKernel`, `Nucleus` |
| Privileged abstractions (Liedtke 1995 §3) | `AddressSpace`, `Thread`, `Ipc` |
| Kernel objects (Klein et al. 2009; Brinch Hansen 1970) | `Capability`, `Message`, `Endpoint` |
| User space (Liedtke 1995 §4; Haertig et al. 1997) | `UserServer`, `Pager` |
| Mechanism vs. policy (Levin et al. 1975; Liedtke 1996) | `Scheduler`, `Mechanism`, `Policy` |
| Protection (Dijkstra 1968; Klein et al. 2009) | `PrivilegedMode`, `UserMode`, `TrustedComputingBase` |

Taxonomy: `Microkernel` / `MonolithicKernel` / `Nucleus` is-a `Kernel`; `Pager` is-a `UserServer`; `Ipc` / `Scheduler` is-a `Mechanism`. Oppositions: `MonolithicKernel` ↔ `Microkernel`, `PrivilegedMode` ↔ `UserMode`. Mechanism and policy are **separated**, not opposed — a custom `Separates` edge (Levin et al. 1975).

Custom edge kinds: `Privileges` (`Kernel` → the four privileged primitives), `Isolates` (`AddressSpace` → `Thread`), `Mediates` (`Kernel` → `Message`), `Grants` (`Capability` → `AddressSpace`/`Endpoint`), `RunsInUserSpace` (`UserServer`/`Pager` → `UserMode`), `Separates` (`Mechanism` → `Policy`).

## Qualities

- `KernelPrivilege` → `Privilege { Privileged, UserSpace }` — which side of the protection boundary a concept lives on; `None` for the five mode-neutral concepts (`Capability`, `Message`, `Endpoint`, `Mechanism`, `TrustedComputingBase`), each justified in the doc comment.
- `IsMechanism` → `bool` — the Levin et al. (1975) classification: `true` for `Mechanism`/`Ipc`/`Scheduler`, `false` for `Policy`, `None` elsewhere.

## Domain axioms

| Axiom | Source | Claim |
|---|---|---|
| `MinimalPrivilegedSet` | Liedtke (1995) §2 | the `Privileges`-edge targets of `Kernel` are exactly the set {`AddressSpace`, `Thread`, `Ipc`, `Scheduler`} — set equality, not a count |
| `MechanismPolicySeparation` | Levin et al. (1975) | the `Separates` edge exists and the `IsMechanism` classification is disjoint and non-vacuous |
| `ServersRunUnprivileged` | Liedtke (1995) §4; Haertig et al. (1997) | every `RunsInUserSpace` source has `KernelPrivilege = UserSpace`; `UserServer` and `Pager` both carry the edge |
| `MicrokernelMinimizesTcb` | Klein et al. (2009); Liedtke (1996) | `Microkernel` and `MonolithicKernel` are both Subsumption-children of `Kernel` and an `Opposition` edge connects them |

## Engine

[`engine.rs`](engine.rs) — a minimal kernel state, every constant documented and cited: `KernelSituation` (threads bound to address spaces, endpoint FIFO queues, a current thread) with `KernelAction::{Send, Receive, Switch}`. The transition function enforces address-space isolation (a `Send` naming a foreign buffer space is rejected — Liedtke 1995 §3.1) and kernel mediation (the only delivery path is `Send` → endpoint queue → `Receive`, with per-delivery endpoint provenance — Brinch Hansen 1970). The fixture is the canonical two-thread client–server configuration across two address spaces and one endpoint.

## Cross-functors

- **`Microkernel → System`** ([`system_functor.rs`](system_functor.rs)): threads and servers → `Component`; IPC, messages, endpoints → `Interaction`; kernels, scheduler, mechanism → `Controller`; capabilities and policy → `Constraint`; address spaces, CPU modes, TCB → `Boundary`. Kinds: `Privileges` → `Regulates`, `Isolates`/`Separates` → `Separates`, `Mediates`/`Grants` → `Governs`, `RunsInUserSpace` → `ComposesInto`.
- **`Microkernel → Concurrency`** ([`concurrency_functor.rs`](concurrency_functor.rs)): the faithful pairings are `Thread → Process` and `Ipc → Channel` (Hoare 1978: communicating sequential processes IS the thread+IPC model). `Message`/`Endpoint` collapse onto `Channel`, `AddressSpace` reads forgetfully as `MutualExclusion`, servers collapse onto `Process`, and the privilege/protection vocabulary collapses onto `Synchronization` — every collapse documented per arm.

## Files

- `ontology.rs` — `MicrokernelOntology`, two qualities, four domain axioms
- `engine.rs` — the kernel-state fixture and its guarded transition function
- `system_functor.rs` — `MicrokernelToSystem` + functor laws
- `concurrency_functor.rs` — `MicrokernelToConcurrency` + functor laws
- `tests.rs` — proptest sweeps + engine guard tests
- `mod.rs`, `README.md`, `citings.md`
