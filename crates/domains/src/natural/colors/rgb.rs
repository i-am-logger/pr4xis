#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

/// A color channel value (0-255).
pub type Channel = u8;

/// RGB color with enforcement: all values clamped to valid range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb {
    pub r: Channel,
    pub g: Channel,
    pub b: Channel,
}

impl Rgb {
    pub const BLACK: Self = Rgb { r: 0, g: 0, b: 0 };
    pub const WHITE: Self = Rgb {
        r: 255,
        g: 255,
        b: 255,
    };
    pub const RED: Self = Rgb { r: 255, g: 0, b: 0 };
    pub const GREEN: Self = Rgb { r: 0, g: 255, b: 0 };
    pub const BLUE: Self = Rgb { r: 0, g: 0, b: 255 };
    pub const YELLOW: Self = Rgb {
        r: 255,
        g: 255,
        b: 0,
    };
    pub const CYAN: Self = Rgb {
        r: 0,
        g: 255,
        b: 255,
    };
    pub const MAGENTA: Self = Rgb {
        r: 255,
        g: 0,
        b: 255,
    };

    pub fn new(r: Channel, g: Channel, b: Channel) -> Self {
        Self { r, g, b }
    }

    /// Parse hex color string (#RRGGBB or RRGGBB).
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        // Length AND ASCII: the byte-index slices below are valid char
        // boundaries only when every char is one byte. A 6-byte string with a
        // multibyte char (e.g. "aébcd") would otherwise panic slicing
        // mid-character — `from_hex` is public and must be total.
        if hex.len() != 6 || !hex.is_ascii() {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Rgb::new(r, g, b))
    }

    /// Convert to hex string (#rrggbb).
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Luminance (perceived brightness, 0.0-1.0).
    ///
    /// This is the WCAG 2.1 §1.4.3 relative luminance: a dimensionless
    /// ratio normalised to `[0, 1]` from the sRGB channels directly (not
    /// the sRGB-linearized BT.709 formula `srgb::relative_luminance` uses —
    /// see that function's doc comment for the distinction), so it is
    /// carried as `Quantity` with unit `UNITLESS`, not an absolute
    /// photometric luminance in cd/m².
    pub fn luminance(&self) -> Quantity {
        Quantity::from_unit(
            0.2126 * (self.r as f64 / 255.0)
                + 0.7152 * (self.g as f64 / 255.0)
                + 0.0722 * (self.b as f64 / 255.0),
            &unit::UNITLESS,
        )
    }

    /// Is this a dark color? (luminance < 0.5)
    pub fn is_dark(&self) -> bool {
        self.luminance().value < 0.5
    }

    /// Contrast ratio against another color (WCAG formula).
    /// Returns 1.0 to 21.0, a dimensionless ratio by definition.
    pub fn contrast_ratio(&self, other: Rgb) -> Quantity {
        let l1 = self.luminance().value + 0.05;
        let l2 = other.luminance().value + 0.05;
        Quantity::from_unit(if l1 > l2 { l1 / l2 } else { l2 / l1 }, &unit::UNITLESS)
    }

    /// WCAG AA compliance: contrast ratio >= 4.5 for normal text.
    pub fn wcag_aa(&self, other: Rgb) -> bool {
        self.contrast_ratio(other).value >= 4.5
    }

    /// WCAG AAA compliance: contrast ratio >= 7.0 for normal text.
    pub fn wcag_aaa(&self, other: Rgb) -> bool {
        self.contrast_ratio(other).value >= 7.0
    }

    /// Hue in degrees (0-360). Returns None for achromatic colors.
    pub fn hue(&self) -> Option<Quantity> {
        let r = self.r as f64 / 255.0;
        let g = self.g as f64 / 255.0;
        let b = self.b as f64 / 255.0;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        if delta < 0.001 {
            return None; // achromatic
        }

        let hue = if (max - r).abs() < 0.001 {
            60.0 * (((g - b) / delta) % 6.0)
        } else if (max - g).abs() < 0.001 {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        Some(Quantity::from_unit(
            if hue < 0.0 { hue + 360.0 } else { hue },
            &unit::DEGREE,
        ))
    }

    /// Saturation (0.0-1.0), a dimensionless ratio.
    pub fn saturation(&self) -> Quantity {
        let r = self.r as f64 / 255.0;
        let g = self.g as f64 / 255.0;
        let b = self.b as f64 / 255.0;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        Quantity::from_unit(
            if max < 0.001 { 0.0 } else { (max - min) / max },
            &unit::UNITLESS,
        )
    }

    /// Is this an achromatic color (gray/black/white)?
    pub fn is_achromatic(&self) -> bool {
        self.r == self.g && self.g == self.b
    }

    /// Invert the color.
    pub fn invert(&self) -> Rgb {
        Rgb::new(255 - self.r, 255 - self.g, 255 - self.b)
    }

    /// Grayscale version (using luminance).
    pub fn grayscale(&self) -> Rgb {
        let l = (self.luminance().value * 255.0) as u8;
        Rgb::new(l, l, l)
    }
}

#[cfg(test)]
mod totality_tests {
    use super::Rgb;

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn from_hex_is_total_over_non_ascii() {
        // 6-BYTE strings containing a multibyte char must return None, not
        // panic slicing mid-character. `from_hex` is public and must be total.
        assert!(Rgb::from_hex("aébcd").is_none()); // é occupies bytes 1..3
        assert!(Rgb::from_hex("ééé").is_none()); // 3×2 = 6 bytes
        assert!(Rgb::from_hex("ff8800").is_some()); // valid ASCII still parses
    }
}
