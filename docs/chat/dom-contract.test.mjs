// The demonstrator's DOM contract, gated.
//
// This exists because of a real boot failure: dashboard.js bound four element
// ids that index.html no longer carried, so `$(id).addEventListener` threw on
// `null` and the ENTIRE caregiver tab rendered blank — while every Rust test
// stayed green, because none of them can see an HTML id. A judge opening the
// published URL would have met an empty page.
//
// Three checks, each the mechanical form of a failure that actually happened:
//
//   1. Every id the scripts look up exists in index.html.
//   2. Every named import resolves to a real export.
//   3. Both modules parse.
//
// It reads the shipped files as text rather than executing them, so it needs
// no browser and no DOM — the point is the contract between the markup and
// the scripts, which is checkable statically and was not being checked.
import fs from 'node:fs';

const here = (p) => new URL(p, import.meta.url).pathname;
const html = fs.readFileSync(here('./index.html'), 'utf8');
const sources = {
  'dashboard.js': fs.readFileSync(here('./dashboard.js'), 'utf8'),
  'chat-ui.js': fs.readFileSync(here('./chat-ui.js'), 'utf8'),
  'index.html': html,
};

let fail = 0;
const check = (ok, msg) => { console.log(`${ok ? 'ok  ' : 'FAIL'}  ${msg}`); if (!ok) fail += 1; };

// ---- 1. every looked-up id exists ------------------------------------------
// "Exists" means declared in the markup OR assigned by the scripts themselves
// (`el.id = 'x'`), because parts of the source catalog are built at runtime.
// Either way the id has a definition somewhere in the shipped files; what this
// catches is the id that has none.
const declared = new Set([
  ...[...html.matchAll(/\bid="([A-Za-z0-9_-]+)"/g)].map((m) => m[1]),
]);
for (const src of Object.values(sources)) {
  for (const m of src.matchAll(/\.id\s*=\s*'([A-Za-z0-9_-]+)'/g)) declared.add(m[1]);
}
let missing = 0;
for (const [file, src] of Object.entries(sources)) {
  // `$('x')` is the shared lookup helper; getElementById is the raw form.
  const looked = new Set([
    ...[...src.matchAll(/\$\('([A-Za-z0-9_-]+)'\)/g)].map((m) => m[1]),
    ...[...src.matchAll(/getElementById\('([A-Za-z0-9_-]+)'\)/g)].map((m) => m[1]),
  ]);
  for (const id of looked) {
    if (!declared.has(id)) { console.log(`      ${file} looks up #${id}, which index.html does not declare`); missing += 1; }
  }
}
check(missing === 0, `every id the scripts look up is declared in index.html (${declared.size} declared)`);

// ---- 2. every named import resolves to a real export ------------------------
const exportsOf = (src) => new Set(
  [...src.matchAll(/^export\s+(?:async\s+)?(?:function|const|let|class)\s+([A-Za-z0-9_$]+)/gm)].map((m) => m[1]),
);
const moduleExports = {
  './dashboard.js': exportsOf(sources['dashboard.js']),
  './chat-ui.js': exportsOf(sources['chat-ui.js']),
};
let unresolved = 0;
for (const [file, src] of Object.entries(sources)) {
  for (const m of src.matchAll(/import\s*\{([^}]+)\}\s*from\s*'(\.\/[A-Za-z0-9._-]+)'/g)) {
    const target = moduleExports[m[2]];
    if (!target) continue;                       // not one of ours
    for (const raw of m[1].split(',')) {
      const name = raw.trim().split(/\s+as\s+/)[0].trim();
      if (!name) continue;
      if (!target.has(name)) { console.log(`      ${file} imports { ${name} } from ${m[2]}, which does not export it`); unresolved += 1; }
    }
  }
}
check(unresolved === 0, 'every named import resolves to a real export');

