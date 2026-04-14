# Orbit -- Keplerian Orbital Elements Ontology

Models the six classical (Keplerian) orbital elements that fully specify a bound two-body orbit, together with their units and physical bounds. The category is fully connected because all six elements are interdependent: fixing any subset still leaves the others as coupled parameters of a single orbit.

Key references:
- Vallado 2013: *Fundamentals of Astrodynamics and Applications*, 4th ed.
- Curtis 2014: *Orbital Mechanics for Engineering Students*, 3rd ed.
- Battin 1999: *An Introduction to the Mathematics and Methods of Astrodynamics*

## Entities (6)

| Category | Entities |
|---|---|
| Orbital elements (6) | SemiMajorAxis, Eccentricity, Inclination, RAAN, ArgPeriapsis, TrueAnomaly |

## Category

`OrbitOntology for OrbitCategory` via `define_ontology!`, relation `ElementDependency`, fully connected.

## Qualities

| Quality | Type | Description |
|---|---|---|
| ElementUnit | &'static str | SI unit: SemiMajorAxis=km, Eccentricity=dimensionless, Inclination/RAAN/ArgPeriapsis/TrueAnomaly=rad |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| EccentricityBounded | Eccentricity lies in [0, 1) for bound (elliptical) orbits | Vallado 2013 |
| SemiMajorAxisPositive | Semi-major axis is positive for bound orbits | Curtis 2014 |
| (structural) | Identity and composition laws over the OrbitCategory | auto-generated |

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one.

## Files

- `ontology.rs` -- `OrbitalElement`, `OrbitCategory`, `ElementUnit` quality, orbital-mechanics axioms
- `propagator.rs` -- Keplerian propagation (element → state vector → element over time)
- `engine.rs` -- orbit-propagation engine used by tests
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
