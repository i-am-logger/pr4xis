# Temporal -- Instants, intervals, durations, time systems, Allen's interval algebra

Models time as a category whose objects are time systems (TAI, GPS, UTC, Unix, TT, TCB, MET) and whose morphisms are the conversions between them. Layered on top: instants, durations, intervals, clock error models, and Allen's 13 interval relations. The axioms verify the total order of time, the duration metric (identity, antisymmetry, additivity), Allen's joint exhaustivity and inverse law, and the IS-GPS-200 / IAU 2000 fixed offsets between time scales.

Key references:
- Allen 1983: *Maintaining Knowledge about Temporal Intervals* (CACM)
- IS-GPS-200: GPS Interface Specification (GPS = TAI - 19 s)
- IAU 2000 Resolution B1.9 (TT = TAI + 32.184 s)
- BIPM: *International System of Units* (UTC, TAI definitions)

## Entities (7 time systems)

| Category | Entities |
|---|---|
| Continuous atomic / dynamical (4) | TAI, GPS, TT, TCB |
| Civil with leap seconds (2) | UTC, Unix |
| Mission-elapsed (1) | MET |

## Category

The objects are the seven `TimeSystem` variants; the morphisms (`TimeSystemConversion`) are the offset and scaling functions that map an instant in one system to the corresponding instant in another. Conversion is closed under composition and an identity conversion exists for each system.

## Qualities

| Quality | Type | Description |
|---|---|---|
| HasLeapSeconds | bool | UTC=true, Unix=true; TAI/GPS/TT/TCB/MET=false |
| IsContinuous | bool | TAI/GPS/TT/TCB/MET=true; UTC/Unix=false (leap seconds break continuity) |

## Axioms (9)

| Axiom | Description | Source |
|---|---|---|
| TotalOrder | For any two instants, exactly one of <, =, > holds | metric/order |
| DurationNonNegativity | Duration from earlier to later instant is positive | metric |
| DurationIdentity | d(a,a) = 0 | metric |
| DurationAntisymmetry | d(a,b) = -d(b,a) | metric |
| DurationAdditivity | d(a,b) + d(b,c) = d(a,c) | metric |
| AllenExhaustive | The 13 Allen relations are jointly exhaustive and pairwise disjoint | Allen 1983 |
| AllenInverseLaw | If R(X,Y) then R^{-1}(Y,X) | Allen 1983 |
| GpsTaiConversion | GPS = TAI - 19 seconds (fixed offset) | IS-GPS-200 |
| TtTaiConversion | TT = TAI + 32.184 seconds | IAU 2000 Resolution B1.9 |

Plus the auto-generated structural axioms from `define_ontology!` (category laws on the time-system conversion category).

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one. The applied sensor-fusion `SensorTimeOntology` defines its own time-system category that mirrors much of this one; an explicit functor between the two would replace that duplication.

## Files

- `ontology.rs` -- Time-system category, HasLeapSeconds and IsContinuous qualities, 9 axioms, tests
- `time_system.rs` -- `TimeSystem` enum and the `convert` function (offsets between scales)
- `instant.rs` -- `Instant { seconds, system }`: `is_before`, `duration_to`
- `duration.rs` -- `Duration` as a metric on instants (d(t1,t2) = |t2 - t1|)
- `interval.rs` -- `Interval { start, end }` bounded temporal region
- `allen.rs` -- Allen's 13 interval relations and `relate(x, y, eps)` discriminator
- `clock.rs` -- Clock error model (offset, drift) layered on `Duration`
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
