#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::engine::{Action, Situation};

use crate::applied::navigation::imu::strapdown::{ImuSample, NavState, mechanize};
use crate::formal::math::temporal::duration::Duration;

/// INS situation: the current navigation state.
#[derive(Debug, Clone, PartialEq)]
pub struct InsSituation {
    pub nav_state: NavState,
    pub step: usize,
    pub total_time: Duration,
}

impl Situation for InsSituation {}

/// INS action: process an IMU sample.
#[derive(Debug, Clone)]
pub struct InsAction {
    pub sample: ImuSample,
}

impl Action for InsAction {
    type Sit = InsSituation;
}

/// Apply strapdown mechanization.
pub fn apply_ins(situation: &InsSituation, action: &InsAction) -> Result<InsSituation, String> {
    if action.sample.dt.is_negative() {
        return Err("IMU sample dt must be non-negative".into());
    }
    let new_nav = mechanize(&situation.nav_state, &action.sample);
    Ok(InsSituation {
        nav_state: new_nav,
        step: situation.step + 1,
        total_time: situation.total_time.add(&action.sample.dt),
    })
}
