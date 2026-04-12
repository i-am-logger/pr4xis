pub mod adjunction;
#[allow(clippy::module_inception)]
pub mod category;
pub mod entity;
pub mod functor;
pub mod invariants;
pub mod morphism;
pub mod relationship;
pub mod traced;
pub mod transformation;
pub mod validate;

pub use adjunction::Adjunction;
pub use category::Category;
pub use entity::Entity;
pub use functor::Functor;
pub use invariants::{FullyConnected, NoDeadStates};
pub use morphism::{Morphism, compose_all, direct_morphisms};
pub use relationship::Relationship;
pub use transformation::NaturalTransformation;
