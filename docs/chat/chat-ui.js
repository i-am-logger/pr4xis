// pr4xis shared chat renderer — imported by both surfaces of the ONE app
// (docs/chat/index.html): the generic Chat tab's inline module and the
// Caregiver tab's controller (dashboard.js). This module is also the sole
// definition site of `createEngine`, which the shell calls exactly once.
//
// THE WHOLE POINT of this module: one renderer maps EVERY typed ChatOutcome
// that crosses the wasm wire (answered / abstained / conditional /
// rule_applies / rule_does_not_apply) to its card. Card STYLE keys off the
// wire `outcome` variant (structurally honest — no per-question anything);
// every user-facing SENTENCE comes off the wire (`response`, `why`,
// `rule_*`), never assembled from JS string literals. The wire contract this
// renders is crates/wasm/src/lib.rs `Pr4xis::chat` (fields: response,
// outcome, unresolved?, rule_name?, rule_definition?, rule_citation?,
// missing_facts?, ontologies[], from_ontology, duration_us, trace, why?).

// ---------------------------------------------------------------------------
// Worker RPC client — same protocol the pages have always used
// (main posts {id,type,args}; worker replies {id,status,payload|phase}).
// ---------------------------------------------------------------------------
export function createEngine(workerUrl) {
  const worker = new Worker(workerUrl, { type: 'module' });
  let nextRpcId = 1;
  const pending = new Map();
  worker.onmessage = (e) => {
    const m = e.data;
    const p = pending.get(m.id);
    if (!p) {
      // A reply matching no pending call (foreign postMessage, or a worker
      // fail() on a malformed inbound message) is surfaced, never dropped.
      console.error('worker reply with no pending call:', m);
      return;
    }
    if (m.status === 'progress') { if (p.onProgress) p.onProgress(m); return; }
    pending.delete(m.id);
    if (m.status === 'ok') p.resolve(m.payload);
    else p.reject(new Error(m.payload));
  };
  worker.onerror = (e) => console.error('worker error:', e.message || e);
  function call(type, args, onProgress) {
    const id = nextRpcId++;
    return new Promise((resolve, reject) => {
      pending.set(id, { resolve, reject, onProgress });
      worker.postMessage({ id, type, args: args || {} });
    });
  }
  return { worker, call };
}

// ---------------------------------------------------------------------------
// Plain-label projection for an ontology id.
//
// The AUTHORITATIVE plain label is loaded from a cited .prx label table and
// arrives ON THE WIRE (design §5.2, Slice 2). Until that wire field exists we
// apply a PURE TYPOGRAPHIC normalization only (underscore -> space) — this
// carries zero domain/English knowledge (it is not a hardcoded id->English
// map), it just stops raw `caregiving_lexicon` snake_case from reaching the
// reader. The moment the wire carries `o.label`, we prefer it verbatim.
// ---------------------------------------------------------------------------
export function ontologyLabel(o) {
  if (o && typeof o === 'object') {
    if (o.label) return o.label;                 // future authoritative wire field
    return String(o.ontology || '').replace(/_/g, ' ');
  }
  return String(o || '').replace(/_/g, ' ');
}

// An unresolved entry is `{surface, source_id?}` — the term to show, plus the
// source the engine resolved it to where it could. Older wire payloads sent a
// bare string; tolerating both keeps a cached wasm from blanking the term list.
export function unresolvedSurface(u) {
  if (u && typeof u === 'object') return String(u.surface ?? '');
  return String(u ?? '');
}

export function fmtDuration(us) {
  if (typeof us !== 'number' || !Number.isFinite(us) || us < 0) return '';
  // Browsers deliberately coarsen `performance.now()` as a Spectre mitigation,
  // so a fast in-memory turn can measure as literally zero. Report that as the
  // bound it is rather than as "0µs" — a rendered zero reads as a broken
  // measurement, and claims a precision the clock does not have.
  if (us < 100) return 'below 100µs';
  return us < 1000 ? `${Math.round(us)}µs` : `${(us / 1000).toFixed(1)}ms`;
}

export function addMessage(text, cls, target) {
  const div = document.createElement('div');
  div.className = `message ${cls}`;
  div.textContent = text;
  target.appendChild(div);
  target.scrollTop = target.scrollHeight;
  return div;
}

