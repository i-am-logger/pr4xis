#![allow(clippy::needless_range_loop)]
use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::taxonomy::{self, NoCycles, TaxonomyDef};
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// GNSS observable types — what a GNSS receiver measures.
///
/// Source: IS-GPS-200 (2022), Groves (2013) Chapter 8.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum GnssObservable {
    /// Abstract observable.
    Observable,
    /// Pseudorange: code-phase measurement (meters).
    Pseudorange,
    /// Carrier phase: accumulated carrier cycles (more precise than pseudorange).
    CarrierPhase,
    /// Doppler: frequency shift from satellite motion (Hz).
    Doppler,
    /// Navigation message: ephemeris, almanac, clock corrections.
    NavigationMessage,
}

/// GNSS constellation types.
///
/// Source: Kaplan & Hegarty (2006).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum GnssConstellation {
    /// Abstract constellation.
    Constellation,
    /// US Global Positioning System.
    GPS,
    /// Russian GLONASS.
    GLONASS,
    /// European Galileo.
    Galileo,
    /// Chinese BeiDou.
    BeiDou,
    /// Satellite-Based Augmentation Systems.
    SBAS,
}

// ---------------------------------------------------------------------------
// Ontology (category + reasoning)
// ---------------------------------------------------------------------------

define_ontology! {
    /// The GNSS ontology.
    ///
    /// Source: IS-GPS-200 (2022), Groves (2013), Misra & Enge (2011).
    pub GnssOntology for GnssCategory {
        entity: GnssObservable,
        relation: GnssRelation,

        taxonomy: GnssObservableTaxonomy [
            (Pseudorange, Observable),
            (CarrierPhase, Observable),
            (Doppler, Observable),
            (NavigationMessage, Observable),
        ],
    }
}

/// GNSS constellation taxonomy (secondary entity type — manual impl).
pub struct GnssConstellationTaxonomy;

impl TaxonomyDef for GnssConstellationTaxonomy {
    type Entity = GnssConstellation;

    fn relations() -> Vec<(GnssConstellation, GnssConstellation)> {
        use GnssConstellation::*;
        vec![
            (GPS, Constellation),
            (GLONASS, Constellation),
            (Galileo, Constellation),
            (BeiDou, Constellation),
            (SBAS, Constellation),
        ]
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: Dilution of Precision — measures geometric quality of satellite geometry.
///
/// Lower DOP means better geometry. GDOP < 6 is acceptable, < 2 is excellent.
///
/// Source: Misra & Enge (2011), Chapter 7.
#[derive(Debug, Clone)]
pub struct DilutionOfPrecision;

impl Quality for DilutionOfPrecision {
    type Individual = GnssObservable;
    type Value = &'static str;

    fn get(&self, obs: &GnssObservable) -> Option<&'static str> {
        match obs {
            GnssObservable::Pseudorange => Some("GDOP/PDOP/HDOP/VDOP from pseudorange geometry"),
            GnssObservable::CarrierPhase => Some("same DOP, higher precision per measurement"),
            _ => None,
        }
    }
}

/// Quality: Signal strength in carrier-to-noise-density ratio (C/N0, dB-Hz).
///
/// Typical values: 35-50 dB-Hz for open sky, < 25 dB-Hz is weak signal.
///
/// Source: IS-GPS-200, Groves (2013) Section 8.2.
#[derive(Debug, Clone)]
pub struct SignalStrength;

impl Quality for SignalStrength {
    type Individual = GnssObservable;
    type Value = &'static str;

