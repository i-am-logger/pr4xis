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
//! line-by-line, locates the `"<key>" = "<sha>"` line for the requested
//! key, and replaces just the SHA hex while leaving every other byte
//! (comments, ordering, whitespace) untouched. New keys are appended at
//! the end of the `[hashes]` section, before the next `[section]` header
//! if any, otherwise at end-of-file.
//!
//! Citation: Tom Preston-Werner et al. (2024) *TOML: Tom's Obvious
//! Minimal Language* v1.0.0, §2 (comments are syntactically significant
//! to humans). RFC 1952 / 5234 ABNF-style "leave bytes alone unless they
//! match the production" rewriting principle. Dolstra (2006) §5.1
//! (content-addressed pinning) — the value being rewritten is a SHA-256
//! per FIPS 180-4 §6.2.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

/// Errors returned by [`set_hash`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockfileWriteError {
    /// The lockfile text has no `[hashes]` section. A praxis.lock without
    /// `[hashes]` is malformed — refuse to write blind.
    MissingHashesSection,
    /// The provided SHA-256 hex is not 64 lowercase hex characters.
    /// Mirrors `registry::is_lowercase_hex_sha256` so we never write a
    /// malformed value (FIPS 180-4 §6.2: SHA-256 output is 256 bits → 64
    /// hex chars; TOML doesn't care about case but the load-time
    /// parser rejects mixed-case for canonical signatures, and we keep
    /// raw hashes consistent).
    InvalidSha256(String),
}

