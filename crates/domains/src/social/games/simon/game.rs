#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::color::SimonColor;
use super::input::Input;

/// The state of a Simon Says game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameState {
    /// Simon is showing the sequence. Player watches.
    Showing,
    /// Player is inputting. `inputs_so_far` tracks progress.
    Inputting { inputs_so_far: usize },
    /// Round completed successfully.
    RoundComplete,
    /// Game over — wrong input.
    GameOver {
        round: usize,
        expected: SimonColor,
        got: SimonColor,
        position: usize,
    },
}

/// Result of a single input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoundResult {
    /// Correct so far, more inputs needed.
    Correct { remaining: usize },
    /// Sequence complete for this round.
    RoundComplete { round: usize },
    /// Wrong input — game over.
    Wrong {
        position: usize,
        expected: SimonColor,
        got: SimonColor,
    },
    /// Input rejected — not in inputting state.
    InvalidState,
}

/// A Simon Says game. The sequence grows each round.
/// The ontology: only correct inputs advance the game.
#[derive(Debug, Clone, PartialEq)]
pub struct Game {
    sequence: Vec<SimonColor>,
    state: GameState,
    round: usize,
    seed: u64,
}

impl Game {
    /// Start a new game with a seed for reproducibility.
    pub fn new(seed: u64) -> Self {
        let mut game = Self {
            sequence: Vec::new(),
            state: GameState::Showing,
            round: 0,
            seed,
        };
        game.extend_sequence();
        game
    }

    /// The current sequence the player must reproduce.
    pub fn sequence(&self) -> &[SimonColor] {
        &self.sequence
    }

    /// Current game state.
    pub fn state(&self) -> &GameState {
        &self.state
    }

    /// Current round (1-indexed).
    pub fn round(&self) -> usize {
        self.round
    }

    /// Sequence length for current round.
    pub fn sequence_length(&self) -> usize {
        self.sequence.len()
    }

    /// Transition from Showing to Inputting.
    pub fn start_input(&mut self) -> Result<(), &'static str> {
        match self.state {
            GameState::Showing => {
                self.state = GameState::Inputting { inputs_so_far: 0 };
                Ok(())
            }
            _ => Err("can only start input from showing state"),
        }
    }

    /// Player presses a color. Enforces sequence correctness.
    pub fn input(&mut self, color: SimonColor) -> RoundResult {
        let position = match self.state {
            GameState::Inputting { inputs_so_far } => inputs_so_far,
            _ => return RoundResult::InvalidState,
        };

        let expected = self.sequence[position];

        if color != expected {
            self.state = GameState::GameOver {
                round: self.round,
                expected,
                got: color,
                position,
            };
            return RoundResult::Wrong {
                position,
                expected,
                got: color,
            };
        }

        let new_position = position + 1;

        if new_position == self.sequence.len() {
            // Round complete
            self.state = GameState::RoundComplete;
            RoundResult::RoundComplete { round: self.round }
        } else {
            self.state = GameState::Inputting {
                inputs_so_far: new_position,
            };
            RoundResult::Correct {
                remaining: self.sequence.len() - new_position,
            }
        }
    }

    /// Advance to next round (extends sequence, resets to showing).
    pub fn next_round(&mut self) -> Result<(), &'static str> {
        match self.state {
            GameState::RoundComplete => {
                self.extend_sequence();
                self.state = GameState::Showing;
                Ok(())
            }
            _ => Err("can only advance from round complete state"),
        }
    }

    /// Replay the correct sequence for current round. Returns inputs needed.
    pub fn replay_correct(&self) -> Vec<Input> {
        self.sequence
            .iter()
            .enumerate()
            .map(|(i, &c)| Input::new(c, i))
            .collect()
    }

    fn extend_sequence(&mut self) {
        self.round += 1;
        // Deterministic pseudo-random based on seed + round
        let new_seed = self
            .seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(self.round as u64);
        self.sequence.push(SimonColor::from_seed(new_seed));
    }
}
