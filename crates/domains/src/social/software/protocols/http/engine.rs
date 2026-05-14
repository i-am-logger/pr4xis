#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::connection::{Connection, ConnectionAction};
use pr4xis::engine::{Action, Engine, Precondition, Situation};
use pr4xis::logic::proof::{Counterexample, SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

fn axiom_meta(name: &'static str, description: &'static str, citation: &'static str) -> Provenance {
    Provenance {
        name: OntologyName::new_static(name),
        description: Label::new_static(description),
        citation: Citation::parse_static(citation),
        module_path: ModulePath::new_static(module_path!()),
    }
}

impl Situation for Connection {}

#[derive(Debug, Clone, PartialEq)]
pub struct HttpAction(pub ConnectionAction);

impl Action for HttpAction {
    type Sit = Connection;
}

pub struct ValidTransition;

impl Precondition<HttpAction> for ValidTransition {
    fn check(&self, conn: &Connection, action: &HttpAction) -> Verdict {
        let meta = axiom_meta(
            "valid_transition",
            "connection action must be valid for current state",
            "RFC 9110 (2022) HTTP Semantics §3; RFC 9112 (2022) HTTP/1.1 §9 Connection Management",
        );
        match conn.apply(action.0) {
            Ok(_) => Ok(Box::new(SimpleProof::new(meta))),
            Err(_) => Err(Box::new(SimpleCounterexample::new(meta))),
        }
    }
}

fn apply_http(
    conn: &Connection,
    action: &HttpAction,
) -> Result<Connection, Box<dyn Counterexample>> {
    let meta = axiom_meta(
        "valid_transition",
        "connection action must be valid for current state",
        "RFC 9110 (2022) HTTP Semantics §3",
    );
    conn.apply(action.0)
        .map_err(|_| Box::new(SimpleCounterexample::new(meta)) as Box<dyn Counterexample>)
}

pub type HttpEngine = Engine<HttpAction>;

pub fn new_connection(max_retries: u32) -> HttpEngine {
    Engine::new(
        Connection::new(max_retries),
        vec![Box::new(ValidTransition)],
        apply_http,
    )
}
