# Glossary

Terms used across the pr4xis docs, in plain English. For deeper coverage of any of these, see [Concepts](../understand/concepts.md), [Architecture](../understand/architecture.md), or [Foundations](../understand/foundations.md).

## Axiom

A statement that is taken as given, without needing to be proven from anything else. The starting point of a chain of reasoning. In pr4xis, every axiom is either grounded in a published source (a textbook, a paper, a standard) or is a structural rule of the substrate itself (no cycles in taxonomies, etc.). When pr4xis says a claim is provable, it means the claim can be derived from the axioms by logical and categorical operations.

## Ontology

In pr4xis, an ontology is more than a list of facts. It is a **category** (in the formal mathematical sense) of concepts and the kinded morphisms between them (subsumption, parthood, causation, opposition, …), plus the axioms the structure must satisfy. Every domain in pr4xis — biology, chess, sensor fusion, traffic signals, judicial workflow — is an ontology in this stricter sense. Authored declaratively via the `ontology!` proc macro.

## Category

A mathematical structure with **objects**, **morphisms** (directed maps between objects), **composition** (combining two morphisms into a third), and **identity** (a morphism that does nothing). Two laws govern composition: associativity (the order of grouping doesn't matter) and identity (composing with identity changes nothing). pr4xis treats every domain as a category and verifies the laws at test time. See [Concepts](../understand/concepts.md) for the long version.

## Morphism

A directed map from one object to another inside a category. In pr4xis, a morphism is an `Arrow` between two `Concept` values, carrying a `Kind` tag and per-instance provenance — for example, a `Subsumption`-kinded `Arrow` `Dog → Mammal` represents "dog is a mammal".

## Functor

A **structure-preserving map between two categories**. If `F: A → B` is a functor, then every object `x` in category `A` has a corresponding object `F(x)` in category `B`, and every morphism `f: x → y` in `A` has a corresponding morphism `F(f): F(x) → F(y)` in `B`. Two laws hold: identities are preserved, and composition is preserved. In pr4xis, functors are how two ontologies are *proved* to share structure — when the functor laws hold, the source ontology faithfully embeds in the target. The workspace has more than 95 functor implementations; run `grep -rn "impl Functor" crates/domains/src/ crates/pr4xis/src/ | wc -l` for the live count.

## Functor laws

The two conditions a functor must satisfy:

1. **Identity preservation**: `F(id_x) = id_{F(x)}` — the identity morphism in the source maps to the identity morphism in the target.
2. **Composition preservation**: `F(g ∘ f) = F(g) ∘ F(f)` — composing two morphisms before mapping gives the same result as mapping each then composing.

Each law is an `Axiom` whose `verify()` returns a typed `Verdict` (proof or counterexample); they live at `crates/pr4xis/src/category/laws.rs` as `functor_law_axioms::<F>()` / `assert_functor_laws::<F>()`, run by `cargo test -p pr4xis category::laws`. Every functor in pr4xis must pass.

## Adjunction

A pair of functors `F: A → B` and `G: B → A` that are "optimal inverses" of each other in a precise categorical sense (the unit `η: Id_A → G ∘ F` and counit `ε: F ∘ G → Id_B` natural transformations satisfy the triangle identities). In pr4xis, adjunctions are the mechanism for **gap detection**: when you take an object in `A`, apply `F` to get its image in `B`, then apply `G` to come back, you should get the original object. If you don't, the original ontology has a missing distinction the math just surfaced. The bioelectricity Kv discovery is the canonical example — see [Gap detection](../research/gap-detection.md).

## Reasoning system

In pr4xis, a category becomes an ontology when one or more reasoning systems are layered on top of it, each interpreting the morphisms in a specific way:

- **Taxonomy** — interprets some morphisms as `is-a`, with axioms `NoCycles` and `Antisymmetric`
- **Mereology** — interprets some morphisms as `part-of`, with axiom `WeakSupplementation`
- **Causation** — interprets some morphisms as `causes`, with axiom `NoSelfCausation`
- **Opposition** — interprets some morphism pairs as `opposes`, with axioms `Symmetric` and `Irreflexive`
- **Context** — disambiguates entities by context (`ContextDef::resolve`)
- **Analogy** — `Analogy<F>` is a wrapper around a functor `F`, treating the functor as a proven analogy between two domains

Each reasoning system is a Rust trait that an ontology can implement.

## Engine

The runtime layer of pr4xis. An `Engine<A>` carries a current `Situation` (immutable world state), a list of `Precondition`s (rules that must hold before any action), and a function that applies an `Action` to produce a new `Situation`. When you call `engine.next(action)`, the engine checks every precondition; if all pass, the action is applied and a trace entry is recorded; if any fail, an `EngineError::Violated` is returned with the failing precondition named, and the engine is recoverable for rollback. Supports `back()`, `forward()`, and branching.

## Situation

An **immutable snapshot of the world** at a single point in time. Every action produces a new situation; the old situation is preserved in the engine's history stack. This is what enables undo, redo, and branching without mutation.

## Action

A proposed change to the current situation. Actions are checked against the engine's preconditions before they are applied. An action that violates a precondition is blocked, named, and recoverable — never silently approximated.

## Precondition

A rule that must hold before an action can be applied. A precondition takes the current situation and the proposed action and returns either `Satisfied` (with a reason) or `Violated` (with the failing rule and a diagnostic). Both carry context, so traces are useful for debugging and auditing.

## Trace

The full structured history of every action the engine has processed. Each trace entry records the precondition results, the resulting situation (or the violation), and the timing. Used for audit, replay, and the `TracedPipeline` writer monad in chat-style applications.

## Quality

A property that inheres in an entity (DOLCE term). For example, in the colors ontology, an `Rgb` entity has a `Luminance` quality. Qualities are how pr4xis attaches measurable or comparable values to objects without making the objects themselves carry the values.

## Substrate

A word the README uses for "the engineering layer that makes ontologies composable with mathematical proof". Loosely: the parts of pr4xis that aren't specific to any one domain — the categorical machinery, the engine, the reasoning systems, the validators. The opposite of "the domains" themselves.

## DOLCE

A foundational ontology from Masolo et al. (2003) that classifies all of being into Endurants (physical objects, social objects, mental objects), Perdurants (events, processes), and Qualities. pr4xis uses DOLCE as the upper-layer classification that domain ontologies classify their concepts against.

## WordNet

An open lexical database of English (~107K concepts, decades of curation). pr4xis ingests WordNet via its `codegen::wordnet` build-time generator and exposes it as the English ontology. The WASM browser demo at [pr4xis.dev](https://pr4xis.dev) loads it at startup.

## ontology!

A Rust proc macro at `pr4xis::ontology` (re-exported from `pr4xis-derive`) that takes a declarative ontology specification (`name:`, `source:`, `concepts:`, `labels:`, optional sugar clauses `is_a:` / `has_a:` / `causes:` / `opposes:` for the canonical kinds, optional free-form `edges:` for other kinded morphisms, optional inline `axioms:` block) and emits the full implementation: the `Concept` enum, the `Category` impl, the `Arrow` impl with kind tagging, an `Ontology` impl whose `fn axioms()` inherits structural axioms from the catalog, and a type-level `fn meta() -> Provenance` for trace attribution. The macro is the canonical way to author an ontology in pr4xis. Every file at `crates/domains/src/**/ontology.rs` is an instance.

## Provenance

The type returned by every ontology's type-level `fn meta()` (and by every `Functor`, `Adjunction`, and `Axiom` for that matter). Carries the `name`, `description`, and `citation` set by the `ontology!` macro from its `name:` + `source:` clauses, plus version metadata. Used by the engine to attribute trace entries to the ontologies that produced them. Defined at `crates/pr4xis/src/ontology/meta.rs`.

## Categorical extensional mereology (CEM)

The classical formal theory of parts and wholes — Simons (1987), Stanford Encyclopedia of Philosophy on [Mereology](https://plato.stanford.edu/entries/mereology/). In pr4xis, parthood is expressed as `Parthood`-kinded morphisms in an ontology's `Category` — the structural-axioms catalog (`structural_axioms_for::<C>()`) attaches `NoCyclesOnKind` automatically (OBO-RO; Smith et al. 2005). The full CEM `WeakSupplementation` axiom (if a whole has a proper part, it has another disjoint part — Casati & Varzi 1999) is not in the catalog; ontologies that need it add it as a domain axiom in their `Ontology::axioms()` impl. Heim's modernized syntrometric logic also grounds part/whole reasoning in CEM, which is one of the structural alignments cited in [Foundations](../understand/foundations.md).

## Kripke semantics

A formal semantics for modal logic in which truth depends on which "possible world" you are evaluating in. pr4xis does not use full Kripke semantics today, but the modernized syntrometric logic tradition (Heim 1980, formalized 2025) does, and pr4xis's pattern of multiple ontologies viewing the same domain through different functors is the computational realization of an aspect-relative Kripke frame. See the foundations doc for the connection.

## Property-based testing

A testing technique in which invariants are expressed as properties that must hold for ALL inputs, and a library generates random inputs to look for counterexamples. pr4xis uses [proptest](https://github.com/proptest-rs/proptest) for property-based testing of category laws, axiom satisfaction, and domain invariants. See the [Wikipedia article on property testing](https://en.wikipedia.org/wiki/Software_testing#Property_testing) for the broader context.

## Manifest (`praxis.toml`)

The declarative registry of external sources praxis knows about. Lives at the workspace root, one `[sources.<name>]` block per source, naming the version, the [`SourceTaxonomy`](#sourcetaxonomy) type, and the authoritative URL. Read at startup; unknown types fail closed. See [Register a Source](../use/register-a-source.md).

## Lock (`praxis.lock`)

The integrity layer next to [`praxis.toml`](#manifest-praxistoml). Pins the expected content digest for every registered source's on-disk bytes under `[hashes]`, in the tagged grammar `<algorithm>:<64 lowercase hex>` — `blake3:` for every praxis-emitted pin (BLAKE3 is the one emit algorithm), `sha256:` / bare hex loadable as SHA-256. The `LockManifestAgreement` axiom verifies manifest, lock, and local file all agree.

## PdfBuildExtraction

The typed const a codegen module emits at build time for sources whose authoritative format is PDF (see `crates/domains/src/applied/data_provisioning/build_extraction.rs`). One of five variants — `Extracted { text, bytes_hash }` / `NotOnDisk` / `ParseFailed` / `Encrypted` / `UnsupportedContentType`. Downstream `canonical_audit.rs` modules pattern-match on the variant. Anchored against W3C PROV-O (Lebo et al. 2013) as a typed `prov:Activity` outcome; each variant cites either an ISO 32000-2 section or a Wilkinson FAIR principle. The `PdfBuildExtractionTotality` axiom enforces exhaustiveness.

**Scope:** PDF-format sources are case law (court opinions), administrative orders, and similar court-system publications. **Statutes are NOT in this set** — US statutes load via USLM XML from `uscode.house.gov` per 1 U.S.C. § 204, through the bytes ⇄ Statute composed lens (M4.λ.3.b), not through `PdfBuildExtraction`. See the [`Registered source`](#registered-source) entry's content-type matrix for the canonical format per source category.

## Registered source

A `[sources.<name>]` entry in [`praxis.toml`](#manifest-praxistoml) plus its matching [`praxis.lock`](#lock-praxislock) entry plus the on-disk artifact at the path derived from the source's [`SourceTaxonomy`](#sourcetaxonomy) type. The unit the engine reasons about; the unit the [`pr4xis update`](../use/register-a-source.md#the-cli--pr4xis-update) CLI fetches and verifies.

## SourceTaxonomy

The ontology behind a registered source's `type` field. Roots at `Source` and branches into several families — `Lexicon` (`Language`, `DomainLexicon`, `LegalLexicon`, `SchemaVocabulary` leaves), `LegalCorpus` (`Statute` / `UsFederalStatute`, `UsCodeTitle`, `Regulation`, `ConstitutionalArticle`, `ProceduralRule`, `CaseLaw` leaves), and schema/test families (`SchemaSpec` with leaves including `OntologyVocabulary`, plus `TestSuite`). The `is_a` chain drives the decoder family and the on-disk path convention; `Adjoins` edges connect families that interoperate at runtime. Hart 1961's primary/secondary rule distinction attaches as a quality. The full leaf set is the `concepts:` block in `crates/domains/src/formal/meta/source_taxonomy/ontology.rs`.

## Data provisioning

The engine subsystem at `pr4xis_domains::applied::data_provisioning` that reads [`praxis.toml`](#manifest-praxistoml) and [`praxis.lock`](#lock-praxislock), exposes typed `RegistryEntry` values to the rest of the runtime, and enforces eight axioms over the registered set (`LockManifestAgreement`, `RegistryUniquenessByNameVersion`, `IdentityClaimsUseLeaves`, `DecoderTotalityPerKind`, …). Loaded once per process via `OnceLock`; new manifest entries are visible only after process restart. The [`pr4xis update`](../use/register-a-source.md#the-cli--pr4xis-update) CLI is the operator-side surface of the same subsystem.

## `.prx`

The self-contained archive format praxis packs a loaded source into. praxis reads a `.prx` back in milliseconds — instead of re-parsing the original source — after checking its fingerprint and refusing anything that has been altered, and it can still rebuild the original bytes exactly. Today praxis archives its own OWL ontologies, its U.S. Code (USLM) text, and the English dictionary (WordNet) this way; the English and U.S. Code archives are compact — smaller than the source download — for the fast read-back. The format is one realisation of the [Archive](#archive) ontology; the realisation lives at `crates/domains/src/social/software/markup/xml/owl/prx.rs` (gated on `feature = "prx"`).

## Archive

Content-addressed Merkle-DAG storage for an archived source, declared as the `OntologyArchiveStorage` ontology (`crates/domains/src/formal/meta/ontology_archive/`) rather than described only in prose — its concepts (`ContentAddressableNode`, `MerkleDag`, `MerkleRoot`, `BinaryEnvelope`, `CompressedForm`, `SourcePin`, `LoadGate`, `IntegrityClaim`) and its guarantees are runnable axioms. Each node is named by the cryptographic hash of its bytes (Merkle 1987; Benet 2014 IPFS), so identical content yields the identical address and the store deduplicates. The emit/load round-trip is a well-behaved lens (Foster et al. 2007): rebuilding from the archive reproduces the source bytes exactly.

## Graph slice / GraphSnapshot

A content-addressed graph-*slice* primitive (`crates/domains/src/formal/meta/praxis_knowledge_graph/snapshot.rs`): select a slice of the knowledge graph as the **relational image** of a `RootSet` under an `EdgeKindFilter` — computed through the category's own `morphisms_from` over the transitive closure the `ontology!` macro materializes, not a re-derived traversal — and content-address it as a Merkle DAG. The result is a `ReachableSubgraph`. Edges of a filtered kind that *leave* the slice (`from` inside, `to` outside) are its `UnboundReference`s; a slice with none is closed. The slice rehydrates through the same fail-closed admit gate the `.prx` archive uses, reusing the same content-hash and codec primitives — no parallel hash or codec.

## IntegrityClaim

A typed, verifiable claim binding a resource to its expected content hash (W3C Subresource Integrity 2016) — a first-class concept in the [Archive](#archive) ontology, not a bare string compare. The underlying content hash is multi-algorithm: the `RawHash` leaf of the `ArtifactIdentity` taxonomy (`crates/domains/src/formal/meta/artifact_identity/`) covers SHA-256, SHA-512, and BLAKE3 (the archive's `SourcePin` records a BLAKE3 content address — praxis emits under one algorithm and verifies claims under any). A claim is discharged — never merely trusted — by the [fail-closed load gate](#fail-closed-load-gate).

## Fail-closed load gate

The `LoadGate` concept (and the `LoadGateFailsClosed` axiom) of the [Archive](#archive) ontology. It admits a node only by *re-deriving* the content address from the node's own bytes and checking it equals the externally recorded pin; it never trusts an embedded self-asserted label. On a mismatch, an unverifiable claim, or an absent pin, nothing is installed. Grounded in Dolstra (2006) fixed-output derivations, W3C SRI (2016), and TUF (Samuel et al. 2010).

## Codegen / async loading / mmap

Three different mechanisms pr4xis supports for delivering ontology data into the runtime, all proven equivalent as functors from the same `OntologyBuilder` source:

- **Codegen** (build-time): pre-compile declarative source into static Rust. Used by the WordNet ontology in the WASM demo.
- **Async loading** (runtime): load ontology data from a file or stream asynchronously. Used for ontologies that are too large to embed or that need hot reloading.
- **Memory-mapped files** (runtime, zero-copy): mmap a precomputed binary directly into memory.

The choice between them is operational, not semantic. See [Architecture](../understand/architecture.md) for the layer description.

---

- **Document date:** 2026-04-14
- **Verification:** every term that names a code element (`ontology!`, `Engine`, `ContextDef`, etc.) corresponds to actual code in `crates/pr4xis/src/` or `crates/domains/src/`. Grep to verify.
