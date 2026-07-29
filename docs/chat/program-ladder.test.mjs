// Smoke-test the Program Ladder renderer against the REAL shipped corpus,
// with a minimal DOM shim. Verifies: no runtime throw, every number, the
// rail widths, that the rows sum to the bank, and that no '%' or '→' reaches
// rendered TEXT (CSS custom-property values are allowed to be percentages).
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const SRC = new URL('./dashboard.js', import.meta.url).pathname;
let src = fs.readFileSync(SRC, 'utf8');

// ---- minimal DOM ----
class El {
  constructor(tag) {
    this.tag = tag; this.children = []; this.className = ''; this.style = new Style();
    this.attrs = {}; this._text = null;
  }
  set textContent(v) { this._text = String(v); this.children = []; }
  get textContent() {
    if (this._text !== null) return this._text;
    return this.children.map((c) => (typeof c === 'string' ? c : c.textContent)).join('');
  }
  set innerHTML(v) { this._text = String(v); }
  appendChild(c) { this.children.push(c); this._text = null; return c; }
  append(...cs) { for (const c of cs) this.children.push(c); this._text = null; }
  replaceChildren(...cs) { this.children = cs; this._text = null; }
  setAttribute(k, v) { this.attrs[k] = v; }
  querySelectorAll() { return []; }
  get classList() {
    const self = this;
    return { add(c) { self.className += ' ' + c; }, remove() {}, toggle() {} };
  }
}
class Style {
  constructor() { this.props = {}; }
  setProperty(k, v) { this.props[k] = v; }
}
const MOUNTS = {};
for (const id of ['track1-roadmap', 'track2-roadmap', 'evidence-roadmap', 'evidence-kpis',
                  't-snapshot', 't-corpus', 'corpus-version-chip']) MOUNTS[id] = new El('div');

globalThis.document = {
  createElement: (t) => new El(t),
  getElementById: (id) => MOUNTS[id] || null,
  querySelectorAll: () => [],
  documentElement: { },
};
// node 24 already provides a real webcrypto; only shim when absent.
globalThis.fetch = async () => { throw new Error('no network in smoke test'); };
globalThis.window = { getSelection: () => ({}) };
// node 24 provides `navigator`; the renderer never touches it anyway.

// Strip the one import (chat-ui.js pulls in browser-only code) and expose internals.
src = src.replace(/^import .*$/m, 'const outcomeLabel = () => "";\nconst ontologyLabel = () => "";');
src += `
export { state, roadmapModel, renderRoadmap, renderTrackRoadmap, poolForSub, ROADMAP_COPY, computeKpis, HEADLINE_KPI_DEFS };
`;
// The renderer is written for the browser; to exercise it in node it has to
// be loaded as a module, so it is written once to a temp file with its one
// browser-only import stubbed. os.tmpdir() keeps this machine-independent.
const tmp = path.join(os.tmpdir(), `pr4xis-program-ladder-${process.pid}.mjs`);
fs.writeFileSync(tmp, src);
const M = await import(tmp);
fs.rmSync(tmp, { force: true });

// ---- real corpus ----
const slim = JSON.parse(fs.readFileSync(new URL('../caregiver-corpus-slim.json', import.meta.url).pathname, 'utf8'));
M.state.slim = slim.questions;
const status = JSON.parse(fs.readFileSync(new URL('../caregiver-corpus-status.json', import.meta.url).pathname, 'utf8'));
M.state.ceilings = (status.ceilings && typeof status.ceilings === 'object') ? status.ceilings : null;

// Also exercise the cap-tick path with the ceilings the committed ratchet holds.
// The SHIPPED ceilings block, read from the artifact rather than retyped, so
// this harness exercises exactly the object the page receives.
const RATCHET = status.ceilings;
// A deliberately incomplete ceilings object: the page must degrade to prose,
// never print NaN, if a regenerator ever omits a key.
const RATCHET_PARTIAL = { missing_term: 2786, unparsed_known_term: 1223, possible_misroute: 66 };

let fail = 0;
const check = (ok, msg) => { console.log(`${ok ? 'ok  ' : 'FAIL'}  ${msg}`); if (!ok) fail++; };

