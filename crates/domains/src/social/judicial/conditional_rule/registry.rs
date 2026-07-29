//! A small, hand-authored registry of real `ConditionalRule`s — the same
//! methodology Sergot, Sadri, Kowalski, Kriwaczek, Hammond & Cory (1986)
//! "The British Nationality Act as a Logic Program" (*CACM* 29(5):370-386)
//! itself used: a real statute, reduced by a human to named, evaluable
//! conditions, not an automated NLP extraction (no such extractor exists in
//! this codebase — `statute_structure::term_extractor`'s auto-derived
//! `StructuralTerm`s carry no evidence-requirement semantics; see
//! `LegalTerm.required_evidence`'s own doc history). Every entry here is
//! verified against the real on-disk USC text
//! (`crates/domains/data/legal/uscode/usc_title_42/`), never invented.
//!
//! This module also supplies [`rule_for_predicate_and_object`] — the ONE
//! matching function `LexicalReasoner::conditional_rule_for_predicate`
//! delegates to (task #28). It gates on BOTH the predicate ("eligible
//! for") AND the object sharing real content words with the matched
//! rule's OWN cited evidence text — never on predicate alone. The
//! [`registry`] holds more than one rule (the family-Medicaid
//! asset-transfer penalty and the HCBS critical-incident reporting
//! requirement); predicate-only matching would confidently attach SOME
//! rule to ANY "eligible for X" question (Medicare, TennCare, a driver's
//! license — anything), which is confidently WRONG far more often than it
//! is right; the whole point of this capability (task #20's Safety Exhibit
//! Test framing) is refusing rather than guessing. Each rule is scored only
//! against ITS OWN `required_evidence` vocabulary, never an invented
//! trigger-word list — no rule text, no possible match — and the rule with
//! the greatest overlap wins.

#[allow(unused_imports)]
use alloc::collections::BTreeSet;
#[allow(unused_imports)]
use alloc::{vec, vec::Vec};

use crate::cognitive::linguistics::english::LexicalReasoner;
use crate::formal::meta::identifier_format::Identifier;
use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
use crate::social::judicial::citation::PinpointCite;
use crate::social::judicial::citation::ontology::PinpointCitationConcept;
use crate::social::judicial::ontology::{
    EvidenceRequirement, EvidenceType, LegalTerm, RequirementLevel,
};
use crate::social::judicial::source_text::SourceTextRef;

use super::ConditionalRule;

/// 42 U.S.C. § 1396p(c)(1)(A) — the Medicaid asset-transfer penalty period.
///
/// Verified against the real on-disk USLM text
/// (`/us/usc/t42/s1396p/c/1/A`): "the State plan must provide that if an
/// institutionalized individual or the spouse of such an individual (or, at
/// the option of a State, a noninstitutionalized individual or the spouse
/// of such an individual) disposes of assets for less than fair market
/// value on or after the look-back date specified in subparagraph (B)(i),
/// the individual is ineligible for medical assistance for services
/// described in subparagraph (C)(i) ... during the period beginning on the
/// date specified in subparagraph (D) and equal to the number of months
/// specified in subparagraph (E)."
///
/// Reduced to its ONE `Boolean`-typed core threshold fact — did a
/// disqualifying transfer occur — per this module's Sergot-et-al.
/// methodology. The penalty-DURATION calculation ((c)(1)(B)/(D)/(E), a
/// date/month arithmetic problem) is deliberately NOT modeled: this
/// codebase has no cited numeral→`Quantity` bridge yet (Track 1.4 of the
/// standing turing-benchmark plan), so a `Currency`/`Duration`-typed
/// element here would have no session-side slot-filling that could ever
/// satisfy it — an honest scope boundary, not an oversight.
pub fn asset_transfer_penalty_rule() -> ConditionalRule {
    let term = LegalTerm {
        id: Identifier::curie("usc42:1396p_c_1_a_asset_transfer_penalty").expect("valid CURIE"),
        name: SourceTextRef::new("Medicaid asset-transfer penalty period"),
        definition: SourceTextRef::new(
            "if an institutionalized individual (or spouse) disposes of assets for less \
             than fair market value on or after the applicable look-back date, the \
             individual is ineligible for nursing-facility and related Medicaid services \
             for a period computed from that transfer",
        ),
        source_text: Some(SourceTextRef::new("42 U.S.C. § 1396p(c)(1)(A)")),
        subsection: Some(
            PinpointCite::new().push(PinpointCitationConcept::Subsection, "(c)(1)(A)"),
        ),
        required_evidence: vec![EvidenceRequirement {
            field: SourceTextRef::new(
                "whether you (or your spouse) disposed of assets for less than fair \
                 market value on or after the look-back date",
            ),
            field_type: EvidenceType::Boolean,
            required: RequirementLevel::Required,
            description: Some(SourceTextRef::new(
                "42 U.S.C. § 1396p(c)(1)(A) — the transfer trigger; (c)(1)(B) defines \
                 the look-back date itself",
            )),
        }],
        obligations: vec![],
        deadlines: vec![],
        rights: vec![],
        remedies: vec![],
        burdens: vec![],
        exceptions: vec![],
    };
    ConditionalRule::from_term(term, SourceTaxonomyConcept::UsFederalStatute)
}

