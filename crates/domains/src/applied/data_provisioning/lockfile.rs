//! Lockfile writer — rewrites entries in `praxis.lock` while preserving
//! comments and key ordering.
//!
//! The default `toml` crate's serialize path discards comments and
//! reorders keys to its own canonical form, which loses the per-source
//! provenance commentary above each `[hashes]` line (citations,
//! release-point references, lens-versus-canonical notes). The repo's
//! `praxis.lock` carries that commentary by convention — it is
//! editor-visible documentation, not just data. The writer therefore
//! operates on the text representation directly: it walks the file
//! line-by-line, locates the `"<key>" = "<digest>"` line for the requested
//! key, and replaces just the digest value while leaving every other byte
//! (comments, ordering, whitespace) untouched. New keys are appended at
//! the end of the `[hashes]` section, before the next `[section]` header
//! if any, otherwise at end-of-file.
//!
//! Written values use the tagged digest grammar the parser
//! ([`super::registry::LockDigest`]) loads: praxis-emitted pins are written
//! `blake3:<64 lowercase hex>` — the one emit algorithm per format epoch
//! ([`pr4xis_runtime::address::ADDRESS_ALGORITHM`], BLAKE3 — Aumasson,
//! O'Connor, Neves & Wilcox-O'Hearn 2020).
//!
//! Citation: Tom Preston-Werner et al. (2024) *TOML: Tom's Obvious
//! Minimal Language* v1.0.0, §2 (comments are syntactically significant
//! to humans). RFC 1952 / 5234 ABNF-style "leave bytes alone unless they
//! match the production" rewriting principle. Dolstra (2006) §5.1
//! (content-addressed pinning).

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

use pr4xis_runtime::address::ADDRESS_ALGORITHM;

use super::registry::algorithm_tag;

/// Errors returned by the `set_*` writers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockfileWriteError {
    /// The lockfile text has no `[hashes]` section. A praxis.lock without
    /// `[hashes]` is malformed — refuse to write blind.
    MissingHashesSection,
    /// The provided digest is not 64 lowercase hex characters (the hex
    /// form of one 256-bit digest). Mirrors the parse-side shape check
    /// (`registry::LockDigest::parse`) so we never write a value the
    /// loader would reject.
    InvalidDigest(String),
    /// The lockfile text has no section by the requested name (e.g.
    /// `[archive_signatures]`). Same fail-closed stance as
    /// [`Self::MissingHashesSection`] — refuse to write blind into a
    /// lockfile missing the section the value belongs in.
    MissingSection(String),
}

