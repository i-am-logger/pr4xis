# praxis-colors

[![crates.io](https://img.shields.io/crates/v/praxis-colors.svg)](https://crates.io/crates/praxis-colors)
[![docs.rs](https://img.shields.io/docsrs/praxis-colors)](https://docs.rs/praxis-colors)

RGB color theory with mixing modes and WCAG accessibility compliance.

Part of the [Praxis](https://github.com/i-am-logger/praxis) framework.

## Overview

Models RGB colors as typed values with perceptual properties -- luminance, hue, saturation -- and enforces accessibility standards. Provides four color mixing modes (additive, average, multiply, screen) and WCAG contrast-ratio checks for AA/AAA compliance. Colors carry enough structure for theming, blending, and palette generation.

## Key Types

| Type | Description |
|---|---|
| `Rgb` | RGB color with luminance, hue, saturation, WCAG contrast checks, inversion, and grayscale |
| `Channel` | A color channel value (`u8`, 0-255) |
| `MixMode` | Mixing modes: `Additive`, `Average`, `Multiply`, `Screen` |
| `mix` | Mix two colors using a given mode |
| `blend` | Alpha-blend a foreground over a background |
| `complement` | Compute the complementary (inverted) color |
| `mix_many` | Average an arbitrary number of colors |

## Example

```rust
use praxis_colors::{Rgb, MixMode, mix, blend};

let red = Rgb::RED;
let blue = Rgb::BLUE;

// Screen blend (lightening)
let purple = mix(red, blue, MixMode::Screen);

// WCAG accessibility check
assert!(Rgb::BLACK.wcag_aa(Rgb::WHITE));  // 21:1 contrast
assert!(red.is_dark());

// Alpha blending
let half = blend(Rgb::WHITE, Rgb::BLACK, 0.5);
```

## License

CC BY-NC-SA 4.0
