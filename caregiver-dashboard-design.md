# Caregiver Evaluation Bench — Design

> **Superseded in part, 2026-07-25.** The two-page split this record designs (a standalone `/caregiver.html` beside the generic `/`) was merged back into ONE app: `docs/chat/index.html` now carries the caregiver surface as a hash-routed tab (`#caregiver`, `#caregiver/tracks/1|2`, `#caregiver/ask|evidence|method`) on a single engine boot, and `/caregiver.html` no longer exists as a file or a URL. Everything below is the historical design as written; read every `/caregiver.html` and `docs/chat/caregiver.html` reference in it as `#caregiver` inside `docs/chat/index.html`.

**Scope**: the ACL Caregiver AI Challenge demo surface redesign (deadline 2026-07-31). One buildable design. Every choice cites an inward anchor (`file:line`) or a named outward source. Baseline: the two prior praxis-design-pass outputs *as amended by their adversarial critiques* (scratchpad: `why_layer_synthesis.md` + `why_layer_critique.md`, `redesign_synthesis.md` + `redesign_critique.md`). Rejected alternatives from those passes (frontend-JS sentence assembly, parsing the trace string, a bare `Track` enum, prettifying the string-split trace) are not re-litigated here.

**Honesty rules (non-negotiable, restated because every section below is checked against them)**: (1) nothing hardcoded against specific questions; (2) sample questions drawn at runtime from real corpus data; (3) every displayed number live-computed in-browser or paired with the exact re-deriving test command; (4) live reproduction of the published Smart-40 protocol is encouraged, pre-staging its results is not; (5) TRL 3+ research-prototype disclosure stays prominent; modest framing.

---

## 0. FORM DECISION

### Candidates evaluated

**(i) Single caregiver dashboard tab** (current shape: third tab inside the generic page, `docs/chat/index.html:318-322`).
- *For*: zero structural work; one engine boot.
- *Against*: keeps caregiver branding inside the generic product, directly against the maintainer's new direction (chat becomes generic). Fails the judge's first 10 seconds (NN/g dwell-time analysis: value proposition must be above the fold; a judge lands on a generic "pr4xis" self-model tab, not their track). Fails the track-mismatch test outright: Track 1 and Track 2 are judged as *separate applications by different judge pools* (acl.gov judging pages, live-fetched in `redesign_critique.md`), and a merged tab addresses neither pool in its own vocabulary.
- *Score*: weakest.

**(ii) Two fully separate track surfaces** (two pages, one per track).
- *For*: perfect judge fit — a Track 2 judge never sees family-caregiver framing; matches GSA Prize & Challenge Toolkit rubric-driven scoring (judges score only against their announced criteria).
- *Against*: duplicates every live exhibit, engine boot, and chat renderer across two files in an 8-day runway; the two tracks share ~everything real (same engine, same corpus file with track tags, same Smart-40 protocol, same wire protocol), so duplication is pure drift risk. The corpus itself says the split is a *tag*, not a partition — 528 questions are tagged `both` (corpus fixture tally, `caregiver_question_corpus.json`).
- *Score*: right goal, wrong mechanics.

**(iii) Guided evaluation-tour** (a linear wizard walking the judge exhibit-by-exhibit).
- *For*: golden-path narrative order is research-backed (Storylane/Reprise demo-engineering guidance; YC narrative-first advice).
- *Against*: rubric-driven judges are non-linear scanners — GSA's toolkit says judges work from a review template with per-criterion questions; a wizard that hides exhibit N+2 behind exhibit N+1 fights that. A tour rail also risks a third disclosure level (NN/g: usability deteriorates beyond two) and reads as choreographed marketing, which the trust literature says measurably damages credibility for health information (Sbaffi & Rowley 2017: advertising/marketing tone is the single most trust-damaging factor).
- *Score*: good as an *ordering principle*, wrong as the *form*.

**(iv) CHOSEN — one "Caregiver Evaluation Bench" page with two hash-routed track lenses.**
A dedicated page (`docs/chat/caregiver.html`, deployed as `/caregiver.html`), separate from the de-caregiver-ed generic chat, whose **section order is the golden path** (problem → live instrument → validation → known limits → method/evidence) and whose **track lens** (`#track1` / `#track2`) is a pure, data-driven filter:

- The lens sets the hero title and track pill ("Track 1 — Grounded Care Navigator" / "Track 2 — EVV/HCBS Compliance Navigator", wording already on-page today at `docs/chat/index.html:331-343`).
- The lens filters the corpus sampler by the corpus's own `track` tag (`track1_family`+`both` vs `track2_workforce`+`both`) — data-driven per honesty rule 1, since the tags live in the shipped corpus file, not in page code.
- The lens reorders nothing else and hides nothing: both narrative PDFs stay linked, the Smart-40 console is identical under both lenses (the protocol is track-agnostic by construction).
- No hash → a compact two-card track chooser at top; everything below works in "all tracks" mode.

*Why this wins*: it delivers (ii)'s judge fit — each narrative links its own lens URL, so a Track 2 judge lands on a page whose first sentence names Track 2 — at (i)'s cost, one page and one engine boot. It uses (iii)'s golden path as the page's reading order rather than as a navigation cage, satisfying both NN/g's 10-second rule (track name + one live proof above the fold) and GSA's scanner-judge model (a rubric map band, §2.6, lets a judge jump straight to the exhibit for the criterion they are scoring). Per-answer track scoping uses the already-crossing `result.ontologies` list, never a bare Track enum (decision carried from `redesign_synthesis.md` §1b, `SourceTaxonomy` anchor `crates/domains/src/formal/meta/source_taxonomy/ontology.rs:184`).

