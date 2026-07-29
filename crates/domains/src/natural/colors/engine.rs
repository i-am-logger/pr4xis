#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::mixing::{MixMode, blend, mix};
use super::rgb::Rgb;
use pr4xis::engine::{Action, Engine, Precondition, Situation};
use pr4xis::logic::proof::{Counterexample, SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

fn axiom_meta(name: &'static str, description: &'static str, citation: &'static str) -> Provenance {
    Provenance {
        name: OntologyName::new_static(name),
        description: Label::new_static(description),
        citation: Citation::parse_static(citation),
        module_path: ModulePath::new_static(module_path!()),
    }
}

impl Situation for Rgb {}

#[derive(Debug, Clone, PartialEq)]
pub enum ColorAction {
    Mix {
        color: Rgb,
        mode: MixMode,
    },
    Blend {
        color: Rgb,
        alpha: f64,
    },
    Invert,
    Grayscale,
    SetChannel {
        r: Option<u8>,
        g: Option<u8>,
        b: Option<u8>,
    },
}

impl Action for ColorAction {
    type Sit = Rgb;
}

/// WCAG contrast check: warn if resulting color has poor contrast with black/white.
pub struct ContrastCheck;

impl Precondition<ColorAction> for ContrastCheck {
    fn check(&self, color: &Rgb, action: &ColorAction) -> Verdict {
        let meta = axiom_meta(
            "contrast_check",
            "result must have usable contrast",
            "WCAG 2.1 (2018) §1.4.3 Contrast (Minimum); ISO 9241-303:2011 luminance contrast",
        );
        let result = apply_color(color, action).unwrap_or(*color);
        let contrast_black = result.contrast_ratio(Rgb::BLACK).value;
        let contrast_white = result.contrast_ratio(Rgb::WHITE).value;
        let best_contrast = contrast_black.max(contrast_white);

        if best_contrast < 2.0 {
            Err(Box::new(SimpleCounterexample::new(meta)))
        } else {
            Ok(Box::new(SimpleProof::new(meta)))
        }
    }
}

/// Alpha must be 0.0-1.0.
pub struct ValidAlpha;

impl Precondition<ColorAction> for ValidAlpha {
    fn check(&self, _color: &Rgb, action: &ColorAction) -> Verdict {
        let meta = axiom_meta(
            "valid_alpha",
            "blend alpha must be 0.0-1.0",
            "Porter & Duff (1984) Compositing Digital Images, SIGGRAPH '84 §3",
        );
        if let ColorAction::Blend { alpha, .. } = action
            && (*alpha < 0.0 || *alpha > 1.0)
        {
            return Err(Box::new(SimpleCounterexample::new(meta)));
        }
        Ok(Box::new(SimpleProof::new(meta)))
    }
}

fn apply_color(color: &Rgb, action: &ColorAction) -> Result<Rgb, Box<dyn Counterexample>> {
    Ok(match action {
        ColorAction::Mix { color: other, mode } => mix(*color, *other, *mode),
        ColorAction::Blend { color: fg, alpha } => blend(*color, *fg, *alpha),
        ColorAction::Invert => color.invert(),
        ColorAction::Grayscale => color.grayscale(),
        ColorAction::SetChannel { r, g, b } => Rgb::new(
            r.unwrap_or(color.r),
            g.unwrap_or(color.g),
            b.unwrap_or(color.b),
        ),
    })
}

pub type ColorEngine = Engine<ColorAction>;

pub fn new_color(initial: Rgb) -> ColorEngine {
    Engine::new(
        initial,
        vec![Box::new(ValidAlpha), Box::new(ContrastCheck)],
        apply_color,
    )
}
