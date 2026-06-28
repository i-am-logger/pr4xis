#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

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
/// Converts gamma-encoded sRGB `[0,1]` to linear light `[0,1]`.
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
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if srgb_linearize().is_continuous(1e-6) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "SrgbContinuity",
        "sRGB EOTF is continuous at threshold 0.04045",
        "IEC 61966-2-1 (1999) sRGB standard"
    );
}
pr4xis::register_axiom!(SrgbContinuity, "IEC 61966-2-1 (1999) sRGB standard");

/// BT.709 luma coefficients are a convex combination (sum to 1.0, all non-negative).
///
/// Source: ITU-R BT.709-6 — luminance is a weighted average.
pub struct LumaConvex;

impl Axiom for LumaConvex {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let lc = bt709_luminance();
        if lc.is_convex() && lc.is_non_negative() {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "LumaConvex",
        "BT.709 luma coefficients sum to 1.0",
        "ITU-R BT.709-6 (2015) Parameter values for the HDTV standards"
    );
}
pr4xis::register_axiom!(
    LumaConvex,
    "ITU-R BT.709-6 (2015) Parameter values for the HDTV standards"
);

/// Luminance is bounded: 0.0 for black, ~1.0 for white.
///
/// Source: follows from convexity of weights on inputs in `[0,1]`.
pub struct LuminanceBounded;

impl Axiom for LuminanceBounded {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let black_l = relative_luminance(&Rgb::BLACK);
        let white_l = relative_luminance(&Rgb::WHITE);
        if Interval::UNIT.contains(black_l)
            && Interval::UNIT.contains(white_l)
            && black_l < 0.01
            && white_l > 0.99
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "LuminanceBounded",
        "luminance in [0, 1] for valid sRGB colors",
        "ITU-R BT.709-6 (2015); follows from convexity of weights on inputs in [0,1]"
    );
}
pr4xis::register_axiom!(LuminanceBounded, "ITU-R BT.709-6 (2015)");

/// WCAG contrast ratio is bounded: [1.0, 21.0].
///
/// Source: WCAG 2.1 — minimum is 1:1 (identical), maximum is 21:1 (black/white).
pub struct ContrastBounded;

impl Axiom for ContrastBounded {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let min = contrast_ratio(&Rgb::BLACK, &Rgb::BLACK);
        let max = contrast_ratio(&Rgb::WHITE, &Rgb::BLACK);
        if (min - 1.0).abs() < 0.01 && (max - 21.0).abs() < 0.1 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "ContrastBounded",
        "WCAG contrast ratio in [1.0, 21.0]",
        "W3C WCAG 2.1 (2018) Success Criterion 1.4.3"
    );
}
pr4xis::register_axiom!(
    ContrastBounded,
    "W3C WCAG 2.1 (2018) Success Criterion 1.4.3"
);

/// Luminance monotonicity: brighter colors have higher luminance.
///
/// If R1 >= R2, G1 >= G2, B1 >= B2 then L1 >= L2.
pub struct LuminanceMonotone;

impl Axiom for LuminanceMonotone {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let dark = Rgb::new(64, 64, 64);
        let mid = Rgb::new(128, 128, 128);
        let light = Rgb::new(192, 192, 192);
        if relative_luminance(&dark) < relative_luminance(&mid)
            && relative_luminance(&mid) < relative_luminance(&light)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "LuminanceMonotone",
        "luminance is monotone: brighter channels → higher luminance",
        "ITU-R BT.709-6 (2015) — follows from non-negativity of luminance coefficients"
    );
}
pr4xis::register_axiom!(LuminanceMonotone, "ITU-R BT.709-6 (2015)");

/// Screen blend is dual of multiply: Screen(a,b) = 1 - Multiply(1-a, 1-b).
///
/// Source: W3C Compositing and Blending Level 1, Section 13.1
pub struct ScreenDualOfMultiply;

