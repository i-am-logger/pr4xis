#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::rgb::Rgb;

/// Color mixing modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MixMode {
    /// Additive (light): RGB channels added, clamped to 255.
    Additive,
    /// Average: weighted mean of channels.
    Average,
    /// Multiply: channels multiplied (darkening).
    Multiply,
    /// Screen: inverse multiply (lightening).
    Screen,
}

/// Mix two colors with a given mode.
pub fn mix(a: Rgb, b: Rgb, mode: MixMode) -> Rgb {
    match mode {
        MixMode::Additive => Rgb::new(
            a.r.saturating_add(b.r),
            a.g.saturating_add(b.g),
            a.b.saturating_add(b.b),
        ),
        MixMode::Average => Rgb::new(
            ((a.r as u16 + b.r as u16) / 2) as u8,
            ((a.g as u16 + b.g as u16) / 2) as u8,
            ((a.b as u16 + b.b as u16) / 2) as u8,
        ),
        MixMode::Multiply => Rgb::new(
            ((a.r as u16 * b.r as u16) / 255) as u8,
            ((a.g as u16 * b.g as u16) / 255) as u8,
            ((a.b as u16 * b.b as u16) / 255) as u8,
        ),
        MixMode::Screen => {
            let inv_a = a.invert();
            let inv_b = b.invert();
            let mult = mix(inv_a, inv_b, MixMode::Multiply);
            mult.invert()
        }
    }
}

/// Blend color `fg` over `bg` with alpha (0.0 = fully bg, 1.0 = fully fg).
pub fn blend(bg: Rgb, fg: Rgb, alpha: f64) -> Rgb {
    let alpha = alpha.clamp(0.0, 1.0);
    let inv = 1.0 - alpha;
    Rgb::new(
        (bg.r as f64 * inv + fg.r as f64 * alpha) as u8,
        (bg.g as f64 * inv + fg.g as f64 * alpha) as u8,
        (bg.b as f64 * inv + fg.b as f64 * alpha) as u8,
    )
}

/// Complementary color (opposite on color wheel).
pub fn complement(color: Rgb) -> Rgb {
    color.invert()
}

/// Mix many colors by averaging.
pub fn mix_many(colors: &[Rgb]) -> Option<Rgb> {
    if colors.is_empty() {
        return None;
    }
    let n = colors.len() as u32;
    let r: u32 = colors.iter().map(|c| c.r as u32).sum();
    let g: u32 = colors.iter().map(|c| c.g as u32).sum();
    let b: u32 = colors.iter().map(|c| c.b as u32).sum();
    Some(Rgb::new((r / n) as u8, (g / n) as u8, (b / n) as u8))
}
