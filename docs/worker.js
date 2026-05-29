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
      case 'load':
        await loadSource(id, args.name, args.url, args.totalHint);
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
