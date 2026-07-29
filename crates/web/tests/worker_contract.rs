//! Worker RPC contract: every `call('TYPE')` site in the chat UI must
//! have a matching `case 'TYPE':` branch in the worker, and vice versa.
//!
//! This is the regression coverage for the `available_ontologies` bug —
//! the wasm exposed the method, the UI called it, but the worker's
//! switch had no case so the worker fell through to its
//! `unknown message type` default at runtime.
//!
//! The one worker (`docs/worker.js`) serves ONE single-page app,
//! `docs/chat/index.html`, which boots ONE `Pr4xis` instance per page load
//! from its ONE `createEngine` call. That app carries the generic chat +
//! self-model/source-loader tool (the Chat and Engine tabs) and the ACL
//! Caregiver AI Challenge judge-facing surface (the Caregiver tab — a
//! persistent sidebar over Overview / Track 1 Case / Track 2 Case / Ask /
//! Evidence Lab / Method, hash-routed under `#caregiver`). The shell's
//! inline module owns that single boot and hands the SAME engine object to
//! the caregiver controller (`docs/chat/dashboard.js`) via
//! `mountCaregiver(engine)`; both reach the worker only through the shared
//! renderer (`docs/chat/chat-ui.js`, `createEngine`/`sendChat`/…). A worker
//! case is "called" if ANY worker-consumer UI file calls it, so the
//! contract reads all of them and unions their `call('TYPE')` sites —
//! otherwise a caregiver-tab-only RPC (e.g. the Smart-40 console's
//! `chat_batch`) would read as an orphan case.
//!
//! The contract reads the source files directly (no browser, no
//! wasm-bindgen-test) so the gate runs as a vanilla `cargo test`
//! integration test in every CI job.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("workspace root must be reachable from crates/web/")
}

/// The UI source files that invoke the worker via `call('TYPE')` — the ONE
/// app (`index.html`, whose inline module owns the app's single
/// `createEngine`/boot call), the caregiver tab's controller
/// (`dashboard.js`, which boots nothing and receives the shared engine via
/// `mountCaregiver`), and the shared renderer both import (`chat-ui.js`). A
/// file that is not present yet contributes nothing; the contract unions
/// the call sites of whichever files exist.
const WORKER_CONSUMER_UI_FILES: [&str; 3] = [
    "docs/chat/index.html",
    "docs/chat/dashboard.js",
    "docs/chat/chat-ui.js",
];

/// Every literal `call('TYPE')` RPC type invoked across all worker-consumer UI
/// files — the union, since the single worker serves every page.
fn ui_call_types(root: &Path) -> HashSet<String> {
    let mut out = HashSet::new();
    for rel in WORKER_CONSUMER_UI_FILES {
        if let Ok(src) = fs::read_to_string(root.join(rel)) {
            out.extend(literal_call_types(&src));
        }
    }
    out
}

#[test]
fn worker_routes_every_rpc_type_used_by_chat_ui() {
    let root = workspace_root();
    let worker =
        fs::read_to_string(root.join("docs/worker.js")).expect("docs/worker.js must exist");

    let ui_types = ui_call_types(&root);
    let worker_cases = case_clauses(&worker);

    let missing: Vec<&String> = ui_types.difference(&worker_cases).collect();
    assert!(
        missing.is_empty(),
        "docs/worker.js is missing `case 'TYPE':` branches for these RPC \
         types invoked from a worker-consumer UI file ({WORKER_CONSUMER_UI_FILES:?}): \
         {missing:?}.\n\n\
         Add a matching `case 'TYPE': ... break;` clause in the worker's \
         `switch (type)` block so the call doesn't fall through to the \
         `unknown message type` default at runtime."
    );
}

