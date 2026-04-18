/// Theming ontology — formal structure of color schemes.
///
/// Connects base16/base24 slots to color science (sRGB, WCAG).
/// Axioms enforce scheme invariants that every valid theme must satisfy.
use super::base16::{ColorSlot, Polarity, SemanticRole};
use crate::natural::colors::rgb::Rgb;
use crate::natural::colors::srgb;
use pr4xis::category::{Category, Entity, Relationship};
use pr4xis::ontology::upper::being::Being;
use pr4xis::ontology::upper::classify::Classified;
use pr4xis::ontology::{Axiom, Quality};
use std::collections::HashMap;

/// A concrete color palette: binds each slot to an Rgb color.
pub type Palette = HashMap<ColorSlot, Rgb>;

// ── Qualities ──

/// Quality: the semantic role of a color slot.
#[derive(Debug, Clone)]
pub struct SlotRole;

impl Quality for SlotRole {
    type Individual = ColorSlot;
    type Value = SemanticRole;
    fn get(&self, slot: &ColorSlot) -> Option<SemanticRole> {
        Some(slot.role())
    }
}

/// Quality: ANSI terminal index for a slot.
#[derive(Debug, Clone)]
pub struct AnsiIndex;

impl Quality for AnsiIndex {
    type Individual = ColorSlot;
    type Value = u8;
    fn get(&self, slot: &ColorSlot) -> Option<u8> {
        slot.ansi_index()
    }
}

/// Quality: position in the monotone luminance ramp.
#[derive(Debug, Clone)]
pub struct RampPosition;

impl Quality for RampPosition {
    type Individual = ColorSlot;
    type Value = u8;
    fn get(&self, slot: &ColorSlot) -> Option<u8> {
        slot.ramp_position()
    }
}

// ── Relationships ──

/// A morphism from a base24 bright slot to its base16 origin.
///
/// Source: base24 spec — base12 is bright variant of base08, etc.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrightVariantOf {
    pub bright: ColorSlot,
    pub base: ColorSlot,
}

impl Relationship for BrightVariantOf {
    type Object = ColorSlot;
    type Kind = ();
    fn source(&self) -> ColorSlot {
        self.bright
    }
    fn target(&self) -> ColorSlot {
        self.base
    }
    fn kind(&self) {}
}

/// A morphism mapping a base16 slot to an ANSI terminal index.
///
/// Source: tinted-theming shell template convention
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnsiMapping {
    pub slot: ColorSlot,
    pub ansi: ColorSlot, // the slot that maps to the same ANSI index
}

impl Relationship for AnsiMapping {
    type Object = ColorSlot;
    type Kind = ();
    fn source(&self) -> ColorSlot {
        self.slot
    }
    fn target(&self) -> ColorSlot {
        self.ansi
    }
    fn kind(&self) {}
}

// ── Category ──

/// The theming category: slots as objects, bright-variant-of as morphisms.
pub struct ThemingCategory;

impl Category for ThemingCategory {
    type Object = ColorSlot;
    type Morphism = BrightVariantOf;

    fn identity(obj: &ColorSlot) -> BrightVariantOf {
        BrightVariantOf {
            bright: *obj,
            base: *obj,
        }
    }

    fn compose(f: &BrightVariantOf, g: &BrightVariantOf) -> Option<BrightVariantOf> {
        if f.base != g.bright {
            return None;
        }
        Some(BrightVariantOf {
            bright: f.bright,
            base: g.base,
        })
    }

    fn morphisms() -> Vec<BrightVariantOf> {
        let slots = ColorSlot::variants();
        // Identity morphisms + bright variant relationships
        let mut morphisms: Vec<BrightVariantOf> = slots
            .iter()
            .map(|s| BrightVariantOf {
                bright: *s,
                base: *s,
            })
            .collect();
        // Bright variant morphisms
        for slot in &slots {
            if let Some(base) = slot.bright_variant_of() {
                morphisms.push(BrightVariantOf {
                    bright: *slot,
                    base,
                });
            }
        }
        morphisms
    }
}

