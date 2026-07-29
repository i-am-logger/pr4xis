//! `TaskClaim` — the fact table connecting a registered component to the
//! [`super::ontology::NLPTaskConcept`] it claims, at a
//! [`super::ontology::TaskReachabilityConcept`] verdict.
//!
//! NOT a `Functor`: which NLP tasks a component implements is many-to-many
//! by nature (one component can claim several tasks; one task can be
//! claimed by several components), so a single-valued `map_object` could
//! only misrepresent it — a plain, typed fact table is the honest shape,
//! the same categorical judgment `self_model`'s own `Component`↔`Capability`
//! edges already make (`HasCapability`/`EnabledBy` are relational, not
//! functorial, for the identical reason).
//!
//! Keyed at CONCEPT grain (`component` + `unit`), not ontology grain: a
//! single registered ontology can carry two independently-verdicted units at
//! once — the shape this key exists to make possible, illustrated historically
//! by the `Response` ontology's `ResponseFrame::AssertKnowledge` (reachable
//! from `crate::chat`'s answer path) and `ResponseFrame::PhaticReturn`
//! (originally `NoDetectorCallSite`, closed by task #4 — both units are
//! `Reachable` now, see `known_claims`) — an ontology-grain key could not have
//! held both distinct truths for one component while they differed.
//!
//! Verdicts are produced by ordinary, tagged Rust test functions (the same
//! `#[pr4xis::praxis_value]` pattern `crates/praxis-corpus-tests`' capability
//! ratchet already uses), NOT by a runtime-introspected registry: every
//! `linkme::distributed_slice` in `pr4xis::ontology::registry` — the
//! pattern a registry-based `ReachabilityCheck` mechanism would have to
//! join — is `#[cfg(not(target_arch = "wasm32"))]`, empty on wasm32. Since
//! this project's actual deployed chat surface is wasm
//! (`crates/wasm` depends on `pr4xis-chat`), a runtime registry would be
//! silently inert on the one target where the answer honestly matters. The
//! verdict TABLE (a plain data fact, computed once natively and committed)
//! is what ships — as ordinary `.prx`-loadable data, read identically by
//! both native and wasm builds, exactly like this codebase's other
//! data-not-code projections.
//!
//! Task #11 (the NLP-task completeness gate) adds the second half of the
//! coverage story: [`ComponentOptOut`]/[`known_opt_outs`] for components
//! that are NLP-relevant-adjacent but structurally cannot carry a task claim
//! (they ARE the taxonomy, or are infrastructure, or the taxonomy itself has
//! no matching task type yet) — see [`OptOutReason`] and
//! [`nlp_relevant_domain_prefixes`] for the scoping predicate the gate
//! (`crates/praxis-corpus-tests/tests/nlp_task_reachability.rs`) diffs
//! against.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::meta::{ConceptName, OntologyName};

use super::ontology::{NLPTaskConcept, TaskReachabilityConcept};

/// The Rust module-path domain PREFIXES the task #11 completeness gate
/// treats as "NLP-relevant" — the module subtree that actually implements
/// language processing, plus the one dialogue-systems formalization that
/// sits outside it.
///
/// `"cognitive.linguistics"` is this codebase's entire linguistics
/// implementation tree (`crates/domains/src/cognitive/linguistics/`):
/// tokenization, morphology, orthography, the lexicon, pragmatics,
/// semantics, WordNet — the exact scope Jurafsky & Martin's *Speech and
/// Language Processing* covers and [`super::ontology`]'s own `NLPTaskOntology`
/// is derived from.
///
/// `"formal.information.dialogue"` sits OUTSIDE that subtree but is
/// unambiguously NLP-relevant: `formal::information::dialogue`'s own module
/// doc calls itself "the science of conversation" and cites Austin (1962),
/// Traum (1994), Clark (1996), Ginzburg (2012), Stalnaker (2002) — the SAME
/// literature family [`NLPTaskConcept::DialogueManagement`] cites (SLP3
/// Ch.25).
///
/// A [`pr4xis::ontology::Vocabulary`]'s [`domain()`](pr4xis::ontology::Vocabulary::domain)
/// falls under one of these iff it equals the prefix or starts with
/// `"{prefix}."` — see [`domain_is_nlp_relevant`].
pub fn nlp_relevant_domain_prefixes() -> [&'static str; 2] {
    ["cognitive.linguistics", "formal.information.dialogue"]
}

