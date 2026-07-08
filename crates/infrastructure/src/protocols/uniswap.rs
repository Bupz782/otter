use alloy::primitives::{Address, U256};
use alloy::providers::ProviderBuilder;
use alloy::sol;
use alloy::sol_types::SolCall;
use domain::models::intent::Asset;
use domain::models::transaction::Transaction;
use domain::protocols::{DexProtocol, ProtocolError};

use super::aave::{Network, address_for};

sol! {
    #[sol(rpc)]
    interface IQuoterV2 {
        struct QuoteExactInputSingleParams {
            address tokenIn;
            address tokenOut;
            uint24 fee;
            uint256 amountIn;
            uint160 sqrtPriceLimitX96;
        }

        function quoteExactInputSingle(QuoteExactInputSingleParams memory params)
            external
            view
            returns (uint256 amountOut, uint160 sqrtPriceX96After, uint32 initializedTicksCrossed, uint256 gasEstimate);
    }

    #[sol(rpc)]
    interface ISwapRouter {
        struct ExactInputSingleParams {
            address tokenIn;
            address tokenOut;
            uint24 fee;
            address recipient;
            uint256 deadline;
            uint256 amountIn;
            uint256 amountOutMinimum;
            uint160 sqrtPriceLimitX96;
        }

        function exactInputSingle(ExactInputSingleParams calldata params) external payable returns (uint256 amountOut);
    }
}

/// Default Uniswap V3 fee tier: 0.30 %.
const DEFAULT_FEE_TIER: u32 = 3000;

/// Dummy non-zero address used for callers that only need read-only Uniswap
/// methods (e.g. `get_quote`).
pub const DUMMY_RECIPIENT: &str = "0x0000000000000000000000000000000000000001";

/// Uniswap V3 protocol adapter.
///
/// Uses QuoterV2 to estimate outputs and encodes real SwapRouter
/// `exactInputSingle` calldata.
#[derive(Debug, Clone)]
pub struct UniswapAdapter {
    rpc_url: String,
    router_address: Address,
    quoter_address: Address,
    /// Address receiving the swapped output. Must be an explicit, non-zero
    /// address supplied by the caller.
    recipient: Address,
}

impl UniswapAdapter {
    /// Create an adapter targeting the given SwapRouter and QuoterV2.
    ///
    /// `recipient` is required and must not be the zero address. Use
    /// [`DUMMY_RECIPIENT`] for read-only operations where the address is not
    /// used.
    pub fn new(
        rpc_url: impl Into<String>,
        router_address: impl Into<String>,
        quoter_address: impl Into<String>,
        recipient: impl Into<String>,
    ) -> Result<Self, ProtocolError> {
        let router_address = router_address
            .into()
            .parse()
            .map_err(|e| ProtocolError::OperationFailed(format!("invalid router address: {e}")))?;
        let quoter_address = quoter_address
            .into()
            .parse()
            .map_err(|e| ProtocolError::OperationFailed(format!("invalid quoter address: {e}")))?;
        let recipient: Address = recipient.into().parse().map_err(|e| {
            ProtocolError::OperationFailed(format!("invalid recipient address: {e}"))
        })?;
        if recipient.is_zero() {
            return Err(ProtocolError::OperationFailed(
                "recipient must not be the zero address".to_string(),
            ));
        }
        Ok(Self {
            rpc_url: rpc_url.into(),
            router_address,
            quoter_address,
            recipient,
        })
    }

    /// Adapter pointing at the official Uniswap V3 mainnet contracts.
    pub fn mainnet(
        rpc_url: impl Into<String>,
        recipient: impl Into<String>,
    ) -> Result<Self, ProtocolError> {
        Self::new(
            rpc_url,
            "0xE592427A0AEce92De3Edee1F18E0157C05861564",
            "0x61fFE014bA17989E743c5F6cB21bF96909c97d5b",
            recipient,
        )
    }

    /// Adapter pointing at the official Uniswap V3 Sepolia contracts.
    pub fn sepolia(
        rpc_url: impl Into<String>,
        recipient: impl Into<String>,
    ) -> Result<Self, ProtocolError> {
        Self::new(
            rpc_url,
            "0x3bFA4769FB09eefC5a80d6E87c3B9C650f7Ae48E",
            "0xEd1f64733475F1b43a963611B4E0A6A50e39c40d",
            recipient,
        )
    }

    fn network(&self) -> Network {
        if self.router_address.to_string().to_lowercase()
            == "0x3bfa4769fb09eefc5a80d6e87c3b9c650f7ae48e"
        {
            Network::Sepolia
        } else {
            Network::Mainnet
        }
    }

    fn token_pair(
        from: &Asset,
        to: &Asset,
        network: Network,
    ) -> Result<(Address, Address), ProtocolError> {
        if from == to {
            return Err(ProtocolError::InvalidAmount(
                "cannot swap an asset for itself".to_string(),
            ));
        }
        let from_addr = address_for(from, network)?;
        let to_addr = address_for(to, network)?;
        Ok((from_addr, to_addr))
    }

