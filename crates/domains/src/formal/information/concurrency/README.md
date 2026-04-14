# Concurrency -- Agents, shared resources, synchronization, hazards

Models the universal structure of simultaneous activity: multiple agents acting on shared resources under synchronization and protocol, with the two classic hazards (deadlock and race condition) as first-class concepts. The category is dense enough that chess, traffic, and conversation can all be expressed as functors into it.

Key references:
- Hoare 1978: *Communicating Sequential Processes* (CSP)
- Milner 1980: *A Calculus of Communicating Systems* (CCS)
- Hewitt 1973: Actor Model
- Lamport 1978: *Time, Clocks and the Ordering of Events in a Distributed System*

## Entities (10)

| Category | Entities |
|---|---|
| Agents and resources (2) | Agent, SharedResource |
| Control flow (4) | Action, Synchronization, State, Protocol |
| Messaging (2) | Message, Future |
| Hazards (2) | Deadlock, RaceCondition |

## Category

Dense kinded relation graph over the ten concurrency concepts. Key edges: `Agent ActsOn SharedResource`, `Synchronization Controls Agent`, `Protocol Governs Action`, `Action Changes State`, plus `Deadlock ArisesFrom Synchronization` and `RaceCondition UnsynchronizedAccess SharedResource`. Composition closes enough paths (Synchronization → SharedResource, Agent → RaceCondition, Action → Future) that functors from chess and system theory land inside it.

## Qualities

| Quality | Type | Description |
|---|---|---|
| IsHazard | bool | Deadlock = true, RaceCondition = true, all other concepts = false |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| (structural) | Identity and composition laws over the concurrency kinded relation graph | auto-generated |

## Functors

**Outgoing (2):**

| Functor | Target | File |
|---|---|---|
| ConcurrencyToSystems | systems | `systems_functor.rs` |
| ConcurrencyToChess | chess | `chess_functor.rs` |

**Incoming (1):**

| Functor | Source | File |
|---|---|---|
| EventsToConcurrency | events | `../events/concurrent_functor.rs` |

## Files

- `ontology.rs` -- `ConcurrencyConcept`, dense kinded category, IsHazard quality, tests
- `systems_functor.rs` -- Concurrency → systems-thinking functor
- `chess_functor.rs` -- Concurrency → chess functor (two players, shared board, turn-taking)
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
