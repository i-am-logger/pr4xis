/// Color scheme types and their semantic mappings.
///
/// Sources:
/// - Base16: https://github.com/tinted-theming/home/blob/main/styling.md
/// - Base24: https://github.com/tinted-theming/base24/blob/main/styling.md
/// - ECMA-48 (5th Ed, 1991): SGR parameters 30-37, 90-97
/// - Vogix16: vogix design system semantic color mapping
///
/// The scheme taxonomy: Base16 is the foundation.
/// Base24 extends Base16 with 8 additional slots.
/// Vogix16 adds semantic names to Base16 slots.
/// Ansi16 maps to terminal SGR codes (ECMA-48).
use super::base16::ColorSlot;
use pr4xis::category::Entity;
use pr4xis::ontology::Axiom;

/// Color scheme type — each defines a different mapping/naming convention
/// over the same underlying color slots.
///
/// Source: tinted-theming ecosystem + vogix design system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchemeType {
    /// 16 colors: base00-base0F (monotone ramp + accents)
    Base16,
    /// 24 colors: base16 + base10-base11 (dark bg) + base12-base17 (bright accents)
    Base24,
    /// 16 colors with semantic names: danger, success, warning, etc.
    Vogix16,
    /// 16 ANSI terminal colors: color00-color15 (SGR 30-37, 90-97)
    Ansi16,
}

impl Entity for SchemeType {
    fn variants() -> Vec<Self> {
        vec![Self::Base16, Self::Base24, Self::Vogix16, Self::Ansi16]
    }
}

impl SchemeType {
    /// Number of color slots this scheme defines.
    pub fn slot_count(&self) -> usize {
        match self {
            Self::Base16 => 16,
            Self::Base24 => 24,
            Self::Vogix16 => 16,
            Self::Ansi16 => 16,
        }
    }

    /// Does this scheme extend Base16?
    pub fn extends_base16(&self) -> bool {
        matches!(self, Self::Base24 | Self::Vogix16)
    }

    /// Return the color slots used by this scheme type.
    ///
    /// Base16/Vogix16/Ansi16: only the 16 base16 slots (base00-base0F)
    /// Base24: all 24 slots (base00-base17)
    pub fn slots(&self) -> Vec<ColorSlot> {
        match self {
            Self::Base24 => ColorSlot::variants(),
            _ => ColorSlot::variants()
                .into_iter()
                .filter(|s| s.is_base16())
                .collect(),
        }
    }
}

// ── Vogix16 semantic mapping ──

/// Vogix16 semantic color name.
///
/// Source: vogix design system (vogix16-themes)
/// Maps semantic concepts to base16 accent slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Vogix16Semantic {
    // Monochromatic (base00-07)
    Background,
    BackgroundSurface,
    BackgroundSelection,
    ForegroundComment,
    ForegroundBorder,
    ForegroundText,
    ForegroundHeading,
    ForegroundBright,
    // Functional (base08-0F)
    Success,
    Warning,
    Notice,
    Danger,
    Active,
    Link,
    Highlight,
    Special,
}

impl Entity for Vogix16Semantic {
    fn variants() -> Vec<Self> {
        vec![
            Self::Background,
            Self::BackgroundSurface,
            Self::BackgroundSelection,
            Self::ForegroundComment,
            Self::ForegroundBorder,
            Self::ForegroundText,
            Self::ForegroundHeading,
            Self::ForegroundBright,
            Self::Success,
            Self::Warning,
            Self::Notice,
            Self::Danger,
            Self::Active,
            Self::Link,
            Self::Highlight,
            Self::Special,
        ]
    }
}

impl Vogix16Semantic {
    /// Map semantic name to base16 slot.
    ///
    /// Source: vogix16 design system specification
    pub fn to_slot(&self) -> ColorSlot {
        match self {
            Self::Background => ColorSlot::Base00,
            Self::BackgroundSurface => ColorSlot::Base01,
            Self::BackgroundSelection => ColorSlot::Base02,
            Self::ForegroundComment => ColorSlot::Base03,
            Self::ForegroundBorder => ColorSlot::Base04,
            Self::ForegroundText => ColorSlot::Base05,
            Self::ForegroundHeading => ColorSlot::Base06,
            Self::ForegroundBright => ColorSlot::Base07,
            Self::Success => ColorSlot::Base08,
            Self::Warning => ColorSlot::Base09,
            Self::Notice => ColorSlot::Base0A,
            Self::Danger => ColorSlot::Base0B,
            Self::Active => ColorSlot::Base0C,
            Self::Link => ColorSlot::Base0D,
            Self::Highlight => ColorSlot::Base0E,
            Self::Special => ColorSlot::Base0F,
        }
    }

