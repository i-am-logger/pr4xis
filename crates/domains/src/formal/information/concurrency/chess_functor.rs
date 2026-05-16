#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::entity::Concept;
use pr4xis::category::{Arrow, Category, Functor};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

use super::ontology::*;

/// Chess modeled as a concurrent system.
///
/// Two players (agents) sharing a board (resource),
/// coordinated by turn-taking (synchronization),
/// governed by rules (protocol).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChessConcurrent {
    /// White or Black — the two agents.
    Player,
    /// The chess board — shared resource both players act on.
    Board,
    /// A chess move — action by one agent.
    Move,
    /// Turn-taking — White goes, then Black, then White...
    TurnTaking,
    /// The current position — all pieces, whose turn, castling rights, etc.
    Position,
    /// Legal move rules — what moves are allowed.
    Rules,
    /// Stalemate — a form of deadlock (neither player can win).
    Stalemate,
    /// Time pressure — race condition analog (in timed games).
    TimePressure,
    /// The opponent's response — a future that resolves when they move.
    OpponentResponse,
    /// The move notation — the message announcing the action (e.g., "Nf3").
    MoveNotation,
}

impl Concept for ChessConcurrent {
    fn variants() -> Vec<Self> {
        vec![
            Self::Player,
            Self::Board,
            Self::Move,
            Self::TurnTaking,
            Self::Position,
            Self::Rules,
            Self::Stalemate,
            Self::TimePressure,
            Self::OpponentResponse,
            Self::MoveNotation,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChessConcurrentRelation {
    pub from: ChessConcurrent,
    pub to: ChessConcurrent,
    pub kind: ChessConcurrentRelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChessConcurrentRelationKind {
    Identity,
    ActsOn,
    Controls,
    Governs,
    Changes,
    Produces,
    Becomes,
    ArisesFrom,
    UnsynchronizedAccess,
    Composed,
}

impl Arrow for ChessConcurrentRelation {
    type Object = ChessConcurrent;
    type Kind = ChessConcurrentRelationKind;
    fn source(&self) -> ChessConcurrent {
        self.from
    }
    fn target(&self) -> ChessConcurrent {
        self.to
    }
    fn kind(&self) -> ChessConcurrentRelationKind {
        self.kind
    }
    fn meta(&self) -> Provenance {
        Provenance {
            name: OntologyName::new_static("ChessConcurrentRelation"),
            description: Label::new_static(
                "Chess as concurrent system: players (agents) sharing the board (resource) under turn-taking (synchronization).",
            ),
            citation: Citation::parse_static(
                "Hoare (1978) Communicating Sequential Processes, CACM 21(8); Hewitt (1973) A Universal Modular ACTOR Formalism for AI, IJCAI-73; Lamport (1978) Time, Clocks, and the Ordering of Events in a Distributed System, CACM 21(7)",
            ),
            module_path: ModulePath::new_static(module_path!()),
        }
    }
}

pub struct ChessConcurrentCategory;

impl Category for ChessConcurrentCategory {
    type Object = ChessConcurrent;
    type Morphism = ChessConcurrentRelation;

    fn identity(obj: &ChessConcurrent) -> ChessConcurrentRelation {
        ChessConcurrentRelation {
            from: *obj,
            to: *obj,
            kind: ChessConcurrentRelationKind::Identity,
        }
    }

    fn compose(
        f: &ChessConcurrentRelation,
        g: &ChessConcurrentRelation,
    ) -> Option<ChessConcurrentRelation> {
        if f.to != g.from {
            return None;
        }
        if f.kind == ChessConcurrentRelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == ChessConcurrentRelationKind::Identity {
            return Some(f.clone());
        }
        Some(ChessConcurrentRelation {
            from: f.from,
            to: g.to,
            kind: ChessConcurrentRelationKind::Composed,
        })
    }

    fn morphisms() -> Vec<ChessConcurrentRelation> {
        use ChessConcurrent::*;
        use ChessConcurrentRelationKind::*;

        let mut m = Vec::new();

        for c in ChessConcurrent::variants() {
            m.push(ChessConcurrentRelation {
                from: c,
                to: c,
                kind: Identity,
            });
        }

        // Player acts on Board
        m.push(ChessConcurrentRelation {
            from: Player,
            to: Board,
            kind: ActsOn,
        });
        // TurnTaking controls Player (whose turn)
        m.push(ChessConcurrentRelation {
            from: TurnTaking,
            to: Player,
            kind: Controls,
        });
        // Rules govern Move
        m.push(ChessConcurrentRelation {
            from: Rules,
            to: Move,
            kind: Governs,
        });
        // Move changes Position
        m.push(ChessConcurrentRelation {
            from: Move,
            to: Position,
            kind: Changes,
        });
        // Move produces MoveNotation ("e4", "Nf3")
        m.push(ChessConcurrentRelation {
            from: Move,
            to: MoveNotation,
            kind: Produces,
        });
        // MoveNotation becomes OpponentResponse (waiting for reply)
        m.push(ChessConcurrentRelation {
            from: MoveNotation,
            to: OpponentResponse,
            kind: Becomes,
        });
        // Stalemate arises from TurnTaking (no legal moves for active player)
        m.push(ChessConcurrentRelation {
            from: TurnTaking,
            to: Stalemate,
            kind: ArisesFrom,
        });
        // TimePressure arises from Board (clock is shared resource)
        m.push(ChessConcurrentRelation {
            from: Board,
            to: TimePressure,
            kind: UnsynchronizedAccess,
        });

        // Transitive
        m.push(ChessConcurrentRelation {
            from: TurnTaking,
            to: Board,
            kind: Composed,
        });
        m.push(ChessConcurrentRelation {
            from: Rules,
            to: Position,
            kind: Composed,
        });
        m.push(ChessConcurrentRelation {
            from: Rules,
            to: MoveNotation,
            kind: Composed,
        });
        m.push(ChessConcurrentRelation {
            from: Player,
            to: TimePressure,
            kind: Composed,
        });
        m.push(ChessConcurrentRelation {
            from: Move,
            to: OpponentResponse,
            kind: Composed,
        });

        m
    }
}

/// Functor: Chess → Concurrency.
/// Proves chess IS a concurrent system.
pub struct ChessToConcurrency;

impl Functor for ChessToConcurrency {
    type Source = ChessConcurrentCategory;
    type Target = ConcurrencyCategory;

    fn map_object(obj: &ChessConcurrent) -> ConcurrencyConcept {
        match obj {
            ChessConcurrent::Player => ConcurrencyConcept::Agent,
            ChessConcurrent::Board => ConcurrencyConcept::SharedResource,
            ChessConcurrent::Move => ConcurrencyConcept::Action,
            ChessConcurrent::TurnTaking => ConcurrencyConcept::Synchronization,
            ChessConcurrent::Position => ConcurrencyConcept::State,
            ChessConcurrent::Rules => ConcurrencyConcept::Protocol,
            ChessConcurrent::Stalemate => ConcurrencyConcept::Deadlock,
            ChessConcurrent::TimePressure => ConcurrencyConcept::RaceCondition,
            ChessConcurrent::OpponentResponse => ConcurrencyConcept::Future,
            ChessConcurrent::MoveNotation => ConcurrencyConcept::Message,
        }
    }

    fn map_morphism(m: &ChessConcurrentRelation) -> ConcurrencyRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = match m.kind {
            ChessConcurrentRelationKind::Identity => ConcurrencyRelationKind::Identity,
            ChessConcurrentRelationKind::ActsOn => ConcurrencyRelationKind::ActsOn,
            ChessConcurrentRelationKind::Controls => ConcurrencyRelationKind::Controls,
            ChessConcurrentRelationKind::Governs => ConcurrencyRelationKind::Governs,
            ChessConcurrentRelationKind::Changes => ConcurrencyRelationKind::Changes,
            ChessConcurrentRelationKind::Produces => ConcurrencyRelationKind::Produces,
            ChessConcurrentRelationKind::Becomes => ConcurrencyRelationKind::Becomes,
            ChessConcurrentRelationKind::ArisesFrom => ConcurrencyRelationKind::ArisesFrom,
            ChessConcurrentRelationKind::UnsynchronizedAccess => {
                ConcurrencyRelationKind::UnsynchronizedAccess
            }
            ChessConcurrentRelationKind::Composed => ConcurrencyRelationKind::Identity,
        };
        ConcurrencyRelation { from, to, kind }
    }
}
pr4xis::register_functor!(ChessToConcurrency);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    /// Daubert prong 2 (testable methodology): the cross-ontology
    /// mapping must preserve identity and composition — the
    /// categorical functor laws.
    #[test]
    fn functor_laws() {
        assert_functor_laws::<ChessToConcurrency>();
    }
}
