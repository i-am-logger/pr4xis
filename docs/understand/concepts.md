# Concepts

This document explains the conceptual model behind pr4xis — what an ontology *is* in this system, why category theory is the substrate, and how domains compose. For the layer structure and runtime mechanics, see [Architecture](architecture.md). For the academic lineage and source papers, see [Foundations](foundations.md).

## What is an ontology in pr4xis?

Most ontology systems treat an ontology as a **graph of facts** — a set of triples saying that A is-a B and B has-part C, queryable via SPARQL or a graph database. pr4xis treats an ontology as a **category** — a mathematical structure with objects, morphisms, composition, and identity, plus a set of reasoning systems built on top of that structure.

The difference matters because it determines what you can prove. A graph of facts can be queried; a category can be *composed* with other categories under proof, and the composition can be checked at compile time and test time. If two ontologies share structure, a categorical functor between them is a theorem about that shared structure — not a heuristic, not an alignment score, not a similarity measurement.

Every domain in `crates/domains/src/` is an ontology in this stricter sense: an enum of concepts (the objects), an `Arrow` impl whose morphisms carry `Kind` tags (Subsumption / Parthood / Causation / Opposition / Equivalence / domain-specific kinds), the structural axioms attached to each kind by the catalog (no cycles, antisymmetric subsumption, symmetric opposition, …), any domain-specific axioms the source paper motivates, and a `Provenance` carried by `fn meta()` for trace attribution. The `ontology!` macro emits all of this from a single declarative block.

## Categories

A category has:

- **Objects** — the things the category is about. In pr4xis, every object is a `Concept` (Guarino 2009 — a finite, enumerable Rust enum variant).
- **Morphisms** — directed maps between objects. In pr4xis, every morphism is an `Arrow` between two concepts, carrying a `Kind` tag and per-instance provenance (Mac Lane 1971; Awodey 2010).
- **Composition** — if `f: A → B` and `g: B → C`, then `g ∘ f: A → C` exists and is itself a morphism.
- **Identity** — for every object `A`, there is a morphism `id_A: A → A`.

Two laws govern these:

- **Associativity:** `(h ∘ g) ∘ f = h ∘ (g ∘ f)` — the order of grouping does not matter.
- **Identity:** `id_B ∘ f = f = f ∘ id_A` — composing with identity changes nothing.

These laws sound trivial but they have a consequence pr4xis exploits everywhere: **if your domain model satisfies them, it has no dead states, no unreachable objects, and no broken compositions.** The laws are themselves first-class axioms: `category::laws::assert_category_laws` verifies them for every category in the workspace, and each underlying axiom's `verify()` returns a typed `Verdict` — a `Proof` when the law holds or a `Counterexample` that names what broke, never a bare boolean or an error string.

## Reasoning systems

A category is a structure. **Relation kinds** are interpretations of its morphisms that answer specific kinds of questions. The structural-axioms catalog (`pr4xis::ontology::reasoning::structural_axioms_for`) reads each morphism's `Kind` and attaches the right algebraic properties (OBO-RO; Smith et al. 2005; Tarski 1941):

- **Subsumption** (`is-a`) — `NoCyclesOnKind` (a thing cannot be its own ancestor) and `AntisymmetricOnKind` (if A is-a B and B is-a A, then A = B). Answers "is dog a mammal?".
- **Parthood** (`part-of`) — `NoCyclesOnKind`. The full CEM `WeakSupplementation` (Casati & Varzi 1999) is available as a hand-written domain axiom for ontologies that need it. Answers "what are the parts of an esophagus?".
- **Causation** (`causes`) — `AsymmetricOnKind` (Lewis 1973) and `IrreflexiveOnKind`. Answers "what caused this event?".
- **Opposition** (`opposes`) — `SymmetricOnKind` (if A opposes B, B opposes A) and `IrreflexiveOnKind` (a thing does not oppose itself). Answers "what is the opposite of cold?".
- **Context** — disambiguates concepts by context (`ContextDef::resolve`). A potassium channel in a constitutive context is not the same as a potassium channel in a therapeutic context. Closes gaps that adjunctions surface.

The `ontology!` macro provides sugar clauses (`is_a:` / `has_a:` / `causes:` / `opposes:`) for the canonical kinds and a free-form `edges:` clause for any other kinded morphism the ontology needs.

## Functors

A **functor** is a structure-preserving map between two categories. If `F: Source → Target` is a functor, then:

- For every object `A` in the source, there is an object `F(A)` in the target.
- For every morphism `f: A → B` in the source, there is a morphism `F(f): F(A) → F(B)` in the target.
- Identities are preserved: `F(id_A) = id_{F(A)}`.
- Composition is preserved: `F(g ∘ f) = F(g) ∘ F(f)`.

