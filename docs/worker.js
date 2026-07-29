// pr4xis Web Worker — the whole engine lives here so the main thread (UI)
// never blocks. Downloading a source's authoritative USLM XML and parsing
// it into a live UsCode both run here; the main thread only renders the
// progress the worker posts. (wasm-bindgen Wasm-in-Worker pattern.)
//
// RPC protocol: main posts { id, type, args }; worker replies
//   { id, status: 'ok', payload }            — success
//   { id, status: 'err', payload }           — error message
//   { id, status: 'progress', phase, received, total }  — load progress
import init, { Pr4xis } from './pkg/pr4xis_wasm.js';

let pr4xis = null;

async function ensureReady() {
  if (!pr4xis) {
    await init();
    pr4xis = new Pr4xis();
  }
}

self.onmessage = async (e) => {
  const { id, type, args } = e.data;
  if (id === undefined) {
    // A malformed inbound message (no RPC id) can never be answered — log
    // loudly instead of posting an orphan reply no pending call can match.
    console.error('worker received message with no id:', e.data);
    return;
  }
  try {
    await ensureReady();
    switch (type) {
      case 'init':
        reply(id, {
          concept_count: pr4xis.concept_count(),
          word_count: pr4xis.word_count(),
        });
        break;
      case 'chat':
        reply(id, pr4xis.chat(args.input));
        break;
      case 'chat_batch': {
        // Stateless batch (the Smart-40 console / live-slice path). A JS loop
        // over the STATEFUL chat() is unsafe — a Conditional turn's pending
        // rule would consume the next question — so batching goes through the
        // dedicated stateless export. `chat_batch(questions: string[])` takes a
        // JS array and returns Presentation JSON `{"results":[…]}`, each row
        // identical in shape to a single chat() turn. Guarded so an older
        // cached wasm fails honestly instead of falling through at runtime.
        if (typeof pr4xis.chat_batch !== 'function') { fail(id, 'unsupported:chat_batch'); break; }
        const questions = Array.isArray(args.questions) ? args.questions : [];
        // Chunk so the UI gets determinate progress. chat_batch is stateless,
        // so calling it per sub-array is equivalent to one call over the whole.
        const CHUNK = 5;
        const out = [];
        for (let i = 0; i < questions.length; i += CHUNK) {
          const parsed = JSON.parse(pr4xis.chat_batch(questions.slice(i, i + CHUNK)));
          const rows = Array.isArray(parsed) ? parsed : (parsed.results || []);
          for (const r of rows) out.push(r);
          self.postMessage({ id, status: 'progress', phase: 'batch', completed: Math.min(i + CHUNK, questions.length), total: questions.length });
        }
        reply(id, JSON.stringify({ results: out }));
        break;
      }
      case 'reset_session':
        // Clear any pending Conditional slot-fill so a batch run — or a fresh
        // interactive turn — starts from a clean session. Guarded for older wasm.
        if (typeof pr4xis.reset_session !== 'function') { fail(id, 'unsupported:reset_session'); break; }
        pr4xis.reset_session();
        reply(id, null);
        break;
      case 'pipeline_ontologies':
        // The compile-time ontologies the pipeline reasons THROUGH — a
        // different population from the runtime-loaded vocabularies
        // `self_describe` reports. Guarded for an older wasm.
        if (typeof pr4xis.pipeline_ontologies !== 'function') { fail(id, 'unsupported:pipeline_ontologies'); break; }
        reply(id, pr4xis.pipeline_ontologies());
        break;
      case 'auto_load_budget':
        // The largest transfer the page may start unasked, computed by the
        // engine from the sizes it stages (a two-class natural break, Jenks
        // 1967) rather than declared in the page. Guarded for an older wasm;
        // the page treats an unsupported reply as "ask about everything",
        // which is the conservative reading.
        if (typeof pr4xis.auto_load_budget !== 'function') { fail(id, 'unsupported:auto_load_budget'); break; }
        reply(id, pr4xis.auto_load_budget());
        break;
      case 'eager_resident':
        // The names this deployment fetches at startup — the third residency
        // state beside the embedded manifest's resident and one-click entries.
        // Guarded for an older wasm; the page treats an unsupported reply as
        // "load nothing eagerly", which is the behaviour it shipped with.
        if (typeof pr4xis.eager_resident !== 'function') { fail(id, 'unsupported:eager_resident'); break; }
        reply(id, pr4xis.eager_resident());
        break;
      case 'unload':
        // The inverse of `load`: drop one runtime ontology by id and re-ground
        // what remains. Guarded for an older wasm that predates the export, so
        // a stale deploy degrades to "unsupported" rather than throwing.
        if (typeof pr4xis.unload !== 'function') { fail(id, 'unsupported:unload'); break; }
        reply(id, pr4xis.unload(args.id));
        break;
      case 'verify_palette': {
        // The engine audits the page's OWN rendered colour tokens against its
        // cited contrast axioms. Signature is TWO parallel arrays
        // `verify_palette(slot_keys: string[], hexes: string[])` — pass the
        // token object's keys and values (never a JSON string). Non-slot alias
        // keys / unparseable hexes are skipped engine-side. Guarded so an older
        // cached wasm replies 'unsupported' rather than falling through.
        if (typeof pr4xis.verify_palette !== 'function') { fail(id, 'unsupported:verify_palette'); break; }
        const vars = args.vars || {};
        reply(id, pr4xis.verify_palette(Object.keys(vars), Object.values(vars)));
        break;
      }
      case 'self_describe':
        reply(id, pr4xis.self_describe());
        break;
      case 'available_sources':
        reply(id, pr4xis.available_sources());
        break;
      case 'available_ontologies':
        reply(id, pr4xis.available_ontologies());
        break;
      case 'available_usc_archives':
        reply(id, pr4xis.available_usc_archives());
        break;
      case 'embedded_demo_prx':
        reply(id, pr4xis.embedded_demo_prx());
        break;
      case 'load': {
        // THE one load path (doc §3) — one message shape into the single typed
        // `Pr4xis::load(name, encoding, version, rootHex, payload)`. The worker
        // no longer branches per format: it streams the bytes if there is a URL
        // (a USLM title, an OWL `.owl` source, or an OWL `.prx.gz` all travel as
        // ONE Uint8Array — the Rust decoder the `encoding` names does the
        // format-specific parse) and passes `null` for an embedded load (the
        // demo bytes ship in the wasm; Rust resolves them by name). `version`
        // pins the OWL `.prx.gz` lock lookup; `rootHex` the content-addressed
        // `.prx` gate; both are `null` when the encoding does not use them.
        const { name, encoding, url, version, rootHex } = args;
        const payload = url ? await streamBinary(id, url) : null;
        const n = payload ? payload.length : 0;
        // The materialize/verify phase (parse, gunzip + bytecheck, or the
        // Merkle-root re-derivation) is the visible work after any download.
        self.postMessage({ id, status: 'progress', phase: 'parse', received: n, total: n });
        reply(id, pr4xis.load(name, encoding, version ?? null, rootHex ?? null, payload));
        break;
      }
      default:
        fail(id, `unknown message type: ${type}`);
    }
  } catch (err) {
    fail(id, String(err && err.message ? err.message : err));
  }
};

function reply(id, payload) {
  self.postMessage({ id, status: 'ok', payload });
}
function fail(id, payload) {
  self.postMessage({ id, status: 'err', payload });
}

// The SINGLE streaming download — reports `download` progress every chunk and
// returns the concatenated bytes for the one `load` path. USLM XML, OWL `.owl`
// source, and the OWL `.prx.gz` envelope all travel through here as one
// Uint8Array; the text-vs-binary split is gone because the Rust decoder the
// `encoding` names does the UTF-8 decode where a text format needs it.
async function streamBinary(id, url) {
  const resp = await fetch(url);
  if (!resp.ok) throw new Error(`HTTP ${resp.status} fetching ${url}`);
  const total = parseInt(resp.headers.get('content-length') || '0', 10);
  const reader = resp.body.getReader();
  const chunks = [];
  let received = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    chunks.push(value);
    received += value.length;
    self.postMessage({ id, status: 'progress', phase: 'download', received, total });
  }
  const buf = new Uint8Array(received);
  let off = 0;
  for (const c of chunks) {
    buf.set(c, off);
    off += c.length;
  }
  return buf;
}
