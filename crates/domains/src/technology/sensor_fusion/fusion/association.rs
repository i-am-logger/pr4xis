use pr4xis::category::Entity;
use pr4xis::ontology::Quality;

/// Data association strategies as an ontological classification.
///
/// Data association answers: "which measurement came from which target?"
/// This is the fundamental challenge in multi-target, multi-sensor fusion.
///
/// Source: Bar-Shalom & Li (1995), *Multitarget-Multisensor Tracking:
///         Principles and Techniques*, Chapter 3.
///         Blackman & Popoli (1999), *Design and Analysis of Modern
///         Tracking Systems*, Chapter 7.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssociationStrategy {
    /// Nearest-neighbor (NN): assign each measurement to the closest track.
    /// O(nm) where n=tracks, m=measurements.
    /// Simple but greedy — can make locally optimal but globally suboptimal choices.
    NearestNeighbor,
    /// Global nearest-neighbor (GNN): optimal assignment via the Hungarian algorithm.
    /// O(n^3). Globally optimal single assignment, but doesn't handle ambiguity.
    /// Source: Kuhn (1955), "The Hungarian method."
    GlobalNearestNeighbor,
    /// Joint Probabilistic Data Association (JPDA): soft association.
    /// Each measurement contributes to multiple tracks weighted by probability.
    /// Handles clutter and missed detections gracefully.
    /// Source: Bar-Shalom & Fortmann (1988), *Tracking and Data Association*.
    JPDA,
    /// Multiple Hypothesis Tracking (MHT): defers association decisions.
    /// Maintains a tree of hypotheses, pruning unlikely branches.
    /// Theoretically optimal but exponential complexity without pruning.
    /// Source: Reid (1979), "An algorithm for tracking multiple targets."
    MHT,
    /// Auction-based assignment: sensors "bid" for measurements.
    /// Good for distributed fusion architectures.
    /// Source: Bertsekas (1988), "The auction algorithm."
    Auction,
}

impl Entity for AssociationStrategy {
    fn variants() -> Vec<Self> {
        vec![
            Self::NearestNeighbor,
            Self::GlobalNearestNeighbor,
            Self::JPDA,
            Self::MHT,
            Self::Auction,
        ]
    }
}

/// Quality: whether the strategy makes hard (deterministic) or soft (probabilistic) assignments.
#[derive(Debug, Clone)]
pub struct AssignmentType;

/// Hard assignment means each measurement goes to exactly one track.
/// Soft assignment means measurements contribute to multiple tracks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Assignment {
    Hard,
    Soft,
}

impl Quality for AssignmentType {
    type Individual = AssociationStrategy;
    type Value = Assignment;

    fn get(&self, strategy: &AssociationStrategy) -> Option<Assignment> {
        Some(match strategy {
            AssociationStrategy::NearestNeighbor => Assignment::Hard,
            AssociationStrategy::GlobalNearestNeighbor => Assignment::Hard,
            AssociationStrategy::JPDA => Assignment::Soft,
            AssociationStrategy::MHT => Assignment::Hard, // hard per hypothesis
            AssociationStrategy::Auction => Assignment::Hard,
        })
    }
}

/// Quality: computational complexity class.
#[derive(Debug, Clone)]
pub struct ComplexityClass;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Complexity {
    /// O(nm)
    Linear,
    /// O(n^3)
    Cubic,
    /// Exponential (without pruning)
    Exponential,
}

impl Quality for ComplexityClass {
    type Individual = AssociationStrategy;
    type Value = Complexity;

    fn get(&self, strategy: &AssociationStrategy) -> Option<Complexity> {
        Some(match strategy {
            AssociationStrategy::NearestNeighbor => Complexity::Linear,
            AssociationStrategy::GlobalNearestNeighbor => Complexity::Cubic,
            AssociationStrategy::JPDA => Complexity::Exponential,
            AssociationStrategy::MHT => Complexity::Exponential,
            AssociationStrategy::Auction => Complexity::Cubic,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_strategies_have_assignment_type() {
        let q = AssignmentType;
        for strategy in AssociationStrategy::variants() {
            assert!(q.get(&strategy).is_some());
        }
    }

    #[test]
    fn jpda_is_soft_assignment() {
        let q = AssignmentType;
        assert_eq!(q.get(&AssociationStrategy::JPDA), Some(Assignment::Soft));
    }

    #[test]
    fn nn_is_linear() {
        let q = ComplexityClass;
        assert_eq!(
            q.get(&AssociationStrategy::NearestNeighbor),
            Some(Complexity::Linear)
        );
    }

    #[test]
    fn five_strategy_variants() {
        assert_eq!(AssociationStrategy::variants().len(), 5);
    }
}