    /// Build the `exactInputSingle` calldata without fetching a quote.
    fn build_exact_input_single(
        &self,
        from: &Asset,
        to: &Asset,
        amount: u128,
        amount_out_min: u128,
    ) -> Result<ISwapRouter::exactInputSingleCall, ProtocolError> {
        let (token_in, token_out) = Self::token_pair(from, to, self.network())?;
        Ok(ISwapRouter::exactInputSingleCall {
            params: ISwapRouter::ExactInputSingleParams {
                tokenIn: token_in,
                tokenOut: token_out,
                fee: DEFAULT_FEE_TIER,
                recipient: self.recipient,
                deadline: U256::from(u64::MAX),
                amountIn: U256::from(amount),
                amountOutMinimum: U256::from(amount_out_min),
                sqrtPriceLimitX96: U256::ZERO,
            },
        })
    }
}

impl DexProtocol for UniswapAdapter {
    fn get_quote(&self, from: &Asset, to: &Asset, amount: u128) -> Result<u128, ProtocolError> {
        if amount == 0 {
            return Err(ProtocolError::InvalidAmount(
                "amount must be > 0".to_string(),
            ));
        }
        let (token_in, token_out) = Self::token_pair(from, to, self.network())?;

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| ProtocolError::OperationFailed(format!("tokio runtime: {e}")))?;
        let amount_out = rt.block_on(async {
            let url = self
                .rpc_url
                .parse()
                .map_err(|e| ProtocolError::OperationFailed(format!("invalid rpc url: {e}")))?;
            let provider = ProviderBuilder::new().on_http(url);
            let quoter = IQuoterV2::new(self.quoter_address, provider);
            let params = IQuoterV2::QuoteExactInputSingleParams {
                tokenIn: token_in,
                tokenOut: token_out,
                fee: DEFAULT_FEE_TIER,
                amountIn: U256::from(amount),
                sqrtPriceLimitX96: U256::ZERO,
            };
            let result = quoter
                .quoteExactInputSingle(params)
                .call()
                .await
                .map_err(|e| {
                    ProtocolError::OperationFailed(format!("quoteExactInputSingle failed: {e}"))
                })?;
            Ok::<_, ProtocolError>(result.amountOut)
        })?;

        amount_out
            .try_into()
            .map_err(|_| ProtocolError::InvalidAmount("quote overflow".to_string()))
    }

    fn swap(
        &self,
        from: &Asset,
        to: &Asset,
        amount: u128,
        slippage_bps: u16,
    ) -> Result<Transaction, ProtocolError> {
        if amount == 0 {
            return Err(ProtocolError::InvalidAmount(
                "amount must be > 0".to_string(),
            ));
        }
        if slippage_bps > 10_000 {
            return Err(ProtocolError::InvalidAmount(
                "slippage must be <= 10000 bps".to_string(),
            ));
        }
        // Use the quoted output minus slippage as amountOutMinimum.
        let amount_out = self.get_quote(from, to, amount)?;
        let amount_out_min = amount_out
            .checked_mul((10_000 - slippage_bps) as u128)
            .and_then(|v| v.checked_div(10_000))
            .ok_or_else(|| {
                ProtocolError::InvalidAmount("slippage calculation overflow".to_string())
            })?;

        let call = self.build_exact_input_single(from, to, amount, amount_out_min)?;
        Ok(
            Transaction::new(self.router_address.to_string(), 0, 300_000)
                .with_data(call.abi_encode()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_USER: &str = "0x1111111111111111111111111111111111111111";

    #[test]
    fn swap_encodes_valid_calldata() {
        let uniswap = UniswapAdapter::sepolia("http://localhost:8545", TEST_USER).unwrap();
        let call = uniswap
            .build_exact_input_single(&Asset::Eth, &Asset::Usdc, 1_000_000_000_000_000_000, 0)
            .unwrap();
        let tx = Transaction::new(uniswap.router_address.to_string(), 0, 300_000)
            .with_data(call.abi_encode());
        assert_eq!(
            tx.to.to_lowercase(),
            uniswap.router_address.to_string().to_lowercase()
        );
        assert!(!tx.data.is_empty());
        assert_eq!(&tx.data[0..4], ISwapRouter::exactInputSingleCall::SELECTOR);
    }

    #[test]
    fn get_quote_rejects_same_asset() {
        let uniswap = UniswapAdapter::sepolia("http://localhost:8545", TEST_USER).unwrap();
        assert!(uniswap.get_quote(&Asset::Eth, &Asset::Eth, 1).is_err());
    }

    #[test]
    fn swap_rejects_excessive_slippage() {
        let uniswap = UniswapAdapter::sepolia("http://localhost:8545", TEST_USER).unwrap();
        assert!(uniswap.swap(&Asset::Eth, &Asset::Usdc, 1, 20_000).is_err());
    }

    #[test]
    fn rejects_zero_recipient() {
        assert!(
            UniswapAdapter::sepolia(
                "http://localhost:8545",
                "0x0000000000000000000000000000000000000000",
            )
            .is_err()
        );
    }
}