/// Whether a [`pr4xis::ontology::Vocabulary::domain`] string falls under one
/// of [`nlp_relevant_domain_prefixes`] — the completeness gate's scoping
/// predicate, factored out so both the gate itself
/// (`crates/praxis-corpus-tests/tests/nlp_task_reachability.rs`) and this
/// module's own tests read the identical rule.
pub fn domain_is_nlp_relevant(domain: &str) -> bool {
    nlp_relevant_domain_prefixes().iter().any(|prefix| {
        // Exact match, or a proper sub-domain (guards against a false
        // segment match like "cognitive.linguistics2").
        domain == *prefix
            || domain
                .strip_prefix(prefix)
                .is_some_and(|rest| rest.starts_with('.'))
    })
}

/// Why a registered, NLP-relevant-adjacent component is DELIBERATELY
/// excluded from the claim requirement — task #11's completeness gate
/// requires every component in [`nlp_relevant_domain_prefixes`]'s scope to
/// have either a [`TaskClaim`] (this module) or a [`ComponentOptOut`] (never
/// silently neither). The three leaf reasons are this codebase's own
/// measured categories from the 2026-07 completeness sweep — the same
/// "codebase's own measured taxonomy" precedent
/// [`TaskReachabilityConcept`]'s five failure shapes already establish (see
/// `super::ontology`'s module doc) — grounded as follows:
///
/// - [`IsMeta`](Self::IsMeta): Nelson & Narens (1990) *Metamemory: A
///   Theoretical Framework*, In Bower (ed.) — the monitoring/object-level
///   distinction: a component that IS the meta-level vocabulary describing
///   NLP tasks (or their reachability) cannot also be an object-level task
///   implementation without circularity, exactly as `constitution-gate.sh`
///   excludes its own `constitution_coverage` meta-test from the set of
///   guarantee witnesses it counts.
/// - [`Infrastructure`](Self::Infrastructure): Parnas (1972) "On the
///   Criteria to Be Used in Decomposing Systems into Modules," CACM 15(12)
///   — information-hiding modules: a substrate module other claimed
///   components are built ON TOP OF is not itself a task-performing unit.
/// - [`NoMatchingTaskType`](Self::NoMatchingTaskType): Jurafsky & Martin,
///   *Speech and Language Processing* — the SAME source `NLPTaskConcept`
///   itself is derived from covers more capabilities (e.g. natural-language
///   generation) than the current 18-concept enumeration represents; a
///   component implementing one of those uncovered capabilities cannot be
///   claimed without either mis-attributing it to an ill-fitting task or
///   first extending `NLPTaskOntology` (out of scope for this gate).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptOutReason {
    /// This component IS the NLP-task (or task-reachability) taxonomy
    /// itself, not a unit that implements one of the tasks it describes.
    IsMeta,
    /// This component is cross-cutting linguistic infrastructure (a formal
    /// substrate other claimed components are built on), not itself a
    /// task-performing unit.
    Infrastructure,
    /// This component implements a real NLP capability, but no
    /// [`NLPTaskConcept`] variant covers it yet — a taxonomy gap in
    /// `NLPTaskOntology`, not a wiring gap in this component.
    NoMatchingTaskType,
}

/// Free-text justification for a [`ComponentOptOut`] — WHY this specific
/// component gets this [`OptOutReason`], beyond the closed reason category.
/// A typed wrapper (not a bare `&str`) so the note stays queryable exactly
/// like every other structural field here (mirrors [`TaskClaim`]'s own
/// `evidence: OntologyName` discipline).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptOutNote(pub &'static str);

