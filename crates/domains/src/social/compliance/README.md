# Compliance -- Law of Armed Conflict Engagement Ontology

Models the Escalation of Force (EOF) ladder, IFF classification, and LOAC/ROE engagement rules. The compliance category has escalation levels as objects and permitted transitions as morphisms; the ladder is strictly sequential upward, and de-escalation and abort are available from every state.

Key references:
- Geneva Conventions I-IV (1949) and Additional Protocols I & II (1977)
- Hague Convention (1954) Cultural Property
- US DoD Directive 3000.09 (2023) *Autonomy in Weapon Systems*
- NATO MC 362/1 Rules of Engagement
- NATO STANAG 4162 (IFF)
- US Army FM 5-19 Composite Risk Management

## Entities

| Category | Entities |
|---|---|
| Escalation levels (11) | Observe, Identify, Classify, Alert, Warn, ShowForce, NonLethal, WarningAction, Engage, Deescalate, Abort |
| Classification (4 enums) | IffClassification, EntityType, ProtectedStatus, Confidence |
| Authorization (1 enum) | Autonomous, OperatorNotified, OperatorApproved, CommanderAuthorized |

## Category

Objects are `EscalationLevel` values; morphisms are `EscalationTransition` records. The generated morphism set includes identities, the nine-step ladder `Observe → Identify → Classify → Alert → Warn → ShowForce → NonLethal → WarningAction → Engage`, every-level edges to `Deescalate` and `Abort`, and the transitive closure over the ladder so that multi-step composition resolves.

## Qualities

| Quality | Type | Description |
|---|---|---|
| RequiredAuthorization | Authorization | Each level's minimum authorization: Observe/Identify = Autonomous, Classify/Alert = OperatorNotified, Warn/ShowForce/NonLethal = OperatorApproved, WarningAction/Engage = CommanderAuthorized |

## Axioms (6)

| Axiom | Description | Source |
|---|---|---|
| DistinctionPrinciple | Engagement requires MilitaryObjective classification | Protocol I, Art. 48 |
| CivilianPresumption | Unknown persons are assumed civilian | Protocol I, Art. 50(1) |
| HumanInTheLoop | Persons require human PositiveId for engagement | DoD Directive 3000.09 |
| SequentialEscalation | Escalation must be sequential — no skipping levels | Protocol I, Art. 57 |
| AdvanceWarning | Advance warning before engagement when feasible | Protocol I, Art. 57(2)(c) |
| AbortAlwaysAvailable | Abort is always available from any escalation level | structural |

Plus the auto-generated structural axioms from category laws.

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../docs/use/compose-via-functor.md) to add one.

## Files

- `ontology.rs` -- `ComplianceCategory`, `ComplianceOntology`, `EscalationTransition`, `RequiredAuthorization` quality, tests
- `escalation.rs` -- `EscalationLevel`, `Authorization`, `can_transition`, `EscalationDenial`
- `classification.rs` -- `IffClassification`, `EntityType`, `ProtectedStatus`, `Confidence`, `ClassifiedEntity`
- `law.rs` -- six LOAC axiom implementations (Distinction, Civilian Presumption, Human-in-the-Loop, Sequential Escalation, Advance Warning, Abort Always Available)
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
