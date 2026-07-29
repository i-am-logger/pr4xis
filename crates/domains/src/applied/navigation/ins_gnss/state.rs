//! INS/GNSS operational state — the lifecycle of an integrated system.
//!
//! Extracted from the main INS/GNSS ontology to eliminate the dual-enum
//! smell (primary ontology + manual TaxonomyDef for InsGnssState).
//!
//! Source: Groves (2013) Section 14.2.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality, QualityKind};

use crate::formal::math::quantity::unit::SECOND;
use crate::formal::math::quantity::value::{Quantity, QuantityRange};

pr4xis::ontology! {
    name: "InsGnssState",
    source: "Groves (2013) Section 14.2",

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

/// Quality: typical duration of each state, a [`QuantityRange`] in seconds,
/// NOT a prose string.
///
/// `None` for the abstract `State` ("varies", implementation-dependent) and
/// for `NavigationMode`: steady-state full navigation has no typical
/// duration ceiling at all — it persists indefinitely as long as INS and
/// GNSS stay locked, unlike the other (transient) states, which do have a
/// bounded typical duration. `Coasting` and `GnssReacquired` had no explicit
/// digits in the original description ("seconds to minutes", "seconds"); the
/// ranges below are an order-of-magnitude reading of that prose (lower bound
/// = 1 s, the finest unit named; upper bounds anchored to the neighboring
/// order named — "minutes" capped at `Initializing`'s own cited 10-minute
/// figure, "seconds" capped just under a minute). `Initializing` alone has
/// an explicit numeric figure in Groves (2013) Section 14.2.
#[derive(Debug, Clone)]
pub struct StateDuration;

impl Quality for StateDuration {
    type Individual = InsGnssStateConcept;
    type Value = QuantityRange;
    const KIND: QualityKind = QualityKind::Physical;

    fn get(&self, s: &InsGnssStateConcept) -> Option<QuantityRange> {
        let secs = |lo: f64, hi: f64| QuantityRange {
            min: Quantity::from_unit(lo, &SECOND),
            max: Quantity::from_unit(hi, &SECOND),
        };
        Some(match s {
            InsGnssStateConcept::State => return None,
            // Steady state — no typical duration ceiling, unlike the
            // transient states below.
            InsGnssStateConcept::NavigationMode => return None,
            // "seconds to minutes": order-of-magnitude, capped at the
            // 10-minute figure Initializing cites below.
            InsGnssStateConcept::Coasting => secs(1.0, 600.0),
            // "seconds (transient)": order-of-magnitude, sub-minute.
            InsGnssStateConcept::GnssReacquired => secs(1.0, 59.0),
            // 1-10 minutes (alignment).
            InsGnssStateConcept::Initializing => secs(60.0, 600.0),
        })
    }
}

impl Ontology for InsGnssStateOntology {
    type Cat = InsGnssStateCategory;
    type Qual = StateDuration;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn has_five_concepts() {
        assert_eq!(InsGnssStateConcept::variants().len(), 5);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<InsGnssStateCategory>();
    }
}
