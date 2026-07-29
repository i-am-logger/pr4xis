# Citings — SUMO -- WordNet↔Upper-Ontology Mapping

Every published source this ontology stands on. Entries below are drawn from the ontology's [README.md](README.md) and the doc comments on its ontology, reader, store, and regen modules.

## Primary sources

- Niles, I. & Pease, A. (2001). "Towards a Standard Upper Ontology." *Proceedings of the 2nd International Conference on Formal Ontology in Information Systems (FOIS 2001)*, pp. 2-9. SUMO itself, the upper ontology the loaded terms belong to.
- Niles, I. & Pease, A. (2003). "Linking Lexicons and Ontologies: Mapping WordNet to the Suggested Upper Merged Ontology." *Proceedings of the IEEE International Conference on Information and Knowledge Engineering (IKE 2003)*, pp. 412-416. The WordNet↔SUMO mapping methodology and the six-suffix relation legend (`=`/`+`/`@`/`:`/`[`/`]`) `ontology.rs`'s `SumoRelationKind` quotes verbatim from the source file's own header comment.
- `ontologyportal/sumo` GitHub repository (<https://github.com/ontologyportal/sumo>), branch `master`, commit `152b9abc440477073b5e4e573983b730a619da7b` (pinned 2026-07-14 — no tagged release exists for the `WordNetMappings/` subdirectory). Source of the loaded `WordNetMappings30-{noun,verb,adj,adv}.txt` files and the GPL license notice, reproduced in `crates/domains/data/sumo/sumo-LICENSE.txt`.
- Fellbaum, C., ed. (1998). *WordNet: An Electronic Lexical Database*. MIT Press. `wndb(5WN)` man page conventions — the WNDB `data.*` synset-record format (`offset lex_filenum ss_type w_cnt (word lex_id)+ p_cnt (pointer)* | gloss`) and the sense-key format (`lemma%ss_type:lex_filenum:lex_id[:head_word:head_id]`) `regenerate.rs`'s `parse_data_line`/`oewn_sense_id_for_member` implement.
- Princeton University WordNet License (<https://wordnet.princeton.edu/license-and-commercial-use>). Governs the underlying Princeton WordNet 3.0 `data.*` content each SUMO line annotates (distinct from, and more permissive than, the GPL that governs the SUMO annotation layer itself — see `sumo-LICENSE.txt`'s "Neither notice specifies a GPL version" section for the full disambiguation).
- Matentzoglu, N. et al. (2022). "A Simple Standard for Sharing Ontological Mappings (SSSOM)." *Database* (Oxford), Vol. 2022, baac035. <https://doi.org/10.1093/database/baac035>. `sssom.rs`'s `SssomMapping`/`SssomMappingSet` shape (mandatory-field subset) and `formal::information::schema::sssom`'s data model.
- SEMAPV — Semantic Mapping Vocabulary (<https://github.com/mapping-commons/semantic-mapping-vocabulary>). Source of the `semapv:ManualMappingCuration` justification CURIE `sssom.rs` mints for the Niles & Pease (2003) hand-curated crosswalk.

## Cross-references

- Workspace bibliography: [`docs/papers/references.md`](../../../../../../docs/papers/references.md)
- The full literature-grounded rationale for scoping cross-source corroboration by relation kind (including VerbNet's measured regression that established the discipline this ontology's own consultation follows) is in `english/word_sense.rs`'s module doc comment, not repeated here.
- The offline sense-key resolution pattern this ontology's `regenerate.rs` follows is VerbNet's own (`cognitive::linguistics::verbnet::store`'s module doc, `oewn_sense_id_for_sense_key`) — see that module for the citation grounding the OEWN `Sense`-id derivation itself.
- Code-level citations: `grep -n 'Niles\|Pease\|SUMO' *.rs` in this directory and in `crates/chat/src/lib.rs`.

## Pending verification

Every entry under **Primary sources** is a short pointer. For each one, confirm that a full citation (Author, Year, Title, DOI/URL) exists in `docs/papers/references.md`. Where no entry exists, add it (or a local PDF under a `papers/` subdirectory) before declaring the ontology citation-complete.

Open items for human review:

- [ ] Cross-check every primary source against `docs/papers/references.md`
- [ ] Confirm the `ontologyportal/sumo` `master` branch pin (commit `152b9abc`) remains resolvable at re-fetch time — unlike FrameNet/ConceptNet/VerbNet, this source has no tagged release to pin against, so drift is possible upstream between now and any future `pr4xis update sumo --lock`
- [ ] The GPL version ambiguity noted in `sumo-LICENSE.txt` (the source's own notices link only to the generic `gnu.org/copyleft/gpl.html` landing page, never a specific GPLv2/GPLv3 text) is the SOURCE's ambiguity, not introduced here — flag for legal review if this repository is ever commercially exploited (per the user-approved carve-out: this one file remains GPL, distinct from the repository's own CC BY-NC-SA 4.0 claim)
- [ ] If this ontology depends on a paper not yet in the workspace bibliography, move/copy the PDF into a local `papers/` subdirectory and link it from the primary source line above

---

- **Document date:** 2026-07-14
- **How this file is maintained:** hand-authored (not via the `per-ontology-rollout` skill — this directory's `ontology.rs` is a typed instance-data loader, mirroring `conceptnet/ontology.rs`'s and `framenet/ontology.rs`'s shape, not a `pr4xis::ontology!`-macro vocabulary, so the skill's validation step does not apply). Update by hand as code-comment citations, local PDFs, and `docs/papers/references.md` entries are added.
