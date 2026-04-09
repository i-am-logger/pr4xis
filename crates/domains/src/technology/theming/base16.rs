/// Base16 / Base24 color scheme slots.
///
/// Source: tinted-theming/home styling.md
///   base00-base07: monotone ramp (background → foreground)
///   base08-base0F: chromatic accents (red → brown)
///
/// Source: tinted-theming/base24 styling.md
///   base10-base11: darker backgrounds (panels, sidebars)
///   base12-base17: bright accent variants
///
/// Source: ECMA-48 Section 8.3.117 (SGR)
///   ANSI colors 0-7 (normal), 8-15 (bright)
use praxis::category::Entity;

/// A named slot in a base16/base24 color scheme.
///
/// 24 total: 00-0F (base16) + 10-17 (base24 extension).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorSlot {
    // ── Base16: monotone ramp (background → foreground) ──
    /// Default background
    Base00,
    /// Lighter background (status bars, line highlight)
    Base01,
    /// Selection background
    Base02,
    /// Comments, invisibles, line highlighting
    Base03,
    /// Dark foreground (status bars)
    Base04,
    /// Default foreground, caret, delimiters
    Base05,
    /// Light foreground (rarely used)
    Base06,
    /// Light background (rarely used)
    Base07,

    // ── Base16: chromatic accents ──
    /// Variables, XML tags, diff deleted (red-ish)
    Base08,
    /// Integers, booleans, constants (orange-ish)
    Base09,
    /// Classes, markup bold, search text (yellow-ish)
    Base0A,
    /// Strings, diff inserted (green-ish)
    Base0B,
    /// Support, regex, escape characters (cyan-ish)
    Base0C,
    /// Functions, methods, headings (blue-ish)
    Base0D,
    /// Keywords, storage, selector (purple-ish)
    Base0E,
    /// Deprecated, embedded tags (brown-ish)
    Base0F,

    // ── Base24 extension: darker backgrounds ──
    /// Darker background (sidebars, panels)
    Base10,
    /// Darkest background (contrast borders, dividers)
    Base11,

    // ── Base24 extension: bright accent variants ──
    /// Bright red (variant of base08)
    Base12,
    /// Bright yellow (variant of base0A)
    Base13,
    /// Bright green (variant of base0B)
    Base14,
    /// Bright cyan (variant of base0C)
    Base15,
    /// Bright blue (variant of base0D)
    Base16,
    /// Bright purple (variant of base0E)
    Base17,
}

impl Entity for ColorSlot {
    fn variants() -> Vec<Self> {
        vec![
            Self::Base00,
            Self::Base01,
            Self::Base02,
            Self::Base03,
            Self::Base04,
            Self::Base05,
            Self::Base06,
            Self::Base07,
            Self::Base08,
            Self::Base09,
            Self::Base0A,
            Self::Base0B,
            Self::Base0C,
            Self::Base0D,
            Self::Base0E,
            Self::Base0F,
            Self::Base10,
            Self::Base11,
            Self::Base12,
            Self::Base13,
            Self::Base14,
            Self::Base15,
            Self::Base16,
            Self::Base17,
        ]
    }
}

/// Semantic role of a color slot.
///
/// Source: tinted-theming styling.md
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticRole {
    Background,
    Foreground,
    Accent,
    DarkBackground,
    BrightAccent,
}

impl Entity for SemanticRole {
    fn variants() -> Vec<Self> {
        vec![
            Self::Background,
            Self::Foreground,
            Self::Accent,
            Self::DarkBackground,
            Self::BrightAccent,
        ]
    }
}

