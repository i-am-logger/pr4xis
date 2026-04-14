# Intellectual Foundations

pr4xis draws from and synthesizes several academic traditions. This document traces the lineage and identifies where pr4xis extends existing work. For the runtime mechanics see [Architecture](architecture.md). For the conceptual model — what an ontology *is* in this system — see [Concepts](concepts.md).

## Distinction-Calculus Lineage (Spencer-Brown → Heim → pr4xis)

Before going section by section through the modern category-theoretic and cybernetic foundations, it is worth naming the older tradition pr4xis sits in: **distinction-calculus**, the line of thought that treats the act of drawing a boundary as the primitive operation from which everything else is derived.

- **G. Spencer-Brown, *Laws of Form* (1969)** — starts with one instruction: "Draw a distinction." From that single act, all of logic, Boolean algebra, and self-reference emerge. Already cited later in this document under "Distinction (Spencer-Brown)".

- **Burkhard Heim, *Syntrometrische Maximentelezentrik* (mid-20th century, published posthumously)** — a logical framework built from predicates, permutation operators, mereological composition, and goal-oriented "telecenters". Recently formalized using category theory, modal logic, Kripke semantics, and mereology in *A Modernized Syntrometric Logic: Foundations and Applications* (heim-theory.com, 2025).

- **pr4xis (2025–2026)** — composable ontologies in Rust, with category-theoretic functor proofs between domains. The substrate is novel as executable code; the structural ideas have prior art in Heim and Spencer-Brown.

The honest claim: pr4xis is the first **executable, machine-checkable** instance of this tradition across many domains. It does not adopt Heim's physical-metaphysical claims (twelve-dimensional spacetime, particle mass formulas, teleological cosmology). The structural overlap with the modernized syntrometric logic is concrete and verifiable: both treat domains as categories with structure-preserving functors between them, use Kripke-style aspect-relative semantics, ground part/whole reasoning in classical extensional mereology, and model self-reference as a natural transformation. See [issue #51](https://github.com/i-am-logger/pr4xis/issues/51) for the first-pass verification of this lineage.

## Modern Foundations (Section by Section)

## Category Theory

The mathematical foundation. Category theory studies composition — how things combine while preserving structure.

**Key concepts used in pr4xis:**
- Objects and morphisms → Entity and Relationship
- Composition → transitive closure in reasoning ontology
- Functors → structure-preserving maps (analogy, translation, ontology evolution)
- Natural transformations → transformations between functors
- Axioms → category laws (identity, associativity, closure) verified exhaustively
- Category of elements (Grothendieck construction) → automatic trace schema derivation
- Discrete fibrations → every presheaf IS a fibration (Riehl 2019)

**Category of Elements (El)** — the construction that makes traceability automatic. Given a functor F: C → Set, El(F) has objects (c, x) where x ∈ F(c) and morphisms tracking how elements relate. Applied to ontology schemas: El unpacks the schema into individual observable elements, each decorated with PROV-O provenance. T(C) = El(C) +_O O_obs is the trace schema functor.

**Key references:**
- Saunders Mac Lane, *Categories for the Working Mathematician* (1971) — the foundational text- Steve Awodey, *Category Theory* (2010) — modern introduction
- Emily Riehl, *Category Theory in Context* (2016) — accessible with rich examples- Emily Riehl, *Categorical Notions of Fibration* (2019) — discrete fibrations = Set-valued functors- Bartosz Milewski, *Category Theory for Programmers* (2019) — bridge to software engineering
- Brendan Fong & David Spivak, *Seven Sketches in Compositionality* (2019) — applied category theory- Alexander Grothendieck, *SGA1: Revetements etales et groupe fondamental* (1961) — original fibered category definition
- Spivak, *Functorial Data Migration* (2012) — El(I) for database instances
- Spivak, *Category Theory for the Sciences* (2014) — pedagogical El treatment

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

## Control Systems and Cybernetics

Control theory is the general science of feedback and regulation. Cybernetics is a specific type: control systems that involve communication (Wiener 1948). The distinction matters — not all control is cybernetic, and not all communication is control. pr4xis's Engine is a cybernetic control loop; metacognition is second-order cybernetics.

