# Praxis Architecture: Zero-Knowledge Code via Composable Domain Ontologies with Categorical Proof

## Abstract

We present praxis architecture — a software architecture where application code contains zero domain knowledge. All intelligence lives in formally axiomatized ontologies grounded in a foundational upper ontology (DOLCE). Cross-domain composition is achieved via category-theoretic functors that mathematically preserve behavioral properties. The architecture is justified by the Good Regulator Theorem (Conant & Ashby, 1970): every effective controller must contain a model of its system. In praxis, the ontology IS the model. We demonstrate the architecture across 25+ domains (physics, chess, natural language, traffic control, judicial proceedings) with 1,809 machine-verified proofs.

## 1. Introduction

### The problem

Traditional software architectures embed domain knowledge in code. Domain-Driven Design puts the domain model in classes. Model-Driven Architecture generates code from UML models. Expert systems externalize rules but as flat if-then productions without compositional guarantees.

### The claim

We propose that domain knowledge should live entirely in composable ontologies — formal descriptions of what exists and how things relate — and that the code should be a generic engine with no domain knowledge whatsoever. We call this **praxis architecture**.

### The proof

The claim is not aspirational. We demonstrate it with a working system containing 25+ domain ontologies, 10+ proven functors between domains, and 1,809 machine-verified proofs. The same generic engine enforces chess rules, physics laws, grammatical constraints, and legal procedures.

## 2. Theoretical Foundations

### 2.1 Category Theory as the Composition Mechanism

- Objects and morphisms (Mac Lane 1971)
- Functors: structure-preserving maps between categories
- Natural transformations: transformations between functors
- Compact closed categories: pregroups for grammar (Lambek 1999)

### 2.2 The Good Regulator Theorem

Conant & Ashby (1970): every effective regulator must be (or contain) a model of the system it regulates. In praxis: the ontology IS the model. This is not a design choice — it is a mathematical requirement for effective control.

### 2.3 Requisite Variety

Ashby (1956): a controller must have at least as many states as the disturbances it regulates. The ontology must be as rich as the domain. An ontology that is too simple cannot regulate its domain effectively.

### 2.4 DOLCE Upper Ontology

Masolo et al. (2003): classification of being into Endurant (Physical, Social, Mental, Abstract), Perdurant (Event, Process), and Quality. Every domain ontology in praxis is classified by Being type via a verified functor.

## 3. Architecture

### 3.1 Five Layers

```
Logic     → Axioms, propositions, inference
Category  → Entity, Relationship, Category, Functor
Ontology  → Domain knowledge, reasoning patterns, DOLCE
Engine    → Situation, Action, Precondition, enforcement
Codegen   → Build-time ontology generation from data sources
```

### 3.2 The Engine as Control System

The Engine implements a closed-loop control pattern. We prove this with a functor from Control Systems to the Engine pattern:

| Control | Engine |
|---|---|
| Plant | Situation |
| Controller | Precondition evaluation |
| Sensor | Situation observation |
| Actuator | Action execution |
| Model | Ontology (Conant-Ashby) |
| Feedback Loop | Engine.next() cycle |

### 3.3 Zero Domain Knowledge in Code

The Engine trait, Category trait, and Functor trait contain no domain-specific logic. Adding a new domain (e.g., chess, traffic, English grammar) requires only:
1. Defining an ontology (Entity enum + Relationship struct + Category impl)
2. Defining actions and situations (Action + Situation traits)
3. Defining preconditions (Precondition trait)

No code paths change. The framework is invariant across domains.

## 4. Composition via Functors

### 4.1 Cross-Domain Proofs

Every functor is verified by `check_functor_laws()`:
- Identity preservation: F(id_A) = id_{F(A)}
- Composition preservation: F(g∘f) = F(g)∘F(f)

