use application::events::{Event, EventBus};
use application::orchestrator::{Orchestrator, OrchestratorError};
use application::use_cases::execute_intent::{ExecuteIntentUseCase, ExecutionError};
use domain::models::condition::{Comparator, Condition, Metric};
use domain::models::delegation::{
    DelegationMessage, DelegationProof, field_from_u32, field_from_u64, field_from_u128,
    hash_delegation,
};
use domain::models::execution_plan::{ExecutionPlan, ExecutionStep};
use domain::models::intent::{Asset, ConditionalIntent, DexType, Intent, LendingType, Protocol};
use domain::ports::evm_port::EvmPort;
use domain::ports::price_oracle_port::PriceOraclePort;
use domain::ports::wallet_port::WalletPort;
use domain::ports::zkp_port::ZkpPort;
use infrastructure::blockchain::{
    AlloyEvmAdapter, CompositeOracle, LocalWalletAdapter, MockEvmAdapter, MockOracleAdapter,
    OracleNetwork,
};
use infrastructure::parsers::RegexParser;
use infrastructure::zkp::{MockZkpAdapter, NoirAdapter};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn print_usage() {
    eprintln!(
        "Usage: otter_cli <command> [options]

Commands:
  parse <intent>         Parse a natural-language intent and print the structured result.
  plan <intent>          Parse an intent and build an execution plan.
  start <intent>         Start the orchestrator daemon and monitor an intent.
  status                 Print the current daemon state and active intents.
  execute <intent>       Run the full pipeline: parse → condition → prove → submit on-chain.
  prove <intent>         Parse and prove an intent, writing proof.bin and public_inputs.bin.
  verify-onchain         Verify a proof on-chain against the vault's verifier contract (view call).

parse / plan options:
  (none)

start options:
  --network <sepolia|mainnet>  Chainlink/Aave network to monitor. Default: sepolia
  --interval <seconds>         Price/yield update interval. Default: 5

execute — Offline / mock mode (no --vault):
  Uses a mock ZKP and a mock EVM. Useful for quickly testing parsing and conditions.

execute — On-chain mode (requires --vault, --private-key, --rpc-url):
  Generates a real Noir UltraHonk proof and submits executeWithProof to the vault.

execute options:
  --rpc-url <url>        EVM RPC endpoint. Default: http://localhost:8545
  --private-key <hex>    Hex private key used to sign the delegation and txs.
                         Also read from OTTER_PRIVATE_KEY.
  --vault <address>      DelegationVault contract address.
  --delegate             Also register the delegation on-chain before executing.
  --price <value>        Default price for conditional intents (USD, 6 decimals). Default: 3_000_000_000
  --amount <value>       Max amount allowed per intent type. Default: 10_000_000_000
  --timestamp <value>    Current timestamp. Default: now (unix seconds)
  --circuit-dir <path>   Path to the Noir circuit. Default: ./delegation_circuit
  --bb-bin <path>        Barretenberg bb binary. Default: ~/.bb/bb

prove options:
  --private-key <hex>    Hex private key used to sign the delegation.
                         Also read from OTTER_PRIVATE_KEY.
  --output-dir <path>    Directory to write proof.bin and public_inputs.bin. Default: .
  --amount <value>       Max amount allowed per intent type. Default: 10_000_000_000
  --timestamp <value>    Current timestamp. Default: now (unix seconds)
  --circuit-dir <path>   Path to the Noir circuit. Default: ./delegation_circuit
  --bb-bin <path>        Barretenberg bb binary. Default: ~/.bb/bb

verify-onchain options:
  --proof <path>         Path to proof.bin.
  --public-inputs <path> Path to public_inputs.bin.
  --rpc-url <url>        EVM RPC endpoint. Default: http://localhost:8545
  --vault <address>      DelegationVault contract address.
  --private-key <hex>    Hex private key (not used for the view call, required to build the adapter).
                         Also read from OTTER_PRIVATE_KEY.

Examples:
  otter_cli parse \"lend 1000 USDC on Aave\"
  otter_cli plan \"swap 1 ETH for USDC on Uniswap\"

  # Mock execution
  otter_cli execute \"swap 1000 USDC for ETH on Uniswap\"

  # Local Anvil end-to-end (start anvil first, then deploy the vault)
  otter_cli execute \"swap 1 ETH for USDC on Uniswap if price > 2_000_000_000\" \\
      --rpc-url http://localhost:8545 \\
      --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \\
      --vault 0x... \\
      --delegate
"
    );
}

