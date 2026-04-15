# Visualization -- Formal model of visual encoding and perception

Formalizes the foundational visualization literature as ontological structures with axioms that can be verified and composed. The four pillars: Bertin's visual variables, Cleveland & McGill's perceptual accuracy ranking, Munzner's channel effectiveness, and Wickham's layered grammar of graphics.

The visualization ontology is the **middle layer** of the HMI stack: data-with-measurement-levels flows in from ontologies, visual encodings are chosen, geometry is composed via the grammar-of-graphics pipeline, and the result is rendered onto a surface via the `surfaces` ontology. Each stage is a functor.

Key references:
- Bertin 1967: *Semiology of Graphics* — the 7 visual variables
- Cleveland & McGill 1984: *Graphical Perception* (JASA 79:387) — perceptual accuracy ranking
- Munzner 2014: *Visualization Analysis and Design* — channel effectiveness, nested model
- Shneiderman 1996: *The Eyes Have It* (IEEE VL) — overview-zoom-filter-details mantra
- Wickham 2010: *A Layered Grammar of Graphics* (J Comp Graph Stat 19:3) — compositional pipeline
- Stevens 1946: *On the Theory of Scales of Measurement* — nominal/ordinal/interval/ratio

## Entities

| Category | Entities |
|---|---|
| Bertin's visual variables (7) | Position, Size, Value, Texture, Color, Orientation, Shape |
| Perceptual tasks (6) | PositionCommonScale, PositionNonAligned, Length, Angle, Area, Volume |
| Data levels (Stevens) (4) | Nominal, Ordinal, Interval, Ratio |
| Geometry types | Point, Line, Bar, Area, Text, Rect |
| Interaction levels (Shneiderman) | Overview, ZoomFilter, DetailsOnDemand |
| Grammar layers (Wickham) | Data, Aesthetics, Geom, Stat, Scale, Coord, Facet |

## Qualities

| Quality | Type | Description |
|---|---|---|
| BertinProperties | `VariableProperties` | Per visual variable: associative (can group), selective (can isolate), ordered (has natural order), quantitative (supports magnitude judgment), length/size of alphabet |
| AccuracyRank | usize | Cleveland-McGill perceptual task ranking (1 = most accurate, 6 = least) |
| GeomUseCase | &'static str | Recommended geom for each data-level × interaction-level combination |
| RecommendedVis | &'static str | Interaction-level → recommended overview/zoom/detail geom (Shneiderman) |
| PipelineOrder | usize | Wickham grammar-of-graphics layer order (Data → Aesthetics → … → Facet) |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| (structural) | Identity and composition laws | auto-generated |
| PositionIsMostAccurate | `PositionCommonScale` ranks first in Cleveland-McGill's perceptual hierarchy | Cleveland & McGill 1984 |
| AreaIsLessAccurateThanLength | Length judgments beat area judgments | Cleveland & McGill 1984 |
| ShneidermanMantraOrdered | Overview → ZoomFilter → DetailsOnDemand in that order | Shneiderman 1996 |
| GrammarLayersOrdered | Wickham's pipeline is strictly ordered: Data precedes everything, Facet is last | Wickham 2010 |

See `ontology.rs` for the full `impl Axiom for …` blocks.

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one. The visualization ontology is consumed by `applied/hmi/report/spec.rs` (which picks encoding grammar from data level) and by `applied/hmi/explorer/ontology.rs` (which uses the visual-variables + accuracy ranking to lay out the reasoning-trace graph).

## Files

- `ontology.rs` -- `VisualVariable`, `PerceptualTask`, `DataLevel`, `GeomType`, `InteractionLevel`, `GrammarLayer`, qualities, axioms, tests
- `README.md` -- this file
- `citings.md` -- per-ontology bibliography
- `mod.rs` -- module declarations

Previous home: `applied/theming/visualization.rs`, moved here by [#66](https://github.com/i-am-logger/pr4xis/pull/66).
