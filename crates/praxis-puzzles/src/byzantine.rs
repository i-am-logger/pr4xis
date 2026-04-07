use praxis_engine::{Action, Engine, Precondition, PreconditionResult, Situation};

/// Byzantine Generals: N generals must agree on Attack or Retreat.
/// Up to F are traitors who send conflicting messages.
/// Consensus requires N > 3F.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Order {
    Attack,
    Retreat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Loyalty {
    Loyal,
    Traitor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub generals: Vec<Loyalty>,
    /// messages[i][j] = order that general i sent to general j
    pub messages: Vec<Vec<Option<Order>>>,
    /// What each general decides
    pub decisions: Vec<Option<Order>>,
    pub commander_order: Option<Order>,
    pub phase: ByzPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByzPhase {
    CommanderSends,
    LieutenantsSend,
    Decide,
    Resolved,
}

impl State {
    pub fn new(loyalties: Vec<Loyalty>) -> Self {
        let n = loyalties.len();
        Self {
            generals: loyalties,
            messages: vec![vec![None; n]; n],
            decisions: vec![None; n],
            commander_order: None,
            phase: ByzPhase::CommanderSends,
        }
    }

    pub fn n(&self) -> usize {
        self.generals.len()
    }
    pub fn traitor_count(&self) -> usize {
        self.generals
            .iter()
            .filter(|g| **g == Loyalty::Traitor)
            .count()
    }

    /// Did all loyal generals reach the same decision?
    pub fn consensus_reached(&self) -> bool {
        let loyal_decisions: Vec<Order> = self
            .generals
            .iter()
            .zip(self.decisions.iter())
            .filter(|(g, d)| **g == Loyalty::Loyal && d.is_some())
            .map(|(_, d)| d.unwrap())
            .collect();
        if loyal_decisions.is_empty() {
            return true;
        }
        loyal_decisions.iter().all(|d| *d == loyal_decisions[0])
    }
}

impl Situation for State {
    fn describe(&self) -> String {
        let decided: usize = self.decisions.iter().filter(|d| d.is_some()).count();
        format!(
            "n={} traitors={} phase={:?} decided={}/{}",
            self.n(),
            self.traitor_count(),
            self.phase,
            decided,
            self.n()
        )
    }
    fn is_terminal(&self) -> bool {
        self.phase == ByzPhase::Resolved
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ByzAction {
    /// Commander sends order to all lieutenants (traitor commander may lie)
    CommanderBroadcast(Order),
    /// Lieutenant i relays what they received to lieutenant j
    Relay {
        from: usize,
        to: usize,
        claimed_order: Order,
    },
    /// General i makes a decision based on majority
    Decide(usize),
    /// Finalize
    Resolve,
}

impl Action for ByzAction {
    type Sit = State;
    fn describe(&self) -> String {
        match self {
            ByzAction::CommanderBroadcast(o) => format!("commander broadcasts {:?}", o),
            ByzAction::Relay {
                from,
                to,
                claimed_order,
            } => format!(
                "general {} tells {} the order is {:?}",
                from, to, claimed_order
            ),
            ByzAction::Decide(i) => format!("general {} decides", i),
            ByzAction::Resolve => "resolve".into(),
        }
    }
}

struct ByzRules;
impl Precondition<ByzAction> for ByzRules {
    fn check(&self, s: &State, a: &ByzAction) -> PreconditionResult {
        match (s.phase, a) {
            (ByzPhase::CommanderSends, ByzAction::CommanderBroadcast(_)) => {
                PreconditionResult::satisfied("byz_rules", "commander phase")
            }
            (
                ByzPhase::LieutenantsSend,
                ByzAction::Relay {
                    from,
                    to,
                    claimed_order,
                },
            ) => {
                if *from >= s.n() || *to >= s.n() {
                    return PreconditionResult::violated(
                        "byz_rules",
                        "general index out of range",
                        &s.describe(),
                        &a.describe(),
                    );
                }
                // Loyal generals must relay truthfully
                if s.generals[*from] == Loyalty::Loyal
                    && let Some(received) = s.messages[0][*from]
                    && *claimed_order != received
                {
                    return PreconditionResult::violated(
                        "byz_rules",
                        &format!(
                            "loyal general {} must relay truthfully (received {:?})",
                            from, received
                        ),
                        &s.describe(),
                        &a.describe(),
                    );
                }
                PreconditionResult::satisfied("byz_rules", "relay valid")
            }
            (ByzPhase::Decide, ByzAction::Decide(i)) => {
                if *i >= s.n() {
                    PreconditionResult::violated(
                        "byz_rules",
                        "general index out of range",
                        &s.describe(),
                        &a.describe(),
                    )
                } else if s.decisions[*i].is_some() {
                    PreconditionResult::violated(
                        "byz_rules",
                        "already decided",
                        &s.describe(),
                        &a.describe(),
                    )
                } else {
                    PreconditionResult::satisfied("byz_rules", "can decide")
                }
            }
            (ByzPhase::Decide, ByzAction::Resolve) => {
                if s.decisions.iter().all(|d| d.is_some()) {
                    PreconditionResult::satisfied("byz_rules", "all decided")
                } else {
                    PreconditionResult::violated(
                        "byz_rules",
                        "not all generals decided",
                        &s.describe(),
                        &a.describe(),
                    )
                }
            }
            _ => PreconditionResult::violated(
                "byz_rules",
                &format!("{:?} not valid in {:?} phase", a, s.phase),
                &s.describe(),
                &a.describe(),
            ),
        }
    }
    fn describe(&self) -> &str {
        "Byzantine generals protocol rules"
    }
}

fn apply_byz(s: &State, a: &ByzAction) -> State {
    let mut n = s.clone();
    match a {
        ByzAction::CommanderBroadcast(order) => {
            n.commander_order = Some(*order);
            // Commander (general 0) sends to all
            for i in 1..n.n() {
                if n.generals[0] == Loyalty::Loyal {
                    n.messages[0][i] = Some(*order);
                } else {
                    // Traitor commander can send different orders
                    n.messages[0][i] = Some(if i % 2 == 0 {
                        *order
                    } else {
                        match order {
                            Order::Attack => Order::Retreat,
                            Order::Retreat => Order::Attack,
                        }
                    });
                }
            }
            n.phase = ByzPhase::LieutenantsSend;
        }
        ByzAction::Relay {
            from,
            to,
            claimed_order,
        } => {
            n.messages[*from][*to] = Some(*claimed_order);
            // Check if all lieutenants have relayed to all others — advance phase
            let mut all_relayed = true;
            for i in 1..n.n() {
                for j in 1..n.n() {
                    if i != j && n.messages[i][j].is_none() {
                        all_relayed = false;
                    }
                }
            }
            if all_relayed {
                n.phase = ByzPhase::Decide;
            }
        }
        ByzAction::Decide(i) => {
            // Majority vote from all messages received
            let mut attacks = 0u32;
            let mut retreats = 0u32;
            for j in 0..n.n() {
                if let Some(order) = n.messages[j][*i] {
                    match order {
                        Order::Attack => attacks += 1,
                        Order::Retreat => retreats += 1,
                    }
                }
            }
            n.decisions[*i] = Some(if attacks >= retreats {
                Order::Attack
            } else {
                Order::Retreat
            });
        }
        ByzAction::Resolve => {
            n.phase = ByzPhase::Resolved;
        }
    }
    n
}

pub fn new_puzzle(loyalties: Vec<Loyalty>) -> Engine<ByzAction> {
    Engine::new(State::new(loyalties), vec![Box::new(ByzRules)], apply_byz)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_4_generals_1_traitor_consensus() {
        // 4 generals, general 0 is commander (loyal), general 3 is traitor
        let mut e = new_puzzle(vec![
            Loyalty::Loyal,
            Loyalty::Loyal,
            Loyalty::Loyal,
            Loyalty::Traitor,
        ]);
        e = e
            .next(ByzAction::CommanderBroadcast(Order::Attack))
            .unwrap();
        // Loyal lieutenants relay truthfully
        for from in 1..4 {
            for to in 1..4 {
                if from != to {
                    let order = if from == 3 {
                        Order::Retreat
                    } else {
                        Order::Attack
                    }; // traitor lies
                    e = e
                        .next(ByzAction::Relay {
                            from,
                            to,
                            claimed_order: order,
                        })
                        .unwrap();
                }
            }
        }
        e = e.next(ByzAction::Decide(1)).unwrap();
        e = e.next(ByzAction::Decide(2)).unwrap();
        e = e.next(ByzAction::Decide(3)).unwrap();
        // n=4, f=1, n > 3f (4 > 3) — consensus should be reached
        assert!(e.situation().consensus_reached());
    }

    #[test]
    fn test_loyal_must_relay_truthfully() {
        let e = new_puzzle(vec![
            Loyalty::Loyal,
            Loyalty::Loyal,
            Loyalty::Loyal,
            Loyalty::Traitor,
        ])
        .next(ByzAction::CommanderBroadcast(Order::Attack))
        .unwrap();
        // Loyal general 1 received Attack, tries to relay Retreat — should fail
        let result = e.next(ByzAction::Relay {
            from: 1,
            to: 2,
            claimed_order: Order::Retreat,
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_n_gt_3f_required() {
        // 3 generals, 1 traitor: n=3, f=1, n > 3f? 3 > 3 = false
        // Consensus not guaranteed
        let loyalties = vec![Loyalty::Loyal, Loyalty::Loyal, Loyalty::Traitor];
        let state = State::new(loyalties);
        assert_eq!(state.n(), 3);
        assert_eq!(state.traitor_count(), 1);
        // 3 > 3*1 is false — below the threshold
    }

    #[test]
    fn test_wrong_phase_blocked() {
        let e = new_puzzle(vec![Loyalty::Loyal, Loyalty::Loyal, Loyalty::Loyal]);
        assert!(e.next(ByzAction::Decide(0)).is_err()); // need commander broadcast first
    }
}
