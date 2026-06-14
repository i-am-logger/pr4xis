# praxis as a self-aware system — architecture & migration (2026-06-14)

Authoritative design for the current PR (WASM UI + reasoning alignment). Produced by an
inward→outward→synthesize→critique design pass (praxis-law gate + completeness critic) over
the full **load → reason → answer → self-model → page** loop. Refuted findings dropped;
critique revisions folded in. Status markers: **FIX NOW** vs **INFORMATIONAL**.

> One sentence: there must be exactly **one** loaded set, of exactly **one** type —
> `Vec<RuntimeOntology>` — that is simultaneously (a) what the single `ComposedReasoner`
> composes over, (b) what the `SelfModelInstance` eigenform reifies, and (c) what the page
> projects. Today there are **two** (`Pr4xis.loaded: Vec<LoadedSource>` and
> `Pr4xis.runtime_ontologies: Vec<RuntimeOntology>`) and only the second reaches the reasoner.
> Collapse the first into the second through typed projectors; point the self-model at it; key
> a content-addressed history on its `ContentAddress`; thread `OntologyName` (not `&'static str`)
> as the provenance token. **Almost everything already exists.**

## The symptom that proves it

Load USC **Title 15** in the page → ask about it → no answer / not connected. Verified cause:
`load_source` (`crates/wasm/src/lib.rs:227`) materializes a title into `LoadedPayload::UsCode`
and pushes onto `Pr4xis.loaded` (`:231`); but `chat()` answers through the `ComposedReasoner`
(`:186`), which only holds `runtime_ontologies`. **The reasoner never sees `self.loaded`.** It's
a wire-break, not missing capability.

---

## 1. The unified type — `RuntimeOntology` is already the colimit object

`RuntimeOntology` (`crates/pr4xis-runtime/src/ontology.rs:380`) carries everything the reasoner
needs: `id: OntologyName` (the typed provenance token), `root: ContentAddress` (the Merkle
identity / history key), `closure(): MaterializedClosure` (the pre-folded answer source),
`archive().nodes[].lexical` (the gloss answers). `ComposedReasoner` (`composed.rs:78`) already
**is** the colimit (Goguen): `new(english, loaded: Vec<RuntimeOntology>)` grounds every loaded
node into one Lemon lexicon and assigns disjoint `ConceptId`s (`composed.rs:149`); `decode()`
recovers each concept's owner as the typed sum `English(ConceptId) | Loaded(ConceptRef)`
(`composed.rs:62,240`) — never `String ==`. **It already works for one kind (`.prx`).**

**Delete `LoadedSource`/`LoadedPayload`** (`lib.rs:90,99`) — the `enum { UsCode | Owl }` + `match`
(`lib.rs:111,120`) is the *encoded* working representation our rule forbids. Replace with
**projection-as-data**: each kind has a functor into `RuntimeOntology`; after projection nothing
in the runtime branches on kind.

| Kind | Functor | Status |
|---|---|---|
| `.prx` archive | `load → materialize` | EXISTS, wired (`lib.rs:516`) |
| **USC title** | `project_archive(usc)` (`uslm/corpus/bridge.rs:49`, Parthood `:39`, lexical `:87`) | **EXISTS but UNWIRED — test-only. This is the fix.** |
| OWL vocab | *new* `owl/bridge.rs` | MISSING (mechanical: `LoadedOwlVocabulary::{entities,subsumption_edges,label_of}` `vocabulary.rs:336,342,351`) |
| function-words / WordNet | embedded `English` (its own `.prx.gz`) | EXISTS |

### 1.3 The one real reasoning gap — Parthood
`ComposedReasoner` pre-fold filters `if edge.kind != Subsumption { continue }` (`composed.rs:191`)
and its loaded closure reads are Subsumption-only (`:332,:355`). USC projects **Parthood**
(`bridge.rs:39`). So a USC `RuntimeOntology` composes as isolated points *for taxonomy* — but
**gloss answers ("what is section X") need none of this**: they read `onto.lexical()`
(`composed.rs:164`), which `project_archive` populates. **So Title 15 gloss answers light up the
instant `load_source` is wired (Step 1); full structural queries need Step 3.** Fix: pre-fold +
closure reads iterate `RelationKind::transitive()` (`ontology.rs:126`), loaded-not-`== Subsumption`.

---

## 2. The SelfModel — the eigenform over the live loaded set

`SelfModelInstance::observe(components)` (`instance.rs:72`) is the von Foerster operator `X=F(X)`
(`instance.rs:9`). **Today `F` is applied to a constant**: `loaded_ontologies(_lang)` ignores its
arg and returns `describe_knowledge_base()` + a static English `Vocabulary` (`chat/lib.rs:786`);
`total_concepts/morphisms` (`instance.rs:73`) sum only the substrate — blind to the live set. So
`X=F(X)` holds *vacuously* (Smith's reflective drift: the meta level reifies a description not
causally connected to the object level).

