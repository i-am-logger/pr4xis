use praxis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};

/// Two jugs with different capacities. Measure a target amount.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct State {
    pub jug_a: u32,
    pub jug_b: u32,
    pub cap_a: u32,
    pub cap_b: u32,
    pub target: u32,
}

impl State {
    pub fn new(cap_a: u32, cap_b: u32, target: u32) -> Self {
        Self {
            jug_a: 0,
            jug_b: 0,
            cap_a,
            cap_b,
            target,
        }
    }
}

impl Situation for State {
    fn describe(&self) -> String {
        format!(
            "A={}/{} B={}/{} target={}",
            self.jug_a, self.cap_a, self.jug_b, self.cap_b, self.target
        )
    }
    fn is_terminal(&self) -> bool {
        self.jug_a == self.target || self.jug_b == self.target
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JugAction {
    FillA,
    FillB,
    EmptyA,
    EmptyB,
    PourAtoB,
    PourBtoA,
}

impl Action for JugAction {
    type Sit = State;
    fn describe(&self) -> String {
        format!("{:?}", self)
    }
}

struct NoOp;
impl Precondition<JugAction> for NoOp {
    fn check(&self, s: &State, a: &JugAction) -> PreconditionResult {
        let next = apply_jug(s, a);
        if next.jug_a == s.jug_a && next.jug_b == s.jug_b {
            PreconditionResult::violated(
                "no_op",
                "action has no effect",
                &s.describe(),
                &a.describe(),
            )
        } else {
            PreconditionResult::satisfied("no_op", "state changed")
        }
    }
    fn describe(&self) -> &str {
        "action must change the state"
    }
}

fn apply_jug(s: &State, a: &JugAction) -> State {
    let mut n = s.clone();
    match a {
        JugAction::FillA => n.jug_a = n.cap_a,
        JugAction::FillB => n.jug_b = n.cap_b,
        JugAction::EmptyA => n.jug_a = 0,
        JugAction::EmptyB => n.jug_b = 0,
        JugAction::PourAtoB => {
            let pour = n.jug_a.min(n.cap_b - n.jug_b);
            n.jug_a -= pour;
            n.jug_b += pour;
        }
        JugAction::PourBtoA => {
            let pour = n.jug_b.min(n.cap_a - n.jug_a);
            n.jug_b -= pour;
            n.jug_a += pour;
        }
    }
    n
}

/// Classic: 3L and 5L jugs, measure 4L.
pub fn new_classic() -> Engine<JugAction> {
    Engine::new(State::new(3, 5, 4), vec![Box::new(NoOp)], apply_jug)
}

pub fn new_puzzle(cap_a: u32, cap_b: u32, target: u32) -> Engine<JugAction> {
    Engine::new(
        State::new(cap_a, cap_b, target),
        vec![Box::new(NoOp)],
        apply_jug,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn arb_action() -> impl Strategy<Value = JugAction> {
        prop_oneof![
            Just(JugAction::FillA),
            Just(JugAction::FillB),
            Just(JugAction::EmptyA),
            Just(JugAction::EmptyB),
            Just(JugAction::PourAtoB),
            Just(JugAction::PourBtoA),
        ]
    }

    #[test]
    fn test_classic_solution() {
        let e = new_classic()
            .next(JugAction::FillB)
            .unwrap() // A=0 B=5
            .next(JugAction::PourBtoA)
            .unwrap() // A=3 B=2
            .next(JugAction::EmptyA)
            .unwrap() // A=0 B=2
            .next(JugAction::PourBtoA)
            .unwrap() // A=2 B=0
            .next(JugAction::FillB)
            .unwrap() // A=2 B=5
            .next(JugAction::PourBtoA)
            .unwrap(); // A=3 B=4
        assert!(e.is_terminal());
        assert_eq!(e.situation().jug_b, 4);
    }

    #[test]
    fn test_no_op_blocked() {
        let e = new_classic();
        assert!(e.next(JugAction::EmptyA).is_err()); // already empty
    }

    proptest! {
        /// Jugs never exceed capacity
        #[test]
        fn prop_never_overflow(actions in proptest::collection::vec(arb_action(), 0..20)) {
            let mut e = new_classic();
            for a in actions {
                match e.next(a) {
                    Ok(next) => {
                        prop_assert!(next.situation().jug_a <= next.situation().cap_a);
                        prop_assert!(next.situation().jug_b <= next.situation().cap_b);
                        e = next;
                    }
                    Err((prev, _)) => { e = prev; }
                }
            }
        }

        /// Total water is conserved (except fill/empty)
        #[test]
        fn prop_pour_conserves(a_start in 0..4u32, b_start in 0..6u32) {
            let mut s = State::new(3, 5, 4);
            s.jug_a = a_start.min(3);
            s.jug_b = b_start.min(5);
            let total = s.jug_a + s.jug_b;
            let poured = apply_jug(&s, &JugAction::PourAtoB);
            prop_assert_eq!(poured.jug_a + poured.jug_b, total);
            let poured = apply_jug(&s, &JugAction::PourBtoA);
            prop_assert_eq!(poured.jug_a + poured.jug_b, total);
        }
    }
}
