use super::state::{State, is_valid_transition, state_timeout_seconds};
use crate::events::{Event, EventBus};
use crate::ports::ExecutionPort;
use crate::services::strategy_planner::StrategyPlanner;
use crate::use_cases::parse_intent::{ParseIntentError, ParseIntentUseCase};
use crate::use_cases::plan_execution::{PlanExecutionError, PlanExecutionUseCase};
use domain::models::condition::Metric;
use domain::models::delegation::DelegationMessage;
use domain::models::execution_plan::ExecutionPlan;
use domain::models::intent::{Asset, ConditionalIntent, Intent};
use domain::ports::evm_port::EvmPort;
use domain::ports::intent_parser_port::IntentParserPort;
use domain::ports::price_oracle_port::{OracleError, PriceOraclePort};
use domain::ports::zkp_port::ZkpPort;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// An intent currently being monitored by the orchestrator.
#[derive(Debug, Clone)]
pub struct ActiveIntent {
    pub id: String,
    /// Original natural-language text used to create the intent.
    pub text: String,
    pub conditional: ConditionalIntent,
}

/// Coordinates the high-level automation flow: parse → plan → execute.
///
/// The orchestrator is intentionally small right now. It owns the parser and
/// strategy planner so that CLI commands such as `metis parse` and `metis plan`
/// can reuse the same business rules as the end-to-end execution path.
#[derive(Clone)]
pub struct Orchestrator<P, O, Z, E> {
    parser: P,
    oracle: O,
    #[allow(dead_code)]
    zkp: Z,
    #[allow(dead_code)]
    evm: E,
    execution: Option<Arc<dyn ExecutionPort>>,
    state: State,
    /// Instant at which the current state was entered. Used for timeouts.
    state_entered_at: Instant,
    active_intents: Vec<ActiveIntent>,
    /// Intent IDs that already triggered an on-chain execution.
    executed_intents: Arc<Mutex<HashSet<String>>>,
    /// Intent IDs currently being executed (prevents duplicate submissions).
    executing_intents: Arc<Mutex<HashSet<String>>>,
    /// Delegation limits for the active intents.
    delegation: Option<DelegationMessage>,
    /// Signature over the delegation hash.
    signature: Option<[u8; 64]>,
    /// Timestamp used when proving the delegation.
    timestamp: u64,
}

/// Errors surfaced by the orchestrator when parsing or planning fails.
#[derive(Debug, PartialEq, Eq)]
pub enum OrchestratorError {
    ParseFailed(String),
    PlanFailed(String),
    InvalidIntent(String),
}

impl std::fmt::Display for OrchestratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrchestratorError::ParseFailed(msg) => write!(f, "parse failed: {}", msg),
            OrchestratorError::PlanFailed(msg) => write!(f, "plan failed: {}", msg),
            OrchestratorError::InvalidIntent(msg) => write!(f, "invalid intent: {}", msg),
        }
    }
}

impl std::error::Error for OrchestratorError {}

impl From<ParseIntentError> for OrchestratorError {
    fn from(err: ParseIntentError) -> Self {
        match err {
            ParseIntentError::ParsingFailed(msg)
            | ParseIntentError::InvalidIntent(msg)
            | ParseIntentError::LlmError(msg) => OrchestratorError::ParseFailed(msg),
        }
    }
}

impl From<PlanExecutionError> for OrchestratorError {
    fn from(err: PlanExecutionError) -> Self {
        match err {
            PlanExecutionError::PlanningFailed(msg)
            | PlanExecutionError::ValidationFailed(msg)
            | PlanExecutionError::InvalidIntent(msg) => OrchestratorError::PlanFailed(msg),
        }
    }
}

