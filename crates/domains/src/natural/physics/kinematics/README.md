# Kinematics -- Position, velocity, acceleration, jerk, motion models

Models the successive time-derivatives of position — position (0th), velocity (1st), acceleration (2nd), jerk (3rd) — as a category over classical mechanics. Axioms verify that velocity and acceleration behave as derivatives, that constant-velocity and constant-acceleration propagation match the closed-form integrals of Newton's laws, and that Galilean velocity addition is commutative.

Key references:
- Newton 1687: *Principia* (laws of motion)
- Goldstein 2002: *Classical Mechanics* (3rd ed.), Chapter 1
- Bar-Shalom, Li & Kirubarajan 2001: *Estimation with Applications to Tracking and Navigation* (motion models)

## Entities (4)

| Category | Entities |
|---|---|
| Kinematic quantities (4) | Position, Velocity, Acceleration, Jerk |

## Category

Objects are `KinematicQuantity` variants; the relation kind is `DerivativeRelation`, encoding the "derivative of" link from one order to the next.

## Qualities

| Quality | Type | Description |
|---|---|---|
| DerivativeOrder | usize | 0 for Position, 1 for Velocity, 2 for Acceleration, 3 for Jerk |
| SiUnit | &'static str | SI unit: m, m/s, m/s², m/s³ |

## Axioms (8)

| Axiom | Description | Source |
|---|---|---|
| VelocityIsDerivativeOfPosition | v = dx/dt: constant velocity gives displacement = v·dt exactly | Newton 1687 |
| AccelerationIsDerivativeOfVelocity | a = dv/dt: velocity change / time yields acceleration | Newton 1687 |
| ConstantVelocityPropagation | x(t+dt) = x(t) + v·dt | Goldstein 2002 |
| ConstantAccelerationPropagation | x(t+dt) = x(t) + v·dt + ½·a·dt² | Goldstein 2002 |
| VelocityUpdateUnderAcceleration | v(t+dt) = v(t) + a·dt under constant acceleration | Goldstein 2002 |
| StaticModelInvariance | Static model leaves position unchanged regardless of velocity | Bar-Shalom et al. 2001 |
| SpeedNonNegative | \|v\| ≥ 0 for every velocity | metric space |
| VelocityAdditionCommutative | Galilean v₁ + v₂ = v₂ + v₁ | Galilean relativity |

Plus the auto-generated structural axioms from `define_ontology!` (category laws on the kinded relation graph).

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one. Kinematics composes against `formal/math/geometry` (for `Point3`) and `formal/math/temporal` (for `Instant`); when those links are promoted to explicit functors, they will land here.

## Files

- `ontology.rs` -- `KinematicQuantity` entity, category, `DerivativeOrder`/`SiUnit` qualities, 8 axioms, tests
- `position.rs` -- `TimedPosition` (geometry + time)
- `velocity.rs` -- `Velocity` vector, speed, addition, `acceleration_to`
- `acceleration.rs` -- `Acceleration` vector, gravity constant
- `trajectory.rs` -- `KinematicState` (position + velocity + acceleration) and propagation
- `motion_model.rs` -- `MotionModelType` (Static, ConstantVelocity, ConstantAcceleration) and `propagate`
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
