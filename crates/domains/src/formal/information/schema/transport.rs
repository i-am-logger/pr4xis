// Schema-governed transport — Spivak's Presentation/Algebra adjunction.
//
// Data crosses boundaries (WASM→JS, Rust→CLI, ontology→surface) as
// Presentation instances governed by a Schema. No serde, no JSON.parse,
// no format strings. The Schema IS the type, the Presentation IS the
// wire format, the Algebra IS the live runtime form.
//
// A Presentation is generators + equations (CQL, Wisnesky et al. 2017).
// For transport, generators are named typed values and equations are
// schema constraints. Evaluation turns a Presentation into an Algebra
// (live object). Present turns an Algebra into a Presentation (wire form).
//
// Source: Spivak "Functorial Data Migration" (2012);
//         Wisnesky et al. "Algebraic Databases" (2017)

use std::collections::BTreeMap;

/// A value in a Presentation — the atomic data unit.
///
/// This is NOT a primitive. It's an instance of SchemaConcept::Population —
/// the set of individuals for one entity type. The variants correspond to
/// the base types that populations can contain.
#[derive(Debug, Clone, PartialEq)]
pub enum SchemaValue {
    Text(String),
    Integer(i64),
    Unsigned(u64),
    Float(f64),
    Boolean(bool),
    List(Vec<SchemaValue>),
    Record(Presentation),
    Absent,
}

/// A Presentation — the syntactic form of an Instance (CQL).
///
/// Instance of SchemaConcept::Presentation. Contains generators
/// (named values) governed by a schema. The schema determines which
/// names are valid and what types they carry.
///
/// This is what crosses boundaries. Not JSON. Not serde bytes.
/// A typed, schema-governed key-value structure that the receiver
/// evaluates into its own Algebra (live runtime form).
#[derive(Debug, Clone, PartialEq)]
pub struct Presentation {
    generators: BTreeMap<&'static str, SchemaValue>,
}

impl Presentation {
    pub fn new() -> Self {
        Self {
            generators: BTreeMap::new(),
        }
    }

    pub fn set(&mut self, name: &'static str, value: SchemaValue) {
        self.generators.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<&SchemaValue> {
        self.generators.get(name)
    }

    pub fn text(&self, name: &str) -> Option<&str> {
        match self.get(name)? {
            SchemaValue::Text(s) => Some(s),
            _ => None,
        }
    }

    pub fn unsigned(&self, name: &str) -> Option<u64> {
        match self.get(name)? {
            SchemaValue::Unsigned(n) => Some(*n),
            _ => None,
        }
    }

    pub fn list(&self, name: &str) -> Option<&[SchemaValue]> {
        match self.get(name)? {
            SchemaValue::List(v) => Some(v),
            _ => None,
        }
    }

    pub fn generators(&self) -> &BTreeMap<&'static str, SchemaValue> {
        &self.generators
    }

    /// Render as JSON — a SURFACE encoding of the Presentation.
    /// The JSON is not the Presentation itself; it's one possible
    /// realization through the HMI surface functor.
    pub fn to_json(&self) -> String {
        render_json_value(&SchemaValue::Record(self.clone()))
    }
}

impl Default for Presentation {
    fn default() -> Self {
        Self::new()
    }
}

/// The Present trait — Algebra → Presentation.
///
/// Instance of the Schema ontology's Presents morphism
/// (Algebra → Presentation). Any live runtime type that can
/// be presented as a schema-governed Presentation implements this.
///
/// This replaces serde::Serialize. The schema governs what gets
/// presented, not a derive macro.
pub trait Present {
    fn present(&self) -> Presentation;
}

/// The Evaluate trait — Presentation → Algebra.
///
/// Instance of the Schema ontology's Evaluates morphism
/// (Presentation → Algebra). Constructs a live runtime type
/// from a schema-governed Presentation.
///
/// This replaces serde::Deserialize.
pub trait Evaluate: Sized {
    fn evaluate(presentation: &Presentation) -> Option<Self>;
}

fn render_json_value(v: &SchemaValue) -> String {
    match v {
        SchemaValue::Text(s) => {
            let escaped = s
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
                .replace('\r', "\\r")
                .replace('\t', "\\t");
            format!("\"{escaped}\"")
        }
        SchemaValue::Integer(n) => n.to_string(),
        SchemaValue::Unsigned(n) => n.to_string(),
        SchemaValue::Float(f) => f.to_string(),
        SchemaValue::Boolean(b) => b.to_string(),
        SchemaValue::List(items) => {
            let rendered: Vec<String> = items.iter().map(render_json_value).collect();
            format!("[{}]", rendered.join(","))
        }
        SchemaValue::Record(p) => {
            let fields: Vec<String> = p
                .generators()
                .iter()
                .map(|(k, v)| format!("\"{}\":{}", k, render_json_value(v)))
                .collect();
            format!("{{{}}}", fields.join(","))
        }
        SchemaValue::Absent => "null".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presentation_roundtrip() {
        let mut p = Presentation::new();
        p.set("name", SchemaValue::Text("pr4xis".into()));
        p.set("version", SchemaValue::Text("0.10.0".into()));
        p.set("ontology_count", SchemaValue::Unsigned(121));

        assert_eq!(p.text("name"), Some("pr4xis"));
        assert_eq!(p.unsigned("ontology_count"), Some(121));
    }

    #[test]
    fn presentation_to_json() {
        let mut p = Presentation::new();
        p.set("name", SchemaValue::Text("pr4xis".into()));
        p.set("count", SchemaValue::Unsigned(42));
        p.set("active", SchemaValue::Boolean(true));

        let json = p.to_json();
        assert!(json.contains("\"name\":\"pr4xis\""));
        assert!(json.contains("\"count\":42"));
        assert!(json.contains("\"active\":true"));
    }

    #[test]
    fn nested_presentation() {
        let mut inner = Presentation::new();
        inner.set("label", SchemaValue::Text("test".into()));

        let mut outer = Presentation::new();
        outer.set("items", SchemaValue::List(vec![SchemaValue::Record(inner)]));

        let json = outer.to_json();
        assert!(json.contains("\"label\":\"test\""));
    }
}