#[test]
fn every_worker_case_is_actually_called_by_the_chat_ui() {
    let root = workspace_root();
    let worker =
        fs::read_to_string(root.join("docs/worker.js")).expect("docs/worker.js must exist");

    let ui_types = ui_call_types(&root);
    let worker_cases = case_clauses(&worker);

    // Two RPC types are dispatched via a variable (`const rpc = kind === 'prx'
    // ? 'load_prx' : 'load_owl_source';` in docs/chat/index.html), so they
    // don't appear as `call('load_prx')` literals in the UI source. Whitelist
    // them so the orphan check doesn't false-positive.
    let dynamic_dispatch: HashSet<String> = ["load_prx", "load_owl_source"]
        .iter()
        .map(|s| (*s).into())
        .collect();

    let orphans: Vec<&String> = worker_cases
        .difference(&ui_types)
        .filter(|c| !dynamic_dispatch.contains(*c))
        .collect();
    assert!(
        orphans.is_empty(),
        "docs/worker.js has `case 'TYPE':` branches with no caller in any \
         worker-consumer UI file ({WORKER_CONSUMER_UI_FILES:?}): {orphans:?}. \
         Either a page lost a call site or the case is dead — wire a \
         `call('TYPE', …)` in the page that uses it, or delete the case."
    );
}

/// Extract every `case 'NAME':` (or `"NAME"`) clause from a JavaScript
/// file. Token-scans rather than per-line so multiple clauses on one
/// line (e.g. minified or single-line `switch`) are picked up too.
fn case_clauses(src: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    let bytes = src.as_bytes();
    let needle = b"case ";
    let mut i = 0;
    while i + needle.len() < bytes.len() {
        if &bytes[i..i + needle.len()] != needle {
            i += 1;
            continue;
        }
        // `case ` must follow a non-identifier byte to avoid matching
        // identifiers like `staircase`.
        if i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_') {
            i += 1;
            continue;
        }
        i += needle.len();
        if i >= bytes.len() {
            break;
        }
        let quote = bytes[i];
        if quote != b'\'' && quote != b'"' {
            continue;
        }
        i += 1;
        let name_start = i;
        while i < bytes.len() && bytes[i] != quote {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        if let Ok(name) = std::str::from_utf8(&bytes[name_start..i])
            && !name.is_empty()
        {
            out.insert(name.to_string());
        }
        i += 1;
    }
    out
}

/// Extract every literal `call('NAME', ...)` (or `"NAME"`) call type
/// from a JavaScript / HTML source. Variables (e.g. `call(rpc, ...)`)
/// are not extracted — the [`every_worker_case_is_actually_called_by_the_chat_ui`]
/// test handles dynamic dispatch via a whitelist.
fn literal_call_types(src: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    let bytes = src.as_bytes();
    let needle = b"call(";
    let mut i = 0;
    while i + needle.len() < bytes.len() {
        if &bytes[i..i + needle.len()] != needle {
            i += 1;
            continue;
        }
        // Skip the function definition: `function call(` — there is no
        // matching closing quote, so it gets filtered below by the
        // quote check, but the explicit skip keeps intent clear.
        let prefix_start = i.saturating_sub(9);
        let prefix = &bytes[prefix_start..i];
        let is_definition = prefix.ends_with(b"function ");
        i += needle.len();
        if is_definition {
            continue;
        }
        if i >= bytes.len() {
            break;
        }
        let quote = bytes[i];
        if quote != b'\'' && quote != b'"' {
            continue;
        }
        i += 1;
        let name_start = i;
        while i < bytes.len() && bytes[i] != quote {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let Ok(name) = std::str::from_utf8(&bytes[name_start..i]) else {
            i += 1;
            continue;
        };
        if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            out.insert(name.to_string());
        }
        i += 1;
    }
    out
}

#[test]
fn extractor_handles_single_and_double_quotes() {
    let src = "x = call('a'); y = call(\"b\"); call(rpc, args); function call(t) {}";
    let types = literal_call_types(src);
    assert!(types.contains("a"));
    assert!(types.contains("b"));
    assert!(!types.contains("rpc"));
    assert!(!types.contains("t"));
    assert_eq!(types.len(), 2);
}

#[test]
fn case_extractor_handles_quoted_string_cases() {
    let src = "switch (x) { case 'foo': break; case \"bar\": break; default: throw; }";
    let cases = case_clauses(src);
    assert!(cases.contains("foo"));
    assert!(cases.contains("bar"));
    assert_eq!(cases.len(), 2);
}
