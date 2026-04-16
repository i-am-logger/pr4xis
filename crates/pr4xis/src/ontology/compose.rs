// Compose API — runtime ontology composition via Heim's Korporator.
//
// The Korporator operates in two modes (Heim 1980, Teil A §3.1):
//   Coupling (Kopplung): link ontologies while preserving structure = Coproduct (A || B)
//   Composition (Mischung): merge into unified structure = Pushout over span
//
// Total Korporator: composes all aspects (full merge)
// Partial Korporator: selectively composes specified aspects
//
// The builder pattern IS the formation process (Hatchuel & Weil C-K Theory 2009):
//   concept() = expand concept space
//   is_a() = structure via taxonomy
//   compose() = Korporator coupling
//   build() = freeze into validated ontology
//
// The Ontology carries its complete self-description: structure, lexical
// metadata (Lemon), source citations, DOLCE classification, provenance.
// build.rs freezes this into static binary data (first Futamura projection).
//
// Source: Heim "Syntrometrische Maximentelezentrik" (1980) §3.1-3.2;
//         Goguen & Burstall "Institutions" (1992);
//         Spivak "Functorial Data Migration" (2012);
//         Hatchuel & Weil "C-K Design Theory" (2009);
//         McCrae et al. "The Lemon Cookbook" (W3C 2016)

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ontology::Vocabulary;
use crate::ontology::upper::being::Being;

/// Lexical metadata for a concept — Ontolex-Lemon (W3C 2016).
///
/// Each concept carries its labels, definitions, and senses
/// in the ontology's source language. This is what gets frozen
/// into the binary alongside the structure.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Lexical {
    pub label: String,
    pub definition: String,
    pub language: String,
}

impl Lexical {
    pub fn new(label: &str, definition: &str, language: &str) -> Self {
        Self {
            label: String::from(label),
            definition: String::from(definition),
            language: String::from(language),
        }
    }
}

/// A concept in a composed ontology — an object with lexical metadata.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Concept {
    pub name: String,
    pub lexical: Option<Lexical>,
}

/// A morphism in a composed ontology — a directed, typed edge.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub kind: EdgeKind,
}

/// The kind of morphism — corresponds to reasoning systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EdgeKind {
    Identity,
    IsA,
    HasA,
    Causes,
    Opposes,
    Custom,
}

/// How an ontology arrived in the system — observable staging quality.
///
/// Maps directly to the Staging ontology (Futamura 1971):
///   Embedded = StaticInput (first Futamura projection, frozen at build time)
///   Async = DynamicInput (loaded from network/disk at runtime)
///   Mmap = DynamicInput (memory-mapped file, demand-paged by OS)
///   Composed = DynamicInput (runtime Korporator composition)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Staging {
    Embedded,
    Async,
    Mmap,
    Composed,
}

/// A runtime ontology — the Ontology in Heim's terminology.
///
/// Carries its complete self-description: structure (concepts + edges),
/// lexical metadata (Lemon labels + definitions), source citations,
/// DOLCE classification, staging, and provenance.
///
/// build.rs freezes this into static binary data. At runtime, pr4xis
/// can answer "what is X?" without constructing anything.
///
/// Ontology = <f, z, m> where:
///   z (Metrophor) = the identity/essence (name + being)
///   f (Synkolator) = the generative law (edges)
///   m = the synkolation level (depth of composition)
#[derive(Debug, Clone)]
pub struct Ontology {
    name: String,
    source: String,
    being: Option<Being>,
    concepts: BTreeMap<String, Concept>,
    edges: BTreeSet<Edge>,
    level: usize,
    staging: Staging,
    provenance: Vec<String>,
}

