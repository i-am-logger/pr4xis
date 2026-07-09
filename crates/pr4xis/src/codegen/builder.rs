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
    /// Cross-references (`rdfs:seeAlso` / `skos:related`). Populated from
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

    /// Add a cross-reference (`rdfs:seeAlso` / `skos:related`).
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
