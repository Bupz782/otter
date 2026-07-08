# Infrastructure Crate Changes

## Summary of Code Improvements

### 1. Regex Parser Optimization (`src/parsers/regex_parser.rs`)

#### Before
- Regex patterns were compiled on every function call
- Code duplication for asset/protocol parsing across 4+ functions
- No trait abstraction for parsers

#### After
- **Static compiled regex** using `std::sync::OnceLock`:
  ```rust
  static LEND_REGEX: OnceLock<Regex> = OnceLock::new();
  static SWAP_REGEX: OnceLock<Regex> = OnceLock::new();
  // etc.
  ```
  Patterns are compiled once on first use, then reused.

- **Trait-based architecture**:
  ```rust
  pub trait IntentParser {
      fn parse(&self, text: &str) -> Result<Intent, ParseError>;
      fn description(&self) -> &'static str;
  }
  ```
  Individual parsers: `LendParser`, `SwapParser`, `BorrowParser`, `StakeParser`

- **Shared helper functions** to eliminate duplication:
  - `parse_asset()` - Parse asset strings to `Asset` enum
  - `parse_dex_protocol()` - Parse DEX protocol strings
  - `parse_lending_protocol()` - Parse lending protocol strings
  - `parse_amount()` - Parse amount with proper decimal handling
  - `parse_comparator()` - Parse comparison operators
  - `parse_metric()` - Parse metric types

- **Backward compatibility**: Original `RegexParser::parse_lend()`, etc. functions still work

---

### 2. LLM Configuration (`src/llm/config.rs`)

#### New File: Configuration struct for LLM parameters

```rust
pub struct LlmConfig {
    pub n_ctx: u32,           // Context window size (default: 2048)
    pub batch_size: usize,    // Batch size for tokens (default: 1024)
    pub temperature: f32,     // Sampling temperature (default: 0.7)
    pub top_p: f32,          // Top-p (nucleus) sampling (default: 0.9)
    pub top_k: i32,          // Top-k sampling (default: 40)
    pub repeat_penalty: f32, // Repetition penalty (default: 1.1)
    pub seed: Option<u32>,   // Random seed for reproducibility
    pub threads: Option<usize>, // Number of threads
    pub gpu_layers: u32,     // GPU layers to offload
}
```

#### Features
- **Builder pattern** for easy configuration:
  ```rust
  let config = LlmConfig::new()
      .with_n_ctx(4096)
      .with_temperature(0.5)
      .with_seed(42);
  ```

- **Input validation**: Values are clamped to valid ranges
  - `temperature`: 0.0 to 2.0
  - `top_p`: 0.0 to 1.0
  - `top_k`: minimum 1
  - `repeat_penalty`: minimum 1.0

- **Presets**:
  - `LlmConfig::deterministic()` - For reproducible results (temp=0, seed=42)
  - `LlmConfig::creative()` - For varied outputs (temp=0.9, top_p=0.95)

---

### 3. Local LLM Client Improvements (`src/llm/local_client.rs` + `src/llm/error.rs`)

#### Split into Multiple Files
- `src/llm/error.rs` - `LlmError` enum with `thiserror`
- `src/llm/local_client.rs` - `LocalLlmClient` implementation
- `src/llm/config.rs` - `LlmConfig` (new)

#### Error Handling
- **Before**: Used `expect()` and `unwrap()`, panicking on errors
- **After**: Proper `Result` types with `LlmError` enum:
  ```rust
  pub enum LlmError {
      ModelNotLoaded,
      BackendNotLoaded,
      BackendInit(String),
      ModelLoad(String),
      ContextCreation(String),
      Tokenization(String),
      Generation(String),
      InvalidConfig(String),
  }
  ```

#### Configuration Support
- **Before**: Hardcoded values (n_ctx=2048, batch_size=1024)
- **After**: Uses `LlmConfig` with constructor options:
  ```rust
  // With defaults
  let client = LocalLlmClient::new(model_path, system_prompt);
  
  // With custom config
  let client = LocalLlmClient::with_config(model_path, system_prompt, config);
  
  // Update config later
  client.set_config(new_config);
  ```

#### New Methods
- `is_loaded()` - Check if model is loaded
- `unload()` - Free memory by dropping model/backend
- `config()` - Get current configuration reference

---

### 4. Test Binary Improvements (`src/bin/test.rs`)

#### Before
- Hardcoded absolute path: `/Users/fr158286/Perso/...`
- Would fail on any other machine
- Used `panic!` for error handling

#### After
- **Environment variable support**: `otter_MODEL_PATH`
- **Auto-discovery**: Walks up directory tree to find workspace root
- **Relative path fallback**: `./models/Qwen3-8B-Q4_K_M.gguf`
- **File existence check** with helpful error messages
- **Proper error handling** with exit codes
- **Demonstrates new config**:
  ```rust
  let config = LlmConfig::new()
      .with_n_ctx(4096)
      .with_temperature(0.2)
      .with_seed(42);
  ```

---

### 5. Dependencies

#### Added
- `thiserror = "2.0"` - For ergonomic error definitions

---

## File Structure Changes

```
src/llm/
├── mod.rs              # Updated exports
├── config.rs           # NEW: LlmConfig with builder pattern
├── error.rs            # NEW: LlmError enum (extracted from local_client.rs)
├── local_client.rs     # REFACTORED: Split error, added config support
├── cache.rs            # (empty - unchanged)
├── model_loader.rs     # (empty - unchanged)
├── prompt_builder.rs   # (empty - unchanged)
└── response_parser.rs  # (empty - unchanged)

src/parsers/
├── mod.rs              # Updated exports (added IntentParser trait)
├── error.rs            # (unchanged)
└── regex_parser.rs     # REFACTORED: OnceLock, traits, helpers
```

---

## Test Results

```
running 27 tests
test llm::config::tests::test_creative_preset ... ok
test llm::config::tests::test_builder_pattern ... ok
test llm::config::tests::test_default_config ... ok
test llm::config::tests::test_temperature_clamping ... ok
test llm::config::tests::test_deterministic_preset ... ok
test llm::config::tests::test_top_p_clamping ... ok
test llm::local_client::tests::test_client_with_config ... ok
test llm::local_client::tests::test_client_creation ... ok
test llm::local_client::tests::test_unload ... ok
test llm::local_client::tests::test_set_config ... ok
test parsers::regex_parser::tests::test_parser_descriptions ... ok
test parsers::regex_parser::tests::test_parse_* ... ok (17 tests)

test result: ok. 27 passed; 0 failed; 0 ignored
```

---

## Migration Guide

### For existing code using RegexParser

No changes needed - backward compatible:
```rust
// Still works
let intent = RegexParser::parse_lend("lend 100 USDC on Aave")?;
```

### For new trait-based usage
```rust
use infrastructure::parsers::{LendParser, IntentParser};

let parser = LendParser;
let intent = parser.parse("lend 100 USDC on Aave")?;
```

### For LLM client with config
```rust
use infrastructure::llm::{LocalLlmClient, LlmConfig};

let config = LlmConfig::new()
    .with_n_ctx(4096)
    .with_temperature(0.5);

let client = LocalLlmClient::with_config(model_path, prompt, config);
client.load()?;
let output = client.generate("Hello", 100)?;
```

### For error handling
```rust
use infrastructure::llm::LlmError;

match client.load() {
    Ok(_) => println!("Loaded"),
    Err(LlmError::ModelLoad(msg)) => eprintln!("Failed to load: {}", msg),
    Err(e) => eprintln!("Error: {}", e),
}
```
