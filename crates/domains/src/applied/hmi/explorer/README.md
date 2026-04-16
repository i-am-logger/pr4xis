# Explorer -- Self-referential visualization of reasoning traces

Models the ontology explorer as a first-class domain. The explorer visualizes pr4xis's own reasoning process in real time: **concept nodes light up as axioms evaluate**, colored by the active theme, with edges drawn for taxonomy / mereology / causation / opposition. It is deliberately self-referential — the explorer is itself a pr4xis ontology, consumed by any surface that wants to render reasoning traces.

The self-referential claim is concrete: the explorer uses `applied/hmi/theming/`'s `ThemePalette` to color its own nodes, so changing the active theme re-colors the visualization of the theming ontology through which the theme is changed. This is Brian Smith 1984 reflective-tower in a small frame.

Key references:
- Mendez et al. 2023: *Evonne* (EuroVis 2023) — proof-tree visualization
- Srisuchinnawong et al. 2021: *NeuroVis* — neural activation encoding
- Wongsuphasawat et al. 2017: *TensorFlow Graph Visualizer* (IEEE VAST) — dataflow graphs at scale
- Beck et al. 2017: *A Taxonomy and Survey of Dynamic Graph Visualization* (Computer Graphics Forum) — temporal animation
- Smith 1984: *Reflection and Semantics in a Procedural Language* — reflective towers
- W3C PROV-O 2013 — provenance data model

## Entities

| Category | Entities |
|---|---|
| Graph primitives | `ConceptNode { id, label, kind, activation }`, `ConceptEdge { from, to, label, kind }`, `OntologyGraph { nodes, edges }` |
| Concept kinds | `Entity`, `Relation`, `Quality`, `Axiom` |
| Edge kinds | `IsA` (taxonomy), `PartOf` (mereology), `Causes` (causation), `Opposes` (opposition), `Uses` (reference) |
| Trace | `TraceStep { step, activated, reasons }`, `ReasoningTrace { question, steps, result }` |
| Activation | `ActivationState` — Inactive, Active, Evaluated, Satisfied, Violated |
| Shader params | `ShaderParams { uniforms, ... }` — GPU-side controls for the trace renderer |

## Qualities

| Quality | Type | Description |
|---|---|---|
| ShaderUniform | f32 / Rgb / Vec3 | Parameters for the trace renderer's fragment shader (activation intensity, pulse speed, glow radius) |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| (structural) | Identity and composition laws | auto-generated |
| ActivationThemeMapped | Every activation state maps to a distinct palette slot in the active theme | self-referential invariant |
| GraphConnected | The ontology graph is connected — no orphan concepts | graph well-formedness |
| TraceMinimalSteps | A reasoning trace has at most one activation per step per concept | Evonne / NeuroVis |

See `ontology.rs` for the full `impl Axiom for …` blocks.

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one. The explorer is a **consumer** of `applied/hmi/theming/` (for palette) and `applied/hmi/visualization/` (for Bertin's visual variables and accuracy ranking). It is the **target** of any ontology that wants to render its own reasoning — including the future pr4xis.dev `/visualize/ontologies/` page, which will render the 106-ontology graph live.

## Files

- `ontology.rs` -- `ConceptNode`, `ConceptEdge`, `ActivationState`, `TraceStep`, `ReasoningTrace`, `OntologyGraph`, `ActivationThemeMapped`, `GraphConnected`, `TraceMinimalSteps` axioms, tests
- `shader_params.rs` -- `ShaderParams` for GPU-side trace rendering, uniforms, `Interval`-bounded ranges
- `README.md` -- this file
- `citings.md` -- per-ontology bibliography
- `mod.rs` -- module declarations

Previous home: `applied/theming/explorer.rs` + `shader_params.rs`, moved here by [#66](https://github.com/i-am-logger/pr4xis/pull/66).
