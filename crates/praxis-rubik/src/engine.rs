use crate::cube::Cube;
use crate::moves::Move;
use praxis_engine::{Action, Engine, Precondition, PreconditionResult, Situation};

impl Situation for Cube {
    fn describe(&self) -> String {
        if self.is_solved() {
            "SOLVED".to_string()
        } else {
            let counts = self.color_counts();
            format!("cube (colors: {:?})", counts)
        }
    }

    fn is_terminal(&self) -> bool {
        self.is_solved()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RubikAction(pub Move);

impl Action for RubikAction {
    type Sit = Cube;

    fn describe(&self) -> String {
        self.0.notation().to_string()
    }
}

pub struct ColorInvariant;

impl Precondition<RubikAction> for ColorInvariant {
    fn check(&self, cube: &Cube, _action: &RubikAction) -> PreconditionResult {
        let counts = cube.color_counts();
        if counts.iter().all(|&c| c == 9) {
            PreconditionResult::satisfied("color_invariant", "9 of each color")
        } else {
            PreconditionResult::violated(
                "color_invariant",
                "color counts corrupted",
                &cube.describe(),
                &_action.describe(),
            )
        }
    }

    fn describe(&self) -> &str {
        "each color must have exactly 9 stickers"
    }
}

fn apply_rubik(cube: &Cube, action: &RubikAction) -> Cube {
    cube.apply(action.0)
}

pub type RubikEngine = Engine<RubikAction>;

pub fn new_cube() -> RubikEngine {
    Engine::new(Cube::solved(), vec![Box::new(ColorInvariant)], apply_rubik)
}