impl core::fmt::Display for LockfileWriteError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LockfileWriteError::MissingHashesSection => {
                write!(f, "praxis.lock has no `[hashes]` section")
            }
            LockfileWriteError::InvalidDigest(s) => {
                write!(f, "not a 64-char lowercase hex digest: {s:?}")
            }
            LockfileWriteError::MissingSection(name) => {
                write!(f, "praxis.lock has no `[{name}]` section")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for LockfileWriteError {}

/// Set the content digest for `key` (a `"name@version"` string, without
/// quotes) to `digest_hex` in the `[hashes]` section of `lockfile_text`,
/// returning the rewritten text. The value is written in the tagged
/// emit form (`blake3:<digest_hex>`).
///
/// - If `key` already has a line in `[hashes]`, only the value is
///   replaced; the line's comments, whitespace, and surrounding lines
///   are preserved byte-for-byte.
/// - If `key` is absent, a new `"key" = "blake3:digest"` line is appended
///   at the end of the `[hashes]` section (immediately before the next
///   `[section]` header, or at end-of-file if `[hashes]` is the last).
/// - If the lockfile has no `[hashes]` section, returns
///   [`LockfileWriteError::MissingHashesSection`] — writing blind to a
///   malformed lockfile would silently corrupt it.
///
/// `digest_hex` must be 64 lowercase hex characters (the emit-leg
/// `ContentAddress::to_hex()` form). Mixed-case or off-length inputs are
/// rejected with [`LockfileWriteError::InvalidDigest`].
pub fn set_hash(
    lockfile_text: &str,
    key: &str,
    digest_hex: &str,
) -> Result<String, LockfileWriteError> {
    // Preserve `set_hash`'s specific error contract: a `[hashes]`-less
    // lockfile reports the specific `MissingHashesSection`, not the
    // generic `MissingSection`.
    set_in_section(lockfile_text, "hashes", key, digest_hex).map_err(|e| match e {
        LockfileWriteError::MissingSection(_) => LockfileWriteError::MissingHashesSection,
        other => other,
    })
}

/// Set the archive `MerkleRoot` signature for `key` (a `"name@version"`
/// string, without quotes) to `digest_hex` in the `[archive_signatures]`
/// section of `lockfile_text`, returning the rewritten text. Written in the
/// tagged emit form (`blake3:<digest_hex>`).
///
/// This is the write-side companion to the fail-closed `.prx.gz` load
/// gate: a loaded archive's re-derived `MerkleRoot` is checked against
/// this pin (`owl::prx::load_prx_gz_from_lock` /
/// `uslm::corpus::prx::load_usc_prx_gz_from_lock`). `pr4xis compile
/// --lock` records the pin here once the archive has been emitted.
///
/// Identical comment-/order-preserving rewrite to [`set_hash`], only
/// targeting `[archive_signatures]`. `digest_hex` must be 64 lowercase hex
/// characters; off-length/mixed-case inputs return
/// [`LockfileWriteError::InvalidDigest`], a missing section returns
/// [`LockfileWriteError::MissingSection`].
pub fn set_archive_signature(
    lockfile_text: &str,
    key: &str,
    digest_hex: &str,
) -> Result<String, LockfileWriteError> {
    set_in_section(lockfile_text, "archive_signatures", key, digest_hex)
}

/// Set the COMPACT archive content address for `key` (a `"name@version"`
/// string) to `digest_hex` in the `[compact_archive_signatures]` section of
/// `lockfile_text`, returning the rewritten text. Written in the tagged emit
/// form (`blake3:<digest_hex>`).
///
/// The write-side companion to the compact runtime load gate
/// (`uslm::corpus::prx::load_compact_usc_prx_gz_gated`): a loaded compact
/// archive's re-hashed content address is checked against this pin. Unlike
/// [`set_archive_signature`] the pinned address is portable across toolchains
/// (the compact codec is dependency-free bit-packing, not rkyv). Same
/// comment-/order-preserving rewrite; `digest_hex` must be 64 lowercase hex
/// chars.
pub fn set_compact_archive_signature(
    lockfile_text: &str,
    key: &str,
    digest_hex: &str,
) -> Result<String, LockfileWriteError> {
    set_in_section(lockfile_text, "compact_archive_signatures", key, digest_hex)
}

/// Set the English STORE-BUNDLE content address for `key` (a `"name@version"`
/// string) to `digest_hex` in the `[store_bundle_signatures]` section of
/// `lockfile_text`, returning the rewritten text. Written in the tagged emit
/// form (`blake3:<digest_hex>`).
///
/// The write-side companion to the store-bundle load gate
/// (`lmf::prx::load_english_store_bundle_gz_gated`): a loaded bundle's
/// re-hashed framed bytes are checked against this pin. Like
/// [`set_archive_signature`] — and UNLIKE [`set_compact_archive_signature`] —
/// the pinned address is a per-toolchain build output (four of the nine framed
/// buffers are rkyv envelopes), so it is valid only within one lockstep (the
/// wasm binary's embedded bundle, the native `.prx-cache`), never a published
/// portable wire. Same comment-/order-preserving rewrite; `digest_hex` must be
/// 64 lowercase hex chars.
pub fn set_store_bundle_signature(
    lockfile_text: &str,
    key: &str,
    digest_hex: &str,
) -> Result<String, LockfileWriteError> {
    set_in_section(lockfile_text, "store_bundle_signatures", key, digest_hex)
}

/// Set the canonical-form signature for `key` (a `"name@version"` string)
/// to `digest_hex` in the `[canonical_signatures]` section of
/// `lockfile_text`, returning the rewritten text. Written in the tagged
/// emit form (`blake3:<digest_hex>`).
///
/// The write-side companion to the canonical-form verification legs (the
/// round-trip harness and the `.prx` load gate's graph-identity leg):
/// `digest_hex` is the content address of the bytes the source's registered
/// `WellBehavedLens` emits as canonical form. Same comment-/order-preserving
/// rewrite as [`set_hash`].
pub fn set_canonical_signature(
    lockfile_text: &str,
    key: &str,
    digest_hex: &str,
) -> Result<String, LockfileWriteError> {
    set_in_section(lockfile_text, "canonical_signatures", key, digest_hex)
}

/// Set the byte-exact round-trip signature for `key` (a `"name@version"`
/// string) to `digest_hex` in the `[byte_exact_signatures]` section of
/// `lockfile_text`, returning the rewritten text. Written in the tagged
/// emit form (`blake3:<digest_hex>`).
///
/// Because byte-exactness means `put(get(b)) == b`, the value MUST equal the
/// source's `[hashes]` pin — the parse side (`registry::parse_praxis_lock`)
/// enforces that equality on load, so a divergent write surfaces immediately.
/// Same comment-/order-preserving rewrite as [`set_hash`].
pub fn set_byte_exact_signature(
    lockfile_text: &str,
    key: &str,
    digest_hex: &str,
) -> Result<String, LockfileWriteError> {
    set_in_section(lockfile_text, "byte_exact_signatures", key, digest_hex)
}

/// Comment-/order-preserving rewrite of a `"key" = "value"` line within a
/// named TOML section — the shared core of the `set_*` writers. Validates
/// `digest_hex` (64 lowercase hex), lowers it to the tagged emit form
/// (`<tag(ADDRESS_ALGORITHM)>:<digest_hex>` — the ONE wire lowering, shared
/// with the parser via [`algorithm_tag`]), then locates `[section]`, replaces
/// the value on an existing `"key"` line byte-for-byte, or appends a new line
/// at the section's end (before its trailing blank, or at EOF). Returns
/// [`LockfileWriteError::MissingSection`] if `[section]` is absent.
fn set_in_section(
    lockfile_text: &str,
    section: &str,
    key: &str,
    digest_hex: &str,
) -> Result<String, LockfileWriteError> {
    if !is_lowercase_hex_digest(digest_hex) {
        return Err(LockfileWriteError::InvalidDigest(digest_hex.to_string()));
    }
    let value = format!("{}:{}", algorithm_tag(ADDRESS_ALGORITHM), digest_hex);

    let lines: Vec<&str> = lockfile_text.split_inclusive('\n').collect();

    // Locate the section's bounds. `section_start` is the line index
    // immediately after the `[section]` header; `section_end` is the line
    // index of the next `[section]` header, or `lines.len()` if the
    // section runs to end-of-file.
    let Some(header_idx) = lines.iter().position(|l| is_section_header(l, section)) else {
        return Err(LockfileWriteError::MissingSection(section.to_string()));
    };
    let section_start = header_idx + 1;
    let section_end = lines[section_start..]
        .iter()
        .position(is_any_section_header)
        .map(|offset| section_start + offset)
        .unwrap_or(lines.len());

    // Scan the section for an existing `"key" = "..."` line. The TOML
    // spec allows arbitrary whitespace around `=`; we accept the same.
    if let Some(existing_idx) = lines[section_start..section_end]
        .iter()
        .position(|l| line_assigns_key_to(l, key))
    {
        let absolute = section_start + existing_idx;
        let new_line = replace_value_on_line(lines[absolute], &value);
        let mut out = String::with_capacity(lockfile_text.len());
        for (i, line) in lines.iter().enumerate() {
            if i == absolute {
                out.push_str(&new_line);
            } else {
                out.push_str(line);
            }
        }
        return Ok(out);
    }

    // No existing line — append `"key" = "value"` at the end of the
    // section, immediately before any trailing blank line that separates
    // it from the next `[section]` header. Keeps the visual structure
    // intact.
    let mut insert_idx = section_end;
    while insert_idx > section_start && lines[insert_idx - 1].trim().is_empty() {
        insert_idx -= 1;
    }
    let new_line = format!("\"{key}\" = \"{value}\"\n");
    let mut out = String::with_capacity(lockfile_text.len() + new_line.len());
    for (i, line) in lines.iter().enumerate() {
        if i == insert_idx {
            out.push_str(&new_line);
        }
        out.push_str(line);
    }
    if insert_idx == lines.len() {
        // Append at end-of-file. Ensure the file ends with a newline
        // before the new line (idempotent if it already did).
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&new_line);
    }
    Ok(out)
}

