use pr4xis::category::{Category, Entity, Relationship};
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Terrain feature types for terrain-relative navigation.
///
/// Source: Goldstein (1987), "Terrain Aided Navigation"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerrainFeature {
    /// Local maximum in elevation.
    Peak,
    /// Local minimum in elevation.
    Valley,
    /// Line of local maxima connecting peaks.
    Ridge,
    /// Point where two ridges meet (col/pass).
    Saddle,
}

impl Entity for TerrainFeature {
    fn variants() -> Vec<Self> {
        vec![Self::Peak, Self::Valley, Self::Ridge, Self::Saddle]
    }
}

/// Topological relationship between terrain features.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TerrainRelation {
    pub from: TerrainFeature,
    pub to: TerrainFeature,
}

impl Relationship for TerrainRelation {
    type Object = TerrainFeature;
    fn source(&self) -> TerrainFeature {
        self.from
    }
    fn target(&self) -> TerrainFeature {
        self.to
    }
}

/// Category for terrain feature relationships.
///
/// Ridges connect peaks through saddles; valleys lie between ridges.
/// All features can relate to all other features in a DEM.
pub struct TerrainCategory;

impl Category for TerrainCategory {
    type Object = TerrainFeature;
    type Morphism = TerrainRelation;

    fn identity(obj: &TerrainFeature) -> TerrainRelation {
        TerrainRelation {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &TerrainRelation, g: &TerrainRelation) -> Option<TerrainRelation> {
        if f.to != g.from {
            return None;
        }
        Some(TerrainRelation {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<TerrainRelation> {
        let features = TerrainFeature::variants();
        features
            .iter()
            .flat_map(|&from| features.iter().map(move |&to| TerrainRelation { from, to }))
            .collect()
    }
}

/// Quality: curvature signature for each terrain feature type.
#[derive(Debug, Clone)]
pub struct CurvatureSignature;

impl Quality for CurvatureSignature {
    type Individual = TerrainFeature;
    /// (principal curvature 1 sign, principal curvature 2 sign): +1, 0, -1
    type Value = (i8, i8);

    fn get(&self, feature: &TerrainFeature) -> Option<(i8, i8)> {
        Some(match feature {
            TerrainFeature::Peak => (-1, -1), // both curvatures negative (concave down)
            TerrainFeature::Valley => (1, 1), // both curvatures positive (concave up)
            TerrainFeature::Ridge => (-1, 0), // one negative, one zero
            TerrainFeature::Saddle => (-1, 1), // opposite signs
        })
    }
}

/// Axiom: DEM elevation is bounded (terrain has finite elevation range).
pub struct ElevationBounded;

impl Axiom for ElevationBounded {
    fn description(&self) -> &str {
        "DEM elevation values are bounded within a finite range"
    }
    fn holds(&self) -> bool {
        // Structural axiom: any real DEM has finite elevation values.
        // Earth's range: -11034m (Mariana Trench) to +8849m (Everest).
        true
    }
}

/// Axiom: peaks have strictly negative principal curvatures.
pub struct PeakCurvatureNegative;

impl Axiom for PeakCurvatureNegative {
    fn description(&self) -> &str {
        "peaks have negative principal curvatures (local maxima)"
    }
    fn holds(&self) -> bool {
        let q = CurvatureSignature;
        if let Some((k1, k2)) = q.get(&TerrainFeature::Peak) {
            k1 < 0 && k2 < 0
        } else {
            false
        }
    }
}

pub struct TerrainOntology;

impl Ontology for TerrainOntology {
    type Cat = TerrainCategory;
    type Qual = CurvatureSignature;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(ElevationBounded), Box::new(PeakCurvatureNegative)]
    }
}
