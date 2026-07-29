//! Conditional-rule-frame ontology — the Governance/Slot/Filler/Verdict
//! category and its axioms. See `mod.rs` for the value types
//! (`ConditionalRule`/`AppliedElement`/`FactValue`/`SlotState`) and full
//! literature.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "ConditionalRuleFrame",
    source: "Fillmore, C. J. (1976) Annals NYAS 280(1); Fillmore, C. J. (1982) \
             Frame Semantics, Linguistics in the Morning Calm; Minsky, M. (1974) \
             MIT-AI Memo 306; Bobrow, D. G., Kaplan, R. M., Norman, D. A., \
             Thompson, H. & Winograd, T. (1977) GUS, Artificial Intelligence 8(2)",

    concepts: [Governance, Slot, Filler, Verdict],

    labels: {
        Governance: ("en", "Governance",
            "Minsky's FRAME: the fixed procedural structure -- here, the cited LegalTerm governing a class of question."),
        Slot: ("en", "Slot",
            "Fillmore's FRAME ELEMENT: a named fact (an EvidenceRequirement) the frame's evaluation depends on."),
        Filler: ("en", "Filler",
            "GUS: the specific value that fills a Slot once known (a FactValue) -- private to this asker, never part of the loaded rule."),
        Verdict: ("en", "Verdict",
            "The rule's Applicability -- reachable only once every REQUIRED Slot is filled."),
    },

    edges: [
        (Governance, Slot, Requires),
        (Slot, Filler, FilledBy),
        (Slot, Verdict, Determines),
    ],
}

impl Ontology for ConditionalRuleFrameOntology {
    type Cat = ConditionalRuleFrameCategory;
    type Qual = NoQuality;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(VerdictIsReachedOnlyThroughSlots));
        axioms.push(Box::new(ConditionalRuleGroundedInBindingLaw));
        axioms.push(Box::new(
            ApplicabilityNamesExactlyTheUnfilledRequiredElements,
        ));
        axioms
    }
}

/// No domain [`Quality`] is needed over this frame's structural concepts —
/// the classification work is done by the already-registered `HartRule`
/// quality on `SourceTaxonomyConcept`, reused directly rather than
/// re-derived here.
#[derive(Debug, Clone)]
pub struct NoQuality;

impl Quality for NoQuality {
    type Individual = ConditionalRuleFrameConcept;
    type Value = ();

    fn get(&self, _individual: &Self::Individual) -> Option<()> {
        None
    }
}

/// A `Verdict` is reachable ONLY by way of a `Slot` — there is no direct
/// `Governance`-to-`Verdict` edge in the checked category. Fillmore (1982):
/// a frame's interpretability is CONSTITUTED by its elements, not separable
/// from them — an evaluation cannot bypass the frame elements it is defined
/// over.
#[derive(Debug, Clone)]
pub struct VerdictIsReachedOnlyThroughSlots;

