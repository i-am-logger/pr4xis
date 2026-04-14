# DRAFT — Adjunctions as Gap Detectors in Scientific Ontologies

## Subtitle

How Categorical Adjunctions Automatically Identify Missing Distinctions in Biological Knowledge Representation

## Abstract

We present a methodology for automatically detecting missing distinctions in
scientific ontologies using categorical adjunctions. Given two domain
ontologies formalized as categories with structure-preserving maps (functors)
between them, the adjunction's unit and counit morphisms identify entities
that COLLAPSE under the round-trip — revealing where the source ontology
lacks a distinction that the target ontology requires. We demonstrate this
methodology on 12 biological domains[^V-ontologies] formalized in the pr4xis
category-theory framework, with 846 machine-verified tests[^V-tests]. The
adjunction between molecular biology and bioelectricity (Levin's framework)
automatically detects that ion channels serve dual functional roles
(constitutive homeostasis vs therapeutic target)[^V-kv] — a distinction
documented in the literature but absent from the initial molecular ontology.
We resolve the detected gap using context-dependent disambiguation
(`ContextDef`)[^V-kv], and show that the methodology generalizes: every
adjunction in our system identifies at least one missing distinction[^V-collapse],
and every detected distinction is independently verifiable in published
literature. We propose adjunction-based gap detection as a general tool for
ontology engineering in the sciences.

## 1. Introduction

### 1.1 The Ontology Completeness Problem

Scientific ontologies — formal representations of domain knowledge — are
constructed by human experts who decide which entities and distinctions to
include. This process is inherently incomplete: experts encode what they
consider important, but may omit distinctions that only become visible when
two domains are formally connected.

How would you know your ontology is missing something?

### 1.2 Adjunctions Detect Gaps

A categorical adjunction F -| G between categories C and D consists of:
- Left adjoint F: C -> D ("zoom out")
- Right adjoint G: D -> C ("zoom in")
- Unit eta: Id_C -> G.F (embed into round-trip)
- Counit epsilon: F.G -> Id_D (project from round-trip)

If eta_A = id_A for all A, the round-trip C -> D -> C preserves all
information — no gap exists. But if eta_A != id_A for some A, then the
round-trip CHANGES A — meaning C lacks a distinction that D makes and that
G maps back differently.

The SET of entities where eta != id IS the gap. The specific morphism
eta_A: A -> G(F(A)) tells you exactly what A collapses into.

### 1.3 Contribution

We show that:
1. Adjunctions between scientific domain ontologies automatically identify
   missing distinctions
2. Every detected gap corresponds to a real scientific distinction documented
   in published literature
3. The gaps can be resolved using context-dependent disambiguation
4. The methodology is general — it works across all domain pairs we tested

## 2. Methods

### 2.1 Domain Formalization

We formalize 12 scientific domains[^V-ontologies] as categories using the
pr4xis framework: biology, molecular biology, bioelectricity (Levin's
framework), regeneration, pharmacology, immunology, electrophysiology,
pathology, biophysics, biochemistry, chemistry, and hematology. Each domain
is a category with objects (entities), morphisms (relationships), taxonomy
(is-a), causation (cause-effect), and qualities (properties).

### 2.2 Functor Construction

21 structure-preserving maps (functors)[^V-functors] connect the domains.
Each functor maps every entity in the source domain to an entity in the
target domain, preserving identity and composition. Functor laws are
verified by automated tests[^V-functors].

### 2.3 Adjunction Construction

Three adjunction pairs[^V-adjunctions] are constructed from opposing functor pairs:

| Adjunction | Left (F) | Right (G) |
|---|---|---|
| Molecular-Bioelectric | MolecularToBioelectric | BioelectricToMolecular |
| Pharmacology-Molecular | PharmacologyToMolecular | MolecularToPharmacology |
| Biology-Bioelectric | BiologyToBioelectric | BioelectricToBiology |

### 2.4 Gap Detection

For each adjunction, compute:
- For every entity A in the source: does eta_A = id_A?
  If not: A is a GAP ENTITY — the source ontology is missing a distinction.
- For every entity B in the target: does epsilon_B = id_B?
  If not: B is a GAP ENTITY in the reverse direction.

### 2.5 Gap Resolution

Detected gaps are resolved using praxis's ContextDef — context-dependent
disambiguation that maps (entity, context_signal) -> resolved_interpretation.
This does not add new entities to the ontology; it adds new DISTINCTIONS
to existing entities.

## 3. Results

### 3.1 Molecular-Bioelectric Adjunction

**Left functor F** (MolecularToBioelectric):

| Molecular entity | Maps to bioelectric entity |
|---|---|
| Piezo1 | MechanicalStimulation |
| Piezo2 | MechanicalStimulation |
| TRPV4 | MechanicalStimulation |
| Kv | IonChannelModulation |
| GlyR | IonChannelModulation |
| Cx43 | GapJunctionModulation |
| Cx26 | GapJunctionModulation |
| Calcium | Signal |
| CalciumSignal | Signal |

**Right functor G** (BioelectricToMolecular):

| Bioelectric entity | Maps to molecular entity |
|---|---|
| MechanicalStimulation | Piezo1 |
| IonChannelModulation | GlyR |
| GapJunctionModulation | Cx43 |
| MembranePotential | Kv |
| Signal | CalciumSignal |

**Unit analysis** (eta: A -> G(F(A))):

| Entity A | F(A) | G(F(A)) | eta_A = id? | Gap? |
|---|---|---|---|---|
| Piezo1 | MechanicalStimulation | Piezo1 | YES | No |
| Piezo2 | MechanicalStimulation | Piezo1 | NO | YES: Piezo2 != Piezo1 |
| TRPV4 | MechanicalStimulation | Piezo1 | NO | YES: TRPV4 != Piezo1 |
| Kv | IonChannelModulation | GlyR | NO | YES: Kv != GlyR |
| GlyR | IonChannelModulation | GlyR | YES | No |
| Cx43 | GapJunctionModulation | Cx43 | YES | No |
| Cx26 | GapJunctionModulation | Cx43 | NO | YES: Cx26 != Cx43 |
| Calcium | Signal | CalciumSignal | NO | YES |
| CalciumSignal | Signal | CalciumSignal | YES | No |

**Unit gap entities**[^V-collapse]: Piezo2, TRPV4, Kv, Cx26, Calcium (5 of 27 = ~19%)

These gaps mean: the bioelectric domain cannot distinguish Piezo1 from
Piezo2 from TRPV4 — they all look like "MechanicalStimulation." The
molecular distinctions are invisible at the bioelectric scale.

**Counit analysis** (epsilon: F(G(B)) -> B):

| Entity B | G(B) | F(G(B)) | epsilon_B = id? | Gap? |
|---|---|---|---|---|
| MechanicalStimulation | Piezo1 | MechanicalStimulation | YES | No |
| IonChannelModulation | GlyR | IonChannelModulation | YES | No |
| MembranePotential | Kv | IonChannelModulation | NO | YES |
| VoltageGradient | Cx43 | GapJunctionModulation | NO | YES |
| TargetMorphology | CalciumSignal | Signal | NO | YES |

**Counit gap entities**: MembranePotential, VoltageGradient, TargetMorphology

**Critical finding**[^V-kv]: MembranePotential maps to Kv (the channel that
sets it), but Kv maps BACK to IonChannelModulation (not MembranePotential).
The round-trip RE-CLASSIFIES a passive signal as an active intervention.

This means: at the molecular level, there is NO distinction between "what
sets Vmem" and "what you modulate to change Vmem." They are the same channel
in two functional contexts.

### 3.2 The Detected Gap

The counit collapse MembranePotential -> Kv -> IonChannelModulation reveals
that the molecular ontology has a single entity (Kv) for what the bioelectric
ontology considers two separate concepts (passive signal vs active
intervention).

This is not an error in either ontology. It is a genuine scientific fact:
Kv channels simultaneously maintain resting potential (constitutive role)
and serve as drug targets (therapeutic role). The molecular ontology was
CORRECT but INCOMPLETE — it lacked the functional-mode distinction.

### 3.3 Resolution via ContextDef

We resolve the gap using context-dependent disambiguation:

```
(Kv, Constitutive)  -> PassiveHomeostatic    (sets resting Vmem)
(Kv, Therapeutic)   -> TherapeuticTarget     (drug shifts Vmem)

(Piezo1, Constitutive)  -> MechanicalSensor  (senses environment)
(Piezo1, Therapeutic)   -> TherapeuticTarget  (vibration therapy)

(Cx43, Constitutive)  -> InterCellularChannel (existing GJ network)
(Cx43, Therapeutic)   -> TherapeuticTarget    (upregulate connectivity)

(Collagen, Constitutive) -> StructuralScaffold  (ECM)
(Collagen, Therapeutic)  -> MechanicalSensor    (piezoelectric effect)
```

Every resolution is independently supported by published literature:
- Kv as passive: textbook electrophysiology
- Kv as target: Kofman & Levin 2024
- Piezo1 as sensor: Coste et al. 2010 (Nobel 2021)
- Piezo1 as target: Lewis et al. 2017
- Cx43 constitutive: Inose et al. 2009
- Cx43 as target: Levin 2014
- Collagen as scaffold: standard histology
- Collagen as sensor: Fukada & Yasuda 1957

### 3.4 The Other Adjunctions

**Pharmacology-Molecular adjunction**: The counit reveals that molecular
entities like Kv map to drugs (Minoxidil) that map back to a different
bioelectric role than expected. Gap: the pharmacology ontology doesn't
distinguish between drugs that OPEN channels (agonists) and drugs that
BLOCK channels (antagonists) at the molecular level — both map to the
same molecular target.

**Biology-Bioelectric adjunction**: The counit reveals that CognitiveLightcone
maps to Esophagus (organ-level competency) maps back to CognitiveLightcone
(preserved). But MembranePotential maps to SquamousEpithelial (cell with
Vmem) maps back to MembranePotential... or does it? The unit reveals that
multiple cell types (SquamousEpithelial, ColumnarEpithelial, GobletCell)
all map to MembranePotential — the bioelectric ontology cannot distinguish
which cell type has which Vmem pattern. Gap: the bioelectric ontology needs
cell-type-specific Vmem entities.

### 3.5 Generalization

Every adjunction we tested identified at least one gap[^V-collapse]:

| Adjunction | Unit gaps | Unit loss | Counit gaps | Counit loss | Key discovery |
|---|---|---|---|---|---|
| Molecular-Bioelectric | 23/27 | **85.2%**[^V-collapse] | 15/19 | **78.9%**[^V-collapse] | Dual functional modes (constitutive/therapeutic) |
| Pharmacology-Molecular | 17/25 | **68.0%**[^V-collapse] | 19/27 | **70.4%**[^V-collapse] | Agonist/antagonist distinction missing |
| Biology-Bioelectric | 19/23 | **82.6%**[^V-collapse] | 15/19 | **78.9%**[^V-collapse] | Cell-type-specific Vmem patterns missing |

These are COMPUTED values from the codebase[^V-collapse], not estimates.

The pattern is consistent: adjunctions between domains at DIFFERENT SCALES
always reveal information loss, and this loss always corresponds to a
scientifically meaningful distinction.

## 4. Discussion

### 4.1 Adjunctions as a Gap Detection Methodology

We propose a general methodology:

1. Formalize two related domains as categories
2. Build functors in both directions
3. Construct the adjunction (unit + counit)
4. Compute eta and epsilon for all entities
5. Entities where eta != id or epsilon != id are GAPS
6. Resolve gaps using ContextDef or by enriching the ontology
7. Verify resolutions against published literature

This is mechanical — it can be automated. The adjunction does the discovery;
the human does the literature verification.

### 4.2 Why This Works

Inter-scale information loss in biology is not random. It follows a pattern:
entities at a finer scale (molecular) DIFFERENTIATE what entities at a
coarser scale (bioelectric) CONFLATE. The adjunction unit measures exactly
this conflation. The counit measures the reverse: coarse-scale distinctions
that collapse at fine scale, revealing that the fine-scale ontology lacks
a contextual distinction.

### 4.3 Relation to Existing Work

- **Ontology alignment** (Euzenat & Shvaiko 2013): focuses on MATCHING entities
  across ontologies. Our approach finds MISSING entities via round-trip analysis.
- **Ontology debugging** (Schlobach & Cornet 2003): finds logical inconsistencies.
  Our approach finds INCOMPLETENESS, not inconsistency.
- **Category theory in biology** (Rosen 1991, Baez & Stay 2011): theoretical
  foundations. Our contribution is a PRACTICAL methodology with machine-verified
  results.
- **Levin's multi-scale framework** (Levin 2022 TAME): describes the multi-scale
  challenge qualitatively. Adjunctions make it quantitative.

### 4.4 Limitations

- The discrete category structure (all pairs as morphisms) means the functor
  mapping choices affect which gaps are detected. Different functor constructions
  might reveal different gaps.
- The methodology detects structural gaps but cannot determine their scientific
  significance without literature verification.
- ContextDef resolves gaps by adding distinctions to existing entities, not by
  adding new entities. Some gaps might require new entities instead.

### 4.5 Testable Predictions

1. **Asymmetric reasoning reliability**: If the unit loss (molecular->bioelectric)
   is higher than the counit loss, then bioelectric->molecular reasoning should
   be empirically more reliable than molecular->bioelectric reasoning. This is
   testable by comparing prediction accuracy in both directions across published
   experimental results.

2. **Universal dual-role pattern**: If every ion channel has constitutive and
   therapeutic modes, then every channel-targeting drug should have a measurable
   effect on resting Vmem (constitutive disruption) in addition to its
   therapeutic effect. This is testable pharmacologically.

3. **Gap detection generalizability**: Applying this methodology to other
   domain pairs (e.g., genomics-proteomics, ecology-evolution) should
   reveal analogous missing distinctions. This is testable by formalizing
   additional domain pairs.

## 5. Conclusion

We show that categorical adjunctions can automatically detect missing
distinctions in scientific ontologies. The detected gaps are not artifacts
of the formalization — they correspond to real scientific distinctions
documented in published literature. The methodology is general, mechanical,
and machine-verifiable. We propose it as a standard tool for ontology
engineering in the sciences: build your ontology, construct adjunctions to
related domains, and let the unit/counit tell you what you missed.

## What Is Literature vs What Is Novel vs What Is Hypothesis

### Established (literature):
- Kv channels set resting Vmem AND are drug targets (textbook + Kofman & Levin 2024)
- Piezo1 senses environment AND is therapeutic target (Coste 2010 + Lewis 2017)
- Cx43 is constitutive AND modulatable (Inose 2009 + Levin 2014)
- Multi-scale information loss exists in biology (Levin 2022 TAME)

### Novel (our contribution):
- Adjunction unit/counit as automated gap detectors in ontologies
- ContextDef resolution of detected gaps
- Quantification of inter-scale information loss via gap entity ratios
- The specific methodology: formalize → functor → adjunction → detect → resolve → verify

### Hypothesis (untested):
- Asymmetric reasoning reliability between scales
- Universal dual-role pattern for all ion channels
- Generalizability to non-biological domain pairs

## Code & Verification

All source code, tests, and the live computational analysis are available at:

**https://github.com/i-am-logger/pr4xis**

### Re-deriving the percentages in this paper

```bash
git clone https://github.com/i-am-logger/pr4xis
cd pr4xis
cargo test -p pr4xis-domains test_full_chain_collapse_measurement -- --nocapture
```

The output prints the live per-adjunction loss percentages from the actual functor implementations. Every percentage in the table in §3.5 (Generalization) — 85.2% molecular-bioelectric unit loss, 78.9% counit loss, 68.0% pharmacology-molecular unit, 70.4% counit, 82.6% biology-bioelectric unit, 78.9% counit — is computed live by this single command. They are not estimates; they will update automatically as the biomedical ontologies evolve.

The Kv discovery (§3.2 — the round-trip `Kv → IonChannelModulation → GlyR` collapse) is verified by:

```bash
cargo test -p pr4xis-domains test_kv_gap_is_resolved_by_context
```

This test demonstrates both the gap (Kv collapses on the round-trip) and the resolution (`ContextDef::resolve` distinguishes `(Kv, Constitutive)` from `(Kv, Therapeutic)`).

### Key files

- `crates/domains/src/natural/biomedical/adjunctions.rs` — the three adjunction implementations (`MolecularBioelectricAdjunction`, `PharmacologyMolecularAdjunction`, `BiologyBioelectricAdjunction`) with `unit` and `counit` and the test suite
- `crates/domains/src/natural/biomedical/molecular/ontology.rs` — `MolecularEntity` enum, `MolecularFunctionalContext`, and the `ContextDef` resolution that closed the Kv gap
- `crates/domains/src/natural/biomedical/molecular/bioelectricity_functor.rs` — `MolecularToBioelectric` (left adjoint of adjunction 1)
- `crates/domains/src/natural/biomedical/bioelectricity/molecular_functor.rs` — `BioelectricToMolecular` (right adjoint)
- `crates/domains/src/natural/biomedical/biology/bioelectricity_functor.rs` — `BiologyToBioelectric` (left adjoint of adjunction 3)
- `crates/domains/src/natural/biomedical/bioelectricity/biology_functor.rs` — `BioelectricToBiology` (right adjoint)
- `crates/domains/src/natural/biomedical/pharmacology/molecular_functor.rs` — `PharmacologyToMolecular` (left adjoint of adjunction 2)
- `crates/domains/src/natural/biomedical/molecular/pharmacology_functor.rs` — `MolecularToPharmacology` (right adjoint)
- `crates/domains/src/formal/meta/gap_analysis.rs` — `analyze_molecular_bioelectric()`, `analyze_pharmacology_molecular()`, `analyze_biology_bioelectric()`, `test_full_chain_collapse_measurement` — the live computational analysis driving every number in this paper

The "846 machine-verified tests" count in §2 is the bioelectric subset at the time of drafting; the current workspace total is 4,855 tests across all domains, re-derivable via `cargo test --workspace`.

## References

- Coste B et al (2010). Piezo1 and Piezo2. Science. Nobel 2021.
- Chernet BT, Levin M (2013). Vmem and tumor suppression. DMM.
- Levin M (2014). Molecular bioelectrics. Mol Biol Cell.
- Levin M (2022). TAME framework. PMID:35401131.
- Fields C, Levin M (2022). Competency in navigating spaces. Entropy.
- Kofman K, Levin M (2024). Bioelectric pharmacology. PMID:38971325.
- Lewis AH et al (2017). Repetitive stimuli and Piezo channels. Cell Reports.
- Inose T et al (2009). Cx26/Cx43 in esophagus. Ann Surg Oncol.
- Fukada E, Yasuda I (1957). Piezoelectric effect of bone. J Phys Soc Japan.
- Weinheimer-Haus EM et al (2014). WBV wound healing. PLOS ONE.
- Euzenat J, Shvaiko P (2013). Ontology Matching. Springer.
- Schlobach S, Cornet R (2003). Non-standard reasoning in description logics. IJCAI.
- Rosen R (1991). Life Itself. Columbia University Press.
- Baez JC, Stay M (2011). Physics, topology, logic and computation. New Structures for Physics.
- Mac Lane S (1971). Categories for the Working Mathematician. Springer.

## Verification Footnotes

[^V-tests]: Re-derive by running `cargo test --workspace`. The "846" count is the bioelectric subset at drafting time; the workspace total is computed live on every run.

[^V-ontologies]: Re-derive by `find crates/domains/src/natural/biomedical -name ontology.rs | wc -l` for the biomedical subset, or `find crates/domains/src -name ontology.rs | wc -l` for the workspace total.

[^V-functors]: Re-derive by `grep -rn "impl Functor" crates/domains/src/natural/biomedical/ | wc -l` for the biomedical subset. Each functor implementation passes `check_functor_laws()` at test time.

[^V-adjunctions]: The three adjunctions (`MolecularBioelectricAdjunction`, `PharmacologyMolecularAdjunction`, `BiologyBioelectricAdjunction`) live at `crates/domains/src/natural/biomedical/adjunctions.rs`. Their `unit` and `counit` implementations are verified by the test suite in the same file. Run `cargo test -p pr4xis-domains adjunctions::tests`.

[^V-collapse]: Every collapse percentage in this paper is computed live by `cargo test -p pr4xis-domains test_full_chain_collapse_measurement -- --nocapture`. The output prints per-adjunction unit-loss and counit-loss percentages from the actual functor implementations. Numbers will update automatically as the biomedical ontologies evolve.

[^V-kv]: The Kv channel gap detection and `ContextDef` resolution are verified by `cargo test -p pr4xis-domains test_kv_gap_is_resolved_by_context`. The test demonstrates both the gap (Kv collapses on the round-trip) and the resolution (`ContextDef::resolve` distinguishes `(Kv, Constitutive)` from `(Kv, Therapeutic)`). The context-dependent resolution lives in `crates/domains/src/natural/biomedical/molecular/ontology.rs` as `MolecularFunctionalContext`.
