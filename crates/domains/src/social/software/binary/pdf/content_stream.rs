//! Content-stream interpreter — operator sequence → typed events.
//!
//! A PDF page's content stream (ISO 32000-2:2020 §7.8.2) is a
//! sequence of PostScript-style operators that paint the page. For
//! text extraction the relevant operators are:
//!
//! - **Text object boundaries** (§9.4.1) — `BT` / `ET`.
//! - **Text state** (§9.3) — `Tf` (font), `Tc`, `Tw`, `TL`, `Tz`,
//!   `Tr`, `Ts`. Only `Tf` is needed to identify which font's
//!   encoding to use for the bytes that follow.
//! - **Text positioning** (§9.4.2) — `Td`, `TD`, `Tm`, `T*`. We
//!   ignore positioning for text *content* extraction; the praxis
//!   rule is text-only, not layout-faithful.
//! - **Text showing** (§9.4.3) — `Tj`, `TJ`, `'`, `"`. These carry
//!   the glyph-code bytes that Phase 4's font decoder will map to
//!   Unicode.
//!
//! Non-text operators we recognize and flag (per
//! `feedback_pdf_text_only_until_image_understanding`):
//!
//! - **Inline images** (§8.9.7) — `BI` … `EI`.
//! - **XObject invocation** (§8.8) — `Do` — may be an image XObject
//!   (flag as image) or a form XObject (flag as form; its inner
//!   content stream isn't recursively walked in this phase).
//! - **Vector path painting** (§8.5) — `S`, `s`, `f`, `F`, `f*`,
//!   `B`, `B*`, `b`, `b*`, `n`. These are the operators that
//!   actually paint vector ink; bare `m`/`l`/`c`/`re` etc. just
//!   build the current path without painting and aren't flagged
//!   by themselves.
//!
//! All other operators (graphics state, color, clipping, marked
//! content) are silently consumed — they don't carry text and they
//! don't paint flag-worthy content on their own.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::FlaggedKind;

// ─────────────────────────────────────────────────────────────────────
// Output types
// ─────────────────────────────────────────────────────────────────────

/// One text-showing event from a content stream — the raw glyph-code
/// bytes together with the font name that's in scope.
///
/// Phase 4 takes this and produces decoded `String` chunks by
/// resolving the font name against the page's `/Resources /Font`
/// dictionary and applying the font's `/ToUnicode` CMap or
/// `/Encoding` entry.
///
/// `Eq` is not derived because `font_size` is `f32` (PDF user
/// units are real-valued per §8.3.2.3); bit-exact equality on
/// font sizes is not semantically meaningful.
#[derive(Debug, Clone, PartialEq)]
pub struct TextShowEvent {
    /// PDF name of the font in effect (the operand to the most
    /// recent `Tf` operator). Resolves to a font resource via
    /// `Resources /Font /<name>`.
    pub font_name: String,
    /// Font size in PDF user units, as supplied to `Tf`. Carried
    /// through for completeness; not used by Phase 4 byte→Unicode
    /// mapping but useful for downstream layout reasoning.
    pub font_size: f32,
    /// Raw bytes from the text-showing operator's string operand.
    /// For `TJ` (array form), the individual string segments are
    /// concatenated in order; numeric (kerning) entries are dropped.
    pub bytes: Vec<u8>,
}

/// Non-text content the interpreter encountered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphicsEvent {
    pub kind: FlaggedKind,
    /// Operator that triggered the flag (`Do`, `BI`, `S`, `f`, …)
    /// — useful for diagnostics.
    pub operator: String,
    /// Operand summary for the operator (e.g. the XObject name for
    /// `Do`). Empty for operators with no name operand.
    pub detail: String,
}

/// Result of walking a single content stream.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ContentStreamWalk {
    pub text_events: Vec<TextShowEvent>,
    pub graphics_events: Vec<GraphicsEvent>,
}

// ─────────────────────────────────────────────────────────────────────
// Errors — every parse failure named.
// ─────────────────────────────────────────────────────────────────────

/// Why a content stream couldn't be walked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentStreamError {
    /// The bytes didn't decode as a PDF content stream — lopdf's
    /// operator parser failed.
    Malformed { detail: String },
}

