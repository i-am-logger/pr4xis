//! The **completeness meter** — an honest, non-failing report of how far the
//! universal compiler's `.prx → source` decompile leg has reached, per source.
//!
//! # The meter, not a new judge
//!
//! [`RoundTripFidelity`] already grades a source's round-trip: the universal
//! FLOOR ([`RoundTripFidelity::RawBytesComplementFloor`], byte-exact via a
//! stored constant complement) vs the target
//! ([`RoundTripFidelity::ByteExactGraphFaithful`], regenerated from the typed
//! ontology graph alone). The [`super::harness`] already runs the right PutGet
//! law per source and renders a [`HarnessOutcome`] verdict. This module does
//! NOT re-decide either: it *reads* the harness verdict and the achieved
//! `RoundTripFidelity` and lays them out as one [`CompletenessReport`] row per
//! source, naming the work that remains to reach graph-faithfulness.
//!
//! # What each row honestly states
//!
//! For every source it states whether the round-trip is *floor-via-stored-
//! complement* or *graph-faithful*, and — when still on the floor — names the
//! per-source gap (OWL: `write_owl_exact` + RDFC #258; USC: `write_uslm`;
//! WordNet: `write_wordnet`). The count of sources still on the floor is the
//! remaining gap to a graph-only universal compiler. It is a *report*: it never
//! fails CI, and it never blocks the all-sources source round-trip test (that
//! test is green at the floor today).
//!
//! # The anti-lie cross-check
//!
//! A source can NEVER falsely claim graph-faithfulness. Each row carries both
//! the *declared* tier (the lens type's `WellBehavedLens::FIDELITY`, as the
//! harness registered it) and the *achieved* tier (the `RoundTripFidelity` the
//! emitted `.prx` actually carries, as [`super::decompile::decompile`] returns
//! it). [`CompletenessReport::tier_is_consistent`] asserts they agree;
//! [`declared_matches_achieved`](crate::formal::meta::well_behaved_lens::completeness::declared_matches_achieved())
//! turns a disagreement into a test
//! failure. So a `.prx` carrying a `RawBytesComplementFloor` complement under a
//! lens that declares `ByteExactGraphFaithful` (or the reverse) is caught — the
//! meter cannot be made to over-claim.

use alloc::{string::String, vec::Vec};

use super::decompile::DecompileKind;
use super::harness::{HarnessOutcome, run_round_trip_harness};
use super::lens_trait::RoundTripFidelity;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

/// One source's completeness row — what tier its `.prx → source` round-trip
/// reaches, and the gap to graph-faithfulness if it is still on the floor.
///
/// Scoped to sources with a `.prx` decompile leaf (OWL / USC / WordNet): those
/// are the universal compiler's compile/decompile pairs. The canonical-form
/// lenses with no `.prx` consumer (XSD, DTD, plaintext) are the harness's
/// concern, not the decompile meter's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletenessReport {
    /// The registry source key (`"{name}@{version}"`) this row describes.
    pub source: String,
    /// The decompile leaf this source routes through (always present — the
    /// meter is scoped to `.prx`-consumer sources).
    pub kind: DecompileKind,
    /// The fidelity the source's lens *declares* it can reach — the lens type's
    /// `WellBehavedLens::FIDELITY` const, as registered with the harness.
    pub declared: RoundTripFidelity,
    /// The fidelity the round-trip *achieves* in practice — the harness verdict
    /// rendered as a tier, in a THREE-state type.
    ///
    /// This was `Option<RoundTripFidelity>`, and that conflation was a real
    /// hole in the anti-lie check. `None` carried two opposite meanings — "not
    /// measured in this lane" and "measured, and the declared law DID NOT
    /// HOLD" — and [`Self::tier_is_consistent`] answered `true` for `None`,
    /// which is right for the first and catastrophically wrong for the second.
    /// So a `LawViolated` / `ByteLawViolated` / `SignatureMismatch` /
    /// `LoadError` verdict passed the anti-lie check and rendered in
    /// [`Self::summary_line`] as the reassuring "graph-faithful (declared) —
    /// byte-exact proof in the slow / all-sources lane", string-identical to a
    /// legitimate oversize deferral. `achieved_tier`'s own comment said it
    /// reported `None` "so `tier_is_consistent` flags it"; it did not.
    /// Observed live on `caregiving_lexicon@2026` and
    /// `hcbs_compliance_lexicon@2026`.
    pub achieved: AchievedFidelity,
    /// The work remaining to lift this source from the floor to
    /// graph-faithfulness (the per-source byte-exact writer), or `None` once it
    /// already declares — and achieves — `ByteExactGraphFaithful`.
    pub graph_faithful_gap: Option<&'static str>,
}

