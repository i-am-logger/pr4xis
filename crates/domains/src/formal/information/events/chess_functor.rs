#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::entity::Concept;
use pr4xis::category::{Arrow, Category, Functor};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

use super::ontology::*;

/// Chess as an event-driven system.
///
/// Chess IS event-driven: moves are events (immutable facts),
/// the board state is derived from the move history (event sourcing),
/// and players react to events (the opponent's move triggers your response).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChessEvent {
    /// A chess move — the fundamental event ("e4", "Nf3", "O-O").
    Move,
    /// The intention to move — may be illegal (rejected command).
    MoveAttempt,
    /// The board position — derived from move history.
    Position,
    /// The rules engine — validates and processes moves.
    RulesEngine,
    /// The game record — PGN is literally an event log.
    GameRecord,
    /// The game — routes moves between players.
    Game,
    /// A view of the position (e.g., material count, evaluation).
    Evaluation,
    /// Watching for specific positions (e.g., checkmate detection).
    CheckDetection,
    /// A complete game sequence (opening, middlegame, endgame).
    GamePhase,
    /// Move notation rules (PGN, algebraic).
    NotationRules,
}

impl Concept for ChessEvent {
    fn variants() -> Vec<Self> {
        vec![
            Self::Move,
            Self::MoveAttempt,
            Self::Position,
            Self::RulesEngine,
            Self::GameRecord,
            Self::Game,
            Self::Evaluation,
            Self::CheckDetection,
            Self::GamePhase,
            Self::NotationRules,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChessEventRelation {
    pub from: ChessEvent,
    pub to: ChessEvent,
    pub kind: ChessEventRelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChessEventRelationKind {
    Identity,
    Triggers,
    AppendedTo,
    ReactsTo,
    Routes,
    Changes,
    DerivedFrom,
    ListensTo,
    Composes,
    Defines,
    Composed,
}

impl Arrow for ChessEventRelation {
    type Object = ChessEvent;
    type Kind = ChessEventRelationKind;
    fn source(&self) -> ChessEvent {
        self.from
    }
    fn target(&self) -> ChessEvent {
        self.to
    }
    fn kind(&self) -> ChessEventRelationKind {
        self.kind
    }
    fn meta(&self) -> Provenance {
        Provenance {
            name: OntologyName::new_static("ChessEventRelation"),
            description: Label::new_static(
                "Chess as event-driven system: moves are immutable events appended to the game record; rules engine reacts to events; positions derived from event history.",
            ),
            citation: Citation::parse_static(
                "Fowler (2005) Event Sourcing, martinfowler.com; Young (2010) CQRS Documents; Hewitt (1973) A Universal Modular ACTOR Formalism for AI, IJCAI-73",
            ),
            module_path: ModulePath::new_static(module_path!()),
        }
    }
}

pub struct ChessEventCategory;

impl Category for ChessEventCategory {
    type Object = ChessEvent;
    type Morphism = ChessEventRelation;

    fn identity(obj: &ChessEvent) -> ChessEventRelation {
        ChessEventRelation {
            from: *obj,
            to: *obj,
            kind: ChessEventRelationKind::Identity,
        }
    }

    fn compose(f: &ChessEventRelation, g: &ChessEventRelation) -> Option<ChessEventRelation> {
        if f.to != g.from {
            return None;
        }
        if f.kind == ChessEventRelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == ChessEventRelationKind::Identity {
            return Some(f.clone());
        }
        Some(ChessEventRelation {
            from: f.from,
            to: g.to,
            kind: ChessEventRelationKind::Composed,
        })
    }

    fn morphisms() -> Vec<ChessEventRelation> {
        use ChessEvent::*;
        use ChessEventRelationKind::*;

        let mut m = Vec::new();

        for c in ChessEvent::variants() {
            m.push(ChessEventRelation {
                from: c,
                to: c,
                kind: Identity,
            });
        }

        m.push(ChessEventRelation {
            from: MoveAttempt,
            to: Move,
            kind: Triggers,
        });
        m.push(ChessEventRelation {
            from: Move,
            to: GameRecord,
            kind: AppendedTo,
        });
        m.push(ChessEventRelation {
            from: RulesEngine,
            to: Move,
            kind: ReactsTo,
        });
        m.push(ChessEventRelation {
            from: Game,
            to: RulesEngine,
            kind: Routes,
        });
        m.push(ChessEventRelation {
            from: Move,
            to: Position,
            kind: Changes,
        });
        m.push(ChessEventRelation {
            from: Evaluation,
            to: GameRecord,
            kind: DerivedFrom,
        });
        m.push(ChessEventRelation {
            from: CheckDetection,
            to: Game,
            kind: ListensTo,
        });
        m.push(ChessEventRelation {
            from: GamePhase,
            to: Move,
            kind: Composes,
        });
        m.push(ChessEventRelation {
            from: NotationRules,
            to: Move,
            kind: Defines,
        });

        // Transitive
        m.push(ChessEventRelation {
            from: MoveAttempt,
            to: Position,
            kind: Composed,
        });
        m.push(ChessEventRelation {
            from: MoveAttempt,
            to: GameRecord,
            kind: Composed,
        });
        m.push(ChessEventRelation {
            from: Game,
            to: Move,
            kind: Composed,
        });
        m.push(ChessEventRelation {
            from: CheckDetection,
            to: RulesEngine,
            kind: Composed,
        });
        m.push(ChessEventRelation {
            from: GamePhase,
            to: Position,
            kind: Composed,
        });
        m.push(ChessEventRelation {
            from: GamePhase,
            to: GameRecord,
            kind: Composed,
        });

        // Self-composed
        for c in ChessEvent::variants() {
            m.push(ChessEventRelation {
                from: c,
                to: c,
                kind: Composed,
            });
        }

        m
    }
}

/// Functor: Chess → EventDriven. Proves chess IS event-driven.
pub struct ChessToEvents;

impl Functor for ChessToEvents {
    type Source = ChessEventCategory;
    type Target = EventCategory;

    fn map_object(obj: &ChessEvent) -> EventConcept {
        match obj {
            ChessEvent::Move => EventConcept::Event,
            ChessEvent::MoveAttempt => EventConcept::Command,
            ChessEvent::Position => EventConcept::State,
            ChessEvent::RulesEngine => EventConcept::Handler,
            ChessEvent::GameRecord => EventConcept::EventLog,
            ChessEvent::Game => EventConcept::EventBus,
            ChessEvent::Evaluation => EventConcept::Projection,
            ChessEvent::CheckDetection => EventConcept::Subscription,
            ChessEvent::GamePhase => EventConcept::Saga,
            ChessEvent::NotationRules => EventConcept::EventSchema,
        }
    }

    fn map_morphism(m: &ChessEventRelation) -> EventRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        // Per #166: EventRelationKind (auto-generated) no longer emits a
        // Composed variant. Source-side ChessEventRelationKind::Composed
        // collapses to the target's Identity (heterogeneous composition is
        // partial — the functor preserves identity and the typed direct
        // edges; transitive paths are reconstructed by the target's
        // compose()).
        let kind = if m.kind == ChessEventRelationKind::Identity {
            EventRelationKind::Identity
        } else if m.kind == ChessEventRelationKind::Composed || from == to {
            EventRelationKind::Identity
        } else {
            match m.kind {
                ChessEventRelationKind::Triggers => EventRelationKind::Triggers,
                ChessEventRelationKind::AppendedTo => EventRelationKind::AppendedTo,
                ChessEventRelationKind::ReactsTo => EventRelationKind::ReactsTo,
                ChessEventRelationKind::Routes => EventRelationKind::Routes,
                ChessEventRelationKind::Changes => EventRelationKind::Changes,
                ChessEventRelationKind::DerivedFrom => EventRelationKind::DerivedFrom,
                ChessEventRelationKind::ListensTo => EventRelationKind::ListensTo,
                ChessEventRelationKind::Composes => EventRelationKind::Composes,
                ChessEventRelationKind::Defines => EventRelationKind::Defines,
                _ => EventRelationKind::Identity,
            }
        };
        EventRelation { from, to, kind }
    }
}
pr4xis::register_functor!(ChessToEvents);
