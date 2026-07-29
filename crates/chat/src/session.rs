//! Multi-turn conversation state (task #17) — the asker's answer to a rule's
//! missing fact carries across turns, so [`ChatOutcome::Conditional`]'s
//! prompt is followed up rather than left hanging on a stateless per-call
//! API. Grounded in the SAME Bobrow, Kaplan, Norman, Thompson & Winograd
//! (1977) "GUS, A Frame-Driven Dialog System" frame this crate's
//! `conditional_rule` capability already implements for the FIRST turn (ask
//! for the unfilled slot, never guess) — [`ChatSession`] is GUS's turn-taking
//! loop made explicit: hold the frame between turns, fill one slot per
//! answer, re-evaluate, ask again or conclude. Turn-to-turn persistence
//! itself is Grosz & Sidner (1986) "Attention, Intentions, and the Structure
//! of Discourse" (*Computational Linguistics* 12(3):175-204) — the
//! "attentional state" that survives across an extended exchange, here
//! narrowed to exactly the one frame this capability ever has open.
//!
//! One host object per asker: the CLI's REPL owns one [`ChatSession`] for its
//! process lifetime; the WASM `Pr4xis` struct owns one per page session.
//! Neither host thread-shares a session, so no `Send`/`Sync` bound is
//! required (matches [`pr4xis_domains::cognitive::linguistics::composed::
//! ComposedReasoner`]'s own `Rc`-based non-`Sync` design, which a session
//! reasons over via `&dyn LexicalReasoner`).

use pr4xis::category::NonEmpty;
use pr4xis_domains::cognitive::linguistics::english::{English, LexicalReasoner};
use pr4xis_domains::cognitive::linguistics::language::Language;
use pr4xis_domains::cognitive::linguistics::lexicon::pos::Polarity;
use pr4xis_domains::cognitive::linguistics::pragmatics::realize::{
    self, ConditionalPresentation, ResponseContent, RuleVerdictPresentation,
};
use pr4xis_domains::cognitive::linguistics::pragmatics::response::ResponseFrame;
use pr4xis_domains::cognitive::linguistics::pragmatics::speech_act::SpeechAct;
use pr4xis_domains::formal::information::diagnostics::trace_functors::PipelineTrace;
use pr4xis_domains::formal::information::diagnostics::trace_impls;
use pr4xis_domains::social::judicial::conditional_rule::{
    Applicability, AppliedElement, ConditionalRule, FactValue, SlotState,
};
use pr4xis_domains::social::judicial::ontology::EvidenceType;

use crate::{ChatOutcome, ProcessResult, WasmSafeTimer, process_with_reasoner};

/// The one rule-application currently mid-turn — the GUS frame with an
/// unfilled slot, held between the turn that raised it and the turn that
/// answers it.
struct PendingRule {
    rule: ConditionalRule,
    missing: NonEmpty<AppliedElement>,
}

/// Multi-turn conversation state over the `pr4xis-chat` pipeline. `ask` is
/// the ONE entry point a host (CLI REPL, WASM `Pr4xis`) drives per turn — it
/// transparently either runs a fresh question through
/// [`process_with_reasoner`] or, when a rule is mid-turn, tries to interpret
/// this turn's input as the missing fact.
#[derive(Default)]
pub struct ChatSession {
    pending: Option<PendingRule>,
}

impl ChatSession {
    pub fn new() -> Self {
        Self { pending: None }
    }

    /// Whether a rule is currently awaiting a fact — a host can use this to
    /// change its prompt ("still waiting on: …") without inspecting the last
    /// `ProcessResult`.
    pub fn is_awaiting_fact(&self) -> bool {
        self.pending.is_some()
    }

    /// Drive one turn. `lang` is the linguistic substrate, `reasoner` the
    /// lexical-reasoning surface — same contract as
    /// [`process_with_reasoner`]. When no rule is pending this IS
    /// `process_with_reasoner`, byte-for-byte (a session with no open frame
    /// changes nothing about single-turn behavior).
    pub fn ask(
        &mut self,
        lang: &English,
        reasoner: &dyn LexicalReasoner,
        input: &str,
    ) -> ProcessResult {
        if let Some(pending) = self.pending.take() {
            return self.resume(lang, reasoner, pending, input);
        }
        let result = process_with_reasoner(lang, reasoner, input);
        if let ChatOutcome::Conditional { rule, missing } = &result.outcome {
            self.pending = Some(PendingRule {
                rule: (**rule).clone(),
                missing: missing.clone(),
            });
        }
        result
    }