**Coordination constraint discovered during this pass**: the Track 2 narrative cites the demo page *by line number* (`docs/chat/index.html:334-335`, `:337-338`, `:346` — verified in `docs/caregiver-challenge/track2-phase1-narrative.md:33,108`). Restructuring the page invalidates those citations. The narratives are built from markdown in CI and the submission is not yet sent (blocked on org/UEI per memory), so Slice 5 regenerates both narratives with updated anchors and the new lens URLs. This is mandatory, not optional.

---

## 1. Information architecture

### 1.1 Generic chat page (`docs/chat/index.html` → deployed `/index.html`) — de-caregiver-ed

Content inventory after the change:

| Keep | Change | Remove |
|---|---|---|
| Header: pr4xis logo + status pill (`index.html:310-316`) | Tab bar becomes two tabs: **Chat** (default) and **Engine** (the current self-model dashboard, `renderDashboard` `index.html:892-1318`, unchanged) | Caregiver tab + label from page-level tab bar (`index.html:321`) |
| Chat log + shared chat renderer (outcome cards, Why? layer, trace accordion — §5) | `#input-area` moves *inside* the chat tab content (kills the display-toggle quirk at `index.html:398-401,652`) | Caregiver-flavored comments in the generic path (`index.html:592-594`) |
| Worker RPC (`docs/worker.js`, already fully generic) | Placeholder: generic ("Ask a question — e.g. is a dog a mammal?") replaces the current text at `index.html:399`; boot message de-branded (`index.html:535-540`) | `initialize()` coupling that enables caregiver controls (`index.html:545-547`) |
| Sources catalog + loaders (`index.html:1045-1284`) | A single prominent link card above the chat: "ACL Caregiver AI Challenge — evaluation bench →" linking `/caregiver.html` | Unconditional `renderCaregiverStatus()` at load (`index.html:528`) |
| | `addMessage` takes an explicit target log (no default to main chat, `index.html:450`) | Track example button groups (`index.html:350-383`) — replaced on the bench by the runtime sampler |
| | Dead CSS: `.narrative-link.pending` (`index.html:196-198`) deleted | |

### 1.2 Caregiver Evaluation Bench (`docs/chat/caregiver.html` → deployed `/caregiver.html`) — full inventory

1. **Hero band**: track pill (lens), one-sentence what-it-is, TRL 3+ summary box, "runs entirely in your browser" pill, engine-load progress.
2. **Live instrument band**: question input + uncurated corpus sampler + predict-then-reveal + full outcome rendering (all four ChatOutcome variants, §5).
3. **Validation band**: Smart-40 live console; "run a live slice" (N random corpus questions); measured-capability strip (moved from `index.html:505-528`).
4. **Known-limits band**: gap counters with re-derive commands; "show me a question we currently fail" (drawn from snapshot labels, run live).
5. **Method & evidence band**: model-card sections; AI-nutrition-facts disclosure box; rubric map; narrative + Smart-40 PDF links; WCAG self-audit panel.
6. **Footer**: reproduce-everything command list, repo link, state fingerprint (`self_describe().state_cid`).

Shared assets: `docs/chat/tokens.css` (design tokens + components, single source for the palette gate test) and `docs/chat/chat-ui.js` (shared chat renderer used by both pages). CI copy step extended (`.github/workflows/ci.yml:616-623`): `caregiver.html`, `tokens.css`, `chat-ui.js`, plus the two new data files (§3) join the assembly.

---

## 2. Bench layout, section by section

Text wireframe (one column, max-width ~72rem, sections are full-width bands):

