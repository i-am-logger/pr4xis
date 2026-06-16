//! # pr4xis-runtime — load a `.prx` ontology as data, interpret it
//!
//! The data-driven half of the North Star "praxis = `.prx` files + a runtime".
//! Where `pr4xis` is the COMPILE-TIME ontology model (the `ontology!` macro and
//! the `Category` / `Functor` / `FreeCategory` traits) and `pr4xis-domains` holds
//! the compiled-in ontologies, this crate is the RUNTIME: it loads a `.prx` —
//! the serialized ontology — as DATA and interprets it, without the ontology
//! being compiled in.
//!
//! ## The load/store cycle is the free ⊣ forgetful adjunction
//!
//! - A `.prx` is a free-category presentation: generators + relations +
//!   connections-as-action-on-generators — open-world-generic by construction.
//! - **Load** deserializes the bytes into that free form — always possible, no
//!   compiled types required.
//! - **Rebind** uses each connection's serialized action-on-generators as the
//!   interpretation a `FreeExtension` needs to map the free generators into a
//!   closed-world `Category`: partially (unknown generators stay free —
//!   graceful open-world) and faithfully (a generator rebinds to a concept only
//!   when their content [addresses](crate::address) agree).
//!
//! ## The runtime grounds only the hash
//!
//! Everything the runtime knows about the `.prx` FORMAT it learns from the
//! meta-`.prx` (the ontology that defines the format). The one thing it cannot
//! learn by reference — because it is what reference MEANS — is the
//! content-address primitive ([`address`]). Identity bottoms out there.
//!
//! ## Status
//!
//! Scaffolding the runtime kernel. First primitive: [`address`] (the hash
//! ground). The loader (bytes → free category), the rebind (`FreeExtension` →
//! closed world), the canonical codec, and the meta-`.prx` interpreter land on
//! top of it, per the consolidated build spine.

pub mod address;
pub mod apply;
pub mod archive;
pub mod codec;
pub mod connection;
pub mod definition;
#[cfg(feature = "emit")]
pub mod emit;
pub mod grounding;
pub mod load;
pub mod meta;
#[cfg(feature = "emit")]
pub mod ontology;
pub mod rebind;
pub mod recursive_address;
