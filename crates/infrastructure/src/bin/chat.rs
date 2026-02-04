use infrastructure::llm::{LlmConfig, LocalLlmClient};
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;

fn main() {
    // Init logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .init();

    println!("🚀 Loading LLM...\n");

    let model_path = get_model_path();
    println!("📁 Model: {}\n", model_path.display());

    if !model_path.exists() {
        eprintln!("❌ Model not found: {:?}", model_path);
        eprintln!("💡 Set METIS_MODEL_PATH or run ./scripts/download-model.sh");
        std::process::exit(1);
    }

    let system_prompt = r#"You are a DeFi Intent Parser. Convert user requests to JSON ONLY.

STRICT ENUMS (ONLY these values allowed):
- Assets: Eth, Dai, Usdc, Wbtc, Link, Sol
- DexType (for Swap): Uniswap, Sushiswap, Balancer
- LendingType (for Lend/Borrow/Stake): Aave, Compound
- Metric: Yield, Price, GasCost, Volume
- Comparator: GreaterThan, LessThan, EqualTo, LessThanOrEqualTo, GreaterThanOrEqualTo

AMOUNTS: Convert to raw u128 with decimals (Eth/Dai/Link=18, Usdc=6, Wbtc=8, Sol=9)

SUCCESS OUTPUT:
{
  "intent": {
    "Swap": { "from_asset": "Asset", "to_asset": "Asset", "amount": u128, "protocol": "DexType" }
    OR "Lend"|"Borrow"|"Stake": { "asset": "Asset", "amount": u128, "protocol": "LendingType" }
  },
  "condition": { "Comparison": { "metric": "Metric", "comparator": "Comparator", "value": u128 } } | null
}

ERROR OUTPUT (when request is invalid):
{ "error": "reason" }

STRICT RULES:
- NEVER invent or guess missing amounts. If no amount specified → return error
- NEVER substitute unknown assets (BTC, MATIC, USDT, etc.) → return error
- NEVER substitute unknown protocols (Pancakeswap, Maker, etc.) → return error
- "%" or "yield" → metric: "Yield" (basis points: 3% = 300)
- "price" → metric: "Price" (6 decimals: $2500 = 2500000000)
- Default protocol: Uniswap for Swap, Aave for Lend/Borrow/Stake
- Output raw JSON only, no markdown, no explanation"#;

    let config = LlmConfig::new()
        .with_n_ctx(4096)
        .with_temperature(0.2)
        .with_seed(42);

    let mut client =
        LocalLlmClient::with_config(model_path.to_str().unwrap(), system_prompt, config);

    match client.load() {
        Ok(_) => println!("✅ Model loaded!\n"),
        Err(e) => {
            eprintln!("❌ Failed to load: {}", e);
            std::process::exit(1);
        }
    }

    println!("💬 Type your prompts (Ctrl+C to quit)\n");
    println!("{}", "-".repeat(50));

    loop {
        print!("\n> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }

        let prompt = input.trim();
        if prompt.is_empty() {
            continue;
        }

        if prompt == "quit" || prompt == "exit" {
            break;
        }

        if prompt == "clear" {
            client.clear_cache();
            println!("🗑️  Cache cleared");
            continue;
        }

        println!("\n🤔 Generating...\n");

        match client.generate_text(prompt, 500) {
            Ok(raw_response) => {
                println!("📤 Output:\n{}", raw_response);

                // Strip <think>...</think> block if present
                let cleaned = if let Some(idx) = raw_response.find("</think>") {
                    raw_response[idx + 8..].trim()
                } else {
                    raw_response.trim()
                };

                // Try to parse as JSON
                match serde_json::from_str::<serde_json::Value>(cleaned) {
                    Ok(json) => {
                        println!(
                            "\n✅ Valid JSON:\n{}",
                            serde_json::to_string_pretty(&json).unwrap()
                        );
                    }
                    Err(e) => {
                        println!("\n⚠️  Not valid JSON: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Error: {}", e);
            }
        }

        println!("\n{}", "-".repeat(50));
    }

    client.unload();
    println!("\n👋 Bye!");
}

fn get_model_path() -> PathBuf {
    if let Ok(path) = env::var("METIS_MODEL_PATH") {
        return PathBuf::from(path);
    }

    let mut path = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    loop {
        let cargo_toml = path.join("Cargo.toml");
        if cargo_toml.exists()
            && let Ok(content) = std::fs::read_to_string(&cargo_toml)
            && content.contains("[workspace]")
        {
            return path.join("models").join("Qwen3-8B-Q4_K_M.gguf");
        }

        if !path.pop() {
            break;
        }
    }

    PathBuf::from("models/Qwen3-8B-Q4_K_M.gguf")
}