/// True iff `line` is a `[name]` section header (ignoring leading
/// whitespace and trailing comments / newline).
fn is_section_header(line: &str, name: &str) -> bool {
    let trimmed = strip_trailing_comment(line.trim_end_matches('\n')).trim();
    trimmed
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .map(str::trim)
        == Some(name)
}

/// True iff `line` is any `[section]` header — used to detect the
/// `[hashes]` section's end.
fn is_any_section_header(line: &&str) -> bool {
    let trimmed = strip_trailing_comment(line.trim_end_matches('\n')).trim();
    trimmed.starts_with('[') && trimmed.ends_with(']')
}

/// True iff `line` (a single TOML line) assigns a value to `key` (where
/// `key` is the unquoted identifier — the comparison adds quotes).
fn line_assigns_key_to(line: &str, key: &str) -> bool {
    let trimmed = line.trim_start();
    let quoted = format!("\"{key}\"");
    trimmed.starts_with(&quoted) && trimmed[quoted.len()..].trim_start().starts_with('=')
}

/// Replace the quoted string value on a `"key" = "..."` line with
/// `new_value`, leaving everything else (key, whitespace, trailing
/// comment, newline) intact.
fn replace_value_on_line(line: &str, new_value: &str) -> String {
    // Find the `=` then the opening quote of the value.
    let Some(eq_pos) = line.find('=') else {
        return line.to_string();
    };
    let after_eq = &line[eq_pos + 1..];
    let Some(open_offset_in_after) = after_eq.find('"') else {
        return line.to_string();
    };
    let open_quote_pos = eq_pos + 1 + open_offset_in_after;
    let value_start = open_quote_pos + 1;
    let after_open_quote = &line[value_start..];
    let Some(close_offset) = after_open_quote.find('"') else {
        return line.to_string();
    };
    let close_quote_pos = value_start + close_offset;

    let mut out = String::with_capacity(line.len() + new_value.len());
    out.push_str(&line[..value_start]);
    out.push_str(new_value);
    out.push_str(&line[close_quote_pos..]);
    out
}

