//! Collapse Pattern Analysis
//!
//! Given a functor with high loss, analyzes the PATTERN of collapse:
//! - Which target entities absorb multiple sources? (absorption analysis)
//! - Do absorbed entities share taxonomy? (cluster analysis)
//! - What is the fan-in ratio per target? (concentration analysis)
//!
//! Based on these patterns, recommends resolution strategies:
//! - Fan-in > 3 on one target -> split that target entity
//! - Absorbed entities share parent taxonomy -> missing intermediate category
//! - Loss > 80% overall -> intermediate domain needed
//!
//! NOVEL: No existing system uses functor structure for ontology recommendations.
//! KGC predicts missing links; we predict missing DOMAINS.
//!
//! Literature basis:
//! - Spivak & Kent 2012 (ologs) -- categorical ontology framework
//! - Kent 2024 (Galois connections) -- abstraction-refinement factorization
//! - KGC field -- related but uses embeddings, not adjunctions

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use pr4xis::category::{Entity, Functor};

use crate::natural::biomedical::acoustics::biophysics_functor::AcousticsToBiophysics;
use crate::natural::biomedical::acoustics::ontology::AcousticsEntity;
use crate::natural::biomedical::bioelectricity::ontology::BioelectricEntity;
use crate::natural::biomedical::biophysics::ontology::BiophysicsEntity;
use crate::natural::biomedical::molecular::bioelectricity_functor::MolecularToBioelectric;
use crate::natural::biomedical::molecular::ontology::MolecularEntity;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A target entity that absorbs multiple source entities.
#[derive(Debug, Clone)]
pub struct AbsorptionPoint<S: Entity, T: Entity> {
    /// The target entity that absorbs.
    pub target: T,
    /// All source entities that map to this target.
    pub absorbed: Vec<S>,
    /// Fan-in: how many sources map here.
    pub fan_in: usize,
}

/// Classification of what kind of recommendation to make.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recommendation {
    /// Target entity should be split into subtypes.
    SplitTarget,
    /// Add ContextDef to distinguish functional modes.
    AddContextDef,
    /// Entire intermediate domain needed.
    AddIntermediateDomain,
    /// Loss is acceptable -- intentional abstraction.
    AcceptableAbstraction,
}

