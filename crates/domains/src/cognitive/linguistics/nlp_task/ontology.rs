//! NLPTask — a cited taxonomy of NLP task types, and TaskReachability — a
//! typed classification of whether a claimed task is actually usable from
//! the live chat pipeline or merely present in code.
//!
//! Built to close a measured "capability dishonesty" gap: a 2026-07 wiring
//! audit of this codebase's NLP/linguistics tree (19 modules surveyed,
//! adversarially re-verified) found several NLP capabilities that ARE
//! registered/self-described but are NOT actually reachable from a live
//! conversation turn — `self_describe()`/capability-query answers could not
//! distinguish "this concept exists in code" from "this concept can be
//! USED in a live turn." `TaskClaim` (in `super::claims`) is the relation
//! that connects a registered `SelfModel`-described component
//! (`crate::cognitive::cognition::self_model`) to the `NLPTaskConcept` it
//! claims, at a `TaskReachabilityConcept` verdict — see that module for why the
//! connection is a plain typed fact table, not a `Functor` (the claim
//! relation is many-to-many by nature: one component can claim several
//! tasks, one task can be claimed by several components).

// NLPTask — the taxonomy of NLP task types this codebase's capabilities are
// classified against.
//
// Source: Jurafsky & Martin, *Speech and Language Processing* (3rd ed.
// draft, web.stanford.edu/~jurafsky/slp3) — the concept set follows the
// book's own chapter organization (Ch.2 tokenization; Ch.17 "Sequence
// Labeling for Parts of Speech and Named Entities"; Ch.18/19 parsing;
// Ch.20 information extraction; Ch.21 semantic role labeling; Ch.22
// sentiment/connotation; Ch.23 "Coreference Resolution and Entity
// Linking"; Ch.24 discourse coherence; Ch.25 dialogue systems; App. D
// spelling/the noisy channel — the exact Shannon (1948)/Kernighan, Church
// & Gale (1990) citation pair already grounding
// crate::cognitive::linguistics::orthography::channel in this codebase,
// direct confirmation this taxonomy fits code that already exists here);
// cross-checked against the ACL Rolling Review Area Keywords
// (aclrollingreview.org/areas), the field's current working taxonomy of
// NLP research areas. J&M is an already-accepted house source — cited for
// speech-act taxonomy at pragmatics/dialogue/citings.md.
pr4xis::ontology! {
    name: "NLPTask",
    source: "Jurafsky & Martin, Speech and Language Processing (3rd ed. draft, web.stanford.edu/~jurafsky/slp3); ACL Rolling Review Area Keywords (aclrollingreview.org/areas)",

    concepts: [
        Tokenization,
        SpellingCorrection,
        PartOfSpeechTagging,
        NamedEntityRecognition,
        SequenceLabeling,
        ConstituencyParsing,
        DependencyParsing,
        RelationExtraction,
        EventExtraction,
        SemanticRoleLabeling,
        SentimentAnalysis,
        CoreferenceResolution,
        EntityLinking,
        DiscourseCoherence,
        DialogueManagement,
        MachineTranslation,
        InformationRetrieval,
        WordSenseDisambiguation,
    ],

    labels: {
        Tokenization: ("en", "Tokenization", "Splitting surface text into word/subword units (SLP3 Ch.2)."),
        SpellingCorrection: ("en", "Spelling correction", "Noisy-channel correction of a misspelled surface (SLP3 App. D; Shannon 1948; Kernighan, Church & Gale 1990)."),
        PartOfSpeechTagging: ("en", "Part-of-speech tagging", "Assigning a word its grammatical category (SLP3 Ch.17)."),
        NamedEntityRecognition: ("en", "Named entity recognition", "Identifying spans that name a person, place, organization, or other entity (SLP3 Ch.17)."),
        SequenceLabeling: ("en", "Sequence labeling", "The generalization over per-token tagging tasks (SLP3 Ch.17 title, subsuming POS tagging and NER)."),
        ConstituencyParsing: ("en", "Constituency parsing", "Recovering a phrase-structure tree over a sentence (SLP3 Ch.18)."),
        DependencyParsing: ("en", "Dependency parsing", "Recovering head-dependent relations over a sentence (SLP3 Ch.19)."),
        RelationExtraction: ("en", "Relation extraction", "Identifying a stated relation between two named entities (SLP3 Ch.20)."),
        EventExtraction: ("en", "Event extraction", "Identifying an event and its participants from text (SLP3 Ch.20)."),
        SemanticRoleLabeling: ("en", "Semantic role labeling", "Assigning predicate-argument roles (who did what to whom) (SLP3 Ch.21)."),
        SentimentAnalysis: ("en", "Sentiment analysis", "Classifying the affective stance a text expresses (SLP3 Ch.22)."),
        CoreferenceResolution: ("en", "Coreference resolution", "Grouping mentions that refer to the same discourse entity (SLP3 Ch.23)."),
        EntityLinking: ("en", "Entity linking", "Resolving a mention to a specific knowledge-base entity (SLP3 Ch.23)."),
        DiscourseCoherence: ("en", "Discourse coherence", "Recovering the rhetorical/coherence structure connecting sentences (SLP3 Ch.24)."),
        DialogueManagement: ("en", "Dialogue management", "Tracking conversational state and selecting a system act (SLP3 Ch.25)."),
        MachineTranslation: ("en", "Machine translation", "Translating text from one language to another (SLP3, MT chapter)."),
        InformationRetrieval: ("en", "Information retrieval", "Ranking or selecting documents/passages relevant to a query (SLP3, IR/QA chapters)."),
        WordSenseDisambiguation: ("en", "Word sense disambiguation", "Selecting the intended sense of an ambiguous word in context (SLP3, WSD chapter)."),
    },

    edges: [
        // SLP3 Ch.17's own title names sequence labeling as the
        // generalization over POS tagging and NER — a textually licensed
        // Subsumption, not an invented one. Canonical OBO-RO name
        // `Subsumption` (Smith et al. 2005), matching
        // `pr4xis::ontology::reasoning::catalog`'s exact match key, so this
        // edge inherits `NoCyclesOnKind`/`AntisymmetricOnKind` for free.
        (SequenceLabeling, PartOfSpeechTagging, Subsumption),
        (SequenceLabeling, NamedEntityRecognition, Subsumption),
    ],
}

