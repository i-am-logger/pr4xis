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
      case 'self_describe':
        reply(id, pr4xis.self_describe());
        break;
      case 'available_sources':
        reply(id, pr4xis.available_sources());
        break;
      case 'available_ontologies':
        reply(id, pr4xis.available_ontologies());
        break;
      case 'embedded_demo_prx':
        reply(id, pr4xis.embedded_demo_prx());
        break;
      case 'load_embedded_demo_prx':
        // Load the new-format `.prx` embedded in the wasm — no network. The
        // wasm gate re-derives the archive's Merkle root and refuses on
        // mismatch (fail-closed); the chat then answers from the loaded gloss.
        reply(id, pr4xis.load_embedded_demo_prx());
        break;
      case 'load':
        await loadSource(id, args.name, args.url, args.totalHint);
        reply(id, null);
        break;
      case 'load_prx':
        await loadPrx(id, args.name, args.version, args.url);
        reply(id, null);
        break;
      case 'load_owl_source':
        await loadOwlSource(id, args.name, args.url);
        reply(id, null);
        break;
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

// Download (streaming, with progress) + parse into a live UsCode. Both run
// in the worker, so the main thread stays responsive throughout — even for
// a large title.
async function loadSource(id, name, url, totalHint) {
  const resp = await fetch(url);
  if (!resp.ok) throw new Error(`HTTP ${resp.status} fetching ${url}`);
  const total = parseInt(resp.headers.get('content-length') || totalHint || '0', 10);

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
  const xml = new TextDecoder('utf-8').decode(buf);

  // Parse phase runs here, in the worker — no main-thread freeze.
  self.postMessage({ id, status: 'progress', phase: 'parse', received, total });
  pr4xis.load_source(name, xml); // → live UsCode (throws on malformed XML)
}

// Stream the `.prx.gz` distribution envelope (binary), then hand the raw
// bytes to `load_prx`. The wasm gate gunzips, bytecheck-validates the rkyv
// envelope, and asserts the embedded source-hash matches the praxis.lock
// pin baked into the build — fail-closed (throws JsValue on any mismatch).
async function loadPrx(id, name, version, url) {
  const buf = await streamBinary(id, url);
  // Switch to the materialisation phase BEFORE the gate runs (gunzip +
  // bytecheck + rkyv access are the visible work).
  self.postMessage({ id, status: 'progress', phase: 'parse', received: buf.length, total: buf.length });
  pr4xis.load_prx(name, version, buf); // throws JsValue on validate fail
}

// Stream the authoritative `.owl` source (text) and parse via the pure-Rust
// OWL reader. No embedded hash on this leg — trust rests on the host having
// fetched from the pinned source URL.
async function loadOwlSource(id, name, url) {
  const buf = await streamBinary(id, url);
  self.postMessage({ id, status: 'progress', phase: 'parse', received: buf.length, total: buf.length });
  const xml = new TextDecoder('utf-8').decode(buf);
  pr4xis.load_owl_source(name, xml); // throws JsValue on malformed OWL
}

// Shared streaming download — reports `download` progress every chunk and
// returns the concatenated bytes. Identical to the body of `loadSource`'s
// download leg, factored out so the two ontology loaders post the same
// progress messages the meta page already knows how to render.
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
