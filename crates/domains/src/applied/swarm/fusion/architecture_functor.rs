//! Cross-functor: DistributedFusion → the existing `FusionArchitecture`
//! enum.
//!
//! `FusionArchitecture` (applied/sensor_fusion/fusion/architecture.rs)
//! is a bare taxonomy enum with no `Category`. Rather than rewrite it,
//! this file re-exposes it as a *discrete* category — identity-only
//! morphisms, exactly the technique of
//! `formal::systems::mape_k::pipeline_step_functor` — and carries every
//! `DistributedFusion` concept to the architecture class it instantiates.
//!
//! # The mapping (Liggins et al. 2008 Ch. 2; Castanedo 2013 — the
//! architecture enum's own sources)
//!
//! Every concept of this ontology describes fusion where each peer runs
//! its own filter and results are combined across the network — the
//! `Distributed` architecture class, so the whole ontology collapses
//! onto that one object (each collapse documented):
//!
//! | DistributedFusion | FusionArchitecture | Why |
//! |---|---|---|
//! | `NetworkFusionArchitecture` | `Distributed` | The abstract parent of exactly the distributed patterns |
//! | `DistributedKalmanFilter` | `Distributed` | Local filters + consensus is the distributed pattern par excellence |
//! | `CiOverNetwork` | `Distributed` | CI exists precisely because distributed fusion loses cross-correlations |
//! | `ConsensusEstimate`, `InnovationExchange` | `Distributed` | The product and the step of distributed fusion (documented collapse) |
//! | `DataIncest` | `Distributed` | The failure mode is intrinsic to distributed information flow on cycles (documented collapse) |
//!
//! Because every object lands on `Distributed`, every source morphism
//! maps to the identity on `Distributed` — the functor factors through
//! the one-object subcategory, which is exactly what "this whole
//! ontology is about the distributed architecture class" means.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Category, Functor};

use super::ontology::{DistributedFusionCategory, DistributedFusionConcept};
use crate::applied::sensor_fusion::fusion::architecture::FusionArchitecture;

/// The 5-variant `FusionArchitecture` enum, re-exposed as a category so
/// it can be a `Functor::Target`. It is a *discrete* category — no
/// morphisms beyond identities — because the enum declares no edges
/// between its variants. Same wrapper technique as
/// `PipelineStepCategory` in `mape_k::pipeline_step_functor`.
pub struct FusionArchitectureCategory;

/// Identity-only wrapper morphism for `FusionArchitecture`.
///
/// Fields are private so callers cannot construct non-identity morphisms
/// that would break the discrete-category contract. Construct via
/// [`FusionArchitectureMorphism::identity`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FusionArchitectureMorphism {
    from: FusionArchitecture,
    to: FusionArchitecture,
}

impl FusionArchitectureMorphism {
    /// The only public constructor — the identity morphism on `arch`.
    pub fn identity(arch: FusionArchitecture) -> Self {
        Self {
            from: arch,
            to: arch,
        }
    }
}

/// The single relation kind of the discrete wrapper: every morphism is
/// an identity tagged `Identity`. Per OBO-RO (Smith 2005) every arrow
/// carries a relation-kind tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FusionArchitectureRelationKind {
    /// The only morphisms of a discrete category.
    Identity,
}

impl pr4xis::category::Arrow for FusionArchitectureMorphism {
    type Object = FusionArchitecture;
    type Kind = FusionArchitectureRelationKind;

    fn source(&self) -> FusionArchitecture {
        self.from
    }
    fn target(&self) -> FusionArchitecture {
        self.to
    }
    fn kind(&self) -> FusionArchitectureRelationKind {
        FusionArchitectureRelationKind::Identity
    }
    fn meta(&self) -> pr4xis::ontology::meta::Provenance {
        use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};
        Provenance {
            name: OntologyName::new_static("FusionArchitectureMorphism"),
            description: Label::new_static(
                "identity morphism in the discrete FusionArchitecture category; carries the Identity kind tag per OBO-RO",
            ),
            citation: Citation::parse_static(
                "Liggins et al. (2008) Handbook of Multisensor Data Fusion Ch. 2; Castanedo (2013); Smith et al. (2005) Genome Biology 6:R46 OBO-RO",
            ),
            module_path: ModulePath::new_static(module_path!()),
        }
    }
}

impl Category for FusionArchitectureCategory {
    type Object = FusionArchitecture;
    type Morphism = FusionArchitectureMorphism;

    fn identity(obj: &FusionArchitecture) -> FusionArchitectureMorphism {
        FusionArchitectureMorphism::identity(*obj)
    }

    fn compose(
        f: &FusionArchitectureMorphism,
        g: &FusionArchitectureMorphism,
    ) -> Option<FusionArchitectureMorphism> {
        // Discrete category: the only valid composition is identity
        // after identity on the same object.
        if f.from == f.to && g.from == g.to && f.to == g.from {
            Some(FusionArchitectureMorphism::identity(f.from))
        } else {
            None
        }
    }

    fn morphisms() -> Vec<FusionArchitectureMorphism> {
        use pr4xis::category::FinitelyGenerated;
        FusionArchitecture::variants()
            .into_iter()
            .map(FusionArchitectureMorphism::identity)
            .collect()
    }
}

impl pr4xis::category::NamedCategory for FusionArchitectureCategory {
    fn ontology_name() -> pr4xis::ontology::meta::OntologyName {
        pr4xis::ontology::meta::OntologyName::new_static("FusionArchitecture")
    }
}

/// Maps every `DistributedFusion` concept to the architecture class it
/// instantiates — `Distributed`, for all of them (see the module table).
pub struct DistributedFusionToFusionArchitecture;

impl Functor for DistributedFusionToFusionArchitecture {
    type Source = DistributedFusionCategory;
    type Target = FusionArchitectureCategory;

    fn map_object(obj: &DistributedFusionConcept) -> FusionArchitecture {
        use DistributedFusionConcept as D;
        match obj {
            // Every concept describes per-peer filters whose results are
            // combined across the network — the Distributed class
            // (Liggins et al. 2008 Ch. 2; documented collapses in the
            // module table).
            D::NetworkFusionArchitecture
            | D::DistributedKalmanFilter
            | D::CiOverNetwork
            | D::ConsensusEstimate
            | D::InnovationExchange
            | D::DataIncest => FusionArchitecture::Distributed,
        }
    }

    fn map_morphism(
        m: &<DistributedFusionCategory as Category>::Morphism,
    ) -> FusionArchitectureMorphism {
        use pr4xis::category::Arrow;
        // Every object maps to Distributed, so every morphism image is
        // the identity on Distributed — the target is discrete and the
        // functor factors through its one-object subcategory.
        FusionArchitectureCategory::identity(&Self::map_object(&m.source()))
    }
}
pr4xis::register_functor!(
    DistributedFusionToFusionArchitecture,
    "Liggins et al. (2008) Handbook of Multisensor Data Fusion Ch. 2; Castanedo (2013)"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::{assert_category_laws, assert_functor_laws};

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn wrapper_category_laws() {
        assert_category_laws::<FusionArchitectureCategory>();
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn distributed_fusion_to_architecture_functor_laws() {
        assert_functor_laws::<DistributedFusionToFusionArchitecture>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn every_concept_is_distributed() {
        for c in DistributedFusionConcept::variants() {
            assert_eq!(
                DistributedFusionToFusionArchitecture::map_object(&c),
                FusionArchitecture::Distributed,
                "{c:?} should land on the Distributed class"
            );
        }
    }
}
