# praxis-legal

[![crates.io](https://img.shields.io/crates/v/praxis-legal.svg)](https://crates.io/crates/praxis-legal)
[![docs.rs](https://img.shields.io/docsrs/praxis-legal)](https://docs.rs/praxis-legal)

Legal ontology and case lifecycle enforcement -- evidence, authority, claims, motions, rulings.

Part of the [Praxis](https://github.com/i-am-logger/praxis) framework.

## Overview

Models the legal domain as typed state machines with enforced transitions. Cases progress through phases (pre-filing through appeal/closure) with validated actions at each step. Motions follow their own lifecycle (pending, opposed, under advisement, ruled). Evidence is structured as sourced facts with severity, narrative chains, and temporal proximity analysis. Legal authority is hierarchical -- from constitutional provisions down to professional body rules -- with weighted precedent tracking.

## Key Types

| Type | Description |
|---|---|
| `Case` | Full litigation lifecycle container with phase, motions, rulings, and event history |
| `CasePhase` | State machine: PreFiling, Filed, Discovery, Motions, PreTrial, Trial, PostTrial, Appeal, Closed |
| `CaseAction` | Actions that advance a case: File, BeginDiscovery, FileMotion, Settle, Dismiss, etc. |
| `Decision` | A motion before the court with type, status, arguments, and assessment |
| `MotionStatus` | Motion state machine: Pending, Opposed, UnderAdvisement, Granted, Denied, GrantedInPart, Moot, Withdrawn |
| `Entity` | Legal actors: Person, Corporation, LawFirm, Agency, Court |
| `Fact` | Sourced evidence atom with claim, typed value, date precision, actors, and narrative |
| `Authority` | Legal authority hierarchy: Constitution, Legislature, SupremeCourt, AppellateCourt, Regulation, etc. |
| `Rule` | Compositional legal rules: conditions (AllOf, AnyOf, Not) mapping to consequences with recommendations |
| `Severity` | Ordered severity levels: Info, Low, Medium, High, Critical |

## Example

```rust
use praxis_legal::lifecycle::{Case, CaseAction};
use praxis_legal::entity::{Entity, Court};

let court = Entity::Court(Court {
    name: "N.D. Cal.".into(),
    district: Some("Northern".into()),
    circuit: Some("9th".into()),
});

let mut case = Case::new("Doe v. Acme Corp.");
let result = case.act(CaseAction::File {
    court,
    date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
});
assert!(result.is_ok());
```

## License

CC BY-NC-SA 4.0
