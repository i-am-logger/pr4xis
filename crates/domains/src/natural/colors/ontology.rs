use super::rgb::Rgb;
use pr4xis::category::Entity;
use pr4xis::define_dense_category;
use pr4xis::ontology::{Axiom, Quality};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum PrimaryColor {
    Black,
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    White,
}

impl PrimaryColor {
    pub fn rgb(&self) -> Rgb {
        match self {
            PrimaryColor::Black => Rgb::BLACK,
            PrimaryColor::Red => Rgb::RED,
            PrimaryColor::Green => Rgb::GREEN,
            PrimaryColor::Blue => Rgb::BLUE,
            PrimaryColor::Yellow => Rgb::YELLOW,
            PrimaryColor::Cyan => Rgb::CYAN,
            PrimaryColor::Magenta => Rgb::MAGENTA,
            PrimaryColor::White => Rgb::WHITE,
        }
    }
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

define_dense_category! {
    /// Discrete category over primary color entities.
    pub ColorCategory {
        entity: PrimaryColor,
        relation: ColorMix,
    }
}

#[derive(Debug, Clone)]
pub struct Luminance;
impl Quality for Luminance {
    type Individual = PrimaryColor;
    type Value = f64;
    fn get(&self, c: &PrimaryColor) -> Option<f64> {
        Some(c.rgb().luminance())
    }
}

#[derive(Debug, Clone)]
pub struct IsPrimary;
impl Quality for IsPrimary {
    type Individual = PrimaryColor;
    type Value = ();
    fn get(&self, c: &PrimaryColor) -> Option<()> {
        if matches!(
            c,
            PrimaryColor::Red | PrimaryColor::Green | PrimaryColor::Blue
        ) {
            Some(())
        } else {
            None
        }
    }
}

pub struct ComplementsAddToWhite;
impl Axiom for ComplementsAddToWhite {
    fn description(&self) -> &str {
        "complement pairs add to white"
    }
    fn holds(&self) -> bool {
        [
            (PrimaryColor::Red, PrimaryColor::Cyan),
            (PrimaryColor::Green, PrimaryColor::Magenta),
            (PrimaryColor::Blue, PrimaryColor::Yellow),
        ]
        .iter()
        .all(|(a, b)| {
            let (ar, br) = (a.rgb(), b.rgb());
            ar.r.saturating_add(br.r) == 255
                && ar.g.saturating_add(br.g) == 255
                && ar.b.saturating_add(br.b) == 255
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_8_colors() {
        assert_eq!(PrimaryColor::variants().len(), 8);
    }

    #[test]
    fn test_category_laws() {
        pr4xis::category::validate::check_category_laws::<ColorCategory>().unwrap();
    }

    #[test]
    fn test_luminance() {
        assert_eq!(Luminance.get(&PrimaryColor::Black), Some(0.0));
        assert!(Luminance.get(&PrimaryColor::White).unwrap() > 0.99);
    }

    #[test]
    fn test_primaries() {
        assert_eq!(IsPrimary.individuals_with().len(), 3);
    }

    #[test]
    fn test_complements() {
        assert!(ComplementsAddToWhite.holds());
    }
}
