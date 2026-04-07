use praxis_engine::{Action, Engine, Precondition, PreconditionResult, Situation};

/// Monty Hall: 3 doors, 1 car, 2 goats.
/// Host always reveals a goat, player can switch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub car_door: u8, // 0-2: which door has the car
    pub player_choice: Option<u8>,
    pub host_revealed: Option<u8>,
    pub final_choice: Option<u8>,
    pub phase: Phase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    ChooseDoor,
    HostReveals,
    SwitchOrStay,
    Resolved,
}

impl State {
    pub fn new(car_door: u8) -> Self {
        assert!(car_door < 3);
        Self {
            car_door,
            player_choice: None,
            host_revealed: None,
            final_choice: None,
            phase: Phase::ChooseDoor,
        }
    }

    pub fn won(&self) -> Option<bool> {
        self.final_choice.map(|c| c == self.car_door)
    }
}

impl Situation for State {
    fn describe(&self) -> String {
        format!(
            "phase={:?} choice={:?} revealed={:?} final={:?}",
            self.phase, self.player_choice, self.host_revealed, self.final_choice
        )
    }
    fn is_terminal(&self) -> bool {
        self.phase == Phase::Resolved
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MontyAction {
    ChooseDoor(u8),
    HostReveal(u8),
    Stay,
    Switch,
}

impl Action for MontyAction {
    type Sit = State;
    fn describe(&self) -> String {
        match self {
            MontyAction::ChooseDoor(d) => format!("choose door {}", d),
            MontyAction::HostReveal(d) => format!("host reveals door {}", d),
            MontyAction::Stay => "stay".into(),
            MontyAction::Switch => "switch".into(),
        }
    }
}

struct MontyRules;
impl Precondition<MontyAction> for MontyRules {
    fn check(&self, s: &State, a: &MontyAction) -> PreconditionResult {
        match (s.phase, a) {
            (Phase::ChooseDoor, MontyAction::ChooseDoor(d)) => {
                if *d >= 3 {
                    PreconditionResult::violated(
                        "monty_rules",
                        "door must be 0-2",
                        &s.describe(),
                        &a.describe(),
                    )
                } else {
                    PreconditionResult::satisfied("monty_rules", "valid door choice")
                }
            }
            (Phase::HostReveals, MontyAction::HostReveal(d)) => {
                if *d >= 3 {
                    return PreconditionResult::violated(
                        "monty_rules",
                        "door must be 0-2",
                        &s.describe(),
                        &a.describe(),
                    );
                }
                // Host can't reveal the car
                if *d == s.car_door {
                    return PreconditionResult::violated(
                        "monty_rules",
                        "host cannot reveal the car",
                        &s.describe(),
                        &a.describe(),
                    );
                }
                // Host can't reveal player's choice
                if Some(*d) == s.player_choice {
                    return PreconditionResult::violated(
                        "monty_rules",
                        "host cannot reveal player's door",
                        &s.describe(),
                        &a.describe(),
                    );
                }
                PreconditionResult::satisfied("monty_rules", "host reveals a goat")
            }
            (Phase::SwitchOrStay, MontyAction::Stay) => {
                PreconditionResult::satisfied("monty_rules", "player stays")
            }
            (Phase::SwitchOrStay, MontyAction::Switch) => {
                PreconditionResult::satisfied("monty_rules", "player switches")
            }
            _ => PreconditionResult::violated(
                "monty_rules",
                &format!("{:?} not valid in {:?} phase", a, s.phase),
                &s.describe(),
                &a.describe(),
            ),
        }
    }
    fn describe(&self) -> &str {
        "Monty Hall game rules"
    }
}

fn apply_monty(s: &State, a: &MontyAction) -> State {
    let mut n = s.clone();
    match a {
        MontyAction::ChooseDoor(d) => {
            n.player_choice = Some(*d);
            n.phase = Phase::HostReveals;
        }
        MontyAction::HostReveal(d) => {
            n.host_revealed = Some(*d);
            n.phase = Phase::SwitchOrStay;
        }
        MontyAction::Stay => {
            n.final_choice = n.player_choice;
            n.phase = Phase::Resolved;
        }
        MontyAction::Switch => {
            // Switch to the door that is neither player's choice nor host's revealed
            let other = (0..3u8)
                .find(|&d| Some(d) != n.player_choice && Some(d) != n.host_revealed)
                .unwrap();
            n.final_choice = Some(other);
            n.phase = Phase::Resolved;
        }
    }
    n
}

pub fn new_game(car_door: u8) -> Engine<MontyAction> {
    Engine::new(
        State::new(car_door),
        vec![Box::new(MontyRules)],
        apply_monty,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_switch_wins_when_initial_wrong() {
        // Car behind door 2, player picks 0, host reveals 1
        let e = new_game(2)
            .next(MontyAction::ChooseDoor(0))
            .unwrap()
            .next(MontyAction::HostReveal(1))
            .unwrap()
            .next(MontyAction::Switch)
            .unwrap();
        assert!(e.is_terminal());
        assert_eq!(e.situation().won(), Some(true));
    }

    #[test]
    fn test_stay_loses_when_initial_wrong() {
        let e = new_game(2)
            .next(MontyAction::ChooseDoor(0))
            .unwrap()
            .next(MontyAction::HostReveal(1))
            .unwrap()
            .next(MontyAction::Stay)
            .unwrap();
        assert_eq!(e.situation().won(), Some(false));
    }

    #[test]
    fn test_stay_wins_when_initial_right() {
        let e = new_game(0)
            .next(MontyAction::ChooseDoor(0))
            .unwrap()
            .next(MontyAction::HostReveal(1))
            .unwrap()
            .next(MontyAction::Stay)
            .unwrap();
        assert_eq!(e.situation().won(), Some(true));
    }

    #[test]
    fn test_host_cant_reveal_car() {
        let e = new_game(1).next(MontyAction::ChooseDoor(0)).unwrap();
        assert!(e.next(MontyAction::HostReveal(1)).is_err()); // door 1 has the car
    }

    #[test]
    fn test_host_cant_reveal_player_choice() {
        let e = new_game(1).next(MontyAction::ChooseDoor(0)).unwrap();
        assert!(e.next(MontyAction::HostReveal(0)).is_err()); // player chose 0
    }

    #[test]
    fn test_wrong_phase_blocked() {
        assert!(new_game(0).next(MontyAction::Stay).is_err());
        assert!(new_game(0).next(MontyAction::HostReveal(1)).is_err());
    }

    proptest! {
        /// Switching wins 2/3 of the time (when initial choice is wrong)
        #[test]
        fn prop_switch_wins_when_wrong(car in 0..3u8, choice in 0..3u8) {
            prop_assume!(car != choice);
            // Host reveals the remaining goat door
            let host_door = (0..3u8).find(|&d| d != car && d != choice).unwrap();
            let e = new_game(car)
                .next(MontyAction::ChooseDoor(choice)).unwrap()
                .next(MontyAction::HostReveal(host_door)).unwrap()
                .next(MontyAction::Switch).unwrap();
            prop_assert_eq!(e.situation().won(), Some(true));
        }

        /// Staying wins only when initial choice was right
        #[test]
        fn prop_stay_wins_iff_right(car in 0..3u8, choice in 0..3u8) {
            // Find a valid host reveal
            let host_door = (0..3u8).find(|&d| d != car && d != choice).unwrap();
            let e = new_game(car)
                .next(MontyAction::ChooseDoor(choice)).unwrap()
                .next(MontyAction::HostReveal(host_door)).unwrap()
                .next(MontyAction::Stay).unwrap();
            prop_assert_eq!(e.situation().won(), Some(car == choice));
        }

        /// Host always has a valid door to reveal
        #[test]
        fn prop_host_always_has_option(car in 0..3u8, choice in 0..3u8) {
            let host_options: Vec<u8> = (0..3u8).filter(|&d| d != car && d != choice).collect();
            prop_assert!(!host_options.is_empty());
        }

        /// Game always reaches terminal after 3 actions
        #[test]
        fn prop_always_terminal(car in 0..3u8, choice in 0..3u8, switch in proptest::bool::ANY) {
            let host_door = (0..3u8).find(|&d| d != car && d != choice).unwrap();
            let e = new_game(car)
                .next(MontyAction::ChooseDoor(choice)).unwrap()
                .next(MontyAction::HostReveal(host_door)).unwrap()
                .next(if switch { MontyAction::Switch } else { MontyAction::Stay }).unwrap();
            prop_assert!(e.is_terminal());
        }
    }
}