/// What the harness actually established about a source's round-trip.
///
/// THREE states, because the two that used to share `None` demand opposite
/// treatment from the anti-lie check: an unmeasured source cannot contradict
/// its declaration, whereas a REFUTED one contradicts it maximally.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AchievedFidelity {
    /// Measured here, and the law matching the declared tier HELD. The source
    /// genuinely reaches this fidelity.
    Proven(RoundTripFidelity),
    /// NOT measured in this lane, so nothing here contradicts the declaration:
    /// the source is not provisioned on this machine, or it is an oversize
    /// byte-exact source deferred to the slow lane, or it has no canonical
    /// lens at all. The proof, if any, lives elsewhere. Vacuously consistent —
    /// and the meter must say "pending", never credit a tier it did not
    /// measure.
    NotMeasuredHere,
    /// Measured here, and the declared law DID NOT HOLD — carrying the harness
    /// verdict that refuted it. This is the state that previously hid inside
    /// `None`. It must fail every consistency check and must never render as
    /// reassuring prose.
    Refuted(&'static str),
}

impl CompletenessReport {
    /// `true` when the source's round-trip is byte-exact via the *stored
    /// constant complement* (the universal FLOOR), not from the graph alone.
    /// The honest statement the task requires: "floor-via-stored-complement".
    /// Requires a PROVEN measurement — a refuted or unmeasured source is not
    /// on the floor, it is unestablished.
    #[must_use]
    pub fn is_floor_via_stored_complement(&self) -> bool {
        self.achieved == AchievedFidelity::Proven(RoundTripFidelity::RawBytesComplementFloor)
    }

    /// `true` when the source's round-trip regenerates the bytes from the typed
    /// ontology graph alone (no stored complement) — STAGE 2's target.
    #[must_use]
    pub fn is_graph_faithful(&self) -> bool {
        self.achieved == AchievedFidelity::Proven(RoundTripFidelity::ByteExactGraphFaithful)
    }

    /// `true` when the harness MEASURED this source and its declared law did
    /// not hold. The state the meter exists to surface.
    #[must_use]
    pub fn is_refuted(&self) -> bool {
        matches!(self.achieved, AchievedFidelity::Refuted(_))
    }

    /// The anti-lie invariant: the *declared* tier must equal the *achieved*
    /// tier whenever the source is provisioned (so an achieved tier exists). A
    /// source whose `.prx` carries a different fidelity than its lens declares
    /// is lying about its round-trip; this returns `false` for it.
    ///
    /// When the source is NOT MEASURED in this lane there is nothing here to
    /// contradict the declaration, so the check is vacuously satisfied — but
    /// a REFUTED source contradicts it maximally and returns `false`. Those
    /// two used to share `None` and both answered `true`; see
    /// [`Self::achieved`] for the hole that created.
    #[must_use]
    pub fn tier_is_consistent(&self) -> bool {
        match self.achieved {
            AchievedFidelity::Proven(achieved) => achieved == self.declared,
            AchievedFidelity::NotMeasuredHere => true,
            AchievedFidelity::Refuted(_) => false,
        }
    }

    /// One honest human line for this source — the meter's printable form.
    #[must_use]
    pub fn summary_line(&self) -> String {
        let tier = match self.achieved {
            AchievedFidelity::Proven(RoundTripFidelity::RawBytesComplementFloor) => {
                alloc::string::String::from("floor-via-stored-complement")
            }
            AchievedFidelity::Proven(RoundTripFidelity::ByteExactGraphFaithful) => {
                alloc::string::String::from("graph-faithful")
            }
            // MEASURED AND FAILED. This must never read like the deferral case
            // below — it previously did, byte-for-byte, which is what let a
            // hard law violation present as reassurance.
            AchievedFidelity::Refuted(why) => {
                alloc::format!(
                    "*** LAW REFUTED — declared {:?} but {why} ***",
                    self.declared
                )
            }
            // No in-crate measurement. The reason depends on what the source
            // DECLARES: a graph-faithful source with no fast-lane measurement is
            // either oversize (deferred to the slow lane) or not provisioned —
            // its byte-exact proof lives in the slow `ci_gate_passes_giants` +
            // the all-sources source round-trip test, NOT on the floor; a
            // floor-declared source with no measurement is genuinely pending.
            AchievedFidelity::NotMeasuredHere => match self.declared {
                RoundTripFidelity::ByteExactGraphFaithful => alloc::string::String::from(
                    "graph-faithful (declared) — byte-exact proof in the slow / all-sources lane",
                ),
                RoundTripFidelity::RawBytesComplementFloor => {
                    alloc::string::String::from("floor — pending in-crate verification")
                }
            },
        };
        match self.graph_faithful_gap {
            Some(gap) => alloc::format!("{}: {tier} — gap to graph-faithful: {gap}", self.source),
            None => alloc::format!("{}: {tier}", self.source),
        }
    }
}

