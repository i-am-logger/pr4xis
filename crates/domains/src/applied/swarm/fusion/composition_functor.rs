//! Cross-functor: DistributedFusion → the existing `CompositionStrategy`
//! enum.
//!
//! `CompositionStrategy` (applied/sensor_fusion/fusion/composition.rs)
//! is a bare enum with no `Category`, so — like
//! `mape_k::pipeline_step_functor` — this file wraps it in a category
//! beside the functor. One deliberate difference from that exemplar:
//! the wrapper here is the **indiscrete** category (exactly one
//! morphism between each ordered pair of objects — the dual of the
//! discrete category of Mac Lane 1971 CWM Ch. I §2). A *discrete*
//! wrapper cannot work for this functor: the `DistributedFusion` source
//! category is connected (e.g. the `Subsumption` edge
//! `CiOverNetwork → NetworkFusionArchitecture`), so any lawful functor
//! into a discrete category would have to collapse ALL concepts onto a
//! single strategy — destroying exactly the CI-vs-information-fusion
//! distinction this functor exists to state. The indiscrete wrapper's
//! unique arrows carry no relational claims (its hom-sets are
//! singletons), so it adds no assertions about the strategies while
//! letting the object mapping be lawful.
//!
//! # The mapping (Julier & Uhlmann 1997; Mutambara 1998; Carlson 1990 —
//! the composition enum's own sources)
//!
//! | DistributedFusion | CompositionStrategy | Why |
//! |---|---|---|
//! | `CiOverNetwork` | `CovarianceIntersection` | Concept for concept (Julier & Uhlmann 1997) |
//! | `DistributedKalmanFilter` | `InformationFusion` | Consensus on information contributions is additive information fusion (Mutambara 1998 Ch. 3) |
//! | `InnovationExchange` | `InformationFusion` | Exchanging information contributions IS the additive step (documented collapse) |
//! | `ConsensusEstimate` | `InformationFusion` | The agreed estimate is the product of the additive information combination (documented collapse) |
//! | `DataIncest` | `InformationFusion` | Incest is the failure mode OF assuming independence in additive information fusion (documented collapse) |
//! | `NetworkFusionArchitecture` | `InformationFusion` | The abstract parent collapses onto the additive default the other concepts refine or repair (documented collapse) |

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Category, Functor};

use super::ontology::{DistributedFusionCategory, DistributedFusionConcept};
use crate::applied::sensor_fusion::fusion::composition::CompositionStrategy;

/// The 3-variant `CompositionStrategy` enum, re-exposed as the
/// *indiscrete* category (one morphism between each ordered pair — Mac
/// Lane 1971 CWM Ch. I §2, the dual of the discrete case) so it can be
/// a `Functor::Target` for a connected source without collapsing every
/// object.
pub struct CompositionStrategyCategory;

/// The unique wrapper morphism between an ordered pair of strategies.
/// In an indiscrete category every hom-set is a singleton, so the pair
/// of endpoints determines the morphism; the self-morphism is the
/// identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompositionStrategyMorphism {
    from: CompositionStrategy,
    to: CompositionStrategy,
}

impl CompositionStrategyMorphism {
    /// The unique morphism from `from` to `to`.
    pub fn between(from: CompositionStrategy, to: CompositionStrategy) -> Self {
        Self { from, to }
    }
}

/// The single relation kind of the indiscrete wrapper: every hom-set is
/// a singleton, so the one tag says exactly that — the arrow carries no
/// relational claim beyond connectedness of the wrapper. Per OBO-RO
/// (Smith 2005) every arrow carries a relation-kind tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompositionStrategyRelationKind {
    /// The unique arrow of an indiscrete hom-set.
    Indiscrete,
}

impl pr4xis::category::Arrow for CompositionStrategyMorphism {
    type Object = CompositionStrategy;
    type Kind = CompositionStrategyRelationKind;

    fn source(&self) -> CompositionStrategy {
        self.from
    }
    fn target(&self) -> CompositionStrategy {
        self.to
    }
    fn kind(&self) -> CompositionStrategyRelationKind {
        CompositionStrategyRelationKind::Indiscrete
    }
    fn meta(&self) -> pr4xis::ontology::meta::Provenance {
        use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};
        Provenance {
            name: OntologyName::new_static("CompositionStrategyMorphism"),
            description: Label::new_static(
                "unique morphism of the indiscrete CompositionStrategy category (singleton hom-sets; no relational claim); carries the Indiscrete kind tag per OBO-RO",
            ),
            citation: Citation::parse_static(
                "Mac Lane (1971) Categories for the Working Mathematician Ch. I; Julier & Uhlmann (1997); Mutambara (1998); Carlson (1990); Smith et al. (2005) Genome Biology 6:R46 OBO-RO",
            ),
            module_path: ModulePath::new_static(module_path!()),
        }
    }
}