/// One fact: `component` is deliberately excluded from the claim
/// requirement, for `reason`, with `note` explaining the specifics — the
/// "(b)" half of task #11's completeness gate (every NLP-relevant component
/// has either a [`TaskClaim`] or a `ComponentOptOut`, never silently
/// neither).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentOptOut {
    /// The registered component being opted out — the EXACT name
    /// `describe_knowledge_base()` (or `pr4xis_chat::loaded_ontologies`)
    /// registers it under, same discipline as [`TaskClaim::component`].
    pub component: OntologyName,
    pub reason: OptOutReason,
    pub note: OptOutNote,
}

impl ComponentOptOut {
    pub fn new(component: &'static str, reason: OptOutReason, note: &'static str) -> Self {
        Self {
            component: OntologyName::new_static(component),
            reason,
            note: OptOutNote(note),
        }
    }
}

/// The committed table of deliberate opt-outs — every NLP-relevant
/// component this session's task #11 sweep confirmed does NOT correspond to
/// a claimable NLP-task-implementing unit, with why. Read by the same
/// completeness gate that reads [`known_claims`]
/// (`crates/praxis-corpus-tests/tests/nlp_task_reachability.rs`).
pub fn known_opt_outs() -> Vec<ComponentOptOut> {
    vec![
        ComponentOptOut::new(
            "NLPTaskOntology",
            OptOutReason::IsMeta,
            "this IS the NLP-task taxonomy the completeness gate checks components against",
        ),
        ComponentOptOut::new(
            "TaskReachabilityOntology",
            OptOutReason::IsMeta,
            "this IS the reachability-verdict taxonomy TaskClaim.reachability is typed over",
        ),
        ComponentOptOut::new(
            "LemonOntology",
            OptOutReason::Infrastructure,
            "Ontolex-Lemon (W3C Lexicon Model for Ontologies) -- represents lexicon-entry \
             structure for building/reading lexical resources; not itself invoked to process \
             a live utterance (0 call sites in crates/chat/src/lib.rs, confirmed by direct \
             grep of the 2026-07 sweep)",
        ),
        ComponentOptOut::new(
            "PipelineOntology",
            OptOutReason::Infrastructure,
            "the Parse-dashv-Generate adjunction (de Groote ACG 2001; Lambek & Scott 1986) -- \
             the category-theoretic bridge relating parsing/generation as adjoint functors, \
             consumed by other claimed components rather than performing a task itself",
        ),
        ComponentOptOut::new(
            "WordNetOntology",
            OptOutReason::Infrastructure,
            "the taxonomy of WordNet's own relation/synset kinds (hypernym, meronym, ...) -- \
             the schema the already-claimed \"English (WordNet)\"/composed::reaches gap \
             (WordSenseDisambiguation, DispatcherArmIncomplete) is typed against, not a \
             separate task-performing unit",
        ),
        ComponentOptOut::new(
            "MontagueOntology",
            OptOutReason::NoMatchingTaskType,
            "Montague (1973) compositional/formal semantics -- reachable in practice via \
             PipelineStep::INTERPRET's trace attribution (trace_functors.rs), but \
             NLPTaskConcept has no \"compositional semantics\" / \"semantic parsing\" variant \
             to claim it against yet",
        ),
        ComponentOptOut::new(
            "ProductionOntology",
            OptOutReason::NoMatchingTaskType,
            "Levelt (1989) Conceptualizer/Formulator/Articulator generation pipeline -- SLP3 \
             covers NLG, but NLPTaskConcept has no NaturalLanguageGeneration variant to claim \
             it against yet",
        ),
        ComponentOptOut::new(
            "NlgOntology",
            OptOutReason::NoMatchingTaskType,
            "Reiter & Dale (2000) NLG pipeline (content determination / microplanning / \
             realization) -- same taxonomy gap as ProductionOntology: no NaturalLanguageGeneration \
             NLPTaskConcept variant exists yet",
        ),
    ]
}