/// 42 C.F.R. § 441.302(a)(6)(i)(A) — the HCBS "critical incident" definition
/// the 2024 CMS "Ensuring Access to Medicaid Services" final rule (CMS-2442-F,
/// 89 FR 40542, May 10 2024) added to the State's incident-management-system
/// assurance. A Track-2 (direct-care workforce / HCBS compliance) rule, the
/// sibling of the family-Medicaid [`asset_transfer_penalty_rule`] above.
///
/// The (a)(6) assurance reads, VERBATIM (eCFR current, cross-checked against
/// Cornell LII 42 CFR 441.302 — this Title-42 CFR text is NOT among the
/// repo's on-disk USLM sources, which are the U.S. *Code*, so the source of
/// record here is the eCFR/GPO regulation itself, not a local file):
///
/// > Assurance that the State operates and maintains an incident management
/// > system that identifies, reports, triages, investigates, resolves,
/// > tracks, and trends critical incidents.
///
/// and (a)(6)(i)(A), the definitional provision this rule reduces, reads
/// VERBATIM:
///
/// > (A) Define critical incident to include, at a minimum—
/// > (1) Verbal, physical, sexual, psychological, or emotional abuse;
/// > (2) Neglect;
/// > (3) Exploitation including financial exploitation;
/// > (4) Misuse or unauthorized use of restrictive interventions or seclusion;
/// > (5) A medication error resulting in a telephone call to, or a
/// >     consultation with, a poison control center, an emergency department
/// >     visit, an urgent care visit, a hospitalization, or death; or
/// > (6) An unexplained or unanticipated death, including but not limited to
/// >     a death caused by abuse or neglect;
///
/// The "at a minimum" language is load-bearing: the six enumerated categories
/// are a FEDERAL FLOOR, not a ceiling — a State's own HCBS program may define
/// ADDITIONAL reportable critical incidents (the residual arm). So deciding
/// "is X a reportable critical incident" genuinely conditions on TWO facts,
/// modeled here as the two `required_evidence` elements below:
///   1. `Boolean` — does X fall within one of the six enumerated FEDERAL
///      categories (§ 441.302(a)(6)(i)(A)(1)-(6))?
///   2. `Concept` — WHICH State's HCBS program applies, since the residual
///      set beyond the federal floor is State-defined ("at a minimum").
///
/// Both are `Required`. This is a CONJUNCTIVE OVER-APPROXIMATION of a rule
/// that is really DISJUNCTIVE ("X is reportable if it is in the federal floor
/// OR the applicable State adds it"): `ConditionalRule::applicability` is
/// Sergot et al. (1986)'s purely conjunctive reduction, with no disjunctive
/// arm yet, so listing both facts as Required makes a freshly-loaded rule ask
/// for BOTH the category-membership AND the State rather than guess either —
/// the safe direction for an abstention-first system (GUS "ask, don't
/// guess"), never a wrong answer. Faithfully modeling the OR (state residual
/// moot once federal-floor membership is established) needs a disjunctive
/// reduction this engine does not have — the same honest scope-boundary the
/// asset-transfer rule documents for the penalty-DURATION arithmetic it
/// declines to model.
///
/// `source_kind` is [`SourceTaxonomyConcept::Regulation`], not
/// `UsFederalStatute`: this is a legislative agency rule (force and effect of
/// law under 42 U.S.C. § 1302 rulemaking authority), and the enumerated
/// reportable-category definition exists ONLY at the regulation level — no
/// U.S. Code section states it. `Regulation` is Hart-Secondary but is squarely
/// within the `LegalCorpus` subtree (binding legal text), which is the floor
/// [`super::ontology::ConditionalRuleGroundedInBindingLaw`] actually enforces
/// — never a non-binding `Lexicon` gloss.
pub fn critical_incident_reporting_rule() -> ConditionalRule {
    let term = LegalTerm {
        id: Identifier::curie("cfr42:441_302_a_6_critical_incident_reporting")
            .expect("valid CURIE"),
        name: SourceTextRef::new("HCBS critical-incident reporting requirement"),
        definition: SourceTextRef::new(
            "the State's incident-management-system assurance must define critical incident \
             to include, at a minimum: verbal, physical, sexual, psychological, or emotional \
             abuse; neglect; exploitation including financial exploitation; misuse or \
             unauthorized use of restrictive interventions or seclusion; a medication error \
             resulting in a poison-control call, an emergency-department, urgent-care, or \
             hospital visit, or death; or an unexplained or unanticipated death — and a State \
             may define additional reportable critical incidents beyond this federal minimum",
        ),
        source_text: Some(SourceTextRef::new("42 C.F.R. § 441.302(a)(6)")),
        subsection: Some(
            PinpointCite::new().push(PinpointCitationConcept::Subsection, "(a)(6)(i)(A)"),
        ),
        required_evidence: vec![
            EvidenceRequirement {
                field: SourceTextRef::new(
                    "whether the incident falls within one of the six critical-incident \
                     categories the federal minimum enumerates (abuse, neglect, exploitation, \
                     misuse of restrictive interventions or seclusion, a medication error, or \
                     an unexplained death)",
                ),
                field_type: EvidenceType::Boolean,
                required: RequirementLevel::Required,
                description: Some(SourceTextRef::new(
                    "42 C.F.R. § 441.302(a)(6)(i)(A)(1)-(6) — the enumerated federal floor",
                )),
            },
            EvidenceRequirement {
                // Field text is deliberately worded WITHOUT the bare tokens
                // "program"/"state" (only the possessive "State's"): the
                // matcher (`rule_for_predicate_and_object`) scores an "eligible
                // for X" object against this very text, and the caregiver
                // corpus's own eligibility questions ("eligible for the PCA
                // program", "the Self-Determination Program", …) would
                // otherwise collide on "program" and be pulled from an honest
                // abstain into this rule. Distinctive critical-incident
                // vocabulary only — no generic-eligibility word — keeps this
                // rule confined to genuine critical-incident objects.
                field: SourceTextRef::new(
                    "which State's HCBS critical-incident definition applies, since each \
                     jurisdiction may define reportable critical incidents beyond the six \
                     federal categories",
                ),
                field_type: EvidenceType::Concept,
                required: RequirementLevel::Required,
                description: Some(SourceTextRef::new(
                    "42 C.F.R. § 441.302(a)(6)(i)(A) — the \"at a minimum\" residual: the \
                     applicable State's program determines the reportable set beyond the \
                     federal floor",
                )),
            },
        ],
        obligations: vec![],
        deadlines: vec![],
        rights: vec![],
        remedies: vec![],
        burdens: vec![],
        exceptions: vec![],
    };
    ConditionalRule::from_term(term, SourceTaxonomyConcept::Regulation)
}

