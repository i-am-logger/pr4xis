/// Definition of a single entity (concept) in the ontology.
#[derive(Debug, Clone)]
pub struct EntityDef {
    pub id: String,
    pub label: String,
    pub pos: Option<String>,
    pub definitions: Vec<String>,
    pub examples: Vec<String>,
    pub lemmas: Vec<String>,
}

impl EntityDef {
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            pos: None,
            definitions: Vec::new(),
            examples: Vec::new(),
            lemmas: Vec::new(),
        }
    }

    pub fn pos(mut self, pos: &str) -> Self {
        self.pos = Some(pos.into());
        self
    }

    pub fn definition(mut self, def: &str) -> Self {
        self.definitions.push(def.into());
        self
    }

    pub fn lemma(mut self, lemma: &str) -> Self {
        self.lemmas.push(lemma.into());
        self
    }
}

/// Configuration for code generation.
///
/// Per-def trait names (taxonomy/equivalence/opposition/mereology/causation)
/// were removed in #168 alongside the per-def traits themselves. Relations
/// now travel through the codegen output as `RAW_*` arrays consumed by
/// `from_codegen` in `pr4xis-domains::cognitive::linguistics::language`.
///
/// `entity_marker_path` is the fully-qualified Rust path to the phantom
/// marker type `P` for the emitted `CodegenData<P>`. For English this is
/// `"pr4xis_domains::cognitive::linguistics::english::English"`; for
/// statute codegens whose marker is defined inline alongside the
/// generated module (e.g. the local `Sox1514aId` newtype) leave it as a
/// bare ident — the emitter detects single-ident paths and skips the
/// `use` declaration so the local definition resolves.
#[derive(Debug, Clone)]
pub struct GenerateConfig {
    pub module_name: String,
    pub entity_type_name: String,
    pub entity_marker_path: String,
}

impl GenerateConfig {
    /// Construct a config whose marker `P` is the same type the codegen
    /// emits inline (`entity_type_name`). Use this for statute codegens
    /// whose concept enum is defined locally in the generated module.
    pub fn new(module_name: &str, entity_type: &str) -> Self {
        Self {
            module_name: module_name.into(),
            entity_type_name: entity_type.into(),
            entity_marker_path: entity_type.into(),
        }
    }

    /// Construct a config with an external phantom-marker path. Use this
    /// when the marker type for `CodegenData<P>` lives in another crate
    /// — e.g. English uses
    /// `"pr4xis_domains::cognitive::linguistics::english::English"` so
    /// the wasm crate's `language::from_codegen(&CODEGEN_DATA)` consumer
    /// gets `CodegenData<English>` directly without a turbofish.
    pub fn with_marker(module_name: &str, entity_type: &str, entity_marker_path: &str) -> Self {
        Self {
            module_name: module_name.into(),
            entity_type_name: entity_type.into(),
            entity_marker_path: entity_marker_path.into(),
        }
    }
}

/// Builder for constructing an ontology from data.
///
/// Use this in build.rs to parse external data and generate
/// static Rust code implementing praxis reasoning traits.
#[derive(Debug, Clone, Default)]
pub struct OntologyBuilder {
    pub entities: Vec<EntityDef>,
    pub taxonomy: Vec<(String, String)>,
    pub equivalence: Vec<(String, String)>,
    pub opposition: Vec<(String, String)>,
    pub mereology: Vec<(String, String)>,
    pub causation: Vec<(String, String)>,
    /// Cross-references (SKOS `seeAlso` / `related`). Populated from
    /// WordNet's `also_synset` / `also_sense` on the English side.
    ///
    /// Literature: Miles & Bechhofer (2009) "SKOS Simple Knowledge
    /// Organization System Reference", W3C Recommendation §8.
    pub references: Vec<(String, String)>,
    /// Word text → entity IDs (for lookup generation)
    pub word_index: Vec<(String, String)>,
}

