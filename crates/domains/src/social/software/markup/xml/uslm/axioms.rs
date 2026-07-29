//! USLM structural axioms — pure validators over a parsed [`UsCodeTitle`].
//!
//! Each `axiom_*` returns `Ok(())` when the structural invariant holds for the
//! whole title, or `Err(reason)` naming the first violation. They ground in the
//! LRC's published USLM conventions (identifier URNs, strict hierarchy nesting,
//! the `<num>` duplicate-numbering footnote), not in hand-curated lists.
//!
//! Gated on `#[cfg(any(test, feature = "test-internals"))]`: the crate's own
//! `#[cfg(test)]` modules use them, and so does the workspace heavy-corpus test
//! crate (`crates/praxis-corpus-tests`, which depends on `pr4xis-domains` with
//! `features = ["test-internals"]`) so it can validate a full title parsed ONCE
//! under `cargo test` instead of re-parsing it per process-isolated nextest
//! test. Not part of the normal/published API surface.

use alloc::{format, string::String, vec::Vec};

use super::corpus::{UsCodeSection, UsCodeSubdivision, UsCodeTitle};

/// Every `<section>` carries a non-empty `<num>` (USLM Schema; the § cannot be
/// cited without its number, Bluebook §3.3).
pub fn axiom_every_section_has_num(title: &UsCodeTitle) -> Result<(), String> {
    for s in &title.sections {
        if s.num.is_empty() {
            return Err(format!("section {} has empty num", s.identifier));
        }
    }
    Ok(())
}

/// Every container (the title, every section, every nested subdivision) has a
/// non-empty USLM identifier URN — without it cross-references can't resolve.
pub fn axiom_every_container_has_identifier(title: &UsCodeTitle) -> Result<(), String> {
    if title.identifier.is_empty() {
        return Err("title identifier is empty".into());
    }
    for s in &title.sections {
        if s.identifier.is_empty() {
            return Err(format!("section with num {} has empty identifier", s.num));
        }
        for child in &s.children {
            axiom_check_subdivision_identifier(child)?;
        }
    }
    Ok(())
}

pub fn axiom_check_subdivision_identifier(d: &UsCodeSubdivision) -> Result<(), String> {
    if d.identifier.is_empty() {
        return Err(format!(
            "{:?} with num {} has empty identifier",
            d.kind, d.num
        ));
    }
    for c in &d.children {
        axiom_check_subdivision_identifier(c)?;
    }
    Ok(())
}

/// Child container identifiers strictly extend the parent's, separated by `/`
/// (USLM's hierarchical URN paths; Bluebook §3.3).
pub fn axiom_child_identifier_extends_parent(title: &UsCodeTitle) -> Result<(), String> {
    for s in &title.sections {
        for child in &s.children {
            axiom_check_subdivision_extends(&s.identifier, child)?;
        }
    }
    Ok(())
}

pub fn axiom_check_subdivision_extends(
    parent_id: &str,
    d: &UsCodeSubdivision,
) -> Result<(), String> {
    if !d.identifier.starts_with(parent_id) {
        return Err(format!(
            "{:?} identifier {} does not extend parent {}",
            d.kind, d.identifier, parent_id
        ));
    }
    let suffix = &d.identifier[parent_id.len()..];
    if !suffix.starts_with('/') {
        return Err(format!(
            "{:?} identifier {} extends parent {} without `/` separator",
            d.kind, d.identifier, parent_id
        ));
    }
    for c in &d.children {
        axiom_check_subdivision_extends(&d.identifier, c)?;
    }
    Ok(())
}

/// The hierarchy is strictly nested: a child's `kind.nesting_depth()` is
/// strictly greater than its parent's (Subsection 0 > Paragraph 1 > … > Subitem 6).
pub fn axiom_hierarchy_strictly_nested(title: &UsCodeTitle) -> Result<(), String> {
    for s in &title.sections {
        for child in &s.children {
            // Children of a Section start the hierarchy — any depth is
            // acceptable as long as further nesting is strict.
            axiom_check_strict_nesting(child)?;
        }
    }
    Ok(())
}

