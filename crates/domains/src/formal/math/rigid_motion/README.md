# Rigid Motion -- SE(3) as rotation + translation

Models the special Euclidean group SE(3) of 3D rigid-body transformations as a `Pose` consisting of a unit quaternion (rotation) and a translation vector. The category is reused from the rotation ontology, since SE(3) extends SO(3) with translations rather than introducing new representations. The axioms verify the SE(3) group laws (associativity, identity, inverse) and the consistency between composing two poses and applying them sequentially to a point — both must produce the same result for any test point.

Key references:
- Murray, Li & Sastry 1994: *A Mathematical Introduction to Robotic Manipulation*
- Lynch & Park 2017: *Modern Robotics: Mechanics, Planning, and Control*

## Entities

The `RigidMotionOntology` reuses `RotationCategory` (and its `RotationRepr` entities and `ParameterCount` quality) as its underlying category. There is no separate SE(3) entity enum; the SE(3) extension lives in the `Pose` type and the four group axioms verified on canonical poses.

## Category

`RotationCategory` (see [../rotation/README.md](../rotation/README.md)). The rigid-motion ontology adds no new objects or morphisms; it adds axioms over the SE(3) group structure carried by `Pose` values.

## Qualities

Reuses `ParameterCount` from the rotation ontology.

## Axioms (4)

| Axiom | Description | Source |
|---|---|---|
| Associativity | (A*B)*C = A*(B*C) | SE(3) group |
| IdentityElement | Identity pose is the neutral element: T*I = I*T = T | SE(3) group |
| InverseExists | T * T^{-1} = identity | SE(3) group |
| CompositionConsistency | Composing poses then transforming equals sequential transforms | SE(3) action |

The structural axioms come from the rotation ontology's `define_ontology!` block; this ontology adds only the SE(3) domain axioms.

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one. The applied SLAM, odometry, frame-transform, and sensor-fusion domains all consume the `Pose` type directly; an explicit functor from rigid-motion to each of those would replace the current direct type usage with a categorical morphism. The category-level extension `RotationCategory → RigidMotion` (embedding SO(3) into SE(3) by setting translation = 0) is also a natural functor that does not yet exist as a file.

## Files

- `ontology.rs` -- `RigidMotionOntology` (reuses `RotationCategory`), 4 SE(3) axioms, tests, `canonical_poses` helper
- `pose.rs` -- `Pose { rotation: Quaternion, translation: [f64; 3] }`: `identity`, `from_translation`, `from_rotation`, `compose`, `inverse`, `transform_point`
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
