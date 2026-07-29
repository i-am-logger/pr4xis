//! wasm export contract for the Caregiver Evaluation Bench.
//!
//! The bench's worker wires against a fixed set of `Pr4xis` exports and
//! wire-field names (the batch console, the reset control, the typed-trace
//! renderer). This gate reads `crates/wasm/src/lib.rs` directly — no wasm build,
//! no browser — and fails on drift, the SAME source-scanning discipline as the
//! sibling `worker_contract.rs`. A renamed or dropped export the bench depends on
//! then breaks a vanilla `cargo test`, not a judge's browser at demo time.
//!
//! It pins names and argument SHAPE (what the frontend marshals against), not
//! formatting: a reflow of a signature is fine; a changed parameter type, a
//! dropped export, or a renamed wire field is not.

use std::fs;
use std::path::PathBuf;

/// The wasm crate's `lib.rs` source, reached from `crates/web/` — the one file
/// the whole export surface lives in.
fn wasm_lib_src() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("wasm")
        .join("src")
        .join("lib.rs")
        .canonicalize()
        .expect("crates/wasm/src/lib.rs must be reachable from crates/web/");
    fs::read_to_string(path).expect("crates/wasm/src/lib.rs must be readable")
}

#[test]
fn wasm_exports_the_bench_batch_and_reset_entries() {
    let src = wasm_lib_src();

    // The whole `impl Pr4xis` is `#[wasm_bindgen]`, so any `pub fn` in it is a JS
    // export. The bench's Smart-40 console + reset control need these two.
    assert!(
        src.contains("pub fn chat_batch("),
        "crates/wasm/src/lib.rs must export `chat_batch` — the Smart-40 console \
         drives it to run the published protocol statelessly."
    );
    assert!(
        src.contains("questions: Vec<String>"),
        "`chat_batch` must accept a JS `string[]` (Vec<String>) — the worker \
         marshals an array of question strings against this shape."
    );
    assert!(
        src.contains("pub fn reset_session("),
        "crates/wasm/src/lib.rs must export `reset_session` — a host abandoning a \
         Conditional slot-fill dialogue clears the pending frame with it."
    );
    assert!(
        src.contains("pub fn verify_palette("),
        "crates/wasm/src/lib.rs must export `verify_palette` — the WCAG self-audit \
         panel calls it to check the page's live tokens against the theming axioms."
    );
    assert!(
        src.contains("slot_keys: Vec<String>") && src.contains("hexes: Vec<String>"),
        "`verify_palette` must accept two JS `string[]` (slot keys + hex values) — \
         the worker marshals `Object.keys`/`Object.values` of the token set."
    );
}

#[test]
fn verify_palette_emits_the_self_audit_wire_fields() {
    let src = wasm_lib_src();

    // The self-audit panel reads these fields verbatim per check row.
    for field in [
        r#""checks""#,
        r#""axiom""#,
        r#""citation""#,
        r#""pair""#,
        r#""ratio""#,
        r#""required""#,
        r#""polarity""#,
    ] {
        assert!(
            src.contains(field),
            "verify_palette must emit the `{field}` wire field the WCAG panel renders."
        );
    }
}

#[test]
fn chat_emits_the_typed_trace_beside_the_flattened_string() {
    let src = wasm_lib_src();

    // Additive, not breaking: the flattened `trace` string stays for the existing
    // string renderer; the typed `trace_structured` list is the new structured
    // surface the bench's trace accordion renders without parsing the string. Both
    // wire-field names must be emitted (the frontend feature-detects on
    // `trace_structured`, NOT on `trace` becoming an array).
    assert!(
        src.contains(r#""trace""#),
        "chat() must keep emitting the flattened `trace` string field (back-compat)."
    );
    assert!(
        src.contains(r#""trace_structured""#),
        "chat() must emit the typed `trace_structured` field beside `trace`."
    );
    // Each typed-trace step carries these fields the renderer reads verbatim.
    for field in [r#""phase""#, r#""functor_connections""#, r#""reference""#] {
        assert!(
            src.contains(field),
            "each typed-trace step must carry the `{field}` field."
        );
    }
}

#[test]
fn chat_batch_records_carry_the_bench_wire_fields() {
    let src = wasm_lib_src();

    // The batch envelope + the per-question `question` echo. Field-name literals,
    // not call syntax — `chat` and `chat_batch` share one builder, so the receiver
    // variable is an implementation detail, but the wire field NAMES are contract.
    for field in [r#""results""#, r#""question""#, r#""response""#] {
        assert!(
            src.contains(field),
            "`chat_batch` must emit the `{field}` wire field the console reads."
        );
    }

    // A batch row is the FULL `chat` envelope, so both share ONE builder, and the
    // per-question outcome fields come from the SHARED `write_outcome` lowering —
    // a batch row and a single `chat` turn are byte-identical in shape.
    assert!(
        src.contains("fn build_chat_presentation("),
        "`chat` and `chat_batch` must share the full-envelope builder so a batch \
         row is identical to a single answer."
    );
    assert!(
        src.contains("fn write_outcome("),
        "the shared outcome lowering `write_outcome` must exist."
    );
    for field in [
        r#""outcome""#,
        r#""unresolved""#,
        r#""rule_name""#,
        r#""ontologies""#,
    ] {
        assert!(
            src.contains(field),
            "the shared full-envelope projection must emit the `{field}` field."
        );
    }
}

/// The DEFINITION-PROVENANCE record crosses the wire as its own field, carrying
/// its own engine-realized label.
///
/// It is deliberately NOT folded into `why`: `why` names the ontologies the turn
/// OPENED, this names the document a recited gloss was WRITTEN FROM, and a page
/// that had to split the `why` sentence to tell them apart would be doing engine
/// work in JavaScript. Shipping the label too is what lets the renderer carry no
/// wording of its own for a channel the engine alone knows exists.
#[test]
fn chat_emits_the_definition_provenance_record_with_its_own_label() {
    let src = wasm_lib_src();

    assert!(
        src.contains(r#""definition_provenance""#),
        "chat() must emit the `definition_provenance` record beside `why`."
    );
    for field in [r#""label""#, r#""detail""#] {
        assert!(
            src.contains(field),
            "the definition-provenance record must carry the `{field}` field the \
             renderer displays verbatim."
        );
    }
}