```
┌────────────────────────────────────────────────────────────────────┐
│ pr4xis logo   Caregiver Evaluation Bench      [engine: loading ▓▓░]│
│                                                                    │
│ TRACK 2 — EVV/HCBS COMPLIANCE NAVIGATOR        (lens pill, from #) │
│ A citation-grounded reasoning engine that answers from loaded      │
│ statute/regulation definitions — and abstains, naming what it      │
│ does not know, when none is loaded.                                │
│ ┌─ RESEARCH PROTOTYPE ────────────────────────────────────────┐    │
│ │ • Technology Readiness Level 3+ — validated proof of concept│    │
│ │ • Under live test with one customer (alpha)                 │    │
│ │ • Runs entirely in your browser. No server. No account.     │    │
│ │   No question you type ever leaves this device.             │    │
│ │ • Every number on this page is computed here, now, or shows │    │
│ │   the exact test command that re-derives it.                │    │
│ └─────────────────────────────────────────────────────────────┘    │
├────────────────────────────────────────────────────────────────────┤
│ ASK THE ENGINE                                                     │
│ [ input …………………………………………………………… ] [Ask]                            │
│ From the evaluation corpus (drawn at random, uncurated):           │
│ (chip) "Is Hospice included in EVV?"  #3960 · track2 · state FAQ   │
│ (chip) "…"  (chip) "…"        [↻ draw 3 more]  [predict: ans/abs?] │
│ ── answer stream: outcome cards + Why? + trace (see §5) ──         │
├────────────────────────────────────────────────────────────────────┤
│ VALIDATION — reproduce our published evidence, live                │
│ ┌ Smart-40 Validation Protocol (as submitted to ACL) ┐             │
│ │ [Load published sources…] [Run all 40 in my browser]│            │
│ │  1 Standard  "What is dementia?"   published:Answered  you:—     │
│ │  …40 rows, task-list style, statuses fill as engine returns…     │
│ │  agreement: 38/40 (computed live; mismatches shown in red)       │
│ └ re-derive offline: cargo test -p praxis-corpus-tests             │
│      probe_smart40_validation_log -- --nocapture ┘                 │
│ ┌ Live corpus slice ┐  ┌ Measured capability (snapshot) ┐          │
│ │ [Run 25 random]   │  │ 4,121/4,617 green · 4 gap ctrs │          │
│ │ ▷ outcome tiles   │  │ chip: cargo test …regenerate_… │          │
│ └───────────────────┘  └────────────────────────────────┘          │
├────────────────────────────────────────────────────────────────────┤
│ WHAT IT CANNOT DO YET (known limits, measured)                     │
│ 218 missing-term · 186 unparsed · 68 over-answered · 24 misroute   │
│ [show me one we fail →]  (draws one, runs it live, fails honestly) │
├────────────────────────────────────────────────────────────────────┤
│ METHOD & EVIDENCE                                                  │
│ model card · AI facts label · rubric map · PDFs · WCAG self-audit  │
├────────────────────────────────────────────────────────────────────┤
│ Reproduce everything: (command list) · state: bafy… · repo link    │
└────────────────────────────────────────────────────────────────────┘
```

### 2.1 Hero / status strip

- **Form**: USWDS *summary box* pattern (`role="region"` + `aria-labelledby`, tinted background, border) — USWDS's own guidance draws exactly the needed line: summary box is for *standing* key information; alerts are only for new/changed state. The TRL disclosure therefore lives here permanently, never in a dismissible alert. (USWDS Summary box component.)
- **Copy discipline**: plain language, ≤20 words/sentence, active voice (CDC plain-language guidance); the four bullets in the wireframe are the entire hero — no adjectives, no feature list (Sbaffi & Rowley 2017; repo rule `feedback_modest_framing`).
- **Engine load**: determinate progress with named stages ("downloading engine (16 MB)… installing lexicons… ready"), driven by the worker's existing per-chunk progress events (`docs/worker.js:95-116`). NN/g response-time limits: a frozen first paint is an abandonment event.
- The "no question leaves this device" line restates the Track 2 narrative's structural-privacy claim (`track2-phase1-narrative.md:17`) — a headline feature per the Hugging Face Spaces zero-setup-reviewer affordance and AARP's offline/broad-inclusivity checklist item.

### 2.2 Live metrics tiles (every number: live or commanded)

| Tile | How derived |
|---|---|
| Concepts loaded | `pr4xis.concept_count()` live (`crates/wasm/src/lib.rs:782`) |
| Ontologies loaded | `loaded_ontology_count()` live (`lib.rs:863-865`) |
| Corpus questions | `length` of fetched `caregiver-corpus-slim.json`, live |
| This track's slice | live filter count over the slim corpus `track` field |
| Snapshot pass rate | computed live from slim-corpus per-question labels (Green/total); labeled "snapshot of <date>" + command chip `cargo test -p praxis-corpus-tests regenerate_caregiver_snapshot` (the command already named in `docs/caregiver-corpus-status.json` `regenerated_by`) |
| Session ledger | live counts of Answered / Abstained / Conditional / RuleResolved for this session (Amershi G2; Kompa et al. 2021 abstention-statistics argument) |
| Engine memory | `self_describe().linear_memory_bytes` live (`lib.rs:997-1001`) |

Rule for the whole page: any number not computable in-browser renders as a **command chip** (monospace, copy button) under the figure — the dashboard-native form of `feedback_cite_the_test`, mirroring HELM's published-rerun contract.

### 2.3 Live exhibits — see §3 (the six WOW exhibits live in bands 2–5).

### 2.4 Citation display

- Per-answer **source chips** from `result.ontologies` (kind `loaded` marked, `lib.rs:763-777`), displayed via the plain-label projection table (§5.2) — never raw `caregiving_lexicon` snake_case (why-layer critique, break 1).
- Conditional/rule outcomes render `rule_citation` (source text or bluebook subsection, `lib.rs:699-757`) as a quoted block inside the outcome card.
- After the typed-trace fix, trace nodes carry `FunctorConnection.reference` — the literature citations ("Shannon 1948", "Kamp 1981") currently computed then thrown away at the flatten (`trace_functors.rs:269-297,477-483`) — rendered as small cite tags on trace rows. Pattern: Shape-of-AI citation stack (inline markers + chips + expandable source list), with unresolved groundings shown explicitly, not hidden.

### 2.5 PDF / narrative links

One evidence row in the Method band: Track 1 narrative PDF, Track 2 narrative PDF, Smart-40 validation log PDF (all already deployed under `/caregiver-challenge/`, `ci.yml:616-623`). The lens promotes the current track's narrative to first position. The dead `.pending` style (`index.html:196-198`) does not migrate.

### 2.6 Rubric map ("judge's map") + gap/honesty surface

