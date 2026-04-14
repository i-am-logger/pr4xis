# Measurement -- VIM measurement concepts + Stevens scale types

Models the science of quantification as a ten-concept category grounded in the *International Vocabulary of Metrology* (VIM, JCGM 200:2012) and Stevens' (1946) theory of measurement scales. Every `Result` carries `Uncertainty` (VIM axiom — a bare number is not a measurement result), traces to a reference via `Traceability`, and has a Stevens `ScaleType` that constrains which statistics are permissible on it.

Key references:
- JCGM 200:2012 (VIM): *International Vocabulary of Metrology* (measurand, result, uncertainty, traceability)
- Stevens 1946: *On the Theory of Scales of Measurement* (nominal/ordinal/interval/ratio)
- Krantz, Luce, Suppes & Tversky 1971: *Foundations of Measurement*
- JCGM 100:2008 (GUM): *Guide to the Expression of Uncertainty in Measurement*
- QUDT — Quantities, Units, Dimensions, Types (W3C ontology)

## Entities (10)

| Category | Entities |
|---|---|
| Target and process (3) | Measurand, Measurement, Procedure |
| Outputs (3) | Indication, Result, Uncertainty |
| Reference and standards (3) | Unit, Traceability, Principle |
| Meta (1) | ScaleType |

## Category

Morphisms: `Measurement Targets Measurand`, `Measurement Produces Result`, `Result Carries Uncertainty`, `Result ExpressedIn Unit`, `Measurement Follows Procedure BasedOn Principle`, `Result TracesTo Traceability`, `Measurement Yields Indication CorrectedTo Result`, `Result HasScale ScaleType`. Composition closes Measurement → Uncertainty, Measurement → Principle, and Measurement → Unit.

## Qualities

| Quality | Type | Description |
|---|---|---|
| ScaleKindQuality | ScaleKind | Result = Ratio (default); ScaleType = None (meta-concept); others = None |

Plus the `ScaleKind` enum (Nominal / Ordinal / Interval / Ratio) with methods `permits_mean`, `permits_median`, `permits_ratio` encoding Stevens' rules for permissible statistics.

## Axioms

| Axiom | Description | Source |
|---|---|---|
| (structural) | Identity and composition laws over the measurement kinded relation graph | auto-generated |

The domain content is encoded as morphism edges and enum methods verified by unit tests: Result MUST carry Uncertainty (VIM 2.9), Measurement Produces Result (VIM 2.1), Measurement Targets Measurand (VIM 2.3), Result has Traceability (VIM 2.41), Indication CorrectedTo Result (Krantz 1971 homomorphism), Nominal permits only mode, Ordinal permits median not mean, Interval permits mean not ratio, Ratio permits everything (Stevens 1946).

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one. Measurement is a foundational ontology that applied domains (sensor fusion, benchmarking, calibration) will compose against.

## Files

- `ontology.rs` -- `MeasurementConcept`, measurement category, ScaleKind enum + permissible-statistics methods, tests
- `benchmark.rs` -- Benchmark measurement support
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
