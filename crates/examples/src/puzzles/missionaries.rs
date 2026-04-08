use praxis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};

/// Missionaries and cannibals. Boat holds 2.
/// Cannibals can't outnumber missionaries on either bank.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct State {
    pub missionaries_left: u8,
    pub cannibals_left: u8,
    pub boat_left: bool, // true = boat on left bank
}

impl State {
    pub fn initial() -> Self {
        Self {
            missionaries_left: 3,
            cannibals_left: 3,
            boat_left: true,
        }
    }

    pub fn missionaries_right(&self) -> u8 {
        3 - self.missionaries_left
    }
    pub fn cannibals_right(&self) -> u8 {
        3 - self.cannibals_left
    }

    pub fn is_safe(&self) -> bool {
        // On left: missionaries >= cannibals (or no missionaries)
        let safe_left =
            self.missionaries_left == 0 || self.missionaries_left >= self.cannibals_left;
        let safe_right =
            self.missionaries_right() == 0 || self.missionaries_right() >= self.cannibals_right();
        safe_left && safe_right
    }
}

impl Situation for State {
    fn describe(&self) -> String {
        let boat = if self.boat_left { "<boat>" } else { "" };
        format!(
            "[{}M {}C {}] ~~~ [{}M {}C {}]",
            self.missionaries_left,
            self.cannibals_left,
            boat,
            self.missionaries_right(),
            self.cannibals_right(),
            if !self.boat_left { "<boat>" } else { "" }
        )
    }
    fn is_terminal(&self) -> bool {
        self.missionaries_left == 0 && self.cannibals_left == 0
    }
}

/// Move: (missionaries, cannibals) in the boat. At least 1, at most 2 total.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Crossing {
    pub missionaries: u8,
    pub cannibals: u8,
}

impl Crossing {
    pub fn new(m: u8, c: u8) -> Self {
        Self {
            missionaries: m,
            cannibals: c,
        }
    }
    pub fn total(&self) -> u8 {
        self.missionaries + self.cannibals
    }
}

impl Action for Crossing {
    type Sit = State;
    fn describe(&self) -> String {
        format!("cross {}M {}C", self.missionaries, self.cannibals)
    }
}

struct ValidCrossing;
impl Precondition<Crossing> for ValidCrossing {
    fn check(&self, s: &State, a: &Crossing) -> PreconditionResult {
        if a.total() == 0 || a.total() > 2 {
            return PreconditionResult::violated(
                "valid_crossing",
                "boat holds 1-2 people",
                &s.describe(),
                &a.describe(),
            );
        }
        if s.boat_left {
            if a.missionaries > s.missionaries_left || a.cannibals > s.cannibals_left {
                return PreconditionResult::violated(
                    "valid_crossing",
                    "not enough people on left bank",
                    &s.describe(),
                    &a.describe(),
                );
            }
        } else {
            if a.missionaries > s.missionaries_right() || a.cannibals > s.cannibals_right() {
                return PreconditionResult::violated(
                    "valid_crossing",
                    "not enough people on right bank",
                    &s.describe(),
                    &a.describe(),
                );
            }
        }
        PreconditionResult::satisfied("valid_crossing", "crossing is valid")
    }
    fn describe(&self) -> &str {
        "boat holds 1-2, people must be available"
    }
}

struct SafeResult;
impl Precondition<Crossing> for SafeResult {
    fn check(&self, s: &State, a: &Crossing) -> PreconditionResult {
        let next = apply_mc(s, a);
        if next.is_safe() {
            PreconditionResult::satisfied("safe_result", "missionaries safe on both banks")
        } else {
            PreconditionResult::violated(
                "safe_result",
                "cannibals would outnumber missionaries",
                &s.describe(),
                &a.describe(),
            )
        }
    }
    fn describe(&self) -> &str {
        "cannibals can't outnumber missionaries on either bank"
    }
}

fn apply_mc(s: &State, a: &Crossing) -> State {
    let mut n = s.clone();
    if s.boat_left {
        n.missionaries_left = n.missionaries_left.saturating_sub(a.missionaries);
        n.cannibals_left = n.cannibals_left.saturating_sub(a.cannibals);
    } else {
        n.missionaries_left = (n.missionaries_left + a.missionaries).min(3);
        n.cannibals_left = (n.cannibals_left + a.cannibals).min(3);
    }
    n.boat_left = !s.boat_left;
    n
}

pub fn new_puzzle() -> Engine<Crossing> {
    Engine::new(
        State::initial(),
        vec![Box::new(ValidCrossing), Box::new(SafeResult)],
        apply_mc,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_known_solution() {
        let e = new_puzzle()
            .next(Crossing::new(1, 1))
            .unwrap() // 2M2C | 1M1C
            .next(Crossing::new(1, 0))
            .unwrap() // 3M2C | 0M1C
            .next(Crossing::new(0, 2))
            .unwrap() // 3M0C | 0M3C
            .next(Crossing::new(0, 1))
            .unwrap() // 3M1C | 0M2C
            .next(Crossing::new(2, 0))
            .unwrap() // 1M1C | 2M2C
            .next(Crossing::new(1, 1))
            .unwrap() // 2M2C | 1M1C
            .next(Crossing::new(2, 0))
            .unwrap() // 0M2C | 3M1C
            .next(Crossing::new(0, 1))
            .unwrap() // 0M3C | 3M0C
            .next(Crossing::new(0, 2))
            .unwrap() // 0M1C | 3M2C
            .next(Crossing::new(0, 1))
            .unwrap() // 0M2C | 3M1C
            .next(Crossing::new(0, 2))
            .unwrap(); // 0M0C | 3M3C
        assert!(e.is_terminal());
    }

    #[test]
    fn test_cannibals_outnumber_blocked() {
        // Send 2 missionaries, leaving 1M 3C on left
        assert!(new_puzzle().next(Crossing::new(2, 0)).is_err());
    }

    #[test]
    fn test_boat_too_full() {
        assert!(new_puzzle().next(Crossing::new(2, 1)).is_err());
    }

    #[test]
    fn test_empty_boat() {
        assert!(new_puzzle().next(Crossing::new(0, 0)).is_err());
    }

    proptest! {
        #[test]
        fn prop_always_safe(crossings in proptest::collection::vec((0..3u8, 0..3u8), 0..20)) {
            let mut e = new_puzzle();
            for (m, c) in crossings {
                match e.next(Crossing::new(m, c)) {
                    Ok(next) => { prop_assert!(next.situation().is_safe()); e = next; }
                    Err((prev, _)) => { e = prev; }
                }
            }
        }

        #[test]
        fn prop_total_preserved(crossings in proptest::collection::vec((0..3u8, 0..3u8), 0..20)) {
            let mut e = new_puzzle();
            for (m, c) in crossings {
                match e.next(Crossing::new(m, c)) {
                    Ok(next) => {
                        let s = next.situation();
                        prop_assert_eq!(s.missionaries_left + s.missionaries_right(), 3);
                        prop_assert_eq!(s.cannibals_left + s.cannibals_right(), 3);
                        e = next;
                    }
                    Err((prev, _)) => { e = prev; }
                }
            }
        }
    }
}
