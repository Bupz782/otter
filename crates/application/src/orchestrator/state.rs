/// High-level states of the automation lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum State {
    /// Waiting for the next command or event.
    #[default]
    Idle,
    /// Actively monitoring on-chain conditions for active intents.
    Monitoring,
    /// Parsing a natural-language intent into a structured domain object.
    Parsing,
    /// Building an execution plan from a parsed intent.
    Planning,
    /// Analyzing a condition that was just met.
    Analyzing,
    /// Deciding whether to execute now or wait.
    Deciding,
    /// Generating a zero-knowledge proof.
    Proving,
    /// Submitting a transaction on-chain.
    Submitting,
    /// Waiting for on-chain confirmation.
    Confirming,
    /// Executing a planned intent end-to-end.
    Executing,
    /// A non-recoverable error occurred; details are held in the orchestrator logs.
    Error(String),
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Idle => write!(f, "idle"),
            State::Monitoring => write!(f, "monitoring"),
            State::Parsing => write!(f, "parsing"),
            State::Planning => write!(f, "planning"),
            State::Analyzing => write!(f, "analyzing"),
            State::Deciding => write!(f, "deciding"),
            State::Proving => write!(f, "proving"),
            State::Submitting => write!(f, "submitting"),
            State::Confirming => write!(f, "confirming"),
            State::Executing => write!(f, "executing"),
            State::Error(msg) => write!(f, "error: {}", msg),
        }
    }
}

/// Return true if `from → to` is a valid state-machine transition.
///
/// `Error` can be reached from any state. From `Error`, only `Idle` and
/// `Monitoring` are allowed (rollback / recovery).
pub fn is_valid_transition(from: &State, to: &State) -> bool {
    use State::*;
    match (from, to) {
        // Any state may transition to Error.
        (_, Error(_)) => true,
        // Recovery from Error.
        (Error(_), Idle) | (Error(_), Monitoring) => true,
        // Operational flow.
        (Idle, Monitoring)
        | (Idle, Parsing)
        | (Monitoring, Analyzing)
        | (Analyzing, Deciding)
        | (Deciding, Proving)
        | (Proving, Submitting)
        | (Proving, Executing)
        | (Executing, Submitting)
        | (Submitting, Confirming)
        | (Confirming, Idle)
        | (Confirming, Monitoring)
        | (Parsing, Idle)
        | (Parsing, Planning)
        | (Planning, Idle)
        | (Executing, Idle)
        | (Executing, Monitoring) => true,
        // Self-transitions are allowed (e.g. staying in Monitoring).
        (a, b) if a == b => true,
        _ => false,
    }
}

/// Maximum time allowed in each state before a timeout is raised.
pub fn state_timeout_seconds(state: &State) -> Option<u64> {
    match state {
        State::Parsing => Some(30),
        State::Planning => Some(30),
        State::Proving => Some(120),
        State::Submitting => Some(120),
        State::Confirming => Some(300),
        State::Analyzing => Some(60),
        State::Deciding => Some(60),
        State::Executing => Some(120),
        _ => None,
    }
}