**Fix (one redirection):** `observe()` folds the **live `self.runtime_ontologies`** via a thin
`Vocabulary`-adapter (name = `id().as_str()`, concepts = `archive().nodes.len()`, morphisms = Σ
`node.edges.len()`). One object level, reified once → "entities" moves the moment you load Title 15.
The loaded set **is** the memory; the self-model is its eigenform (stable under re-observe,
verifiable by content-address equality over sorted `root()`s).

### 2.3 Provenance as data — a TYPED SUM (law-critique revision)
`chat()` computes `trace.all_participating_ontologies()` every turn (`lib.rs:190`) then **discards**
it. The trace can't name a loaded ontology because `PipelineTraceEntry.ontology` and
`all_participating_ontologies()` return `&'static str` (`trace_functors.rs:47,473`).

**Revision (do NOT flat-replace with `OntologyName`):** `PipelineStep` is `const fn new(.. &'static str ..)`
const-constructed (`trace_functors.rs:52`) and ~10 tests compare `== "Communication (Shannon)"`
(`:726-767`). Use a typed **sum**:
```
enum TraceOntology { Compiled(&'static str), Loaded(OntologyName) }
```
Compiled steps keep their const literal; a loaded-ontology step (from `decode()`, `composed.rs:240`)
carries the real `OntologyName`. `all_participating_ontologies() -> Vec<TraceOntology>`. Add a
self-model dimension `reasoned_over: Vec<OntologyName>`. The "Title-15-names-Title-15" acceptance
test asserts the `Loaded(..)` variant.

### 2.4 Content-addressed history (the recovered "lost" dimension)
There is **zero** temporal structure today (grep = 0). Add an append-only
`history: Vec<LoadEvent>` (`Load | Replace`, each carrying the affected `root: ContentAddress`),
appended in `install_runtime_ontology` (`lib.rs:492`); state-CID = Merkle fold over sorted roots,
**reusing** the `GraphSnapshot` codecs + fail-closed gate (`praxis_knowledge_graph/snapshot.rs`) —
no new hash/codec. Emit in `present()`; render a memory/history panel.

---

## 3. The page is a pure projection

`renderDashboard` already reads `JSON.parse(call('self_describe'))` (`index.html:545`) with **no**
per-source `UsCode/Owl` branching — structurally a projection. Once §2 makes `self_describe`
reflect the live set, stats/catalog/memory/trace/history all render from praxis introspecting
itself. **Residue to remove:** the hardcoded demo example `"what is a correctservice"` at **both**
`index.html:524` AND `:646` (law critique: design named only :524). The catalog name-join drops
loaded-but-unregistered runtime ontologies (`catalog.rs:256`) — fix: include them by
`OntologyName`, not registry membership.

---

## 4. Completeness additions (the missing 20% — all FIX NOW unless noted)

1. **Abstention as a typed outcome (highest).** `chat()` returns `String`; there is no
   `Answered | Abstained { reason }` sum — yet abstention is the most-tested behavior
   (`lib.rs:597,766`). A self-aware system must model *what it cannot answer* (= what to load).
   Add a typed `ChatOutcome` on the chat result; surface a self-model `gaps: Vec<Surface>`
   ("asked but not loaded") panel.
2. **The `success: bool` already in the trace is dropped.** `PipelineTraceEntry.success`
   (`trace_functors.rs:198`) is thrown away by `all_participating_ontologies()` (`:473`) → a
   provenance *lie* (records an ontology as "reasoned over" on a failed step). `reasoned_over`
   must distinguish *traversed* from *answered-from* (carry `(TraceOntology, success)`).
3. **`total_concepts` double-count (INTEGRITY, no test catches).** After folding adapters into
   `observe()`, English risks being counted in both the substrate `Vocabulary` and the adapters.
   Add the invariant test `total_concepts == composed.concept_count()` (the reasoner's coproduct
   cardinality) and fold the coproduct count, not a naive sum.
4. **History: loading vs reasoning events (SCOPE DECISION).** §2.4 logs *load* events. `chat()` is
   `&self` (`lib.rs:177`) so **no turn is ever recorded** — every answer/abstention/provenance
   evaporates. Either `chat(&mut self)` to append reasoning-events (provenance+outcome,
   content-addressed by `(input_hash, state_root)`) **or** explicitly scope reasoning-history out.
   *Decide before Step 6.*
