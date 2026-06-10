//! Traffic — intersection conflict model.
//!
//! # Literature
//!
//! - **Highway Capacity Manual (TRB)** — Transportation Research Board's
//!   empirical foundation for intersection conflict analysis at signalized
//!   intersections; defines conflicting movements as the basis for phase
//!   design.
//! - **Webster (1958)** "Traffic Signal Settings", Road Research Laboratory
//!   Technical Paper No. 39 — foundational mathematical model of
//!   intersection delay; introduces orthogonal-phase signal timing where
//!   north–south and east–west traffic move alternately.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Traffic",
    source: "Highway Capacity Manual (TRB); Webster (1958) Traffic Signal Settings, RRL Technical Paper No. 39",

    concepts: [North, South, East, West],

    labels: {
        North: ("en", "North",
            "Cardinal direction at an intersection — northbound traffic flow."),
        South: ("en", "South",
            "Cardinal direction at an intersection — southbound traffic flow."),
        East: ("en", "East",
            "Cardinal direction at an intersection — eastbound traffic flow."),
        West: ("en", "West",
            "Cardinal direction at an intersection — westbound traffic flow."),
    },

    opposes: [
        // Polar-opposite cardinal pairs (same-axis, opposite direction).
        // Per Webster (1958) signal-phase design, polar opposites share a
        // green phase; orthogonal pairs are the conflicting movements.
        (North, South),
        (South, North),
        (East, West),
        (West, East),
    ],
}

/// Conflict-relation membership: which directions conflict with the
/// north–south axis (the orthogonal east–west axis).
///
/// Webster (1958) §2 — orthogonal-phase signal timing assigns conflicting
/// movements to disjoint green phases. East and West conflict with North;
/// South (polar-opposite of North) does not.
#[derive(Debug, Clone)]
pub struct ConflictsWithNorth;

impl Quality for ConflictsWithNorth {
    type Individual = TrafficConcept;
    type Value = ();

    fn get(&self, dir: &TrafficConcept) -> Option<()> {
        match dir {
            TrafficConcept::East | TrafficConcept::West => Some(()),
            _ => None,
        }
    }
}

impl Ontology for TrafficOntology {
    type Cat = TrafficCategory;
    type Qual = ConflictsWithNorth;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(OrthogonalConflicts));
        axioms
    }
}

/// Axiom: N–S and E–W form orthogonal conflict pairs at an intersection.
///
/// Webster (1958): a signalised intersection's two green phases serve the
/// two non-conflicting axes; the conflict relation is orthogonal — East and
/// West conflict with North, South does not (it shares North's phase).
pub struct OrthogonalConflicts;

impl Axiom for OrthogonalConflicts {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if ConflictsWithNorth.get(&TrafficConcept::East).is_some()
            && ConflictsWithNorth.get(&TrafficConcept::West).is_some()
            && ConflictsWithNorth.get(&TrafficConcept::South).is_none()
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "OrthogonalConflicts",
        "north-south and east-west are orthogonal conflict pairs at an intersection",
        "Webster (1958) Traffic Signal Settings, RRL Technical Paper No. 39"
    );
}

pr4xis::register_axiom!(
    OrthogonalConflicts,
    "Webster (1958) Traffic Signal Settings, RRL Technical Paper No. 39"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<TrafficCategory>();
    }

    #[test]
    fn ontology_validates() {
        TrafficOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn four_cardinal_directions() {
        assert_eq!(TrafficConcept::variants().len(), 4);
    }

    #[test]
    fn conflicts_with_north_picks_east_west() {
        let q = ConflictsWithNorth;
        assert!(q.get(&TrafficConcept::East).is_some());
        assert!(q.get(&TrafficConcept::West).is_some());
        assert!(q.get(&TrafficConcept::North).is_none());
        assert!(q.get(&TrafficConcept::South).is_none());
    }

    #[test]
    fn polar_opposites_via_opposition_kind() {
        // Webster's polar opposites: N↔S, E↔W share a green phase.
        let opposed: Vec<_> = TrafficCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == TrafficRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opposed.contains(&(TrafficConcept::North, TrafficConcept::South)));
        assert!(opposed.contains(&(TrafficConcept::East, TrafficConcept::West)));
    }

    #[test]
    fn orthogonal_conflicts_holds() {
        match OrthogonalConflicts.verify() {
            Ok(_) => {}
            Err(c) => panic!("OrthogonalConflicts failed: {}", c.meta().name.as_str()),
        }
    }

    fn arb_direction() -> impl Strategy<Value = TrafficConcept> {
        proptest::sample::select(TrafficConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in TrafficCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in TrafficOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(
                        false,
                        "axiom failed: {}",
                        c.meta().name.as_str()
                    );
                }
            }
        }

        #[test]
        fn prop_conflicts_with_north_total(d in arb_direction()) {
            // Total function: defined on East/West (Some), undefined on N/S (None).
            // Either way, calling .get is total — never panics.
            let _ = ConflictsWithNorth.get(&d);
        }

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            // Both directions of each polar pair are explicit edges.
            let opposed: std::collections::HashSet<_> = TrafficCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == TrafficRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in opposed.iter() {
                prop_assert!(opposed.contains(&(*b, *a)),
                    "opposition not symmetric: {:?} → {:?} but not back", a, b);
            }
        }
    }
}
