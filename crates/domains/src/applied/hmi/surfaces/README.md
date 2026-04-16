# Surfaces -- Abstract render targets for theming and content pipelines

A `Surface` is anything that renders colors and content from a theme palette: terminal emulators, window managers, hardware RGB devices, shaders, browsers, chat UIs, PDF generators. Theme application is a **functor** from `ThemeCategory` to `SurfaceCategory`; a theme change is a **natural transformation** that updates all surface functors consistently.

This ontology defines the abstract framework. Concrete surfaces (wezterm, hyprland, openrgb, etc.) are instances of the framework.

Key references:
- Mac Lane 1971: *Categories for the Working Mathematician* — functors, natural transformations
- Harel 1987: *Statecharts: a visual formalism for complex systems* — parallel regions (surfaces update simultaneously)
- Czaplicki & Chong 2013: *Async FRP for GUIs* (PLDI 2013) — synchronous vs asynchronous propagation
- Beaudouin-Lafon 2000: *Instrumental Interaction* (CHI 2000) — the interaction-algebra framing
- Thimbleby 2004: *User Interface Design with Matrix Algebra* (ACM TOCHI)

## Entities

| Category | Entities |
|---|---|
| Capabilities | `SurfaceCapability` — Ansi16, TrueColor, HdrRgb, AlphaBlending, GpuShader, … |
| Types | `SurfaceType { name, capabilities }` — concrete render target |
| Mappings | `SlotMapping { slot, key, transform }` — slot → surface-specific key |
| Transforms | `ColorTransform` — Hex, RrGgBb, Rgba, HexNoHash, Srgb |
| Functor | `SurfaceFunctor { surface, mappings }` — a concrete theme→surface functor |
| Naturality | `ThemeChangeNaturality { functors }` — the natural transformation over all functors |

## Category

`SurfaceCategory` is discrete over `SurfaceType` objects. Morphisms are the `SurfaceFunctor` instances — each pairing a palette slot with a surface-specific configuration key and a color transform. Composition threads `SlotMapping`s; identity is the empty mapping set.

## Qualities

| Quality | Type | Description |
|---|---|---|
| SlotCoverage | usize | How many palette slots a given `SurfaceCapability` covers (e.g., `Ansi16` covers 16, `TrueColor` covers all 24) |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| (structural) | Identity and composition laws over `SurfaceCategory` | auto-generated |
| ThemeChangeIsNatural | Changing the active theme updates every surface functor in parallel, preserving the naturality square | Mac Lane 1971 |

Domain axioms and their `impl Axiom for` blocks live in `ontology.rs`; the category-law check is via the auto-generated `check_category_laws::<SurfaceCategory>`.

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one. The surfaces ontology is the **target** of the theme-application functor from `applied/hmi/theming/`: each concrete surface (wezterm, hyprland, ghostty, kitty, chromium userstyle, etc.) receives the active palette through a `SurfaceFunctor` instance. A future PR will add `BrowserSurface`, `ChatSurface`, `PDFSurface`, and `PrintSurface` for the pr4xis.dev site work — each will be a new `SurfaceType` instance.

## Files

- `ontology.rs` -- `SurfaceCapability`, `SurfaceType`, `SlotMapping`, `ColorTransform`, `SurfaceFunctor`, `ThemeChangeNaturality`, `SlotCoverage`, category impl, tests
- `README.md` -- this file
- `citings.md` -- per-ontology bibliography
- `mod.rs` -- module declarations

Previous home: `applied/theming/surfaces.rs`, moved here by [#66](https://github.com/i-am-logger/pr4xis/pull/66).