/// The lowercased, whitespace-tokenized CONTENT words of `text` — function
/// words (via `en.is_function_word`, the SAME loaded-lexicon check the
/// tokenizer itself uses) are dropped. No lemmatization: an object phrase
/// must share an exact word FORM with the rule's own text to match (a
/// real, honestly-documented limitation — "asset" will not match
/// "assets" — not a hidden one).
fn content_words(en: &dyn LexicalReasoner, text: &str) -> BTreeSet<alloc::string::String> {
    text.split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|w| !w.is_empty() && !en.is_function_word(w))
        .collect()
}

/// `LexicalReasoner::conditional_rule_for_predicate`'s real implementation
/// (task #28), delegated to by `English`/`ComposedReasoner`. `predicate`
/// must be exactly `"eligible for"` (the ONE predicate `predicate_lexicon`
/// currently recognizes as a rule-governance surface); `object` is the
/// question's named object surface (e.g. `extract_entity_name` on the
/// first argument). Returns `Some` only when `object`'s content words
/// overlap with a registry rule's OWN cited evidence text — see the module
/// doc for why this is required, not optional.
pub fn rule_for_predicate_and_object(
    en: &dyn LexicalReasoner,
    predicate: &str,
    object: &str,
) -> Option<ConditionalRule> {
    if predicate != "eligible for" {
        return None;
    }
    let object_words = content_words(en, object);
    if object_words.is_empty() {
        return None;
    }
    // Attach the registry rule whose OWN cited evidence vocabulary the object
    // shares the MOST content words with; zero overlap never matches (refuse
    // rather than guess), and a tie is broken deterministically by iteration
    // order. With a MULTI-entry registry, predicate-only matching would
    // confidently attach SOME rule to ANY "eligible for X" question; scoring
    // each rule against its OWN evidence text keeps every rule confined to
    // objects that name its subject matter — the same per-rule-vocabulary
    // gate the single-entry version used, generalized. The registry's rules
    // carry disjoint vocabularies (asset-transfer vs. critical-incident), so
    // a with-overlap tie between two DIFFERENT rules does not arise here.
    registry()
        .into_iter()
        .filter_map(|rule| {
            let rule_words: BTreeSet<alloc::string::String> = rule
                .elements
                .iter()
                .flat_map(|el| content_words(en, &el.requirement.field.text))
                .collect();
            let overlap = object_words.intersection(&rule_words).count();
            (overlap > 0).then_some((overlap, rule))
        })
        .max_by_key(|(overlap, _)| *overlap)
        .map(|(_, rule)| rule)
}

