use application::events::{Event, EventBus};
use application::ports::ExecutionPort;
use application::orchestrator::{ActiveIntent, Orchestrator};
use axum::{
    Extension, Json, Router,
    body::Body,
    extract::ws::{Message, WebSocket},
    extract::{ConnectInfo, Path, State as AxumState, WebSocketUpgrade},
    http::{Request, StatusCode, header},
    middleware::{Next, from_fn, from_fn_with_state},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use domain::models::condition::Metric;
use domain::models::delegation::DelegationMessage;
use domain::models::execution_plan::ExecutionPlan;
use domain::models::intent::{Asset, ConditionalIntent};
use domain::ports::intent_parser_port::IntentParserPort;
use domain::ports::price_oracle_port::PriceOraclePort;
use domain::ports::storage_port::StrategyRecord;
use domain::ports::wallet_port::WalletPort;
use domain::ports::{BlockchainPort, DelegationRecord, ExecutionRecord, IntentRecord, StoragePort};
use infrastructure::blockchain::{
    AlloyEvmAdapter, CompositeOracle, HealthEntry, LocalWalletAdapter, MultiChainAdapter,
    OracleNetwork,
};
use infrastructure::config::Config;
use infrastructure::parsers::{HybridParser, LlmIntentParser, RegexParser};
use infrastructure::services::OnChainExecutionService;
use infrastructure::storage::{PgStorage, SqliteStorage};
use infrastructure::zkp::NoirAdapter;
use interfaces::auth::{AuthService, AuthUser};
use interfaces::secrets::load_private_key;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, RwLock};
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

/// Parser type used by the API daemon: a hybrid LLM parser when a model is
/// available, the deterministic regex parser otherwise, shared behind a trait
/// object so both variants fit the same orchestrator type.
type AgentParser = Arc<dyn IntentParserPort + Send + Sync>;

/// Concrete orchestrator type used by the API daemon.
type AgentOrchestrator = Orchestrator<AgentParser, CompositeOracle, NoirAdapter, AlloyEvmAdapter>;

/// Shared application state.
struct AppState {
    orchestrator: Arc<RwLock<AgentOrchestrator>>,
    storage: Arc<dyn StoragePort>,
    bus: EventBus,
    metrics: Arc<Metrics>,
    execution_enabled: bool,
    metrics_enabled: bool,
    version: &'static str,
    auth_enabled: bool,
    auth_service: Option<Arc<AuthService>>,
    rate_limit_per_minute: u32,
    request_counts: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    cors_allowed_origins: String,
    event_tx: tokio::sync::broadcast::Sender<Event>,
    agents: Vec<AgentSummary>,
    agent_pubkey: Option<AgentPubkey>,
    /// Multi-network EVM adapter registry (empty when no key is configured).
    multichain: Arc<MultiChainAdapter>,
    /// Cached `/api/v1/networks` healthchecks keyed by network name.
    network_health: Arc<Mutex<HashMap<String, HealthEntry>>>,
    /// Simulated MEV capture store; `None` when the backend is unavailable
    /// (e.g. non-SQLite storage), in which case rebates read as zero.
    mev: Option<Arc<infrastructure::mev::SimulatedMevCapture>>,
    /// Share of captured profit rebated to the vault owner, in basis points.
    rebate_bps: u64,
}

#[derive(Debug, Clone)]
struct AgentPubkey {
    x: String,
    y: String,
}

impl AppState {
    async fn read_orchestrator(&self) -> tokio::sync::RwLockReadGuard<'_, AgentOrchestrator> {
        self.orchestrator.read().await
    }

    async fn write_orchestrator(&self) -> tokio::sync::RwLockWriteGuard<'_, AgentOrchestrator> {
        self.orchestrator.write().await
    }
}

#[derive(Debug, Default)]
struct Metrics {
    price_updates: AtomicU64,
    conditions_met: AtomicU64,
    executions: AtomicU64,
    errors: AtomicU64,
    gas_used_total: AtomicU64,
    proof_verification_errors: AtomicU64,
    rpc_errors: AtomicU64,
    vault_balance: AtomicU64,
}

impl Metrics {
    fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            price_updates: self.price_updates.load(Ordering::Relaxed),
            conditions_met: self.conditions_met.load(Ordering::Relaxed),
            executions: self.executions.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
            gas_used_total: self.gas_used_total.load(Ordering::Relaxed),
            proof_verification_errors: self.proof_verification_errors.load(Ordering::Relaxed),
            rpc_errors: self.rpc_errors.load(Ordering::Relaxed),
            vault_balance: self.vault_balance.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Serialize)]
struct MetricsSnapshot {
    price_updates: u64,
    conditions_met: u64,
    executions: u64,
    errors: u64,
    gas_used_total: u64,
    proof_verification_errors: u64,
    rpc_errors: u64,
    vault_balance: u64,
}

#[derive(Debug, Deserialize)]
struct ParseRequest {
    text: String,
}

#[derive(Debug, Serialize)]
struct ParseResponse {
    intent: ConditionalIntent,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug, Serialize)]
struct IntentSummary {
    id: String,
    text: String,
    state: String,
}

#[derive(Debug, Serialize)]
struct IntentsResponse {
    intents: Vec<IntentSummary>,
}

#[derive(Debug, Serialize)]
struct DelegationSummary {
    hash: String,
    payload_json: String,
    signature: String,
    created_at: i64,
}

#[derive(Debug, Serialize)]
struct DelegationsResponse {
    delegations: Vec<DelegationSummary>,
}

#[derive(Debug, Serialize)]
struct ExecutionSummary {
    id: String,
    intent_id: String,
    tx_hash: String,
    status: String,
    gas_used: u64,
    created_at: i64,
}

#[derive(Debug, Serialize)]
struct ExecutionsResponse {
    executions: Vec<ExecutionSummary>,
}

#[derive(Debug, Deserialize)]
struct CreateIntentRequest {
    text: String,
    /// Optional target network name (see `OTTER_NETWORKS`). Defaults to the
    /// `default` network when omitted.
    network: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateIntentResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ChallengeRequest {
    address: String,
}

#[derive(Debug, Serialize)]
struct ChallengeResponse {
    message: String,
}

#[derive(Debug, Deserialize)]
struct VerifyRequest {
    message: String,
    signature: String,
}

#[derive(Debug, Serialize)]
struct VerifyResponse {
    token: String,
}

#[derive(Debug, Serialize)]
struct PlanResponse {
    plan: ExecutionPlan,
}

#[derive(Debug, Deserialize)]
struct SetDelegationRequest {
    pubkey_x: String,
    pubkey_y: String,
    allowed_intents: String,
    max_amounts: Vec<String>,
    allowed_protocols: Vec<String>,
    expiry: String,
    nonce: String,
    target_contract: String,
    signature: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SetDelegationResponse {
    delegation_hash: String,
}

#[derive(Debug, Serialize)]
struct IntentDetailResponse {
    id: String,
    text: String,
    intent: ConditionalIntent,
    state: String,
    created_at: i64,
    updated_at: i64,
}

#[derive(Debug, Serialize)]
struct ActiveIntentSummary {
    id: String,
    text: String,
    intent: ConditionalIntent,
}

#[derive(Debug, Serialize)]
struct OrchestratorStateResponse {
    state: String,
    active_intents: Vec<ActiveIntentSummary>,
    execution_enabled: bool,
}

#[derive(Debug, Serialize, Clone)]
struct AgentSummary {
    id: String,
    name: String,
    operated_by: String,
    /// Always true while agents come from `default_agents()` (anomaly A2);
    /// remove once agents are served from persisted/on-chain data.
    demo: bool,
    risk_profile: String,
    bond: u64,
    reputation: f64,
    proofs_submitted: u64,
    yield_generated: u64,
    mev_captured: u64,
    uptime: f64,
    strategies: u32,
    followers: u64,
    description: String,
}

#[derive(Debug, Serialize)]
struct AgentsResponse {
    agents: Vec<AgentSummary>,
    /// True while the payload embeds demonstration data (anomaly A2).
    demo: bool,
}

#[derive(Debug, Serialize)]
struct AgentPubkeyResponse {
    pubkey_x: String,
    pubkey_y: String,
}

#[derive(Debug, Serialize, Clone)]
struct StrategySummary {
    id: String,
    agent_id: String,
    agent_name: String,
    title: String,
    description: String,
    raw_text: String,
    risk_profile: String,
    copies: u64,
    /// `private` or `public` (sharing).
    visibility: String,
    /// Times this strategy was forked by other users.
    fork_count: u64,
    total_volume: u64,
    apy: f64,
    created_at: i64,
    updated_at: i64,
}

#[derive(Debug, Serialize)]
struct StrategiesResponse {
    strategies: Vec<StrategySummary>,
    /// True while the payload embeds demonstration data (anomaly A2).
    demo: bool,
}

#[derive(Debug, Serialize)]
struct PositionSummary {
    asset: String,
    protocol: String,
    chain: String,
    amount: String,
    value: u64,
    apy: f64,
}

#[derive(Debug, Serialize)]
struct PortfolioResponse {
    address: String,
    total_balance: u128,
    allocated: u128,
    available: u128,
    yield_earned: f64,
    mev_rebates: f64,
    positions: Vec<PositionSummary>,
}

#[derive(Debug, Serialize)]
struct ProofSummary {
    id: String,
    proof_type: String,
    intent_id: Option<String>,
    verifier: String,
    constraints: u64,
    proof_time: f64,
    timestamp: i64,
    verified: bool,
    tx_hash: Option<String>,
}

#[derive(Debug, Serialize)]
struct ProofsResponse {
    proofs: Vec<ProofSummary>,
    /// True while the payload embeds demonstration data (anomaly A2).
    demo: bool,
}

#[derive(Debug, Serialize)]
struct LeaderboardEntry {
    rank: u32,
    agent_id: String,
    agent_name: String,
    proofs_submitted: u64,
    yield_generated: u64,
    mev_captured: u64,
    uptime: f64,
}

#[derive(Debug, Serialize)]
struct LeaderboardResponse {
    entries: Vec<LeaderboardEntry>,
    /// True while the payload embeds demonstration data (anomaly A2).
    demo: bool,
}

#[derive(Debug, Deserialize)]
struct CreateStrategyRequest {
    title: String,
    description: String,
    raw_text: String,
    agent_id: String,
    risk_profile: String,
    /// `private` (default) or `public` (shareable).
    visibility: Option<String>,
}

#[derive(Debug, Serialize)]
struct StrategyDetailResponse {
    id: String,
    title: String,
    description: String,
    raw_text: String,
    intent: ConditionalIntent,
    creator_address: Option<String>,
    agent_id: String,
    agent_name: String,
    risk_profile: String,
    copies: u64,
    total_volume: u64,
    apy: f64,
    created_at: i64,
    updated_at: i64,
}

#[derive(Debug, Serialize)]
struct StrategyCreatedResponse {
    id: String,
}

#[derive(Debug, Serialize)]
struct ForkStrategyResponse {
    strategy_id: String,
    redirect_to: String,
}

#[derive(Debug, Deserialize)]
struct DelegationHashRequest {
    pubkey_x: String,
    pubkey_y: String,
    allowed_intents: String,
    max_amounts: Vec<String>,
    allowed_protocols: Vec<String>,
    expiry: String,
    nonce: String,
    target_contract: String,
}

#[derive(Debug, Serialize)]
struct DelegationHashResponse {
    delegation_hash: String,
}

fn app(state: Arc<AppState>) -> Router {
    let cors = build_cors(&state.cors_allowed_origins);

    let public = Router::new()
        .route("/api/v1/auth/challenge", post(auth_challenge))
        .route("/api/v1/auth/verify", post(auth_verify))
        .route("/api/v1/ws", get(ws_handler))
        .route("/health", get(health))
        .route("/api/v1/health", get(health))
        .route("/health/live", get(health_live))
        .route("/ready", get(ready))
        .route("/metrics", get(metrics))
        .route("/api/v1/networks", get(list_networks));

    // Endpoints serving built-in demonstration data (hardcoded agents,
    // seeded strategies, synthetic solvency proof) are grouped so the
    // X-Demo-Data header is applied consistently. Remove the marker once
    // they serve persisted/on-chain data (anomaly A2, mandatory pre-mainnet).
    let demo = Router::new()
        .route("/api/v1/agents", get(list_agents))
        .route("/api/v1/agents/:id", get(get_agent))
        .route("/api/v1/strategies", get(list_strategies))
        .route("/api/v1/proofs", get(list_proofs))
        .route("/api/v1/leaderboard", get(get_leaderboard))
        .route_layer(from_fn(demo_data_header));

    let protected = Router::new()
        .route("/api/v1/intents/parse", post(parse_intent))
        .route("/api/v1/intents/plan", post(plan_intent))
        .route("/api/v1/intents/:id", get(get_intent).delete(delete_intent))
        .route("/api/v1/intents", get(list_intents).post(create_intent))
        .route(
            "/api/v1/delegation",
            get(list_delegations).post(set_delegation),
        )
        .route("/api/v1/delegation/hash", post(delegation_hash))
        .route("/api/v1/agents/:id/pubkey", get(get_agent_pubkey))
        .route("/api/v1/strategies", post(create_strategy))
        .route("/api/v1/strategies/:id", get(get_strategy))
        .route("/api/v1/strategies/:id/fork", post(fork_strategy))
        .route("/api/v1/portfolio", get(get_portfolio))
        .route("/api/v1/executions", get(list_executions))
        .route("/api/v1/rebates", get(list_rebates))
        .route("/api/v1/rebates/pending", post(pending_rebates))
        .route("/api/v1/orchestrator/state", get(orchestrator_state))
        .merge(demo)
        .route_layer(from_fn_with_state(state.clone(), auth_middleware));

    public
        .merge(protected)
        .layer(cors)
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .with_state(state)
}

fn build_cors(origins: &str) -> CorsLayer {
    if origins.trim() == "*" {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        let allowed: Vec<header::HeaderValue> = origins
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse::<header::HeaderValue>().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(allowed))
            .allow_methods(Any)
            .allow_headers(Any)
    }
}

/// Stamps `X-Demo-Data: true` on responses so clients can tell the payload
/// embeds built-in demonstration data (anomaly A2). Applied only to the demo
/// route group; remove together with the hardcoded data pre-mainnet.
async fn demo_data_header(request: Request<Body>, next: Next) -> Response {
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        header::HeaderName::from_static("x-demo-data"),
        header::HeaderValue::from_static("true"),
    );
    response
}