for (const [sub, mount] of [['overall', 'evidence-roadmap'], ['track1', 'track1-roadmap'], ['track2', 'track2-roadmap']]) {
  const pool = M.poolForSub(sub);
  const m = M.roadmapModel(pool, sub === 'overall' ? RATCHET : null);
  const sum = m.rows.reduce((a, r) => a + r.count, 0);
  console.log(`\n== ${sub} ==  bank=${m.bank}  sources=${m.sources}  shared=${m.shared}`);
  for (const r of m.rows) console.log(`   ${r.stage.padEnd(6)} ${String(r.count).padStart(5)}  len=${r.len.padStart(7)}  cap=${r.cap ?? '—'}  ratchet=${r.ratchet}  ${r.title}`);
  check(sum === m.bank, `${sub}: rows sum (${sum}) === bank (${m.bank})`);
  check(m.rows.length === 4, `${sub}: exactly the four declared classes, no untriaged label`);
  check(m.rows[0].stage === 'Done' && m.rows[0].ratchet === 'floor', `${sub}: Done leads and is the floor`);
  check(Math.max(...m.rows.map((r) => parseFloat(r.len))) === 100, `${sub}: scale max is the largest ROW (one row at 100%)`);
  check(parseFloat(m.rows[0].len) < 100, `${sub}: bank is NOT the scale (Done < 100%)`);

  // Render for real.
  M.renderRoadmap(mount, sub, sub === 'overall' ? RATCHET : null);
  const host = MOUNTS[mount];
  const text = host.children.map((c) => c.textContent).join(' | ');
  check(host.children.length === 4, `${sub}: mount got heading + bank + rows + foot`);
  check(!/%/.test(text), `${sub}: NO percentage in rendered text`);
  check(!/→/.test(text), `${sub}: NO arrow in rendered text`);
  check(!/\b(NaN|undefined|null)\b/.test(text), `${sub}: no NaN/undefined/null reaches rendered text`);
  check(text.includes(m.bank.toLocaleString()), `${sub}: bank caption states ${m.bank.toLocaleString()}`);
  check(text.includes(String(m.sources)), `${sub}: source count ${m.sources} is live in the caption`);
  const capsDrawn = [...host.children[2].children].filter((li) => li.children.some((c) => c.className === 'rm-rail' && c.children.some((x) => x.className === 'rm-cap'))).length;
  check(capsDrawn === (sub === 'overall' ? 3 : 0), `${sub}: cap ticks drawn = ${capsDrawn} (ceilings are corpus-wide)`);
  // Every row carries --rm-len; ceilinged rows on the overall mount carry --rm-cap.
  const lens = [...host.children[2].children].map((li) => li.style.props['--rm-len']);
  check(lens.every((v) => typeof v === 'string' && v.endsWith('%')), `${sub}: every row has a --rm-len`);
  console.log(`   foot: ${host.children[3].textContent}`);
}

// The one headline tile. It reports a LIVE sample, not a corpus label — the
// label it used to count (`OverAnswered`) can no longer be emitted by
// `classify_case` at all, so counting it produced a confident zero that
// measured nothing. Before the check runs the tile must say so.
check(M.HEADLINE_KPI_DEFS.length === 1, 'headline: exactly one KPI tile survives');
const kPending = M.computeKpis(M.poolForSub('overall'));
check(kPending.liveGrounded === null, 'headline: pending until the live check has run');
check(M.HEADLINE_KPI_DEFS[0].fmt(kPending) === '—',
  'headline: renders a dash, never a zero it has not measured');
check(/run the/i.test(M.HEADLINE_KPI_DEFS[0].sub(kPending)),
  'headline: the pending caption says how to fill it in');

// Once the check has run, the tile reports what it observed.
M.state.liveCitationSample = { answered: 5, named: 5, fromLexicon: 3 };
const kLive = M.computeKpis(M.poolForSub('overall'));
check(kLive.liveGrounded === 3, 'headline: reports answers drawing on a loaded authority');
check(M.HEADLINE_KPI_DEFS[0].fmt(kLive) === '3', 'headline: renders the measured figure');
const sub0 = M.HEADLINE_KPI_DEFS[0].sub(kLive);
check(!/%/.test(sub0), 'headline: no percentage');
check(/sampled live/.test(sub0), 'headline: names the live denominator it measured against');

// THE regression this tile has now had twice: a figure that cannot move.
//
// Both previous versions were structural zeroes — first a corpus tag the
// engine can no longer emit, then `answered - named`, which is identically 0
// because every turn's trace carries at least one Compiled entry. A test that
// feeds equal counts and asserts 0 cannot tell a real measurement from a
// constant. So feed a sample where every answer used ONLY compiled general
// vocabulary and one where every answer reached a loaded lexicon: the tile
// must read differently. If it does not, it is measuring nothing again.
M.state.liveCitationSample = { answered: 4, named: 4, fromLexicon: 0 };
const kNoneGrounded = M.computeKpis(M.poolForSub('overall'));
M.state.liveCitationSample = { answered: 4, named: 4, fromLexicon: 4 };
const kAllGrounded = M.computeKpis(M.poolForSub('overall'));
check(kNoneGrounded.liveGrounded === 0 && kAllGrounded.liveGrounded === 4,
  'headline: the figure MOVES with what was actually loaded (0 vs 4 on identical answered/named)');
check(M.HEADLINE_KPI_DEFS[0].fmt(kNoneGrounded) !== M.HEADLINE_KPI_DEFS[0].fmt(kAllGrounded),
  'headline: two genuinely different samples must not render the same number');
M.state.liveCitationSample = null;

// A relabelling upstream must still be accounted for.
const mutated = M.poolForSub('overall').map((e, i) => (i < 7 ? { ...e, label: 'BrandNewLabel' } : e));
const mm = M.roadmapModel(mutated, null);
check(mm.rows.some((r) => r.untriaged && r.count === 7), 'undeclared label gets a Triage row (nothing silently drops)');
check(mm.rows.reduce((a, r) => a + r.count, 0) === mm.bank, 'with an undeclared label, rows still sum to the bank');

// ---- degraded-artifact path: a ceilings block missing green_floor ----------
// A regenerator that ever drops a key must produce prose, never "NaN".
M.renderRoadmap('evidence-roadmap', 'overall', RATCHET_PARTIAL);
const degradedFoot = MOUNTS['evidence-roadmap'].children[3].textContent;
check(!/\b(NaN|undefined|null)\b/.test(degradedFoot),
  'partial ceilings: foot degrades to prose rather than printing NaN');
check(/committed floor/.test(degradedFoot),
  'partial ceilings: the floor is still named, just without a figure');

console.log(fail ? `\n${fail} FAILURE(S)` : '\nall checks passed');
process.exit(fail ? 1 : 0);