5. **`install` replaces in place with no history.** Two install methods exist; until Step 2
   deletes `LoadedSource`, history is kind-dependent. The `Replace` event must be emitted inside
   the `retain` (`lib.rs:493`), capturing the *displaced* root.
6. **Fail-closed on recompose (INTEGRITY).** `install_runtime_ontology` rebuilds the whole
   `ComposedReasoner` every load (`:496`) and `ComposedReasoner::new` returns a value, not a
   `Result` — a malformed projected archive (Parthood-only USC → isolated points) yields a reasoner
   that's "loaded" but answers nothing while the dashboard shows green. Validate post-grounding
   (non-empty closure / ≥1 lexical) before committing `composed`, mirroring the `.prx` gate.
7. **Capabilities ≠ size.** After Step 1, USC answers *gloss* but not *parthood* (until Step 3) —
   card goes green while half its capability is dark. Add per-ontology
   `capabilities: {gloss, subsumption, parthood}` derived from which `RelationKind`s its closure
   actually populates (`MaterializedClosure`). Without it, "loaded" still lies.
8. **Mechanical residue at the kind-name layer (INFORMATIONAL).** `RelationKind::from_edge_kind`
   matches `"Subsumption"/"Parthood"/"Causation"` string literals (`ontology.rs:116`) fed by static
   `SECTION_KIND`/`COMPOSES_REL` (`bridge.rs:33,39`) — a `String==` codec at the materialize
   boundary. Named here as surviving residue; de-mechanize opportunistically.

---

## 5. Migration path (smallest-first; preserves the 6 `loaded_ontology_count()==0` assertions)

None of the 6 acceptance tests (`lib.rs:614,637,683,713,804,823`) load a USC/OWL source, so all
stay green; `loaded_ontology_count()`'s honest meaning becomes "ontologies the reasoner composes
over" — which after unification is *every* load.

- **Step 1 — wire `load_source` → `project_archive → materialize → install_runtime_ontology`.**
  *Unblocks "load Title 15 → ask about it"* (gloss answers). A few lines; first production caller
  of an already-tested functor. **Highest impact, smallest change.**
- **Step 2 — OWL `bridge.rs`; route OWL through `install_runtime_ontology`; delete
  `LoadedSource`/`LoadedPayload`** (the two universes collapse to one).
- **Step 3 — Parthood reasoning** (`RelationKind::transitive()` in pre-fold + closure reads) →
  full USC structural queries.
- **Step 4 — causal-connection eigenform** (`observe()` over the live set) → stats stop lying.
- **Step 5 — `OntologyName` provenance** (the typed `TraceOntology` sum) + `reasoned_over` +
  success-aware (`#4.2`).
- **Step 6 — content-addressed history** + memory/history panel; resolve the reasoning-history
  scope decision (`#4.4`).
- **Cross-cutting:** abstention (`ChatOutcome`, `#4.1`), fail-closed recompose (`#4.6`),
  capabilities (`#4.7`), double-count invariant (`#4.3`).

---

## 6. `.prx` in the release + content-addressed naming (FIX NOW)

The 0.24.0 release carries **no** `.prx` — the post-deploy `gh release upload --clobber`
(`ci.yml:444-454`) hit `HTTP 422: Cannot upload assets to an immutable release` (release-plz makes
immutable releases; the release is created *before* the `pages` job emits the `.prx`). These are
the **canonical immutable archive** (`emit_prx.rs:6`) — important, **do not delete the step**.
Fix: attach the `.prx` *at release creation* (emit before release-plz finalizes), or disable repo
immutable-releases. **Content-address the distribution filenames** (hash-keyed, not
`{name}-{version}`) → write-once, immutable-friendly assets + cache-forever Pages URLs; the page's
`prx_url` (`build.rs`) reads the hash from the embedded lock.

Also: the emitter is OWL-only — generalize over a single `emits_loadable_prx()` predicate
(`OntologyVocabulary | Language | ClosedClassLexicon`) so function-words/WordNet `.prx` ship too;
all of emitter / `build.rs` manifests / tests derive from that one predicate.

## 7. CI (FIX NOW)
`wasm-pack build --target web --release` runs in **both** the `wasm` job (`ci.yml:308`) and the
`pages` job (`:413`), and the cache `shared-key`s mismatch (`:281` vs `:392`) so `pages` recompiles
cold (the header comment `:25` is *false*). Fix: `wasm` job uploads `pkg`/`sources` as an artifact;
`pages` downloads instead of rebuilding. Removes one full wasm32 release compile + wasm-pack install
+ WordNet fetch per deploy.

---

## 8. Bulletproofing — praxis-level, property-based (proptest), not brittle examples