/// One fact: `component`'s `unit` claims `task`, verified to `reachability`
/// by the test named `evidence`.
///
/// `evidence: OntologyName` — not a bare `&'static str` — names the stable,
/// re-runnable check that produced this verdict, the same "typed name
/// carries identity" discipline `pr4xis::ontology::registry::axiom_by_name`
/// already documents for axiom rebinding (`OntologyName: PartialEq<str>`
/// decides the match; it never unwraps to a bare `String ==`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskClaim {
    /// The registered component (ontology) this unit belongs to.
    pub component: OntologyName,
    /// The specific unit within `component` making the claim — a variant
    /// name, function name, or other stable sub-identifier (e.g.
    /// `"ResponseFrame::PhaticReturn"`), typed so it is never confused with
    /// a bare string comparison.
    pub unit: ConceptName,
    /// Which NLP task `unit` claims to implement.
    pub task: NLPTaskConcept,
    /// Whether that claim actually holds, and if not, which of the five
    /// measured failure shapes it is.
    pub reachability: TaskReachabilityConcept,
    /// The stable name of the check/test that produced this verdict.
    pub evidence: OntologyName,
}

impl TaskClaim {
    pub fn new(
        component: &'static str,
        unit: &'static str,
        task: NLPTaskConcept,
        reachability: TaskReachabilityConcept,
        evidence: &'static str,
    ) -> Self {
        Self {
            component: OntologyName::new_static(component),
            unit: ConceptName::new_static(unit),
            task,
            reachability,
            evidence: OntologyName::new_static(evidence),
        }
    }

    /// Is this claim actually usable from a live conversation turn?
    pub fn is_reachable(&self) -> bool {
        self.reachability == TaskReachabilityConcept::Reachable
    }
}

