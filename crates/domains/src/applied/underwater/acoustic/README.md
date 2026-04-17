# Acoustic -- Underwater Positioning Ontology

Models the three canonical underwater acoustic positioning systems (USBL, LBL, SBL) and their accuracy characteristics. The category is fully connected because systems can be combined: USBL calibrated against LBL, SBL fused with LBL, etc. Physical bounds on sound speed and range appear as axioms.

Key references:
- Milne 1983: *Underwater Acoustic Positioning Systems*
- Vickery 1998: *Acoustic Positioning Systems — A Practical Overview*
- Kinsey, Eustice & Whitcomb 2006: *A Survey of Underwater Vehicle Navigation*

## Entities (3)

| Category | Entities |
|---|---|
| Acoustic systems (3) | USBL, LBL, SBL |

## Category

`AcousticOntology`/`AcousticCategory`/`AcousticConcept` via `pr4xis::ontology!`, relation `AcousticRelation`, fully connected.

## Qualities

| Quality | Type | Description |
|---|---|---|
| PositioningAccuracy | &'static str | USBL=0.1-1% slant range, LBL=0.01-0.1 m within baseline, SBL=0.1-1% slant range |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| SoundSpeedPositive | Sound speed in seawater is strictly positive (typically 1400-1600 m/s) | Milne 1983 |
| RangeNonNegative | Acoustic range (= sound_speed × travel_time / 2) is non-negative | Milne 1983 |
| (structural) | Identity and composition laws over the AcousticCategory | auto-generated |

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one.

## Files

- `ontology.rs` -- `AcousticConcept`, `AcousticCategory`, `PositioningAccuracy` quality, sound-speed and range axioms
- `engine.rs` -- positioning engine used by tests
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
