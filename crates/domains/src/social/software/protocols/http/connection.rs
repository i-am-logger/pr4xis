/// HTTP connection state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionState {
    Idle,
    Connecting,
    SendingRequest,
    AwaitingResponse,
    ReceivingResponse,
    Complete,
    Error,
    Closed,
}

/// Actions on a connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionAction {
    Connect,
    SendRequest,
    ReceiveResponse,
    Complete,
    Retry,
    Close,
    Reset,
}

/// Result of a connection action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionResult {
    Ok,
    Rejected { reason: &'static str },
}

/// HTTP connection with state machine enforcement.
#[derive(Debug, Clone, PartialEq)]
pub struct Connection {
    pub state: ConnectionState,
    pub retries: u32,
    pub max_retries: u32,
    pub keep_alive: bool,
}

impl Connection {
    pub fn new(max_retries: u32) -> Self {
        Self {
            state: ConnectionState::Idle,
            retries: 0,
            max_retries,
            keep_alive: true,
        }
    }

    /// Valid next states from current state.
    fn valid_transitions(&self) -> Vec<(ConnectionAction, ConnectionState)> {
        match self.state {
            ConnectionState::Idle => vec![
                (ConnectionAction::Connect, ConnectionState::Connecting),
                (ConnectionAction::Close, ConnectionState::Closed),
            ],
            ConnectionState::Connecting => vec![
                (
                    ConnectionAction::SendRequest,
                    ConnectionState::SendingRequest,
                ),
                (ConnectionAction::Reset, ConnectionState::Error),
            ],
            ConnectionState::SendingRequest => vec![
                (
                    ConnectionAction::ReceiveResponse,
                    ConnectionState::AwaitingResponse,
                ),
                (ConnectionAction::Reset, ConnectionState::Error),
            ],
            ConnectionState::AwaitingResponse => vec![
                (
                    ConnectionAction::ReceiveResponse,
                    ConnectionState::ReceivingResponse,
                ),
                (ConnectionAction::Reset, ConnectionState::Error),
            ],
            ConnectionState::ReceivingResponse => vec![
                (ConnectionAction::Complete, ConnectionState::Complete),
                (ConnectionAction::Reset, ConnectionState::Error),
            ],
            ConnectionState::Complete => {
                let mut v = vec![(ConnectionAction::Close, ConnectionState::Closed)];
                if self.keep_alive {
                    v.push((
                        ConnectionAction::SendRequest,
                        ConnectionState::SendingRequest,
                    ));
                }
                v
            }
            ConnectionState::Error => {
                let mut v = vec![(ConnectionAction::Close, ConnectionState::Closed)];
                if self.retries < self.max_retries {
                    v.push((ConnectionAction::Retry, ConnectionState::Connecting));
                }
                v
            }
            ConnectionState::Closed => vec![], // terminal
        }
    }

    /// Apply an action. Returns Err if invalid from current state.
    pub fn apply(&self, action: ConnectionAction) -> Result<Connection, &'static str> {
        let transitions = self.valid_transitions();
        let next_state = transitions
            .iter()
            .find(|(a, _)| *a == action)
            .map(|(_, s)| *s);

        match next_state {
            Some(state) => {
                let mut conn = self.clone();
                conn.state = state;
                if action == ConnectionAction::Retry {
                    conn.retries += 1;
                }
                Ok(conn)
            }
            None => Err("invalid action for current connection state"),
        }
    }

    /// Is the connection in a terminal state?
    pub fn is_terminal(&self) -> bool {
        self.state == ConnectionState::Closed
    }

    /// Can we retry?
    pub fn can_retry(&self) -> bool {
        self.state == ConnectionState::Error && self.retries < self.max_retries
    }
}
