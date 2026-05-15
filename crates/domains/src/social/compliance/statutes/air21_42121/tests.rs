//! Tests for the auto-generated AIR21 § 42121 ontology and runtime.

use super::{Air2142121Id, CODEGEN_DATA, statute, try_statute};
use crate::formal::meta::identifier_format::Identifier;
use crate::social::judicial::ontology::RelationType;
use pr4xis::category::Concept;

// =============================================================================
// Codegen smoke tests
// =============================================================================

#[test]
fn seventeen_concepts() {
    assert_eq!(Air2142121Id::variants().len(), 17);
}

#[test]
fn codegen_data_entity_count_matches() {
    assert_eq!(CODEGEN_DATA.entity_count, 17);
    assert_eq!(CODEGEN_DATA.entity_labels.len(), 17);
}

#[test]
fn first_concept_is_discrimination_prohibited() {
    assert_eq!(CODEGEN_DATA.entity_labels[0], "Discrimination Prohibited");
}

#[test]
fn relations_total_twenty_one_across_kinds() {
    let total = CODEGEN_DATA.taxonomy.len()
        + CODEGEN_DATA.mereology.len()
        + CODEGEN_DATA.opposition.len()
        + CODEGEN_DATA.equivalence.len()
        + CODEGEN_DATA.causation.len();
    assert_eq!(total, 21);
}

#[test]
fn empty_word_index_until_adjunction_codegen() {
    assert_eq!(CODEGEN_DATA.word_index.len(), 0);
}

// =============================================================================
// Statute runtime — high-quality tests
// =============================================================================

mod statute_runtime {
    use super::*;

    fn id(curie: &str) -> Identifier {
        Identifier::curie(curie.to_string()).expect("valid CURIE")
    }

    #[test]
    fn statute_constructs_without_error() {
        assert!(try_statute().is_ok());
    }

    #[test]
    fn statute_identifies_as_air21_42121_2010() {
        let s = statute();
        assert_eq!(s.name(), "air21_42121");
        assert_eq!(s.version(), "2010");
    }

    #[test]
    fn description_cites_49_usc_42121() {
        let desc = &statute().description().text;
        assert!(
            desc.contains("49 U.S.C") || desc.contains("42121"),
            "description should cite 49 U.S.C. § 42121; got: {desc}"
        );
    }

    #[test]
    fn description_carries_lock_provenance() {
        let ctx = statute()
            .description()
            .context_uri
            .as_deref()
            .expect("description carries context URI");
        assert_eq!(ctx, "praxis-lock://air21_42121@2010");
    }

    #[test]
    fn term_count_is_seventeen() {
        assert_eq!(statute().terms().len(), 17);
    }

    #[test]
    fn relation_count_is_twenty_one() {
        assert_eq!(statute().relations().len(), 21);
    }

