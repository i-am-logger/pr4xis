#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::color::SimonColor;
use super::game::{Game, GameState};
use pr4xis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};

impl Situation for Game {
    fn describe(&self) -> String {
        format!(
            "round={} seq_len={} state={:?}",
            self.round(),
            self.sequence_length(),
            self.state()
        )
    }

    fn is_terminal(&self) -> bool {
        matches!(self.state(), GameState::GameOver { .. })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SimonAction {
    StartInput,
    Press(SimonColor),
    NextRound,
}

impl Action for SimonAction {
    type Sit = Game;

    fn describe(&self) -> String {
        match self {
            SimonAction::StartInput => "start input".into(),
            SimonAction::Press(c) => format!("press {:?}", c),
            SimonAction::NextRound => "next round".into(),
        }
    }
}

pub struct ValidState;

impl Precondition<SimonAction> for ValidState {
    fn check(&self, game: &Game, action: &SimonAction) -> PreconditionResult {
        match action {
            SimonAction::StartInput => {
                let ok = matches!(game.state(), GameState::Showing);
                if ok {
                    PreconditionResult::satisfied("valid_state", "in showing state")
                } else {
                    PreconditionResult::violated(
                        "valid_state",
                        "must be in showing state to start input",
                        &game.describe(),
                        &action.describe(),
                    )
                }
            }
            SimonAction::Press(_) => {
                let ok = matches!(game.state(), GameState::Inputting { .. });
                if ok {
                    PreconditionResult::satisfied("valid_state", "in inputting state")
                } else {
                    PreconditionResult::violated(
                        "valid_state",
                        "must be in inputting state to press",
                        &game.describe(),
                        &action.describe(),
                    )
                }
            }
            SimonAction::NextRound => {
                let ok = matches!(game.state(), GameState::RoundComplete);
                if ok {
                    PreconditionResult::satisfied("valid_state", "round complete")
                } else {
                    PreconditionResult::violated(
                        "valid_state",
                        "must complete round before advancing",
                        &game.describe(),
                        &action.describe(),
                    )
                }
            }
        }
    }

    fn describe(&self) -> &str {
        "action must be valid for current game state"
    }
}

fn apply_simon(game: &Game, action: &SimonAction) -> Result<Game, String> {
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