- A compact table mapping each announced criterion to its on-page exhibit anchor, using the *verified* rubric structure (Transparency / Empowerment / User Error Reduction / Usability / Integration / Interoperability are sub-criteria of "4. Usability and Integration"; plus Responsiveness to Need, Implementation, Partnerships, Alignment with AI Principles — per the live-fetched acl.gov pages in `redesign_critique.md`). GSA toolkit: judges score strictly against announced criteria; exhibits that don't map earn nothing.
- The **known-limits band** is a first-class section, not a footnote: the four gap counters (from `caregiver-corpus-status.json`, fetched as today at `index.html:505-528`) each with the regenerating command, plus the "show me one we fail" control (§3.4). Anthropic Transparency-Hub pattern: capabilities and known limitations on one structured surface; PAIR "set the right expectations".

---

## 3. WOW EXHIBITS (all client-side; feasibility stated per exhibit)

### 3.1 Smart-40 Live Console — "reproduce the ACL-submitted protocol in your browser"

- **Judge sees**: a GOV.UK task-list of the exact 40 published inputs (28 corpus-by-index + 4 messy + 4 adversarial + 4 conditional — `scratch_probe.rs:6346-6449`), every status a grey "Not run" tag until the real engine returns it; a *published outcome* column beside a *your run* column; a live agreement counter; mismatches rendered in red, never suppressed. Framed on-page as ACM-style artifact evaluation ("Results Reproduced" = re-obtained by someone other than the authors, run-to-run variation tolerated — SIGMOD reproducibility genre).
- **Criteria**: Transparency, Implementation credibility, Alignment with AI Principles.
- **Why honest**: rule 4 verbatim — same specified inputs, outcomes computed fresh, never pre-labeled; the published column is clearly labeled as the shipped log, and disagreement displays as disagreement.
- **Data**: `docs/smart40-protocol.json` — emitted by the *same probe* that writes `smart40_validation_log_dump.json` (repo root, 40 entries with category/question/source/outcome — verified this session), so protocol and published log share one source of truth. Ship the dump as-is; the page treats its `outcome` field as "published log" only.
- **wasm API needed (does not exist yet)**: `chat_batch(questions_json) -> String` running each question statelessly via `pr4xis_chat::process_with_reasoner` against `self.composed` (mirror of the native probe, `scratch_probe.rs:6340`), plus `reset_session()` (one line, `crates/chat/src/session.rs:55-90` has no reset today). A naive JS loop over `chat()` is UNSAFE: `chat()` is stateful and a Conditional turn's pending rule consumes the next call as a slot-fill (`lib.rs:685-688` + `session.rs:81-86`).
- **Fidelity caveat, shown on-page**: the native run's reasoner includes USC-with-defines-overlay when title XML is on disk (`crates/praxis-corpus-tests/src/caregiver.rs:65-121`); the browser default is English + LegalSources + 2 care lexicons. The console offers a preflight "Load the sources the published run used" step via `available_usc_archives()` + `load("rkyv-archive")` (archives bake the defines overlay, `crates/wasm/build.rs:484-567`); skipping it shows a banner "running without X — differences will appear below" — honest either way.
- **Worker**: one new message type each for `chat_batch` and `reset_session` (`docs/worker.js:31-77`), chunked with progress yields.

### 3.2 Uncurated corpus sampler with provenance + predict-then-reveal

- **Judge sees**: three question chips drawn uniformly at random (`crypto.getRandomValues`) from the real corpus, filtered only by the lens's track tag; each chip shows corpus index + track + source domain as provenance ("#3960 · track2 · state Medicaid FAQ"); a "draw 3 more" shuffle; an optional "will it answer or abstain?" predict toggle before running (Distill predict-then-reveal: self-explanation measurably improves engagement and learning). The index is the honesty proof — anyone can check the same index in the public fixture.
- **Criteria**: Transparency, Responsiveness to Need, Empowerment (recognition over recall — AARP cognitive checklist).
- **Why honest**: rule 2 by construction — runtime random draw from the shipped corpus, no curation; failures surface naturally and render as reasoned refusals (§3.5).
- **Data**: `docs/caregiver-corpus-slim.json` `{q, track, capability, topic, label, source}` (~778 KB raw / 141 KB gzip, measured this session), generated by a new test in `praxis-corpus-tests` alongside `regenerate_caregiver_snapshot` so the file carries a `regenerated_by` command. Ship `docs/adversarial-corpus.json` (160 authored questions, no licensing exposure, `src/adversarial.rs:54-72`) the same way.
- **wasm API**: none — existing `chat()`.
- **Feasibility**: generator test + fetch + render; 1 day inside Slice 3.

### 3.3 Conditional-rule walkthrough — live human-in-the-loop slot-filling

- **Judge sees**: typing "is a car eligible for the assets" (or drawing one of the four published uncertainty inputs) produces a Conditional card: the rule name, its cited definition (`rule_citation`), and each `missing_facts` item rendered as an input prompt; answering routes back through the same stateful session to a RuleResolved verdict card. The engine only asks when it genuinely lacks a premise — the "timely clarification" condition the mixed-initiative IR literature ties to satisfaction gains (Aliannejadi et al.; ACL DialDoc 2022).
- **Criteria**: Empowerment ("supports human judgment as opposed to replacing it" — verbatim rubric), User Error Reduction, Transparency.
- **Why honest**: pure rendering of wire fields computed live per turn; nothing per-question.
- **Data/wasm**: NONE NEW — `conditional` and `rule_applies`/`rule_does_not_apply` outcomes with `rule_name/rule_definition/rule_citation/missing_facts` already cross the wire and are *never rendered today* (`lib.rs:699-757`; `index.html:586` renders only `abstained`). The session statefulness that makes multi-turn resolution work already exists (`session.rs:81-86`). This is the cheapest wow on the list: the engine capability is finished and invisible.