async fn auth_middleware(
    AxumState(state): AxumState<Arc<AppState>>,
    headers: header::HeaderMap,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    if !state.auth_enabled {
        request.extensions_mut().insert(Option::<AuthUser>::None);
        return next.run(request).await;
    }

    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let service = match state.auth_service.as_ref() {
        Some(s) => s,
        None => {
            return unauthorized("authentication service unavailable");
        }
    };

    match token {
        Some(token) => match service.validate_token(token) {
            Ok(user) => {
                request.extensions_mut().insert(Some(user));
                next.run(request).await
            }
            Err(err) => unauthorized(&format!("invalid token: {}", err)),
        },
        None => unauthorized("missing or invalid authorization header"),
    }
}

fn unauthorized(message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: message.to_string(),
        }),
    )
        .into_response()
}

async fn rate_limit_middleware(
    AxumState(state): AxumState<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if state.rate_limit_per_minute == 0 {
        return next.run(request).await;
    }

    let key = rate_limit_key(&state, request.headers(), addr.ip());
    let now = Instant::now();
    let window = Duration::from_secs(60);

    let mut counts = state.request_counts.lock().await;
    let entries = counts.entry(key).or_default();
    entries.retain(|t| now.duration_since(*t) < window);

    if entries.len() >= state.rate_limit_per_minute as usize {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(ErrorResponse {
                error: "rate limit exceeded".to_string(),
            }),
        )
            .into_response();
    }

    entries.push(now);
    drop(counts);

    next.run(request).await
}

/// Rate limiting is per authenticated user (JWT `sub`) when a valid token is
/// presented, per source IP otherwise (US-422).
fn rate_limit_key(state: &AppState, headers: &header::HeaderMap, ip: std::net::IpAddr) -> String {
    if let Some(service) = state.auth_service.as_ref() {
        let token = headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));
        if let Some(user) = token.and_then(|t| service.validate_token(t).ok()) {
            return format!("user:{}", user.address);
        }
    }
    format!("ip:{}", ip)
}

async fn auth_challenge(
    AxumState(state): AxumState<Arc<AppState>>,
    Json(body): Json<ChallengeRequest>,
) -> Result<Json<ChallengeResponse>, AppError> {
    let service = state
        .auth_service
        .as_ref()
        .ok_or_else(|| AppError::Internal("authentication disabled".to_string()))?;
    let message = service
        .generate_challenge(&body.address)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(ChallengeResponse { message }))
}

async fn auth_verify(
    AxumState(state): AxumState<Arc<AppState>>,
    Json(body): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, AppError> {
    let service = state
        .auth_service
        .as_ref()
        .ok_or_else(|| AppError::Internal("authentication disabled".to_string()))?;
    let token = service
        .verify_signature(&body.message, &body.signature)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(VerifyResponse { token }))
}

async fn ws_handler(ws: WebSocketUpgrade, AxumState(state): AxumState<Arc<AppState>>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.event_tx.subscribe();
    while let Ok(event) = rx.recv().await {
        let payload = match serde_json::to_string(&event) {
            Ok(json) => json,
            Err(err) => {
                tracing::warn!(%err, "failed to serialize event for websocket");
                continue;
            }
        };
        if socket.send(Message::Text(payload)).await.is_err() {
            break;
        }
    }
}

fn default_agents() -> Vec<AgentSummary> {
    vec![
        AgentSummary {
            id: "agent-1".to_string(),
            name: "Aave Ace".to_string(),
            operated_by: "Otter".to_string(),
            demo: true,
            risk_profile: "Conservative".to_string(),
            bond: 50_000,
            reputation: 4.9,
            proofs_submitted: 12_403,
            yield_generated: 2_450_000,
            mev_captured: 18_400,
            uptime: 99.98,
            strategies: 12,
            followers: 3_420,
            description: "Otter-operated lending specialist. Executes conservative lending strategies across Aave markets with steady, audited yields.".to_string(),
        },
        AgentSummary {
            id: "agent-2".to_string(),
            name: "Uni-Unicorn".to_string(),
            operated_by: "Otter".to_string(),
            demo: true,
            risk_profile: "Balanced".to_string(),
            bond: 75_000,
            reputation: 4.7,
            proofs_submitted: 8_932,
            yield_generated: 4_120_000,
            mev_captured: 52_300,
            uptime: 99.91,
            strategies: 8,
            followers: 2_180,
            description: "Otter-operated liquidity execution agent. Runs protected swap and LP flows, capturing MEV rebates for depositors.".to_string(),
        },
        AgentSummary {
            id: "agent-3".to_string(),
            name: "Compound King".to_string(),
            operated_by: "Otter".to_string(),
            demo: true,
            risk_profile: "Conservative".to_string(),
            bond: 32_000,
            reputation: 4.5,
            proofs_submitted: 5_611,
            yield_generated: 980_000,
            mev_captured: 6_100,
            uptime: 99.85,
            strategies: 5,
            followers: 890,
            description: "Otter-operated Compound specialist. Automates rate arbitrage and rebalancing between Compound markets.".to_string(),
        },
        AgentSummary {
            id: "agent-4".to_string(),
            name: "Cross-Chain Carl".to_string(),
            operated_by: "Otter".to_string(),
            demo: true,
            risk_profile: "Advanced".to_string(),
            bond: 100_000,
            reputation: 4.8,
            proofs_submitted: 3_420,
            yield_generated: 1_890_000,
            mev_captured: 12_400,
            uptime: 99.72,
            strategies: 6,
            followers: 1_560,
            description: "Otter-operated multi-chain strategist. Chases the best risk-adjusted yields across Ethereum and Arbitrum.".to_string(),
        },
    ]
}

fn default_strategies() -> Vec<StrategySummary> {
    vec![
        StrategySummary {
            id: "strategy-1".to_string(),
            agent_id: "agent-1".to_string(),
            agent_name: "Aave Ace".to_string(),
            title: "Steady USDC Lending".to_string(),
            description: "Otter official strategy. Lend USDC on Aave Ethereum whenever supply APY exceeds 3%.".to_string(),
            raw_text: "Lend USDC on Aave if yield > 3%".to_string(),
            risk_profile: "Conservative".to_string(),
            copies: 1_240,
            visibility: "public".to_string(),
            fork_count: 1_240,
            total_volume: 5_400_000,
            apy: 4.1,
            created_at: 1_720_000_000,
            updated_at: 1_720_000_000,
        },
        StrategySummary {
            id: "strategy-2".to_string(),
            agent_id: "agent-2".to_string(),
            agent_name: "Uni-Unicorn".to_string(),
            title: "Low-Gas ETH Swaps".to_string(),
            description: "Otter official strategy. Swap USDC to ETH on Uniswap only when base fee is below 20 gwei.".to_string(),
            raw_text: "Swap USDC to ETH on Uniswap when gas < 20 gwei".to_string(),
            risk_profile: "Balanced".to_string(),
            copies: 856,
            visibility: "public".to_string(),
            fork_count: 856,
            total_volume: 2_100_000,
            apy: 0.0,
            created_at: 1_720_500_000,
            updated_at: 1_720_500_000,
        },
        StrategySummary {
            id: "strategy-3".to_string(),
            agent_id: "agent-4".to_string(),
            agent_name: "Cross-Chain Carl".to_string(),
            title: "Arbitrum Yield Chase".to_string(),
            description: "Otter official strategy. Move USDC to the highest yielding Aave or Compound market across chains.".to_string(),
            raw_text: "Lend USDC on highest yield market across Ethereum and Arbitrum".to_string(),
            risk_profile: "Advanced".to_string(),
            copies: 643,
            visibility: "public".to_string(),
            fork_count: 643,
            total_volume: 1_800_000,
            apy: 5.2,
            created_at: 1_720_900_000,
            updated_at: 1_720_900_000,
        },
    ]
}

fn load_agent_pubkey(private_key: &[u8; 32]) -> Option<AgentPubkey> {
    let wallet = LocalWalletAdapter::from_bytes(private_key).ok()?;
    let (x, y) = wallet.pubkey().ok()?;
    Some(AgentPubkey {
        x: format!("0x{}", hex::encode(x)),
        y: format!("0x{}", hex::encode(y)),
    })
}

#[tokio::main]
async fn main() {
    let config = load_config();

    let env_filter = tracing_subscriber::EnvFilter::try_new(&config.log_level)
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    if config.log_format.eq_ignore_ascii_case("json") {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(env_filter)
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    }

    if let Err(err) = config.validate() {
        tracing::error!(%err, "configuration validation failed");
        std::process::exit(1);
    }

    let storage: Arc<dyn StoragePort> = if config.database_url.starts_with("postgres://") {
        Arc::new(
            PgStorage::new(&config.database_url)
                .await
                .expect("failed to connect to PostgreSQL storage"),
        )
    } else {
        Arc::new(SqliteStorage::new(&config.database_url).expect("failed to open SQLite storage"))
    };

    let (orchestrator, execution_enabled, mev_store) = build_orchestrator(&config).await;
    let orchestrator = Arc::new(RwLock::new(orchestrator));

    // Hydrate in-memory active intents from storage so monitoring survives
    // process restarts.
    if let Err(err) = hydrate_active_intents(&orchestrator, &storage).await {
        tracing::error!(?err, "failed to hydrate active intents from storage");
    }

    // Seed default strategy templates when the table is empty so the UI has
    // something to display on a fresh install.
    if let Ok(records) = storage.list_strategies().await {
        if records.is_empty()
            && let Err(err) = seed_default_strategies(&storage, &orchestrator).await
        {
            tracing::warn!(?err, "failed to seed default strategies");
        }
    } else {
        tracing::warn!("failed to check strategies table for seeding");
    }

    let metrics = Arc::new(Metrics::default());
    let (bus, mut receiver) = EventBus::new(256);
    let (event_tx, _) = tokio::sync::broadcast::channel::<Event>(256);

    let auth_service = if config.auth_enabled {
        let secret = match resolve_jwt_secret(&config) {
            Ok(secret) => secret,
            Err(err) => {
                tracing::error!(%err, "invalid auth configuration");
                std::process::exit(1);
            }
        };
        Some(Arc::new(AuthService::new(secret, config.jwt_ttl_hours)))
    } else {
        None
    };

    let state = Arc::new(AppState {
        orchestrator: orchestrator.clone(),
        storage: storage.clone(),
        bus: bus.clone(),
        metrics: metrics.clone(),
        execution_enabled,
        metrics_enabled: config.metrics_enabled,
        version: env!("CARGO_PKG_VERSION"),
        auth_enabled: config.auth_enabled,
        auth_service,
        rate_limit_per_minute: config.rate_limit_per_minute,
        request_counts: Arc::new(Mutex::new(HashMap::new())),
        cors_allowed_origins: config.cors_allowed_origins.clone(),
        event_tx: event_tx.clone(),
        agents: default_agents(),
        agent_pubkey: load_private_key(&config)
            .ok()
            .and_then(|(key, _)| load_agent_pubkey(&key)),
        multichain: build_multichain_adapter(&config),
        network_health: Arc::new(Mutex::new(HashMap::new())),
        mev: mev_store.clone(),
        rebate_bps: infrastructure::mev::rebate_bps_from_env(),
    });

    // Background monitoring loop: fetch on-chain metrics for active intents.
    let monitor_state = Arc::clone(&state);
    let monitor_interval = Duration::from_secs(config.monitoring_interval_secs.max(5));
    tokio::spawn(async move {
        monitoring_loop(monitor_state, monitor_interval).await;
    });

    // Event processor loop: drive the orchestrator state machine.
    let processor_state = Arc::clone(&state);
    tokio::spawn(async move {
        while let Some(event) = receiver.recv().await {
            let _ = processor_state.event_tx.send(event.clone());

            track_event(&processor_state, &event);
            persist_event(&processor_state, &event).await;

            let mut orchestrator = processor_state.write_orchestrator().await;
            orchestrator
                .process_event(event, &processor_state.bus)
                .await;
        }
    });

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.api_port))
        .await
        .expect("failed to bind API port");

    tracing::info!(
        execution_enabled,
        "Otter API listening on http://0.0.0.0:{}",
        config.api_port
    );

    let server = axum::serve(
        listener,
        app(state).into_make_service_with_connect_info::<std::net::SocketAddr>(),
    );
    tokio::select! {
        result = server => result.expect("server failed"),
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("shutdown signal received");
        }
    }
}