**Connections to pr4xis:**
- Feedback loops → Engine (situation → precondition check → action → new situation)
- Requisite variety (Ashby) → ontology must be as complex as the domain it models
- Autopoiesis → self-creating systems (pr4xis generating its own ontologies via codegen)
- Second-order cybernetics → the observer is part of the system (pr4xis reasoning about itself via OwnToDolce functor)

**Three key theorems:**
- **Requisite Variety** (Ashby 1956): a controller must have at least as many states as the disturbances it regulates. V(controller) >= V(disturbances).
- **Good Regulator Theorem** (Conant & Ashby 1970): every effective controller must be (or contain) a model of its system. THIS IS WHY THE ENGINE NEEDS AN ONTOLOGY.
- **Perceptual Control** (Powers 1973): systems control their inputs (perceptions), not their outputs (behavior). Behavior is the means, not the end.

**Key references:**
- Norbert Wiener, *Cybernetics* (1948) — the founding text; control + communication
- W. Ross Ashby, *An Introduction to Cybernetics* (1956) — requisite variety, homeostasis- Roger Conant & W. Ross Ashby, *Every Good Regulator of a System Must Be a Model of That System* (1970) — the regulator theorem
- William Powers, *Behavior: The Control of Perception* (1973) — perceptual control theory
- Stafford Beer, *Brain of the Firm* (1972) — Viable System Model (5 recursive control levels)
- Karl Åström & Richard Murray, *Feedback Systems* (2008) — modern treatment (free online)
- Humberto Maturana & Francisco Varela, *Autopoiesis and Cognition* (1980) — self-creating systems
- Heinz von Foerster, *Observing Systems* (1981) — second-order cybernetics
- Gregory Bateson, *Steps to an Ecology of Mind* (1972) — patterns that connect

## DOLCE (Upper Ontology)

The philosophical classification of being. DOLCE provides the taxonomy of existence that pr4xis uses to classify domains.

**Why DOLCE over BFO or SUMO:**
- DOLCE was designed specifically for *linguistic and cognitive engineering* — exactly pr4xis's domain
- Its Endurant/Perdurant/Quality distinction maps naturally to pr4xis's Situation/Action/Quality
- Its Social Object category captures rules, standards, and institutions — most of what pr4xis models

