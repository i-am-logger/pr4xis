use pr4xis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};

/// Prisoner's Dilemma: two players cooperate or defect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    Cooperate,
    Defect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Round {
    pub player_a: Choice,
    pub player_b: Choice,
    pub score_a: i32,
    pub score_b: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub rounds: Vec<Round>,
    pub total_a: i32,
    pub total_b: i32,
    pub pending_a: Option<Choice>,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub fn new() -> Self {
        Self {
            rounds: vec![],
            total_a: 0,
            total_b: 0,
            pending_a: None,
        }
    }

    fn payoff(a: Choice, b: Choice) -> (i32, i32) {
        match (a, b) {
            (Choice::Cooperate, Choice::Cooperate) => (3, 3),
            (Choice::Cooperate, Choice::Defect) => (0, 5),
            (Choice::Defect, Choice::Cooperate) => (5, 0),
            (Choice::Defect, Choice::Defect) => (1, 1),
        }
    }
}

impl Situation for State {
    fn describe(&self) -> String {
        format!(
            "round={} A={} B={} pending={:?}",
            self.rounds.len() + 1,
            self.total_a,
            self.total_b,
            self.pending_a
        )
    }
    fn is_terminal(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrisonerAction {
    PlayerA(Choice),
    PlayerB(Choice),
}

impl Action for PrisonerAction {
    type Sit = State;
    fn describe(&self) -> String {
        match self {
            PrisonerAction::PlayerA(c) => format!("A: {:?}", c),
            PrisonerAction::PlayerB(c) => format!("B: {:?}", c),
        }
    }
}

struct TurnOrder;
impl Precondition<PrisonerAction> for TurnOrder {
    fn check(&self, s: &State, a: &PrisonerAction) -> PreconditionResult {
        match (s.pending_a, a) {
            (None, PrisonerAction::PlayerA(_)) => {
                PreconditionResult::satisfied("turn_order", "A goes first")
            }
            (Some(_), PrisonerAction::PlayerB(_)) => {
                PreconditionResult::satisfied("turn_order", "B goes after A")
            }
            (None, PrisonerAction::PlayerB(_)) => PreconditionResult::violated(
                "turn_order",
                "A must choose first",
                &s.describe(),
                &a.describe(),
            ),
            (Some(_), PrisonerAction::PlayerA(_)) => PreconditionResult::violated(
                "turn_order",
                "A already chose, waiting for B",
                &s.describe(),
                &a.describe(),
            ),
        }
    }
    fn describe(&self) -> &str {
        "A chooses first, then B"
    }
}

fn apply_prisoner(s: &State, a: &PrisonerAction) -> Result<State, String> {
    let mut n = s.clone();
    match a {
        PrisonerAction::PlayerA(c) => {
            n.pending_a = Some(*c);
        }
        PrisonerAction::PlayerB(b_choice) => {
            let a_choice = n.pending_a.unwrap();
            let (sa, sb) = State::payoff(a_choice, *b_choice);
            n.rounds.push(Round {
                player_a: a_choice,
                player_b: *b_choice,
                score_a: sa,
                score_b: sb,
            });
            n.total_a += sa;
            n.total_b += sb;
            n.pending_a = None;
        }
    }
    Ok(n)
}

pub fn new_game() -> Engine<PrisonerAction> {
    Engine::new(State::new(), vec![Box::new(TurnOrder)], apply_prisoner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_mutual_cooperation() {
        let e = new_game()
            .next(PrisonerAction::PlayerA(Choice::Cooperate))
            .unwrap()
            .next(PrisonerAction::PlayerB(Choice::Cooperate))
            .unwrap();
        assert_eq!(e.situation().total_a, 3);
        assert_eq!(e.situation().total_b, 3);
    }

    #[test]
    fn test_betrayal() {
        let e = new_game()
            .next(PrisonerAction::PlayerA(Choice::Cooperate))
            .unwrap()
            .next(PrisonerAction::PlayerB(Choice::Defect))
            .unwrap();
        assert_eq!(e.situation().total_a, 0);
        assert_eq!(e.situation().total_b, 5);
    }

    #[test]
    fn test_mutual_defection() {
        let e = new_game()
            .next(PrisonerAction::PlayerA(Choice::Defect))
            .unwrap()
            .next(PrisonerAction::PlayerB(Choice::Defect))
            .unwrap();
        assert_eq!(e.situation().total_a, 1);
        assert_eq!(e.situation().total_b, 1);
    }

    #[test]
    fn test_b_cant_go_first() {
        assert!(
            new_game()
                .next(PrisonerAction::PlayerB(Choice::Cooperate))
                .is_err()
        );
    }

    proptest! {
        #[test]
        fn prop_payoffs_symmetric(a in prop_oneof![Just(Choice::Cooperate), Just(Choice::Defect)],
                                   b in prop_oneof![Just(Choice::Cooperate), Just(Choice::Defect)]) {
            let (sa, sb) = State::payoff(a, b);
            let (sb2, sa2) = State::payoff(b, a);
            prop_assert_eq!(sa, sa2);
            prop_assert_eq!(sb, sb2);
        }

        #[test]
        fn prop_scores_non_negative(a in prop_oneof![Just(Choice::Cooperate), Just(Choice::Defect)],
                                     b in prop_oneof![Just(Choice::Cooperate), Just(Choice::Defect)]) {
            let (sa, sb) = State::payoff(a, b);
            prop_assert!(sa >= 0);
            prop_assert!(sb >= 0);
        }
    }
}