fn load_config() -> Config {
    let path = std::env::var("OTTER_CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
    if std::path::Path::new(&path).exists() {
        Config::from_file(&path).expect("failed to load config file")
    } else {
        Config::from_env()
    }
}

/// Resolve the JWT secret used to sign tokens. On public networks
/// (`mainnet`/`sepolia`) an explicit `OTTER_JWT_SECRET` is mandatory: starting
/// with a random secret would silently invalidate every token on restart.
/// Locally (no network or any other value) a random dev secret is allowed,
/// with a warning.
fn resolve_jwt_secret(config: &Config) -> Result<String, String> {
    if !config.jwt_secret.is_empty() {
        return Ok(config.jwt_secret.clone());
    }
    let network = config.network.as_deref().unwrap_or_default().to_lowercase();
    if matches!(network.as_str(), "mainnet" | "sepolia") {
        return Err(format!(
            "OTTER_JWT_SECRET must be set when auth is enabled on network '{}'; \
             refusing to start with a random secret",
            network
        ));
    }
    tracing::warn!("auth enabled but no OTTER_JWT_SECRET set; generating a random dev secret");
    let bytes: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
    Ok(hex::encode(bytes))
}

/// Build the multi-network EVM registry from config. Requires the agent
/// private key (shared across networks); without one the registry is empty
/// and `/api/v1/networks` reports no networks.
fn build_multichain_adapter(config: &Config) -> Arc<MultiChainAdapter> {
    let networks = config.resolve_networks();
    match load_private_key(config) {
        Ok((key, _)) => MultiChainAdapter::new(&networks, &hex::encode(key), None)
            .map(Arc::new)
            .unwrap_or_else(|err| {
                tracing::warn!(%err, "multi-chain adapter initialization failed");
                Arc::new(MultiChainAdapter::empty())
            }),
        Err(_) => {
            tracing::info!("no agent private key configured; multi-chain routing disabled");
            Arc::new(MultiChainAdapter::empty())
        }
    }
}

#[derive(Debug, Serialize)]
struct NetworkStatusResponse {
    name: String,
    chain_id: u64,
    vault_address: String,
    healthy: bool,
}

async fn list_networks(
    AxumState(state): AxumState<Arc<AppState>>,
) -> Json<Vec<NetworkStatusResponse>> {
    let summaries = state.multichain.network_summaries();
    let mut out = Vec::with_capacity(summaries.len());
    for summary in summaries {
        let cached = {
            let map = state.network_health.lock().await;
            map.get(&summary.name).cloned()
        };
        let healthy = match cached {
            Some(entry) if entry.is_fresh() => entry.healthy,
            _ => {
                // Healthcheck: a successful eth_chainId round-trip.
                let healthy = state
                    .multichain
                    .rpc_chain_id(Some(&summary.name))
                    .await
                    .is_ok();
                state.network_health.lock().await.insert(
                    summary.name.clone(),
                    HealthEntry {
                        healthy,
                        checked_at: std::time::Instant::now(),
                    },
                );
                healthy
            }
        };
        out.push(NetworkStatusResponse {
            name: summary.name,
            chain_id: summary.chain_id,
            vault_address: summary.vault_address,
            healthy,
        });
    }
    Json(out)
}

#[derive(Debug, Serialize)]
struct RebatesResponse {
    /// Total rebated profit for the caller, in wei.
    total_rebated_wei: String,
    rebate_bps: u64,
}

async fn list_rebates(
    AxumState(state): AxumState<Arc<AppState>>,
    Extension(user): Extension<Option<AuthUser>>,
) -> Result<Json<RebatesResponse>, AppError> {
    let Some(mev) = &state.mev else {
        return Ok(Json(RebatesResponse {
            total_rebated_wei: "0".to_string(),
            rebate_bps: state.rebate_bps,
        }));
    };
    // Without auth, attribute all captures to the single local operator.
    let owner = user
        .as_ref()
        .map(|u| u.address.clone())
        .unwrap_or_else(|| "local-operator".to_string());
    let total = mev
        .total_rebate(&owner)
        .map_err(|e| AppError::Internal(format!("rebate lookup failed: {e}")))?;
    Ok(Json(RebatesResponse {
        total_rebated_wei: total.to_string(),
        rebate_bps: state.rebate_bps,
    }))
}

/// Preview endpoint for pending rebates (same value as `list_rebates` in V1:
/// captures are rebated as soon as they are recorded).
async fn pending_rebates(
    state: AxumState<Arc<AppState>>,
    user: Extension<Option<AuthUser>>,
) -> Result<Json<RebatesResponse>, AppError> {
    list_rebates(state, user).await
}

fn parse_network(value: Option<&String>) -> OracleNetwork {
    match value.map(|s| s.to_lowercase()).as_deref() {
        Some("mainnet") => OracleNetwork::Mainnet,
        _ => OracleNetwork::Sepolia,
    }
}

async fn build_orchestrator(
    config: &Config,
) -> (
    AgentOrchestrator,
    bool,
    Option<Arc<infrastructure::mev::SimulatedMevCapture>>,
) {
    let parser = build_intent_parser(config);
    let network = parse_network(config.network.as_ref());
    let oracle = CompositeOracle::new(config.rpc_url.clone(), network)
        .expect("failed to initialize composite oracle");
    let zkp = NoirAdapter::from_config(config);

    // Simulated MEV capture store, sharing the main SQLite database. Only
    // available for SQLite backends; PostgreSQL gets `None` (rebates read
    // as zero) until the capture adapter grows a pg implementation.
    let mev_store = if config.database_url.starts_with("postgres://") {
        None
    } else {
        infrastructure::mev::SimulatedMevCapture::new(&config.database_url)
            .map(Arc::new)
            .map_err(|err| tracing::warn!(%err, "MEV capture store unavailable"))
            .ok()
    };

    // Load the agent private key once; it is used both for EVM transaction
    // signing and for signing delegation messages inside the execution service.
    let maybe_private_key = if config.vault_address.is_some() {
        load_private_key(config).ok()
    } else {
        None
    };

    // If no real credentials are provided, fall back to a dummy EVM adapter.
    // Execution is disabled in that case so the adapter is never invoked.
    let evm = match (&maybe_private_key, &config.vault_address) {
        (Some((pk, _)), Some(vault)) => {
            AlloyEvmAdapter::new(config.rpc_url.clone(), &hex::encode(pk), vault)
                .expect("failed to initialize EVM adapter")
        }
        _ => dummy_evm_adapter(config),
    };

    if config.execution_enabled && config.vault_address.is_some() {
        let (private_key, key_source) =
            maybe_private_key.expect("execution is enabled but no agent private key was provided");
        tracing::info!(key_source = %key_source, "loaded agent private key");
        let starting_nonce = evm
            .get_transaction_count()
            .await
            .map(|n| n + 1)
            .unwrap_or(1);
        let execution: Arc<dyn ExecutionPort> = if let Some(mev) = &mev_store {
            Arc::new(
                OnChainExecutionService::new(
                    RegexParser::new(),
                    oracle.clone(),
                    zkp.clone(),
                    evm.clone(),
                    &private_key,
                    starting_nonce,
                    &config.nonce_store_path,
                    config.chain_id,
                )
                .expect("failed to initialize execution service")
                .with_mev(Arc::clone(mev) as Arc<dyn domain::ports::mev_port::MevPort>),
            )
        } else {
            Arc::new(
                OnChainExecutionService::new(
                    RegexParser::new(),
                    oracle.clone(),
                    zkp.clone(),
                    evm.clone(),
                    &private_key,
                    starting_nonce,
                    &config.nonce_store_path,
                    config.chain_id,
                )
                .expect("failed to initialize execution service"),
            )
        };
        return (
            Orchestrator::new_with_executor(parser, oracle, zkp, evm, execution),
            true,
            mev_store,
        );
    }

    tracing::warn!(
        "running without on-chain execution; set OTTER_EXECUTION_ENABLED=true and provide OTTER_PRIVATE_KEY / OTTER_VAULT_ADDRESS to enable"
    );
    (Orchestrator::new(parser, oracle, zkp, evm), false, mev_store)
}

/// Build the intent parser for the API daemon. When a GGUF model exists at
/// `config.model_path` and loads successfully, use the hybrid parser (LLM with
/// regex fallback); otherwise keep the deterministic regex parser so the API
/// stays fully functional without a model.
fn build_intent_parser(config: &Config) -> AgentParser {
    if std::path::Path::new(&config.model_path).exists() {
        let llm = LlmIntentParser::new(&config.model_path);
        let load_result = llm.client_mut().load();
        match load_result {
            Ok(()) => {
                tracing::info!(
                    model_path = %config.model_path,
                    "LLM model loaded; using hybrid intent parser (LLM with regex fallback)"
                );
                return Arc::new(HybridParser::new(llm));
            }
            Err(err) => {
                tracing::warn!(
                    model_path = %config.model_path,
                    %err,
                    "failed to load LLM model; falling back to regex intent parser"
                );
            }
        }
    } else {
        tracing::info!(
            model_path = %config.model_path,
            "no LLM model found; using regex intent parser"
        );
    }
    Arc::new(RegexParser::new())
}

fn dummy_evm_adapter(config: &Config) -> AlloyEvmAdapter {
    let dummy_key = "0x0000000000000000000000000000000000000000000000000000000000000001";
    let dummy_vault = "0x0000000000000000000000000000000000000000";
    AlloyEvmAdapter::new(config.rpc_url.clone(), dummy_key, dummy_vault)
        .expect("dummy EVM adapter should always be valid")
}

async fn hydrate_active_intents(
    orchestrator: &Arc<RwLock<AgentOrchestrator>>,
    storage: &Arc<dyn StoragePort>,
) -> Result<(), AppError> {
    let records = storage.list_intents().await?;
    let mut orchestrator = orchestrator.write().await;
    for record in records {
        if record.state == "active" {
            orchestrator.add_active_intent(record.id, record.text, record.intent);
        }
    }
    Ok(())
}

async fn monitoring_loop(state: Arc<AppState>, interval: Duration) {
    let mut tick = tokio::time::interval(interval);
    loop {
        tick.tick().await;

        let (intents, oracle) = {
            let orchestrator = state.read_orchestrator().await;
            (
                orchestrator.active_intents().to_vec(),
                orchestrator.oracle_adapter().clone(),
            )
        };

        if intents.is_empty() {
            continue;
        }

        // Fetch the union of (asset, metric) pairs currently being monitored.
        let pairs = collect_metric_pairs(&intents);

        for (asset, metric) in pairs {
            let oracle = oracle.clone();
            let asset_for_task = asset.clone();
            let value =
                tokio::task::spawn_blocking(move || oracle.fetch(&metric, Some(&asset_for_task)))
                    .await;

            match value {
                Ok(Ok(value)) => {
                    if let Err(err) = state.bus.publish(Event::PriceUpdated {
                        asset: asset.clone(),
                        metric,
                        value,
                    }) {
                        tracing::debug!(%err, "event bus full while publishing price update");
                    }
                }
                Ok(Err(err)) => {
                    tracing::warn!(?asset, ?metric, %err, "metric fetch failed");
                    state.metrics.rpc_errors.fetch_add(1, Ordering::Relaxed);
                    let _ = state.bus.publish(Event::Error {
                        source: "monitor".to_string(),
                        message: format!("{:?} {:?} fetch failed: {}", asset, metric, err),
                    });
                }
                Err(err) => {
                    tracing::error!(?asset, ?metric, %err, "metric fetch task failed");
                    state.metrics.rpc_errors.fetch_add(1, Ordering::Relaxed);
                    let _ = state.bus.publish(Event::Error {
                        source: "monitor".to_string(),
                        message: format!("{:?} {:?} fetch task failed: {}", asset, metric, err),
                    });
                }
            }
        }
    }
}

fn collect_metric_pairs(intents: &[ActiveIntent]) -> HashSet<(Asset, Metric)> {
    let mut pairs = HashSet::new();
    for intent in intents {
        let asset = AgentOrchestrator::primary_asset_of(&intent.conditional.intent);
        if let Some(condition) = &intent.conditional.condition {
            pairs.insert((asset, *condition.metric()));
        } else {
            // Default to price monitoring for unconditional intents.
            pairs.insert((asset, Metric::Price));
        }
    }
    pairs
}

fn track_event(state: &AppState, event: &Event) {
    match event {
        Event::PriceUpdated { .. } => state.metrics.price_updates.fetch_add(1, Ordering::Relaxed),
        Event::ConditionMet { .. } => state.metrics.conditions_met.fetch_add(1, Ordering::Relaxed),
        Event::TransactionConfirmed { .. } => {
            state.metrics.executions.fetch_add(1, Ordering::Relaxed)
        }
        Event::Error { .. } => state.metrics.errors.fetch_add(1, Ordering::Relaxed),
        _ => 0,
    };
}

async fn persist_event(state: &AppState, event: &Event) {
    let update = match event {
        Event::TransactionSubmitted { intent_id, tx_hash } => {
            Some((intent_id.clone(), format!("submitted:{}", tx_hash)))
        }
        Event::TransactionConfirmed {
            intent_id,
            receipt,
            gas_used,
        } => {
            state
                .metrics
                .gas_used_total
                .fetch_add(*gas_used, Ordering::Relaxed);
            let exec_record = ExecutionRecord {
                id: format!("{}-{}", intent_id, receipt),
                intent_id: intent_id.clone(),
                tx_hash: receipt.clone(),
                status: "success".to_string(),
                gas_used: *gas_used,
                created_at: now_secs(),
            };
            if let Err(err) = state.storage.save_execution(&exec_record).await {
                tracing::error!(?err, intent_id = %intent_id, "failed to persist execution record");
            }
            Some((intent_id.clone(), format!("executed:{}", receipt)))
        }
        Event::Error { source, message } if source == "executor" => {
            // We do not know the intent id here; leave state transition to
            // explicit error handling in future versions.
            None
        }
        _ => None,
    };

    if let Some((id, state_str)) = update
        && let Err(err) = update_intent_state(&state.storage, &id, &state_str).await
    {
        tracing::error!(?err, intent_id = %id, "failed to persist intent state");
    }
}

async fn update_intent_state(
    storage: &Arc<dyn StoragePort>,
    id: &str,
    new_state: &str,
) -> Result<(), AppError> {
    let mut record = storage.get_intent(id).await?.ok_or(AppError::Storage(
        domain::ports::StorageError::NotFound(id.to_string()),
    ))?;
    record.state = new_state.to_string();
    record.updated_at = now_secs();
    storage.save_intent(&record).await?;
    Ok(())
}

const MAX_INTENT_TEXT_LEN: usize = 2000;

fn validate_intent_text(text: &str) -> Result<(), AppError> {
    if text.trim().is_empty() {
        return Err(AppError::Validation("intent text is empty".to_string()));
    }
    if text.len() > MAX_INTENT_TEXT_LEN {
        return Err(AppError::Validation(format!(
            "intent text exceeds {} characters",
            MAX_INTENT_TEXT_LEN
        )));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_delegation_fields(
    pubkey_x: &str,
    pubkey_y: &str,
    allowed_intents: &str,
    max_amounts: &[String],
    allowed_protocols: &[String],
    expiry: &str,
    nonce: &str,
    target_contract: &str,
) -> Result<(), AppError> {
    if max_amounts.len() != 10 {
        return Err(AppError::Validation(format!(
            "max_amounts must have exactly 10 elements, got {}",
            max_amounts.len()
        )));
    }
    if allowed_protocols.len() != 5 {
        return Err(AppError::Validation(format!(
            "allowed_protocols must have exactly 5 elements, got {}",
            allowed_protocols.len()
        )));
    }
    validate_hex_field(pubkey_x, "pubkey_x")?;
    validate_hex_field(pubkey_y, "pubkey_y")?;
    validate_hex_field(allowed_intents, "allowed_intents")?;
    validate_hex_field(expiry, "expiry")?;
    validate_hex_field(nonce, "nonce")?;
    validate_hex_field(target_contract, "target_contract")?;
    for (i, v) in max_amounts.iter().enumerate() {
        validate_hex_field(v, &format!("max_amounts[{i}]"))?;
    }
    for (i, v) in allowed_protocols.iter().enumerate() {
        validate_hex_field(v, &format!("allowed_protocols[{i}]"))?;
    }
    Ok(())
}

fn validate_hex_field(value: &str, name: &str) -> Result<(), AppError> {
    let cleaned = value.trim().strip_prefix("0x").unwrap_or(value);
    if cleaned.len() != 64 {
        return Err(AppError::Validation(format!(
            "{name} must be 32 bytes (64 hex chars), got {}",
            cleaned.len()
        )));
    }
    if !cleaned.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AppError::Validation(format!(
            "{name} contains invalid hex characters"
        )));
    }
    Ok(())
}

fn matches_user(record_user: &Option<String>, user: &Option<String>) -> bool {
    match (record_user, user) {
        // When authentication is disabled the handler receives no user; treat
        // that as a wildcard so the API remains usable in dev/default mode.
        (_, None) => true,
        (Some(a), Some(b)) => a.eq_ignore_ascii_case(b),
        (None, Some(_)) => false,
    }
}

fn decode_hex32(value: &str) -> Result<[u8; 32], String> {
    let cleaned = value.trim().strip_prefix("0x").unwrap_or(value);
    let decoded = hex::decode(cleaned).map_err(|e| format!("invalid hex: {e}"))?;
    if decoded.len() != 32 {
        return Err(format!("expected 32 bytes, got {}", decoded.len()));
    }
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&decoded);
    Ok(bytes)
}

