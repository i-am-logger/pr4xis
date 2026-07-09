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
