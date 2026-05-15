//! Smoke tests for the auto-generated SOX 1514A ontology.
//!
//! Validates the build.rs → praxis.lock → codegen pipeline end-to-end:
//! the generated `Sox1514aId` enum and `CODEGEN_DATA` static must
//! reflect the 28-term / 18-relation structure declared in
//! `praxis.lock`'s `[structural."sox_1514a@2002"]` block.

use super::{CODEGEN_DATA, Sox1514aId};
use pr4xis::category::Concept;

#[test]
fn twenty_eight_concepts() {
    assert_eq!(Sox1514aId::variants().len(), 28);
}

#[test]
fn codegen_data_entity_count_matches() {
    assert_eq!(CODEGEN_DATA.entity_count, 28);
    assert_eq!(CODEGEN_DATA.entity_labels.len(), 28);
}

#[test]
fn first_concept_is_covered_employer() {
    // Sources are sorted in praxis.lock declaration order; the first
    // term in SOX 1514A's structural block is "Covered Employer".
    assert_eq!(CODEGEN_DATA.entity_labels[0], "Covered Employer");
}

#[test]
fn relations_total_eighteen_across_kinds() {
    // 18 relations distributed across taxonomy / mereology / opposition /
    // equivalence / causation per the codegen's lossy mapping.
    let total = CODEGEN_DATA.taxonomy.len()
        + CODEGEN_DATA.mereology.len()
        + CODEGEN_DATA.opposition.len()
        + CODEGEN_DATA.equivalence.len()
        + CODEGEN_DATA.causation.len();
    assert_eq!(total, 18);
}

#[test]
fn empty_word_index_until_adjunction_codegen() {
    // SOX 1514A's structural block carries no lemmas (lemmas extraction
    // is M5 adjunction-to-English work). WORD_INDEX must therefore be
    // empty, but the symbol must still exist — the codegen emits it
    // unconditionally per the M3c WORD_INDEX-always-defined fix.
    assert_eq!(CODEGEN_DATA.word_index.len(), 0);
}

#[test]
fn lookup_returns_empty_for_any_word_pre_m5() {
    use super::lookup;
    // Before adjunction codegen, all lookups miss.
    assert_eq!(lookup("retaliation"), &[] as &[u32]);
    assert_eq!(lookup("anything"), &[] as &[u32]);
}

// ─────────────────────────────────────────────────────────────────────
//  Live `Statute` instance — high-quality tests proving praxis
//  understands SOX 1514A end-to-end.
// ─────────────────────────────────────────────────────────────────────

mod statute_runtime {
    //! Verifies the [`super::super::statute()`] accessor — `Statute`
    //! materialized from `praxis.lock`'s `[structural."sox_1514a@2002"]`
    //! block — has the structure SOX § 806 actually has. Each test
    //! either checks a property that must hold of any well-formed
    //! statute (uniqueness, endpoint-resolution) or a specific
    //! domain fact about 18 U.S.C. § 1514A traceable to the
    //! statutory text.
    //!
    //! Citation discipline: every domain-specific assertion below
    //! pins to a subsection of the statute. The lock structural
    //! block's term `name`/`definition` fields paraphrase the
    //! statute; the citations here ground the assertions in the
    //! enacted text.

    use crate::formal::meta::identifier_format::Identifier;
    use crate::social::compliance::statutes::sox_1514a::{statute, try_statute};

    fn id(curie: &str) -> Identifier {
        Identifier::curie(curie.to_string()).expect("valid CURIE")
    }

    #[test]
    fn statute_constructs_without_error() {
        // `try_statute` proves no `StatuteConstructError` is reachable
        // from the current `praxis.lock` data — every term id is a
        // valid CURIE, every relation endpoint resolves, every
        // relation kind is known.
        assert!(try_statute().is_ok());
    }

    #[test]
    fn statute_identifies_as_sox_1514a_2002() {
        let s = statute();
        assert_eq!(s.name(), "sox_1514a");
        assert_eq!(s.version(), "2002");
    }