fn decode_hex_array<const N: usize>(values: &[String]) -> Result<[[u8; 32]; N], String> {
    if values.len() != N {
        return Err(format!("expected {} hex strings, got {}", N, values.len()));
    }
    let mut array = [[0u8; 32]; N];
    for (i, v) in values.iter().enumerate() {
        array[i] = decode_hex32(v)?;
    }
    Ok(array)
}

fn decode_signature(values: &[String]) -> Result<[u8; 64], String> {
    if values.len() != 64 {
        return Err(format!(
            "expected 64 byte hex strings, got {}",
            values.len()
        ));
    }
    let mut signature = [0u8; 64];
    for (i, v) in values.iter().enumerate() {
        let cleaned = v.trim().strip_prefix("0x").unwrap_or(v);
        let decoded = hex::decode(cleaned).map_err(|e| format!("invalid signature hex: {e}"))?;
        if decoded.len() != 1 {
            return Err("signature bytes must be 1-byte hex strings".to_string());
        }
        signature[i] = decoded[0];
    }
    Ok(signature)
}

async fn set_delegation(
    AxumState(state): AxumState<Arc<AppState>>,
    Extension(user): Extension<Option<AuthUser>>,
    Json(body): Json<SetDelegationRequest>,
) -> Result<Json<SetDelegationResponse>, AppError> {
    validate_delegation_fields(
        &body.pubkey_x,
        &body.pubkey_y,
        &body.allowed_intents,
        &body.max_amounts,
        &body.allowed_protocols,
        &body.expiry,
        &body.nonce,
        &body.target_contract,
    )?;

    let user_address = user.map(|u| u.address);
    let delegation = DelegationMessage {
        pubkey_x: decode_hex32(&body.pubkey_x).map_err(AppError::Validation)?,
        pubkey_y: decode_hex32(&body.pubkey_y).map_err(AppError::Validation)?,
        allowed_intents: decode_hex32(&body.allowed_intents).map_err(AppError::Validation)?,
        max_amounts: decode_hex_array::<10>(&body.max_amounts).map_err(AppError::Validation)?,
        allowed_protocols: decode_hex_array::<5>(&body.allowed_protocols)
            .map_err(AppError::Validation)?,
        expiry: decode_hex32(&body.expiry).map_err(AppError::Validation)?,
        nonce: decode_hex32(&body.nonce).map_err(AppError::Validation)?,
        target_contract: decode_hex32(&body.target_contract).map_err(AppError::Validation)?,
    };
    let signature = decode_signature(&body.signature).map_err(AppError::Validation)?;
    let delegation_hash = domain::models::delegation::hash_delegation(&delegation);

    {
        let mut orchestrator = state.write_orchestrator().await;
        orchestrator.set_delegation(delegation.clone(), signature);
    }

    let hash_hex = format!("0x{}", hex::encode(delegation_hash));

    let payload_json = serde_json::to_string(&delegation)
        .map_err(|e| AppError::Internal(format!("failed to serialize delegation: {e}")))?;
    let delegation_record = DelegationRecord {
        hash: hash_hex.clone(),
        payload_json,
        signature: format!("0x{}", hex::encode(signature)),
        created_at: now_secs(),
        user_address,
    };
    state.storage.save_delegation(&delegation_record).await?;

    tracing::info!(%hash_hex, "delegation set and persisted");
    Ok(Json(SetDelegationResponse {
        delegation_hash: hash_hex,
    }))
}

async fn delegation_hash(
    Json(body): Json<DelegationHashRequest>,
) -> Result<Json<DelegationHashResponse>, AppError> {
    validate_delegation_fields(
        &body.pubkey_x,
        &body.pubkey_y,
        &body.allowed_intents,
        &body.max_amounts,
        &body.allowed_protocols,
        &body.expiry,
        &body.nonce,
        &body.target_contract,
    )?;

    let delegation = DelegationMessage {
        pubkey_x: decode_hex32(&body.pubkey_x).map_err(AppError::Validation)?,
        pubkey_y: decode_hex32(&body.pubkey_y).map_err(AppError::Validation)?,
        allowed_intents: decode_hex32(&body.allowed_intents).map_err(AppError::Validation)?,
        max_amounts: decode_hex_array::<10>(&body.max_amounts).map_err(AppError::Validation)?,
        allowed_protocols: decode_hex_array::<5>(&body.allowed_protocols)
            .map_err(AppError::Validation)?,
        expiry: decode_hex32(&body.expiry).map_err(AppError::Validation)?,
        nonce: decode_hex32(&body.nonce).map_err(AppError::Validation)?,
        target_contract: decode_hex32(&body.target_contract).map_err(AppError::Validation)?,
    };
    let hash = domain::models::delegation::hash_delegation(&delegation);
    Ok(Json(DelegationHashResponse {
        delegation_hash: format!("0x{}", hex::encode(hash)),
    }))
}

async fn list_agents(AxumState(state): AxumState<Arc<AppState>>) -> Json<AgentsResponse> {
    Json(AgentsResponse {
        agents: state.agents.clone(),
        demo: true,
    })
}

