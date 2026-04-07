use praxis_engine::{Action, Engine, Precondition, PreconditionResult, Situation};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Bank {
    Left,
    Right,
}

impl Bank {
    pub fn opposite(&self) -> Bank {
        match self {
            Bank::Left => Bank::Right,
            Bank::Right => Bank::Left,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct State {
    pub farmer: Bank,
    pub wolf: Bank,
    pub goat: Bank,
    pub cabbage: Bank,
}

impl State {
    pub fn initial() -> Self {
        Self {
            farmer: Bank::Left,
            wolf: Bank::Left,
            goat: Bank::Left,
            cabbage: Bank::Left,
        }
    }

    pub fn is_safe(&self) -> bool {
        (self.wolf != self.goat || self.farmer == self.wolf)
            && (self.goat != self.cabbage || self.farmer == self.goat)
    }
}

impl Situation for State {
    fn describe(&self) -> String {
        format!(
            "F:{:?} W:{:?} G:{:?} C:{:?}",
            self.farmer, self.wolf, self.goat, self.cabbage
        )
    }
    fn is_terminal(&self) -> bool {
        self.farmer == Bank::Right
            && self.wolf == Bank::Right
            && self.goat == Bank::Right
            && self.cabbage == Bank::Right
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Crossing {
    Alone,
    WithWolf,
    WithGoat,
    WithCabbage,
}

impl Action for Crossing {
    type Sit = State;
    fn describe(&self) -> String {
        match self {
            Crossing::Alone => "cross alone".into(),
            Crossing::WithWolf => "cross with wolf".into(),
            Crossing::WithGoat => "cross with goat".into(),
            Crossing::WithCabbage => "cross with cabbage".into(),
        }
    }
}

struct ItemWithFarmer;
impl Precondition<Crossing> for ItemWithFarmer {
    fn check(&self, s: &State, a: &Crossing) -> PreconditionResult {
        let bank = match a {
            Crossing::Alone => return PreconditionResult::satisfied("item_with_farmer", "solo"),
            Crossing::WithWolf => s.wolf,
            Crossing::WithGoat => s.goat,
            Crossing::WithCabbage => s.cabbage,
        };
        if bank == s.farmer {
            PreconditionResult::satisfied("item_with_farmer", "item on farmer's bank")
        } else {
            PreconditionResult::violated(
                "item_with_farmer",
                "item on other bank",
                &s.describe(),
                &a.describe(),
            )
        }
    }
    fn describe(&self) -> &str {
        "item must be on farmer's bank"
    }
}

struct SafeResult;
impl Precondition<Crossing> for SafeResult {
    fn check(&self, s: &State, a: &Crossing) -> PreconditionResult {
        let next = apply_crossing(s, a);
        if next.is_safe() {
            PreconditionResult::satisfied("safe_result", "no one gets eaten")
        } else {
            let reason = if next.wolf == next.goat && next.farmer != next.wolf {
                "wolf would eat goat"
            } else {
                "goat would eat cabbage"
            };
            PreconditionResult::violated("safe_result", reason, &s.describe(), &a.describe())
        }
    }
    fn describe(&self) -> &str {
        "result must be safe"
    }
}

fn apply_crossing(s: &State, a: &Crossing) -> State {
    let mut n = s.clone();
    let dest = s.farmer.opposite();
    n.farmer = dest;
    match a {
        Crossing::Alone => {}
        Crossing::WithWolf => n.wolf = dest,
        Crossing::WithGoat => n.goat = dest,
        Crossing::WithCabbage => n.cabbage = dest,
    }
    n
}

pub fn new_puzzle() -> Engine<Crossing> {
    Engine::new(
        State::initial(),
        vec![Box::new(ItemWithFarmer), Box::new(SafeResult)],
        apply_crossing,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn arb_crossing() -> impl Strategy<Value = Crossing> {
        prop_oneof![
            Just(Crossing::Alone),
            Just(Crossing::WithWolf),
            Just(Crossing::WithGoat),
            Just(Crossing::WithCabbage)
        ]
    }

    #[test]
    fn test_known_solution() {
        let e = new_puzzle()
            .next(Crossing::WithGoat)
            .unwrap()
            .next(Crossing::Alone)
            .unwrap()
            .next(Crossing::WithWolf)
            .unwrap()
            .next(Crossing::WithGoat)
            .unwrap()
            .next(Crossing::WithCabbage)
            .unwrap()
            .next(Crossing::Alone)
            .unwrap()
            .next(Crossing::WithGoat)
            .unwrap();
        assert!(e.is_terminal());
    }

    #[test]
    fn test_wolf_eats_goat_blocked() {
        assert!(new_puzzle().next(Crossing::WithCabbage).is_err());
    }

    #[test]
    fn test_cant_take_from_other_bank() {
        let e = new_puzzle()
            .next(Crossing::WithGoat)
            .unwrap()
            .next(Crossing::Alone)
            .unwrap();
        assert!(e.next(Crossing::WithGoat).is_err());
    }

    proptest! {
        #[test]
        fn prop_always_safe(crossings in proptest::collection::vec(arb_crossing(), 0..20)) {
            let mut e = new_puzzle();
            for c in crossings {
                match e.next(c) {
                    Ok(next) => { prop_assert!(next.situation().is_safe()); e = next; }
                    Err((prev, _)) => { e = prev; }
                }
            }
        }
    }
}
