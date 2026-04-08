# praxis-physics

[![crates.io](https://img.shields.io/crates/v/praxis-physics.svg)](https://crates.io/crates/praxis-physics)
[![docs.rs](https://img.shields.io/docsrs/praxis-physics)](https://docs.rs/praxis-physics)

Physics laws as enforceable ontologies -- mechanics, electromagnetism, quantum mechanics, relativity, energy conservation, and Maxwell's equations.

Part of the [Praxis](https://github.com/i-am-logger/praxis) framework.

## Overview

Models fundamental physics as engine-driven state machines where the laws of physics are preconditions, not suggestions. Newton's F=ma enforces mass conservation and positive time. Ohm's law V=IR is maintained after every circuit change. The Heisenberg uncertainty principle blocks measurements that would violate it. Special relativity enforces v < c and derives time dilation and length contraction. Maxwell's equations enforce Gauss's laws (including no magnetic monopoles) and derive c = 1/sqrt(mu_0 * epsilon_0). The ontology layer classifies 14 physics laws across 5 branches as a category.

## Key Types

| Type | Description |
|---|---|
| `mechanics::Particle` | Mass, position, velocity with F=ma, momentum, and kinetic energy |
| `electromagnetism::Circuit` | V, I, R with Ohm's law enforced; Coulomb force |
| `quantum::QuantumParticle` | Position/momentum uncertainty with Heisenberg principle enforced |
| `relativity::Body` | Rest mass + velocity with v < c enforced; Lorentz factor, E=mc^2, time dilation, length contraction |
| `energy::System` | Mass, velocity, height with KE + PE conservation enforced |
| `maxwell::EMField` | E and B fields with Gauss's laws, Poynting vector, and energy density |
| `maxwell::Vec3` | 3D vector with dot product, cross product, and scaling |
| `ontology::PhysicsLaw` | 14 laws across 5 branches (Mechanics, Conservation, Electromagnetism, Relativity, Quantum) |
| `ontology::PhysicsCategory` | Laws as category objects with derivation morphisms |

## Example

```rust
use praxis_physics::relativity;
use praxis_physics::quantum;
use praxis_physics::maxwell;

// Speed of light enforced -- cannot reach c
let body = relativity::new_body(1.0).unwrap();
assert!(body.next(relativity::RelativityAction::SetVelocity {
    v: relativity::C
}).is_err());

// Heisenberg uncertainty principle enforced
let particle = quantum::new_minimum_uncertainty();
assert!(particle.situation().heisenberg_holds());

// Speed of light derived from Maxwell's equations
let c = maxwell::speed_of_light();
assert!((c - 2.998e8).abs() < 1e6);
```

## License

CC BY-NC-SA 4.0
