use pr4xis::engine::{Action, Situation};

use crate::technology::navigation::imu::strapdown::{ImuSample, NavState, mechanize};

/// INS situation: the current navigation state.
#[derive(Debug, Clone, PartialEq)]
pub struct InsSituation {
    pub nav_state: NavState,
    pub step: usize,
    pub total_time: f64,
}

impl Situation for InsSituation {
    fn describe(&self) -> String {
        format!(
            "INS step={}, t={:.3}s, pos=({:.2},{:.2},{:.2})",
            self.step,
            self.total_time,
            self.nav_state.position.x,
            self.nav_state.position.y,
            self.nav_state.position.z,
        )
    }

    fn is_terminal(&self) -> bool {
        false
    }
}

/// INS action: process an IMU sample.
#[derive(Debug, Clone)]
pub struct InsAction {
    pub sample: ImuSample,
}

impl Action for InsAction {
    type Sit = InsSituation;

    fn describe(&self) -> String {
        format!("IMU(dt={:.4}s)", self.sample.dt)
    }
}

/// Apply strapdown mechanization.
pub fn apply_ins(situation: &InsSituation, action: &InsAction) -> Result<InsSituation, String> {
    if action.sample.dt < 0.0 {
        return Err("IMU sample dt must be non-negative".into());
    }
    let new_nav = mechanize(&situation.nav_state, &action.sample);
    Ok(InsSituation {
        nav_state: new_nav,
        step: situation.step + 1,
        total_time: situation.total_time + action.sample.dt,
    })
}