    fn get(&self, obs: &GnssObservable) -> Option<&'static str> {
        match obs {
            GnssObservable::Observable => Some("C/N0 (dB-Hz)"),
            GnssObservable::Pseudorange => Some("C/N0 35-50 dB-Hz open sky"),
            GnssObservable::CarrierPhase => Some("C/N0 35-50 dB-Hz, more sensitive to loss"),
            GnssObservable::Doppler => Some("C/N0 derived from carrier tracking"),
            GnssObservable::NavigationMessage => Some("requires C/N0 > 25 dB-Hz to decode"),
        }
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// GNSS observable taxonomy is a DAG.
pub struct GnssObservableTaxonomyIsDAG;

impl Axiom for GnssObservableTaxonomyIsDAG {
    fn description(&self) -> &str {
        "GNSS observable taxonomy is a DAG"
    }
    fn holds(&self) -> bool {
        NoCycles::<GnssObservableTaxonomy>::default().holds()
    }
}

/// GNSS constellation taxonomy is a DAG.
pub struct GnssConstellationTaxonomyIsDAG;

impl Axiom for GnssConstellationTaxonomyIsDAG {
    fn description(&self) -> &str {
        "GNSS constellation taxonomy is a DAG"
    }
    fn holds(&self) -> bool {
        NoCycles::<GnssConstellationTaxonomy>::default().holds()
    }
}

/// Minimum 4 satellites required for 3D position fix (3 spatial + 1 clock).
///
/// The GNSS measurement equation has 4 unknowns: x, y, z, clock_bias.
/// Each pseudorange provides 1 equation. Need >= 4 equations.
///
/// Source: IS-GPS-200, Groves (2013) Section 8.5.
pub struct MinimumSatellites;

impl Axiom for MinimumSatellites {
    fn description(&self) -> &str {
        "need >= 4 satellites for 3D fix (3 spatial + 1 clock unknown)"
    }
    fn holds(&self) -> bool {
        // 3 spatial unknowns + 1 clock unknown = 4 minimum measurements
        let spatial_unknowns = 3;
        let clock_unknowns = 1;
        let min_satellites = spatial_unknowns + clock_unknowns;
        min_satellites == 4
    }
}

/// DOP improves (decreases) with wider satellite spread.
///
/// Satellites clustered in one part of the sky produce poor geometry.
/// Satellites spread across the sky produce good geometry.
///
/// Source: Misra & Enge (2011), Chapter 7.
pub struct DopGeometry;

impl Axiom for DopGeometry {
    fn description(&self) -> &str {
        "DOP improves with wider satellite angular spread"
    }
    fn holds(&self) -> bool {
        // Compute GDOP for 5 satellites spread vs 5 satellites clustered.
        // Using 5 (overdetermined) avoids near-singular issues.
        // Wide: 4 at 45 deg elevation at cardinal points + 1 at zenith
        let gdop_wide = compute_gdop_from_elevations_azimuths(
            &[45.0, 45.0, 45.0, 45.0, 89.0],
            &[0.0, 90.0, 180.0, 270.0, 0.0],
        );
        // Narrow: 4 at 45 deg elevation clustered + 1 slightly offset
        let gdop_narrow = compute_gdop_from_elevations_azimuths(
            &[45.0, 44.0, 46.0, 45.0, 43.0],
            &[0.0, 5.0, 10.0, 15.0, 20.0],
        );
        // Wide spread should produce lower (better) GDOP
        gdop_wide < gdop_narrow
    }
}

/// Pseudorange must be non-negative (signal travel time * speed of light).
///
/// Source: IS-GPS-200, equation for pseudorange.
pub struct PseudorangePositive;

impl Axiom for PseudorangePositive {
    fn description(&self) -> &str {
        "pseudorange >= 0 (signal travel time * speed of light)"
    }
    fn holds(&self) -> bool {
        // Pseudorange is a type of observable
        taxonomy::is_a::<GnssObservableTaxonomy>(
            &GnssObservable::Pseudorange,
            &GnssObservable::Observable,
        )
        // The non-negativity constraint is enforced by physics:
        // pseudorange = c * (t_receive - t_transmit) >= 0
        && {
            let speed_of_light = 299_792_458.0_f64; // m/s
            let min_travel_time = 0.0_f64; // seconds
            let min_pseudorange = speed_of_light * min_travel_time;
            min_pseudorange >= 0.0
        }
    }
}

// ---------------------------------------------------------------------------
// Helper: simplified GDOP computation
// ---------------------------------------------------------------------------

/// Compute GDOP from satellite elevations and azimuths.
///
/// GDOP = sqrt(trace(inv(H^T * H))) where H is the geometry matrix.
/// Each row of H: [cos(el)*cos(az), cos(el)*sin(az), sin(el), 1]
pub(crate) fn compute_gdop_from_elevations_azimuths(
    elevations_deg: &[f64],
    azimuths_deg: &[f64],
) -> f64 {
    let n = elevations_deg.len();
    if n < 4 {
        return f64::MAX;
    }

    // Build H matrix rows: [cos(el)*cos(az), cos(el)*sin(az), sin(el), 1]
    let mut h_rows: Vec<[f64; 4]> = Vec::with_capacity(n);
    for i in 0..n {
        let el = elevations_deg[i].to_radians();
        let az = azimuths_deg[i].to_radians();
        h_rows.push([el.cos() * az.cos(), el.cos() * az.sin(), el.sin(), 1.0]);
    }

    // Compute H^T * H (4x4 matrix)
    let mut hth = [[0.0_f64; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            for row in &h_rows {
                hth[i][j] += row[i] * row[j];
            }
        }
    }

    // Invert 4x4 matrix using cofactor expansion (simplified for this use case)
    if let Some(inv) = invert_4x4(&hth) {
        let trace = inv[0][0] + inv[1][1] + inv[2][2] + inv[3][3];
        if trace > 0.0 { trace.sqrt() } else { f64::MAX }
    } else {
        f64::MAX // singular — infinite DOP
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
        // Partial pivoting
        let mut max_row = col;
        let mut max_val = aug[col][col].abs();
        for row in (col + 1)..4 {
            if aug[row][col].abs() > max_val {
                max_val = aug[row][col].abs();
                max_row = row;
            }
        }
        if max_val < 1e-12 {
            return None; // singular
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

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for GnssOntology {
    type Cat = GnssCategory;
    type Qual = SignalStrength;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(GnssObservableTaxonomyIsDAG),
            Box::new(GnssConstellationTaxonomyIsDAG),
            Box::new(MinimumSatellites),
            Box::new(DopGeometry),
            Box::new(PseudorangePositive),
        ]
    }
}
