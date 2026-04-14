# Signal Processing -- Nyquist-Shannon sampling, spectra, filtering

Models the foundational concepts of digital signal processing: time and frequency domains, sample rate, bandwidth, Nyquist rate, and aliasing. The category is discrete over six signal-domain concepts; the axioms verify the Nyquist-Shannon sampling theorem and the bandwidth-positivity precondition against concrete sampling rates.

Key references:
- Shannon 1949: *Communication in the Presence of Noise* (Proc. IRE)
- Nyquist 1928: *Certain Topics in Telegraph Transmission Theory* (Trans. AIEE)
- Oppenheim & Willsky 1997: *Signals and Systems* (2nd ed.)

## Entities (6)

| Category | Entities |
|---|---|
| Domains (2) | TimeDomain, FrequencyDomain |
| Rates and bands (2) | SampleRate, Bandwidth |
| Sampling-theorem concepts (2) | NyquistRate, AliasFrequency |

## Category

Discrete category over the six signal-domain concept entities — objects only, identity morphisms only. The mathematical content lives in the axioms over canonical bandwidths and sample rates.

## Qualities

| Quality | Type | Description |
|---|---|---|
| ConceptDescription | &'static str | Textual description: TimeDomain="signal represented as amplitude vs time, x(t)", FrequencyDomain="signal represented as amplitude/phase vs frequency, X(f) = F{x(t)}", NyquistRate="minimum sample rate to avoid aliasing: f_nyquist = 2 * f_max", etc. |

## Axioms (3)

| Axiom | Description | Source |
|---|---|---|
| NyquistTheorem | Sampling at f_s >= 2*bandwidth preserves signal information | Shannon 1949 |
| AliasingOccursBelowNyquist | Aliasing occurs when sample rate < 2 * bandwidth | Nyquist 1928 |
| BandwidthPositive | Bandwidth is positive, therefore Nyquist rate is positive | standard |

Plus the auto-generated structural axioms from `define_ontology!` (category laws on the discrete category).

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one. The natural hearing ontology defines its own `SignalProcessingOntology` that mirrors many of these concepts; an explicit functor between the two would replace that duplication.

## Files

- `ontology.rs` -- Entity, discrete category, ConceptDescription quality, 3 axioms, tests, `test_low_pass_filter` helper
- `filter.rs` -- Digital filter types and `FirstOrderLowPass` implementation (alpha form)
- `sampling.rs` -- `nyquist_rate(bw)`, `is_adequately_sampled(fs, bw)` (Shannon-Nyquist)
- `spectrum.rs` -- Frequency-domain analysis: bandwidth, DFT resolution Δf = f_s / N
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
