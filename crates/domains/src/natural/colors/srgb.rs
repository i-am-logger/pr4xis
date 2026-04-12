/// sRGB color science — linearization, luminance, contrast.
///
/// Built on math::functions primitives (Piecewise, LinearCombination, OffsetRatio).
///
/// Sources:
/// - IEC 61966-2-1 (sRGB standard): transfer function
/// - ITU-R BT.709-6: luma coefficients (0.2126, 0.7152, 0.0722)
/// - W3C WCAG 2.1: relative luminance, contrast ratio, compliance levels
use super::rgb::Rgb;
use crate::formal::math::functions::{Interval, LinearCombination, OffsetRatio, Piecewise};
use pr4xis::ontology::Axiom;

/// sRGB electro-optical transfer function (EOTF).
/// Converts gamma-encoded sRGB [0,1] to linear light [0,1].
///
/// Source: IEC 61966-2-1, Section 5.2
///   if C_srgb <= 0.04045: C_lin = C_srgb / 12.92
///   else: C_lin = ((C_srgb + 0.055) / 1.055) ^ 2.4
pub fn srgb_linearize() -> Piecewise {
    Piecewise {
        threshold: 0.04045,
        below: |c| c / 12.92,
        above: |c| ((c + 0.055) / 1.055).powf(2.4),
    }
}

/// BT.709 luminance coefficients as a linear combination.
///
/// Source: ITU-R BT.709-6
///   Y = 0.2126 R + 0.7152 G + 0.0722 B
///
/// These derive from the CIE 1931 chromaticity coordinates of the
/// Rec. 709 primaries (R: 0.64/0.33, G: 0.30/0.60, B: 0.15/0.06)
/// relative to illuminant D65 (0.3127/0.3290).
pub fn bt709_luminance() -> LinearCombination {
    LinearCombination::new(vec![0.2126, 0.7152, 0.0722])
}

/// WCAG 2.1 contrast ratio formula.
///
/// Source: W3C WCAG 2.1 "contrast ratio" definition
///   CR = (L_lighter + 0.05) / (L_darker + 0.05)
///
/// The 0.05 offset accounts for ambient light (viewing flare factor).
pub fn wcag_contrast() -> OffsetRatio {
    OffsetRatio { offset: 0.05 }
}

/// WCAG compliance levels.
///
/// Source: WCAG 2.1 SC 1.4.3 (AA) and SC 1.4.6 (AAA)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WcagLevel {
    /// SC 1.4.3: contrast >= 4.5 for normal text, >= 3.0 for large text
    AA,
    /// SC 1.4.6: contrast >= 7.0 for normal text, >= 4.5 for large text
    AAA,
}

impl WcagLevel {
    pub fn min_contrast_normal(&self) -> f64 {
        match self {
            WcagLevel::AA => 4.5,
            WcagLevel::AAA => 7.0,
        }
    }

    pub fn min_contrast_large(&self) -> f64 {
        match self {
            WcagLevel::AA => 3.0,
            WcagLevel::AAA => 4.5,
        }
    }
}

/// Compute the relative luminance of an Rgb color per WCAG 2.1.
///
/// Applies sRGB linearization to each channel, then BT.709 weighted sum.
pub fn relative_luminance(color: &Rgb) -> f64 {
    let linearize = srgb_linearize();
    let luma = bt709_luminance();

    let r_lin = linearize.eval(color.r as f64 / 255.0);
    let g_lin = linearize.eval(color.g as f64 / 255.0);
    let b_lin = linearize.eval(color.b as f64 / 255.0);

    luma.eval(&[r_lin, g_lin, b_lin])
}

/// Compute WCAG contrast ratio between two colors.
pub fn contrast_ratio(a: &Rgb, b: &Rgb) -> f64 {
    let la = relative_luminance(a);
    let lb = relative_luminance(b);
    wcag_contrast().eval(la, lb)
}

/// Check WCAG compliance between foreground and background.
pub fn wcag_compliant(fg: &Rgb, bg: &Rgb, level: WcagLevel) -> bool {
    contrast_ratio(fg, bg) >= level.min_contrast_normal()
}

/// Is this a dark color? (relative luminance < 0.5)
pub fn is_dark(color: &Rgb) -> bool {
    relative_luminance(color) < 0.5
}

// ── Axioms ──

/// sRGB linearization is continuous at threshold 0.04045.
///
/// The piecewise segments must agree: 0.04045/12.92 = ((0.04045+0.055)/1.055)^2.4
/// Source: IEC 61966-2-1 specifies this threshold precisely for continuity.
pub struct SrgbContinuity;

