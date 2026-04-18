#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::engine::{Action, Situation};

/// A celestial observation: measured altitude and azimuth of a body.
#[derive(Debug, Clone, PartialEq)]
pub struct CelestialObservation {
    /// Name or catalog ID of the celestial body.
    pub body_name: String,
    /// Measured altitude above horizon (degrees).
    pub altitude_deg: f64,
    /// Measured azimuth from north (degrees).
    pub azimuth_deg: f64,
    /// Known declination of the body (degrees).
    pub declination_deg: f64,
    /// Known Greenwich Hour Angle of the body (degrees).
    pub gha_deg: f64,
}

/// Celestial fix: position determined from celestial observations.
#[derive(Debug, Clone, PartialEq)]
pub struct CelestialFix {
    /// Latitude (degrees, north positive).
    pub latitude: f64,
    /// Longitude (degrees, east positive).
    pub longitude: f64,
    /// Number of observations used.
    pub num_observations: usize,
}

/// Celestial navigation situation.
#[derive(Debug, Clone, PartialEq)]
pub struct CelestialSituation {
    /// Accumulated observations.
    pub observations: Vec<CelestialObservation>,
    /// Current fix (if computed).
    pub fix: Option<CelestialFix>,
    /// Assumed position for sight reduction (latitude, longitude in degrees).
    pub assumed_position: (f64, f64),
    /// Step counter.
    pub step: usize,
}

impl Situation for CelestialSituation {
    fn describe(&self) -> String {
        let pos = self
            .fix
            .as_ref()
            .map(|f| format!("({:.4}N, {:.4}E)", f.latitude, f.longitude))
            .unwrap_or_else(|| "no fix".to_string());
        format!(
            "Celestial step={}, obs={}, fix={}",
            self.step,
            self.observations.len(),
            pos
        )
    }

    fn is_terminal(&self) -> bool {
        false
    }
}

/// Celestial navigation action.
#[derive(Debug, Clone)]
pub enum CelestialAction {
    /// Observe a celestial body.
    Observe(CelestialObservation),
    /// Compute a fix from accumulated observations.
    ComputeFix,
}

impl Action for CelestialAction {
    type Sit = CelestialSituation;

    fn describe(&self) -> String {
        match self {
            CelestialAction::Observe(obs) => {
                format!(
                    "observe {} (alt={:.2} az={:.2})",
                    obs.body_name, obs.altitude_deg, obs.azimuth_deg
                )
            }
            CelestialAction::ComputeFix => "compute celestial fix".to_string(),
        }
    }
}

/// Apply a celestial navigation action.
pub fn apply_celestial(
    situation: &CelestialSituation,
    action: &CelestialAction,
) -> Result<CelestialSituation, String> {
    match action {
        CelestialAction::Observe(obs) => {
            if obs.altitude_deg < -90.0 || obs.altitude_deg > 90.0 {
                return Err(format!(
                    "altitude must be in [-90, 90], got {}",
                    obs.altitude_deg
                ));
            }
            let mut new_obs = situation.observations.clone();
            new_obs.push(obs.clone());
            Ok(CelestialSituation {
                observations: new_obs,
                fix: situation.fix.clone(),
                assumed_position: situation.assumed_position,
                step: situation.step + 1,
            })
        }
        CelestialAction::ComputeFix => {
            if situation.observations.len() < 2 {
                return Err(format!(
                    "need >= 2 observations for fix, have {}",
                    situation.observations.len()
                ));
            }
            let fix = compute_celestial_fix(&situation.observations, situation.assumed_position)?;
            Ok(CelestialSituation {
                observations: situation.observations.clone(),
                fix: Some(fix),
                assumed_position: situation.assumed_position,
                step: situation.step + 1,
            })
        }
    }
}

/// Compute a celestial fix using the intercept method (Marcq St. Hilaire).
///
/// For each observation:
///   1. Compute the calculated altitude from the assumed position.
///   2. Intercept = observed altitude - calculated altitude.
///   3. Line of position is perpendicular to the azimuth at distance = intercept.
///
/// With 2+ lines of position, solve for the fix by least squares.
///
/// Source: Bowditch (2002) Chapter 18, Sight Reduction.
fn compute_celestial_fix(
    observations: &[CelestialObservation],
    assumed_position: (f64, f64),
) -> Result<CelestialFix, String> {
    let n = observations.len();
    if n < 2 {
        return Err("need >= 2 observations".into());
    }

    let lat_ap = assumed_position.0.to_radians();
    let lon_ap = assumed_position.1.to_radians();

    // Solve via least squares in local tangent plane (nautical miles from AP)
    let mut ata = [[0.0_f64; 2]; 2];
    let mut atb = [0.0_f64; 2];

    for obs in observations {
        let dec = obs.declination_deg.to_radians();
        let gha = obs.gha_deg.to_radians();
        let lha = gha + lon_ap; // local hour angle

        // Calculated altitude: sin(Hc) = sin(lat)*sin(dec) + cos(lat)*cos(dec)*cos(LHA)
        let sin_hc = lat_ap.sin() * dec.sin() + lat_ap.cos() * dec.cos() * lha.cos();
        let hc_deg = sin_hc.clamp(-1.0, 1.0).asin().to_degrees();

        // Azimuth to the body (for the line of position direction)
        let az_rad = obs.azimuth_deg.to_radians();

        // Intercept: difference between observed and calculated altitude (in nautical miles)
        // 1 arcminute of altitude = 1 nautical mile
        let intercept_nm = (obs.altitude_deg - hc_deg) * 60.0;

        // Line of position: direction perpendicular to azimuth
        // Correction in [north, east] nautical miles:
        //   delta_north = intercept * cos(azimuth)
        //   delta_east  = intercept * sin(azimuth)
        let a_north = az_rad.cos();
        let a_east = az_rad.sin();

        // Accumulate normal equations: A^T A x = A^T b
        ata[0][0] += a_north * a_north;
        ata[0][1] += a_north * a_east;
        ata[1][0] += a_east * a_north;
        ata[1][1] += a_east * a_east;
        atb[0] += a_north * intercept_nm;
        atb[1] += a_east * intercept_nm;
    }

    // Solve 2x2 system
    let det = ata[0][0] * ata[1][1] - ata[0][1] * ata[1][0];
    if det.abs() < 1e-10 {
        return Err("observations are collinear — no unique fix".into());
    }
    let delta_north = (ata[1][1] * atb[0] - ata[0][1] * atb[1]) / det;
    let delta_east = (ata[0][0] * atb[1] - ata[1][0] * atb[0]) / det;

    // Convert nautical miles to degrees
    let lat_fix = assumed_position.0 + delta_north / 60.0;
    let cos_lat = lat_ap.cos();
    if cos_lat.abs() < 1e-10 {
        return Err("celestial fix undefined at poles".into());
    }
    let lon_fix = assumed_position.1 + delta_east / (60.0 * cos_lat);

    Ok(CelestialFix {
        latitude: lat_fix,
        longitude: lon_fix,
        num_observations: n,
    })
}
