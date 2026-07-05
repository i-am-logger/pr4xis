//! Functor: SmartElement → ConstitutiveProtocol.
//!
//! The constitutive reading of a smart element: to sign the estimates it
//! gossips, an element acts under an `Identity` (Lamport 1979 — to be able
//! to sign is what an identity is); its autonomic manager is the
//! `AuthoringAgency` that authors those signed events; and its self-
//! protection — the exclusion of a caught equivocator — is `Slashing`, the
//! structural penalty whose proof is a constructable artifact (Buterin &
//! Griffith 2017; Li et al. 2004 SUNDR).
//!
//! Honesty note: the element's self-description (`LocalOntology`, `Teds`)
//! is mapped to `DeviceId` — the external self-describing handle of an
//! identity — NOT to `Constitution` / `ChannelManifest`, which would
//! falsely claim the element founds a channel. It does not.
//!
//! # Object mapping (each arm documented)
//!
//! | SmartElement | ConstitutiveProtocol | Why |
//! |---|---|---|
//! | `SmartElement`, `SmartSensor`, `SmartDriver` | `Identity` | The element signs its estimates — to sign is to be an identity (Lamport 1979) |
//! | `AutonomicManager` | `AuthoringAgency` | It authors the element's signed events (documented) |
//! | `SelfProtection` | `Slashing` | Exclusion after constructable proof is the slashing penalty (Buterin & Griffith 2017) |
//! | `LocalOntology`, `Teds` | `DeviceId` | Honest collapse: the element's external self-describing handle — NOT the Constitution/ChannelManifest |
//! | `ManagedElement`, `Transducer`, `Ncap`, `SelfStarProperty`, `SelfConfiguration`, `SelfHealing`, `SelfOptimization` | `PraxisEvent` | The element's lived activity under its identity (documented umbrella collapse) |
//!
//! # Morphism-kind mapping
//!
//! The four canonical Relations-ontology kinds map to their namesakes; the
//! five custom edges collapse into two constitutive buckets:
//!
//! - `Exhibits`, `Operates`, `Manages` → `Authors` (`Exhibits` lands on
//!   the real `Identity → PraxisEvent Authors` edge under the object map);
//! - `Carries`, `DescribedBy` → `NamedBy` (`Carries` lands on the real
//!   `Identity → DeviceId NamedBy` edge under the object map).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Functor};

use super::ontology::{
    SmartElementCategory, SmartElementConcept, SmartElementRelation, SmartElementRelationKind,
};
use crate::social::protocols::constitutive::ontology::{
    ConstitutiveProtocolCategory, ConstitutiveProtocolConcept, ConstitutiveProtocolRelation,
    ConstitutiveProtocolRelationKind,
};

/// Maps each smart-element concept to its constitutive-protocol reading.
pub struct SmartElementToConstitutiveProtocol;

impl Functor for SmartElementToConstitutiveProtocol {
    type Source = SmartElementCategory;
    type Target = ConstitutiveProtocolCategory;

    fn map_object(obj: &SmartElementConcept) -> ConstitutiveProtocolConcept {
        use ConstitutiveProtocolConcept as P;
        use SmartElementConcept as C;
        match obj {
            // The element signs its estimates — to sign is to be an
            // identity (Lamport 1979).
            C::SmartElement | C::SmartSensor | C::SmartDriver => P::Identity,
            // The manager authors the element's signed events.
            C::AutonomicManager => P::AuthoringAgency,
            // Exclusion after constructable proof is the slashing penalty.
            C::SelfProtection => P::Slashing,
            // Honest collapse: the external self-describing handle — NOT
            // the Constitution/ChannelManifest (which the element does not
            // found).
            C::LocalOntology | C::Teds => P::DeviceId,
            // The element's lived activity under its identity (documented
            // umbrella collapse).
            C::ManagedElement
            | C::Transducer
            | C::Ncap
            | C::SelfStarProperty
            | C::SelfConfiguration
            | C::SelfHealing
            | C::SelfOptimization => P::PraxisEvent,
        }
    }

