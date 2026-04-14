# Terrain -- Terrain-relative navigation features

Models topographic features of a digital elevation model as ontology entities and attaches a curvature signature to each. Terrain-relative navigation compares measured elevation to a map, and the feature taxonomy (peaks, valleys, ridges, saddles) is what the matcher uses to register a pose against a DEM.

Key references:
- Goldstein 1987: *Terrain Aided Navigation*
- Bergen, Dickmann, Haykin 1996 (terrain contour matching)

## Entities (4)

| Category | Entities |
|---|---|
| Extrema (2) | Peak, Valley |
| Lines (2) | Ridge, Saddle |

## Qualities

| Quality | Type | Description |
|---|---|---|
| CurvatureSignature | (i8, i8) | Sign pair of the two principal curvatures: Peak=(−,−), Valley=(+,+), Ridge=(−,0), Saddle=(−,+) |

## Axioms (2)

| Axiom | Description | Source |
|---|---|---|
| ElevationBounded | DEM elevation values are bounded within a finite range | standard (physical bound on any real DEM) |
| PeakCurvatureNegative | Peaks have negative principal curvatures (local maxima) | differential geometry of surfaces |

Plus the auto-generated structural axioms from `define_ontology!` (category laws for `TerrainCategory`).

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one. A natural extension is a functor from this feature ontology into the localization / SLAM ontologies so that terrain features can participate in landmark observation.

## Files

- `ontology.rs` -- `TerrainFeature` entity, `TerrainCategory`, `CurvatureSignature` quality, 2 axioms, tests
- `engine.rs` -- `DemTile` runtime type for DEM-backed reasoning
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
