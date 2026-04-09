# Intellectual Foundations

Praxis draws from and synthesizes several academic traditions. This document traces the lineage and identifies where praxis extends existing work.

## Category Theory

The mathematical foundation. Category theory studies composition — how things combine while preserving structure.

**Key concepts used in praxis:**
- Objects and morphisms → Entity and Relationship
- Composition → transitive closure in reasoning ontology
- Functors → structure-preserving maps (analogy, translation, ontology evolution)
- Natural transformations → transformations between functors
- Axioms → category laws (identity, associativity, closure) verified exhaustively

**Key references:**
- Saunders Mac Lane, *Categories for the Working Mathematician* (1971) — the foundational text
- Steve Awodey, *Category Theory* (2010) — modern introduction
- Emily Riehl, *Category Theory in Context* (2016) — accessible with rich examples
- Bartosz Milewski, *Category Theory for Programmers* (2019) — bridge to software engineering

## Systems Thinking

The conceptual foundation. Systems thinking studies wholes, relationships, and patterns of organization.

**Key insight:** Category theory and systems thinking are isomorphic — the same structure viewed from different angles. Category theory provides the formal proof layer; systems thinking provides the intuitive conceptual layer. A functor maps between them.

| Category theory | Systems thinking |
|---|---|
| Object | Component |
| Morphism | Relationship |
| Composition | Integration |
| Functor | Analogy/Mapping |
| Natural transformation | Evolution |
| Identity | Homeostasis |
| Axiom | Invariant |
| Category | System |

**Key references:**
- Ludwig von Bertalanffy, *General System Theory* (1968) — the founding text
- Donella Meadows, *Thinking in Systems* (2008) — accessible introduction
- Peter Senge, *The Fifth Discipline* (1990) — systems thinking in practice
- Fritjof Capra & Pier Luigi Luisi, *The Systems View of Life* (2014) — synthesis across disciplines

## Cybernetics

The science of control and communication in systems. Praxis's Engine is a cybernetic control loop; metacognition is second-order cybernetics.

**Connections to praxis:**
- Feedback loops → Engine (situation → precondition check → action → new situation)
- Requisite variety (Ashby) → ontology must be as complex as the domain it models
- Autopoiesis → self-creating systems (praxis generating its own ontologies via codegen)
- Second-order cybernetics → the observer is part of the system (praxis reasoning about itself via PraxisToDolce functor)

**Key references:**
- Norbert Wiener, *Cybernetics* (1948) — the founding text
- W. Ross Ashby, *An Introduction to Cybernetics* (1956) — requisite variety, homeostasis
- Stafford Beer, *Brain of the Firm* (1972) — Viable System Model
- Humberto Maturana & Francisco Varela, *Autopoiesis and Cognition* (1980) — self-creating systems
- Heinz von Foerster, *Observing Systems* (1981) — second-order cybernetics
- Gregory Bateson, *Steps to an Ecology of Mind* (1972) — patterns that connect

## DOLCE (Upper Ontology)

The philosophical classification of being. DOLCE provides the taxonomy of existence that praxis uses to classify domains.

**Why DOLCE over BFO or SUMO:**
- DOLCE was designed specifically for *linguistic and cognitive engineering* — exactly praxis's domain
- Its Endurant/Perdurant/Quality distinction maps naturally to praxis's Situation/Action/Quality
- Its Social Object category captures rules, standards, and institutions — most of what praxis models

