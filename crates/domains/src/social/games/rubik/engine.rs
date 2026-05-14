#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::cube::Cube;
use super::moves::Move;
use pr4xis::engine::{Action, Engine, Precondition, Situation};
use pr4xis::logic::proof::{Counterexample, SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

impl Situation for Cube {}

#[derive(Debug, Clone, PartialEq)]
pub struct RubikAction(pub Move);

impl Action for RubikAction {
    type Sit = Cube;
}

/// Helper: build typed Provenance for a Rubik's-cube precondition axiom.
fn rubik_meta(name: &'static str, description: &'static str) -> Provenance {
    Provenance {
        name: OntologyName::new_static(name),
        description: Label::new_static(description),
        citation: Citation::parse_static(
            "Singmaster (1979) Notes on Rubik's Magic Cube; Joyner (2008) Adventures in Group Theory",
        ),
        module_path: ModulePath::new_static(module_path!()),
    }
}

pub struct ColorInvariant;

impl Precondition<RubikAction> for ColorInvariant {
    fn check(&self, cube: &Cube, _action: &RubikAction) -> Verdict {
        let meta = rubik_meta("ColorInvariant", "each color must have exactly 9 stickers");
        let counts = cube.color_counts();
        if counts.iter().all(|&c| c == 9) {
            Ok(Box::new(SimpleProof::new(meta)))
        } else {
            Err(Box::new(SimpleCounterexample::new(meta)))
        }
    }
}

fn apply_rubik(cube: &Cube, action: &RubikAction) -> Result<Cube, Box<dyn Counterexample>> {
    Ok(cube.apply(action.0))
}

pub type RubikEngine = Engine<RubikAction>;

pub fn new_cube() -> RubikEngine {
    Engine::new(Cube::solved(), vec![Box::new(ColorInvariant)], apply_rubik)
}