// ---------------------------------------------------------------------------
// Trace accordion. Feature-detects the wire shape:
//   - Array  -> the TYPED trace (design §5.3, Slice 1): a list of step records
//               {ontology, operation, phase, detail, success, reasoned_over,
//                functor_connections:[{target, functor, reference}]}. Per-step
//               literature citations render as small cite tags.
//   - String -> the legacy pipe/colon trace. Split confined to THIS technical
//               drill-down panel only (never the plain layer) and retired the
//               moment the typed trace lands.
// ---------------------------------------------------------------------------
export function buildTrace(trace) {
  const acc = document.createElement('div');
  acc.className = 'accordion';
  const btn = document.createElement('button');
  btn.className = 'accordion-toggle';
  btn.setAttribute('aria-expanded', 'false');
  btn.textContent = '▶ Show technical pipeline';
  const panel = document.createElement('div');
  panel.className = 'accordion-panel';
  const rows = document.createElement('div');
  rows.className = 'trace-rows';
  panel.appendChild(rows);

  if (Array.isArray(trace)) {
    for (const step of trace) {
      const row = document.createElement('div');
      // warn/err hues from the typed node's own status/success fields.
      const cls = step.success === false ? ' err' : (step.status === 'warn' ? ' warn' : '');
      row.className = 'trace-row' + cls;
      const ont = document.createElement('span');
      ont.className = 't-ont';
      ont.textContent = ontologyLabel(step);
      row.appendChild(ont);
      const op = document.createElement('span');
      const phase = step.phase ? `[${step.phase}] ` : '';
      op.textContent = ` → ${phase}${step.operation || ''}${step.detail ? ': ' + step.detail : ''}`;
      row.appendChild(op);
      const conns = Array.isArray(step.functor_connections) ? step.functor_connections : [];
      for (const c of conns) {
        if (c && c.reference) {
          const cite = document.createElement('span');
          cite.className = 't-cite';
          cite.textContent = `— ${c.reference}`;
          row.appendChild(cite);
        }
      }
      rows.appendChild(row);
    }
  } else if (typeof trace === 'string') {
    for (const part of trace.split(' | ')) {
      const fields = part.split(':');
      const row = document.createElement('div');
      row.className = 'trace-row';
      if (fields.length >= 4) {
        const st = fields[0].trim();
        if (st === 'warn') row.classList.add('warn');
        else if (st === 'err') row.classList.add('err');
        const ont = document.createElement('span');
        ont.className = 't-ont';
        ont.textContent = fields[1].trim();
        row.appendChild(ont);
        const rest = document.createElement('span');
        rest.textContent = ` → ${fields[2].trim()}: ${fields.slice(3).join(':').trim()}`;
        row.appendChild(rest);
      } else {
        row.textContent = part;
      }
      rows.appendChild(row);
    }
  }

  btn.addEventListener('click', () => {
    const open = panel.getAttribute('data-open') === 'true';
    panel.setAttribute('data-open', String(!open));
    btn.setAttribute('aria-expanded', String(!open));
    btn.textContent = open ? '▶ Show technical pipeline' : '▼ Hide technical pipeline';
  });
  acc.appendChild(btn);
  acc.appendChild(panel);
  return acc;
}

// Only LOADED sources become always-visible chips — `result.ontologies`
// deliberately interleaves compiled-pipeline participants (e.g.
// "LemonOntology") with loaded domain sources (a documented, tested wire
// contract; see wasm/src/lib.rs), but a raw internal pipeline-stage name is
// not a user-facing "source" and must never render as one. The full,
// unfiltered list already renders correctly inside the collapsed trace
// accordion (buildTrace) — this is the ONLY slot that needs filtering.
function sourceChips(result) {
  const onts = Array.isArray(result.ontologies) ? result.ontologies : [];
  const loadedOnts = onts.filter((o) => o && typeof o === 'object' && o.kind === 'loaded');
  if (!loadedOnts.length) return null;
  const wrap = document.createElement('div');
  wrap.className = 'answer-sources';
  for (const o of loadedOnts) {
    const chip = document.createElement('span');
    chip.className = 'source-chip loaded';
    chip.textContent = ontologyLabel(o) + ' ✦';
    wrap.appendChild(chip);
  }
  return wrap;
}