impl core::fmt::Display for ContentStreamError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Malformed { detail } => write!(f, "malformed content stream: {detail}"),
        }
    }
}

impl std::error::Error for ContentStreamError {}

// ─────────────────────────────────────────────────────────────────────
// Walk
// ─────────────────────────────────────────────────────────────────────

/// Walk a content stream byte sequence and collect text + graphics
/// events. Per ISO 32000-2:2020 §7.8.2 the bytes are a PostScript-
/// style operator stream; `lopdf::content::Content::decode` does the
/// tokenization, leaving us to interpret the operator names.
pub fn walk_content_stream(bytes: &[u8]) -> Result<ContentStreamWalk, ContentStreamError> {
    let content =
        lopdf::content::Content::decode(bytes).map_err(|e| ContentStreamError::Malformed {
            detail: format!("{e}"),
        })?;

    let mut state = TextState::default();
    let mut out = ContentStreamWalk::default();

    for op in &content.operations {
        match op.operator.as_str() {
            // ─── Text object boundaries (§9.4.1) ───
            "BT" => state.in_text_object = true,
            "ET" => state.in_text_object = false,

            // ─── Text state — only Tf is relevant (§9.3) ───
            "Tf" => {
                // operand[0] = font name (Name); operand[1] = size (Number)
                if op.operands.len() >= 2 {
                    if let Some(name) = operand_name(&op.operands[0]) {
                        state.current_font = Some(name);
                    }
                    state.current_size = operand_f32(&op.operands[1]).unwrap_or(0.0);
                }
            }

            // ─── Text-showing operators (§9.4.3) ───
            "Tj" => {
                // Tj: <str> Tj — show string
                if let Some(bytes) = op.operands.first().and_then(operand_string_bytes) {
                    push_text_event(&mut out, &state, bytes);
                }
            }
            "'" => {
                // ': <str> ' — move to next line and show string.
                // Same payload shape as Tj for our purposes.
                if let Some(bytes) = op.operands.first().and_then(operand_string_bytes) {
                    push_text_event(&mut out, &state, bytes);
                }
            }
            "\"" => {
                // ": <aw> <ac> <str> " — set word/char spacing,
                // move to next line, and show string. The string is
                // the third operand.
                if let Some(bytes) = op.operands.get(2).and_then(operand_string_bytes) {
                    push_text_event(&mut out, &state, bytes);
                }
            }
            "TJ" => {
                // TJ: <array> TJ — show array of strings + numbers.
                // We concatenate the string segments and drop numeric
                // kerning entries.
                if let Some(lopdf::Object::Array(items)) = op.operands.first() {
                    let mut chunk: Vec<u8> = Vec::new();
                    for item in items {
                        if let Some(bytes) = operand_string_bytes(item) {
                            chunk.extend_from_slice(&bytes);
                        }
                        // Numeric items (Integer / Real) are kerning
                        // adjustments — skip.
                    }
                    if !chunk.is_empty() {
                        push_text_event(&mut out, &state, chunk);
                    }
                }
            }

            // ─── XObject invocation (§8.8) ───
            "Do" => {
                let name = op
                    .operands
                    .first()
                    .and_then(operand_name)
                    .unwrap_or_default();
                // We can't tell from the operator alone whether the
                // XObject is an image or a form — flag as Form here;
                // Phase 5's flagged-content walker resolves the
                // /Subtype against page resources and reclassifies.
                out.graphics_events.push(GraphicsEvent {
                    kind: FlaggedKind::FormXObject,
                    operator: "Do".to_string(),
                    detail: name,
                });
            }

            // ─── Inline images (§8.9.7) ───
            // lopdf surfaces inline images as a single operator
            // "BI...ID...EI"; the operator string varies. Match on
            // anything starting with "BI" to catch the family.
            op_name if op_name == "BI" || op_name == "EI" || op_name.starts_with("BI") => {
                out.graphics_events.push(GraphicsEvent {
                    kind: FlaggedKind::InlineImage,
                    operator: op.operator.clone(),
                    detail: String::new(),
                });
            }

            // ─── Vector path painting (§8.5.3) ───
            // The operators that actually paint (stroke/fill); bare
            // m/l/c/re/v/y/h build the current path without painting.
            "S" | "s" | "f" | "F" | "f*" | "B" | "B*" | "b" | "b*" => {
                out.graphics_events.push(GraphicsEvent {
                    kind: FlaggedKind::VectorPath,
                    operator: op.operator.clone(),
                    detail: String::new(),
                });
            }

            // Everything else — graphics state, color, clipping,
            // marked content, path construction — is consumed
            // silently. Not text, not flag-worthy on its own.
            _ => {}
        }
    }

    Ok(out)
}

