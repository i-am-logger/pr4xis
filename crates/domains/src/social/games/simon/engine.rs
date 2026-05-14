#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::color::SimonColor;
use super::game::{Game, GameState};
use pr4xis::engine::{Action, Engine, Precondition, Situation};
use pr4xis::logic::proof::{Counterexample, SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

impl Situation for Game {}

#[derive(Debug, Clone, PartialEq)]
pub enum SimonAction {
    StartInput,
    Press(SimonColor),
    NextRound,
}

impl Action for SimonAction {
    type Sit = Game;
}

fn simon_meta(name: &'static str, description: &'static str) -> Provenance {
    Provenance {
        name: OntologyName::new_static(name),
        description: Label::new_static(description),
        citation: Citation::parse_static(
            "Baer & Morrison (1978) Simon — Milton Bradley electronic memory game (US Patent 4,207,087)",
        ),
        module_path: ModulePath::new_static(module_path!()),
    }
}

pub struct ValidState;

impl Precondition<SimonAction> for ValidState {
    fn check(&self, game: &Game, action: &SimonAction) -> Verdict {
        let meta = simon_meta("ValidState", "action must be valid for current game state");
        let ok = match action {
            SimonAction::StartInput => matches!(game.state(), GameState::Showing),
            SimonAction::Press(_) => matches!(game.state(), GameState::Inputting { .. }),
            SimonAction::NextRound => matches!(game.state(), GameState::RoundComplete),
        };
        if ok {
            Ok(Box::new(SimpleProof::new(meta)))
        } else {
            Err(Box::new(SimpleCounterexample::new(meta)))
        }
    }
}

fn apply_simon(game: &Game, action: &SimonAction) -> Result<Game, Box<dyn Counterexample>> {
    let mut next = game.clone();
    match action {
        SimonAction::StartInput => {
            let _ = next.start_input();
        }
        SimonAction::Press(color) => {
            next.input(*color);
        }
        SimonAction::NextRound => {
            let _ = next.next_round();
        }
    }
    Ok(next)
}

pub type SimonEngine = Engine<SimonAction>;

pub fn new_simon(seed: u64) -> SimonEngine {
    Engine::new(Game::new(seed), vec![Box::new(ValidState)], apply_simon)
}