fn parse_u128(s: &str) -> Result<u128, String> {
    let cleaned = s.replace(['_', ','], "");
    cleaned
        .parse::<u128>()
        .map_err(|e| format!("cannot parse integer '{}': {}", s, e))
}

fn default_bb_bin() -> String {
    std::env::var("HOME")
        .map(|home| format!("{}/.bb/bb", home))
        .unwrap_or_else(|_| "bb".to_string())
}

fn default_circuit_dir() -> PathBuf {
    std::env::var("OTTER_CIRCUIT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("delegation_circuit"))
}

fn asset_symbol(asset: &Asset) -> &'static str {
    match asset {
        Asset::Eth => "ETH",
        Asset::Dai => "DAI",
        Asset::Usdc => "USDC",
        Asset::Wbtc => "WBTC",
        Asset::Link => "LINK",
        Asset::Sol => "SOL",
    }
}

fn dex_name(protocol: &DexType) -> &'static str {
    match protocol {
        DexType::Uniswap => "Uniswap",
        DexType::Sushiswap => "Sushiswap",
        DexType::Balancer => "Balancer",
    }
}

fn lending_name(protocol: &LendingType) -> &'static str {
    match protocol {
        LendingType::Aave => "Aave",
        LendingType::Compound => "Compound",
    }
}

fn protocol_name(protocol: &Protocol) -> String {
    match protocol {
        Protocol::Dex(dex) => dex_name(dex).to_string(),
        Protocol::Lending(lending) => lending_name(lending).to_string(),
    }
}

fn format_amount(asset: &Asset, amount: u128) -> String {
    format!("{} {}", asset.format_amount(amount), asset_symbol(asset))
}

fn format_condition(condition: &Condition) -> String {
    match condition {
        Condition::Comparison {
            metric,
            comparator,
            value,
        } => {
            let metric_name = match metric {
                Metric::Price => "price",
                Metric::Yield => "yield",
                Metric::GasCost => "gas cost",
                Metric::Volume => "volume",
            };
            let cmp = match comparator {
                Comparator::GreaterThan => ">",
                Comparator::LessThan => "<",
                Comparator::EqualTo => "=",
                Comparator::GreaterThanOrEqualTo => ">=",
                Comparator::LessThanOrEqualTo => "<=",
            };
            format!("{} {} {}", metric_name, cmp, value)
        }
    }
}

fn format_intent(conditional: &ConditionalIntent) -> String {
    let intent_str = match &conditional.intent {
        Intent::Lend {
            asset,
            amount,
            protocol,
        } => format!(
            "Lend {} on {}",
            format_amount(asset, *amount),
            lending_name(protocol)
        ),
        Intent::Swap {
            from_asset,
            to_asset,
            amount,
            protocol,
        } => format!(
            "Swap {} for {} on {}",
            format_amount(from_asset, *amount),
            asset_symbol(to_asset),
            dex_name(protocol)
        ),
        Intent::Borrow {
            asset,
            amount,
            collateral,
            collateral_amount,
            protocol,
        } => format!(
            "Borrow {} with {} collateral on {}",
            format_amount(asset, *amount),
            format_amount(collateral, *collateral_amount),
            lending_name(protocol)
        ),
        Intent::Stake {
            asset,
            amount,
            protocol,
        } => format!(
            "Stake {} on {}",
            format_amount(asset, *amount),
            lending_name(protocol)
        ),
        Intent::Composite { intents } => {
            let parts: Vec<String> = intents
                .iter()
                .map(|i| {
                    format_intent(&ConditionalIntent {
                        intent: i.clone(),
                        condition: None,
                        network: None,
                    })
                })
                .collect();
            format!("Composite: {}", parts.join(" then "))
        }
    };

    let condition_str = conditional
        .condition
        .as_ref()
        .map(format_condition)
        .unwrap_or_else(|| "none".to_string());

    format!(
        "Parsed intent:\n  Action: {}\n  Condition: {}",
        intent_str, condition_str
    )
}

