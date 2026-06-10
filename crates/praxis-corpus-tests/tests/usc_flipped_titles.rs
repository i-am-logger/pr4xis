//! Flipped-title byte-exact gates (USLM slice U7) — lifted out of the
//! `pr4xis-domains` `#[cfg(test)]` modules.
//!
//! Each "flipped" title is a smaller positive-law USC title that rides the SAME
//! generic `uscdoc_mixed` document-wrapper path Title 1 proved, but exercises
//! USLM families ABSENT from Title 1 (the `<continuation>` flush-text family, the
//! XHTML `<table>` family). They reconstruct BYTE-FOR-BYTE from the typed
//! `UsCodeTitle` ontology + captured `UslmSyntaxComplement` over the LITERAL
//! on-disk file (CRLFs included). Capturing + re-emitting four mid-size titles is
//! heavy, so it runs here in the one-process heavy-corpus lane rather than under
//! the strict fast-lane per-test cap.
//!
//! The `corrupt_first_text_in` helper is carried verbatim from
//! `uslm::lens::writer`'s `#[cfg(test)]` module (where it is shared with the
//! bare-section corruption meta-tests that stay in-crate). USC titles are
//! externally provisioned (`pr4xis update`), so a plain checkout skips gracefully.

use pr4xis_domains::social::software::markup::xml::uslm::UsCodeContentNode;
use pr4xis_domains::social::software::markup::xml::uslm::lens::writer::{
    capture_uslm_complement, reconstruct_uslm_source,
};
use pr4xis_runtime::address::ContentAddress;

use praxis_corpus_tests::workspace_root;

/// Corrupt the FIRST descendant `#PCDATA` `UsCodeContentNode::Text` leaf in a
/// mixed-content node list (pre-order), rewriting it to a different value.
/// Returns whether a leaf was found and mutated. The flipped titles' text leaves
/// sit one or two levels deep, so the walk recurses through element nodes.
fn corrupt_first_text_in(nodes: &mut [UsCodeContentNode]) -> bool {
    for node in nodes {
        match node {
            UsCodeContentNode::Text(t) => {
                *t = format!("{t}-CORRUPTED");
                return true;
            }
            UsCodeContentNode::Ref { children, .. }
            | UsCodeContentNode::Date { children, .. }
            | UsCodeContentNode::Inline { children, .. }
            | UsCodeContentNode::Para { children, .. }
            | UsCodeContentNode::Generic { children, .. } => {
                if corrupt_first_text_in(children) {
                    return true;
                }
            }
        }
    }
    false
}

