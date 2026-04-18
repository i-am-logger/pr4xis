//! INS/GNSS operational state — the lifecycle of an integrated system.
//!
//! Extracted from the main INS/GNSS ontology to eliminate the dual-enum
//! smell (primary ontology + manual TaxonomyDef for InsGnssState).
//!
//! Source: Groves (2013) Section 14.2.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "InsGnssState",
    source: "Groves (2013) Section 14.2",
    being: Event,

    concepts: [State, NavigationMode, Coasting, GnssReacquired, Initializing],

    labels: {
        State: ("en", "INS/GNSS state", "Abstract operational state — root of the taxonomy."),
        NavigationMode: ("en", "Navigation mode", "Full navigation: both INS and GNSS active."),
        Coasting: ("en", "Coasting", "INS only; GNSS unavailable. Error grows quadratically."),
        GnssReacquired: ("en", "GNSS reacquired", "GNSS signals recovered after outage."),
        Initializing: ("en", "Initializing", "System alignment in progress."),
    },

    is_a: [
        (NavigationMode, State),
        (Coasting, State),
        (GnssReacquired, State),
        (Initializing, State),
    ],
}

/// Quality: typical duration of each state.
#[derive(Debug, Clone)]
pub struct StateDuration;

impl Quality for StateDuration {
    type Individual = InsGnssStateConcept;
    type Value = &'static str;

    fn get(&self, s: &InsGnssStateConcept) -> Option<&'static str> {
        Some(match s {
            InsGnssStateConcept::State => "varies",
            InsGnssStateConcept::NavigationMode => "indefinite (steady state)",
            InsGnssStateConcept::Coasting => "seconds to minutes",
            InsGnssStateConcept::GnssReacquired => "seconds (transient)",
            InsGnssStateConcept::Initializing => "1-10 minutes (alignment)",
        })
    }
}

impl Ontology for InsGnssStateOntology {
    type Cat = InsGnssStateCategory;
    type Qual = StateDuration;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Concept;

    #[test]
    fn has_five_concepts() {
        assert_eq!(InsGnssStateConcept::variants().len(), 5);
    }

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<InsGnssStateCategory>().unwrap();
    }
}
