# Changelog

## [0.28.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.25.5...pr4xis-domains-v0.28.0) - 2026-07-10

### Chore

- move the remaining internal working docs out of the published docs/ tree

### Docs

- fix 109 feature-gated intra-doc links — the dev-ci docs stage was red
- *(legal_sources)* clarify the strict-subsumption test comment (PR #231 review)

### Feat

- *(english)* [**breaking**] audit-5 wave 5 — ship the 9 store buffers as the load payload; the +348 MiB wasm transient is dead
- *(hardening)* audit-5 wave 4 — validated PackedCsr entry, load ∀-properties, self-describing load envelope
- *(data)* [**breaking**] DEFLATE the raw-source .prx tier — envelope v2 with a typed PayloadEncoding
- *(domains)* the lens machinery self-describes -- Lens ontology (W3.4)
- *(domains)* the succinct wire codec self-describes -- SuccinctCodec ontology (W3.3)
- *(domains)* the DAG-CBOR canonical codec self-describes -- CanonicalCodec ontology (W3.1)
- *(domains)* words (declared types) are pointers into English (W2.2)
- *(domains)* [**breaking**] generalize grounding -- any .prx carries its grounding functor as data (W2.1)
- *(domains)* SourceRole ontology -- the sources catalog shows only chat-loadable knowledge (Step 3a)
- *(domains)* statutes compose -- a loaded USC section reaches the statute/law taxonomy (Step 4)
- *(runtime)* Lever A -- RuntimeOntology reasons over the archived rkyv buffer, zero-copy (Step 1c)
- *(domains)* intern lexical surfaces -- Symbol-keyed surface_index (Step 1b)
- *(runtime)* lazy memoized reachability -- drop the pre-folded owned closure (Step 1a)
- *(runtime)* [**breaking**] mint lexical surfaces on every emitted ontology by default
- *(chat)* answer conceptual legal questions from a loaded ontology
- *(domains)* constitutive protocol — moderation events, modes, and admissions (prx parity)
- *(domains)* smart_element — the smart driver / smart sensor synthesis
- *(domains)* applied/swarm family — consensus + distributed fusion
- *(domains)* applied/operating_system family — microkernel, scheduler, bus, driver
- *(domains)* concurrency, parallelism, and constitutive-protocol ontologies
- integrate the new ontologies via literature-grounded functors
- [**breaking**] type the ontology stack end-to-end (Vector/Matrix, coordinate/level/angle)
- constitution coverage — every test declares its guarantee, gate-enforced

### Fix

- *(packed-csr)* [**breaking**] declare per-column run arity (RunArity) — fixes the untrusted-buffer GET panic
- *(audit)* audit-4 — 21 confirmed findings on the post-review commits, all fixed or justified
- *(example)* stop resident_memory holding the test-only english_runtime_ontology foil
- *(review)* third-review polish + hardening -- doc accuracy, test teeth, one primitive-leak
- *(grounding)* make into-English reachable via the public load path + fail-close two silent drops
- *(audit)* reground misattributed/over-claimed citations + delete a tautological axiom
- *(domains)* repr(transparent) on Ref -- guarantee the zero-copy cast soundness
- *(domains)* sync the stale registry-root const in build.rs + add drift-guard
- *(domains)* correctness deep-review — founder-untouchability + scheduler deadline horizon
- *(domains)* ontological-purity audit corrections across the smart-edge ontologies
- *(docs)* repair broken intra-doc links surfaced by the Docs gate
- *(data)* pin cito/doco to immutable SPAR URLs, not mutable /current/
- make rubber-stamp axioms falsifiable + add property-based coverage
- harden 18 latent panic/overflow sites (audit pass-2 latent findings)
- 9 more reachable panic/DoS sites from a second, deeper audit
- 3 more reachable panic/DoS sites the audit verifier over-refuted
- bound 4 more reachable panic/DoS sites found by an exhaustive audit

### Perf

- *(reasoner)* [**breaking**] audit-5 wave 2 — loaded-only surface overlay (−15.5 MiB, + a real dup bug fixed)
- *(reasoner)* [**breaking**] audit-5 wave 1 — kill 4 owned-re-copy gaps (measured −22 MiB + a leak class)
- *(lens)* owned-consuming put leg — the rich stores MOVE into their mirrors at build
- *(english)* archive the remaining owned relational maps — English owns no WordNet HashMap
- *(wasm,domains)* single substrate instance -- collapse the ~112 MiB double-hold (W2)
- *(domains)* [**breaking**] English IS a RuntimeOntology -- archived taxonomy edges + Sync per-query BFS, drop the eager closure (W1)
- *(domains)* drop English's retained sense_to_id index -- reclaim ~13.9 MiB
- *(domains)* English concept records as a zero-copy archived buffer (S3)
- *(domains)* English word_index as a zero-copy archived buffer -- reclaim ~240 MiB (S2)

### Refactor

- *(reach)* [**breaking**] ReachSubstrate engine — LazyKindReach and the 216 MiB English bridge deleted
- *(reach)* [**breaking**] one graded-reach kernel for both engines — the de-privilege-English core
- *(english)* all 9 archived stores → 2 cited WellBehavedLens families; English owns nothing

### Test

- *(laws)* audit-5 wave 6 — the last census cells: grounding axioms, generated lens properties, one kernel, honest Lemon
- *(english)* direct cast tests for word_index + concept_store zero-copy stores

## [0.27.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.25.5...pr4xis-domains-v0.27.0) - 2026-07-06

### Docs

- *(legal_sources)* clarify the strict-subsumption test comment (PR #231 review)

### Feat

- *(runtime)* [**breaking**] mint lexical surfaces on every emitted ontology by default
- *(chat)* answer conceptual legal questions from a loaded ontology
- *(domains)* constitutive protocol — moderation events, modes, and admissions (prx parity)
- *(domains)* smart_element — the smart driver / smart sensor synthesis
- *(domains)* applied/swarm family — consensus + distributed fusion
- *(domains)* applied/operating_system family — microkernel, scheduler, bus, driver
- *(domains)* concurrency, parallelism, and constitutive-protocol ontologies
- integrate the new ontologies via literature-grounded functors
- [**breaking**] type the ontology stack end-to-end (Vector/Matrix, coordinate/level/angle)
- constitution coverage — every test declares its guarantee, gate-enforced

### Fix

- *(domains)* correctness deep-review — founder-untouchability + scheduler deadline horizon
- *(domains)* ontological-purity audit corrections across the smart-edge ontologies
- *(docs)* repair broken intra-doc links surfaced by the Docs gate
- *(data)* pin cito/doco to immutable SPAR URLs, not mutable /current/
- make rubber-stamp axioms falsifiable + add property-based coverage
- harden 18 latent panic/overflow sites (audit pass-2 latent findings)
- 9 more reachable panic/DoS sites from a second, deeper audit
- 3 more reachable panic/DoS sites the audit verifier over-refuted
- bound 4 more reachable panic/DoS sites found by an exhaustive audit

## [0.26.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.25.5...pr4xis-domains-v0.26.0) - 2026-07-04

### Feat

- integrate the new ontologies via literature-grounded functors
- [**breaking**] type the ontology stack end-to-end (Vector/Matrix, coordinate/level/angle)
- constitution coverage — every test declares its guarantee, gate-enforced

### Fix

- *(docs)* repair broken intra-doc links surfaced by the Docs gate
- *(data)* pin cito/doco to immutable SPAR URLs, not mutable /current/
- make rubber-stamp axioms falsifiable + add property-based coverage
- harden 18 latent panic/overflow sites (audit pass-2 latent findings)
- 9 more reachable panic/DoS sites from a second, deeper audit
- 3 more reachable panic/DoS sites the audit verifier over-refuted
- bound 4 more reachable panic/DoS sites found by an exhaustive audit

## [0.25.4](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.25.3...pr4xis-domains-v0.25.4) - 2026-06-19

### Build

- *(release)* version the domains crate via release-plz
- *(release)* single workspace version via inheritance — fix the release-plz drift

### Docs

- *(domains)* green the workspace rustdoc over the OWL/USC bridges
- *(domains)* fix two rustdoc intra-doc links (CI Docs step)
- complete validation overhaul — per-def trait sweep + rustdoc rot + mdBook ([#176](https://github.com/i-am-logger/pr4xis/pull/176))
- *(#173)* per-ontology rollout for the 5 new HMI sub-ontologies (#69)
- rewrite + per-ontology rollout (#57, #55, #52, #46, #44) ([#63](https://github.com/i-am-logger/pr4xis/pull/63))
- color science source papers
- pregroup grammar research — parsing as group algebra

### Feat

- *(input)* WmAction ontology + Hyprland realization functor
- *(web)* the chat shows every ontology it reasoned over, loaded ones included (U6, U7)
- *(web)* the self-model reports its own memory footprint (U2)
- *(runtime)* a snapshot's address is portable, not toolchain-bound (A7)
- *(runtime)* snapshot any ontology, not just the self-model graph (A6)
- *(runtime)* one generic loader for every envelope (A5)
- *(runtime)* a functor or adjunction in a snapshot re-binds, not refused (A4)
- *(runtime)* load the relation-kind vocabulary from the Relations ontology (A3, slice b)
- *(theming)* vogix16 semantic keys use snake_case to match the design system
- *(self-aware)* the catalog includes loaded-but-unregistered ontologies (§3)
- *(runtime,chat)* relation-parametric image/meet/chain — Parthood evidence chains
- *(chat)* single-word loaded entities type NP — "is X part of Y" parses with a one-word X
- *(self-aware)* content-addressed load history + state fingerprint (Step 6 / §2.4)
- *(self-aware)* per-ontology capabilities — what each loaded ontology can answer (§4.7)
- *(self-aware)* provenance names the loaded ontology a turn reasoned over (Step 5 / §2.3)
- *(self-aware)* the SelfModel eigenform observes the live loaded set (Step 4 / §2)
- *(lambek,chat)* "is X part of Y" parses from raw text (Step 5 — the parse)
- *(domains)* trim the relation lexicon to non-default complement-headed surfaces
- *(chat)* lower a question's predicate to a typed relation kind (Step 3)
- *(domains)* relation_lexicon.prx — the loaded "part of"↦Parthood surface map (Step 3)
- *(domains)* relation-parametric LexicalReasoner::reaches (Step 3 spine)
- *(domains)* the USC→praxis functor lives in usc_functor.prx, not Rust ([#203](https://github.com/i-am-logger/pr4xis/pull/203))
- *(domains)* the OWL→praxis functor lives in owl_functor.prx, not Rust ([#203](https://github.com/i-am-logger/pr4xis/pull/203))
- *(domains)* the WordNet→praxis functor lives in english_functor.prx, not Rust ([#203](https://github.com/i-am-logger/pr4xis/pull/203))
- *(uslm)* mint USC heading + citation Forms — sections answerable by "section N" (§9c)
- *(owl,wasm)* mint OWL label Forms — loaded OWL answerable by its label (§9b)
- *(composed)* index a loaded concept's Form-atom surfaces (§9 lexicalization)
- *(chat)* multi-token surface recognition — phrase/citation lookup
- *(uslm)* lift USC→praxis to functor-as-data (off the baked Parthood/Section)
- *(owl)* OWL→praxis as a functor-as-data projection (not a baked converter)
- *(derive,runtime)* transitive kinds are loaded from data — Phase A Step 4
- *(runtime)* relation kind is a ConceptRef, not a closed enum — Phase A Step 3
- *(runtime)* transitive_kinds — relation transitivity as loaded data, not a constant
- *(morphology)* ground the productive rules in registered CatVar; structural rule count (D-2/D-12)
- *(morphology)* irregular forms LOADED from the registered AGID source (MORPH/D-1)
- *(linguistics)* [**breaking**] literature-honest determiner/interjection features (FW-B)
- *(domains)* function-words load from a committed .prx — the ClosedClassLexicon (FW-A)
- *(linguistics)* wh-questions via a loaded OLiA→CCG category functor ([#169](https://github.com/i-am-logger/pr4xis/pull/169))
- *(linguistics)* math operators as a loaded vocabulary — the #169 tokenizer fix
- generic grounding — ground(lens) over the substrate, denotes is one lens
- *(domains)* USC → Archive — statute provisions in the generic substrate (the ontological way)
- *(domains)* persist the denotes pointer column in the USC envelope (G3b-2b code)
- *(domains)* the denotes-floor producer — statute prose → ontolex:Form pointers (G3b-2a)
- *(domains)* the honest denotes floor — a span grounds into an ontolex:Form (G3b-1)
- *(runtime)* EdgeTarget — the foreign-atom slot, byte-exact (grounding G2)
- *(domains)* bridge loaded English to a RuntimeOntology — the #87 grounding gate
- *(domains)* the WordNet→praxis functor, carried as .prx data
- *(domains)* project English into a runtime Archive — the WordNet→.prx source
- `.prx` — praxis' knowledge, in a file ([#186](https://github.com/i-am-logger/pr4xis/pull/186))
- [**breaking**] sync all praxis crates to one version (0.22.0) ([#190](https://github.com/i-am-logger/pr4xis/pull/190))
- [**breaking**] restore praxis publishing to crates.io ([#188](https://github.com/i-am-logger/pr4xis/pull/188))
- *(cli)* one-command corpus updates via `pr4xis update --lock` ([#183](https://github.com/i-am-logger/pr4xis/pull/183))
- praxis + praxis-cli gain the registered-source mechanism (SOX 1514A, AIR21 42121) ([#179](https://github.com/i-am-logger/pr4xis/pull/179))
- *(deps,ci)* [**breaking**] pure-Rust crypto + always-latest devenv + PR-title gate ([#177](https://github.com/i-am-logger/pr4xis/pull/177))
- *(#91)* pr4xis core + domains run no_std + alloc (#157)
- *(#148)* Lemon meta on every structural entity — uniform registry for ontologies, axioms, functors, adjunctions, nat-trans (#150)
- *(#117)* MAPE-K ontology — chat pipeline's literature-grounded home (#146)
- *(#62)* Heim syntrometric lineage — consolidated stack (#143)
- *(#62)* Heim syntrometry Phase 1 — lineage verified by functor laws (#135)
- *(#131)* TerminalFunctor helper — reusable one-object collapse (#134)
- *(#130)* Category::Op<C> + empirical 4th failure mode discovery (#133)
- *(#123)* Resilience ontology — Nygard/Brooker/Armstrong/Patterson (#128)
- *(#124)* Endofunctor trait — first-class C → C functor (#127)
- *(#122)* Dependability ontology — Avizienis-Laprie-Randell-Landwehr (2004) (#125)
- typed Vocabulary — OntologyName, ModulePath, structured Citation ([#111](https://github.com/i-am-logger/pr4xis/pull/111))
- compose API — runtime ontology composition via Korporator ([#103](https://github.com/i-am-logger/pr4xis/pull/103)) ([#108](https://github.com/i-am-logger/pr4xis/pull/108))
- Ontolex-Lemon, consciousness C1×C2, complete functor chain, Vocabulary API ([#88](https://github.com/i-am-logger/pr4xis/pull/88)) ([#104](https://github.com/i-am-logger/pr4xis/pull/104))
- define_ontology! being: clause + register all 108 ontologies ([#76](https://github.com/i-am-logger/pr4xis/pull/76)) ([#84](https://github.com/i-am-logger/pr4xis/pull/84))
- artifact_identity + data_provisioning — ontological external-data subsystem, no more LFS ([#71](https://github.com/i-am-logger/pr4xis/pull/71))
- staging ontology — Futamura's partial-evaluation framework as a meta-ontology ([#67](https://github.com/i-am-logger/pr4xis/pull/67))
- enforce ontology patterns — define_ontology! everywhere, 4851 tests
- integrate Kleisli + anamorphism + Yoneda into causation reasoning
- integrate algebraic structures into reasoning + tracing
- F-algebra, MonoidalCategory, Optics, MonadTransformer (4 structures)
- complete algebraic structure library — 7 new structures
- Reader + State monads with property-based tests
- migrate remaining 5 biomedical Ontology impls to structural + domain split
- migrate Ontology impls to structural + domain axiom split
- Ontology trait — structural + domain axioms merged via monoid
- define_ontology! clean API — concepts/is_a/has_a/causes/opposes + auto structural axioms
- migrate 41 ontologies to define_ontology! macro (-3163 lines)
- define_ontology! macro — generates Category + Taxonomy + Mereology + Causation + Opposition + OntologyMeta
- Monoid + Writer monad + TracedCategory refactor (Moggi 1991, Mac Lane 1971)
- restructure to academic hierarchy (DOLCE-aligned)
- migrate ~65 Pattern B ontologies to define_dense_category! (-5325 lines)
- migrate 30 Pattern A ontologies to define_category! macro (-4404 lines)
- derive macros — #[derive(Entity)] + define_category! + define_dense_category!
- dev-web serves chatbot at /, presentation at /decks/technical
- rename praxis → pr4xis across entire codebase
- migrate burp + pssst ontologies — 123 categories, 61 functors, 4707 tests
- migrate 11 vogix ontologies into praxis theming (3117 tests)
- prop tests + functor connections across 15 ontologies (2934 tests, 18 functors)
- PregroupCategory — proper Category with proven laws
- NoisyChannel→Communication + DRT→Dialogue functors (proven)
- Diagnostics→Control functor (FDI IS control — Gertler 1998, proven)
- Communication→Control + Diagnostics→Metacognition functors (proven)
- TraceSchema ontology — T(C) = El(C) + O_obs (Spivak 2012)
- trace shows Lambek notation (S[q]/NP) not Rust debug format
- proper ontology trace — each step reports what it did with status
- proper ontology trace — each step reports what it did with status
- trace functors — map pipeline steps to Diagnostics/PROV ontologies
- Diagnostics ontology + TracedCategory (writer monad on categories)
- proper ontology trace — each step reports what it did with status
- add Ontology Alignment and NLG pipeline ontologies
- rich taxonomy responses with path, definitions, and subtypes
- add criterion benchmarks for all ontologies and chat pipeline
- add Instance ontology (Spivak) and SystemsToSchema functor
- add durability, volatility, measurement, and benchmark ontologies
- merge sensor-fusion ontologies, add schema/storage/consistency ontologies, praxis-web
- speech production ontology (Levelt pipeline as category)
- Traum grounding state machine as finite category
- extend dialogue ontology with QUD, CommonGround, Intention, Repair
- function words as LMF data, extend LmfPos with closed-class types
- extended sentence test suite, chart type selection fix
- self-model ontology, CYK chart parser, adjunction, response generation
- integration tests with full WordNet — expose real failures honestly
- docs/chat/ for GitHub Pages — presentation embeds live chatbot
- complete scheme taxonomy — Vogix16, Ansi16, Base24, SchemeType
- Turing test benchmark — 18 questions, 3 pass, 15 need ontologies
- ColorSlot::key() — canonical theme file key names
- Rgb::from_hex and to_hex for color parsing
- Language::pregroup_types — end-to-end pregroup pipeline through Language trait
- Lambek → Pregroup functor — ontology evolution proven
- load WordNet verb frames for transitivity — no more defaults
- pregroup grammar ontology — parsing as group algebra
- integrate ontologies via functors, wire into chatbot pipeline
- control systems ontology + foundational cybernetics papers
- math functions + sRGB color science + theming ontology
- cognition ontologies — distinction, epistemics, metacognition
- question grammar types + Q semantic domain in Lambek/Montague
- Montague functor — type-driven syntax-semantics interpretation
- Lambek grammar — syntax as category, text understood through type reduction
- dialogue ontology + chatbot CLI — praxis can chat
- SystemsToEvents functor — closes System ↔ EventDriven ↔ Concurrent triangle
- event-driven ontology — chess IS event-driven IS concurrent (proven)
- SystemsToConcurrency functor — every system IS concurrent (proven)
- concurrency ontology — chess IS concurrent (proven via functor)
- Language trait, orthography, morphology, cached reasoning queries
- English language ontology — 107k concepts, nanosecond queries
- information ontology — what bits, bytes, references, and text ARE
- WordNet-LMF ontology — full 107k synset load in 3.8s
- XML ontology, enhanced property tests, systems thinking completeness
- DOLCE upper ontology, domain restructure, linguistics, systems thinking, codegen

### Fix

- *(cli)* reason over loaded statutes; load English owned, not leaked
- *(reasoning)* Parthood is irreflexive; a grounded edge is not a morphism
- *(derive)* has_a: emits part→whole Parthood edges (BFO:0000050 alignment)
- *(chat)* a Parthood answer phrases "is part of", not "is a"
- *(morphology)* satisfy clippy --all-targets --release in the AGID regenerate helper
- *(linguistics)* address Copilot review — cache operator vocab + don't split glyphs inside words
- *(docs,grounding)* broken intra-doc links + Copilot review
- *(#62)* address 11 copilot comments on consolidated Heim PR #143 (#144)
- clippy clean — no dead code, no unused imports, no stubs
- remove unused Category imports from test modules (clippy -D warnings)
- qualify kind refs in define_category! macro (avoid Identity ambiguity) + LOC badge
- update release-please config for pr4xis rename + add version to path deps
- add WordNet XML (LFS) + tinted-schemes submodule for CI
- skip data-dependent tests when WordNet/themes not available (CI)
- clarify Base16 has 16 slots, Base24 has 24
- remove hardcoded quit/exit — farewell detection through language lexicon
- remove all hardcoded pronoun/noun matching from dialogue engine
- taxonomy query works — 'is a dog a mammal' answered correctly
- copula type from CCG research — question 'is X a Y' now parses to Q type
- resolve all clippy warnings for strict CI

### Perf

- *(morphology)* cache the parsed irregulars table behind OnceLock in std (Copilot review)
- praxis tests give faster feedback and catch slowdowns earlier ([#185](https://github.com/i-am-logger/pr4xis/pull/185))

### Refactor

- *(self-aware)* address branch-review findings — doc-rot + one Form-aware counter
- *(decoders)* exhaustive has_decoder_for + per-module DECODES const (D-22)
- *(lmf)* verb transitivity from the loaded frame text, not the id prefix (D-15)
- *(registry)* source disk paths as praxis.toml data, not Rust dispatch (D-9)
- *(linguistics)* delete the dead legacy discourse module (D-11)
- *(lexicon)* collapse pos_to_olia_fragments to the canonical anchor (D-13)
- *(xsd)* derive datatype base_type + groups from the loaded hierarchy (D-16/D-6/D-5)
- *(relations)* relation→structural-property as loaded edges, not a Rust match (D-7)
- *(meta)* derive identity leaves, domain rank, and the XSD baseline from loaded data (D-18/D-19/D-20/D-17)
- *(domains)* derive is_leaf from the loaded graph + type the theme polarity (D-18, D-10)
- *(domains)* lower determiner/interjection features through ONE codec, not scattered synset.contains
- *(linguistics)* migrate ALL lexical-category assignment into the loaded functor (Batch K)
- migrate pr4xis-domains to feat/logic-ontologies API ([#170](https://github.com/i-am-logger/pr4xis/pull/170))
- *(#113)* batch 3 — final cognitive ontologies (lemon, consciousness, self_model) (#120)
- *(#113)* batch 2 — dialogue/pragmatics cluster (7 ontologies) (#119)
- *(#113)* migrate 18 ontologies to ontology! proc macro (#116)
- applied/theming/ → applied/hmi/{theming,surfaces,visualization,input,report,explorer}/ ([#66](https://github.com/i-am-logger/pr4xis/pull/66))
- move meta/methodological ontologies from biomedical to formal
- response generation through SVO grammar, not hardcoded strings
- load function words from LMF data file instead of hardcoded Rust
- rename Lambek types::english to types::svo (language-agnostic)
- replace hardcoded chat strings with response realization ontology
- merge English + EnglishLanguage into one type
- delete function_words.rs and vocabulary.rs — all lookups through Language
- Language trait as single lexical interface — tokenizer is language-agnostic
- remove hardcoded Montague, keep Lambek grammar clean
- revert hardcoded CLI, add dialogue ontology, add missing tests
- merge science into domains, reorganize by ontology, harden engine
- consolidate 18 crates → 4 workspace members

### Research

- *(#98)* kinded-functor failures diagnosed — three distinct problems, none lax (#129)

### Style

- rustfmt

### Test

- *(chat,domains)* "is X part of Y" answers over a loaded Parthood mereology (Step 3)
- property-based tests for math functions + sRGB color science
- comprehensive prop-based tests for cognition ontologies

## [0.25.3](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.25.2...pr4xis-domains-v0.25.3) - 2026-06-17

### Build

- *(release)* version the domains crate via release-plz
- *(release)* single workspace version via inheritance — fix the release-plz drift

### Docs

- *(domains)* green the workspace rustdoc over the OWL/USC bridges
- *(domains)* fix two rustdoc intra-doc links (CI Docs step)
- complete validation overhaul — per-def trait sweep + rustdoc rot + mdBook ([#176](https://github.com/i-am-logger/pr4xis/pull/176))
- *(#173)* per-ontology rollout for the 5 new HMI sub-ontologies (#69)
- rewrite + per-ontology rollout (#57, #55, #52, #46, #44) ([#63](https://github.com/i-am-logger/pr4xis/pull/63))
- color science source papers
- pregroup grammar research — parsing as group algebra

### Feat

- *(web)* the chat shows every ontology it reasoned over, loaded ones included (U6, U7)
- *(web)* the self-model reports its own memory footprint (U2)
- *(runtime)* a snapshot's address is portable, not toolchain-bound (A7)
- *(runtime)* snapshot any ontology, not just the self-model graph (A6)
- *(runtime)* one generic loader for every envelope (A5)
- *(runtime)* a functor or adjunction in a snapshot re-binds, not refused (A4)
- *(runtime)* load the relation-kind vocabulary from the Relations ontology (A3, slice b)
- *(theming)* vogix16 semantic keys use snake_case to match the design system
- *(self-aware)* the catalog includes loaded-but-unregistered ontologies (§3)
- *(runtime,chat)* relation-parametric image/meet/chain — Parthood evidence chains
- *(chat)* single-word loaded entities type NP — "is X part of Y" parses with a one-word X
- *(self-aware)* content-addressed load history + state fingerprint (Step 6 / §2.4)
- *(self-aware)* per-ontology capabilities — what each loaded ontology can answer (§4.7)
- *(self-aware)* provenance names the loaded ontology a turn reasoned over (Step 5 / §2.3)
- *(self-aware)* the SelfModel eigenform observes the live loaded set (Step 4 / §2)
- *(lambek,chat)* "is X part of Y" parses from raw text (Step 5 — the parse)
- *(domains)* trim the relation lexicon to non-default complement-headed surfaces
- *(chat)* lower a question's predicate to a typed relation kind (Step 3)
- *(domains)* relation_lexicon.prx — the loaded "part of"↦Parthood surface map (Step 3)
- *(domains)* relation-parametric LexicalReasoner::reaches (Step 3 spine)
- *(domains)* the USC→praxis functor lives in usc_functor.prx, not Rust ([#203](https://github.com/i-am-logger/pr4xis/pull/203))
- *(domains)* the OWL→praxis functor lives in owl_functor.prx, not Rust ([#203](https://github.com/i-am-logger/pr4xis/pull/203))
- *(domains)* the WordNet→praxis functor lives in english_functor.prx, not Rust ([#203](https://github.com/i-am-logger/pr4xis/pull/203))
- *(uslm)* mint USC heading + citation Forms — sections answerable by "section N" (§9c)
- *(owl,wasm)* mint OWL label Forms — loaded OWL answerable by its label (§9b)
- *(composed)* index a loaded concept's Form-atom surfaces (§9 lexicalization)
- *(chat)* multi-token surface recognition — phrase/citation lookup
- *(uslm)* lift USC→praxis to functor-as-data (off the baked Parthood/Section)
- *(owl)* OWL→praxis as a functor-as-data projection (not a baked converter)
- *(derive,runtime)* transitive kinds are loaded from data — Phase A Step 4
- *(runtime)* relation kind is a ConceptRef, not a closed enum — Phase A Step 3
- *(runtime)* transitive_kinds — relation transitivity as loaded data, not a constant
- *(morphology)* ground the productive rules in registered CatVar; structural rule count (D-2/D-12)
- *(morphology)* irregular forms LOADED from the registered AGID source (MORPH/D-1)
- *(linguistics)* [**breaking**] literature-honest determiner/interjection features (FW-B)
- *(domains)* function-words load from a committed .prx — the ClosedClassLexicon (FW-A)
- *(linguistics)* wh-questions via a loaded OLiA→CCG category functor ([#169](https://github.com/i-am-logger/pr4xis/pull/169))
- *(linguistics)* math operators as a loaded vocabulary — the #169 tokenizer fix
- generic grounding — ground(lens) over the substrate, denotes is one lens
- *(domains)* USC → Archive — statute provisions in the generic substrate (the ontological way)
- *(domains)* persist the denotes pointer column in the USC envelope (G3b-2b code)
- *(domains)* the denotes-floor producer — statute prose → ontolex:Form pointers (G3b-2a)
- *(domains)* the honest denotes floor — a span grounds into an ontolex:Form (G3b-1)
- *(runtime)* EdgeTarget — the foreign-atom slot, byte-exact (grounding G2)
- *(domains)* bridge loaded English to a RuntimeOntology — the #87 grounding gate
- *(domains)* the WordNet→praxis functor, carried as .prx data
- *(domains)* project English into a runtime Archive — the WordNet→.prx source
- `.prx` — praxis' knowledge, in a file ([#186](https://github.com/i-am-logger/pr4xis/pull/186))
- [**breaking**] sync all praxis crates to one version (0.22.0) ([#190](https://github.com/i-am-logger/pr4xis/pull/190))
- [**breaking**] restore praxis publishing to crates.io ([#188](https://github.com/i-am-logger/pr4xis/pull/188))
- *(cli)* one-command corpus updates via `pr4xis update --lock` ([#183](https://github.com/i-am-logger/pr4xis/pull/183))
- praxis + praxis-cli gain the registered-source mechanism (SOX 1514A, AIR21 42121) ([#179](https://github.com/i-am-logger/pr4xis/pull/179))
- *(deps,ci)* [**breaking**] pure-Rust crypto + always-latest devenv + PR-title gate ([#177](https://github.com/i-am-logger/pr4xis/pull/177))
- *(#91)* pr4xis core + domains run no_std + alloc (#157)
- *(#148)* Lemon meta on every structural entity — uniform registry for ontologies, axioms, functors, adjunctions, nat-trans (#150)
- *(#117)* MAPE-K ontology — chat pipeline's literature-grounded home (#146)
- *(#62)* Heim syntrometric lineage — consolidated stack (#143)
- *(#62)* Heim syntrometry Phase 1 — lineage verified by functor laws (#135)
- *(#131)* TerminalFunctor helper — reusable one-object collapse (#134)
- *(#130)* Category::Op<C> + empirical 4th failure mode discovery (#133)
- *(#123)* Resilience ontology — Nygard/Brooker/Armstrong/Patterson (#128)
- *(#124)* Endofunctor trait — first-class C → C functor (#127)
- *(#122)* Dependability ontology — Avizienis-Laprie-Randell-Landwehr (2004) (#125)
- typed Vocabulary — OntologyName, ModulePath, structured Citation ([#111](https://github.com/i-am-logger/pr4xis/pull/111))
- compose API — runtime ontology composition via Korporator ([#103](https://github.com/i-am-logger/pr4xis/pull/103)) ([#108](https://github.com/i-am-logger/pr4xis/pull/108))
- Ontolex-Lemon, consciousness C1×C2, complete functor chain, Vocabulary API ([#88](https://github.com/i-am-logger/pr4xis/pull/88)) ([#104](https://github.com/i-am-logger/pr4xis/pull/104))
- define_ontology! being: clause + register all 108 ontologies ([#76](https://github.com/i-am-logger/pr4xis/pull/76)) ([#84](https://github.com/i-am-logger/pr4xis/pull/84))
- artifact_identity + data_provisioning — ontological external-data subsystem, no more LFS ([#71](https://github.com/i-am-logger/pr4xis/pull/71))
- staging ontology — Futamura's partial-evaluation framework as a meta-ontology ([#67](https://github.com/i-am-logger/pr4xis/pull/67))
- enforce ontology patterns — define_ontology! everywhere, 4851 tests
- integrate Kleisli + anamorphism + Yoneda into causation reasoning
- integrate algebraic structures into reasoning + tracing
- F-algebra, MonoidalCategory, Optics, MonadTransformer (4 structures)
- complete algebraic structure library — 7 new structures
- Reader + State monads with property-based tests
- migrate remaining 5 biomedical Ontology impls to structural + domain split
- migrate Ontology impls to structural + domain axiom split
- Ontology trait — structural + domain axioms merged via monoid
- define_ontology! clean API — concepts/is_a/has_a/causes/opposes + auto structural axioms
- migrate 41 ontologies to define_ontology! macro (-3163 lines)
- define_ontology! macro — generates Category + Taxonomy + Mereology + Causation + Opposition + OntologyMeta
- Monoid + Writer monad + TracedCategory refactor (Moggi 1991, Mac Lane 1971)
- restructure to academic hierarchy (DOLCE-aligned)
- migrate ~65 Pattern B ontologies to define_dense_category! (-5325 lines)
- migrate 30 Pattern A ontologies to define_category! macro (-4404 lines)
- derive macros — #[derive(Entity)] + define_category! + define_dense_category!
- dev-web serves chatbot at /, presentation at /decks/technical
- rename praxis → pr4xis across entire codebase
- migrate burp + pssst ontologies — 123 categories, 61 functors, 4707 tests
- migrate 11 vogix ontologies into praxis theming (3117 tests)
- prop tests + functor connections across 15 ontologies (2934 tests, 18 functors)
- PregroupCategory — proper Category with proven laws
- NoisyChannel→Communication + DRT→Dialogue functors (proven)
- Diagnostics→Control functor (FDI IS control — Gertler 1998, proven)
- Communication→Control + Diagnostics→Metacognition functors (proven)
- TraceSchema ontology — T(C) = El(C) + O_obs (Spivak 2012)
- trace shows Lambek notation (S[q]/NP) not Rust debug format
- proper ontology trace — each step reports what it did with status
- proper ontology trace — each step reports what it did with status
- trace functors — map pipeline steps to Diagnostics/PROV ontologies
- Diagnostics ontology + TracedCategory (writer monad on categories)
- proper ontology trace — each step reports what it did with status
- add Ontology Alignment and NLG pipeline ontologies
- rich taxonomy responses with path, definitions, and subtypes
- add criterion benchmarks for all ontologies and chat pipeline
- add Instance ontology (Spivak) and SystemsToSchema functor
- add durability, volatility, measurement, and benchmark ontologies
- merge sensor-fusion ontologies, add schema/storage/consistency ontologies, praxis-web
- speech production ontology (Levelt pipeline as category)
- Traum grounding state machine as finite category
- extend dialogue ontology with QUD, CommonGround, Intention, Repair
- function words as LMF data, extend LmfPos with closed-class types
- extended sentence test suite, chart type selection fix
- self-model ontology, CYK chart parser, adjunction, response generation
- integration tests with full WordNet — expose real failures honestly
- docs/chat/ for GitHub Pages — presentation embeds live chatbot
- complete scheme taxonomy — Vogix16, Ansi16, Base24, SchemeType
- Turing test benchmark — 18 questions, 3 pass, 15 need ontologies
- ColorSlot::key() — canonical theme file key names
- Rgb::from_hex and to_hex for color parsing
- Language::pregroup_types — end-to-end pregroup pipeline through Language trait
- Lambek → Pregroup functor — ontology evolution proven
- load WordNet verb frames for transitivity — no more defaults
- pregroup grammar ontology — parsing as group algebra
- integrate ontologies via functors, wire into chatbot pipeline
- control systems ontology + foundational cybernetics papers
- math functions + sRGB color science + theming ontology
- cognition ontologies — distinction, epistemics, metacognition
- question grammar types + Q semantic domain in Lambek/Montague
- Montague functor — type-driven syntax-semantics interpretation
- Lambek grammar — syntax as category, text understood through type reduction
- dialogue ontology + chatbot CLI — praxis can chat
- SystemsToEvents functor — closes System ↔ EventDriven ↔ Concurrent triangle
- event-driven ontology — chess IS event-driven IS concurrent (proven)
- SystemsToConcurrency functor — every system IS concurrent (proven)
- concurrency ontology — chess IS concurrent (proven via functor)
- Language trait, orthography, morphology, cached reasoning queries
- English language ontology — 107k concepts, nanosecond queries
- information ontology — what bits, bytes, references, and text ARE
- WordNet-LMF ontology — full 107k synset load in 3.8s
- XML ontology, enhanced property tests, systems thinking completeness
- DOLCE upper ontology, domain restructure, linguistics, systems thinking, codegen

### Fix

- *(cli)* reason over loaded statutes; load English owned, not leaked
- *(reasoning)* Parthood is irreflexive; a grounded edge is not a morphism
- *(derive)* has_a: emits part→whole Parthood edges (BFO:0000050 alignment)
- *(chat)* a Parthood answer phrases "is part of", not "is a"
- *(morphology)* satisfy clippy --all-targets --release in the AGID regenerate helper
- *(linguistics)* address Copilot review — cache operator vocab + don't split glyphs inside words
- *(docs,grounding)* broken intra-doc links + Copilot review
- *(#62)* address 11 copilot comments on consolidated Heim PR #143 (#144)
- clippy clean — no dead code, no unused imports, no stubs
- remove unused Category imports from test modules (clippy -D warnings)
- qualify kind refs in define_category! macro (avoid Identity ambiguity) + LOC badge
- update release-please config for pr4xis rename + add version to path deps
- add WordNet XML (LFS) + tinted-schemes submodule for CI
- skip data-dependent tests when WordNet/themes not available (CI)
- clarify Base16 has 16 slots, Base24 has 24
- remove hardcoded quit/exit — farewell detection through language lexicon
- remove all hardcoded pronoun/noun matching from dialogue engine
- taxonomy query works — 'is a dog a mammal' answered correctly
- copula type from CCG research — question 'is X a Y' now parses to Q type
- resolve all clippy warnings for strict CI

### Perf

- *(morphology)* cache the parsed irregulars table behind OnceLock in std (Copilot review)
- praxis tests give faster feedback and catch slowdowns earlier ([#185](https://github.com/i-am-logger/pr4xis/pull/185))

### Refactor

- *(self-aware)* address branch-review findings — doc-rot + one Form-aware counter
- *(decoders)* exhaustive has_decoder_for + per-module DECODES const (D-22)
- *(lmf)* verb transitivity from the loaded frame text, not the id prefix (D-15)
- *(registry)* source disk paths as praxis.toml data, not Rust dispatch (D-9)
- *(linguistics)* delete the dead legacy discourse module (D-11)
- *(lexicon)* collapse pos_to_olia_fragments to the canonical anchor (D-13)
- *(xsd)* derive datatype base_type + groups from the loaded hierarchy (D-16/D-6/D-5)
- *(relations)* relation→structural-property as loaded edges, not a Rust match (D-7)
- *(meta)* derive identity leaves, domain rank, and the XSD baseline from loaded data (D-18/D-19/D-20/D-17)
- *(domains)* derive is_leaf from the loaded graph + type the theme polarity (D-18, D-10)
- *(domains)* lower determiner/interjection features through ONE codec, not scattered synset.contains
- *(linguistics)* migrate ALL lexical-category assignment into the loaded functor (Batch K)
- migrate pr4xis-domains to feat/logic-ontologies API ([#170](https://github.com/i-am-logger/pr4xis/pull/170))
- *(#113)* batch 3 — final cognitive ontologies (lemon, consciousness, self_model) (#120)
- *(#113)* batch 2 — dialogue/pragmatics cluster (7 ontologies) (#119)
- *(#113)* migrate 18 ontologies to ontology! proc macro (#116)
- applied/theming/ → applied/hmi/{theming,surfaces,visualization,input,report,explorer}/ ([#66](https://github.com/i-am-logger/pr4xis/pull/66))
- move meta/methodological ontologies from biomedical to formal
- response generation through SVO grammar, not hardcoded strings
- load function words from LMF data file instead of hardcoded Rust
- rename Lambek types::english to types::svo (language-agnostic)
- replace hardcoded chat strings with response realization ontology
- merge English + EnglishLanguage into one type
- delete function_words.rs and vocabulary.rs — all lookups through Language
- Language trait as single lexical interface — tokenizer is language-agnostic
- remove hardcoded Montague, keep Lambek grammar clean
- revert hardcoded CLI, add dialogue ontology, add missing tests
- merge science into domains, reorganize by ontology, harden engine
- consolidate 18 crates → 4 workspace members

### Research

- *(#98)* kinded-functor failures diagnosed — three distinct problems, none lax (#129)

### Style

- rustfmt

### Test

- *(chat,domains)* "is X part of Y" answers over a loaded Parthood mereology (Step 3)
- property-based tests for math functions + sRGB color science
- comprehensive prop-based tests for cognition ontologies

## [0.25.2](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.25.1...pr4xis-domains-v0.25.2) - 2026-06-17

### Build

- *(release)* version the domains crate via release-plz
- *(release)* single workspace version via inheritance — fix the release-plz drift

### Docs

- *(domains)* green the workspace rustdoc over the OWL/USC bridges
- *(domains)* fix two rustdoc intra-doc links (CI Docs step)
- complete validation overhaul — per-def trait sweep + rustdoc rot + mdBook ([#176](https://github.com/i-am-logger/pr4xis/pull/176))
- *(#173)* per-ontology rollout for the 5 new HMI sub-ontologies (#69)
- rewrite + per-ontology rollout (#57, #55, #52, #46, #44) ([#63](https://github.com/i-am-logger/pr4xis/pull/63))
- color science source papers
- pregroup grammar research — parsing as group algebra

### Feat

- *(web)* the chat shows every ontology it reasoned over, loaded ones included (U6, U7)
- *(web)* the self-model reports its own memory footprint (U2)
- *(runtime)* a snapshot's address is portable, not toolchain-bound (A7)
- *(runtime)* snapshot any ontology, not just the self-model graph (A6)
- *(runtime)* one generic loader for every envelope (A5)
- *(runtime)* a functor or adjunction in a snapshot re-binds, not refused (A4)
- *(runtime)* load the relation-kind vocabulary from the Relations ontology (A3, slice b)
- *(theming)* vogix16 semantic keys use snake_case to match the design system
- *(self-aware)* the catalog includes loaded-but-unregistered ontologies (§3)
- *(runtime,chat)* relation-parametric image/meet/chain — Parthood evidence chains
- *(chat)* single-word loaded entities type NP — "is X part of Y" parses with a one-word X
- *(self-aware)* content-addressed load history + state fingerprint (Step 6 / §2.4)
- *(self-aware)* per-ontology capabilities — what each loaded ontology can answer (§4.7)
- *(self-aware)* provenance names the loaded ontology a turn reasoned over (Step 5 / §2.3)
- *(self-aware)* the SelfModel eigenform observes the live loaded set (Step 4 / §2)
- *(lambek,chat)* "is X part of Y" parses from raw text (Step 5 — the parse)
- *(domains)* trim the relation lexicon to non-default complement-headed surfaces
- *(chat)* lower a question's predicate to a typed relation kind (Step 3)
- *(domains)* relation_lexicon.prx — the loaded "part of"↦Parthood surface map (Step 3)
- *(domains)* relation-parametric LexicalReasoner::reaches (Step 3 spine)
- *(domains)* the USC→praxis functor lives in usc_functor.prx, not Rust ([#203](https://github.com/i-am-logger/pr4xis/pull/203))
- *(domains)* the OWL→praxis functor lives in owl_functor.prx, not Rust ([#203](https://github.com/i-am-logger/pr4xis/pull/203))
- *(domains)* the WordNet→praxis functor lives in english_functor.prx, not Rust ([#203](https://github.com/i-am-logger/pr4xis/pull/203))
- *(uslm)* mint USC heading + citation Forms — sections answerable by "section N" (§9c)
- *(owl,wasm)* mint OWL label Forms — loaded OWL answerable by its label (§9b)
- *(composed)* index a loaded concept's Form-atom surfaces (§9 lexicalization)
- *(chat)* multi-token surface recognition — phrase/citation lookup
- *(uslm)* lift USC→praxis to functor-as-data (off the baked Parthood/Section)
- *(owl)* OWL→praxis as a functor-as-data projection (not a baked converter)
- *(derive,runtime)* transitive kinds are loaded from data — Phase A Step 4
- *(runtime)* relation kind is a ConceptRef, not a closed enum — Phase A Step 3
- *(runtime)* transitive_kinds — relation transitivity as loaded data, not a constant
- *(morphology)* ground the productive rules in registered CatVar; structural rule count (D-2/D-12)
- *(morphology)* irregular forms LOADED from the registered AGID source (MORPH/D-1)
- *(linguistics)* [**breaking**] literature-honest determiner/interjection features (FW-B)
- *(domains)* function-words load from a committed .prx — the ClosedClassLexicon (FW-A)
- *(linguistics)* wh-questions via a loaded OLiA→CCG category functor ([#169](https://github.com/i-am-logger/pr4xis/pull/169))
- *(linguistics)* math operators as a loaded vocabulary — the #169 tokenizer fix
- generic grounding — ground(lens) over the substrate, denotes is one lens
- *(domains)* USC → Archive — statute provisions in the generic substrate (the ontological way)
- *(domains)* persist the denotes pointer column in the USC envelope (G3b-2b code)
- *(domains)* the denotes-floor producer — statute prose → ontolex:Form pointers (G3b-2a)
- *(domains)* the honest denotes floor — a span grounds into an ontolex:Form (G3b-1)
- *(runtime)* EdgeTarget — the foreign-atom slot, byte-exact (grounding G2)
- *(domains)* bridge loaded English to a RuntimeOntology — the #87 grounding gate
- *(domains)* the WordNet→praxis functor, carried as .prx data
- *(domains)* project English into a runtime Archive — the WordNet→.prx source
- `.prx` — praxis' knowledge, in a file ([#186](https://github.com/i-am-logger/pr4xis/pull/186))
- [**breaking**] sync all praxis crates to one version (0.22.0) ([#190](https://github.com/i-am-logger/pr4xis/pull/190))
- [**breaking**] restore praxis publishing to crates.io ([#188](https://github.com/i-am-logger/pr4xis/pull/188))
- *(cli)* one-command corpus updates via `pr4xis update --lock` ([#183](https://github.com/i-am-logger/pr4xis/pull/183))
- praxis + praxis-cli gain the registered-source mechanism (SOX 1514A, AIR21 42121) ([#179](https://github.com/i-am-logger/pr4xis/pull/179))
- *(deps,ci)* [**breaking**] pure-Rust crypto + always-latest devenv + PR-title gate ([#177](https://github.com/i-am-logger/pr4xis/pull/177))
- *(#91)* pr4xis core + domains run no_std + alloc (#157)
- *(#148)* Lemon meta on every structural entity — uniform registry for ontologies, axioms, functors, adjunctions, nat-trans (#150)
- *(#117)* MAPE-K ontology — chat pipeline's literature-grounded home (#146)
- *(#62)* Heim syntrometric lineage — consolidated stack (#143)
- *(#62)* Heim syntrometry Phase 1 — lineage verified by functor laws (#135)
- *(#131)* TerminalFunctor helper — reusable one-object collapse (#134)
- *(#130)* Category::Op<C> + empirical 4th failure mode discovery (#133)
- *(#123)* Resilience ontology — Nygard/Brooker/Armstrong/Patterson (#128)
- *(#124)* Endofunctor trait — first-class C → C functor (#127)
- *(#122)* Dependability ontology — Avizienis-Laprie-Randell-Landwehr (2004) (#125)
- typed Vocabulary — OntologyName, ModulePath, structured Citation ([#111](https://github.com/i-am-logger/pr4xis/pull/111))
- compose API — runtime ontology composition via Korporator ([#103](https://github.com/i-am-logger/pr4xis/pull/103)) ([#108](https://github.com/i-am-logger/pr4xis/pull/108))
- Ontolex-Lemon, consciousness C1×C2, complete functor chain, Vocabulary API ([#88](https://github.com/i-am-logger/pr4xis/pull/88)) ([#104](https://github.com/i-am-logger/pr4xis/pull/104))
- define_ontology! being: clause + register all 108 ontologies ([#76](https://github.com/i-am-logger/pr4xis/pull/76)) ([#84](https://github.com/i-am-logger/pr4xis/pull/84))
- artifact_identity + data_provisioning — ontological external-data subsystem, no more LFS ([#71](https://github.com/i-am-logger/pr4xis/pull/71))
- staging ontology — Futamura's partial-evaluation framework as a meta-ontology ([#67](https://github.com/i-am-logger/pr4xis/pull/67))
- enforce ontology patterns — define_ontology! everywhere, 4851 tests
- integrate Kleisli + anamorphism + Yoneda into causation reasoning
- integrate algebraic structures into reasoning + tracing
- F-algebra, MonoidalCategory, Optics, MonadTransformer (4 structures)
- complete algebraic structure library — 7 new structures
- Reader + State monads with property-based tests
- migrate remaining 5 biomedical Ontology impls to structural + domain split
- migrate Ontology impls to structural + domain axiom split
- Ontology trait — structural + domain axioms merged via monoid
- define_ontology! clean API — concepts/is_a/has_a/causes/opposes + auto structural axioms
- migrate 41 ontologies to define_ontology! macro (-3163 lines)
- define_ontology! macro — generates Category + Taxonomy + Mereology + Causation + Opposition + OntologyMeta
- Monoid + Writer monad + TracedCategory refactor (Moggi 1991, Mac Lane 1971)
- restructure to academic hierarchy (DOLCE-aligned)
- migrate ~65 Pattern B ontologies to define_dense_category! (-5325 lines)
- migrate 30 Pattern A ontologies to define_category! macro (-4404 lines)
- derive macros — #[derive(Entity)] + define_category! + define_dense_category!
- dev-web serves chatbot at /, presentation at /decks/technical
- rename praxis → pr4xis across entire codebase
- migrate burp + pssst ontologies — 123 categories, 61 functors, 4707 tests
- migrate 11 vogix ontologies into praxis theming (3117 tests)
- prop tests + functor connections across 15 ontologies (2934 tests, 18 functors)
- PregroupCategory — proper Category with proven laws
- NoisyChannel→Communication + DRT→Dialogue functors (proven)
- Diagnostics→Control functor (FDI IS control — Gertler 1998, proven)
- Communication→Control + Diagnostics→Metacognition functors (proven)
- TraceSchema ontology — T(C) = El(C) + O_obs (Spivak 2012)
- trace shows Lambek notation (S[q]/NP) not Rust debug format
- proper ontology trace — each step reports what it did with status
- proper ontology trace — each step reports what it did with status
- trace functors — map pipeline steps to Diagnostics/PROV ontologies
- Diagnostics ontology + TracedCategory (writer monad on categories)
- proper ontology trace — each step reports what it did with status
- add Ontology Alignment and NLG pipeline ontologies
- rich taxonomy responses with path, definitions, and subtypes
- add criterion benchmarks for all ontologies and chat pipeline
- add Instance ontology (Spivak) and SystemsToSchema functor
- add durability, volatility, measurement, and benchmark ontologies
- merge sensor-fusion ontologies, add schema/storage/consistency ontologies, praxis-web
- speech production ontology (Levelt pipeline as category)
- Traum grounding state machine as finite category
- extend dialogue ontology with QUD, CommonGround, Intention, Repair
- function words as LMF data, extend LmfPos with closed-class types
- extended sentence test suite, chart type selection fix
- self-model ontology, CYK chart parser, adjunction, response generation
- integration tests with full WordNet — expose real failures honestly
- docs/chat/ for GitHub Pages — presentation embeds live chatbot
- complete scheme taxonomy — Vogix16, Ansi16, Base24, SchemeType
- Turing test benchmark — 18 questions, 3 pass, 15 need ontologies
- ColorSlot::key() — canonical theme file key names
- Rgb::from_hex and to_hex for color parsing
- Language::pregroup_types — end-to-end pregroup pipeline through Language trait
- Lambek → Pregroup functor — ontology evolution proven
- load WordNet verb frames for transitivity — no more defaults
- pregroup grammar ontology — parsing as group algebra
- integrate ontologies via functors, wire into chatbot pipeline
- control systems ontology + foundational cybernetics papers
- math functions + sRGB color science + theming ontology
- cognition ontologies — distinction, epistemics, metacognition
- question grammar types + Q semantic domain in Lambek/Montague
- Montague functor — type-driven syntax-semantics interpretation
- Lambek grammar — syntax as category, text understood through type reduction
- dialogue ontology + chatbot CLI — praxis can chat
- SystemsToEvents functor — closes System ↔ EventDriven ↔ Concurrent triangle
- event-driven ontology — chess IS event-driven IS concurrent (proven)
- SystemsToConcurrency functor — every system IS concurrent (proven)
- concurrency ontology — chess IS concurrent (proven via functor)
- Language trait, orthography, morphology, cached reasoning queries
- English language ontology — 107k concepts, nanosecond queries
- information ontology — what bits, bytes, references, and text ARE
- WordNet-LMF ontology — full 107k synset load in 3.8s
- XML ontology, enhanced property tests, systems thinking completeness
- DOLCE upper ontology, domain restructure, linguistics, systems thinking, codegen

### Fix

- *(cli)* reason over loaded statutes; load English owned, not leaked
- *(reasoning)* Parthood is irreflexive; a grounded edge is not a morphism
- *(derive)* has_a: emits part→whole Parthood edges (BFO:0000050 alignment)
- *(chat)* a Parthood answer phrases "is part of", not "is a"
- *(morphology)* satisfy clippy --all-targets --release in the AGID regenerate helper
- *(linguistics)* address Copilot review — cache operator vocab + don't split glyphs inside words
- *(docs,grounding)* broken intra-doc links + Copilot review
- *(#62)* address 11 copilot comments on consolidated Heim PR #143 (#144)
- clippy clean — no dead code, no unused imports, no stubs
- remove unused Category imports from test modules (clippy -D warnings)
- qualify kind refs in define_category! macro (avoid Identity ambiguity) + LOC badge
- update release-please config for pr4xis rename + add version to path deps
- add WordNet XML (LFS) + tinted-schemes submodule for CI
- skip data-dependent tests when WordNet/themes not available (CI)
- clarify Base16 has 16 slots, Base24 has 24
- remove hardcoded quit/exit — farewell detection through language lexicon
- remove all hardcoded pronoun/noun matching from dialogue engine
- taxonomy query works — 'is a dog a mammal' answered correctly
- copula type from CCG research — question 'is X a Y' now parses to Q type
- resolve all clippy warnings for strict CI

### Perf

- *(morphology)* cache the parsed irregulars table behind OnceLock in std (Copilot review)
- praxis tests give faster feedback and catch slowdowns earlier ([#185](https://github.com/i-am-logger/pr4xis/pull/185))

### Refactor

- *(self-aware)* address branch-review findings — doc-rot + one Form-aware counter
- *(decoders)* exhaustive has_decoder_for + per-module DECODES const (D-22)
- *(lmf)* verb transitivity from the loaded frame text, not the id prefix (D-15)
- *(registry)* source disk paths as praxis.toml data, not Rust dispatch (D-9)
- *(linguistics)* delete the dead legacy discourse module (D-11)
- *(lexicon)* collapse pos_to_olia_fragments to the canonical anchor (D-13)
- *(xsd)* derive datatype base_type + groups from the loaded hierarchy (D-16/D-6/D-5)
- *(relations)* relation→structural-property as loaded edges, not a Rust match (D-7)
- *(meta)* derive identity leaves, domain rank, and the XSD baseline from loaded data (D-18/D-19/D-20/D-17)
- *(domains)* derive is_leaf from the loaded graph + type the theme polarity (D-18, D-10)
- *(domains)* lower determiner/interjection features through ONE codec, not scattered synset.contains
- *(linguistics)* migrate ALL lexical-category assignment into the loaded functor (Batch K)
- migrate pr4xis-domains to feat/logic-ontologies API ([#170](https://github.com/i-am-logger/pr4xis/pull/170))
- *(#113)* batch 3 — final cognitive ontologies (lemon, consciousness, self_model) (#120)
- *(#113)* batch 2 — dialogue/pragmatics cluster (7 ontologies) (#119)
- *(#113)* migrate 18 ontologies to ontology! proc macro (#116)
- applied/theming/ → applied/hmi/{theming,surfaces,visualization,input,report,explorer}/ ([#66](https://github.com/i-am-logger/pr4xis/pull/66))
- move meta/methodological ontologies from biomedical to formal
- response generation through SVO grammar, not hardcoded strings
- load function words from LMF data file instead of hardcoded Rust
- rename Lambek types::english to types::svo (language-agnostic)
- replace hardcoded chat strings with response realization ontology
- merge English + EnglishLanguage into one type
- delete function_words.rs and vocabulary.rs — all lookups through Language
- Language trait as single lexical interface — tokenizer is language-agnostic
- remove hardcoded Montague, keep Lambek grammar clean
- revert hardcoded CLI, add dialogue ontology, add missing tests
- merge science into domains, reorganize by ontology, harden engine
- consolidate 18 crates → 4 workspace members

### Research

- *(#98)* kinded-functor failures diagnosed — three distinct problems, none lax (#129)

### Style

- rustfmt

### Test

- *(chat,domains)* "is X part of Y" answers over a loaded Parthood mereology (Step 3)
- property-based tests for math functions + sRGB color science
- comprehensive prop-based tests for cognition ontologies

## [0.25.1](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.25.0...pr4xis-domains-v0.25.1) - 2026-06-16

### Build

- *(release)* version the domains crate via release-plz
- *(release)* single workspace version via inheritance — fix the release-plz drift

### Docs

- *(domains)* green the workspace rustdoc over the OWL/USC bridges
- *(domains)* fix two rustdoc intra-doc links (CI Docs step)
- complete validation overhaul — per-def trait sweep + rustdoc rot + mdBook ([#176](https://github.com/i-am-logger/pr4xis/pull/176))
- *(#173)* per-ontology rollout for the 5 new HMI sub-ontologies (#69)
- rewrite + per-ontology rollout (#57, #55, #52, #46, #44) ([#63](https://github.com/i-am-logger/pr4xis/pull/63))
- color science source papers
- pregroup grammar research — parsing as group algebra

### Feat

- *(self-aware)* the catalog includes loaded-but-unregistered ontologies (§3)
- *(runtime,chat)* relation-parametric image/meet/chain — Parthood evidence chains
- *(chat)* single-word loaded entities type NP — "is X part of Y" parses with a one-word X
- *(self-aware)* content-addressed load history + state fingerprint (Step 6 / §2.4)
- *(self-aware)* per-ontology capabilities — what each loaded ontology can answer (§4.7)
- *(self-aware)* provenance names the loaded ontology a turn reasoned over (Step 5 / §2.3)
- *(self-aware)* the SelfModel eigenform observes the live loaded set (Step 4 / §2)
- *(lambek,chat)* "is X part of Y" parses from raw text (Step 5 — the parse)
- *(domains)* trim the relation lexicon to non-default complement-headed surfaces
- *(chat)* lower a question's predicate to a typed relation kind (Step 3)
- *(domains)* relation_lexicon.prx — the loaded "part of"↦Parthood surface map (Step 3)
- *(domains)* relation-parametric LexicalReasoner::reaches (Step 3 spine)
- *(domains)* the USC→praxis functor lives in usc_functor.prx, not Rust ([#203](https://github.com/i-am-logger/pr4xis/pull/203))
- *(domains)* the OWL→praxis functor lives in owl_functor.prx, not Rust ([#203](https://github.com/i-am-logger/pr4xis/pull/203))
- *(domains)* the WordNet→praxis functor lives in english_functor.prx, not Rust ([#203](https://github.com/i-am-logger/pr4xis/pull/203))
- *(uslm)* mint USC heading + citation Forms — sections answerable by "section N" (§9c)
- *(owl,wasm)* mint OWL label Forms — loaded OWL answerable by its label (§9b)
- *(composed)* index a loaded concept's Form-atom surfaces (§9 lexicalization)
- *(chat)* multi-token surface recognition — phrase/citation lookup
- *(uslm)* lift USC→praxis to functor-as-data (off the baked Parthood/Section)
- *(owl)* OWL→praxis as a functor-as-data projection (not a baked converter)
- *(derive,runtime)* transitive kinds are loaded from data — Phase A Step 4
- *(runtime)* relation kind is a ConceptRef, not a closed enum — Phase A Step 3
- *(runtime)* transitive_kinds — relation transitivity as loaded data, not a constant
- *(morphology)* ground the productive rules in registered CatVar; structural rule count (D-2/D-12)
- *(morphology)* irregular forms LOADED from the registered AGID source (MORPH/D-1)
- *(linguistics)* [**breaking**] literature-honest determiner/interjection features (FW-B)
- *(domains)* function-words load from a committed .prx — the ClosedClassLexicon (FW-A)
- *(linguistics)* wh-questions via a loaded OLiA→CCG category functor ([#169](https://github.com/i-am-logger/pr4xis/pull/169))
- *(linguistics)* math operators as a loaded vocabulary — the #169 tokenizer fix
- generic grounding — ground(lens) over the substrate, denotes is one lens
- *(domains)* USC → Archive — statute provisions in the generic substrate (the ontological way)
- *(domains)* persist the denotes pointer column in the USC envelope (G3b-2b code)
- *(domains)* the denotes-floor producer — statute prose → ontolex:Form pointers (G3b-2a)
- *(domains)* the honest denotes floor — a span grounds into an ontolex:Form (G3b-1)
- *(runtime)* EdgeTarget — the foreign-atom slot, byte-exact (grounding G2)
- *(domains)* bridge loaded English to a RuntimeOntology — the #87 grounding gate
- *(domains)* the WordNet→praxis functor, carried as .prx data
- *(domains)* project English into a runtime Archive — the WordNet→.prx source
- `.prx` — praxis' knowledge, in a file ([#186](https://github.com/i-am-logger/pr4xis/pull/186))
- [**breaking**] sync all praxis crates to one version (0.22.0) ([#190](https://github.com/i-am-logger/pr4xis/pull/190))
- [**breaking**] restore praxis publishing to crates.io ([#188](https://github.com/i-am-logger/pr4xis/pull/188))
- *(cli)* one-command corpus updates via `pr4xis update --lock` ([#183](https://github.com/i-am-logger/pr4xis/pull/183))
- praxis + praxis-cli gain the registered-source mechanism (SOX 1514A, AIR21 42121) ([#179](https://github.com/i-am-logger/pr4xis/pull/179))
- *(deps,ci)* [**breaking**] pure-Rust crypto + always-latest devenv + PR-title gate ([#177](https://github.com/i-am-logger/pr4xis/pull/177))
- *(#91)* pr4xis core + domains run no_std + alloc (#157)
- *(#148)* Lemon meta on every structural entity — uniform registry for ontologies, axioms, functors, adjunctions, nat-trans (#150)
- *(#117)* MAPE-K ontology — chat pipeline's literature-grounded home (#146)
- *(#62)* Heim syntrometric lineage — consolidated stack (#143)
- *(#62)* Heim syntrometry Phase 1 — lineage verified by functor laws (#135)
- *(#131)* TerminalFunctor helper — reusable one-object collapse (#134)
- *(#130)* Category::Op<C> + empirical 4th failure mode discovery (#133)
- *(#123)* Resilience ontology — Nygard/Brooker/Armstrong/Patterson (#128)
- *(#124)* Endofunctor trait — first-class C → C functor (#127)
- *(#122)* Dependability ontology — Avizienis-Laprie-Randell-Landwehr (2004) (#125)
- typed Vocabulary — OntologyName, ModulePath, structured Citation ([#111](https://github.com/i-am-logger/pr4xis/pull/111))
- compose API — runtime ontology composition via Korporator ([#103](https://github.com/i-am-logger/pr4xis/pull/103)) ([#108](https://github.com/i-am-logger/pr4xis/pull/108))
- Ontolex-Lemon, consciousness C1×C2, complete functor chain, Vocabulary API ([#88](https://github.com/i-am-logger/pr4xis/pull/88)) ([#104](https://github.com/i-am-logger/pr4xis/pull/104))
- define_ontology! being: clause + register all 108 ontologies ([#76](https://github.com/i-am-logger/pr4xis/pull/76)) ([#84](https://github.com/i-am-logger/pr4xis/pull/84))
- artifact_identity + data_provisioning — ontological external-data subsystem, no more LFS ([#71](https://github.com/i-am-logger/pr4xis/pull/71))
- staging ontology — Futamura's partial-evaluation framework as a meta-ontology ([#67](https://github.com/i-am-logger/pr4xis/pull/67))
- enforce ontology patterns — define_ontology! everywhere, 4851 tests
- integrate Kleisli + anamorphism + Yoneda into causation reasoning
- integrate algebraic structures into reasoning + tracing
- F-algebra, MonoidalCategory, Optics, MonadTransformer (4 structures)
- complete algebraic structure library — 7 new structures
- Reader + State monads with property-based tests
- migrate remaining 5 biomedical Ontology impls to structural + domain split
- migrate Ontology impls to structural + domain axiom split
- Ontology trait — structural + domain axioms merged via monoid
- define_ontology! clean API — concepts/is_a/has_a/causes/opposes + auto structural axioms
- migrate 41 ontologies to define_ontology! macro (-3163 lines)
- define_ontology! macro — generates Category + Taxonomy + Mereology + Causation + Opposition + OntologyMeta
- Monoid + Writer monad + TracedCategory refactor (Moggi 1991, Mac Lane 1971)
- restructure to academic hierarchy (DOLCE-aligned)
- migrate ~65 Pattern B ontologies to define_dense_category! (-5325 lines)
- migrate 30 Pattern A ontologies to define_category! macro (-4404 lines)
- derive macros — #[derive(Entity)] + define_category! + define_dense_category!
- dev-web serves chatbot at /, presentation at /decks/technical
- rename praxis → pr4xis across entire codebase
- migrate burp + pssst ontologies — 123 categories, 61 functors, 4707 tests
- migrate 11 vogix ontologies into praxis theming (3117 tests)
- prop tests + functor connections across 15 ontologies (2934 tests, 18 functors)
- PregroupCategory — proper Category with proven laws
- NoisyChannel→Communication + DRT→Dialogue functors (proven)
- Diagnostics→Control functor (FDI IS control — Gertler 1998, proven)
- Communication→Control + Diagnostics→Metacognition functors (proven)
- TraceSchema ontology — T(C) = El(C) + O_obs (Spivak 2012)
- trace shows Lambek notation (S[q]/NP) not Rust debug format
- proper ontology trace — each step reports what it did with status
- proper ontology trace — each step reports what it did with status
- trace functors — map pipeline steps to Diagnostics/PROV ontologies
- Diagnostics ontology + TracedCategory (writer monad on categories)
- proper ontology trace — each step reports what it did with status
- add Ontology Alignment and NLG pipeline ontologies
- rich taxonomy responses with path, definitions, and subtypes
- add criterion benchmarks for all ontologies and chat pipeline
- add Instance ontology (Spivak) and SystemsToSchema functor
- add durability, volatility, measurement, and benchmark ontologies
- merge sensor-fusion ontologies, add schema/storage/consistency ontologies, praxis-web
- speech production ontology (Levelt pipeline as category)
- Traum grounding state machine as finite category
- extend dialogue ontology with QUD, CommonGround, Intention, Repair
- function words as LMF data, extend LmfPos with closed-class types
- extended sentence test suite, chart type selection fix
- self-model ontology, CYK chart parser, adjunction, response generation
- integration tests with full WordNet — expose real failures honestly
- docs/chat/ for GitHub Pages — presentation embeds live chatbot
- complete scheme taxonomy — Vogix16, Ansi16, Base24, SchemeType
- Turing test benchmark — 18 questions, 3 pass, 15 need ontologies
- ColorSlot::key() — canonical theme file key names
- Rgb::from_hex and to_hex for color parsing
- Language::pregroup_types — end-to-end pregroup pipeline through Language trait
- Lambek → Pregroup functor — ontology evolution proven
- load WordNet verb frames for transitivity — no more defaults
- pregroup grammar ontology — parsing as group algebra
- integrate ontologies via functors, wire into chatbot pipeline
- control systems ontology + foundational cybernetics papers
- math functions + sRGB color science + theming ontology
- cognition ontologies — distinction, epistemics, metacognition
- question grammar types + Q semantic domain in Lambek/Montague
- Montague functor — type-driven syntax-semantics interpretation
- Lambek grammar — syntax as category, text understood through type reduction
- dialogue ontology + chatbot CLI — praxis can chat
- SystemsToEvents functor — closes System ↔ EventDriven ↔ Concurrent triangle
- event-driven ontology — chess IS event-driven IS concurrent (proven)
- SystemsToConcurrency functor — every system IS concurrent (proven)
- concurrency ontology — chess IS concurrent (proven via functor)
- Language trait, orthography, morphology, cached reasoning queries
- English language ontology — 107k concepts, nanosecond queries
- information ontology — what bits, bytes, references, and text ARE
- WordNet-LMF ontology — full 107k synset load in 3.8s
- XML ontology, enhanced property tests, systems thinking completeness
- DOLCE upper ontology, domain restructure, linguistics, systems thinking, codegen

### Fix

- *(reasoning)* Parthood is irreflexive; a grounded edge is not a morphism
- *(derive)* has_a: emits part→whole Parthood edges (BFO:0000050 alignment)
- *(chat)* a Parthood answer phrases "is part of", not "is a"
- *(morphology)* satisfy clippy --all-targets --release in the AGID regenerate helper
- *(linguistics)* address Copilot review — cache operator vocab + don't split glyphs inside words
- *(docs,grounding)* broken intra-doc links + Copilot review
- *(#62)* address 11 copilot comments on consolidated Heim PR #143 (#144)
- clippy clean — no dead code, no unused imports, no stubs
- remove unused Category imports from test modules (clippy -D warnings)
- qualify kind refs in define_category! macro (avoid Identity ambiguity) + LOC badge
- update release-please config for pr4xis rename + add version to path deps
- add WordNet XML (LFS) + tinted-schemes submodule for CI
- skip data-dependent tests when WordNet/themes not available (CI)
- clarify Base16 has 16 slots, Base24 has 24
- remove hardcoded quit/exit — farewell detection through language lexicon
- remove all hardcoded pronoun/noun matching from dialogue engine
- taxonomy query works — 'is a dog a mammal' answered correctly
- copula type from CCG research — question 'is X a Y' now parses to Q type
- resolve all clippy warnings for strict CI

### Perf

- *(morphology)* cache the parsed irregulars table behind OnceLock in std (Copilot review)
- praxis tests give faster feedback and catch slowdowns earlier ([#185](https://github.com/i-am-logger/pr4xis/pull/185))

### Refactor

- *(self-aware)* address branch-review findings — doc-rot + one Form-aware counter
- *(decoders)* exhaustive has_decoder_for + per-module DECODES const (D-22)
- *(lmf)* verb transitivity from the loaded frame text, not the id prefix (D-15)
- *(registry)* source disk paths as praxis.toml data, not Rust dispatch (D-9)
- *(linguistics)* delete the dead legacy discourse module (D-11)
- *(lexicon)* collapse pos_to_olia_fragments to the canonical anchor (D-13)
- *(xsd)* derive datatype base_type + groups from the loaded hierarchy (D-16/D-6/D-5)
- *(relations)* relation→structural-property as loaded edges, not a Rust match (D-7)
- *(meta)* derive identity leaves, domain rank, and the XSD baseline from loaded data (D-18/D-19/D-20/D-17)
- *(domains)* derive is_leaf from the loaded graph + type the theme polarity (D-18, D-10)
- *(domains)* lower determiner/interjection features through ONE codec, not scattered synset.contains
- *(linguistics)* migrate ALL lexical-category assignment into the loaded functor (Batch K)
- migrate pr4xis-domains to feat/logic-ontologies API ([#170](https://github.com/i-am-logger/pr4xis/pull/170))
- *(#113)* batch 3 — final cognitive ontologies (lemon, consciousness, self_model) (#120)
- *(#113)* batch 2 — dialogue/pragmatics cluster (7 ontologies) (#119)
- *(#113)* migrate 18 ontologies to ontology! proc macro (#116)
- applied/theming/ → applied/hmi/{theming,surfaces,visualization,input,report,explorer}/ ([#66](https://github.com/i-am-logger/pr4xis/pull/66))
- move meta/methodological ontologies from biomedical to formal
- response generation through SVO grammar, not hardcoded strings
- load function words from LMF data file instead of hardcoded Rust
- rename Lambek types::english to types::svo (language-agnostic)
- replace hardcoded chat strings with response realization ontology
- merge English + EnglishLanguage into one type
- delete function_words.rs and vocabulary.rs — all lookups through Language
- Language trait as single lexical interface — tokenizer is language-agnostic
- remove hardcoded Montague, keep Lambek grammar clean
- revert hardcoded CLI, add dialogue ontology, add missing tests
- merge science into domains, reorganize by ontology, harden engine
- consolidate 18 crates → 4 workspace members

### Research

- *(#98)* kinded-functor failures diagnosed — three distinct problems, none lax (#129)

### Test

- *(chat,domains)* "is X part of Y" answers over a loaded Parthood mereology (Step 3)
- property-based tests for math functions + sRGB color science
- comprehensive prop-based tests for cognition ontologies

## [0.25.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.24.0...pr4xis-domains-v0.25.0) - 2026-06-16

### Build

- *(release)* version the domains crate via release-plz
- *(release)* single workspace version via inheritance — fix the release-plz drift

### Docs

- *(domains)* green the workspace rustdoc over the OWL/USC bridges
- *(domains)* fix two rustdoc intra-doc links (CI Docs step)
- complete validation overhaul — per-def trait sweep + rustdoc rot + mdBook ([#176](https://github.com/i-am-logger/pr4xis/pull/176))
- *(#173)* per-ontology rollout for the 5 new HMI sub-ontologies (#69)
- rewrite + per-ontology rollout (#57, #55, #52, #46, #44) ([#63](https://github.com/i-am-logger/pr4xis/pull/63))
- color science source papers
- pregroup grammar research — parsing as group algebra

### Feat

- *(self-aware)* the catalog includes loaded-but-unregistered ontologies (§3)
- *(runtime,chat)* relation-parametric image/meet/chain — Parthood evidence chains
- *(chat)* single-word loaded entities type NP — "is X part of Y" parses with a one-word X
- *(self-aware)* content-addressed load history + state fingerprint (Step 6 / §2.4)
- *(self-aware)* per-ontology capabilities — what each loaded ontology can answer (§4.7)
- *(self-aware)* provenance names the loaded ontology a turn reasoned over (Step 5 / §2.3)
- *(self-aware)* the SelfModel eigenform observes the live loaded set (Step 4 / §2)
- *(lambek,chat)* "is X part of Y" parses from raw text (Step 5 — the parse)
- *(domains)* trim the relation lexicon to non-default complement-headed surfaces
- *(chat)* lower a question's predicate to a typed relation kind (Step 3)
- *(domains)* relation_lexicon.prx — the loaded "part of"↦Parthood surface map (Step 3)
- *(domains)* relation-parametric LexicalReasoner::reaches (Step 3 spine)
- *(domains)* the USC→praxis functor lives in usc_functor.prx, not Rust ([#203](https://github.com/i-am-logger/pr4xis/pull/203))
- *(domains)* the OWL→praxis functor lives in owl_functor.prx, not Rust ([#203](https://github.com/i-am-logger/pr4xis/pull/203))
- *(domains)* the WordNet→praxis functor lives in english_functor.prx, not Rust ([#203](https://github.com/i-am-logger/pr4xis/pull/203))
- *(uslm)* mint USC heading + citation Forms — sections answerable by "section N" (§9c)
- *(owl,wasm)* mint OWL label Forms — loaded OWL answerable by its label (§9b)
- *(composed)* index a loaded concept's Form-atom surfaces (§9 lexicalization)
- *(chat)* multi-token surface recognition — phrase/citation lookup
- *(uslm)* lift USC→praxis to functor-as-data (off the baked Parthood/Section)
- *(owl)* OWL→praxis as a functor-as-data projection (not a baked converter)
- *(derive,runtime)* transitive kinds are loaded from data — Phase A Step 4
- *(runtime)* relation kind is a ConceptRef, not a closed enum — Phase A Step 3
- *(runtime)* transitive_kinds — relation transitivity as loaded data, not a constant
- *(morphology)* ground the productive rules in registered CatVar; structural rule count (D-2/D-12)
- *(morphology)* irregular forms LOADED from the registered AGID source (MORPH/D-1)
- *(linguistics)* [**breaking**] literature-honest determiner/interjection features (FW-B)
- *(domains)* function-words load from a committed .prx — the ClosedClassLexicon (FW-A)
- *(linguistics)* wh-questions via a loaded OLiA→CCG category functor ([#169](https://github.com/i-am-logger/pr4xis/pull/169))
- *(linguistics)* math operators as a loaded vocabulary — the #169 tokenizer fix
- generic grounding — ground(lens) over the substrate, denotes is one lens
- *(domains)* USC → Archive — statute provisions in the generic substrate (the ontological way)
- *(domains)* persist the denotes pointer column in the USC envelope (G3b-2b code)
- *(domains)* the denotes-floor producer — statute prose → ontolex:Form pointers (G3b-2a)
- *(domains)* the honest denotes floor — a span grounds into an ontolex:Form (G3b-1)
- *(runtime)* EdgeTarget — the foreign-atom slot, byte-exact (grounding G2)
- *(domains)* bridge loaded English to a RuntimeOntology — the #87 grounding gate
- *(domains)* the WordNet→praxis functor, carried as .prx data
- *(domains)* project English into a runtime Archive — the WordNet→.prx source
- `.prx` — praxis' knowledge, in a file ([#186](https://github.com/i-am-logger/pr4xis/pull/186))
- [**breaking**] sync all praxis crates to one version (0.22.0) ([#190](https://github.com/i-am-logger/pr4xis/pull/190))
- [**breaking**] restore praxis publishing to crates.io ([#188](https://github.com/i-am-logger/pr4xis/pull/188))
- *(cli)* one-command corpus updates via `pr4xis update --lock` ([#183](https://github.com/i-am-logger/pr4xis/pull/183))
- praxis + praxis-cli gain the registered-source mechanism (SOX 1514A, AIR21 42121) ([#179](https://github.com/i-am-logger/pr4xis/pull/179))
- *(deps,ci)* [**breaking**] pure-Rust crypto + always-latest devenv + PR-title gate ([#177](https://github.com/i-am-logger/pr4xis/pull/177))
- *(#91)* pr4xis core + domains run no_std + alloc (#157)
- *(#148)* Lemon meta on every structural entity — uniform registry for ontologies, axioms, functors, adjunctions, nat-trans (#150)
- *(#117)* MAPE-K ontology — chat pipeline's literature-grounded home (#146)
- *(#62)* Heim syntrometric lineage — consolidated stack (#143)
- *(#62)* Heim syntrometry Phase 1 — lineage verified by functor laws (#135)
- *(#131)* TerminalFunctor helper — reusable one-object collapse (#134)
- *(#130)* Category::Op<C> + empirical 4th failure mode discovery (#133)
- *(#123)* Resilience ontology — Nygard/Brooker/Armstrong/Patterson (#128)
- *(#124)* Endofunctor trait — first-class C → C functor (#127)
- *(#122)* Dependability ontology — Avizienis-Laprie-Randell-Landwehr (2004) (#125)
- typed Vocabulary — OntologyName, ModulePath, structured Citation ([#111](https://github.com/i-am-logger/pr4xis/pull/111))
- compose API — runtime ontology composition via Korporator ([#103](https://github.com/i-am-logger/pr4xis/pull/103)) ([#108](https://github.com/i-am-logger/pr4xis/pull/108))
- Ontolex-Lemon, consciousness C1×C2, complete functor chain, Vocabulary API ([#88](https://github.com/i-am-logger/pr4xis/pull/88)) ([#104](https://github.com/i-am-logger/pr4xis/pull/104))
- define_ontology! being: clause + register all 108 ontologies ([#76](https://github.com/i-am-logger/pr4xis/pull/76)) ([#84](https://github.com/i-am-logger/pr4xis/pull/84))
- artifact_identity + data_provisioning — ontological external-data subsystem, no more LFS ([#71](https://github.com/i-am-logger/pr4xis/pull/71))
- staging ontology — Futamura's partial-evaluation framework as a meta-ontology ([#67](https://github.com/i-am-logger/pr4xis/pull/67))
- enforce ontology patterns — define_ontology! everywhere, 4851 tests
- integrate Kleisli + anamorphism + Yoneda into causation reasoning
- integrate algebraic structures into reasoning + tracing
- F-algebra, MonoidalCategory, Optics, MonadTransformer (4 structures)
- complete algebraic structure library — 7 new structures
- Reader + State monads with property-based tests
- migrate remaining 5 biomedical Ontology impls to structural + domain split
- migrate Ontology impls to structural + domain axiom split
- Ontology trait — structural + domain axioms merged via monoid
- define_ontology! clean API — concepts/is_a/has_a/causes/opposes + auto structural axioms
- migrate 41 ontologies to define_ontology! macro (-3163 lines)
- define_ontology! macro — generates Category + Taxonomy + Mereology + Causation + Opposition + OntologyMeta
- Monoid + Writer monad + TracedCategory refactor (Moggi 1991, Mac Lane 1971)
- restructure to academic hierarchy (DOLCE-aligned)
- migrate ~65 Pattern B ontologies to define_dense_category! (-5325 lines)
- migrate 30 Pattern A ontologies to define_category! macro (-4404 lines)
- derive macros — #[derive(Entity)] + define_category! + define_dense_category!
- dev-web serves chatbot at /, presentation at /decks/technical
- rename praxis → pr4xis across entire codebase
- migrate burp + pssst ontologies — 123 categories, 61 functors, 4707 tests
- migrate 11 vogix ontologies into praxis theming (3117 tests)
- prop tests + functor connections across 15 ontologies (2934 tests, 18 functors)
- PregroupCategory — proper Category with proven laws
- NoisyChannel→Communication + DRT→Dialogue functors (proven)
- Diagnostics→Control functor (FDI IS control — Gertler 1998, proven)
- Communication→Control + Diagnostics→Metacognition functors (proven)
- TraceSchema ontology — T(C) = El(C) + O_obs (Spivak 2012)
- trace shows Lambek notation (S[q]/NP) not Rust debug format
- proper ontology trace — each step reports what it did with status
- proper ontology trace — each step reports what it did with status
- trace functors — map pipeline steps to Diagnostics/PROV ontologies
- Diagnostics ontology + TracedCategory (writer monad on categories)
- proper ontology trace — each step reports what it did with status
- add Ontology Alignment and NLG pipeline ontologies
- rich taxonomy responses with path, definitions, and subtypes
- add criterion benchmarks for all ontologies and chat pipeline
- add Instance ontology (Spivak) and SystemsToSchema functor
- add durability, volatility, measurement, and benchmark ontologies
- merge sensor-fusion ontologies, add schema/storage/consistency ontologies, praxis-web
- speech production ontology (Levelt pipeline as category)
- Traum grounding state machine as finite category
- extend dialogue ontology with QUD, CommonGround, Intention, Repair
- function words as LMF data, extend LmfPos with closed-class types
- extended sentence test suite, chart type selection fix
- self-model ontology, CYK chart parser, adjunction, response generation
- integration tests with full WordNet — expose real failures honestly
- docs/chat/ for GitHub Pages — presentation embeds live chatbot
- complete scheme taxonomy — Vogix16, Ansi16, Base24, SchemeType
- Turing test benchmark — 18 questions, 3 pass, 15 need ontologies
- ColorSlot::key() — canonical theme file key names
- Rgb::from_hex and to_hex for color parsing
- Language::pregroup_types — end-to-end pregroup pipeline through Language trait
- Lambek → Pregroup functor — ontology evolution proven
- load WordNet verb frames for transitivity — no more defaults
- pregroup grammar ontology — parsing as group algebra
- integrate ontologies via functors, wire into chatbot pipeline
- control systems ontology + foundational cybernetics papers
- math functions + sRGB color science + theming ontology
- cognition ontologies — distinction, epistemics, metacognition
- question grammar types + Q semantic domain in Lambek/Montague
- Montague functor — type-driven syntax-semantics interpretation
- Lambek grammar — syntax as category, text understood through type reduction
- dialogue ontology + chatbot CLI — praxis can chat
- SystemsToEvents functor — closes System ↔ EventDriven ↔ Concurrent triangle
- event-driven ontology — chess IS event-driven IS concurrent (proven)
- SystemsToConcurrency functor — every system IS concurrent (proven)
- concurrency ontology — chess IS concurrent (proven via functor)
- Language trait, orthography, morphology, cached reasoning queries
- English language ontology — 107k concepts, nanosecond queries
- information ontology — what bits, bytes, references, and text ARE
- WordNet-LMF ontology — full 107k synset load in 3.8s
- XML ontology, enhanced property tests, systems thinking completeness
- DOLCE upper ontology, domain restructure, linguistics, systems thinking, codegen

### Fix

- *(reasoning)* Parthood is irreflexive; a grounded edge is not a morphism
- *(derive)* has_a: emits part→whole Parthood edges (BFO:0000050 alignment)
- *(chat)* a Parthood answer phrases "is part of", not "is a"
- *(morphology)* satisfy clippy --all-targets --release in the AGID regenerate helper
- *(linguistics)* address Copilot review — cache operator vocab + don't split glyphs inside words
- *(docs,grounding)* broken intra-doc links + Copilot review
- *(#62)* address 11 copilot comments on consolidated Heim PR #143 (#144)
- clippy clean — no dead code, no unused imports, no stubs
- remove unused Category imports from test modules (clippy -D warnings)
- qualify kind refs in define_category! macro (avoid Identity ambiguity) + LOC badge
- update release-please config for pr4xis rename + add version to path deps
- add WordNet XML (LFS) + tinted-schemes submodule for CI
- skip data-dependent tests when WordNet/themes not available (CI)
- clarify Base16 has 16 slots, Base24 has 24
- remove hardcoded quit/exit — farewell detection through language lexicon
- remove all hardcoded pronoun/noun matching from dialogue engine
- taxonomy query works — 'is a dog a mammal' answered correctly
- copula type from CCG research — question 'is X a Y' now parses to Q type
- resolve all clippy warnings for strict CI

### Perf

- *(morphology)* cache the parsed irregulars table behind OnceLock in std (Copilot review)
- praxis tests give faster feedback and catch slowdowns earlier ([#185](https://github.com/i-am-logger/pr4xis/pull/185))

### Refactor

- *(self-aware)* address branch-review findings — doc-rot + one Form-aware counter
- *(decoders)* exhaustive has_decoder_for + per-module DECODES const (D-22)
- *(lmf)* verb transitivity from the loaded frame text, not the id prefix (D-15)
- *(registry)* source disk paths as praxis.toml data, not Rust dispatch (D-9)
- *(linguistics)* delete the dead legacy discourse module (D-11)
- *(lexicon)* collapse pos_to_olia_fragments to the canonical anchor (D-13)
- *(xsd)* derive datatype base_type + groups from the loaded hierarchy (D-16/D-6/D-5)
- *(relations)* relation→structural-property as loaded edges, not a Rust match (D-7)
- *(meta)* derive identity leaves, domain rank, and the XSD baseline from loaded data (D-18/D-19/D-20/D-17)
- *(domains)* derive is_leaf from the loaded graph + type the theme polarity (D-18, D-10)
- *(domains)* lower determiner/interjection features through ONE codec, not scattered synset.contains
- *(linguistics)* migrate ALL lexical-category assignment into the loaded functor (Batch K)
- migrate pr4xis-domains to feat/logic-ontologies API ([#170](https://github.com/i-am-logger/pr4xis/pull/170))
- *(#113)* batch 3 — final cognitive ontologies (lemon, consciousness, self_model) (#120)
- *(#113)* batch 2 — dialogue/pragmatics cluster (7 ontologies) (#119)
- *(#113)* migrate 18 ontologies to ontology! proc macro (#116)
- applied/theming/ → applied/hmi/{theming,surfaces,visualization,input,report,explorer}/ ([#66](https://github.com/i-am-logger/pr4xis/pull/66))
- move meta/methodological ontologies from biomedical to formal
- response generation through SVO grammar, not hardcoded strings
- load function words from LMF data file instead of hardcoded Rust
- rename Lambek types::english to types::svo (language-agnostic)
- replace hardcoded chat strings with response realization ontology
- merge English + EnglishLanguage into one type
- delete function_words.rs and vocabulary.rs — all lookups through Language
- Language trait as single lexical interface — tokenizer is language-agnostic
- remove hardcoded Montague, keep Lambek grammar clean
- revert hardcoded CLI, add dialogue ontology, add missing tests
- merge science into domains, reorganize by ontology, harden engine
- consolidate 18 crates → 4 workspace members

### Research

- *(#98)* kinded-functor failures diagnosed — three distinct problems, none lax (#129)

### Test

- *(chat,domains)* "is X part of Y" answers over a loaded Parthood mereology (Step 3)
- property-based tests for math functions + sRGB color science
- comprehensive prop-based tests for cognition ontologies

## [0.24.0](https://github.com/i-am-logger/pr4xis/releases/tag/pr4xis-domains-v0.24.0) - 2026-06-14

### Build

- *(release)* version the domains crate via release-plz
- *(release)* single workspace version via inheritance — fix the release-plz drift

### Docs

- *(domains)* fix two rustdoc intra-doc links (CI Docs step)
- complete validation overhaul — per-def trait sweep + rustdoc rot + mdBook ([#176](https://github.com/i-am-logger/pr4xis/pull/176))
- *(#173)* per-ontology rollout for the 5 new HMI sub-ontologies (#69)
- rewrite + per-ontology rollout (#57, #55, #52, #46, #44) ([#63](https://github.com/i-am-logger/pr4xis/pull/63))
- color science source papers
- pregroup grammar research — parsing as group algebra

### Feat

- *(morphology)* ground the productive rules in registered CatVar; structural rule count (D-2/D-12)
- *(morphology)* irregular forms LOADED from the registered AGID source (MORPH/D-1)
- *(linguistics)* [**breaking**] literature-honest determiner/interjection features (FW-B)
- *(domains)* function-words load from a committed .prx — the ClosedClassLexicon (FW-A)
- *(linguistics)* wh-questions via a loaded OLiA→CCG category functor ([#169](https://github.com/i-am-logger/pr4xis/pull/169))
- *(linguistics)* math operators as a loaded vocabulary — the #169 tokenizer fix
- generic grounding — ground(lens) over the substrate, denotes is one lens
- *(domains)* USC → Archive — statute provisions in the generic substrate (the ontological way)
- *(domains)* persist the denotes pointer column in the USC envelope (G3b-2b code)
- *(domains)* the denotes-floor producer — statute prose → ontolex:Form pointers (G3b-2a)
- *(domains)* the honest denotes floor — a span grounds into an ontolex:Form (G3b-1)
- *(runtime)* EdgeTarget — the foreign-atom slot, byte-exact (grounding G2)
- *(domains)* bridge loaded English to a RuntimeOntology — the #87 grounding gate
- *(domains)* the WordNet→praxis functor, carried as .prx data
- *(domains)* project English into a runtime Archive — the WordNet→.prx source
- `.prx` — praxis' knowledge, in a file ([#186](https://github.com/i-am-logger/pr4xis/pull/186))
- [**breaking**] sync all praxis crates to one version (0.22.0) ([#190](https://github.com/i-am-logger/pr4xis/pull/190))
- [**breaking**] restore praxis publishing to crates.io ([#188](https://github.com/i-am-logger/pr4xis/pull/188))
- *(cli)* one-command corpus updates via `pr4xis update --lock` ([#183](https://github.com/i-am-logger/pr4xis/pull/183))
- praxis + praxis-cli gain the registered-source mechanism (SOX 1514A, AIR21 42121) ([#179](https://github.com/i-am-logger/pr4xis/pull/179))
- *(deps,ci)* [**breaking**] pure-Rust crypto + always-latest devenv + PR-title gate ([#177](https://github.com/i-am-logger/pr4xis/pull/177))
- *(#91)* pr4xis core + domains run no_std + alloc (#157)
- *(#148)* Lemon meta on every structural entity — uniform registry for ontologies, axioms, functors, adjunctions, nat-trans (#150)
- *(#117)* MAPE-K ontology — chat pipeline's literature-grounded home (#146)
- *(#62)* Heim syntrometric lineage — consolidated stack (#143)
- *(#62)* Heim syntrometry Phase 1 — lineage verified by functor laws (#135)
- *(#131)* TerminalFunctor helper — reusable one-object collapse (#134)
- *(#130)* Category::Op<C> + empirical 4th failure mode discovery (#133)
- *(#123)* Resilience ontology — Nygard/Brooker/Armstrong/Patterson (#128)
- *(#124)* Endofunctor trait — first-class C → C functor (#127)
- *(#122)* Dependability ontology — Avizienis-Laprie-Randell-Landwehr (2004) (#125)
- typed Vocabulary — OntologyName, ModulePath, structured Citation ([#111](https://github.com/i-am-logger/pr4xis/pull/111))
- compose API — runtime ontology composition via Korporator ([#103](https://github.com/i-am-logger/pr4xis/pull/103)) ([#108](https://github.com/i-am-logger/pr4xis/pull/108))
- Ontolex-Lemon, consciousness C1×C2, complete functor chain, Vocabulary API ([#88](https://github.com/i-am-logger/pr4xis/pull/88)) ([#104](https://github.com/i-am-logger/pr4xis/pull/104))
- define_ontology! being: clause + register all 108 ontologies ([#76](https://github.com/i-am-logger/pr4xis/pull/76)) ([#84](https://github.com/i-am-logger/pr4xis/pull/84))
- artifact_identity + data_provisioning — ontological external-data subsystem, no more LFS ([#71](https://github.com/i-am-logger/pr4xis/pull/71))
- staging ontology — Futamura's partial-evaluation framework as a meta-ontology ([#67](https://github.com/i-am-logger/pr4xis/pull/67))
- enforce ontology patterns — define_ontology! everywhere, 4851 tests
- integrate Kleisli + anamorphism + Yoneda into causation reasoning
- integrate algebraic structures into reasoning + tracing
- F-algebra, MonoidalCategory, Optics, MonadTransformer (4 structures)
- complete algebraic structure library — 7 new structures
- Reader + State monads with property-based tests
- migrate remaining 5 biomedical Ontology impls to structural + domain split
- migrate Ontology impls to structural + domain axiom split
- Ontology trait — structural + domain axioms merged via monoid
- define_ontology! clean API — concepts/is_a/has_a/causes/opposes + auto structural axioms
- migrate 41 ontologies to define_ontology! macro (-3163 lines)
- define_ontology! macro — generates Category + Taxonomy + Mereology + Causation + Opposition + OntologyMeta
- Monoid + Writer monad + TracedCategory refactor (Moggi 1991, Mac Lane 1971)
- restructure to academic hierarchy (DOLCE-aligned)
- migrate ~65 Pattern B ontologies to define_dense_category! (-5325 lines)
- migrate 30 Pattern A ontologies to define_category! macro (-4404 lines)
- derive macros — #[derive(Entity)] + define_category! + define_dense_category!
- dev-web serves chatbot at /, presentation at /decks/technical
- rename praxis → pr4xis across entire codebase
- migrate burp + pssst ontologies — 123 categories, 61 functors, 4707 tests
- migrate 11 vogix ontologies into praxis theming (3117 tests)
- prop tests + functor connections across 15 ontologies (2934 tests, 18 functors)
- PregroupCategory — proper Category with proven laws
- NoisyChannel→Communication + DRT→Dialogue functors (proven)
- Diagnostics→Control functor (FDI IS control — Gertler 1998, proven)
- Communication→Control + Diagnostics→Metacognition functors (proven)
- TraceSchema ontology — T(C) = El(C) + O_obs (Spivak 2012)
- trace shows Lambek notation (S[q]/NP) not Rust debug format
- proper ontology trace — each step reports what it did with status
- proper ontology trace — each step reports what it did with status
- trace functors — map pipeline steps to Diagnostics/PROV ontologies
- Diagnostics ontology + TracedCategory (writer monad on categories)
- proper ontology trace — each step reports what it did with status
- add Ontology Alignment and NLG pipeline ontologies
- rich taxonomy responses with path, definitions, and subtypes
- add criterion benchmarks for all ontologies and chat pipeline
- add Instance ontology (Spivak) and SystemsToSchema functor
- add durability, volatility, measurement, and benchmark ontologies
- merge sensor-fusion ontologies, add schema/storage/consistency ontologies, praxis-web
- speech production ontology (Levelt pipeline as category)
- Traum grounding state machine as finite category
- extend dialogue ontology with QUD, CommonGround, Intention, Repair
- function words as LMF data, extend LmfPos with closed-class types
- extended sentence test suite, chart type selection fix
- self-model ontology, CYK chart parser, adjunction, response generation
- integration tests with full WordNet — expose real failures honestly
- docs/chat/ for GitHub Pages — presentation embeds live chatbot
- complete scheme taxonomy — Vogix16, Ansi16, Base24, SchemeType
- Turing test benchmark — 18 questions, 3 pass, 15 need ontologies
- ColorSlot::key() — canonical theme file key names
- Rgb::from_hex and to_hex for color parsing
- Language::pregroup_types — end-to-end pregroup pipeline through Language trait
- Lambek → Pregroup functor — ontology evolution proven
- load WordNet verb frames for transitivity — no more defaults
- pregroup grammar ontology — parsing as group algebra
- integrate ontologies via functors, wire into chatbot pipeline
- control systems ontology + foundational cybernetics papers
- math functions + sRGB color science + theming ontology
- cognition ontologies — distinction, epistemics, metacognition
- question grammar types + Q semantic domain in Lambek/Montague
- Montague functor — type-driven syntax-semantics interpretation
- Lambek grammar — syntax as category, text understood through type reduction
- dialogue ontology + chatbot CLI — praxis can chat
- SystemsToEvents functor — closes System ↔ EventDriven ↔ Concurrent triangle
- event-driven ontology — chess IS event-driven IS concurrent (proven)
- SystemsToConcurrency functor — every system IS concurrent (proven)
- concurrency ontology — chess IS concurrent (proven via functor)
- Language trait, orthography, morphology, cached reasoning queries
- English language ontology — 107k concepts, nanosecond queries
- information ontology — what bits, bytes, references, and text ARE
- WordNet-LMF ontology — full 107k synset load in 3.8s
- XML ontology, enhanced property tests, systems thinking completeness
- DOLCE upper ontology, domain restructure, linguistics, systems thinking, codegen

### Fix

- *(morphology)* satisfy clippy --all-targets --release in the AGID regenerate helper
- *(linguistics)* address Copilot review — cache operator vocab + don't split glyphs inside words
- *(docs,grounding)* broken intra-doc links + Copilot review
- *(#62)* address 11 copilot comments on consolidated Heim PR #143 (#144)
- clippy clean — no dead code, no unused imports, no stubs
- remove unused Category imports from test modules (clippy -D warnings)
- qualify kind refs in define_category! macro (avoid Identity ambiguity) + LOC badge
- update release-please config for pr4xis rename + add version to path deps
- add WordNet XML (LFS) + tinted-schemes submodule for CI
- skip data-dependent tests when WordNet/themes not available (CI)
- clarify Base16 has 16 slots, Base24 has 24
- remove hardcoded quit/exit — farewell detection through language lexicon
- remove all hardcoded pronoun/noun matching from dialogue engine
- taxonomy query works — 'is a dog a mammal' answered correctly
- copula type from CCG research — question 'is X a Y' now parses to Q type
- resolve all clippy warnings for strict CI

### Perf

- *(morphology)* cache the parsed irregulars table behind OnceLock in std (Copilot review)
- praxis tests give faster feedback and catch slowdowns earlier ([#185](https://github.com/i-am-logger/pr4xis/pull/185))

### Refactor

- *(decoders)* exhaustive has_decoder_for + per-module DECODES const (D-22)
- *(lmf)* verb transitivity from the loaded frame text, not the id prefix (D-15)
- *(registry)* source disk paths as praxis.toml data, not Rust dispatch (D-9)
- *(linguistics)* delete the dead legacy discourse module (D-11)
- *(lexicon)* collapse pos_to_olia_fragments to the canonical anchor (D-13)
- *(xsd)* derive datatype base_type + groups from the loaded hierarchy (D-16/D-6/D-5)
- *(relations)* relation→structural-property as loaded edges, not a Rust match (D-7)
- *(meta)* derive identity leaves, domain rank, and the XSD baseline from loaded data (D-18/D-19/D-20/D-17)
- *(domains)* derive is_leaf from the loaded graph + type the theme polarity (D-18, D-10)
- *(domains)* lower determiner/interjection features through ONE codec, not scattered synset.contains
- *(linguistics)* migrate ALL lexical-category assignment into the loaded functor (Batch K)
- migrate pr4xis-domains to feat/logic-ontologies API ([#170](https://github.com/i-am-logger/pr4xis/pull/170))
- *(#113)* batch 3 — final cognitive ontologies (lemon, consciousness, self_model) (#120)
- *(#113)* batch 2 — dialogue/pragmatics cluster (7 ontologies) (#119)
- *(#113)* migrate 18 ontologies to ontology! proc macro (#116)
- applied/theming/ → applied/hmi/{theming,surfaces,visualization,input,report,explorer}/ ([#66](https://github.com/i-am-logger/pr4xis/pull/66))
- move meta/methodological ontologies from biomedical to formal
- response generation through SVO grammar, not hardcoded strings
- load function words from LMF data file instead of hardcoded Rust
- rename Lambek types::english to types::svo (language-agnostic)
- replace hardcoded chat strings with response realization ontology
- merge English + EnglishLanguage into one type
- delete function_words.rs and vocabulary.rs — all lookups through Language
- Language trait as single lexical interface — tokenizer is language-agnostic
- remove hardcoded Montague, keep Lambek grammar clean
- revert hardcoded CLI, add dialogue ontology, add missing tests
- merge science into domains, reorganize by ontology, harden engine
- consolidate 18 crates → 4 workspace members

### Research

- *(#98)* kinded-functor failures diagnosed — three distinct problems, none lax (#129)

### Test

- property-based tests for math functions + sRGB color science
- comprehensive prop-based tests for cognition ontologies

## [0.22.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.7.0...pr4xis-domains-v0.22.0) - 2026-06-01

### Added

- [**breaking**] sync all praxis crates to one version (0.22.0) ([#190](https://github.com/i-am-logger/pr4xis/pull/190))
- [**breaking**] restore praxis publishing to crates.io ([#188](https://github.com/i-am-logger/pr4xis/pull/188))
- *(cli)* one-command corpus updates via `pr4xis update --lock` ([#183](https://github.com/i-am-logger/pr4xis/pull/183))
- praxis + praxis-cli gain the registered-source mechanism (SOX 1514A, AIR21 42121) ([#179](https://github.com/i-am-logger/pr4xis/pull/179))
- *(deps,ci)* [**breaking**] pure-Rust crypto + always-latest devenv + PR-title gate ([#177](https://github.com/i-am-logger/pr4xis/pull/177))
- *(#91)* pr4xis core + domains run no_std + alloc (#157)
- *(#148)* Lemon meta on every structural entity — uniform registry for ontologies, axioms, functors, adjunctions, nat-trans (#150)
- *(#117)* MAPE-K ontology — chat pipeline's literature-grounded home (#146)
- *(#62)* Heim syntrometric lineage — consolidated stack (#143)
- *(#62)* Heim syntrometry Phase 1 — lineage verified by functor laws (#135)
- *(#131)* TerminalFunctor helper — reusable one-object collapse (#134)
- *(#130)* Category::Op<C> + empirical 4th failure mode discovery (#133)
- *(#123)* Resilience ontology — Nygard/Brooker/Armstrong/Patterson (#128)
- *(#124)* Endofunctor trait — first-class C → C functor (#127)
- *(#122)* Dependability ontology — Avizienis-Laprie-Randell-Landwehr (2004) (#125)
- typed Vocabulary — OntologyName, ModulePath, structured Citation ([#111](https://github.com/i-am-logger/pr4xis/pull/111))
- compose API — runtime ontology composition via Korporator ([#103](https://github.com/i-am-logger/pr4xis/pull/103)) ([#108](https://github.com/i-am-logger/pr4xis/pull/108))
- Ontolex-Lemon, consciousness C1×C2, complete functor chain, Vocabulary API ([#88](https://github.com/i-am-logger/pr4xis/pull/88)) ([#104](https://github.com/i-am-logger/pr4xis/pull/104))
- define_ontology! being: clause + register all 108 ontologies ([#76](https://github.com/i-am-logger/pr4xis/pull/76)) ([#84](https://github.com/i-am-logger/pr4xis/pull/84))
- artifact_identity + data_provisioning — ontological external-data subsystem, no more LFS ([#71](https://github.com/i-am-logger/pr4xis/pull/71))
- staging ontology — Futamura's partial-evaluation framework as a meta-ontology ([#67](https://github.com/i-am-logger/pr4xis/pull/67))
- enforce ontology patterns — define_ontology! everywhere, 4851 tests

### Fixed

- *(#62)* address 11 copilot comments on consolidated Heim PR #143 (#144)

### Other

- release master ([#184](https://github.com/i-am-logger/pr4xis/pull/184))
- praxis tests give faster feedback and catch slowdowns earlier ([#185](https://github.com/i-am-logger/pr4xis/pull/185))
- release master ([#181](https://github.com/i-am-logger/pr4xis/pull/181))
- release master ([#178](https://github.com/i-am-logger/pr4xis/pull/178))
- complete validation overhaul — per-def trait sweep + rustdoc rot + mdBook ([#176](https://github.com/i-am-logger/pr4xis/pull/176))
- release master ([#175](https://github.com/i-am-logger/pr4xis/pull/175))
- release master ([#173](https://github.com/i-am-logger/pr4xis/pull/173))
- migrate pr4xis-domains to feat/logic-ontologies API ([#170](https://github.com/i-am-logger/pr4xis/pull/170))
- release master ([#158](https://github.com/i-am-logger/pr4xis/pull/158))
- Literature alignment: kinded morphisms, Arrow unification, Concept rename, macro cleanup ([#156](https://github.com/i-am-logger/pr4xis/pull/156))
- release master ([#151](https://github.com/i-am-logger/pr4xis/pull/151))
- release master ([#147](https://github.com/i-am-logger/pr4xis/pull/147))
- release master ([#145](https://github.com/i-am-logger/pr4xis/pull/145))
- release master ([#126](https://github.com/i-am-logger/pr4xis/pull/126))
- *(#98)* kinded-functor failures diagnosed — three distinct problems, none lax (#129)
- release master ([#115](https://github.com/i-am-logger/pr4xis/pull/115))
- *(#113)* batch 3 — final cognitive ontologies (lemon, consciousness, self_model) (#120)
- *(#113)* batch 2 — dialogue/pragmatics cluster (7 ontologies) (#119)
- *(#113)* migrate 18 ontologies to ontology! proc macro (#116)
- release master ([#105](https://github.com/i-am-logger/pr4xis/pull/105))
- release master ([#85](https://github.com/i-am-logger/pr4xis/pull/85))
- release master ([#73](https://github.com/i-am-logger/pr4xis/pull/73))
- *(#173)* per-ontology rollout for the 5 new HMI sub-ontologies (#69)
- release master ([#68](https://github.com/i-am-logger/pr4xis/pull/68))
- applied/theming/ → applied/hmi/{theming,surfaces,visualization,input,report,explorer}/ ([#66](https://github.com/i-am-logger/pr4xis/pull/66))
- rewrite + per-ontology rollout (#57, #55, #52, #46, #44) ([#63](https://github.com/i-am-logger/pr4xis/pull/63))
- release master ([#43](https://github.com/i-am-logger/pr4xis/pull/43))

## [0.21.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.20.0...pr4xis-domains-v0.21.0) (2026-05-30)


### Features

* **cli:** one-command corpus updates via `pr4xis update --lock` ([#183](https://github.com/i-am-logger/pr4xis/issues/183)) ([f13a5b5](https://github.com/i-am-logger/pr4xis/commit/f13a5b5f767d6611357d56e953bd385eca9fff28))


### Performance Improvements

* praxis tests give faster feedback and catch slowdowns earlier ([#185](https://github.com/i-am-logger/pr4xis/issues/185)) ([f66529b](https://github.com/i-am-logger/pr4xis/commit/f66529b9d205d55390a814741e59987b27ac57ca))

## [0.20.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.19.0...pr4xis-domains-v0.20.0) (2026-05-30)


### Features

* praxis + praxis-cli gain the registered-source mechanism (SOX 1514A, AIR21 42121) ([#179](https://github.com/i-am-logger/pr4xis/issues/179)) ([917981a](https://github.com/i-am-logger/pr4xis/commit/917981a0bc3051d87c06661ec973ef6cfec79e3a))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * pr4xis bumped from 0.15.0 to 0.16.0
  * dev-dependencies
    * pr4xis bumped from 0.15.0 to 0.16.0
  * build-dependencies
    * pr4xis bumped from 0.15.0 to 0.16.0

## [0.19.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.18.2...pr4xis-domains-v0.19.0) (2026-05-15)


### ⚠ BREAKING CHANGES

* **deps,ci:** pure-Rust crypto + always-latest devenv + PR-title gate ([#177](https://github.com/i-am-logger/pr4xis/issues/177))

### Features

* **deps,ci:** pure-Rust crypto + always-latest devenv + PR-title gate ([#177](https://github.com/i-am-logger/pr4xis/issues/177)) ([aee4c3a](https://github.com/i-am-logger/pr4xis/commit/aee4c3a77a4f3102d11ae9eb121420b01857f9f3))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * pr4xis bumped from 0.14.1 to 0.15.0

## [0.18.2](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.18.1...pr4xis-domains-v0.18.2) (2026-05-15)


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * pr4xis bumped from 0.14.0 to 0.14.1

## [0.18.1](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.18.0...pr4xis-domains-v0.18.1) (2026-05-15)


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * pr4xis bumped from 0.13.0 to 0.14.0

## [0.18.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.17.0...pr4xis-domains-v0.18.0) (2026-04-18)


### Features

* **#91:** pr4xis core + domains run no_std + alloc ([#157](https://github.com/i-am-logger/pr4xis/issues/157)) ([a62d317](https://github.com/i-am-logger/pr4xis/commit/a62d31770255fc8ca77d747280debd36674529f0))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * pr4xis bumped from 0.12.0 to 0.13.0

## [0.17.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.16.0...pr4xis-domains-v0.17.0) (2026-04-18)


### Features

* **#148:** Lemon meta on every structural entity — uniform registry for ontologies, axioms, functors, adjunctions, nat-trans ([#150](https://github.com/i-am-logger/pr4xis/issues/150)) ([f18bbc5](https://github.com/i-am-logger/pr4xis/commit/f18bbc5cbf116b6c4539b41df318c7f921e996cb))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * pr4xis bumped from 0.11.0 to 0.12.0

## [0.16.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.15.0...pr4xis-domains-v0.16.0) (2026-04-18)


### Features

* **#117:** MAPE-K ontology — chat pipeline's literature-grounded home ([#146](https://github.com/i-am-logger/pr4xis/issues/146)) ([de14e42](https://github.com/i-am-logger/pr4xis/commit/de14e42b83ef2539b9cab39a073bce145657be5e))

## [0.15.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.14.0...pr4xis-domains-v0.15.0) (2026-04-17)


### Features

* **#122:** Dependability ontology — Avizienis-Laprie-Randell-Landwehr (2004) ([#125](https://github.com/i-am-logger/pr4xis/issues/125)) ([29a99da](https://github.com/i-am-logger/pr4xis/commit/29a99daba4cbfbb6d6b7d2900e82697204aec01e))
* **#123:** Resilience ontology — Nygard/Brooker/Armstrong/Patterson ([#128](https://github.com/i-am-logger/pr4xis/issues/128)) ([cad2f89](https://github.com/i-am-logger/pr4xis/commit/cad2f89613cd80e9cf04d081a2c7c7b3405b92ac))
* **#124:** Endofunctor trait — first-class C → C functor ([#127](https://github.com/i-am-logger/pr4xis/issues/127)) ([d88b21d](https://github.com/i-am-logger/pr4xis/commit/d88b21d45372d33b82f76e5a6eaee885692dec19))
* **#130:** Category::Op&lt;C&gt; + empirical 4th failure mode discovery ([#133](https://github.com/i-am-logger/pr4xis/issues/133)) ([ceeb01d](https://github.com/i-am-logger/pr4xis/commit/ceeb01d81e4100d405def3423d6b487f1a8376df))
* **#131:** TerminalFunctor helper — reusable one-object collapse ([#134](https://github.com/i-am-logger/pr4xis/issues/134)) ([6d01283](https://github.com/i-am-logger/pr4xis/commit/6d01283bb914f29fc06faeedfea91d78648de31b))
* **#62:** Heim syntrometric lineage — consolidated stack ([#143](https://github.com/i-am-logger/pr4xis/issues/143)) ([21b1b81](https://github.com/i-am-logger/pr4xis/commit/21b1b81607861b0d8a6ecb15b9dc55a2288f0f99))
* **#62:** Heim syntrometry Phase 1 — lineage verified by functor laws ([#135](https://github.com/i-am-logger/pr4xis/issues/135)) ([599ef24](https://github.com/i-am-logger/pr4xis/commit/599ef2408e06acbbde7ebff85ef029e9a87e2ac8))
* add criterion benchmarks for all ontologies and chat pipeline ([9e8c5e1](https://github.com/i-am-logger/pr4xis/commit/9e8c5e1626c9717f5ecfb99662553bdf30201ec7))
* add durability, volatility, measurement, and benchmark ontologies ([e33ad6e](https://github.com/i-am-logger/pr4xis/commit/e33ad6e0b39567a4415b9320309caa496d50034e))
* add Instance ontology (Spivak) and SystemsToSchema functor ([750eac5](https://github.com/i-am-logger/pr4xis/commit/750eac5bb461e9d9ec166a2e853ec1189e6fe552))
* add Ontology Alignment and NLG pipeline ontologies ([46101fc](https://github.com/i-am-logger/pr4xis/commit/46101fc2770b6b14827327fdcf7e376396a630dd))
* artifact_identity + data_provisioning — ontological external-data subsystem, no more LFS ([#71](https://github.com/i-am-logger/pr4xis/issues/71)) ([94be051](https://github.com/i-am-logger/pr4xis/commit/94be051131eaf8dfb960b4bf91f8be8af07f0ac6))
* cognition ontologies — distinction, epistemics, metacognition ([b20e3c7](https://github.com/i-am-logger/pr4xis/commit/b20e3c788dc810d0e285c1cfabebd819fbe8bbe5))
* ColorSlot::key() — canonical theme file key names ([9d84544](https://github.com/i-am-logger/pr4xis/commit/9d845446a9b8e07425828ef9786c3f67beb246e3))
* Communication→Control + Diagnostics→Metacognition functors (proven) ([b7fa833](https://github.com/i-am-logger/pr4xis/commit/b7fa833af6bdc4a6ea69b8ffdb30bbb06a0dfd16))
* complete algebraic structure library — 7 new structures ([281f9e3](https://github.com/i-am-logger/pr4xis/commit/281f9e3087a97a988c33bd9deff27956a9cce759))
* complete scheme taxonomy — Vogix16, Ansi16, Base24, SchemeType ([d75e4f5](https://github.com/i-am-logger/pr4xis/commit/d75e4f51321cde732b83439d62d1981d07fd7266))
* compose API — runtime ontology composition via Korporator ([#103](https://github.com/i-am-logger/pr4xis/issues/103)) ([#108](https://github.com/i-am-logger/pr4xis/issues/108)) ([848d986](https://github.com/i-am-logger/pr4xis/commit/848d986457f82a758f3315c049063b53962ed00f))
* concurrency ontology — chess IS concurrent (proven via functor) ([38edc0c](https://github.com/i-am-logger/pr4xis/commit/38edc0ce4f1b2dc2a5b06afd602f1defabd52c6c))
* control systems ontology + foundational cybernetics papers ([3c1eab1](https://github.com/i-am-logger/pr4xis/commit/3c1eab1c26fd31028078d9d7c668e2fff0c70a24))
* define_ontology! being: clause + register all 108 ontologies ([#76](https://github.com/i-am-logger/pr4xis/issues/76)) ([#84](https://github.com/i-am-logger/pr4xis/issues/84)) ([1b27fc9](https://github.com/i-am-logger/pr4xis/commit/1b27fc974e1a4b018542ad4ea6ae57e3f4d9f561))
* define_ontology! clean API — concepts/is_a/has_a/causes/opposes + auto structural axioms ([366f284](https://github.com/i-am-logger/pr4xis/commit/366f28459f606ef56323910decd72b9be085e624))
* define_ontology! macro — generates Category + Taxonomy + Mereology + Causation + Opposition + OntologyMeta ([07a1b54](https://github.com/i-am-logger/pr4xis/commit/07a1b549ecac1d321d679bddc6dfcaee9cb14138))
* derive macros — #[derive(Entity)] + define_category! + define_dense_category! ([e598947](https://github.com/i-am-logger/pr4xis/commit/e598947d89ff36d1e4d84ac09ad1720915034483))
* dev-web serves chatbot at /, presentation at /decks/technical ([12ef79a](https://github.com/i-am-logger/pr4xis/commit/12ef79a376cb75ab75920295fddbf6269fe2db1a))
* Diagnostics ontology + TracedCategory (writer monad on categories) ([b3beba6](https://github.com/i-am-logger/pr4xis/commit/b3beba6b9e28c2426850fa88fea98a3133b8edea))
* Diagnostics→Control functor (FDI IS control — Gertler 1998, proven) ([42f9b27](https://github.com/i-am-logger/pr4xis/commit/42f9b27235b02251cb0262c84c169567e27445be))
* dialogue ontology + chatbot CLI — praxis can chat ([b62fd99](https://github.com/i-am-logger/pr4xis/commit/b62fd99d4243ef8c39bb99c506aa298cc5a6ccb7))
* docs/chat/ for GitHub Pages — presentation embeds live chatbot ([b4936c6](https://github.com/i-am-logger/pr4xis/commit/b4936c65d06a26595c2943c7bb6f690ed8ec3b69))
* DOLCE upper ontology, domain restructure, linguistics, systems thinking, codegen ([368c583](https://github.com/i-am-logger/pr4xis/commit/368c5835965915495ea43d1b5e3dcbc76b1a93e6))
* enforce ontology patterns — define_ontology! everywhere, 4851 tests ([63031ee](https://github.com/i-am-logger/pr4xis/commit/63031ee4bc8b96f874b9e3b0e192e881494265f0))
* English language ontology — 107k concepts, nanosecond queries ([bf5113f](https://github.com/i-am-logger/pr4xis/commit/bf5113ffbaa471c1d7247b4ab28b4bba78d56c5e))
* event-driven ontology — chess IS event-driven IS concurrent (proven) ([24bf058](https://github.com/i-am-logger/pr4xis/commit/24bf058eff868bf169f27e3e4e0c7633d42a339a))
* extend dialogue ontology with QUD, CommonGround, Intention, Repair ([9d782e0](https://github.com/i-am-logger/pr4xis/commit/9d782e07b7e62d8fef47bd1d40d73aa3c4d35ba1))
* extended sentence test suite, chart type selection fix ([f869c68](https://github.com/i-am-logger/pr4xis/commit/f869c684cab5af3734f6ca53169eb138fd3dfa97))
* F-algebra, MonoidalCategory, Optics, MonadTransformer (4 structures) ([22211e2](https://github.com/i-am-logger/pr4xis/commit/22211e2c6d9a71e29d126e066622a74d58d39948))
* function words as LMF data, extend LmfPos with closed-class types ([0cbffab](https://github.com/i-am-logger/pr4xis/commit/0cbffab9585a3e3f7c202a498e28d3f733c66db9))
* information ontology — what bits, bytes, references, and text ARE ([28be527](https://github.com/i-am-logger/pr4xis/commit/28be5273775a6278ff6fd3edf99612a33c938e78))
* integrate algebraic structures into reasoning + tracing ([5f9651d](https://github.com/i-am-logger/pr4xis/commit/5f9651d18238565f7b4f5915327da23e1dfde594))
* integrate Kleisli + anamorphism + Yoneda into causation reasoning ([393d94c](https://github.com/i-am-logger/pr4xis/commit/393d94cf10050d43ec7cd663833596c0bc3ce4b3))
* integrate ontologies via functors, wire into chatbot pipeline ([3527502](https://github.com/i-am-logger/pr4xis/commit/3527502c2c7dbf18c8f26553ddbca423f6af0ae6))
* integration tests with full WordNet — expose real failures honestly ([e444807](https://github.com/i-am-logger/pr4xis/commit/e444807e07fa326fa66128075cd44ca29a70a0ce))
* Lambek → Pregroup functor — ontology evolution proven ([b9b9efb](https://github.com/i-am-logger/pr4xis/commit/b9b9efb6a7cbab8d152c5c558ac57ff7124e2e7d))
* Lambek grammar — syntax as category, text understood through type reduction ([3c1054d](https://github.com/i-am-logger/pr4xis/commit/3c1054db5c53bfbd171cf5212a021d964cb9512d))
* Language trait, orthography, morphology, cached reasoning queries ([89daa5a](https://github.com/i-am-logger/pr4xis/commit/89daa5a45dcdf211e336dceb99503c9a9babda11))
* Language::pregroup_types — end-to-end pregroup pipeline through Language trait ([75d2967](https://github.com/i-am-logger/pr4xis/commit/75d296736b4216d4c741d92426cbf51c61333ff5))
* load WordNet verb frames for transitivity — no more defaults ([d8403b8](https://github.com/i-am-logger/pr4xis/commit/d8403b8cd8b981dacedb98da150fcc4a9d6b3ab7))
* math functions + sRGB color science + theming ontology ([deff009](https://github.com/i-am-logger/pr4xis/commit/deff0096e946d1cd72b293f2ee1bf55f89dd3642))
* merge sensor-fusion ontologies, add schema/storage/consistency ontologies, praxis-web ([a2fe629](https://github.com/i-am-logger/pr4xis/commit/a2fe629aedbe2363a33b76772e997d1558e14259))
* migrate ~65 Pattern B ontologies to define_dense_category! (-5325 lines) ([638f7a5](https://github.com/i-am-logger/pr4xis/commit/638f7a5e5f40cb2c5ea896d0a65cb6cc71a4913a))
* migrate 11 vogix ontologies into praxis theming (3117 tests) ([2b288a1](https://github.com/i-am-logger/pr4xis/commit/2b288a1dcf974cccbd08716e7b674aabaee55718))
* migrate 30 Pattern A ontologies to define_category! macro (-4404 lines) ([527ad7f](https://github.com/i-am-logger/pr4xis/commit/527ad7f0e773ff5b89ea05b5db9128756419bca4))
* migrate 41 ontologies to define_ontology! macro (-3163 lines) ([f261830](https://github.com/i-am-logger/pr4xis/commit/f2618306508c5e7745a903a946220965fb522541))
* migrate burp + pssst ontologies — 123 categories, 61 functors, 4707 tests ([3299cdc](https://github.com/i-am-logger/pr4xis/commit/3299cdcb7e068ba89cd215c641b66d3e3fed4120))
* migrate Ontology impls to structural + domain axiom split ([8a79323](https://github.com/i-am-logger/pr4xis/commit/8a793238c6d70160acc9cedf4e1341de9836386e))
* migrate remaining 5 biomedical Ontology impls to structural + domain split ([e3a7653](https://github.com/i-am-logger/pr4xis/commit/e3a76532fea89dc093df0ae6c36cec8905ef6090))
* Monoid + Writer monad + TracedCategory refactor (Moggi 1991, Mac Lane 1971) ([32b6292](https://github.com/i-am-logger/pr4xis/commit/32b6292239c77d0306b15aa2ad16271b1a48e60b))
* Montague functor — type-driven syntax-semantics interpretation ([5ce26d2](https://github.com/i-am-logger/pr4xis/commit/5ce26d224486d45d7c1c6d92a745d0c6e98595d9))
* NoisyChannel→Communication + DRT→Dialogue functors (proven) ([92c37a1](https://github.com/i-am-logger/pr4xis/commit/92c37a1d6159613afc22791846bb3b16e6829772))
* Ontolex-Lemon, consciousness C1×C2, complete functor chain, Vocabulary API ([#88](https://github.com/i-am-logger/pr4xis/issues/88)) ([#104](https://github.com/i-am-logger/pr4xis/issues/104)) ([d3a5a46](https://github.com/i-am-logger/pr4xis/commit/d3a5a46aca23292c85078390499b696c9bff3c0e))
* Ontology trait — structural + domain axioms merged via monoid ([32ac02b](https://github.com/i-am-logger/pr4xis/commit/32ac02b9ff557fb01d7c9bcc04c227fd07476992))
* pregroup grammar ontology — parsing as group algebra ([08881d8](https://github.com/i-am-logger/pr4xis/commit/08881d8d30c38fd2a55193a09175641614d6ad04))
* PregroupCategory — proper Category with proven laws ([b77c12f](https://github.com/i-am-logger/pr4xis/commit/b77c12f821ee4ae3a94ba9cb8055ab175fd51fb9))
* prop tests + functor connections across 15 ontologies (2934 tests, 18 functors) ([cf77284](https://github.com/i-am-logger/pr4xis/commit/cf772848478dd7929b0d959265369ac82674fb5e))
* proper ontology trace — each step reports what it did with status ([f7b1b7f](https://github.com/i-am-logger/pr4xis/commit/f7b1b7f7bfb5502e6c851236893e9d0a0d4a4241))
* proper ontology trace — each step reports what it did with status ([1605d7e](https://github.com/i-am-logger/pr4xis/commit/1605d7e94d7c1af2debce45ec7534257d9c67bd2))
* proper ontology trace — each step reports what it did with status ([20739ae](https://github.com/i-am-logger/pr4xis/commit/20739ae4f4b9e3926e6b22747add976f3e01249c))
* question grammar types + Q semantic domain in Lambek/Montague ([05d5ea9](https://github.com/i-am-logger/pr4xis/commit/05d5ea97c98abc6c375968000b833feb7ba14f05))
* Reader + State monads with property-based tests ([3d1b01e](https://github.com/i-am-logger/pr4xis/commit/3d1b01ef72d751e31a2829d9eac120d87c2ccdef))
* rename praxis → pr4xis across entire codebase ([5e971f7](https://github.com/i-am-logger/pr4xis/commit/5e971f77ac3041a5e35209216d09f41e55cf8a0d))
* restructure to academic hierarchy (DOLCE-aligned) ([44997fa](https://github.com/i-am-logger/pr4xis/commit/44997fae2ed61f693b592839cc8f27efb4cc35bc))
* Rgb::from_hex and to_hex for color parsing ([79b88cc](https://github.com/i-am-logger/pr4xis/commit/79b88cc9d5f96521065b62a5673eb4be9906bc60))
* rich taxonomy responses with path, definitions, and subtypes ([4bd0fc8](https://github.com/i-am-logger/pr4xis/commit/4bd0fc84bdf99a03ea7d7423cc1ad5f073f6d3cc))
* self-model ontology, CYK chart parser, adjunction, response generation ([0f67d8d](https://github.com/i-am-logger/pr4xis/commit/0f67d8d629065b3e76d3acd4567a03e9cc346c7e))
* speech production ontology (Levelt pipeline as category) ([b991418](https://github.com/i-am-logger/pr4xis/commit/b991418fad697aa8a2d06499bb7f81bb263daa58))
* staging ontology — Futamura's partial-evaluation framework as a meta-ontology ([#67](https://github.com/i-am-logger/pr4xis/issues/67)) ([7d824bb](https://github.com/i-am-logger/pr4xis/commit/7d824bbd59ab3ded3073a9b1780e9375aa27851e))
* SystemsToConcurrency functor — every system IS concurrent (proven) ([e53bcbb](https://github.com/i-am-logger/pr4xis/commit/e53bcbbffbd8fc255af169aa14f050f1397da9b1))
* SystemsToEvents functor — closes System ↔ EventDriven ↔ Concurrent triangle ([7bca79b](https://github.com/i-am-logger/pr4xis/commit/7bca79b4bc9fc8c5710a5a0c170b59a3ccf41ac8))
* trace functors — map pipeline steps to Diagnostics/PROV ontologies ([f3d234f](https://github.com/i-am-logger/pr4xis/commit/f3d234f81ad3ffc8458b25fc5280249f07658cba))
* trace shows Lambek notation (S[q]/NP) not Rust debug format ([3f24ff5](https://github.com/i-am-logger/pr4xis/commit/3f24ff5493ba958594ed72f9d4717936acfcb957))
* TraceSchema ontology — T(C) = El(C) + O_obs (Spivak 2012) ([2e0df62](https://github.com/i-am-logger/pr4xis/commit/2e0df6279843fb7bc9bfcca70bdf614f0cbb6db5))
* Traum grounding state machine as finite category ([00c811d](https://github.com/i-am-logger/pr4xis/commit/00c811d6ed75b72be3625c21b5d6e4b042b0a33a))
* Turing test benchmark — 18 questions, 3 pass, 15 need ontologies ([9c7c64e](https://github.com/i-am-logger/pr4xis/commit/9c7c64e18f96b6f497c170fac35efac2ab99bdac))
* typed Vocabulary — OntologyName, ModulePath, structured Citation ([#111](https://github.com/i-am-logger/pr4xis/issues/111)) ([a05b34c](https://github.com/i-am-logger/pr4xis/commit/a05b34c061a8c0c784a72c20a8afe1deec7fae7b))
* WordNet-LMF ontology — full 107k synset load in 3.8s ([904b69e](https://github.com/i-am-logger/pr4xis/commit/904b69e12f5c6baeef4fe405b8442f7c97df4a36))
* XML ontology, enhanced property tests, systems thinking completeness ([7e00e60](https://github.com/i-am-logger/pr4xis/commit/7e00e60311abc63022b0598b1f1b8ea4b3f8eabc))


### Bug Fixes

* **#62:** address 11 copilot comments on consolidated Heim PR [#143](https://github.com/i-am-logger/pr4xis/issues/143) ([#144](https://github.com/i-am-logger/pr4xis/issues/144)) ([ecad1c0](https://github.com/i-am-logger/pr4xis/commit/ecad1c0b7e79922a91f38c9a46a911c18b8b60f9))
* add WordNet XML (LFS) + tinted-schemes submodule for CI ([eaf6b97](https://github.com/i-am-logger/pr4xis/commit/eaf6b97d96c9ac85f5f3fc6b0bec848b4952786c))
* clarify Base16 has 16 slots, Base24 has 24 ([f876311](https://github.com/i-am-logger/pr4xis/commit/f87631199d432fedc7999eb086a28e856b84677f))
* clippy clean — no dead code, no unused imports, no stubs ([def3e3e](https://github.com/i-am-logger/pr4xis/commit/def3e3ef7ea816f16826184f6fa77c833d938df9))
* copula type from CCG research — question 'is X a Y' now parses to Q type ([2a05413](https://github.com/i-am-logger/pr4xis/commit/2a05413a9cdf8ccec1ffb98d46d8751c0e2830da))
* qualify kind refs in define_category! macro (avoid Identity ambiguity) + LOC badge ([4ab4d34](https://github.com/i-am-logger/pr4xis/commit/4ab4d34b2aab9a77d1082288a8bf616940e55e39))
* remove all hardcoded pronoun/noun matching from dialogue engine ([fac095f](https://github.com/i-am-logger/pr4xis/commit/fac095f6a20509360e2c6dddada83e599c72977d))
* remove hardcoded quit/exit — farewell detection through language lexicon ([1bb193b](https://github.com/i-am-logger/pr4xis/commit/1bb193bf68d6c16bfe609f839aec456d08bf0603))
* remove unused Category imports from test modules (clippy -D warnings) ([5d8a1f8](https://github.com/i-am-logger/pr4xis/commit/5d8a1f85dcd55869c2cd56b7ed1bf0f10b68c398))
* resolve all clippy warnings for strict CI ([79ff81b](https://github.com/i-am-logger/pr4xis/commit/79ff81ba0983283e738516dc8e0be55773add52d))
* skip data-dependent tests when WordNet/themes not available (CI) ([04834f6](https://github.com/i-am-logger/pr4xis/commit/04834f62a5385e4333d379ee61e8fa577908996d))
* taxonomy query works — 'is a dog a mammal' answered correctly ([27043ce](https://github.com/i-am-logger/pr4xis/commit/27043cec07a98102b1a72ba3d65c5bbc9207bca4))
* update release-please config for pr4xis rename + add version to path deps ([ff60744](https://github.com/i-am-logger/pr4xis/commit/ff60744ee9dbdd64d2a964b39c286253216e9a58))

## [0.14.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.13.0...pr4xis-domains-v0.14.0) (2026-04-17)


### Features

* **#122:** Dependability ontology — Avizienis-Laprie-Randell-Landwehr (2004) ([#125](https://github.com/i-am-logger/pr4xis/issues/125)) ([29a99da](https://github.com/i-am-logger/pr4xis/commit/29a99daba4cbfbb6d6b7d2900e82697204aec01e))
* **#123:** Resilience ontology — Nygard/Brooker/Armstrong/Patterson ([#128](https://github.com/i-am-logger/pr4xis/issues/128)) ([cad2f89](https://github.com/i-am-logger/pr4xis/commit/cad2f89613cd80e9cf04d081a2c7c7b3405b92ac))
* **#124:** Endofunctor trait — first-class C → C functor ([#127](https://github.com/i-am-logger/pr4xis/issues/127)) ([d88b21d](https://github.com/i-am-logger/pr4xis/commit/d88b21d45372d33b82f76e5a6eaee885692dec19))
* **#130:** Category::Op&lt;C&gt; + empirical 4th failure mode discovery ([#133](https://github.com/i-am-logger/pr4xis/issues/133)) ([ceeb01d](https://github.com/i-am-logger/pr4xis/commit/ceeb01d81e4100d405def3423d6b487f1a8376df))
* **#131:** TerminalFunctor helper — reusable one-object collapse ([#134](https://github.com/i-am-logger/pr4xis/issues/134)) ([6d01283](https://github.com/i-am-logger/pr4xis/commit/6d01283bb914f29fc06faeedfea91d78648de31b))
* **#62:** Heim syntrometric lineage — consolidated stack ([#143](https://github.com/i-am-logger/pr4xis/issues/143)) ([21b1b81](https://github.com/i-am-logger/pr4xis/commit/21b1b81607861b0d8a6ecb15b9dc55a2288f0f99))
* **#62:** Heim syntrometry Phase 1 — lineage verified by functor laws ([#135](https://github.com/i-am-logger/pr4xis/issues/135)) ([599ef24](https://github.com/i-am-logger/pr4xis/commit/599ef2408e06acbbde7ebff85ef029e9a87e2ac8))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * pr4xis bumped from 0.10.0 to 0.11.0

## [0.13.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.12.0...pr4xis-domains-v0.13.0) (2026-04-17)


### Features

* compose API — runtime ontology composition via Korporator ([#103](https://github.com/i-am-logger/pr4xis/issues/103)) ([#108](https://github.com/i-am-logger/pr4xis/issues/108)) ([848d986](https://github.com/i-am-logger/pr4xis/commit/848d986457f82a758f3315c049063b53962ed00f))
* typed Vocabulary — OntologyName, ModulePath, structured Citation ([#111](https://github.com/i-am-logger/pr4xis/issues/111)) ([a05b34c](https://github.com/i-am-logger/pr4xis/commit/a05b34c061a8c0c784a72c20a8afe1deec7fae7b))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * pr4xis bumped from 0.9.0 to 0.10.0

## [0.12.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.11.0...pr4xis-domains-v0.12.0) (2026-04-16)


### Features

* Ontolex-Lemon, consciousness C1×C2, complete functor chain, Vocabulary API ([#88](https://github.com/i-am-logger/pr4xis/issues/88)) ([#104](https://github.com/i-am-logger/pr4xis/issues/104)) ([d3a5a46](https://github.com/i-am-logger/pr4xis/commit/d3a5a46aca23292c85078390499b696c9bff3c0e))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * pr4xis bumped from 0.8.0 to 0.9.0

## [0.11.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.10.0...pr4xis-domains-v0.11.0) (2026-04-16)


### Features

* define_ontology! being: clause + register all 108 ontologies ([#76](https://github.com/i-am-logger/pr4xis/issues/76)) ([#84](https://github.com/i-am-logger/pr4xis/issues/84)) ([1b27fc9](https://github.com/i-am-logger/pr4xis/commit/1b27fc974e1a4b018542ad4ea6ae57e3f4d9f561))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * pr4xis bumped from 0.7.0 to 0.8.0

## [0.10.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.9.0...pr4xis-domains-v0.10.0) (2026-04-16)


### Features

* artifact_identity + data_provisioning — ontological external-data subsystem, no more LFS ([#71](https://github.com/i-am-logger/pr4xis/issues/71)) ([94be051](https://github.com/i-am-logger/pr4xis/commit/94be051131eaf8dfb960b4bf91f8be8af07f0ac6))

## [0.9.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.8.0...pr4xis-domains-v0.9.0) (2026-04-15)


### Features

* staging ontology — Futamura's partial-evaluation framework as a meta-ontology ([#67](https://github.com/i-am-logger/pr4xis/issues/67)) ([7d824bb](https://github.com/i-am-logger/pr4xis/commit/7d824bbd59ab3ded3073a9b1780e9375aa27851e))

## [0.8.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.7.0...pr4xis-domains-v0.8.0) (2026-04-13)


### Features

* enforce ontology patterns — define_ontology! everywhere, 4851 tests ([63031ee](https://github.com/i-am-logger/pr4xis/commit/63031ee4bc8b96f874b9e3b0e192e881494265f0))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * pr4xis bumped from 0.6.0 to 0.7.0

## [0.7.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.6.0...pr4xis-domains-v0.7.0) (2026-04-12)


### Features

* complete algebraic structure library — 7 new structures ([281f9e3](https://github.com/i-am-logger/pr4xis/commit/281f9e3087a97a988c33bd9deff27956a9cce759))
* define_ontology! clean API — concepts/is_a/has_a/causes/opposes + auto structural axioms ([366f284](https://github.com/i-am-logger/pr4xis/commit/366f28459f606ef56323910decd72b9be085e624))
* define_ontology! macro — generates Category + Taxonomy + Mereology + Causation + Opposition + OntologyMeta ([07a1b54](https://github.com/i-am-logger/pr4xis/commit/07a1b549ecac1d321d679bddc6dfcaee9cb14138))
* derive macros — #[derive(Entity)] + define_category! + define_dense_category! ([e598947](https://github.com/i-am-logger/pr4xis/commit/e598947d89ff36d1e4d84ac09ad1720915034483))
* F-algebra, MonoidalCategory, Optics, MonadTransformer (4 structures) ([22211e2](https://github.com/i-am-logger/pr4xis/commit/22211e2c6d9a71e29d126e066622a74d58d39948))
* integrate algebraic structures into reasoning + tracing ([5f9651d](https://github.com/i-am-logger/pr4xis/commit/5f9651d18238565f7b4f5915327da23e1dfde594))
* integrate Kleisli + anamorphism + Yoneda into causation reasoning ([393d94c](https://github.com/i-am-logger/pr4xis/commit/393d94cf10050d43ec7cd663833596c0bc3ce4b3))
* migrate ~65 Pattern B ontologies to define_dense_category! (-5325 lines) ([638f7a5](https://github.com/i-am-logger/pr4xis/commit/638f7a5e5f40cb2c5ea896d0a65cb6cc71a4913a))
* migrate 30 Pattern A ontologies to define_category! macro (-4404 lines) ([527ad7f](https://github.com/i-am-logger/pr4xis/commit/527ad7f0e773ff5b89ea05b5db9128756419bca4))
* migrate 41 ontologies to define_ontology! macro (-3163 lines) ([f261830](https://github.com/i-am-logger/pr4xis/commit/f2618306508c5e7745a903a946220965fb522541))
* migrate Ontology impls to structural + domain axiom split ([8a79323](https://github.com/i-am-logger/pr4xis/commit/8a793238c6d70160acc9cedf4e1341de9836386e))
* migrate remaining 5 biomedical Ontology impls to structural + domain split ([e3a7653](https://github.com/i-am-logger/pr4xis/commit/e3a76532fea89dc093df0ae6c36cec8905ef6090))
* Monoid + Writer monad + TracedCategory refactor (Moggi 1991, Mac Lane 1971) ([32b6292](https://github.com/i-am-logger/pr4xis/commit/32b6292239c77d0306b15aa2ad16271b1a48e60b))
* Ontology trait — structural + domain axioms merged via monoid ([32ac02b](https://github.com/i-am-logger/pr4xis/commit/32ac02b9ff557fb01d7c9bcc04c227fd07476992))
* Reader + State monads with property-based tests ([3d1b01e](https://github.com/i-am-logger/pr4xis/commit/3d1b01ef72d751e31a2829d9eac120d87c2ccdef))
* restructure to academic hierarchy (DOLCE-aligned) ([44997fa](https://github.com/i-am-logger/pr4xis/commit/44997fae2ed61f693b592839cc8f27efb4cc35bc))


### Bug Fixes

* clippy clean — no dead code, no unused imports, no stubs ([def3e3e](https://github.com/i-am-logger/pr4xis/commit/def3e3ef7ea816f16826184f6fa77c833d938df9))
* qualify kind refs in define_category! macro (avoid Identity ambiguity) + LOC badge ([4ab4d34](https://github.com/i-am-logger/pr4xis/commit/4ab4d34b2aab9a77d1082288a8bf616940e55e39))
* remove unused Category imports from test modules (clippy -D warnings) ([5d8a1f8](https://github.com/i-am-logger/pr4xis/commit/5d8a1f85dcd55869c2cd56b7ed1bf0f10b68c398))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * pr4xis bumped from 0.5.0 to 0.6.0

## [0.6.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.5.0...pr4xis-domains-v0.6.0) (2026-04-12)


### Features

* add criterion benchmarks for all ontologies and chat pipeline ([9e8c5e1](https://github.com/i-am-logger/pr4xis/commit/9e8c5e1626c9717f5ecfb99662553bdf30201ec7))
* add durability, volatility, measurement, and benchmark ontologies ([e33ad6e](https://github.com/i-am-logger/pr4xis/commit/e33ad6e0b39567a4415b9320309caa496d50034e))
* add Instance ontology (Spivak) and SystemsToSchema functor ([750eac5](https://github.com/i-am-logger/pr4xis/commit/750eac5bb461e9d9ec166a2e853ec1189e6fe552))
* add Ontology Alignment and NLG pipeline ontologies ([46101fc](https://github.com/i-am-logger/pr4xis/commit/46101fc2770b6b14827327fdcf7e376396a630dd))
* cognition ontologies — distinction, epistemics, metacognition ([b20e3c7](https://github.com/i-am-logger/pr4xis/commit/b20e3c788dc810d0e285c1cfabebd819fbe8bbe5))
* ColorSlot::key() — canonical theme file key names ([9d84544](https://github.com/i-am-logger/pr4xis/commit/9d845446a9b8e07425828ef9786c3f67beb246e3))
* Communication→Control + Diagnostics→Metacognition functors (proven) ([b7fa833](https://github.com/i-am-logger/pr4xis/commit/b7fa833af6bdc4a6ea69b8ffdb30bbb06a0dfd16))
* complete scheme taxonomy — Vogix16, Ansi16, Base24, SchemeType ([d75e4f5](https://github.com/i-am-logger/pr4xis/commit/d75e4f51321cde732b83439d62d1981d07fd7266))
* concurrency ontology — chess IS concurrent (proven via functor) ([38edc0c](https://github.com/i-am-logger/pr4xis/commit/38edc0ce4f1b2dc2a5b06afd602f1defabd52c6c))
* control systems ontology + foundational cybernetics papers ([3c1eab1](https://github.com/i-am-logger/pr4xis/commit/3c1eab1c26fd31028078d9d7c668e2fff0c70a24))
* dev-web serves chatbot at /, presentation at /decks/technical ([12ef79a](https://github.com/i-am-logger/pr4xis/commit/12ef79a376cb75ab75920295fddbf6269fe2db1a))
* Diagnostics ontology + TracedCategory (writer monad on categories) ([b3beba6](https://github.com/i-am-logger/pr4xis/commit/b3beba6b9e28c2426850fa88fea98a3133b8edea))
* Diagnostics→Control functor (FDI IS control — Gertler 1998, proven) ([42f9b27](https://github.com/i-am-logger/pr4xis/commit/42f9b27235b02251cb0262c84c169567e27445be))
* dialogue ontology + chatbot CLI — praxis can chat ([b62fd99](https://github.com/i-am-logger/pr4xis/commit/b62fd99d4243ef8c39bb99c506aa298cc5a6ccb7))
* docs/chat/ for GitHub Pages — presentation embeds live chatbot ([b4936c6](https://github.com/i-am-logger/pr4xis/commit/b4936c65d06a26595c2943c7bb6f690ed8ec3b69))
* DOLCE upper ontology, domain restructure, linguistics, systems thinking, codegen ([368c583](https://github.com/i-am-logger/pr4xis/commit/368c5835965915495ea43d1b5e3dcbc76b1a93e6))
* English language ontology — 107k concepts, nanosecond queries ([bf5113f](https://github.com/i-am-logger/pr4xis/commit/bf5113ffbaa471c1d7247b4ab28b4bba78d56c5e))
* event-driven ontology — chess IS event-driven IS concurrent (proven) ([24bf058](https://github.com/i-am-logger/pr4xis/commit/24bf058eff868bf169f27e3e4e0c7633d42a339a))
* extend dialogue ontology with QUD, CommonGround, Intention, Repair ([9d782e0](https://github.com/i-am-logger/pr4xis/commit/9d782e07b7e62d8fef47bd1d40d73aa3c4d35ba1))
* extended sentence test suite, chart type selection fix ([f869c68](https://github.com/i-am-logger/pr4xis/commit/f869c684cab5af3734f6ca53169eb138fd3dfa97))
* function words as LMF data, extend LmfPos with closed-class types ([0cbffab](https://github.com/i-am-logger/pr4xis/commit/0cbffab9585a3e3f7c202a498e28d3f733c66db9))
* information ontology — what bits, bytes, references, and text ARE ([28be527](https://github.com/i-am-logger/pr4xis/commit/28be5273775a6278ff6fd3edf99612a33c938e78))
* integrate ontologies via functors, wire into chatbot pipeline ([3527502](https://github.com/i-am-logger/pr4xis/commit/3527502c2c7dbf18c8f26553ddbca423f6af0ae6))
* integration tests with full WordNet — expose real failures honestly ([e444807](https://github.com/i-am-logger/pr4xis/commit/e444807e07fa326fa66128075cd44ca29a70a0ce))
* Lambek → Pregroup functor — ontology evolution proven ([b9b9efb](https://github.com/i-am-logger/pr4xis/commit/b9b9efb6a7cbab8d152c5c558ac57ff7124e2e7d))
* Lambek grammar — syntax as category, text understood through type reduction ([3c1054d](https://github.com/i-am-logger/pr4xis/commit/3c1054db5c53bfbd171cf5212a021d964cb9512d))
* Language trait, orthography, morphology, cached reasoning queries ([89daa5a](https://github.com/i-am-logger/pr4xis/commit/89daa5a45dcdf211e336dceb99503c9a9babda11))
* Language::pregroup_types — end-to-end pregroup pipeline through Language trait ([75d2967](https://github.com/i-am-logger/pr4xis/commit/75d296736b4216d4c741d92426cbf51c61333ff5))
* load WordNet verb frames for transitivity — no more defaults ([d8403b8](https://github.com/i-am-logger/pr4xis/commit/d8403b8cd8b981dacedb98da150fcc4a9d6b3ab7))
* math functions + sRGB color science + theming ontology ([deff009](https://github.com/i-am-logger/pr4xis/commit/deff0096e946d1cd72b293f2ee1bf55f89dd3642))
* merge sensor-fusion ontologies, add schema/storage/consistency ontologies, praxis-web ([a2fe629](https://github.com/i-am-logger/pr4xis/commit/a2fe629aedbe2363a33b76772e997d1558e14259))
* migrate 11 vogix ontologies into praxis theming (3117 tests) ([2b288a1](https://github.com/i-am-logger/pr4xis/commit/2b288a1dcf974cccbd08716e7b674aabaee55718))
* migrate burp + pssst ontologies — 123 categories, 61 functors, 4707 tests ([3299cdc](https://github.com/i-am-logger/pr4xis/commit/3299cdcb7e068ba89cd215c641b66d3e3fed4120))
* Montague functor — type-driven syntax-semantics interpretation ([5ce26d2](https://github.com/i-am-logger/pr4xis/commit/5ce26d224486d45d7c1c6d92a745d0c6e98595d9))
* NoisyChannel→Communication + DRT→Dialogue functors (proven) ([92c37a1](https://github.com/i-am-logger/pr4xis/commit/92c37a1d6159613afc22791846bb3b16e6829772))
* pregroup grammar ontology — parsing as group algebra ([08881d8](https://github.com/i-am-logger/pr4xis/commit/08881d8d30c38fd2a55193a09175641614d6ad04))
* PregroupCategory — proper Category with proven laws ([b77c12f](https://github.com/i-am-logger/pr4xis/commit/b77c12f821ee4ae3a94ba9cb8055ab175fd51fb9))
* prop tests + functor connections across 15 ontologies (2934 tests, 18 functors) ([cf77284](https://github.com/i-am-logger/pr4xis/commit/cf772848478dd7929b0d959265369ac82674fb5e))
* proper ontology trace — each step reports what it did with status ([f7b1b7f](https://github.com/i-am-logger/pr4xis/commit/f7b1b7f7bfb5502e6c851236893e9d0a0d4a4241))
* proper ontology trace — each step reports what it did with status ([1605d7e](https://github.com/i-am-logger/pr4xis/commit/1605d7e94d7c1af2debce45ec7534257d9c67bd2))
* proper ontology trace — each step reports what it did with status ([20739ae](https://github.com/i-am-logger/pr4xis/commit/20739ae4f4b9e3926e6b22747add976f3e01249c))
* question grammar types + Q semantic domain in Lambek/Montague ([05d5ea9](https://github.com/i-am-logger/pr4xis/commit/05d5ea97c98abc6c375968000b833feb7ba14f05))
* rename praxis → pr4xis across entire codebase ([5e971f7](https://github.com/i-am-logger/pr4xis/commit/5e971f77ac3041a5e35209216d09f41e55cf8a0d))
* Rgb::from_hex and to_hex for color parsing ([79b88cc](https://github.com/i-am-logger/pr4xis/commit/79b88cc9d5f96521065b62a5673eb4be9906bc60))
* rich taxonomy responses with path, definitions, and subtypes ([4bd0fc8](https://github.com/i-am-logger/pr4xis/commit/4bd0fc84bdf99a03ea7d7423cc1ad5f073f6d3cc))
* self-model ontology, CYK chart parser, adjunction, response generation ([0f67d8d](https://github.com/i-am-logger/pr4xis/commit/0f67d8d629065b3e76d3acd4567a03e9cc346c7e))
* speech production ontology (Levelt pipeline as category) ([b991418](https://github.com/i-am-logger/pr4xis/commit/b991418fad697aa8a2d06499bb7f81bb263daa58))
* SystemsToConcurrency functor — every system IS concurrent (proven) ([e53bcbb](https://github.com/i-am-logger/pr4xis/commit/e53bcbbffbd8fc255af169aa14f050f1397da9b1))
* SystemsToEvents functor — closes System ↔ EventDriven ↔ Concurrent triangle ([7bca79b](https://github.com/i-am-logger/pr4xis/commit/7bca79b4bc9fc8c5710a5a0c170b59a3ccf41ac8))
* trace functors — map pipeline steps to Diagnostics/PROV ontologies ([f3d234f](https://github.com/i-am-logger/pr4xis/commit/f3d234f81ad3ffc8458b25fc5280249f07658cba))
* trace shows Lambek notation (S[q]/NP) not Rust debug format ([3f24ff5](https://github.com/i-am-logger/pr4xis/commit/3f24ff5493ba958594ed72f9d4717936acfcb957))
* TraceSchema ontology — T(C) = El(C) + O_obs (Spivak 2012) ([2e0df62](https://github.com/i-am-logger/pr4xis/commit/2e0df6279843fb7bc9bfcca70bdf614f0cbb6db5))
* Traum grounding state machine as finite category ([00c811d](https://github.com/i-am-logger/pr4xis/commit/00c811d6ed75b72be3625c21b5d6e4b042b0a33a))
* Turing test benchmark — 18 questions, 3 pass, 15 need ontologies ([9c7c64e](https://github.com/i-am-logger/pr4xis/commit/9c7c64e18f96b6f497c170fac35efac2ab99bdac))
* WordNet-LMF ontology — full 107k synset load in 3.8s ([904b69e](https://github.com/i-am-logger/pr4xis/commit/904b69e12f5c6baeef4fe405b8442f7c97df4a36))
* XML ontology, enhanced property tests, systems thinking completeness ([7e00e60](https://github.com/i-am-logger/pr4xis/commit/7e00e60311abc63022b0598b1f1b8ea4b3f8eabc))


### Bug Fixes

* add WordNet XML (LFS) + tinted-schemes submodule for CI ([eaf6b97](https://github.com/i-am-logger/pr4xis/commit/eaf6b97d96c9ac85f5f3fc6b0bec848b4952786c))
* clarify Base16 has 16 slots, Base24 has 24 ([f876311](https://github.com/i-am-logger/pr4xis/commit/f87631199d432fedc7999eb086a28e856b84677f))
* copula type from CCG research — question 'is X a Y' now parses to Q type ([2a05413](https://github.com/i-am-logger/pr4xis/commit/2a05413a9cdf8ccec1ffb98d46d8751c0e2830da))
* remove all hardcoded pronoun/noun matching from dialogue engine ([fac095f](https://github.com/i-am-logger/pr4xis/commit/fac095f6a20509360e2c6dddada83e599c72977d))
* remove hardcoded quit/exit — farewell detection through language lexicon ([1bb193b](https://github.com/i-am-logger/pr4xis/commit/1bb193bf68d6c16bfe609f839aec456d08bf0603))
* resolve all clippy warnings for strict CI ([79ff81b](https://github.com/i-am-logger/pr4xis/commit/79ff81ba0983283e738516dc8e0be55773add52d))
* skip data-dependent tests when WordNet/themes not available (CI) ([04834f6](https://github.com/i-am-logger/pr4xis/commit/04834f62a5385e4333d379ee61e8fa577908996d))
* taxonomy query works — 'is a dog a mammal' answered correctly ([27043ce](https://github.com/i-am-logger/pr4xis/commit/27043cec07a98102b1a72ba3d65c5bbc9207bca4))
* update release-please config for pr4xis rename + add version to path deps ([ff60744](https://github.com/i-am-logger/pr4xis/commit/ff60744ee9dbdd64d2a964b39c286253216e9a58))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * pr4xis bumped from 0.4.0 to 0.5.0

## [0.5.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-domains-v0.4.0...pr4xis-domains-v0.5.0) (2026-04-12)


### Features

* add criterion benchmarks for all ontologies and chat pipeline ([9e8c5e1](https://github.com/i-am-logger/pr4xis/commit/9e8c5e1626c9717f5ecfb99662553bdf30201ec7))
* add durability, volatility, measurement, and benchmark ontologies ([e33ad6e](https://github.com/i-am-logger/pr4xis/commit/e33ad6e0b39567a4415b9320309caa496d50034e))
* add Instance ontology (Spivak) and SystemsToSchema functor ([750eac5](https://github.com/i-am-logger/pr4xis/commit/750eac5bb461e9d9ec166a2e853ec1189e6fe552))
* add Ontology Alignment and NLG pipeline ontologies ([46101fc](https://github.com/i-am-logger/pr4xis/commit/46101fc2770b6b14827327fdcf7e376396a630dd))
* cognition ontologies — distinction, epistemics, metacognition ([b20e3c7](https://github.com/i-am-logger/pr4xis/commit/b20e3c788dc810d0e285c1cfabebd819fbe8bbe5))
* ColorSlot::key() — canonical theme file key names ([9d84544](https://github.com/i-am-logger/pr4xis/commit/9d845446a9b8e07425828ef9786c3f67beb246e3))
* Communication→Control + Diagnostics→Metacognition functors (proven) ([b7fa833](https://github.com/i-am-logger/pr4xis/commit/b7fa833af6bdc4a6ea69b8ffdb30bbb06a0dfd16))
* complete scheme taxonomy — Vogix16, Ansi16, Base24, SchemeType ([d75e4f5](https://github.com/i-am-logger/pr4xis/commit/d75e4f51321cde732b83439d62d1981d07fd7266))
* concurrency ontology — chess IS concurrent (proven via functor) ([38edc0c](https://github.com/i-am-logger/pr4xis/commit/38edc0ce4f1b2dc2a5b06afd602f1defabd52c6c))
* control systems ontology + foundational cybernetics papers ([3c1eab1](https://github.com/i-am-logger/pr4xis/commit/3c1eab1c26fd31028078d9d7c668e2fff0c70a24))
* Diagnostics ontology + TracedCategory (writer monad on categories) ([b3beba6](https://github.com/i-am-logger/pr4xis/commit/b3beba6b9e28c2426850fa88fea98a3133b8edea))
* Diagnostics→Control functor (FDI IS control — Gertler 1998, proven) ([42f9b27](https://github.com/i-am-logger/pr4xis/commit/42f9b27235b02251cb0262c84c169567e27445be))
* dialogue ontology + chatbot CLI — praxis can chat ([b62fd99](https://github.com/i-am-logger/pr4xis/commit/b62fd99d4243ef8c39bb99c506aa298cc5a6ccb7))
* docs/chat/ for GitHub Pages — presentation embeds live chatbot ([b4936c6](https://github.com/i-am-logger/pr4xis/commit/b4936c65d06a26595c2943c7bb6f690ed8ec3b69))
* DOLCE upper ontology, domain restructure, linguistics, systems thinking, codegen ([368c583](https://github.com/i-am-logger/pr4xis/commit/368c5835965915495ea43d1b5e3dcbc76b1a93e6))
* English language ontology — 107k concepts, nanosecond queries ([bf5113f](https://github.com/i-am-logger/pr4xis/commit/bf5113ffbaa471c1d7247b4ab28b4bba78d56c5e))
* event-driven ontology — chess IS event-driven IS concurrent (proven) ([24bf058](https://github.com/i-am-logger/pr4xis/commit/24bf058eff868bf169f27e3e4e0c7633d42a339a))
* extend dialogue ontology with QUD, CommonGround, Intention, Repair ([9d782e0](https://github.com/i-am-logger/pr4xis/commit/9d782e07b7e62d8fef47bd1d40d73aa3c4d35ba1))
* extended sentence test suite, chart type selection fix ([f869c68](https://github.com/i-am-logger/pr4xis/commit/f869c684cab5af3734f6ca53169eb138fd3dfa97))
* function words as LMF data, extend LmfPos with closed-class types ([0cbffab](https://github.com/i-am-logger/pr4xis/commit/0cbffab9585a3e3f7c202a498e28d3f733c66db9))
* information ontology — what bits, bytes, references, and text ARE ([28be527](https://github.com/i-am-logger/pr4xis/commit/28be5273775a6278ff6fd3edf99612a33c938e78))
* integrate ontologies via functors, wire into chatbot pipeline ([3527502](https://github.com/i-am-logger/pr4xis/commit/3527502c2c7dbf18c8f26553ddbca423f6af0ae6))
* integration tests with full WordNet — expose real failures honestly ([e444807](https://github.com/i-am-logger/pr4xis/commit/e444807e07fa326fa66128075cd44ca29a70a0ce))
* Lambek → Pregroup functor — ontology evolution proven ([b9b9efb](https://github.com/i-am-logger/pr4xis/commit/b9b9efb6a7cbab8d152c5c558ac57ff7124e2e7d))
* Lambek grammar — syntax as category, text understood through type reduction ([3c1054d](https://github.com/i-am-logger/pr4xis/commit/3c1054db5c53bfbd171cf5212a021d964cb9512d))
* Language trait, orthography, morphology, cached reasoning queries ([89daa5a](https://github.com/i-am-logger/pr4xis/commit/89daa5a45dcdf211e336dceb99503c9a9babda11))
* Language::pregroup_types — end-to-end pregroup pipeline through Language trait ([75d2967](https://github.com/i-am-logger/pr4xis/commit/75d296736b4216d4c741d92426cbf51c61333ff5))
* load WordNet verb frames for transitivity — no more defaults ([d8403b8](https://github.com/i-am-logger/pr4xis/commit/d8403b8cd8b981dacedb98da150fcc4a9d6b3ab7))
* math functions + sRGB color science + theming ontology ([deff009](https://github.com/i-am-logger/pr4xis/commit/deff0096e946d1cd72b293f2ee1bf55f89dd3642))
* merge sensor-fusion ontologies, add schema/storage/consistency ontologies, praxis-web ([a2fe629](https://github.com/i-am-logger/pr4xis/commit/a2fe629aedbe2363a33b76772e997d1558e14259))
* migrate 11 vogix ontologies into praxis theming (3117 tests) ([2b288a1](https://github.com/i-am-logger/pr4xis/commit/2b288a1dcf974cccbd08716e7b674aabaee55718))
* migrate burp + pssst ontologies — 123 categories, 61 functors, 4707 tests ([3299cdc](https://github.com/i-am-logger/pr4xis/commit/3299cdcb7e068ba89cd215c641b66d3e3fed4120))
* Montague functor — type-driven syntax-semantics interpretation ([5ce26d2](https://github.com/i-am-logger/pr4xis/commit/5ce26d224486d45d7c1c6d92a745d0c6e98595d9))
* NoisyChannel→Communication + DRT→Dialogue functors (proven) ([92c37a1](https://github.com/i-am-logger/pr4xis/commit/92c37a1d6159613afc22791846bb3b16e6829772))
* pregroup grammar ontology — parsing as group algebra ([08881d8](https://github.com/i-am-logger/pr4xis/commit/08881d8d30c38fd2a55193a09175641614d6ad04))
* PregroupCategory — proper Category with proven laws ([b77c12f](https://github.com/i-am-logger/pr4xis/commit/b77c12f821ee4ae3a94ba9cb8055ab175fd51fb9))
* prop tests + functor connections across 15 ontologies (2934 tests, 18 functors) ([cf77284](https://github.com/i-am-logger/pr4xis/commit/cf772848478dd7929b0d959265369ac82674fb5e))
* proper ontology trace — each step reports what it did with status ([f7b1b7f](https://github.com/i-am-logger/pr4xis/commit/f7b1b7f7bfb5502e6c851236893e9d0a0d4a4241))
* proper ontology trace — each step reports what it did with status ([1605d7e](https://github.com/i-am-logger/pr4xis/commit/1605d7e94d7c1af2debce45ec7534257d9c67bd2))
* proper ontology trace — each step reports what it did with status ([20739ae](https://github.com/i-am-logger/pr4xis/commit/20739ae4f4b9e3926e6b22747add976f3e01249c))
* question grammar types + Q semantic domain in Lambek/Montague ([05d5ea9](https://github.com/i-am-logger/pr4xis/commit/05d5ea97c98abc6c375968000b833feb7ba14f05))
* rename praxis → pr4xis across entire codebase ([5e971f7](https://github.com/i-am-logger/pr4xis/commit/5e971f77ac3041a5e35209216d09f41e55cf8a0d))
* Rgb::from_hex and to_hex for color parsing ([79b88cc](https://github.com/i-am-logger/pr4xis/commit/79b88cc9d5f96521065b62a5673eb4be9906bc60))
* rich taxonomy responses with path, definitions, and subtypes ([4bd0fc8](https://github.com/i-am-logger/pr4xis/commit/4bd0fc84bdf99a03ea7d7423cc1ad5f073f6d3cc))
* self-model ontology, CYK chart parser, adjunction, response generation ([0f67d8d](https://github.com/i-am-logger/pr4xis/commit/0f67d8d629065b3e76d3acd4567a03e9cc346c7e))
* speech production ontology (Levelt pipeline as category) ([b991418](https://github.com/i-am-logger/pr4xis/commit/b991418fad697aa8a2d06499bb7f81bb263daa58))
* SystemsToConcurrency functor — every system IS concurrent (proven) ([e53bcbb](https://github.com/i-am-logger/pr4xis/commit/e53bcbbffbd8fc255af169aa14f050f1397da9b1))
* SystemsToEvents functor — closes System ↔ EventDriven ↔ Concurrent triangle ([7bca79b](https://github.com/i-am-logger/pr4xis/commit/7bca79b4bc9fc8c5710a5a0c170b59a3ccf41ac8))
* trace functors — map pipeline steps to Diagnostics/PROV ontologies ([f3d234f](https://github.com/i-am-logger/pr4xis/commit/f3d234f81ad3ffc8458b25fc5280249f07658cba))
* trace shows Lambek notation (S[q]/NP) not Rust debug format ([3f24ff5](https://github.com/i-am-logger/pr4xis/commit/3f24ff5493ba958594ed72f9d4717936acfcb957))
* TraceSchema ontology — T(C) = El(C) + O_obs (Spivak 2012) ([2e0df62](https://github.com/i-am-logger/pr4xis/commit/2e0df6279843fb7bc9bfcca70bdf614f0cbb6db5))
* Traum grounding state machine as finite category ([00c811d](https://github.com/i-am-logger/pr4xis/commit/00c811d6ed75b72be3625c21b5d6e4b042b0a33a))
* Turing test benchmark — 18 questions, 3 pass, 15 need ontologies ([9c7c64e](https://github.com/i-am-logger/pr4xis/commit/9c7c64e18f96b6f497c170fac35efac2ab99bdac))
* WordNet-LMF ontology — full 107k synset load in 3.8s ([904b69e](https://github.com/i-am-logger/pr4xis/commit/904b69e12f5c6baeef4fe405b8442f7c97df4a36))
* XML ontology, enhanced property tests, systems thinking completeness ([7e00e60](https://github.com/i-am-logger/pr4xis/commit/7e00e60311abc63022b0598b1f1b8ea4b3f8eabc))


### Bug Fixes

* add WordNet XML (LFS) + tinted-schemes submodule for CI ([eaf6b97](https://github.com/i-am-logger/pr4xis/commit/eaf6b97d96c9ac85f5f3fc6b0bec848b4952786c))
* clarify Base16 has 16 slots, Base24 has 24 ([f876311](https://github.com/i-am-logger/pr4xis/commit/f87631199d432fedc7999eb086a28e856b84677f))
* copula type from CCG research — question 'is X a Y' now parses to Q type ([2a05413](https://github.com/i-am-logger/pr4xis/commit/2a05413a9cdf8ccec1ffb98d46d8751c0e2830da))
* remove all hardcoded pronoun/noun matching from dialogue engine ([fac095f](https://github.com/i-am-logger/pr4xis/commit/fac095f6a20509360e2c6dddada83e599c72977d))
* remove hardcoded quit/exit — farewell detection through language lexicon ([1bb193b](https://github.com/i-am-logger/pr4xis/commit/1bb193bf68d6c16bfe609f839aec456d08bf0603))
* resolve all clippy warnings for strict CI ([79ff81b](https://github.com/i-am-logger/pr4xis/commit/79ff81ba0983283e738516dc8e0be55773add52d))
* skip data-dependent tests when WordNet/themes not available (CI) ([04834f6](https://github.com/i-am-logger/pr4xis/commit/04834f62a5385e4333d379ee61e8fa577908996d))
* taxonomy query works — 'is a dog a mammal' answered correctly ([27043ce](https://github.com/i-am-logger/pr4xis/commit/27043cec07a98102b1a72ba3d65c5bbc9207bca4))
* update release-please config for pr4xis rename + add version to path deps ([ff60744](https://github.com/i-am-logger/pr4xis/commit/ff60744ee9dbdd64d2a964b39c286253216e9a58))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * pr4xis bumped from 0.3.0 to 0.4.0

## [0.4.0](https://github.com/i-am-logger/pr4xis/compare/praxis-domains-v0.3.1...praxis-domains-v0.4.0) (2026-04-12)


### Features

* rename praxis → pr4xis across entire codebase ([5e971f7](https://github.com/i-am-logger/pr4xis/commit/5e971f77ac3041a5e35209216d09f41e55cf8a0d))

## [0.3.1](https://github.com/i-am-logger/pr4xis/compare/praxis-domains-v0.3.0...praxis-domains-v0.3.1) (2026-04-12)


### Bug Fixes

* add WordNet XML (LFS) + tinted-schemes submodule for CI ([eaf6b97](https://github.com/i-am-logger/pr4xis/commit/eaf6b97d96c9ac85f5f3fc6b0bec848b4952786c))

## [0.3.0](https://github.com/i-am-logger/pr4xis/compare/praxis-domains-v0.2.0...praxis-domains-v0.3.0) (2026-04-12)


### Features

* add criterion benchmarks for all ontologies and chat pipeline ([9e8c5e1](https://github.com/i-am-logger/pr4xis/commit/9e8c5e1626c9717f5ecfb99662553bdf30201ec7))
* add durability, volatility, measurement, and benchmark ontologies ([e33ad6e](https://github.com/i-am-logger/pr4xis/commit/e33ad6e0b39567a4415b9320309caa496d50034e))
* add Instance ontology (Spivak) and SystemsToSchema functor ([750eac5](https://github.com/i-am-logger/pr4xis/commit/750eac5bb461e9d9ec166a2e853ec1189e6fe552))
* add Ontology Alignment and NLG pipeline ontologies ([46101fc](https://github.com/i-am-logger/pr4xis/commit/46101fc2770b6b14827327fdcf7e376396a630dd))
* ColorSlot::key() — canonical theme file key names ([9d84544](https://github.com/i-am-logger/pr4xis/commit/9d845446a9b8e07425828ef9786c3f67beb246e3))
* Communication→Control + Diagnostics→Metacognition functors (proven) ([b7fa833](https://github.com/i-am-logger/pr4xis/commit/b7fa833af6bdc4a6ea69b8ffdb30bbb06a0dfd16))
* complete scheme taxonomy — Vogix16, Ansi16, Base24, SchemeType ([d75e4f5](https://github.com/i-am-logger/pr4xis/commit/d75e4f51321cde732b83439d62d1981d07fd7266))
* control systems ontology + foundational cybernetics papers ([3c1eab1](https://github.com/i-am-logger/pr4xis/commit/3c1eab1c26fd31028078d9d7c668e2fff0c70a24))
* Diagnostics ontology + TracedCategory (writer monad on categories) ([b3beba6](https://github.com/i-am-logger/pr4xis/commit/b3beba6b9e28c2426850fa88fea98a3133b8edea))
* Diagnostics→Control functor (FDI IS control — Gertler 1998, proven) ([42f9b27](https://github.com/i-am-logger/pr4xis/commit/42f9b27235b02251cb0262c84c169567e27445be))
* docs/chat/ for GitHub Pages — presentation embeds live chatbot ([b4936c6](https://github.com/i-am-logger/pr4xis/commit/b4936c65d06a26595c2943c7bb6f690ed8ec3b69))
* extend dialogue ontology with QUD, CommonGround, Intention, Repair ([9d782e0](https://github.com/i-am-logger/pr4xis/commit/9d782e07b7e62d8fef47bd1d40d73aa3c4d35ba1))
* extended sentence test suite, chart type selection fix ([f869c68](https://github.com/i-am-logger/pr4xis/commit/f869c684cab5af3734f6ca53169eb138fd3dfa97))
* function words as LMF data, extend LmfPos with closed-class types ([0cbffab](https://github.com/i-am-logger/pr4xis/commit/0cbffab9585a3e3f7c202a498e28d3f733c66db9))
* integrate ontologies via functors, wire into chatbot pipeline ([3527502](https://github.com/i-am-logger/pr4xis/commit/3527502c2c7dbf18c8f26553ddbca423f6af0ae6))
* integration tests with full WordNet — expose real failures honestly ([e444807](https://github.com/i-am-logger/pr4xis/commit/e444807e07fa326fa66128075cd44ca29a70a0ce))
* Lambek → Pregroup functor — ontology evolution proven ([b9b9efb](https://github.com/i-am-logger/pr4xis/commit/b9b9efb6a7cbab8d152c5c558ac57ff7124e2e7d))
* Language::pregroup_types — end-to-end pregroup pipeline through Language trait ([75d2967](https://github.com/i-am-logger/pr4xis/commit/75d296736b4216d4c741d92426cbf51c61333ff5))
* load WordNet verb frames for transitivity — no more defaults ([d8403b8](https://github.com/i-am-logger/pr4xis/commit/d8403b8cd8b981dacedb98da150fcc4a9d6b3ab7))
* math functions + sRGB color science + theming ontology ([deff009](https://github.com/i-am-logger/pr4xis/commit/deff0096e946d1cd72b293f2ee1bf55f89dd3642))
* merge sensor-fusion ontologies, add schema/storage/consistency ontologies, praxis-web ([a2fe629](https://github.com/i-am-logger/pr4xis/commit/a2fe629aedbe2363a33b76772e997d1558e14259))
* migrate 11 vogix ontologies into praxis theming (3117 tests) ([2b288a1](https://github.com/i-am-logger/pr4xis/commit/2b288a1dcf974cccbd08716e7b674aabaee55718))
* migrate burp + pssst ontologies — 123 categories, 61 functors, 4707 tests ([3299cdc](https://github.com/i-am-logger/pr4xis/commit/3299cdcb7e068ba89cd215c641b66d3e3fed4120))
* NoisyChannel→Communication + DRT→Dialogue functors (proven) ([92c37a1](https://github.com/i-am-logger/pr4xis/commit/92c37a1d6159613afc22791846bb3b16e6829772))
* pregroup grammar ontology — parsing as group algebra ([08881d8](https://github.com/i-am-logger/pr4xis/commit/08881d8d30c38fd2a55193a09175641614d6ad04))
* PregroupCategory — proper Category with proven laws ([b77c12f](https://github.com/i-am-logger/pr4xis/commit/b77c12f821ee4ae3a94ba9cb8055ab175fd51fb9))
* prop tests + functor connections across 15 ontologies (2934 tests, 18 functors) ([cf77284](https://github.com/i-am-logger/pr4xis/commit/cf772848478dd7929b0d959265369ac82674fb5e))
* proper ontology trace — each step reports what it did with status ([f7b1b7f](https://github.com/i-am-logger/pr4xis/commit/f7b1b7f7bfb5502e6c851236893e9d0a0d4a4241))
* proper ontology trace — each step reports what it did with status ([1605d7e](https://github.com/i-am-logger/pr4xis/commit/1605d7e94d7c1af2debce45ec7534257d9c67bd2))
* proper ontology trace — each step reports what it did with status ([20739ae](https://github.com/i-am-logger/pr4xis/commit/20739ae4f4b9e3926e6b22747add976f3e01249c))
* Rgb::from_hex and to_hex for color parsing ([79b88cc](https://github.com/i-am-logger/pr4xis/commit/79b88cc9d5f96521065b62a5673eb4be9906bc60))
* rich taxonomy responses with path, definitions, and subtypes ([4bd0fc8](https://github.com/i-am-logger/pr4xis/commit/4bd0fc84bdf99a03ea7d7423cc1ad5f073f6d3cc))
* self-model ontology, CYK chart parser, adjunction, response generation ([0f67d8d](https://github.com/i-am-logger/pr4xis/commit/0f67d8d629065b3e76d3acd4567a03e9cc346c7e))
* speech production ontology (Levelt pipeline as category) ([b991418](https://github.com/i-am-logger/pr4xis/commit/b991418fad697aa8a2d06499bb7f81bb263daa58))
* trace functors — map pipeline steps to Diagnostics/PROV ontologies ([f3d234f](https://github.com/i-am-logger/pr4xis/commit/f3d234f81ad3ffc8458b25fc5280249f07658cba))
* trace shows Lambek notation (S[q]/NP) not Rust debug format ([3f24ff5](https://github.com/i-am-logger/pr4xis/commit/3f24ff5493ba958594ed72f9d4717936acfcb957))
* TraceSchema ontology — T(C) = El(C) + O_obs (Spivak 2012) ([2e0df62](https://github.com/i-am-logger/pr4xis/commit/2e0df6279843fb7bc9bfcca70bdf614f0cbb6db5))
* Traum grounding state machine as finite category ([00c811d](https://github.com/i-am-logger/pr4xis/commit/00c811d6ed75b72be3625c21b5d6e4b042b0a33a))
* Turing test benchmark — 18 questions, 3 pass, 15 need ontologies ([9c7c64e](https://github.com/i-am-logger/pr4xis/commit/9c7c64e18f96b6f497c170fac35efac2ab99bdac))


### Bug Fixes

* clarify Base16 has 16 slots, Base24 has 24 ([f876311](https://github.com/i-am-logger/pr4xis/commit/f87631199d432fedc7999eb086a28e856b84677f))
* remove all hardcoded pronoun/noun matching from dialogue engine ([fac095f](https://github.com/i-am-logger/pr4xis/commit/fac095f6a20509360e2c6dddada83e599c72977d))
* remove hardcoded quit/exit — farewell detection through language lexicon ([1bb193b](https://github.com/i-am-logger/pr4xis/commit/1bb193bf68d6c16bfe609f839aec456d08bf0603))
* skip data-dependent tests when WordNet/themes not available (CI) ([04834f6](https://github.com/i-am-logger/pr4xis/commit/04834f62a5385e4333d379ee61e8fa577908996d))

## [0.2.0](https://github.com/i-am-logger/praxis/compare/praxis-domains-v0.1.0...praxis-domains-v0.2.0) (2026-04-09)


### Features

* cognition ontologies — distinction, epistemics, metacognition ([b20e3c7](https://github.com/i-am-logger/praxis/commit/b20e3c788dc810d0e285c1cfabebd819fbe8bbe5))
* concurrency ontology — chess IS concurrent (proven via functor) ([38edc0c](https://github.com/i-am-logger/praxis/commit/38edc0ce4f1b2dc2a5b06afd602f1defabd52c6c))
* dialogue ontology + chatbot CLI — praxis can chat ([b62fd99](https://github.com/i-am-logger/praxis/commit/b62fd99d4243ef8c39bb99c506aa298cc5a6ccb7))
* DOLCE upper ontology, domain restructure, linguistics, systems thinking, codegen ([368c583](https://github.com/i-am-logger/praxis/commit/368c5835965915495ea43d1b5e3dcbc76b1a93e6))
* English language ontology — 107k concepts, nanosecond queries ([bf5113f](https://github.com/i-am-logger/praxis/commit/bf5113ffbaa471c1d7247b4ab28b4bba78d56c5e))
* event-driven ontology — chess IS event-driven IS concurrent (proven) ([24bf058](https://github.com/i-am-logger/praxis/commit/24bf058eff868bf169f27e3e4e0c7633d42a339a))
* information ontology — what bits, bytes, references, and text ARE ([28be527](https://github.com/i-am-logger/praxis/commit/28be5273775a6278ff6fd3edf99612a33c938e78))
* Lambek grammar — syntax as category, text understood through type reduction ([3c1054d](https://github.com/i-am-logger/praxis/commit/3c1054db5c53bfbd171cf5212a021d964cb9512d))
* Language trait, orthography, morphology, cached reasoning queries ([89daa5a](https://github.com/i-am-logger/praxis/commit/89daa5a45dcdf211e336dceb99503c9a9babda11))
* Montague functor — type-driven syntax-semantics interpretation ([5ce26d2](https://github.com/i-am-logger/praxis/commit/5ce26d224486d45d7c1c6d92a745d0c6e98595d9))
* question grammar types + Q semantic domain in Lambek/Montague ([05d5ea9](https://github.com/i-am-logger/praxis/commit/05d5ea97c98abc6c375968000b833feb7ba14f05))
* SystemsToConcurrency functor — every system IS concurrent (proven) ([e53bcbb](https://github.com/i-am-logger/praxis/commit/e53bcbbffbd8fc255af169aa14f050f1397da9b1))
* SystemsToEvents functor — closes System ↔ EventDriven ↔ Concurrent triangle ([7bca79b](https://github.com/i-am-logger/praxis/commit/7bca79b4bc9fc8c5710a5a0c170b59a3ccf41ac8))
* WordNet-LMF ontology — full 107k synset load in 3.8s ([904b69e](https://github.com/i-am-logger/praxis/commit/904b69e12f5c6baeef4fe405b8442f7c97df4a36))
* XML ontology, enhanced property tests, systems thinking completeness ([7e00e60](https://github.com/i-am-logger/praxis/commit/7e00e60311abc63022b0598b1f1b8ea4b3f8eabc))


### Bug Fixes

* copula type from CCG research — question 'is X a Y' now parses to Q type ([2a05413](https://github.com/i-am-logger/praxis/commit/2a05413a9cdf8ccec1ffb98d46d8751c0e2830da))
* resolve all clippy warnings for strict CI ([79ff81b](https://github.com/i-am-logger/praxis/commit/79ff81ba0983283e738516dc8e0be55773add52d))
* taxonomy query works — 'is a dog a mammal' answered correctly ([27043ce](https://github.com/i-am-logger/praxis/commit/27043cec07a98102b1a72ba3d65c5bbc9207bca4))