/// Render the harness verdict for one source as an [`AchievedFidelity`].
///
/// The harness runs the law that matches the lens's declared fidelity (the
/// canonical PutGet law for the floor, the byte-exact law for graph-faithful)
/// and reports the outcome. Three verdict classes map to the three states:
///
/// * the law for `declared` HELD ⟹ [`AchievedFidelity::Proven`]`(declared)`;
/// * the source was not measured in this lane (not on disk, or oversize and
///   deferred to the slow lane) ⟹ [`AchievedFidelity::NotMeasuredHere`];
/// * the law for `declared` DID NOT HOLD, or the source is unloadable /
///   unregistered ⟹ [`AchievedFidelity::Refuted`], carrying which law failed.
///
/// The third class is what makes the cross-check bite: it flags the
/// inconsistency instead of silently crediting the declaration.
fn achieved_tier(outcome: &HarnessOutcome, declared: RoundTripFidelity) -> AchievedFidelity {
    match outcome {
        // The law that matches `declared` held (and the signature is either
        // matched or not-yet-pinned) — the source genuinely reaches `declared`.
        HarnessOutcome::Verified | HarnessOutcome::LawHoldsSignatureUnpinned { .. } => {
            AchievedFidelity::Proven(declared)
        }
        // Not provisioned — nothing measured here; the meter states "pending".
        HarnessOutcome::SourceNotOnDisk { .. } => AchievedFidelity::NotMeasuredHere,
        // Oversize byte-exact source deferred out of the fast harness for the CI
        // budget — proven in the slow lane (`ci_gate_passes_giants` + the
        // all-sources source round-trip test), not measured in this fast view.
        // Like `SourceNotOnDisk`, the proof lives elsewhere, so the meter states
        // "pending" rather than crediting a tier it did not measure here.
        HarnessOutcome::OversizeDeferred { .. } => AchievedFidelity::NotMeasuredHere,
        // The declared law did NOT hold (or the source is unloadable). Report
        // REFUTED — a distinct state from "not measured", so
        // `tier_is_consistent` flags it instead of crediting an unproven
        // declaration. This comment previously described behaviour the code
        // did not have: both arms returned `None`, and `tier_is_consistent`
        // answered `true` for `None`.
        HarnessOutcome::SignatureMismatch { .. } => {
            AchievedFidelity::Refuted("the emitted .prx does not match its committed signature")
        }
        HarnessOutcome::LawViolated(_) => {
            AchievedFidelity::Refuted("the canonical PutGet law did not hold")
        }
        HarnessOutcome::ByteLawViolated(_) => {
            AchievedFidelity::Refuted("the byte-exact round-trip law did not hold")
        }
        HarnessOutcome::LoadError { .. } => {
            AchievedFidelity::Refuted("the source could not be loaded through its own lens")
        }
        HarnessOutcome::SourceNotRegistered => AchievedFidelity::Refuted(
            "the lens is registered but its source is not in the registry",
        ),
    }
}

