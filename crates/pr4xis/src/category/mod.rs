#[macro_use]
pub mod macros;
pub mod adjunction;
pub mod applicative;
#[allow(clippy::module_inception)]
pub mod category;
pub mod comonad;
pub mod entity;
pub mod free;
pub mod functor;
pub mod galois;
pub mod invariants;
pub mod kleisli;
pub mod monad;
pub mod monoid;
pub mod morphism;
pub mod reader;
pub mod relationship;
pub mod semigroup;
pub mod state;
pub mod traced;
pub mod transformation;
pub mod validate;
pub mod yoneda;

pub use adjunction::Adjunction;
pub use applicative::Ap;
pub use category::Category;
pub use comonad::{Cofree, Focused};
pub use entity::Entity;
pub use free::Chain;
pub use functor::Functor;
pub use galois::GaloisConnection;
pub use invariants::{FullyConnected, NoDeadStates};
pub use kleisli::KleisliMorphism;
pub use monad::Writer;
pub use monoid::Monoid;
pub use morphism::{Morphism, compose_all, direct_morphisms};
#[doc(hidden)]
pub use pr4xis_derive::Entity;
pub use reader::Reader;
pub use relationship::Relationship;
pub use semigroup::{NonEmpty, Semigroup};
pub use state::State;
pub use transformation::NaturalTransformation;
pub use yoneda::{CoYoneda, Yoneda, YonedaProfile};