    /// Try to interpret `input` as the answer to `pending.missing.head` — the
    /// ONE element GUS's "ask, don't guess" contract names. Two evidence
    /// types have a grounded free-text interpretation today:
    /// [`EvidenceType::Concept`] (resolved via the SAME `reasoner.lookup`
    /// every define/is-a turn already uses) and [`EvidenceType::Boolean`]
    /// (resolved via `lang`'s loaded response-interjection [`Polarity`] —
    /// "yes"/"ok"/"yeah" affirm, "no"/"nope" deny, satisfying or explicitly
    /// NOT satisfying the element). Every other evidence type honestly
    /// re-asks rather than invent a parse this codebase has no cited
    /// grammar for (Date/Currency/Duration/Count need the not-yet-built
    /// numeral→Quantity bridge; Document/Narrative/Text have no typed
    /// `FactValue` shape — see `FactValue`'s own doc comment for why a
    /// free-text catch-all is deliberately NOT invented here).
    fn resume(
        &mut self,
        lang: &English,
        reasoner: &dyn LexicalReasoner,
        pending: PendingRule,
        input: &str,
    ) -> ProcessResult {
        let field_type = pending.missing.head.requirement.field_type;
        let field = pending.missing.head.requirement.field.text.clone();
        let trimmed = input.trim().to_lowercase();

        if field_type == EvidenceType::Concept
            && !trimmed.is_empty()
            && let Some(&id) = reasoner.lookup(&trimmed).first()
        {
            let filled = pending
                .rule
                .with_element_filled(&field, SlotState::Satisfied(FactValue::Concept(id)));
            return self.finish_or_continue(filled);
        }

        if field_type == EvidenceType::Boolean
            && let Some(polarity) = lang
                .lexical_lookup_all(&trimmed)
                .iter()
                .find_map(|entry| entry.response_polarity())
        {
            let state = match polarity {
                Polarity::Affirmative => SlotState::Satisfied(FactValue::Boolean(true)),
                Polarity::Negative => SlotState::NotSatisfied(FactValue::Boolean(false)),
            };
            let filled = pending.rule.with_element_filled(&field, state);
            return self.finish_or_continue(filled);
        }

        // Cannot interpret this turn as the missing fact — re-state the SAME
        // prompt (never fabricate a parse) and keep the frame open.
        let response = realize_awaiting_fact(&pending.rule, &pending.missing);
        let outcome = ChatOutcome::Conditional {
            rule: Box::new(pending.rule.clone()),
            missing: pending.missing.clone(),
        };
        self.pending = Some(pending);
        minimal_result(response, outcome)
    }

    /// Re-evaluate `rule.applicability()` after a slot fill: conclude with a
    /// verdict, or open the NEXT missing element as a fresh `Conditional`
    /// prompt.
    fn finish_or_continue(&mut self, rule: ConditionalRule) -> ProcessResult {
        match rule.applicability() {
            Applicability::Applies => {
                let response = realize_verdict(&rule, true);
                minimal_result(
                    response,
                    ChatOutcome::RuleResolved {
                        rule: Box::new(rule),
                        applies: true,
                    },
                )
            }
            Applicability::DoesNotApply => {
                let response = realize_verdict(&rule, false);
                minimal_result(
                    response,
                    ChatOutcome::RuleResolved {
                        rule: Box::new(rule),
                        applies: false,
                    },
                )
            }
            Applicability::Indeterminate { missing } => {
                let response = realize_awaiting_fact(&rule, &missing);
                self.pending = Some(PendingRule {
                    rule: rule.clone(),
                    missing: missing.clone(),
                });
                minimal_result(
                    response,
                    ChatOutcome::Conditional {
                        rule: Box::new(rule),
                        missing,
                    },
                )
            }
        }
    }
}

fn citation_of(rule: &ConditionalRule) -> String {
    rule.term
        .source_text
        .as_ref()
        .map(|s| s.text.clone())
        .or_else(|| rule.term.subsection.as_ref().map(|c| c.to_bluebook()))
        .unwrap_or_default()
}

fn realize_awaiting_fact(rule: &ConditionalRule, missing: &NonEmpty<AppliedElement>) -> String {
    let presentation = ConditionalPresentation {
        rule_name: rule.term.name.text.clone(),
        citation: citation_of(rule),
        rule_text: rule.term.definition.text.clone(),
        missing_facts: missing
            .iter()
            .map(|el| el.requirement.field.text.clone())
            .collect(),
    };
    let content =
        ResponseContent::new(ResponseFrame::RuleAwaitingFact).with_conditional(presentation);
    realize::realize(&content)
}

fn realize_verdict(rule: &ConditionalRule, applies: bool) -> String {
    let presentation = RuleVerdictPresentation {
        rule_name: rule.term.name.text.clone(),
        citation: citation_of(rule),
        rule_text: rule.term.definition.text.clone(),
    };
    let frame = if applies {
        ResponseFrame::RuleApplies
    } else {
        ResponseFrame::RuleDoesNotApply
    };
    let content = ResponseContent::new(frame).with_verdict(presentation);
    realize::realize(&content)
}

/// A `ProcessResult` for a resume-turn — no chart parse happened (the input
/// was interpreted directly against the pending slot's evidence type, not
/// run through tokenize/chart-reduce), so this mirrors `process_with_
/// reasoner`'s own empty-input early return: an honest empty trace/token
/// count, `parsed: false`.
fn minimal_result(response: String, outcome: ChatOutcome) -> ProcessResult {
    let start = WasmSafeTimer::now();
    ProcessResult {
        response,
        user_act: SpeechAct::Assertion,
        system_act: SpeechAct::Assertion,
        duration_us: start.elapsed_us(),
        token_count: 0,
        parsed: false,
        trace: PipelineTrace::from_traceable(&trace_impls::TokenizeResult { tokens: &[] }),
        from_ontology: true,
        outcome,
        // A resume-turn's outcome is always a rule verdict or a re-asked
        // conditional prompt — both self-explaining frames (the response already
        // states the rule and what it needs), so no Why? panel.
        why: None,
        // A resume-turn recites no definition, so there is no gloss whose
        // authorship could be stated.
        definition_provenance: None,
    }
}