/// Build the completeness meter over every registered lens — one
/// [`CompletenessReport`] per source, reusing the harness verdict (the meter is
/// the harness, surfaced per source). Non-failing: every source produces a row
/// whatever its tier or provisioning state.
///
/// The achieved tier comes from the harness running the law that matches each
/// lens's declared fidelity; the declared tier comes from the registration's
/// `fidelity`. A source on the floor carries its named graph-faithful gap; one
/// that already declares (and the harness confirms it achieves)
/// `ByteExactGraphFaithful` carries `None` — there is nothing left to close.
#[must_use]
pub fn completeness_meter() -> Vec<CompletenessReport> {
    use super::harness::lens_registrations;
    use crate::applied::data_provisioning::registry::data_sources;

    let results = run_round_trip_harness();
    let lenses = lens_registrations();
    let mut out = Vec::new();

    // Enumerate EVERY `.prx`-consumer source (OWL / USC / WordNet) — the same
    // set the all-sources source round-trip test exercises — not just the subset
    // that happens to carry a registered canonical-form lens. A source with no
    // registered lens (WordNet has no canonical writer yet) is, by construction,
    // on the FLOOR: it can only declare the floor, and `write_wordnet` is its
    // open gap. Scoping the meter to lens registrations would silently drop that
    // gap and under-count the remaining work — the one dishonesty this meter
    // exists to prevent.
    for entry in data_sources() {
        let Some(kind) = DecompileKind::from_content_type(entry.content_type()) else {
            continue;
        };

        // A registered lens for this source, if any. Its `FIDELITY` is the
        // DECLARED tier; without a lens the source can only declare the FLOOR
        // (no graph-faithful writer ⇒ no graph-faithful claim).
        let lens = lenses
            .iter()
            .find(|r| r.source_name == entry.name && r.source_version == entry.version);
        let declared = lens
            .map(|r| r.fidelity)
            .unwrap_or(RoundTripFidelity::RawBytesComplementFloor);

        // The ACHIEVED tier is the harness verdict — but only a lens-registered
        // source provisioned on disk has one. A source with no lens (WordNet) is
        // not measured by the in-crate harness; its floor round-trip is proven by
        // the all-sources source round-trip test instead, so the meter states
        // `NotMeasuredHere` rather than crediting a tier it did not measure.
        // A source the harness DID measure and whose law FAILED reports
        // `Refuted` — never `NotMeasuredHere`. Collapsing those two into a
        // single `None` is what previously let a law violation read as a
        // pending deferral.
        let achieved = lens
            .and_then(|r| results.iter().find(|h| h.key == r.key))
            .map_or(AchievedFidelity::NotMeasuredHere, |h| {
                achieved_tier(&h.outcome, declared)
            });

        // The gap is named while the source is not yet graph-faithful; closing
        // it (a byte-exact `write_<kind>`) is exactly what STAGE 2 builds.
        let graph_faithful_gap = match declared {
            RoundTripFidelity::RawBytesComplementFloor => Some(kind.graph_faithful_gap()),
            RoundTripFidelity::ByteExactGraphFaithful => None,
        };

        out.push(CompletenessReport {
            source: alloc::format!("{}@{}", entry.name, entry.version),
            kind,
            declared,
            achieved,
            graph_faithful_gap,
        });
    }
    out.sort_by(|a, b| a.source.cmp(&b.source));
    out
}

/// The number of `.prx`-consumer sources still on the stored-complement FLOOR —
/// i.e. NOT yet graph-faithful, each still carrying a named `write_<kind>` gap.
/// This is the remaining gap to a graph-only universal compiler, counted over
/// EVERY registered `.prx` source (OWL / USC / WordNet), whether or not its
/// corpus is provisioned on THIS machine — a source's gap to graph-faithfulness
/// is a property of the compiler, not of local disk state. Counting only the
/// sources whose floor is *measured* in-crate would silently drop the corpora
/// that aren't bundled (WordNet) and under-state the gap. Does NOT block the
/// all-sources source round-trip test (green at the floor today).
#[must_use]
pub fn floor_source_count(meter: &[CompletenessReport]) -> Quantity {
    let n = meter
        .iter()
        .filter(|r| r.graph_faithful_gap.is_some())
        .count();
    Quantity::from_unit(n as f64, &unit::UNITLESS)
}