// ---- 3. both modules parse ---------------------------------------------------
// A syntax error in either file blanks the app exactly like a null binding.
for (const name of ['dashboard.js', 'chat-ui.js']) {
  let ok = true;
  try {
    // Import the module URL to force a real parse. Execution is not attempted:
    // these are browser modules. `new Function` would not see import syntax,
    // so parse via the loader and treat a SyntaxError as the failure.
    new (Function.prototype.bind.call(Function, null))();  // no-op; keeps the shape explicit
    await import(here(`./${name}`)).catch((e) => { if (e instanceof SyntaxError) throw e; });
  } catch (e) {
    ok = false;
    console.log(`      ${name}: ${e}`);
  }
  check(ok, `${name} parses`);
}

// ---- 4. every hash route linked to is a route the router knows --------------
// Copy on this page points readers at views by hash. When a view is renamed or
// removed, a stale `href="#..."` sends a reviewer to a blank panel — which is
// the same failure as a null binding, one layer up. The router's own table is
// the source of truth.
const routes = new Set([...html.matchAll(/^\s*'([a-z0-9/-]+)':\s*\{\s*tab:/gm)].map((m) => m[1]));
// The three tabs are routes in their own right.
for (const t of [...html.matchAll(/data-tab="([a-z0-9-]+)"/g)].map((m) => m[1])) routes.add(t.replace(/-tab$/, ''));
let stale = 0;
for (const [file, src] of Object.entries(sources)) {
  // Literal hrefs only. Routes built by concatenation are covered below, by
  // checking the values that feed the concatenation.
  const linked = new Set([
    ...[...src.matchAll(/href="#([a-z0-9/-]+)"/g)].map((m) => m[1]),
    ...[...src.matchAll(/href=\\?'#([a-z0-9/-]+)\\?'/g)].map((m) => m[1]),
  ]);
  for (const r of linked) {
    if (!routes.has(r)) { console.log(`      ${file} links to #${r}, which the router does not define`); stale += 1; }
  }
}
// The Evidence Lab builds its route as `#caregiver/evidence/` + the button's
// own `data-sub`. Every value that markup can supply must be a real route.
for (const m of html.matchAll(/data-sub="([a-z0-9-]+)"/g)) {
  const r = `caregiver/evidence/${m[1]}`;
  if (!routes.has(r)) { console.log(`      index.html carries data-sub="${m[1]}", but #${r} is not a route`); stale += 1; }
}
check(stale === 0, `every linked hash route is defined by the router (${routes.size} routes)`);

// ---- 5. every principle grade is one the submitted appendices use ----------
// The demonstrator and the appendices grade the same seven ACL principles. A
// status the badge table does not know renders as NO badge at all — silently,
// so the page looks fine while a graded principle has quietly lost its grade.
// The appendices use exactly three words; this pins the demo to them.
const dash = sources['dashboard.js'];
const gradeTable = dash.match(/const PRINCIPLE_GRADES = \{([\s\S]*?)\n\};/);
const knownGrades = new Set(
  gradeTable ? [...gradeTable[1].matchAll(/^\s*'?([a-z-]+)'?:/gm)].map((m) => m[1]) : [],
);
const used = [...dash.matchAll(/status:\s*'([a-z-]+)'/g)].map((m) => m[1])
  .filter((s) => s !== 'pending');   // the honesty panel's own runtime state
let ungraded = 0;
for (const s of new Set(used)) {
  if (!knownGrades.has(s)) { console.log(`      status '${s}' has no entry in PRINCIPLE_GRADES — it would render no badge`); ungraded += 1; }
}
check(knownGrades.size > 0 && ungraded === 0,
  `every principle status maps to a grade (${knownGrades.size} grades, ${new Set(used).size} in use)`);

// ---- 6. every data-* attribute a script READS is one some script WRITES ----
// The size gate in `goLoadSource` first reached for the catalog's own lookup
// maps, which are `const` inside the render function — a ReferenceError that
// would have fired exactly when a caregiver followed an abstention. The fix
// was to carry the fact on the element (`data-bytes`), and this pins that
// contract: a dataset key read but never written is the same dangling
// reference one layer along, and it fails silently as `undefined` rather than
// loudly as a throw.
const DATASET_WRITE = /\.dataset\.([A-Za-z0-9_]+)\s*=/g;
const DATASET_READ = /\.dataset\.([A-Za-z0-9_]+)\b(?!\s*=[^=])/g;
const written = new Set();
for (const src of Object.values(sources)) {
  for (const m of src.matchAll(DATASET_WRITE)) written.add(m[1]);
}
// Attributes authored directly in the markup count as written too.
for (const m of html.matchAll(/\bdata-([a-z0-9-]+)=/g)) {
  written.add(m[1].replace(/-([a-z])/g, (_, c) => c.toUpperCase()));
}
let dangling = 0;
for (const [file, src] of Object.entries(sources)) {
  for (const m of src.matchAll(DATASET_READ)) {
    if (!written.has(m[1])) { console.log(`      ${file} reads .dataset.${m[1]}, which nothing writes`); dangling += 1; }
  }
}
check(dangling === 0, `every dataset key read is one something writes (${written.size} written)`);

// --- Unload is gated on residency, never on staging -------------------------
// The page once decided releasability with `s.staging !== 'embedded'`. That
// reads the medium (where the bytes came from) as if it were the choice (who
// asked for them), and the two come apart precisely on the bases that ship in
// the box: they are embedded AND resident, but the engine reported them as
// `async` because `loaded_refs` tagged every runtime ontology that way. So the
// guard passed, the cards grew an Unload button, and pressing it removed a base
// no Load button could restore — a one-way door in a demo whose whole claim is
// that knowledge is loaded rather than trained.
//
// The engine now computes `releasable` from residency and the page reads it.
// This pins that: no staging-derived releasability test may come back.
const unloadCalls = [...html.matchAll(/if\s*\(([^)]*)\)\s*[^\n]*unloadButton\(/g)].map((m) => m[1]);
check(
  unloadCalls.length > 0,
  `the page still offers Unload (${unloadCalls.length} guarded call sites)`,
);
check(
  unloadCalls.every((g) => /\breleasable\b/.test(g)),
  'every unloadButton call is guarded on the engine-computed `releasable`',
);
check(
  !unloadCalls.some((g) => /\bstaging\b/.test(g)),
  'no unloadButton call infers releasability from `staging`',
);

// --- The boot disclosure agrees with what the deployment actually fetches ---
// The page tells a reader it will fetch "six ... vocabularies ... and Title 42".
// That sentence is authored prose, but what it describes is declared data — the
// `EAGER` list in crates/wasm/build.rs — and nothing coupled them. Twelve lines
// above it the page promises that every number on it is computed live or names
// the command that re-derives it, and "six" was neither.
//
// A regex over English is defensible exactly here: this asserts agreement
// between two authored artifacts, not runtime behaviour. Add a vocabulary to
// EAGER and the sentence goes stale silently; this makes it go red instead.
const buildRs = fs.readFileSync(here('../../crates/wasm/build.rs'), 'utf8');
const eagerBlock = buildRs.match(/const EAGER: &\[&str\] = &\[([\s\S]*?)\n\s*\];/);
check(!!eagerBlock, 'build.rs still declares an EAGER residency list');
if (eagerBlock) {
  const eager = [...eagerBlock[1].matchAll(/"([a-z0-9_]+)"/g)].map((m) => m[1]);
  const vocabs = eager.filter((n) => !n.startsWith('usc_'));
  const titles = eager.filter((n) => n.startsWith('usc_title_'));
  const NUMBER_WORD = ['zero', 'one', 'two', 'three', 'four', 'five', 'six',
    'seven', 'eight', 'nine', 'ten'];
  const word = NUMBER_WORD[vocabs.length] ?? String(vocabs.length);
  check(
    html.includes(`${word} citation and provenance vocabularies`),
    `the boot disclosure says "${word}" vocabularies, matching EAGER's ${vocabs.length}`,
  );
  for (const t of titles) {
    const n = t.replace('usc_title_', '');
    check(
      new RegExp(`Title&nbsp;${n}\\b`).test(html),
      `the boot disclosure names Title ${n}, which EAGER fetches`,
    );
  }
  // And the disclosure must reach the DEFAULT tab, not only a hidden one.
  const chatTab = html.slice(html.indexOf('id="chat-tab"'), html.indexOf('id="caregiver-tab"'));
  check(
    /downloads once/.test(chatTab) && /background/.test(chatTab),
    'the transfer disclosure appears on the default (Chat) tab, before any navigation',
  );
}

// --- Each proof step is a DISTINCT fact, and its wording comes from the engine
// The "Why this result?" panel once listed the same population twice: a
// "Grounded in" step realized from `why` (whose {sources} slot the engine fills
// from `reasoned_over`) and a "Sources reasoned over" step the page assembled by
// filtering `result.ontologies` for kind 'loaded'. They are the same set by
// construction, so the panel read as two independent corroborations of one fact
// — while the genuinely third fact, which DOCUMENT a recited definition was
// written from, appeared nowhere.
//
// The engine now sends that third fact as its own record, label included. Two
// things must stay true, and both are checkable in the shipped file:
//   (a) the loaded-ontology list is not rebuilt into a second proof step, and
//   (b) the page never decides what is missing from `why` by reading `why`.
const chatUi = sources['chat-ui.js'];
const proofBody = chatUi.slice(
  chatUi.indexOf('export function buildProofSteps'),
  chatUi.indexOf('export function renderProofPath'),
);
check(
  proofBody.includes('result.definition_provenance'),
  'buildProofSteps renders the engine-sent definition-provenance record',
);
check(
  !/kind\s*===\s*'loaded'/.test(proofBody),
  'buildProofSteps no longer restates the loaded-ontology list as a second step',
);
check(
  !/parseWhy[\s\S]{0,400}?\.(includes|indexOf|match|replace)\(/.test(
    proofBody.slice(proofBody.indexOf('const prov')),
  ),
  'the provenance step is taken from the model, never matched out of the `why` prose',
);

// --- the decision record carries every field a later reader needs -----------
// The record is the artifact a compliance function keeps: its own comment says
// it must be readable "months later, without the browser session that made it".
// Twice now a field has been added to the wire and NOT to the record — first
// `state_cid`/`engine_version`, then `definition_provenance` when a definition's
// authority moved out of the gloss text into structured `dcterms:source`. Both
// times the record still LOOKED complete, because a missing field reads as an
// answer that simply had no authority.
//
// Checked as source text rather than by executing the module: `buildRecordDownload`
// creates a Blob and clicks an anchor, neither of which exists here.
const recordFn = sources['chat-ui.js'].slice(
  sources['chat-ui.js'].indexOf('function buildRecordDownload'),
);
const REQUIRED_RECORD_FIELDS = [
  'question', 'outcome', 'response', 'unresolved',
  'rule_name', 'rule_citation', 'missing_facts', 'ontologies', 'trace',
  'engine_version', 'state_cid',   // which engine, under what knowledge
  'why',                           // what it opened
  'definition_provenance',         // what the wording was authored from
];
let missingField = 0;
for (const f of REQUIRED_RECORD_FIELDS) {
  if (!new RegExp(`^\\s*${f}:`, 'm').test(recordFn)) {
    console.log(`      the decision record omits '${f}'`);
    missingField += 1;
  }
}
check(missingField === 0,
  `the decision record carries every field a later reader needs (${REQUIRED_RECORD_FIELDS.length})`);

console.log(fail ? `\n${fail} FAILURE(S)` : '\nall checks passed');
process.exit(fail ? 1 : 0);
