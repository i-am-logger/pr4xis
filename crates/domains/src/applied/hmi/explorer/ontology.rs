/// Ontology explorer — self-referential visualization of reasoning traces.
///
/// The explorer visualizes the ontology using the ontology's own theme.
/// Concept nodes light up as axioms evaluate, colored by the active theme.
///
/// Sources:
/// - Mendez et al., "Evonne" (EuroVis 2023): proof tree visualization
/// - Srisuchinnawong et al., "NeuroVis" (2021): neural activation encoding
/// - Wongsuphasawat et al., "TensorFlow Graph Visualizer" (VAST 2017): dataflow
/// - Beck et al., "Dynamic Graph Visualization" (2017): temporal animation
/// - W3C PROV-O: provenance data model
use crate::applied::hmi::report::generator::ThemePalette;
use pr4xis::ontology::Axiom;

/// A concept node in the ontology graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConceptNode {
    pub id: String,
    pub label: String,
    pub kind: ConceptKind,
}

/// What kind of ontology concept this node represents.
///
/// Source: OWL 2 structural specification + praxis type system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConceptKind {
    /// A type/entity (e.g., ColorSlot, Mode, SchemeType)
    Entity,
    /// A relationship/morphism (e.g., bright-variant-of, mode transition)
    Relationship,
    /// An axiom/rule (e.g., LuminanceMonotonicity, WcagForegroundContrast)
    AxiomNode,
    /// A quality/property (e.g., Polarity, SemanticRole)
    Quality,
    /// A data value (e.g., a specific Rgb color, a luminance value)
    Value,
}

/// An edge connecting two concept nodes.
#[derive(Debug, Clone)]
pub struct ConceptEdge {
    pub from: String,
    pub to: String,
    pub label: String,
    pub kind: EdgeKind,
}

/// What kind of relationship the edge represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    /// Taxonomic (is-a)
    IsA,
    /// Mereological (has-a / part-of)
    HasA,
    /// Dependency (axiom depends on concept)
    DependsOn,
    /// Evaluation flow (axiom evaluates concept)
    Evaluates,
    /// Produces (axiom produces result)
    Produces,
}

/// Activation state of a node during reasoning trace playback.
///
/// Source: NeuroVis 4-channel encoding (Srisuchinnawong 2021)
/// Mapped to theme colors from the active vogix palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActivationState {
    /// Not yet evaluated — dim/inactive
    Inactive,
    /// Currently being evaluated — pulsing/highlighted
    Evaluating,
    /// Axiom satisfied — success color
    Satisfied,
    /// Axiom violated — danger color
    Violated,
    /// Intermediate result — accent color
    Intermediate,
}

/// Maps activation states to semantic color roles from the theme ontology.
///
/// The explorer uses the active theme's functional colors to render itself.
/// This is self-referential: the theming ontology colors the theming visualization.
pub fn activation_to_theme_role(state: ActivationState) -> &'static str {
    match state {
        ActivationState::Inactive => "foreground-comment", // base03: muted
        ActivationState::Evaluating => "active",           // base0C: highlighted
        ActivationState::Satisfied => "success",           // base08: green
        ActivationState::Violated => "danger",             // base0B: red
        ActivationState::Intermediate => "link",           // base0D: blue
    }
}

/// A single step in a reasoning trace.
///
/// Source: praxis Engine::Trace concept
#[derive(Debug, Clone)]
pub struct TraceStep {
    pub step: usize,
    /// Nodes activated in this step
    pub activated: Vec<String>,
    /// Their activation state
    pub state: ActivationState,
    /// Description of what happened
    pub description: String,
}

/// A complete reasoning trace — sequence of steps from question to answer.
#[derive(Debug, Clone)]
pub struct ReasoningTrace {
    pub question: String,
    pub steps: Vec<TraceStep>,
    pub result: ActivationState,
}

impl ReasoningTrace {
    pub fn new(question: impl Into<String>) -> Self {
        Self {
            question: question.into(),
            steps: Vec::new(),
            result: ActivationState::Inactive,
        }
    }

