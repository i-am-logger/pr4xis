pub mod axiom;
pub mod category;
pub mod entity;
pub mod functor;
pub mod morphism;
pub mod relationship;
pub mod transformation;
pub mod validate;

pub use axiom::{Axiom, FullyConnected, NoDeadStates};
pub use category::Category;
pub use entity::Entity;
pub use functor::Functor;
pub use morphism::{Morphism, compose_all, direct_morphisms};
pub use relationship::Relationship;
pub use transformation::NaturalTransformation;