impl OntologyBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_entity(&mut self, entity: EntityDef) -> &mut Self {
        self.entities.push(entity);
        self
    }

    pub fn add_taxonomy(&mut self, child: &str, parent: &str) -> &mut Self {
        self.taxonomy.push((child.into(), parent.into()));
        self
    }

    pub fn add_equivalence(&mut self, a: &str, b: &str) -> &mut Self {
        self.equivalence.push((a.into(), b.into()));
        self
    }

    pub fn add_opposition(&mut self, a: &str, b: &str) -> &mut Self {
        self.opposition.push((a.into(), b.into()));
        self
    }

    pub fn add_mereology(&mut self, whole: &str, part: &str) -> &mut Self {
        self.mereology.push((whole.into(), part.into()));
        self
    }

    pub fn add_causation(&mut self, cause: &str, effect: &str) -> &mut Self {
        self.causation.push((cause.into(), effect.into()));
        self
    }

    /// Add a SKOS-style cross-reference (`seeAlso` / `related`).
    pub fn add_reference(&mut self, from: &str, to: &str) -> &mut Self {
        self.references.push((from.into(), to.into()));
        self
    }

    pub fn add_word_index(&mut self, word: &str, entity_id: &str) -> &mut Self {
        self.word_index.push((word.into(), entity_id.into()));
        self
    }

    /// Generate Rust source code from the collected data.
    pub fn generate(&self, config: &GenerateConfig) -> String {
        super::generate::generate_rust(self, config)
    }

    /// Project the built ontology into the owned archival shape
    /// ([`crate::archive::OwnedCodegenData`]) — the build→archive bridge.
    ///
    /// Produces the *same* columnar data the generated `CODEGEN_DATA`
    /// static would carry (see [`super::generate::generate_rust`]):
    /// entity index = position in [`Self::entities`]; `entity_kind` =
    /// `pos` (empty if none); `entity_defs` = first definition; edge and
    /// word-index string-ids are mapped to those indices. The word index
    /// is grouped by word and word-sorted so the runtime's binary-search
    /// `lookup` holds. Edges whose endpoints aren't known entities are
    /// dropped (same as codegen, which can only reference emitted ids).
    ///
    /// This lets build.rs serialize a source to a content-addressed
    /// `.rkyv` blob instead of (or alongside) a Rust `static` — the
    /// delivery format for non-embedded stagings. See
    /// [`crate::archive`].
    pub fn to_owned_codegen_data(&self) -> crate::archive::OwnedCodegenData {
        use alloc::collections::BTreeMap;

        let id_map: BTreeMap<&str, u64> = self
            .entities
            .iter()
            .enumerate()
            .map(|(i, e)| (e.id.as_str(), i as u64))
            .collect();

        let map_edge = |pair: &(String, String)| -> Option<(u64, u64)> {
            Some((*id_map.get(pair.0.as_str())?, *id_map.get(pair.1.as_str())?))
        };
        let map_edges = |edges: &[(String, String)]| -> Vec<(u64, u64)> {
            edges.iter().filter_map(map_edge).collect()
        };

        // Group the (word, id) pairs by word; word-sorted via BTreeMap so
        // the runtime `CodegenData::lookup` binary search is valid.
        let mut by_word: BTreeMap<String, Vec<u64>> = BTreeMap::new();
        for (word, id) in &self.word_index {
            if let Some(&idx) = id_map.get(id.as_str()) {
                by_word.entry(word.clone()).or_default().push(idx);
            }
        }

        crate::archive::OwnedCodegenData {
            entity_count: self.entities.len() as u64,
            entity_ids: self.entities.iter().map(|e| e.id.clone()).collect(),
            entity_kind: self
                .entities
                .iter()
                .map(|e| e.pos.clone().unwrap_or_default())
                .collect(),
            entity_labels: self.entities.iter().map(|e| e.label.clone()).collect(),
            entity_defs: self
                .entities
                .iter()
                .map(|e| e.definitions.first().cloned().unwrap_or_default())
                .collect(),
            word_index: by_word.into_iter().collect(),
            taxonomy: map_edges(&self.taxonomy),
            mereology: map_edges(&self.mereology),
            opposition: map_edges(&self.opposition),
            equivalence: map_edges(&self.equivalence),
            causation: map_edges(&self.causation),
            references: map_edges(&self.references),
        }
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn relation_count(&self) -> usize {
        self.taxonomy.len()
            + self.equivalence.len()
            + self.opposition.len()
            + self.mereology.len()
            + self.causation.len()
            + self.references.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_builder() -> OntologyBuilder {
        let mut b = OntologyBuilder::new();
        b.add_entity(
            EntityDef::new("oewn-dog-n", "dog")
                .pos("n")
                .definition("a domesticated carnivore")
                .lemma("dog"),
        );
        b.add_entity(
            EntityDef::new("oewn-animal-n", "animal")
                .pos("n")
                .definition("a living organism"),
        );
        b.add_entity(EntityDef::new("oewn-being-n", "being").pos("n"));
        b.add_taxonomy("oewn-dog-n", "oewn-animal-n");
        b.add_taxonomy("oewn-animal-n", "oewn-being-n");
        b.add_reference("oewn-dog-n", "oewn-being-n");
        b.add_word_index("dog", "oewn-dog-n");
        b.add_word_index("animal", "oewn-animal-n");
        b
    }

    #[test]
    fn to_owned_codegen_data_matches_columns() {
        let owned = sample_builder().to_owned_codegen_data();
        assert_eq!(owned.entity_count, 3);
        assert_eq!(
            owned.entity_ids,
            ["oewn-dog-n", "oewn-animal-n", "oewn-being-n"]
        );
        assert_eq!(owned.entity_kind, ["n", "n", "n"]);
        assert_eq!(owned.entity_labels, ["dog", "animal", "being"]);
        // First definition per entity; empty when none.
        assert_eq!(
            owned.entity_defs,
            ["a domesticated carnivore", "a living organism", ""]
        );
        // Edges map string ids to entity indices.
        assert_eq!(owned.taxonomy, [(0, 1), (1, 2)]);
        assert_eq!(owned.references, [(0, 2)]);
        // Word index is grouped and word-sorted.
        assert_eq!(
            owned.word_index,
            [
                ("animal".to_string(), vec![1u64]),
                ("dog".to_string(), vec![0u64])
            ]
        );
    }

    #[test]
    fn dropped_edge_when_endpoint_unknown() {
        let mut b = sample_builder();
        // An edge referencing an id with no entity must be dropped, the
        // same as codegen (which can only reference emitted ids).
        b.add_taxonomy("oewn-dog-n", "not-an-entity");
        let owned = b.to_owned_codegen_data();
        assert_eq!(owned.taxonomy, [(0, 1), (1, 2)]);
    }

    #[test]
    fn round_trips_through_rkyv_and_lookup_holds() {
        struct P;
        let owned = sample_builder().to_owned_codegen_data();
        let bytes = crate::archive::to_archive_bytes(&owned).expect("serialize");
        let back = crate::archive::from_archive_bytes(&bytes).expect("deserialize");
        assert_eq!(owned, back);
        // The leaked view's binary-search lookup must resolve.
        let cg: crate::codegen_data::CodegenData<P> = back.to_codegen_data_leaked();
        assert_eq!(cg.lookup("dog")[0].value(), 0);
        assert_eq!(cg.lookup("animal")[0].value(), 1);
        assert!(cg.lookup("being").is_empty());
    }
}