The existing chat tests are example-based and cover only the embedded `.prx`/OWL path — **no test
loads a USC title and queries it**, which is why Title 15 silently never worked. Invariants over the
loaded *family* (native `mod acceptance` / `crates/chat` / `crates/domains` — `web.rs` runs only
under wasm-pack):

- **`loading_a_usc_title_makes_it_queryable` (the headline; write it to FAIL today).** Load a Title
  fixture → ask a gloss it should answer → assert a real answer + `reasoned_over` credits the title's
  `OntologyName`. Green after Step 1.
- **Load-then-ask (proptest over the family).** For any registered loadable `S` and concept `C ∈ S`:
  after loading `S`, `chat("what is C")` answers and credits `S`; for `C` in no loaded source, it
  **abstains** (typed `Abstained`, `#4.1`).
- **Provenance soundness ∧ completeness.** `reasoned_over` == exactly the ontologies that supplied a
  `ConceptId` used in a *successful* answer — no spurious credit (`#4.2`), no omission.
- **Self-model fidelity (eigenform).** `self_describe()` loaded set == reasoner `loaded()`; totals ==
  real per-ontology sums == `composed.concept_count()` (`#4.3`), under arbitrary load sequences.
- **Monotonicity.** Loading never shrinks what's answerable.
- **Catalog answerability ≠ registry membership.** Load the embedded demo → assert its `OntologyName`
  appears in `self_describe().catalog` as Loaded (the inverse of the current silent drop, `#3`).
- **Capabilities honesty.** A Parthood-only USC reports `{gloss:true, parthood:false}` before Step 3
  (`#4.7`).
- **Fail-closed recompose.** A malformed projected archive is refused, leaving `composed` unchanged
  (`#4.6`).

---

## Decisions (settled by the maintainer, 2026-06-14)
1. **Reasoning-history (`#4.4`): BOTH.** `chat(&mut self)` records a reasoning-event per turn
   (provenance set + outcome, content-addressed by `(input_hash, state_root)`), in addition to the
   load-history. Praxis remembers how it's been *used*, not just how its knowledge evolved.
2. **Immutable releases (`#6`): ATTACH AT CREATION + CONTENT-ADDRESS.** Keep immutable releases;
   emit `.prx` before release-plz finalizes and attach as write-once content-addressed assets.
3. **Content-addressed `.prx` filenames (`#6`): ADOPT NOW** (write-once distribution; hash-keyed
   Pages URLs).

## Build order
Test-first, **Step 1 (the Title-15 wire) first**, on a fresh branch off master (0.24.0).

---

## 9. Queryability — the lexicalization praxis-way (Step 1b)