/// Every hand-authored [`ConditionalRule`] this registry can attach to an
/// "eligible for X" question — the family-Medicaid asset-transfer penalty
/// (Track 1) and the HCBS critical-incident reporting requirement (Track 2).
/// Each is gated on its OWN cited evidence vocabulary by
/// [`rule_for_predicate_and_object`], so adding an entry never widens another
/// entry's matches.
pub fn registry() -> Vec<ConditionalRule> {
    vec![
        asset_transfer_penalty_rule(),
        critical_incident_reporting_rule(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The registry entry must be internally well-formed: a real citation,
    /// exactly one Required element, and the element's field text must be
    /// the same string `Applicability::Indeterminate` will later name as
    /// missing (so a caller can round-trip it without re-deriving text).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn asset_transfer_penalty_rule_is_well_formed() {
        let rule = asset_transfer_penalty_rule();
        assert_eq!(
            rule.term.source_text.as_ref().map(|s| s.text.as_str()),
            Some("42 U.S.C. § 1396p(c)(1)(A)")
        );
        assert_eq!(rule.elements.len(), 1);
        assert_eq!(
            rule.elements[0].requirement.field_type,
            EvidenceType::Boolean
        );
        assert_eq!(
            rule.elements[0].requirement.required,
            RequirementLevel::Required
        );
    }

    /// A freshly-loaded rule (every element `Unfilled`) is `Indeterminate`,
    /// naming exactly the one Boolean element — the GUS "ask, don't guess"
    /// contract over REAL data, not a hand-built test fixture.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn asset_transfer_penalty_rule_starts_indeterminate() {
        use crate::social::judicial::conditional_rule::Applicability;

        let rule = asset_transfer_penalty_rule();
        match rule.applicability() {
            Applicability::Indeterminate { missing } => {
                assert_eq!(missing.len(), 1);
                assert_eq!(
                    missing.head.requirement.field.text,
                    rule.elements[0].requirement.field.text
                );
            }
            other => panic!("expected Indeterminate on a freshly-loaded rule, got {other:?}"),
        }
    }

    /// An object sharing a content word with the rule's OWN cited evidence
    /// text ("assets" appears verbatim in `asset_transfer_penalty_rule`'s
    /// evidence field) matches; the SAME predicate over an unrelated real
    /// program name (the shape of the real 4617-question corpus's "eligible
    /// for X" questions — Medicare, Medicaid, TennCare, none of which
    /// mention asset transfers) does not.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn matches_only_when_the_object_shares_the_rules_own_vocabulary() {
        use crate::cognitive::linguistics::english::English;

        let english = English::sample_static();
        let matched =
            rule_for_predicate_and_object(english, "eligible for", "the transfer of assets")
                .expect("an object naming the rule's own evidence vocabulary must match");
        assert_eq!(
            matched.term.id.value(),
            "usc42:1396p_c_1_a_asset_transfer_penalty",
            "an asset-transfer object must resolve to the asset-transfer rule, not another entry"
        );
        assert!(
            rule_for_predicate_and_object(english, "eligible for", "Medicare home health services")
                .is_none(),
            "an unrelated real program name must NOT match — the corpus's own \
             'eligible for X' questions must keep abstaining, not get a wrong rule"
        );
        assert!(
            rule_for_predicate_and_object(english, "eligible for", "TennCare").is_none(),
            "TennCare shares no vocabulary with any registry rule"
        );
    }

    /// The Track-2 critical-incident rule is internally well-formed: the real
    /// C.F.R. citation, exactly the two `Required` evidence elements (a
    /// `Boolean` federal-floor test and a `Concept` State residual), and a
    /// `Regulation` source_kind (not a statute — the enumerated categories
    /// exist only at the regulation level).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn critical_incident_reporting_rule_is_well_formed() {
        let rule = critical_incident_reporting_rule();
        assert_eq!(
            rule.term.source_text.as_ref().map(|s| s.text.as_str()),
            Some("42 C.F.R. § 441.302(a)(6)")
        );
        assert_eq!(rule.source_kind, SourceTaxonomyConcept::Regulation);
        assert_eq!(rule.elements.len(), 2);
        assert_eq!(
            rule.elements[0].requirement.field_type,
            EvidenceType::Boolean
        );
        assert_eq!(
            rule.elements[1].requirement.field_type,
            EvidenceType::Concept
        );
        assert!(
            rule.elements
                .iter()
                .all(|el| el.requirement.required == RequirementLevel::Required),
            "both the federal-floor and State-residual facts are Required"
        );
    }

    /// A freshly-loaded critical-incident rule is `Indeterminate`, naming BOTH
    /// unfilled Required facts (the conjunctive over-ask of a disjunctive rule
    /// — see the function's own doc), with the `Boolean` federal-floor test
    /// first so the multi-turn slot-filler can answer it with yes/no before
    /// the `Concept` State residual.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn critical_incident_reporting_rule_starts_indeterminate() {
        use crate::social::judicial::conditional_rule::Applicability;

        let rule = critical_incident_reporting_rule();
        match rule.applicability() {
            Applicability::Indeterminate { missing } => {
                assert_eq!(missing.len(), 2);
                assert_eq!(
                    missing.head.requirement.field_type,
                    EvidenceType::Boolean,
                    "the Boolean federal-floor test must be the first slot asked"
                );
            }
            other => panic!("expected Indeterminate on a freshly-loaded rule, got {other:?}"),
        }
    }

    /// Each registry object routes to ITS OWN rule and never the sibling: an
    /// asset-transfer object never resolves to the critical-incident rule, and
    /// a critical-incident object never resolves to the asset-transfer rule.
    /// The disjoint-vocabulary invariant the multi-entry registry relies on.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn registry_dispatches_each_object_to_its_own_rule() {
        use crate::cognitive::linguistics::english::English;

        let english = English::sample_static();
        let incident =
            rule_for_predicate_and_object(english, "eligible for", "critical incident reporting")
                .expect("a critical-incident object must match the critical-incident rule");
        assert_eq!(
            incident.term.id.value(),
            "cfr42:441_302_a_6_critical_incident_reporting"
        );
        let assets =
            rule_for_predicate_and_object(english, "eligible for", "the transfer of assets")
                .expect("an asset-transfer object must match the asset-transfer rule");
        assert_eq!(
            assets.term.id.value(),
            "usc42:1396p_c_1_a_asset_transfer_penalty"
        );
    }

    /// A different predicate never matches, regardless of the object.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_different_predicate_never_matches() {
        use crate::cognitive::linguistics::english::English;

        let english = English::sample_static();
        assert!(
            rule_for_predicate_and_object(english, "responsible for", "the transfer of assets")
                .is_none()
        );
    }
}