impl<P, O, Z, E> Orchestrator<P, O, Z, E>
where
    P: IntentParserPort + Clone + Send + 'static,
    O: PriceOraclePort + Clone + Send + 'static,
    Z: ZkpPort + Clone + Send + 'static,
    E: EvmPort + Clone + Send + 'static,
{
    /// Create a new orchestrator around the given intent parser and price oracle.
    ///
    /// Use this constructor for read-only operations such as `parse` and `plan`.
    /// For the daemon execution path, use [`Self::new_with_executor`].
    pub fn new(parser: P, oracle: O, zkp: Z, evm: E) -> Self {
        Self {
            parser,
            oracle,
            zkp,
            evm,
            execution: None,
            state: State::Idle,
            state_entered_at: Instant::now(),
            active_intents: Vec::new(),
            executed_intents: Arc::new(Mutex::new(HashSet::new())),
            executing_intents: Arc::new(Mutex::new(HashSet::new())),
            delegation: None,
            signature: None,
            timestamp: 1_000_000,
        }
    }

    /// Create an orchestrator that can execute intents end-to-end.
    pub fn new_with_executor(
        parser: P,
        oracle: O,
        zkp: Z,
        evm: E,
        execution: Arc<dyn ExecutionPort>,
    ) -> Self {
        let mut orchestrator = Self::new(parser, oracle, zkp, evm);
        orchestrator.execution = Some(execution);
        orchestrator
    }

    /// Set the user delegation and signature used for on-chain execution.
    pub fn set_delegation(&mut self, delegation: DelegationMessage, signature: [u8; 64]) {
        self.delegation = Some(delegation.clone());
        self.signature = Some(signature);
        if let Some(execution) = self.execution.clone() {
            execution.set_delegation(delegation, signature);
        }
    }

    /// Set the timestamp used when proving the delegation.
    pub fn set_timestamp(&mut self, timestamp: u64) {
        self.timestamp = timestamp;
    }

    /// Parse a natural-language intent into a structured `ConditionalIntent`.
    pub fn parse(&mut self, text: &str) -> Result<ConditionalIntent, OrchestratorError> {
        self.transition(State::Parsing);
        let use_case = ParseIntentUseCase::new(&self.parser);
        let result = use_case.execute(text);
        match &result {
            Ok(_) => self.transition(State::Idle),
            Err(err) => self.transition(State::Error(format!("{:?}", err))),
        }
        result.map_err(Into::into)
    }

    /// Parse a natural-language intent and build an `ExecutionPlan` from it.
    pub fn plan(&mut self, text: &str) -> Result<ExecutionPlan, OrchestratorError> {
        self.transition(State::Planning);
        let conditional = self.parse(text)?;
        let plan_use_case = PlanExecutionUseCase::new(StrategyPlanner::new());
        let result = plan_use_case.execute_conditional(&conditional);
        match &result {
            Ok(_) => self.transition(State::Idle),
            Err(err) => self.transition(State::Error(format!("{:?}", err))),
        }
        result.map_err(Into::into)
    }

    /// Submit a new intent to the orchestrator for monitoring and execution.
    pub fn submit_intent(&mut self, text: &str) -> Result<String, OrchestratorError> {
        let conditional = self.parse(text)?;
        let id = format!("intent-{}", self.active_intents.len() + 1);
        self.active_intents.push(ActiveIntent {
            id: id.clone(),
            text: text.to_string(),
            conditional,
        });
        self.transition(State::Monitoring);
        Ok(id)
    }

    /// Run the orchestrator event loop until the receiver closes.
    pub async fn run(&mut self, receiver: &mut mpsc::Receiver<Event>, bus: &EventBus) {
        self.transition(State::Idle);
        while let Some(event) = receiver.recv().await {
            self.process_event(event, bus).await;
        }
    }

    /// Process a single event and transition state accordingly.
    pub async fn process_event(&mut self, event: Event, bus: &EventBus) {
        self.handle_event(event, bus).await;
    }

    /// Return the current orchestrator state.
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Return the list of active intents.
    pub fn active_intents(&self) -> &[ActiveIntent] {
        &self.active_intents
    }

    /// Fetch the current value of a metric for an asset from the oracle.
    pub fn fetch_metric(&self, metric: &Metric, asset: &Asset) -> Result<u128, OracleError> {
        self.oracle.fetch(metric, Some(asset))
    }

    /// Fetch a metric value asynchronously, running the oracle call in a
    /// blocking task so it can safely create its own HTTP runtime.
    pub async fn fetch_metric_async(
        &self,
        metric: &Metric,
        asset: &Asset,
    ) -> Result<u128, OracleError> {
        let oracle = self.oracle.clone();
        let metric = *metric;
        let asset = asset.clone();
        tokio::task::spawn_blocking(move || {
            std::thread::spawn(move || oracle.fetch(&metric, Some(&asset)))
                .join()
                .map_err(|_| OracleError::FetchFailed("oracle fetch panicked".to_string()))?
        })
        .await
        .map_err(|_| OracleError::FetchFailed("oracle fetch panicked".to_string()))?
    }

    /// Return the primary asset of an intent, used for price/yield monitoring.
    pub fn primary_asset_of(intent: &Intent) -> Asset {
        Self::primary_asset(intent)
    }

    /// Add an already-parsed intent to the active set for monitoring.
    pub fn add_active_intent(&mut self, id: String, text: String, conditional: ConditionalIntent) {
        self.active_intents.push(ActiveIntent {
            id,
            text,
            conditional,
        });
    }

    /// Remove an intent from the active set by id.
    pub fn remove_active_intent(&mut self, id: &str) {
        self.active_intents.retain(|i| i.id != id);
    }

    /// Borrow the configured ZKP adapter.
    pub fn zkp_adapter(&self) -> &Z {
        &self.zkp
    }

    /// Borrow the configured EVM adapter.
    pub fn evm_adapter(&self) -> &E {
        &self.evm
    }

    fn transition(&mut self, new_state: State) {
        if !is_valid_transition(&self.state, &new_state) {
            let msg = format!("invalid state transition: {} -> {}", self.state, new_state);
            tracing::warn!(%msg);
            self.state = State::Error(msg);
            self.state_entered_at = Instant::now();
            return;
        }

        if self.state != new_state {
            tracing::info!(
                from = %self.state,
                to = %new_state,
                "orchestrator state transition"
            );
            self.state = new_state;
            self.state_entered_at = Instant::now();
        }
    }

    /// Return true if the orchestrator has been in the current state longer than
    /// the configured timeout.
    pub fn is_state_timed_out(&self) -> bool {
        state_timeout_seconds(&self.state)
            .map(|secs| self.state_entered_at.elapsed() > Duration::from_secs(secs))
            .unwrap_or(false)
    }

    async fn handle_event(&mut self, event: Event, bus: &EventBus) {
        if self.is_state_timed_out() {
            let msg = format!("timeout in state {}", self.state);
            tracing::warn!(%msg);
            let _ = bus.publish(Event::Error {
                source: "orchestrator".to_string(),
                message: msg.clone(),
            });
            self.transition(State::Error(msg));
            return;
        }

        match event {
            Event::PriceUpdated {
                asset,
                metric,
                value,
            } => {
                let oracle = self.oracle.clone();
                let intents = self.active_intents.clone();
                let bus = bus.clone();
                std::thread::spawn(move || {
                    Self::evaluate_conditions(&oracle, &intents, &asset, &metric, value, &bus);
                });
            }
            Event::ConditionMet { intent_id } => {
                if self.is_already_handled(&intent_id) {
                    return;
                }

                self.transition(State::Analyzing);
                let Some(intent) = self.find_intent(&intent_id).cloned() else {
                    self.transition(State::Idle);
                    return;
                };
                let plan_use_case = PlanExecutionUseCase::new(StrategyPlanner::new());
                if let Err(err) = plan_use_case.execute_conditional(&intent.conditional) {
                    let _ = bus.publish(Event::Error {
                        source: "planner".to_string(),
                        message: format!("{:?}", err),
                    });
                    self.transition(State::Idle);
                    return;
                }

                self.transition(State::Deciding);

                let Some(execution) = self.execution.clone() else {
                    let _ = bus.publish(Event::Error {
                        source: "executor".to_string(),
                        message: "execution port not configured".to_string(),
                    });
                    self.transition(State::Idle);
                    return;
                };

                self.transition(State::Proving);
                self.mark_executing(&intent_id);
                let bus_for_task = bus.clone();
                let intent_id_for_task = intent_id.clone();
                let input = intent.text.clone();
                let executed = self.executed_intents.clone();
                let executing = self.executing_intents.clone();

                std::thread::spawn(move || {
                    let result = execution.execute(&input);
                    match result {
                        Ok(tx_hash) => {
                            let _ = bus_for_task.publish(Event::TransactionSubmitted {
                                intent_id: intent_id_for_task.clone(),
                                tx_hash: tx_hash.clone(),
                            });

                            match execution.confirm(&tx_hash) {
                                Ok(result) if result.success => {
                                    {
                                        let mut guard =
                                            executed.lock().unwrap_or_else(|e| e.into_inner());
                                        guard.insert(intent_id_for_task.clone());
                                    }
                                    let _ = bus_for_task.publish(Event::TransactionConfirmed {
                                        intent_id: intent_id_for_task.clone(),
                                        receipt: tx_hash,
                                        gas_used: result.gas_used,
                                    });
                                }
                                Ok(result) => {
                                    eprintln!(
                                        "[orchestrator] transaction {} failed on-chain for {} (gas used: {})",
                                        tx_hash, intent_id_for_task, result.gas_used
                                    );
                                    let _ = bus_for_task.publish(Event::Error {
                                        source: "executor".to_string(),
                                        message: format!("transaction {} failed on-chain", tx_hash),
                                    });
                                }
                                Err(err) => {
                                    eprintln!(
                                        "[orchestrator] confirmation failed for {}: {}",
                                        intent_id_for_task, err
                                    );
                                    let _ = bus_for_task.publish(Event::Error {
                                        source: "executor".to_string(),
                                        message: err.to_string(),
                                    });
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!(
                                "[orchestrator] execution failed for {}: {}",
                                intent_id_for_task, err
                            );
                            let _ = bus_for_task.publish(Event::Error {
                                source: "executor".to_string(),
                                message: err.to_string(),
                            });
                        }
                    }

                    {
                        let mut guard = executing.lock().unwrap_or_else(|e| e.into_inner());
                        guard.remove(&intent_id_for_task);
                    }
                });
            }
            Event::IntentParsed { .. } => {
                // Intent submission is handled synchronously; this event is
                // mainly for audit/logging in future versions.
            }
            Event::ProofGenerated { intent_id, .. } => {
                let _ = &intent_id;
                self.transition(State::Submitting);
            }
            Event::TransactionSubmitted { intent_id, tx_hash } => {
                let _ = &intent_id;
                println!("[orchestrator] transaction submitted: {}", tx_hash);
                self.transition(State::Confirming);
            }
            Event::TransactionConfirmed {
                intent_id,
                receipt,
                gas_used,
            } => {
                let _ = &intent_id;
                println!(
                    "[orchestrator] transaction confirmed: {} (gas used: {})",
                    receipt, gas_used
                );
                self.transition(State::Idle);
            }
            Event::Error { source, message } => {
                eprintln!("[orchestrator] error from {}: {}", source, message);
                self.transition(State::Error(format!("{}: {}", source, message)));
            }
        }
    }

    fn find_intent(&self, id: &str) -> Option<&ActiveIntent> {
        self.active_intents.iter().find(|i| i.id == id)
    }

    fn is_already_handled(&self, id: &str) -> bool {
        let executed = self
            .executed_intents
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .contains(id);
        let executing = self
            .executing_intents
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .contains(id);
        executed || executing
    }

    fn mark_executing(&self, id: &str) {
        self.executing_intents
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id.to_string());
    }

    fn evaluate_conditions(
        oracle: &O,
        intents: &[ActiveIntent],
        asset: &Asset,
        metric: &Metric,
        _value: u128,
        bus: &EventBus,
    ) {
        for intent in intents {
            let primary_asset = Self::primary_asset(&intent.conditional.intent);
            if let Some(condition) = &intent.conditional.condition
                && primary_asset == *asset
                && *condition.metric() == *metric
            {
                let evaluator =
                    crate::use_cases::evaluate_condition::EvaluateConditionUseCase::new(oracle);
                if evaluator.execute(condition, Some(asset)).unwrap_or(false) {
                    let _ = bus.publish(Event::ConditionMet {
                        intent_id: intent.id.clone(),
                    });
                }
            }
        }
    }

    fn primary_asset(intent: &domain::models::intent::Intent) -> Asset {
        use domain::models::intent::Intent;
        match intent {
            Intent::Lend { asset, .. }
            | Intent::Stake { asset, .. }
            | Intent::Borrow { asset, .. } => asset.clone(),
            Intent::Swap { from_asset, .. } => from_asset.clone(),
            Intent::Composite { intents } => intents
                .first()
                .map(|i| Self::primary_asset(i))
                .unwrap_or(Asset::Eth),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::{ExecutionPort, ExecutionResult};
    use crate::use_cases::execute_intent::ExecutionError;
    use domain::models::condition::Metric;
    use domain::models::delegation::DelegationMessage;
    use domain::models::intent::{Asset, Intent, LendingType};
    use infrastructure::blockchain::{MockEvmAdapter, MockOracleAdapter};
    use infrastructure::parsers::RegexParser;
    use infrastructure::zkp::MockZkpAdapter;
    use std::sync::Arc;

    struct OkExecutionPort;

    impl ExecutionPort for OkExecutionPort {
        fn execute(&self, _input: &str) -> Result<String, ExecutionError> {
            Ok("0xdeadbeef".to_string())
        }

        fn set_delegation(&self, _delegation: DelegationMessage, _signature: [u8; 64]) {}

        fn confirm(&self, _tx_hash: &str) -> Result<ExecutionResult, ExecutionError> {
            Ok(ExecutionResult {
                success: true,
                gas_used: 21000,
            })
        }
    }

    fn test_orchestrator(
        oracle: MockOracleAdapter,
    ) -> Orchestrator<RegexParser, MockOracleAdapter, MockZkpAdapter, MockEvmAdapter> {
        Orchestrator::new(
            RegexParser::new(),
            oracle,
            MockZkpAdapter::new(),
            MockEvmAdapter::new(),
        )
    }

    fn test_orchestrator_with_execution(
        oracle: MockOracleAdapter,
    ) -> Orchestrator<RegexParser, MockOracleAdapter, MockZkpAdapter, MockEvmAdapter> {
        Orchestrator::new_with_executor(
            RegexParser::new(),
            oracle,
            MockZkpAdapter::new(),
            MockEvmAdapter::new(),
            Arc::new(OkExecutionPort),
        )
    }

    #[test]
    fn parse_valid_intent() {
        let mut orchestrator = test_orchestrator(MockOracleAdapter::new());
        let conditional = orchestrator.parse("lend 1000 USDC on Aave").unwrap();
        assert_eq!(orchestrator.state, State::Idle);
        assert!(matches!(
            conditional.intent,
            Intent::Lend {
                asset: Asset::Usdc,
                amount: 1_000_000_000,
                protocol: LendingType::Aave,
            }
        ));
    }

    #[test]
    fn parse_invalid_intent_returns_error() {
        let mut orchestrator = test_orchestrator(MockOracleAdapter::new());
        let result = orchestrator.parse("do something weird");
        assert!(result.is_err());
        assert!(matches!(orchestrator.state, State::Error(_)));
    }

    #[test]
    fn plan_valid_intent() {
        let mut orchestrator = test_orchestrator(MockOracleAdapter::new());
        let plan = orchestrator.plan("swap 1 ETH for USDC on Uniswap").unwrap();
        assert_eq!(orchestrator.state, State::Idle);
        assert_eq!(plan.step_count(), 2);
        assert!(plan.gas_estimation().is_some());
    }

    #[test]
    fn plan_conditional_intent_preserves_condition() {
        let mut orchestrator = test_orchestrator(MockOracleAdapter::new());
        let plan = orchestrator
            .plan("lend 1000 USDC on Aave if yield > 3")
            .unwrap();
        assert_eq!(orchestrator.state, State::Idle);
        assert!(matches!(
            plan.protocol(),
            domain::models::intent::Protocol::Lending(LendingType::Aave)
        ));
    }

    #[test]
    fn plan_invalid_intent_returns_error() {
        let mut orchestrator = test_orchestrator(MockOracleAdapter::new());
        let result = orchestrator.plan("borrow 1000 USDC with 0 ETH on Aave");
        assert!(result.is_err());
        assert!(matches!(orchestrator.state, State::Error(_)));
    }

    #[tokio::test]
    async fn process_event_triggers_condition_met() {
        let mut oracle = MockOracleAdapter::new();
        oracle.set(Metric::Price, None, 2_500_000_000);

        let mut orchestrator = test_orchestrator_with_execution(oracle);

        let _id = orchestrator
            .submit_intent("swap 1 ETH for USDC on Uniswap if price > 2_000_000000")
            .unwrap();

        let (bus, mut receiver) = EventBus::new(16);
        bus.publish(Event::PriceUpdated {
            asset: Asset::Eth,
            metric: Metric::Price,
            value: 2_500_000_000,
        })
        .unwrap();

        let event = receiver.recv().await.expect("price update event");
        orchestrator.process_event(event, &bus).await;

        let condition_event = receiver.recv().await.expect("condition met event");
        assert!(matches!(condition_event, Event::ConditionMet { .. }));
        orchestrator.process_event(condition_event, &bus).await;
        assert!(matches!(orchestrator.state, State::Proving));
    }

    #[tokio::test]
    async fn process_event_triggers_yield_condition_met() {
        let mut oracle = MockOracleAdapter::new();
        oracle.set(Metric::Yield, Some(Asset::Usdc), 5);

        let mut orchestrator = test_orchestrator(oracle);

        let _id = orchestrator
            .submit_intent("lend 1000 USDC on Aave if yield > 3")
            .unwrap();

        let (bus, mut receiver) = EventBus::new(16);
        bus.publish(Event::PriceUpdated {
            asset: Asset::Usdc,
            metric: Metric::Yield,
            value: 5,
        })
        .unwrap();

        let event = receiver.recv().await.expect("yield update event");
        orchestrator.process_event(event, &bus).await;

        let condition_event = receiver.recv().await.expect("condition met event");
        assert!(matches!(condition_event, Event::ConditionMet { .. }));
    }

    #[tokio::test]
    async fn execution_confirms_transaction_before_marking_executed() {
        let mut oracle = MockOracleAdapter::new();
        oracle.set(Metric::Price, None, 2_500_000_000);

        let mut orchestrator = test_orchestrator_with_execution(oracle);

        let id = orchestrator
            .submit_intent("swap 1 ETH for USDC on Uniswap if price > 2_000_000000")
            .unwrap();

        let (bus, mut receiver) = EventBus::new(16);
        bus.publish(Event::PriceUpdated {
            asset: Asset::Eth,
            metric: Metric::Price,
            value: 2_500_000_000,
        })
        .unwrap();

        let event = receiver.recv().await.expect("price update event");
        orchestrator.process_event(event, &bus).await;

        let condition_event = receiver.recv().await.expect("condition met event");
        orchestrator.process_event(condition_event, &bus).await;

        // Drain events emitted by the execution thread until confirmation.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            match tokio::time::timeout(remaining, receiver.recv()).await {
                Ok(Some(event)) => {
                    let is_terminal = matches!(
                        event,
                        Event::TransactionConfirmed { .. } | Event::Error { .. }
                    );
                    orchestrator.process_event(event, &bus).await;
                    if is_terminal {
                        break;
                    }
                }
                _ => panic!("timeout waiting for transaction confirmation"),
            }
        }

        assert!(
            orchestrator
                .executed_intents
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .contains(&id),
            "intent should be marked executed after confirmation"
        );
        assert_eq!(orchestrator.state, State::Idle);
    }

    #[test]
    fn submit_intent_transitions_to_monitoring() {
        let mut orchestrator = test_orchestrator_with_execution(MockOracleAdapter::new());
        let _id = orchestrator
            .submit_intent("lend 1000 USDC on Aave if yield > 3")
            .unwrap();
        assert_eq!(orchestrator.state, State::Monitoring);
    }

    #[test]
    fn invalid_state_transition_goes_to_error() {
        let mut orchestrator = test_orchestrator_with_execution(MockOracleAdapter::new());
        orchestrator.transition(State::Confirming);
        // Submitting directly from Confirming is invalid.
        orchestrator.transition(State::Submitting);
        assert!(matches!(orchestrator.state, State::Error(_)));
    }

    #[test]
    fn state_timeout_detects_long_running_state() {
        let mut orchestrator = test_orchestrator_with_execution(MockOracleAdapter::new());
        // Force-enter Proving with an old timestamp by manipulating the field.
        orchestrator.state = State::Proving;
        orchestrator.state_entered_at = Instant::now() - Duration::from_secs(300);
        assert!(orchestrator.is_state_timed_out());
    }
}
