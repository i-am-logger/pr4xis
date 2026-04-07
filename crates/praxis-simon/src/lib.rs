mod color;
mod game;
mod input;

pub use color::SimonColor;
pub use game::{Game, GameState, RoundResult};
pub use input::Input;

pub mod engine;
pub mod ontology;

#[cfg(test)]
mod tests;
