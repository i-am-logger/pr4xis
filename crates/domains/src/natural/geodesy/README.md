# Geodesy -- Ellipsoid, coordinate frames, and frame conversions

Models the coordinate systems used to locate things on and near the Earth — geodetic (lat/lon/alt), ECEF, NED, and ENU — as a category whose morphisms are conversions between frames. The WGS84 ellipsoid is the default reference body; Bowring's iterative geodetic↔ECEF conversion and the NED↔ENU involution are both verified at test time, and an internal `NedToEnuFunctor` proves the swap is an isometric self-inverse.

Key references:
- NIMA TR8350.2 2000: *Department of Defense World Geodetic System 1984*
- Torge & Müller 2012: *Geodesy* (4th ed.), Chapter 5
- Bowring 1976: *Transformation from spatial to geographical coordinates*
- Groves 2013: *Principles of GNSS, Inertial, and Multisensor Integrated Navigation Systems* (coordinate frames)

## Entities (4)

| Category | Entities |
|---|---|
| Global (2) | Geodetic, ECEF |
| Local tangent plane (2) | NED, ENU |

## Category

Objects are `CoordinateSystem` variants; morphisms are `CoordinateConversion { from, to }` entries. The category captures which conversions exist between frames; the numerics live in `conversion.rs`.

## Qualities

| Quality | Type | Description |
|---|---|---|
| ComponentCount | usize | Number of scalar components of a coordinate in this frame (3 for all four) |

## Axioms (8)

| Axiom | Description | Source |
|---|---|---|
| GeodeticEcefRoundtrip | Geodetic → ECEF → Geodetic is identity to 1e-10 rad / 1 cm tolerance | Bowring 1976 |
| NedEnuRoundtrip | NED → ENU → NED is identity (involution) | standard |
| NedEnuIsometry | NED → ENU preserves Euclidean distance | standard |
| GreatCircleSymmetry | d(a,b) = d(b,a) | metric space |
| GreatCircleSelfZero | d(p,p) = 0 | metric space |
| GreatCircleTriangleInequality | d(a,c) ≤ d(a,b) + d(b,c) (1 m tolerance for spherical approximation) | metric space |
| Wgs84Consistency | b = a(1−f) and e² = 2f − f² | NIMA TR8350.2 2000 |
| NedEnuFunctorIdentity | NedToEnu functor preserves identity: F(id_A) = id_{F(A)} | category law |

Plus the auto-generated structural axioms from `define_ontology!` (category laws on the kinded relation graph).

## Functors

`NedToEnuFunctor` (internal endofunctor on `GeodesyCategory`) swaps NED ↔ ENU objects and rewrites morphisms accordingly; `NedEnuFunctorIdentity` proves it preserves identity morphisms.

No cross-domain functors yet — see [Compose via functor](../../../../../docs/use/compose-via-functor.md) to add one. Navigation, sensor fusion, and kinematics compose against this ontology via the concrete `Geodetic` / `Ecef` / `Ned` / `Enu` value types.

## Files

- `ontology.rs` -- `CoordinateSystem` entity, category, `ComponentCount` quality, 8 axioms, `NedToEnuFunctor`, tests
- `coordinate.rs` -- `Geodetic`, `Ecef`, `Ned`, `Enu` value types and NED↔ENU conversions
- `ellipsoid.rs` -- `Ellipsoid` struct and WGS84 parameters (NIMA TR8350.2)
- `conversion.rs` -- geodetic↔ECEF (Bowring 1976) and great-circle distance
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
