//! Version adjunction — the version-polymorphic structure shared by
//! every artifact published in several versions (XSD, XML, PDF / ISO
//! 32000, the USLM User Guide, statutes, citations, ...).
//!
//! See [`ontology`] for the `Versioning` ontology, the adjoint
//! endofunctors `LocalizeVersion ⊣ AbstractVersion`, and the
//! instance-level generic [`VersionedArtifact`](ontology::VersionedArtifact).
//!
//! ## Citation
//!
//! - **Roddick (1995)** schema versioning; **Noy & Klein (2004)**
//!   ontology evolution; **Mimram & Di Giusto (2013)** categorical
//!   patches; **Mac Lane (1998) §IV.1**; **Bancilhon & Spyratos
//!   (1981)** constant complement; **Grothendieck (1971) SGA1
//!   Exposé VI** fibered categories.

pub mod ontology;

#[cfg(test)]
mod tests;
