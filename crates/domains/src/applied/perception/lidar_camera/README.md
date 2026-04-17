# LiDAR-Camera -- Fusion Pipeline Ontology

Models the LiDAR+camera sensor-fusion pipeline as a category whose objects are the four pipeline stages (Detection → Projection → Association → Fusion) and whose morphisms are forward transitions plus their transitive closures. The pipeline is strictly sequential: no backward morphisms exist. Projection respects depth ordering under the pinhole camera model.

Key references:
- Caltagirone, Bellone, Svensson & Wahde 2019: *LiDAR-Camera Fusion for Road Detection*
- Geiger, Lenz & Urtasun 2012: *KITTI* dataset and calibration
- Zhang 2000: *A Flexible New Technique for Camera Calibration*

## Entities (4)

| Category | Entities |
|---|---|
| Fusion stages (4) | Detection, Projection, Association, Fusion |

## Pipeline

```mermaid
graph LR
    Detection --> Projection --> Association --> Fusion
```

Category: `LidarCameraCategory`. Morphisms: identities on each stage plus forward steps and transitive closures (Detection→Association, Detection→Fusion, Projection→Fusion). No backward morphisms.

## Qualities

| Quality | Type | Description |
|---|---|---|
| StageDescription | &'static str | Natural-language description of each fusion stage |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| ProjectionPreservesOrdering | Projection preserves depth ordering of LiDAR points (pinhole model) | Zhang 2000 |
| PipelineIsSequential | Fusion pipeline stages execute in order; no backward morphisms | Caltagirone et al. 2019 |
| (structural) | Identity and composition laws over the LidarCameraCategory | auto-generated |

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one.

## Files

- `ontology.rs` -- `LidarCameraConcept`, `LidarCameraRelation`, `LidarCameraCategory`, `StageDescription` quality, projection and sequentiality axioms
- `calibration.rs` -- camera/LiDAR calibration utilities
- `engine.rs` -- fusion engine used by tests
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
