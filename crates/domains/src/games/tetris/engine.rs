use super::game::{Game, GameAction};
use praxis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};

impl Situation for Game {
    fn describe(&self) -> String {
        format!(
            "score={} level={} game_over={}",
            self.score, self.level, self.game_over
        )
    }

    fn is_terminal(&self) -> bool {
        self.game_over
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TetrisAction(pub GameAction);

impl Action for TetrisAction {
    type Sit = Game;

    fn describe(&self) -> String {
        format!("{:?}", self.0)
    }
}

pub struct GameActive;

impl Precondition<TetrisAction> for GameActive {
    fn check(&self, game: &Game, action: &TetrisAction) -> PreconditionResult {
        if game.game_over {
            PreconditionResult::violated(
                "game_active",
                "game is over",
                &game.describe(),
                &action.describe(),
            )
        } else {
            PreconditionResult::satisfied("game_active", "game in progress")
        }
    }

    fn describe(&self) -> &str {
        "game must not be over"
    }
}

fn apply_tetris(game: &Game, action: &TetrisAction) -> Game {
    let mut next = game.clone();
    next.act(action.0);
    next
}

pub type TetrisEngine = Engine<TetrisAction>;

pub fn new_tetris(seed: u64) -> TetrisEngine {
    Engine::new(Game::new(seed), vec![Box::new(GameActive)], apply_tetris)
}
