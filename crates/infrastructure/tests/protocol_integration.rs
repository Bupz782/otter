use domain::models::intent::Asset;
use domain::{DexProtocol, LendingProtocol};
use infrastructure::protocols::{AaveAdapter, UniswapAdapter};

/// Integration tests against a live Sepolia RPC.
///
/// Set `OTTER_TEST_RPC_URL` to a Sepolia endpoint to run them, e.g.:
/// `OTTER_TEST_RPC_URL=https://rpc.sepolia.org cargo test -p infrastructure --test protocol_integration`
fn rpc_url() -> Option<String> {
    std::env::var("OTTER_TEST_RPC_URL").ok()
}

#[test]
fn aave_get_apy_on_sepolia() {
    let Some(url) = rpc_url() else {
        eprintln!("skipping integration test: OTTER_TEST_RPC_URL not set");
        return;
    };

    let aave = AaveAdapter::sepolia(&url, "0x1111111111111111111111111111111111111111")
        .expect("valid sepolia adapter");
    let apy = aave.get_apy(&Asset::Usdc).expect("fetch USDC APY");
    assert!(apy > 0.0, "APY should be positive");
    assert!(apy < 50.0, "APY should be reasonable");
}

#[test]
fn uniswap_get_quote_on_sepolia() {
    let Some(url) = rpc_url() else {
        eprintln!("skipping integration test: OTTER_TEST_RPC_URL not set");
        return;
    };

    let uniswap = UniswapAdapter::sepolia(&url, "0x1111111111111111111111111111111111111111")
        .expect("valid sepolia adapter");
    // 0.001 ETH
    let amount = 1_000_000_000_000_000u128;
    let quote = uniswap
        .get_quote(&Asset::Eth, &Asset::Usdc, amount)
        .expect("quote ETH->USDC");
    assert!(quote > 0, "quote should be positive");
}