// ---------------------------------------------------------------------------
// Outcome -> presentation metadata (badge text, status class, tag class). ONE
// typed mapping so every surface — the answer card, the Smart-40 console, the
// gap feed — agrees on the same label AND the same colour for the same wire
// `outcome`, never a per-component ad hoc string.
// ---------------------------------------------------------------------------
export function outcomeMeta(result) {
  const o = result && result.outcome;
  if (o === 'answered') return { cls: 'st-answered', tag: 'tag-success', badge: 'Answered' };
  if (o === 'abstained') return { cls: 'st-abstained', tag: 'tag-warning', badge: 'Abstained' };
  if (o === 'conditional') return { cls: 'st-conditional', tag: 'tag-info', badge: 'More information required' };
  if (o === 'rule_applies') return { cls: 'st-answered', tag: 'tag-success', badge: 'Rule applies' };
  if (o === 'rule_does_not_apply') return { cls: 'st-conditional', tag: 'tag-info', badge: 'Rule does not apply' };
  return { cls: 'st-idle', tag: 'tag-neutral', badge: o ? String(o) : '—' };
}

// ---------------------------------------------------------------------------
// The stepped "Why this result?" proof path (design brief: render the wire's
// `why` / rule / missing-facts data as a clean vertical sequence of
// {icon, label, detail} steps, not a text dump). EVERY step is real wire
// data restructured for presentation — nothing here is invented or guessed:
//   - Conditional / RuleResolved: the cited rule (name + citation), what it
//     requires (definition), and each named missing fact — the SAME
//     `rule_name`/`rule_citation`/`rule_definition`/`missing_facts` the
//     result card already renders, just walked as discrete steps.
//   - Answered: the `why` sentence (provenance — what grounded the answer)
//     as step one, then each line of the deferred "Full reasoning chain:"
//     section (doc: `ProcessResult.why`, `crates/chat/src/lib.rs`) as a
//     following step — parsed from the two clearly-delimited sections the
//     wire already carries (`realize_why` + `build_taxonomy_response`'s
//     `label: definition` lines), never re-derived or invented.
//   - Abstained: the named unresolved surfaces, then the `why` gloss.
// Returns `[]` when there is nothing real to show (e.g. a self-explaining
// frame with no `why`) — the caller renders an honest empty state instead.
// ---------------------------------------------------------------------------
function parseWhy(why) {
  if (!why) return { primary: '', deferred: [] };
  const marker = '\n\nFull reasoning chain:\n';
  const at = why.indexOf(marker);
  if (at === -1) return { primary: why, deferred: [] };
  const primary = why.slice(0, at);
  const chain = why.slice(at + marker.length);
  const deferred = chain.split('\n').filter(Boolean).map((line) => {
    const sep = line.indexOf(': ');
    if (sep === -1) return { label: null, detail: line };
    return { label: line.slice(0, sep), detail: line.slice(sep + 2) };
  });
  return { primary, deferred };
}

export function buildProofSteps(result) {
  const outcome = result.outcome;
  const steps = [];
  if (outcome === 'conditional' || outcome === 'rule_applies' || outcome === 'rule_does_not_apply') {
    if (result.rule_name) {
      steps.push({ icon: '⚖', label: result.rule_name, detail: result.rule_citation || '' });
    }
    if (result.rule_definition) {
      steps.push({ icon: '📄', label: 'What the rule requires', detail: result.rule_definition });
    }
    if (outcome === 'conditional') {
      const missing = Array.isArray(result.missing_facts) ? result.missing_facts : [];
      for (const field of missing) {
        steps.push({ icon: '❓', label: 'Missing fact', detail: field, tone: 'warn' });
      }
    } else {
      steps.push({
        icon: outcome === 'rule_applies' ? '✅' : '⛔',
        label: outcome === 'rule_applies' ? 'Rule applies' : 'Rule does not apply',
        detail: result.response || '',
      });
    }
    return steps;
  }
  if (outcome === 'abstained') {
    const terms = Array.isArray(result.unresolved) ? result.unresolved : [];
    if (terms.length) {
      steps.push({
        icon: '❓',
        label: terms.length === 1 ? 'Unresolved term' : 'Unresolved terms',
        detail: terms.map(unresolvedSurface).join(', '),
        tone: 'warn',
      });
    }
    const { primary } = parseWhy(result.why);
    if (primary) steps.push({ icon: '🛈', label: 'Why it abstained', detail: primary, tone: 'warn' });
    if (!steps.length && result.response) {
      steps.push({ icon: '🛈', label: 'Why it abstained', detail: result.response, tone: 'warn' });
    }
    return steps;
  }
  // Answered (or an unknown/legacy shape): the provenance sentence, then each
  // deferred evidence line, from the SAME `why` string the plain Why? panel
  // renders — parsed, never re-derived from a different source.
  const { primary, deferred } = parseWhy(result.why);
  if (primary) steps.push({ icon: '📚', label: 'Grounded in', detail: primary });
  for (const d of deferred) {
    const isSubtypes = d.label && d.label.startsWith('types of ');
    steps.push({
      icon: isSubtypes ? '🔗' : '📖',
      label: d.label || 'Evidence',
      detail: d.detail,
    });
  }
  // DEFINITION PROVENANCE — the one provenance fact the sentence above does not
  // already state: which document a recited gloss was WRITTEN FROM, as opposed
  // to which ontologies this turn opened.
  //
  // There is deliberately no second step here listing `result.ontologies`
  // filtered to kind 'loaded'. That list is, by construction, the exact
  // population `why`'s primary sentence is realized from (the engine fills its
  // {sources} slot from the same `reasoned_over` vector the wire's `ontologies`
  // records are projected from), so rendering both read as two independent
  // corroborations of one fact — while the genuinely third fact, the gloss's
  // own authorship, was nowhere on screen. The distinction is the ENGINE's: it
  // sends this record, with its own realized label, only when a recited
  // definition declares a source. Both strings are engine-realized from the
  // loaded explain-frames table, so nothing here decides what to call it, and
  // nothing here inspects the `why` prose to work out what is missing from it.
  const prov = result.definition_provenance;
  if (prov && prov.label && prov.detail) {
    steps.push({ icon: '🖋', label: prov.label, detail: prov.detail });
  }
  return steps;
}

