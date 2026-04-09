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

#[cfg(test)]
pub(crate) use argument::*;
#[cfg(test)]
pub(crate) use authority::*;
#[cfg(test)]
pub(crate) use decision::*;
#[cfg(test)]
pub(crate) use element::*;
#[cfg(test)]
pub(crate) use engine::*;
#[cfg(test)]
pub(crate) use entity::*;
#[cfg(test)]
pub(crate) use fact::*;
#[cfg(test)]
pub(crate) use finding::*;
#[cfg(test)]
pub(crate) use lifecycle::*;
#[cfg(test)]
pub(crate) use ontology::*;
#[cfg(test)]
pub(crate) use rule::*;
#[cfg(test)]
pub(crate) use source::*;

#[cfg(test)]
mod tests;
