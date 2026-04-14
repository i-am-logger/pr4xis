# Rotation -- SO(3) with quaternion, DCM, Euler, and axis-angle representations

Models the rotation group SO(3) as a category whose objects are the four standard parameterizations of a 3D rotation (unit quaternion, direction-cosine matrix, Euler angles, axis-angle) and whose morphisms are the conversion functions between them. The group axioms (closure, associativity, identity, inverse) are verified on a fixed set of canonical quaternions, and the quaternion ↔ DCM roundtrip plus DCM properness (R^T R = I, det = +1) are checked numerically.

Key references:
- Hamilton 1844: quaternions
- Shoemake 1985: *Animating Rotation with Quaternion Curves* (SIGGRAPH)
- Diebel 2006: *Representing Attitude: Euler Angles, Unit Quaternions, and Rotation Vectors*
- Stuelpnagel 1964: *On the Parametrization of the Three-Dimensional Rotation Group*

## Entities (4)

| Category | Entities |
|---|---|
| Singularity-free (2) | Quaternion (4 params, 1 constraint), DCM (9 params, 6 constraints) |
| Singular (2) | Euler (3 params, gimbal lock), AxisAngle (4 params, undefined axis at zero angle) |

## Category

Discrete category over the four representation entities, joined by the conversion morphisms `ReprConversion`. Each conversion preserves the underlying SO(3) element; the set of conversions is closed under composition and the identity conversion exists for each representation.

## Qualities

| Quality | Type | Description |
|---|---|---|
| ParameterCount | usize | Quaternion=4, DCM=9, Euler=3, AxisAngle=4 |
| HasSingularity | bool | Quaternion=false, DCM=false, Euler=true (gimbal lock), AxisAngle=true (undefined axis at zero) |

## Axioms (6)

| Axiom | Description | Source |
|---|---|---|
| UnitNormClosure | Quaternion multiplication preserves unit norm (SO(3) closure) | SO(3) group |
| Associativity | (a*b)*c = a*(b*c) | SO(3) group |
| IdentityElement | q*I = I*q = q | SO(3) group |
| InverseExists | q * q^{-1} = identity | SO(3) group |
| DcmOrthogonality | Quaternion-to-DCM produces proper rotation: R^T R = I, det(R) = +1 | SO(3) definition |
| QuaternionDcmRoundtrip | Quaternion -> DCM -> quaternion roundtrip preserves rotation | Shoemake 1985 |

Plus the auto-generated structural axioms from `define_ontology!` (category laws on the representation category).

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one. The rigid-motion ontology (SE(3)) reuses `RotationCategory` directly as its category type rather than going through a functor; an explicit `RotationToRigidMotion` embedding would make that relationship categorical.

## Files

- `ontology.rs` -- Entity, representation category, ParameterCount and HasSingularity qualities, 6 SO(3) axioms, tests
- `quaternion.rs` -- `Quaternion` type: `from_axis_angle`, `multiply`, `inverse`, `identity`, `norm`
- `dcm.rs` -- `Dcm` (3x3 direction cosine matrix): `from_quaternion`, `to_quaternion`, `is_proper_rotation`
- `euler.rs` -- Euler angle representation with `EulerSequence` convention enum
- `axis_angle.rs` -- Axis-angle representation: `axis` (unit vector) + `angle` (radians)
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
