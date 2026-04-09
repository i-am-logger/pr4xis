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

## Where Praxis Extends Existing Work

1. **Category theory + DOLCE synthesis.** Using category theory as the formal proof mechanism for upper ontological classification. Existing work uses either category theory OR formal ontology; praxis combines them with a verified functor.

2. **Self-application.** Using the system's own tools (functors) to evolve its own ontology. The PraxisToDolce functor is praxis reasoning about itself — second-order cybernetics formalized in code.

3. **Ontology evolution via functor.** When transforming ontologies, create the new one alongside and prove the mapping. This pattern is implicit in categorical database migration (Spivak) but praxis applies it to ontological evolution explicitly.

4. **Reasoning ontology as reusable patterns.** Taxonomy, mereology, causation, equivalence, opposition, context — formalized as generic category patterns that any domain instantiates. Individual patterns exist in the literature; the unified set with axioms and property-based testing is new.

5. **Build-time ontology generation.** Using Rust's build system to parse authoritative data sources (WordNet, W3C specs) through ontological understanding (not mechanical parsing) and generate static, tested code. The "no mechanical processing" principle — every data interaction goes through an ontology.
