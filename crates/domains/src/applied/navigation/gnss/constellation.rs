//! GNSS constellations — the satellite systems providing the signals.
//!
//! Extracted from the main GNSS ontology to eliminate the dual-enum smell
//! (primary ontology + manual TaxonomyDef for GnssConstellation). Each
//! constellation is its own concept here; the signals they provide flow
//! through the `GnssObservable` ontology.
//!
//! Source: Kaplan & Hegarty (2006); IS-GPS-200.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "GnssConstellation",
    source: "Kaplan & Hegarty (2006); IS-GPS-200",
    being: SocialObject,

    concepts: [Constellation, GPS, GLONASS, Galileo, BeiDou, SBAS],

    labels: {
        Constellation: ("en", "GNSS constellation", "Abstract GNSS constellation — root of the taxonomy."),
        GPS: ("en", "GPS", "US Global Positioning System."),
        GLONASS: ("en", "GLONASS", "Russian GLONASS (Globalnaya Navigatsionnaya Sputnikovaya Sistema)."),
        Galileo: ("en", "Galileo", "European Galileo."),
        BeiDou: ("en", "BeiDou", "Chinese BeiDou."),
        SBAS: ("en", "SBAS", "Satellite-Based Augmentation Systems (WAAS, EGNOS, MSAS, GAGAN)."),
    },

    is_a: [
        (GPS, Constellation),
        (GLONASS, Constellation),
        (Galileo, Constellation),
        (BeiDou, Constellation),
        (SBAS, Constellation),
    ],
}

/// Quality: approximate number of operational satellites in the constellation.
#[derive(Debug, Clone)]
pub struct SatelliteCount;

impl Quality for SatelliteCount {
    type Individual = GnssConstellationConcept;
    type Value = &'static str;

    fn get(&self, c: &GnssConstellationConcept) -> Option<&'static str> {
        Some(match c {
            GnssConstellationConcept::Constellation => "varies",
            GnssConstellationConcept::GPS => "~31 (6 orbital planes)",
            GnssConstellationConcept::GLONASS => "~24 (3 orbital planes)",
            GnssConstellationConcept::Galileo => "~28 (3 orbital planes)",
            GnssConstellationConcept::BeiDou => "~35 (MEO + IGSO + GEO)",
            GnssConstellationConcept::SBAS => "depends on region (GEO)",
        })
    }
}

impl Ontology for GnssConstellationOntology {
    type Cat = GnssConstellationCategory;
    type Qual = SatelliteCount;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Concept;

    #[test]
    fn has_six_concepts() {
        assert_eq!(GnssConstellationConcept::variants().len(), 6);
    }

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<GnssConstellationCategory>().unwrap();
    }
}