pub fn axiom_check_strict_nesting(parent: &UsCodeSubdivision) -> Result<(), String> {
    let parent_depth = parent.kind.nesting_depth();
    for child in &parent.children {
        if child.kind.nesting_depth().value <= parent_depth.value {
            return Err(format!(
                "{:?} (depth {}) at identifier {} has child {:?} (depth {}) — not strictly nested",
                parent.kind,
                parent_depth.value,
                parent.identifier,
                child.kind,
                child.kind.nesting_depth().value
            ));
        }
        axiom_check_strict_nesting(child)?;
    }
    Ok(())
}

/// A section URN repeats within a title ONLY when every occurrence carries the
/// LRC's duplicate-numbering cross-reference footnote inside its `<num>`. (The
/// § 3598 case disproves "headings always differ"; the `<num>` footnote is the
/// LRC's own disambiguation mechanism. Per `feedback_bottom_up_loaded_not_encoded`,
/// this grounds in the loaded XML's publication structure, not a curated list.)
pub fn axiom_section_identifiers_unique(title: &UsCodeTitle) -> Result<(), String> {
    let mut by_urn: hashbrown::HashMap<&str, Vec<&UsCodeSection>> = hashbrown::HashMap::new();
    for s in &title.sections {
        by_urn.entry(s.identifier.as_str()).or_default().push(s);
    }
    for (urn, group) in &by_urn {
        if group.len() == 1 {
            continue;
        }
        for s in group {
            let is_lrc_dup = s
                .num_footnote
                .as_deref()
                .map(is_lrc_duplicate_numbering_footnote)
                .unwrap_or(false);
            if !is_lrc_dup {
                return Err(format!(
                    "URN {urn:?} appears {} times, but the occurrence headed {:?} carries no \
                     LRC duplicate-numbering footnote (\"Another section N is set out…\") in its \
                     <num>. A repeated URN is legitimate only when every occurrence bears that \
                     footnote; absent it, this is a parse error or corpus corruption.",
                    group.len(),
                    s.heading
                ));
            }
        }
    }
    Ok(())
}

/// True iff `footnote` is the LRC's duplicate-numbering cross-reference idiom —
/// "Another section N is set out [after|preceding] this section."
pub fn is_lrc_duplicate_numbering_footnote(footnote: &str) -> bool {
    footnote.contains("Another section") && footnote.contains("set out")
}

/// All `<ref href="...">` URNs follow the USLM identifier shape (root-relative,
/// begin with `/`).
pub fn axiom_ref_hrefs_well_formed(title: &UsCodeTitle) -> Result<(), String> {
    for s in &title.sections {
        for r in &s.refs {
            axiom_check_ref_shape(&r.href, &s.identifier)?;
        }
        for child in &s.children {
            axiom_check_subdivision_refs(child)?;
        }
    }
    Ok(())
}

pub fn axiom_check_subdivision_refs(d: &UsCodeSubdivision) -> Result<(), String> {
    for r in &d.refs {
        axiom_check_ref_shape(&r.href, &d.identifier)?;
    }
    for c in &d.children {
        axiom_check_subdivision_refs(c)?;
    }
    Ok(())
}

pub fn axiom_check_ref_shape(href: &str, in_identifier: &str) -> Result<(), String> {
    if href.is_empty() {
        return Err(format!(
            "empty ref href encountered in subtree of {in_identifier}"
        ));
    }
    if !href.starts_with('/') {
        return Err(format!(
            "ref href {href:?} in subtree of {in_identifier} not URN-rooted (expected /...)"
        ));
    }
    Ok(())
}

/// Derive a praxis statute_name from a USLM section identifier — lowercase +
/// slash-to-underscore (e.g. `/us/usc/t18/s1514A` → `usc_t18_s1514a`). The
/// result satisfies the CURIE prefix pattern `[a-z][a-z0-9_]*`.
pub fn section_identifier_to_statute_name(identifier: &str) -> String {
    let trimmed = identifier.trim_start_matches('/');
    trimmed.replace('/', "_").to_lowercase()
}