    #[test]
    fn description_cites_18_usc_1514a() {
        // The description in praxis.lock should pin the corpus location.
        // Sarbanes–Oxley § 806 is codified at 18 U.S.C. § 1514A.
        let desc = &statute().description().text;
        assert!(
            desc.contains("18 U.S.C") || desc.contains("1514A"),
            "description should cite 18 U.S.C. § 1514A; got: {desc}"
        );
    }

    #[test]
    fn description_carries_lock_provenance() {
        // SourceTextRef::with_context should pin to the lock URI so
        // downstream tooling can trace back to the structural block.
        let ctx = statute()
            .description()
            .context_uri
            .as_deref()
            .expect("description carries context URI");
        assert_eq!(ctx, "praxis-lock://sox_1514a@2002");
    }

    #[test]
    fn term_count_is_twenty_eight() {
        // 28 terms enumerated in [structural."sox_1514a@2002".terms].
        // Matches `Sox1514aId::variants().len()` (the parallel codegen
        // path); a mismatch between codegen and runtime would be a
        // pipeline drift.
        assert_eq!(statute().terms().len(), 28);
    }

    #[test]
    fn relation_count_is_eighteen() {
        // 18 relations enumerated in
        // [structural."sox_1514a@2002".relations].
        assert_eq!(statute().relations().len(), 18);
    }