/// Print the completeness meter to stderr — one honest line per source plus the
/// floor/graph-faithful tally. Procedural tooling (a CLI surface), not a
/// `#[test]`, matching [`super::harness::dump_unpinned_signatures`].
pub fn print_completeness_meter() {
    let meter = completeness_meter();
    // The remaining gap = every `.prx` source not yet graph-faithful, split into
    // those whose floor this binary VERIFIED in-crate and those whose corpus
    // isn't provisioned here (proven instead by the all-sources round-trip test)
    // — so the count is complete AND traceable, never an in-crate-only undercount.
    let floor = floor_source_count(&meter).value as usize;
    let graph_faithful = meter.iter().filter(|r| r.is_graph_faithful()).count();
    let measured_floor = meter
        .iter()
        .filter(|r| r.is_floor_via_stored_complement())
        .count();
    let pending = floor.saturating_sub(measured_floor);
    for r in &meter {
        eprintln!("{}", r.summary_line());
    }
    eprintln!(
        "decompile completeness: {graph_faithful} graph-faithful, {floor} still on the \
         stored-complement floor (= the remaining gap to a graph-only universal compiler; \
         {measured_floor} floor-verified in-crate, {pending} pending external provisioning, \
         covered by the all-sources source round-trip test)"
    );
}