Proven functors include:
- Traffic IS Systems (TrafficToSystems)
- Chess IS EventDriven (ChessToEvents)
- Chess IS Concurrent (ChessToConcurrency)
- EventDriven IS Concurrent (EventsToConcurrency)
- Systems IS Concurrent (SystemsToConcurrency)
- Dialogue IS Communication (DialogueToCommunication)
- Control IS Engine (ControlToEngine)
- Praxis types IS DOLCE (PraxisToDolce)

### 4.2 Composable Proof Chains

If A IS B (functor) and B IS C (functor), then A IS C (composition). The composed proof is automatic and correct by the functor composition theorem. This gives us:

```
Chess IS EventDriven IS Concurrent
Systems IS Concurrent IS EventDriven
Traffic IS Systems IS Concurrent
```

### 4.3 Ontology Evolution via Functor

When transforming ontologies, praxis creates the new ontology alongside the old and proves the mapping via functor. The functor guarantees structure preservation. This is implicit in Spivak's functorial data migration (2010) but applied here to ontological evolution.

## 5. Natural Language as Ontological Composition

### 5.1 The Linguistics Pipeline

```
Text → Language::lexical_lookup → Pregroup algebra → Montague functor → Speech acts → Discourse
```

Every arrow is a functor. The tokenizer is language-agnostic (parameterized by `&dyn Language`). The pregroup grammar (Lambek 1999) is an algebraic ontology — parsing is group-like contraction. The Montague functor maps syntax to semantics. Speech acts (Searle 1976) classify what utterances DO. Discourse reference (Kamp 1981, Grosz/Joshi/Weinstein 1995) tracks entities across utterances.

### 5.2 Spelling Correction as Adjunction

The noisy channel model (Shannon 1948, applied by Kernighan/Church/Gale 1990) is an adjunction:
- F: Lang → Obs (the channel functor — words become misspellings)
- G: Obs → Lang (Bayesian right adjoint — correction)
- G∘F ≠ Id (information loss through channel)

### 5.3 No Hardcoded Word Knowledge

The system contains zero hardcoded English words. Function words are constructed during language initialization from OLiA categories. Content words come from WordNet. Verb transitivity comes from WordNet subcategorization frames. Pronoun classification (anaphoric vs interrogative) comes from OLiA's PronounKind taxonomy. A Hebrew or Turkish implementation would use the same code with different ontology data.

## 6. Related Work

| Approach | What it does | How praxis differs |
|---|---|---|
| Ontology-Driven Architecture (W3C 2006) | Uses ontologies to describe software | Praxis uses ontologies AS the software |
| Spivak's Ologs (2012) | Category-theoretic knowledge representation | Praxis adds behavioral enforcement (Engine) |
| Fiadeiro's Categories for SE (2005) | Categorical component composition | Praxis composes domain ontologies, not components |
| Expert Systems (1980s) | Externalized if-then rules | Praxis uses composable categories, not flat rules |
| DDD (Evans 2003) | Domain model in code | Praxis: domain model IS the ontology, code is generic |
| Palantir Ontology SDK (2020s) | Ontology as data layer | Praxis: ontology as behavioral specification |

## 7. Evaluation

- 25+ domain ontologies
- 10+ proven cross-domain functors
- 1,809 machine-verified proofs
- Physics, chess, music, linguistics, traffic, law, logic puzzles
- Property-based testing with proptest
- Full WordNet English (107K concepts) loaded in <200ms
- All proofs execute in <5 seconds on a single core

## 8. Conclusion

Praxis architecture demonstrates that a software system can achieve zero domain knowledge in code while maintaining behavioral correctness across dozens of domains. The key enablers are:
1. Category theory as the composition mechanism (functors preserve structure)
2. The Good Regulator Theorem as architectural justification (the ontology must model the system)
3. DOLCE as the classification foundation (every domain has a type of being)
4. The Engine as a generic control loop (one pattern, all domains)

The name "praxis" is not marketing — it is a claim backed by 1,809 proofs.

## References

[See docs/foundations.md for full bibliography with 50+ academic sources]
