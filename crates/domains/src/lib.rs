#![cfg_attr(not(feature = "std"), no_std)]

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