impl Classified for ThemingCategory {
    fn being() -> Being {
        Being::Quality
    }
    fn classification_reason() -> &'static str {
        "theming is visual quality specification (base16, color palettes)"
    }
}

// ── Palette Axioms ──

/// Luminance monotonicity: base00 through base07 must form an ordered ramp.
///
/// Source: base16 styling.md — the monotone scale from darkest to lightest.
/// For dark themes: L(base00) < L(base01) < ... < L(base07)
/// For light themes: L(base00) > L(base01) > ... > L(base07)
pub struct LuminanceMonotonicity {
    pub palette: Palette,
}

impl Axiom for LuminanceMonotonicity {
    fn description(&self) -> &str {
        "base00-base07 form a monotone luminance ramp (base16 spec)"
    }
    fn holds(&self) -> bool {
        let ramp_slots = [
            ColorSlot::Base00,
            ColorSlot::Base01,
            ColorSlot::Base02,
            ColorSlot::Base03,
            ColorSlot::Base04,
            ColorSlot::Base05,
            ColorSlot::Base06,
            ColorSlot::Base07,
        ];
        let luminances: Vec<f64> = ramp_slots
            .iter()
            .filter_map(|s| self.palette.get(s).map(srgb::relative_luminance))
            .collect();

        if luminances.len() < 8 {
            return false; // incomplete palette
        }

        // Must be monotone (either all increasing or all decreasing)
        let increasing = luminances.windows(2).all(|w| w[0] <= w[1]);
        let decreasing = luminances.windows(2).all(|w| w[0] >= w[1]);
        increasing || decreasing
    }
}
pr4xis::register_axiom!(LuminanceMonotonicity);

/// WCAG AA compliance: foreground slots must have >= 4.5:1 contrast against background.
///
/// Source: WCAG 2.1 SC 1.4.3
pub struct WcagForegroundContrast {
    pub palette: Palette,
}

impl Axiom for WcagForegroundContrast {
    fn description(&self) -> &str {
        "foreground (base05) has >= 4.5:1 contrast against background (base00) (WCAG AA)"
    }
    fn holds(&self) -> bool {
        let bg = match self.palette.get(&ColorSlot::Base00) {
            Some(c) => c,
            None => return false,
        };
        let fg = match self.palette.get(&ColorSlot::Base05) {
            Some(c) => c,
            None => return false,
        };
        srgb::wcag_compliant(fg, bg, srgb::WcagLevel::AA)
    }
}
pr4xis::register_axiom!(WcagForegroundContrast);

/// Bright variants must be brighter than their base counterparts.
///
/// Source: base24 spec — bright slots are lighter/more vivid versions.
pub struct BrightVariantBrighter {
    pub palette: Palette,
}

impl Axiom for BrightVariantBrighter {
    fn description(&self) -> &str {
        "bright accent variants have higher luminance than their base (base24 spec)"
    }
    fn holds(&self) -> bool {
        let pairs = [
            (ColorSlot::Base12, ColorSlot::Base08),
            (ColorSlot::Base13, ColorSlot::Base0A),
            (ColorSlot::Base14, ColorSlot::Base0B),
            (ColorSlot::Base15, ColorSlot::Base0C),
            (ColorSlot::Base16, ColorSlot::Base0D),
            (ColorSlot::Base17, ColorSlot::Base0E),
        ];
        pairs.iter().all(|(bright, base)| {
            match (self.palette.get(bright), self.palette.get(base)) {
                (Some(b), Some(n)) => srgb::relative_luminance(b) >= srgb::relative_luminance(n),
                _ => true, // skip if slots not present (base16-only palette)
            }
        })
    }
}
pr4xis::register_axiom!(BrightVariantBrighter);

