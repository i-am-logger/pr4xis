//! Colors — eight RGB/CMY primaries + achromatic endpoints.
//!
//! Models the eight discrete primaries used by additive (RGB) and
//! subtractive (CMY) colour reproduction systems plus the achromatic
//! endpoints. The continuous color science (sRGB EOTF, BT.709 luma,
//! WCAG contrast) lives in the sibling `srgb.rs` / `mixing.rs` modules;
//! this ontology is the discrete categorical layer.
//!
//! # Literature
//!
//! - **IEC 61966-2-1** *sRGB standard* — the sRGB transfer function and
//!   primaries; canonical reference for RGB primary identity and the
//!   complement relation in sRGB space.
//! - **ITU-R BT.709-6** *Parameter values for the HDTV standards for
//!   production and international programme exchange* — luma coefficients
//!   (0.2126, 0.7152, 0.0722) used by relative luminance.
//! - **W3C WCAG 2.1** *Web Content Accessibility Guidelines* §1.4.3 —
//!   relative luminance, contrast ratio.

use super::rgb::Rgb;
use crate::formal::math::quantity::unit::UNITLESS;
use crate::formal::math::quantity::value::Quantity;
use pr4xis::ontology::{Axiom, Ontology, Quality, QualityKind};

pr4xis::ontology! {
    name: "Color",
    source: "IEC 61966-2-1 (sRGB); ITU-R BT.709-6; W3C WCAG 2.1",

    concepts: [Black, Red, Green, Blue, Yellow, Cyan, Magenta, White],

    labels: {
        Black: ("en", "Black",
            "Achromatic endpoint — zero luminance in the sRGB cube (0,0,0). IEC 61966-2-1."),
        Red: ("en", "Red",
            "Additive primary — sRGB (255,0,0). IEC 61966-2-1 primary R."),
        Green: ("en", "Green",
            "Additive primary — sRGB (0,255,0). IEC 61966-2-1 primary G."),
        Blue: ("en", "Blue",
            "Additive primary — sRGB (0,0,255). IEC 61966-2-1 primary B."),
        Yellow: ("en", "Yellow",
            "Subtractive primary — sRGB (255,255,0). Complement of Blue in additive RGB."),
        Cyan: ("en", "Cyan",
            "Subtractive primary — sRGB (0,255,255). Complement of Red in additive RGB."),
        Magenta: ("en", "Magenta",
            "Subtractive primary — sRGB (255,0,255). Complement of Green in additive RGB."),
        White: ("en", "White",
            "Achromatic endpoint — full intensity in all sRGB channels (255,255,255). IEC 61966-2-1."),
    },

    opposes: [
        // sRGB additive complement pairs — each pair channel-sums to white.
        (Red, Cyan),
        (Cyan, Red),
        (Green, Magenta),
        (Magenta, Green),
        (Blue, Yellow),
        (Yellow, Blue),
        // Achromatic polarity.
        (Black, White),
        (White, Black),
    ],
}

impl ColorConcept {
    /// sRGB triplet for the primary. IEC 61966-2-1 standard primaries.
    pub fn rgb(&self) -> Rgb {
        match self {
            ColorConcept::Black => Rgb::BLACK,
            ColorConcept::Red => Rgb::RED,
            ColorConcept::Green => Rgb::GREEN,
            ColorConcept::Blue => Rgb::BLUE,
            ColorConcept::Yellow => Rgb::YELLOW,
            ColorConcept::Cyan => Rgb::CYAN,
            ColorConcept::Magenta => Rgb::MAGENTA,
            ColorConcept::White => Rgb::WHITE,
        }
    }
}

/// Quality: relative luminance per BT.709 / WCAG 2.1.
///
/// WCAG 2.1 §1.4.3 relative luminance is normalised to the dimensionless
/// range `[0, 1]` (0 = black, 1 = reference white), so it is carried as a
/// dimensionless [`Quantity`] (unit `UNITLESS`) rather than an absolute
/// photometric luminance in cd/m².
#[derive(Debug, Clone)]
pub struct Luminance;

impl Quality for Luminance {
    type Individual = ColorConcept;
    type Value = Quantity;
    const KIND: QualityKind = QualityKind::Physical;