fn format_step(index: usize, step: &ExecutionStep) -> String {
    match step {
        ExecutionStep::Approve {
            asset,
            spender,
            amount,
        } => format!(
            "{}. Approve {} for {}",
            index,
            format_amount(asset, *amount),
            spender
        ),
        ExecutionStep::Supply {
            asset,
            amount,
            protocol,
        } => format!(
            "{}. Supply {} on {}",
            index,
            format_amount(asset, *amount),
            protocol_name(protocol)
        ),
        ExecutionStep::Borrow {
            asset,
            amount,
            protocol,
        } => format!(
            "{}. Borrow {} on {}",
            index,
            format_amount(asset, *amount),
            protocol_name(protocol)
        ),
        ExecutionStep::Repay {
            asset,
            amount,
            protocol,
        } => format!(
            "{}. Repay {} on {}",
            index,
            format_amount(asset, *amount),
            protocol_name(protocol)
        ),
        ExecutionStep::Withdraw {
            asset,
            amount,
            protocol,
        } => format!(
            "{}. Withdraw {} from {}",
            index,
            format_amount(asset, *amount),
            protocol_name(protocol)
        ),
        ExecutionStep::SwapExactTokens {
            from_asset,
            to_asset,
            amount_in,
            min_amount_out,
            protocol,
        } => format!(
            "{}. Swap {} for at least {} on {}",
            index,
            format_amount(from_asset, *amount_in),
            format_amount(to_asset, *min_amount_out),
            protocol_name(protocol)
        ),
        ExecutionStep::Stake {
            asset,
            amount,
            protocol,
        } => format!(
            "{}. Stake {} on {}",
            index,
            format_amount(asset, *amount),
            protocol_name(protocol)
        ),
        ExecutionStep::Unstake {
            asset,
            amount,
            protocol,
        } => format!(
            "{}. Unstake {} from {}",
            index,
            format_amount(asset, *amount),
            protocol_name(protocol)
        ),
    }
}

fn format_plan(plan: &ExecutionPlan) -> String {
    let mut lines = vec![format!(
        "Execution plan: {} ({} steps{})",
        plan.description(),
        plan.step_count(),
        plan.gas_estimation()
            .map(|g| format!(", ~{} gas", g))
            .unwrap_or_default()
    )];
    for (i, step) in plan.steps().iter().enumerate() {
        lines.push(format_step(i + 1, step));
    }
    lines.join("\n")
}

fn run_parse(intent_text: &str) {
    let parser = RegexParser::new();
    let oracle = MockOracleAdapter::new();
    let zkp = MockZkpAdapter::new();
    let evm = MockEvmAdapter::new();
    let mut orchestrator = Orchestrator::new(parser, oracle, zkp, evm);
    match orchestrator.parse(intent_text) {
        Ok(conditional) => println!("{}", format_intent(&conditional)),
        Err(
            OrchestratorError::ParseFailed(msg)
            | OrchestratorError::PlanFailed(msg)
            | OrchestratorError::InvalidIntent(msg),
        ) => {
            eprintln!("Failed to parse intent: {}", msg);
            std::process::exit(1);
        }
    }
}