// ─────────────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────────────

/// Mutable interpreter state — only what we actually need for text
/// extraction (the font in scope). PDF's full text-state machine
/// also tracks the text matrix, leading, character spacing, etc.;
/// those affect *layout* but not text content.
#[derive(Debug, Default)]
struct TextState {
    in_text_object: bool,
    current_font: Option<String>,
    current_size: f32,
}

fn push_text_event(out: &mut ContentStreamWalk, state: &TextState, bytes: Vec<u8>) {
    // Per §9.4.3, text-showing operators are valid only inside a
    // BT/ET text object. lopdf may surface them outside (malformed
    // streams) — we still capture them but record the font that's
    // in scope, which may be None.
    let font_name = state.current_font.clone().unwrap_or_default();
    out.text_events.push(TextShowEvent {
        font_name,
        font_size: state.current_size,
        bytes,
    });
    // Silence the unused-field warning while in_text_object is
    // tracked for future BT/ET correctness axioms.
    let _ = state.in_text_object;
}

fn operand_name(o: &lopdf::Object) -> Option<String> {
    match o {
        lopdf::Object::Name(bytes) => Some(String::from_utf8_lossy(bytes).into_owned()),
        _ => None,
    }
}

fn operand_f32(o: &lopdf::Object) -> Option<f32> {
    match o {
        lopdf::Object::Integer(i) => Some(*i as f32),
        lopdf::Object::Real(r) => Some(*r),
        _ => None,
    }
}

