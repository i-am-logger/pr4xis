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
#[derive(Debug, Clone)]
pub struct GenerateConfig {
    pub module_name: String,
    pub entity_type_name: String,
    pub taxonomy_name: Option<String>,
    pub equivalence_name: Option<String>,
    pub opposition_name: Option<String>,
    pub mereology_name: Option<String>,
    pub causation_name: Option<String>,
}

impl GenerateConfig {
    pub fn new(module_name: &str, entity_type: &str) -> Self {
        Self {
            module_name: module_name.into(),
            entity_type_name: entity_type.into(),
            taxonomy_name: None,
            equivalence_name: None,
            opposition_name: None,
            mereology_name: None,
            causation_name: None,
        }
    }

    pub fn taxonomy(mut self, name: &str) -> Self {
        self.taxonomy_name = Some(name.into());
        self
    }

    pub fn equivalence(mut self, name: &str) -> Self {
        self.equivalence_name = Some(name.into());
        self
    }

    pub fn opposition(mut self, name: &str) -> Self {
        self.opposition_name = Some(name.into());
        self
    }

    pub fn mereology(mut self, name: &str) -> Self {
        self.mereology_name = Some(name.into());
        self
    }

    pub fn causation(mut self, name: &str) -> Self {
        self.causation_name = Some(name.into());
        self
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
    }
}
