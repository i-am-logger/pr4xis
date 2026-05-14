use pr4xis::engine::{Action, Engine, Precondition, Situation};
use pr4xis::logic::proof::{Counterexample, SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

fn axiom_meta(name: &'static str, description: &'static str, citation: &'static str) -> Provenance {
    Provenance {
        name: OntologyName::new_static(name),
        description: Label::new_static(description),
        citation: Citation::parse_static(citation),
        module_path: ModulePath::new_static(module_path!()),
    }
}

const MONTY_CITATION: &str =
    "Selvin (1975) A Problem in Probability (letter), The American Statistician 29(1):67-71";

/// Monty Hall: 3 doors, 1 car, 2 goats.
/// Host always reveals a goat, player can switch.
///
/// Source: Selvin (1975) — original formulation; vos Savant (1990, Parade)
/// popularised the "always switch" answer.
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

    /// The game ends once the player has chosen Stay or Switch.
    pub fn is_terminal(&self) -> bool {
        self.phase == Phase::Resolved
    }
}

impl Situation for State {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MontyAction {
    ChooseDoor(u8),
    HostReveal(u8),
    Stay,
    Switch,
}

impl Action for MontyAction {
    type Sit = State;
}

struct MontyRules;
impl Precondition<MontyAction> for MontyRules {
    fn check(&self, s: &State, a: &MontyAction) -> Verdict {
        let meta = axiom_meta("monty_rules", "Monty Hall game rules", MONTY_CITATION);
        match (s.phase, a) {
            (Phase::ChooseDoor, MontyAction::ChooseDoor(d)) => {
                if *d >= 3 {
                    Err(Box::new(SimpleCounterexample::new(meta)))
                } else {
                    Ok(Box::new(SimpleProof::new(meta)))
                }
            }
            (Phase::HostReveals, MontyAction::HostReveal(d)) => {
                if *d >= 3 {
                    return Err(Box::new(SimpleCounterexample::new(meta)));
                }
                // Host can't reveal the car
                if *d == s.car_door {
                    return Err(Box::new(SimpleCounterexample::new(meta)));
                }
                // Host can't reveal player's choice
                if Some(*d) == s.player_choice {
                    return Err(Box::new(SimpleCounterexample::new(meta)));
                }
                Ok(Box::new(SimpleProof::new(meta)))
            }
            (Phase::SwitchOrStay, MontyAction::Stay) => Ok(Box::new(SimpleProof::new(meta))),
            (Phase::SwitchOrStay, MontyAction::Switch) => Ok(Box::new(SimpleProof::new(meta))),
            _ => Err(Box::new(SimpleCounterexample::new(meta))),
        }
    }
}

fn apply_monty(s: &State, a: &MontyAction) -> Result<State, Box<dyn Counterexample>> {
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
    Ok(n)
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
        assert!(e.situation().is_terminal());
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
            prop_assert!(e.situation().is_terminal());
        }
    }
}