// Render `buildProofSteps(result)` as a `.proof-path` of `.proof-step`
// elements into `target` (replacing its children), plus a "View full proof &
// citations" toggle that reveals the SAME typed-trace accordion `renderResult`
// uses (never a second, divergent trace renderer). Returns the wrapping
// element. An empty step list renders an honest `.proof-empty` note, never a
// fabricated placeholder step.
export function renderProofPath(result, target, opts = {}) {
  const includeTraceToggle = opts.includeTraceToggle !== false;
  target.replaceChildren();
  const steps = buildProofSteps(result);
  if (!steps.length) {
    const empty = document.createElement('div');
    empty.className = 'proof-empty';
    empty.textContent = 'This outcome’s primary answer already states its own reason — see the result panel.';
    target.appendChild(empty);
    return target;
  }
  const path = document.createElement('div');
  path.className = 'proof-path';
  for (const step of steps) {
    const row = document.createElement('div');
    row.className = 'proof-step';
    const ic = document.createElement('div');
    ic.className = 'ps-ic';
    ic.textContent = step.icon || '•';
    ic.setAttribute('aria-hidden', 'true');
    const body = document.createElement('div');
    body.className = 'ps-body';
    const label = document.createElement('div');
    label.className = 'ps-label';
    label.textContent = step.label;
    const detail = document.createElement('div');
    detail.className = 'ps-detail';
    detail.textContent = step.detail;
    body.append(label, detail);
    row.append(ic, body);
    path.appendChild(row);
  }
  target.appendChild(path);

  const trace = Array.isArray(result.trace_structured) ? result.trace_structured : result.trace;
  if (includeTraceToggle && trace !== undefined && trace !== null && trace !== '') {
    const toggle = document.createElement('button');
    toggle.className = 'btn btn-ghost';
    toggle.type = 'button';
    toggle.style.marginTop = 'var(--sp-2)';
    toggle.textContent = 'View full proof & citations';
    const panel = document.createElement('div');
    panel.style.display = 'none';
    toggle.addEventListener('click', () => {
      const open = panel.style.display !== 'none';
      panel.style.display = open ? 'none' : 'block';
      toggle.textContent = open ? 'View full proof & citations' : 'Hide full proof & citations';
    });
    panel.appendChild(buildTrace(trace));
    const chips = sourceChips(result);
    if (chips) panel.insertBefore(chips, panel.firstChild);
    target.append(toggle, panel);
  }
  return target;
}

