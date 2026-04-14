# Theming -- Color Scheme Ontology

Models color schemes as formal structures: base16/base24 slots as objects, bright-variant-of as morphisms, and semantic roles, ANSI indices, and ramp positions as qualities. Palette axioms enforce scheme invariants every valid theme must satisfy: the base00–base07 monotone luminance ramp, WCAG AA foreground/background contrast, and bright variants being brighter than their base counterparts. Polarity (dark/light) is derived from base00 relative luminance.

Key references:
- Base16 styling spec: tinted-theming/home/styling.md
- Base24 styling spec: tinted-theming/base24/styling.md
- CIE 1931 colorimetry / sRGB relative luminance
- WCAG 2.1 SC 1.4.3: contrast requirements for accessibility
- ECMA-48 5th Ed 1991: SGR parameters 30-37, 90-97 for ANSI colors
- Porter & Duff 1984: *Compositing Digital Images*
- Bertin 1967: *Semiology of Graphics* (visual variables)
- Cleveland & McGill 1984: *Graphical Perception* (perceptual task ranking)
- Harel 1987: *Statecharts* (mode graphs, parallel regions)

## Entities

| Category | Entities |
|---|---|
| Color slots | Base00 .. Base0F (base16), Base10 .. Base17 (base24 extensions) |

Semantic roles: Background, Foreground, Accent, Surface, etc. (see `base16::SemanticRole`).

## Category

Objects: `ColorSlot`. Morphisms: `BrightVariantOf { bright, base }`. Identity: self-variant. Composition: `(a→b) ∘ (b→c) = (a→c)`. A secondary relation `AnsiMapping` records ANSI terminal index assignments.

## Qualities

| Quality | Type | Description |
|---|---|---|
| SlotRole | SemanticRole | Background / Foreground / Accent / Surface / etc. |
| AnsiIndex | u8 | ANSI terminal index for each slot that has one |
| RampPosition | u8 | Position in the monotone luminance ramp (0..7 for base00..base07) |

## Axioms (palette-parameterised)

| Axiom | Description | Source |
|---|---|---|
| LuminanceMonotonicity | base00-base07 form a monotone (all-increasing or all-decreasing) luminance ramp | base16 styling spec |
| WcagForegroundContrast | base05 over base00 meets WCAG AA (≥ 4.5:1) | WCAG 2.1 SC 1.4.3 |
| BrightVariantBrighter | Each base24 bright slot has higher relative luminance than its base counterpart | base24 styling spec |
| (structural) | Identity and composition laws over the ThemingCategory | auto-generated |

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../docs/use/compose-via-functor.md) to add one. The theming ontology composes against `natural/colors` (sRGB, relative luminance, WCAG) rather than exposing functors of its own.

## Files

- `ontology.rs` -- `Palette`, `ThemingCategory`, `SlotRole`/`AnsiIndex`/`RampPosition` qualities, palette axioms, `detect_polarity`
- `base16.rs` -- `ColorSlot`, `SemanticRole`, `Polarity`, slot metadata (ANSI index, ramp position, bright-variant-of)
- `schemes.rs` -- bundled base16/base24 color schemes
- `variants.rs` -- theme variants (dark/light, high-contrast, etc.)
- `theme_package.rs` -- composite theme package structure
- `shader_params.rs` -- shader uniforms derived from a palette
- `surfaces.rs` -- rendering surfaces (terminal, editor, web) the theme targets
- `modes.rs` -- mode graph (Harel statecharts) for UI modes
- `keybindings.rs` -- keybinding tables tied to modes
- `visualization.rs` -- Bertin / Cleveland-McGill visual variables
- `report.rs` -- theme report types
- `report_spec.rs` -- report specification
- `validate_themes.rs` -- end-to-end theme validation pass
- `explorer.rs` -- self-referential theme-explorer UI
- `papers/draft-color-theming-ontology.md` -- draft paper on the theming ontology (not modified here)
- `mod.rs` -- module declarations