impl Ontology {
    /// Begin building a new ontology.
    pub fn create(name: &str) -> OntologyBuilder {
        OntologyBuilder {
            name: String::from(name),
            source: String::new(),
            being: None,
            concepts: BTreeMap::new(),
            edges: BTreeSet::new(),
            base_level: 0,
            provenance: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn being(&self) -> Option<Being> {
        self.being
    }

    pub fn concept_names(&self) -> BTreeSet<String> {
        self.concepts.keys().cloned().collect()
    }

    pub fn concept(&self, name: &str) -> Option<&Concept> {
        self.concepts.get(name)
    }

    pub fn edges(&self) -> &BTreeSet<Edge> {
        &self.edges
    }

    pub fn concept_count(&self) -> usize {
        self.concepts.len()
    }

    pub fn morphism_count(&self) -> usize {
        self.edges.len()
    }

    /// The synkolation level — depth of composition.
    /// Level 0 = base ontology. Each compose() increments.
    pub fn level(&self) -> usize {
        self.level
    }

    /// How this ontology arrived in the system — self-observation.
    pub fn staging(&self) -> Staging {
        self.staging
    }

    /// Provenance chain — which ontologies were composed to produce this one.
    pub fn provenance(&self) -> &[String] {
        &self.provenance
    }

    /// Coupling (Kopplung) — Korporator mode 1.
    ///
    /// Links two ontologies while preserving their individual structure.
    /// This is the Coproduct (A || B): all concepts from both, no identification.
    ///
    /// Heim: "Total Korporator" when both ontologies are fully included.
    pub fn couple(&self, other: &Ontology) -> Ontology {
        let mut concepts = self.concepts.clone();
        for (k, v) in &other.concepts {
            concepts.entry(k.clone()).or_insert_with(|| v.clone());
        }

        let mut edges = self.edges.clone();
        edges.extend(other.edges.iter().cloned());

        let provenance =
            merge_provenance(&self.provenance, &self.name, &other.provenance, &other.name);

        Ontology {
            name: alloc::format!("{}||{}", self.name, other.name),
            source: alloc::format!("{} + {}", self.source, other.source),
            being: self.being.or(other.being),
            concepts,
            edges,
            level: core::cmp::max(self.level, other.level) + 1,
            staging: Staging::Composed,
            provenance,
        }
    }

    /// Composition (Mischung) — Korporator mode 2.
    ///
    /// Merges two ontologies into a unified structure by identifying shared concepts.
    /// This is the Pushout over a span: concepts with the same name are unified.
    ///
    /// Shared concepts become the span; non-shared concepts are preserved.
    /// Edges referencing shared concepts are rewritten to the unified form.
    pub fn compose(&self, other: &Ontology) -> Ontology {
        let shared: BTreeSet<String> = self
            .concepts
            .keys()
            .filter(|k| other.concepts.contains_key(*k))
            .cloned()
            .collect();

        let mut concepts = self.concepts.clone();
        for (k, v) in &other.concepts {
            concepts.entry(k.clone()).or_insert_with(|| v.clone());
        }

        let mut edges = self.edges.clone();
        edges.extend(other.edges.iter().cloned());

        let provenance =
            merge_provenance(&self.provenance, &self.name, &other.provenance, &other.name);

        let suffix = if shared.is_empty() {
            alloc::format!("{}||{}", self.name, other.name)
        } else {
            alloc::format!("{}&{}", self.name, other.name)
        };

        Ontology {
            name: suffix,
            source: alloc::format!("{} + {}", self.source, other.source),
            being: self.being.or(other.being),
            concepts,
            edges,
            level: core::cmp::max(self.level, other.level) + 1,
            staging: Staging::Composed,
            provenance,
        }
    }

    /// Partial Korporator — selective coupling.
    ///
    /// Couples only the specified concepts from the other ontology.
    /// Heim: "Partielle Ontologykorporationen" (§3.2).
    pub fn couple_partial(&self, other: &Ontology, selected: &[&str]) -> Ontology {
        let selected_set: BTreeSet<String> = selected.iter().map(|s| String::from(*s)).collect();

        let mut concepts = self.concepts.clone();
        for (k, v) in &other.concepts {
            if selected_set.contains(k) {
                concepts.entry(k.clone()).or_insert_with(|| v.clone());
            }
        }

        let all_kept: BTreeSet<String> = concepts.keys().cloned().collect();
        let mut edges = self.edges.clone();
        for e in &other.edges {
            if all_kept.contains(&e.from) && all_kept.contains(&e.to) {
                edges.insert(e.clone());
            }
        }

        let provenance =
            merge_provenance(&self.provenance, &self.name, &other.provenance, &other.name);

        Ontology {
            name: alloc::format!("{}+partial({})", self.name, other.name),
            source: alloc::format!("{} + {} (partial)", self.source, other.source),
            being: self.being.or(other.being),
            concepts,
            edges,
            level: core::cmp::max(self.level, other.level) + 1,
            staging: Staging::Composed,
            provenance,
        }
    }

    /// Specialize — restrict to a sub-ontology via taxonomy.
    ///
    /// Returns a new Ontology containing only the specified concept
    /// and everything reachable from it via IsA edges (its descendants).
    /// This is Delta migration: pulling back through the taxonomy functor.
    pub fn specialize(&self, root: &str) -> Option<Ontology> {
        if !self.concepts.contains_key(root) {
            return None;
        }

        let mut reachable = BTreeSet::new();
        reachable.insert(String::from(root));

        let mut changed = true;
        while changed {
            changed = false;
            for edge in &self.edges {
                if edge.kind == EdgeKind::IsA
                    && reachable.contains(&edge.to)
                    && reachable.insert(edge.from.clone())
                {
                    changed = true;
                }
            }
        }

        let concepts: BTreeMap<String, Concept> = self
            .concepts
            .iter()
            .filter(|(k, _)| reachable.contains(*k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let edges: BTreeSet<Edge> = self
            .edges
            .iter()
            .filter(|e| reachable.contains(&e.from) && reachable.contains(&e.to))
            .cloned()
            .collect();

        Some(Ontology {
            name: alloc::format!("{}[{}]", self.name, root),
            source: self.source.clone(),
            being: self.being,
            concepts,
            edges,
            level: self.level,
            staging: self.staging,
            provenance: self.provenance.clone(),
        })
    }

    /// Query — Delta migration (Spivak 2012).
    ///
    /// Given a concept name, returns all concepts related to it
    /// and the edges connecting them. This is ΔF: pulling data
    /// backward along the query functor.
    pub fn query(&self, name: &str) -> Vec<&Edge> {
        self.edges
            .iter()
            .filter(|e| e.from == name || e.to == name)
            .collect()
    }

    /// Produce a Vocabulary from this runtime Ontology.
    ///
    /// Leaks the name and source strings once per call. For long-running
    /// processes, call this once and cache the result.
    pub fn vocabulary(&self) -> Vocabulary {
        Vocabulary {
            ontology_name: Box::leak(self.name.clone().into_boxed_str()),
            module_path: "runtime::compose",
            source: Box::leak(self.source.clone().into_boxed_str()),
            being: self.being,
            concept_count: self.concepts.len(),
            morphism_count: self.edges.len(),
        }
    }

    /// The shared concepts between two ontologies — the span.
    pub fn shared_with(&self, other: &Ontology) -> BTreeSet<String> {
        self.concepts
            .keys()
            .filter(|k| other.concepts.contains_key(*k))
            .cloned()
            .collect()
    }

    /// Validate structural integrity.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for edge in &self.edges {
            if !self.concepts.contains_key(&edge.from) {
                errors.push(alloc::format!(
                    "edge references unknown source concept: {}",
                    edge.from
                ));
            }
            if !self.concepts.contains_key(&edge.to) {
                errors.push(alloc::format!(
                    "edge references unknown target concept: {}",
                    edge.to
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    // =========================================================================
    // Modification operations — ontology evolution via composition.
    // Never mutate. Always produce a new ontology.
    // Source: Spivak "Functorial Data Migration" (2012);
    //         memory feedback_evolution_via_functor.md
    // =========================================================================

    /// Extend — returns a builder pre-populated with this ontology's data.
    ///
    /// Add concepts, edges, lexical metadata, then build() a new ontology.
    /// The original is unchanged. Provenance tracks the extension.
    pub fn extend(&self) -> OntologyBuilder {
        OntologyBuilder {
            name: self.name.clone(),
            source: self.source.clone(),
            being: self.being,
            concepts: self.concepts.clone(),
            edges: self
                .edges
                .iter()
                .filter(|e| e.kind != EdgeKind::Identity)
                .cloned()
                .collect(),
            base_level: self.level,
            provenance: {
                let mut p = self.provenance.clone();
                if !p.contains(&self.name) {
                    p.push(self.name.clone());
                }
                p
            },
        }
    }

    /// Without — returns new ontology with a concept and its edges removed.
    ///
    /// All edges referencing the concept are also removed.
    /// The original is unchanged.
    pub fn without(&self, concept: &str) -> Ontology {
        let mut concepts = self.concepts.clone();
        concepts.remove(concept);

        let edges: BTreeSet<Edge> = self
            .edges
            .iter()
            .filter(|e| e.from != concept && e.to != concept)
            .cloned()
            .collect();

        let mut provenance = self.provenance.clone();
        if !provenance.contains(&self.name) {
            provenance.push(self.name.clone());
        }

        Ontology {
            name: alloc::format!("{}-{}", self.name, concept),
            source: self.source.clone(),
            being: self.being,
            concepts,
            edges,
            level: self.level,
            staging: Staging::Composed,
            provenance,
        }
    }

    /// Restrict — returns new ontology containing only the specified concepts.
    ///
    /// Only edges where both endpoints are in the set are kept.
    /// Unlike specialize() which follows taxonomy, this takes an explicit set.
    pub fn restrict(&self, keep: &[&str]) -> Ontology {
        let keep_set: BTreeSet<String> = keep.iter().map(|s| String::from(*s)).collect();

        let concepts: BTreeMap<String, Concept> = self
            .concepts
            .iter()
            .filter(|(k, _)| keep_set.contains(*k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let edges: BTreeSet<Edge> = self
            .edges
            .iter()
            .filter(|e| keep_set.contains(&e.from) && keep_set.contains(&e.to))
            .cloned()
            .collect();

        let mut provenance = self.provenance.clone();
        if !provenance.contains(&self.name) {
            provenance.push(self.name.clone());
        }

        Ontology {
            name: alloc::format!("{}|restricted", self.name),
            source: self.source.clone(),
            being: self.being,
            concepts,
            edges,
            level: self.level,
            staging: Staging::Composed,
            provenance,
        }
    }

    /// Rename — returns new ontology with a concept renamed.
    ///
    /// All edges referencing the old name are updated.
    /// Provenance tracks the rename.
    pub fn rename(&self, old: &str, new: &str) -> Ontology {
        let mut concepts = BTreeMap::new();
        for (k, v) in &self.concepts {
            let key = if k == old {
                String::from(new)
            } else {
                k.clone()
            };
            let mut concept = v.clone();
            if concept.name == old {
                concept.name = String::from(new);
            }
            concepts.insert(key, concept);
        }

        let edges: BTreeSet<Edge> = self
            .edges
            .iter()
            .map(|e| Edge {
                from: if e.from == old {
                    String::from(new)
                } else {
                    e.from.clone()
                },
                to: if e.to == old {
                    String::from(new)
                } else {
                    e.to.clone()
                },
                kind: e.kind,
            })
            .collect();

        let mut provenance = self.provenance.clone();
        if !provenance.contains(&self.name) {
            provenance.push(self.name.clone());
        }

        Ontology {
            name: alloc::format!("{}[{}→{}]", self.name, old, new),
            source: self.source.clone(),
            being: self.being,
            concepts,
            edges,
            level: self.level,
            staging: Staging::Composed,
            provenance,
        }
    }

    /// Evolve — functor-based transformation producing a new ontology.
    ///
    /// Applies a concept name mapping. Concepts not in the mapping are kept as-is.
    /// This is the runtime equivalent of a typed Functor's map_object.
    pub fn evolve(&self, mapping: &[(&str, &str)]) -> Ontology {
        let map: BTreeMap<&str, &str> = mapping.iter().copied().collect();

        let resolve = |name: &str| -> String {
            map.get(name)
                .map(|s| String::from(*s))
                .unwrap_or_else(|| String::from(name))
        };

        let mut concepts = BTreeMap::new();
        for (k, v) in &self.concepts {
            let new_name = resolve(k);
            let mut concept = v.clone();
            concept.name = new_name.clone();
            concepts.insert(new_name, concept);
        }

        let edges: BTreeSet<Edge> = self
            .edges
            .iter()
            .map(|e| Edge {
                from: resolve(&e.from),
                to: resolve(&e.to),
                kind: e.kind,
            })
            .collect();

        let mut provenance = self.provenance.clone();
        if !provenance.contains(&self.name) {
            provenance.push(self.name.clone());
        }

        Ontology {
            name: alloc::format!("{}→evolved", self.name),
            source: self.source.clone(),
            being: self.being,
            concepts,
            edges,
            level: self.level + 1,
            staging: Staging::Composed,
            provenance,
        }
    }
}

fn merge_provenance(
    left: &[String],
    left_name: &str,
    right: &[String],
    right_name: &str,
) -> Vec<String> {
    let mut provenance = left.to_vec();
    provenance.extend(right.iter().cloned());
    if !provenance.contains(&String::from(left_name)) {
        provenance.insert(0, String::from(left_name));
    }
    if !provenance.contains(&String::from(right_name)) {
        provenance.push(String::from(right_name));
    }
    provenance
}

/// Builder for constructing an Ontology step by step.
///
/// The builder IS the formation process (C-K Theory, Hatchuel & Weil 2009):
/// each method call expands the concept space or adds structure.
pub struct OntologyBuilder {
    name: String,
    source: String,
    being: Option<Being>,
    concepts: BTreeMap<String, Concept>,
    edges: BTreeSet<Edge>,
    base_level: usize,
    provenance: Vec<String>,
}

impl OntologyBuilder {
    pub fn source(mut self, source: &str) -> Self {
        self.source = String::from(source);
        self
    }

    pub fn being(mut self, being: Being) -> Self {
        self.being = Some(being);
        self
    }

    /// Add a concept (no lexical metadata yet — add via label/definition).
    pub fn concept(mut self, name: &str) -> Self {
        self.concepts.entry(String::from(name)).or_insert(Concept {
            name: String::from(name),
            lexical: None,
        });
        self
    }

    /// Add multiple concepts at once.
    pub fn concepts(mut self, names: &[&str]) -> Self {
        for name in names {
            self.concepts.entry(String::from(*name)).or_insert(Concept {
                name: String::from(*name),
                lexical: None,
            });
        }
        self
    }

    /// Set the label for a concept (Lemon: ontolex:writtenRep).
    pub fn label(mut self, concept: &str, lang: &str, label: &str) -> Self {
        let entry = self
            .concepts
            .entry(String::from(concept))
            .or_insert(Concept {
                name: String::from(concept),
                lexical: None,
            });
        if let Some(ref mut lex) = entry.lexical {
            lex.label = String::from(label);
            lex.language = String::from(lang);
        } else {
            entry.lexical = Some(Lexical::new(label, "", lang));
        }
        self
    }

    /// Set the definition for a concept (Lemon: skos:definition).
    pub fn definition(mut self, concept: &str, lang: &str, def: &str) -> Self {
        let entry = self
            .concepts
            .entry(String::from(concept))
            .or_insert(Concept {
                name: String::from(concept),
                lexical: None,
            });
        if let Some(ref mut lex) = entry.lexical {
            lex.definition = String::from(def);
            lex.language = String::from(lang);
        } else {
            entry.lexical = Some(Lexical::new("", def, lang));
        }
        self
    }

    pub fn is_a(mut self, child: &str, parent: &str) -> Self {
        self.ensure_concept(child);
        self.ensure_concept(parent);
        self.edges.insert(Edge {
            from: String::from(child),
            to: String::from(parent),
            kind: EdgeKind::IsA,
        });
        self
    }

    pub fn has_a(mut self, whole: &str, part: &str) -> Self {
        self.ensure_concept(whole);
        self.ensure_concept(part);
        self.edges.insert(Edge {
            from: String::from(whole),
            to: String::from(part),
            kind: EdgeKind::HasA,
        });
        self
    }

    pub fn causes(mut self, cause: &str, effect: &str) -> Self {
        self.ensure_concept(cause);
        self.ensure_concept(effect);
        self.edges.insert(Edge {
            from: String::from(cause),
            to: String::from(effect),
            kind: EdgeKind::Causes,
        });
        self
    }

    pub fn opposes(mut self, a: &str, b: &str) -> Self {
        self.ensure_concept(a);
        self.ensure_concept(b);
        self.edges.insert(Edge {
            from: String::from(a),
            to: String::from(b),
            kind: EdgeKind::Opposes,
        });
        self
    }

    pub fn edge(mut self, from: &str, to: &str, kind: EdgeKind) -> Self {
        self.ensure_concept(from);
        self.ensure_concept(to);
        self.edges.insert(Edge {
            from: String::from(from),
            to: String::from(to),
            kind,
        });
        self
    }

    /// Build the Ontology. Adds identity edges for all concepts.
    pub fn build(mut self) -> Ontology {
        for name in self.concepts.keys().cloned().collect::<Vec<_>>() {
            self.edges.insert(Edge {
                from: name.clone(),
                to: name,
                kind: EdgeKind::Identity,
            });
        }

        Ontology {
            name: self.name,
            source: self.source,
            being: self.being,
            concepts: self.concepts,
            edges: self.edges,
            level: self.base_level,
            staging: Staging::Composed,
            provenance: self.provenance,
        }
    }

    fn ensure_concept(&mut self, name: &str) {
        self.concepts.entry(String::from(name)).or_insert(Concept {
            name: String::from(name),
            lexical: None,
        });
    }
}

/// Load a static ontology's Vocabulary into a Ontology skeleton.
pub fn from_vocabulary(vocab: &Vocabulary) -> Ontology {
    Ontology {
        name: String::from(vocab.ontology_name),
        source: String::from(vocab.source),
        being: vocab.being,
        concepts: BTreeMap::new(),
        edges: BTreeSet::new(),
        level: 0,
        staging: Staging::Embedded,
        provenance: Vec::new(),
    }
}

/// A Metroplex — a Ontology-of-Syntrices (Heim).
///
/// Recursive hierarchical composition: each Ontology at one level
/// becomes a base element at the next. The Metroplex tracks the
/// full hierarchy and enforces coherence across grades.
///
/// Metroplex grades correspond to Staging levels (Futamura):
///   Grade 0 = base ontologies
///   Grade 1 = composed ontologies
///   Grade 2 = ontologies-of-ontologies (meta-level)
#[derive(Debug, Clone)]
pub struct Metroplex {
    name: String,
    grades: BTreeMap<usize, Vec<Ontology>>,
}

impl Metroplex {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            grades: BTreeMap::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add(&mut self, syntrix: Ontology) {
        let level = syntrix.level();
        self.grades.entry(level).or_default().push(syntrix);
    }

    pub fn grade(&self, level: usize) -> &[Ontology] {
        self.grades.get(&level).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn grade_count(&self) -> usize {
        self.grades.len()
    }

    pub fn total_ontologies(&self) -> usize {
        self.grades.values().map(|v| v.len()).sum()
    }

    /// All vocabularies across all grades.
    pub fn vocabularies(&self) -> Vec<Vocabulary> {
        self.grades
            .values()
            .flat_map(|syntrices| syntrices.iter().map(|s| s.vocabulary()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_build() {
        let bio = Ontology::create("Biology")
            .source("Mayr (1982)")
            .being(Being::AbstractObject)
            .concept("Animal")
            .concept("Mammal")
            .concept("Dog")
            .is_a("Dog", "Mammal")
            .is_a("Mammal", "Animal")
            .build();

        assert_eq!(bio.name(), "Biology");
        assert_eq!(bio.concept_count(), 3);
        assert_eq!(bio.level(), 0);
        assert_eq!(bio.staging(), Staging::Composed);
        assert!(bio.validate().is_ok());
    }

    #[test]
    fn concept_with_lexical_metadata() {
        let bio = Ontology::create("Biology")
            .source("Mayr (1982)")
            .concept("Cell")
            .label("Cell", "en", "Cell")
            .definition("Cell", "en", "The basic structural unit of all organisms")
            .build();

        let cell = bio.concept("Cell").unwrap();
        let lex = cell.lexical.as_ref().unwrap();
        assert_eq!(lex.label, "Cell");
        assert_eq!(lex.language, "en");
        assert!(lex.definition.contains("structural unit"));
    }

    #[test]
    fn couple_is_coproduct() {
        let bio = Ontology::create("Biology")
            .concept("Animal")
            .concept("Cell")
            .is_a("Cell", "Animal")
            .build();

        let chem = Ontology::create("Chemistry")
            .concept("Molecule")
            .concept("Atom")
            .has_a("Molecule", "Atom")
            .build();

        let coupled = bio.couple(&chem);

        assert_eq!(coupled.concept_count(), 4);
        assert!(coupled.concept("Animal").is_some());
        assert!(coupled.concept("Molecule").is_some());
        assert_eq!(coupled.level(), 1);
        assert_eq!(coupled.staging(), Staging::Composed);
        assert!(coupled.name().contains("||"));
    }

    #[test]
    fn compose_identifies_shared() {
        let bio = Ontology::create("Biology")
            .concept("Cell")
            .concept("Organism")
            .is_a("Cell", "Organism")
            .build();

        let molecular = Ontology::create("Molecular")
            .concept("Cell")
            .concept("Protein")
            .has_a("Cell", "Protein")
            .build();

        let shared = bio.shared_with(&molecular);
        assert!(shared.contains("Cell"));
        assert_eq!(shared.len(), 1);

        let composed = bio.compose(&molecular);
        assert_eq!(composed.concept_count(), 3);
        assert!(composed.name().contains("&"));
    }

    #[test]
    fn partial_couple() {
        let full = Ontology::create("Full")
            .concept("A")
            .concept("B")
            .concept("C")
            .is_a("A", "B")
            .is_a("B", "C")
            .build();

        let base = Ontology::create("Base").concept("X").build();

        let partial = base.couple_partial(&full, &["A", "B"]);

        assert!(partial.concept("A").is_some());
        assert!(partial.concept("B").is_some());
        assert!(partial.concept("X").is_some());
        assert!(partial.concept("C").is_none());
    }

    #[test]
    fn specialize_follows_taxonomy() {
        let onto = Ontology::create("Taxonomy")
            .concept("Animal")
            .concept("Mammal")
            .concept("Dog")
            .concept("Cat")
            .concept("Plant")
            .is_a("Dog", "Mammal")
            .is_a("Cat", "Mammal")
            .is_a("Mammal", "Animal")
            .build();

        let mammals = onto.specialize("Mammal").unwrap();

        assert!(mammals.concept("Mammal").is_some());
        assert!(mammals.concept("Dog").is_some());
        assert!(mammals.concept("Cat").is_some());
        assert!(mammals.concept("Animal").is_none());
        assert!(mammals.concept("Plant").is_none());
    }

    #[test]
    fn query_returns_related_edges() {
        let onto = Ontology::create("Test")
            .concept("A")
            .concept("B")
            .concept("C")
            .is_a("A", "B")
            .has_a("B", "C")
            .build();

        let edges = onto.query("B");
        let non_identity: Vec<_> = edges
            .iter()
            .filter(|e| e.kind != EdgeKind::Identity)
            .collect();
        assert_eq!(non_identity.len(), 2);
    }

    #[test]
    fn metroplex_grades() {
        let bio = Ontology::create("Biology").concept("Cell").build();

        let chem = Ontology::create("Chemistry").concept("Atom").build();

        let biochem = bio.couple(&chem);

        let mut mplex = Metroplex::new("Science");
        mplex.add(bio.clone());
        mplex.add(chem.clone());
        mplex.add(biochem);

        assert_eq!(mplex.grade(0).len(), 2);
        assert_eq!(mplex.grade(1).len(), 1);
        assert_eq!(mplex.grade_count(), 2);
        assert_eq!(mplex.total_ontologies(), 3);
    }

    #[test]
    fn vocabulary_bridge() {
        let onto = Ontology::create("TestOntology")
            .source("Test (2024)")
            .being(Being::AbstractObject)
            .concept("X")
            .concept("Y")
            .is_a("X", "Y")
            .build();

        let vocab = onto.vocabulary();
        assert_eq!(vocab.ontology_name, "TestOntology");
        assert_eq!(vocab.concept_count, 2);
        assert!(vocab.morphism_count > 0);
        assert_eq!(vocab.being, Some(Being::AbstractObject));
    }

    #[test]
    fn validate_catches_orphan_edges() {
        let mut s = Ontology::create("Bad").concept("A").build();
        s.edges.insert(Edge {
            from: String::from("A"),
            to: String::from("Z"),
            kind: EdgeKind::IsA,
        });

        assert!(s.validate().is_err());
    }

    #[test]
    fn compose_level_increments() {
        let a = Ontology::create("A").concept("X").build();
        let b = Ontology::create("B").concept("Y").build();
        let ab = a.couple(&b);
        assert_eq!(ab.level(), 1);

        let c = Ontology::create("C").concept("Z").build();
        let abc = ab.couple(&c);
        assert_eq!(abc.level(), 2);
    }

    #[test]
    fn provenance_tracks_composition() {
        let a = Ontology::create("Alpha").concept("X").build();
        let b = Ontology::create("Beta").concept("Y").build();
        let composed = a.couple(&b);

        assert!(composed.provenance().contains(&String::from("Alpha")));
        assert!(composed.provenance().contains(&String::from("Beta")));
    }

    #[test]
    fn from_vocabulary_is_embedded_staging() {
        let vocab = Vocabulary {
            ontology_name: "Test",
            module_path: "test::path",
            source: "Test (2024)",
            being: Some(Being::AbstractObject),
            concept_count: 5,
            morphism_count: 10,
        };

        let s = from_vocabulary(&vocab);
        assert_eq!(s.staging(), Staging::Embedded);
        assert_eq!(s.name(), "Test");
    }

    #[test]
    fn lexical_metadata_preserved_through_composition() {
        let a = Ontology::create("A")
            .concept("Cell")
            .label("Cell", "en", "Cell")
            .definition("Cell", "en", "Basic unit of life")
            .build();

        let b = Ontology::create("B")
            .concept("Atom")
            .label("Atom", "en", "Atom")
            .build();

        let composed = a.couple(&b);
        let cell = composed.concept("Cell").unwrap();
        assert!(cell.lexical.is_some());
        assert_eq!(cell.lexical.as_ref().unwrap().label, "Cell");

        let atom = composed.concept("Atom").unwrap();
        assert!(atom.lexical.is_some());
        assert_eq!(atom.lexical.as_ref().unwrap().label, "Atom");
    }

    // === Modification operations ===

    #[test]
    fn extend_adds_to_existing() {
        let bio = Ontology::create("Biology")
            .concept("Cell")
            .concept("Tissue")
            .is_a("Cell", "Tissue")
            .build();

        let extended = bio
            .extend()
            .concept("Organ")
            .is_a("Tissue", "Organ")
            .build();

        assert_eq!(extended.concept_count(), 3);
        assert!(extended.concept("Organ").is_some());
        assert!(extended.concept("Cell").is_some());
    }

    #[test]
    fn without_removes_concept_and_edges() {
        let onto = Ontology::create("Test")
            .concept("A")
            .concept("B")
            .concept("C")
            .is_a("A", "B")
            .is_a("B", "C")
            .build();

        let reduced = onto.without("B");
        assert_eq!(reduced.concept_count(), 2);
        assert!(reduced.concept("B").is_none());
        let b_edges: Vec<_> = reduced
            .edges()
            .iter()
            .filter(|e| e.from == "B" || e.to == "B")
            .collect();
        assert!(b_edges.is_empty());
    }

    #[test]
    fn restrict_keeps_only_specified() {
        let onto = Ontology::create("Test")
            .concepts(&["A", "B", "C", "D"])
            .is_a("A", "B")
            .is_a("C", "D")
            .build();

        let restricted = onto.restrict(&["A", "B"]);
        assert_eq!(restricted.concept_count(), 2);
        assert!(restricted.concept("A").is_some());
        assert!(restricted.concept("C").is_none());
    }

    #[test]
    fn rename_updates_concept_and_edges() {
        let onto = Ontology::create("Test")
            .concept("OldName")
            .concept("Other")
            .is_a("OldName", "Other")
            .build();

        let renamed = onto.rename("OldName", "NewName");
        assert!(renamed.concept("NewName").is_some());
        assert!(renamed.concept("OldName").is_none());
        let edges: Vec<_> = renamed
            .edges()
            .iter()
            .filter(|e| e.kind == EdgeKind::IsA)
            .collect();
        assert_eq!(edges[0].from, "NewName");
    }

    #[test]
    fn evolve_applies_mapping() {
        let onto = Ontology::create("Communication")
            .concept("Sender")
            .concept("Receiver")
            .concept("Feedback")
            .is_a("Feedback", "Sender")
            .build();

        let evolved = onto.evolve(&[
            ("Sender", "Controller"),
            ("Receiver", "Plant"),
            ("Feedback", "Sensor"),
        ]);

        assert!(evolved.concept("Controller").is_some());
        assert!(evolved.concept("Plant").is_some());
        assert!(evolved.concept("Sensor").is_some());
        assert!(evolved.concept("Sender").is_none());
    }

    #[test]
    fn original_unchanged_after_modification() {
        let original = Ontology::create("Original")
            .concept("A")
            .concept("B")
            .build();

        let _ = original.without("A");
        let _ = original.rename("A", "Z");
        let _ = original.extend().concept("C").build();

        assert_eq!(original.concept_count(), 2);
        assert!(original.concept("A").is_some());
    }
}
