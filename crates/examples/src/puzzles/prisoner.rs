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

const PRISONER_CITATION: &str = "Flood & Dresher (1950, RAND); Tucker (1950) A Two-Person Dilemma, Stanford lecture notes; \
     Axelrod (1984) The Evolution of Cooperation, Basic Books";

/// Prisoner's Dilemma: two players cooperate or defect.
///
/// Source: Flood & Dresher (1950, RAND); named and popularised by Tucker
/// (1950); standard payoffs from Axelrod (1984).
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

    /// The repeated game has no built-in terminal state.
    pub fn is_terminal(&self) -> bool {
        false
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

impl Situation for State {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrisonerAction {
    PlayerA(Choice),
    PlayerB(Choice),
}

impl Action for PrisonerAction {
    type Sit = State;
}

struct TurnOrder;
impl Precondition<PrisonerAction> for TurnOrder {
    fn check(&self, s: &State, a: &PrisonerAction) -> Verdict {
        let meta = axiom_meta("turn_order", "A chooses first, then B", PRISONER_CITATION);
        match (s.pending_a, a) {
            (None, PrisonerAction::PlayerA(_)) => Ok(Box::new(SimpleProof::new(meta))),
            (Some(_), PrisonerAction::PlayerB(_)) => Ok(Box::new(SimpleProof::new(meta))),
            (None, PrisonerAction::PlayerB(_)) => Err(Box::new(SimpleCounterexample::new(meta))),
            (Some(_), PrisonerAction::PlayerA(_)) => Err(Box::new(SimpleCounterexample::new(meta))),
        }
    }
}

fn apply_prisoner(s: &State, a: &PrisonerAction) -> Result<State, Box<dyn Counterexample>> {
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