// TaskReachability — is a claimed NLPTaskConcept actually usable from a
// live conversation turn, or merely present in code?
//
// Source: Grove & Chambers, "A Framework for Call Graph Construction
// Algorithms," ACM TOPLAS 23(6) (2001) — the formal distinction between
// reachable-from-a-designated-entry-point and merely-present/linked is
// exactly the gap a bare "is this registered" check erases. The five leaf
// shapes below are NOT external literature — they are this codebase's own
// five confirmed, distinct failure modes from the 2026-07 wiring audit,
// cited as this codebase's own measured taxonomy rather than dressed up
// with a borrowed paper they don't come from, in the same
// empirically-grounded-taxonomy style already established by
// docs/caregiver-corpus-status.json's MissingTerm/UnparsedKnownTerm/
// OverAnswered/PossibleMisroute classes.
pr4xis::ontology! {
    name: "TaskReachability",
    source: "Grove & Chambers, 'A Framework for Call Graph Construction Algorithms,' ACM TOPLAS 23(6) (2001) -- reachable-from-entry-points vs merely-linked, the formal distinction this classification instantiates; the five leaf shapes are this codebase's own measured taxonomy from its 2026-07 NLP-ontology wiring audit",

    concepts: [
        Reachable,
        Unclaimed,
        NotReachable,
        NoDetectorCallSite,
        CarryingTypeHasNoSlot,
        ClassifierBypassed,
        DispatcherArmIncomplete,
        OwnTestsOnly,
    ],

    labels: {
        Reachable: ("en", "Reachable", "A verdict-producing probe drove the real live entry point and observed the claimed effect."),
        Unclaimed: ("en", "Unclaimed", "No TaskClaim has been registered for this component/task pair at all -- silence, not a verdict."),
        NotReachable: ("en", "Not reachable", "A claim was checked and found NOT reachable from the live pipeline -- the parent of the five specific failure shapes below."),
        NoDetectorCallSite: ("en", "No detector call site", "Built and tested, but zero call sites from any live entry point invoke it (e.g. pragmatics::ResponseFrame::PhaticReturn -- nothing in chat detects a greeting to invoke it)."),
        CarryingTypeHasNoSlot: ("en", "Carrying type has no slot", "The runtime type that would need to carry the claimed value has no field/accessor for it (e.g. PosTag::Verb had no tense/aspect slot for TenseAspect)."),
        ClassifierBypassed: ("en", "Classifier bypassed", "A correct, unit-tested classifier exists, but the live call site uses a cruder inline mechanism instead (e.g. orthography's Channel/classify_etiology exists, but try_spelling_correction called raw Damerau-Levenshtein nearest-neighbor instead)."),
        DispatcherArmIncomplete: ("en", "Dispatcher arm incomplete", "Data + accessors exist, but the one generic reachability predicate the answer path calls is missing the match arm for this case (e.g. composed.rs::reaches() had no arms for derivation/pertainym/domain_topic/exemplifies)."),
        OwnTestsOnly: ("en", "Own tests only", "Fully built and correct, but invoked only from its own module's tests -- never from any live entry point (e.g. verbnet's defines_lens chain)."),
    },

    edges: [
        (NoDetectorCallSite, NotReachable, Specializes),
        (CarryingTypeHasNoSlot, NotReachable, Specializes),
        (ClassifierBypassed, NotReachable, Specializes),
        (DispatcherArmIncomplete, NotReachable, Specializes),
        (OwnTestsOnly, NotReachable, Specializes),
    ],
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::FinitelyGenerated;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn nlp_task_has_eighteen_concepts() {
        assert_eq!(NLPTaskConcept::variants().len(), 18);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn nlp_task_category_identity_law() {
        for obj in NLPTaskConcept::variants() {
            let id = NLPTaskCategory::identity(&obj);
            assert_eq!(id.from, obj);
            assert_eq!(id.to, obj);
            assert_eq!(id.kind, NLPTaskRelationKind::Identity);
        }
    }

    #[pr4xis::praxis_value(Deterministic, Extensible)]
    #[test]
    fn nlp_task_category_composition_with_identity() {
        let morphisms = NLPTaskCategory::morphisms();
        for m in &morphisms {
            let id_left = NLPTaskCategory::identity(&m.from);
            let id_right = NLPTaskCategory::identity(&m.to);
            let left = NLPTaskCategory::compose(&id_left, m).unwrap();
            assert_eq!(left.from, m.from);
            assert_eq!(left.to, m.to);
            let right = NLPTaskCategory::compose(m, &id_right).unwrap();
            assert_eq!(right.from, m.from);
            assert_eq!(right.to, m.to);
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn sequence_labeling_subsumes_pos_tagging_and_ner() {
        // SLP3 Ch.17's own title licenses this pair of edges.
        let morphisms = NLPTaskCategory::morphisms();
        assert!(
            morphisms
                .iter()
                .any(|m| m.from == NLPTaskConcept::SequenceLabeling
                    && m.to == NLPTaskConcept::PartOfSpeechTagging
                    && m.kind == NLPTaskRelationKind::Subsumption)
        );
        assert!(
            morphisms
                .iter()
                .any(|m| m.from == NLPTaskConcept::SequenceLabeling
                    && m.to == NLPTaskConcept::NamedEntityRecognition
                    && m.kind == NLPTaskRelationKind::Subsumption)
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn coreference_and_entity_linking_are_not_falsely_linked() {
        // SLP3 co-titles a chapter over both and ACL RR co-lists them under
        // "Discourse and Pragmatics" -- but co-occurring in a chapter/area
        // heading is not a licensed IS-A, so no edge exists between them.
        let morphisms = NLPTaskCategory::morphisms();
        assert!(!morphisms.iter().any(|m| {
            (m.from == NLPTaskConcept::CoreferenceResolution
                && m.to == NLPTaskConcept::EntityLinking)
                || (m.from == NLPTaskConcept::EntityLinking
                    && m.to == NLPTaskConcept::CoreferenceResolution)
        }));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn task_reachability_has_eight_concepts() {
        assert_eq!(TaskReachabilityConcept::variants().len(), 8);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn task_reachability_category_identity_law() {
        for obj in TaskReachabilityConcept::variants() {
            let id = TaskReachabilityCategory::identity(&obj);
            assert_eq!(id.from, obj);
            assert_eq!(id.to, obj);
            assert_eq!(id.kind, TaskReachabilityRelationKind::Identity);
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_specific_failure_shape_specializes_not_reachable() {
        let morphisms = TaskReachabilityCategory::morphisms();
        for leaf in [
            TaskReachabilityConcept::NoDetectorCallSite,
            TaskReachabilityConcept::CarryingTypeHasNoSlot,
            TaskReachabilityConcept::ClassifierBypassed,
            TaskReachabilityConcept::DispatcherArmIncomplete,
            TaskReachabilityConcept::OwnTestsOnly,
        ] {
            assert!(
                morphisms.iter().any(|m| m.from == leaf
                    && m.to == TaskReachabilityConcept::NotReachable
                    && m.kind == TaskReachabilityRelationKind::Specializes),
                "{leaf:?} must specialize NotReachable"
            );
        }
    }
}
