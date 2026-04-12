#[macro_use]
pub mod macros;
pub mod adjunction;
#[allow(clippy::module_inception)]
pub mod category;
pub mod entity;
pub mod functor;
pub mod invariants;
pub mod monad;
pub mod monoid;
pub mod morphism;
pub mod relationship;
pub mod traced;
pub mod transformation;
pub mod validate;

pub use adjunction::Adjunction;
pub use category::Category;
pub use entity::Entity;

// Re-export derive macros — users write `#[derive(Entity)]` and it just works.
// The derive macro and the trait share the name but different namespaces.
pub use functor::Functor;
pub use invariants::{FullyConnected, NoDeadStates};
pub use monad::Writer;
pub use monoid::Monoid;
pub use morphism::{Morphism, compose_all, direct_morphisms};
#[doc(hidden)]
pub use pr4xis_derive::Entity;
pub use relationship::Relationship;
pub use transformation::NaturalTransformation;
