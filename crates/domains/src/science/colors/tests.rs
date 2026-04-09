use super::*;
use proptest::prelude::*;

fn arb_rgb() -> impl Strategy<Value = Rgb> {
    (0..=255u8, 0..=255u8, 0..=255u8).prop_map(|(r, g, b)| Rgb::new(r, g, b))
}

fn arb_alpha() -> impl Strategy<Value = f64> {
    (0..=100u32).prop_map(|a| a as f64 / 100.0)
}

// =============================================================================
// RGB enforcement tests
// =============================================================================

#[test]
fn test_constants() {
    assert_eq!(Rgb::BLACK.luminance(), 0.0);
    assert!(Rgb::WHITE.luminance() > 0.99);
    assert!(Rgb::BLACK.is_dark());
    assert!(!Rgb::WHITE.is_dark());
}

#[test]
fn test_achromatic() {
    assert!(Rgb::BLACK.is_achromatic());
    assert!(Rgb::WHITE.is_achromatic());
    assert!(Rgb::new(128, 128, 128).is_achromatic());
    assert!(!Rgb::RED.is_achromatic());
}

#[test]
fn test_invert() {
    assert_eq!(Rgb::BLACK.invert(), Rgb::WHITE);
    assert_eq!(Rgb::WHITE.invert(), Rgb::BLACK);
    assert_eq!(Rgb::RED.invert(), Rgb::CYAN);
}

#[test]
fn test_wcag_contrast() {
    assert!(Rgb::BLACK.wcag_aa(Rgb::WHITE));
    assert!(Rgb::BLACK.wcag_aaa(Rgb::WHITE));
    assert!(!Rgb::new(128, 128, 128).wcag_aa(Rgb::new(140, 140, 140)));
}

#[test]
fn test_additive_mixing() {
    assert_eq!(mix(Rgb::RED, Rgb::GREEN, MixMode::Additive), Rgb::YELLOW);
    assert_eq!(mix(Rgb::RED, Rgb::BLUE, MixMode::Additive), Rgb::MAGENTA);
    assert_eq!(mix(Rgb::GREEN, Rgb::BLUE, MixMode::Additive), Rgb::CYAN);
}

#[test]
fn test_multiply_with_black() {
    assert_eq!(mix(Rgb::RED, Rgb::BLACK, MixMode::Multiply), Rgb::BLACK);
}

#[test]
fn test_multiply_with_white() {
    assert_eq!(mix(Rgb::RED, Rgb::WHITE, MixMode::Multiply), Rgb::RED);
}

#[test]
fn test_blend_extremes() {
    assert_eq!(blend(Rgb::RED, Rgb::BLUE, 0.0), Rgb::RED);
    assert_eq!(blend(Rgb::RED, Rgb::BLUE, 1.0), Rgb::BLUE);
}

#[test]
fn test_mix_many() {
    let avg = mix_many(&[Rgb::BLACK, Rgb::WHITE]).unwrap();
    assert_eq!(avg, Rgb::new(127, 127, 127));
    assert_eq!(mix_many(&[]), None);
}

#[test]
fn test_complement() {
    assert_eq!(complement(Rgb::RED), Rgb::CYAN);
    assert_eq!(complement(Rgb::BLACK), Rgb::WHITE);
}

#[test]
fn test_grayscale() {
    let gray = Rgb::RED.grayscale();
    assert!(gray.is_achromatic());
}

// =============================================================================
// Property-based tests
// =============================================================================

