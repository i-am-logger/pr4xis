pub mod argument;
pub mod authority;
pub mod decision;
pub mod element;
pub mod engine;
pub mod entity;
pub mod fact;
pub mod finding;
pub mod lifecycle;
pub mod ontology;
pub mod rule;
pub mod source;

pub use engine::{LegalAction, LegalEngine, new_case};
pub use entity::Entity;
pub use lifecycle::{Case, CaseAction, CasePhase, PhaseTag};

pub(crate) use argument::*;
pub(crate) use authority::*;
pub(crate) use decision::*;
pub(crate) use element::*;
pub(crate) use engine::*;
pub(crate) use entity::*;
pub(crate) use fact::*;
pub(crate) use finding::*;
pub(crate) use lifecycle::*;
pub(crate) use ontology::*;
pub(crate) use rule::*;
pub(crate) use source::*;

#[cfg(test)]
mod tests;