fn operand_string_bytes(o: &lopdf::Object) -> Option<Vec<u8>> {
    match o {
        lopdf::Object::String(bytes, _format) => Some(bytes.clone()),
        _ => None,
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal content stream as bytes. Each line is one
    /// PDF operator; we use ASCII-only forms so the parser sees
    /// clean tokens.
    fn stream(s: &str) -> Vec<u8> {
        s.as_bytes().to_vec()
    }

    #[test]
    fn empty_stream_yields_empty_walk() {
        let w = walk_content_stream(b"").expect("empty stream is valid");
        assert!(w.text_events.is_empty());
        assert!(w.graphics_events.is_empty());
    }

    #[test]
    fn single_tj_captures_text_event() {
        let bytes = stream("BT\n/F1 12 Tf\n(Hello world) Tj\nET\n");
        let w = walk_content_stream(&bytes).expect("parse");
        assert_eq!(w.text_events.len(), 1);
        assert_eq!(w.text_events[0].font_name, "F1");
        assert_eq!(w.text_events[0].font_size, 12.0);
        assert_eq!(w.text_events[0].bytes, b"Hello world");
    }

    #[test]
    fn tj_outside_text_object_still_captured() {
        // PDF spec calls this malformed but lopdf surfaces it; we
        // capture so downstream auditors see the malformation
        // rather than silently dropping content.
        let bytes = stream("/F1 10 Tf\n(loose text) Tj\n");
        let w = walk_content_stream(&bytes).expect("parse");
        assert_eq!(w.text_events.len(), 1);
        assert_eq!(w.text_events[0].bytes, b"loose text");
    }

    #[test]
    fn tj_array_concatenates_string_segments_dropping_numbers() {
        // TJ array: ["Hello" -100 " " "world"]
        // — minus the number which is kerning.
        let bytes = stream("BT\n/F1 12 Tf\n[(Hello) -100 ( ) (world)] TJ\nET\n");
        let w = walk_content_stream(&bytes).expect("parse");
        assert_eq!(w.text_events.len(), 1);
        assert_eq!(w.text_events[0].bytes, b"Hello world");
    }

    #[test]
    fn quote_apostrophe_operator_treated_like_tj() {
        let bytes = stream("BT\n/F2 10 Tf\n(next line) '\nET\n");
        let w = walk_content_stream(&bytes).expect("parse");
        assert_eq!(w.text_events.len(), 1);
        assert_eq!(w.text_events[0].font_name, "F2");
        assert_eq!(w.text_events[0].bytes, b"next line");
    }

    #[test]
    fn double_quote_takes_third_operand_as_string() {
        // ": aw ac str  — third operand is the string."
        let bytes = stream("BT\n/F3 14 Tf\n0 0 (with spacing) \"\nET\n");
        let w = walk_content_stream(&bytes).expect("parse");
        assert_eq!(w.text_events.len(), 1);
        assert_eq!(w.text_events[0].bytes, b"with spacing");
    }

    #[test]
    fn tf_changes_current_font() {
        let bytes = stream("BT\n/F1 12 Tf\n(first) Tj\n/F2 14 Tf\n(second) Tj\nET\n");
        let w = walk_content_stream(&bytes).expect("parse");
        assert_eq!(w.text_events.len(), 2);
        assert_eq!(w.text_events[0].font_name, "F1");
        assert_eq!(w.text_events[0].font_size, 12.0);
        assert_eq!(w.text_events[1].font_name, "F2");
        assert_eq!(w.text_events[1].font_size, 14.0);
    }

    #[test]
    fn do_operator_records_form_xobject_event() {
        let bytes = stream("/Im0 Do\n");
        let w = walk_content_stream(&bytes).expect("parse");
        assert_eq!(w.graphics_events.len(), 1);
        assert_eq!(w.graphics_events[0].kind, FlaggedKind::FormXObject);
        assert_eq!(w.graphics_events[0].operator, "Do");
        assert_eq!(w.graphics_events[0].detail, "Im0");
    }

    #[test]
    fn paint_operators_record_vector_path_events() {
        // Build a small path then paint it: rectangle then fill.
        let bytes = stream("100 100 200 50 re\nf\n");
        let w = walk_content_stream(&bytes).expect("parse");
        // The `re` operator just constructs the path — not flagged.
        // The `f` operator paints it — flagged.
        assert_eq!(w.graphics_events.len(), 1);
        assert_eq!(w.graphics_events[0].kind, FlaggedKind::VectorPath);
        assert_eq!(w.graphics_events[0].operator, "f");
    }

    #[test]
    fn stroke_and_fill_both_recorded() {
        let bytes = stream("100 100 m\n200 200 l\nS\n50 50 60 60 re\nf\n");
        let w = walk_content_stream(&bytes).expect("parse");
        assert_eq!(w.graphics_events.len(), 2);
        assert_eq!(w.graphics_events[0].operator, "S");
        assert_eq!(w.graphics_events[1].operator, "f");
    }

    #[test]
    fn text_and_graphics_separately_collected() {
        let bytes = stream("BT\n/F1 12 Tf\n(label) Tj\nET\n100 100 50 50 re\nf\n/Im0 Do\n");
        let w = walk_content_stream(&bytes).expect("parse");
        assert_eq!(w.text_events.len(), 1);
        assert_eq!(w.text_events[0].bytes, b"label");
        assert_eq!(w.graphics_events.len(), 2);
        // VectorPath from the `f`, FormXObject from the `Do`.
        let kinds: Vec<_> = w.graphics_events.iter().map(|e| e.kind).collect();
        assert!(kinds.contains(&FlaggedKind::VectorPath));
        assert!(kinds.contains(&FlaggedKind::FormXObject));
    }

    #[test]
    fn malformed_stream_handled_without_corruption() {
        // Garbage bytes. lopdf is permissive — it may accept this
        // as an empty parse, or reject it. Either outcome is fine;
        // we just verify we don't silently corrupt or panic.
        let bytes = stream("\x00\x01\x02 not a content stream ###");
        match walk_content_stream(&bytes) {
            Ok(_) | Err(ContentStreamError::Malformed { .. }) => {}
        }
    }

    #[test]
    fn walk_is_deterministic_on_same_input() {
        let bytes = stream("BT\n/F1 12 Tf\n[(Hello) -50 (world)] TJ\nET\n50 50 100 100 re\nf\n");
        let w1 = walk_content_stream(&bytes).expect("parse 1");
        let w2 = walk_content_stream(&bytes).expect("parse 2");
        assert_eq!(w1, w2);
    }

    // ── Property-based ────────────────────────────────────────────

    use proptest::prelude::*;

    /// Random-ish text inside a Tj string operand. We restrict to
    /// printable ASCII excluding the PDF literal-string escape
    /// characters `()` and `\` so the generated stream is always
    /// well-formed without us having to do escape handling.
    fn arb_printable_text() -> impl Strategy<Value = String> {
        proptest::collection::vec(any::<char>(), 0..40).prop_map(|chars| {
            chars
                .into_iter()
                .filter(|c| c.is_ascii() && !matches!(*c, '(' | ')' | '\\' | '\r' | '\n' | '\0'))
                .collect()
        })
    }

    proptest! {
        /// walk_content_stream is deterministic: same input bytes
        /// produce the same ContentStreamWalk every time.
        #[test]
        fn prop_walk_is_deterministic(text in arb_printable_text()) {
            let bytes = stream(&format!("BT\n/F1 12 Tf\n({text}) Tj\nET\n"));
            let w1 = walk_content_stream(&bytes).expect("parse 1");
            let w2 = walk_content_stream(&bytes).expect("parse 2");
            prop_assert_eq!(w1, w2);
        }

        /// A single Tj with arbitrary printable payload produces
        /// exactly one TextShowEvent whose bytes equal the payload.
        #[test]
        fn prop_tj_round_trips_payload_bytes(text in arb_printable_text()) {
            let bytes = stream(&format!("BT\n/F1 12 Tf\n({text}) Tj\nET\n"));
            let w = walk_content_stream(&bytes).expect("parse");
            prop_assert_eq!(w.text_events.len(), 1);
            prop_assert_eq!(&w.text_events[0].bytes, &text.as_bytes());
        }

        /// N copies of `Tj` inside one BT/ET produce exactly N
        /// text events — no events are silently merged or dropped.
        #[test]
        fn prop_n_tj_calls_yield_n_text_events(n in 0u32..16) {
            let mut s = String::from("BT\n/F1 12 Tf\n");
            for i in 0..n {
                s.push_str(&format!("(chunk{i}) Tj\n"));
            }
            s.push_str("ET\n");
            let w = walk_content_stream(s.as_bytes()).expect("parse");
            prop_assert_eq!(w.text_events.len() as u32, n);
        }

        /// N painting operators (rectangle + fill, repeated)
        /// produce exactly N graphics events with kind
        /// VectorPath. Crosses Phase 3's own invariants for
        /// content-vs-painting operator separation.
        #[test]
        fn prop_n_fills_yield_n_vector_path_events(n in 0u32..16) {
            let mut s = String::new();
            for i in 0..n {
                s.push_str(&format!("{} 0 10 10 re\nf\n", i * 20));
            }
            let w = walk_content_stream(s.as_bytes()).expect("parse");
            prop_assert_eq!(w.graphics_events.len() as u32, n);
            for e in &w.graphics_events {
                prop_assert_eq!(e.kind, FlaggedKind::VectorPath);
            }
        }

        /// Random bytes never panic the walker — they either
        /// parse to some (possibly empty) walk or return a named
        /// `Malformed` error. No silent corruption, no crashes.
        #[test]
        fn prop_random_bytes_never_panic(
            bytes in proptest::collection::vec(any::<u8>(), 0..256),
        ) {
            match walk_content_stream(&bytes) {
                Ok(_) | Err(ContentStreamError::Malformed { .. }) => { /* OK */ }
            }
        }

        /// `Tf <name> <size>` updates the current font in scope:
        /// after a `Tf`, any following Tj reports the new font on
        /// its TextShowEvent.
        #[test]
        fn prop_tf_changes_current_font_for_following_tj(
            font_id in 0u32..32,
            size in 1u32..72,
        ) {
            let s = format!(
                "BT\n/F{font_id} {size} Tf\n(text) Tj\nET\n",
            );
            let w = walk_content_stream(s.as_bytes()).expect("parse");
            prop_assert_eq!(w.text_events.len(), 1);
            prop_assert_eq!(&w.text_events[0].font_name, &format!("F{font_id}"));
            prop_assert_eq!(w.text_events[0].font_size, size as f32);
        }
    }
}