proptest! {
    /// Luminance is always 0.0-1.0
    #[test]
    fn prop_luminance_range(c in arb_rgb()) {
        let l = c.luminance();
        prop_assert!((0.0..=1.0).contains(&l), "luminance {} out of range", l);
    }

    /// Black has luminance 0, white has luminance ~1
    #[test]
    fn prop_black_darkest(_x in 0..1u8) {
        prop_assert_eq!(Rgb::BLACK.luminance(), 0.0);
        prop_assert!(Rgb::WHITE.luminance() > 0.99);
    }

    /// Double invert = identity
    #[test]
    fn prop_double_invert(c in arb_rgb()) {
        prop_assert_eq!(c.invert().invert(), c);
    }

    /// Contrast ratio is >= 1.0
    #[test]
    fn prop_contrast_at_least_1(a in arb_rgb(), b in arb_rgb()) {
        prop_assert!(a.contrast_ratio(b) >= 1.0);
    }

    /// Contrast ratio is symmetric
    #[test]
    fn prop_contrast_symmetric(a in arb_rgb(), b in arb_rgb()) {
        let ab = a.contrast_ratio(b);
        let ba = b.contrast_ratio(a);
        prop_assert!((ab - ba).abs() < 0.001);
    }

    /// Contrast with self = 1.0
    #[test]
    fn prop_self_contrast(c in arb_rgb()) {
        let ratio = c.contrast_ratio(c);
        prop_assert!((ratio - 1.0).abs() < 0.001);
    }

    /// Black vs white has max contrast (21:1)
    #[test]
    fn prop_max_contrast(_x in 0..1u8) {
        let ratio = Rgb::BLACK.contrast_ratio(Rgb::WHITE);
        prop_assert!(ratio > 20.0);
    }

    /// Grayscale is always achromatic
    #[test]
    fn prop_grayscale_achromatic(c in arb_rgb()) {
        prop_assert!(c.grayscale().is_achromatic());
    }

    /// Achromatic colors have no hue
    #[test]
    fn prop_achromatic_no_hue(v in 0..=255u8) {
        let c = Rgb::new(v, v, v);
        prop_assert!(c.hue().is_none());
    }

    /// Pure colors have hue
    #[test]
    fn prop_pure_colors_have_hue(_x in 0..1u8) {
        prop_assert!(Rgb::RED.hue().is_some());
        prop_assert!(Rgb::GREEN.hue().is_some());
        prop_assert!(Rgb::BLUE.hue().is_some());
    }

    /// Saturation is 0.0-1.0
    #[test]
    fn prop_saturation_range(c in arb_rgb()) {
        let s = c.saturation();
        prop_assert!((0.0..=1.0).contains(&s));
    }

    /// Achromatic has 0 saturation
    #[test]
    fn prop_achromatic_zero_saturation(v in 0..=255u8) {
        prop_assert_eq!(Rgb::new(v, v, v).saturation(), 0.0);
    }

    /// Additive mix with black = identity
    #[test]
    fn prop_additive_black_identity(c in arb_rgb()) {
        prop_assert_eq!(mix(c, Rgb::BLACK, MixMode::Additive), c);
    }

    /// Additive mix is commutative
    #[test]
    fn prop_additive_commutative(a in arb_rgb(), b in arb_rgb()) {
        prop_assert_eq!(mix(a, b, MixMode::Additive), mix(b, a, MixMode::Additive));
    }

    /// Average mix is commutative
    #[test]
    fn prop_average_commutative(a in arb_rgb(), b in arb_rgb()) {
        prop_assert_eq!(mix(a, b, MixMode::Average), mix(b, a, MixMode::Average));
    }

    /// Multiply with white = identity
    #[test]
    fn prop_multiply_white_identity(c in arb_rgb()) {
        prop_assert_eq!(mix(c, Rgb::WHITE, MixMode::Multiply), c);
    }

    /// Multiply with black = black
    #[test]
    fn prop_multiply_black_absorbs(c in arb_rgb()) {
        prop_assert_eq!(mix(c, Rgb::BLACK, MixMode::Multiply), Rgb::BLACK);
    }

    /// Multiply is commutative
    #[test]
    fn prop_multiply_commutative(a in arb_rgb(), b in arb_rgb()) {
        prop_assert_eq!(mix(a, b, MixMode::Multiply), mix(b, a, MixMode::Multiply));
    }

    /// Screen with black = identity
    #[test]
    fn prop_screen_black_identity(c in arb_rgb()) {
        prop_assert_eq!(mix(c, Rgb::BLACK, MixMode::Screen), c);
    }

    /// Screen with white = white
    #[test]
    fn prop_screen_white_absorbs(c in arb_rgb()) {
        prop_assert_eq!(mix(c, Rgb::WHITE, MixMode::Screen), Rgb::WHITE);
    }

    /// Blend at 0 = background
    #[test]
    fn prop_blend_zero(bg in arb_rgb(), fg in arb_rgb()) {
        prop_assert_eq!(blend(bg, fg, 0.0), bg);
    }

    /// Blend at 1 = foreground
    #[test]
    fn prop_blend_one(bg in arb_rgb(), fg in arb_rgb()) {
        prop_assert_eq!(blend(bg, fg, 1.0), fg);
    }

    /// Blend with self = self (within rounding)
    #[test]
    fn prop_blend_self(c in arb_rgb(), alpha in arb_alpha()) {
        let result = blend(c, c, alpha);
        // Allow +/- 1 for floating point rounding
        prop_assert!((result.r as i16 - c.r as i16).abs() <= 1);
        prop_assert!((result.g as i16 - c.g as i16).abs() <= 1);
        prop_assert!((result.b as i16 - c.b as i16).abs() <= 1);
    }

    /// mix_many of single color = that color
    #[test]
    fn prop_mix_many_single(c in arb_rgb()) {
        prop_assert_eq!(mix_many(&[c]), Some(c));
    }

    /// mix_many is order-independent (commutative)
    #[test]
    fn prop_mix_many_commutative(a in arb_rgb(), b in arb_rgb()) {
        prop_assert_eq!(mix_many(&[a, b]), mix_many(&[b, a]));
    }

    /// Complement of complement = self (within rounding)
    #[test]
    fn prop_complement_involution(c in arb_rgb()) {
        prop_assert_eq!(complement(complement(c)), c);
    }

    /// WCAG AA implies WCAG large text (ratio >= 3.0)
    #[test]
    fn prop_wcag_aa_implies_large(a in arb_rgb(), b in arb_rgb()) {
        if a.wcag_aa(b) {
            prop_assert!(a.contrast_ratio(b) >= 3.0);
        }
    }

    /// WCAG AAA implies AA
    #[test]
    fn prop_wcag_aaa_implies_aa(a in arb_rgb(), b in arb_rgb()) {
        if a.wcag_aaa(b) {
            prop_assert!(a.wcag_aa(b));
        }
    }
}