/// Assert the anti-lie cross-check over the whole meter: every source's
/// declared tier matches its achieved tier (when provisioned). Returns the keys
/// of any source that lies about its round-trip fidelity — a non-empty result
/// is a test failure. The guarantee that the meter can never over-claim
/// graph-faithfulness.
#[must_use]
pub fn declared_matches_achieved(meter: &[CompletenessReport]) -> Vec<String> {
    meter
        .iter()
        .filter(|r| !r.tier_is_consistent())
        .map(|r| r.source.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::lens_trait::{FailureStage, LensLawFailure};
    use super::*;
    use alloc::string::ToString;

    /// Every [`HarnessOutcome`] variant, so the tests below enumerate the real
    /// verdict space rather than a hand-picked sample. A new variant makes
    /// `achieved_tier`'s match non-exhaustive at compile time and shows up
    /// here the moment it is added.
    fn every_outcome() -> Vec<HarnessOutcome> {
        let failure = || LensLawFailure {
            stage: FailureStage::DigestMismatch,
            message: "synthetic".to_string(),
            input_digest: None,
            roundtrip_digest: None,
        };
        alloc::vec![
            HarnessOutcome::Verified,
            HarnessOutcome::LawHoldsSignatureUnpinned {
                computed_digest_hex: "00".to_string(),
            },
            HarnessOutcome::SourceNotOnDisk {
                path: "/nowhere".to_string(),
            },
            HarnessOutcome::OversizeDeferred { size_bytes: 1 },
            HarnessOutcome::SignatureMismatch {
                expected: "blake3:00".to_string(),
                computed_digest_hex: "11".to_string(),
            },
            HarnessOutcome::LawViolated(failure()),
            HarnessOutcome::ByteLawViolated(failure()),
            HarnessOutcome::LoadError {
                path: "/nowhere".to_string(),
                message: "synthetic".to_string(),
            },
            HarnessOutcome::SourceNotRegistered,
        ]
    }

    fn row(declared: RoundTripFidelity, achieved: AchievedFidelity) -> CompletenessReport {
        CompletenessReport {
            source: "synthetic@0".to_string(),
            kind: DecompileKind::UsCode,
            declared,
            achieved,
            graph_faithful_gap: None,
        }
    }

    const BOTH_TIERS: [RoundTripFidelity; 2] = [
        RoundTripFidelity::RawBytesComplementFloor,
        RoundTripFidelity::ByteExactGraphFaithful,
    ];

    /// THE REGRESSION TEST for the anti-lie hole: a measured-and-failed
    /// verdict must NEVER pass `tier_is_consistent`. Before the three-state
    /// `AchievedFidelity`, every one of these rendered as `None` and
    /// `tier_is_consistent` answered `true` — so `declared_matches_achieved`
    /// returned an empty "no liars" list for a source whose law had just been
    /// refuted.
    #[test]
    #[pr4xis::praxis_value(Honest)]
    fn a_refuted_measurement_is_never_tier_consistent() {
        for declared in BOTH_TIERS {
            for outcome in every_outcome().iter().filter(|o| o.is_failure()) {
                let r = row(declared, achieved_tier(outcome, declared));
                assert!(
                    r.is_refuted(),
                    "{outcome:?} is a harness FAILURE, so the row must be Refuted"
                );
                assert!(
                    !r.tier_is_consistent(),
                    "{outcome:?} refuted the declared {declared:?} law, \
                     so the row must NOT pass the anti-lie check"
                );
                assert_eq!(
                    declared_matches_achieved(core::slice::from_ref(&r)),
                    alloc::vec!["synthetic@0".to_string()],
                    "{outcome:?} must be REPORTED as a liar, not filtered out"
                );
            }
        }
    }

    /// The second half of the same hole: a refuted row used to render prose
    /// STRING-IDENTICAL to a legitimate slow-lane deferral, so reading the
    /// meter could not distinguish "proof lives elsewhere" from "the law just
    /// failed". The two renderings must differ for every declared tier.
    #[test]
    #[pr4xis::praxis_value(Honest)]
    fn a_refuted_row_never_reads_like_a_pending_deferral() {
        for declared in BOTH_TIERS {
            let deferred = row(declared, AchievedFidelity::NotMeasuredHere).summary_line();
            for outcome in every_outcome().iter().filter(|o| o.is_failure()) {
                let refuted = row(declared, achieved_tier(outcome, declared)).summary_line();
                assert_ne!(
                    refuted, deferred,
                    "{outcome:?} under {declared:?} must not render as the deferral line"
                );
                assert!(
                    refuted.contains("REFUTED"),
                    "a refuted row must say so plainly, got {refuted:?}"
                );
            }
        }
    }

    /// `Refuted` must mean exactly what the harness calls a failure. These are
    /// two independent predicates over the same verdict space
    /// ([`HarnessOutcome::is_failure`] gates the CI axiom, `achieved_tier`
    /// gates the meter); if they ever drift apart, one of the two gates goes
    /// quietly blind. This pins them together.
    #[test]
    #[pr4xis::praxis_value(Verifiable)]
    fn refuted_agrees_with_the_harness_failure_predicate() {
        for declared in BOTH_TIERS {
            for outcome in every_outcome() {
                assert_eq!(
                    matches!(
                        achieved_tier(&outcome, declared),
                        AchievedFidelity::Refuted(_)
                    ),
                    outcome.is_failure(),
                    "{outcome:?}: the meter's Refuted and the harness's is_failure \
                     must classify every verdict identically"
                );
            }
        }
    }

    /// The legitimate deferral still passes vacuously — the fix must not turn
    /// "not measured in this lane" into a failure. `SourceNotOnDisk` (no
    /// corpus provisioned) and `OversizeDeferred` (proven in the slow lane)
    /// are the two honest ways to have no local measurement.
    #[test]
    #[pr4xis::praxis_value(Verifiable)]
    fn an_unmeasured_source_stays_vacuously_consistent() {
        for declared in BOTH_TIERS {
            for outcome in every_outcome().iter().filter(|o| !o.is_failure()) {
                let r = row(declared, achieved_tier(outcome, declared));
                assert!(
                    r.tier_is_consistent(),
                    "{outcome:?} does not contradict a declared {declared:?}"
                );
            }
            for outcome in [
                HarnessOutcome::SourceNotOnDisk {
                    path: "/nowhere".to_string(),
                },
                HarnessOutcome::OversizeDeferred { size_bytes: 1 },
            ] {
                assert_eq!(
                    achieved_tier(&outcome, declared),
                    AchievedFidelity::NotMeasuredHere,
                    "{outcome:?} is an absence of measurement, not a proof"
                );
            }
        }
    }

    /// The original anti-lie property, still holding: a PROVEN measurement is
    /// consistent exactly when it equals the declaration. A source cannot
    /// declare graph-faithful and be credited for reaching only the floor.
    #[test]
    #[pr4xis::praxis_value(Honest)]
    fn a_proven_measurement_must_equal_the_declaration() {
        for declared in BOTH_TIERS {
            for proven in BOTH_TIERS {
                assert_eq!(
                    row(declared, AchievedFidelity::Proven(proven)).tier_is_consistent(),
                    declared == proven,
                    "declared {declared:?} vs proven {proven:?}"
                );
            }
        }
    }

    /// A verified source is credited with the tier whose law the harness
    /// actually ran — never a higher one. `Verified` means "the law matching
    /// `declared` held", so the achieved tier IS `declared` and nothing more.
    #[test]
    #[pr4xis::praxis_value(Honest)]
    fn a_floor_source_is_never_credited_with_graph_faithfulness() {
        let r = row(
            RoundTripFidelity::RawBytesComplementFloor,
            achieved_tier(
                &HarnessOutcome::Verified,
                RoundTripFidelity::RawBytesComplementFloor,
            ),
        );
        assert!(r.is_floor_via_stored_complement());
        assert!(!r.is_graph_faithful());
        assert!(r.tier_is_consistent());
    }
}
