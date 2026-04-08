pub mod color;
pub mod engine;
pub mod game;
pub mod input;
pub mod ontology;

pub use color::SimonColor;
pub use game::{Game, GameState, RoundResult};
pub use input::Input;

#[cfg(test)]
mod tests;
pub use engine::*;
pub use ontology::*;
