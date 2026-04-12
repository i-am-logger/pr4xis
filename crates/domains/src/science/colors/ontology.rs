use super::rgb::Rgb;
use pr4xis::category::{Category, Entity, Relationship};
use pr4xis::ontology::{Axiom, Quality};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl Entity for PrimaryColor {
    fn variants() -> Vec<Self> {
        vec![
            PrimaryColor::Black,
            PrimaryColor::Red,
            PrimaryColor::Green,
            PrimaryColor::Blue,
            PrimaryColor::Yellow,
            PrimaryColor::Cyan,
            PrimaryColor::Magenta,
            PrimaryColor::White,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorMix {
    pub from: PrimaryColor,
    pub to: PrimaryColor,
}

impl Relationship for ColorMix {
    type Object = PrimaryColor;
    fn source(&self) -> PrimaryColor {
        self.from
    }
    fn target(&self) -> PrimaryColor {
        self.to
    }
}

pub struct ColorCategory;

impl Category for ColorCategory {
    type Object = PrimaryColor;
    type Morphism = ColorMix;

    fn identity(obj: &PrimaryColor) -> ColorMix {
        ColorMix {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &ColorMix, g: &ColorMix) -> Option<ColorMix> {
        if f.to != g.from {
            return None;
        }
        Some(ColorMix {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<ColorMix> {
        let c = PrimaryColor::variants();
        c.iter()
            .flat_map(|&a| c.iter().map(move |&b| ColorMix { from: a, to: b }))
            .collect()
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