/// Polarity detection: derive dark/light from base00 luminance.
///
/// Source: base16 convention — base00 is the default background.
pub fn detect_polarity(palette: &Palette) -> Option<Polarity> {
    let bg = palette.get(&ColorSlot::Base00)?;
    if srgb::is_dark(bg) {
        Some(Polarity::Dark)
    } else {
        Some(Polarity::Light)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dark_palette() -> Palette {
        let mut p = HashMap::new();
        // Catppuccin Mocha-like dark palette
        p.insert(ColorSlot::Base00, Rgb::new(30, 30, 46)); // dark bg
        p.insert(ColorSlot::Base01, Rgb::new(49, 50, 68));
        p.insert(ColorSlot::Base02, Rgb::new(69, 71, 90));
        p.insert(ColorSlot::Base03, Rgb::new(88, 91, 112));
        p.insert(ColorSlot::Base04, Rgb::new(108, 112, 134));
        p.insert(ColorSlot::Base05, Rgb::new(205, 214, 244)); // light fg
        p.insert(ColorSlot::Base06, Rgb::new(216, 222, 233));
        p.insert(ColorSlot::Base07, Rgb::new(236, 239, 244));
        // Accents
        p.insert(ColorSlot::Base08, Rgb::new(243, 139, 168)); // red
        p.insert(ColorSlot::Base09, Rgb::new(250, 179, 135)); // orange
        p.insert(ColorSlot::Base0A, Rgb::new(249, 226, 175)); // yellow
        p.insert(ColorSlot::Base0B, Rgb::new(166, 227, 161)); // green
        p.insert(ColorSlot::Base0C, Rgb::new(148, 226, 213)); // cyan
        p.insert(ColorSlot::Base0D, Rgb::new(137, 180, 250)); // blue
        p.insert(ColorSlot::Base0E, Rgb::new(203, 166, 247)); // purple
        p.insert(ColorSlot::Base0F, Rgb::new(242, 205, 205)); // brown
        p
    }

    #[test]
    fn test_category_laws() {
        pr4xis::category::validate::check_category_laws::<ThemingCategory>().unwrap();
    }

    #[test]
    fn test_slot_role_quality() {
        let role = SlotRole;
        assert_eq!(role.get(&ColorSlot::Base00), Some(SemanticRole::Background));
        assert_eq!(role.get(&ColorSlot::Base05), Some(SemanticRole::Foreground));
        assert_eq!(role.get(&ColorSlot::Base08), Some(SemanticRole::Accent));
    }

    #[test]
    fn test_ansi_quality() {
        let ansi = AnsiIndex;
        assert_eq!(ansi.get(&ColorSlot::Base00), Some(0));
        assert_eq!(ansi.get(&ColorSlot::Base08), Some(1));
        assert_eq!(ansi.get(&ColorSlot::Base05), Some(7));
        // 16 slots have ANSI indices
        assert_eq!(ansi.individuals_with().len(), 16);
    }

    #[test]
    fn test_luminance_monotonicity() {
        let palette = dark_palette();
        assert!(LuminanceMonotonicity { palette }.holds());
    }

    #[test]
    fn test_wcag_foreground_contrast() {
        let palette = dark_palette();
        assert!(WcagForegroundContrast { palette }.holds());
    }

    #[test]
    fn test_detect_polarity_dark() {
        let palette = dark_palette();
        assert_eq!(detect_polarity(&palette), Some(Polarity::Dark));
    }

    #[test]
    fn test_detect_polarity_light() {
        let mut palette = dark_palette();
        palette.insert(ColorSlot::Base00, Rgb::new(239, 241, 245)); // light bg
        assert_eq!(detect_polarity(&palette), Some(Polarity::Light));
    }

    #[test]
    fn test_ramp_position_quality() {
        let ramp = RampPosition;
        assert_eq!(ramp.get(&ColorSlot::Base00), Some(0));
        assert_eq!(ramp.get(&ColorSlot::Base07), Some(7));
        assert_eq!(ramp.get(&ColorSlot::Base08), None); // accent, not ramp
        assert_eq!(ramp.individuals_with().len(), 8);
    }
}