// ---------------------------------------------------------------------------
// THE renderer. Appends the full outcome rendering for one parsed wire
// `result` into `target`. `opts`:
//   onAbstainLoad(term)  — user asked to load a source for an unresolved term
//   onSlotAnswer(text)   — user answered a Conditional's missing fact; the
//                          caller routes it back through the SAME stateful
//                          session (a follow-up chat() turn resolves the rule)
// Returns the outcome card element.
// ---------------------------------------------------------------------------
export function renderResult(result, target, opts = {}) {
  const outcome = result.outcome;
  const card = document.createElement('div');

  // --- outcome card -------------------------------------------------------
  if (outcome === 'abstained') {
    const terms = Array.isArray(result.unresolved) ? result.unresolved : [];
    card.className = 'alert alert-warning';
    card.setAttribute('role', 'status');
    const head = document.createElement('div');
    head.className = 'alert-heading';
    head.textContent = 'Abstained — no grounded answer';
    card.appendChild(head);
    const body = document.createElement('div');
    body.className = 'alert-body';
    body.textContent = result.response;          // wire sentence, verbatim
    card.appendChild(body);
    if (terms.length) {
      // Named unresolved terms -> a next step (load a source).
      const next = document.createElement('div');
      next.className = 'refusal-next';
      const lbl = document.createElement('span');
      lbl.className = 'refusal-terms';
      const shown = terms.map(unresolvedSurface);
      lbl.textContent = shown.length === 1
        ? `Unresolved: “${shown[0]}”.`
        : `Unresolved: ${shown.map(t => `“${t}”`).join(', ')}.`;
      next.appendChild(lbl);
      if (opts.onAbstainLoad) {
        const btn = document.createElement('button');
        btn.className = 'btn btn-secondary';
        btn.style.marginLeft = '0.5rem';
        btn.textContent = 'Load a source →';
        // The WHOLE entry, not just its surface: it carries the source the
        // engine resolved the citation to, and the host routes on that rather
        // than re-deriving it from the characters.
        btn.addEventListener('click', () => opts.onAbstainLoad(terms[0]));
        next.appendChild(btn);
      }
      card.appendChild(next);
    }
    // else: EMPTY unresolved[] (design critique #9) — a generic plain-language
    // refusal with NO term list and NO load button; the wire `response` above
    // is the whole message. Styled warning gold (designed safety behavior),
    // never error red.
  } else if (outcome === 'conditional') {
    card.className = 'alert alert-info';
    card.setAttribute('role', 'status');
    const head = document.createElement('div');
    head.className = 'alert-heading';
    head.textContent = 'Conditional — one fact needed';
    card.appendChild(head);
    if (result.rule_citation) {
      const cite = document.createElement('div');
      cite.className = 'rule-cite';
      cite.textContent = (result.rule_name ? result.rule_name + '\n' : '') + result.rule_citation;
      card.appendChild(cite);
    }
    const body = document.createElement('div');
    body.className = 'alert-body';
    body.textContent = result.response;
    card.appendChild(body);
    const missing = Array.isArray(result.missing_facts) ? result.missing_facts : [];
    for (const field of missing) {
      const slot = document.createElement('div');
      slot.className = 'slot';
      const label = document.createElement('label');
      label.textContent = field;
      slot.appendChild(label);
      if (opts.onSlotAnswer) {
        const controls = document.createElement('div');
        controls.className = 'slot-controls';
        // Boolean premises dominate these rules; offer Yes/No plus a free
        // text answer. Whatever the user picks is routed back through the
        // SAME stateful session as the next chat turn — the engine resolves
        // the pending rule to a verdict. We assemble no sentence here.
        for (const ans of ['Yes', 'No']) {
          const b = document.createElement('button');
          b.className = 'btn btn-secondary';
          b.textContent = ans;
          b.addEventListener('click', () => opts.onSlotAnswer(ans));
          controls.appendChild(b);
        }
        slot.appendChild(controls);
      }
      card.appendChild(slot);
    }
  } else if (outcome === 'rule_applies' || outcome === 'rule_does_not_apply') {
    const applies = outcome === 'rule_applies';
    card.className = 'alert ' + (applies ? 'alert-success' : 'alert-info');
    card.setAttribute('role', 'status');
    const head = document.createElement('div');
    head.className = 'alert-heading';
    head.textContent = applies ? 'Rule applies' : 'Rule does not apply';
    card.appendChild(head);
    if (result.rule_citation) {
      const cite = document.createElement('div');
      cite.className = 'rule-cite';
      cite.textContent = (result.rule_name ? result.rule_name + '\n' : '') + result.rule_citation;
      card.appendChild(cite);
    }
    const body = document.createElement('div');
    body.className = 'alert-body';
    body.textContent = result.response;
    card.appendChild(body);
  } else {
    // answered (or an unknown/legacy shape) — success card.
    card.className = 'alert alert-success';
    card.setAttribute('role', 'status');
    const head = document.createElement('div');
    head.className = 'alert-heading';
    head.textContent = result.from_ontology === false ? 'Answered (general substrate)' : 'Answered';
    card.appendChild(head);
    const body = document.createElement('div');
    body.className = 'alert-body';
    body.textContent = result.response;
    card.appendChild(body);
    const chips = sourceChips(result);
    if (chips) card.appendChild(chips);
  }
  target.appendChild(card);

  // --- Why this result? layer — the stepped proof path (design brief), not
  // the old plain-text panel: icon + bold label + one-line detail per real
  // wire fact (rule/citation/missing-facts, or the why sentence + its
  // deferred evidence chain), collapsed behind one toggle so a chat bubble
  // stays scannable. Renders nothing when there is no real step to show
  // (`buildProofSteps` returns `[]` for a self-explaining outcome).
  const steps = buildProofSteps(result);
  if (steps.length) {
    const wrap = document.createElement('div');
    const btn = document.createElement('button');
    btn.className = 'why-toggle';
    btn.setAttribute('aria-expanded', 'false');
    btn.textContent = 'Why this result? ▸';
    const panel = document.createElement('div');
    panel.className = 'why-panel';
    btn.addEventListener('click', () => {
      const open = panel.getAttribute('data-open') === 'true';
      panel.setAttribute('data-open', String(!open));
      btn.setAttribute('aria-expanded', String(!open));
      btn.textContent = open ? 'Why this result? ▸' : 'Why this result? ▾';
    });
    // No nested trace toggle here — the separate accordion below already
    // covers the full typed trace for this same bubble.
    renderProofPath(result, panel, { includeTraceToggle: false });
    wrap.append(btn, panel);
    target.appendChild(wrap);
  }

  // --- technical trace (deepest level) ------------------------------------
  // Prefer the TYPED trace (per-step literature references); fall back to the
  // legacy flattened string for an older engine build (feature-detected).
  const trace = Array.isArray(result.trace_structured) ? result.trace_structured : result.trace;
  if (trace !== undefined && trace !== null && trace !== '') {
    target.appendChild(buildTrace(trace));
  }

  // --- reasoned-over footer ----------------------------------------------
  const onts = Array.isArray(result.ontologies) ? result.ontologies : [];
  const footer = document.createElement('div');
  footer.className = 'reasoned-footer';
  const dur = fmtDuration(result.duration_us);
  const stat = document.createElement('span');
  stat.textContent = `${onts.length} ontolog${onts.length === 1 ? 'y' : 'ies'} reasoned over${dur ? ' · ' + dur : ''} · in-browser`;
  footer.appendChild(stat);
  footer.appendChild(buildRecordDownload(result));
  target.appendChild(footer);

  target.scrollTop = target.scrollHeight;
  return card;
}