    /// The theme file key name for this semantic color.
    pub fn key(&self) -> &'static str {
        match self {
            Self::Background => "background",
            Self::BackgroundSurface => "background-surface",
            Self::BackgroundSelection => "background-selection",
            Self::ForegroundComment => "foreground-comment",
            Self::ForegroundBorder => "foreground-border",
            Self::ForegroundText => "foreground-text",
            Self::ForegroundHeading => "foreground-heading",
            Self::ForegroundBright => "foreground-bright",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Notice => "notice",
            Self::Danger => "danger",
            Self::Active => "active",
            Self::Link => "link",
            Self::Highlight => "highlight",
            Self::Special => "special",
        }
    }

    /// Is this a functional (accent) color?
    pub fn is_functional(&self) -> bool {
        matches!(
            self,
            Self::Success
                | Self::Warning
                | Self::Notice
                | Self::Danger
                | Self::Active
                | Self::Link
                | Self::Highlight
                | Self::Special
        )
    }
}

// ── ANSI16 terminal colors ──

/// ANSI 16-color terminal slot.
///
/// Source: ECMA-48 Section 8.3.117 (SGR)
/// Normal colors: SGR 30-37 (fg), 40-47 (bg)
/// Bright colors: SGR 90-97 (fg), 100-107 (bg)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ansi16Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl Entity for Ansi16Color {
    fn variants() -> Vec<Self> {
        vec![
            Self::Black,
            Self::Red,
            Self::Green,
            Self::Yellow,
            Self::Blue,
            Self::Magenta,
            Self::Cyan,
            Self::White,
            Self::BrightBlack,
            Self::BrightRed,
            Self::BrightGreen,
            Self::BrightYellow,
            Self::BrightBlue,
            Self::BrightMagenta,
            Self::BrightCyan,
            Self::BrightWhite,
        ]
    }
}

impl Ansi16Color {
    /// ANSI color index (0-15).
    pub fn index(&self) -> u8 {
        match self {
            Self::Black => 0,
            Self::Red => 1,
            Self::Green => 2,
            Self::Yellow => 3,
            Self::Blue => 4,
            Self::Magenta => 5,
            Self::Cyan => 6,
            Self::White => 7,
            Self::BrightBlack => 8,
            Self::BrightRed => 9,
            Self::BrightGreen => 10,
            Self::BrightYellow => 11,
            Self::BrightBlue => 12,
            Self::BrightMagenta => 13,
            Self::BrightCyan => 14,
            Self::BrightWhite => 15,
        }
    }

    /// SGR foreground parameter.
    ///
    /// Source: ECMA-48 Section 8.3.117
    pub fn sgr_fg(&self) -> u8 {
        if self.index() < 8 {
            30 + self.index()
        } else {
            82 + self.index()
        }
    }

    /// SGR background parameter.
    pub fn sgr_bg(&self) -> u8 {
        if self.index() < 8 {
            40 + self.index()
        } else {
            92 + self.index()
        }
    }

    /// Theme file key name (color00, color01, ..., color15).
    pub fn key(&self) -> String {
        format!("color{:02}", self.index())
    }

    /// Is this a bright variant?
    pub fn is_bright(&self) -> bool {
        self.index() >= 8
    }

    /// Map to base16 slot via tinted-theming convention.
    ///
    /// Source: tinted-theming shell template
    pub fn to_base16_slot(&self) -> ColorSlot {
        match self {
            Self::Black => ColorSlot::Base00,
            Self::Red => ColorSlot::Base08,
            Self::Green => ColorSlot::Base0B,
            Self::Yellow => ColorSlot::Base0A,
            Self::Blue => ColorSlot::Base0D,
            Self::Magenta => ColorSlot::Base0E,
            Self::Cyan => ColorSlot::Base0C,
            Self::White => ColorSlot::Base05,
            Self::BrightBlack => ColorSlot::Base03,
            Self::BrightRed => ColorSlot::Base12,
            Self::BrightGreen => ColorSlot::Base14,
            Self::BrightYellow => ColorSlot::Base13,
            Self::BrightBlue => ColorSlot::Base16,
            Self::BrightMagenta => ColorSlot::Base17,
            Self::BrightCyan => ColorSlot::Base15,
            Self::BrightWhite => ColorSlot::Base07,
        }
    }
}

// ── Axioms ──

/// All vogix16 semantics map to distinct base16 slots (bijection on 16 slots).
pub struct Vogix16Bijection;