### 3.4 "Show me a question we currently fail" — honest-gap probe

- **Judge sees**: a button in the known-limits band that samples one question whose *snapshot* label is a gap class (MissingTerm / UnparsedKnownTerm / OverAnswered / PossibleMisroute), states the snapshot's expectation in plain language, runs it live, and shows what actually happens — including when the engine has since improved and the snapshot is stale (that too renders honestly, with the regenerate command).
- **Criteria**: Alignment with AI Principles, Transparency, Implementation credibility (a team that exhibits its failures is running a real test program — trust-calibration studies show exposing limitations improves calibrated trust, arXiv 2605.18036).
- **Why honest**: labels are harness-computed data (snapshot file, same order as fixture — `caregiver_question_corpus.snapshot.json`), not curation; the outcome shown is always the live one; the label is displayed as "snapshot of <date>" with its command.
- **Data**: the `label` field already in the slim corpus (§3.2). **wasm API**: none.

### 3.5 Adversarial open mic — reasoned refusal on judge-typed input

- **Judge sees**: an invitation in the instrument band: "Try to make it guess — invent a statute, a program, a fake threshold." Plus chips drawn randomly from the 160 authored adversarial questions (four categories shown as neutral tags). An abstention renders as a GOV.UK error-summary-shaped **reasoned refusal**: plain statement, the named unresolved terms from `unresolved[]`, and a next step (load a source / try a definable term) — never a bare "I don't know" (abstention-survey literature: reasoned refusals with a next step outperform bare denials; CERTA study: honest low-certainty communication builds calibrated trust).
- **Criteria**: User Error Reduction (verbatim rubric: "designing to prevent user error"), Alignment with AI Principles, Transparency.
- **Why honest**: the judge types their own input; outcome computed live; adversarial chips are runtime-random over the shipped authored fixture. The existing live-only `.limit-tag` discipline (`index.html:150-153`) carries over unchanged.
- **Data**: `docs/adversarial-corpus.json` (§3.2). **wasm API**: none.

### 3.6 The engine audits its own page — live WCAG self-check

- **Judge sees**: a small panel in the Method band: "This page's colors are verified by the engine's own cited color ontology." It lists each axiom (name, citation from `axiom_meta!` — "W3C WCAG 2.1 SC 1.4.3 (Contrast Minimum)"), its live verdict, and the numeric contrast ratios for the rendered fg/bg pairs, computed by the wasm engine from the page's *actual* computed styles (`getComputedStyle(...).getPropertyValue('--base05')` etc.).
- **Criteria**: Usability, Integration (the demo integrates its own reasoning machinery into its skin), Transparency.
- **Why honest**: nothing asserted — the ratios are computed at view time from the real DOM tokens by cited code (`srgb::contrast_ratio`, `crates/domains/src/natural/colors/srgb.rs:50-255`); if a future edit breaks contrast, the panel shows the failure.
- **wasm API needed (does not exist yet)**: `verify_palette(css_vars_json: &str) -> String` on `Pr4xis` — builds a `Palette` via `ColorSlot::variants()` key-matching + `Rgb::from_hex` (`base16.rs:22-159`, `rgb.rs:49-105`), runs `WcagForegroundContrast`, `LuminanceMonotonicity`, `detect_polarity`, the new muted-pair axiom (§4), and per-pair `srgb::wcag_compliant`; returns a Presentation record list. Pure addition — `crates/wasm` already depends on `pr4xis-domains` (`crates/wasm/Cargo.toml:24`). Simultaneously cures the theming ontology's orphaned-mechanism debt (zero callers today; repo rule `feedback_no_orphaned_mechanisms`).

---

## 4. Visual design direction

**Aesthetic target**: a serious research instrument — closer to a government evaluation report than a SaaS landing page. Credibility devices: federal-familiar tokens, visible derivation commands, dense-but-scannable evidence tables. No hero gradients, no testimonials, no adjectives.

### 4.1 Tokens (single source: `docs/chat/tokens.css`)