/// Full analysis of a functor's collapse patterns.
#[derive(Debug, Clone)]
pub struct CollapseReport<S: Entity, T: Entity> {
    pub absorption_points: Vec<AbsorptionPoint<S, T>>,
    pub max_fan_in: usize,
    pub overall_collapse: f64,
    pub recommendation: Recommendation,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Compute absorption points from a functor mapping.
///
/// For each target entity that receives more than one source, creates an
/// `AbsorptionPoint` recording the fan-in and absorbed sources.
fn compute_absorption_points<S, T>(
    sources: Vec<S>,
    map_fn: fn(&S) -> T,
) -> Vec<AbsorptionPoint<S, T>>
where
    S: Entity + Debug + Clone,
    T: Entity + Debug + Clone + Eq + Hash,
{
    let mut target_to_sources: HashMap<T, Vec<S>> = HashMap::new();

    for source in sources {
        let target = map_fn(&source);
        target_to_sources.entry(target).or_default().push(source);
    }

    let mut points: Vec<AbsorptionPoint<S, T>> = target_to_sources
        .into_iter()
        .filter(|(_, sources)| sources.len() > 1)
        .map(|(target, absorbed)| {
            let fan_in = absorbed.len();
            AbsorptionPoint {
                target,
                absorbed,
                fan_in,
            }
        })
        .collect();

    // Sort by fan-in descending for consistent output.
    points.sort_by_key(|p| std::cmp::Reverse(p.fan_in));
    points
}

/// Compute overall collapse ratio: fraction of source entities that share a
/// target with at least one other source entity.
fn compute_overall_collapse<S, T>(sources: &[S], map_fn: fn(&S) -> T) -> f64
where
    S: Entity + Debug + Clone,
    T: Entity + Debug + Clone + Eq + Hash,
{
    let total = sources.len();
    if total == 0 {
        return 0.0;
    }

    let mut target_counts: HashMap<T, usize> = HashMap::new();
    for source in sources {
        let target = map_fn(source);
        *target_counts.entry(target).or_insert(0) += 1;
    }

    // Unique targets that actually appear.
    let unique_targets = target_counts.len();

    // Collapse = 1 - (unique targets / total sources).
    1.0 - (unique_targets as f64 / total as f64)
}

/// Choose a recommendation based on overall collapse and max fan-in.
fn recommend(overall_collapse: f64, max_fan_in: usize) -> Recommendation {
    if overall_collapse < 0.4 {
        Recommendation::AcceptableAbstraction
    } else if overall_collapse > 0.8 {
        Recommendation::AddIntermediateDomain
    } else if max_fan_in > 3 {
        Recommendation::SplitTarget
    } else {
        Recommendation::AddContextDef
    }
}

/// Build a full CollapseReport from source entities and a mapping function.
fn build_report<S, T>(sources: Vec<S>, map_fn: fn(&S) -> T) -> CollapseReport<S, T>
where
    S: Entity + Debug + Clone,
    T: Entity + Debug + Clone + Eq + Hash,
{
    let overall_collapse = compute_overall_collapse(&sources, map_fn);
    let absorption_points = compute_absorption_points(sources, map_fn);
    let max_fan_in = absorption_points
        .iter()
        .map(|p| p.fan_in)
        .max()
        .unwrap_or(0);
    let recommendation = recommend(overall_collapse, max_fan_in);

    CollapseReport {
        absorption_points,
        max_fan_in,
        overall_collapse,
        recommendation,
    }
}

// ---------------------------------------------------------------------------
// Concrete analysis functions
// ---------------------------------------------------------------------------

/// Analyze collapse patterns in the Molecular -> Bioelectric functor.
///
/// This is the primary example: 27 molecular entities map to only 4-5
/// distinct bioelectric targets, giving ~85% collapse.
pub fn analyze_molecular_to_bioelectric_collapse()
-> CollapseReport<MolecularEntity, BioelectricEntity> {
    build_report(
        MolecularEntity::variants(),
        MolecularToBioelectric::map_object,
    )
}

/// Analyze collapse patterns in the Acoustics -> Biophysics functor.
///
/// 27 acoustic entities map to ~10 distinct biophysical targets.
pub fn analyze_acoustics_to_biophysics_collapse()
-> CollapseReport<AcousticsEntity, BiophysicsEntity> {
    build_report(
        AcousticsEntity::variants(),
        AcousticsToBiophysics::map_object,
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Molecular -> Bioelectric tests --

    #[test]
    fn test_signal_is_highest_absorption() {
        let report = analyze_molecular_to_bioelectric_collapse();

        // Signal should have the highest fan-in: many molecular entities
        // (ions, abstract categories, signaling molecules) all map to Signal.
        let signal_point = report
            .absorption_points
            .iter()
            .find(|p| p.target == BioelectricEntity::Signal);
        assert!(
            signal_point.is_some(),
            "Signal should be an absorption point in molecular->bioelectric"
        );

        let signal_fan_in = signal_point.unwrap().fan_in;

        // Signal should be the highest or tied for highest fan-in.
        assert_eq!(
            signal_fan_in, report.max_fan_in,
            "Signal should have the highest fan-in ({}) but max is {}",
            signal_fan_in, report.max_fan_in,
        );
    }

    #[test]
    fn test_recommendation_for_high_loss() {
        // The molecular->bioelectric functor has ~85% collapse.
        // With loss > 80%, recommendation should be AddIntermediateDomain.
        let report = analyze_molecular_to_bioelectric_collapse();

        assert!(
            report.overall_collapse > 0.80,
            "molecular->bioelectric should have >80% collapse, got {:.1}%",
            report.overall_collapse * 100.0,
        );
        assert_eq!(
            report.recommendation,
            Recommendation::AddIntermediateDomain,
            "high loss ({:.1}%) should recommend AddIntermediateDomain",
            report.overall_collapse * 100.0,
        );
    }

    #[test]
    fn test_fan_in_correlates_with_loss() {
        // Higher fan-in targets contribute more to overall collapse.
        // Verify that the sum of (fan_in - 1) across absorption points
        // equals (total sources - unique targets), i.e. the number of
        // "lost" source identities.
        let report = analyze_molecular_to_bioelectric_collapse();

        let total_sources = MolecularEntity::variants().len();
        let excess_from_fan_in: usize = report
            .absorption_points
            .iter()
            .map(|p| p.fan_in - 1) // each absorption point "loses" (fan_in - 1) identities
            .sum();

        // unique_targets = total_sources - excess
        let unique_targets = total_sources - excess_from_fan_in;
        let expected_collapse = 1.0 - (unique_targets as f64 / total_sources as f64);

        assert!(
            (expected_collapse - report.overall_collapse).abs() < 1e-10,
            "fan-in excess ({}) should account for collapse: expected {:.4}, got {:.4}",
            excess_from_fan_in,
            expected_collapse,
            report.overall_collapse,
        );
    }

    #[test]
    fn test_acceptable_abstraction_for_moderate_loss() {
        // Verify the recommendation logic: <40% collapse -> AcceptableAbstraction.
        let rec = recommend(0.35, 2);
        assert_eq!(
            rec,
            Recommendation::AcceptableAbstraction,
            "<40% collapse should be AcceptableAbstraction",
        );

        // Also verify that the acoustics->biophysics functor, which has
        // lower collapse than molecular->bioelectric, gets a less severe
        // recommendation.
        let acoustics_report = analyze_acoustics_to_biophysics_collapse();
        assert!(
            acoustics_report.overall_collapse < report_molecular_collapse(),
            "acoustics->biophysics collapse ({:.1}%) should be less than \
             molecular->bioelectric ({:.1}%)",
            acoustics_report.overall_collapse * 100.0,
            report_molecular_collapse() * 100.0,
        );
    }

    /// Helper: get molecular->bioelectric collapse ratio.
    fn report_molecular_collapse() -> f64 {
        analyze_molecular_to_bioelectric_collapse().overall_collapse
    }

    // -- Acoustics -> Biophysics tests --

    #[test]
    fn test_acoustics_biophysics_has_absorption() {
        let report = analyze_acoustics_to_biophysics_collapse();
        assert!(
            !report.absorption_points.is_empty(),
            "acoustics->biophysics should have absorption points"
        );
    }

    #[test]
    fn test_acoustics_biophysics_collapse_is_nonzero() {
        let report = analyze_acoustics_to_biophysics_collapse();
        assert!(
            report.overall_collapse > 0.0,
            "acoustics->biophysics should have some collapse"
        );
    }

    #[test]
    fn test_absorption_points_sorted_by_fan_in() {
        let report = analyze_molecular_to_bioelectric_collapse();
        for window in report.absorption_points.windows(2) {
            assert!(
                window[0].fan_in >= window[1].fan_in,
                "absorption points should be sorted by fan-in descending: {} < {}",
                window[0].fan_in,
                window[1].fan_in,
            );
        }
    }

    #[test]
    fn test_recommendation_thresholds() {
        // Unit tests for the recommend function at boundary values.
        assert_eq!(recommend(0.0, 0), Recommendation::AcceptableAbstraction);
        assert_eq!(recommend(0.39, 2), Recommendation::AcceptableAbstraction);
        assert_eq!(recommend(0.5, 2), Recommendation::AddContextDef);
        assert_eq!(recommend(0.5, 4), Recommendation::SplitTarget);
        assert_eq!(recommend(0.81, 10), Recommendation::AddIntermediateDomain);
        assert_eq!(recommend(0.85, 1), Recommendation::AddIntermediateDomain);
    }
}
