# Report -- Data → visualization → surface → frozen artifact pipeline

Models the act of validating ontology data against axioms, picking visual encodings via the visualization ontology, rendering to a target surface, and freezing the result as a report (JSON, HTML, SVG, PDF). The report is a **functor** from `ValidationResults → OutputFormat`; different surfaces consume the same underlying data through different branches of the functor.

The three sub-modules break the pipeline into inert stages:

1. **`validator.rs`** — runs axioms over ontology instances, emits `ThemeResult` / `ValidationDetail` records
2. **`spec.rs`** — picks visual encodings per data level using the visualization ontology, produces a `ReportSpec`
3. **`generator.rs`** — renders a `ReportSpec` + results into a target format (JSON, HTML), styled from the active theme via `ThemePalette`

This is pr4xis's prior art for "the report as a forgetful functor from dynamic ontology state to a frozen artifact" — see also `formal/meta/staging/` which lifts this pattern to Futamura's partial-evaluation framework.

Key references:
- W3C EARL 1.0 — *Evaluation And Report Language*: `earl:Assertion`, `earl:subject`, `earl:test`, `earl:result`
- Vega-Lite 2017 (Satyanarayan et al.) — declarative visualization grammar
- Wickham 2010 — *Layered Grammar of Graphics*
- Shneiderman 1996 — overview-zoom-filter-details mantra
- Tufte 1983 — *The Visual Display of Quantitative Information*: data-ink ratio

## Entities

| Category | Entities |
|---|---|
| Validation (`validator.rs`) | `ThemeResult { theme, variant, scheme, … }`, `ValidationDetail { monotone, wcag_aa, contrast_ratio, … }` |
| Specification (`spec.rs`) | `DataField { name, level, description }`, `EncodingAssignment { field, variable, geom, rank }`, `ReportSpec { name, assignments, interaction_level }` |
| Generation (`generator.rs`) | `ThemePalette { bg, card, border, pass, fail, … }`, `Report` serialized output |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| (structural) | Identity and composition laws over the report pipeline | auto-generated |
| DefaultIsOptimal | The default encoding assignment for a given data level is the Cleveland-McGill top-ranked perceptual task | Cleveland & McGill 1984 |
| OverrideMustBeNoWorse | A human-authored encoding override must rank at least as high as the default (no worse) | soft guarantee |
| DataInkRatioBounded | Generated HTML reports cap non-data chrome to < 30% of total ink | Tufte 1983 |

See `spec.rs` and `generator.rs` for the `impl Axiom for …` blocks.

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one. The report ontology is the **consumer** of:

- `applied/hmi/theming/` — supplies the `ThemePalette` used for styling
- `applied/hmi/visualization/` — supplies `DataLevel`, `VisualVariable`, `GeomType`, `AccuracyRank`, `InteractionLevel`
- `applied/hmi/surfaces/` — supplies the concrete render target (HTML string, JSON, PDF, …)

And a producer of frozen artifacts. A future PR (#60) will wire this into CI so every test run refreshes a live `pr4xis-report.json` on GitHub Pages.

## Files

- `generator.rs` -- `ThemePalette`, HTML/JSON rendering, the functor `ValidationResults → OutputFormat`
- `spec.rs` -- `DataField`, `EncodingAssignment`, `ReportSpec`, the functor `DataOntology → VisualizationOntology → RenderSurface`
- `validator.rs` -- `ThemeResult`, `ValidationDetail`, `scan_themes(path)` — loads themes and runs the theming axioms
- `README.md` -- this file
- `citings.md` -- per-ontology bibliography
- `mod.rs` -- module declarations

Previous home: `applied/theming/report.rs` + `report_spec.rs` + `validate_themes.rs`, moved (and renamed) here by [#66](https://github.com/i-am-logger/pr4xis/pull/66).