    #[test]
    fn all_term_ids_are_unique() {
        // Property: a well-formed statute never assigns the same
        // CURIE to two terms — duplicates would break term_by_id
        // lookups silently.
        let mut ids: alloc::vec::Vec<&str> = statute()
            .terms()
            .iter()
            .map(|t| t.id.value.as_str())
            .collect();
        ids.sort_unstable();
        let n = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), n, "found duplicate term CURIEs");
    }

    #[test]
    fn all_relation_endpoints_resolve_to_terms() {
        // Property: every relation's `from` and `to` is a CURIE that
        // names an existing term. `Statute::from_structural` already
        // enforces this at construction, but the test guards against
        // future drift in the validation logic.
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
    fn all_term_curies_use_sox_1514a_prefix() {
        // Property: every CURIE lives in the `sox_1514a:` namespace.
        // Mixed-namespace terms would indicate the structural block
        // is splicing in foreign concepts, which the statute itself
        // doesn't introduce.
        for t in statute().terms() {
            assert!(
                t.id.value.starts_with("sox_1514a:"),
                "term id `{}` is not in sox_1514a: namespace",
                t.id.value
            );
        }
    }

    #[test]
    fn first_term_is_covered_employer() {
        // SOX § 806(a) — "Civil action to protect against retaliation
        // in fraud cases" — leads with the definition of which
        // employers are covered. Term order in the structural block
        // mirrors the statutory text.
        let s = statute();
        let t = &s.terms()[0];
        assert_eq!(t.id.value, "sox_1514a:a");
        assert_eq!(t.name.text, "Covered Employer");
    }

    #[test]
    fn prohibition_on_retaliation_is_central() {
        // sox_1514a:a_v3 ("Prohibition on Retaliation") is the
        // gravamen of § 806 — the substantive rule that all the
        // other terms either elaborate, elect into, or condition.
        // Multiple terms `Requires` it as a prerequisite element of
        // a § 806 claim.
        let prohibition = id("sox_1514a:a_v3");
        let s = statute();
        let t = s.term_by_id(&prohibition).expect("term exists");
        assert_eq!(t.name.text, "Prohibition on Retaliation");
        // Inbound `Requires` relations: a, a_v4, a_v5 → a_v3.
        // (Plus a Composes from 1, 2; Plus an ExhaustionRequiredFor from b1.)
        let inbound_count = s.relations_to(&prohibition).count();
        assert!(
            inbound_count >= 3,
            "expected at least 3 inbound relations to Prohibition on Retaliation; got {inbound_count}"
        );
    }

    #[test]
    fn protected_activity_categories_compose_into_prohibition() {
        // § 806(a)(1) ("providing information about violations") and
        // § 806(a)(2) ("participating in proceedings") are the two
        // statutory categories of protected activity. Both Compose
        // into the prohibition (a_v3) — they're constituent elements
        // of a § 806 claim's actus reus.
        let s = statute();
        let category_1 = id("sox_1514a:1");
        let category_2 = id("sox_1514a:2");
        let prohibition_curie = "sox_1514a:a_v3";

        let cat_1_composes_into_prohibition = s.relations_from(&category_1).any(|r| {
            r.to.value == prohibition_curie
                && matches!(
                    r.relation,
                    crate::social::judicial::ontology::RelationType::Composes { .. }
                )
        });
        let cat_2_composes_into_prohibition = s.relations_from(&category_2).any(|r| {
            r.to.value == prohibition_curie
                && matches!(
                    r.relation,
                    crate::social::judicial::ontology::RelationType::Composes { .. }
                )
        });
        assert!(
            cat_1_composes_into_prohibition,
            "§ 806(a)(1) protected activity must Compose into Prohibition on Retaliation"
        );
        assert!(
            cat_2_composes_into_prohibition,
            "§ 806(a)(2) protected activity must Compose into Prohibition on Retaliation"
        );
    }

    #[test]
    fn protected_activity_subcategories_are_alternatives_to_parent() {
        // § 806(a)(1)(A)–(C) enumerates the three reporting channels
        // for protected information: federal agency, congressional
        // committee, supervisor. Each is an AlternativeTo the parent
        // category — satisfying any one suffices for protection.
        let s = statute();
        let parent_curie = "sox_1514a:1";
        for sub in &["sox_1514a:1a", "sox_1514a:1b", "sox_1514a:1c"] {
            let sub_id = id(sub);
            let is_alt = s.relations_from(&sub_id).any(|r| {
                r.to.value == parent_curie
                    && matches!(
                        r.relation,
                        crate::social::judicial::ontology::RelationType::AlternativeTo
                    )
            });
            assert!(
                is_alt,
                "{sub} should be AlternativeTo {parent_curie} per § 806(a)(1)(A)–(C)"
            );
        }
    }

    #[test]
    fn osha_filing_is_exhaustion_requirement() {
        // § 806(b)(1) requires the complainant to file with the
        // Secretary of Labor (OSHA) before suing in federal court —
        // a classic administrative-exhaustion requirement. The lock
        // models this as: b1 ExhaustionRequiredFor a_v3.
        let s = statute();
        let b1 = id("sox_1514a:b1");
        let prohibition_curie = "sox_1514a:a_v3";
        let is_exhaustion = s.relations_from(&b1).any(|r| {
            r.to.value == prohibition_curie
                && matches!(
                    r.relation,
                    crate::social::judicial::ontology::RelationType::ExhaustionRequiredFor
                )
        });
        assert!(
            is_exhaustion,
            "§ 806(b)(1) OSHA filing must be ExhaustionRequiredFor Prohibition on Retaliation"
        );
    }

    #[test]
    fn confidentiality_provisions_are_safe_harbors() {
        // § 806(e)(1)-(2) provides safe harbors: rights of
        // employees and invalidity of predispute arbitration
        // agreements both SafeHarborFor c1 (the substantive remedy
        // hook). e1 and e2 in the structural block model this.
        let s = statute();
        let c1_curie = "sox_1514a:c1";
        for sh in &["sox_1514a:e1", "sox_1514a:e2"] {
            let sh_id = id(sh);
            let is_safe_harbor = s.relations_from(&sh_id).any(|r| {
                r.to.value == c1_curie
                    && matches!(
                        r.relation,
                        crate::social::judicial::ontology::RelationType::SafeHarborFor
                    )
            });
            assert!(
                is_safe_harbor,
                "{sh} should be SafeHarborFor {c1_curie} per § 806(e)(1)-(2)"
            );
        }
    }

    #[test]
    fn term_by_curie_finds_existing_term() {
        let t = statute()
            .term_by_curie("sox_1514a:a")
            .expect("sox_1514a:a exists");
        assert_eq!(t.name.text, "Covered Employer");
    }

    #[test]
    fn term_by_curie_returns_none_for_unknown() {
        assert!(statute().term_by_curie("sox_1514a:nonexistent").is_none());
        assert!(statute().term_by_curie("other_statute:a").is_none());
    }

    #[test]
    fn term_by_id_and_term_by_curie_agree() {
        // Property: the two lookup paths must converge for every
        // valid term id.
        let s = statute();
        for t in s.terms() {
            let by_id = s.term_by_id(&t.id);
            let by_curie = s.term_by_curie(t.id.value.as_str());
            assert!(by_id.is_some());
            assert!(by_curie.is_some());
            assert_eq!(by_id.map(|x| &x.id), by_curie.map(|x| &x.id));
        }
    }

    #[test]
    fn relations_from_returns_only_outgoing() {
        let s = statute();
        let prohibition = id("sox_1514a:a_v3");
        for r in s.relations_from(&prohibition) {
            assert_eq!(r.from.value, "sox_1514a:a_v3");
        }
    }

    #[test]
    fn relations_to_returns_only_incoming() {
        let s = statute();
        let prohibition = id("sox_1514a:a_v3");
        for r in s.relations_to(&prohibition) {
            assert_eq!(r.to.value, "sox_1514a:a_v3");
        }
    }

    #[test]
    fn relation_iteration_is_partition_consistent() {
        // Property: every relation in `relations()` appears in either
        // `relations_from(r.from)` or `relations_to(r.to)` (in fact
        // both). No silent dropping or aliasing.
        let s = statute();
        for r in s.relations() {
            let in_from = s.relations_from(&r.from).any(|rr| core::ptr::eq(rr, r));
            let in_to = s.relations_to(&r.to).any(|rr| core::ptr::eq(rr, r));
            assert!(in_from && in_to);
        }
    }

    #[test]
    fn statute_is_idempotent() {
        // OnceLock caching: the same `&'static Statute` reference is
        // returned every call.
        let a = statute() as *const _;
        let b = statute() as *const _;
        assert!(core::ptr::eq(a, b));
    }

    #[test]
    fn every_term_carries_a_non_empty_name_and_definition() {
        // Sanity: the structural block must populate name and
        // definition for every term. An empty name would silently
        // degrade `term_by_curie` UX; an empty definition would
        // signal extraction failure.
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
    fn every_term_text_carries_lock_provenance() {
        // Every SourceTextRef constructed by `from_structural` must
        // pin to the lock URI — proves the constructor wired
        // `with_context` consistently.
        for t in statute().terms() {
            assert_eq!(
                t.name.context_uri.as_deref(),
                Some("praxis-lock://sox_1514a@2002")
            );
            assert_eq!(
                t.definition.context_uri.as_deref(),
                Some("praxis-lock://sox_1514a@2002")
            );
        }
    }

    #[test]
    fn predispute_arbitration_invalidity_term_present() {
        // § 806(e)(2): "No predispute arbitration agreement shall be
        // valid or enforceable, if the agreement requires arbitration
        // of a dispute arising under this section." Added by
        // Dodd-Frank § 922 in 2010 — must be present in the modern
        // (2002+) ontology version.
        let t = statute()
            .term_by_curie("sox_1514a:e2")
            .expect("e2 term present");
        assert!(
            t.definition.text.to_lowercase().contains("arbitration"),
            "e2 definition should mention arbitration; got: {}",
            t.definition.text
        );
    }

    #[test]
    fn codegen_concept_count_agrees_with_statute_runtime() {
        // Cross-path coherence: the compile-time codegen path
        // (Sox1514aId concept enum) and the runtime path
        // (Statute::terms) MUST agree on term count. Drift between
        // them indicates either a stale `OUT_DIR/...codegen.rs` or a
        // bug in `Statute::from_structural`.
        use crate::social::compliance::statutes::sox_1514a::Sox1514aId;
        use pr4xis::category::Concept;
        assert_eq!(Sox1514aId::variants().len(), statute().terms().len());
    }
}
