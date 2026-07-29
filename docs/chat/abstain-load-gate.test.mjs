// The abstention → source-load path, gated.
//
// When the engine declines and names an unresolved term, the answer offers
// "Load a source →". The engine hands over `{surface, source_id?}` — the term,
// and the source it resolved the citation to. `goLoadSource` routes to the
// Engine tab on that source and decides whether to START the download or hand
// the choice to the reader. That decision spends the reader's data, so it is
// worth a test rather than a careful reading:
//
//   1. A small title loads on sight.
//   2. A large one does NOT — it is focused and ringed, and nothing is
//      clicked, because Title 42's archive is 34.86 MiB and a caregiver on a
//      metered connection did not ask for that.
//   3. `AUTO_LOAD_MAX_BYTES` is readable from inside `goLoadSource` at call
//      time even though it is declared further down the module — a `const` in
//      its temporal dead zone would throw here, and the throw would land
//      exactly when someone followed a decline.
//   4. A term that names no title still filters the catalog rather than
//      stranding the reader.
//
// The functions live in an inline `<script type="module">` in index.html, so
// they are sliced out by name and given a stub DOM. Slicing is deliberate:
// the alternative is evaluating the whole module, which boots a Web Worker.
import fs from 'node:fs';

const here = (p) => new URL(p, import.meta.url).pathname;
const html = fs.readFileSync(here('./index.html'), 'utf8');

// ---- slice the three declarations under test --------------------------------
function slice(startPattern, endPattern) {
  const a = html.indexOf(startPattern);
  if (a < 0) throw new Error(`not found in index.html: ${startPattern}`);
  const b = html.indexOf(endPattern, a);
  if (b < 0) throw new Error(`end not found after: ${startPattern}`);
  return html.slice(a, b);
}
const fnAccessors = slice('const surfaceOf =', '\n    function goLoadSource');
const fnGoLoad = slice('function goLoadSource(unresolved) {', '\n    async function initialize()');
// The budget is now a `let` the engine fills in at boot, not a literal. The
// harness declares its own so the size-gate scenarios below can set it — what
// is under test here is the DECISION `goLoadSource` makes given a budget, not
// where the number comes from (that is the engine's `auto_load_budget`, covered
// by `the_auto_load_budget_is_the_catalogs_own_natural_break`).
const declLine = html.match(/let AUTO_LOAD_MAX_BYTES = [^;]+;/);
if (!declLine) throw new Error('AUTO_LOAD_MAX_BYTES declaration not found in index.html');

// ---- minimal DOM ------------------------------------------------------------
const events = [];
class El {
  constructor(tag) {
    this.tag = tag; this.dataset = {}; this.className = ''; this.value = '';
    this.clicked = 0; this.focused = 0; this.parentElement = null; this.children = [];
    this.style = {};
  }
  addEventListener() {}
  dispatchEvent(e) { events.push(`${this.tag}:${e.type}`); }
  click() { this.clicked += 1; events.push(`click:${this.dataset.loadFast || this.dataset.load}`); }
  focus() { this.focused += 1; events.push('focus'); }
  scrollIntoView() { events.push('scroll'); }
  querySelector() { return null; }
  get classList() {
    const self = this;
    return { add(c) { self.className += ` ${c}`; }, remove() {}, toggle() {}, contains: (c) => self.className.includes(c) };
  }
}

let fail = 0;
const check = (ok, msg) => { console.log(`${ok ? 'ok  ' : 'FAIL'}  ${msg}`); if (!ok) fail += 1; };

// One scenario = one fresh catalog.
function scenario(buttons) {
  events.length = 0;
  const filter = new El('input');
  const grid = new El('div');
  filter.parentElement = { querySelector: () => grid };
  const byLoadFast = new Map();
  const byLoad = new Map();
  for (const [name, bytes, kind] of buttons) {
    const b = new El('button');
    b.dataset.bytes = String(bytes);
    if (kind === 'fast') { b.dataset.loadFast = name; byLoadFast.set(name, b); }
    else { b.dataset.load = name; byLoad.set(name, b); }
  }
  globalThis.document = {
    getElementById: (id) => (id === 'src-filter' ? filter : null),
    querySelector: (sel) => {
      let m = sel.match(/\[data-load-fast="([^"]+)"\]/);
      if (m) return byLoadFast.get(m[1]) || null;
      m = sel.match(/\[data-load="([^"]+)"\]/);
      if (m) return byLoad.get(m[1]) || null;
      return null;
    },
  };
  globalThis.Event = class { constructor(type) { this.type = type; } };
  globalThis.navigate = () => events.push('navigate');
  return { filter, grid, byLoadFast, byLoad };
}

const src = `${declLine[0]}\n${fnAccessors}\n${fnGoLoad}\n`
  + `function setBudget(b) { AUTO_LOAD_MAX_BYTES = b; }\n`
  + `function getBudget() { return AUTO_LOAD_MAX_BYTES; }\n`
  + `export { goLoadSource, surfaceOf, sourceIdOf, setBudget, getBudget };\n`;
const tmp = here('./_abstain_gate_tmp.mjs');
fs.writeFileSync(tmp, src);
const M = await import(tmp);
fs.rmSync(tmp, { force: true });

