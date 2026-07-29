use crate::applied::localization::terrain::ontology::{
    CurvatureSign, CurvatureSignature, TerrainConcept,
};
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;
use pr4xis::category::FinitelyGenerated;
use pr4xis::ontology::Quality;

/// A Digital Elevation Model (DEM) tile.
#[derive(Debug, Clone)]
pub struct DemTile {
    /// Elevation values in row-major order (meters).
    pub elevations: Vec<f64>,
    /// Number of columns.
    pub cols: usize,
    /// Number of rows.
    pub rows: usize,
    /// Grid spacing. LENGTH (BIPM SI Brochure 2019).
    pub resolution: Quantity,
}

impl DemTile {
    pub fn new(elevations: Vec<f64>, cols: usize, rows: usize, resolution: Quantity) -> Self {
        assert_eq!(elevations.len(), cols * rows);
        Self {
            elevations,
            cols,
            rows,
            resolution,
        }
    }

    /// Get elevation at grid position.
    ///
    /// Returns a [`Quantity`] in `Dimension::LENGTH` — the DEM's
    /// `elevations` field is documented in meters (BIPM SI Brochure 2019).
    pub fn elevation(&self, col: usize, row: usize) -> Quantity {
        Quantity::from_unit(self.elevations[row * self.cols + col], &unit::METER)
    }

    /// Estimate the principal-curvature signs of the elevation surface at a
    /// grid cell from a locally-fit quartic surface over the 3x3
    /// neighbourhood.
    ///
    /// Returns the two Hessian eigenvalue signs `(k1, k2)` in `unit::CURVATURE`
    /// — the same typed signature `terrain::ontology::CurvatureSignature`
    /// declares for each [`TerrainConcept`], so [`Self::classify_feature`] can
    /// match against it directly instead of an independent heuristic.
    ///
    /// Source: Zevenbergen, L.W. & Thorne, C.R. (1987). "Quantitative
    /// analysis of land surface topography." Earth Surface Processes and
    /// Landforms, 12(1), 47-56 — the finite-difference coefficients D, E, F
    /// of a quartic surface fit exactly through a 3x3 elevation window, from
    /// which the surface Hessian `Zxx = 2D`, `Zyy = 2E`, `Zxy = F` follows.
    /// The Hessian's eigenvalue signs are Goldstein's (1987) §3
    /// second-derivative test (do Carmo 1976, §3-2, principal curvatures as
    /// Hessian eigenvalues).
    fn principal_curvatures(&self, col: usize, row: usize) -> Option<(Quantity, Quantity)> {
        if col == 0 || col >= self.cols - 1 || row == 0 || row >= self.rows - 1 {
            return None; // border cell
        }
        let z = |dc: i32, dr: i32| -> f64 {
            let nc = (col as i32 + dc) as usize;
            let nr = (row as i32 + dr) as usize;
            self.elevation(nc, nr).value
        };
        let l2 = self.resolution.value * self.resolution.value;

        let z1 = z(-1, -1);
        let z2 = z(0, -1);
        let z3 = z(1, -1);
        let z4 = z(-1, 0);
        let z5 = z(0, 0);
        let z6 = z(1, 0);
        let z7 = z(-1, 1);
        let z8 = z(0, 1);
        let z9 = z(1, 1);

        // Zevenbergen & Thorne (1987) finite-difference coefficients.
        let d = ((z4 + z6) / 2.0 - z5) / l2;
        let e = ((z2 + z8) / 2.0 - z5) / l2;
        let f = (-z1 + z3 + z7 - z9) / (4.0 * l2);

        // Surface Hessian: Zxx = 2D, Zyy = 2E, Zxy = F.
        let zxx = 2.0 * d;
        let zyy = 2.0 * e;
        let zxy = f;

        // Eigenvalues of [[Zxx, Zxy], [Zxy, Zyy]].
        let trace = zxx + zyy;
        let det = zxx * zyy - zxy * zxy;
        let disc = (trace * trace - 4.0 * det).max(0.0).sqrt();
        let lambda1 = (trace + disc) / 2.0;
        let lambda2 = (trace - disc) / 2.0;

        Some((
            Quantity::from_unit(lambda1, &unit::RECIPROCAL_METER),
            Quantity::from_unit(lambda2, &unit::RECIPROCAL_METER),
        ))
    }