    fn get(&self, c: &ColorConcept) -> Option<Quantity> {
        Some(Quantity::from_unit(c.rgb().luminance(), &UNITLESS))
    }
}

/// Quality: whether a primary is an additive primary (R, G, B).
#[derive(Debug, Clone)]
pub struct IsAdditivePrimary;

impl Quality for IsAdditivePrimary {
    type Individual = ColorConcept;
    type Value = ();

    fn get(&self, c: &ColorConcept) -> Option<()> {
        if matches!(
            c,
            ColorConcept::Red | ColorConcept::Green | ColorConcept::Blue
        ) {
            Some(())
        } else {
            None
        }
    }
}

impl Ontology for ColorOntology {
    type Cat = ColorCategory;
    type Qual = Luminance;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(ComplementsAddToWhite));
        axioms
    }
}

/// Axiom: each sRGB additive-complement pair sums per-channel to white (255).
///
/// IEC 61966-2-1: in the sRGB encoding the complement of an additive
/// primary in the (R,G,B) cube is the channel-wise inversion; the three
/// canonical complement pairs (R↔C, G↔M, B↔Y) saturate each channel to
/// 255.
pub struct ComplementsAddToWhite;

impl Axiom for ComplementsAddToWhite {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let pairs = [
            (ColorConcept::Red, ColorConcept::Cyan),
            (ColorConcept::Green, ColorConcept::Magenta),
            (ColorConcept::Blue, ColorConcept::Yellow),
        ];
        for (a, b) in pairs {
            let (ar, br) = (a.rgb(), b.rgb());
            if !(ar.r.saturating_add(br.r) == 255
                && ar.g.saturating_add(br.g) == 255
                && ar.b.saturating_add(br.b) == 255)
            {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "ComplementsAddToWhite",
        "each sRGB additive complement pair (R/C, G/M, B/Y) saturates every channel to 255",
        "IEC 61966-2-1 (sRGB)"
    );
}

pr4xis::register_axiom!(ComplementsAddToWhite, "IEC 61966-2-1 (sRGB)");

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn eight_colors() {
        assert_eq!(ColorConcept::variants().len(), 8);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<ColorCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        ColorOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn luminance_endpoints() {
        assert_eq!(Luminance.get(&ColorConcept::Black).unwrap().value, 0.0);
        assert!(Luminance.get(&ColorConcept::White).unwrap().value > 0.99);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn three_additive_primaries() {
        let q = IsAdditivePrimary;
        let count = ColorConcept::variants()
            .iter()
            .filter(|c| q.get(c).is_some())
            .count();
        assert_eq!(count, 3);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn complements_add_to_white_holds() {
        match ComplementsAddToWhite.verify() {
            Ok(_) => {}
            Err(c) => panic!("ComplementsAddToWhite failed: {}", c.meta().name.as_str()),
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn complement_pairs_present_as_opposition() {
        // sRGB additive complement pairs are encoded as Opposition-kinded
        // morphisms in the category.
        let opposed: Vec<_> = ColorCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == ColorRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opposed.contains(&(ColorConcept::Red, ColorConcept::Cyan)));
        assert!(opposed.contains(&(ColorConcept::Green, ColorConcept::Magenta)));
        assert!(opposed.contains(&(ColorConcept::Blue, ColorConcept::Yellow)));
        assert!(opposed.contains(&(ColorConcept::Black, ColorConcept::White)));
    }

    fn arb_color() -> impl Strategy<Value = ColorConcept> {
        proptest::sample::select(ColorConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_luminance_total(c in arb_color()) {
            prop_assert!(Luminance.get(&c).is_some());
        }

        #[test]
        fn prop_luminance_bounded(c in arb_color()) {
            let l = Luminance.get(&c).unwrap().value;
            prop_assert!((0.0..=1.0).contains(&l));
        }

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            let opposed: std::collections::HashSet<_> = ColorCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == ColorRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in opposed.iter() {
                prop_assert!(opposed.contains(&(*b, *a)),
                    "opposition not symmetric: {:?} → {:?} but not back", a, b);
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in ColorOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }
    }

    pr4xis::register_praxis_value!(prop_luminance_total, Verifiable);
    pr4xis::register_praxis_value!(prop_luminance_bounded, Verifiable);
    pr4xis::register_praxis_value!(prop_opposition_is_symmetric, Verifiable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
}