// =============================================================================
// Engine tests — Situation/Action/Precondition/Trace
// =============================================================================

#[test]
fn engine_invert_twice_returns_original() {
    let e = new_color(Rgb::new(100, 150, 200));
    let e = e.try_next(ColorAction::Invert).unwrap();
    let e = e.try_next(ColorAction::Invert).unwrap();
    assert_eq!(*e.situation(), Rgb::new(100, 150, 200));
}

#[test]
fn engine_blend_invalid_alpha_rejected() {
    let e = new_color(Rgb::new(128, 128, 128));
    let result = e.try_next(ColorAction::Blend {
        color: Rgb::new(255, 0, 0),
        alpha: 1.5,
    });
    assert!(result.is_err());
}

#[test]
fn engine_mix_and_back() {
    let e = new_color(Rgb::new(255, 0, 0)); // Red
    let e = e
        .try_next(ColorAction::Mix {
            color: Rgb::new(0, 0, 255),
            mode: MixMode::Average,
        })
        .unwrap();
    let mixed = *e.situation();
    let e = e.back().unwrap();
    assert_eq!(*e.situation(), Rgb::new(255, 0, 0));
    let e = e.forward().unwrap();
    assert_eq!(*e.situation(), mixed);
}

#[test]
fn engine_set_channel() {
    let e = new_color(Rgb::new(0, 0, 0));
    let e = e
        .try_next(ColorAction::SetChannel {
            r: Some(255),
            g: None,
            b: None,
        })
        .unwrap();
    assert_eq!(e.situation().r, 255);
    assert_eq!(e.situation().g, 0);
}

#[test]
fn engine_trace_records() {
    let e = new_color(Rgb::new(128, 128, 128));
    let e = e.try_next(ColorAction::Invert).unwrap();
    let e = e.try_next(ColorAction::Grayscale).unwrap();
    assert_eq!(e.trace().entries().len(), 2);
    assert!(e.trace().entries().iter().all(|entry| entry.success));
}