impl Axiom for SrgbContinuity {
    fn description(&self) -> &str {
        "sRGB EOTF is continuous at threshold 0.04045 (IEC 61966-2-1)"
    }
    fn holds(&self) -> bool {
        srgb_linearize().is_continuous(1e-6)
    }
}

/// BT.709 luma coefficients are a convex combination (sum to 1.0, all non-negative).
///
/// Source: ITU-R BT.709-6 — luminance is a weighted average.
pub struct LumaConvex;

impl Axiom for LumaConvex {
    fn description(&self) -> &str {
        "BT.709 luma coefficients sum to 1.0 (ITU-R BT.709-6)"
    }
    fn holds(&self) -> bool {
        let lc = bt709_luminance();
        lc.is_convex() && lc.is_non_negative()
    }
}

/// Luminance is bounded: 0.0 for black, ~1.0 for white.
///
/// Source: follows from convexity of weights on inputs in [0,1].
pub struct LuminanceBounded;

impl Axiom for LuminanceBounded {
    fn description(&self) -> &str {
        "luminance in [0, 1] for valid sRGB colors"
    }
    fn holds(&self) -> bool {
        let black_l = relative_luminance(&Rgb::BLACK);
        let white_l = relative_luminance(&Rgb::WHITE);
        Interval::UNIT.contains(black_l)
            && Interval::UNIT.contains(white_l)
            && black_l < 0.01
            && white_l > 0.99
    }
}

/// WCAG contrast ratio is bounded: [1.0, 21.0].
///
/// Source: WCAG 2.1 — minimum is 1:1 (identical), maximum is 21:1 (black/white).
pub struct ContrastBounded;

impl Axiom for ContrastBounded {
    fn description(&self) -> &str {
        "WCAG contrast ratio in [1.0, 21.0]"
    }
    fn holds(&self) -> bool {
        let min = contrast_ratio(&Rgb::BLACK, &Rgb::BLACK);
        let max = contrast_ratio(&Rgb::WHITE, &Rgb::BLACK);
        (min - 1.0).abs() < 0.01 && (max - 21.0).abs() < 0.1
    }
}

/// Luminance monotonicity: brighter colors have higher luminance.
///
/// If R1 >= R2, G1 >= G2, B1 >= B2 then L1 >= L2.
pub struct LuminanceMonotone;

impl Axiom for LuminanceMonotone {
    fn description(&self) -> &str {
        "luminance is monotone: brighter channels → higher luminance"
    }
    fn holds(&self) -> bool {
        // Test: (128,128,128) has higher luminance than (64,64,64)
        let dark = Rgb::new(64, 64, 64);
        let mid = Rgb::new(128, 128, 128);
        let light = Rgb::new(192, 192, 192);
        relative_luminance(&dark) < relative_luminance(&mid)
            && relative_luminance(&mid) < relative_luminance(&light)
    }
}

/// Screen blend is dual of multiply: Screen(a,b) = 1 - Multiply(1-a, 1-b).
///
/// Source: W3C Compositing and Blending Level 1, Section 13.1
pub struct ScreenDualOfMultiply;

