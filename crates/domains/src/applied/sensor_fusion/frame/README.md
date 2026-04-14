# Frame -- Reference frames and coordinate transforms

Models the multi-frame structure of sensor fusion as a category whose objects are reference frames (ECEF, ECI, NED, ENU, Body, IMU, Camera, LiDAR, Radar, GNSS) and whose morphisms are coordinate transforms between them. Every sensor measurement lives in some frame, and fusion requires transforming measurements into a common frame; the category laws make that composition explicit. A handedness quality captures the right-handed convention shared by all standard frames.

Key references:
- Groves 2013: *Principles of GNSS, Inertial, and Multisensor Integrated Navigation*, Chapter 2 — "Coordinate Frames"
- Sola, Deray, Atchuthan 2018: *A Micro Lie Theory for State Estimation in Robotics*
- IERS Conventions 2010, Chapter 4 (terrestrial reference frames)
- ISO 1151 / SAE J670 (body-frame convention)

## Entities (10 reference frames)

| Category | Entities |
|---|---|
| Inertial / Earth-fixed (2) | ECI, ECEF |
| Local-level (2) | NED, ENU |
| Platform (1) | Body |
| Sensor-fixed (5) | IMU, Camera, LiDAR, Radar, GNSS |

The objects are the `ReferenceFrame` variants; the morphisms are `FrameTransform { from, to }`, closed under composition whenever source and target match.

## Qualities

| Quality | Type | Description |
|---|---|---|
| FrameConvention | Handedness | Handedness of each frame — every standard frame in sensor fusion is right-handed (NED, ENU, ECEF, Body) |

## Axioms (4)

| Axiom | Description | Source |
|---|---|---|
| TransformsComposeAssociatively | Frame transforms compose associatively: (f∘g)∘h = f∘(g∘h) | category law |
| IdentityExists | Every frame has an identity transform that is neutral under composition | category law |
| TransformsInvertible | Every transform T(A,B) has an inverse T(B,A) — coordinate transforms form SE(3) | Sola et al. 2018 |
| AllFramesRightHanded | All reference frames use right-handed coordinate conventions | Groves 2013 Chapter 2; IERS 2010; ISO 1151 |

Category laws are additionally verified via `check_category_laws`.

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one. The frame category is the natural target of functors from every sensor-producing ontology (IMU, GNSS, celestial, AHRS) that needs to place its measurements in a common frame.

## Files

- `ontology.rs` -- `FrameCategory`, `FrameConvention` quality, `Handedness` enum, 4 axioms, tests
- `reference.rs` -- `ReferenceFrame` enum and its `Entity` implementation
- `transform.rs` -- `FrameTransform { from, to }` morphism type
- `lever_arm.rs` -- `LeverArm` (translation offset between frames on a rigid body)
- `boresight.rs` -- `Boresight` (rotational offset between frames on a rigid body)
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