impl Axiom for VerdictIsReachedOnlyThroughSlots {
    fn verify(&self) -> Verdict {
        let m = ConditionalRuleFrameCategory::morphisms();
        let no_direct_edge = !m.iter().any(|e| {
            e.source() == ConditionalRuleFrameConcept::Governance
                && e.target() == ConditionalRuleFrameConcept::Verdict
        });
        let requires_slot = m.iter().any(|e| {
            e.source() == ConditionalRuleFrameConcept::Governance
                && e.target() == ConditionalRuleFrameConcept::Slot
        });
        let slot_determines_verdict = m.iter().any(|e| {
            e.source() == ConditionalRuleFrameConcept::Slot
                && e.target() == ConditionalRuleFrameConcept::Verdict
        });
        if no_direct_edge && requires_slot && slot_determines_verdict {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "VerdictIsReachedOnlyThroughSlots",
        "A ConditionalRule's Verdict (Applicability) is derivable only by way of its Slots (AppliedElements), never bypassing them",
        "Fillmore, C. J. (1982) 'Frame Semantics', Linguistics in the Morning Calm, 111-137"
    );
}
pr4xis::register_axiom!(
    VerdictIsReachedOnlyThroughSlots,
    "Fillmore, C. J. (1982) 'Frame Semantics', Linguistics in the Morning Calm"
);

/// A [`super::ConditionalRule`] is only ever constructed over BINDING LAW —
/// text with the force and effect of law: a statute or constitutional article
/// (Hart 1961 §V-VI primary rule), a procedural rule, a legislative AGENCY
/// REGULATION (force of law under its enabling act), or case-law precedent
/// (binds via stare decisis). NEVER a non-binding secondary GLOSS — a lexicon
/// DEFINITION (Black's, a bare statutory-definition recital) that describes a
/// term without itself binding. That binding-vs-gloss line is exactly the
/// `LegalCorpus`-vs-`Lexicon` split in `formal::meta::source_taxonomy`: every
/// binding source sits in the `LegalCorpus` subtree, every gloss in the
/// `Lexicon` subtree — checked directly against that ALREADY-LOADED taxonomy
/// (`is_legal_corpus`/`is_lexicon`), not a re-derived classification. This
/// admits BOTH the Hart-primary `UsFederalStatute` (the asset-transfer rule's
/// source) and the Hart-secondary-but-force-of-law `Regulation` (the 42
/// C.F.R. § 441.302(a)(6) critical-incident rule's source), while still
/// refusing a `LegalLexicon` gloss. A regulation is Hart-secondary yet binding
/// — the Hart primary/secondary axis is orthogonal to the binding/gloss axis
/// this rule-of-recognition floor cares about.
#[derive(Debug, Clone)]
pub struct ConditionalRuleGroundedInBindingLaw;

impl Axiom for ConditionalRuleGroundedInBindingLaw {
    fn verify(&self) -> Verdict {
        use crate::formal::meta::source_taxonomy::ontology::{
            SourceTaxonomyConcept as C, is_legal_corpus, is_lexicon,
        };
        // Both source_kinds a ConditionalRule is actually constructed over
        // today -- UsFederalStatute (asset-transfer) and Regulation
        // (critical-incident) -- must be binding legal text (LegalCorpus
        // subtree); the canonical gloss, LegalLexicon, must be a Lexicon and
        // NOT LegalCorpus, so it is refused.
        let statute_binding = is_legal_corpus(C::UsFederalStatute);
        let regulation_binding = is_legal_corpus(C::Regulation);
        let lexicon_refused = is_lexicon(C::LegalLexicon) && !is_legal_corpus(C::LegalLexicon);
        if statute_binding && regulation_binding && lexicon_refused {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ConditionalRuleGroundedInBindingLaw",
        "ConditionalRule.source_kind must be binding law (the LegalCorpus subtree -- statute, regulation, procedural rule, constitutional article, or case-law precedent), never a non-binding Lexicon gloss; UsFederalStatute and Regulation are LegalCorpus, LegalLexicon is not",
        "Hart, H. L. A. (1961) The Concept of Law, Oxford, §V-VI"
    );
}
pr4xis::register_axiom!(
    ConditionalRuleGroundedInBindingLaw,
    "Hart, H. L. A. (1961) The Concept of Law, Oxford, §V-VI"
);

/// A `ConditionalRule` with exactly one Required, Unfilled element resolves
/// `applicability()` to `Indeterminate`, naming exactly that element — the
/// Sergot-reduction decidability property this whole capability exists to
/// deliver.
#[derive(Debug, Clone)]
pub struct ApplicabilityNamesExactlyTheUnfilledRequiredElements;

impl Axiom for ApplicabilityNamesExactlyTheUnfilledRequiredElements {
    fn verify(&self) -> Verdict {
        use super::{Applicability, ConditionalRule, SlotState};
        use crate::formal::meta::identifier_format::Identifier;
        use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
        use crate::social::judicial::ontology::{
            EvidenceRequirement, EvidenceType, LegalTerm, RequirementLevel,
        };
        use crate::social::judicial::source_text::SourceTextRef;

        let req = EvidenceRequirement {
            field: SourceTextRef::new("monthly income"),
            field_type: EvidenceType::Currency,
            required: RequirementLevel::Required,
            description: None,
        };
        let term = LegalTerm {
            id: Identifier::curie("witness:income_test").expect("valid CURIE"),
            name: SourceTextRef::new("witness rule"),
            definition: SourceTextRef::new("qualifies if income is below the limit"),
            source_text: None,
            subsection: None,
            required_evidence: vec![req.clone()],
            obligations: vec![],
            deadlines: vec![],
            rights: vec![],
            remedies: vec![],
            burdens: vec![],
            exceptions: vec![],
        };
        let rule = ConditionalRule::from_term(term, SourceTaxonomyConcept::UsFederalStatute);
        match rule.applicability() {
            Applicability::Indeterminate { missing } => {
                let ok = missing.len() == 1
                    && missing.head.requirement == req
                    && matches!(missing.head.state, SlotState::Unfilled);
                if ok {
                    Ok(Box::new(SimpleProof::new(self.meta())))
                } else {
                    Err(Box::new(SimpleCounterexample::new(self.meta())))
                }
            }
            _ => Err(Box::new(SimpleCounterexample::new(self.meta()))),
        }
    }

    pr4xis::axiom_meta!(
        "ApplicabilityNamesExactlyTheUnfilledRequiredElements",
        "a ConditionalRule whose sole element is Required and Unfilled resolves applicability() to Indeterminate{missing} naming exactly that element",
        "Sergot, M., Sadri, F., Kowalski, R., Kriwaczek, F., Hammond, P. & Cory, T. (1986) 'The British Nationality Act as a Logic Program', CACM 29(5), 370-386; Minsky, M. (1974) MIT-AI Memo 306"
    );
}
pr4xis::register_axiom!(
    ApplicabilityNamesExactlyTheUnfilledRequiredElements,
    "Sergot et al. (1986) CACM 29(5); Minsky (1974) MIT-AI Memo 306"
);