impl Axiom for ScreenDualOfMultiply {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use super::mixing::{MixMode, mix};
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let pairs = [
            (Rgb::new(100, 150, 200), Rgb::new(50, 100, 150)),
            (Rgb::RED, Rgb::BLUE),
            (Rgb::new(0, 0, 0), Rgb::new(255, 255, 255)),
        ];
        let all_dual = pairs.iter().all(|(a, b)| {
            let screen = mix(*a, *b, MixMode::Screen);
            let manual = Rgb::new(
                (255.0 - (255.0 - a.r as f64) * (255.0 - b.r as f64) / 255.0) as u8,
                (255.0 - (255.0 - a.g as f64) * (255.0 - b.g as f64) / 255.0) as u8,
                (255.0 - (255.0 - a.b as f64) * (255.0 - b.b as f64) / 255.0) as u8,
            );
            (screen.r as i16 - manual.r as i16).abs() <= 1
                && (screen.g as i16 - manual.g as i16).abs() <= 1
                && (screen.b as i16 - manual.b as i16).abs() <= 1
        });
        if all_dual {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "ScreenDualOfMultiply",
        "screen blend is dual of multiply: Screen(a,b) = 1 - Multiply(1-a, 1-b)",
        "W3C Compositing and Blending Level 1 (2015) §13.1"
    );
}
pr4xis::register_axiom!(
    ScreenDualOfMultiply,
    "W3C Compositing and Blending Level 1 (2015) §13.1"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_srgb_linearize_identity_at_zero() {
        let f = srgb_linearize();
        assert!((f.eval(0.0) - 0.0).abs() < 1e-10);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_srgb_linearize_identity_at_one() {
        let f = srgb_linearize();
        assert!((f.eval(1.0) - 1.0).abs() < 1e-6);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_srgb_continuity() {
        assert!(SrgbContinuity.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_bt709_convex() {
        assert!(LumaConvex.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_luminance_black() {
        assert!(relative_luminance(&Rgb::BLACK) < 0.001);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_luminance_white() {
        assert!(relative_luminance(&Rgb::WHITE) > 0.99);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_luminance_bounded() {
        assert!(LuminanceBounded.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_contrast_same_color() {
        let ratio = contrast_ratio(&Rgb::new(128, 128, 128), &Rgb::new(128, 128, 128));
        assert!((ratio - 1.0).abs() < 0.01);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_contrast_black_white() {
        let ratio = contrast_ratio(&Rgb::WHITE, &Rgb::BLACK);
        assert!((ratio - 21.0).abs() < 0.1);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_contrast_bounded() {
        assert!(ContrastBounded.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_luminance_monotone() {
        assert!(LuminanceMonotone.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_wcag_aa_white_on_black() {
        assert!(wcag_compliant(&Rgb::WHITE, &Rgb::BLACK, WcagLevel::AA));
        assert!(wcag_compliant(&Rgb::WHITE, &Rgb::BLACK, WcagLevel::AAA));
    }

    #[pr4xis::praxis_value(Honest, Verifiable)]
    #[test]
    fn test_wcag_aa_fails_similar() {
        // Two similar grays should fail AA
        assert!(!wcag_compliant(
            &Rgb::new(128, 128, 128),
            &Rgb::new(140, 140, 140),
            WcagLevel::AA
        ));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_is_dark() {
        assert!(is_dark(&Rgb::BLACK));
        assert!(is_dark(&Rgb::new(30, 30, 30)));
        assert!(!is_dark(&Rgb::WHITE));
        assert!(!is_dark(&Rgb::new(200, 200, 200)));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_screen_dual_of_multiply() {
        assert!(ScreenDualOfMultiply.verify().is_ok());
    }

    // ── Property-based tests ──
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_luminance_bounded(r in 0u8..=255, g in 0u8..=255, b in 0u8..=255) {
            let color = Rgb::new(r, g, b);
            let l = relative_luminance(&color);
            prop_assert!((0.0..=1.0).contains(&l), "luminance({:?}) = {} not in [0,1]", color, l);
        }

        #[test]
        fn prop_contrast_ratio_bounded(
            r1 in 0u8..=255, g1 in 0u8..=255, b1 in 0u8..=255,
            r2 in 0u8..=255, g2 in 0u8..=255, b2 in 0u8..=255,
        ) {
            let a = Rgb::new(r1, g1, b1);
            let b = Rgb::new(r2, g2, b2);
            let cr = contrast_ratio(&a, &b);
            prop_assert!((1.0..=21.1).contains(&cr), "contrast({:?}, {:?}) = {}", a, b, cr);
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

    pr4xis::register_praxis_value!(prop_luminance_bounded, Verifiable);
    pr4xis::register_praxis_value!(prop_contrast_ratio_bounded, Verifiable);
    pr4xis::register_praxis_value!(prop_contrast_symmetric, Verifiable);
    pr4xis::register_praxis_value!(prop_contrast_identity, Verifiable);
    pr4xis::register_praxis_value!(prop_luminance_monotone_gray, Verifiable);
    pr4xis::register_praxis_value!(prop_dark_light_partition, Verifiable);
}
