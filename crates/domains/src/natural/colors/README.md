# Colors -- RGB primaries, sRGB color science, mixing

Models the eight RGB primary colors as a discrete category and carries the sRGB transfer function, luminance, and WCAG contrast machinery against which mixing operations are validated. The ontology itself lives on `PrimaryColor`; the continuous color science sits in `srgb.rs` and is consumed by the engine and mixing layer.

Key references:
- IEC 61966-2-1: *sRGB standard* (transfer function)
- ITU-R BT.709-6 (luma coefficients 0.2126, 0.7152, 0.0722)
- W3C WCAG 2.1 (relative luminance, contrast ratio, compliance levels)

## Entities (8)

| Category | Entities |
|---|---|
| Additive primaries (3) | Red, Green, Blue |
| Subtractive primaries (3) | Cyan, Magenta, Yellow |
| Achromatic (2) | Black, White |

## Category

Discrete category over the eight `PrimaryColor` variants with a `ColorMix` relation kind. Mixing, complements, and sRGB-space operations live as functions over `Rgb`, not as category morphisms.

## Qualities

| Quality | Type | Description |
|---|---|---|
| Luminance | f64 | Relative luminance of the RGB value (BT.709 luma) |
| IsPrimary | () | Holds for Red, Green, Blue (additive primaries) |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| ComplementsAddToWhite | Complement pairs (R/C, G/M, B/Y) saturate to white in every channel | standard |

Plus the auto-generated structural axioms from `define_ontology!` (category laws on the discrete category).

The sRGB layer in `srgb.rs` carries additional axioms (`SrgbContinuity`, `LumaConvex`, `LuminanceBounded`, `ContrastBounded`, `LuminanceMonotone`, `ScreenDualOfMultiply`) sourced from IEC 61966-2-1 and WCAG 2.1; these validate the continuous color science used by mixing and the engine.

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../docs/use/compose-via-functor.md) to add one. Theming and visualization compose against this ontology via the `Rgb` value type.

## Files

- `ontology.rs` -- `PrimaryColor` entity, discrete category, `Luminance`/`IsPrimary` qualities, `ComplementsAddToWhite` axiom, tests
- `rgb.rs` -- `Rgb` struct with channel clamping and canonical constants
- `srgb.rs` -- sRGB EOTF, relative luminance, WCAG contrast ratios, and sRGB-space axioms
- `mixing.rs` -- `MixMode` (Additive, Average, Multiply, Screen), `blend`, `mix`, `complement`
- `engine.rs` -- `Situation`/`Action`/`Precondition` wiring over `Rgb`
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