impl core::fmt::Display for LockfileWriteError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LockfileWriteError::MissingHashesSection => {
                write!(f, "praxis.lock has no `[hashes]` section")
            }
            LockfileWriteError::InvalidSha256(s) => {
                write!(f, "not a 64-char lowercase hex SHA-256: {s:?}")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for LockfileWriteError {}

/// Set the SHA-256 for `key` (a `"name@version"` string, without quotes)
/// to `sha` in the `[hashes]` section of `lockfile_text`, returning the
/// rewritten text.
///
/// - If `key` already has a line in `[hashes]`, only the hex value is
///   replaced; the line's comments, whitespace, and surrounding lines
///   are preserved byte-for-byte.
/// - If `key` is absent, a new `"key" = "sha"` line is appended at the
///   end of the `[hashes]` section (immediately before the next
///   `[section]` header, or at end-of-file if `[hashes]` is the last).
/// - If the lockfile has no `[hashes]` section, returns
///   [`LockfileWriteError::MissingHashesSection`] — writing blind to a
///   malformed lockfile would silently corrupt it.
///
/// `sha` must be 64 lowercase hex characters (one SHA-256 per
/// FIPS 180-4 §6.2). Mixed-case or off-length inputs are rejected with
/// [`LockfileWriteError::InvalidSha256`].
pub fn set_hash(lockfile_text: &str, key: &str, sha: &str) -> Result<String, LockfileWriteError> {
    if !is_lowercase_hex_sha256(sha) {
        return Err(LockfileWriteError::InvalidSha256(sha.to_string()));
    }

    let lines: Vec<&str> = lockfile_text.split_inclusive('\n').collect();

    // Locate the [hashes] section's bounds. `hashes_start` is the line
    // index immediately after the `[hashes]` header; `hashes_end` is the
    // line index of the next `[section]` header, or `lines.len()` if
    // `[hashes]` runs to end-of-file.
    let Some(hashes_header_idx) = lines.iter().position(|l| is_section_header(l, "hashes")) else {
        return Err(LockfileWriteError::MissingHashesSection);
    };
    let hashes_start = hashes_header_idx + 1;
    let hashes_end = lines[hashes_start..]
        .iter()
        .position(is_any_section_header)
        .map(|offset| hashes_start + offset)
        .unwrap_or(lines.len());

    // Scan the section for an existing `"key" = "..."` line. The TOML
    // spec allows arbitrary whitespace around `=`; we accept the same.
    if let Some(existing_idx) = lines[hashes_start..hashes_end]
        .iter()
        .position(|l| line_assigns_key_to(l, key))
    {
        let absolute = hashes_start + existing_idx;
        let new_line = replace_value_on_line(lines[absolute], sha);
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

    // No existing line — append `"key" = "sha"` at the end of the
    // `[hashes]` section, immediately before any trailing blank line
    // that separates `[hashes]` from the next `[section]` header. This
    // keeps the visual structure intact.
    let mut insert_idx = hashes_end;
    while insert_idx > hashes_start && lines[insert_idx - 1].trim().is_empty() {
        insert_idx -= 1;
    }
    let new_line = format!("\"{key}\" = \"{sha}\"\n");
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

/// True iff `s` is exactly 64 lowercase hex characters — the canonical
/// form of a SHA-256 per FIPS 180-4 §6.2 (256 bits → 64 hex) using the
/// ASCII subset `[0-9a-f]`. Mirrors the parser's validity check so we
/// never write a value that would fail to load.
fn is_lowercase_hex_sha256(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Strip a `# comment` suffix from a line, returning everything before
/// the first un-quoted `#`. For our purposes (section headers and
/// hash lines), the `#` always starts a comment because hashes and
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
\"english_wordnet@2025\" = \"0000000000000000000000000000000000000000000000000000000000000000\"
\"another@1.0\"          = \"1111111111111111111111111111111111111111111111111111111111111111\"

[canonical_signatures]
\"english_wordnet@2025\" = \"2222222222222222222222222222222222222222222222222222222222222222\"
";

    const SHA_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const SHA_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    #[test]
    fn replaces_existing_hash_preserving_comments() {
        let out = set_hash(SAMPLE, "english_wordnet@2025", SHA_A).unwrap();
        // Preamble preserved
        assert!(out.starts_with("# header comment\n[hashes]\n# wordnet header\n"));
        // Existing line was rewritten to the new sha
        assert!(out.contains(&format!("\"english_wordnet@2025\" = \"{SHA_A}\"")));
        // Other key untouched
        assert!(out.contains("\"another@1.0\"          = \"1111111111"));
        // [canonical_signatures] preserved unchanged
        assert!(out.contains("[canonical_signatures]\n\"english_wordnet@2025\" = \"222222"));
        // No accidental duplication
        assert_eq!(
            out.matches("english_wordnet@2025\" = \"a").count(),
            1,
            "rewritten key must appear exactly once in [hashes]"
        );
    }

    #[test]
    fn appends_new_hash_under_hashes_section_before_canonical_section() {
        let out = set_hash(SAMPLE, "fresh_source@v1", SHA_A).unwrap();
        // New line lives after the existing [hashes] entries but before
        // [canonical_signatures] — that is, inside the [hashes] section.
        let new_pos = out
            .find(&format!("\"fresh_source@v1\" = \"{SHA_A}\""))
            .expect("new line must be present");
        let canon_pos = out
            .find("[canonical_signatures]")
            .expect("[canonical_signatures] must still exist");
        assert!(
            new_pos < canon_pos,
            "new key must be inside [hashes] (before [canonical_signatures])"
        );
        // Pre-existing keys untouched
        assert!(out.contains("\"english_wordnet@2025\" = \"0000000"));
        assert!(out.contains("\"another@1.0\"          = \"111111111"));
    }

    #[test]
    fn appends_at_end_when_hashes_is_last_section() {
        let single = "[hashes]\n\"k@1\" = \"0000000000000000000000000000000000000000000000000000000000000000\"\n";
        let out = set_hash(single, "new@v1", SHA_B).unwrap();
        assert!(out.contains("\"new@v1\" = \"bbbbbbbbb"));
        assert!(out.ends_with(
            "\"new@v1\" = \"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\"\n"
        ));
    }

    #[test]
    fn missing_hashes_section_returns_error() {
        let no_hashes = "[other]\nfoo = \"bar\"\n";
        assert_eq!(
            set_hash(no_hashes, "k@1", SHA_A).unwrap_err(),
            LockfileWriteError::MissingHashesSection
        );
    }

    #[test]
    fn rejects_non_lowercase_hex_sha256() {
        // Wrong length
        let err = set_hash(SAMPLE, "k@1", "abc").unwrap_err();
        assert!(matches!(err, LockfileWriteError::InvalidSha256(_)));
        // Uppercase hex
        let err = set_hash(SAMPLE, "k@1", &"A".repeat(64)).unwrap_err();
        assert!(matches!(err, LockfileWriteError::InvalidSha256(_)));
        // Non-hex character
        let err = set_hash(SAMPLE, "k@1", &"g".repeat(64)).unwrap_err();
        assert!(matches!(err, LockfileWriteError::InvalidSha256(_)));
    }

    #[test]
    fn idempotent_when_rewriting_to_same_value() {
        let same = "0000000000000000000000000000000000000000000000000000000000000000";
        let out = set_hash(SAMPLE, "english_wordnet@2025", same).unwrap();
        assert_eq!(
            out, SAMPLE,
            "rewriting to same value must leave the file unchanged"
        );
    }

    #[test]
    fn preserves_inline_comment_after_value() {
        let input = "[hashes]\n\"k@1\" = \"0000000000000000000000000000000000000000000000000000000000000000\"  # inline note\n";
        let out = set_hash(input, "k@1", SHA_A).unwrap();
        assert!(
            out.contains(&format!("\"k@1\" = \"{SHA_A}\"  # inline note\n")),
            "inline comment must survive rewrite; got: {out:?}"
        );
    }

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