    pub fn add_step(
        &mut self,
        activated: Vec<String>,
        state: ActivationState,
        desc: impl Into<String>,
    ) {
        self.steps.push(TraceStep {
            step: self.steps.len(),
            activated,
            state,
            description: desc.into(),
        });
    }

    pub fn conclude(&mut self, result: ActivationState) {
        self.result = result;
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }
}

/// The ontology graph — nodes and edges forming the knowledge structure.
#[derive(Debug, Clone, Default)]
pub struct OntologyGraph {
    pub nodes: Vec<ConceptNode>,
    pub edges: Vec<ConceptEdge>,
}

impl OntologyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, id: impl Into<String>, label: impl Into<String>, kind: ConceptKind) {
        self.nodes.push(ConceptNode {
            id: id.into(),
            label: label.into(),
            kind,
        });
    }

    pub fn add_edge(
        &mut self,
        from: impl Into<String>,
        to: impl Into<String>,
        label: impl Into<String>,
        kind: EdgeKind,
    ) {
        self.edges.push(ConceptEdge {
            from: from.into(),
            to: to.into(),
            label: label.into(),
            kind,
        });
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

/// Build the theming ontology graph — all concepts and relationships.
pub fn theming_ontology_graph() -> OntologyGraph {
    let mut g = OntologyGraph::new();

    // Entities
    g.add_node("color_slot", "ColorSlot", ConceptKind::Entity);
    g.add_node("scheme_type", "SchemeType", ConceptKind::Entity);
    g.add_node("polarity", "Polarity", ConceptKind::Entity);
    g.add_node("palette", "Palette", ConceptKind::Entity);
    g.add_node("semantic_role", "SemanticRole", ConceptKind::Entity);
    g.add_node("vogix16", "Vogix16Semantic", ConceptKind::Entity);
    g.add_node("ansi16", "Ansi16Color", ConceptKind::Entity);

    // Values
    for i in 0..=7 {
        g.add_node(
            format!("base0{}", i),
            format!("base0{}", i),
            ConceptKind::Value,
        );
    }
    for c in "89ABCDEF".chars() {
        g.add_node(
            format!("base0{}", c),
            format!("base0{}", c),
            ConceptKind::Value,
        );
    }

    // Axioms
    g.add_node("ax_mono", "LuminanceMonotonicity", ConceptKind::AxiomNode);
    g.add_node("ax_wcag", "WcagForegroundContrast", ConceptKind::AxiomNode);
    g.add_node("ax_bright", "BrightVariantBrighter", ConceptKind::AxiomNode);
    g.add_node("ax_bijection_v", "Vogix16Bijection", ConceptKind::AxiomNode);
    g.add_node("ax_bijection_a", "Ansi16Bijection", ConceptKind::AxiomNode);

    // Qualities
    g.add_node("q_luminance", "RelativeLuminance", ConceptKind::Quality);
    g.add_node("q_contrast", "ContrastRatio", ConceptKind::Quality);

    // Relationships
    g.add_edge("palette", "color_slot", "contains", EdgeKind::HasA);
    g.add_edge("palette", "polarity", "has polarity", EdgeKind::HasA);
    g.add_edge("scheme_type", "color_slot", "defines slots", EdgeKind::HasA);
    g.add_edge("color_slot", "semantic_role", "has role", EdgeKind::HasA);
    g.add_edge("vogix16", "color_slot", "maps to", EdgeKind::IsA);
    g.add_edge("ansi16", "color_slot", "maps to", EdgeKind::IsA);

    // Axiom dependencies
    g.add_edge("ax_mono", "q_luminance", "uses", EdgeKind::DependsOn);
    for i in 0..=7 {
        g.add_edge(
            "ax_mono",
            format!("base0{}", i),
            "evaluates",
            EdgeKind::Evaluates,
        );
    }
    g.add_edge("ax_wcag", "q_contrast", "uses", EdgeKind::DependsOn);
    g.add_edge("ax_wcag", "base00", "evaluates bg", EdgeKind::Evaluates);
    g.add_edge("ax_wcag", "base05", "evaluates fg", EdgeKind::Evaluates);

    // Bright variant axiom evaluates accent slots
    for c in "89ABCDEF".chars() {
        g.add_edge(
            "ax_bright",
            format!("base0{}", c),
            "evaluates",
            EdgeKind::Evaluates,
        );
    }

    // Bijection axioms connect schemes to slots
    g.add_edge("ax_bijection_v", "vogix16", "maps", EdgeKind::Evaluates);
    g.add_edge("ax_bijection_v", "color_slot", "to", EdgeKind::Evaluates);
    g.add_edge("ax_bijection_a", "ansi16", "maps", EdgeKind::Evaluates);
    g.add_edge("ax_bijection_a", "color_slot", "to", EdgeKind::Evaluates);

    g
}

/// Build a sample reasoning trace for monotonicity evaluation.
pub fn monotonicity_trace(palette_name: &str, passes: bool) -> ReasoningTrace {
    let mut t = ReasoningTrace::new(format!(
        "Does {} satisfy luminance monotonicity?",
        palette_name
    ));

    t.add_step(
        vec!["palette".into()],
        ActivationState::Evaluating,
        "Load palette",
    );
    t.add_step(
        (0..=7).map(|i| format!("base0{}", i)).collect(),
        ActivationState::Evaluating,
        "Extract base00-base07 ramp slots",
    );
    t.add_step(
        vec!["q_luminance".into()],
        ActivationState::Evaluating,
        "Compute relative luminance per slot (WCAG 2.1)",
    );
    t.add_step(
        vec!["ax_mono".into()],
        ActivationState::Evaluating,
        "Check luminance ordering",
    );

    if passes {
        t.add_step(
            vec!["ax_mono".into()],
            ActivationState::Satisfied,
            "Monotonicity satisfied ✓",
        );
        t.conclude(ActivationState::Satisfied);
    } else {
        t.add_step(
            vec!["ax_mono".into()],
            ActivationState::Violated,
            "Monotonicity violated ✗ — break detected",
        );
        t.conclude(ActivationState::Violated);
    }

    t
}

// ── Axioms ──

/// Every concept kind maps to a theme color role.
pub struct ActivationThemeMapped;

impl Axiom for ActivationThemeMapped {
    fn description(&self) -> &str {
        "every activation state maps to a theme color role (self-referential theming)"
    }
    fn holds(&self) -> bool {
        let states = [
            ActivationState::Inactive,
            ActivationState::Evaluating,
            ActivationState::Satisfied,
            ActivationState::Violated,
            ActivationState::Intermediate,
        ];
        states
            .iter()
            .all(|s| !activation_to_theme_role(*s).is_empty())
    }
}

/// Theming ontology graph is connected (no isolated nodes).
pub struct GraphConnected;

impl Axiom for GraphConnected {
    fn description(&self) -> &str {
        "theming ontology graph has no isolated nodes"
    }
    fn holds(&self) -> bool {
        let g = theming_ontology_graph();
        let connected: std::collections::HashSet<&str> = g
            .edges
            .iter()
            .flat_map(|e| [e.from.as_str(), e.to.as_str()])
            .collect();
        // All nodes should be referenced by at least one edge
        g.nodes.iter().all(|n| connected.contains(n.id.as_str()))
    }
}

/// A reasoning trace has at least 2 steps (start + conclusion).
pub struct TraceMinimalSteps;

impl Axiom for TraceMinimalSteps {
    fn description(&self) -> &str {
        "reasoning trace has at least 2 steps"
    }
    fn holds(&self) -> bool {
        let t = monotonicity_trace("test", true);
        t.step_count() >= 2
    }
}

/// Generate an interactive HTML explorer for the ontology graph.
///
/// Uses D3.js force-directed layout with neural-activation animation.
/// The explorer is self-referential: themed using the palette it visualizes.
///
/// Sources:
/// - Bostock, D3-force (2017): force-directed layout
/// - Srisuchinnawong, NeuroVis (2021): activation encoding (opacity + scale + glow)
/// - Beck et al., Dynamic Graph Vis (2017): temporal animation patterns
pub fn to_explorer_html(
    graph: &OntologyGraph,
    trace: Option<&ReasoningTrace>,
    palette: &ThemePalette,
) -> String {
    // Escape strings for safe embedding in JSON inside <script> tags.
    // Prevents XSS via </script> injection and malformed JSON from " or \ chars.
    let esc = |s: &str| -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('<', "\\u003c")
            .replace('>', "\\u003e")
    };

    // Serialize graph to JSON
    let nodes_json: String = graph
        .nodes
        .iter()
        .map(|n| {
            format!(
                r#"{{"id":"{}","label":"{}","kind":"{}"}}"#,
                esc(&n.id),
                esc(&n.label),
                kind_str(n.kind)
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let edges_json: String = graph
        .edges
        .iter()
        .map(|e| {
            format!(
                r#"{{"source":"{}","target":"{}","label":"{}","kind":"{}"}}"#,
                esc(&e.from),
                esc(&e.to),
                esc(&e.label),
                edge_kind_str(e.kind)
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let trace_json = match trace {
        Some(t) => {
            let steps: String = t
                .steps
                .iter()
                .map(|s| {
                    let activated: String = s
                        .activated
                        .iter()
                        .map(|a| format!(r#""{}""#, esc(a)))
                        .collect::<Vec<_>>()
                        .join(",");
                    format!(
                        r#"{{"step":{},"activated":[{}],"state":"{}","desc":"{}"}}"#,
                        s.step,
                        activated,
                        activation_str(s.state),
                        esc(&s.description)
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                r#"{{"question":"{}","steps":[{}],"result":"{}"}}"#,
                esc(&t.question),
                steps,
                activation_str(t.result)
            )
        }
        None => "null".to_string(),
    };

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Vogix Ontology Explorer</title>
<style>
:root {{
  --bg:{bg}; --fg:{fg}; --fg2:{fg2}; --border:{border};
  --pass:{pass}; --fail:{fail}; --warn:{warn};
  --accent:{accent}; --card:{card}; --hover:{hover};
}}
* {{ margin:0; padding:0; box-sizing:border-box; }}
body {{ background:var(--bg); color:var(--fg); font-family:'Inter',system-ui,sans-serif; overflow:hidden; }}
#explorer {{ width:100vw; height:100vh; }}
svg {{ width:100%; height:100%; }}

.controls {{
  position:fixed; top:1rem; left:1rem; z-index:10;
  background:var(--card); border:1px solid var(--border); border-radius:6px;
  padding:0.75rem 1rem; font-size:0.8rem; max-width:320px;
}}
.controls h2 {{ font-size:0.9rem; margin-bottom:0.5rem; }}
.controls p {{ color:var(--fg2); font-size:0.7rem; margin-bottom:0.5rem; }}
.controls button {{
  background:var(--bg); border:1px solid var(--border); color:var(--fg);
  padding:0.3rem 0.6rem; border-radius:3px; cursor:pointer; font-size:0.7rem;
  margin-right:0.25rem;
}}
.controls button:hover {{ border-color:var(--accent); }}
.controls button.active {{ background:var(--accent); color:var(--bg); }}
#trace-info {{ margin-top:0.5rem; font-size:0.7rem; color:var(--fg2); }}
#step-desc {{ color:var(--fg); font-weight:600; }}

.legend {{
  position:fixed; bottom:1rem; left:1rem; z-index:10;
  background:var(--card); border:1px solid var(--border); border-radius:6px;
  padding:0.5rem 0.75rem; font-size:0.65rem;
}}
.legend-item {{ display:flex; align-items:center; gap:0.4rem; margin:0.2rem 0; }}
.legend-dot {{ width:10px; height:10px; border-radius:50%; }}

/* Node styles */
.node {{ cursor:pointer; }}
.node circle {{ stroke-width:1.5; transition:r 0.3s, opacity 0.3s; }}
.node text {{ font-size:7px; fill:var(--fg2); pointer-events:none; }}
.node.active circle {{ filter:drop-shadow(0 0 6px var(--accent)); }}
.node.satisfied circle {{ filter:drop-shadow(0 0 8px var(--pass)); }}
.node.violated circle {{ filter:drop-shadow(0 0 8px var(--fail)); }}

.link {{ stroke-opacity:0.3; fill:none; }}
.link-label {{ font-size:5px; fill:var(--fg2); opacity:0.5; }}

.tooltip {{
  position:fixed; display:none; background:var(--card); border:1px solid var(--border);
  border-radius:4px; padding:0.4rem 0.6rem; font-size:0.75rem; z-index:20;
  pointer-events:none;
}}
</style>
</head>
<body>
<div id="explorer">
<svg></svg>
</div>

<div class="controls">
  <h2>Ontology Explorer</h2>
  <p id="graph-info"></p>
  <div id="trace-controls" style="display:none">
    <button onclick="resetTrace()">Reset</button>
    <button onclick="stepTrace()">Step</button>
    <button onclick="playTrace()">Play</button>
    <div id="trace-info">
      <div id="step-desc"></div>
    </div>
  </div>
</div>

<div class="legend">
  <div class="legend-item"><div class="legend-dot" style="background:var(--accent)"></div>Entity</div>
  <div class="legend-item"><div class="legend-dot" style="background:var(--warn)"></div>Axiom</div>
  <div class="legend-item"><div class="legend-dot" style="background:var(--pass)"></div>Quality</div>
  <div class="legend-item"><div class="legend-dot" style="background:var(--fg2)"></div>Value</div>
  <div class="legend-item"><div class="legend-dot" style="background:var(--fail)"></div>Relationship</div>
</div>

<div class="tooltip" id="tooltip"></div>

<script>
// ── Minimal self-contained force simulation (no external deps) ──
const G = {{nodes:[{nodes_json}],links:[{edges_json}]}};
const T = {trace_json};
const style = getComputedStyle(document.documentElement);
const colors = {{
  entity: style.getPropertyValue('--accent').trim(),
  axiom: style.getPropertyValue('--warn').trim(),
  quality: style.getPropertyValue('--pass').trim(),
  value: style.getPropertyValue('--fg2').trim(),
  relationship: style.getPropertyValue('--fail').trim(),
}};
const activation = {{
  inactive: style.getPropertyValue('--fg2').trim(),
  evaluating: style.getPropertyValue('--accent').trim(),
  satisfied: style.getPropertyValue('--pass').trim(),
  violated: style.getPropertyValue('--fail').trim(),
  intermediate: style.getPropertyValue('--accent').trim(),
}};

const kindColor = k => colors[k] || colors.value;
const kindRadius = k => ({{ entity:8, axiom:10, quality:7, value:4, relationship:6 }})[k] || 5;

const W = window.innerWidth, H = window.innerHeight;
const svg = document.querySelector('svg');
svg.setAttribute('viewBox', `0 0 ${{W}} ${{H}}`);

// Build id→node index map
const idxMap = {{}};
G.nodes.forEach((n,i) => {{ n.x = W/2 + (Math.random()-0.5)*200; n.y = H/2 + (Math.random()-0.5)*200; n.vx=0; n.vy=0; idxMap[n.id]=i; }});
G.links.forEach(l => {{ l.si = idxMap[l.source]; l.ti = idxMap[l.target]; }});

// Create SVG elements
const ns = 'http://www.w3.org/2000/svg';
const linkG = document.createElementNS(ns,'g');
const nodeG = document.createElementNS(ns,'g');
svg.appendChild(linkG); svg.appendChild(nodeG);

const linkEls = G.links.map(l => {{
  const line = document.createElementNS(ns,'line');
  line.setAttribute('class','link');
  line.setAttribute('stroke', l.kind==='evaluates'?'var(--warn)':l.kind==='depends_on'?'var(--accent)':'var(--border)');
  line.setAttribute('stroke-width', l.kind==='evaluates'?1.5:1);
  linkG.appendChild(line);
  return line;
}});

const nodeEls = G.nodes.map(n => {{
  const g = document.createElementNS(ns,'g');
  g.setAttribute('class','node');
  const c = document.createElementNS(ns,'circle');
  c.setAttribute('r', kindRadius(n.kind));
  c.setAttribute('fill', kindColor(n.kind));
  c.setAttribute('stroke', 'var(--bg)');
  c.setAttribute('stroke-width', '1.5');
  c.setAttribute('opacity', '0.85');
  g.appendChild(c);
  const t = document.createElementNS(ns,'text');
  t.textContent = n.label;
  t.setAttribute('dx', kindRadius(n.kind)+3);
  t.setAttribute('dy', 3);
  t.style.fontSize = '7px';
  t.style.fill = 'var(--fg2)';
  t.style.pointerEvents = 'none';
  g.appendChild(t);
  nodeG.appendChild(g);
  return {{ g, c, n }};
}});

// Tooltip + drag
const tooltip = document.getElementById('tooltip');
let dragNode = null, dragOff = [0,0];
nodeEls.forEach(el => {{
  el.g.addEventListener('mouseover', e => {{
    tooltip.style.display = 'block';
    tooltip.style.left = (e.clientX+10)+'px';
    tooltip.style.top = (e.clientY-10)+'px';
    tooltip.innerHTML = `<strong>${{el.n.label}}</strong><br><span style="color:var(--fg2)">${{el.n.kind}}</span>`;
  }});
  el.g.addEventListener('mouseout', () => tooltip.style.display='none');
  el.g.addEventListener('mousedown', e => {{
    dragNode = el.n; dragOff = [e.clientX-el.n.x, e.clientY-el.n.y]; e.preventDefault();
    if (settled) {{ settled = false; requestAnimationFrame(tick); }}
  }});
}});
document.addEventListener('mousemove', e => {{
  if (dragNode) {{ dragNode.x = e.clientX-dragOff[0]; dragNode.y = e.clientY-dragOff[1]; dragNode.vx=0; dragNode.vy=0; }}
}});
document.addEventListener('mouseup', () => {{ dragNode=null; }});

// Force simulation (velocity verlet)
let alpha = 1;
let settled = false;
function tick() {{
  if (settled && !dragNode) return; // Stop after settling to save CPU
  alpha *= 0.99;
  if (alpha < 0.002 && !dragNode) {{ settled = true; }}
  if (dragNode) {{ settled = false; alpha = Math.max(alpha, 0.1); }}
  const nodes = G.nodes;
  // Charge (repulsion)
  for (let i=0; i<nodes.length; i++) {{
    for (let j=i+1; j<nodes.length; j++) {{
      let dx = nodes[j].x-nodes[i].x, dy = nodes[j].y-nodes[i].y;
      let d2 = dx*dx+dy*dy+1;
      let f = -120*alpha/d2;
      let fx = dx/Math.sqrt(d2)*f, fy = dy/Math.sqrt(d2)*f;
      nodes[i].vx -= fx; nodes[i].vy -= fy;
      nodes[j].vx += fx; nodes[j].vy += fy;
    }}
  }}
  // Links (attraction)
  G.links.forEach(l => {{
    const s = nodes[l.si], t = nodes[l.ti];
    let dx = t.x-s.x, dy = t.y-s.y;
    let d = Math.sqrt(dx*dx+dy*dy)+0.1;
    let f = (d-60)*0.05*alpha;
    let fx = dx/d*f, fy = dy/d*f;
    s.vx += fx; s.vy += fy;
    t.vx -= fx; t.vy -= fy;
  }});
  // Center
  nodes.forEach(n => {{ n.vx += (W/2-n.x)*0.01*alpha; n.vy += (H/2-n.y)*0.01*alpha; }});
  // Integrate + dampen
  nodes.forEach(n => {{
    if (n === dragNode) return;
    n.vx *= 0.6; n.vy *= 0.6;
    n.x += n.vx; n.y += n.vy;
  }});
  // Render
  linkEls.forEach((el,i) => {{
    const l = G.links[i];
    el.setAttribute('x1', nodes[l.si].x); el.setAttribute('y1', nodes[l.si].y);
    el.setAttribute('x2', nodes[l.ti].x); el.setAttribute('y2', nodes[l.ti].y);
  }});
  nodeEls.forEach(el => {{
    el.g.setAttribute('transform', `translate(${{el.n.x}},${{el.n.y}})`);
  }});
  requestAnimationFrame(tick);
}}
tick();

document.getElementById('graph-info').textContent =
  `${{G.nodes.length}} nodes, ${{G.links.length}} edges`;

// ── Trace playback (neural activation animation) ──
let traceStep = -1;
let playing = false;

if (T) {{
  document.getElementById('trace-controls').style.display = 'block';
  document.getElementById('trace-info').innerHTML =
    `<em>${{T.question}}</em>`;
}}

function resetTrace() {{
  traceStep = -1;
  playing = false;
  node.classed('active', false).classed('satisfied', false).classed('violated', false);
  node.selectAll('circle').attr('opacity', 0.85).attr('r', d => kindRadius(d.kind));
  document.getElementById('step-desc').textContent = '';
}}

function stepTrace() {{
  if (!T || traceStep >= T.steps.length - 1) return;
  traceStep++;
  const s = T.steps[traceStep];
  const ids = new Set(s.activated);

  node.each(function(d) {{
    const el = d3.select(this);
    if (ids.has(d.id)) {{
      el.classed('active', s.state === 'evaluating')
        .classed('satisfied', s.state === 'satisfied')
        .classed('violated', s.state === 'violated');
      el.select('circle')
        .transition().duration(400)
        .attr('r', kindRadius(d.kind) * 1.6)
        .attr('opacity', 1)
        .attr('fill', activation[s.state] || activation.evaluating)
        .transition().duration(600)
        .attr('r', kindRadius(d.kind) * 1.2);
    }}
  }});

  document.getElementById('step-desc').textContent =
    `Step ${{s.step + 1}}/${{T.steps.length}}: ${{s.desc}}`;
}}

function playTrace() {{
  if (playing) return;
  playing = true;
  resetTrace();
  let i = 0;
  const interval = setInterval(() => {{
    if (i >= T.steps.length) {{ clearInterval(interval); playing = false; return; }}
    traceStep = i - 1;
    stepTrace();
    i++;
  }}, 1200);
}}
</script>
</body>
</html>"##,
        bg = palette.bg,
        fg = palette.fg,
        fg2 = palette.fg2,
        border = palette.border,
        pass = palette.pass,
        fail = palette.fail,
        warn = palette.warn,
        accent = palette.accent,
        card = palette.card,
        hover = palette.hover,
        nodes_json = nodes_json,
        edges_json = edges_json,
        trace_json = trace_json,
    )
}

fn kind_str(k: ConceptKind) -> &'static str {
    match k {
        ConceptKind::Entity => "entity",
        ConceptKind::Relationship => "relationship",
        ConceptKind::AxiomNode => "axiom",
        ConceptKind::Quality => "quality",
        ConceptKind::Value => "value",
    }
}

fn edge_kind_str(k: EdgeKind) -> &'static str {
    match k {
        EdgeKind::IsA => "is_a",
        EdgeKind::HasA => "has_a",
        EdgeKind::DependsOn => "depends_on",
        EdgeKind::Evaluates => "evaluates",
        EdgeKind::Produces => "produces",
    }
}

fn activation_str(s: ActivationState) -> &'static str {
    match s {
        ActivationState::Inactive => "inactive",
        ActivationState::Evaluating => "evaluating",
        ActivationState::Satisfied => "satisfied",
        ActivationState::Violated => "violated",
        ActivationState::Intermediate => "intermediate",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_5_activation_states() {
        let states = [
            ActivationState::Inactive,
            ActivationState::Evaluating,
            ActivationState::Satisfied,
            ActivationState::Violated,
            ActivationState::Intermediate,
        ];
        assert_eq!(states.len(), 5);
    }

    #[test]
    fn test_activation_theme_mapped() {
        assert!(ActivationThemeMapped.holds());
    }

    #[test]
    fn test_theming_graph_has_nodes() {
        let g = theming_ontology_graph();
        assert!(g.node_count() > 20);
        assert!(g.edge_count() > 15);
    }

    #[test]
    fn test_graph_connected() {
        assert!(GraphConnected.holds());
    }

    #[test]
    fn test_monotonicity_trace_pass() {
        let t = monotonicity_trace("test-dark", true);
        assert!(t.step_count() >= 4);
        assert_eq!(t.result, ActivationState::Satisfied);
    }

    #[test]
    fn test_monotonicity_trace_fail() {
        let t = monotonicity_trace("catppuccin-mocha", false);
        assert_eq!(t.result, ActivationState::Violated);
    }

    #[test]
    fn test_trace_minimal_steps() {
        assert!(TraceMinimalSteps.holds());
    }

    #[test]
    fn test_activation_roles_are_distinct() {
        let states = [
            ActivationState::Inactive,
            ActivationState::Evaluating,
            ActivationState::Satisfied,
            ActivationState::Violated,
            ActivationState::Intermediate,
        ];
        let roles: Vec<_> = states
            .iter()
            .map(|s| activation_to_theme_role(*s))
            .collect();
        let unique: std::collections::HashSet<_> = roles.iter().collect();
        assert_eq!(
            roles.len(),
            unique.len(),
            "activation states must map to distinct theme roles"
        );
    }

    #[test]
    fn test_concept_kinds() {
        let g = theming_ontology_graph();
        let entity_count = g
            .nodes
            .iter()
            .filter(|n| n.kind == ConceptKind::Entity)
            .count();
        let axiom_count = g
            .nodes
            .iter()
            .filter(|n| n.kind == ConceptKind::AxiomNode)
            .count();
        let value_count = g
            .nodes
            .iter()
            .filter(|n| n.kind == ConceptKind::Value)
            .count();
        assert!(entity_count >= 5);
        assert!(axiom_count >= 3);
        assert!(value_count >= 8); // base00-base07 at minimum
    }

    #[test]
    fn test_edge_kinds() {
        let g = theming_ontology_graph();
        let has_a = g
            .edges
            .iter()
            .filter(|e| matches!(e.kind, EdgeKind::HasA))
            .count();
        let evaluates = g
            .edges
            .iter()
            .filter(|e| matches!(e.kind, EdgeKind::Evaluates))
            .count();
        assert!(has_a >= 4);
        assert!(evaluates >= 8); // mono evaluates base00-base07
    }

    #[test]
    fn test_explorer_html_structure() {
        let g = theming_ontology_graph();
        let t = monotonicity_trace("test-dark", true);
        let html = to_explorer_html(&g, Some(&t), &ThemePalette::default());
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Ontology Explorer"));
        assert!(
            !html.contains("src=\"http"),
            "explorer must be self-contained (no CDN)"
        );
        assert!(html.contains("ColorSlot"));
        assert!(html.contains("LuminanceMonotonicity"));
    }

    #[test]
    fn test_explorer_html_themed() {
        let g = theming_ontology_graph();
        let palette = ThemePalette {
            bg: "#1e1e2e".into(),
            fg: "#cdd6f4".into(),
            ..ThemePalette::default()
        };
        let html = to_explorer_html(&g, None, &palette);
        assert!(html.contains("#1e1e2e"));
        assert!(html.contains("#cdd6f4"));
    }

    #[test]
    fn test_explorer_html_no_trace() {
        let g = theming_ontology_graph();
        let html = to_explorer_html(&g, None, &ThemePalette::default());
        assert!(html.contains("const T = null;"));
    }

    #[test]
    fn test_explorer_generates_to_docs() {
        let g = theming_ontology_graph();
        let t = monotonicity_trace("catppuccin-mocha", false);
        let html = to_explorer_html(&g, Some(&t), &ThemePalette::default());

        let docs_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs");
        std::fs::create_dir_all(&docs_dir).unwrap();
        std::fs::write(docs_dir.join("explorer.html"), &html).unwrap();

        assert!(html.len() > 1000);
    }

    // ── Property-based tests ──
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_trace_always_has_conclusion(passes in prop::bool::ANY) {
            let t = monotonicity_trace("test", passes);
            prop_assert_ne!(t.result, ActivationState::Inactive, "trace must conclude");
        }

        #[test]
        fn prop_activation_roles_non_empty(idx in 0usize..5) {
            let states = [
                ActivationState::Inactive,
                ActivationState::Evaluating,
                ActivationState::Satisfied,
                ActivationState::Violated,
                ActivationState::Intermediate,
            ];
            let role = activation_to_theme_role(states[idx]);
            prop_assert!(!role.is_empty());
        }
    }
}