/// The committed table of known task claims — originally this session's
/// four confirmed wiring-audit gaps, broadened by task #11 (the NLP-task
/// completeness gate) into the FULL claim table every NLP-relevant
/// registered component's coverage is checked against: a component either
/// has an entry here (verified `Reachable`, a specific `NotReachable` shape,
/// or honestly `Unclaimed`) or an entry in [`known_opt_outs`]. The SINGLE
/// source both
/// `crates/praxis-corpus-tests/tests/nlp_task_reachability.rs` (which
/// independently reverifies each verdict by driving the real live entry
/// point) and the live chat answer surface
/// (`crate::cognitive::cognition::self_model` via `SelfModelInstance::
/// with_task_claims`) read — never duplicated between them. Compiles (and
/// is read) identically on native and wasm32: this is a plain data table,
/// not a registry query.
///
/// `component` is the EXACT name `describe_knowledge_base()` registers a
/// component under — confirmed by direct probe of the live component list
/// (`ontology! { name: "X", .. }` registers as `"XOntology"`, e.g. the
/// `Response` ontology's own `name:` field registers as `"ResponseOntology"`,
/// not `"Response"` — a real naming bug this table shipped with once and
/// corrected here after probing the actual 219-component live list). Two
/// claims have NO matching live component and are honestly left that way
/// rather than force-fit to a plausible-looking wrong name:
/// `Token::tense` lives in the `Text` ontology (`text/ontology.rs`,
/// registers as `"TextOntology"`), not a standalone `Token` component;
/// `defines_lens` lives in `social::judicial::statute_structure::grounding`
/// (confirmed via direct file search — NOT under `verbnet/`, which has no
/// `ontology!` block of its own at all), a module that registers no
/// self-model component — so this claim currently cannot surface in the
/// live capability-query prose (`crate::answer_self_referential`) even
/// though it is real and verified; it still appears in the committed
/// status JSON and the corpus reverification tests.
///
/// `ResponseFrame::PhaticReturn`'s original `NoDetectorCallSite` verdict
/// (this table's own flagship gap example) was closed by task #4: `crate::
/// chat::is_phatic`/`is_phatic_interjection` now detect a loaded phatic
/// interjection (or an all-phatic-and-otherwise-content-free utterance) on
/// every live turn and dispatch through `answer_phatic`, reusing this exact
/// `jakobson_function_of_interjection` classifier chain — no longer
/// reachable "only from its own tests."
pub fn known_claims() -> Vec<TaskClaim> {
    vec![
        TaskClaim::new(
            "TextOntology",
            "Token::tense",
            NLPTaskConcept::PartOfSpeechTagging,
            TaskReachabilityConcept::CarryingTypeHasNoSlot,
            "tense_accessor_exists_but_no_producer_populates_it",
        ),
        TaskClaim::new(
            "ChannelOntology",
            "channel::classify_etiology",
            NLPTaskConcept::SpellingCorrection,
            TaskReachabilityConcept::ClassifierBypassed,
            "spelling_correction_bypasses_the_noisy_channel_classifier",
        ),
        TaskClaim::new(
            "English (WordNet)",
            "composed::reaches",
            NLPTaskConcept::WordSenseDisambiguation,
            TaskReachabilityConcept::DispatcherArmIncomplete,
            "composed_reaches_has_no_arm_for_derivation",
        ),
        TaskClaim::new(
            "VerbNet",
            "grounding::defines_lens",
            NLPTaskConcept::InformationRetrieval,
            TaskReachabilityConcept::OwnTestsOnly,
            "defines_lens_is_reachable_only_from_its_own_tests",
        ),
        // ---------------------------------------------------------------
        // Task #11 completeness sweep (2026-07) additions below — every
        // remaining component under `nlp_relevant_domain_prefixes()` that
        // isn't in `known_opt_outs()` gets an entry here, verified or
        // honestly `Unclaimed`.
        // ---------------------------------------------------------------
        TaskClaim::new(
            "LambekOntology",
            "tokenize::tokenize + reduce::chart_reduce (live parse path)",
            NLPTaskConcept::Tokenization,
            TaskReachabilityConcept::Reachable,
            "lambek_tokenize_and_parse_are_reachable_from_a_live_turn",
        ),
        TaskClaim::new(
            "Lexicon",
            "lexicon::pos::PosTag (via Language::lexical_lookup)",
            NLPTaskConcept::PartOfSpeechTagging,
            TaskReachabilityConcept::Reachable,
            "lexicon_pos_tag_drives_live_predicate_routing",
        ),
        TaskClaim::new(
            "Spelling Errors",
            "distance::closest_matches",
            NLPTaskConcept::SpellingCorrection,
            TaskReachabilityConcept::ClassifierBypassed,
            // Same evidence as ChannelOntology above: try_spelling_correction
            // calls distance::closest_matches (raw edit distance), never the
            // conceptually-typed SpellingErrorConcept/ErrorEtiology classifier
            // this component registers.
            "spelling_correction_bypasses_the_noisy_channel_classifier",
        ),
        TaskClaim::new(
            "Tense & Aspect",
            "morphology::tense::TenseAspect (via Token::tense)",
            NLPTaskConcept::PartOfSpeechTagging,
            TaskReachabilityConcept::CarryingTypeHasNoSlot,
            // Same root cause as TextOntology/Token::tense above: nothing
            // populates a live token's tense field, so a TenseAspect value
            // is never actually assigned during live processing either.
            "tense_accessor_exists_but_no_producer_populates_it",
        ),
        TaskClaim::new(
            "ResponseOntology",
            "ResponseFrame::AssertKnowledge",
            NLPTaskConcept::DialogueManagement,
            TaskReachabilityConcept::Reachable,
            "response_assert_knowledge_is_reachable_from_a_live_turn",
        ),
        TaskClaim::new(
            "ResponseOntology",
            "ResponseFrame::PhaticReturn",
            NLPTaskConcept::DialogueManagement,
            TaskReachabilityConcept::Reachable,
            // Task #4 closed this gap; the SAME corpus test that reverifies
            // it (nlp_task_reachability.rs) is this claim's evidence.
            "phatic_return_has_a_live_detector",
        ),
        // The seven entries below are honestly `Unclaimed` (`super::
        // ontology::TaskReachabilityConcept::Unclaimed`: "no verdict has
        // been registered ... silence, not a verdict" — used here exactly
        // as specified, as an EXPLICIT acknowledgment rather than a missing
        // row): each component is confidently NLP-relevant and its closest
        // NLPTaskConcept is a defensible literature-grounded fit (cited per
        // entry), but reachability from the live chat entry point has not
        // been individually driven-and-verified the way the Reachable/
        // NotReachable-shape entries above were. Per task #11's own scope
        // ("report gaps rather than fixing dozens of individual capability
        // claims"), these are reported as explicit gaps, not silently
        // absent from the table.
        TaskClaim::new(
            "DiscourseOntology",
            "discourse::ontology (RST/coherence relations)",
            NLPTaskConcept::DiscourseCoherence,
            TaskReachabilityConcept::Unclaimed,
            "nlp_relevant_components_pending_reachability_triage_task11_sweep",
        ),
        TaskClaim::new(
            "FragmentOntology",
            "fragment::ontology (non-sentential utterance resolution)",
            // Fernandez & Ginzburg fragment resolution is a dialogue-systems
            // capability (QUD/context-dependent interpretation) per J&M
            // Ch.25 -- DialogueManagement is the closest existing fit.
            NLPTaskConcept::DialogueManagement,
            TaskReachabilityConcept::Unclaimed,
            "nlp_relevant_components_pending_reachability_triage_task11_sweep",
        ),
        TaskClaim::new(
            "GroundingOntology",
            "pragmatics::grounding::ontology (Clark & Schaefer contribution model)",
            NLPTaskConcept::DialogueManagement,
            TaskReachabilityConcept::Unclaimed,
            "nlp_relevant_components_pending_reachability_triage_task11_sweep",
        ),
        TaskClaim::new(
            "PlanningOntology",
            "pragmatics::planning::ontology (speech-act planning)",
            NLPTaskConcept::DialogueManagement,
            TaskReachabilityConcept::Unclaimed,
            "nlp_relevant_components_pending_reachability_triage_task11_sweep",
        ),
        TaskClaim::new(
            "ReferenceOntology",
            "pragmatics::reference (DRT + Centering theory)",
            NLPTaskConcept::CoreferenceResolution,
            TaskReachabilityConcept::Unclaimed,
            "nlp_relevant_components_pending_reachability_triage_task11_sweep",
        ),
        TaskClaim::new(
            "DialogueOntology",
            "dialogue::ontology (Austin/Traum/Clark/Ginzburg dialogue model)",
            NLPTaskConcept::DialogueManagement,
            TaskReachabilityConcept::Unclaimed,
            "nlp_relevant_components_pending_reachability_triage_task11_sweep",
        ),
        TaskClaim::new(
            "DialogueGroundingOntology",
            "dialogue::grounding (grounding-state finite state machine)",
            NLPTaskConcept::DialogueManagement,
            TaskReachabilityConcept::Unclaimed,
            "nlp_relevant_components_pending_reachability_triage_task11_sweep",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_reachable_claim_reports_reachable() {
        let claim = TaskClaim::new(
            "Response",
            "ResponseFrame::AssertKnowledge",
            NLPTaskConcept::DialogueManagement,
            TaskReachabilityConcept::Reachable,
            "assert_knowledge_is_dispatched_from_a_resolved_question",
        );
        assert!(claim.is_reachable());
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_not_reachable_claim_reports_not_reachable() {
        let claim = TaskClaim::new(
            "Response",
            "ResponseFrame::PhaticReturn",
            NLPTaskConcept::DialogueManagement,
            TaskReachabilityConcept::NoDetectorCallSite,
            "phatic_return_has_no_live_detector",
        );
        assert!(!claim.is_reachable());
        assert_eq!(
            claim.reachability,
            TaskReachabilityConcept::NoDetectorCallSite
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn two_units_of_the_same_component_carry_independent_verdicts() {
        // The flagship case this concept-grain key exists to make possible:
        // one ontology (`Response`), two units, two different verdicts.
        let assert_knowledge = TaskClaim::new(
            "Response",
            "ResponseFrame::AssertKnowledge",
            NLPTaskConcept::DialogueManagement,
            TaskReachabilityConcept::Reachable,
            "assert_knowledge_is_dispatched_from_a_resolved_question",
        );
        let phatic_return = TaskClaim::new(
            "Response",
            "ResponseFrame::PhaticReturn",
            NLPTaskConcept::DialogueManagement,
            TaskReachabilityConcept::NoDetectorCallSite,
            "phatic_return_has_no_live_detector",
        );
        assert_eq!(assert_knowledge.component, phatic_return.component);
        assert_ne!(assert_knowledge.unit, phatic_return.unit);
        assert_ne!(assert_knowledge.reachability, phatic_return.reachability);
        assert!(assert_knowledge.is_reachable());
        assert!(!phatic_return.is_reachable());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn known_claims_is_non_empty() {
        // `crates/praxis-corpus-tests/tests/nlp_task_reachability.rs` is the
        // full independent reverification (each claim driven against its
        // real live entry point, including the Reachable ones); this is
        // just this crate's own sanity guard that the shared table itself
        // is well-formed.
        //
        // Before task #11 this table only ever tracked confirmed GAPS from
        // the original wiring audit, so a Reachable verdict here used to
        // signal staleness ("do not just eyeball the code, prove it").
        // Task #11 broadened `known_claims()` into the full NLP-task
        // completeness table every registered component's claim-or-opt-out
        // status is read from, so it now also carries CONFIRMED-working
        // entries (`ResponseOntology`/`AssertKnowledge`, `LambekOntology`'s
        // tokenize+parse path, `Lexicon`'s live PosTag routing, ...) —
        // Reachable verdicts are an expected, individually-evidenced part
        // of the table now, not a red flag.
        let claims = known_claims();
        assert!(!claims.is_empty());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn known_opt_outs_is_non_empty_and_disjoint_from_known_claims() {
        let claims = known_claims();
        let opt_outs = known_opt_outs();
        assert!(!opt_outs.is_empty());
        for out in &opt_outs {
            assert!(
                !claims.iter().any(|c| c.component == out.component),
                "{:?} appears in BOTH known_claims() and known_opt_outs() -- a \
                 component is either claimed or opted out, never both",
                out.component
            );
        }
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn every_opt_out_note_is_non_empty() {
        // A `ComponentOptOut` with an empty note would satisfy the type
        // system while being exactly as silent as no entry at all.
        for out in known_opt_outs() {
            assert!(
                !out.note.0.is_empty(),
                "{:?} opted out with an empty note",
                out.component
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn nlp_relevant_domain_prefixes_are_recognized() {
        assert!(domain_is_nlp_relevant("cognitive.linguistics"));
        assert!(domain_is_nlp_relevant("cognitive.linguistics.lambek"));
        assert!(domain_is_nlp_relevant("formal.information.dialogue"));
        assert!(domain_is_nlp_relevant(
            "formal.information.dialogue.grounding"
        ));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn domains_outside_the_nlp_prefixes_are_not_relevant() {
        assert!(!domain_is_nlp_relevant("applied.navigation.gnss"));
        assert!(!domain_is_nlp_relevant("natural.biomedical.chemistry"));
        // A near-miss segment must not false-positive (e.g. a hypothetical
        // "cognitive.linguistics2" sibling domain).
        assert!(!domain_is_nlp_relevant("cognitive.linguistics2"));
        assert!(!domain_is_nlp_relevant("cognitive.cognition.epistemics"));
    }
}