    /// Classify terrain feature at a grid cell against the ontology's own
    /// cited [`CurvatureSignature`] (Goldstein 1987 §3), using
    /// [`TerrainClassificationCriteria::standard`] to decide near-zero
    /// curvature. See [`Self::classify_feature_with`] to supply a
    /// deployment-tuned criteria instead.
    pub fn classify_feature(&self, col: usize, row: usize) -> Option<TerrainConcept> {
        self.classify_feature_with(col, row, &TerrainClassificationCriteria::standard())
    }

    /// Classify terrain feature at a grid cell by matching its estimated
    /// principal-curvature signs against each [`TerrainConcept`]'s own
    /// [`CurvatureSignature`] (Goldstein 1987 §3) — the engine and the
    /// ontology read the same definition, instead of an independently
    /// hand-rolled neighbor-counting heuristic.
    pub fn classify_feature_with(
        &self,
        col: usize,
        row: usize,
        criteria: &TerrainClassificationCriteria,
    ) -> Option<TerrainConcept> {
        let (l1, l2) = self.principal_curvatures(col, row)?;
        let s1 = criteria.sign_of(&l1);
        let s2 = criteria.sign_of(&l2);
        TerrainConcept::variants().into_iter().find(|concept| {
            matches!(
                CurvatureSignature.get(concept),
                Some((k1, k2)) if (k1 == s1 && k2 == s2) || (k1 == s2 && k2 == s1)
            )
        })
    }

    /// Compute terrain match score between a measured profile and the DEM.
    ///
    /// Returns the mean absolute elevation difference, as a [`Quantity`] in
    /// `Dimension::LENGTH` — this is a mean absolute error between two sets
    /// of elevation values (each in meters, BIPM SI Brochure 2019), not a
    /// unitless correlation coefficient; the residual carries the same
    /// dimension as the elevations it is computed from.
    pub fn match_profile(&self, col_start: usize, row: usize, profile: &[f64]) -> Quantity {
        let n = profile.len().min(self.cols - col_start);
        if n == 0 {
            return Quantity::from_unit(f64::INFINITY, &unit::METER);
        }
        let sum: f64 = (0..n)
            .map(|i| (self.elevation(col_start + i, row).value - profile[i]).abs())
            .sum();
        Quantity::from_unit(sum / n as f64, &unit::METER)
    }
}

/// Tunable curvature-classification criteria — the ontological replacement
/// for a hardcoded "is this curvature near enough to zero" magic number.
///
/// A 3x3 finite-difference curvature estimate is dominated by DEM sampling
/// noise near zero, so distinguishing [`CurvatureSign::Planar`] from a
/// genuinely nonzero sign needs a tolerance. Wood, J. (1996), "The
/// Geomorphological Characterisation of Digital Elevation Models," PhD
/// thesis, University of Leicester, §3.2, makes this tolerance an explicit,
/// deployment-tunable parameter of curvature-based landform classification
/// — the same tuning-surface role
/// [`RelationCriteria`](crate::social::military::situation::kinematic_relation::RelationCriteria)
/// plays for kinematic-relation classification.
#[derive(Debug, Clone)]
pub struct TerrainClassificationCriteria {
    /// Principal-curvature magnitude below which a direction is treated as
    /// locally planar rather than convex/concave. CURVATURE (m⁻¹).
    pub planarity_tolerance: Quantity,
}

impl TerrainClassificationCriteria {
    /// Illustrative default (Wood 1996 treats this as a deployment-tunable
    /// analysis parameter, not a universal constant): a curvature magnitude
    /// of 10⁻³ m⁻¹ corresponds to a radius of curvature of ~1 km, well
    /// beyond the noise floor of a 3x3 finite-difference estimate at typical
    /// terrain-relative-navigation DEM postings (tens of meters, Goldstein
    /// 1987).
    pub fn standard() -> Self {
        Self {
            planarity_tolerance: Quantity::from_unit(1e-3, &unit::RECIPROCAL_METER),
        }
    }

    /// Classify a single principal-curvature eigenvalue's sign under this
    /// criteria's tolerance.
    fn sign_of(&self, lambda: &Quantity) -> CurvatureSign {
        if lambda.value < -self.planarity_tolerance.value {
            CurvatureSign::Convex
        } else if lambda.value > self.planarity_tolerance.value {
            CurvatureSign::Concave
        } else {
            CurvatureSign::Planar
        }
    }
}