    #[test]
    fn all_term_ids_are_unique() {
        let mut ids: alloc::vec::Vec<&str> = statute()
            .terms()
            .iter()
            .map(|t| t.id.value.as_str())
            .collect();
        ids.sort_unstable();
        let n = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), n);
    }

    #[test]
    fn all_relation_endpoints_resolve_to_terms() {
        let s = statute();
        let term_ids: hashbrown::HashSet<&str> =
            s.terms().iter().map(|t| t.id.value.as_str()).collect();
        for (i, r) in s.relations().iter().enumerate() {
            assert!(
                term_ids.contains(r.from.value.as_str()),
                "relation #{i}: from `{}` is not a term",
                r.from.value
            );
            assert!(
                term_ids.contains(r.to.value.as_str()),
                "relation #{i}: to `{}` is not a term",
                r.to.value
            );
        }
    }

    #[test]
    fn all_term_curies_use_air21_prefix() {
        for t in statute().terms() {
            assert!(
                t.id.value.starts_with("air21_42121:"),
                "term id `{}` is not in air21_42121: namespace",
                t.id.value
            );
        }
    }

    // ── Domain-specific facts traceable to the statute ───────────────────

    #[test]
    fn discrimination_prohibited_is_substantive_root() {
        let a = statute()
            .term_by_curie("air21_42121:a")
            .expect("term a exists");
        assert_eq!(a.name.text, "Discrimination Prohibited");
        // Four protected-activity sub-terms Compose into a.
        let inbound_compose: usize = statute()
            .relations_to(&id("air21_42121:a"))
            .filter(|r| matches!(r.relation, RelationType::Composes { .. }))
            .count();
        assert_eq!(
            inbound_compose, 4,
            "four protected activities Compose into a"
        );
    }

    #[test]
    fn four_protected_activities_present() {
        for sub in &[
            "air21_42121:a_1",
            "air21_42121:a_2",
            "air21_42121:a_3",
            "air21_42121:a_4",
        ] {
            assert!(
                statute().term_by_curie(sub).is_some(),
                "{sub} should be present as a protected activity"
            );
        }
    }

    #[test]
    fn burden_framework_has_four_clauses() {
        let b2b = id("air21_42121:b2b");
        let clauses: usize = statute()
            .relations_to(&b2b)
            .filter(|r| matches!(r.relation, RelationType::Composes { .. }))
            .count();
        assert_eq!(clauses, 4, "§ 42121(b)(2)(B) must Compose from 4 clauses");
    }

    #[test]
    fn investigation_gate_defense_is_affirmative_defense_to_prima_facie() {
        let b2b_ii = id("air21_42121:b2b_ii");
        let is_defense = statute().relations_from(&b2b_ii).any(|r| {
            r.to.value == "air21_42121:b2b_i"
                && matches!(r.relation, RelationType::AffirmativeDefenseTo)
        });
        assert!(
            is_defense,
            "clause (ii) must be AffirmativeDefenseTo clause (i)"
        );
    }

    #[test]
    fn merits_defense_is_affirmative_defense_to_merits_showing() {
        let b2b_iv = id("air21_42121:b2b_iv");
        let is_defense = statute().relations_from(&b2b_iv).any(|r| {
            r.to.value == "air21_42121:b2b_iii"
                && matches!(r.relation, RelationType::AffirmativeDefenseTo)
        });
        assert!(
            is_defense,
            "clause (iv) must be AffirmativeDefenseTo clause (iii)"
        );
    }

    #[test]
    fn investigation_gate_exhaustion_for_merits() {
        let b2b_i = id("air21_42121:b2b_i");
        let is_exhaustion = statute().relations_from(&b2b_i).any(|r| {
            r.to.value == "air21_42121:b2b_iii"
                && matches!(r.relation, RelationType::ExhaustionRequiredFor)
        });
        assert!(
            is_exhaustion,
            "clause (i) prima facie gate must be ExhaustionRequiredFor clause (iii) merits"
        );
    }

    #[test]
    fn procedural_sequence_b1_b2_b3_holds() {
        // b1 (complaint) Precedes b2 (investigation) Precedes b3 (final order)
        let s = statute();
        let b1 = id("air21_42121:b1");
        let b2 = id("air21_42121:b2");
        let b3 = id("air21_42121:b3");

        let b1_precedes_b2 = s.relations_from(&b1).any(|r| {
            r.to.value == "air21_42121:b2" && matches!(r.relation, RelationType::Precedes { .. })
        });
        let b2_precedes_b3 = s.relations_from(&b2).any(|r| {
            r.to.value == "air21_42121:b3" && matches!(r.relation, RelationType::Precedes { .. })
        });
        assert!(b1_precedes_b2, "complaint must Precede investigation");
        assert!(b2_precedes_b3, "investigation must Precede final order");
        // Ensure both terms exist for the test invariant.
        assert!(s.term_by_id(&b3).is_some());
    }

    #[test]
    fn de_novo_court_review_is_alternative_to_enforcement() {
        let b5 = id("air21_42121:b5");
        let is_alt = statute().relations_from(&b5).any(|r| {
            r.to.value == "air21_42121:b4" && matches!(r.relation, RelationType::AlternativeTo)
        });
        assert!(
            is_alt,
            "de novo court review (b5) must be AlternativeTo enforcement action (b4)"
        );
    }

    #[test]
    fn complaint_filing_requires_substantive_prohibition() {
        let b1 = id("air21_42121:b1");
        let requires_a = statute()
            .relations_from(&b1)
            .any(|r| r.to.value == "air21_42121:a" && matches!(r.relation, RelationType::Requires));
        assert!(
            requires_a,
            "complaint filing must Require substantive prohibition"
        );
    }

    #[test]
    fn statute_is_idempotent() {
        let a = statute() as *const _;
        let b = statute() as *const _;
        assert!(core::ptr::eq(a, b));
    }

    #[test]
    fn every_term_carries_non_empty_name_and_definition() {
        for t in statute().terms() {
            assert!(
                !t.name.text.is_empty(),
                "term {} has empty name",
                t.id.value
            );
            assert!(
                !t.definition.text.is_empty(),
                "term {} has empty definition",
                t.id.value
            );
        }
    }

    #[test]
    fn every_term_carries_lock_provenance() {
        for t in statute().terms() {
            assert_eq!(
                t.name.context_uri.as_deref(),
                Some("praxis-lock://air21_42121@2010")
            );
            assert_eq!(
                t.definition.context_uri.as_deref(),
                Some("praxis-lock://air21_42121@2010")
            );
        }
    }

    #[test]
    fn codegen_concept_count_agrees_with_statute_runtime() {
        assert_eq!(Air2142121Id::variants().len(), statute().terms().len());
    }
}