Implementing Step 1 surfaced that the wire is necessary but not sufficient: loaded concepts are
queryable only by their **identity** name, which for USC is a URN (`/us/usc/t18/s1`) and for OWL an
IRI — neither tokenizes as natural language. A design pass (willing to **challenge** the repo's
Ontolex-Lemon commitment, surveying SKOS / Frege / Kripke / Russell and the repo's own substrate)
returned the principled foundation:

**Foundation: Frege (*Sinn/Bedeutung*, 1892) + Kripke (rigid designation, 1980), realized through
the #87 content-addressed `ontolex:Form` + `denotes` floor — NOT a deeper Lemon-struct commitment.**
- **Frege:** one *Bedeutung* (reference = the identity), many *Sinne* (surfaces = forms). The line
  `let surface = node.name.to_lowercase()` (`composed.rs:154`) **collapses Sinn into Bedeutung** — it
  re-derives the surface from the identity. That is the bug; a URN *correctly* doesn't tokenize.
- **Kripke:** a URN/IRI is a **rigid designator** (fixed by baptism, not description) — so it stays
  opaque identity; NL surfaces are independently-baptized co-referring **names**, modeled as the
  content-addressed `Form` atoms reached by a `denotes` edge. More Kripkean than Lemon's descriptive
  `denotes = sense∘reference`.
- **Lemon `otherForm` / SKOS `altLabel` are *named views*** of the one substrate `denotes`/`Form`
  relation, carried as `.prx` functor data — never Rust storage or a `match`. Committing harder to the
  Lemon struct would re-encode in one domain what the substrate does generically ([[feedback-ontological-not-rust-primitives]]).

**The principle (invariant):** three distinct channels —
`IDENTITY` = `Definition.name` (URN/IRI, content address; **never** a surface) ·
`GLOSS` = `Definition.lexical` (the "what is X" answer; kept) ·
`SURFACE(s)` = a set of `ontolex:Form` atoms, each reached by a typed `denotes` edge
(`EdgeTarget::Grounded`, `definition.rs:51`). One concept → many Forms → many surfaces.

**Generic mechanism (one loop, no per-kind code):** rewrite `composed.rs:151-176` to index nodes
with `kind == FORM_KIND` (the `writtenRep` is the surface) bound to their concept via the `denotes`
edge — reusing `Lexicon::add_entry` (`lexicon.rs:163`) and the `ground`/role machinery
(`grounding.rs:41,71`) unchanged. Each **bridge** mints its Form atoms (`form_atom`,
`english/bridge.rs:143`) + `denotes` edges: USC `section.heading` (kept as gloss **and** minted as a
Form) + citation Forms from `title_number()`/`num` ("title 18", "section 1514A"); OWL needs a **new
`project_archive`** (`name=iri`, `lexical=rdfs:comment`, Form from `rdfs:label`); English already does
this (`project_archive_with_forms`, `english/bridge.rs:163`). The relation-kind (`heading ↦ denotes`)
rides as a `.prx` functor row beside `wordnet_to_praxis_functor`.

**Over-generation guard (Lemon `canonicalForm` vs `otherForm`):** the canonical surface is the
**whole** heading/label string (not word-by-word); variant surfaces ("section 1514A") are *curated*
Forms minted from **structured fields**, not from tokenizing prose. Single content words are *not*
minted → abstention is **strengthened** (the URN no longer pollutes the index), and existing
domain-salience ranking (`lexicon.rs:98`, Koeling 2005) handles legitimate collisions.

**Critique (NEEDS-REVISION) — must-do before/while implementing:**
1. **Do NOT delete `composed.rs:154` before the PRODUCERS mint Forms.** The live callers build from
   `emit::<StatuteCategory>()` whose `node.name` *is* a natural word, and `grounding_unions_the_loaded_surface_into_the_lexicon`
   relies on the lowercase. Add Form-minting to `emit()` + the bridges **first**, then delete 154 —
   else regress `lookup`.
2. **The reconciliation (RESOLVED): a distinct *role*, not a third meaning of `denotes`.** The
   existing `denotes` is the **prose floor** — a statute provision → the English-word `Form`s in its
   text, *cross-archive* (`statute_structure/grounding.rs`, via `project_archive_with_forms`). The new
   need is **lexicalization** — a concept → its *own* label `Form`, *same-archive*. Both ride the
   content-addressed `Form` substrate, distinguished by the **role carried as data** (`grounding.rs:71`):
   use Lemon's `canonicalForm` (the whole heading, functional — one per concept) + `otherForm` (curated
   citation surfaces from `num`/`title_number`), **reserving `denotes` for the prose floor**. The
   grounding indexes the `canonicalForm`/`otherForm`-role `Form`s as queryable surfaces; the `denotes`
   floor is untouched. (Mechanism note: English's *own* surfaces come from its WordNet lexicon
   `word_index`, not from `Form`-edge traversal — `composed.rs:133`; so the `Form`-surface channel is a
   **new** mechanism for loaded ontologies. Add it **additively** — index `canonicalForm` Forms
   *alongside* `node.name` — so nothing regresses; delete the `node.name` collapse only once every
   producer mints its label `Form`s.)
3. OWL's real location is `social/software/markup/xml/owl/` (`vocabulary.rs` `iri`/`rdfs:label`/`rdfs:comment`).

---

## 10. FOUNDATIONAL — `ontology!` ≡ `.prx`, the modality-neutral substrate, and the real leak

A deep design pass (homoiconicity/reflection, category-as-presentation, Gärdenfors, lenses) on the
maintainer's principle ("code is ontological; no difference between `ontology!` and `.prx`; knowledge
isn't only language — it can be shapes, drawing, who knows").

**`ontology!` and `.prx` are one presented category in two serializations** — both functors over the
same `Archive`. This is *already real* for the reasoning skeleton: `emit::<Cat>()` (`emit.rs:172`)
lowers a compiled `Category` into the byte-identical `Archive` a `.prx` loads, round-trip-tested
(`meta.rs:113`). **English already ships as a `.prx`** (`ENGLISH_PRX_GZ`, `wasm/lib.rs:26`); its
"privilege" is purely *in-memory* — which Rust type the bytes become (`English` struct) and that
`ComposedReasoner` pins it as a base (`composed.rs:80`). De-privilege it: consume English **as a
`RuntimeOntology`** like any other — `english_runtime_ontology()` (`bridge.rs:224`) exists and is
proven to give identical `is_a` answers. The one missing direction is **`reify: Archive → Category`**
(the inverse of `emit`); the meta-ontology *names* it (the `Parse⊣Generate` `Lens` at `meta.rs:93`)
but doesn't interpret it (`apply` fail-closes on `Lens`, `apply.rs:84`).