The third and fourth conditions are the **functor laws**. If a Rust `impl Functor for X` passes `category::laws::assert_functor_laws`, the laws hold and the functor is a categorically valid claim that the source domain's structure embeds into the target. Like the category laws, each functor law is an axiom whose `verify()` returns a typed `Verdict` — a `Proof` or a `Counterexample`, not a boolean.

This is what pr4xis means when it says "domains compose with proof". A functor from `Pharmacology → Molecular` is not an analogy or a heuristic mapping — it is a verified theorem that pharmacological structure faithfully embeds in molecular structure. The workspace ships more than 95 such functor implementations; to count the current total, run `grep -rn "impl Functor" crates/domains/src/ crates/pr4xis/src/ | wc -l`.

## Adjunctions and gap detection

When two functors come in opposed pairs, `F: A → B` and `G: B → A`, with `F` going one way and `G` going the other, the pair may form an **adjunction**. The technical definition involves natural transformations called *unit* (`η: Id_A → G ∘ F`) and *counit* (`ε: F ∘ G → Id_B`); the practical consequence is that `F` and `G` are "optimal inverses" of each other, even when neither is a true inverse.

The reason adjunctions matter for pr4xis is **gap detection**. If you take an object `A` in the source category and apply `G(F(A))` — a round-trip through both functors — you get back to the source category. If `G(F(A)) ≠ A`, the source ontology has a missing distinction: the round-trip collapsed `A` into something else because the target ontology cannot represent the difference.

Every collapsed entity is a missing distinction the math detected automatically. This is how the bioelectricity adjunction in `crates/domains/src/natural/biomedical/` discovered that voltage-gated potassium channels (Kv) serve two functionally distinct roles — homeostatic and therapeutic — that the molecular ontology had collapsed into a single entity. The adjunction surfaced the gap; a `ContextDef` resolution then disambiguated the two roles, and the gap closed.

For the live percentages of how much information is lost in each round-trip across the biomedical stack, run `cargo test -p pr4xis-domains test_full_chain_collapse_measurement -- --nocapture`.

## Composition is the point

Categories are the substrate. Functors are the maps between them. Adjunctions are the paired functors that detect what's missing. Together they answer one question directly: **does this composition preserve structure, with proof?**

The functor from pharmacology to the molecular ontology is not just an alignment exercise. If the functor laws hold, the composition is a theorem. If they don't hold, you cannot pretend the two ontologies are saying the same thing — the system tells you exactly which morphism breaks. The biomedical adjunctions go further: a round-trip through the paired functors surfaces a distinction the target ontology cannot represent — the Kv-channel gap detection above is exactly such a case.

This is what pr4xis adds to the existing landscape of formal ontologies. The ontologies have been there for decades; the categorical substrate that makes their composition machine-checkable is the missing piece.

## The Self-Model — categories all the way down

Because pr4xis describes its own structure with the same machinery it uses for any other domain, there is a self-model ontology (`crates/domains/src/cognitive/cognition/self_model.rs`) that models pr4xis's own architectural concepts — what an ontology is, what a reasoning system is, how they relate — as objects in a category, through exactly the same `Ontology` trait it uses for biology or chess. Self-reference is modeled categorically as a natural transformation, not as a special case in the runtime.

This is a small but load-bearing detail: it is the reason pr4xis can extend itself without bolting on metaprogramming. Every new capability is just another ontology, and every new capability is automatically composable with everything that came before.

## Reading legal text

Legal text is English, but it does not read like English. A statute can redefine a word for its own purposes, and when it does, the redefinition is law, not lexicography: in ordinary English a "person" is a human being, but 1 U.S.C. §1 — the **Dictionary Act** — provides that throughout the U.S. Code "person" includes corporations, companies, associations, firms, partnerships, societies, and joint stock companies as well as individuals. A reader that brings only a dictionary's senses to a statute gets the law wrong. pr4xis models this with two pieces: a lexicon in which one word carries many senses, and a precedence order that decides which definition governs a given use.