/// HARD BYTE-EXACT GATE (slice U7): each FLIPPED on-disk USC title reconstructs
/// BYTE-FOR-BYTE from the typed `UsCodeTitle` ontology + captured
/// `UslmSyntaxComplement`, over the LITERAL on-disk file (CRLFs included). These
/// titles exercise USLM families ABSENT from Title 1 — the `<continuation>`
/// flush-text family and the XHTML `<table>` family — yet regenerate byte-exact
/// because the generic mixed-content backbone carries every such element as a
/// `UsCodeContentNode::Generic` node, node-for-node, and the byte kernel restores
/// their concrete-syntax residue (the prolog `<?xml-stylesheet?>` PI, the §2.11
/// prolog CRLFs, the §4.6 `&amp;` predefined-entity form, the start-tag attribute
/// order, the inter-element white-space).
#[test]
fn flipped_titles_reconstruct_byte_exact() {
    // (number, expected source digest == the `[hashes]` pin == the
    // `[byte_exact_signatures]` pin). Each is the literal on-disk file's content
    // address; `put(get(b)) == b` makes the round-trip output hash equal it.
    const FLIPPED: &[(&str, &str)] = &[
        (
            "28",
            "fb9d714e6b0f1da383981cb8d0d02afa81af30e70f055354585bd2a7453981c8",
        ),
        (
            "18",
            "e00e7187ee9b1b95cc612c9c3b40596b05f3421cf9e6b2917b5290585f1fcd0a",
        ),
        (
            "29",
            "0c59a41dddfcc3a5a53aeef020c2ed58c649ef5d6af209316b5c5919977a3843",
        ),
        (
            "50",
            "f42660413a471f1133c1e23f93038cae9ac0437885358492e119a0fcd98aa311",
        ),
    ];
    for (n, expected_address) in FLIPPED {
        let path = workspace_root().join(format!(
            "crates/domains/data/legal/uscode/usc_title_{n}/usc_title_{n}-pl-119-90.xml"
        ));
        let Ok(bytes) = std::fs::read(&path) else {
            continue; // corpus not provisioned — skip gracefully
        };
        // Sanity: the source's content address is the pinned value (else a corpus
        // swap silently weakened this gate).
        assert_eq!(
            &ContentAddress::of(&bytes).to_hex(),
            expected_address,
            "title {n} on-disk source must hash to its pinned content address"
        );
        let src = String::from_utf8(bytes).expect("title is UTF-8");
        // Sanity: this title genuinely exercises a family ABSENT from Title 1
        // (continuation and/or XHTML table) — else the slice is vacuous.
        assert!(
            src.contains("<continuation") || src.contains("<table"),
            "title {n} must exercise a beyond-Title-1 family (continuation/table)"
        );
        let (title, complement) = capture_uslm_complement(&src).expect("capture flipped title");
        // The residue is GENUINELY present (a vacuous round-trip would lie). A full
        // `<uscDoc>` document carries its inter-element white-space as verbatim
        // `#PCDATA` in the semantic mixed tree (not as the `content_whitespace`
        // residue the bare-section slice uses), so the genuine byte residue here is
        // the start-tag attribute overrides (the reordered/dropped
        // `<uscDoc>`/`<section>`/`<table>` attrs), the prolog `<?xml-stylesheet?>`
        // PI, and the §2.11 prolog CRLFs.
        assert!(
            !complement.regenerated.attribute_overrides.is_empty(),
            "title {n} must carry genuine start-tag attribute-override residue"
        );
        assert!(
            complement
                .syntax_decisions
                .prolog()
                .after_xml_decl
                .contains("<?xml-stylesheet"),
            "title {n} must carry the prolog <?xml-stylesheet?> PI residue"
        );
        assert!(
            !complement.syntax_decisions.eol_form().is_empty(),
            "title {n} must carry the §2.11 prolog-CRLF EOL-form residue"
        );
        let out = reconstruct_uslm_source(&title, &complement).expect("reconstruct");
        assert_eq!(
            out,
            src.as_bytes(),
            "title {n} must reconstruct byte-for-byte from the graph + complement"
        );
        assert_eq!(
            &ContentAddress::of(&out).to_hex(),
            expected_address,
            "title {n} round-trip output must hash to its pinned content address"
        );
    }
}

/// META-TEST (slice U7 has TEETH on EVERY flipped title): for each flipped title,
/// capture the real `<uscDoc>` document, CORRUPT a #PCDATA Text leaf deep in the
/// `uscdoc_mixed` backbone (continuation/table family content), and assert the
/// byte-exact reconstruction NO LONGER equals the source. Proves the flipped
/// titles' beyond-Title-1 families are reproduced from the EXACT text, not merely
/// a backbone the positional diff could reconcile. The UNcorrupted (clean)
/// round-trip is the control proven for all four by
/// `flipped_titles_reconstruct_byte_exact`.
#[test]
fn corrupted_flipped_title_breaks_byte_exact_gate() {
    for n in ["28", "18", "29", "50"] {
        let path = workspace_root().join(format!(
            "crates/domains/data/legal/uscode/usc_title_{n}/usc_title_{n}-pl-119-90.xml"
        ));
        let Ok(bytes) = std::fs::read(&path) else {
            continue; // corpus not provisioned — skip gracefully
        };
        let src = String::from_utf8(bytes).expect("title is UTF-8");
        let (mut title, complement) = capture_uslm_complement(&src).expect("capture flipped title");

        // Corrupt the FIRST #PCDATA Text leaf anywhere in the document backbone.
        // The element backbone stays identical (the complement's pre-order walk
        // still succeeds), but a faithful writer must now emit different bytes.
        let uscdoc = title
            .uscdoc_mixed
            .as_mut()
            .expect("a flipped title is a full <uscDoc> document with a captured backbone");
        assert!(
            corrupt_first_text_in(&mut uscdoc.nodes),
            "title {n} backbone must carry a #PCDATA Text leaf to corrupt"
        );

        let out = reconstruct_uslm_source(&title, &complement)
            .expect("reconstruct still runs on a corrupted-but-backbone-valid model");
        assert_ne!(
            out,
            src.as_bytes(),
            "a corrupted title {n} backbone #PCDATA value MUST diverge the byte-exact \
             reconstruction — the U7 flipped-title gate has teeth"
        );
    }
}