impl ColorSlot {
    /// Canonical key name as used in theme files (base00, base0A, etc).
    ///
    /// Source: tinted-theming convention — hex digits uppercase.
    pub fn key(&self) -> &'static str {
        match self {
            Self::Base00 => "base00", Self::Base01 => "base01",
            Self::Base02 => "base02", Self::Base03 => "base03",
            Self::Base04 => "base04", Self::Base05 => "base05",
            Self::Base06 => "base06", Self::Base07 => "base07",
            Self::Base08 => "base08", Self::Base09 => "base09",
            Self::Base0A => "base0A", Self::Base0B => "base0B",
            Self::Base0C => "base0C", Self::Base0D => "base0D",
            Self::Base0E => "base0E", Self::Base0F => "base0F",
            Self::Base10 => "base10", Self::Base11 => "base11",
            Self::Base12 => "base12", Self::Base13 => "base13",
            Self::Base14 => "base14", Self::Base15 => "base15",
            Self::Base16 => "base16", Self::Base17 => "base17",
        }
    }

    /// The semantic role of this slot per the base16/base24 spec.
    pub fn role(&self) -> SemanticRole {
        match self {
            Self::Base00 | Self::Base01 | Self::Base02 | Self::Base03 => SemanticRole::Background,
            Self::Base04 | Self::Base05 | Self::Base06 | Self::Base07 => SemanticRole::Foreground,
            Self::Base08
            | Self::Base09
            | Self::Base0A
            | Self::Base0B
            | Self::Base0C
            | Self::Base0D
            | Self::Base0E
            | Self::Base0F => SemanticRole::Accent,
            Self::Base10 | Self::Base11 => SemanticRole::DarkBackground,
            Self::Base12
            | Self::Base13
            | Self::Base14
            | Self::Base15
            | Self::Base16
            | Self::Base17 => SemanticRole::BrightAccent,
        }
    }

    /// Is this a base16 slot (00-0F)?
    pub fn is_base16(&self) -> bool {
        !matches!(
            self,
            Self::Base10
                | Self::Base11
                | Self::Base12
                | Self::Base13
                | Self::Base14
                | Self::Base15
                | Self::Base16
                | Self::Base17
        )
    }

    /// The conventional ANSI terminal color index (0-15) for this slot.
    ///
    /// Source: tinted-theming shell template convention
    ///   ANSI 0=base00, 1=base08, 2=base0B, 3=base0A,
    ///   4=base0D, 5=base0E, 6=base0C, 7=base05,
    ///   8=base03, 9=base12, 10=base14, 11=base13,
    ///   12=base16, 13=base17, 14=base15, 15=base07
    pub fn ansi_index(&self) -> Option<u8> {
        match self {
            Self::Base00 => Some(0),  // black
            Self::Base08 => Some(1),  // red
            Self::Base0B => Some(2),  // green
            Self::Base0A => Some(3),  // yellow
            Self::Base0D => Some(4),  // blue
            Self::Base0E => Some(5),  // magenta
            Self::Base0C => Some(6),  // cyan
            Self::Base05 => Some(7),  // white
            Self::Base03 => Some(8),  // bright black
            Self::Base12 => Some(9),  // bright red
            Self::Base14 => Some(10), // bright green
            Self::Base13 => Some(11), // bright yellow
            Self::Base16 => Some(12), // bright blue
            Self::Base17 => Some(13), // bright magenta
            Self::Base15 => Some(14), // bright cyan
            Self::Base07 => Some(15), // bright white
            _ => None,
        }
    }

    /// Position in the monotone luminance ramp (base00-base07).
    /// Returns None for accent and extension slots.
    pub fn ramp_position(&self) -> Option<u8> {
        match self {
            Self::Base00 => Some(0),
            Self::Base01 => Some(1),
            Self::Base02 => Some(2),
            Self::Base03 => Some(3),
            Self::Base04 => Some(4),
            Self::Base05 => Some(5),
            Self::Base06 => Some(6),
            Self::Base07 => Some(7),
            _ => None,
        }
    }

    /// The base16 accent slot that this base24 bright variant extends.
    /// Returns None for non-bright slots.
    pub fn bright_variant_of(&self) -> Option<ColorSlot> {
        match self {
            Self::Base12 => Some(Self::Base08), // bright red ← red
            Self::Base13 => Some(Self::Base0A), // bright yellow ← yellow
            Self::Base14 => Some(Self::Base0B), // bright green ← green
            Self::Base15 => Some(Self::Base0C), // bright cyan ← cyan
            Self::Base16 => Some(Self::Base0D), // bright blue ← blue
            Self::Base17 => Some(Self::Base0E), // bright purple ← purple
            _ => None,
        }
    }
}

/// Color scheme polarity — derived from the background luminance.
///
/// Source: base16 convention — base00 is always the "default background".
/// A dark theme has low-luminance base00, a light theme has high-luminance base00.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Polarity {
    Dark,
    Light,
}

impl Entity for Polarity {
    fn variants() -> Vec<Self> {
        vec![Self::Dark, Self::Light]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_24_slots() {
        assert_eq!(ColorSlot::variants().len(), 24);
    }

    #[test]
    fn test_16_base16_slots() {
        let base16_count = ColorSlot::variants()
            .iter()
            .filter(|s| s.is_base16())
            .count();
        assert_eq!(base16_count, 16);
    }

    #[test]
    fn test_8_base24_extension_slots() {
        let ext_count = ColorSlot::variants()
            .iter()
            .filter(|s| !s.is_base16())
            .count();
        assert_eq!(ext_count, 8);
    }

    #[test]
    fn test_roles_cover_all_slots() {
        for slot in ColorSlot::variants() {
            let _ = slot.role(); // should not panic
        }
    }

    #[test]
    fn test_ansi_mapping_16_slots() {
        let mapped: Vec<_> = ColorSlot::variants()
            .iter()
            .filter_map(|s| s.ansi_index())
            .collect();
        assert_eq!(mapped.len(), 16);
        // All indices 0-15 present
        for i in 0..16 {
            assert!(mapped.contains(&i), "ANSI index {} not mapped", i);
        }
    }

    #[test]
    fn test_ramp_positions() {
        let ramp: Vec<_> = ColorSlot::variants()
            .iter()
            .filter_map(|s| s.ramp_position().map(|p| (*s, p)))
            .collect();
        assert_eq!(ramp.len(), 8);
        // Positions 0-7
        for i in 0..8 {
            assert!(ramp.iter().any(|(_, p)| *p == i));
        }
    }

    #[test]
    fn test_bright_variants() {
        let brights: Vec<_> = ColorSlot::variants()
            .iter()
            .filter_map(|s| s.bright_variant_of().map(|base| (*s, base)))
            .collect();
        assert_eq!(brights.len(), 6); // 6 bright variants (no bright orange/brown)
    }

    #[test]
    fn test_polarity_variants() {
        assert_eq!(Polarity::variants().len(), 2);
    }

    #[test]
    fn test_semantic_roles() {
        assert_eq!(SemanticRole::variants().len(), 5);
        assert_eq!(ColorSlot::Base00.role(), SemanticRole::Background);
        assert_eq!(ColorSlot::Base05.role(), SemanticRole::Foreground);
        assert_eq!(ColorSlot::Base08.role(), SemanticRole::Accent);
        assert_eq!(ColorSlot::Base10.role(), SemanticRole::DarkBackground);
        assert_eq!(ColorSlot::Base12.role(), SemanticRole::BrightAccent);
    }
}