**One word, many senses.** The lexicon (`crates/domains/src/cognitive/linguistics/lemon/lexicon.rs`) follows the W3C OntoLex-Lemon model (2016; McCrae et al. 2017): each written word has exactly one `LexicalEntry`, and that entry carries many `Sense`s, each pointing at an ontology concept. `Lexicon::add_sense` *appends* to the one shared entry — so when the legal layer teaches the lexicon a legal meaning of "person", the word is not duplicated and its ordinary meaning is not overwritten; both senses live side by side. A sense may carry a domain marker (OntoLex's `dct:subject`, e.g. `"legal"`), and which sense is *predominant* depends on the domain of the question being asked — Koeling, McCarthy & Carroll (2005) showed the predominant sense of a polysemous word is domain-dependent. The ranking is simple: a sense whose domain matches the query is most salient, a general unmarked sense is the default fall-through, and a sense from some other domain ranks last. `Lexicon::resolve(word, domain)` returns the winner, and the ordering is not merely asserted: the `SenseOrderIsStrictPartialOrder` axiom verifies it is irreflexive, asymmetric, and transitive, so "the predominant sense" is always well-defined.

**A defined term is a first-class sense.** When a statute defines a term, pr4xis records a `LegalDefinition` (`crates/domains/src/social/judicial/statute_structure/definition_scope.rs`): the term, the scope it applies to, and the concept it binds the term to. The load-bearing modeling choice is that the defined term's *identity anchors in the legal definition, not in the English word*: legal "person" references a concept of its own (`usc_title_1:person`), distinct from WordNet's `person.n.01`. `DefinitionLexicon::mint_into` then mints each defined term into the shared lexicon as a `"legal"`-domain sense — alongside, never instead of, the general sense.

**The precedence ladder.** A term like "person" may be defined at several scopes at once, captured by `DefinitionScope`:

- **Enacted** — a definition with stated applicability ("In this section …", "For purposes of this title …" — 26 U.S.C. §7701). It governs every use inside the subtree its citation names, and its specificity rises with the scope's depth: a section-level definition is more specific than a title-level one.
- **DictionaryAct** — 1 U.S.C. §1, the default for the entire U.S. Code.
- **OrdinaryMeaning** — no statutory definition; the word means what English says it means.

When more than one scope governs a use, the more specific displaces the more general — the general/specific canon from the statutory-interpretation literature (Scalia & Garner 2012, *Reading Law* §28, *lex specialis*). So the ladder reads: enacting section > title-wide definitions > the Dictionary Act > ordinary English. pr4xis models this as a *priority ordering*, not as disjoint namespaces, because resolution is a fall-through — when a rung does not apply, the use falls to the next — and a fall-through is an ordering (Reiter 1980 on default reasoning; Prakken & Sartor 1996 on defeasible rule priorities in legal reasoning). The `DefinitionScopePrecedenceIsStrictPartialOrder` axiom verifies the ladder is a strict partial order, so a well-defined governing definition always exists.

The ladder is also **defeasible — it yields**. The Dictionary Act applies "unless the context indicates otherwise", and §7701 yields where its definition would be "manifestly incompatible" with the provision at hand — the Supreme Court treated exactly this clause as a soft, contextual escape in *Rowland v. California Men's Colony*, 506 U.S. 194 (1993). `resolve_definition` therefore takes a contextual-defeater predicate alongside the candidate definitions: a defeated definition does not abort resolution, it falls through to the next-most-specific governing definition.

**"person", resolved twice.** `dictionary_act_definitions()` ships the twelve Dictionary Act terms — "person", "whoever", "officer", "signature", "subscription", "oath", "sworn", "writing" (1 U.S.C. §1), "vessel" (§3), "vehicle" (§4), "company" and "association" (§5) — each bound at code-wide scope. Mint them into a lexicon that already knows WordNet's "person", and the same word resolves differently per register:

```rust,ignore
let mut lex = Lexicon::new("en");
lex.add_sense("person", "english_wordnet", "person.n.01", None);
dictionary_act_definitions().mint_into(&mut lex);

lex.resolve("person", Some("legal")); // → usc_title_1:person  (the Title-1 sense)
lex.resolve("person", None);          // → english_wordnet:person.n.01
```

One entry, two senses, both reachable: the legal register elevates the statutory meaning, the default stays WordNet's.

This is the *definitional* layer — which sense a word resolves to, and which definition governs where. The Dictionary Act layer is a hand-coded prototype of what the Title 1 corpus loader will produce, and typed statute-to-statute cross-references ("as defined in section 3(a)" resolving to the cited provision as a first-class edge) are future work.

## Related

- [Architecture](architecture.md) — the five-layer Rust stack and runtime mechanics
- [Foundations](foundations.md) — academic lineage; every concept above traced to its source paper
- [README](../../README.md) — the project entry point with the LLM contrast table and the bioelectricity gap-detection result
- Per-ontology READMEs (pending [#57](https://github.com/i-am-logger/pr4xis/issues/57)) — for what each individual ontology contains
- Per-ontology diagrams (pending [#59](https://github.com/i-am-logger/pr4xis/issues/59)) — for the visual "neural network of an ontology" view

---

- **Document date:** 2026-04-14
- **Verification:** the category and functor law axioms (`category::laws::assert_category_laws` / `assert_functor_laws`) are exercised by `cargo test -p pr4xis category`; the functor count comes from `grep -rn "impl Functor" crates/domains/src/ crates/pr4xis/src/`; the round-trip collapse measurement from `cargo test -p pr4xis-domains test_full_chain_collapse_measurement -- --nocapture`; and the legal-text resolution behavior (sense elevation, lex-specialis precedence, the contextual defeater, the "person" example) from `cargo test -p pr4xis-domains definition_scope` and `cargo test -p pr4xis-domains lexicon`.
