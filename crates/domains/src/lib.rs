#![cfg_attr(not(feature = "std"), no_std)]
// The XHTML 1.0 schema codegen produces a mutually-recursive web of
// `Vec<*ElementTypeContent>` enums whose auto-derived `Send`/`Sync`
// trait resolution exceeds rustc's default 128-deep recursion budget
// during rustdoc analysis. (Cargo build/test pass at default depth;
// only rustdoc's deeper `Send`/`Sync` check exceeds it.) The depth
// is intrinsic to the XHTML inline content model — every inline
// element can contain every other inline element per W3C XHTML 1.0
// §4.2 — so capturing it faithfully requires the cycle. Raising the
// budget is the minimum-information-loss fix; the alternatives
// (Box<dyn Trait>, doc(hidden) on the whole module) either change
// semantics or hide the grammar from documentation.
#![recursion_limit = "1024"]

extern crate alloc;

// Academic discipline hierarchy (DOLCE-aligned)
pub mod applied; // Process — engineering, navigation, sensors, robotics
pub mod cognitive; // MentalObject — linguistics, cognition
pub mod formal; // AbstractObject — math, information, systems, computation
pub mod natural; // PhysicalEndurant — physics, biology, chemistry, geodesy
pub mod social; // SocialObject — governance, games, protocols, standards

// Manual registrations for ontologies with hand-written impls.
// Emits linkme distributed_slice entries into pr4xis::ontology::VOCABULARIES.
mod manual_registrations;