async fn get_agent(
    AxumState(state): AxumState<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<AgentSummary>, AppError> {
    let agent = state
        .agents
        .iter()
        .find(|a| a.id == id)
        .ok_or(AppError::Storage(domain::ports::StorageError::NotFound(id)))?;
    Ok(Json(agent.clone()))
}

async fn get_agent_pubkey(
    AxumState(state): AxumState<Arc<AppState>>,
) -> Result<Json<AgentPubkeyResponse>, AppError> {
    let pubkey = state
        .agent_pubkey
        .as_ref()
        .ok_or_else(|| AppError::Internal("agent public key not configured".to_string()))?;
    Ok(Json(AgentPubkeyResponse {
        pubkey_x: pubkey.x.clone(),
        pubkey_y: pubkey.y.clone(),
    }))
}

async fn list_strategies(
    AxumState(state): AxumState<Arc<AppState>>,
) -> Result<Json<StrategiesResponse>, AppError> {
    let records = state.storage.list_strategies().await?;
    let strategies = records
        .into_iter()
        .map(map_strategy_record_to_summary)
        .collect();
    Ok(Json(StrategiesResponse {
        strategies,
        demo: true,
    }))
}

async fn get_strategy(
    AxumState(state): AxumState<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<StrategyDetailResponse>, AppError> {
    let record = state
        .storage
        .get_strategy(&id)
        .await?
        .ok_or(AppError::Storage(domain::ports::StorageError::NotFound(id)))?;
    Ok(Json(map_strategy_record_to_detail(record, &state.agents)))
}

async fn create_strategy(
    AxumState(state): AxumState<Arc<AppState>>,
    Extension(user): Extension<Option<AuthUser>>,
    Json(body): Json<CreateStrategyRequest>,
) -> Result<Json<StrategyCreatedResponse>, AppError> {
    let conditional = {
        let mut orchestrator = state.write_orchestrator().await;
        orchestrator.parse(&body.raw_text)?
    };

    let strategy = domain::models::strategy::Strategy {
        id: format!("strategy-{}", uuid::Uuid::new_v4()),
        title: body.title,
        description: body.description,
        raw_text: body.raw_text,
        intent: conditional,
        creator_address: user.map(|u| u.address),
        agent_id: body.agent_id,
        risk_profile: body.risk_profile,
        copies: 0,
        visibility: domain::models::strategy::StrategyVisibility::parse(
            body.visibility.as_deref().unwrap_or("private"),
        ),
        fork_count: 0,
        total_volume: 0,
        apy: 0.0,
        created_at: now_secs(),
        updated_at: now_secs(),
    };
    strategy
        .validate()
        .map_err(|e| AppError::Validation(format!("{:?}", e)))?;

    let record = strategy_to_record(strategy);
    state.storage.save_strategy(&record).await?;

    tracing::info!(strategy_id = %record.id, "strategy created");
    Ok(Json(StrategyCreatedResponse { id: record.id }))
}

async fn fork_strategy(
    AxumState(state): AxumState<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ForkStrategyResponse>, AppError> {
    state
        .storage
        .get_strategy(&id)
        .await?
        .ok_or(AppError::Storage(domain::ports::StorageError::NotFound(
            id.clone(),
        )))?;
    state.storage.increment_strategy_copies(&id).await?;
    Ok(Json(ForkStrategyResponse {
        strategy_id: id.clone(),
        redirect_to: format!("/app/delegations/new?strategy={}", id),
    }))
}

fn strategy_to_record(strategy: domain::models::strategy::Strategy) -> StrategyRecord {
    StrategyRecord {
        id: strategy.id,
        title: strategy.title,
        description: strategy.description,
        raw_text: strategy.raw_text,
        intent_json: serde_json::to_string(&strategy.intent).expect("intent serializes"),
        creator_address: strategy.creator_address,
        agent_id: strategy.agent_id,
        risk_profile: strategy.risk_profile,
        copies: strategy.copies,
        visibility: strategy.visibility.as_str().to_string(),
        fork_count: strategy.fork_count,
        total_volume: strategy.total_volume,
        apy: strategy.apy,
        created_at: strategy.created_at,
        updated_at: strategy.updated_at,
    }
}

fn map_strategy_record_to_summary(record: StrategyRecord) -> StrategySummary {
    StrategySummary {
        id: record.id,
        agent_id: record.agent_id.clone(),
        agent_name: agent_name_fallback(&record.agent_id),
        title: record.title,
        description: record.description,
        raw_text: record.raw_text,
        risk_profile: record.risk_profile,
        copies: record.copies,
        visibility: record.visibility.clone(),
        fork_count: record.fork_count,
        total_volume: record.total_volume,
        apy: record.apy,
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

fn map_strategy_record_to_detail(
    record: StrategyRecord,
    agents: &[AgentSummary],
) -> StrategyDetailResponse {
    let intent: ConditionalIntent =
        serde_json::from_str(&record.intent_json).expect("stored intent deserializes");
    StrategyDetailResponse {
        id: record.id,
        title: record.title,
        description: record.description,
        raw_text: record.raw_text,
        intent,
        creator_address: record.creator_address,
        agent_id: record.agent_id.clone(),
        agent_name: agents
            .iter()
            .find(|a| a.id == record.agent_id)
            .map(|a| a.name.clone())
            .unwrap_or_else(|| agent_name_fallback(&record.agent_id)),
        risk_profile: record.risk_profile,
        copies: record.copies,
        total_volume: record.total_volume,
        apy: record.apy,
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

fn agent_name_fallback(agent_id: &str) -> String {
    match agent_id {
        "agent-1" => "Aave Ace".to_string(),
        "agent-2" => "Uni-Unicorn".to_string(),
        "agent-3" => "Compound King".to_string(),
        "agent-4" => "Cross-Chain Carl".to_string(),
        _ => "Otter Agent".to_string(),
    }
}

async fn seed_default_strategies(
    storage: &Arc<dyn StoragePort>,
    orchestrator: &Arc<RwLock<AgentOrchestrator>>,
) -> Result<(), AppError> {
    for summary in default_strategies() {
        let intent = {
            let mut orchestrator = orchestrator.write().await;
            match orchestrator.parse(&summary.raw_text) {
                Ok(intent) => intent,
                Err(err) => {
                    tracing::warn!(strategy_id = %summary.id, ?err, "failed to parse default strategy raw text");
                    continue;
                }
            }
        };
        let record = StrategyRecord {
            id: summary.id,
            title: summary.title,
            description: summary.description,
            raw_text: summary.raw_text,
            intent_json: serde_json::to_string(&intent).expect("intent serializes"),
            creator_address: None,
            agent_id: summary.agent_id,
            risk_profile: summary.risk_profile,
            copies: summary.copies,
            visibility: summary.visibility.clone(),
            fork_count: summary.fork_count,
            total_volume: summary.total_volume,
            apy: summary.apy,
            created_at: summary.created_at,
            updated_at: summary.created_at,
        };
        storage.save_strategy(&record).await?;
    }
    Ok(())
}

async fn get_portfolio(
    AxumState(state): AxumState<Arc<AppState>>,
) -> Result<Json<PortfolioResponse>, AppError> {
    let evm = {
        let orchestrator = state.read_orchestrator().await;
        orchestrator.evm_adapter().clone()
    };
    let address = evm.signer_address();

    if state.execution_enabled {
        let address_for_task = address.clone();
        let balance =
            tokio::task::spawn_blocking(move || evm.get_balance(&address_for_task).unwrap_or(0))
                .await
                .map_err(|e| AppError::Internal(format!("balance task failed: {e}")))?;
        Ok(Json(PortfolioResponse {
            address,
            total_balance: balance,
            allocated: balance,
            available: 0,
            yield_earned: 0.0,
            mev_rebates: 0.0,
            positions: vec![],
        }))
    } else {
        Ok(Json(PortfolioResponse {
            address,
            total_balance: 0,
            allocated: 0,
            available: 0,
            yield_earned: 0.0,
            mev_rebates: 0.0,
            positions: vec![],
        }))
    }
}

async fn list_proofs(
    AxumState(state): AxumState<Arc<AppState>>,
) -> Result<Json<ProofsResponse>, AppError> {
    let executions = state.storage.list_executions().await?;
    let mut proofs: Vec<ProofSummary> = executions
        .into_iter()
        .enumerate()
        .map(|(i, e)| ProofSummary {
            id: format!("proof-exec-{}", i),
            proof_type: "execution".to_string(),
            intent_id: Some(e.intent_id),
            verifier: "ExecutionVerifier".to_string(),
            constraints: 512,
            proof_time: 0.9,
            timestamp: e.created_at,
            verified: e.status == "success",
            tx_hash: Some(e.tx_hash),
        })
        .collect();
    proofs.push(ProofSummary {
        id: "proof-solvency-1".to_string(),
        proof_type: "solvency".to_string(),
        intent_id: None,
        verifier: "SolvencyVerifier".to_string(),
        constraints: 1240,
        proof_time: 2.4,
        timestamp: now_secs(),
        verified: true,
        tx_hash: None,
    });
    proofs.sort_by_key(|a| std::cmp::Reverse(a.timestamp));
    Ok(Json(ProofsResponse { proofs, demo: true }))
}

async fn get_leaderboard(AxumState(state): AxumState<Arc<AppState>>) -> Json<LeaderboardResponse> {
    let mut entries: Vec<LeaderboardEntry> = state
        .agents
        .iter()
        .map(|a| LeaderboardEntry {
            rank: 0,
            agent_id: a.id.clone(),
            agent_name: a.name.clone(),
            proofs_submitted: a.proofs_submitted,
            yield_generated: a.yield_generated,
            mev_captured: a.mev_captured,
            uptime: a.uptime,
        })
        .collect();
    entries.sort_by_key(|a| std::cmp::Reverse(a.proofs_submitted));
    for (i, entry) in entries.iter_mut().enumerate() {
        entry.rank = (i + 1) as u32;
    }
    Json(LeaderboardResponse {
        entries,
        demo: true,
    })
}

async fn parse_intent(
    AxumState(state): AxumState<Arc<AppState>>,
    Json(body): Json<ParseRequest>,
) -> Result<Json<ParseResponse>, AppError> {
    validate_intent_text(&body.text)?;

    let intent = {
        let mut orchestrator = state.write_orchestrator().await;
        orchestrator.parse(&body.text)?
    };
    Ok(Json(ParseResponse { intent }))
}

async fn list_intents(
    AxumState(state): AxumState<Arc<AppState>>,
    Extension(user): Extension<Option<AuthUser>>,
) -> Result<Json<IntentsResponse>, AppError> {
    let user = user.map(|u| u.address);
    let records = state.storage.list_intents().await?;
    let intents = records
        .into_iter()
        .filter(|r| matches_user(&r.user_address, &user))
        .map(|r| IntentSummary {
            id: r.id,
            text: r.text,
            state: r.state,
        })
        .collect();
    Ok(Json(IntentsResponse { intents }))
}

async fn list_delegations(
    AxumState(state): AxumState<Arc<AppState>>,
    Extension(user): Extension<Option<AuthUser>>,
) -> Result<Json<DelegationsResponse>, AppError> {
    let user = user.map(|u| u.address);
    let records = state.storage.list_delegations().await?;
    let delegations = records
        .into_iter()
        .filter(|r| matches_user(&r.user_address, &user))
        .map(|r| DelegationSummary {
            hash: r.hash,
            payload_json: r.payload_json,
            signature: r.signature,
            created_at: r.created_at,
        })
        .collect();
    Ok(Json(DelegationsResponse { delegations }))
}

async fn list_executions(
    AxumState(state): AxumState<Arc<AppState>>,
) -> Result<Json<ExecutionsResponse>, AppError> {
    let records = state.storage.list_executions().await?;
    let executions = records
        .into_iter()
        .map(|r| ExecutionSummary {
            id: r.id,
            intent_id: r.intent_id,
            tx_hash: r.tx_hash,
            status: r.status,
            gas_used: r.gas_used,
            created_at: r.created_at,
        })
        .collect();
    Ok(Json(ExecutionsResponse { executions }))
}

async fn create_intent(
    AxumState(state): AxumState<Arc<AppState>>,
    Extension(user): Extension<Option<AuthUser>>,
    Json(body): Json<CreateIntentRequest>,
) -> Result<Json<CreateIntentResponse>, AppError> {
    validate_intent_text(&body.text)?;

    // Reject unknown networks up front so intents never silently fall back to
    // a different chain than the one the user selected.
    if let Some(ref network) = body.network
        && !state
            .multichain
            .network_names()
            .iter()
            .any(|n| n == network)
    {
        return Err(AppError::Validation(format!(
            "unknown network '{}'; configured networks: {:?}",
            network,
            state.multichain.network_names()
        )));
    }

    let user_address = user.map(|u| u.address);
    let conditional = {
        let mut orchestrator = state.write_orchestrator().await;
        orchestrator.parse(&body.text)?
    };

    let id = format!("intent-{}", uuid::Uuid::new_v4());
    let now = now_secs();
    let record = IntentRecord {
        id: id.clone(),
        text: body.text.clone(),
        intent: conditional.clone(),
        state: "active".to_string(),
        created_at: now,
        updated_at: now,
        user_address,
    };
    state.storage.save_intent(&record).await?;

    {
        let mut orchestrator = state.write_orchestrator().await;
        orchestrator.add_active_intent(id.clone(), body.text, conditional);
    }

    tracing::info!(intent_id = %id, "new intent created");
    Ok(Json(CreateIntentResponse { id }))
}

async fn plan_intent(
    AxumState(state): AxumState<Arc<AppState>>,
    Json(body): Json<ParseRequest>,
) -> Result<Json<PlanResponse>, AppError> {
    let plan = {
        let mut orchestrator = state.write_orchestrator().await;
        orchestrator.plan(&body.text)?
    };
    Ok(Json(PlanResponse { plan }))
}

async fn get_intent(
    AxumState(state): AxumState<Arc<AppState>>,
    Extension(user): Extension<Option<AuthUser>>,
    Path(id): Path<String>,
) -> Result<Json<IntentDetailResponse>, AppError> {
    let user = user.map(|u| u.address);
    let record = state
        .storage
        .get_intent(&id)
        .await?
        .ok_or(AppError::Storage(domain::ports::StorageError::NotFound(id)))?;
    if !matches_user(&record.user_address, &user) {
        return Err(AppError::Storage(domain::ports::StorageError::NotFound(
            record.id,
        )));
    }
    Ok(Json(IntentDetailResponse {
        id: record.id,
        text: record.text,
        intent: record.intent,
        state: record.state,
        created_at: record.created_at,
        updated_at: record.updated_at,
    }))
}

async fn delete_intent(
    AxumState(state): AxumState<Arc<AppState>>,
    Extension(user): Extension<Option<AuthUser>>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let user = user.map(|u| u.address);
    let record = state.storage.get_intent(&id).await?;
    if let Some(ref rec) = record
        && !matches_user(&rec.user_address, &user)
    {
        return Err(AppError::Forbidden(
            "not allowed to cancel this intent".to_string(),
        ));
    }

    {
        let mut orchestrator = state.write_orchestrator().await;
        orchestrator.remove_active_intent(&id);
    }

    match record {
        Some(mut record) => {
            record.state = "cancelled".to_string();
            record.updated_at = now_secs();
            state.storage.save_intent(&record).await?;
            tracing::info!(intent_id = %id, "intent cancelled");
            Ok(StatusCode::NO_CONTENT)
        }
        None => Err(AppError::Storage(domain::ports::StorageError::NotFound(id))),
    }
}

async fn orchestrator_state(
    AxumState(state): AxumState<Arc<AppState>>,
) -> Json<OrchestratorStateResponse> {
    let orchestrator = state.read_orchestrator().await;
    let active_intents = orchestrator
        .active_intents()
        .iter()
        .map(|i| ActiveIntentSummary {
            id: i.id.clone(),
            text: i.text.clone(),
            intent: i.conditional.clone(),
        })
        .collect();
    Json(OrchestratorStateResponse {
        state: orchestrator.state().to_string(),
        active_intents,
        execution_enabled: state.execution_enabled,
    })
}

async fn metrics(AxumState(state): AxumState<Arc<AppState>>) -> Response {
    if !state.metrics_enabled {
        return (StatusCode::NOT_FOUND, "metrics disabled").into_response();
    }

    let snapshot = state.metrics.snapshot();
    let (active_intents, witness_ms, prove_ms, verify_ms) = {
        let orchestrator = state.read_orchestrator().await;
        let zkp = orchestrator.zkp_adapter();
        (
            orchestrator.active_intents().len(),
            zkp.last_witness_ms(),
            zkp.last_prove_ms(),
            zkp.last_verify_ms(),
        )
    };

    let body = format!(
        "# HELP otter_price_updates_total Number of on-chain price updates received.\n\
         # TYPE otter_price_updates_total counter\n\
         otter_price_updates_total {}\n\
         # HELP otter_conditions_met_total Number of times an intent condition was satisfied.\n\
         # TYPE otter_conditions_met_total counter\n\
         otter_conditions_met_total {}\n\
         # HELP otter_executions_total Number of successful on-chain executions.\n\
         # TYPE otter_executions_total counter\n\
         otter_executions_total {}\n\
         # HELP otter_errors_total Number of errors reported by the agent.\n\
         # TYPE otter_errors_total counter\n\
         otter_errors_total {}\n\
         # HELP otter_gas_used_total Total gas used by successful on-chain executions.\n\
         # TYPE otter_gas_used_total counter\n\
         otter_gas_used_total {}\n\
         # HELP otter_active_intents Number of currently active intents.\n\
         # TYPE otter_active_intents gauge\n\
         otter_active_intents {}\n\
         # HELP otter_execution_enabled Whether on-chain execution is enabled.\n\
         # TYPE otter_execution_enabled gauge\n\
         otter_execution_enabled {}\n\
         # HELP otter_proof_witness_seconds Last Noir witness generation time in seconds.\n\
         # TYPE otter_proof_witness_seconds gauge\n\
         otter_proof_witness_seconds {}\n\
         # HELP otter_proof_prove_seconds Last bb prove time in seconds.\n\
         # TYPE otter_proof_prove_seconds gauge\n\
         otter_proof_prove_seconds {}\n\
         # HELP otter_proof_verify_seconds Last bb verify time in seconds.\n\
         # TYPE otter_proof_verify_seconds gauge\n\
         otter_proof_verify_seconds {}\n\
         # HELP otter_proof_verification_errors_total Number of failed on-chain proof verifications.\n\
         # TYPE otter_proof_verification_errors_total counter\n\
         otter_proof_verification_errors_total {}\n\
         # HELP otter_rpc_errors_total Number of RPC call failures.\n\
         # TYPE otter_rpc_errors_total counter\n\
         otter_rpc_errors_total {}\n\
         # HELP otter_vault_balance Agent/vault ETH balance in wei.\n\
         # TYPE otter_vault_balance gauge\n\
         otter_vault_balance {}\n",
        snapshot.price_updates,
        snapshot.conditions_met,
        snapshot.executions,
        snapshot.errors,
        snapshot.gas_used_total,
        active_intents,
        if state.execution_enabled { 1 } else { 0 },
        witness_ms as f64 / 1000.0,
        prove_ms as f64 / 1000.0,
        verify_ms as f64 / 1000.0,
        snapshot.proof_verification_errors,
        snapshot.rpc_errors,
        snapshot.vault_balance as f64 / 1e18,
    );

    (
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        body,
    )
        .into_response()
}

async fn health(AxumState(state): AxumState<Arc<AppState>>) -> Response {
    let body = serde_json::json!({
        "status": "up",
        "version": state.version,
        "timestamp": now_secs(),
    });
    (StatusCode::OK, Json(body)).into_response()
}

async fn health_live() -> &'static str {
    "ok"
}

async fn ready(AxumState(state): AxumState<Arc<AppState>>) -> Response {
    let mut checks: Vec<(&'static str, Result<(), String>)> = Vec::new();

    checks.push((
        "storage",
        state
            .storage
            .health_check()
            .await
            .map_err(|e| e.to_string()),
    ));

    let (evm, oracle) = {
        let orchestrator = state.read_orchestrator().await;
        (
            orchestrator.evm_adapter().clone(),
            orchestrator.oracle_adapter().clone(),
        )
    };

    let rpc_task = tokio::task::spawn_blocking(move || {
        evm.get_balance(&evm.signer_address())
            .map(|_| ())
            .map_err(|e| format!("rpc: {}", e))
    });

    let oracle_task = tokio::task::spawn_blocking(move || {
        oracle
            .fetch(&Metric::Price, Some(&Asset::Eth))
            .map(|_| ())
            .map_err(|e| format!("oracle: {}", e))
    });

    let (rpc_check, oracle_check) = tokio::join!(rpc_task, oracle_task);
    checks.push(("rpc", rpc_check.map_err(|e| e.to_string()).and_then(|r| r)));
    checks.push((
        "oracle",
        oracle_check.map_err(|e| e.to_string()).and_then(|r| r),
    ));

    if checks.iter().all(|(_, r)| r.is_ok()) {
        let body = serde_json::json!({"status": "ready", "checks": {}});
        (StatusCode::OK, Json(body)).into_response()
    } else {
        let failures: serde_json::Map<String, serde_json::Value> = checks
            .into_iter()
            .filter_map(|(name, result)| match result {
                Err(msg) => Some((name.to_string(), serde_json::Value::String(msg))),
                Ok(()) => None,
            })
            .collect();
        let body = serde_json::json!({"status": "not_ready", "failures": failures});
        (StatusCode::SERVICE_UNAVAILABLE, Json(body)).into_response()
    }
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[derive(Debug)]
enum AppError {
    Orchestrator(application::orchestrator::OrchestratorError),
    Storage(domain::ports::StorageError),
    Config(String),
    Internal(String),
    Validation(String),
    Forbidden(String),
}

impl From<application::orchestrator::OrchestratorError> for AppError {
    fn from(err: application::orchestrator::OrchestratorError) -> Self {
        Self::Orchestrator(err)
    }
}

impl From<domain::ports::StorageError> for AppError {
    fn from(err: domain::ports::StorageError) -> Self {
        Self::Storage(err)
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        Self::Config(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Orchestrator(err) => {
                let status = match err {
                    application::orchestrator::OrchestratorError::ParseFailed(_)
                    | application::orchestrator::OrchestratorError::InvalidIntent(_)
                    | application::orchestrator::OrchestratorError::PlanFailed(_) => {
                        StatusCode::BAD_REQUEST
                    }
                };
                (status, err.to_string())
            }
            AppError::Storage(err) => {
                let status = match err {
                    domain::ports::StorageError::NotFound(_) => StatusCode::NOT_FOUND,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                };
                (status, err.to_string())
            }
            AppError::Config(msg) | AppError::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
        };
        let body = Json(ErrorResponse { error: message });
        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::{ConnectInfo, Extension};
    use infrastructure::blockchain::{AlloyEvmAdapter, CompositeOracle, OracleNetwork};
    use infrastructure::parsers::RegexParser;
    use infrastructure::storage::SqliteStorage;
    use infrastructure::zkp::NoirAdapter;
    use std::sync::atomic::Ordering;
    use tower::util::ServiceExt;

    fn temp_db_path() -> String {
        let dir = std::env::temp_dir();
        let file = format!("otter-test-{}.db", uuid::Uuid::new_v4());
        dir.join(file).to_string_lossy().to_string()
    }

    async fn test_state() -> Arc<AppState> {
        let parser: AgentParser = Arc::new(RegexParser::new());
        let oracle =
            CompositeOracle::new("http://localhost:8545".to_string(), OracleNetwork::Sepolia)
                .expect("composite oracle should build");
        let zkp = NoirAdapter::new("delegation_circuit", "nargo", None::<String>);
        let evm = AlloyEvmAdapter::new(
            "http://localhost:8545".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000",
        )
        .expect("dummy evm adapter should build");
        let orchestrator = Orchestrator::new(parser, oracle, zkp, evm);

        let storage: Arc<dyn StoragePort> =
            Arc::new(SqliteStorage::new(temp_db_path()).expect("sqlite storage should open"));

        Arc::new(AppState {
            orchestrator: Arc::new(RwLock::new(orchestrator)),
            storage,
            bus: EventBus::new(1).0,
            metrics: Arc::new(Metrics::default()),
            execution_enabled: true,
            metrics_enabled: true,
            version: "0.0.0-test",
            auth_enabled: false,
            auth_service: None,
            rate_limit_per_minute: 0,
            request_counts: Arc::new(Mutex::new(HashMap::new())),
            cors_allowed_origins: "*".to_string(),
            event_tx: tokio::sync::broadcast::channel(1).0,
            agents: default_agents(),
            agent_pubkey: None,
            multichain: Arc::new(infrastructure::blockchain::MultiChainAdapter::empty()),
            network_health: Arc::new(Mutex::new(HashMap::new())),
            mev: None,
            rebate_bps: infrastructure::mev::DEFAULT_REBATE_BPS,
        })
    }

    #[tokio::test]
    async fn metrics_endpoint_returns_prometheus_format() {
        let state = test_state().await;
        state.metrics.price_updates.fetch_add(3, Ordering::Relaxed);
        state.metrics.executions.fetch_add(1, Ordering::Relaxed);
        state
            .metrics
            .gas_used_total
            .fetch_add(42_000, Ordering::Relaxed);

        let response = metrics(AxumState(state)).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should be readable");
        let text = String::from_utf8(body.to_vec()).expect("metrics should be utf-8");
        assert!(text.contains("otter_price_updates_total 3"));
        assert!(text.contains("otter_executions_total 1"));
        assert!(text.contains("otter_gas_used_total 42000"));
        assert!(text.contains("otter_active_intents 0"));
        assert!(text.contains("otter_proof_witness_seconds 0"));
        assert!(text.contains("otter_proof_verification_errors_total 0"));
        assert!(text.contains("otter_rpc_errors_total 0"));
        assert!(text.contains("otter_vault_balance 0"));
    }

    #[tokio::test]
    async fn metrics_endpoint_disabled_returns_not_found() {
        let state = test_state().await;
        // Arc is immutable; recreate with metrics disabled.
        let disabled_state = Arc::new(AppState {
            orchestrator: state.orchestrator.clone(),
            storage: state.storage.clone(),
            bus: state.bus.clone(),
            metrics: state.metrics.clone(),
            execution_enabled: state.execution_enabled,
            metrics_enabled: false,
            version: state.version,
            auth_enabled: state.auth_enabled,
            auth_service: state.auth_service.clone(),
            rate_limit_per_minute: state.rate_limit_per_minute,
            request_counts: state.request_counts.clone(),
            cors_allowed_origins: state.cors_allowed_origins.clone(),
            event_tx: state.event_tx.clone(),
            agents: state.agents.clone(),
            agent_pubkey: state.agent_pubkey.clone(),
            multichain: Arc::new(infrastructure::blockchain::MultiChainAdapter::empty()),
            network_health: Arc::new(Mutex::new(HashMap::new())),
            mev: None,
            rebate_bps: infrastructure::mev::DEFAULT_REBATE_BPS,
        });

        let response = metrics(AxumState(disabled_state)).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    fn test_addr() -> std::net::SocketAddr {
        std::net::SocketAddr::from(([127, 0, 0, 1], 1234))
    }

    fn with_connect_info(req: Request<Body>) -> Request<Body> {
        let (mut parts, body) = req.into_parts();
        parts.extensions.insert(ConnectInfo(test_addr()));
        Request::from_parts(parts, body)
    }

    #[tokio::test]
    async fn auth_disabled_allows_unauthenticated_requests() {
        let state = test_state().await;
        let router = app(state.clone());

        let req = with_connect_info(
            Request::builder()
                .method("GET")
                .uri("/api/v1/intents")
                .body(Body::empty())
                .unwrap(),
        );
        let response = router.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn health_endpoint_returns_status_and_version() {
        let state = test_state().await;
        let req = with_connect_info(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        );
        let response = app(state).oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "up");
        assert_eq!(json["version"], "0.0.0-test");
    }

    #[tokio::test]
    async fn auth_challenge_returns_siwe_message() {
        let state = auth_test_state().await;
        let req = with_connect_info(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/challenge")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"address":"0x0000000000000000000000000000000000000000"}"#,
                ))
                .unwrap(),
        );
        let response = app(state).oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let resp: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let message = resp["message"].as_str().unwrap();
        assert!(message.contains("Sign in to Otter agent"));
        assert!(message.contains("Nonce:"));
    }

    #[tokio::test]
    async fn protected_endpoint_requires_valid_token_when_auth_enabled() {
        let state = auth_test_state().await;
        let router = app(state.clone());

        // No token -> 401.
        let no_auth_req = with_connect_info(
            Request::builder()
                .method("POST")
                .uri("/api/v1/intents")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"text":"buy 1 eth at 1000 usdc"}"#))
                .unwrap(),
        );
        let no_auth = router.clone().oneshot(no_auth_req).await.unwrap();
        assert_eq!(no_auth.status(), StatusCode::UNAUTHORIZED);

        // Valid token -> allowed through to handler (parse will fail with bad request, not 401).
        let token = generate_test_token("test-secret", "0x123");
        let with_auth_req = with_connect_info(
            Request::builder()
                .method("POST")
                .uri("/api/v1/intents")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::from(r#"{"text":"buy 1 eth at 1000 usdc"}"#))
                .unwrap(),
        );
        let with_auth = router.oneshot(with_auth_req).await.unwrap();
        assert_ne!(with_auth.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn rate_limit_blocks_excess_requests() {
        let state = rate_limit_test_state().await;
        let router = app(state);
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 1234));

        for i in 0..3 {
            let mut req = Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap();
            req.extensions_mut().insert(ConnectInfo(addr));
            let response = router.clone().oneshot(req).await.unwrap();
            if i < 2 {
                assert_eq!(
                    response.status(),
                    StatusCode::OK,
                    "request {} should pass",
                    i
                );
            } else {
                assert_eq!(
                    response.status(),
                    StatusCode::TOO_MANY_REQUESTS,
                    "request {} should be rate limited",
                    i
                );
            }
        }
    }

    #[tokio::test]
    async fn cors_allows_configured_origin() {
        let state = cors_test_state("https://app.otter.local").await;
        let req = with_connect_info(
            Request::builder()
                .method("OPTIONS")
                .uri("/health")
                .header(header::ORIGIN, "https://app.otter.local")
                .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                .body(Body::empty())
                .unwrap(),
        );
        let response = app(state).oneshot(req).await.unwrap();
        let allow_origin = response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .and_then(|v: &header::HeaderValue| v.to_str().ok())
            .unwrap_or_default();
        assert_eq!(allow_origin, "https://app.otter.local");
    }

    #[tokio::test]
    async fn cors_blocks_unconfigured_origin() {
        let state = cors_test_state("https://app.otter.local").await;
        let req = with_connect_info(
            Request::builder()
                .method("OPTIONS")
                .uri("/health")
                .header(header::ORIGIN, "https://evil.example")
                .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                .body(Body::empty())
                .unwrap(),
        );
        let response = app(state).oneshot(req).await.unwrap();
        assert!(
            response
                .headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .is_none()
        );
    }

    async fn auth_test_state() -> Arc<AppState> {
        let base = test_state().await;
        Arc::new(AppState {
            auth_enabled: true,
            auth_service: Some(Arc::new(AuthService::new("test-secret".to_string(), 24))),
            ..clone_state(&base)
        })
    }

    async fn rate_limit_test_state() -> Arc<AppState> {
        let base = test_state().await;
        Arc::new(AppState {
            rate_limit_per_minute: 2,
            ..clone_state(&base)
        })
    }

    async fn cors_test_state(origins: &str) -> Arc<AppState> {
        let base = test_state().await;
        Arc::new(AppState {
            cors_allowed_origins: origins.to_string(),
            ..clone_state(&base)
        })
    }

    fn clone_state(state: &Arc<AppState>) -> AppState {
        AppState {
            multichain: Arc::new(infrastructure::blockchain::MultiChainAdapter::empty()),
            network_health: Arc::new(Mutex::new(HashMap::new())),
            mev: state.mev.clone(),
            rebate_bps: state.rebate_bps,
            orchestrator: state.orchestrator.clone(),
            storage: state.storage.clone(),
            bus: state.bus.clone(),
            metrics: state.metrics.clone(),
            execution_enabled: state.execution_enabled,
            metrics_enabled: state.metrics_enabled,
            version: state.version,
            auth_enabled: state.auth_enabled,
            auth_service: state.auth_service.clone(),
            rate_limit_per_minute: state.rate_limit_per_minute,
            request_counts: state.request_counts.clone(),
            cors_allowed_origins: state.cors_allowed_origins.clone(),
            event_tx: state.event_tx.clone(),
            agents: state.agents.clone(),
            agent_pubkey: state.agent_pubkey.clone(),
        }
    }

    fn generate_test_token(secret: &str, address: &str) -> String {
        use chrono::{Duration, Utc};
        use jsonwebtoken::{EncodingKey, Header, encode};

        #[derive(serde::Serialize)]
        struct Claims {
            sub: String,
            exp: usize,
            iat: usize,
        }

        let now = Utc::now();
        let exp = now + Duration::hours(24);
        let claims = Claims {
            sub: address.to_lowercase(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };
        encode(
            &Header::new(jsonwebtoken::Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }

    fn zero32() -> String {
        "0x0000000000000000000000000000000000000000000000000000000000000000".to_string()
    }

    fn sig64() -> Vec<String> {
        (0..64).map(|_| "0x00".to_string()).collect()
    }

    fn valid_set_delegation_request() -> SetDelegationRequest {
        SetDelegationRequest {
            pubkey_x: zero32(),
            pubkey_y: zero32(),
            allowed_intents: zero32(),
            max_amounts: (0..10).map(|_| zero32()).collect(),
            allowed_protocols: (0..5).map(|_| zero32()).collect(),
            expiry: zero32(),
            nonce: zero32(),
            target_contract: zero32(),
            signature: sig64(),
        }
    }

    #[tokio::test]
    async fn create_intent_rejects_long_text() {
        let state = test_state().await;
        let long_text = "lend ".to_string() + &"x".repeat(MAX_INTENT_TEXT_LEN);
        let req = CreateIntentRequest {
            text: long_text,
            network: None,
        };
        let response = create_intent(AxumState(state), Extension(None::<AuthUser>), Json(req))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn set_delegation_validates_hex_and_lengths() {
        let state = test_state().await;

        // Wrong max_amounts length.
        let mut bad = valid_set_delegation_request();
        bad.max_amounts = (0..9).map(|_| zero32()).collect();
        let response = set_delegation(
            AxumState(state.clone()),
            Extension(None::<AuthUser>),
            Json(bad),
        )
        .await
        .into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Bad hex length for pubkey_x.
        let mut bad = valid_set_delegation_request();
        bad.pubkey_x = "0x00".to_string();
        let response = set_delegation(
            AxumState(state.clone()),
            Extension(None::<AuthUser>),
            Json(bad),
        )
        .await
        .into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn delegations_scoped_to_authenticated_user() {
        let state = test_state().await;
        let user_a = AuthUser {
            address: "0xaaa".to_string(),
        };
        let user_b = AuthUser {
            address: "0xbbb".to_string(),
        };

        let _ = set_delegation(
            AxumState(state.clone()),
            Extension(Some(user_a.clone())),
            Json(valid_set_delegation_request()),
        )
        .await
        .unwrap();

        let for_a = list_delegations(AxumState(state.clone()), Extension(Some(user_a.clone())))
            .await
            .unwrap();
        assert_eq!(for_a.delegations.len(), 1);

        let for_b = list_delegations(AxumState(state.clone()), Extension(Some(user_b)))
            .await
            .unwrap();
        assert!(for_b.delegations.is_empty());
    }

    #[tokio::test]
    async fn intents_scoped_to_authenticated_user() {
        let state = test_state().await;
        let user_a = AuthUser {
            address: "0xaaa".to_string(),
        };
        let user_b = AuthUser {
            address: "0xbbb".to_string(),
        };

        let create = create_intent(
            AxumState(state.clone()),
            Extension(Some(user_a.clone())),
            Json(CreateIntentRequest {
                text: "lend 100 USDC on Aave".to_string(),
                network: None,
            }),
        )
        .await
        .unwrap();

        let for_a = list_intents(AxumState(state.clone()), Extension(Some(user_a.clone())))
            .await
            .unwrap();
        assert_eq!(for_a.intents.len(), 1);
        assert_eq!(for_a.intents[0].id, create.id);

        let for_b = list_intents(AxumState(state.clone()), Extension(Some(user_b.clone())))
            .await
            .unwrap();
        assert!(for_b.intents.is_empty());

        let detail = get_intent(
            AxumState(state.clone()),
            Extension(Some(user_b)),
            Path(create.id.clone()),
        )
        .await;
        assert_eq!(
            detail.unwrap_err().into_response().status(),
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn strategies_list_is_empty_on_fresh_storage() {
        let state = test_state().await;
        let response = list_strategies(AxumState(state)).await.unwrap();
        assert!(response.strategies.is_empty());
    }

    #[tokio::test]
    async fn create_strategy_persists_and_lists() {
        let state = test_state().await;
        let body = CreateStrategyRequest {
            title: "Test Strategy".to_string(),
            description: "A strategy for testing.".to_string(),
            raw_text: "lend 100 USDC on Aave if yield > 3%".to_string(),
            agent_id: "agent-1".to_string(),
            risk_profile: "Conservative".to_string(),
            visibility: None,
        };

        let created = create_strategy(
            AxumState(state.clone()),
            Extension(None::<AuthUser>),
            Json(body),
        )
        .await
        .unwrap();

        let listed = list_strategies(AxumState(state)).await.unwrap();
        assert_eq!(listed.strategies.len(), 1);
        assert_eq!(listed.strategies[0].id, created.id);
        assert_eq!(listed.strategies[0].agent_name, "Aave Ace");
    }

    #[tokio::test]
    async fn get_strategy_returns_created_strategy_detail() {
        let state = test_state().await;
        let body = CreateStrategyRequest {
            title: "Yield Hunt".to_string(),
            description: "Find yield.".to_string(),
            raw_text: "lend 100 USDC on Aave if yield > 3%".to_string(),
            agent_id: "agent-1".to_string(),
            risk_profile: "Conservative".to_string(),
            visibility: None,
        };

        let created = create_strategy(
            AxumState(state.clone()),
            Extension(None::<AuthUser>),
            Json(body),
        )
        .await
        .unwrap();

        let detail = get_strategy(AxumState(state), Path(created.id.clone()))
            .await
            .unwrap();
        assert_eq!(detail.title, "Yield Hunt");
        assert_eq!(detail.raw_text, "lend 100 USDC on Aave if yield > 3%");
        assert!(detail.intent.condition.is_some());
    }

    #[tokio::test]
    async fn get_strategy_returns_not_found_for_missing_id() {
        let state = test_state().await;
        let response = get_strategy(AxumState(state), Path("strategy-missing".to_string())).await;
        assert_eq!(
            response.unwrap_err().into_response().status(),
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn fork_strategy_handler_increments_copies() {
        let state = test_state().await;
        let body = CreateStrategyRequest {
            title: "Forkable Strategy".to_string(),
            description: "A strategy to fork.".to_string(),
            raw_text: "lend 100 USDC on Aave if yield > 3%".to_string(),
            agent_id: "agent-1".to_string(),
            risk_profile: "Conservative".to_string(),
            visibility: None,
        };

        let created = create_strategy(
            AxumState(state.clone()),
            Extension(None::<AuthUser>),
            Json(body),
        )
        .await
        .unwrap();

        let forked = fork_strategy(AxumState(state.clone()), Path(created.id.clone()))
            .await
            .unwrap();
        assert_eq!(forked.strategy_id, created.id);

        let listed = list_strategies(AxumState(state)).await.unwrap();
        assert_eq!(listed.strategies[0].copies, 1);
    }

    #[tokio::test]
    async fn create_strategy_requires_auth_when_enabled() {
        let state = auth_test_state().await;
        let router = app(state.clone());
        let req = with_connect_info(
            Request::builder()
                .method("POST")
                .uri("/api/v1/strategies")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"title":"T","description":"D","raw_text":"Lend 1 USDC on Aave","agent_id":"agent-1","risk_profile":"Conservative"}"#))
                .unwrap(),
        );
        let response = router.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn create_strategy_persists_and_returns_id() {
        let state = test_state().await;
        let router = app(state.clone());
        let req = with_connect_info(
            Request::builder()
                .method("POST")
                .uri("/api/v1/strategies")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"title":"T","description":"D","raw_text":"Lend 1 USDC on Aave","agent_id":"agent-1","risk_profile":"Conservative"}"#))
                .unwrap(),
        );
        let response = router.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json["id"].as_str().unwrap().starts_with("strategy-"));
    }

    #[tokio::test]
    async fn fork_strategy_increments_copies() {
        let state = test_state().await;
        let id = seed_strategy(&state).await;
        let router = app(state.clone());
        let req = with_connect_info(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/strategies/{}/fork", id))
                .body(Body::empty())
                .unwrap(),
        );
        let response = router.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let record = state.storage.get_strategy(&id).await.unwrap().unwrap();
        assert_eq!(record.copies, 1);
    }

    async fn seed_strategy(state: &Arc<AppState>) -> String {
        let record = domain::ports::storage_port::StrategyRecord {
            id: "strategy-test".to_string(),
            title: "Test".to_string(),
            description: "D".to_string(),
            raw_text: "Lend 1 USDC".to_string(),
            intent_json: serde_json::to_string(&domain::models::intent::ConditionalIntent {
                intent: domain::models::intent::Intent::Lend {
                    asset: domain::models::intent::Asset::Usdc,
                    amount: 1_000_000,
                    protocol: domain::models::intent::LendingType::Aave,
                },
                condition: None,
            })
            .unwrap(),
            creator_address: None,
            agent_id: "agent-1".to_string(),
            risk_profile: "Conservative".to_string(),
            copies: 0,
            visibility: "private".to_string(),
            fork_count: 0,
            total_volume: 0,
            apy: 0.0,
            created_at: 0,
            updated_at: 0,
        };
        state.storage.save_strategy(&record).await.unwrap();
        record.id
    }

    #[tokio::test]
    async fn create_strategy_rejects_invalid_risk_profile() {
        let state = test_state().await;
        let body = CreateStrategyRequest {
            title: "Bad Strategy".to_string(),
            description: "A strategy with bad risk profile.".to_string(),
            raw_text: "lend 100 USDC on Aave if yield > 3%".to_string(),
            agent_id: "agent-1".to_string(),
            risk_profile: "Wild".to_string(),
            visibility: None,
        };

        let response =
            create_strategy(AxumState(state), Extension(None::<AuthUser>), Json(body)).await;
        assert_eq!(
            response.unwrap_err().into_response().status(),
            StatusCode::BAD_REQUEST
        );
    }

    #[tokio::test]
    async fn parse_intent_rejects_long_text() {
        let state = test_state().await;
        let long_text = "lend ".to_string() + &"x".repeat(MAX_INTENT_TEXT_LEN);
        let response = parse_intent(AxumState(state), Json(ParseRequest { text: long_text }))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn parse_intent_rejects_empty_text() {
        let state = test_state().await;
        let response = parse_intent(
            AxumState(state),
            Json(ParseRequest {
                text: "   ".to_string(),
            }),
        )
        .await
        .into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    async fn rate_limit_auth_test_state() -> Arc<AppState> {
        let base = test_state().await;
        Arc::new(AppState {
            auth_enabled: true,
            auth_service: Some(Arc::new(AuthService::new("test-secret".to_string(), 24))),
            rate_limit_per_minute: 2,
            ..clone_state(&base)
        })
    }

    fn authed_get(uri: &str, token: Option<&str>) -> Request<Body> {
        let mut builder = Request::builder().method("GET").uri(uri);
        if let Some(token) = token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {}", token));
        }
        with_connect_info(builder.body(Body::empty()).unwrap())
    }

    #[tokio::test]
    async fn rate_limit_scoped_per_user() {
        let state = rate_limit_auth_test_state().await;
        let router = app(state);
        let token_a = generate_test_token("test-secret", "0xaaa");
        let token_b = generate_test_token("test-secret", "0xbbb");

        // User A exhausts its quota: 2 requests pass, the third is limited.
        for i in 0..3 {
            let response = router
                .clone()
                .oneshot(authed_get("/health", Some(&token_a)))
                .await
                .unwrap();
            if i < 2 {
                assert_eq!(
                    response.status(),
                    StatusCode::OK,
                    "request {} should pass",
                    i
                );
            } else {
                assert_eq!(
                    response.status(),
                    StatusCode::TOO_MANY_REQUESTS,
                    "request {} should be rate limited",
                    i
                );
            }
        }

        // User B, from the same IP, has its own quota and still passes.
        let response = router
            .clone()
            .oneshot(authed_get("/health", Some(&token_b)))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Unauthenticated requests keep their own per-IP bucket.
        let response = router
            .clone()
            .oneshot(authed_get("/health", None))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn jwt_secret_required_on_public_networks() {
        for network in ["mainnet", "sepolia"] {
            let config = Config {
                auth_enabled: true,
                network: Some(network.to_string()),
                ..Default::default()
            };
            assert!(
                resolve_jwt_secret(&config).is_err(),
                "network {} should require OTTER_JWT_SECRET",
                network
            );
        }
    }

    #[test]
    fn jwt_secret_random_allowed_locally() {
        // No network configured (local dev): a random dev secret is generated.
        let local = Config {
            auth_enabled: true,
            ..Default::default()
        };
        assert!(resolve_jwt_secret(&local).is_ok());

        let localhost = Config {
            auth_enabled: true,
            network: Some("localhost".to_string()),
            ..Default::default()
        };
        assert!(resolve_jwt_secret(&localhost).is_ok());

        // An explicit secret is always honored, even on a public network.
        let explicit = Config {
            auth_enabled: true,
            network: Some("mainnet".to_string()),
            jwt_secret: "fixed-secret".to_string(),
            ..Default::default()
        };
        assert_eq!(resolve_jwt_secret(&explicit).unwrap(), "fixed-secret");
    }

    #[test]
    fn build_intent_parser_falls_back_to_regex_without_model() {
        let config = Config {
            model_path: "models/definitely-missing.gguf".to_string(),
            ..Default::default()
        };
        let parser = build_intent_parser(&config);
        let intent = parser.parse("lend 100 USDC on Aave");
        assert!(intent.is_ok(), "regex fallback should parse: {:?}", intent);
    }

    #[tokio::test]
    async fn siwe_end_to_end_real_signature() {
        use k256::ecdsa::{RecoveryId, Signature, SigningKey};
        use sha3::{Digest, Keccak256};

        // Well-known Anvil test account #0 private key.
        let private_key =
            hex::decode("ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
                .unwrap();
        let signing_key = SigningKey::from_slice(&private_key).unwrap();
        let encoded = signing_key.verifying_key().to_encoded_point(false);
        let digest = Keccak256::digest(&encoded.as_bytes()[1..]);
        let address_bytes: [u8; 20] = digest[12..].try_into().unwrap();
        // The SIWE parser requires an EIP-55 checksummed address.
        let address = siwe::eip55(&address_bytes);

        let state = auth_test_state().await;
        let router = app(state.clone());

        // 1. Request a SIWE challenge for the signer's address.
        let req = with_connect_info(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/challenge")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(format!(r#"{{"address":"{}"}}"#, address)))
                .unwrap(),
        );
        let response = router.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let resp: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let message = resp["message"].as_str().unwrap().to_string();

        // 2. Sign the challenge with EIP-191 personal_sign.
        let prefixed = format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message);
        let prehash = Keccak256::digest(prefixed.as_bytes());
        let (signature, recovery_id): (Signature, RecoveryId) =
            signing_key.sign_prehash_recoverable(&prehash).unwrap();
        let mut signature_bytes = signature.to_bytes().to_vec();
        signature_bytes.push(recovery_id.to_byte() + 27);
        let signature_hex = format!("0x{}", hex::encode(signature_bytes));

        // 3. Exchange the signed challenge for a JWT.
        let verify_body = serde_json::json!({ "message": message, "signature": signature_hex });
        let req = with_connect_info(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/verify")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(verify_body.to_string()))
                .unwrap(),
        );
        let response = router.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let resp: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let token = resp["token"].as_str().unwrap().to_string();

        // 4. The JWT subject must be the signer's address (lowercased).
        let user = state
            .auth_service
            .as_ref()
            .unwrap()
            .validate_token(&token)
            .unwrap();
        assert_eq!(user.address, address.to_lowercase());

        // 5. A protected endpoint accepts the issued JWT.
        let req = with_connect_info(
            Request::builder()
                .method("GET")
                .uri("/api/v1/intents")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        );
        let response = router.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn demo_endpoints_set_x_demo_data_header_and_flag() {
        let state = test_state().await;
        let router = app(state);

        for uri in [
            "/api/v1/agents",
            "/api/v1/agents/agent-1",
            "/api/v1/strategies",
            "/api/v1/proofs",
            "/api/v1/leaderboard",
        ] {
            let req = with_connect_info(
                Request::builder()
                    .method("GET")
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            );
            let response = router.clone().oneshot(req).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK, "{uri}");
            assert_eq!(
                response
                    .headers()
                    .get("x-demo-data")
                    .and_then(|v| v.to_str().ok()),
                Some("true"),
                "{uri} must carry the X-Demo-Data header"
            );
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(
                json["demo"],
                serde_json::json!(true),
                "{uri} must be flagged as demo in the payload"
            );
        }
    }

    #[tokio::test]
    async fn non_demo_endpoints_do_not_set_x_demo_data_header() {
        let state = test_state().await;
        let router = app(state);

        for uri in ["/health", "/api/v1/intents", "/api/v1/portfolio"] {
            let req = with_connect_info(
                Request::builder()
                    .method("GET")
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            );
            let response = router.clone().oneshot(req).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK, "{uri}");
            assert!(
                response.headers().get("x-demo-data").is_none(),
                "{uri} must not carry the X-Demo-Data header"
            );
        }
    }
}
