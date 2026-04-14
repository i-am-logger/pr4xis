# Judicial -- Legal Terms and Case Lifecycle Ontology

Models legal reasoning at two layers: (1) rich types for legal terms — obligations, deadlines, burdens, remedies, exceptions, evidence requirements, and their typed relations — organised into legal categories; and (2) a case lifecycle category whose objects are phases and whose morphisms are the permitted phase transitions. A `LegalEngine` drives a case through the lifecycle while the term-level registry validates facts against required evidence.

Key references:
- Hart 1961: *The Concept of Law* (primary and secondary rules)
- MacCormick 1978: *Legal Reasoning and Legal Theory*
- Sartor 2005: *Legal Reasoning: A Cognitive Approach to the Law*
- US Federal Rules of Civil Procedure (case phases, deadlines, motions)
- Restatement (Second) of Contracts (obligation, remedy, burden framing)

## Entities

| Category | Entities |
|---|---|
| Lifecycle phases (9) | PreFiling, Filed, Discovery, Motions, PreTrial, Trial, PostTrial, Appeal, Closed |
| Legal term components | LegalTerm, Valence, Obligation, ObligationLanguage, Deadline, DeadlineDuration, BurdenOfProof, ProofStandard, Remedy, Exception, EvidenceRequirement, RequirementLevel, EvidenceType |
| Relations (13 kinds) | Requires, Precedes, Implies, Contradicts, Composes, SubtypeOf, Triggers, Negates, AlternativeTo, Rebuts, AffirmativeDefenseTo, SafeHarborFor, ExhaustionRequiredFor |
| Container | LegalCategory, OntologyRegistry |

## Category

`CaseLifecycleCategory` has `PhaseTag` as objects and `PhaseTransitionRel` as morphisms. Morphisms include identities, every valid transition declared by `PhaseTag::valid_transitions`, and the composition closure so multi-step transitions resolve.

```mermaid
graph LR
    PreFiling --> Filed
    Filed --> Discovery
    Discovery --> Motions
    Motions --> PreTrial
    PreTrial --> Trial
    Trial --> PostTrial
    PostTrial --> Appeal
    PostTrial --> Closed
    Appeal --> Closed
```

## Qualities

| Quality | Type | Description |
|---|---|---|
| IsTerminalPhase | () | Marks phases where no further transitions are permitted (only `Closed`) |

## Axioms (2)

| Axiom | Description | Source |
|---|---|---|
| OnlyClosedIsTerminal | Only `Closed` is a terminal phase | structural |
| NoDeadPhases | Every non-terminal phase has at least one valid transition | structural |

Plus the auto-generated structural axioms from category laws.

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../docs/use/compose-via-functor.md) to add one.

## Files

- `ontology.rs` -- `LegalTerm`, `LegalCategory`, `RelationType`, `OntologyRegistry`, `CaseLifecycleCategory`/`CaseLifecycleOntology`, `IsTerminalPhase` quality, `OnlyClosedIsTerminal`/`NoDeadPhases` axioms, tests
- `lifecycle.rs` -- `Case`, `CaseAction`, `CasePhase`, `PhaseTag`, phase transition rules
- `authority.rs` -- `Authority` (statute, regulation, case law, etc.)
- `source.rs` -- source document and citation types
- `rule.rs` -- rule representation
- `element.rs` -- rule element representation
- `fact.rs` -- typed fact representation and validation
- `finding.rs` -- factual findings
- `decision.rs` -- adjudicated decisions
- `argument.rs` -- legal argument construction
- `entity.rs` -- party/actor entities in a case
- `engine.rs` -- `LegalEngine`, `LegalAction`, `new_case` runtime
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