impl Axiom for ScreenDualOfMultiply {
    fn description(&self) -> &str {
        "screen blend is dual of multiply (W3C Compositing Level 1)"
    }
    fn holds(&self) -> bool {
        use super::mixing::{MixMode, mix};
        // Test with several color pairs
        let pairs = [
            (Rgb::new(100, 150, 200), Rgb::new(50, 100, 150)),
            (Rgb::RED, Rgb::BLUE),
            (Rgb::new(0, 0, 0), Rgb::new(255, 255, 255)),
        ];
        pairs.iter().all(|(a, b)| {
            let screen = mix(*a, *b, MixMode::Screen);
            // Manual: 1 - (1-a)(1-b) per channel
            let manual = Rgb::new(
                (255.0 - (255.0 - a.r as f64) * (255.0 - b.r as f64) / 255.0) as u8,
                (255.0 - (255.0 - a.g as f64) * (255.0 - b.g as f64) / 255.0) as u8,
                (255.0 - (255.0 - a.b as f64) * (255.0 - b.b as f64) / 255.0) as u8,
            );
            // Allow 1 unit rounding error per channel
            (screen.r as i16 - manual.r as i16).abs() <= 1
                && (screen.g as i16 - manual.g as i16).abs() <= 1
                && (screen.b as i16 - manual.b as i16).abs() <= 1
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srgb_linearize_identity_at_zero() {
        let f = srgb_linearize();
        assert!((f.eval(0.0) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_srgb_linearize_identity_at_one() {
        let f = srgb_linearize();
        assert!((f.eval(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_srgb_continuity() {
        assert!(SrgbContinuity.holds());
    }

    #[test]
    fn test_bt709_convex() {
        assert!(LumaConvex.holds());
    }

    #[test]
    fn test_luminance_black() {
        assert!(relative_luminance(&Rgb::BLACK) < 0.001);
    }

    #[test]
    fn test_luminance_white() {
        assert!(relative_luminance(&Rgb::WHITE) > 0.99);
    }

    #[test]
    fn test_luminance_bounded() {
        assert!(LuminanceBounded.holds());
    }

    #[test]
    fn test_contrast_same_color() {
        let ratio = contrast_ratio(&Rgb::new(128, 128, 128), &Rgb::new(128, 128, 128));
        assert!((ratio - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_contrast_black_white() {
        let ratio = contrast_ratio(&Rgb::WHITE, &Rgb::BLACK);
        assert!((ratio - 21.0).abs() < 0.1);
    }

    #[test]
    fn test_contrast_bounded() {
        assert!(ContrastBounded.holds());
    }

    #[test]
    fn test_luminance_monotone() {
        assert!(LuminanceMonotone.holds());
    }

    #[test]
    fn test_wcag_aa_white_on_black() {
        assert!(wcag_compliant(&Rgb::WHITE, &Rgb::BLACK, WcagLevel::AA));
        assert!(wcag_compliant(&Rgb::WHITE, &Rgb::BLACK, WcagLevel::AAA));
    }

    #[test]
    fn test_wcag_aa_fails_similar() {
        // Two similar grays should fail AA
        assert!(!wcag_compliant(
            &Rgb::new(128, 128, 128),
            &Rgb::new(140, 140, 140),
            WcagLevel::AA
        ));
    }

    #[test]
    fn test_is_dark() {
        assert!(is_dark(&Rgb::BLACK));
        assert!(is_dark(&Rgb::new(30, 30, 30)));
        assert!(!is_dark(&Rgb::WHITE));
        assert!(!is_dark(&Rgb::new(200, 200, 200)));
    }

    #[test]
    fn test_screen_dual_of_multiply() {
        assert!(ScreenDualOfMultiply.holds());
    }

    // ── Property-based tests ──
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_luminance_bounded(r in 0u8..=255, g in 0u8..=255, b in 0u8..=255) {
            let color = Rgb::new(r, g, b);
            let l = relative_luminance(&color);
            prop_assert!(l >= 0.0 && l <= 1.0, "luminance({:?}) = {} not in [0,1]", color, l);
        }

        #[test]
        fn prop_contrast_ratio_bounded(
            r1 in 0u8..=255, g1 in 0u8..=255, b1 in 0u8..=255,
            r2 in 0u8..=255, g2 in 0u8..=255, b2 in 0u8..=255,
        ) {
            let a = Rgb::new(r1, g1, b1);
            let b = Rgb::new(r2, g2, b2);
            let cr = contrast_ratio(&a, &b);
            prop_assert!(cr >= 1.0 && cr <= 21.1, "contrast({:?}, {:?}) = {}", a, b, cr);
        }

        #[test]
        fn prop_contrast_symmetric(
            r1 in 0u8..=255, g1 in 0u8..=255, b1 in 0u8..=255,
            r2 in 0u8..=255, g2 in 0u8..=255, b2 in 0u8..=255,
        ) {
            let a = Rgb::new(r1, g1, b1);
            let b = Rgb::new(r2, g2, b2);
            prop_assert!((contrast_ratio(&a, &b) - contrast_ratio(&b, &a)).abs() < 1e-10);
        }

        #[test]
        fn prop_contrast_identity(r in 0u8..=255, g in 0u8..=255, b in 0u8..=255) {
            let color = Rgb::new(r, g, b);
            let cr = contrast_ratio(&color, &color);
            prop_assert!((cr - 1.0).abs() < 0.01, "contrast with self should be 1.0, got {}", cr);
        }

        #[test]
        fn prop_luminance_monotone_gray(a in 0u8..=255, b in 0u8..=255) {
            // For grayscale: brighter channel → higher luminance
            let ca = Rgb::new(a, a, a);
            let cb = Rgb::new(b, b, b);
            if a <= b {
                prop_assert!(relative_luminance(&ca) <= relative_luminance(&cb) + 1e-10);
            }
        }

        #[test]
        fn prop_dark_light_partition(r in 0u8..=255, g in 0u8..=255, b in 0u8..=255) {
            // Every color is either dark or not dark (complete partition)
            let color = Rgb::new(r, g, b);
            let dark = is_dark(&color);
            let l = relative_luminance(&color);
            prop_assert_eq!(dark, l < 0.5);
        }
    }
}