**Key references:**
- Claudio Masolo et al., *WonderWeb Deliverable D18: Ontology Library* (2003) — the original DOLCE specification
- Stefano Borgo et al., *DOLCE: A Descriptive Ontology for Linguistic and Cognitive Engineering* (2022) — updated formalization ([arXiv:2308.01597](https://arxiv.org/abs/2308.01597))
- Barry Smith, *Basic Formal Ontology* (BFO) — the main alternative (used by US DOD/IC since 2024)
- Ian Niles & Adam Pease, *Towards a Standard Upper Ontology* (2001) — SUMO

## Category Theory Applied to Systems

The synthesis that praxis builds on — researchers who have explicitly connected category theory to systems.

**Robert Rosen** — *Life Itself: A Comprehensive Inquiry into the Nature, Origin, and Fabrication of Life* (1991). Used category theory to model living systems as "relational" rather than mechanical. Key insight: a living system is characterized by its organization (morphisms), not its material (objects). Directly relevant to praxis's ontological approach — we model rules and relationships, not physical stuff.

**David Spivak** — *Category Theory for the Sciences* (2014). Created "ologs" (ontology logs) — category theory applied to knowledge representation. His databases-as-categories framework is conceptually close to what praxis does with ontologies. Also: *Seven Sketches in Compositionality* (2018, with Brendan Fong) — accessible introduction to applied category theory.

**Andrée Ehresmann & Jean-Paul Vanbremeersch** — *Memory Evolutive Systems* (MES). Category-theoretic framework for complex adaptive systems that form, maintain, and evolve internal representations. Directly relevant to praxis's metacognition roadmap — the system building internal models and evolving them.

**John Baez & Mike Stay** — *Physics, Topology, Logic and Computation: A Rosetta Stone* (2009). Demonstrates that category theory reveals deep structural parallels between physics, topology, logic, and computation. The "Rosetta Stone" metaphor aligns with praxis's use of functors to map between domains.

**Brendan Fong & David Spivak** — *An Invitation to Applied Category Theory* (2019). Covers monoidal categories, operads, and hypergraph categories applied to databases, circuits, and signal flow — all systems.

**Eugenia Cheng** — *The Joy of Abstraction* (2022). Accessible bridge between category theory and everyday thinking. Relevant to making praxis's formal foundations understandable.

## Formal Ontology in Information Science

The discipline of building rigorous ontologies for computational systems.

**Key references:**
- Nicola Guarino, *Formal Ontology in Information Systems* (1998) — foundational text
- Thomas Gruber, *A Translation Approach to Portable Ontology Specifications* (1993) — "an ontology is a specification of a conceptualization"
- Nicola Guarino & Christopher Welty, *Evaluating Ontological Decisions with OntoClean* (2002) — rigorous methodology for ontological analysis

## WordNet

The lexical database that praxis uses for the English language ontology.

**Key references:**
- George Miller, *WordNet: A Lexical Database for English* (1995) — the original
- Christiane Fellbaum (ed.), *WordNet: An Electronic Lexical Database* (1998) — comprehensive reference
- John McCrae et al., *English WordNet: A New Open-Source Wordnet for English* (2020) — the open version praxis uses

## Categorial Grammar and Compositional Semantics

The formal foundation for praxis's language understanding pipeline. Semantics IS a functor from syntax to meaning — this is not metaphor, it's the literal mathematical framework.

### Lambek Grammar (syntax as category)

Words have types (noun, verb, etc.). Types combine via function application: a transitive verb is a function that takes a noun phrase on the right and returns a verb phrase. The grammar IS a category: types are objects, type reductions are morphisms, and composition is guaranteed by the calculus.

**Key references:**
- Joachim Lambek, *The Mathematics of Sentence Structure* (1958) — the founding paper; syntax as algebraic calculus
- Joachim Lambek, *Type Grammar Revisited* (1999) — pregroup grammars, simplified Lambek calculus

### Montague Semantics (meaning via functor)

Every syntactic rule has a corresponding semantic rule. Interpretation IS a functor from the syntax category to a logic category. Compositionality: the meaning of the whole is a function of the meanings of the parts.

**Key references:**
- Richard Montague, *The Proper Treatment of Quantification in Ordinary English* (1973) — the founding paper
- Barbara Partee, *Montague Grammar* (1976) — accessible introduction
- Janssen, *Compositionality* (1997) — formal treatment of the compositionality principle

### DisCoCat (the modern synthesis)

Distributional Compositional Categorical model. The syntax category is Lambek pregroups, the semantics category is vector spaces (or logic), and the interpretation IS a functor preserving composition. This is exactly what praxis does — functors between categories — applied to language.

**Key references:**
- Bob Coecke, Mehrnoosh Sadrzadeh, Stephen Clark, *Mathematical Foundations for a Compositional Distributional Model of Meaning* (2010) — the DisCoCat paper
- Giovanni de Felice, *Categorical Tools for Natural Language Processing* (2022, Oxford thesis) — comprehensive modern treatment
- Coecke & Kissinger, *Picturing Quantum Processes* (2017) — string diagrams (the visual language of DisCoCat)
- Alexis Toumi et al., *DisCoPy* — open-source Python toolkit for computing with string diagrams and functors

### Type-Logical Grammar

Types-as-formulas, proofs-as-programs applied to natural language. A derivation of a sentence IS a proof that its types compose correctly. The Curry-Howard correspondence gives us: parsing = proof search, semantics = proof normalization.

**Key references:**
- Glyn Morrill, *Type Logical Grammar* (1994) — the standard reference
- Michael Moortgat, *Categorial Type Logics* (1997) — comprehensive survey
- Stanford Encyclopedia entry on [Typelogical Grammar](https://plato.stanford.edu/entries/typelogical-grammar/)

### The Pipeline (all functors)

```
Text → Tokens → SyntaxCategory → SemanticCategory → PragmaticCategory
         ↑            ↑                 ↑                  ↑
      Lexicon     Lambek grammar   Montague functor    Speech acts
```

Every arrow is a functor. Every step preserves structure. This is the theoretical foundation for praxis's chatbot — no mechanical parsing, only ontological understanding through functors.

## Metacognition and Self-Awareness

The theoretical foundation for praxis knowing what it knows and — critically — what it DOESN'T know. A system that can reason about its own ontological gaps.

### Second-Order Cybernetics (von Foerster)

First-order cybernetics is "the cybernetics of observed systems." Second-order cybernetics is "the cybernetics of observing systems" — the observer enters the domain of observation. When praxis reasons about why its grammar failed to parse a sentence, it IS second-order cybernetics: the system observing its own observing.

**Key references:**
- Heinz von Foerster, *Observing Systems* (1981) — the founding text of second-order cybernetics
- Ranulph Glanville, [*Second Order Cybernetics*](https://www.pangaro.com/glanville/Glanville-SECOND_ORDER_CYBERNETICS.pdf) (PDF) — comprehensive introduction
- Humberto Maturana & Francisco Varela, *Autopoiesis and Cognition* (1980) — self-creating systems
- Ernst von Glasersfeld, *Radical Constructivism* (1995) — knowledge as constructed, not discovered

### Meta-Ontology for Introspection (MOI)

An ontological model for tracing metacognitive experiences — what the system knew, when it knew it, and what happened when knowledge was insufficient. Directly applicable to praxis's self-diagnosis of grammar and ontology gaps.

**Key references:**
- Olivares-Alarcos et al., [*Towards an Ontology for Robot Introspection and Metacognition*](https://www.researchgate.net/publication/376738738_Towards_an_Ontology_for_Robot_Introspection_and_Metacognition) (2023) — formal introspection model

### Metacognitive Bi-Level Architecture

Metacognition is a two-level system: an object level (actual reasoning) and a meta level (monitors, evaluates, controls the object level). When praxis's grammar fails to parse, the meta level detects the failure, diagnoses the gap, and decides whether to ask for clarification or attempt repair.

**Key references:**
- Wang & Zhao, [*Metacognition in LLMs*](https://aclanthology.org/2025.emnlp-main.171.pdf) (EMNLP 2025) — metacognitive capabilities and limitations
- Qi et al., [*Meta-R1: Empowering Large Reasoning Models with Metacognition*](https://arxiv.org/pdf/2508.17291) — metacognitive monitoring and control

### Open World Assumption

When a query fails, does it mean "false" or "I don't know"? The Open World Assumption (OWA) treats absence of knowledge as uncertainty, not falsity. Praxis must distinguish:
- "Is a dog a mammal?" → YES (positive knowledge)
- "Is a dog a vehicle?" → NO (explicit negative knowledge via taxonomy)
- "Is a quark a boson?" → I DON'T KNOW (absent from ontology — open world)

**Key references:**
- [Open World Assumption — Wikipedia](https://en.wikipedia.org/wiki/Closed-world_assumption)
- [Open World vs Closed World — Dataversity](https://www.dataversity.net/articles/introduction-to-open-world-assumption-vs-closed-world-assumption/)

## Where Praxis Extends Existing Work

1. **Category theory + DOLCE synthesis.** Using category theory as the formal proof mechanism for upper ontological classification. Existing work uses either category theory OR formal ontology; praxis combines them with a verified functor.

2. **Self-application.** Using the system's own tools (functors) to evolve its own ontology. The PraxisToDolce functor is praxis reasoning about itself — second-order cybernetics formalized in code.

3. **Ontology evolution via functor.** When transforming ontologies, create the new one alongside and prove the mapping. This pattern is implicit in categorical database migration (Spivak) but praxis applies it to ontological evolution explicitly.

4. **Reasoning ontology as reusable patterns.** Taxonomy, mereology, causation, equivalence, opposition, context — formalized as generic category patterns that any domain instantiates. Individual patterns exist in the literature; the unified set with axioms and property-based testing is new.

5. **Build-time ontology generation.** Using Rust's build system to parse authoritative data sources (WordNet, W3C specs) through ontological understanding (not mechanical parsing) and generate static, tested code. The "no mechanical processing" principle — every data interaction goes through an ontology.

6. **Cross-domain functor proofs.** Proving that domains ARE instances of abstract ontologies: traffic IS a system (TrafficToSystems), chess IS concurrent (ChessToConcurrency), chess IS event-driven (ChessToEvents), systems ARE concurrent (SystemsToConcurrency), event-driven IS concurrent (EventsToConcurrency). The equivalence triangle System ↔ EventDriven ↔ Concurrent is proven by functor composition.

7. **Lambek + Montague + DisCoCat in Rust with property-based testing.** Implementing categorial grammar, compositional semantics, and the syntax→semantics functor in Rust with exhaustive category law verification. Existing implementations (DisCoPy) are in Python without formal verification. Praxis proves the functor laws hold via property-based testing.