impl Category for CompositionStrategyCategory {
    type Object = CompositionStrategy;
    type Morphism = CompositionStrategyMorphism;

    fn identity(obj: &CompositionStrategy) -> CompositionStrategyMorphism {
        CompositionStrategyMorphism::between(*obj, *obj)
    }

    fn compose(
        f: &CompositionStrategyMorphism,
        g: &CompositionStrategyMorphism,
    ) -> Option<CompositionStrategyMorphism> {
        // Indiscrete category: composable arrows compose into the unique
        // arrow between the outer endpoints.
        if f.to == g.from {
            Some(CompositionStrategyMorphism::between(f.from, g.to))
        } else {
            None
        }
    }

    fn morphisms() -> Vec<CompositionStrategyMorphism> {
        use pr4xis::category::FinitelyGenerated;
        let variants = CompositionStrategy::variants();
        let mut all = Vec::new();
        for from in &variants {
            for to in &variants {
                all.push(CompositionStrategyMorphism::between(*from, *to));
            }
        }
        all
    }
}

impl pr4xis::category::NamedCategory for CompositionStrategyCategory {
    fn ontology_name() -> pr4xis::ontology::meta::OntologyName {
        pr4xis::ontology::meta::OntologyName::new_static("CompositionStrategy")
    }
}

/// Maps each `DistributedFusion` concept to the composition strategy it
/// uses (or refines, or fails against — see the module table).
pub struct DistributedFusionToCompositionStrategy;

impl Functor for DistributedFusionToCompositionStrategy {
    type Source = DistributedFusionCategory;
    type Target = CompositionStrategyCategory;

    fn map_object(obj: &DistributedFusionConcept) -> CompositionStrategy {
        use DistributedFusionConcept as D;
        match obj {
            // Concept for concept (Julier & Uhlmann 1997).
            D::CiOverNetwork => CompositionStrategy::CovarianceIntersection,
            // Consensus on information contributions is additive
            // information fusion; the exchange, its product, its failure
            // mode, and the abstract parent all collapse onto that
            // strategy (documented in the module table).
            D::DistributedKalmanFilter
            | D::InnovationExchange
            | D::ConsensusEstimate
            | D::DataIncest
            | D::NetworkFusionArchitecture => CompositionStrategy::InformationFusion,
        }
    }

    fn map_morphism(
        m: &<DistributedFusionCategory as Category>::Morphism,
    ) -> CompositionStrategyMorphism {
        use pr4xis::category::Arrow;
        // The indiscrete target has exactly one morphism between any
        // ordered pair, so every source morphism maps to the unique
        // arrow between its endpoint images.
        CompositionStrategyMorphism::between(
            Self::map_object(&m.source()),
            Self::map_object(&m.target()),
        )
    }
}
pr4xis::register_functor!(
    DistributedFusionToCompositionStrategy,
    "Julier & Uhlmann (1997); Mutambara (1998); Carlson (1990)"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::{assert_category_laws, assert_functor_laws};

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn wrapper_category_laws() {
        assert_category_laws::<CompositionStrategyCategory>();
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn distributed_fusion_to_composition_functor_laws() {
        assert_functor_laws::<DistributedFusionToCompositionStrategy>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn ci_and_dkf_separate() {
        use DistributedFusionConcept as D;
        assert_eq!(
            DistributedFusionToCompositionStrategy::map_object(&D::CiOverNetwork),
            CompositionStrategy::CovarianceIntersection
        );
        assert_eq!(
            DistributedFusionToCompositionStrategy::map_object(&D::DistributedKalmanFilter),
            CompositionStrategy::InformationFusion
        );
    }

    /// The functor's object map agrees with the target enum's own
    /// consistency quality: the concept this ontology marks consistent
    /// under inter-peer correlation lands on the strategy composition.rs
    /// marks consistent under unknown correlation, and likewise for the
    /// inconsistent pair — the two encodings corroborate each other.
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn consistency_qualities_agree_across_the_functor() {
        use crate::applied::sensor_fusion::fusion::composition::ConsistentUnderUnknownCorrelation;
        use crate::applied::swarm::fusion::ontology::ConsistentUnderInterPeerCorrelation;
        use DistributedFusionConcept as D;
        use pr4xis::ontology::Quality;

        for concept in [D::CiOverNetwork, D::DistributedKalmanFilter] {
            let source_value = ConsistentUnderInterPeerCorrelation.get(&concept);
            let image = DistributedFusionToCompositionStrategy::map_object(&concept);
            let target_value = ConsistentUnderUnknownCorrelation.get(&image);
            assert_eq!(source_value, target_value, "{concept:?}");
        }
    }
}
