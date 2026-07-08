use infrastructure::llm::{LlmConfig, LocalLlmClient};
use std::env;
use std::path::PathBuf;

fn main() {
    println!("🚀 Testing LLM loading...\n");

    let model_path = get_model_path();
    println!("📁 Using model: {}\n", model_path.display());

    if !model_path.exists() {
        eprintln!("❌ Error: Model file not found at {:?}", model_path);
        eprintln!("💡 Hint: Run the download script: ./scripts/download-model.sh");
        eprintln!("   Or set otter_MODEL_PATH environment variable to your model file");
        std::process::exit(1);
    }

    let system_prompt = "You are a DeFi Intelligent Intent Parser. Your role is to translate user natural language into a JSON format that maps exactly to the following Rust structures:

### 1. Schema Constraints
- **Assets (Strict List):** Eth, Dai, Usdc, Wbtc, Link, Sol (Must be PascalCase).
- **Protocols:**
    - DexType (for Swap): Uniswap, Sushiswap, Balancer
    - LendingType (for Lend, Borrow, Stake): Aave, Compound
- **Enums:** All Metric and Comparator values must be PascalCase (e.g., GreaterThan, LessThan, EqualTo, LessThanOrEqualTo, GreaterThanOrEqualTo, Yield, Price, GasCost, Volume).

### 2. Math & Precision (u128)
Convert all human-readable amounts to raw u128 integers:
- Eth/Dai/Link: 18 decimals
- Usdc: 6 decimals
- Wbtc: 8 decimals
- Sol: 9 decimals
- Price Metrics: Default to 6 decimals (e.g., 2500 -> 2500000000).

### 3. Output Format
Return ONLY a JSON object following this EXACT structure:
{
  \"intent\": {
    \"[Action]\": {
      \"from_asset\": \"Asset\",
      \"to_asset\": \"Asset\",
      \"amount\": \"u128\",
      \"protocol\": \"DexType\"
    }
  },
  \"condition\": {
    \"Comparison\": {
      \"metric\": \"Metric\",
      \"comparator\": \"Comparator\",
      \"value\": \"u128\"
    }
  } | null
}
Note: [Action] must be one of: Swap, Lend, Borrow, Stake.

### 4. Logic
- Default protocol for Swap: \"Uniswap\". Default for others: \"Aave\".

### 5. Examples

User input: \"swap 1 ETH for USDC\"
Assistant:
{
  \"intent\": {
    \"Swap\": {
      \"from_asset\": \"Eth\",
      \"to_asset\": \"Usdc\",
      \"amount\": \"1000000000000000000\",
      \"protocol\": \"Uniswap\"
    }
  },
  \"condition\": null
}

User input: \"lend 5000 DAI on Aave if yield > 4%\"
Assistant:
{
  \"intent\": {
    \"Lend\": {
      \"asset\": \"Dai\",
      \"amount\": \"5000000000000000000000\",
      \"protocol\": \"Aave\"
    }
  },
  \"condition\": {
    \"Comparison\": {
      \"metric\": \"Yield\",
      \"comparator\": \"GreaterThan\",
      \"value\": \"4000000\"
    }
  }
}
User input: \"lend 5000 SOL on pancake if yield > 4%\"
assisatan: We do not have the capacity for sol & pancake right now
";

    let config = LlmConfig::new()
        .with_n_ctx(4096)
        .with_temperature(0.2)
        .with_seed(42);

    let mut client =
        LocalLlmClient::with_config(model_path.to_str().unwrap(), system_prompt, config);

    match client.load() {
        Ok(_) => println!("✅ Model loaded successfully"),
        Err(e) => {
            eprintln!("❌ Failed to load model: {}", e);
            std::process::exit(1);
        }
    }

    let prompt = "i want to swap 3 sol for usdt if the price is 2500 or above on pancakeswap";
    println!("\n📝 Prompt: {}", prompt);
    println!("\n🤔 Generating response...\n");

    match client.generate(prompt, 100) {
        Ok(response) => {
            println!("\n📤 Response: {:?}", response);
        }
        Err(e) => {
            eprintln!("❌ Generation failed: {}", e);
            std::process::exit(1);
        }
    }

    client.unload();
    println!("\n✅ Test completed successfully!");
}

fn get_model_path() -> PathBuf {
    if let Ok(path) = env::var("otter_MODEL_PATH") {
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