fn run_plan(intent_text: &str) {
    let parser = RegexParser::new();
    let oracle = MockOracleAdapter::new();
    let zkp = MockZkpAdapter::new();
    let evm = MockEvmAdapter::new();
    let mut orchestrator = Orchestrator::new(parser, oracle, zkp, evm);
    match orchestrator.plan(intent_text) {
        Ok(plan) => println!("{}", format_plan(&plan)),
        Err(
            OrchestratorError::ParseFailed(msg)
            | OrchestratorError::PlanFailed(msg)
            | OrchestratorError::InvalidIntent(msg),
        ) => {
            eprintln!("Failed to plan execution: {}", msg);
            std::process::exit(1);
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DaemonStateSnapshot {
    state: String,
    active_intents: Vec<String>,
    updated_at: u64,
}

fn state_file_path() -> PathBuf {
    std::env::var("OTTER_STATE_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp/otter-daemon-state.json"))
}

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn write_state_file(
    path: &Path,
    state: &application::orchestrator::State,
    intents: &[application::orchestrator::ActiveIntent],
) {
    let snapshot = DaemonStateSnapshot {
        state: state.to_string(),
        active_intents: intents
            .iter()
            .map(|i| format!("{:?}", i.conditional.intent))
            .collect(),
        updated_at: now_unix_secs(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&snapshot) {
        let _ = std::fs::write(path, json);
    }
}

fn read_state_file(path: &Path) -> Option<DaemonStateSnapshot> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

fn run_start(intent_text: &str, option_args: &[String]) {
    let mut interval_secs: u64 = 5;
    let mut rpc_url: String = "http://localhost:8545".to_string();
    let mut network = OracleNetwork::Sepolia;
    let mut private_key: Option<String> = std::env::var("OTTER_PRIVATE_KEY").ok();
    let mut vault_address: Option<String> = None;
    let mut auto_delegate = false;
    let mut max_amount: u128 = 10_000_000_000;
    let mut timestamp: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let mut circuit_dir = default_circuit_dir();
    let mut bb_bin = default_bb_bin();

    let mut i = 0;
    while i < option_args.len() {
        match option_args[i].as_str() {
            "--interval" => {
                i += 1;
                interval_secs =
                    parse_u128(option_args.get(i).expect("missing value for --interval"))
                        .unwrap()
                        .try_into()
                        .expect("interval too large");
            }
            "--network" => {
                i += 1;
                let value = option_args
                    .get(i)
                    .expect("missing value for --network")
                    .as_str();
                network = match value {
                    "sepolia" => OracleNetwork::Sepolia,
                    "mainnet" => OracleNetwork::Mainnet,
                    other => {
                        eprintln!("Unknown network: {}", other);
                        print_usage();
                        std::process::exit(1);
                    }
                };
            }
            "--rpc-url" => {
                i += 1;
                rpc_url = option_args
                    .get(i)
                    .expect("missing value for --rpc-url")
                    .clone();
            }
            "--private-key" => {
                i += 1;
                private_key = Some(
                    option_args
                        .get(i)
                        .expect("missing value for --private-key")
                        .clone(),
                );
            }
            "--vault" => {
                i += 1;
                vault_address = Some(
                    option_args
                        .get(i)
                        .expect("missing value for --vault")
                        .clone(),
                );
            }
            "--delegate" => {
                auto_delegate = true;
            }
            "--amount" => {
                i += 1;
                max_amount =
                    parse_u128(option_args.get(i).expect("missing value for --amount")).unwrap();
            }
            "--timestamp" => {
                i += 1;
                timestamp = parse_u128(option_args.get(i).expect("missing value for --timestamp"))
                    .unwrap()
                    .try_into()
                    .expect("timestamp too large");
            }
            "--circuit-dir" => {
                i += 1;
                circuit_dir =
                    PathBuf::from(option_args.get(i).expect("missing value for --circuit-dir"));
            }
            "--bb-bin" => {
                i += 1;
                bb_bin = option_args
                    .get(i)
                    .expect("missing value for --bb-bin")
                    .clone();
            }
            other => {
                eprintln!("Unknown option: {}", other);
                print_usage();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let parser = RegexParser::new();
    let oracle = CompositeOracle::new(&rpc_url, network).unwrap_or_else(|e| {
        eprintln!("Failed to create composite oracle: {}", e);
        std::process::exit(1);
    });

    let on_chain_mode = vault_address.is_some() && private_key.is_some();

    if on_chain_mode {
        let private_key = private_key.unwrap();
        let vault_address = vault_address.unwrap();

        if !Path::new(&bb_bin).exists() {
            eprintln!("Barretenberg binary not found at {}.", bb_bin);
            std::process::exit(1);
        }

        let wallet = LocalWalletAdapter::from_hex(&private_key).unwrap_or_else(|e| {
            eprintln!("Failed to load wallet: {}", e);
            std::process::exit(1);
        });

        let delegation = build_delegation(&wallet, max_amount).unwrap_or_else(|e| {
            eprintln!("Failed to build delegation: {}", e);
            std::process::exit(1);
        });

        let signature = wallet
            .sign_hash(&hash_delegation(&delegation))
            .unwrap_or_else(|e| {
                eprintln!("Failed to sign delegation: {}", e);
                std::process::exit(1);
            });

        let zkp = NoirAdapter::new(circuit_dir, "nargo", Some(bb_bin));
        let evm = AlloyEvmAdapter::new(rpc_url, &private_key, &vault_address).unwrap_or_else(|e| {
            eprintln!("Failed to create EVM adapter: {}", e);
            std::process::exit(1);
        });

        if auto_delegate {
            println!("Registering delegation on-chain...");
            match evm.ensure_delegated(&delegation) {
                Ok(tx_hash) => println!("Delegation registered: {}", tx_hash),
                Err(err) => {
                    eprintln!("Failed to register delegation: {:?}", err);
                    std::process::exit(1);
                }
            }
        }

        let mut orchestrator = Orchestrator::new(parser, oracle, zkp, evm);
        orchestrator.set_delegation(delegation, signature);
        orchestrator.set_timestamp(timestamp);

        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        rt.block_on(run_monitoring_loop(
            orchestrator,
            intent_text,
            interval_secs,
        ));
    } else {
        let zkp = MockZkpAdapter::new();
        let evm = MockEvmAdapter::new();
        let mut orchestrator = Orchestrator::new(parser, oracle, zkp, evm);
        orchestrator.set_delegation(
            DelegationMessage {
                pubkey_x: [0u8; 32],
                pubkey_y: [0u8; 32],
                allowed_intents: field_from_u32(0x0f),
                max_amounts: [field_from_u128(max_amount); 10],
                allowed_protocols: [
                    field_from_u32(1),
                    field_from_u32(2),
                    field_from_u32(4),
                    field_from_u32(0),
                    field_from_u32(0),
                ],
                expiry: field_from_u64(4_000_000_000),
                nonce: field_from_u64(42),
                target_contract: field_from_u32(0),
            },
            [0u8; 64],
        );
        orchestrator.set_timestamp(timestamp);

        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        rt.block_on(run_monitoring_loop(
            orchestrator,
            intent_text,
            interval_secs,
        ));
    }
}

async fn run_monitoring_loop<O, Z, E>(
    mut orchestrator: Orchestrator<RegexParser, O, Z, E>,
    intent_text: &str,
    interval_secs: u64,
) where
    O: PriceOraclePort + Clone + Send + 'static,
    Z: ZkpPort + Clone + Send + 'static,
    E: EvmPort + Clone + Send + 'static,
{
    let state_path = state_file_path();

    let intent_id = orchestrator.submit_intent(intent_text).unwrap_or_else(|e| {
        eprintln!("Failed to submit intent: {}", e);
        std::process::exit(1);
    });
    write_state_file(
        &state_path,
        orchestrator.state(),
        orchestrator.active_intents(),
    );

    let (bus, mut receiver) = EventBus::new(64);
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_secs));
    let mut tick = 0;
    let mut running = true;

    while running {
        tokio::select! {
            Some(event) = receiver.recv() => {
                println!("[daemon] event: {:?}", event);
                let is_confirmed = matches!(event, Event::TransactionConfirmed { .. });
                orchestrator.process_event(event, &bus).await;
                write_state_file(
                    &state_path,
                    orchestrator.state(),
                    orchestrator.active_intents(),
                );
                if is_confirmed {
                    println!("[daemon] execution flow complete");
                    running = false;
                }
            }
            _ = interval.tick() => {
                tick += 1;
                println!("[daemon] tick #{}", tick);
                for intent in orchestrator.active_intents() {
                    let Some(condition) = &intent.conditional.condition else { continue; };
                    let asset = Orchestrator::<RegexParser, O, Z, E>::primary_asset_of(&intent.conditional.intent);
                    let metric = condition.metric();
                    match orchestrator.fetch_metric_async(metric, &asset).await {
                        Ok(value) => {
                            println!("[daemon] {} {:?} = {}", asset_symbol(&asset), metric, value);
                            let _ = bus.publish(Event::PriceUpdated {
                                asset: asset.clone(),
                                metric: *metric,
                                value,
                            });
                        }
                        Err(err) => {
                            eprintln!("[daemon] failed to fetch {:?} for {:?}: {}", metric, asset, err);
                        }
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("\n[daemon] shutting down on interrupt");
                running = false;
            }
        }
    }

    write_state_file(
        &state_path,
        orchestrator.state(),
        orchestrator.active_intents(),
    );
    println!("[daemon] intent {} monitoring stopped", intent_id);
}

fn run_status() {
    let path = state_file_path();
    match read_state_file(&path) {
        Some(snapshot) => {
            println!("Daemon state: {}", snapshot.state);
            println!("Active intents: {}", snapshot.active_intents.len());
            for (i, intent) in snapshot.active_intents.iter().enumerate() {
                println!("  {}. {}", i + 1, intent);
            }
            println!(
                "Last update: {} ({}s ago)",
                snapshot.updated_at,
                now_unix_secs() - snapshot.updated_at
            );
        }
        None => {
            println!(
                "No daemon state found at {}. Run `otter start` first.",
                path.display()
            );
        }
    }
}

fn build_delegation(
    wallet: &LocalWalletAdapter,
    max_amount: u128,
) -> Result<DelegationMessage, String> {
    let (pubkey_x, pubkey_y) = wallet.pubkey().map_err(|e| e.to_string())?;
    Ok(DelegationMessage {
        pubkey_x,
        pubkey_y,
        allowed_intents: field_from_u32(0x0f),
        max_amounts: [field_from_u128(max_amount); 10],
        allowed_protocols: [
            field_from_u32(1), // Uniswap
            field_from_u32(2), // Sushiswap
            field_from_u32(4), // Aave
            field_from_u32(0),
            field_from_u32(0),
        ],
        expiry: field_from_u64(4_000_000_000),
        nonce: field_from_u64(42),
        target_contract: field_from_u32(0),
    })
}

fn run_execute(intent_text: &str, option_args: &[String]) {
    let mut rpc_url: String = "http://localhost:8545".to_string();
    let mut private_key: Option<String> = std::env::var("OTTER_PRIVATE_KEY").ok();
    let mut vault_address: Option<String> = None;
    let mut auto_delegate = false;
    let mut price: u128 = 3_000_000_000;
    let mut max_amount: u128 = 10_000_000_000;
    let mut timestamp: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let mut circuit_dir = default_circuit_dir();
    let mut bb_bin = default_bb_bin();

    let mut i = 0;
    while i < option_args.len() {
        match option_args[i].as_str() {
            "--rpc-url" => {
                i += 1;
                rpc_url = option_args
                    .get(i)
                    .expect("missing value for --rpc-url")
                    .clone();
            }
            "--private-key" => {
                i += 1;
                private_key = Some(
                    option_args
                        .get(i)
                        .expect("missing value for --private-key")
                        .clone(),
                );
            }
            "--vault" => {
                i += 1;
                vault_address = Some(
                    option_args
                        .get(i)
                        .expect("missing value for --vault")
                        .clone(),
                );
            }
            "--delegate" => {
                auto_delegate = true;
            }
            "--price" => {
                i += 1;
                price = parse_u128(option_args.get(i).expect("missing value for --price")).unwrap();
            }
            "--amount" => {
                i += 1;
                max_amount =
                    parse_u128(option_args.get(i).expect("missing value for --amount")).unwrap();
            }
            "--timestamp" => {
                i += 1;
                timestamp = parse_u128(option_args.get(i).expect("missing value for --timestamp"))
                    .unwrap()
                    .try_into()
                    .expect("timestamp too large");
            }
            "--circuit-dir" => {
                i += 1;
                circuit_dir =
                    PathBuf::from(option_args.get(i).expect("missing value for --circuit-dir"));
            }
            "--bb-bin" => {
                i += 1;
                bb_bin = option_args
                    .get(i)
                    .expect("missing value for --bb-bin")
                    .clone();
            }
            other => {
                eprintln!("Unknown option: {}", other);
                print_usage();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let parser = RegexParser::new();
    let mut oracle = MockOracleAdapter::new();
    oracle.set(Metric::Price, None, price);

    let on_chain_mode = vault_address.is_some() && private_key.is_some();

    if on_chain_mode {
        let private_key = private_key.unwrap();
        let vault_address = vault_address.unwrap();

        let wallet = LocalWalletAdapter::from_hex(&private_key).unwrap_or_else(|e| {
            eprintln!("Failed to load wallet: {}", e);
            std::process::exit(1);
        });

        let delegation = build_delegation(&wallet, max_amount).unwrap_or_else(|e| {
            eprintln!("Failed to build delegation: {}", e);
            std::process::exit(1);
        });

        let delegation_hash = hash_delegation(&delegation);
        let signature = wallet.sign_hash(&delegation_hash).unwrap_or_else(|e| {
            eprintln!("Failed to sign delegation: {}", e);
            std::process::exit(1);
        });

        if !Path::new(&bb_bin).exists() {
            eprintln!(
                "Barretenberg binary not found at {}. Set --bb-bin or install bb.",
                bb_bin
            );
            std::process::exit(1);
        }

        let zkp = NoirAdapter::new(circuit_dir, "nargo", Some(bb_bin));
        let evm = AlloyEvmAdapter::new(rpc_url, &private_key, &vault_address).unwrap_or_else(|e| {
            eprintln!("Failed to create EVM adapter: {}", e);
            std::process::exit(1);
        });

        if auto_delegate {
            println!("Registering delegation on-chain...");
            match evm.ensure_delegated(&delegation) {
                Ok(tx_hash) => println!("Delegation registered: {}", tx_hash),
                Err(err) => {
                    eprintln!("Failed to register delegation: {:?}", err);
                    std::process::exit(1);
                }
            }
        }

        let use_case = ExecuteIntentUseCase::new(parser, oracle, zkp, evm, 11155111);
        match use_case.execute(intent_text, &delegation, &signature, timestamp) {
            Ok(tx_hash) => {
                println!("Intent executed successfully");
                println!("Transaction hash: {}", tx_hash);
            }
            Err(ExecutionError::ConditionNotMet) => {
                eprintln!("Condition not met; execution skipped.");
                std::process::exit(2);
            }
            Err(err) => {
                eprintln!("Execution failed: {:?}", err);
                std::process::exit(1);
            }
        }
    } else {
        // Offline mock mode.
        let delegation = DelegationMessage {
            pubkey_x: [0u8; 32],
            pubkey_y: [0u8; 32],
            allowed_intents: field_from_u32(0x0f),
            max_amounts: [field_from_u128(max_amount); 10],
            allowed_protocols: [
                field_from_u32(1),
                field_from_u32(2),
                field_from_u32(4),
                field_from_u32(0),
                field_from_u32(0),
            ],
            expiry: field_from_u64(4_000_000_000),
            nonce: field_from_u64(42),
            target_contract: field_from_u32(0),
        };
        let signature = [0u8; 64];

        let zkp = MockZkpAdapter::new();
        let evm = MockEvmAdapter::new();

        let use_case = ExecuteIntentUseCase::new(parser, oracle, zkp, evm, 11155111);
        match use_case.execute(intent_text, &delegation, &signature, timestamp) {
            Ok(tx_hash) => {
                println!("Intent parsed and validated (mock execution)");
                println!("Mock transaction hash: {}", tx_hash);
            }
            Err(ExecutionError::ConditionNotMet) => {
                eprintln!("Condition not met; execution skipped.");
                std::process::exit(2);
            }
            Err(err) => {
                eprintln!("Execution failed: {:?}", err);
                std::process::exit(1);
            }
        }
    }
}

fn run_prove(intent_text: &str, option_args: &[String]) {
    let mut private_key: Option<String> = std::env::var("OTTER_PRIVATE_KEY").ok();
    let mut output_dir = PathBuf::from(".");
    let mut timestamp: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let mut max_amount: u128 = 10_000_000_000;
    let mut circuit_dir = default_circuit_dir();
    let mut bb_bin = default_bb_bin();

    let mut i = 0;
    while i < option_args.len() {
        match option_args[i].as_str() {
            "--private-key" => {
                i += 1;
                private_key = Some(
                    option_args
                        .get(i)
                        .expect("missing value for --private-key")
                        .clone(),
                );
            }
            "--output-dir" => {
                i += 1;
                output_dir =
                    PathBuf::from(option_args.get(i).expect("missing value for --output-dir"));
            }
            "--amount" => {
                i += 1;
                max_amount =
                    parse_u128(option_args.get(i).expect("missing value for --amount")).unwrap();
            }
            "--timestamp" => {
                i += 1;
                timestamp = parse_u128(option_args.get(i).expect("missing value for --timestamp"))
                    .unwrap()
                    .try_into()
                    .expect("timestamp too large");
            }
            "--circuit-dir" => {
                i += 1;
                circuit_dir =
                    PathBuf::from(option_args.get(i).expect("missing value for --circuit-dir"));
            }
            "--bb-bin" => {
                i += 1;
                bb_bin = option_args
                    .get(i)
                    .expect("missing value for --bb-bin")
                    .clone();
            }
            other => {
                eprintln!("Unknown option: {}", other);
                print_usage();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let private_key = private_key.unwrap_or_else(|| {
        eprintln!("--private-key or OTTER_PRIVATE_KEY is required for prove");
        std::process::exit(1);
    });

    if !Path::new(&bb_bin).exists() {
        eprintln!("Barretenberg binary not found at {}.", bb_bin);
        std::process::exit(1);
    }

    let wallet = LocalWalletAdapter::from_hex(&private_key).unwrap_or_else(|e| {
        eprintln!("Failed to load wallet: {}", e);
        std::process::exit(1);
    });

    let delegation = build_delegation(&wallet, max_amount).unwrap_or_else(|e| {
        eprintln!("Failed to build delegation: {}", e);
        std::process::exit(1);
    });

    let signature = wallet
        .sign_hash(&hash_delegation(&delegation))
        .unwrap_or_else(|e| {
            eprintln!("Failed to sign delegation: {}", e);
            std::process::exit(1);
        });

    let parser = RegexParser::new();
    let oracle = MockOracleAdapter::new();
    let zkp = NoirAdapter::new(circuit_dir, "nargo", Some(bb_bin));
    let evm = MockEvmAdapter::new();

    let use_case = ExecuteIntentUseCase::new(parser, oracle, zkp, evm, 11155111);
    let (proof, public_inputs) = use_case
        .prove(intent_text, &delegation, &signature, timestamp)
        .unwrap_or_else(|e| {
            eprintln!("Proof generation failed: {:?}", e);
            std::process::exit(1);
        });

    std::fs::create_dir_all(&output_dir).unwrap_or_else(|e| {
        eprintln!("Failed to create output directory: {}", e);
        std::process::exit(1);
    });

    let proof_path = output_dir.join("proof.bin");
    let public_inputs_path = output_dir.join("public_inputs.bin");
    std::fs::write(&proof_path, &proof.proof).unwrap_or_else(|e| {
        eprintln!("Failed to write proof: {}", e);
        std::process::exit(1);
    });
    std::fs::write(&public_inputs_path, &proof.public_inputs).unwrap_or_else(|e| {
        eprintln!("Failed to write public inputs: {}", e);
        std::process::exit(1);
    });

    println!("Generated delegation proof");
    println!(
        "  delegation_hash: 0x{}",
        hex::encode(hash_delegation(&delegation))
    );
    println!("  proof bytes: {}", proof.proof.len());
    println!("  public inputs bytes: {}", proof.public_inputs.len());
    println!("  wrote: {}", proof_path.display());
    println!("  wrote: {}", public_inputs_path.display());

    // Keep public_inputs variable used if needed later; suppress unused warning.
    let _ = public_inputs;
}

fn run_verify_onchain(option_args: &[String]) {
    let mut proof_path = PathBuf::from("proof.bin");
    let mut public_inputs_path = PathBuf::from("public_inputs.bin");
    let mut rpc_url: String = "http://localhost:8545".to_string();
    let mut vault_address: Option<String> = None;
    let mut private_key: Option<String> = std::env::var("OTTER_PRIVATE_KEY").ok();

    let mut i = 0;
    while i < option_args.len() {
        match option_args[i].as_str() {
            "--proof" => {
                i += 1;
                proof_path = PathBuf::from(option_args.get(i).expect("missing value for --proof"));
            }
            "--public-inputs" => {
                i += 1;
                public_inputs_path = PathBuf::from(
                    option_args
                        .get(i)
                        .expect("missing value for --public-inputs"),
                );
            }
            "--rpc-url" => {
                i += 1;
                rpc_url = option_args
                    .get(i)
                    .expect("missing value for --rpc-url")
                    .clone();
            }
            "--vault" => {
                i += 1;
                vault_address = Some(
                    option_args
                        .get(i)
                        .expect("missing value for --vault")
                        .clone(),
                );
            }
            "--private-key" => {
                i += 1;
                private_key = Some(
                    option_args
                        .get(i)
                        .expect("missing value for --private-key")
                        .clone(),
                );
            }
            other => {
                eprintln!("Unknown option: {}", other);
                print_usage();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let vault_address = vault_address.unwrap_or_else(|| {
        eprintln!("--vault is required for verify-onchain");
        std::process::exit(1);
    });
    let private_key = private_key.unwrap_or_else(|| {
        eprintln!("--private-key or OTTER_PRIVATE_KEY is required for verify-onchain");
        std::process::exit(1);
    });

    let proof = std::fs::read(&proof_path).unwrap_or_else(|e| {
        eprintln!("Failed to read proof file: {}", e);
        std::process::exit(1);
    });
    let public_inputs = std::fs::read(&public_inputs_path).unwrap_or_else(|e| {
        eprintln!("Failed to read public inputs file: {}", e);
        std::process::exit(1);
    });

    let evm = AlloyEvmAdapter::new(rpc_url, &private_key, &vault_address).unwrap_or_else(|e| {
        eprintln!("Failed to create EVM adapter: {}", e);
        std::process::exit(1);
    });

    let delegation_proof = DelegationProof {
        proof,
        public_inputs,
    };

    match evm.verify_onchain(&delegation_proof) {
        Ok(true) => println!("Proof is valid on-chain."),
        Ok(false) => {
            eprintln!("Proof is invalid on-chain.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("On-chain verification failed: {:?}", e);
            std::process::exit(1);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let command = args[1].as_str();
    match command {
        "parse" | "plan" | "start" => {
            if args.len() < 3 {
                print_usage();
                std::process::exit(1);
            }
            let intent_text = &args[2];
            match command {
                "parse" => run_parse(intent_text),
                "plan" => run_plan(intent_text),
                "start" => run_start(intent_text, &args[3..]),
                _ => unreachable!(),
            }
        }
        "execute" => {
            if args.len() < 3 {
                print_usage();
                std::process::exit(1);
            }
            run_execute(&args[2], &args[3..]);
        }
        "prove" => {
            if args.len() < 3 {
                print_usage();
                std::process::exit(1);
            }
            run_prove(&args[2], &args[3..]);
        }
        "verify-onchain" => run_verify_onchain(&args[2..]),
        "status" => run_status(),
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            std::process::exit(1);
        }
    }
}
