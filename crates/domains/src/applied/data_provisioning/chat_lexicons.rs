//! Registered definitional chat lexicons — the registry-driven residency
//! surface a chat host (the CLI REPL, the wasm runtime) composes at startup.
//!
//! Selection is BY TAXONOMY KIND, never by source name: a host iterates
//! [`data_sources`](super::registry::data_sources) and asks
//! [`definitional_lexicon_runtime_ontology`] for each entry; an entry whose
//! kind is a chat-composable DEFINITIONAL lexicon leaf (see
//! [`SourceTaxonomyConcept`] — `CaregivingLexicon`, `HcbsComplianceLexicon`;
//! any future sibling registers here the same way) materializes through the
//! ONE generic, provenance-carrying WN-LMF bridge
//! ([`lexicon_runtime_ontology`]),
//! and every other kind answers `None`. Adding a lexicon therefore never
//! touches a host: register the source (`praxis.toml` + `praxis.lock`), add
//! its taxonomy leaf, and map the leaf to its committed bundle in
//! `embedded_definitional_lexicon_prx` — the hosts pick it up by iterating
//! the registry.
//!
//! The definitional/recognizer split this module encodes is the
//! `SourceTaxonomyConcept` docs' own distinction (Pustejovsky 1995
//! domain-scoped qualia; Solan 1993 statutory terms of art): a DEFINITIONAL
//! lexicon carries one cited gloss per synset for the chat to recite, while
//! the closed-class `LegalLexicon` is a statutory RECOGNIZER vocabulary and
//! is deliberately NOT selected here.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

use pr4xis_runtime::ontology::RuntimeOntology;

use super::lexicon_provenance::lexicon_runtime_ontology;
use super::ontology::RegistryEntry;
use crate::cognitive::linguistics::english::bridge::LexiconBridgeError;
use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;

/// The committed `.prx` bundle for a definitional-chat-lexicon TAXONOMY KIND —
/// the one place a leaf maps to its embedded bytes (`include_bytes!` is
/// compile-time, so the mapping is necessarily a finite kind-keyed table; each
/// bundle's `(name, version, bytes)` constants live beside the source's own
/// module). `None` for every kind that is not a definitional chat lexicon —
/// this table IS the predicate the hosts filter the registry by.
fn embedded_definitional_lexicon_prx(kind: SourceTaxonomyConcept) -> Option<&'static [u8]> {
    use crate::social::care::{caregiving_lexicon, hcbs_compliance_lexicon};
    match kind {
        SourceTaxonomyConcept::CaregivingLexicon => {
            Some(caregiving_lexicon::CAREGIVING_LEXICON_PRX)
        }
        SourceTaxonomyConcept::HcbsComplianceLexicon => {
            Some(hcbs_compliance_lexicon::HCBS_COMPLIANCE_LEXICON_PRX)
        }
        _ => None,
    }
}

/// Materialize a registry entry as a chat-composable [`RuntimeOntology`] iff
/// its kind is a definitional chat lexicon — the registry-driven residency
/// step. `None` for every other kind (the host just skips the entry); the
/// inner `Result` carries the generic bridge's typed failure.
///
/// The parameterization is ENTIRELY the entry's registered `(name, version)`
/// plus the kind-keyed committed bundle — no per-lexicon code path, the
/// `us_legal_lexicon` zero-per-domain-code discipline.
pub fn definitional_lexicon_runtime_ontology(
    entry: &RegistryEntry,
) -> Option<Result<RuntimeOntology, LexiconBridgeError>> {
    let prx = embedded_definitional_lexicon_prx(entry.kind)?;
    Some(lexicon_runtime_ontology(&entry.name, &entry.version, prx))
}

/// Resolve a registered definitional chat lexicon BY ITS REGISTERED NAME —
/// the `--load <name>` tier. Walks the registry (never a name table) and
/// defers to [`definitional_lexicon_runtime_ontology`], so exactly the
/// sources the residency pass selects are addressable by name, with the same
/// materialization. `None` when no definitional-lexicon entry carries the
/// name.
pub fn registered_lexicon_by_name(
    name: &str,
) -> Option<Result<RuntimeOntology, LexiconBridgeError>> {
    super::registry::data_sources()
        .iter()
        .filter(|entry| entry.name == name)
        .find_map(definitional_lexicon_runtime_ontology)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::applied::data_provisioning::registry::data_sources;

    /// Every registered definitional-lexicon entry materializes through the
    /// registry-driven dispatch, named after its source — and the registry
    /// actually carries at least one such entry (the residency pass is not
    /// vacuously green).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_registered_definitional_lexicon_materializes_by_kind() {
        let mut selected = 0usize;
        for entry in data_sources() {
            let Some(result) = definitional_lexicon_runtime_ontology(entry) else {
                continue;
            };
            selected += 1;
            let onto = result.unwrap_or_else(|e| {
                panic!("registered lexicon {} must materialize: {e}", entry.name)
            });
            assert_eq!(
                onto.id().as_str(),
                entry.name,
                "the materialized ontology is named after its registered source"
            );
        }
        assert!(
            selected > 0,
            "the registry must carry at least one definitional chat lexicon"
        );
    }

    /// The kind-keyed bundle table and the registry agree BOTH WAYS: every
    /// kind with an embedded bundle has a registered entry, and the
    /// recognizer `LegalLexicon` (plus every non-lexicon kind) is NOT
    /// selected — the definitional/recognizer split the taxonomy documents.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn selection_is_exactly_the_definitional_kinds() {
        for entry in data_sources() {
            let selected = definitional_lexicon_runtime_ontology(entry).is_some();
            assert_eq!(
                selected,
                embedded_definitional_lexicon_prx(entry.kind).is_some(),
                "residency selects exactly the kind-keyed bundle table ({})",
                entry.name
            );
            if entry.kind == SourceTaxonomyConcept::LegalLexicon {
                assert!(
                    !selected,
                    "the closed-class recognizer LegalLexicon is never a definitional chat lexicon"
                );
            }
        }
        assert!(
            registered_lexicon_by_name("no-such-registered-lexicon").is_none(),
            "an unregistered name resolves to nothing"
        );
    }
}
