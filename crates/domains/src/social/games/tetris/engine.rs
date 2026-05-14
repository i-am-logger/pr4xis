#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::game::{Game, GameAction};
use pr4xis::engine::{Action, Engine, Precondition, Situation};
use pr4xis::logic::proof::{Counterexample, SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

impl Situation for Game {}

#[derive(Debug, Clone, PartialEq)]
pub struct TetrisAction(pub GameAction);

impl Action for TetrisAction {
    type Sit = Game;
}

fn tetris_meta(name: &'static str, description: &'static str) -> Provenance {
    Provenance {
        name: OntologyName::new_static(name),
        description: Label::new_static(description),
        citation: Citation::parse_static(
            "Pajitnov (1985) Tetris — original 6x10 well; Demaine, Hohenberger & Liben-Nowell (2004) Tetris is hard, even to approximate, COCOON 2003",
        ),
        module_path: ModulePath::new_static(module_path!()),
    }
}

pub struct GameActive;

impl Precondition<TetrisAction> for GameActive {
    fn check(&self, game: &Game, _action: &TetrisAction) -> Verdict {
        let meta = tetris_meta("GameActive", "game must not be over");
        if game.game_over {
            Err(Box::new(SimpleCounterexample::new(meta)))
        } else {
            Ok(Box::new(SimpleProof::new(meta)))
        }
    }
}

fn apply_tetris(game: &Game, action: &TetrisAction) -> Result<Game, Box<dyn Counterexample>> {
    let mut next = game.clone();
    next.act(action.0);
    Ok(next)
}

pub type TetrisEngine = Engine<TetrisAction>;

pub fn new_tetris(seed: u64) -> TetrisEngine {
    Engine::new(Game::new(seed), vec![Box::new(GameActive)], apply_tetris)
}