**The `.prx` format evolves, minimally:** the substrate is ~80% modality-neutral already
(`Definition{kind,name,edges,axioms}`, `EdgeTarget::Grounded{ontology,atom}`, `Connection{action:
GeneratorAction}`, identity = `ContentAddress`; `kind`/relation-kind as open strings). The leaked
language field is **`Definition.lexical: Option<String>`** (a gloss, baked into the content address —
`definition.rs:163,216`). Demote it: the gloss becomes a `denotes` edge into a language-lens `Form`
atom (`form_atom`, `ground` already exist). Lenses (language / shape / generative-"draw") are **loaded
`Connection`s selected by KIND** — an open, loaded-not-encoded registry; a capability like *drawing*
rides the functor/`Adjunction` (`Σ_F`) machinery, not a noun.

### ⚠ The real leak the critique found (NEEDS-REVISION — and it's the heart of your conflict)
Demoting `Definition.lexical` is the **storage** leak. The **interpreter** leak is deeper and is the
true "code is ontological" violation: **the engine's reasoning — transitive-closure folding in
`MaterializedClosure::fold` (`ontology.rs:222`) — runs only over a CLOSED Rust enum `RelationKind
{Subsumption, Parthood, Causation}` via a hard `match` (`from_edge_kind`, `ontology.rs:116-121`,
`_ => None`).** An edge kind the enum doesn't know is *dropped from the closure*. So:
- a **shape** ontology loads (neutral storage) but is **inert to reasoning** (no foldable geometry relation);
- English's **25 WordNet relations** (derivation/antonym/…) are **un-foldable as data** — only the 3 enum kinds reason.

This is precisely the standing [[feedback-ontological-not-rust-primitives]] rule: the live
representation must BE the loaded ontology, never a Rust enum/match. **The substrate is modality-neutral
only for inert storage, not for reasoning, until relation-kind + transitivity are themselves LOADED**
— e.g. an OBO-RO-style `transitive_over` axiom carried in the meta-ontology and consulted by `fold`,
so reasoning is driven by data, not a compile-time match. (Also: `ComposedReasoner` keys the live join
on a `u64` `ConceptId` offset index — `composed.rs:96-115` — another live-form index the rule names.)

**Secondary critique corrections:** `reify` needs *new compile-time machinery* — the compile-time
`ConnectionFamily` enum has **no `Lens` variant** (`emit.rs:46-68`), so it's more than "finishing an
interpreter." "Draw = `Σ_F` via `apply`" is asserted, not shown — `apply` only emits an `Archive` of
relabeled `Definition`s; it can't mint a structured value (a `LambekType`/a rendered shape), as
`category_projection.rs:14-21` already states. `form_atom` still sets `lexical` (relocates, not
deletes, the field). The compact-succinct codec (`compact_succinct.rs:141`) carries glosses in a
**second** format the migration must also evolve. Dropping `lexical` re-pins 10+ committed `.prx`.

