# Physics -- Laws of physics as a discrete category

Models the fundamental laws of physics — Newton's three laws, the three conservation laws, the four Maxwell equations, the two special-relativity postulates, and the two core quantum relations — as entities in a discrete category labelled by branch. Domain axioms verify that Maxwell's equations together derive the speed of light and that every branch is represented.

Key references:
- Newton 1687: *Philosophiæ Naturalis Principia Mathematica* (mechanics)
- Maxwell 1865: *A Dynamical Theory of the Electromagnetic Field* (electromagnetism)
- Einstein 1905: *Zur Elektrodynamik bewegter Körper* (special relativity)
- Heisenberg 1927 / Planck 1900 (quantum mechanics)

## Entities (14)

| Category | Entities |
|---|---|
| Mechanics (3) | NewtonFirst, NewtonSecond, NewtonThird |
| Conservation (3) | EnergyConservation, MomentumConservation, ChargeConservation |
| Electromagnetism (4) | GaussElectric, GaussMagnetic, FaradayLaw, AmpereMaxwell |
| Relativity (2) | SpeedOfLight, MassEnergy |
| Quantum (2) | Heisenberg, Planck |

## Category

Discrete category over the 14 `PhysicsConcept` variants. The branch classification (`LawBranch` quality) partitions the laws into the five branches above.

## Qualities

| Quality | Type | Description |
|---|---|---|
| LawBranch | Branch | Which branch of physics a law belongs to (Mechanics, Conservation, Electromagnetism, Relativity, Quantum) |

## Axioms (2)

| Axiom | Description | Source |
|---|---|---|
| MaxwellDerivesC | The four Maxwell equations together yield c = 1/√(μ₀ε₀) | Maxwell 1865 |
| AllBranchesRepresented | Every branch of physics has at least one law (no empty branch) | structural |

Plus the auto-generated structural axioms from `pr4xis::ontology!` (category laws on the discrete category).

## Engine-level sub-ontologies

Each sibling file wires a Situation/Action/Precondition engine for one branch, enforcing the branch's governing equations as preconditions on every state transition:

- `mechanics.rs` enforces F = ma and mass conservation on a particle with mass, position, velocity.
- `energy.rs` enforces conservation of total mechanical energy (KE + PE).
- `electromagnetism.rs` enforces Ohm's law V = IR on a circuit.
- `maxwell.rs` enforces all four Maxwell equations and exposes `speed_of_light()`.
- `relativity.rs` enforces v < c, rest mass invariance, and derives the Lorentz factor, time dilation, length contraction, and E = mc².
- `quantum.rs` enforces Heisenberg's uncertainty Δx·Δp ≥ ℏ/2.

The `kinematics/` subdirectory is a full ontology of its own (see [`kinematics/README.md`](kinematics/README.md)).

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../docs/use/compose-via-functor.md) to add one.

## Files

- `ontology.rs` -- `PhysicsConcept` entity, discrete category, `LawBranch` quality, 2 domain axioms, tests
- `mechanics.rs` -- Newtonian mechanics engine (F = ma, mass conservation)
- `energy.rs` -- Energy conservation engine (KE + PE)
- `electromagnetism.rs` -- Circuit engine (Ohm's law)
- `maxwell.rs` -- Maxwell equations engine and derivation of c
- `relativity.rs` -- Special relativity engine (Lorentz factor, dilation, contraction, E = mc²)
- `quantum.rs` -- Quantum engine (Heisenberg uncertainty)
- `kinematics/` -- Kinematics sub-ontology (position, velocity, acceleration, jerk, motion models)
- `mod.rs` -- module declarations
