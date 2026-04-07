use crate::mixing::{MixMode, blend, mix};
use crate::rgb::Rgb;
use praxis_engine::{Action, Engine, Precondition, PreconditionResult, Situation};

impl Situation for Rgb {
    fn describe(&self) -> String {
        format!(
            "rgb({}, {}, {}) lum={:.2}",
            self.r,
            self.g,
            self.b,
            self.luminance()
        )
    }

    fn is_terminal(&self) -> bool {
        false
    }
}

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

    fn describe(&self) -> String {
        match self {
            ColorAction::Mix { color, mode } => format!(
                "mix {:?} with rgb({},{},{})",
                mode, color.r, color.g, color.b
            ),
            ColorAction::Blend { color, alpha } => format!(
                "blend rgb({},{},{}) at {:.0}%",
                color.r,
                color.g,
                color.b,
                alpha * 100.0
            ),
            ColorAction::Invert => "invert".into(),
            ColorAction::Grayscale => "grayscale".into(),
            ColorAction::SetChannel { r, g, b } => format!("set r={:?} g={:?} b={:?}", r, g, b),
        }
    }
}

/// WCAG contrast check: warn if resulting color has poor contrast with black/white.
pub struct ContrastCheck;

impl Precondition<ColorAction> for ContrastCheck {
    fn check(&self, color: &Rgb, action: &ColorAction) -> PreconditionResult {
        // Apply the action speculatively to check the result
        let result = apply_color(color, action);
        let contrast_black = result.contrast_ratio(Rgb::BLACK);
        let contrast_white = result.contrast_ratio(Rgb::WHITE);
        let best_contrast = contrast_black.max(contrast_white);

        if best_contrast < 2.0 {
            PreconditionResult::violated(
                "contrast_check",
                &format!(
                    "result rgb({},{},{}) has very low contrast ({:.1}:1)",
                    result.r, result.g, result.b, best_contrast
                ),
                &color.describe(),
                &action.describe(),
            )
        } else {
            PreconditionResult::satisfied(
                "contrast_check",
                &format!("contrast {:.1}:1 with best background", best_contrast),
            )
        }
    }

    fn describe(&self) -> &str {
        "result must have usable contrast"
    }
}

/// Alpha must be 0.0-1.0.
pub struct ValidAlpha;

impl Precondition<ColorAction> for ValidAlpha {
    fn check(&self, _color: &Rgb, action: &ColorAction) -> PreconditionResult {
        if let ColorAction::Blend { alpha, .. } = action
            && (*alpha < 0.0 || *alpha > 1.0)
        {
            return PreconditionResult::violated(
                "valid_alpha",
                &format!("alpha {} out of range [0,1]", alpha),
                &_color.describe(),
                &action.describe(),
            );
        }
        PreconditionResult::satisfied("valid_alpha", "alpha in range")
    }

    fn describe(&self) -> &str {
        "blend alpha must be 0.0-1.0"
    }
}

fn apply_color(color: &Rgb, action: &ColorAction) -> Rgb {
    match action {
        ColorAction::Mix { color: other, mode } => mix(*color, *other, *mode),
        ColorAction::Blend { color: fg, alpha } => blend(*color, *fg, *alpha),
        ColorAction::Invert => color.invert(),
        ColorAction::Grayscale => color.grayscale(),
        ColorAction::SetChannel { r, g, b } => Rgb::new(
            r.unwrap_or(color.r),
            g.unwrap_or(color.g),
            b.unwrap_or(color.b),
        ),
    }
}

pub type ColorEngine = Engine<ColorAction>;

pub fn new_color(initial: Rgb) -> ColorEngine {
    Engine::new(
        initial,
        vec![Box::new(ValidAlpha), Box::new(ContrastCheck)],
        apply_color,
    )
}
