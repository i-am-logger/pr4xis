#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// Oklab — a perceptual color space whose lightness axis is *uniform*.
///
/// WCAG relative luminance ([`super::srgb::relative_luminance`]) answers
/// "how much light", which is the right question for a contrast threshold and
/// the wrong one for "are these two steps of a ramp visibly different steps".
/// Two colors can differ substantially in relative luminance and still read as
/// the same tone, because relative luminance is linear in light rather than in
/// perceived lightness. Oklab's `L` is built to be perceptually uniform, so a
/// fixed ΔL means the same perceived gap anywhere on the scale — which is what
/// an ordinal ramp needs in order to be *readable as ordered*.
///
/// Both spaces are kept: the contrast axioms stay on WCAG luminance because
/// WCAG defines them that way, and the ramp-separation axiom uses Oklab
/// because separation is a perceptual claim.
///
/// Source: Björn Ottosson (2020), "A perceptual color space for image
/// processing", <https://bottosson.github.io/posts/oklab/> — the linear-sRGB →
/// LMS matrix, the cube-root nonlinearity, and the LMS' → Lab matrix below are
/// that post's published coefficients.
use super::rgb::Rgb;
use super::srgb::srgb_linearize;
use crate::formal::math::functions::LinearCombination;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

/// The cone-response (LMS) matrix rows, applied to LINEAR sRGB.
///
/// Source: Ottosson (2020), `linear_srgb_to_oklab`.
fn lms_long() -> LinearCombination {
    LinearCombination::new(vec![0.412_221_470_8, 0.536_332_536_3, 0.051_445_992_9])
}
fn lms_medium() -> LinearCombination {
    LinearCombination::new(vec![0.211_903_498_2, 0.680_699_545_1, 0.107_396_956_6])
}
fn lms_short() -> LinearCombination {
    LinearCombination::new(vec![0.088_302_461_9, 0.281_718_837_6, 0.629_978_700_5])
}

/// The Lab rows, applied to the cube roots of LMS.
///
/// Source: Ottosson (2020), `linear_srgb_to_oklab`.
fn lab_lightness() -> LinearCombination {
    LinearCombination::new(vec![0.210_454_255_3, 0.793_617_785_0, -0.004_072_046_8])
}
fn lab_green_red() -> LinearCombination {
    LinearCombination::new(vec![1.977_998_495_1, -2.428_592_205_0, 0.450_593_709_9])
}
fn lab_blue_yellow() -> LinearCombination {
    LinearCombination::new(vec![0.025_904_037_1, 0.782_771_766_2, -0.808_675_766_0])
}

/// A color in Oklab: perceptual lightness plus two opponent axes.
///
/// `l` is normalised so that `l = 0` is black and `l = 1` is reference white —
/// the same `[0, 1]` convention as WCAG relative luminance, but perceptually
/// spaced. `a` (green↔red) and `b` (blue↔yellow) are signed and unbounded in
/// principle; for in-gamut sRGB they stay small.
#[derive(Debug, Clone, PartialEq)]
pub struct Oklab {
    pub l: Quantity,
    pub a: Quantity,
    pub b: Quantity,
}

/// Convert an sRGB color to Oklab.
///
/// Linearizes with the IEC 61966-2-1 transfer function already carried by
/// [`srgb_linearize`], then applies Ottosson's two matrices with the cube-root
/// nonlinearity between them. The LMS values are non-negative for every
/// in-gamut sRGB input (all nine matrix coefficients are positive and the
/// linear channels are in `[0, 1]`), so the cube root is taken on non-negative
/// arguments.
///
/// Source: Ottosson (2020), <https://bottosson.github.io/posts/oklab/>
pub fn to_oklab(color: &Rgb) -> Oklab {
    let linearize = srgb_linearize();
    let rgb_lin = [
        linearize.eval(color.r as f64 / 255.0),
        linearize.eval(color.g as f64 / 255.0),
        linearize.eval(color.b as f64 / 255.0),
    ];

    let cube_root = |x: f64| x.powf(1.0 / 3.0);
    let lms = [
        cube_root(lms_long().eval(&rgb_lin)),
        cube_root(lms_medium().eval(&rgb_lin)),
        cube_root(lms_short().eval(&rgb_lin)),
    ];

    Oklab {
        l: Quantity::from_unit(lab_lightness().eval(&lms), &unit::UNITLESS),
        a: Quantity::from_unit(lab_green_red().eval(&lms), &unit::UNITLESS),
        b: Quantity::from_unit(lab_blue_yellow().eval(&lms), &unit::UNITLESS),
    }
}

/// Perceptual lightness of an sRGB color, normalised to `[0, 1]`.
///
/// The measure an ordinal ramp's step separation is stated in: a fixed
/// difference here is the same perceived difference anywhere on the scale.
pub fn lightness(color: &Rgb) -> Quantity {
    to_oklab(color).l
}

// ── Axioms ──

/// Oklab lightness is normalised: black is 0 and reference white is 1.
///
/// This is the property that makes a ΔL threshold meaningful as a fraction of
/// the full perceptual range rather than an arbitrary unit, and it is the
/// check that catches a transposed or mis-transcribed matrix row — the
/// coefficients only sum to unity at white if all six rows are right.
///
/// Source: Ottosson (2020) — Oklab is constructed so that D65 white maps to
/// `L = 1`.
pub struct OklabLightnessNormalised;

impl Axiom for OklabLightnessNormalised {
    fn verify(&self) -> Verdict {
        let epsilon = 1e-6;
        let black = lightness(&Rgb::new(0, 0, 0)).value;
        let white = lightness(&Rgb::new(255, 255, 255)).value;
        if black.abs() < epsilon && (white - 1.0).abs() < epsilon {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "OklabLightnessNormalised",
        "Oklab lightness maps black to 0 and reference white to 1",
        "Ottosson (2020), A perceptual color space for image processing"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_oklab_lightness_normalised() {
        assert!(OklabLightnessNormalised.verify().is_ok());
    }

    /// Oklab's published reference values for the sRGB primaries, to the
    /// precision Ottosson's own table states them.
    ///
    /// Source: Ottosson (2020), the sRGB test-values table.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_oklab_matches_published_reference_values() {
        let cases = [
            (Rgb::new(255, 0, 0), 0.6279, 0.2249, 0.1258),
            (Rgb::new(0, 255, 0), 0.8664, -0.2339, 0.1795),
            (Rgb::new(0, 0, 255), 0.4520, -0.0324, -0.3115),
        ];
        for (rgb, l, a, b) in cases {
            let got = to_oklab(&rgb);
            assert!(
                (got.l.value - l).abs() < 1e-3,
                "L for {rgb:?}: {}",
                got.l.value
            );
            assert!(
                (got.a.value - a).abs() < 1e-3,
                "a for {rgb:?}: {}",
                got.a.value
            );
            assert!(
                (got.b.value - b).abs() < 1e-3,
                "b for {rgb:?}: {}",
                got.b.value
            );
        }
    }

    /// Perceptual lightness is NOT relative luminance — the distinction the
    /// whole module exists for. Mid-grey sits near the middle of the
    /// perceptual scale while carrying about a fifth of the light.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_oklab_lightness_differs_from_relative_luminance() {
        let mid = Rgb::new(128, 128, 128);
        let perceptual = lightness(&mid).value;
        let photometric = super::super::srgb::relative_luminance(&mid).value;
        assert!(
            perceptual > 0.55 && perceptual < 0.65,
            "perceptual {perceptual}"
        );
        assert!(
            photometric > 0.18 && photometric < 0.25,
            "photometric {photometric}"
        );
    }
}
