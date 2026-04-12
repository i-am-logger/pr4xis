use super::connection::{Connection, ConnectionAction};
use pr4xis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};

impl Situation for Connection {
    fn describe(&self) -> String {
        format!(
            "state={:?} retries={}/{} keep_alive={}",
            self.state, self.retries, self.max_retries, self.keep_alive
        )
    }

    fn is_terminal(&self) -> bool {
        self.is_terminal()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HttpAction(pub ConnectionAction);

impl Action for HttpAction {
    type Sit = Connection;

    fn describe(&self) -> String {
        format!("{:?}", self.0)
    }
}

pub struct ValidTransition;

impl Precondition<HttpAction> for ValidTransition {
    fn check(&self, conn: &Connection, action: &HttpAction) -> PreconditionResult {
        match conn.apply(action.0) {
            Ok(_) => PreconditionResult::satisfied(
                "valid_transition",
                &format!("{:?} → {:?}", conn.state, action.0),
            ),
            Err(msg) => PreconditionResult::violated(
                "valid_transition",
                msg,
                &conn.describe(),
                &action.describe(),
            ),
        }
    }

    fn describe(&self) -> &str {
        "connection action must be valid for current state"
    }
}

fn apply_http(conn: &Connection, action: &HttpAction) -> Result<Connection, String> {
    conn.apply(action.0).map_err(|e| e.to_string())
}

pub type HttpEngine = Engine<HttpAction>;

pub fn new_connection(max_retries: u32) -> HttpEngine {
    Engine::new(
        Connection::new(max_retries),
        vec![Box::new(ValidTransition)],
        apply_http,
    )
}