**Key references:**
- Claudio Masolo et al., *WonderWeb Deliverable D18: Ontology Library* (2003) — the original DOLCE specification
- Stefano Borgo et al., *DOLCE: A Descriptive Ontology for Linguistic and Cognitive Engineering* (2022) — updated formalization ([arXiv:2308.01597](https://arxiv.org/abs/2308.01597))
- Barry Smith, *Basic Formal Ontology* (BFO) — the main alternative (used by US DOD/IC since 2024)
- Ian Niles & Adam Pease, *Towards a Standard Upper Ontology* (2001) — SUMO

## Category Theory Applied to Systems

The synthesis that pr4xis builds on — researchers who have explicitly connected category theory to systems.

**Robert Rosen** — *Life Itself: A Comprehensive Inquiry into the Nature, Origin, and Fabrication of Life* (1991). Used category theory to model living systems as "relational" rather than mechanical. Key insight: a living system is characterized by its organization (morphisms), not its material (objects). Directly relevant to pr4xis's ontological approach — we model rules and relationships, not physical stuff.

**David Spivak** — *Category Theory for the Sciences* (2014). Created "ologs" (ontology logs) — category theory applied to knowledge representation. His databases-as-categories framework is conceptually close to what pr4xis does with ontologies. Also: *Seven Sketches in Compositionality* (2018, with Brendan Fong) — accessible introduction to applied category theory.

**Andrée Ehresmann & Jean-Paul Vanbremeersch** — *Memory Evolutive Systems* (MES). Category-theoretic framework for complex adaptive systems that form, maintain, and evolve internal representations. Directly relevant to pr4xis's metacognition roadmap — the system building internal models and evolving them.

**John Baez & Mike Stay** — *Physics, Topology, Logic and Computation: A Rosetta Stone* (2009). Demonstrates that category theory reveals deep structural parallels between physics, topology, logic, and computation. The "Rosetta Stone" metaphor aligns with pr4xis's use of functors to map between domains.

**Brendan Fong & David Spivak** — *An Invitation to Applied Category Theory* (2019). Covers monoidal categories, operads, and hypergraph categories applied to databases, circuits, and signal flow — all systems.

**Eugenia Cheng** — *The Joy of Abstraction* (2022). Accessible bridge between category theory and everyday thinking. Relevant to making pr4xis's formal foundations understandable.

## Formal Ontology in Information Science

The discipline of building rigorous ontologies for computational systems.

**Key references:**
- Nicola Guarino, *Formal Ontology in Information Systems* (1998) — foundational text
- Thomas Gruber, *A Translation Approach to Portable Ontology Specifications* (1993) — "an ontology is a specification of a conceptualization"
- Nicola Guarino & Christopher Welty, *Evaluating Ontological Decisions with OntoClean* (2002) — rigorous methodology for ontological analysis

## WordNet

The lexical database that pr4xis uses for the English language ontology.

**Key references:**
- George Miller, *WordNet: A Lexical Database for English* (1995) — the original
- Christiane Fellbaum (ed.), *WordNet: An Electronic Lexical Database* (1998) — comprehensive reference
- John McCrae et al., *English WordNet: A New Open-Source Wordnet for English* (2020) — the open version pr4xis uses

## Categorial Grammar and Compositional Semantics

The formal foundation for pr4xis's language understanding pipeline. Semantics IS a functor from syntax to meaning — this is not metaphor, it's the literal mathematical framework.

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

Distributional Compositional Categorical model. The syntax category is Lambek pregroups, the semantics category is vector spaces (or logic), and the interpretation IS a functor preserving composition. This is exactly what pr4xis does — functors between categories — applied to language.

**Key references:**
- Bob Coecke, Mehrnoosh Sadrzadeh, Stephen Clark, *Mathematical Foundations for a Compositional Distributional Model of Meaning* (2010) — the DisCoCat paper
- Giovanni de Felice, *Categorical Tools for Natural Language Processing* (2022, Oxford thesis) — comprehensive modern treatment
- Coecke & Kissinger, *Picturing Quantum Processes* (2017) — string diagrams (the visual language of DisCoCat)
- Alexis Toumi et al., *DisCoPy* — open-source Python toolkit for computing with string diagrams and functors

### Pregroup Grammars (the algebraic key)

Lambek (1999) showed that parsing is a group-like computation. A pregroup is a partially ordered monoid where every element has left and right adjoints. Word types are products of basic types with adjoints; parsing is contraction of the product to the sentence type.

**Why pregroups matter for pr4xis:**
- Parsing IS algebra: multiply word types, contract using adjoint laws
- The chart IS a semiring (Goodman 1999): `+` = all derivations, `×` = combining spans
- The Montague functor IS a compact closed functor from the pregroup to vector spaces (DisCoCat)
- Pregroups connect to our existing group theory ontology (Rubik's cube)

**Key references:**
- Lambek, *Type Grammar Revisited* (1999) — pregroups replace slash types with adjoints
- Lambek, *Pregroups and Natural Language Processing* — parsing as group computation- Casadio & Lambek, *A Tale of Four Grammars* (Studia Logica, 2002) — the hierarchy AB → Lambek → Pregroup- Goodman, *Semiring Parsing* (Computational Linguistics, 1999) — chart as semiring- Yeung & Kartsaklis, *A CCG-Based Version of the DisCoCat Framework* (ACL, 2021) — CCG + DisCoCat- Pentus, *Lambek Grammars are Context Free* (1993) — free group interpretation

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

Every arrow is a functor. Every step preserves structure. This is the theoretical foundation for pr4xis's chatbot — no mechanical parsing, only ontological understanding through functors.

## Metacognition and Self-Awareness

The theoretical foundation for pr4xis knowing what it knows and — critically — what it DOESN'T know. A system that can reason about its own ontological gaps.

### Second-Order Cybernetics (von Foerster)

First-order cybernetics is "the cybernetics of observed systems." Second-order cybernetics is "the cybernetics of observing systems" — the observer enters the domain of observation. When pr4xis reasons about why its grammar failed to parse a sentence, it IS second-order cybernetics: the system observing its own observing.

**Key references:**
- Heinz von Foerster, *Observing Systems* (1981) — the founding text of second-order cybernetics
- Ranulph Glanville, [*Second Order Cybernetics*](https://www.pangaro.com/glanville/Glanville-SECOND_ORDER_CYBERNETICS.pdf) (PDF) — comprehensive introduction
- Humberto Maturana & Francisco Varela, *Autopoiesis and Cognition* (1980) — self-creating systems
- Ernst von Glasersfeld, *Radical Constructivism* (1995) — knowledge as constructed, not discovered

### Meta-Ontology for Introspection (MOI)

An ontological model for tracing metacognitive experiences — what the system knew, when it knew it, and what happened when knowledge was insufficient. Directly applicable to pr4xis's self-diagnosis of grammar and ontology gaps.

**Key references:**
- Olivares-Alarcos et al., [*Towards an Ontology for Robot Introspection and Metacognition*](https://www.researchgate.net/publication/376738738_Towards_an_Ontology_for_Robot_Introspection_and_Metacognition) (2023) — formal introspection model

### Metacognitive Bi-Level Architecture

Metacognition is a two-level system: an object level (actual reasoning) and a meta level (monitors, evaluates, controls the object level). When pr4xis's grammar fails to parse, the meta level detects the failure, diagnoses the gap, and decides whether to ask for clarification or attempt repair.

**Key references:**
- Wang & Zhao, [*Metacognition in LLMs*](https://aclanthology.org/2025.emnlp-main.171.pdf) (EMNLP 2025) — metacognitive capabilities and limitations
- Qi et al., [*Meta-R1: Empowering Large Reasoning Models with Metacognition*](https://arxiv.org/pdf/2508.17291) — metacognitive monitoring and control

### Self-Model and Self-Image

Three cybernetic traditions converge on the necessity of self-models:

**Representational** (Craik → Conant-Ashby → Powers → Metzinger): the system holds an explicit internal model of itself. Conant-Ashby (1970) proves this mathematically: any self-regulating system must contain a model of itself (homomorphism). Powers' PCT (1973) places self-image at Level 11 (System Concept) — the highest level of perceptual control.

**Organizational** (Maturana-Varela autopoiesis, 1972/1980): the system does not *have* a model — it *is* its self-producing organization. Operational closure: processes produce the processes that produce them.

**Reflexive** (von Foerster eigenform, 1981; Bateson double description, 1972): the observer must include itself. Self-image = eigenform (fixed point of recursive self-observation). Bateson: valid self-image requires "double description" — self AND context simultaneously.

**Key references:**
- Kenneth Craik, *The Nature of Explanation* (1943) — dual internal model (world + self)
- Conant & Ashby, [*Every Good Regulator of a System Must Be a Model of That System*](https://pespmc1.vub.ac.be/books/Conant_Ashby.pdf) (1970)
- William T. Powers, *Behavior: The Control of Perception* (1973) — PCT, 11-level hierarchy
- Gregory Bateson, *Steps to an Ecology of Mind* (1972) — cybernetics of self, double description
- Maturana & Varela, *Autopoiesis and Cognition* (1980) — operational closure
- Thomas Metzinger, *Being No One* (2003) — Phenomenal Self-Model (PSM)
- Maxwell Maltz, *Psycho-Cybernetics* (1960) — self-image as servo reference signal
- Louis Kauffman, [*EigenForm*](http://homepages.math.uic.edu/~kauffman/Eigen.pdf) (2003) — fixed points of self-reference

### Self-Description and Knowledge Base Introspection

For a system to describe what it knows, it needs formal vocabularies for self-description.

**Key references:**
- W3C, [*VoID — Vocabulary of Interlinked Datasets*](https://www.w3.org/TR/void/) (2011) — describes what a knowledge graph contains
- W3C, [*DCAT — Data Catalog Vocabulary v3*](https://www.w3.org/TR/vocab-dcat-3/) (2024) — catalogs of datasets
- Herre & Loebe, *A Meta-ontological Architecture for Foundational Ontologies* (FOIS 2005) — using ontology's own categories to describe itself
- Brian Cantwell Smith, *Reflection and Semantics in LISP* (POPL 1984) — causal connection requirement
- Kephart & Chess, *The Vision of Autonomic Computing* (IEEE 2003) — MAPE-K, Ksys self-model
- Lewis et al., *A Survey of Self-Awareness in Computing Systems* (IEEE 2011) — five awareness levels
- IEEE 1872.2, *Autonomous Robotics Ontology* (2021) — SelfModel as required class
- Nolte et al., *Towards an Ontology for Robot Introspection and Metacognition* (FOIS 2023) — MOI

### Open World Assumption

When a query fails, does it mean "false" or "I don't know"? The Open World Assumption (OWA) treats absence of knowledge as uncertainty, not falsity. pr4xis must distinguish:
- "Is a dog a mammal?" → YES (positive knowledge)
- "Is a dog a vehicle?" → NO (explicit negative knowledge via taxonomy)
- "Is a quark a boson?" → I DON'T KNOW (absent from ontology — open world)

**Key references:**
- [Open World Assumption — Wikipedia](https://en.wikipedia.org/wiki/Closed-world_assumption)
- [Open World vs Closed World — Dataversity](https://www.dataversity.net/articles/introduction-to-open-world-assumption-vs-closed-world-assumption/)

## Distinction (Spencer-Brown)

The most fundamental concept. Before categories, before logic, before knowledge — there is distinction. The act of drawing a boundary that separates "this" from "not this."

Spencer-Brown's *Laws of Form* (1969) starts with one instruction: "Draw a distinction." From that single act, all of logic, Boolean algebra, and self-reference emerge. Von Foerster recognized this as the foundation of second-order cybernetics — the observer draws the distinction between self and observed.

In pr4xis, distinction is everywhere: Entity (this vs not-this), Boundary (inside vs outside), Opposition (A vs not-A), Bit (0 vs 1), Context (this meaning vs that meaning).

**Key references:**
- G. Spencer-Brown, *Laws of Form* (1969) — the founding text
- Louis Kauffman, [*Laws of Form — An Exploration in Mathematics and Foundations*](http://homepages.math.uic.edu/~kauffman/Laws.pdf)- Robin Robertson, [*Some-thing from No-thing: Spencer-Brown's Laws of Form*](https://www.projectenportfolio.nl/images/1/16/Robertson-Laws_of_Form.pdf)- Francisco Varela, *A Calculus for Self-Reference* (1975) — extending Spencer-Brown to self-referential systems

## Information Theory

The science of quantifying, storing, and communicating information. Foundation for the information ontology (Bit, Byte, Reference, Text).

**Key references:**
- Claude Shannon, [*A Mathematical Theory of Communication*](https://people.math.harvard.edu/~ctm/home/text/others/shannon/entropy/entropy.pdf) (1948) — the founding paper- Alan Turing, *On Computable Numbers* (1936) — computability and information processing

## Mereology

The formal theory of parts and wholes. Foundation for the mereology reasoning ontology (has-a relationships).

**Key references:**
- Peter Simons, *Parts: A Study in Ontology* (1987, Oxford) — the standard reference
- [Stanford Encyclopedia — Mereology](https://plato.stanford.edu/entries/mereology/) — comprehensive formal treatment
- Winston, Chaffin, Herrmann, *A Taxonomy of Part-Whole Relations* (1987) — meronymic relation types

## Concurrency Theory

The formal theory of concurrent and parallel computation. Foundation for the concurrency ontology (Agent, SharedResource, Synchronization, Deadlock).

**Key references:**
- C.A.R. Hoare, [*Communicating Sequential Processes*](https://www.cs.cmu.edu/~crary/819-f09/Hoare78.pdf) (1978) — CSP- Robin Milner, *A Calculus of Communicating Systems* (1980) — CCS
- Carl Hewitt, *A Universal Modular ACTOR Formalism* (1973) — Actor model

## Event-Driven Architecture

Foundational patterns for event-driven systems. Basis for the events ontology (Event, Command, EventLog, Handler, EventBus).

**Key references:**
- Martin Fowler, *Event Sourcing* (2005) — pattern description
- Greg Young, *CQRS Documents* (2010) — Command Query Responsibility Segregation
- [Exploring CQRS and Event Sourcing](https://download.microsoft.com/download/e/a/8/ea8c6e1f-01d8-43ba-992b-35cfcaa4fae3/cqrs_journey_guide.pdf) — Microsoft patterns & practices- Guizzardi et al., *Events as Entities in Ontology-Driven Conceptual Modeling* (2019) — formal event ontology based on UFO-B

## RDF and OWL (Knowledge Representation Standards)

The formal foundation for reading and exchanging ontologies. RDF provides the data model (triples: subject-predicate-object). OWL provides the ontology language built on RDF. Together they define how to express and share formal knowledge — what pr4xis reads when loading OLiA, WordNet-LMF, or any published ontology.

**RDF (Resource Description Framework):**
- Everything is a triple: (subject, predicate, object)
- Subjects are IRIs or blank nodes; predicates are IRIs; objects are IRIs, blank nodes, or literals
- An RDF graph is a set of triples — no ordering, no duplicates
- RDFS adds: rdfs:Class (a set of resources), rdfs:subClassOf (taxonomy), rdf:type (instantiation), rdfs:domain/range (constraints)
- The RDFS class hierarchy: Resource → Class → Datatype; Resource → Literal; Resource → Property

**OWL 2 (Web Ontology Language):**
- Built on RDF, adds formal logic (Description Logic SROIQ)
- Entities: Classes, Object Properties, Data Properties, Annotation Properties, Named Individuals, Datatypes
- Class expressions: intersection, union, complement, oneOf (enumeration), existential/universal restrictions, cardinality constraints
- Property characteristics: functional, inverse functional, transitive, symmetric, asymmetric, reflexive, irreflexive
- Axioms: SubClassOf, EquivalentClasses, DisjointClasses, SubPropertyOf, ClassAssertion, PropertyAssertion
- Three profiles: OWL 2 EL (polynomial, for large ontologies like SNOMED CT), OWL 2 QL (LOGSPACE, for databases), OWL 2 RL (polynomial, for rule engines)

**Key references:**
- W3C, *RDF 1.1 Concepts and Abstract Syntax* (2014) — https://www.w3.org/TR/rdf11-concepts/
- W3C, *RDF Schema 1.1* (2014) — https://www.w3.org/TR/rdf-schema/
- W3C, *OWL 2 Web Ontology Language Structural Specification* (2012) — https://www.w3.org/TR/owl2-syntax/
- W3C, *OWL 2 Web Ontology Language Primer* (2012) — https://www.w3.org/TR/owl2-primer/
- W3C, *OWL 2 Web Ontology Language Direct Semantics* (2012) — https://www.w3.org/TR/owl2-direct-semantics/
- W3C, *OWL 2 Web Ontology Language Profiles* (2012) — https://www.w3.org/TR/owl2-profiles/
- Franz Baader et al., *An Introduction to Description Logics* (2003) — formal logic underlying OWL- Tim Berners-Lee, James Hendler, Ora Lassila, *The Semantic Web* (Scientific American, 2001) — the vision
## OLiA (Ontologies of Linguistic Annotation)

The formal standard for linguistic data categories. OLiA defines 1,300+ linguistic concepts in OWL/DL — every part of speech, morphological feature, and syntactic category across all natural languages. This is what pr4xis loads to KNOW what a Determiner, Copula, or Interrogative is — not from a hand-coded vocabulary, but from the research-grounded ontology.

**Architecture (three tiers):**
1. Reference Model (`olia.owl`) — universal linguistic data categories: MorphosyntacticCategory, MorphologicalFeature, SyntacticCategory
2. Annotation Models — OWL formalizations of specific tagsets (Penn Treebank, Universal Dependencies, EAGLES)
3. Linking Models — rdfs:subClassOf bridges from annotation models to reference model concepts

**Key references:**
- Christian Chiarcos & Maria Sukhareva, *OLiA — Ontologies of Linguistic Annotation* (Semantic Web journal, 2015) — the formal paper- Official URI: http://purl.org/olia/
- GitHub: https://github.com/acoli-repo/olia

## Lexicon Ontology

The formal structure of a language's word inventory. A lexicon is not a list — it is a structured mapping from forms to meanings where both sides have internal structure.

**Three converging models:**
- **LMF** (ISO 24613): Language → Lexicon → LexicalEntry → Form + Sense. The international standard.
- **OntoLex-Lemon** (W3C 2016): Three-way bridge: ontology entity ↔ lexical sense ↔ lexical concept. Bridges lexicons and ontologies in RDF/OWL.
- **Generative Lexicon** (Pustejovsky 1991): Each entry carries four qualia (Aristotle's aitia): FORMAL (is-a), CONSTITUTIVE (has-a), TELIC (purpose), AGENTIVE (origin).

**Open class vs closed class** — nearly universal but not absolute. Open class (nouns, verbs) accepts new members; closed class (determiners, pronouns, copulas) is finite and fixed. In some languages adjectives are closed class. In Japanese pronouns are effectively open.

**Language-agnostic design** — Hebrew uses consonantal roots + vowel patterns (binyanim), not "words." Chinese has no word boundaries. Turkish agglutinates. The `Language` trait provides `lexical_lookup` — each language implements its own segmentation and morphological analysis.

**Key references:**
- Pustejovsky, *The Generative Lexicon* (Computational Linguistics, 1991) — structured lexical entries- Jackendoff, *Foundations of Language* (2002) — the Parallel Architecture; the mental lexicon
- OntoLex-Lemon, *W3C Community Report* (2016) — lexicon-ontology bridge in RDF/OWL
- OntoLex-Lemon as bridge for WordNets (GWC 2019)
- ISO 24613 (LMF), *Lexical Markup Framework* (2019/2024) — international standard
- Farrar & Langendoen, *GOLD: General Ontology for Linguistic Description* (2003)

## Spelling Error Ontology

The science of misspelling. Spelling errors are classified on three orthogonal axes — every error is a point in this three-dimensional space.

**Axis 1 — Etiology (WHY):** Competence errors (the writer doesn't know the spelling) vs performance errors (the writer knows but mistypes). This maps to the dual-route model of spelling production (Coltheart 1981).

**Axis 2 — Linguistic Level (WHAT):** Phonological (sounds wrong), Orthographic (sounds right but spelled wrong), Morphological (morpheme boundary error), Visual (letter shape confusion). The POMAS framework (Silliman, Brimo 2013).

**Axis 3 — Operation (HOW):** Substitution, Deletion, Insertion, Transposition, Run-on, Split. Damerau (1964): >80% of misspellings involve a single operation.

**The noisy channel model** (Shannon 1948, applied by Kernighan, Church & Gale 1990): spelling correction IS a functor — the inverse of the error channel.

**Orthographic Depth Hypothesis** (Katz & Frost 1992): shallow orthographies (Finnish, Spanish) produce mostly performance errors; deep orthographies (English, French) produce mostly competence errors.

**Key references:**
- Damerau, *A technique for computer detection of spelling errors* (1964) — the four basic edit operations
- Kukich, *Techniques for automatically correcting words in text* (ACM Computing Surveys, 1992) — seminal survey
- Brill & Moore, *An improved error model for noisy channel spelling correction* (ACL, 2000) — string-to-string model
- Pollock & Zamora, *Collection and characterization of spelling errors in scientific text* (1983) — 50K+ errors analyzed
- Coltheart, *Dual-route model of reading/spelling* (1981) — cognitive architecture
- Caramazza & Miceli, *The structure of graphemic representations* (1990) — graphemic buffer
- Wing & Baddeley, *Spelling errors in handwriting* (1980) — serial position effects
- Katz & Frost, *Orthographic Depth Hypothesis* (1992) — writing system determines error patterns
- Mitton, *English Spelling and the Computer* (1996) — Birkbeck corpus (36K misspellings)

## Ontological Architecture

pr4xis's architecture is a novel synthesis of five existing ideas that have never been combined:

| Idea | Source | What it contributes |
|---|---|---|
| Ontology as runtime component | Guarino (1998) | The philosophical stance |
| Functorial knowledge composition | Spivak (2012) | The mathematical mechanism |
| Categorical software architecture | Fiadeiro (2005) | Categories for component composition |
| Good Regulator Theorem | Conant & Ashby (1970) | The ontology MUST be a model of the system |
| Ontology Design Patterns | Gangemi (2005) | Reusable ontological building blocks |

**What is novel**: an architecture where domain knowledge lives in composable ontologies rather than in mechanical processing logic. There is no parser-with-special-cases, no rule-engine-with-hardcoded-strings, no if-statements branching on domain values. Cross-domain composition is done via verified functors that preserve behavioral properties, and the architecture is justified by the Good Regulator Theorem.

**Key references:**
- Guarino, *Formal Ontology and Information Systems* (FOIS 1998) — defined ontology-driven systems
- Spivak, *Ologs: A Categorical Framework for Knowledge Representation* (PLoS ONE, 2012)
- Spivak, *Functorial Data Migration* (2010) — functors between database schemas
- Fiadeiro, *Categories for Software Engineering* (Springer, 2005)
- Goguen & Burstall, *Institutions: Abstract Model Theory* (JACM, 1992)
- Gangemi, *Ontology Design Patterns for Semantic Web Content* (2005)
- Guizzardi et al., *UFO: Unified Foundational Ontology* (2022) — OntoUML
- W3C, *Ontology Driven Architectures* (Working Group Note, 2006)
- Pan, Staab et al., *Ontology-Driven Software Development* (Springer, 2012)

## Where pr4xis Extends Existing Work

1. **Category theory + DOLCE synthesis.** Using category theory as the formal proof mechanism for upper ontological classification. Existing work uses either category theory OR formal ontology; pr4xis combines them with a verified functor.

2. **Self-application.** Using the system's own tools (functors) to evolve its own ontology. The OwnToDolce functor is pr4xis reasoning about itself — second-order cybernetics formalized in code.

3. **Ontology evolution via functor.** When transforming ontologies, create the new one alongside and prove the mapping. This pattern is implicit in categorical database migration (Spivak) but pr4xis applies it to ontological evolution explicitly.

4. **Reasoning ontology as reusable patterns.** Taxonomy, mereology, causation, equivalence, opposition, context — formalized as generic category patterns that any domain instantiates. Individual patterns exist in the literature; the unified set with axioms and property-based testing is new.

5. **Build-time ontology generation.** Using Rust's build system to parse authoritative data sources (WordNet, W3C specs) through ontological understanding (not mechanical parsing) and generate static, tested code. The "no mechanical processing" principle — every data interaction goes through an ontology.

6. **Cross-domain functor proofs.** Proving that domains ARE instances of abstract ontologies: traffic IS a system (TrafficToSystems), chess IS concurrent (ChessToConcurrency), chess IS event-driven (ChessToEvents), systems ARE concurrent (SystemsToConcurrency), event-driven IS concurrent (EventsToConcurrency). The equivalence triangle System ↔ EventDriven ↔ Concurrent is proven by functor composition.

7. **Lambek + Montague + DisCoCat in Rust with property-based testing.** Implementing categorial grammar, compositional semantics, and the syntax→semantics functor in Rust with exhaustive category law verification. Existing implementations (DisCoPy) are in Python without formal verification. pr4xis proves the functor laws hold via property-based testing.

## Related

- [Architecture](architecture.md) — the five-layer Rust stack and runtime mechanics
- [Concepts](concepts.md) — what ontologies are and how they compose via functors
- [README](../../README.md) — the project entry point with the LLM contrast table and the bioelectricity gap-detection result
- Per-ontology citings.md (pending [#57](https://github.com/i-am-logger/pr4xis/issues/57)) — once each ontology has its own bibliography, this document becomes the workspace-wide foundations index that the per-ontology files cite into

---

- **Document date:** 2026-04-14
- **Note on paper paths:** This document used to point at `docs/papers/*.pdf` for several references. Per [#57](https://github.com/i-am-logger/pr4xis/issues/57), the PDFs live alongside the ontologies that cite them via per-ontology `citings.md` files; this document cites sources by author/year only.