/// True iff `s` is exactly 64 lowercase hex characters — the hex form of
/// one 256-bit digest (the emit-leg `ContentAddress::to_hex()` shape),
/// using the ASCII subset `[0-9a-f]`. Mirrors the parser's shape check so
/// we never write a value that would fail to load.
fn is_lowercase_hex_digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Strip a `# comment` suffix from a line, returning everything before
/// the first un-quoted `#`. For our purposes (section headers and
/// digest lines), the `#` always starts a comment because digests and
/// section names cannot contain `#`. This is intentionally a simple
/// approximation, not a full TOML lexer.
fn strip_trailing_comment(line: &str) -> &str {
    match line.find('#') {
        Some(i) => &line[..i],
        None => line,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
# header comment
[hashes]
# wordnet header
\"english_wordnet@2025\" = \"blake3:0000000000000000000000000000000000000000000000000000000000000000\"
\"another@1.0\"          = \"1111111111111111111111111111111111111111111111111111111111111111\"

[canonical_signatures]
\"english_wordnet@2025\" = \"blake3:2222222222222222222222222222222222222222222222222222222222222222\"
";

    const HEX_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HEX_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn replaces_existing_hash_preserving_comments() {
        let out = set_hash(SAMPLE, "english_wordnet@2025", HEX_A).unwrap();
        // Preamble preserved
        assert!(out.starts_with("# header comment\n[hashes]\n# wordnet header\n"));
        // Existing line was rewritten to the new tagged digest
        assert!(out.contains(&format!("\"english_wordnet@2025\" = \"blake3:{HEX_A}\"")));
        // Other key untouched (still the bare legacy value)
        assert!(out.contains("\"another@1.0\"          = \"1111111111"));
        // [canonical_signatures] preserved unchanged
        assert!(out.contains("[canonical_signatures]\n\"english_wordnet@2025\" = \"blake3:222222"));
        // No accidental duplication
        assert_eq!(
            out.matches("english_wordnet@2025\" = \"blake3:a").count(),
            1,
            "rewritten key must appear exactly once in [hashes]"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn rewrites_bare_legacy_value_to_tagged_form() {
        // A pre-tagged-grammar line (bare hex) is re-pinned in the tagged
        // emit form — the writer always writes `blake3:<hex>`.
        let out = set_hash(SAMPLE, "another@1.0", HEX_B).unwrap();
        assert!(out.contains(&format!("\"another@1.0\"          = \"blake3:{HEX_B}\"")));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn appends_new_hash_under_hashes_section_before_canonical_section() {
        let out = set_hash(SAMPLE, "fresh_source@v1", HEX_A).unwrap();
        // New line lives after the existing [hashes] entries but before
        // [canonical_signatures] — that is, inside the [hashes] section.
        let new_pos = out
            .find(&format!("\"fresh_source@v1\" = \"blake3:{HEX_A}\""))
            .expect("new line must be present");
        let canon_pos = out
            .find("[canonical_signatures]")
            .expect("[canonical_signatures] must still exist");
        assert!(
            new_pos < canon_pos,
            "new key must be inside [hashes] (before [canonical_signatures])"
        );
        // Pre-existing keys untouched
        assert!(out.contains("\"english_wordnet@2025\" = \"blake3:0000000"));
        assert!(out.contains("\"another@1.0\"          = \"111111111"));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn appends_at_end_when_hashes_is_last_section() {
        let single = "[hashes]\n\"k@1\" = \"blake3:0000000000000000000000000000000000000000000000000000000000000000\"\n";
        let out = set_hash(single, "new@v1", HEX_B).unwrap();
        assert!(out.contains("\"new@v1\" = \"blake3:bbbbbbbbb"));
        assert!(out.ends_with(
            "\"new@v1\" = \"blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\"\n"
        ));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn missing_hashes_section_returns_error() {
        let no_hashes = "[other]\nfoo = \"bar\"\n";
        assert_eq!(
            set_hash(no_hashes, "k@1", HEX_A).unwrap_err(),
            LockfileWriteError::MissingHashesSection
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn rejects_non_lowercase_hex_digest() {
        // Wrong length
        let err = set_hash(SAMPLE, "k@1", "abc").unwrap_err();
        assert!(matches!(err, LockfileWriteError::InvalidDigest(_)));
        // Uppercase hex
        let err = set_hash(SAMPLE, "k@1", &"A".repeat(64)).unwrap_err();
        assert!(matches!(err, LockfileWriteError::InvalidDigest(_)));
        // Non-hex character
        let err = set_hash(SAMPLE, "k@1", &"g".repeat(64)).unwrap_err();
        assert!(matches!(err, LockfileWriteError::InvalidDigest(_)));
        // Already-tagged input: the writer takes the bare emit-leg hex and
        // adds the tag itself — a pre-tagged value is rejected, not
        // double-tagged.
        let err = set_hash(SAMPLE, "k@1", &format!("blake3:{HEX_A}")).unwrap_err();
        assert!(matches!(err, LockfileWriteError::InvalidDigest(_)));
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn idempotent_when_rewriting_to_same_value() {
        let same = "0000000000000000000000000000000000000000000000000000000000000000";
        let out = set_hash(SAMPLE, "english_wordnet@2025", same).unwrap();
        assert_eq!(
            out, SAMPLE,
            "rewriting to same value must leave the file unchanged"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn preserves_inline_comment_after_value() {
        let input = "[hashes]\n\"k@1\" = \"blake3:0000000000000000000000000000000000000000000000000000000000000000\"  # inline note\n";
        let out = set_hash(input, "k@1", HEX_A).unwrap();
        assert!(
            out.contains(&format!("\"k@1\" = \"blake3:{HEX_A}\"  # inline note\n")),
            "inline comment must survive rewrite; got: {out:?}"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn set_canonical_signature_targets_its_section() {
        let out = set_canonical_signature(SAMPLE, "english_wordnet@2025", HEX_A).unwrap();
        // The [hashes] line is untouched; the [canonical_signatures] line moved.
        assert!(out.contains("\"english_wordnet@2025\" = \"blake3:0000000"));
        assert!(out.contains(&format!(
            "[canonical_signatures]\n\"english_wordnet@2025\" = \"blake3:{HEX_A}\""
        )));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn set_byte_exact_signature_requires_its_section() {
        // SAMPLE has no [byte_exact_signatures] section — fail closed.
        assert_eq!(
            set_byte_exact_signature(SAMPLE, "k@1", HEX_A).unwrap_err(),
            LockfileWriteError::MissingSection("byte_exact_signatures".into())
        );
        let with_section = format!("{SAMPLE}\n[byte_exact_signatures]\n");
        let out = set_byte_exact_signature(&with_section, "k@1", HEX_A).unwrap();
        assert!(out.contains(&format!("\"k@1\" = \"blake3:{HEX_A}\"")));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn line_assigns_key_to_matches_quoted_lhs_only() {
        assert!(line_assigns_key_to(
            "\"english_wordnet@2025\" = \"deadbeef\"",
            "english_wordnet@2025"
        ));
        assert!(line_assigns_key_to(
            "  \"english_wordnet@2025\"   =   \"deadbeef\"",
            "english_wordnet@2025"
        ));
        assert!(!line_assigns_key_to(
            "\"other@1.0\" = \"deadbeef\"",
            "english_wordnet@2025"
        ));
        // A line that mentions the key inside a comment must not match.
        assert!(!line_assigns_key_to(
            "# \"english_wordnet@2025\" = \"deadbeef\"",
            "english_wordnet@2025"
        ));
    }
}
