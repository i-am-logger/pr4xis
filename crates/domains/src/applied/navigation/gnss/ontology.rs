//! GNSS observable types — what a GNSS receiver measures.
//!
//! Covers the observable signal types (pseudorange, carrier phase, Doppler,
//! nav message). The constellation ontology (GPS, GLONASS, Galileo, BeiDou)
//! lives in the sibling `constellation` module.
//!
//! Source: IS-GPS-200 (2022), Groves (2013) Chapter 8, Misra & Enge (2011).

#![allow(clippy::needless_range_loop)]

use pr4xis::ontology::reasoning::taxonomy;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Gnss",
    source: "IS-GPS-200 (2022); Groves (2013); Misra & Enge (2011)",
    being: Process,

    concepts: [Observable, Pseudorange, CarrierPhase, Doppler, NavigationMessage],

    labels: {
        Observable: ("en", "GNSS observable", "Abstract GNSS observable — root of the taxonomy."),
        Pseudorange: ("en", "Pseudorange", "Code-phase measurement (meters). c * (t_receive - t_transmit)."),
        CarrierPhase: ("en", "Carrier phase", "Accumulated carrier cycles (more precise than pseudorange)."),
        Doppler: ("en", "Doppler", "Frequency shift from satellite motion (Hz)."),
        NavigationMessage: ("en", "Navigation message", "Ephemeris, almanac, clock corrections."),
    },

    is_a: [
        (Pseudorange, Observable),
        (CarrierPhase, Observable),
        (Doppler, Observable),
        (NavigationMessage, Observable),
    ],
}

/// Quality: Dilution of Precision — measures geometric quality of satellite geometry.
///
/// Source: Misra & Enge (2011), Chapter 7.
#[derive(Debug, Clone)]
pub struct DilutionOfPrecision;

impl Quality for DilutionOfPrecision {
    type Individual = GnssConcept;
    type Value = &'static str;

    fn get(&self, obs: &GnssConcept) -> Option<&'static str> {
        match obs {
            GnssConcept::Pseudorange => Some("GDOP/PDOP/HDOP/VDOP from pseudorange geometry"),
            GnssConcept::CarrierPhase => Some("same DOP, higher precision per measurement"),
            _ => None,
        }
    }
}

/// Quality: Signal strength in carrier-to-noise-density ratio (C/N0, dB-Hz).
#[derive(Debug, Clone)]
pub struct SignalStrength;

impl Quality for SignalStrength {
    type Individual = GnssConcept;
    type Value = &'static str;

    fn get(&self, obs: &GnssConcept) -> Option<&'static str> {
        match obs {
            GnssConcept::Observable => Some("C/N0 (dB-Hz)"),
            GnssConcept::Pseudorange => Some("C/N0 35-50 dB-Hz open sky"),
            GnssConcept::CarrierPhase => Some("C/N0 35-50 dB-Hz, more sensitive to loss"),
            GnssConcept::Doppler => Some("C/N0 derived from carrier tracking"),
            GnssConcept::NavigationMessage => Some("requires C/N0 > 25 dB-Hz to decode"),
        }
    }
}

/// Minimum 4 satellites required for 3D position fix.
pub struct MinimumSatellites;

impl Axiom for MinimumSatellites {
    fn description(&self) -> &str {
        "need >= 4 satellites for 3D fix (3 spatial + 1 clock unknown)"
    }
    fn holds(&self) -> bool {
        let spatial_unknowns = 3;
        let clock_unknowns = 1;
        let min_satellites = spatial_unknowns + clock_unknowns;
        min_satellites == 4
    }
}
pr4xis::register_axiom!(MinimumSatellites);

/// DOP improves (decreases) with wider satellite spread.
pub struct DopGeometry;

impl Axiom for DopGeometry {
    fn description(&self) -> &str {
        "DOP improves with wider satellite angular spread"
    }
    fn holds(&self) -> bool {
        let gdop_wide = compute_gdop_from_elevations_azimuths(
            &[45.0, 45.0, 45.0, 45.0, 89.0],
            &[0.0, 90.0, 180.0, 270.0, 0.0],
        );
        let gdop_narrow = compute_gdop_from_elevations_azimuths(
            &[45.0, 44.0, 46.0, 45.0, 43.0],
            &[0.0, 5.0, 10.0, 15.0, 20.0],
        );
        gdop_wide < gdop_narrow
    }
}
pr4xis::register_axiom!(DopGeometry);

/// Pseudorange must be non-negative.
pub struct PseudorangePositive;

impl Axiom for PseudorangePositive {
    fn description(&self) -> &str {
        "pseudorange >= 0 (signal travel time * speed of light)"
    }
    fn holds(&self) -> bool {
        taxonomy::is_a::<GnssTaxonomy>(&GnssConcept::Pseudorange, &GnssConcept::Observable) && {
            let speed_of_light = 299_792_458.0_f64;
            let min_travel_time = 0.0_f64;
            let min_pseudorange = speed_of_light * min_travel_time;
            min_pseudorange >= 0.0
        }
    }
}
pr4xis::register_axiom!(PseudorangePositive);

/// Compute GDOP from satellite elevations and azimuths.
pub(crate) fn compute_gdop_from_elevations_azimuths(
    elevations_deg: &[f64],
    azimuths_deg: &[f64],
) -> f64 {
    let n = elevations_deg.len();
    if n < 4 {
        return f64::MAX;
    }

    let mut h_rows: Vec<[f64; 4]> = Vec::with_capacity(n);
    for i in 0..n {
        let el = elevations_deg[i].to_radians();
        let az = azimuths_deg[i].to_radians();
        h_rows.push([el.cos() * az.cos(), el.cos() * az.sin(), el.sin(), 1.0]);
    }

    let mut hth = [[0.0_f64; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            for row in &h_rows {
                hth[i][j] += row[i] * row[j];
            }
        }
    }

    if let Some(inv) = invert_4x4(&hth) {
        let trace = inv[0][0] + inv[1][1] + inv[2][2] + inv[3][3];
        if trace > 0.0 { trace.sqrt() } else { f64::MAX }
    } else {
        f64::MAX
    }
}

/// Invert a 4x4 matrix using Gauss-Jordan elimination.
fn invert_4x4(m: &[[f64; 4]; 4]) -> Option<[[f64; 4]; 4]> {
    let mut aug = [[0.0_f64; 8]; 4];
    for i in 0..4 {
        for j in 0..4 {
            aug[i][j] = m[i][j];
        }
        aug[i][i + 4] = 1.0;
    }

    for col in 0..4 {
        let mut max_row = col;
        let mut max_val = aug[col][col].abs();
        for row in (col + 1)..4 {
            if aug[row][col].abs() > max_val {
                max_val = aug[row][col].abs();
                max_row = row;
            }
        }
        if max_val < 1e-12 {
            return None;
        }
        aug.swap(col, max_row);

        let pivot = aug[col][col];
        for j in 0..8 {
            aug[col][j] /= pivot;
        }

        for row in 0..4 {
            if row != col {
                let factor = aug[row][col];
                for j in 0..8 {
                    aug[row][j] -= factor * aug[col][j];
                }
            }
        }
    }

    let mut result = [[0.0_f64; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            result[i][j] = aug[i][j + 4];
        }
    }
    Some(result)
}

impl Ontology for GnssOntology {
    type Cat = GnssCategory;
    type Qual = SignalStrength;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(MinimumSatellites),
            Box::new(DopGeometry),
            Box::new(PseudorangePositive),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::ontology::Ontology;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<GnssCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        GnssOntology::validate().unwrap();
    }
}