impl Axiom for Vogix16Bijection {
    fn description(&self) -> &str {
        "vogix16 semantic names map bijectively to base16 slots"
    }
    fn holds(&self) -> bool {
        let slots: Vec<ColorSlot> = Vogix16Semantic::variants()
            .iter()
            .map(|s| s.to_slot())
            .collect();
        // All 16 slots, all distinct
        slots.len() == 16 && {
            let mut deduped = slots.clone();
            deduped.sort_by_key(|s| format!("{:?}", s));
            deduped.dedup();
            deduped.len() == 16
        }
    }
}
pr4xis::register_axiom!(Vogix16Bijection);

/// All ANSI colors map to distinct base16 slots (bijection on 16 slots).
pub struct Ansi16Bijection;

impl Axiom for Ansi16Bijection {
    fn description(&self) -> &str {
        "ANSI 16 colors map bijectively to base16/base24 slots"
    }
    fn holds(&self) -> bool {
        let slots: Vec<ColorSlot> = Ansi16Color::variants()
            .iter()
            .map(|c| c.to_base16_slot())
            .collect();
        slots.len() == 16 && {
            let mut deduped = slots.clone();
            deduped.sort_by_key(|s| format!("{:?}", s));
            deduped.dedup();
            deduped.len() == 16
        }
    }
}
pr4xis::register_axiom!(Ansi16Bijection);

/// ANSI-to-base16 mapping is consistent with ColorSlot::ansi_index().
/// Every ColorSlot that has an ansi_index maps to the same ANSI color
/// that maps back to that slot.
pub struct AnsiBase16Consistency;

impl Axiom for AnsiBase16Consistency {
    fn description(&self) -> &str {
        "ANSI↔base16 mapping is consistent (round-trip)"
    }
    fn holds(&self) -> bool {
        // For each ANSI color, to_base16_slot().ansi_index() == self.index()
        Ansi16Color::variants().iter().all(|ansi| {
            let slot = ansi.to_base16_slot();
            slot.ansi_index() == Some(ansi.index())
        })
    }
}
pr4xis::register_axiom!(AnsiBase16Consistency);

/// SGR codes follow ECMA-48: fg = 30-37 (normal), 90-97 (bright).
pub struct SgrRanges;

impl Axiom for SgrRanges {
    fn description(&self) -> &str {
        "SGR foreground codes in [30-37] ∪ [90-97] per ECMA-48"
    }
    fn holds(&self) -> bool {
        Ansi16Color::variants().iter().all(|c| {
            let fg = c.sgr_fg();
            (30..=37).contains(&fg) || (90..=97).contains(&fg)
        })
    }
}
pr4xis::register_axiom!(SgrRanges);

#[cfg(test)]
mod tests {
    use super::*;

    // ── SchemeType ──

    #[test]
    fn test_4_scheme_types() {
        assert_eq!(SchemeType::variants().len(), 4);
    }

    #[test]
    fn test_slot_counts() {
        assert_eq!(SchemeType::Base16.slot_count(), 16);
        assert_eq!(SchemeType::Base24.slot_count(), 24);
        assert_eq!(SchemeType::Vogix16.slot_count(), 16);
        assert_eq!(SchemeType::Ansi16.slot_count(), 16);
    }

    #[test]
    fn test_base16_has_16_slots() {
        assert_eq!(SchemeType::Base16.slots().len(), 16);
    }

    #[test]
    fn test_base24_has_24_slots() {
        assert_eq!(SchemeType::Base24.slots().len(), 24);
    }

    #[test]
    fn test_vogix16_has_16_slots() {
        assert_eq!(SchemeType::Vogix16.slots().len(), 16);
    }

    #[test]
    fn test_ansi16_has_16_slots() {
        assert_eq!(SchemeType::Ansi16.slots().len(), 16);
    }

    #[test]
    fn test_base16_slots_are_subset_of_base24() {
        let base16 = SchemeType::Base16.slots();
        let base24 = SchemeType::Base24.slots();
        for slot in &base16 {
            assert!(base24.contains(slot), "{:?} missing from base24", slot);
        }
    }

    #[test]
    fn test_base24_extends_base16() {
        assert!(SchemeType::Base24.extends_base16());
        assert!(SchemeType::Vogix16.extends_base16());
        assert!(!SchemeType::Ansi16.extends_base16());
        assert!(!SchemeType::Base16.extends_base16());
    }

    // ── Vogix16 ──

    #[test]
    fn test_16_vogix16_semantics() {
        assert_eq!(Vogix16Semantic::variants().len(), 16);
    }

    #[test]
    fn test_vogix16_bijection() {
        assert!(Vogix16Bijection.holds());
    }

    #[test]
    fn test_vogix16_functional_count() {
        let functional: Vec<_> = Vogix16Semantic::variants()
            .into_iter()
            .filter(|s| s.is_functional())
            .collect();
        assert_eq!(functional.len(), 8);
    }