// ---------------------------------------------------------------------------
// The decision record — one turn, as a file a compliance function can keep.
//
// A regulated deployer's obligation is not "the interface showed a reason", it
// is "produce the record" — months later, without the browser session that made
// it. So this writes the engine's own wire envelope (the typed outcome, the
// unresolved terms, the governing rule and its citation, the full typed trace,
// the ontologies consulted) to a JSON file the visitor keeps. It is composed
// here rather than in the engine on purpose: the page adds only what the page
// alone knows — the question asked, and the wall-clock the browser measured —
// and copies every reasoning field through untouched, so the record and the
// answer on screen cannot say different things.
//
// Built with a Blob and an object URL that is revoked once the download has
// been handed to the browser, so nothing is uploaded and nothing is retained.
// ---------------------------------------------------------------------------
function buildRecordDownload(result) {
  const btn = document.createElement('button');
  btn.className = 'btn-link record-download';
  btn.type = 'button';
  btn.textContent = 'Download this decision record (JSON)';
  btn.title = 'The engine’s own typed record of this turn — written here in your browser, uploaded nowhere.';
  btn.addEventListener('click', () => {
    const record = {
      schema: 'pr4xis.decision-record.v1',
      question: result.question ?? null,
      outcome: result.outcome ?? null,
      response: result.response ?? null,
      unresolved: Array.isArray(result.unresolved) ? result.unresolved : [],
      rule_name: result.rule_name ?? null,
      rule_citation: result.rule_citation ?? null,
      missing_facts: Array.isArray(result.missing_facts) ? result.missing_facts : [],
      ontologies: Array.isArray(result.ontologies) ? result.ontologies : [],
      trace: result.trace_structured ?? result.trace ?? null,
      duration_us: result.duration_us ?? null,
      // WHERE THE WORDING CAME FROM — distinct from `ontologies`, which is what
      // the engine OPENED. A lexicon definition used to carry its authority as
      // a suffix inside the gloss, so `response` alone preserved it and every
      // surface got it for free. Moving it into structured `dcterms:source`
      // made it inspectable and dropped it from the two surfaces that were not
      // rewired: this record and the Smart-40 panel. A compliance reader
      // opening a record months later would have found a definition with no
      // statement of the document it was written from, under a comment
      // promising every reasoning field is copied through.
      //
      // `why` was never carried either — a pre-existing gap, folded in here
      // because the same reader needs both: what was opened, and what the
      // wording was authored from.
      why: result.why ?? null,
      definition_provenance: result.definition_provenance ?? null,
      // WHICH ENGINE, AND UNDER WHAT KNOWLEDGE. Without these the record
      // cannot be re-derived: the same question answers differently depending
      // on which authorities were loaded, and this session loads some of them
      // in the background while the reader is asking. Both are stamped by the
      // engine on the turn itself, so they describe the state that produced
      // THIS answer rather than whatever was loaded by the time the reader
      // pressed download. `state_cid` is the Merkle fold over the loaded
      // ontology roots; null means nothing was loaded.
      engine_version: result.engine_version ?? null,
      state_cid: result.state_cid ?? null,
      produced_by: 'pr4xis, in-browser (WebAssembly). No network call in the query path.',
    };
    const blob = new Blob([JSON.stringify(record, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    // Name the file after the question, so a folder of these records is
    // readable without opening any of them — which is the whole point of
    // keeping them. Falls back to a plain name when the question is absent.
    const slug = String(record.question || 'decision')
      .toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '').slice(0, 60);
    a.download = `pr4xis-${slug || 'decision'}.json`;
    a.click();
    // Revoke on the next macrotask, not synchronously: the download is queued
    // by click() but not necessarily started, and revoking too early can leave
    // a browser with nothing to fetch.
    setTimeout(() => URL.revokeObjectURL(url), 0);
  });
  return btn;
}

// Parse the outcome label ("Answered", "Abstained", "Conditional", ...) from a
// raw wire result. Used by the Smart-40 console to compare published vs live.
export function outcomeLabel(result) {
  const o = result && result.outcome;
  if (o === 'answered') return 'Answered';
  if (o === 'abstained') return 'Abstained';
  if (o === 'conditional') return 'Conditional';
  if (o === 'rule_applies' || o === 'rule_does_not_apply') return 'Conditional';
  return o ? String(o) : '—';
}

// ---------------------------------------------------------------------------
// Send one turn through the stateful chat() path and render it. Returns the
// parsed result (so predict-then-reveal can compare a prediction against it).
// ---------------------------------------------------------------------------
export async function sendChat(engine, text, log, opts = {}) {
  text = (text || '').trim();
  if (!text) return null;
  if (opts.echoUser !== false) addMessage(text, 'user', log);
  let raw;
  // Time the call HERE, on the host clock. The engine's own `duration_us` is
  // measured with `std::time::Instant`, which does not exist on
  // wasm32-unknown-unknown — `WasmSafeTimer::elapsed_us` is cfg'd to return a
  // literal 0 in the browser (crates/chat/src/lib.rs). Reporting that 0 as a
  // measurement would put a fabricated number on a page whose stated pledge is
  // that every number is computed here, now. `performance.now()` is the clock
  // the browser actually has, so the browser measures what the browser ran.
  const t0 = performance.now();
  try {
    raw = await engine.call('chat', { input: text });
  } catch (e) {
    addMessage(`error: ${e}`, 'info', log);
    return null;
  }
  const elapsedMs = performance.now() - t0;
  let result;
  try { result = JSON.parse(raw); }
  catch { addMessage(raw, 'system', log); return null; }
  // Prefer the engine's own figure where it is real (native builds, CLI and
  // tests measure correctly); fall back to the host measurement in the browser.
  if (!(typeof result.duration_us === 'number' && result.duration_us > 0)) {
    result.duration_us = elapsedMs * 1000;
  }
  renderResult(result, log, opts);
  return result;
}