### Revised migration (foundational, smallest-first)
- **Phase A — loaded relation-kind + transitivity (the real fix, do FIRST).** Make `fold` consult a
  loaded `transitive_over`/relation-kind property in the meta-ontology instead of `match RelationKind`.
  This is what makes reasoning modality-neutral (shapes, English's 25 relations, any kind). Without it
  the rest is cosmetic.
- **Phase B — de-privilege English:** route English through `english_runtime_ontology()`; one
  `Vec<RuntimeOntology>`, no base/offset; cross-ontology `is_a` over the unioned closure (no format change).
- **Phase C — `reify`:** add the `Lens` connection family + interpret `Parse⊣Generate` so
  `reify(emit(C)) ≡ C` (a checkable lens law). Closes `ontology! ≡ .prx`.
- **Phase D — demote `lexical`:** coexist (add `denotes`/Form via `ground`) → read-through-lens → drop
  the field from both codecs → one mechanical re-pin + `praxis.lock` bump.
- **Phase E — capabilities:** interpret `Adjunction` `Σ_F` for generative lenses (draw/shape
  construction); absorb the lambek bespoke interpreter + HMI `engine::Action` into loaded data.

**Verdict:** the unification is real and ~half-built; the format evolution is minimal (drop one field);
but **merge-readiness requires Phase A — loaded relation-kind/transitivity — because the closed
`RelationKind` enum, not `Definition.lexical`, is the leak that makes the substrate language/3-relation
bound.** This is the precise locus of "code is ontological" the maintainer sensed.

---

### Phase A — implementation spec (the kernel change, mapped)

**The two leak sites (verified):** `materialize` (`ontology.rs:571`) and `morphisms_from`
(`ontology.rs:440`) both do `if let Some(kind) = RelationKind::from_edge_kind(kind_name)` — **silently
dropping every edge whose kind isn't one of the 3** before it ever reaches the closure; then
`MaterializedClosure::fold` (`ontology.rs:221`) iterates the hardcoded `RelationKind::transitive()`.

**The change (coordinated, one atomic refactor — it won't compile half-done):**
1. **Open the relation-kind.** `RelationKind` enum → an open type (newtype over the kind string with
   `subsumption()/parthood()/causation()` constructors + `as_str()`). `from_edge_kind` becomes total
   (`RelationKind::new(name)` — never `None`); `RuntimeEdge.kind` carries any kind; the two drop-guards
   become unconditional pushes (no edge dropped).
2. **Data-driven fold.** `MaterializedClosure::fold(edges, transitive: &BTreeSet<RelationKind>)` folds
   one closure per *declared-transitive* kind (read from data), not `RelationKind::transitive()`.
   `reachable_from`/`reaches`/`union`/`subsumption_*` re-key off the open kind.
3. **Consumers (small set):** `composed.rs` (`edge.kind != RelationKind::Subsumption` →
   `…subsumption()`; the closure-query kind args), `english/bridge.rs`, `emit.rs`, `codegen/uslm.rs`,
   runtime tests. The huge domains `RelationKind` list is the *category-layer* type, untouched.
4. **Test (the gate):** a `.prx` with a custom transitive kind (`a -broader→ b -broader→ c`, `broader`
   declared transitive) materializes and `reaches(a, c, broader)` — RED today (dropped at `:571`),
   GREEN after. Plus an inert-shape check (a non-transitive custom kind does *not* fold).

**DECISION (maintainer, 2026-06-14): (A) — the `.prx` self-declares its relation-kind transitivity.**
A relation-kind's transitivity is ontological data carried *in* the file, OBO-RO style: a relation `R`
is transitive iff it declares `R transitive_over R`. Representation (additive, no rkyv/CBOR *schema*
change — only Vec *content* grows, so old `.prx` still deserialize): a `Definition{kind:"RelationKind",
name:R, edges:[("transitive_over", Local(R))]}` node per transitive kind. `materialize` collects the
transitive set from these nodes and threads it to `fold`; archives that declare none fall back to the
literature base (`Subsumption/Parthood/Causation`) **only transitionally**, until every committed
`.prx` is re-emitted with explicit declarations (the one-time re-pin). No caller signature change.

**Execution note (the "no half-fix" reality):** Phase A is *one atomic* refactor (open `RelationKind`
→ the two drop-guards → the data-driven `fold` → consumers → the gate test) plus a one-time re-emit +
`praxis.lock` re-pin of the committed `.prx`. It is the riskiest change in the tree and must land
green under `dev-ci` in a single coherent pass — not partially. It is fully specified above and ready
to execute as the focused first task of the next working block.

**Revised build order for §9 (producers-first, additive, no-regress):**
(1) USC bridge mints `form_atom(heading)` + a `canonicalForm`-role edge per section (and `otherForm`
citations from `num`/`title_number`); (2) `composed.rs` grounding *adds* `canonicalForm`/`otherForm`
`Form` indexing alongside the existing `node.name` surface; (3) the headline test passes (heading
resolves, URN does not over-resolve); (4) once OWL + `emit::<StatuteCategory>` also mint label Forms,
delete the `node.name.to_lowercase()` collapse (`composed.rs:154`). The descriptive "the first section"
(Russell) remains a parked extension needing the projected `num` ordering.

**Test:** `first_section_of_title_18_resolves_by_heading_not_urn` (`composed.rs` tests) — `lookup("obstruction of justice")`
→ the section; `lookup("/us/usc/t18/s1514a")` → `&[]` (URN is **not** a surface); gloss via `define_word`;
`lookup("justice")` does **not** resolve by a single heading word. The descriptive "the first section"
(Russell) needs the projected `num` ordering — a parked extension this design enables.

## Verdict
Pillar 1 (one reasoner, every load) is ~95% — one wire (Step 1) over existing, tested machinery,
and it dissolves the exact symptom. Pillars 2–3 (self-model + page) are ~60% until the completeness
additions land (abstention, success-aware provenance, capabilities, double-count invariant,
fail-closed recompose). All loaded-not-encoded, reusing `RuntimeOntology`/`ConceptRef`/`OntologyName`/
`SelfModelInstance`/`ContentAddress`/`GraphSnapshot` — no forked mechanisms.

---

## 11. Phase A, the ONTOLOGICAL design (supersedes §10's mechanical spec)

A design pass (Datalog/OWL-RL/categorical/rule-algebra) settled how to realize the runtime transitive
closure *ontologically*. **Verdict: MERGE-READY** with a sequencing correction.

**The honest principle.** A transitive closure IS a least fixpoint; the saturation loop is
**irreducible** — every authority (Datalog `T_P`, OWL-RL materialization, categorical `Reach(Q)`,
rule algebra) is `while changed { fire }`. The standard is therefore NOT "delete the loop." It is: the
loop must be a **generic, rule-agnostic engine**, and *which kinds are transitive* + *how a kind is
identified* must be **loaded data, not Rust**.

**Today: 2 wrong, 1 right.**
- ❌ vocabulary: `RelationKind` closed enum + dropping `match` (`ontology.rs:91-123`).
- ❌ policy: hardcoded `RelationKind::transitive()` array (`ontology.rs:126-132`).
- ✅ engine: `ReachabilityClosure::fold` (`quiver.rs:319-387`) is ALREADY the generic saturator (the
  legitimate `Reach(Q)` image, shared with English's closure). **It stays unchanged.** A generic
  interpreter is allowed to be Rust (like `apply`'s table-walk); the violation is the *rule + vocabulary*
  being Rust.

**The substrate already exists, unused.** `domains/.../formal/relations/ontology.rs:65-89,185-209`
declares the relation kinds as **concepts** (`Subsumption`, `Parthood`, `Causation`, `Precedence`,
`Equivalence`, …) and their transitivity as loaded typed edges `(R, Transitive, HasProperty)`;
`RelationProperty::get` already derives transitivity by querying those morphisms. **The runtime never
reads it.**

**The design.**
- **relation kind = `ConceptRef("Relations", name)`.** The edge's kind-name string stays byte-exact on
  the wire (content-address preserved — `definition.rs:158,172-179`) but RESOLVES to a concept, not an
  enum or a `String` map key. Cross-ontology sharing via the one Relations vocabulary.
- **transitivity = the loaded `HasProperty Transitive` edge** (OWL-RL `Transitive(R)`). Re-emit
  `Relations.prx` to change which kinds close; never edit Rust. NOT a `GeneratorAction::Rule` — the
  property edge is the minimal OWL-RL-canonical form, and `rule_algebra::Implication` has no fixpoint
  consumer (only point-checks).
- **closure = the unchanged generic `ReachabilityClosure::fold`, driven by the queried transitive-kind
  set** — not the closed enum, not the hardcoded array, not a string map, not a new fold. **`apply` is
  NOT the interpreter** — it only relabels and provably cannot mint the derived arrow (`category_projection.rs:14-21`);
  forcing the closure through it would relocate the loop and lie about it.
- types: `RuntimeEdge.kind: ConceptRef`; `MaterializedClosure.reachable: BTreeMap<ConceptRef, ReachabilityClosure<ConceptRef>>`;
  `fold(edges, transitive: &BTreeSet<ConceptRef>)`; query surface (`reaches`/`reachable_from`/`is_a`)
  takes a `ConceptRef` kind; bodies (O(1) lookup) unchanged. `materialize` drops the `from_edge_kind`
  filter (`:571`) and threads the loaded transitive set.

**Critique correction (the one real risk).** The macro *independently* hardcodes the same 3 kinds
(`pr4xis-derive/ontology.rs:668-693`), and `pr4xis-derive` has **no** data-load path (the TSV/include
pattern lives in `domains`, which derive can't dep). So if Step 3 expands the runtime to all 7
transitive kinds first, compile-time and runtime closures **diverge** for 4 kinds (Precedence/
Equivalence/Specialisation/Dependence), breaking the cross-tier invariant (`ontology.rs:88-90`).
**Sequence:** Step 3 reads ONLY the 3 macro-matching kinds initially; the macro is unified (a genuine
new build-time `Relations.prx` load) BEFORE the runtime expands to all 7.

**Migration (smallest-first; no committed `.prx` exists, wire bytes unchanged):**
1. Emit `Relations.prx` (the relation-kind concepts + `Transitive` edges); test it contains `(Subsumption, Transitive, HasProperty)`.
2. Add `transitive_kinds(relations: &RuntimeOntology) -> BTreeSet<ConceptRef>` (the `prp-trp` query); test it returns the declared kinds.
3. Re-key `RuntimeEdge.kind`/`MaterializedClosure` to `ConceptRef`; delete `RelationKind`+`from_edge_kind`+`transitive()`; thread the queried set (3 kinds first); port the acceptance tests (`RelationKind::Subsumption` → `ConceptRef::new("Relations","Subsumption")`).
4. Unify the macro to read `Relations.prx` (new derive-time data load) — then expand to all 7.
5. (optional) lift `Transitive` into `meta.rs` so transitivity is bootstrap-available without loading a separate `.prx`.