- **Naming**: `:root` custom properties keyed exactly by `ColorSlot::key()` — `--base00 … --base0F` (+ base24 bright slots if used) — plus semantic aliases keyed by `Vogix16Semantic::key()` (`--danger`, `--success`, `--warning`, `--link`, …) referencing base slots via `var()`. This makes the CSS parseable straight into the ontology's `Palette` (HashMap<ColorSlot, Rgb>, `theming/ontology.rs:17`).
- **Values**: USWDS-derived. Dark variant (default): keep the current GitHub-dark-adjacent grounds (`--base00: #0d1117`) but replace the two failing muted greys — `#484f58` (2.28:1, severe: `index.html:101,122,181,297,626`) and `#6e7681` (4.12:1 borderline: `:197,244`) — with axiom-passing values; accent `--base0D` = USWDS primary blue family; outcome hues from USWDS state-token families mapped per variant: Answered→success greens, Abstained→**warning** golds (abstention is correct behavior, not an error), Conditional→info cyans, engine fault only→error reds (USWDS state color tokens + Alert anatomy). Light variant: USWDS light defaults (#005ea2 primary on white, gray-90 ink). Exact hexes are chosen *by the gate test*, not in this document — candidates go through the axioms and the failing ones are rejected (redesign_synthesis §1c: query the axiom, don't hand-pick hex).
- **Dark/light**: `prefers-color-scheme` + a manual toggle stamping `data-theme`; the two palettes ship as a `ThemePackage` ("caregiver-bench", `add_variant("dark", Polarity::Dark, …)` / `("light", Polarity::Light, …)`) whose `validate()` must be empty (`theme_package.rs:22-84`); the toggle's semantics are the ontology's `VariantSet` darker/lighter navigation (`variants.rs:29-176`). The already-deployed-but-unreferenced light logo assets get wired (`ci.yml:633,635`).

### 4.2 The gate test (build-time, blocks CI)

New `crates/web/tests/palette_wcag.rs` (precedent: `crates/web/tests/worker_contract.rs:18-31` already reads `docs/` files via `CARGO_MANIFEST_DIR` and fails on drift; `crates/web` gains a `pr4xis-domains` dev-dependency):

1. Parse `--baseNN: #hex` pairs from `docs/chat/tokens.css` for both variants; build `Palette` via `ColorSlot::variants().iter().find(|s| s.key() == name)` + `Rgb::from_hex`.
2. Assert (ontological-assertion style, `feedback_ontological_assertions`): `WcagForegroundContrast{palette}.verify().is_ok()`, `LuminanceMonotonicity{...}.verify().is_ok()`, `detect_polarity == Some(expected)`, and `ThemePackage::validate().is_empty()` across both variants.
3. **New axiom required** (critique-confirmed gap): `WcagForegroundContrast` covers only base05-over-base00 (`theming/ontology.rs:249-271`). Add one same-pattern, palette-parameterised axiom in `theming/ontology.rs` — `RenderedPairsMeetAa` — that iterates an ontologically-declared list of rendered semantic pairs (muted-text-on-bg at 4.5:1; large-text/UI pairs at `WcagLevel::min_contrast_large()` 3:1; disabled states exempt per SC 1.4.3), cited to W3C WCAG 2.1 SC 1.4.3/1.4.11. The test wraps its `verify()`.

### 4.3 Type, spacing, components

- **Type scale** (custom properties): body 17px (USWDS `md`), floor 16px everywhere including trace text (ODPHP Health Literacy Online 5.3), meta text ≥16px (not the current 0.7em toggle at `index.html:100-114`), headings 22/32px; line-height ≥1.5; sans-serif, left-aligned (NIA senior-friendly checklist). Font: `system-ui` stack default; optionally Public Sans (SIL OFL, USWDS's face) embedded as a woff2 `data:` URI — a knob, not a dependency (GDS Transport is license-restricted; do not use).
- **Targets**: 44×44 CSS px design size, 24×24 hard floor (WCAG 2.1 SC 2.5.5 AAA / 2.2 SC 2.5.8 AA); spacing on an 8px scale with generous empty space around actives (AARP dexterity checklist).
- **Focus**: 3px solid high-visibility focus token on every interactive element (GOV.UK focus-state signature; WCAG 2.4.7).
- **Components** (hand-transcribed, ~200 lines, class names descriptive — `alert/card/tag/summary-list/task-list/accordion` — never `usa-*`/`govuk-*` branding; USWDS is CC0, govuk-frontend MIT, so transcription is legal and CSP-safe): USWDS alert anatomy for outcome cards (8px left border + tinted bg; `role="status"` for outcomes, `role="alert"` only for engine faults); GOV.UK tag for statuses (non-interactive, sentence case — GDS research verbatim); GOV.UK task list for the Smart-40 console; GOV.UK summary list for per-question evidence rows; USWDS bordered accordion for the trace ("don't hide critical information" — outcome stays visible, only stage detail collapses).
- **Responsive**: single column under 640px; tables/task-lists scroll inside their own `overflow-x: auto` containers; header flex-wrap (carried from Slice 0 of `redesign_synthesis.md`).
- **Progressive disclosure**: hard two-level cap (NN/g): level 1 = answer + outcome card + Why? sentence; level 2 = technical trace accordion. Nothing deeper.

---

## 5. Chat improvements shared by both pages (`docs/chat/chat-ui.js`)

### 5.1 Outcome rendering — complete the wire protocol

Today only `abstained` gets special UI (`index.html:586`); `conditional`, `rule_applies`, `rule_does_not_apply` and all `rule_*` fields render as plain text (inward gap). The shared renderer maps every variant to its card: Answered (success card + source chips), Abstained (reasoned-refusal card, §3.5), Conditional (slot-fill card, §3.3), RuleResolved (verdict card + citation). Card *style* keys off the wire variant (structurally honest — no per-question anything); card *sentences* come off the wire, never from JS literals.

### 5.2 The Why? layer (Design 1 as amended by its critique — build exactly this)

- Third loaded `RealizationFrameTable` (`crates/domains/data/grammar/explain-frames.tsv` → `.prx`, registered in `praxis.toml` mirroring `praxis.toml:610-632`), keyed by the existing `ResponseFrame` — no parallel outcome enum.
- `realize_why(content, reasoned_over) -> Option<String>` beside `realize()` (`pragmatics/realize.rs`); new `why: Option<String>` sibling field through `ResponseResult`/`ProcessResult` (~15 co-located construction sites verified) + one `p.set("why", …)` in wasm.
- **Critique fixes are mandatory**: (a) an `OntologyName → plain-English label` table (same .tsv/.prx/OnceLock shape) so `{sources}` never emits `caregiving_lexicon` raw, generic fallback "your loaded documents"; (b) distinct frame ids `assert_knowledge_grounded`/`assert_knowledge_ungrounded` + a no-duplicate-frame_id test (loader is first-match, `abstain_frames.rs:88-90`); (c) the empty-`reasoned_over` template states the truth — answered from the compiled English substrate — with the real three-way split (empty / built-in lexicon / user-loaded); (d) a test asserting rendered why-text never contains un-projected snake_case identifiers.
- Frames whose primary response already self-explains (`UnprovenRelation`, `RuleAwaitingFact`, `RuleApplies`, `RuleDoesNotApply`, `Comparison`, `PhaticReturn`) get no row, by design.
- New `[sources.*]` entries trigger the 2-pass registry/ROOT_HEX regen lifecycle (`project_data_source_url_change_lifecycle`).
- **Placement**: answer → outcome card → low-emphasis "Why?" toggle → plain sentence → dimmer "Show technical pipeline" → the trace accordion (now rendering the *typed* trace). Never derive the plain layer from the trace string.

### 5.3 Typed trace (Slice 1, the one contained wasm change)

Replace the flatten at `crates/wasm/src/lib.rs:690` + `p.set("trace", …)` at `:778` with `SchemaValue::List` of Records — fields `ontology`, `operation`, `phase` (MAPE-K), `detail`, `success`, `reasoned_over`, `functor_connections[{target, functor, reference}]` — the identical Presentation idiom the `ontologies` field already uses at `lib.rs:763-777`. Everything needed exists typed (`trace_functors.rs:231-239,329-412`). `addTraceAccordion`'s string-split (`index.html:462-498`) is replaced by a structured renderer in `chat-ui.js`; warn/err steps keep a distinct hue from the token set. This surfaces the per-step literature citations for free (§2.4).

### 5.4 Error/abstention presentation

Abstained = calm warning tones, never error red (abstention is designed safety behavior — HELM treats calibration as headline; Amershi "when wrong" group). Identical failure wording at summary and point of failure (GOV.UK error pattern). Engine faults (worker error path, `index.html:555`) are the only red. Copy at 6th–8th-grade level: "We can't answer this yet — we have no loaded definition of X" (CDC plain-language; AARP error-message checklist: no cryptic codes).

---

## 6. Build plan — ordered slices, each independently shippable

Runway: 2026-07-23 → 07-31. Slices are separate commits; each leaves the deployed site whole. `feat!:` prefixes where the wasm wire changes (`feedback_pr_commit_prefix`). Run `devenv ci` before each commit.

**Slice 0 — Split, tokens, gate, a11y (Days 1–2; no wasm).**
- `docs/chat/caregiver.html` NEW: bench skeleton (hero/TRL box, lens routing on `location.hash`, instrument band wired to existing `chat()` via existing worker, validation band hosting the moved status strip, method band with PDF links).
- `docs/chat/index.html`: de-caregiver per §1.1 table; input-area moved inside tab.
- `docs/chat/tokens.css` NEW + `docs/chat/chat-ui.js` NEW (initial extraction of `addMessage`/`handleSend`/render helpers, both pages import it).
- `crates/domains/src/applied/hmi/theming/ontology.rs`: `RenderedPairsMeetAa` axiom (+ tests).
- `crates/web/tests/palette_wcag.rs` NEW (+ `pr4xis-domains` dev-dep in `crates/web/Cargo.toml`); update `crates/web/tests/worker_contract.rs` for the new file layout.
- `.github/workflows/ci.yml:616-623`: copy `caregiver.html`, `tokens.css`, `chat-ui.js`.
- ARIA tablist/labels/`aria-live`, `:focus-visible`, prefers-color-scheme + light logo wiring, header flex-wrap.
- *Shippable*: current features, credible instrument look, gated palette.

**Slice 1 — Typed trace (Day 2–3; one isolated wasm commit).**
- `crates/wasm/src/lib.rs:690,778`: structured trace per §5.3.
- `chat-ui.js`: structured trace renderer replaces `addTraceAccordion`; citation tags on rows.
- *Shippable*: transparency drill-down live on both pages.

**Slice 2 — Why? layer + labels (Days 3–4).**
- `crates/domains/data/grammar/explain-frames.tsv` + `ontology-labels.tsv` NEW; `pragmatics/explain_frames.rs` NEW; `realize_why` in `realize.rs`; `why` field through `trace_impls.rs:262-281`, `chat/src/lib.rs` (~15 sites + `ProcessResult`), `wasm/src/lib.rs` (`p.set("why", …)`); `praxis.toml` + 2-pass lock regen; the four critique-mandated tests.
- `chat-ui.js`: Why? panel between outcome card and trace.
- *Shippable*: plain-language layer live; trace demoted one level.

**Slice 3 — Full outcome UI + samplers (Days 4–5).**
- `chat-ui.js`: Conditional slot-fill card, RuleResolved verdict card, reasoned-refusal card (wire fields already present, `lib.rs:699-757`).
- `crates/praxis-corpus-tests/tests/caregiver_questions_generated.rs`: extend the `regenerate_caregiver_snapshot` pattern to also emit `docs/caregiver-corpus-slim.json` + `docs/adversarial-corpus.json` (each embedding its `regenerated_by` command).
- `caregiver.html`: sampler chips + shuffle + predict-then-reveal; known-limits band with "show me one we fail"; adversarial open-mic chips. CI copy step for the two data files.
- *Shippable*: uncurated sampling replaces the hand-picked example buttons (`index.html:350-383` retired).

**Slice 4 — Batch API + Smart-40 console + live slice (Days 5–6).**
- `crates/chat/src/session.rs`: `reset()`. `crates/wasm/src/lib.rs`: `reset_session()`, `chat_batch()` (stateless per question via `process_with_reasoner`, mirroring `scratch_probe.rs:6331-6337` entry shape).
- `docs/worker.js:31-77`: two new message types, chunked progress.
- `scratch_probe.rs` (probe_smart40_validation_log): also emit `docs/smart40-protocol.json` (same records as the dump) so the browser protocol and the ACL-submitted log share one source; CI copy step.
- `caregiver.html`: task-list console with published-vs-live columns + USC preflight loader (via existing `available_usc_archives()`/`load()`); "run 25 random" live-slice tiles; session ledger tile.
- *Shippable*: the headline validation band.

**Slice 5 — Self-audit + method band + narrative regen (Days 6–7).**
- `crates/wasm/src/lib.rs`: `verify_palette()` export; `caregiver.html` WCAG self-audit panel.
- Method band: model-card sections (Mitchell et al. structure, metrics disaggregated by track), AI-nutrition-facts disclosure box (Twilio label fields, every field honestly answerable), rubric map with verified rubric structure.
- `docs/caregiver-challenge/track1-phase1-narrative.md` + `track2-…`: update demo citations (stale line anchors, new lens URLs `…/caregiver.html#track1|2`); CI rebuilds PDFs.
- *Shippable*: full method/evidence band.

**Slice 6 — Hardening buffer (Days 7–8).**
- Mobile pass, axe/Lighthouse run (stated honestly: automated checks cover ~25% of WCAG criteria — AARP testing guidance), copy read-through at grade level, full `devenv ci`, corpus-gate re-run, cross-browser check.

If time compresses: Slice 5's `verify_palette` panel and Slice 4's USC preflight are the two cut lines — everything else is load-bearing for the honesty/judging story. (Cutting = not building; never stubbing.)

---

## 7. What NOT to do

1. **No pre-labeled outcomes, ever.** The live-only `.limit-tag` discipline and its comment (`index.html:150-153`) are the law of the page. No badge, tag, color, or hint attached to a question before the engine returns. (Honesty rule 1; ACM AE framing: pre-staging converts "Results Reproduced" into a failed review.)
2. **No curated example sets.** The hand-picked 3+7 track buttons (`index.html:350-383`) are retired, not restyled. All suggestions are runtime-random over shipped corpus data with visible provenance. (Rule 2; Ehsan & Riedl explainability-pitfalls.)
3. **No bare numbers.** Any figure without a live derivation gets a command chip or doesn't ship. Never reuse the narrative's static percentages as page copy — the page recomputes or commands. (Rule 3; `feedback_cite_the_test`; note the 4,120 vs 4,121 drift between narrative and status JSON proves why.)
4. **No parsing the trace string, anywhere, for anything.** The plain layer comes from `ResponseFrame`/typed wire fields; the trace panel from the typed list. (Why-layer design §"what must not happen".)
5. **No English sentences assembled in frontend JS.** All user-facing response sentences arrive on the wire from loaded frame tables (`feedback_zero_exceptions_nlg_literals`).
6. **No JS batch loop over stateful `chat()`.** Conditional pending-state corrupts every subsequent question (`session.rs:81-86`). Batch = `chat_batch()` or explicit `reset_session()` between calls.
7. **No `usa-*`/`govuk-*` class names or GDS Transport.** Borrow the design language's patterns (CC0/MIT), never the governments' identity; GDS Transport is license-restricted.
8. **No third disclosure level, no dashboard fold-in.** The full self-model dashboard (`index.html:892-1318`) stays on the generic page's Engine tab; folding it into the bench was rejected for information-overload risk (redesign_synthesis Slice 2; NN/g two-level cap).
9. **No confidence percentages.** Categorical outcome badges only — numeric confidence misleads non-experts (abstention/uncertainty literature; PAIR confidence pattern).
10. **No marketing tone, no adjectives, no "elderly/old" language, no target venues.** Sbaffi & Rowley: advertising is the most trust-damaging signal; AARP language guidance; `feedback_modest_framing`, `feedback_no_target_venues_publicly`.
11. **No softening the TRL disclosure to look finished.** The summary box stays adjacent to the instrument, not in a footer (rule 5; Slice-0 hero design).
12. **No orphaned mechanisms.** Every new table/axiom/export in this design ships wired into the live pages in its own slice — the why-layer, the palette axiom, and `verify_palette` are explicitly paired with their consuming UI (`feedback_no_orphaned_mechanisms`).
13. **No hand-picked hex.** Palette values that haven't passed the gate test don't merge (`feedback_no_magic_numbers`; redesign_synthesis §1c).
14. **No server, no external hosts.** Every exhibit is static files + in-browser WASM; new data files enter the CI assembly step (`ci.yml:616-623`), nothing is fetched cross-origin.