// Before the engine answers, the budget is 0 — so EVERY load is presented
// rather than started. Erring toward asking costs a click; erring the other way
// spends a metered connection's data on someone's behalf.
check(M.getBudget() === 0,
  'the budget starts at 0, so nothing downloads unasked until the engine answers');
{
  const { byLoadFast } = scenario([['usc_title_18', 3.43 * 1024 * 1024, 'fast']]);
  M.goLoadSource({ surface: '18 U.S.C. § 1001', source_id: 'usc_title_18' });
  check(byLoadFast.get('usc_title_18').clicked === 0,
    'and a small title is presented, not started, while the budget is unknown');
}
// The scenarios below exercise the DECISION given a budget the engine supplied.
M.setBudget(5 * 1024 * 1024);

// The page READS the route the engine resolved; it no longer derives one.
//
// The two checks this replaces asserted that a regex in the page extracted
// `usc_title_42` from "42 U.S.C. § 1396n" — which ratcheted an encoded
// citation grammar and an encoded registry naming convention in as tested
// behaviour. Recognition now happens in the engine, against each registered
// title's own published citation forms (see `title_cited_by`, covered by
// `a_citation_surface_routes_to_the_title_it_cites`). What is left to test
// here is that the page reads the field and never re-derives it.
check(M.sourceIdOf({ surface: '42 U.S.C. § 1396n', source_id: 'usc_title_42' }) === 'usc_title_42',
  'the page routes on the source the engine named');
check(M.sourceIdOf({ surface: 'respite care' }) === null,
  'an entry the engine could not route carries no source');
check(M.surfaceOf({ surface: 'respite care' }) === 'respite care',
  'the surface is read for display');
check(M.sourceIdOf('42 U.S.C. § 1396n') === null,
  'a bare-string entry (older wasm) routes nowhere rather than being re-parsed here');
check(M.surfaceOf('respite care') === 'respite care',
  'and a bare-string entry still shows its term');
check(!/uscTitleFromTerm/.test(html),
  'the page-side citation regex is gone, not merely bypassed');
// Comment lines are excluded: the comment that explains why the convention was
// removed necessarily quotes it, and a check that cannot survive its own
// explanation would just get the explanation deleted.
const code = html.split('\n').filter((l) => !/^\s*(\/\/|\*|\/\*)/.test(l)).join('\n');
check(!/usc_title_\$\{/.test(code),
  'and the registry naming convention is not rebuilt anywhere in the page');

// 1. Small title — loads on sight. (Title 18's archive is 3.43 MiB.)
{
  const { byLoadFast } = scenario([['usc_title_18', 3.43 * 1024 * 1024, 'fast']]);
  M.goLoadSource({ surface: '18 U.S.C. § 1001', source_id: 'usc_title_18' });
  const btn = byLoadFast.get('usc_title_18');
  check(btn.clicked === 1, 'a title under the threshold starts loading immediately');
  check(btn.focused === 0 && !btn.className.includes('load-btn-awaiting'),
    'and is not left waiting on the reader');
}

// 2. Large title — presented, never started. (Title 42's archive is 34.86 MiB.)
{
  const { byLoadFast } = scenario([['usc_title_42', 34.86 * 1024 * 1024, 'fast']]);
  M.goLoadSource({ surface: '42 U.S.C. § 1396n', source_id: 'usc_title_42' });
  const btn = byLoadFast.get('usc_title_42');
  check(btn.clicked === 0, 'a title over the threshold does NOT download unasked');
  check(btn.focused === 1, 'the reader is put on the control that would start it');
  check(btn.className.includes('load-btn-awaiting'), 'and it is marked as waiting on them');
  check(events.includes('scroll'), 'after scrolling it into view');
}

// 3. The raw-XML fallback carries its size too, so the gate holds on that path.
{
  const { byLoad } = scenario([['usc_title_42', 34.86 * 1024 * 1024, 'raw']]);
  M.goLoadSource({ surface: '42 U.S.C. § 1396n', source_id: 'usc_title_42' });
  check(byLoad.get('usc_title_42').clicked === 0,
    'the raw-XML route is gated by size as well as the archive route');
}

// 3b. An UNKNOWN size is not a small one. A button the catalog gave no size
//     takes the same branch as an over-budget one, because you cannot spend
//     data you have not measured without asking. (Nothing reaches this today —
//     every button `goLoadSource` looks up carries `data-bytes` — but the old
//     `|| 0` read "no size" as "free", and that default would have gone live
//     silently the first time a manifest recorded 0.)
{
  const { byLoadFast } = scenario([['usc_title_18', 0, 'fast']]);
  const btn = byLoadFast.get('usc_title_18');
  delete btn.dataset.bytes;
  M.goLoadSource({ surface: '18 U.S.C. § 1001', source_id: 'usc_title_18' });
  check(btn.clicked === 0, 'a source with no published size does NOT download unasked');
  check(btn.focused === 1, 'the reader is asked about it instead');
}

// 4. An ordinary term filters the catalog by its own text — when something
//    actually matches.
{
  const { filter, grid } = scenario([]);
  grid.children = [{ style: { display: '' } }];   // one card survives the filter
  M.goLoadSource({ surface: 'respite care' });
  check(filter.value === 'respite care', 'an ordinary term filters the catalog by its own text');
}

// 5. ...and when NOTHING matches, the filter clears rather than leaving the
//    reader staring at an empty grid they cannot explain. This is the branch
//    the code comments call "don't strand the user", and it is the one a
//    caregiver hits most: most unresolved terms are not USC citations and
//    name no source in the catalog at all.
{
  const { filter, grid } = scenario([]);
  grid.children = [];                              // nothing matches
  M.goLoadSource({ surface: 'respite care' });
  check(filter.value === '', 'a term matching no source clears the filter instead of stranding');
}

console.log(fail ? `\n${fail} FAILURE(S)` : '\nall checks passed');
process.exit(fail ? 1 : 0);