    fn map_morphism(m: &SmartElementRelation) -> ConstitutiveProtocolRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            SmartElementRelationKind::Identity => {
                return ConstitutiveProtocolCategory::identity(&from);
            }
            // The four canonical Relations-ontology kinds map to namesakes
            // (Smith 2005 OBO-RO).
            SmartElementRelationKind::Subsumption => ConstitutiveProtocolRelationKind::Subsumption,
            SmartElementRelationKind::Parthood => ConstitutiveProtocolRelationKind::Parthood,
            SmartElementRelationKind::Causation => ConstitutiveProtocolRelationKind::Causation,
            SmartElementRelationKind::Opposition => ConstitutiveProtocolRelationKind::Opposition,
            // Exhibiting behaviour, operating a transducer, and managing the
            // element are the identity authoring signed events — Exhibits
            // lands on the real Identity -> PraxisEvent Authors edge.
            SmartElementRelationKind::Exhibits
            | SmartElementRelationKind::Operates
            | SmartElementRelationKind::Manages => ConstitutiveProtocolRelationKind::Authors,
            // Carrying / describing the element's self-description is the
            // external-naming arrow — Carries lands on the real
            // Identity -> DeviceId NamedBy edge.
            SmartElementRelationKind::Carries | SmartElementRelationKind::DescribedBy => {
                ConstitutiveProtocolRelationKind::NamedBy
            }
        };
        ConstitutiveProtocolRelation { from, to, kind }
    }
}
pr4xis::register_functor!(
    SmartElementToConstitutiveProtocol,
    "Lamport (1979) SRI CSL-98; Buterin & Griffith (2017) arXiv:1710.09437"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn smart_element_to_constitutive_functor_laws() {
        assert_functor_laws::<SmartElementToConstitutiveProtocol>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn elements_are_identities_and_protection_is_slashing() {
        use ConstitutiveProtocolConcept as P;
        use SmartElementConcept as C;
        for c in [C::SmartElement, C::SmartSensor, C::SmartDriver] {
            assert_eq!(
                SmartElementToConstitutiveProtocol::map_object(&c),
                P::Identity
            );
        }
        assert_eq!(
            SmartElementToConstitutiveProtocol::map_object(&C::SelfProtection),
            P::Slashing
        );
    }

    /// Honest collapse: the self-description is a DeviceId, never the
    /// Constitution or ChannelManifest.
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn self_description_is_device_id_not_constitution() {
        use ConstitutiveProtocolConcept as P;
        use SmartElementConcept as C;
        for c in [C::LocalOntology, C::Teds] {
            let image = SmartElementToConstitutiveProtocol::map_object(&c);
            assert_eq!(image, P::DeviceId);
            assert_ne!(image, P::Constitution);
            assert_ne!(image, P::ChannelManifest);
        }
    }

    /// The `Carries` edge (`SmartElement → LocalOntology`) lands on the
    /// real constitutive edge `Identity → DeviceId NamedBy`.
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn carries_lands_on_declared_named_by_edge() {
        let source_edge = SmartElementCategory::morphisms()
            .into_iter()
            .find(|m| {
                m.source() == SmartElementConcept::SmartElement
                    && m.target() == SmartElementConcept::LocalOntology
                    && m.kind() == SmartElementRelationKind::Carries
            })
            .expect("the SmartElement carries LocalOntology edge is declared");
        let image = SmartElementToConstitutiveProtocol::map_morphism(&source_edge);
        assert_eq!(image.from, ConstitutiveProtocolConcept::Identity);
        assert_eq!(image.to, ConstitutiveProtocolConcept::DeviceId);
        assert_eq!(image.kind, ConstitutiveProtocolRelationKind::NamedBy);
        let declared = ConstitutiveProtocolCategory::morphisms()
            .into_iter()
            .any(|m| {
                m.source() == ConstitutiveProtocolConcept::Identity
                    && m.target() == ConstitutiveProtocolConcept::DeviceId
                    && m.kind() == ConstitutiveProtocolRelationKind::NamedBy
            });
        assert!(declared, "Identity -> DeviceId NamedBy is a real edge");
    }
}