    #[test]
    fn test_vogix16_keys() {
        assert_eq!(Vogix16Semantic::Success.key(), "success");
        assert_eq!(Vogix16Semantic::Danger.key(), "danger");
        assert_eq!(Vogix16Semantic::Background.key(), "background");
        assert_eq!(Vogix16Semantic::ForegroundText.key(), "foreground-text");
    }

    #[test]
    fn test_vogix16_mapping() {
        assert_eq!(Vogix16Semantic::Success.to_slot(), ColorSlot::Base08);
        assert_eq!(Vogix16Semantic::Danger.to_slot(), ColorSlot::Base0B);
        assert_eq!(Vogix16Semantic::Background.to_slot(), ColorSlot::Base00);
    }

    // ── ANSI16 ──

    #[test]
    fn test_16_ansi_colors() {
        assert_eq!(Ansi16Color::variants().len(), 16);
    }

    #[test]
    fn test_ansi_indices_0_to_15() {
        let indices: Vec<u8> = Ansi16Color::variants().iter().map(|c| c.index()).collect();
        for i in 0..16 {
            assert!(indices.contains(&i), "missing ANSI index {}", i);
        }
    }

    #[test]
    fn test_ansi16_bijection() {
        assert!(Ansi16Bijection.holds());
    }

    #[test]
    fn test_ansi_base16_consistency() {
        assert!(AnsiBase16Consistency.holds());
    }

    #[test]
    fn test_sgr_ranges() {
        assert!(SgrRanges.holds());
    }

    #[test]
    fn test_ansi_keys() {
        assert_eq!(Ansi16Color::Black.key(), "color00");
        assert_eq!(Ansi16Color::Red.key(), "color01");
        assert_eq!(Ansi16Color::BrightWhite.key(), "color15");
    }

    #[test]
    fn test_ansi_sgr_values() {
        assert_eq!(Ansi16Color::Black.sgr_fg(), 30);
        assert_eq!(Ansi16Color::White.sgr_fg(), 37);
        assert_eq!(Ansi16Color::BrightBlack.sgr_fg(), 90);
        assert_eq!(Ansi16Color::BrightWhite.sgr_fg(), 97);
    }

    #[test]
    fn test_bright_partition() {
        let normal: Vec<_> = Ansi16Color::variants()
            .into_iter()
            .filter(|c| !c.is_bright())
            .collect();
        let bright: Vec<_> = Ansi16Color::variants()
            .into_iter()
            .filter(|c| c.is_bright())
            .collect();
        assert_eq!(normal.len(), 8);
        assert_eq!(bright.len(), 8);
    }

    // ── Property-based tests ──
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_ansi_index_bounded(idx in 0u8..16) {
            let color = &Ansi16Color::variants()[idx as usize];
            prop_assert!(color.index() < 16);
        }

        #[test]
        fn prop_sgr_fg_in_valid_range(idx in 0u8..16) {
            let color = &Ansi16Color::variants()[idx as usize];
            let fg = color.sgr_fg();
            prop_assert!((30..=37).contains(&fg) || (90..=97).contains(&fg));
        }

        #[test]
        fn prop_sgr_bg_in_valid_range(idx in 0u8..16) {
            let color = &Ansi16Color::variants()[idx as usize];
            let bg = color.sgr_bg();
            prop_assert!((40..=47).contains(&bg) || (100..=107).contains(&bg));
        }

        #[test]
        fn prop_sgr_bg_fg_offset_10(idx in 0u8..16) {
            // BG is always FG + 10 per ECMA-48
            let color = &Ansi16Color::variants()[idx as usize];
            prop_assert_eq!(color.sgr_bg(), color.sgr_fg() + 10);
        }

        #[test]
        fn prop_vogix16_to_slot_preserves_role(idx in 0usize..16) {
            use super::super::base16::SemanticRole;
            let semantic = &Vogix16Semantic::variants()[idx];
            let slot = semantic.to_slot();
            // Monochromatic semantics map to background/foreground slots
            // Functional semantics map to accent slots
            if semantic.is_functional() {
                prop_assert_eq!(slot.role(), SemanticRole::Accent);
            } else {
                prop_assert!(matches!(slot.role(), SemanticRole::Background | SemanticRole::Foreground));
            }
        }

        #[test]
        fn prop_ansi_base16_roundtrip(idx in 0u8..16) {
            // ansi → base16 slot → ansi index == original index
            let color = &Ansi16Color::variants()[idx as usize];
            let slot = color.to_base16_slot();
            prop_assert_eq!(slot.ansi_index(), Some(color.index()));
        }
    }
}
