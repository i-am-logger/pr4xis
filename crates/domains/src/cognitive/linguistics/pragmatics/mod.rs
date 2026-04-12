pub mod discourse;
pub mod drt_dialogue_functor;
pub mod generation;
pub mod nlg;
pub mod realize;
pub mod reference;
pub mod response;
pub mod speech_act;

pub use discourse::{Discourse, Turn};
pub use response::{ResponseCategory, ResponseFrame};
pub use speech_act::{Intent, SpeechAct};

#[cfg(test)]
mod tests;
