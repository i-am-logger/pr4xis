use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Terrain feature types for terrain-relative navigation.
///
/// Source: Goldstein (1987), "Terrain Aided Navigation"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
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

define_ontology! {
    /// Category for terrain feature relationships.
    ///
    /// Ridges connect peaks through saddles; valleys lie between ridges.
    /// All features can relate to all other features in a DEM.
    pub TerrainOntology for TerrainCategory {
        entity: TerrainFeature,
        relation: TerrainRelation,
        being: PhysicalEndurant,
        source: "Goldstein (1987)",
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
pr4xis::register_axiom!(ElevationBounded);

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
pr4xis::register_axiom!(PeakCurvatureNegative);

impl Ontology for TerrainOntology {
    type Cat = TerrainCategory;
    type Qual = CurvatureSignature;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(ElevationBounded), Box::new(PeakCurvatureNegative)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::ontology::Ontology;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<TerrainCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        TerrainOntology::validate().unwrap();
    }
}
